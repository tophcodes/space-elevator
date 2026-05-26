use spnav::lcd::{Lcd, LcdError};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct LcdHandle {
    inner: Arc<Mutex<Option<Lcd>>>,
}

impl LcdHandle {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(None)) }
    }

    async fn ensure_open(&self) -> Result<tokio::sync::MutexGuard<'_, Option<Lcd>>, LcdError> {
        let mut guard = self.inner.lock().await;
        if guard.is_none() {
            *guard = Some(Lcd::new()?);
        }
        Ok(guard)
    }

    pub async fn clear(&self) -> Result<(), LcdError> {
        let mut g = self.ensure_open().await?;
        g.as_mut().unwrap().clear()
    }

    pub async fn display_rgb888(&self, rgb: &[u8]) -> Result<(), LcdError> {
        let mut g = self.ensure_open().await?;
        g.as_mut().unwrap().display_bitmap(rgb)
    }
}

impl Default for LcdHandle {
    fn default() -> Self {
        Self::new()
    }
}
