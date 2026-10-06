//! Pure button and pad LED rendering for the full S4 MK3 profile.

use std::path::Path;

use anyhow::{Context as _, ensure};
use serde::Deserialize;

use super::catalog::PadMode;
use super::feedback::FeedbackState;

const REPORT_LEN: usize = 95;
const MIXER_ROW: usize = 4;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    Red,
    Carrot,
    Orange,
    Honey,
    Yellow,
    Lime,
    Green,
    Aqua,
    Cyan,
    Sky,
    Blue,
    Purple,
    Fuchsia,
    Magenta,
    Azalea,
    Salmon,
    White,
}

impl Color {
    /// Return this color's S4 MK3 palette base byte.
    pub const fn base(self) -> u8 {
        match self {
            Self::Red => 4,
            Self::Carrot => 8,
            Self::Orange => 12,
            Self::Honey => 16,
            Self::Yellow => 20,
            Self::Lime => 24,
            Self::Green => 28,
            Self::Aqua => 32,
            Self::Cyan => 36,
            Self::Sky => 40,
            Self::Blue => 44,
            Self::Purple => 48,
            Self::Fuchsia => 52,
            Self::Magenta => 56,
            Self::Azalea => 60,
            Self::Salmon => 64,
            Self::White => 68,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum HotcueColors {
    #[default]
    Djay,
    FixedSlots,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum InactiveDisplay {
    #[default]
    Dim,
    Dark,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuantizeMixed {
    #[default]
    Dim,
    Dark,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LedConfig {
    pub deck_colors: [Color; 4],
    pub stem_colors: [Color; 4],
    pub djay_pad_colors: [Color; 8],
    pub djay_white: Color,
    pub hotcue_colors: HotcueColors,
    pub fixed_hotcue_colors: [Color; 8],
    pub active_brightness: u8,
    pub inactive_brightness: u8,
    pub inactive_display: InactiveDisplay,
    pub palette_active_intensity: u8,
    pub palette_inactive_intensity: u8,
    pub stem_mute_bright: bool,
    pub mute_color: Color,
    pub quantize_mixed: QuantizeMixed,
    pub tempo_center_tolerance: Option<f32>,
    pub loop_enabled: bool,
    pub loop_color: Color,
    pub chase_period_ms: u64,
    pub meter_brightness: u8,
    pub meter_gamma: f32,
    pub meter_stale_ms: u64,
}

impl Default for LedConfig {
    fn default() -> Self {
        Self {
            deck_colors: [Color::Red, Color::Blue, Color::Yellow, Color::Purple],
            stem_colors: [Color::Red, Color::Yellow, Color::Green, Color::Blue],
            // Provisional token order: capture Djay's emitted values before calibration.
            djay_pad_colors: [
                Color::Red,
                Color::Orange,
                Color::Yellow,
                Color::Green,
                Color::Cyan,
                Color::Blue,
                Color::Purple,
                Color::White,
            ],
            djay_white: Color::White,
            hotcue_colors: HotcueColors::Djay,
            fixed_hotcue_colors: [
                Color::Red,
                Color::Orange,
                Color::Yellow,
                Color::Green,
                Color::Cyan,
                Color::Blue,
                Color::Purple,
                Color::White,
            ],
            active_brightness: 127,
            inactive_brightness: 16,
            inactive_display: InactiveDisplay::Dim,
            palette_active_intensity: 2,
            palette_inactive_intensity: 0,
            stem_mute_bright: true,
            mute_color: Color::Red,
            quantize_mixed: QuantizeMixed::Dim,
            tempo_center_tolerance: None,
            loop_enabled: true,
            loop_color: Color::Green,
            chase_period_ms: 2000,
            meter_brightness: 127,
            meter_gamma: 1.0,
            meter_stale_ms: 500,
        }
    }
}

impl LedConfig {
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        let config = match path {
            Some(path) => {
                let source = std::fs::read(path)
                    .with_context(|| format!("read LED config {}", path.display()))?;
                serde_json::from_slice(&source)
                    .with_context(|| format!("parse LED config {}", path.display()))?
            }
            None => Self::default(),
        };
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            (1..=127).contains(&self.active_brightness),
            "active_brightness must be in 1..=127"
        );
        ensure!(
            self.inactive_brightness <= 127,
            "inactive_brightness must be at most 127"
        );
        ensure!(
            self.palette_active_intensity <= 3 && self.palette_inactive_intensity <= 3,
            "palette intensities must be in 0..=3"
        );
        if let Some(tolerance) = self.tempo_center_tolerance {
            ensure!(
                tolerance.is_finite() && (0.0..=0.1).contains(&tolerance) && tolerance > 0.0,
                "tempo_center_tolerance must be finite and in (0, 0.1]"
            );
        }
        ensure!(
            (250..=10000).contains(&self.chase_period_ms),
            "chase_period_ms must be in 250..=10000"
        );
        ensure!(
            matches!(self.meter_brightness, 0 | 125 | 127),
            "meter_brightness must be 0, 125 (dim), or 127 (bright)"
        );
        ensure!(
            self.meter_gamma.is_finite() && (0.1..=5.0).contains(&self.meter_gamma),
            "meter_gamma must be finite and in 0.1..=5"
        );
        ensure!(
            (100..=5000).contains(&self.meter_stale_ms),
            "meter_stale_ms must be in 100..=5000"
        );
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalState {
    /// Zero-based deck index: A=0, B=1, C=2, D=3.
    pub decks: [usize; 2],
    pub shifted: [bool; 2],
    pub pads: [PadMode; 2],
    /// Zero-based deck targeted by each FX strip.
    pub fx: [usize; 2],
    /// Zero-based deck selected by the quick-FX target selector.
    pub quick: usize,
    pub vinyl: [bool; 2],
    pub grid: [bool; 2],
    /// side * 5 + REC, library view, playlist, star, preview; index 10 is filter.
    pub held: [bool; 11],
    /// Physical tempo-fader positions normalized to 0.0..=1.0.
    pub tempo: [Option<f32>; 2],
    /// Jump-size selection belongs to logical decks, not physical sides.
    pub move_selecting: [bool; 4],
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            decks: [0, 1],
            shifted: [false; 2],
            pads: [PadMode::Hotcue; 2],
            fx: [0, 1],
            quick: 0,
            vinyl: [true; 2],
            grid: [false; 2],
            held: [false; 11],
            tempo: [None; 2],
            move_selecting: [false; 4],
        }
    }
}

/// Render the complete `0x80` buttons report without mutating cached state.
pub fn render_buttons(
    config: &LedConfig,
    feedback: &FeedbackState,
    local: &LocalState,
) -> [u8; REPORT_LEN] {
    let mut renderer = ButtonRenderer {
        config,
        feedback,
        local,
        report: [0; REPORT_LEN],
    };
    renderer.report[0] = 0x80;
    for side in 0..2 {
        renderer.deck_lights(side);
        renderer.momentary_lights(side);
        renderer.pads(side);
        renderer.fx_strip(side);
    }
    renderer.mixer();
    renderer.report
}

struct ButtonRenderer<'a> {
    config: &'a LedConfig,
    feedback: &'a FeedbackState,
    local: &'a LocalState,
    report: [u8; REPORT_LEN],
}

impl ButtonRenderer<'_> {
    const fn set(&mut self, offset: usize, value: u8) {
        self.report[offset + 1] = value;
    }

    fn palette(&self, color: Color, active: bool) -> u8 {
        if !active && self.config.inactive_display == InactiveDisplay::Dark {
            return 0;
        }
        color.base().saturating_add(if active {
            self.config.palette_active_intensity
        } else {
            self.config.palette_inactive_intensity
        })
    }

    fn state_palette(&self, color: Color, value: Option<u8>) -> u8 {
        value.map_or(0, |value| self.palette(color, value != 0))
    }

    fn state_raw(&self, value: Option<u8>) -> u8 {
        value.map_or(0, |value| {
            if value != 0 {
                self.config.active_brightness
            } else {
                match self.config.inactive_display {
                    InactiveDisplay::Dim => self.config.inactive_brightness,
                    InactiveDisplay::Dark => 0,
                }
            }
        })
    }

    fn deck_lights(&mut self, side: usize) {
        let deck = self.local.decks[side];
        let color = self.config.deck_colors[deck];
        for (offsets, note, color) in [
            ([8, 31], 1, Color::Orange),
            ([14, 37], 2, color),
            ([15, 38], 8, color),
        ] {
            self.set(
                offsets[side],
                self.state_palette(color, self.feedback.notes[deck][note]),
            );
        }
        for (offsets, note) in [([55, 66], 0), ([60, 71], 3), ([61, 72], 4)] {
            self.set(
                offsets[side],
                self.state_raw(self.feedback.notes[deck][note]),
            );
        }
        for (offsets, mode, color) in [
            ([9, 32], PadMode::Hotcue, color),
            ([57, 68], PadMode::Samples, Color::Cyan),
            ([10, 33], PadMode::Stems, Color::Green),
            ([58, 69], PadMode::Stems, Color::Green),
        ] {
            self.set(
                offsets[side],
                self.palette(color, self.local.pads[side] == mode),
            );
        }
        for slot in 0..2 {
            let selected = [[0, 2], [1, 3]][side][slot];
            self.set(
                [[12, 13], [35, 36]][side][slot],
                self.palette(self.config.deck_colors[selected], deck == selected),
            );
        }
        for (offsets, active) in [
            ([16, 39], !self.local.vinyl[side]),
            ([17, 40], self.local.vinyl[side]),
        ] {
            self.set(offsets[side], self.palette(color, active));
        }
        self.set(
            [59, 70][side],
            self.state_raw(Some(u8::from(self.local.shifted[side]))),
        );
        self.set(
            [18, 41][side],
            self.palette(Color::White, self.local.grid[side]),
        );
        if let Some(tolerance) = self.config.tempo_center_tolerance {
            if self.local.tempo[side].is_some_and(|position| (position - 0.5).abs() <= tolerance) {
                self.set([11, 34][side], self.palette(Color::White, true));
            }
        }
    }

    fn momentary_lights(&mut self, side: usize) {
        for slot in 0..5 {
            let offset = [[56, 19, 20, 21, 22], [67, 42, 43, 44, 45]][side][slot];
            self.set(
                offset,
                self.palette(Color::White, self.local.held[side * 5 + slot]),
            );
        }
    }

    fn pads(&mut self, side: usize) {
        let deck = self.local.decks[side];
        let mode = self.local.pads[side];
        for pad in 0..8 {
            let note = match mode {
                PadMode::Hotcue => 40,
                PadMode::Samples => 48,
                PadMode::Stems => 56,
            } + pad;
            let value = self.feedback.notes[deck][note];
            let rendered = match mode {
                PadMode::Hotcue => cue_color(self.config, pad, value),
                PadMode::Samples => self.state_palette(Color::Cyan, value),
                PadMode::Stems => value.map_or(0, |raw| {
                    let stem = self.config.stem_colors[pad % 4];
                    let active = if pad < 4 {
                        (raw != 0) == self.config.stem_mute_bright
                    } else {
                        raw != 0
                    };
                    let color = if pad < 4 && raw != 0 && active {
                        self.config.mute_color
                    } else {
                        stem
                    };
                    self.palette(color, active)
                }),
            };
            self.set([0, 23][side] + pad, rendered);
        }
    }

    fn fx_strip(&mut self, side: usize) {
        let target = self.local.fx[side];
        for effect in 0..4 {
            self.set(
                [62, 73][side] + effect,
                self.state_palette(
                    self.config.deck_colors[target],
                    self.feedback.notes[target][32 + effect],
                ),
            );
        }
    }

    fn mixer(&mut self) {
        for channel in 0..4 {
            self.set(
                77 + channel * 4,
                self.state_palette(Color::White, self.feedback.notes[MIXER_ROW][channel]),
            );
            self.set(
                78 + channel * 4,
                self.palette(Color::White, self.local.fx[0] == channel),
            );
            self.set(
                79 + channel * 4,
                self.palette(Color::White, self.local.fx[1] == channel),
            );
            self.set(
                46 + channel,
                self.palette(
                    self.config.deck_colors[channel],
                    self.local.quick == channel,
                ),
            );
        }
        for effect in 0..4 {
            let color = [Color::Red, Color::Green, Color::Blue, Color::Yellow][effect];
            self.set(
                50 + effect,
                self.state_palette(color, self.feedback.notes[self.local.quick][16 + effect]),
            );
        }
        self.set(54, self.palette(Color::White, self.local.held[10]));
        let values: [Option<u8>; 4] = std::array::from_fn(|deck| self.feedback.notes[deck][15]);
        let quantize = if values.iter().any(Option::is_none) {
            0
        } else if values
            .iter()
            .all(|value| value.is_some_and(|value| value != 0))
        {
            self.palette(Color::White, true)
        } else {
            match self.config.quantize_mixed {
                QuantizeMixed::Dim => self.palette(Color::White, false),
                QuantizeMixed::Dark => 0,
            }
        };
        self.set(93, quantize);
    }
}

fn cue_color(config: &LedConfig, pad: usize, value: Option<u8>) -> u8 {
    let Some(token) = value else {
        return 0;
    };
    if token == 0 {
        return 0;
    }
    if config.hotcue_colors == HotcueColors::FixedSlots {
        return config.fixed_hotcue_colors.get(pad).map_or(0, |color| {
            color.base().saturating_add(config.palette_active_intensity)
        });
    }
    match token {
        1..=8 => config
            .djay_pad_colors
            .get(usize::from(token - 1))
            .map_or(0, |color| {
                color.base().saturating_add(config.palette_active_intensity)
            }),
        9 => config
            .djay_white
            .base()
            .saturating_add(config.palette_active_intensity),
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feedback() -> FeedbackState {
        FeedbackState::default()
    }

    fn config() -> LedConfig {
        LedConfig::default()
    }

    #[test]
    fn unknown_software_state_renders_dark_and_known_off_dim() {
        let mut feedback = feedback();
        let mut local = LocalState::default();
        feedback.notes[0][0] = Some(0);
        let report = render_buttons(&config(), &feedback, &local);
        assert_eq!(report[1 + 55], 16);
        assert_eq!(report[1 + 66], 0);

        local.pads[0] = PadMode::Samples;
        feedback.notes[0][48] = Some(0);
        let report = render_buttons(&config(), &feedback, &local);
        assert_eq!(report[1], Color::Cyan.base());
        assert_eq!(report[2], 0);
    }

    #[test]
    fn hidden_deck_selection_uses_its_cached_feedback() {
        let mut feedback = feedback();
        feedback.notes[2][0] = Some(127);
        let mut local = LocalState::default();
        local.decks[0] = 2;
        let report = render_buttons(&config(), &feedback, &local);
        assert_eq!(report[1 + 55], 127);
        assert_eq!(report[1 + 12], Color::Red.base());
        assert_eq!(report[1 + 13], Color::Yellow.base() + 2);
    }

    #[test]
    fn hotcue_tokens_and_unrecognized_token_render_without_guessing() {
        let mut feedback = feedback();
        feedback.notes[0][40] = Some(1);
        feedback.notes[0][41] = Some(9);
        feedback.notes[0][42] = Some(127);
        let report = render_buttons(&config(), &feedback, &LocalState::default());
        assert_eq!(report[1], Color::Red.base() + 2);
        assert_eq!(report[2], Color::White.base() + 2);
        assert_eq!(report[3], 0);
    }

    #[test]
    fn fixed_slot_override_and_pad_layers_use_their_own_cached_state() {
        let mut feedback = feedback();
        feedback.notes[0][40] = Some(6);
        feedback.notes[0][48] = Some(127);
        feedback.notes[0][56] = Some(127);
        let mut local = LocalState::default();
        local.pads[0] = PadMode::Samples;
        let report = render_buttons(&config(), &feedback, &local);
        assert_eq!(report[1], Color::Cyan.base() + 2);

        local.pads[0] = PadMode::Stems;
        let report = render_buttons(&config(), &feedback, &local);
        assert_eq!(report[1], Color::Red.base() + 2);
        assert_eq!(report[1 + 10], Color::Green.base() + 2);
        assert_eq!(report[1 + 58], Color::Green.base() + 2);

        let mut fixed = config();
        fixed.hotcue_colors = HotcueColors::FixedSlots;
        fixed.fixed_hotcue_colors[0] = Color::Yellow;
        local.pads[0] = PadMode::Hotcue;
        let report = render_buttons(&fixed, &feedback, &local);
        assert_eq!(report[1], Color::Yellow.base() + 2);
    }

    #[test]
    fn fx_leds_follow_their_target_instead_of_the_displayed_deck() {
        let mut feedback = feedback();
        feedback.notes[2][32] = Some(127);
        feedback.notes[0][32] = Some(0);
        let mut local = LocalState::default();
        local.decks[0] = 0;
        local.fx[0] = 2;
        let report = render_buttons(&config(), &feedback, &local);
        assert_eq!(report[1 + 62], Color::Yellow.base() + 2);
    }

    #[test]
    fn quantize_requires_complete_known_state_and_all_on_for_bright() {
        let mut feedback = feedback();
        for deck in 0..4 {
            feedback.notes[deck][15] = Some(127);
        }
        let report = render_buttons(&config(), &feedback, &LocalState::default());
        assert_eq!(report[1 + 93], Color::White.base() + 2);

        feedback.notes[3][15] = Some(0);
        let report = render_buttons(&config(), &feedback, &LocalState::default());
        assert_eq!(report[1 + 93], Color::White.base());

        feedback.notes[3][15] = None;
        let report = render_buttons(&config(), &feedback, &LocalState::default());
        assert_eq!(report[1 + 93], 0);
    }

    #[test]
    fn config_rejects_unknown_keys_and_invalid_meter_ranges() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!(
            "s4-led-config-{}-{}.json",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        std::fs::write(&path, r#"{"meter_gamma":0.0}"#)?;
        assert!(LedConfig::load(Some(&path)).is_err());
        std::fs::write(&path, r#"{"chase_period_ms":0}"#)?;
        assert!(LedConfig::load(Some(&path)).is_err());
        std::fs::write(&path, r#"{"unexpected":true}"#)?;
        assert!(LedConfig::load(Some(&path)).is_err());
        std::fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn config_defaults_and_explicit_values_load() -> anyhow::Result<()> {
        let default = LedConfig::load(None)?;
        assert_eq!(default.active_brightness, 127);
        assert_eq!(default.hotcue_colors, HotcueColors::Djay);
        assert_eq!(default.chase_period_ms, 2000);

        let path =
            std::env::temp_dir().join(format!("s4-led-override-{}.json", std::process::id()));
        std::fs::write(
            &path,
            r#"{"active_brightness":91,"hotcue_colors":"fixed_slots","loop_color":"blue","chase_period_ms":1000}"#,
        )?;
        let overridden = LedConfig::load(Some(&path))?;
        assert_eq!(overridden.active_brightness, 91);
        assert_eq!(overridden.hotcue_colors, HotcueColors::FixedSlots);
        assert_eq!(overridden.loop_color, Color::Blue);
        assert_eq!(overridden.chase_period_ms, 1000);
        std::fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn inactive_dark_preference_applies_to_local_selectors_and_modes() {
        // Given a preference that differs from default dimming.
        let config = LedConfig {
            inactive_display: InactiveDisplay::Dark,
            ..LedConfig::default()
        };
        // When the initial local state is rendered.
        let report = render_buttons(&config, &FeedbackState::default(), &LocalState::default());
        // Then inactive deck C, sample mode, and CD mode are actually dark.
        assert_eq!(report[14], 0);
        assert_eq!(report[58], 0);
        assert_eq!(report[17], 0);
        assert_eq!(report[13], Color::Red.base() + 2);
    }

    #[test]
    fn palette_intensity_override_changes_local_and_software_pad_lights() {
        // Given non-default intensity and known cue color.
        let config = LedConfig {
            palette_active_intensity: 3,
            ..LedConfig::default()
        };
        let mut feedback = FeedbackState::default();
        feedback.notes[0][40] = Some(1);
        // When rendered.
        let report = render_buttons(&config, &feedback, &LocalState::default());
        // Then both local selectors and returned cue colors honor it.
        assert_eq!(report[13], Color::Red.base() + 3);
        assert_eq!(report[1], Color::Red.base() + 3);
    }
}
