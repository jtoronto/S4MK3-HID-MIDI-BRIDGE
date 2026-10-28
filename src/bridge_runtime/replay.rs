//! Deterministic executable fixture through the production output service.
//! Cached playback permission deliberately survives feedback silence in this
//! testing contract. This replay does not establish disconnect detection or
//! physical motor behavior.

use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

use anyhow::Context as _;
use encdr::{core::descriptor::InputItemDesc, device::loader::DescriptorRegistry};
use s4mk3_hid_midi_bridge::tension::{Config, Engine};
use serde_json::json;

use super::{
    input::Frame,
    paced::{Action, Output},
};
use crate::full;

pub struct Step {
    pub frame: Option<Frame>,
    pub feedback: Vec<u8>,
    pub now_us: u64,
    pub label: &'static str,
}

/// The built-in fixture runs longer than the diagnostic's 15-second pulse cap.
pub fn fixture() -> anyhow::Result<Vec<Step>> {
    let mut registry = DescriptorRegistry::new();
    registry.load_builtins()?;
    let descriptor = crate::full_control_descriptor(
        registry
            .find(crate::S4_VENDOR_ID, crate::S4_PRODUCT_ID)
            .context("S4 descriptor")?,
    )?;
    let fields: Vec<_> = descriptor
        .input_packets
        .iter()
        .flat_map(|packet| &packet.items)
        .filter_map(|item| match item {
            InputItemDesc::Button(button) => {
                Some((button.name.as_str(), button.byte, button.mask.0))
            }
            InputItemDesc::Slider(_)
            | InputItemDesc::Encoder(_)
            | InputItemDesc::EncoderFine(_)
            | InputItemDesc::Touch(_) => None,
        })
        .collect();
    let mut steps = Vec::with_capacity(8202);
    let mut counters = [10_000_u32; 2];
    let mut fractions = [0_u32; 2];
    for index in 0_u32..=8200 {
        let now_us = u64::from(index) * 2000 + 1;
        let mut buffers = [[0; 128]; 3];
        buffers[0][0] = 1;
        buffers[1][0] = 3;
        let mut buttons = Vec::new();
        if (1..=3).contains(&index) {
            buttons.extend(["left_turntable_mode", "right_turntable_mode"]);
        }
        if [65, 85].contains(&index) {
            buttons.push("left_jog_mode");
        }
        if index == 110 {
            buttons.push("left_deck_switch_c");
        }
        if [250, 260].contains(&index) {
            buttons.push("left_turntable_mode");
        }
        for name in buttons {
            let (_, byte, mask) = fields
                .iter()
                .find(|(field, _, _)| *field == name)
                .with_context(|| format!("fixture button {name}"))?;
            buffers[0][byte + 1] |= u8::try_from(*mask)?;
        }
        if (70..80).contains(&index) || (100..141).contains(&index) {
            buffers[0][17] |= 0x10;
        }
        for side in 0..2 {
            fractions[side] += 33 * 2880 * 2000;
            let nominal = fractions[side] / 60_000_000;
            fractions[side] %= 60_000_000;
            let delta = if side == 0 && (100..=200).contains(&index) {
                -10
            } else if side == 0 && (201..220).contains(&index) {
                0
            } else {
                i32::try_from(nominal)? + i32::from(side == 0 && (30..40).contains(&index))
            };
            counters[side] = counters[side].wrapping_add_signed(delta);
            let offset = if side == 0 { 8 } else { 36 };
            buffers[1][offset..offset + 4].copy_from_slice(
                &u32::try_from((now_us * 100) % (u64::from(u32::MAX) + 1))?.to_le_bytes(),
            );
            buffers[1][offset + 4..offset + 8].copy_from_slice(&counters[side].to_le_bytes());
        }
        let (feedback, label) = feedback_step(index);
        steps.push(Step {
            frame: (!(280..311).contains(&index)).then_some(Frame {
                host_us: now_us,
                buffers,
                lengths: [23, 60, 0],
            }),
            feedback,
            now_us,
            label,
        });
    }
    Ok(steps)
}

fn feedback_step(index: u32) -> (Vec<u8>, &'static str) {
    match index {
        0 => (vec![], "startup_motor_off"),
        1 => (vec![], "tt_on_without_feedback"),
        2 => (
            vec![
                0x96, 0, 127, 0x96, 1, 127, 0xb6, 4, 127, 5, 127, 0xb0, 21, 0, 0xb1, 21, 0,
            ],
            "loaded_playing_unknown_internal",
        ),
        5 => (vec![0xb0, 22, 127, 0xb1, 22, 127], "internal_authorized"),
        30 => (vec![], "rim_nudge"),
        45 => (vec![0xb6, 4, 0], "left_pause"),
        55 => (vec![0xb6, 4, 127], "left_resume"),
        65 => (vec![], "vinyl_off"),
        70 => (vec![], "top_touch_no_scratch"),
        85 => (vec![], "vinyl_on"),
        100 => (vec![], "scratch"),
        110 => (
            vec![0x96, 2, 127, 0xb6, 6, 127, 0xb2, 21, 0, 22, 127],
            "held_scratch_select_c",
        ),
        141 => (vec![], "released_backspin"),
        250 => (vec![], "tt_off"),
        260 => (vec![], "tt_on"),
        280 => (vec![], "input_gap"),
        311 => (vec![], "fresh_baseline"),
        400 => (vec![0xb2, 21, 127, 22, 0], "thru"),
        410 => (vec![0xb2, 21, 0, 22, 127], "internal"),
        430 => (vec![0x86, 1, 0], "right_unload"),
        440 => (vec![0x96, 1, 127, 0xb6, 5, 127], "right_reload"),
        470 => (vec![0xb2, 21, 0, 22, 0], "dvs_not_thru"),
        480 => (vec![0xb2, 22, 127], "dvs_to_internal"),
        8000 => (vec![], "continuous_playback_beyond_15_seconds"),
        _ => (vec![], ""),
    }
}

pub fn run(config: full::jog::Config) -> anyhow::Result<()> {
    execute(config, fixture()?, |record| println!("{record}"))
}

pub fn execute(
    config: full::jog::Config,
    steps: Vec<Step>,
    emit: impl Fn(serde_json::Value),
) -> anyhow::Result<()> {
    let replay = RefCell::new(full::replay::Replay::new(config)?);
    let mut engine =
        Engine::with_backspin_strength(Config::default(), config.backspin_assist_strength)?;
    engine.configure_playback(config.platter_settings());
    let now = Cell::new(0);
    let mut midi = Vec::with_capacity(128);
    let mut output = Output::new(|action| {
        match action {
            Action::Motor(report) => {
                emit(json!({"event":"motor","now_us":now.get(),"report":report}));
            }
            Action::Frame(frame, assist) => {
                let mut replay = replay.borrow_mut();
                midi.clear();
                for index in 0..3 {
                    let report = frame.report(index);
                    if !report.is_empty() {
                        replay.report(report, Duration::from_micros(frame.host_us), &mut midi)?;
                    }
                }
                *assist = replay.backspin();
                for message in &midi {
                    emit(json!({"event":"midi","now_us":now.get(),"message":message}));
                }
            }
            Action::Playback(at, state) => {
                midi.clear();
                *state = replay
                    .borrow_mut()
                    .state(Duration::from_micros(at), &mut midi)?;
                for message in &midi {
                    emit(json!({"event":"midi","now_us":at,"message":message}));
                }
            }
            Action::Initialize | Action::Feedback(_) | Action::Screens => {}
        }
        Ok(())
    })?;
    for step in steps {
        now.set(step.now_us);
        // Feedback is decoded before the next motor slot, exactly as live output.
        if !step.feedback.is_empty() {
            replay
                .borrow_mut()
                .feedback(&step.feedback, Duration::from_micros(step.now_us));
        }
        if !step.label.is_empty() {
            emit(json!({"event":"fixture","now_us":step.now_us,"label":step.label}));
        }
        if let Some(frame) = step.frame {
            output.sample(&mut engine, &frame, step.now_us)?;
        }
        output.tick(&mut engine, step.now_us)?;
        if !step.label.is_empty() {
            let local = replay.borrow().leds();
            emit(
                json!({"event":"local","now_us":step.now_us,"motor_enabled":local.motor_enabled,"vinyl":local.vinyl,"decks":local.decks}),
            );
        }
    }
    output.finish()?;
    drop(output);
    midi.clear();
    replay.borrow_mut().finish(&mut midi);
    for message in &midi {
        emit(json!({"event":"midi","now_us":now.get(),"message":message}));
    }
    Ok(())
}
