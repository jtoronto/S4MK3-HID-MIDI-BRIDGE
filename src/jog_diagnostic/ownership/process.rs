//! A deadline belongs to the inventory child and both of its output pipes.

use std::io;
use std::process::{Child, Output};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail, ensure};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};

const POLL: Duration = Duration::from_millis(10);
const DRAIN_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Copy)]
enum Pipe {
    Stdout,
    Stderr,
}

type Packet = (Pipe, io::Result<Vec<u8>>);

struct Drains {
    sender: Sender<Packet>,
    receiver: Receiver<Packet>,
    readers: Vec<JoinHandle<()>>,
    received: usize,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Drains {
    fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(2);
        Self {
            sender,
            receiver,
            readers: Vec::with_capacity(2),
            received: 0,
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    fn start(&mut self, reader: impl io::Read + Send + 'static, pipe: Pipe) -> anyhow::Result<()> {
        let sender = self.sender.clone();
        self.readers.push(
            std::thread::Builder::new()
                .name("s4-inventory-output".to_owned())
                .spawn(move || {
                    let mut reader = reader;
                    let mut bytes = Vec::new();
                    let result = reader.read_to_end(&mut bytes).map(|_| bytes);
                    if let Err(error) = sender.send((pipe, result)) {
                        tracing::debug!(%error, "inventory pipe receiver closed after deadline");
                    }
                })
                .context("start inventory pipe drain")?,
        );
        Ok(())
    }

    fn receive(&mut self, duration: Duration) -> anyhow::Result<()> {
        match self.receiver.recv_timeout(duration) {
            Ok((pipe, result)) => {
                self.received += 1;
                let bytes = result.context("read inventory output pipe")?;
                match pipe {
                    Pipe::Stdout => self.stdout = bytes,
                    Pipe::Stderr => self.stderr = bytes,
                }
                Ok(())
            }
            Err(RecvTimeoutError::Timeout) => Ok(()),
            Err(error @ RecvTimeoutError::Disconnected) => Err(error.into()),
        }
    }

    fn finish(&mut self) -> anyhow::Result<()> {
        let deadline = Instant::now() + DRAIN_TIMEOUT;
        while self.received < self.readers.len() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            ensure!(
                !remaining.is_zero(),
                "inventory output drain cleanup timed out"
            );
            self.receive(remaining)?;
        }
        // A completion packet is sent only after read_to_end has finished.
        // Never join a reader still waiting on an inherited pipe.
        for reader in self.readers.drain(..) {
            reader
                .join()
                .map_err(|_| anyhow::anyhow!("inventory pipe reader panicked"))?;
        }
        Ok(())
    }
}

pub(super) fn capture(child: &mut Child, timeout: Duration) -> anyhow::Result<Output> {
    let id = child.id();
    let deadline = Instant::now() + timeout;
    let mut drains = Drains::new();
    let result = (|| {
        drains.start(
            child
                .stdout
                .take()
                .context("inventory stdout pipe missing")?,
            Pipe::Stdout,
        )?;
        drains.start(
            child
                .stderr
                .take()
                .context("inventory stderr pipe missing")?,
            Pipe::Stderr,
        )?;
        loop {
            let status = child.try_wait().context("poll inventory child")?;
            if let Some(status) = status
                && drains.received == 2
            {
                ensure!(
                    status.success(),
                    "inventory child {id} exited with {status}: {}",
                    String::from_utf8_lossy(&drains.stderr)
                );
                return Ok(Output {
                    status,
                    stdout: std::mem::take(&mut drains.stdout),
                    stderr: std::mem::take(&mut drains.stderr),
                });
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            ensure!(
                !remaining.is_zero(),
                "inventory child {id} timed out after {timeout:?}"
            );
            drains.receive(remaining.min(POLL))?;
        }
    })();
    let result = if result.is_err() {
        with_cleanup(
            result,
            terminate(child),
            "inventory child termination/reaping",
        )
    } else {
        result
    };
    with_cleanup(result, drains.finish(), "inventory output drain")
}

fn terminate(child: &mut Child) -> anyhow::Result<()> {
    if child
        .try_wait()
        .context("inspect owned inventory child")?
        .is_none()
    {
        if let Err(error) = child.kill()
            && child
                .try_wait()
                .context("inspect inventory child after kill failure")?
                .is_none()
        {
            return Err(error).context("terminate owned inventory child");
        }
        child.wait().context("reap owned inventory child")?;
    }
    Ok(())
}

fn with_cleanup<T>(
    result: anyhow::Result<T>,
    cleanup: anyhow::Result<()>,
    operation: &str,
) -> anyhow::Result<T> {
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error).with_context(|| operation.to_owned()),
        (Err(error), Err(cleanup)) => {
            bail!(error.context(format!("{operation} also failed: {cleanup:#}")))
        }
    }
}

#[cfg(test)]
#[path = "process/tests.rs"]
mod tests;
