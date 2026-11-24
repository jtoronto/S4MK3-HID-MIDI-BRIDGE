use std::io::Read;
use std::path::PathBuf;

use anyhow::Context as _;
use tokio::runtime::Runtime;

use super::{DeckOutput, Shared, Transport, lock, shutdown};

pub(super) const ENV: &str = "S4_BRIDGE_NI_JOGT_TRIAL_FILE";
pub(super) const BASELINE: u8 = 107;

pub(super) fn inhibit_drive(desired: [DeckOutput; 2]) -> [DeckOutput; 2] {
    desired.map(|deck| DeckOutput {
        turntable: false,
        playing: false,
        ..deck
    })
}

pub(super) struct Trial {
    path: PathBuf,
    armed: bool,
    gate: Option<&'static str>,
    generation: Option<u64>,
    changed: bool,
}

impl Trial {
    pub(super) fn new(path: PathBuf) -> anyhow::Result<Self> {
        anyhow::ensure!(!path.as_os_str().is_empty(), "{ENV} must name a local file");
        Ok(Self {
            path,
            armed: false,
            gate: None,
            generation: None,
            changed: false,
        })
    }

    pub(super) const fn disarm(&mut self) {
        self.armed = false;
    }

    pub(super) fn update(
        &mut self,
        shared: &Shared,
        runtime: &Runtime,
        session: &mut impl Transport,
        snapshot: ([DeckOutput; 2], u64),
    ) -> anyhow::Result<bool> {
        let (desired, generation) = snapshot;
        let current = || -> anyhow::Result<bool> {
            let mailbox = lock(shared)?;
            let decks = mailbox.decks.as_ref().context("NI deck mode not enabled")?;
            Ok(!mailbox.stop && decks.desired == desired && decks.unsafe_generation == generation)
        };
        if !current()? {
            self.disarm();
            return Ok(false);
        }
        match self.command(desired, generation) {
            Ok(Some(value)) => {
                if !current()? {
                    self.disarm();
                    return Ok(false);
                }
                if let Err(error) = runtime.block_on(session.set_jog_tension(value)) {
                    return match runtime.block_on(session.quiesce()) {
                        Ok(()) => Err(error.context(
                            "NI jog tension submission failed; decks quiesced; tension unconfirmed",
                        )),
                        Err(stop_error) => Err(error.context(format!(
                            "NI jog tension submission failed; quiesce unconfirmed: {stop_error:#}"
                        ))),
                    };
                }
                self.restored(value);
                println!("NI_JOGT_TRIAL_SUBMITTED {value} (not physical feedback)");
            }
            Ok(None) => {}
            Err(error) => {
                return match shutdown(runtime, session, Some(self)) {
                    Ok(()) => {
                        Err(error.context("NI jog tension control file failed; decks quiesced"))
                    }
                    Err(stop_error) => Err(error.context(format!(
                        "NI jog tension control file failed; cleanup unconfirmed: {stop_error:#}"
                    ))),
                };
            }
        }
        Ok(true)
    }

    pub(super) fn command(
        &mut self,
        desired: [DeckOutput; 2],
        generation: u64,
    ) -> anyhow::Result<Option<u8>> {
        if self.generation != Some(generation) {
            self.armed = false;
            self.generation = Some(generation);
        }
        let gate = if desired.iter().any(|deck| deck.ring.is_none()) {
            "feedback-unavailable"
        } else if desired.iter().any(|deck| deck.playing || deck.turntable) {
            "inactive"
        } else {
            "ready"
        };
        if self.gate != Some(gate) {
            println!("NI_JOGT_TRIAL_GATE {gate}");
            self.gate = Some(gate);
        }
        let file = std::fs::File::open(&self.path)?;
        anyhow::ensure!(
            file.metadata()?.is_file(),
            "trial control must be a regular file"
        );
        let mut bytes = Vec::new();
        file.take(5).read_to_end(&mut bytes)?;
        if gate != "ready" {
            self.armed = false;
            return Ok(None);
        }
        if bytes == b"idle" {
            if !self.armed {
                println!("NI_JOGT_TRIAL_IDLE_ARMED");
            }
            self.armed = true;
            return Ok(None);
        }
        let value = match bytes.as_slice() {
            b"25" => 25,
            b"244" => 244,
            b"107" => BASELINE,
            _ => {
                self.armed = false;
                anyhow::bail!("invalid NI jog tension trial control contents");
            }
        };
        if !std::mem::take(&mut self.armed) {
            return Ok(None);
        }
        self.changed = true;
        Ok(Some(value))
    }

    pub(super) const fn restored(&mut self, value: u8) {
        self.changed = value != BASELINE;
    }

    pub(super) const fn needs_restore(&self) -> bool {
        self.changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_require_new_idle_in_fresh_paused_jog() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!("s4-jogt-trial-{}", std::process::id()));
        let mut trial = Trial::new(path.clone())?;
        let ready = [DeckOutput {
            turntable: false,
            playing: false,
            ring: Some(1),
        }; 2];
        std::fs::write(&path, b"idle")?;
        assert_eq!(trial.command(ready, 0)?, None);
        std::fs::write(&path, b"25")?;
        assert_eq!(trial.command(ready, 0)?, Some(25));
        assert_eq!(trial.command(ready, 0)?, None);
        std::fs::write(&path, b"idle")?;
        assert_eq!(trial.command(ready, 0)?, None);
        let unavailable = [
            DeckOutput {
                ring: None,
                ..ready[0]
            },
            ready[1],
        ];
        assert_eq!(trial.command(unavailable, 1)?, None);
        std::fs::write(&path, b"244")?;
        assert_eq!(trial.command(ready, 1)?, None);
        std::fs::write(&path, b"idle")?;
        assert_eq!(trial.command(ready, 1)?, None);
        let playing = [
            DeckOutput {
                playing: true,
                ..ready[0]
            },
            ready[1],
        ];
        std::fs::write(&path, b"244")?;
        assert_eq!(trial.command(playing, 2)?, None);
        assert_eq!(trial.command(ready, 2)?, None);
        std::fs::write(&path, b"idle")?;
        assert_eq!(trial.command(ready, 2)?, None);
        std::fs::write(&path, b"107")?;
        assert_eq!(trial.command(ready, 2)?, Some(107));
        std::fs::remove_file(path)?;
        Ok(())
    }
}
