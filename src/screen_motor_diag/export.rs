use std::io::{BufWriter, Write};
use std::sync::atomic::Ordering;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use super::producer::Producer;
use super::types::Event;
use super::{Capture, Case, Kind, Role};

#[derive(Serialize)]
struct Header {
    kind: &'static str,
    case: Case,
    version: u8,
    utc_unix_us: u64,
}

#[derive(Serialize)]
struct Row<'a> {
    us: u64,
    role: Role,
    seq: u64,
    kind: Kind,
    values: [u64; 8],
    payload: &'a [u8],
}

#[derive(Serialize)]
struct RoleSummary {
    role: Role,
    count: Option<usize>,
    capacity: usize,
    losses: Option<u64>,
}

#[derive(Serialize)]
struct Footer {
    kind: &'static str,
    runtime_ok: bool,
    incomplete: bool,
    first_stop_us: Option<u64>,
    roles: [RoleSummary; 3],
}

fn line(writer: &mut impl Write, value: &impl Serialize) -> Result<()> {
    serde_json::to_writer(&mut *writer, value).context("serialize diagnostic JSONL")?;
    writer.write_all(b"\n").context("write diagnostic JSONL")
}

fn summary(role: Role, producer: Option<&Producer>, bytes: usize) -> RoleSummary {
    RoleSummary {
        role,
        count: producer.map(|item| item.events.len()),
        capacity: bytes / std::mem::size_of::<Event>(),
        losses: producer.map(|item| item.losses),
    }
}

pub(super) fn write(capture: &Capture, runtime_ok: bool) -> Result<()> {
    let mut state = capture
        .inner
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.finished {
        bail!("diagnostic capture already finished");
    }
    state.finished = true;
    let Some(file) = state.file.take() else {
        bail!("diagnostic output unavailable")
    };
    let mut writer = BufWriter::new(file);
    line(
        &mut writer,
        &Header {
            kind: "header",
            case: capture.inner.case,
            version: 1,
            utc_unix_us: capture.inner.utc_unix_us,
        },
    )?;
    for producer in state.slots.iter().flatten() {
        for event in &producer.events {
            line(
                &mut writer,
                &Row {
                    us: event.us,
                    role: producer.role,
                    seq: event.seq,
                    kind: event.kind,
                    values: event.values,
                    payload: &event.payload[..usize::from(event.payload_len)],
                },
            )?;
        }
    }
    let stop = capture.inner.stop.load(Ordering::SeqCst);
    let incomplete = state.incomplete
        || state.slots.iter().any(Option::is_none)
        || state
            .slots
            .iter()
            .flatten()
            .any(|producer| producer.losses != 0);
    line(
        &mut writer,
        &Footer {
            kind: "footer",
            runtime_ok,
            incomplete,
            first_stop_us: stop.checked_sub(1),
            roles: [
                summary(Role::Output, state.slots[0].as_ref(), 24 * 1024 * 1024),
                summary(Role::Input, state.slots[1].as_ref(), 32 * 1024 * 1024),
                summary(Role::Screen, state.slots[2].as_ref(), 8 * 1024 * 1024),
            ],
        },
    )?;
    drop(state);
    writer.flush().context("flush diagnostic JSONL")
}
