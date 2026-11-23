//! Measured backward motion, separate from MIDI gain and motor control.

use std::collections::VecDeque;
use std::time::Duration;

use s4mk3_hid_midi_bridge::jog::decode::FRESHNESS_US;

const WINDOW: Duration = Duration::from_millis(24);
const FRESHNESS: Duration = Duration::from_micros(FRESHNESS_US);

pub(super) struct Motion {
    samples: VecDeque<(Duration, u32)>,
    latest: Option<(Duration, u32, u32)>,
    delta: i32,
    velocities: VecDeque<(Duration, f64)>,
    detected: bool,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            samples: VecDeque::with_capacity(16),
            latest: None,
            delta: 0,
            velocities: VecDeque::with_capacity(16),
            detected: false,
        }
    }
}

impl Motion {
    pub(super) fn reset_window(&mut self) {
        let velocity = self.latest.and_then(|(time, _, _)| {
            self.window(time)
                .map(|(delta, elapsed)| (time, f64::from(delta) / elapsed.as_secs_f64()))
        });
        self.samples.clear();
        self.velocities.clear();
        self.detected = false;
        if let Some(velocity) = velocity {
            self.velocities.push_back(velocity);
        }
        if let Some((time, counter, _)) = self.latest {
            self.samples.push_back((time, counter));
        }
    }

    pub(super) fn observe(&mut self, counter: u32, timer: u32, now: Duration, acceleration: f64) {
        self.delta = 0;
        if let Some((time, previous, previous_timer)) = self.latest {
            if timer == previous_timer && counter == previous {
                return;
            }
            let delta = i32::from_le_bytes(counter.wrapping_sub(previous).to_le_bytes());
            let timer_advances = timer > previous_timer
                || (previous_timer > u32::MAX - 1_000_000 && timer < 1_000_000);
            if now <= time
                || now - time > FRESHNESS
                || !timer_advances
                || delta.unsigned_abs() > 512
            {
                self.samples.clear();
                self.velocities.clear();
                self.detected = false;
            } else {
                self.delta = delta;
            }
        }
        self.latest = Some((now, counter, timer));
        self.samples.push_back((now, counter));
        while self
            .samples
            .get(1)
            .is_some_and(|(time, _)| now.saturating_sub(*time) >= WINDOW)
        {
            self.samples.pop_front();
        }
        if let Some((delta, elapsed)) = self.window(now) {
            let velocity = f64::from(delta) / elapsed.as_secs_f64();
            while self
                .velocities
                .get(1)
                .is_some_and(|(time, _)| now.saturating_sub(*time) >= WINDOW)
            {
                self.velocities.pop_front();
            }
            if let Some(&(time, previous)) = self.velocities.front() {
                let elapsed = now.saturating_sub(time);
                if elapsed >= WINDOW
                    && velocity < 0.0
                    && (velocity - previous) / elapsed.as_secs_f64() <= -acceleration
                {
                    self.detected = true;
                }
            }
            self.velocities.push_back((now, velocity));
        }
        if !self.continues(now) {
            self.detected = false;
        }
    }

    fn window(&self, now: Duration) -> Option<(i32, Duration)> {
        let (latest, counter, _) = self.latest?;
        let &(first, previous) = self.samples.front()?;
        let elapsed = latest.checked_sub(first)?;
        (now.checked_sub(latest)? <= FRESHNESS && elapsed >= WINDOW).then(|| {
            (
                i32::from_le_bytes(counter.wrapping_sub(previous).to_le_bytes()),
                elapsed,
            )
        })
    }

    pub(super) fn qualifies(&self, now: Duration) -> bool {
        self.detected && self.continues(now)
    }

    #[cfg(any(windows, test))]
    pub(super) fn forward_or_rest(&self, now: Duration) -> bool {
        self.window(now).is_some_and(|(delta, _)| delta >= -2)
    }

    pub(super) fn continues(&self, now: Duration) -> bool {
        // Two counts are the captured near-rest sensor chatter. A single
        // zero/duplicate sample also occurs while the platter is still moving.
        self.delta <= 0 && self.window(now).is_some_and(|(delta, _)| delta < -2)
    }
}
