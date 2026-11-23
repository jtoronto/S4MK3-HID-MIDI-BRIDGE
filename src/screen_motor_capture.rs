//! Cold-path configuration and export; devices are closed before JSONL output.

use anyhow::{Context as _, ensure};
use s4mk3_hid_midi_bridge::screen_motor_diag::{self as diag, Capture, Case, Guard, Kind, Role};
use serde::Deserialize;
use std::{fs::OpenOptions, path::PathBuf};

mod startup;
pub use startup::await_endpoint;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Options {
    version: u8,
    case: Case,
    capture_path: PathBuf,
    capture: bool,
}

pub struct Session {
    capture: Option<Capture>,
    guard: Option<Guard>,
}

impl Session {
    pub fn start(args: &crate::Args) -> anyhow::Result<Self> {
        let Some(path) = &args.screen_motor_diagnostic_config else {
            return Ok(Self {
                capture: None,
                guard: None,
            });
        };
        ensure!(
            path.is_absolute(),
            "diagnostic config path must be absolute"
        );
        ensure!(
            matches!(args.midi, Some(crate::MidiProfile::Full)) && args.managed.stdin,
            "screen/motor diagnostics require full managed profile"
        );
        let options: Options = serde_json::from_slice(
            &std::fs::read(path).context("read screen/motor diagnostic config")?,
        )
        .context("parse screen/motor diagnostic config")?;
        ensure!(
            options.version == 1,
            "unsupported diagnostic config version"
        );
        ensure!(
            options.capture_path.is_absolute(),
            "diagnostic capture path must be absolute"
        );
        ensure!(
            options.capture || options.case == Case::Normal,
            "controlled cases require capture"
        );
        let capture = options
            .capture
            .then(|| Capture::start(&options.capture_path, options.case))
            .transpose()?;
        let metadata_path = options.capture_path.with_extension("metadata.json");
        let mut metadata = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(metadata_path)
            .context("reserve diagnostic runtime metadata")?;
        let configs = [&args.led_config, &args.jog_config, &args.tension_config]
            .into_iter()
            .flatten()
            .map(|path| {
                Ok(serde_json::json!({
                    "path": path,
                    "contents": std::fs::read_to_string(path)?
                }))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        serde_json::to_writer_pretty(
            &mut metadata,
            &serde_json::json!({
                "version": 1, "case": options.case, "capture": options.capture,
                "pid": std::process::id(), "executable": std::env::current_exe()?,
                "arguments": std::env::args().collect::<Vec<_>>(),
                "configs": configs, "logging": "info; raw and MIDI capture disabled"
            }),
        )?;
        if let Some(capture) = &capture {
            capture.activate()?;
        }
        let guard = capture.as_ref().map(|capture| capture.enter(Role::Output));
        diag::record(Kind::Lifecycle, [1, 0, 0, 0, 0, 0, 0, 0], &[]);
        Ok(Self { capture, guard })
    }

    pub fn finish(mut self, result: anyhow::Result<()>) -> anyhow::Result<()> {
        diag::record(
            Kind::Lifecycle,
            [2, u64::from(result.is_ok()), 0, 0, 0, 0, 0, 0],
            &[],
        );
        drop(self.guard.take());
        let exported = self
            .capture
            .as_ref()
            .map(|capture| capture.finish(result.is_ok()))
            .transpose();
        match (result, exported) {
            (Ok(()), Ok(_)) => Ok(()),
            (Err(error), Ok(_)) | (Ok(()), Err(error)) => Err(error),
            (Err(error), Err(export)) => {
                Err(error.context(format!("diagnostic export also failed: {export:#}")))
            }
        }
    }
}

pub fn bridge_epoch() -> std::time::Instant {
    let started = std::time::Instant::now();
    diag::record(Kind::Lifecycle, [3, diag::now_us(), 0, 0, 0, 0, 0, 0], &[]);
    started
}

#[cfg(test)]
mod tests;
