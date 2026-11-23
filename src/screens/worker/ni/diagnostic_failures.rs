use super::*;
use crate::screen_motor_diag::{Capture, Case, Role};

struct Failed;
impl Transport for Failed {
    fn poll_control(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
    fn publish(&mut self, _: &[Frame; 2]) -> anyhow::Result<()> {
        anyhow::bail!("publish failed")
    }
    fn close(&mut self) -> anyhow::Result<()> {
        anyhow::bail!("release unconfirmed")
    }
}

#[test]
fn failed_publication_and_release_are_recorded_without_case_readiness() -> anyhow::Result<()> {
    let path =
        std::env::temp_dir().join(format!("s4-screen-terminal-{}.jsonl", std::process::id()));
    let capture = Capture::start(&path, Case::Static)?;
    let trace = capture.clone();
    let (status, received) = crossbeam_channel::bounded(4);
    let mut worker = Worker::spawn(Mailbox::default(), move |shared| {
        let _guard = trace.enter(Role::Screen);
        lifecycle::run_case(
            shared,
            |_, _| Ok(Failed),
            false,
            &move |_, state| {
                assert!(status.send(state).is_ok());
            },
            Case::Static,
        )
    })?;
    worker
        .control()
        .bind_identity(Identity::selected_hid("18F73BA2")?)?;
    worker.publish(
        [page::Page::Stems {
            shifted: false,
            fx: page::FxWarning::default(),
            backspin: None,
            pads: [page::PadState {
                active: Some(false),
                held: false,
            }; 8],
        }; 2],
    );
    worker.control().request(1, Action::On);
    assert_eq!(
        received.recv_timeout(Duration::from_secs(3))?,
        State::Pending
    );
    assert_eq!(
        received.recv_timeout(Duration::from_secs(3))?,
        State::Failed
    );
    drop(worker);
    capture.finish(true)?;
    let rows = std::fs::read_to_string(&path)?
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?;
    for kind in ["screen_publish_end", "screen_release_end"] {
        assert_eq!(
            rows.iter()
                .find(|row| row["kind"] == kind)
                .context("failed span")?["values"][2],
            0
        );
    }
    assert!(!rows.iter().any(|row| row["kind"] == "screen_case_ready"));
    std::fs::remove_file(path)?;
    Ok(())
}
