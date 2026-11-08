//! Private paired-frame IPC with the separately packaged Encdr supervisor.

use tokio::io::AsyncWriteExt as _;
use tokio::process::Command;
use tokio::time::{Instant as Deadline, timeout_at};

use super::*;

mod pipes;
mod process;
mod supervisor;
pub(in crate::screens::worker) use supervisor::Supervisor;

use pipes::Ack;
use process::Process;

#[derive(Debug)]
pub(super) struct Unconfirmed;

impl std::fmt::Display for Unconfirmed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("screen release unconfirmed; NI ownership restoration not established")
    }
}
impl std::error::Error for Unconfirmed {}

#[derive(Clone, Copy)]
struct Limits {
    setup: Duration,
    frame: Duration,
    stop: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            setup: Duration::from_secs(8),
            frame: Duration::from_millis(1500),
            stop: Duration::from_millis(1500),
        }
    }
}

async fn cancelled(cancel: &mut Cancellation) {
    loop {
        if *cancel.borrow_and_update() || cancel.changed().await.is_err() {
            return;
        }
    }
}

async fn bounded<T>(
    cancel: &mut Cancellation,
    deadline: Deadline,
    operation: impl std::future::Future<Output = anyhow::Result<T>>,
) -> anyhow::Result<T> {
    tokio::select! {
        biased;
        () = cancelled(cancel) => anyhow::bail!("screen helper operation cancelled"),
        result = timeout_at(deadline, operation) => result.context("screen helper operation deadline")?,
    }
}

struct Session {
    process: Process,
    cancellation: Cancellation,
    limits: Limits,
    usable: bool,
    closed: bool,
}

impl Session {
    async fn connect(
        command: Command,
        cancellation: Cancellation,
        limits: Limits,
    ) -> anyhow::Result<Self> {
        let mut session = Self {
            process: Process::spawn(command)?,
            cancellation,
            limits,
            usable: true,
            closed: false,
        };
        let ready = bounded(
            &mut session.cancellation,
            Deadline::now() + limits.setup,
            session.process.pipes.reply(Ack::Ready),
        )
        .await;
        if let Err(error) = ready {
            session.usable = false;
            session.close().await.with_context(|| error.to_string())?;
            return Err(error);
        }
        Ok(session)
    }

    async fn exchange(&mut self, record: &[u8], expected: Ack) -> anyhow::Result<()> {
        let input = self
            .process
            .pipes
            .input
            .as_mut()
            .context("screen helper stdin closed")?;
        input.write_all(record).await?;
        input.flush().await?;
        self.process.pipes.reply(expected).await
    }

    async fn publish(&mut self, frames: &[Frame; 2]) -> anyhow::Result<()> {
        self.poll_control().await?;
        let mut record = Vec::with_capacity(307_205);
        record.push(1);
        record.extend_from_slice(&307_200_u32.to_le_bytes());
        for frame in frames {
            anyhow::ensure!(frame.pixels.len() == 76_800, "screen raster dimensions");
            for pixel in &frame.pixels {
                record.extend_from_slice(&pixel.to_le_bytes());
            }
        }
        // An interrupted write poisons the stream: never append STOP to a
        // partial frame or mistake its late ACK for a later generation.
        let mut cancel = self.cancellation.clone();
        let result = bounded(
            &mut cancel,
            Deadline::now() + self.limits.frame,
            self.exchange(&record, Ack::Frame),
        )
        .await;
        self.usable &= result.is_ok();
        result
    }

    async fn poll_control(&mut self) -> anyhow::Result<()> {
        anyhow::ensure!(self.usable && !self.closed, "screen helper session failed");
        tokio::task::yield_now().await;
        let result = (|| {
            self.process.pipes.poll_drains()?;
            anyhow::ensure!(self.process.exited()?.is_none(), "screen helper exited");
            match self.process.pipes.replies.try_recv() {
                Ok(reply) => {
                    reply?;
                    anyhow::bail!("unsolicited screen helper acknowledgement");
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => Ok(()),
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    anyhow::bail!("screen helper stdout disconnected");
                }
            }
        })();
        self.usable &= result.is_ok();
        result
    }

    async fn close(&mut self) -> anyhow::Result<()> {
        if self.closed {
            anyhow::ensure!(self.usable, Unconfirmed);
            return Ok(());
        }
        self.closed = true;
        let deadline = Deadline::now() + self.limits.stop;
        let result = if self.usable {
            timeout_at(deadline, async {
                self.exchange(&[2, 0, 0, 0, 0], Ack::Stopped).await?;
                self.process.pipes.input = None;
                anyhow::ensure!(
                    self.process.reap(deadline).await?.success(),
                    "screen helper stop exit failed"
                );
                self.process.pipes.finish().await?;
                Ok(())
            })
            .await
            .context("screen helper stop deadline")
            .and_then(std::convert::identity)
        } else {
            Err(anyhow::anyhow!("interrupted screen helper transaction"))
        };
        self.usable = result.is_ok();
        if let Err(error) = result {
            if let Err(termination) = self.process.terminate().await {
                tracing::warn!(%termination, "screen supervisor termination/reap unconfirmed");
            }
            return Err(anyhow::Error::new(Unconfirmed).context(error));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
