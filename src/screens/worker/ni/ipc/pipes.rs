//! Bounded protocol reader and independently drained stderr.

use anyhow::Context as _;
use tokio::io::{AsyncBufReadExt as _, AsyncReadExt as _, BufReader};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout};
use tokio::sync::mpsc::{Receiver, channel};
use tokio::task::JoinSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Ack {
    Ready,
    Frame,
    Stopped,
}

pub(super) struct Pipes {
    pub(super) input: Option<ChildStdin>,
    pub(super) replies: Receiver<anyhow::Result<Option<Ack>>>,
    drains: JoinSet<anyhow::Result<()>>,
}

async fn read_reply(reader: &mut BufReader<ChildStdout>) -> anyhow::Result<Option<Ack>> {
    let mut line = Vec::with_capacity(128);
    (&mut *reader)
        .take(1025)
        .read_until(b'\n', &mut line)
        .await?;
    if line.is_empty() {
        return Ok(None);
    }
    anyhow::ensure!(
        line.len() <= 1024 && line.last() == Some(&b'\n'),
        "incomplete or oversized screen helper reply"
    );
    line.pop();
    anyhow::ensure!(
        line.iter().all(|byte| (0x20..=0x7e).contains(byte)),
        "non-ASCII screen helper reply"
    );
    match line.as_slice() {
        b"READY" => Ok(Some(Ack::Ready)),
        b"FRAME_OK" => Ok(Some(Ack::Frame)),
        b"STOPPED" => Ok(Some(Ack::Stopped)),
        error if error.starts_with(b"ERROR ") && error.len() > 6 => {
            anyhow::bail!("screen helper: {}", String::from_utf8_lossy(&error[6..]))
        }
        _ => anyhow::bail!("unexpected screen helper reply"),
    }
}

impl Pipes {
    pub(super) async fn reply(&mut self, expected: Ack) -> anyhow::Result<()> {
        let reply = self
            .replies
            .recv()
            .await
            .context("screen helper stdout disconnected")??;
        anyhow::ensure!(
            reply == Some(expected),
            "unexpected screen helper acknowledgement"
        );
        Ok(())
    }

    pub(super) async fn finish(&mut self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.replies
                .recv()
                .await
                .context("screen helper stdout disconnected before EOF")??
                .is_none(),
            "unsolicited screen helper acknowledgement after STOPPED"
        );
        Ok(())
    }

    pub(super) fn start(input: ChildStdin, output: ChildStdout, mut errors: ChildStderr) -> Self {
        let (acknowledgements, replies) = channel(8);
        let mut drains = JoinSet::new();
        drains.spawn(async move {
            let mut reader = BufReader::new(output);
            loop {
                let reply = read_reply(&mut reader).await;
                let ended = !matches!(reply, Ok(Some(_)));
                // A flooding peer cannot hold either drainer hostage.
                if acknowledgements.try_send(reply).is_err() || ended {
                    return Ok(());
                }
            }
        });
        drains.spawn(async move {
            tokio::io::copy(&mut errors, &mut tokio::io::sink())
                .await
                .context("drain screen helper stderr")?;
            Ok(())
        });
        Self {
            input: Some(input),
            replies,
            drains,
        }
    }

    pub(super) fn poll_drains(&mut self) -> anyhow::Result<()> {
        while let Some(result) = self.drains.try_join_next() {
            result.context("screen helper drainer task")??;
        }
        Ok(())
    }
}
