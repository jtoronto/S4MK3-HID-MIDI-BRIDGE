//! One bounded left-wheel run over the same acquired instance as both displays.

use super::{Cancellation, Connector, Error, Raster, Session, cancelled, protocol};
use std::time::Duration;
use tokio::time::{Instant, sleep_until};

const WRITE: Duration = Duration::from_millis(500);
const SCREEN_PAIR: Duration = Duration::from_secs(1);
const FRAME_INTERVAL: Duration = Duration::from_millis(250);
const BASELINE: Duration = Duration::from_secs(2);
const PLAY: Duration = Duration::from_secs(3);

const MOTOR: u32 = 0x036a_736d;
const TICKS: u32 = 0x036a_7378;
const MODE: u32 = 0x036a_734d;
const TIMELINE: u32 = 0x036a_7354;
const RING_COLOR: u32 = 0x036a_7343;
const RING_MODE: u32 = 0x036a_734c;
const TRUE: u32 = 0x7472_7565;
const PAUSE: (u32, u32) = (MOTOR, 0);
// Windows Traktor 4.4.2 build 158, marked run 032a5c32, requests 37, 44, 64.
const IDLE: [(u32, u32); 3] = [
    PAUSE,
    (0x036a_7373, 0x4205_5555),
    (0x036a_7353, 0x4201_1f79),
];
const TT: [(u32, u32); 3] = [(TICKS, 0), (MODE, 2), (TIMELINE, TRUE)];
const JOG: [(u32, u32); 3] = [(TICKS, 0), (MODE, 1), (TIMELINE, TRUE)];
// The NI service maps argument 1 to HID ring mode 2 (needle).
const RING_ON: [(u32, u32); 2] = [(RING_COLOR, 46), (RING_MODE, 1)];
const RING_OFF: [(u32, u32); 2] = [(RING_MODE, 0), (RING_COLOR, 0)];

fn stopping(stop: &Cancellation) -> bool {
    *stop.borrow() || stop.has_changed().is_err()
}

struct ScreenPatterns {
    left: Vec<u16>,
    right: Vec<u16>,
    tick: usize,
}

impl ScreenPatterns {
    fn new() -> Self {
        Self {
            left: vec![0; protocol::PIXELS],
            right: vec![0; protocol::PIXELS],
            tick: 0,
        }
    }

    fn next(&mut self, background: u16) -> Result<[Raster<'_>; 2], Error> {
        self.tick = (self.tick + 1) % 8;
        for (side, buffer) in [(0, &mut self.left), (1, &mut self.right)] {
            buffer.fill(background);
            let column = self.tick * 40;
            for row in buffer.chunks_exact_mut(320) {
                row[column..column + 40].fill(0xffff);
            }
            // Mark the physical screens with distinct bands regardless of phase.
            buffer[..320 * 24].fill(if side == 0 { 0xf800 } else { 0x07ff });
        }
        Ok([Raster::new(&self.left)?, Raster::new(&self.right)?])
    }
}

impl<C: Connector> Session<C> {
    async fn jog_prepare(
        &mut self,
        commands: &[(u32, u32)],
        stop: &Cancellation,
    ) -> Result<(), Error> {
        for &command in commands {
            if stopping(stop) {
                return Err(Error::Cancelled);
            }
            self.scalar_write(0, command).await?;
        }
        Ok(())
    }

    async fn jog_frames(
        &mut self,
        patterns: &mut ScreenPatterns,
        stop: &mut Cancellation,
        end: Instant,
        color: u16,
    ) -> Result<usize, Error> {
        let mut pairs = 0;
        let mut frame_at = Instant::now();
        loop {
            if frame_at + SCREEN_PAIR > end {
                frame_at = end;
            }
            tokio::select! {
                biased;
                () = cancelled(stop) => return Err(Error::Cancelled),
                () = sleep_until(frame_at) => {}
            }
            if stopping(stop) {
                return Err(Error::Cancelled);
            }
            if Instant::now() + SCREEN_PAIR > end {
                if Instant::now() >= end {
                    return Ok(pairs);
                }
                frame_at = end;
                continue;
            }
            self.publish_pair_with_budget(patterns.next(color)?, WRITE)
                .await?;
            pairs += 1;
            frame_at = Instant::now() + FRAME_INTERVAL;
        }
    }

    pub(super) async fn run_left_jog_once(&mut self, mut stop: Cancellation) -> Result<(), Error> {
        let result = self.jog_trial(&mut stop).await;
        self.close();
        println!("NI_JOG_SESSION_CLOSED");
        result
    }

    async fn jog_trial(&mut self, stop: &mut Cancellation) -> Result<(), Error> {
        let mut patterns = ScreenPatterns::new();
        self.jog_prepare(&IDLE, stop).await?;
        let baseline = self
            .jog_frames(&mut patterns, stop, Instant::now() + BASELINE, 0x0010)
            .await?;
        println!("NI_JOG_BASELINE screen_pairs={baseline}");
        if baseline == 0 {
            return Err(Error::Timeout);
        }

        let trial = self.jog_active_trial(&mut patterns, stop).await;
        // Pause and JOG restoration take priority over ring cleanup.
        let clear = async {
            for command in RING_OFF {
                self.scalar_write(0, command).await?;
            }
            Ok(())
        }
        .await;
        match &clear {
            Ok(()) => println!("NI_JOG_RING_OFF_SUBMITTED"),
            Err(error) => eprintln!("NI_JOG_RING_OFF_UNCONFIRMED ({error})"),
        }
        trial.and(clear)
    }

    async fn jog_active_trial(
        &mut self,
        patterns: &mut ScreenPatterns,
        stop: &mut Cancellation,
    ) -> Result<(), Error> {
        // Once TT preparation starts, run the stop sequence on every path.
        let active = async {
            self.jog_prepare(&TT, stop).await?;
            self.jog_prepare(&RING_ON, stop).await?;
            println!("NI_JOG_RING_ON_SUBMITTED");
            let tt = self
                .jog_frames(patterns, stop, Instant::now() + BASELINE, 0x4010)
                .await?;
            println!("NI_JOG_TT_PAUSED screen_pairs={tt}");
            if tt == 0 {
                return Err(Error::Timeout);
            }
            if stopping(stop) {
                return Err(Error::Cancelled);
            }
            let end = Instant::now() + PLAY;
            println!("NI_JOG_PLAY_REQUEST left_rpm=32.28 max_window_ms=3000");
            self.scalar_write(0, (MOTOR, TRUE)).await?;
            println!("NI_JOG_PLAY_SUBMITTED");
            self.jog_frames(patterns, stop, end, 0x0400).await
        }
        .await;
        if let Err(pause_error) = self.scalar_write(0, PAUSE).await {
            eprintln!(
                "NI_JOG_PAUSE_UNCONFIRMED ({pause_error}): switch the S4 off if either platter keeps moving."
            );
            return active.map(|_| ()).and(Err(pause_error));
        }
        println!("NI_JOG_PAUSE_SUBMITTED");
        for command in JOG {
            self.scalar_write(0, command).await?;
        }
        println!("NI_JOG_MODE_RESTORED");
        let play_pairs = active?;
        let stopped_pairs = self
            .jog_frames(patterns, stop, Instant::now() + BASELINE, 0x8200)
            .await?;
        println!("NI_JOG_SCREEN_PAIRS play={play_pairs} after_pause={stopped_pairs}");
        if play_pairs == 0 || stopped_pairs == 0 {
            return Err(Error::Timeout);
        }
        Ok(())
    }
}
