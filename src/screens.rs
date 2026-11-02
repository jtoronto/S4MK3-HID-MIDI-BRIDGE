//! S4 display raster and interface-scoped USB output, independent of HID/audio.
//!
//! Raw blit framing follows Mixxx's TraktorKontrolS4MK3Screens.qml:
//! 16-byte 0x84/0x21 header, RGB565 big-endian pixels, 4-byte 0x40 footer.

use std::io::Write as _;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context as _, ensure};
use nusb::MaybeFuture as _;
use nusb::transfer::{Bulk, Out};

pub mod info;
pub mod page;
mod text;
mod worker;
pub use worker::{Action as ControlAction, Control, Worker};

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;
pub const PIXEL_BYTES: usize = WIDTH * HEIGHT * 2;
pub const PACKET_BYTES: usize = 16 + PIXEL_BYTES + 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    Left,
    Right,
}

impl Display {
    const fn index(self) -> u8 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
}

/// Logical top-left-origin RGB565 image. Reused between redraws.
pub struct Frame {
    pixels: Vec<u16>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            pixels: vec![0; WIDTH * HEIGHT],
        }
    }
}

impl Frame {
    pub fn clear(&mut self, color: u16) {
        self.pixels.fill(color);
    }

    pub fn rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u16) {
        for row in y..y.saturating_add(height).min(HEIGHT) {
            let start = row * WIDTH + x.min(WIDTH);
            let end = row * WIDTH + x.saturating_add(width).min(WIDTH);
            self.pixels[start..end].fill(color);
        }
    }

    pub fn packet(&self, display: Display, output: &mut Vec<u8>) {
        output.clear();
        output.extend_from_slice(&[
            0x84,
            0,
            display.index(),
            0x21,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            1,
            0x40,
            0,
            0xf0,
        ]);
        for pixel in &self.pixels {
            output.extend_from_slice(&pixel.to_be_bytes());
        }
        output.extend_from_slice(&[0x40, 0, display.index(), 0]);
    }

    /// Offline view of the exact quantized pixels sent to the controller.
    ///
    /// # Errors
    /// Returns an error if the destination exists or file creation/writing fails.
    pub fn write_ppm(&self, path: &Path) -> anyhow::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!("create screen preview {}", path.display()))?;
        write!(file, "P6\n{WIDTH} {HEIGHT}\n255\n")?;
        for pixel in &self.pixels {
            let r = u8::try_from(((pixel >> 11) & 31) * 255 / 31)?;
            let g = u8::try_from(((pixel >> 5) & 63) * 255 / 63)?;
            let b = u8::try_from((pixel & 31) * 255 / 31)?;
            file.write_all(&[r, g, b])?;
        }
        Ok(())
    }
}

#[must_use]
pub const fn rgb(red: u8, green: u8, blue: u8) -> u16 {
    ((red as u16 >> 3) << 11) | ((green as u16 >> 2) << 5) | (blue as u16 >> 3)
}

#[must_use]
pub fn test_pattern(display: Display) -> Frame {
    let mut frame = Frame::default();
    frame.clear(rgb(12, 16, 24));
    let title = match display {
        Display::Left => "LEFT SCREEN",
        Display::Right => "RIGHT SCREEN",
    };
    frame.text(16, 16, title, 2, rgb(255, 255, 255));
    frame.text(16, 48, "TOP LEFT", 1, rgb(255, 255, 255));
    for (index, (label, color)) in [
        ("RED", rgb(255, 0, 0)),
        ("GREEN", rgb(0, 255, 0)),
        ("BLUE", rgb(0, 0, 255)),
        ("WHITE", rgb(255, 255, 255)),
    ]
    .into_iter()
    .enumerate()
    {
        frame.rect(16 + index * 76, 80, 60, 90, color);
        frame.text(16 + index * 76, 180, label, 1, rgb(255, 255, 255));
    }
    frame.text(16, 216, "320 X 240 / RGB565", 1, rgb(255, 255, 255));
    frame.rect(0, 0, 8, 8, rgb(255, 255, 255));
    frame.rect(WIDTH - 8, HEIGHT - 8, 8, 8, rgb(255, 255, 0));
    frame
}

pub struct UsbScreens {
    endpoint: nusb::Endpoint<Bulk, Out>,
    packet: Vec<u8>,
}

impl UsbScreens {
    /// Open only the display interface, without changing device configuration.
    ///
    /// # Errors
    /// Returns an error for enumeration, ambiguous devices, or unavailable interface access.
    pub fn open() -> anyhow::Result<Self> {
        let mut devices = nusb::list_devices()
            .wait()?
            .filter(|device| device.vendor_id() == 0x17cc && device.product_id() == 0x1720);
        let info = devices.next().context("no connected S4 MK3 for screens")?;
        ensure!(
            devices.next().is_none(),
            "connect only one S4 MK3 for screens"
        );
        let device = info.open().wait().context("open S4 screen USB device")?;
        // Never change configuration, reset the device, or detach audio/HID drivers.
        let interface = device
            .claim_interface(4)
            .wait()
            .context("claim S4 screen interface 4; quit NIHardwareAgent/Traktor/Mixxx if busy")?;
        let endpoint = interface
            .endpoint::<Bulk, Out>(0x03)
            .context("open S4 screen bulk OUT 0x03")?;
        Ok(Self {
            endpoint,
            packet: Vec::with_capacity(PACKET_BYTES),
        })
    }

    /// Send a full raster with a bounded transfer deadline.
    ///
    /// # Errors
    /// Returns an error for failed, timed-out, or short USB transfers.
    pub fn write(&mut self, screen: Display, frame: &Frame) -> anyhow::Result<()> {
        frame.packet(screen, &mut self.packet);
        let completion = self
            .endpoint
            .transfer_blocking(self.packet.as_slice().into(), Duration::from_millis(500));
        completion.status.context("S4 screen bulk transfer")?;
        ensure!(
            completion.actual_len == PACKET_BYTES,
            "short S4 screen write: {}/{}",
            completion.actual_len,
            PACKET_BYTES
        );
        tracing::debug!(
            ?screen,
            bytes = completion.actual_len,
            "SCREEN_FRAME_WRITTEN"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_blit_frames_both_displays_with_big_endian_pixels() {
        // Given a known RGB565 frame.
        let mut frame = Frame::default();
        frame.rect(0, 0, 1, 1, rgb(255, 0, 0));
        frame.rect(WIDTH - 1, HEIGHT - 1, 1, 1, rgb(0, 0, 255));
        for display in [Display::Left, Display::Right] {
            // When serialized using the real transport framing.
            let mut packet = Vec::new();
            frame.packet(display, &mut packet);
            // Then screen selection, dimensions, byte order, and footer are exact.
            assert_eq!(packet.len(), PACKET_BYTES);
            assert_eq!(
                &packet[..16],
                &[
                    0x84,
                    0,
                    display.index(),
                    0x21,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    1,
                    0x40,
                    0,
                    0xf0,
                ]
            );
            assert_eq!(&packet[16..18], &[0xf8, 0]);
            assert_eq!(&packet[16 + PIXEL_BYTES - 2..16 + PIXEL_BYTES], &[0, 0x1f]);
            assert_eq!(&packet[16 + PIXEL_BYTES..], &[0x40, 0, display.index(), 0]);
        }
    }

    #[test]
    fn drawing_clips_at_screen_edges_without_wrapping_rows() {
        // Given a black raster.
        let mut frame = Frame::default();
        // When a rectangle extends beyond the bottom-right corner.
        frame.rect(WIDTH - 2, HEIGHT - 2, 20, 20, 0xffff);
        frame.rect(WIDTH + 1, 0, 2, 2, 0xffff);
        // Then only the four in-bounds pixels changed.
        assert_eq!(frame.pixels.iter().filter(|pixel| **pixel != 0).count(), 4);
    }
}
