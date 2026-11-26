//! Fresh upstream CPU encoders; no GPU, USB transport or device engine.
use anyhow::{Context, bail};
use encdr::{
    DeviceDescriptor, PixelFormat, device::loader::DescriptorRegistry, screen::ScreenManager,
};
use std::sync::Arc;

use crate::protocol::{HEIGHT, PAIR_BYTES, SCREEN_BYTES, WIDTH};

pub struct Encoder {
    descriptor: Arc<DeviceDescriptor>,
}

impl Encoder {
    pub fn new() -> anyhow::Result<Self> {
        let mut registry = DescriptorRegistry::new();
        registry.load_builtins().context("load Encdr descriptors")?;
        let descriptor = Arc::clone(
            registry
                .find(0x17cc, 0x1720)
                .context("missing S4 MK3 descriptor")?,
        );
        if descriptor.product_id.0 != 0x1720
            || descriptor.screens.len() != 2
            || descriptor.screens.iter().any(|screen| {
                screen.width != WIDTH
                    || screen.height != HEIGHT
                    || screen.pixel_format != PixelFormat::Bgr565Be
            })
        {
            bail!("unexpected upstream S4 screen descriptor");
        }
        Ok(Self { descriptor })
    }

    pub fn pair(&self, pixels: &[u8]) -> anyhow::Result<[Vec<u8>; 2]> {
        if pixels.len() != PAIR_BYTES {
            bail!("expected {PAIR_BYTES} RGB565 pair bytes");
        }
        let (left, right) = pixels.split_at(SCREEN_BYTES);
        Ok([self.screen(0, left)?, self.screen(1, right)?])
    }

    fn screen(&self, index: usize, pixels: &[u8]) -> anyhow::Result<Vec<u8>> {
        let desc = &self.descriptor.screens[index];
        // Despite its upstream name, Bgr565Be encodes red in bits 15..11,
        // green in 10..5 and blue in 4..0. Convert LE words to BE without
        // expanding to RGBA and re-quantizing/dithering already-quantized pixels.
        let native: Vec<u8> = pixels
            .chunks_exact(2)
            .flat_map(|pixel| [pixel[1], pixel[0]])
            .collect();
        let mut manager = ScreenManager::new(desc, None);
        manager
            .submit(&native, PixelFormat::Bgr565Be, desc)
            .context("upstream CPU encoder omitted initial full frame")
    }
}
