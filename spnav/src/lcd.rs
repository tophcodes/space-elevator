use rusb::{Context, DeviceHandle, UsbContext};
use std::time::Duration;

const LCD_WIDTH: u32 = 640;
const LCD_HEIGHT: u32 = 150;
const USB_VENDOR_ID: u16 = 0x256f;
const USB_PRODUCT_ID: u16 = 0xc633;
const USB_TIMEOUT: Duration = Duration::from_secs(5);
const ENDPOINT_OUT: u8 = 0x01;
const LCD_HEADER_SIZE: usize = 0x200; // 512-byte header; deflate stream follows

// SpaceLCD effect constants (byte 0 of packet header)
#[repr(u8)]
pub enum ScrollMode {
    None = 0x11,        // EFFECT_CUT — static image
    Left = 0x16,        // EFFECT_SCROLL_LEFT
    Right = 0x17,       // EFFECT_SCROLL_RIGHT
    Up = 0x14,          // EFFECT_SCROLL_DOWN (device axis)
    Down = 0x15,        // EFFECT_SCROLL_UP (device axis)
}

pub struct Lcd {
    handle: DeviceHandle<Context>,
}

#[derive(Debug)]
pub enum LcdError {
    DeviceNotFound,
    UsbError(rusb::Error),
    CompressionError(std::io::Error),
    ImageTooLarge,
    InvalidDimensions,
}

impl std::fmt::Display for LcdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound => write!(f, "SpaceMouse Enterprise not found"),
            Self::UsbError(e) => write!(f, "USB error: {}", e),
            Self::CompressionError(e) => write!(f, "Compression error: {}", e),
            Self::ImageTooLarge => write!(f, "Compressed image exceeds 65535 bytes"),
            Self::InvalidDimensions => write!(f, "Image must be 640x150 pixels"),
        }
    }
}

impl From<std::io::Error> for LcdError {
    fn from(e: std::io::Error) -> Self {
        Self::CompressionError(e)
    }
}

impl From<rusb::Error> for LcdError {
    fn from(e: rusb::Error) -> Self {
        Self::UsbError(e)
    }
}

impl std::error::Error for LcdError {}

impl Lcd {
    pub fn new() -> Result<Self, LcdError> {
        let context = Context::new().map_err(LcdError::UsbError)?;

        // Locate the device first, then open it separately: opening can fail
        // for reasons other than absence (most commonly a permissions error on
        // the USB node), and folding open() into the search would report every
        // such failure as DeviceNotFound.
        let device = context
            .devices()
            .map_err(LcdError::UsbError)?
            .iter()
            .find(|dev| {
                dev.device_descriptor()
                    .map(|desc| {
                        desc.vendor_id() == USB_VENDOR_ID && desc.product_id() == USB_PRODUCT_ID
                    })
                    .unwrap_or(false)
            })
            .ok_or(LcdError::DeviceNotFound)?;
        let handle = device.open().map_err(LcdError::UsbError)?;

        // Detach kernel driver and claim interface 0 (vendor/LCD interface)
        if handle.kernel_driver_active(0).unwrap_or(false) {
            handle.detach_kernel_driver(0).map_err(LcdError::UsbError)?;
        }
        handle.claim_interface(0).map_err(LcdError::UsbError)?;

        Ok(Self { handle })
    }

    /// Display RGB888 bitmap (640×150) with optional scroll effect
    pub fn display_bitmap_with_scroll(
        &mut self,
        rgb_data: &[u8],
        scroll: ScrollMode,
    ) -> Result<(), LcdError> {
        let expected_size = (LCD_WIDTH * LCD_HEIGHT * 3) as usize;
        if rgb_data.len() != expected_size {
            return Err(LcdError::InvalidDimensions);
        }

        let rgb565 = Self::rgb888_to_rgb565(rgb_data);
        let compressed = Self::compress_bitmap(&rgb565)?;

        if compressed.len() > 65535 {
            return Err(LcdError::ImageTooLarge);
        }

        // Device expects a 512-byte header (4 meaningful bytes + zero padding);
        // the deflate stream starts at offset 512. See SpaceLCD SPLCD_HEADER_SIZE=0x200.
        let mut packet = vec![0u8; LCD_HEADER_SIZE + compressed.len()];
        packet[0] = scroll as u8;
        packet[1] = 0x0F;
        packet[2] = (compressed.len() & 0xFF) as u8;
        packet[3] = (compressed.len() >> 8) as u8;
        packet[LCD_HEADER_SIZE..].copy_from_slice(&compressed);

        self.send_packet(&packet)
    }

    /// Display RGB888 bitmap without scrolling
    pub fn display_bitmap(&mut self, rgb_data: &[u8]) -> Result<(), LcdError> {
        self.display_bitmap_with_scroll(rgb_data, ScrollMode::None)
    }

    /// Clear display (black)
    pub fn clear(&mut self) -> Result<(), LcdError> {
        let black = vec![0u8; (LCD_WIDTH * LCD_HEIGHT * 3) as usize];
        self.display_bitmap(&black)
    }

    fn rgb888_to_rgb565(rgb888: &[u8]) -> Vec<u8> {
        let mut rgb565 = Vec::with_capacity((LCD_WIDTH * LCD_HEIGHT * 2) as usize);
        for chunk in rgb888.chunks(3) {
            let r5 = (chunk[0] >> 3) as u16;
            let g6 = (chunk[1] >> 2) as u16;
            let b5 = (chunk[2] >> 3) as u16;
            // Device wants BGR565 (R/B swapped vs. standard RGB565). See SpaceLCD render.c rgbtobgr().
            let pixel = (b5 << 11) | (g6 << 5) | r5;
            rgb565.push((pixel & 0xFF) as u8);
            rgb565.push((pixel >> 8) as u8);
        }
        rgb565
    }

    fn compress_bitmap(data: &[u8]) -> Result<Vec<u8>, LcdError> {
        use miniz_oxide::deflate::core::{
            compress, create_comp_flags_from_zip_params, CompressorOxide, TDEFLFlush, TDEFLStatus,
        };

        // Match SpaceLCD's crush(): raw deflate (window_bits -15) with the Z_FIXED strategy.
        // The LCD's on-device inflater only handles fixed-Huffman blocks; dynamic-Huffman
        // output (flate2's default) decodes to garbage. MZ_FIXED forces static blocks.
        const MZ_FIXED: i32 = 4;
        let flags = create_comp_flags_from_zip_params(9, -15, MZ_FIXED);
        let mut comp = CompressorOxide::new(flags);

        let mut out = vec![0u8; data.len() / 2 + 128];
        let mut in_pos = 0;
        let mut out_pos = 0;
        loop {
            let (status, consumed, produced) = compress(
                &mut comp,
                &data[in_pos..],
                &mut out[out_pos..],
                TDEFLFlush::Finish,
            );
            in_pos += consumed;
            out_pos += produced;
            match status {
                TDEFLStatus::Done => break,
                TDEFLStatus::Okay => {
                    if out_pos == out.len() {
                        out.resize(out.len() * 2, 0);
                    }
                }
                _ => {
                    return Err(LcdError::CompressionError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "deflate failed",
                    )))
                }
            }
        }
        out.truncate(out_pos);
        Ok(out)
    }

    fn send_packet(&mut self, packet: &[u8]) -> Result<(), LcdError> {
        for chunk in packet.chunks(64) {
            self.handle
                .write_bulk(ENDPOINT_OUT, chunk, USB_TIMEOUT)
                .map_err(LcdError::UsbError)?;
        }
        Ok(())
    }
}

impl Drop for Lcd {
    fn drop(&mut self) {
        let _ = self.handle.release_interface(0);
    }
}
