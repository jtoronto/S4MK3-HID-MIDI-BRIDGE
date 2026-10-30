//! Native Djay jog mapping preferences and runtime backspin options.

use std::path::Path;

use anyhow::{Context as _, Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScratchDirection {
    #[default]
    Normal,
    Reversed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum VinylMode {
    Bend,
    Scratch,
}

impl From<bool> for VinylMode {
    fn from(enabled: bool) -> Self {
        if enabled { Self::Scratch } else { Self::Bend }
    }
}

impl From<VinylMode> for bool {
    fn from(mode: VinylMode) -> Self {
        match mode {
            VinylMode::Bend => false,
            VinylMode::Scratch => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub scratch_speed: f64,
    pub scratch_reaction: u8,
    pub scratch_direction: ScratchDirection,
    pub pitch_bend_speed: f64,
    pub pitch_bend_reaction: u8,
    pub backspin_enabled: bool,
    pub backspin_assist_enabled: bool,
    pub backspin_assist_strength: u8,
    pub backspin_acceleration: f64,
    pub backspin_speed_increase: f64,
    pub vinyl_enabled_at_startup: bool,
    #[serde(rename = "tt_vinyl_enabled_at_startup")]
    pub tt_startup_mode: VinylMode,
    pub tt_pitch_bend_floor_percent: u8,
    pub platter_target_rpm: u16,
    pub platter_position_sensitivity: f64,
    pub platter_max_force: u16,
    pub platter_base_force: u16,
    pub platter_force_per_rpm: u16,
    pub platter_kp: u16,
    pub platter_ki: u16,
    pub platter_force_rise_per_second: u32,
    pub platter_startup_ms: u16,
    pub platter_startup_force_rise_per_second: u32,
    pub platter_overspeed_limit_rpm: u16,
}

pub const DEFAULT: Config = Config {
    scratch_speed: 2.7,
    scratch_reaction: 150,
    scratch_direction: ScratchDirection::Normal,
    pitch_bend_speed: 2.7,
    pitch_bend_reaction: 17,
    backspin_enabled: true,
    backspin_assist_enabled: true,
    backspin_assist_strength: 100,
    backspin_acceleration: 150_000.0,
    backspin_speed_increase: 5.6,
    vinyl_enabled_at_startup: true,
    tt_startup_mode: VinylMode::Bend,
    tt_pitch_bend_floor_percent: 50,
    platter_target_rpm: 33,
    platter_position_sensitivity: 14.0,
    platter_max_force: 20_000,
    platter_base_force: 1_650,
    platter_force_per_rpm: 110,
    platter_kp: 4_000,
    platter_ki: 3_000,
    platter_force_rise_per_second: 200_000,
    platter_startup_ms: 200,
    platter_startup_force_rise_per_second: 0,
    platter_overspeed_limit_rpm: 0,
};

impl Default for Config {
    fn default() -> Self {
        DEFAULT
    }
}

impl Config {
    pub const fn platter_settings(&self) -> s4mk3_hid_midi_bridge::jog::platter::Settings {
        s4mk3_hid_midi_bridge::jog::platter::Settings {
            count_direction: 1,
            target_rpm: self.platter_target_rpm,
            overspeed_limit_rpm: Some(self.platter_overspeed_limit_rpm),
            max_force: self.platter_max_force,
            base_force: self.platter_base_force,
            force_per_rpm: self.platter_force_per_rpm,
            kp: self.platter_kp,
            ki: self.platter_ki,
            force_rise_per_second: self.platter_force_rise_per_second,
            startup_ms: self.platter_startup_ms,
            startup_force_rise_per_second: self.platter_startup_force_rise_per_second,
        }
    }

    pub const fn motor_assist_enabled(&self) -> bool {
        self.backspin_enabled && self.backspin_assist_enabled && self.backspin_assist_strength > 0
    }

    pub fn load(path: Option<&Path>) -> Result<Self> {
        match path {
            None => Ok(Self::default()),
            Some(path) => {
                let bytes = std::fs::read(path)
                    .with_context(|| format!("read jog mapping preferences {}", path.display()))?;
                serde_json::from_slice(&bytes)
                    .with_context(|| format!("parse jog mapping preferences {}", path.display()))
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        for (name, speed) in [
            ("scratch_speed", self.scratch_speed),
            ("pitch_bend_speed", self.pitch_bend_speed),
        ] {
            ensure!(
                speed.is_finite() && speed > 0.0,
                "{name} must be finite and positive"
            );
        }
        ensure!(
            self.platter_position_sensitivity.is_finite()
                && (0.1..=100.0).contains(&self.platter_position_sensitivity),
            "platter_position_sensitivity must be finite and between 0.1 and 100"
        );
        for (name, reaction) in [
            ("scratch_reaction", self.scratch_reaction),
            ("pitch_bend_reaction", self.pitch_bend_reaction),
        ] {
            ensure!(reaction <= 150, "{name} must be between 0 and 150");
        }
        ensure!(
            self.backspin_acceleration.is_finite() && self.backspin_acceleration > 0.0,
            "backspin_acceleration must be finite and positive"
        );
        ensure!(
            self.backspin_assist_strength <= 100,
            "backspin_assist_strength must be between 0 and 100"
        );
        ensure!(
            self.backspin_speed_increase.is_finite()
                && self.backspin_speed_increase >= 0.0
                && (self.scratch_speed + self.backspin_speed_increase).is_finite(),
            "backspin_speed_increase must be finite and nonnegative without overflowing scratch speed"
        );
        ensure!(
            (1..=95).contains(&self.tt_pitch_bend_floor_percent),
            "tt_pitch_bend_floor_percent must be in 1..=95"
        );
        ensure!(
            (1..=45).contains(&self.platter_target_rpm),
            "platter_target_rpm must be in 1..=45"
        );
        ensure!(
            (1..=25_000).contains(&self.platter_max_force),
            "platter_max_force must be in 1..=25000"
        );
        for (name, value) in [
            ("platter_base_force", self.platter_base_force),
            ("platter_force_per_rpm", self.platter_force_per_rpm),
            ("platter_kp", self.platter_kp),
            ("platter_ki", self.platter_ki),
        ] {
            ensure!(value <= 25_000, "{name} must be in 0..=25000");
        }
        ensure!(
            self.platter_force_rise_per_second <= 1_000_000,
            "platter_force_rise_per_second must be in 0..=1000000"
        );
        ensure!(
            self.platter_startup_ms <= 2_000,
            "platter_startup_ms must be in 0..=2000"
        );
        ensure!(
            self.platter_startup_force_rise_per_second <= 1_000_000,
            "platter_startup_force_rise_per_second must be in 0..=1000000"
        );
        ensure!(
            self.platter_overspeed_limit_rpm <= 250,
            "platter_overspeed_limit_rpm must be in 0..=250"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_preferences_match_defaults_and_partial_overrides() -> Result<()> {
        // Given the configuration shipped to the native app.
        let shipped: Config = serde_json::from_str(include_str!("../../examples/jog-config.json"))?;
        // When it is parsed and a partial user configuration is decoded.
        shipped.validate()?;
        let partial: Config = serde_json::from_str(r#"{"scratch_speed":4.5}"#)?;
        // Then shipped defaults agree and omitted user fields retain those defaults.
        assert!((shipped.backspin_speed_increase - 5.6).abs() < f64::EPSILON);
        assert!((shipped.platter_position_sensitivity - 14.0).abs() < f64::EPSILON);
        assert_eq!(shipped.scratch_direction, ScratchDirection::Normal);
        assert!(shipped.backspin_assist_enabled);
        assert_eq!(shipped.backspin_assist_strength, 100);
        assert!(shipped.vinyl_enabled_at_startup);
        assert_eq!(shipped.tt_startup_mode, VinylMode::Bend);
        assert_eq!(shipped.tt_pitch_bend_floor_percent, 50);
        assert_eq!(
            serde_json::to_value(shipped)?,
            serde_json::to_value(DEFAULT)?
        );
        let mut expected = DEFAULT;
        expected.scratch_speed = 4.5;
        assert_eq!(
            serde_json::to_value(partial)?,
            serde_json::to_value(expected)?
        );
        assert!(serde_json::from_str::<Config>(r#"{"unknown":true}"#).is_err());
        assert!(serde_json::from_str::<Config>(r#"{"scratch_reaction":-1}"#).is_err());
        assert!(serde_json::from_str::<Config>(r#"{"scratch_reaction":17.5}"#).is_err());
        for input in [
            r#"{"backspin_assist_enabled":1}"#,
            r#"{"backspin_assist_enabled":"true"}"#,
            r#"{"backspin_assist_enabled":null}"#,
            r#"{"backspin_assist_strength":true}"#,
            r#"{"backspin_assist_strength":"50"}"#,
            r#"{"backspin_assist_strength":null}"#,
            r#"{"backspin_assist_strength":50.0}"#,
            r#"{"backspin_assist_strength":5e1}"#,
            r#"{"backspin_assist_strength":-1}"#,
            r#"{"backspin_assist_strength":256}"#,
        ] {
            assert!(serde_json::from_str::<Config>(input).is_err());
        }
        for input in [
            r#"{"vinyl_enabled_at_startup":1}"#,
            r#"{"scratch_direction":true}"#,
            r#"{"scratch_direction":"sideways"}"#,
            r#"{"platter_target_rpm":33.0}"#,
            r#"{"platter_target_rpm":"33"}"#,
            r#"{"platter_target_rpm":true}"#,
        ] {
            assert!(serde_json::from_str::<Config>(input).is_err(), "{input}");
        }
        for input in [
            r#"{"platter_target_rpm":0}"#,
            r#"{"tt_pitch_bend_floor_percent":0}"#,
            r#"{"tt_pitch_bend_floor_percent":96}"#,
            r#"{"platter_position_sensitivity":0.0}"#,
            r#"{"platter_position_sensitivity":100.1}"#,
            r#"{"platter_max_force":0}"#,
            r#"{"platter_max_force":25001}"#,
            r#"{"platter_base_force":25001}"#,
            r#"{"platter_force_per_rpm":25001}"#,
            r#"{"platter_kp":25001}"#,
            r#"{"platter_ki":25001}"#,
            r#"{"platter_force_rise_per_second":1000001}"#,
            r#"{"platter_startup_ms":2001}"#,
            r#"{"platter_startup_force_rise_per_second":1000001}"#,
            r#"{"platter_overspeed_limit_rpm":251}"#,
        ] {
            let config: Config = serde_json::from_str(input)?;
            assert!(config.validate().is_err(), "{input}");
        }
        Ok(())
    }

    #[test]
    fn motor_output_requires_detection_enabled_setting_and_nonzero_strength() -> Result<()> {
        // Given all combinations, including a legacy saved configuration.
        let legacy: Config = serde_json::from_str(r#"{"backspin_speed_increase":5.2}"#)?;
        assert!(legacy.backspin_assist_enabled);
        assert_eq!(legacy.backspin_assist_strength, 100);
        let saved: Config = serde_json::from_str(
            r#"{"backspin_speed_increase":0.0,"backspin_assist_enabled":false,"backspin_assist_strength":37}"#,
        )?;
        assert!(!saved.motor_assist_enabled());
        assert!(saved.backspin_speed_increase.abs() < f64::EPSILON);
        assert_eq!(saved.backspin_assist_strength, 37);
        for detection in [false, true] {
            for assist in [false, true] {
                for strength in [0, 37, 100] {
                    let config = Config {
                        backspin_enabled: detection,
                        backspin_assist_enabled: assist,
                        backspin_assist_strength: strength,
                        ..DEFAULT
                    };
                    // When runtime selects whether a motor owner is needed,
                    // then both enables and nonzero strength are necessary.
                    assert_eq!(
                        config.motor_assist_enabled(),
                        detection && assist && strength > 0
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn strength_round_trips_and_rejects_values_above_the_existing_maximum() -> Result<()> {
        // Given a non-default independently selected assist percentage.
        let config: Config = serde_json::from_str(
            r#"{"backspin_assist_enabled":true,"backspin_assist_strength":37,"backspin_speed_increase":5.2}"#,
        )?;
        config.validate()?;
        // When the saved configuration is encoded and decoded.
        let reloaded: Config = serde_json::from_value(serde_json::to_value(config)?)?;
        // Then strength and software boost remain independent, and 101 is rejected.
        assert_eq!(reloaded.backspin_assist_strength, 37);
        assert!((reloaded.backspin_speed_increase - 5.2).abs() < f64::EPSILON);
        let excessive: Config = serde_json::from_str(r#"{"backspin_assist_strength":101}"#)?;
        assert!(excessive.validate().is_err());
        Ok(())
    }

    #[test]
    fn tt_preferences_default_independently_and_round_trip() -> Result<()> {
        // Given legacy preferences with a saved normal-mode Vinyl choice.
        let legacy: Config = serde_json::from_str(r#"{"vinyl_enabled_at_startup":false}"#)?;
        assert!(!legacy.vinyl_enabled_at_startup);
        assert_eq!(legacy.tt_startup_mode, VinylMode::Bend);
        assert_eq!(legacy.tt_pitch_bend_floor_percent, 50);

        // When TT preferences are saved and reloaded.
        let config = Config {
            tt_startup_mode: VinylMode::Scratch,
            tt_pitch_bend_floor_percent: 73,
            ..legacy
        };
        let reloaded: Config = serde_json::from_value(serde_json::to_value(config)?)?;

        // Then each mode's Vinyl preference and the floor remain independent.
        assert!(!reloaded.vinyl_enabled_at_startup);
        assert_eq!(reloaded.tt_startup_mode, VinylMode::Scratch);
        assert_eq!(reloaded.tt_pitch_bend_floor_percent, 73);
        for floor in [1, 95] {
            let bounded = Config {
                tt_pitch_bend_floor_percent: floor,
                ..DEFAULT
            };
            bounded.validate()?;
            assert_eq!(
                serde_json::from_value::<Config>(serde_json::to_value(bounded)?)?
                    .tt_pitch_bend_floor_percent,
                floor
            );
        }
        for input in [
            r#"{"tt_pitch_bend_floor_percent":50.0}"#,
            r#"{"tt_pitch_bend_floor_percent":true}"#,
        ] {
            assert!(serde_json::from_str::<Config>(input).is_err(), "{input}");
        }
        Ok(())
    }

    #[test]
    fn platter_preferences_map_directly_into_shared_settings() {
        let config = Config {
            platter_target_rpm: 40,
            platter_max_force: 19_000,
            platter_base_force: 1_200,
            platter_force_per_rpm: 90,
            platter_kp: 3_500,
            platter_ki: 2_500,
            platter_force_rise_per_second: 100_000,
            platter_startup_ms: 150,
            platter_startup_force_rise_per_second: 5_000,
            platter_overspeed_limit_rpm: 80,
            ..DEFAULT
        };
        let settings = config.platter_settings();

        assert_eq!(settings.count_direction, 1);
        assert_eq!(settings.target_rpm, 40);
        assert_eq!(settings.overspeed_limit_rpm, Some(80));
        assert_eq!(settings.max_force, 19_000);
        assert_eq!(settings.base_force, 1_200);
        assert_eq!(settings.force_per_rpm, 90);
        assert_eq!(settings.kp, 3_500);
        assert_eq!(settings.ki, 2_500);
        assert_eq!(settings.force_rise_per_second, 100_000);
        assert_eq!(settings.startup_ms, 150);
        assert_eq!(settings.startup_force_rise_per_second, 5_000);
    }
}
