use super::*;
use std::io::Write as _;
use std::process::{Command, Stdio};

const CHILD: &str = "S4_INVENTORY_PROCESS_TEST_CHILD";
const SIZE: usize = 256 * 1024;

struct Fixture(Child);

impl Fixture {
    fn start(test: &str) -> anyhow::Result<Self> {
        let (_, test) = test
            .split_once("::")
            .context("fixture test lacks crate prefix")?;
        let child = Command::new(std::env::current_exe()?)
            .args(["--exact", test, "--nocapture"])
            .env(CHILD, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        Ok(Self(child))
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        match self.0.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => {}
            Err(error) => tracing::warn!(%error, "fixture status unavailable"),
        }
        if let Err(error) = self.0.kill() {
            tracing::warn!(%error, "could not terminate process fixture");
        }
        if let Err(error) = self.0.wait() {
            tracing::warn!(%error, "could not reap process fixture");
        }
    }
}

#[test]
fn completion_drains_both_pipes_beyond_their_capacity() -> anyhow::Result<()> {
    if std::env::var_os(CHILD).is_some() {
        std::io::stdout().write_all(&vec![b'o'; SIZE])?;
        std::io::stderr().write_all(&vec![b'e'; SIZE])?;
        std::io::stdout().flush()?;
        std::io::stderr().flush()?;
        std::process::exit(0);
    }
    let mut fixture = Fixture::start(concat!(
        module_path!(),
        "::completion_drains_both_pipes_beyond_their_capacity"
    ))?;
    let output = capture(&mut fixture.0, Duration::from_secs(5))?;
    assert!(output.status.success());
    assert!(output.stdout.ends_with(&vec![b'o'; SIZE]));
    assert_eq!(output.stderr, vec![b'e'; SIZE]);
    assert!(fixture.0.try_wait()?.is_some());
    Ok(())
}

#[test]
fn nonzero_exit_returns_the_child_status_and_causal_stderr() -> anyhow::Result<()> {
    if std::env::var_os(CHILD).is_some() {
        std::io::stderr().write_all(b"inventory query fixture failed")?;
        std::io::stderr().flush()?;
        std::process::exit(23);
    }
    let mut fixture = Fixture::start(concat!(
        module_path!(),
        "::nonzero_exit_returns_the_child_status_and_causal_stderr"
    ))?;
    let error = capture(&mut fixture.0, Duration::from_secs(5))
        .err()
        .context("nonzero fixture succeeded")?;
    assert!(error.to_string().contains("inventory query fixture failed"));
    assert_eq!(
        fixture
            .0
            .try_wait()?
            .context("fixture still running")?
            .code(),
        Some(23)
    );
    Ok(())
}

#[test]
fn expired_deadline_terminates_and_reaps_only_the_owned_child() -> anyhow::Result<()> {
    if std::env::var_os(CHILD).is_some() {
        loop {
            std::thread::park();
        }
    }
    let mut fixture = Fixture::start(concat!(
        module_path!(),
        "::expired_deadline_terminates_and_reaps_only_the_owned_child"
    ))?;
    let mut sibling = Fixture::start(concat!(
        module_path!(),
        "::expired_deadline_terminates_and_reaps_only_the_owned_child"
    ))?;
    let id = fixture.0.id();
    let error = capture(&mut fixture.0, Duration::ZERO)
        .err()
        .context("expired inventory succeeded")?;
    assert!(
        error
            .to_string()
            .contains(&format!("inventory child {id} timed out"))
    );
    assert!(
        !fixture
            .0
            .try_wait()?
            .context("timed-out child still running")?
            .success()
    );
    assert!(sibling.0.try_wait()?.is_none());
    Ok(())
}
