use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{Context, Result};

use super::capture::Inner;
use super::types::Event;
use super::{ACTIVE, Capture, Kind, Role};

thread_local! {
    static PRODUCER: RefCell<Option<Producer>> = const { RefCell::new(None) };
}

pub(super) fn with_inner<T>(f: impl FnOnce(&Inner) -> T) -> Option<T> {
    PRODUCER.with(|tls| {
        let producer = tls.borrow();
        producer.as_ref()?.inner.as_deref().map(f)
    })
}

pub(super) struct Producer {
    pub role: Role,
    pub events: Vec<Event>,
    pub capacity: usize,
    pub losses: u64,
    pub next_seq: u64,
    pub inner: Option<Arc<Inner>>,
}

impl Producer {
    pub fn new(role: Role, bytes: usize) -> Result<Self> {
        let capacity = bytes / std::mem::size_of::<Event>();
        let mut events = Vec::new();
        events
            .try_reserve_exact(capacity)
            .context("reserve diagnostic event storage")?;
        Ok(Self {
            role,
            events,
            capacity,
            losses: 0,
            next_seq: 1,
            inner: None,
        })
    }
}

/// Thread-bound producer claim; dropping it returns its buffer to the capture.
#[must_use]
pub struct Guard {
    claimed: bool,
    _thread_bound: PhantomData<Rc<()>>,
}

pub(super) fn claim(capture: &Capture, role: Role) -> Guard {
    let mut state = capture
        .inner
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let claimed = PRODUCER.with(|tls| {
        let mut tls = tls.borrow_mut();
        if state.finished || tls.is_some() || state.slots[role.index()].is_none() {
            state.incomplete = true;
            return false;
        }
        if let Some(mut producer) = state.slots[role.index()].take() {
            producer.inner = Some(Arc::clone(&capture.inner));
            *tls = Some(producer);
            true
        } else {
            state.incomplete = true;
            false
        }
    });
    Guard {
        claimed,
        _thread_bound: PhantomData,
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if !self.claimed {
            return;
        }
        PRODUCER.with(|tls| {
            if let Some(mut producer) = tls.borrow_mut().take() {
                if let Some(inner) = producer.inner.take() {
                    let mut state = inner
                        .state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let index = producer.role.index();
                    state.slots[index] = Some(producer);
                }
            }
        });
    }
}

/// Enter the active capture, or return a disabled guard.
pub fn enter(role: Role) -> Guard {
    ACTIVE.get().map_or_else(
        || Guard {
            claimed: false,
            _thread_bound: PhantomData,
        },
        |capture| capture.enter(role),
    )
}

/// Record a bounded event in the current thread, returning its sequence.
///
/// Returns zero if no producer is installed or the event was lost.
#[expect(
    clippy::must_use_candidate,
    reason = "most events need no correlation ID"
)]
pub fn record(kind: Kind, values: [u64; 8], payload: &[u8]) -> u64 {
    PRODUCER.with(|tls| {
        let mut tls = tls.borrow_mut();
        let Some(producer) = tls.as_mut() else {
            return 0;
        };
        if payload.len() > 95 || producer.events.len() == producer.capacity {
            producer.losses = producer.losses.saturating_add(1);
            return 0;
        }
        let Some(inner) = producer.inner.as_ref() else {
            return 0;
        };
        let seq = producer.next_seq;
        producer.next_seq = producer.next_seq.saturating_add(1);
        let mut bytes = [0; 95];
        bytes[..payload.len()].copy_from_slice(payload);
        let Ok(payload_len) = u8::try_from(payload.len()) else {
            return 0;
        };
        producer.events.push(Event {
            us: inner.now_us(),
            seq,
            kind,
            values,
            payload: bytes,
            payload_len,
        });
        seq
    })
}
