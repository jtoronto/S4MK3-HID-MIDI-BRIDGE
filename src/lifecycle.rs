//! Managed child protocol. Readiness follows actual handles and initialization.

use std::io::Write as _;
use std::sync::{Arc, Mutex};

use anyhow::Context as _;
use crossbeam_channel::{Receiver, Sender};
use serde::Serialize;

use crate::{Args, MidiProfile, full, midi_transport::Backend};

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum State {
    Starting,
    Ready,
    Stopping,
    Failed,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum Feedback {
    Unknown,
    Received,
    NotApplicable,
}

#[derive(Serialize)]
pub struct Readiness {
    pub hid: bool,
    pub midi_input: Option<bool>,
    pub midi_output: Option<bool>,
    pub controller_initialized: bool,
    feedback: Feedback,
}

#[derive(Serialize)]
struct Failure {
    stage: &'static str,
    message: String,
}

#[derive(Serialize)]
struct Status {
    version: u8,
    state: State,
    profile: &'static str,
    backend: Backend,
    request_id: Option<String>,
    readiness: Readiness,
    error: Option<Failure>,
    #[serde(skip)]
    started: bool,
    #[serde(skip)]
    stop_requested: bool,
}

impl Status {
    fn emit(&self) -> anyhow::Result<()> {
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "S4_BRIDGE_STATUS {}", serde_json::to_string(self)?)?;
        stdout.flush()?;
        Ok(())
    }

    fn request_stop(&mut self, id: Option<u64>) -> anyhow::Result<bool> {
        if self.stop_requested || self.state == State::Failed {
            return Ok(false);
        }
        self.stop_requested = true;
        self.request_id = id.map(|id| id.to_string());
        if self.started {
            self.state = State::Stopping;
            self.emit()?;
        }
        Ok(true)
    }
}

#[derive(Clone)]
pub struct Stop {
    shared: Arc<Mutex<Status>>,
    sender: Sender<()>,
}

impl Stop {
    pub fn request(&self, id: Option<u64>) -> anyhow::Result<()> {
        s4mk3_hid_midi_bridge::screen_motor_diag::request_stop();
        let mut status = self
            .shared
            .lock()
            .map_err(|error| anyhow::anyhow!("lifecycle lock: {error}"))?;
        let requested = status.request_stop(id)?;
        drop(status);
        if requested {
            match self.sender.try_send(()) {
                Ok(()) | Err(crossbeam_channel::TrySendError::Full(())) => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
}

pub struct Session {
    pub stop: Stop,
    pub receiver: Receiver<()>,
    pub screens: Option<full::screens::Output>,
    pub led_config: Option<full::leds::LedConfig>,
    pub feedback_input: Option<full::feedback::FeedbackInput>,
}

fn feedback_before_controller(profile: Option<MidiProfile>, backend: Backend) -> bool {
    matches!(profile, Some(MidiProfile::Full)) && backend == Backend::Bloop
}

impl Session {
    pub fn new(args: &Args) -> anyhow::Result<Self> {
        let full = matches!(args.midi, Some(MidiProfile::Full));
        // Djay can send its initial state as soon as the host publishes BLOOP.
        let feedback_input = feedback_before_controller(args.midi, args.midi_backend)
            .then(|| {
                full::feedback::FeedbackInput::with_backend(
                    full::mapping::PORT_NAME,
                    args.midi_backend,
                    args.capture_handle.clone(),
                )
            })
            .transpose()?;
        let led_config = full
            .then(|| full::leds::LedConfig::load(args.led_config.as_deref()))
            .transpose()?;
        let screens = full
            .then(|| {
                #[cfg(windows)]
                if args.screens.ni_decks {
                    return full::screens::Output::ni_decks();
                }
                if args.screens.control_stdin || (!cfg!(windows) && args.managed.stdin) {
                    full::screens::Output::managed(args.screens.enabled)
                } else {
                    Ok(full::screens::Output::new(args.screens.enabled))
                }
            })
            .transpose()?;
        let (sender, receiver) = crossbeam_channel::bounded(1);
        let shared = Arc::new(Mutex::new(Status {
            version: 1,
            state: State::Starting,
            profile: match args.midi {
                Some(MidiProfile::Full) => "full",
                Some(MidiProfile::Minimal) => "minimal",
                None => "probe",
            },
            backend: args.midi_backend,
            request_id: None,
            readiness: Readiness {
                hid: false,
                midi_input: full.then_some(false),
                midi_output: args.midi.map(|_| false),
                controller_initialized: false,
                feedback: if full {
                    Feedback::Unknown
                } else {
                    Feedback::NotApplicable
                },
            },
            error: None,
            started: false,
            stop_requested: false,
        }));
        let stop = Stop { shared, sender };
        let interrupt = stop.clone();
        ctrlc::set_handler(move || {
            if let Err(error) = interrupt.request(None) {
                tracing::warn!(%error, "signal stop failed");
            }
        })
        .context("install Ctrl-C handler")?;
        if args.managed.stdin || args.screens.control_stdin {
            crate::screens_control::start(
                screens
                    .as_ref()
                    .filter(|_| !args.screens.ni_decks)
                    .and_then(full::screens::Output::control),
                screens
                    .as_ref()
                    .filter(|_| args.screens.ni_decks && args.managed.stdin)
                    .and_then(full::screens::Output::control),
                stop.clone(),
                args.capture_handle.clone(),
            )?;
        }
        {
            let mut status = stop
                .shared
                .lock()
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            status.emit()?;
            status.started = true;
            if status.stop_requested {
                status.state = State::Stopping;
                status.emit()?;
            }
        }
        Ok(Self {
            stop,
            receiver,
            screens,
            led_config,
            feedback_input,
        })
    }

    pub fn milestone(&self, update: impl FnOnce(&mut Readiness)) -> anyhow::Result<()> {
        let mut status = self
            .stop
            .shared
            .lock()
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        update(&mut status.readiness);
        drop(status);
        Ok(())
    }

    pub fn midi_ready(&self) -> anyhow::Result<()> {
        self.milestone(|ready| {
            ready.midi_input = ready.midi_input.map(|_| true);
            ready.midi_output = ready.midi_output.map(|_| true);
        })
    }

    pub fn ready(&self) -> anyhow::Result<()> {
        let mut status = self
            .stop
            .shared
            .lock()
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        if status.state != State::Starting {
            return Ok(());
        }
        if !status.stop_requested {
            anyhow::ensure!(
                status.readiness.hid
                    && status.readiness.midi_input != Some(false)
                    && status.readiness.midi_output != Some(false),
                "incomplete readiness"
            );
            status.readiness.controller_initialized = true;
            status.state = State::Ready;
            status.emit()?;
        }
        drop(status);
        Ok(())
    }

    pub fn feedback(&self, received: bool) -> anyhow::Result<()> {
        if received {
            let mut status = self
                .stop
                .shared
                .lock()
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            if matches!(status.readiness.feedback, Feedback::Unknown) {
                status.readiness.feedback = Feedback::Received;
                if status.state == State::Ready {
                    status.emit()?;
                }
            }
        }
        Ok(())
    }

    pub fn finish(&self, result: anyhow::Result<()>) -> anyhow::Result<()> {
        match result {
            Ok(()) => self.stop.request(None),
            Err(error) => {
                let mut status = self
                    .stop
                    .shared
                    .lock()
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
                status.state = State::Failed;
                status.request_id = None;
                status.error = Some(Failure {
                    stage: "runtime",
                    message: format!("{error:#}"),
                });
                status.emit()?;
                drop(status);
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests;
