//! Bounded passive screen and motor diagnostics.
//!
//! Start before opening devices, activate at most once per process, enter one
//! producer per role, and finish only after all producer guards are dropped.

mod capture;
mod export;
mod producer;
mod types;

pub use capture::Capture;
pub use producer::{Guard, enter, record};
pub use types::{Case, Kind, Role};

use std::sync::OnceLock;

static ACTIVE: OnceLock<Capture> = OnceLock::new();

/// Whether this thread has a capture or one is active process-wide.
#[must_use]
pub fn enabled() -> bool {
    producer::with_inner(|_| ()).is_some() || ACTIVE.get().is_some()
}

/// The active diagnostic case, or normal operation when disabled.
#[must_use]
pub fn case() -> Case {
    producer::with_inner(|inner| inner.case)
        .or_else(|| ACTIVE.get().map(|capture| capture.inner.case))
        .unwrap_or(Case::Normal)
}

/// Microseconds since the active capture's monotonic epoch, or zero.
#[must_use]
pub fn now_us() -> u64 {
    producer::with_inner(capture::Inner::now_us)
        .or_else(|| ACTIVE.get().map(|capture| capture.inner.now_us()))
        .unwrap_or(0)
}

/// Remember the first stop request independently of event-loop processing.
pub fn request_stop() {
    if producer::with_inner(|inner| inner.stop_at(inner.now_us())).is_none()
        && let Some(capture) = ACTIVE.get()
    {
        capture.inner.stop_at(capture.inner.now_us());
    }
}

#[cfg(test)]
mod tests;
