//! Hardware-free adapter for the production parser, feedback cache and translator.

use std::time::Duration;

use anyhow::ensure;
use s4mk3_hid_midi_bridge::tension::PlaybackState;

use super::{Control, Translator, feedback, jog, leds, raw};

pub struct Replay {
    parser: raw::Parser,
    translator: Translator,
    feedback: feedback::FeedbackState,
    decoder: feedback::Decoder,
    controls: Vec<Control>,
}

impl Replay {
    pub fn new(config: jog::Config) -> anyhow::Result<Self> {
        config.validate()?;
        Ok(Self {
            parser: raw::Parser::new()?,
            translator: Translator {
                jog_config: config,
                vinyl: [config.vinyl_enabled_at_startup; 2],
                ..Translator::default()
            },
            feedback: feedback::FeedbackState::default(),
            decoder: feedback::Decoder::default(),
            controls: Vec::with_capacity(128),
        })
    }

    pub fn feedback(&mut self, bytes: &[u8], now: Duration) {
        self.decoder.feed(bytes, |message| {
            self.feedback.apply(message, now);
        });
    }

    pub fn state(
        &mut self,
        now: Duration,
        output: &mut Vec<[u8; 3]>,
    ) -> anyhow::Result<PlaybackState> {
        self.translator.sync_front_panel(&self.feedback, output)?;
        self.translator.observe_jog(None, now, output)?;
        Ok(self.translator.playback_state())
    }

    pub fn report(
        &mut self,
        report: &[u8],
        now: Duration,
        output: &mut Vec<[u8; 3]>,
    ) -> anyhow::Result<()> {
        self.controls.clear();
        ensure!(
            self.parser.parse(report, now, &mut self.controls)?,
            "replay expects buttons or jog reports"
        );
        self.translator
            .report(&self.controls, report, now, &self.feedback, output)
    }

    pub fn backspin(&self) -> [bool; 2] {
        self.translator.backspin_assist()
    }

    pub fn leds(&self) -> leds::LocalState {
        self.translator.local_led_state()
    }

    pub fn finish(&mut self, output: &mut Vec<[u8; 3]>) {
        self.translator.release_all(output);
    }
}
