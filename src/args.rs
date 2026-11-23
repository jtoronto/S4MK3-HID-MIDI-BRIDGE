//! Command-line contracts shared by the runtime and offline exporters.

use crate::{midi_capture, midi_transport};
use clap::{Parser, ValueEnum};
use std::{num::NonZeroU64, path::PathBuf};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum MidiProfile {
    Minimal,
    Full,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ReplayFixture {
    Playback,
}

#[derive(Debug, Parser)]
#[command(version, about, group(clap::ArgGroup::new("jog_context").args(["generate_mapping", "midi", "replay_motor_fixture"])))]
pub struct Args {
    #[command(flatten)]
    pub managed: ManagedArgs,
    /// MIDI transport and mapping layout; export accepts every backend offline.
    #[arg(long, value_enum, default_value_t = midi_transport::Backend::default())]
    pub midi_backend: midi_transport::Backend,
    /// Temporary raw MIDI JSONL capture; creates a new absolute path.
    #[arg(long, value_name = "PATH", requires = "midi")]
    pub midi_capture: Option<PathBuf>,
    #[arg(skip)]
    pub capture_handle: midi_capture::Handle,
    /// Opt-in bounded screen/motor diagnostics; full managed profile only.
    #[arg(long, value_name = "PATH", requires = "managed_stdin", conflicts_with_all = ["raw", "midi_capture", "list", "generate_mapping", "generate_screen_previews", "replay_motor_fixture"])]
    pub screen_motor_diagnostic_config: Option<PathBuf>,
    /// Replay the built-in motor playback fixture without opening HID or MIDI.
    #[arg(long, value_enum, num_args = 0..=1, default_missing_value = "playback", conflicts_with_all = ["midi", "list", "generate_mapping", "generate_screen_previews", "raw"])]
    pub replay_motor_fixture: Option<ReplayFixture>,
    /// List attached S4 MK3 devices without opening their USB interfaces.
    #[arg(long)]
    pub list: bool,
    /// Log every complete input HID report in hex, including its report ID.
    #[arg(long)]
    pub raw: bool,
    /// Publish virtual MIDI; bare --midi retains the verified minimal profile.
    #[arg(long, value_enum, num_args = 0..=1, default_missing_value = "minimal", conflicts_with_all = ["list", "generate_mapping"])]
    pub midi: Option<MidiProfile>,
    /// JSON LED preferences for the full profile; restart after editing.
    #[arg(long, value_name = "PATH", requires = "midi", conflicts_with_all = ["list", "generate_mapping"])]
    pub led_config: Option<PathBuf>,
    /// Saved hardware tension for the full profile; restart after editing.
    #[arg(long, value_name = "PATH", requires = "midi", conflicts_with_all = ["list", "generate_mapping"])]
    pub tension_config: Option<PathBuf>,
    #[command(flatten)]
    pub screens: ScreenArgs,
    /// Export the full native Djay mapping without opening HID or MIDI.
    #[arg(long, value_name = "PATH", conflicts_with_all = ["midi", "list", "raw"])]
    pub generate_mapping: Option<PathBuf>,
    /// JSON jog preferences for mapping export or full-profile backspin behavior.
    #[arg(
        long,
        value_name = "PATH",
        requires = "jog_context",
        conflicts_with = "list"
    )]
    pub jog_config: Option<PathBuf>,
    /// Maximum probe duration; stop earlier with Ctrl-C.
    #[arg(long, default_value = "30")]
    pub seconds: NonZeroU64,
    /// Run until Ctrl-C instead of stopping after a fixed duration.
    #[arg(long, conflicts_with = "seconds")]
    pub until_stopped: bool,
}

#[derive(Debug, clap::Args)]
pub struct ManagedArgs {
    /// Accept managed STOP commands and stop normally when stdin closes.
    #[arg(long = "managed-stdin", id = "managed_stdin", conflicts_with_all = ["list", "generate_mapping", "generate_screen_previews", "replay_motor_fixture"])]
    pub stdin: bool,
    /// Host barrier: prepare capture before the MIDI endpoint is published.
    #[arg(long, hide = true, requires = "screen_motor_diagnostic_config")]
    pub prepare_screen_motor_diagnostics: bool,
}

#[derive(Debug, clap::Args)]
pub struct ScreenArgs {
    /// Windows BLOOP only: use one NI session for both screens, platters and rings.
    #[arg(long, requires = "midi", conflicts_with_all = ["control_stdin", "screen_motor_diagnostic_config", "tension_config"])]
    pub ni_decks: bool,
    /// Experimental Djay feedback and reactive STEMS on both S4 screens (full profile).
    #[arg(long = "screens", requires = "midi", conflicts_with_all = ["list", "generate_mapping"])]
    pub enabled: bool,
    /// Accept live screen ON/OFF/RETRY commands on stdin; EOF stops the bridge.
    #[arg(long = "screen-control-stdin", requires = "midi", conflicts_with_all = ["list", "generate_mapping", "generate_screen_previews"])]
    pub control_stdin: bool,
    /// Export screen routing, feedback, and held-pad rasters without USB or MIDI.
    #[arg(long, value_name = "DIR", conflicts_with_all = ["midi", "list", "raw", "generate_mapping", "jog_config"])]
    pub generate_screen_previews: Option<PathBuf>,
}
