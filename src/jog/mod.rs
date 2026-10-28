//! Pure snapshot decoding and validated fader-play force law.

pub mod continuity;
pub mod damped;
pub mod decode;
pub mod model;
pub mod platter;
pub mod smooth;
pub mod smooth_wheels;

pub use model::{Decision, FaderState, FaderZone, neutral_report, report_for};
