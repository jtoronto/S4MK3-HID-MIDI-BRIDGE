//! Adapt held MIDI owners and confirmed Djay state to the hardware screens.

use s4mk3_hid_midi_bridge::screens::page::{FxWarning, Info, PadState, Page, TempoPickup};

use super::{Address, Translator, catalog::PadMode, feedback::FeedbackState};

mod key;

pub struct Output {
    enabled: bool,
    managed: bool,
    worker: Option<s4mk3_hid_midi_bridge::screens::Worker>,
}

impl Output {
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled,
            managed: false,
            worker: None,
        }
    }

    pub fn managed(enabled: bool) -> anyhow::Result<Self> {
        Ok(Self {
            enabled,
            managed: true,
            worker: Some(s4mk3_hid_midi_bridge::screens::Worker::start_managed(
                enabled,
            )?),
        })
    }

    #[cfg(windows)]
    pub fn ni_decks() -> anyhow::Result<Self> {
        Ok(Self {
            enabled: true,
            managed: true,
            worker: Some(s4mk3_hid_midi_bridge::screens::Worker::start_ni_decks()?),
        })
    }

    #[cfg(windows)]
    pub fn stop_ni_decks(&mut self) -> anyhow::Result<()> {
        use anyhow::Context as _;
        self.worker
            .as_mut()
            .context("NI deck worker missing at shutdown")?
            .stop_ni_decks()
    }

    pub fn control(&self) -> Option<s4mk3_hid_midi_bridge::screens::Control> {
        self.worker
            .as_ref()
            .filter(|_| self.managed)
            .map(s4mk3_hid_midi_bridge::screens::Worker::control)
    }

    /// Publication follows initialized input and a completed native output tick.
    pub fn publish(&mut self, pages: [Page; 2]) {
        if self.managed {
            if let Some(worker) = &mut self.worker {
                worker.publish(pages);
            }
            return;
        }
        if !self.enabled {
            return;
        }
        if self.worker.is_none() {
            match s4mk3_hid_midi_bridge::screens::Worker::start() {
                Ok(worker) => self.worker = Some(worker),
                Err(error) => {
                    tracing::warn!(%error, "SCREENS_DISABLED: could not start screen worker");
                    self.enabled = false;
                    return;
                }
            }
        }
        if let Some(worker) = &mut self.worker {
            worker.publish(pages);
        }
    }

    /// Feedback can refresh an existing worker, but cannot start USB early.
    pub fn refresh(&mut self, pages: [Page; 2]) {
        if let Some(worker) = &mut self.worker {
            worker.refresh(pages);
        }
    }
}

pub(super) fn pages(translator: &Translator, feedback: &FeedbackState) -> [Page; 2] {
    let local = translator.local_led_state();
    std::array::from_fn(|side| {
        let deck = translator.decks[side].channel();
        let mut fx = FxWarning {
            deck,
            ..FxWarning::default()
        };
        for strip in 0..2 {
            if local.fx[strip] != usize::from(deck) {
                continue;
            }
            for (knob, mismatch) in super::leds::fx_pickup(feedback, &local, strip)
                .into_iter()
                .enumerate()
            {
                match mismatch {
                    Some(true) => fx.mismatch[strip] |= 1 << knob,
                    None if local.fx_positions[strip][knob].is_some() => {
                        fx.unknown[strip] |= 1 << knob;
                    }
                    Some(false) | None => {}
                }
            }
        }
        let backspin = translator.backspin_deck(side);
        if let Some(slot) = local.stem_fx_slot[side] {
            let routing = feedback.stem_fx[usize::from(deck)];
            return Page::StemFx {
                deck,
                slot,
                layout: routing.layout(),
                routes: routing.routes[usize::from(slot)],
                fx,
                backspin,
            };
        }
        match translator.pads[side] {
            PadMode::Hotcue | PadMode::Samples => {
                let deck = usize::from(translator.decks[side].channel());
                Page::Info(Info {
                    jump: feedback.info.jump_size(deck),
                    loop_active: feedback.notes[deck][6].map(|value| value != 0),
                    loop_size: feedback.info.active_loop_size(deck),
                    key: feedback.info.key_raw[deck].and_then(key::label),
                    tempo_range: feedback.info.tempo_range,
                    tempo_pickup: translator.tempo_gesture[side].warning(),
                    selecting: translator.move_selecting[deck],
                    fx,
                    backspin,
                })
            }
            PadMode::Stems => {
                let deck = translator.decks[side];
                let notes = &feedback.notes[usize::from(deck.channel())];
                let shifted = translator.shifted[side];
                let solo: [Option<bool>; 4] =
                    std::array::from_fn(|index| notes[60 + index].map(|value| value != 0));
                Page::Stems {
                    shifted,
                    fx,
                    backspin,
                    pads: std::array::from_fn(|index| {
                        let number = [56_u8, 57, 58, 59, 60, 61, 62, 63][index];
                        let exclusive = shifted && index < 4;
                        let owner = Address {
                            channel: deck.channel() + if exclusive { 8 } else { 0 },
                            number,
                        };
                        PadState {
                            active: if exclusive {
                                solo.iter()
                                    .enumerate()
                                    .try_fold(true, |only, (stem, state)| {
                                        state.map(|active| only && active == (stem == index))
                                    })
                            } else {
                                notes[usize::from(number)].map(|value| value != 0)
                            },
                            held: translator
                                .held
                                .values()
                                .flatten()
                                .any(|address| *address == owner),
                        }
                    }),
                }
            }
        }
    })
}

pub fn write_previews(path: &std::path::Path) -> anyhow::Result<()> {
    use s4mk3_hid_midi_bridge::screens::{Frame, page};
    std::fs::create_dir_all(path)?;
    let mut frame = Frame::default();
    for (deck_index, deck) in [
        super::Deck::A,
        super::Deck::B,
        super::Deck::C,
        super::Deck::D,
    ]
    .into_iter()
    .enumerate()
    {
        for (mode, name) in [
            (PadMode::Hotcue, "hotcue"),
            (PadMode::Samples, "samples"),
            (PadMode::Stems, "stems"),
        ] {
            for shifted in [false, true] {
                let translator = Translator {
                    decks: [deck; 2],
                    pads: [mode; 2],
                    shifted: [shifted; 2],
                    ..Translator::default()
                };
                page::render(
                    &pages(&translator, &FeedbackState::default())[0],
                    &mut frame,
                );
                frame.write_ppm(
                    &path.join(format!("deck-{deck_index}-{name}-shift-{shifted}.ppm")),
                )?;
            }
        }
    }
    for (name, muted, solo, shifted, held, known) in [
        ("stems-off", 0, 0, false, false, true),
        ("stems-mixed", 1, 2, false, false, true),
        ("stems-only", 1, 2, true, false, true),
        ("stems-multiple-solos", 0, 3, true, false, true),
        ("stems-held-unknown", 0, 0, false, true, false),
        ("stems-held-on", 4, 0, false, true, true),
    ] {
        let mut translator = Translator {
            pads: [PadMode::Stems; 2],
            shifted: [shifted; 2],
            ..Translator::default()
        };
        let mut feedback = FeedbackState::default();
        if known {
            for pad in 0_u8..8 {
                feedback.apply(
                    super::feedback::Message {
                        status: 0x90,
                        number: 56 + pad,
                        value: if (if pad < 4 { muted } else { solo }) & (1 << (pad % 4)) != 0 {
                            127
                        } else {
                            0
                        },
                    },
                    std::time::Duration::ZERO,
                );
            }
        }
        if held {
            translator.translate(
                &[super::Control::Button("left_pad_3", true)],
                None,
                &mut Vec::new(),
            )?;
        }
        page::render(&pages(&translator, &feedback)[0], &mut frame);
        frame.write_ppm(&path.join(format!("{name}.ppm")))?;
    }
    write_info_previews(path, &mut frame)?;
    write_stem_fx_previews(path, &mut frame)?;
    write_tempo_pickup_previews(path, &mut frame)?;
    write_fx_previews(path, &mut frame)?;
    write_backspin_previews(path, &mut frame)?;
    page::stopped(&mut frame);
    frame.write_ppm(&path.join("stopped.ppm"))?;
    Ok(())
}

fn write_stem_fx_previews(
    path: &std::path::Path,
    frame: &mut s4mk3_hid_midi_bridge::screens::Frame,
) -> anyhow::Result<()> {
    use s4mk3_hid_midi_bridge::screens::page;
    for (name, layout, routes) in [
        (
            "stem-fx-two-stems",
            Some(2),
            [Some(false), Some(true), Some(true), None, None],
        ),
        (
            "stem-fx-four-stems",
            Some(4),
            [Some(false), Some(true), Some(false), None, Some(true)],
        ),
        ("stem-fx-unknown", None, [None; 5]),
    ] {
        page::render(
            &Page::StemFx {
                deck: 0,
                slot: 1,
                layout,
                routes,
                fx: FxWarning::default(),
                backspin: (layout == Some(4)).then_some(0),
            },
            frame,
        );
        frame.write_ppm(&path.join(format!("{name}.ppm")))?;
    }
    Ok(())
}

fn write_backspin_previews(
    path: &std::path::Path,
    frame: &mut s4mk3_hid_midi_bridge::screens::Frame,
) -> anyhow::Result<()> {
    use s4mk3_hid_midi_bridge::screens::page;
    for (name, mode, has_fx) in [
        ("backspin-info", PadMode::Hotcue, false),
        ("backspin-stems", PadMode::Stems, false),
        ("backspin-fx-info", PadMode::Hotcue, true),
        ("backspin-fx-stems", PadMode::Stems, true),
    ] {
        let mut translator = Translator {
            decks: [super::Deck::C, super::Deck::B],
            fx: [super::Deck::A; 2],
            fx_positions: [[Some(0); 4]; 2],
            pads: [mode; 2],
            ..Translator::default()
        };
        translator.move_selecting[0] = true;
        let mut report = [0_u8; 60];
        report[0] = 3;
        let mut snap = |state: &mut Translator, time_ms: u64, counter: u32| -> anyhow::Result<()> {
            for offset in [8, 36] {
                report[offset..offset + 4]
                    .copy_from_slice(&u32::try_from(time_ms + 1)?.to_le_bytes());
            }
            report[12..16].copy_from_slice(&counter.to_le_bytes());
            state.observe_jog(
                Some(&report),
                std::time::Duration::from_millis(time_ms),
                &mut Vec::new(),
            )
        };
        snap(&mut translator, 0, 1_000)?;
        translator.translate(
            &[super::Control::Button("left_wheel_touch", true)],
            None,
            &mut Vec::new(),
        )?;
        for time in [10, 20, 30] {
            snap(&mut translator, time, 1_000)?;
        }
        for time in [40, 50, 60] {
            snap(
                &mut translator,
                time,
                1_000 - u32::try_from(time - 30)? * 10,
            )?;
            translator.translate(
                &[super::Control::Jog("left_jog_wheel", -0.1)],
                None,
                &mut Vec::new(),
            )?;
        }
        translator.translate(
            &[
                super::Control::Button("left_wheel_touch", false),
                super::Control::Button("left_deck_switch_a", true),
            ],
            None,
            &mut Vec::new(),
        )?;
        snap(&mut translator, 70, 600)?;
        let mut feedback = FeedbackState::default();
        feedback.info.key_raw[0] = Some(81);
        feedback.info.tempo_range = Some(8);
        feedback.notes[0][6] = Some(127);
        for (index, state) in feedback.notes[0][56..64].iter_mut().enumerate() {
            *state = Some(if index % 2 == 0 { 127 } else { 0 });
        }
        for number in [86, 98] {
            feedback.apply(
                super::feedback::Message {
                    status: 0xb0,
                    number,
                    value: 127,
                },
                std::time::Duration::ZERO,
            );
        }
        if has_fx {
            feedback.fx_values[0] = [Some(127); 6];
        } else {
            translator.fx = [super::Deck::B; 2];
        }
        page::render(&pages(&translator, &feedback)[0], frame);
        frame.write_ppm(&path.join(format!("{name}.ppm")))?;
    }
    Ok(())
}

fn write_fx_previews(
    path: &std::path::Path,
    frame: &mut s4mk3_hid_midi_bridge::screens::Frame,
) -> anyhow::Result<()> {
    use s4mk3_hid_midi_bridge::screens::page;
    for (name, mode) in [("info", PadMode::Hotcue), ("stems", PadMode::Stems)] {
        for known in [false, true] {
            let mut translator = Translator {
                fx: [super::Deck::A; 2],
                fx_positions: [[Some(0); 4]; 2],
                pads: [mode; 2],
                ..Translator::default()
            };
            translator.move_selecting[0] = true;
            let mut feedback = FeedbackState::default();
            feedback.info.key_raw[0] = Some(117);
            feedback.info.tempo_range = Some(75);
            feedback.notes[0][6] = Some(127);
            feedback.notes[0][56..64].fill(Some(0));
            for number in [80, 90] {
                feedback.apply(
                    super::feedback::Message {
                        status: 0xb0,
                        number,
                        value: 127,
                    },
                    std::time::Duration::ZERO,
                );
            }
            if known {
                feedback.fx_values[0] = [Some(127); 6];
            }
            for bright in [false, true] {
                page::render_phase(&pages(&translator, &feedback)[0], frame, bright);
                frame.write_ppm(
                    &path.join(format!("fx-{name}-known-{known}-bright-{bright}.ppm")),
                )?;
            }
        }
    }
    Ok(())
}

fn write_info_previews(
    path: &std::path::Path,
    frame: &mut s4mk3_hid_midi_bridge::screens::Frame,
) -> anyhow::Result<()> {
    use s4mk3_hid_midi_bridge::screens::page;
    for (name, selecting, bright, active, long) in [
        ("info-inactive", false, true, false, false),
        ("info-active", false, true, true, false),
        ("info-select-bright", true, true, false, false),
        ("info-select-dim", true, false, false, false),
        ("info-long", false, true, true, true),
    ] {
        let mut translator = Translator::default();
        translator.move_selecting[0] = selecting;
        let mut feedback = FeedbackState::default();
        for (status, number, value) in [
            (0xb0, if long { 80 } else { 86 }, 127),
            (0x90, 6, if active { 127 } else { 0 }),
            (0xb0, if long { 90 } else { 98 }, 127),
            (0xb0, 112, if long { 46 } else { 71 }),
            (0xb6, 8, if long { 95 } else { 5 }),
        ] {
            feedback.apply(
                super::feedback::Message {
                    status,
                    number,
                    value,
                },
                std::time::Duration::ZERO,
            );
        }
        page::render_phase(&pages(&translator, &feedback)[0], frame, bright);
        frame.write_ppm(&path.join(format!("{name}.ppm")))?;
    }
    Ok(())
}

fn write_tempo_pickup_previews(
    path: &std::path::Path,
    frame: &mut s4mk3_hid_midi_bridge::screens::Frame,
) -> anyhow::Result<()> {
    use s4mk3_hid_midi_bridge::screens::page;
    for (name, physical, target, warnings) in [
        ("tempo-pickup-right", 36, 92, false),
        ("tempo-pickup-left", 114, 23, true),
        ("tempo-pickup-acquired", 92, 92, false),
    ] {
        let info = Info {
            jump: Some("8"),
            loop_active: Some(false),
            key: Some("9A"),
            tempo_range: Some(4),
            tempo_pickup: TempoPickup::new(physical, target),
            backspin: warnings.then_some(0),
            fx: FxWarning {
                mismatch: [u8::from(warnings), 0],
                ..FxWarning::default()
            },
            ..Info::default()
        };
        page::render(&Page::Info(info), frame);
        frame.write_ppm(&path.join(format!("{name}.ppm")))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
