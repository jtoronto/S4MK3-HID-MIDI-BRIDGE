//! Native Djay jog mapping preferences, independent of HID translation.

use std::path::Path;

use anyhow::{Context as _, Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub scratch_speed: f64,
    pub scratch_reaction: u8,
    pub pitch_bend_speed: f64,
    pub pitch_bend_reaction: u8,
}

pub const DEFAULT: Config = Config {
    scratch_speed: 2.7,
    scratch_reaction: 150,
    pitch_bend_speed: 2.7,
    pitch_bend_reaction: 17,
};

impl Default for Config {
    fn default() -> Self {
        DEFAULT
    }
}

impl Config {
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
        for (name, reaction) in [
            ("scratch_reaction", self.scratch_reaction),
            ("pitch_bend_reaction", self.pitch_bend_reaction),
        ] {
            ensure!(reaction <= 150, "{name} must be between 0 and 150");
        }
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
        Ok(())
    }
}
