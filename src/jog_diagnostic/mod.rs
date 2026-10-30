//! Offline and explicitly opted-in native wheel-tension diagnostics.

#[cfg(test)]
mod assist_preparation_tests;
#[cfg(test)]
mod assist_session_tests;
#[cfg(test)]
mod assist_tests;
mod capture;
use s4mk3_hid_midi_bridge::jog::continuity;
#[cfg(test)]
mod continuity_tests;
mod damped {
    pub use s4mk3_hid_midi_bridge::jog::damped::*;
    pub const fn config() -> super::capture::LogRecord {
        super::capture::LogRecord::Damping {
            window_us: WINDOW_US,
            deadband_counts: DEADBAND_COUNTS,
            full_scale_counts: FULL_SCALE_COUNTS,
            qualification_us: QUALIFICATION_US,
            qualification_counts: QUALIFICATION_COUNTS,
            tail_us: super::continuity::TAIL_US,
            output_interval_us: super::continuity::OUTPUT_INTERVAL_US,
        }
    }
}
#[cfg(test)]
mod damped_session_tests;
#[cfg(test)]
mod damped_tests;
mod decode {
    pub use s4mk3_hid_midi_bridge::jog::decode::*;
    #[cfg(test)]
    pub(super) mod tests {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/jog_diagnostic/decode_tests.rs"
        ));
    }
}
mod live;
mod motor;
mod options;
mod ownership;
mod paced;
mod platter;
mod platter_live;
mod platter_rotation;
mod platter_session;
mod platter_telemetry;
#[cfg(test)]
mod play_tests;
#[cfg(test)]
mod preparation_tests;
mod reader;
mod reader_telemetry;
mod replay;
mod session;
mod smooth {
    pub use s4mk3_hid_midi_bridge::jog::smooth::*;
    pub const fn config() -> super::capture::LogRecord {
        super::capture::LogRecord::Smoothing {
            time_constant_us: TIME_CONSTANT_US,
            rest_cutoff_us: super::continuity::TAIL_US,
            zero_sample_dedup: true,
        }
    }
}
#[cfg(test)]
mod smooth_preparation_tests;
#[cfg(test)]
mod smooth_session_tests;
#[cfg(test)]
mod smooth_tests;
use s4mk3_hid_midi_bridge::jog::smooth_wheels;
#[cfg(test)]
mod tension_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod wheels_lifecycle_tests;
#[cfg(test)]
mod wheels_tests;
mod window;

use std::path::PathBuf;

#[cfg(test)]
use anyhow::Context as _;
use anyhow::ensure;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use decode::Decoder;
#[cfg(test)]
use motor::Controller;
use replay::run as replay;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Prepare and record supervised S4 MK3 wheel-tension experiments"
)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Measure integrated relative raw counts, without any HID output.
    PlatterMeasure(platter::MeasureOptions),
    /// Bounded open-loop fixed force, not constant-RPM playback.
    PlatterDrive(platter::DriveOptions),
    /// Bounded feedback-controlled speed probe, separate from Djay playback.
    PlatterRotate(platter_rotation::Options),
    /// Follow calibrated relative motion; touch holds, release resumes.
    PlatterRing(platter::RingOptions),
    /// Capture buttons/jog snapshots without sending any output reports.
    Capture {
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=120))]
        seconds: u64,
        /// Include fresh mixer/fader snapshots (numbered report 2).
        #[arg(long)]
        analog: bool,
    },
    /// Decode a capture and calculate torque offline; never opens HID.
    Replay {
        #[arg(long)]
        input: PathBuf,
        #[command(flatten)]
        control: ControlOptions,
    },
    /// Run a supervised trial; motor output requires explicit opt-in.
    #[command(visible_alias = "trial")]
    Live {
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=30))]
        seconds: u64,
        #[arg(long)]
        enable_motor: bool,
        /// Fader modes only: run under supervision until Ctrl-C.
        #[arg(long, conflicts_with = "seconds")]
        until_stopped: bool,
        /// Independently compare the two native jog-ring initialization reports.
        #[arg(long)]
        initialize_rings: bool,
        /// Hold neutral before starting; manual preparation is for qualified modes.
        #[arg(long)]
        start_after_neutral: bool,
        #[command(flatten)]
        control: ControlOptions,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Baseline,
    Brake,
    Assist,
    FaderBrake,
    FaderTension,
    FaderContinuity,
    FaderDamped,
    FaderSmooth,
    FaderAssist,
    FaderPlay,
}

impl Mode {
    const fn uses_fader(self) -> bool {
        match self {
            Self::FaderBrake
            | Self::FaderTension
            | Self::FaderContinuity
            | Self::FaderDamped
            | Self::FaderSmooth
            | Self::FaderAssist
            | Self::FaderPlay => true,
            Self::Baseline | Self::Brake | Self::Assist => false,
        }
    }

    const fn is_paced(self) -> bool {
        matches!(
            self,
            Self::FaderContinuity
                | Self::FaderDamped
                | Self::FaderSmooth
                | Self::FaderAssist
                | Self::FaderPlay
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum Side {
    Left,
    Right,
    Both,
}

impl Side {
    const fn mask(self) -> [bool; 2] {
        match self {
            Self::Left => [true, false],
            Self::Right => [false, true],
            Self::Both => [true, true],
        }
    }
}

#[derive(Debug, clap::Args)]
struct ControlOptions {
    #[arg(long, value_enum)]
    mode: Mode,
    /// Physical wheel(s), not logical decks; both is for fader-play only.
    #[arg(long, value_enum, default_value = "left")]
    side: Side,
    /// Fixed modes only: raw units per count/ms (default 1).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=100))]
    gain: Option<u32>,
    /// Braking ceiling: default 100 fixed / 5000 fader-brake / 30000 fader-tension.
    #[arg(long, visible_alias = "force-cap", value_parser = clap::value_parser!(u16).range(1..=60000))]
    max_force: Option<u16>,
    /// Held-touch assistance ceiling; fader-assist defaults to 2000, older bipolar modes 8000.
    #[arg(long, value_parser = clap::value_parser!(u16).range(0..=8000))]
    assist_force: Option<u16>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct ControlArgs {
    mode: Mode,
    side: Side,
    gain: u32,
    max_force: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    assist_force: Option<u16>,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    match args.command {
        Command::PlatterMeasure(options) => platter_live::run(options.into()),
        Command::PlatterDrive(options) => platter_live::run(options.resolve()?),
        Command::PlatterRotate(options) => platter_live::run(options.resolve()?),
        Command::PlatterRing(options) => platter_live::run(options.into()),
        Command::Capture {
            out,
            seconds,
            analog,
        } => live::capture(&out, seconds, analog),
        Command::Replay { input, control } => replay(&input, control.resolve()?),
        Command::Live {
            out,
            seconds,
            enable_motor,
            until_stopped,
            initialize_rings,
            start_after_neutral,
            control,
        } => {
            ensure!(
                enable_motor,
                "live trials require --enable-motor, including baseline"
            );
            let control = control.resolve()?;
            ensure!(
                !start_after_neutral
                    || !until_stopped
                    || matches!(
                        control.mode,
                        Mode::FaderDamped | Mode::FaderSmooth | Mode::FaderAssist | Mode::FaderPlay
                    ),
                "--start-after-neutral with --until-stopped requires a qualified fader mode"
            );
            ensure!(
                !start_after_neutral
                    || matches!(
                        control.mode,
                        Mode::FaderTension
                            | Mode::FaderContinuity
                            | Mode::FaderDamped
                            | Mode::FaderSmooth
                            | Mode::FaderAssist
                            | Mode::FaderPlay
                    ),
                "--start-after-neutral requires a bipolar fader mode"
            );
            ensure!(
                !initialize_rings
                    || matches!(
                        control.mode,
                        Mode::FaderTension | Mode::FaderContinuity | Mode::FaderPlay
                    ),
                "--initialize-rings requires fader-tension, fader-continuity or fader-play"
            );
            ensure!(
                !until_stopped || control.mode.uses_fader(),
                "--until-stopped requires a fader mode"
            );
            ensure!(
                control.mode.uses_fader() || control.max_force <= 500,
                "fixed live --max-force must be <=500"
            );
            live::trial(
                &out,
                live::TrialOptions {
                    seconds: if until_stopped { None } else { Some(seconds) },
                    initialize_rings,
                    start_after_neutral,
                },
                control,
            )
        }
    }
}
