//! Full-profile hardware boundary; the offline handle exists only in macOS tests.

use encdr::{Event, device::parser::PacketParser};
use hidapi::HidDevice;

pub enum Decoder<'a> {
    Native(&'a mut PacketParser),
    // Encdr's DeviceId cannot be constructed without USB enumeration.
    // FullBridge still runs its own raw snapshot decoder.
    #[cfg(all(test, target_os = "macos"))]
    Offline,
}

impl Decoder<'_> {
    pub fn parse(&mut self, payload: &[u8], events: &mut Vec<Event>) {
        match self {
            Self::Native(parser) => parser.parse_from("control", payload, events),
            #[cfg(all(test, target_os = "macos"))]
            Self::Offline => {}
        }
    }
}

pub enum Handle {
    Native(HidDevice),
    #[cfg(all(test, target_os = "macos"))]
    Offline,
}

impl From<HidDevice> for Handle {
    fn from(hid: HidDevice) -> Self {
        Self::Native(hid)
    }
}

pub fn analog(hid: &Handle) -> anyhow::Result<[u8; 79]> {
    match hid {
        Handle::Native(hid) => crate::input::read_analog(hid),
        #[cfg(all(test, target_os = "macos"))]
        Handle::Offline => {
            let mut report = [0; 79];
            report[0] = 2;
            Ok(report)
        }
    }
}

pub fn batch(hid: &Handle, buffers: &mut [[u8; 128]; 3]) -> anyhow::Result<[usize; 3]> {
    match hid {
        Handle::Native(hid) => crate::input::read_batch(hid, true, buffers),
        #[cfg(all(test, target_os = "macos"))]
        Handle::Offline => {
            buffers[0][0] = 1;
            buffers[1][0] = 3;
            Ok([23, 60, 0])
        }
    }
}

pub fn write(hid: &Handle, report: &[u8]) -> anyhow::Result<usize> {
    match hid {
        Handle::Native(hid) => crate::platform::write(hid, report),
        #[cfg(all(test, target_os = "macos"))]
        Handle::Offline => {
            if !matches!(report.first(), Some(0x80 | 0x81)) {
                eprintln!("S4_FIXTURE_FORBIDDEN_HID_WRITE {:?}", report.first());
                anyhow::bail!("offline fixture forbids raw screen, motor and ring writes");
            }
            Ok(report.len())
        }
    }
}
