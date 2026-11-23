//! Latest-state mailbox and screen ownership stay off HID/MIDI/motor service.

use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use anyhow::Context as _;

#[cfg(any(not(windows), test))]
use super::Display;
#[cfg(not(windows))]
use super::UsbScreens;
use super::{Frame, page};

#[cfg(any(not(windows), test))]
mod service;
#[cfg(any(not(windows), test))]
use service::run;
#[cfg(any(windows, test))]
mod ni;
#[cfg(any(windows, test))]
mod ni_decks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    On,
    Off,
    Retry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Active,
    Inactive,
    Failed,
    Pending,
}

impl State {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Failed => "failed",
            Self::Pending => "pending",
        }
    }
}

#[derive(Clone, Copy)]
struct Request {
    id: u64,
    generation: u64,
    action: Action,
}

#[derive(Default)]
struct Mailbox {
    pending: Option<[page::Page; 2]>,
    latest: Option<Request>,
    release_required: bool,
    initialized: bool,
    stop: bool,
    #[cfg(any(windows, test))]
    identity: Option<super::ni_service::Identity>,
    #[cfg(any(windows, test))]
    cancellation: Option<tokio::sync::watch::Sender<bool>>,
    #[cfg(any(windows, test))]
    decks: Option<ni_decks::DeckMailbox>,
}

type Shared = Arc<(Mutex<Mailbox>, Condvar)>;

#[derive(Clone)]
pub struct Control {
    shared: Shared,
}

impl Control {
    pub fn request(&self, id: u64, action: Action) {
        match self.shared.0.lock() {
            Ok(mut mailbox) => {
                #[cfg(any(windows, test))]
                if mailbox.decks.is_some() {
                    return;
                }
                if mailbox.stop {
                    return;
                }
                let generation = mailbox
                    .latest
                    .map_or(1, |old| old.generation.wrapping_add(1));
                mailbox.latest = Some(Request {
                    id,
                    generation,
                    action,
                });
                #[cfg(any(windows, test))]
                if let Some(cancellation) = &mailbox.cancellation {
                    cancellation.send_replace(true);
                }
                if matches!(action, Action::Off | Action::Retry) {
                    mailbox.release_required = true;
                }
                self.shared.1.notify_one();
            }
            Err(error) => tracing::warn!(%error, "could not request S4 screen control"),
        }
    }

    /// Bind the serial from the uniquely selected HID collection before input.
    ///
    /// # Errors
    /// Rejects an invalid serial, rebinding, or binding after initialized input.
    #[cfg(windows)]
    pub fn bind_selected_hid(&self, serial: &str) -> anyhow::Result<()> {
        self.bind_identity(super::ni_service::Identity::selected_hid(serial)?)
    }

    #[cfg(any(windows, test))]
    fn bind_identity(&self, identity: super::ni_service::Identity) -> anyhow::Result<()> {
        let mut mailbox = self
            .shared
            .0
            .lock()
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        anyhow::ensure!(
            !mailbox.initialized && !mailbox.stop,
            "screen identity must precede initialized input"
        );
        anyhow::ensure!(
            mailbox
                .identity
                .as_ref()
                .is_none_or(|old| old.serial() == identity.serial()),
            "screen HID serial cannot be rebound"
        );
        mailbox.identity = Some(identity);
        drop(mailbox);
        self.shared.1.notify_one();
        Ok(())
    }
}

pub struct Worker {
    shared: Shared,
    thread: Option<JoinHandle<()>>,
    last: Option<[page::Page; 2]>,
    #[cfg(any(windows, test))]
    deck_completion: Option<crossbeam_channel::Receiver<()>>,
}

impl Worker {
    /// Start asynchronous screen service; USB ownership remains on its thread.
    ///
    /// # Errors
    /// Returns an error if the worker thread cannot be created.
    pub fn start() -> anyhow::Result<Self> {
        #[cfg(windows)]
        {
            Self::start_windows(true)
        }
        #[cfg(not(windows))]
        Self::start_with(|| {
            let mut usb = UsbScreens::open()?;
            Ok(move |screen, frame: &Frame| usb.write(screen, frame))
        })
    }

    /// Start a controllable screen service without claiming USB before HID input.
    ///
    /// # Errors
    /// Returns an error if the worker thread cannot be created.
    pub fn start_managed(enabled: bool) -> anyhow::Result<Self> {
        #[cfg(windows)]
        {
            Self::start_windows(enabled)
        }
        #[cfg(not(windows))]
        {
            let worker = Self::start_with_config(
                || {
                    let mut usb = UsbScreens::open()?;
                    Ok(move |screen, frame: &Frame| usb.write(screen, frame))
                },
                enabled,
                Some(|id, state: State| println!("S4_SCREEN_STATUS {id} {}", state.label())),
            )?;
            println!(
                "S4_SCREEN_STATUS 0 {}",
                if enabled { "pending" } else { "inactive" }
            );
            Ok(worker)
        }
    }

    #[cfg(any(not(windows), test))]
    fn start_with<F, W>(open: F) -> anyhow::Result<Self>
    where
        F: FnOnce() -> anyhow::Result<W> + Send + 'static,
        W: FnMut(Display, &Frame) -> anyhow::Result<()> + Send + 'static,
    {
        let mut open = Some(open);
        Self::start_with_config(
            move || open.take().context("screen worker already opened")?(),
            true,
            None::<fn(u64, State)>,
        )
    }

    #[cfg(any(not(windows), test))]
    fn start_with_config<F, W, S>(open: F, enabled: bool, status: Option<S>) -> anyhow::Result<Self>
    where
        F: FnMut() -> anyhow::Result<W> + Send + 'static,
        W: FnMut(Display, &Frame) -> anyhow::Result<()> + Send + 'static,
        S: Fn(u64, State) + Send + 'static,
    {
        let mut mailbox = Mailbox::default();
        if status.is_some() {
            mailbox.latest = Some(Request {
                id: 0,
                generation: 0,
                action: if enabled { Action::On } else { Action::Off },
            });
        }
        Self::spawn(mailbox, move |shared| {
            run(shared, open, enabled, status.as_ref())
        })
    }

    #[cfg(windows)]
    fn start_windows(enabled: bool) -> anyhow::Result<Self> {
        println!(
            "S4_SCREEN_STATUS 0 {}",
            if enabled { "pending" } else { "inactive" }
        );
        Self::start_ni(ni::Native::open, enabled, |id, state| {
            println!("S4_SCREEN_STATUS {id} {}", state.label());
        })
    }

    #[cfg(any(windows, test))]
    fn start_ni<F, T, S>(open: F, enabled: bool, status: S) -> anyhow::Result<Self>
    where
        F: FnMut(super::ni_service::Identity, super::ni_service::Cancellation) -> anyhow::Result<T>
            + Send
            + 'static,
        T: ni::Transport + Send + 'static,
        S: Fn(u64, State) + Send + 'static,
    {
        Self::spawn(
            Mailbox {
                latest: Some(Request {
                    id: 0,
                    generation: 0,
                    action: if enabled { Action::On } else { Action::Off },
                }),
                ..Mailbox::default()
            },
            move |shared| ni::run(shared, open, enabled, &status),
        )
    }

    fn spawn(
        mailbox: Mailbox,
        run: impl FnOnce(&Shared) -> anyhow::Result<()> + Send + 'static,
    ) -> anyhow::Result<Self> {
        let shared = Arc::new((Mutex::new(mailbox), Condvar::new()));
        let task_shared = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name(String::from("s4-screens"))
            .spawn(move || {
                if let Err(error) = run(&task_shared) {
                    tracing::warn!(%error, "SCREENS_DISABLED: MIDI, LEDs and tension remain active");
                }
                match task_shared.0.lock() {
                    Ok(mut mailbox) => {
                        mailbox.stop = true;
                        task_shared.1.notify_all();
                    }
                    Err(error) => tracing::warn!(%error, "screen mailbox unavailable at shutdown"),
                }
            })
            .context("start S4 screen worker")?;
        Ok(Self {
            shared,
            thread: Some(thread),
            last: None,
            #[cfg(any(windows, test))]
            deck_completion: None,
        })
    }

    #[must_use]
    pub fn control(&self) -> Control {
        Control {
            shared: Arc::clone(&self.shared),
        }
    }

    pub fn publish(&mut self, pages: [page::Page; 2]) {
        if self.last == Some(pages) {
            return;
        }
        self.last = Some(pages);
        match self.shared.0.lock() {
            Ok(mut mailbox) => {
                if !mailbox.stop {
                    mailbox.pending = Some(pages);
                    mailbox.initialized = true;
                    self.shared.1.notify_one();
                }
            }
            Err(error) => tracing::warn!(%error, "could not publish S4 screen state"),
        }
    }

    pub fn refresh(&mut self, pages: [page::Page; 2]) {
        if self
            .shared
            .0
            .lock()
            .is_ok_and(|mailbox| mailbox.initialized)
        {
            self.publish(pages);
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        #[cfg(any(windows, test))]
        if self.deck_completion.is_some() {
            if self.thread.is_some()
                && let Err(error) = self.stop_ni_decks()
            {
                tracing::warn!(%error, "NI_DECK_STOP_UNCONFIRMED");
            }
            return;
        }
        match self.shared.0.lock() {
            Ok(mut mailbox) => {
                mailbox.stop = true;
                #[cfg(any(windows, test))]
                if let Some(cancellation) = &mailbox.cancellation {
                    cancellation.send_replace(true);
                }
                mailbox.pending = None;
                self.shared.1.notify_one();
            }
            Err(error) => tracing::warn!(%error, "could not stop S4 screen worker"),
        }
        // Full bridge shutdown has already neutralized motors. Live commands never join here.
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            tracing::warn!("S4 screen worker panicked");
        }
    }
}

#[cfg(test)]
mod tests;
