use super::*;
use clap::Parser as _;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn diagnostic_configuration_rejects_invalid_contracts_before_devices() -> anyhow::Result<()> {
    assert!(
        crate::Args::try_parse_from([
            "bridge",
            "--managed-stdin",
            "--prepare-screen-motor-diagnostics",
        ])
        .is_err()
    );
    let root = std::env::temp_dir().join(format!(
        "s4-diag-options-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root)?;
    let config = root.join("config.json");
    let capture = root.join("trace.jsonl");
    let args = crate::Args::try_parse_from([
        "bridge",
        "--midi",
        "full",
        "--managed-stdin",
        "--screen-motor-diagnostic-config",
        config.to_str().context("test path")?,
    ])?;
    for value in [
        serde_json::json!({"version":2,"case":"normal","capture":false,"capture_path":capture}),
        serde_json::json!({"version":1,"case":"static","capture":false,"capture_path":capture}),
        serde_json::json!({"version":1,"case":"normal","capture":false,"capture_path":"relative.jsonl"}),
        serde_json::json!({"version":1,"case":"normal","capture":false,"capture_path":capture,"unknown":true}),
    ] {
        std::fs::write(&config, serde_json::to_vec(&value)?)?;
        assert!(Session::start(&args).is_err());
        assert!(!capture.exists());
    }
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn overhead_control_exports_metadata_without_creating_trace_or_changing_case() -> anyhow::Result<()>
{
    let root = std::env::temp_dir().join(format!(
        "s4-diag-overhead-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&root)?;
    let config = root.join("config.json");
    let capture = root.join("trace.jsonl");
    std::fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!({
            "version":1,"case":"normal","capture":false,"capture_path":capture
        }))?,
    )?;
    let args = crate::Args::try_parse_from([
        "bridge",
        "--midi",
        "full",
        "--managed-stdin",
        "--screen-motor-diagnostic-config",
        config.to_str().context("test path")?,
    ])?;
    Session::start(&args)?.finish(Ok(()))?;
    assert!(!capture.exists());
    let metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(capture.with_extension("metadata.json"))?)?;
    assert_eq!(metadata["capture"], false);
    assert_eq!(diag::case(), Case::Normal);
    std::fs::remove_dir_all(root)?;
    Ok(())
}
