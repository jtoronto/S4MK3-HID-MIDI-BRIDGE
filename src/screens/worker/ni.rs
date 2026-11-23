//! NI ownership stays on the screen worker; no image is written on release.

use super::*;
use crate::screens::ni_service::{Cancellation, Identity};
use std::time::{Duration, Instant};
use tokio::sync::watch;

const CONTROL_POLL: Duration = Duration::from_millis(50);
pub(super) use lifecycle::Presentation;

mod ipc;
mod lifecycle;
#[cfg(windows)]
pub(super) use ipc::Supervisor as Native;
pub(super) use lifecycle::run;

pub(super) trait Transport {
    fn publish(&mut self, frames: &[Frame; 2]) -> anyhow::Result<()>;
    fn poll_control(&mut self) -> anyhow::Result<()>;
    fn close(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

fn emit(shared: &Shared, status: &impl Fn(u64, State), owner: (u64, u64), state: State) {
    if shared.0.lock().is_ok_and(|mailbox| {
        !mailbox.stop
            && mailbox
                .latest
                .is_some_and(|request| request.generation == owner.0)
    }) {
        status(owner.1, state);
    }
}

fn interrupted(shared: &Shared, generation: u64) -> anyhow::Result<bool> {
    let mailbox = shared
        .0
        .lock()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    Ok(mailbox.stop
        || mailbox
            .latest
            .is_some_and(|request| request.generation != generation))
}

fn work(
    shared: &Shared,
    current: &mut Option<[page::Page; 2]>,
    generation: u64,
    polling: bool,
    next_flash: Option<Instant>,
) -> anyhow::Result<(Option<Request>, bool)> {
    let mut mailbox = shared
        .0
        .lock()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    let deadline = polling.then(|| {
        let poll = Instant::now() + CONTROL_POLL;
        next_flash.map_or(poll, |flash| flash.min(poll))
    });
    while mailbox.pending.is_none()
        && mailbox
            .latest
            .is_none_or(|request| request.generation == generation)
        && !mailbox.stop
    {
        if let Some(deadline) = deadline {
            if Instant::now() >= deadline {
                break;
            }
            mailbox = shared
                .1
                .wait_timeout(mailbox, deadline.saturating_duration_since(Instant::now()))
                .map_err(|error| anyhow::anyhow!("{error}"))?
                .0;
        } else {
            mailbox = shared
                .1
                .wait(mailbox)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
        }
    }
    *current = mailbox.pending.take().or(*current);
    let request = mailbox
        .latest
        .filter(|request| request.generation != generation);
    mailbox.release_required = false;
    Ok((request, mailbox.stop))
}

fn acquire(shared: &Shared, generation: u64) -> anyhow::Result<Option<(Identity, Cancellation)>> {
    let mut mailbox = shared
        .0
        .lock()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    if mailbox.stop
        || mailbox
            .latest
            .is_some_and(|request| request.generation != generation)
    {
        return Ok(None);
    }
    let identity = mailbox
        .identity
        .clone()
        .context("selected HID serial not bound to Windows screens")?;
    let (sender, receiver) = watch::channel(false);
    mailbox.cancellation = Some(sender);
    drop(mailbox);
    Ok(Some((identity, receiver)))
}

fn log_render(owner: (u64, u64), rendered: [(page::Page, bool); 2], frames: &[Frame; 2]) {
    if tracing::enabled!(target: "s4_screen_route_diag", tracing::Level::DEBUG) {
        tracing::debug!(
            target: "s4_screen_route_diag",
            generation = owner.0,
            pages = ?rendered.map(|(page, _)| page),
            bright = ?rendered.map(|(_, bright)| bright),
            hashes = ?frames.each_ref().map(|frame| crate::screens::route_fingerprint(&frame.pixels)),
            "SCREEN_ROUTE_RENDER"
        );
    }
}

#[cfg(test)]
mod diagnostic_failures;
#[cfg(test)]
mod tests;
