use flate2::write::DeflateEncoder;
use flate2::Compression;
use hidapi::{HidApi, HidDevice};
use std::io::Write;

const LCD_WIDTH: u32 = 640;
const LCD_HEIGHT: u32 = 150;
const LCD_HEADER_SIZE: usize = 512;
const USB_VENDOR_ID: u16 = 0x256f;
const USB_PRODUCT_ID: u16 = 0xc633;
const LCD_INTERFACE: i32 = 1;

pub struct Lcd {
    device: HidDevice,
}

#[derive(Debug)]
pub enum LcdError {
    DeviceNotFound,
    UsbError(hidapi::HidError),
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

impl std::error::Error for LcdError {}

impl Lcd {
    pub fn new() -> Result<Self, LcdError> {
        let api = HidApi::new().map_err(LcdError::UsbError)?;

        let device = api
            .device_list()
            .find(|dev| {
                dev.vendor_id() == USB_VENDOR_ID
                    && dev.product_id() == USB_PRODUCT_ID
                    && dev.interface_number() == LCD_INTERFACE
            })
            .ok_or(LcdError::DeviceNotFound)?
            .open_device(&api)
            .map_err(LcdError::UsbError)?;

        Ok(Self { device })
    }

    pub fn display_bitmap(&mut self, rgb_data: &[u8]) -> Result<(), LcdError> {
        if rgb_data.len() != (LCD_WIDTH * LCD_HEIGHT * 3) as usize {
            return Err(LcdError::InvalidDimensions);
        }

        let rgb565 = Lcd::rgb888_to_rgb565(rgb_data);
        let compressed = Lcd::compress_bitmap(&rgb565)?;

        let packet = Lcd::build_packet(&compressed)?;
        self.send_packet(&packet)?;

        Ok(())
    }

    fn rgb888_to_rgb565(rgb888: &[u8]) -> Vec<u8> {
        let mut rgb565 = Vec::with_capacity((LCD_WIDTH * LCD_HEIGHT * 2) as usize);

        for chunk in rgb888.chunks(3) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];

            let r5 = (r >> 3) as u16; // keep 5 bits
            let g6 = (g >> 2) as u16; // keep 6 bits
            let b5 = (b >> 3) as u16; // keep 5 bits

            let pixel = (r5 << 11) | (g6 << 5) | b5;
            rgb565.push((pixel & 0xff) as u8);
            rgb565.push((pixel >> 8) as u8);
        }

        rgb565
    }

    fn compress_bitmap(data: &[u8]) -> Result<Vec<u8>, LcdError> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(data)
            .map_err(LcdError::CompressionError)?;

        let compressed = encoder.finish().map_err(LcdError::CompressionError)?;

        if compressed.len() > 65535 {
            return Err(LcdError::ImageTooLarge);
        }

        Ok(compressed)
    }

    fn build_packet(compressed: &[u8]) -> Result<Vec<u8>, LcdError> {
        let mut packet = vec![0u8; LCD_HEADER_SIZE + compressed.len()];

        packet[0] = 0x0b; // command
        packet[1] = 0x01; // subcommand

        let len = compressed.len() as u16;
        packet[2] = (len & 0xff) as u8;
        packet[3] = (len >> 8) as u8;

        let uncompressed_len = (LCD_WIDTH * LCD_HEIGHT * 2) as u32;
        packet[4] = (uncompressed_len & 0xff) as u8;
        packet[5] = ((uncompressed_len >> 8) & 0xff) as u8;
        packet[6] = ((uncompressed_len >> 16) & 0xff) as u8;
        packet[7] = ((uncompressed_len >> 24) & 0xff) as u8;

        // TODO: Implement rest of the packet header

        packet[LCD_HEADER_SIZE..].copy_from_slice(compressed);

        Ok(packet)
    }

    fn send_packet(&mut self, packet: &[u8]) -> Result<(), LcdError> {
        // TODO: Reverse-engineer report structure.
        self.device.write(packet).map_err(LcdError::UsbError)?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), LcdError> {
        let black = vec![0u8; (LCD_WIDTH * LCD_HEIGHT * 3) as usize];
        self.display_bitmap(&black)
    }
}
