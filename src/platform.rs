//! Select the inventoried control collection without claiming USB or audio.

use anyhow::{Context as _, ensure};
use hidapi::{DeviceInfo, HidApi, HidDevice, HidResult};

pub fn write(hid: &HidDevice, report: &[u8]) -> anyhow::Result<usize> {
    write_with(report, cfg!(windows), |bytes| hid.write(bytes))
}

pub fn write_with(
    report: &[u8],
    windows: bool,
    writer: impl FnOnce(&[u8]) -> HidResult<usize>,
) -> anyhow::Result<usize> {
    let written = writer(report)?;
    // The inventoried MI_03 collection has a 95-byte output capability.
    // The selected Windows hidapi C backend pads shorter reports to that size.
    if windows && report.len() < 95 {
        ensure!(written == 95, "short S4 padded output write: {written}/95");
        Ok(report.len())
    } else {
        Ok(written)
    }
}

pub fn collection(api: &HidApi) -> anyhow::Result<&DeviceInfo> {
    let candidates: Vec<_> = api
        .device_list()
        .filter(|device| device.vendor_id() == 0x17cc && device.product_id() == 0x1720)
        .collect();
    let selected: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|device| {
            !cfg!(windows)
                || windows_control(
                    device.interface_number(),
                    device.usage_page(),
                    device.usage(),
                    &device.path().to_string_lossy(),
                )
        })
        .collect();
    ensure!(
        selected.len() == 1,
        "expected one verified S4 HID control collection; matching={}, candidates={:?}",
        selected.len(),
        candidates
            .iter()
            .map(|device| (
                device.path(),
                device.interface_number(),
                device.usage_page(),
                device.usage()
            ))
            .collect::<Vec<_>>()
    );
    selected
        .first()
        .copied()
        .context("S4 control collection absent")
}

fn windows_control(interface: i32, page: u16, usage: u16, path: &str) -> bool {
    // Actual MI_03 inventory: usage FF01:0000. Instance and serial are not pinned
    // to one laptop; exact interface path component excludes audio/display/DFU.
    interface == 3
        && page == 0xff01
        && usage == 0
        && path.to_ascii_lowercase().split('#').nth(1) == Some("vid_17cc&pid_1720&mi_03")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_write_normalizes_only_completed_windows_padding() -> anyhow::Result<()> {
        for (length, returned, windows, expected) in [
            (11, 95, true, 11),
            (28, 95, true, 28),
            (41, 95, true, 41),
            (79, 95, true, 79),
            (95, 95, true, 95),
            (96, 95, true, 95),
            (11, 95, false, 95),
            (11, 11, false, 11),
        ] {
            let mut report = vec![0x5a; length];
            report[0] = 0x31;
            let actual = write_with(&report, windows, |bytes| {
                assert_eq!(bytes, report);
                Ok(returned)
            })?;
            assert_eq!(
                actual, expected,
                "length={length} returned={returned} windows={windows}"
            );
        }
        for returned in [0, 10, 11, 94, 96] {
            assert!(write_with(&[0x31; 11], true, |_| Ok(returned)).is_err());
        }
        Ok(())
    }

    #[test]
    fn windows_collection_requires_inventoried_interface_usage_and_path() {
        let path = r"\\?\hid#vid_17cc&pid_1720&mi_03#9&21c48119&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}";
        assert!(windows_control(3, 0xff01, 0, path));
        for (interface, page, usage, path) in [
            (4, 0xff01, 0, path),
            (3, 1, 0, path),
            (3, 0xff01, 1, path),
            (3, 0xff01, 0, r"\\?\hid#vid_17cc&pid_17200&mi_03#instance"),
            (3, 0xff01, 0, r"\\?\hid#vid_17cc&pid_1720&mi_04#instance"),
        ] {
            assert!(!windows_control(interface, page, usage, path));
        }
    }
}
