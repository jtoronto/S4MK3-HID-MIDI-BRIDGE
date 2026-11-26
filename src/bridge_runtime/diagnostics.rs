//! Passive attribution of real feedback and local deck/mode selection.

use s4mk3_hid_midi_bridge::screen_motor_diag::{self as diag, Kind};

const fn flag(value: Option<bool>) -> u64 {
    match value {
        None => 0,
        Some(false) => 1,
        Some(true) => 2,
    }
}

pub(super) fn feedback(bridge: &crate::full::FullBridge, now_us: u64) {
    if !diag::enabled() {
        return;
    }
    let local = bridge.local_led_state();
    let feedback = bridge.feedback_state();
    for side in 0..2 {
        let deck = local.decks[side];
        let modes = u64::from(local.vinyl[side])
            | (u64::from(local.motor_enabled[side]) << 1)
            | (u64::from(local.shifted[side]) << 2)
            | (u64::from(local.grid[side]) << 3);
        diag::record(
            Kind::Feedback,
            [
                now_us,
                u64::try_from(side).unwrap_or(0),
                u64::try_from(deck).unwrap_or(0),
                flag(feedback.loaded[deck]),
                flag(feedback.playing[deck]),
                flag(feedback.internal[deck]),
                flag(feedback.thru[deck]),
                modes,
            ],
            &[],
        );
    }
}
