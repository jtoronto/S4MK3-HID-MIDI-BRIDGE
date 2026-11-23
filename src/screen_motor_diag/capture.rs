use std::fs::{File, OpenOptions};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use super::producer::{Guard, Producer};
use super::{ACTIVE, Case, Role};

const MIB: usize = 1024 * 1024;

/// A reserved capture and its preallocated per-role buffers.
#[derive(Clone)]
pub struct Capture {
    pub(super) inner: Arc<Inner>,
}

pub(super) struct Inner {
    pub case: Case,
    pub epoch: Instant,
    pub utc_unix_us: u64,
    pub stop: AtomicU64,
    pub state: Mutex<State>,
}

pub(super) struct State {
    pub file: Option<File>,
    pub slots: [Option<Producer>; 3],
    pub incomplete: bool,
    pub finished: bool,
}

impl Capture {
    /// Reserve a new absolute output path and allocate all producer storage.
    ///
    /// # Errors
    /// Returns an error if the path is relative, cannot be created exclusively,
    /// or the event storage cannot be reserved.
    pub fn start(path: &Path, case: Case) -> Result<Self> {
        if !path.is_absolute() {
            bail!("diagnostic output path must be absolute");
        }
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!("reserve diagnostic output {}", path.display()))?;
        let slots = [
            Some(Producer::new(Role::Output, 24 * MIB)?),
            Some(Producer::new(Role::Input, 32 * MIB)?),
            Some(Producer::new(Role::Screen, 8 * MIB)?),
        ];
        let epoch = Instant::now();
        let utc_unix_us = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("system clock precedes Unix epoch")?
                .as_micros(),
        )
        .context("UTC clock exceeds diagnostic timestamp range")?;
        Ok(Self {
            inner: Arc::new(Inner {
                case,
                epoch,
                utc_unix_us,
                stop: AtomicU64::new(0),
                state: Mutex::new(State {
                    file: Some(file),
                    slots,
                    incomplete: false,
                    finished: false,
                }),
            }),
        })
    }

    /// Install this capture as the process-wide capture, exactly once.
    ///
    /// # Errors
    /// Returns an error if a capture was already activated.
    pub fn activate(&self) -> Result<()> {
        ACTIVE
            .set(self.clone())
            .map_err(|_| anyhow::anyhow!("diagnostic capture already activated"))
    }

    /// Claim this capture's role for the current thread until the guard drops.
    pub fn enter(&self, role: Role) -> Guard {
        super::producer::claim(self, role)
    }

    /// Export the capture after all producer guards and threads have stopped.
    ///
    /// # Errors
    /// Returns an error if already finished or if writing or flushing fails.
    /// A failed export leaves the reserved partial file in place.
    pub fn finish(&self, runtime_ok: bool) -> Result<()> {
        super::export::write(self, runtime_ok)
    }
}

impl Inner {
    pub fn now_us(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_micros()).unwrap_or(u64::MAX - 1)
    }

    pub fn stop_at(&self, us: u64) {
        let _previous =
            self.stop
                .compare_exchange(0, us.saturating_add(1), Ordering::SeqCst, Ordering::SeqCst);
    }
}
