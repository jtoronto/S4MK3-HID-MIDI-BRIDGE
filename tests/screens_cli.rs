//! Exercise real screen CLI boundaries without opening the controller.

use std::process::Command;

#[test]
fn screen_output_requires_explicit_full_profile_before_usb() -> anyhow::Result<()> {
    // Given commands that must not open screen USB access.
    for args in [
        vec!["--screens"],
        vec!["--midi", "minimal", "--screens"],
        vec!["--list", "--screens"],
        vec!["--generate-mapping", "/dev/null/not-created", "--screens"],
        vec!["--screen-control-stdin"],
        vec!["--midi", "minimal", "--screen-control-stdin"],
        vec!["--list", "--midi", "full", "--screen-control-stdin"],
        vec![
            "--generate-mapping",
            "/dev/null/not-created",
            "--screen-control-stdin",
        ],
        vec![
            "--generate-screen-previews",
            "/dev/null/not-created",
            "--screen-control-stdin",
        ],
    ] {
        // When the actual executable validates the arguments.
        let output = Command::new(env!("CARGO_BIN_EXE_s4mk3-hid-midi-bridge"))
            .args(args)
            .output()?;
        // Then admission fails before enumeration or hardware access.
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
    Ok(())
}

#[test]
fn real_preview_export_is_offline_and_refuses_overwriting_images() -> anyhow::Result<()> {
    // Given a unique output directory, isolated from every other test process.
    let path = std::env::temp_dir().join(format!(
        "s4-screen-previews-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir(&path)?;
    let result = (|| -> anyhow::Result<()> {
        // When the actual CLI generates all screen states without a live flag.
        let run = || {
            Command::new(env!("CARGO_BIN_EXE_s4mk3-hid-midi-bridge"))
                .arg("--generate-screen-previews")
                .arg(&path)
                .output()
        };
        let output = run()?;
        // Then each raster is complete, no USB opens, and existing pixels survive a repeat.
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("S4_USB_FOUND"));
        let files = std::fs::read_dir(&path)?.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(files.len(), 51);
        for state in ["left", "right", "acquired"] {
            assert!(path.join(format!("tempo-pickup-{state}.ppm")).is_file());
        }
        for file in &files {
            let bytes = std::fs::read(file.path())?;
            assert!(bytes.starts_with(b"P6\n320 240\n255\n"));
            assert_eq!(bytes.len(), b"P6\n320 240\n255\n".len() + 320 * 240 * 3);
        }
        let first = std::fs::read(files[0].path())?;
        assert!(!run()?.status.success());
        assert_eq!(std::fs::read(files[0].path())?, first);
        Ok(())
    })();
    std::fs::remove_dir_all(&path)?;
    result
}
