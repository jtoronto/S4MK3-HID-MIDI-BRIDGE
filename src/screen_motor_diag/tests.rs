use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context as _;
use serde_json::Value;

use super::{Capture, Case, Kind, Role, record};

static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn activation_reaches_all_threads_without_leaking_to_parent() -> anyhow::Result<()> {
    const CHILD: &str = "S4_SCREEN_MOTOR_TEST_CHILD";
    if let Some(path) = std::env::var_os(CHILD) {
        let capture = Capture::start(std::path::Path::new(&path), Case::Acquired)?;
        capture.activate()?;
        {
            let _guard = super::enter(Role::Output);
            record(Kind::Lifecycle, [1, 0, 0, 0, 0, 0, 0, 0], &[]);
            let threads = [Role::Input, Role::Screen].map(|role| {
                std::thread::spawn(move || {
                    let _guard = super::enter(role);
                    assert_eq!(super::case(), Case::Acquired);
                    record(Kind::Lifecycle, [2, 0, 0, 0, 0, 0, 0, 0], &[]);
                })
            });
            for thread in threads {
                thread
                    .join()
                    .map_err(|_| anyhow::anyhow!("trace producer panicked"))?;
            }
            super::request_stop();
        }
        capture.finish(true)?;
        return Ok(());
    }
    let path = path();
    let output = std::process::Command::new(std::env::current_exe()?)
        .args([
            "--exact",
            "screen_motor_diag::tests::activation_reaches_all_threads_without_leaking_to_parent",
        ])
        .env(CHILD, &path)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = rows(&path)?;
    let footer = output.last().context("footer")?;
    assert_eq!(footer["incomplete"], false);
    for role in 0..3 {
        assert_eq!(footer["roles"][role]["count"], 1);
    }
    assert!(footer["first_stop_us"].as_u64().is_some());
    assert!(!super::enabled());
    Ok(())
}

#[test]
fn failed_export_reports_error_and_preserves_partial_file() -> anyhow::Result<()> {
    let path = path();
    let capture = Capture::start(&path, Case::Normal)?;
    {
        let mut state = capture
            .inner
            .state
            .lock()
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        state.file = Some(std::fs::File::open(&path)?);
    }
    assert!(capture.finish(true).is_err());
    assert!(path.exists());
    std::fs::remove_file(path)?;
    Ok(())
}

fn path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "s4-diag-test-{}-{}.jsonl",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}

fn rows(path: &PathBuf) -> anyhow::Result<Vec<Value>> {
    let data = std::fs::read_to_string(path)?;
    std::fs::remove_file(path)?;
    Ok(data
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?)
}

#[test]
fn records_preserve_order_and_span_identity() -> anyhow::Result<()> {
    let path = path();
    let capture = Capture::start(&path, Case::Updates)?;
    let guard = capture.enter(Role::Input);
    let begin = record(Kind::InputBegin, [0; 8], &[4, 5]);
    let end = record(Kind::InputEnd, [begin, 0, 0, 0, 0, 0, 0, 0], &[]);
    drop(guard);
    capture.finish(true)?;
    let output = rows(&path)?;
    assert_eq!(begin, 1);
    assert_eq!(end, 2);
    assert_eq!(output[1]["kind"], "input_begin");
    assert_eq!(output[1]["payload"], serde_json::json!([4, 5]));
    assert_eq!(output[2]["values"][0], output[1]["seq"]);
    assert_eq!(output[3]["incomplete"], false);
    Ok(())
}

#[test]
fn first_stop_is_preserved_in_export() -> anyhow::Result<()> {
    let path = path();
    let capture = Capture::start(&path, Case::Normal)?;
    capture.inner.stop_at(0);
    capture.inner.stop_at(42);
    capture.finish(false)?;
    let output = rows(&path)?;
    assert_eq!(output[1]["first_stop_us"], 0);
    assert_eq!(output[1]["runtime_ok"], false);
    Ok(())
}

#[test]
fn full_and_oversized_records_report_losses() -> anyhow::Result<()> {
    let path = path();
    let capture = Capture::start(&path, Case::Off)?;
    {
        let mut state = capture
            .inner
            .state
            .lock()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        state.slots[Role::Input.index()]
            .as_mut()
            .context("input")?
            .capacity = 1;
    }
    let guard = capture.enter(Role::Input);
    assert_eq!(record(Kind::Sensor, [0; 8], &[0; 96]), 0);
    assert_eq!(record(Kind::Sensor, [0; 8], &[]), 1);
    assert_eq!(record(Kind::Sensor, [0; 8], &[]), 0);
    drop(guard);
    capture.finish(true)?;
    let output = rows(&path)?;
    assert_eq!(output[2]["incomplete"], true);
    assert_eq!(output[2]["roles"][1]["losses"], 2);
    assert_eq!(output[2]["roles"][1]["count"], 1);
    Ok(())
}

#[test]
fn unfinished_guard_and_duplicate_claim_are_incomplete() -> anyhow::Result<()> {
    let path = path();
    let capture = Capture::start(&path, Case::Static)?;
    let guard = capture.enter(Role::Screen);
    let duplicate = capture.enter(Role::Screen);
    drop(duplicate);
    capture.finish(true)?;
    drop(guard);
    let output = rows(&path)?;
    assert_eq!(output[1]["incomplete"], true);
    assert_eq!(output[1]["roles"][2]["count"], Value::Null);
    Ok(())
}

#[test]
fn disabled_producer_is_noop() {
    assert_eq!(record(Kind::Lifecycle, [0; 8], &[]), 0);
    let guard = super::enter(Role::Output);
    assert_eq!(record(Kind::Lifecycle, [0; 8], &[]), 0);
    drop(guard);
    assert!(!super::enabled());
    assert_eq!(super::case(), Case::Normal);
    assert_eq!(super::now_us(), 0);
    super::request_stop();
}

#[test]
fn start_rejects_relative_and_existing_paths() -> anyhow::Result<()> {
    assert!(Capture::start(std::path::Path::new("relative.jsonl"), Case::Normal).is_err());
    let path = path();
    let capture = Capture::start(&path, Case::Normal)?;
    assert!(Capture::start(&path, Case::Normal).is_err());
    capture.finish(true)?;
    let _output = rows(&path)?;
    Ok(())
}

#[test]
fn thread_capture_drives_accessors_and_header_anchor() -> anyhow::Result<()> {
    let path = path();
    let capture = Capture::start(&path, Case::Acquired)?;
    let anchor = capture.inner.utc_unix_us;
    let guard = capture.enter(Role::Screen);
    assert!(super::enabled());
    assert_eq!(super::case(), Case::Acquired);
    let before = super::now_us();
    record(Kind::ScreenReady, [0; 8], &[]);
    let after = super::now_us();
    super::request_stop();
    drop(guard);
    capture.finish(true)?;
    let output = rows(&path)?;
    assert_eq!(output[0]["utc_unix_us"], anchor);
    let event_us = output[1]["us"].as_u64().context("event timestamp")?;
    assert!((before..=after).contains(&event_us));
    assert!(output[2]["first_stop_us"].as_u64().is_some());
    Ok(())
}
