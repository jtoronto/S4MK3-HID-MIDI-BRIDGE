use std::time::Duration;

use num_traits::ToPrimitive as _;
use s4mk3_hid_midi_bridge::screens::page::TempoPickup;

use super::feedback::FeedbackState;

fn position(raw: f32) -> Option<u8> {
    (raw * 127.0).round().to_u8().filter(|&value| value <= 127)
}

fn target(feedback: &FeedbackState, deck: usize) -> Option<u8> {
    if feedback.loaded[deck] == Some(true) {
        feedback.info.tempo_position[deck]
    } else {
        None
    }
}

// Captured sweeps echo within this window. Qualify only a new mismatch;
// movement cannot interrupt an already-confirmed pickup warning.
const ECHO_GRACE: Duration = Duration::from_millis(75);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Indication {
    #[default]
    Off,
    Center,
    Mismatch(TempoPickup),
}

#[derive(Clone, Copy, Default)]
pub(super) struct Gesture {
    deck: Option<usize>,
    target: Option<u8>,
    physical: Option<u8>,
    shifted: bool,
    origin: Option<u8>,
    pending_since: Option<Duration>,
    indication: Indication,
}

impl Gesture {
    pub(super) fn observe(
        &mut self,
        deck: usize,
        feedback: &FeedbackState,
        raw: Option<f32>,
        shifted: bool,
        now: Duration,
    ) {
        if self.deck != Some(deck) {
            *self = Self {
                deck: Some(deck),
                ..Self::default()
            };
        }
        let physical = raw.and_then(position);
        let target = target(feedback, deck);
        let previous_physical = std::mem::replace(&mut self.physical, physical);
        let target_changed = std::mem::replace(&mut self.target, target) != target;
        let shift_changed = std::mem::replace(&mut self.shifted, shifted) != shifted;

        // At original tempo Djay snaps to center before the S4's emitted CC
        // reaches 64. Captures approach that detent from 61 below and 66 above.
        let centered = matches!((physical, target), (Some(61..=66), Some(63..=64)));
        let pickup = physical
            .zip(target)
            .and_then(|(physical, target)| TempoPickup::new(physical, target))
            .filter(|_| !centered);
        if let Some(pickup) = pickup {
            if shift_changed {
                self.origin = physical;
            } else if self.origin.is_none() {
                self.origin = previous_physical.or(physical);
            }
            let since = if target_changed && !matches!(self.indication, Indication::Mismatch(_)) {
                self.pending_since.insert(now)
            } else {
                self.pending_since.get_or_insert(now)
            };
            if now.saturating_sub(*since) >= ECHO_GRACE
                || matches!(self.indication, Indication::Mismatch(_))
            {
                self.indication = Indication::Mismatch(pickup);
            } else {
                self.indication = Indication::Off;
            }
        } else {
            self.origin = None;
            self.pending_since = None;
            self.indication = if centered {
                Indication::Center
            } else {
                Indication::Off
            };
        }
    }

    pub(super) const fn indication(self) -> Indication {
        self.indication
    }

    pub(super) fn warning(self) -> Option<TempoPickup> {
        match self.indication {
            Indication::Mismatch(pickup)
                if !self.shifted
                    && self
                        .origin
                        .zip(self.physical)
                        .is_some_and(|(origin, physical)| origin.abs_diff(physical) > 1) =>
            {
                Some(pickup)
            }
            _ => None,
        }
    }
}
