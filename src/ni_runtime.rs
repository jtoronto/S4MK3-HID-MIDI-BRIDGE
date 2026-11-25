//! Command-line policy for the NI-owned screen and deck runtime.

use anyhow::{Context as _, ensure};
use serde::Deserialize;

use crate::{Args, MidiProfile, midi_transport::Backend};

const DEFAULT_TENSION: u8 = 107;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Tension {
    value: u8,
}

pub fn settings(args: &Args, trial_enabled: bool) -> anyhow::Result<Option<u8>> {
    ensure!(
        !trial_enabled || (cfg!(windows) && args.screens.ni_decks),
        "S4_BRIDGE_NI_JOGT_TRIAL_FILE requires Windows --ni-decks"
    );
    if !args.screens.ni_decks {
        return Ok(None);
    }
    ensure!(
        matches!(args.midi, Some(MidiProfile::Full)),
        "--ni-decks requires --midi full"
    );
    ensure!(
        (cfg!(target_os = "macos") && args.midi_backend == Backend::CoreMidi)
            || (cfg!(windows) && args.midi_backend == Backend::Bloop),
        "--ni-decks requires macOS CoreMIDI or Windows BLOOP"
    );
    if !cfg!(target_os = "macos") {
        ensure!(
            args.screens.ni_tension_config.is_none(),
            "--ni-tension-config requires macOS --ni-decks"
        );
        return Ok(None);
    }
    ensure!(
        !args.screens.enabled,
        "--ni-decks always uses NI service screens; omit --screens"
    );
    args.screens
        .ni_tension_config
        .as_deref()
        .map_or(Ok(Some(DEFAULT_TENSION)), |path| {
            let bytes = std::fs::read(path)
                .with_context(|| format!("read NI tension settings {}", path.display()))?;
            let settings: Tension = serde_json::from_slice(&bytes)
                .with_context(|| format!("parse NI tension settings {}", path.display()))?;
            Ok(Some(settings.value))
        })
}

#[cfg(any(windows, target_os = "macos"))]
pub fn bind(session: &crate::lifecycle::Session, serial: Option<&str>) -> anyhow::Result<()> {
    session
        .screens
        .as_ref()
        .and_then(crate::full::screens::Output::control)
        .context("NI deck worker missing")?
        .bind_selected_hid(serial.context("selected S4 has no serial for NI deck output")?)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use clap::Parser as _;

    #[test]
    fn managed_ni_launch_selects_its_own_tension_and_rejects_direct_options() -> anyhow::Result<()>
    {
        let launch = [
            "bridge",
            "--midi",
            "full",
            "--ni-decks",
            "--managed-stdin",
            "--until-stopped",
        ];
        let args = Args::try_parse_from(launch)?;
        assert_eq!(settings(&args, false)?, Some(107));
        assert!(settings(&args, true).is_err());
        let duplicate_screens = Args::try_parse_from(launch.into_iter().chain(["--screens"]))?;
        assert!(settings(&duplicate_screens, false).is_err());
        let wrong_backend =
            Args::try_parse_from(launch.into_iter().chain(["--midi-backend", "bloop"]))?;
        assert!(settings(&wrong_backend, false).is_err());
        assert!(
            Args::try_parse_from(
                launch
                    .into_iter()
                    .chain(["--tension-config", "direct.json"])
            )
            .is_err()
        );
        let direct = ["bridge", "--midi", "full", "--screens", "--managed-stdin"];
        assert_eq!(settings(&Args::try_parse_from(direct)?, false)?, None);
        assert!(
            Args::try_parse_from(direct.into_iter().chain(["--ni-tension-config", "ni.json"]))
                .is_err()
        );
        Ok(())
    }
}
