//! Kill-on-close containment is assigned before the supervisor runs on Windows.

use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use anyhow::Context as _;
use tokio::process::Command;
use tokio::time::{Instant, timeout_at};

use super::pipes::Pipes;

#[cfg(windows)]
type Child = Box<dyn process_wrap::tokio::ChildWrapper>;
#[cfg(not(windows))]
type Child = tokio::process::Child;

pub(super) struct Process {
    child: Child,
    pub(super) pipes: Pipes,
}

impl Process {
    pub(super) fn spawn(mut command: Command) -> anyhow::Result<Self> {
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        let mut child = {
            use process_wrap::tokio::{CommandWrap, JobObject, KillOnDrop};
            // Do not unwrap the ChildWrapper: its job handle must stay owned.
            // JobObject temporarily suspends, assigns, then resumes the child.
            // KillOnDrop is also required to set JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE.
            CommandWrap::from(command)
                .wrap(KillOnDrop)
                .wrap(JobObject)
                .spawn()
                .context("start contained sibling s4-screen-supervisor.exe")?
        };
        #[cfg(not(windows))]
        let mut child = command.spawn().context("start screen supervisor fixture")?;
        #[cfg(windows)]
        let (input, output, errors) = (
            child.stdin().take(),
            child.stdout().take(),
            child.stderr().take(),
        );
        #[cfg(not(windows))]
        let (input, output, errors) =
            (child.stdin.take(), child.stdout.take(), child.stderr.take());
        let pipes = Pipes::start(
            input.context("screen helper stdin")?,
            output.context("screen helper stdout")?,
            errors.context("screen helper stderr")?,
        );
        Ok(Self { child, pipes })
    }

    pub(super) fn exited(&mut self) -> anyhow::Result<Option<ExitStatus>> {
        self.child
            .try_wait()
            .context("inspect screen supervisor exit")
    }

    pub(super) async fn reap(&mut self, deadline: Instant) -> anyhow::Result<ExitStatus> {
        // JobObjectChild::wait can spawn an unbounded blocking completion-port
        // wait. Poll the wrapper instead, retaining a truly bounded deadline.
        timeout_at(deadline, async {
            let mut tick = tokio::time::interval(Duration::from_millis(10));
            loop {
                if let Some(status) = self.exited()? {
                    return Ok(status);
                }
                tick.tick().await;
            }
        })
        .await
        .context("screen supervisor reap deadline")?
    }

    pub(super) async fn terminate(&mut self) -> anyhow::Result<()> {
        // The Windows wrapper kills the entire owned outer job, including the
        // supervisor's nested worker job. A blocked stdin future is not involved.
        self.child
            .start_kill()
            .context("terminate screen supervisor job")?;
        self.pipes.input = None;
        self.reap(Instant::now() + Duration::from_secs(1)).await?;
        Ok(())
    }
}
