//! Exercise production export and platform admission without runtime activation.

use anyhow::Context as _;
use std::process::Command;

fn executable() -> std::ffi::OsString {
    std::env::var_os("S4_BRIDGE_TEST_EXECUTABLE")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_s4mk3-hid-midi-bridge").into())
}

#[test]
fn windows_export_is_offline_and_preserves_existing_destination() -> anyhow::Result<()> {
    let directory = std::env::temp_dir().join(format!(
        "s4-windows-mapping-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir(&directory)?;
    let result = (|| -> anyhow::Result<()> {
        for (backend, endpoint) in [
            ("loopbe", "LoopBe Internal MIDI"),
            ("bloop", "S4 MK3 Bridge BLOOP"),
        ] {
            let path = directory.join(format!("{backend}.djayMidiMapping"));
            let run = || {
                Command::new(executable())
                    .args(["--midi-backend", backend, "--generate-mapping"])
                    .arg(&path)
                    .output()
            };
            let output = run()?;
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stdout.is_empty());
            let bytes = std::fs::read(&path)?;
            let mapping = plist::Value::from_reader(std::io::Cursor::new(&bytes))?;
            let root = mapping.as_dictionary().context("mapping dictionary")?;
            assert_eq!(root["endpointName"].as_string(), Some(endpoint));
            assert_eq!(root["controls"].as_array().context("controls")?.len(), 582);
            assert!(!run()?.status.success());
            assert_eq!(std::fs::read(&path)?, bytes);
        }
        Ok(())
    })();
    std::fs::remove_dir_all(&directory)?;
    result
}

#[test]
fn unavailable_runtime_backend_fails_before_receipts_or_device_access() -> anyhow::Result<()> {
    let backends: &[&str] = if cfg!(windows) {
        &["core-midi"]
    } else {
        &["loopbe", "bloop"]
    };
    for backend in backends {
        for profile in [None, Some("minimal"), Some("full")] {
            let mut command = Command::new(executable());
            command.args(["--midi-backend", backend, "--managed-stdin"]);
            if let Some(profile) = profile {
                command.args(["--midi", profile]);
            }
            let result = command.output()?;
            assert!(!result.status.success());
            assert!(result.stdout.is_empty());
            let stderr = String::from_utf8(result.stderr)?;
            assert!(stderr.contains("unavailable on this platform"));
            assert!(!stderr.contains("S4_HID_OPEN"));
        }
    }
    Ok(())
}
