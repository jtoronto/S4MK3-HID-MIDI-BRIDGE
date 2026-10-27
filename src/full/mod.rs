//! Full input translation, isolated from the verified minimal MIDI profile.

mod backspin;
pub mod catalog;
pub mod feedback;
pub mod jog;
pub mod leds;
pub mod mapping;
pub mod output;
pub mod screens;

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context as _, bail, ensure};
use encdr::Event;
use midir::os::unix::VirtualOutput as _;
use midir::{MidiOutput, MidiOutputConnection};
use num_traits::ToPrimitive as _;
use tracing::{debug, info, warn};

use catalog::{Binding, Kind, PadMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Deck {
    A,
    B,
    C,
    D,
}

impl Deck {
    const fn channel(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Control {
    Button(&'static str, bool),
    Slider(&'static str, f32),
    Encoder(&'static str, i32),
    Jog(&'static str, f64),
}

impl Control {
    fn from_event(event: &Event) -> Option<Self> {
        match event {
            Event::Button { name, pressed, .. } => Some(Self::Button(name, *pressed)),
            Event::Touch { name, touched, .. } => Some(Self::Button(name, *touched)),
            Event::Slider { name, value, .. } => Some(Self::Slider(name, *value)),
            Event::Encoder { name, delta, .. } => Some(Self::Encoder(name, *delta)),
            Event::EncoderFine { name, delta, .. } => Some(Self::Jog(name, f64::from(*delta))),
            Event::DeviceConnected { .. }
            | Event::DeviceDisconnected { .. }
            | Event::Grid { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Address {
    channel: u8,
    number: u8,
}

impl Address {
    const fn press(self) -> [u8; 3] {
        [0x90 | self.channel, self.number, 127]
    }

    const fn release(self) -> [u8; 3] {
        [0x80 | self.channel, self.number, 0]
    }
}

#[derive(Clone, Copy, Debug)]
struct WheelOwner {
    deck: Deck,
    action: &'static str,
    coasting: bool,
}

#[derive(Clone, Copy, Default)]
struct FxPickup {
    acquired: bool,
    /// Ordered-feedback window for our own outstanding values: low, high, latest.
    pending: Option<(u8, u8, u8)>,
}

impl FxPickup {
    fn observe(&mut self, old: Option<u8>, value: Option<u8>, physical: Option<u8>) {
        if let Some((low, high, latest)) = self.pending {
            if value == Some(latest) {
                self.pending = None;
            } else if value.is_some_and(|value| !(low..=high).contains(&value)) {
                *self = Self::default();
            }
        } else if value != old
            && value
                .zip(physical)
                .is_none_or(|(value, physical)| value.abs_diff(physical) > 1)
        {
            self.acquired = false;
        }
    }

    fn accepts(&mut self, previous: Option<u8>, value: u8, software: Option<u8>) -> bool {
        let Some(software) = software else {
            return false;
        };
        if !self.acquired {
            self.acquired = value.abs_diff(software) <= 1
                || previous.is_some_and(|previous| {
                    (previous.min(value)..=previous.max(value)).contains(&software)
                });
        }
        if self.acquired {
            let (low, high, _) = self.pending.unwrap_or((software, software, software));
            self.pending = Some((low.min(value), high.max(value), value));
        }
        self.acquired
    }
}

struct Translator {
    jog_config: jog::Config,
    decks: [Deck; 2],
    shifted: [bool; 2],
    pads: [PadMode; 2],
    fx: [Deck; 2],
    quick: Deck,
    vinyl: [bool; 2],
    grid: [bool; 2],
    wheels: [Option<WheelOwner>; 2],
    wheel_motion: [backspin::Motion; 2],
    jog_now: Duration,
    residuals: [f64; 2],
    held: HashMap<&'static str, Vec<Address>>,
    switches: [Option<u8>; 5],
    delivered_switches: [Option<u8>; 5],
    front_panel_ready: bool,
    led_held: [bool; 11],
    tempos: [Option<f32>; 2],
    move_selecting: [bool; 4],
    jump_sizes: [Option<usize>; 4],
    jump_revision: [u64; 4],
    jump_restore: [Option<u64>; 4],
    mixer_positions: [Option<u8>; 16],
    delivered_mixer: [Option<u8>; 16],
    fx_positions: [[Option<u8>; 4]; 2],
    fx_values: [[Option<u8>; 6]; 4],
    fx_pickup: [[FxPickup; 6]; 2],
    fx_browse_anchors: [[Option<u8>; 3]; 2],
    eq_selecting: bool,
    thru: [Option<bool>; 4],
}

impl Default for Translator {
    fn default() -> Self {
        Self {
            jog_config: jog::DEFAULT,
            decks: [Deck::A, Deck::B],
            shifted: [false; 2],
            pads: [PadMode::Hotcue; 2],
            fx: [Deck::A, Deck::B],
            quick: Deck::A,
            vinyl: [true; 2],
            grid: [false; 2],
            wheels: [None; 2],
            wheel_motion: std::array::from_fn(|_| backspin::Motion::default()),
            jog_now: Duration::ZERO,
            residuals: [0.0; 2],
            held: HashMap::new(),
            switches: [None; 5],
            delivered_switches: [None; 5],
            front_panel_ready: false,
            led_held: [false; 11],
            tempos: [None; 2],
            move_selecting: [false; 4],
            jump_sizes: [None; 4],
            jump_revision: [0; 4],
            jump_restore: [None; 4],
            mixer_positions: [None; 16],
            delivered_mixer: [None; 16],
            fx_positions: [[None; 4]; 2],
            fx_values: [[None; 6]; 4],
            fx_pickup: [[FxPickup::default(); 6]; 2],
            fx_browse_anchors: [[None; 3]; 2],
            eq_selecting: false,
            thru: [None; 4],
        }
    }
}

pub struct FullBridge {
    feedback_input: feedback::FeedbackInput,
    feedback: feedback::FeedbackState,
    connection: MidiOutputConnection,
    translator: Translator,
    controls: Vec<Control>,
    messages: Vec<[u8; 3]>,
    screens: screens::Output,
}

impl FullBridge {
    pub fn new(
        screen_output: bool,
        screen_control: bool,
        jog_config: jog::Config,
    ) -> anyhow::Result<Self> {
        jog_config.validate()?;
        // Djay can return initial state as soon as the source appears.
        let feedback_input = feedback::FeedbackInput::new(mapping::PORT_NAME)?;
        let connection = MidiOutput::new("S4 MK3 full bridge")?
            .create_virtual(mapping::PORT_NAME)
            .map_err(|error| anyhow::anyhow!("create full MIDI source: {error}"))?;
        info!(port = mapping::PORT_NAME, "MIDI_FULL_READY");
        Ok(Self {
            feedback_input,
            feedback: feedback::FeedbackState::default(),
            connection,
            translator: Translator {
                jog_config,
                ..Translator::default()
            },
            controls: Vec::with_capacity(160),
            messages: Vec::with_capacity(160),
            screens: if screen_control {
                screens::Output::managed(screen_output)?
            } else {
                screens::Output::new(screen_output)
            },
        })
    }

    pub fn screen_control(&self) -> Option<s4mk3_hid_midi_bridge::screens::Control> {
        self.screens.control()
    }

    pub fn drain_feedback(&mut self, now: Duration) -> anyhow::Result<(usize, usize)> {
        let received = self.feedback_input.drain(&mut self.feedback, now)?;
        self.messages.clear();
        self.translator.observe_jog(None, now, &mut self.messages)?;
        self.translator
            .sync_front_panel(&self.feedback, &mut self.messages)?;
        let sent = self.send_messages()?;
        if received != 0 {
            self.screens
                .refresh(screens::pages(&self.translator, &self.feedback));
        }
        Ok((received, sent))
    }

    pub const fn feedback_state(&self) -> &feedback::FeedbackState {
        &self.feedback
    }

    pub fn local_led_state(&self) -> leds::LocalState {
        self.translator.local_led_state()
    }

    pub fn backspin_assist(&self) -> [bool; 2] {
        self.translator.backspin_assist()
    }

    pub fn seed_analog(&mut self, events: &[Event]) -> anyhow::Result<()> {
        self.controls.clear();
        self.controls
            .extend(events.iter().filter_map(Control::from_event));
        self.translator.observe_analog(&self.controls)
    }

    pub fn send_report(
        &mut self,
        events: &[Event],
        report: &[u8],
        now: Duration,
    ) -> anyhow::Result<(usize, usize)> {
        let received = self.feedback_input.drain(&mut self.feedback, now)?;
        self.controls.clear();
        self.controls
            .extend(events.iter().filter_map(Control::from_event));
        self.messages.clear();
        self.translator
            .observe_jog(Some(report), now, &mut self.messages)?;
        let buttons = (report.first() == Some(&1))
            .then(|| {
                report[1..]
                    .try_into()
                    .context("full button payload must have 22 bytes")
            })
            .transpose()?;
        self.translator
            .sync_front_panel(&self.feedback, &mut self.messages)?;
        self.translator
            .translate(&self.controls, buttons, &mut self.messages)?;
        Ok((received, self.send_messages()?))
    }

    pub fn publish_screens(&mut self) {
        self.screens
            .publish(screens::pages(&self.translator, &self.feedback));
    }

    fn send_messages(&mut self) -> anyhow::Result<usize> {
        for message in &self.messages {
            self.connection
                .send(message)
                .context("send full-profile MIDI")?;
        }
        if !self.messages.is_empty() {
            debug!(messages = self.messages.len(), "MIDI_FULL_SENT");
        }
        Ok(self.messages.len())
    }
}

impl Drop for FullBridge {
    fn drop(&mut self) {
        self.messages.clear();
        self.translator.release_all(&mut self.messages);
        for message in &self.messages {
            if let Err(error) = self.connection.send(message) {
                warn!(%error, "could not release full-profile MIDI note");
            }
        }
    }
}

impl Translator {
    fn observe_jog(
        &mut self,
        report: Option<&[u8]>,
        now: Duration,
        output: &mut Vec<[u8; 3]>,
    ) -> anyhow::Result<()> {
        let previous = [self.backspin_deck(0), self.backspin_deck(1)];
        self.jog_now = now;
        if let Some(report) = report
            && report.first() == Some(&3)
        {
            ensure!(report.len() == 60, "full jog report must have 60 bytes");
            for (side, offset) in [(0, 8), (1, 36)] {
                let timer = u32::from_le_bytes(report[offset..offset + 4].try_into()?);
                let counter = u32::from_le_bytes(report[offset + 4..offset + 8].try_into()?);
                self.wheel_motion[side].observe(
                    counter,
                    timer,
                    now,
                    self.jog_config.backspin_acceleration,
                );
            }
        }
        for (side, previous) in previous.into_iter().enumerate() {
            if self.wheels[side].is_some_and(|owner| owner.coasting)
                && !self.wheel_motion[side].continues(now)
            {
                self.end_backspin(side, output);
            }
            let active = self.backspin_deck(side);
            if active != previous {
                self.residuals[side] = 0.0;
                info!(
                    side,
                    deck = active.map(|deck| deck + 1),
                    active = active.is_some(),
                    "BACKSPIN_STATE"
                );
            }
        }
        Ok(())
    }

    fn backspin_deck(&self, side: usize) -> Option<u8> {
        let owner = self.wheels[side]?;
        (self.jog_config.backspin_enabled
            && owner.action == "jog_scratch"
            && !self.grid[side]
            && self.vinyl[side]
            && self.wheel_motion[side].qualifies(self.jog_now))
        .then_some(owner.deck.channel())
    }

    fn backspin_assist(&self) -> [bool; 2] {
        std::array::from_fn(|side| {
            self.jog_config.motor_assist_enabled() && self.backspin_deck(side).is_some()
        })
    }

    fn end_backspin(&mut self, side: usize, output: &mut Vec<[u8; 3]>) {
        self.release(
            if side == 0 {
                "left_wheel_touch"
            } else {
                "right_wheel_touch"
            },
            output,
        );
        self.wheels[side] = None;
        self.residuals[side] = 0.0;
        self.wheel_motion[side].reset_window();
        info!(side, "BACKSPIN_ENDED");
    }

    fn observe_analog(&mut self, controls: &[Control]) -> anyhow::Result<()> {
        for control in controls {
            if let Control::Slider(name, value) = *control {
                ensure!(
                    (0.0..=1.0).contains(&value),
                    "invalid full-profile {name}: {value}"
                );
                if let Some(binding) = catalog::MIXER_BINDINGS
                    .iter()
                    .find(|binding| binding.input == name && (4..20).contains(&binding.number))
                {
                    self.mixer_positions[usize::from(binding.number - 4)] = Some(
                        (value * 127.0)
                            .round()
                            .to_u8()
                            .context("quantize mixer position")?,
                    );
                }
                if let Some((side, suffix)) = name
                    .strip_prefix("fx1_")
                    .map(|suffix| (0, suffix))
                    .or_else(|| name.strip_prefix("fx2_").map(|suffix| (1, suffix)))
                {
                    let knob = match suffix {
                        "knob_1" => Some(0),
                        "knob_2" => Some(1),
                        "knob_3" => Some(2),
                        "dry_wet" => Some(3),
                        _ => None,
                    };
                    if let Some(knob) = knob {
                        self.fx_positions[side][knob] = Some(
                            (value * 127.0)
                                .round()
                                .to_u8()
                                .context("quantize FX position")?,
                        );
                    }
                }
            }
        }
        Ok(())
    }

    fn emit_mixer(&mut self, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        if self.front_panel_ready {
            for (index, position) in self.mixer_positions.iter().copied().enumerate() {
                if let Some(value) = position
                    && self.delivered_mixer[index] != position
                {
                    output.push([0xb4, 4 + u8::try_from(index)?, value]);
                    self.delivered_mixer[index] = position;
                }
            }
        }
        Ok(())
    }

    fn local_led_state(&self) -> leds::LocalState {
        leds::LocalState {
            decks: self.decks.map(|deck| usize::from(deck.channel())),
            shifted: self.shifted,
            pads: self.pads,
            fx: self.fx.map(|deck| usize::from(deck.channel())),
            quick: usize::from(self.quick.channel()),
            vinyl: self.vinyl,
            grid: self.grid,
            held: self.led_held,
            tempo: self.tempos,
            move_selecting: self.move_selecting,
            eq_selecting: self.eq_selecting,
            fx_positions: self.fx_positions,
            fx_acquired: self
                .fx_pickup
                .map(|strip| strip.map(|pickup| pickup.acquired)),
        }
    }

    fn observe_led_inputs(&mut self, controls: &[Control]) {
        const MOMENTARY: [&str; 11] = [
            "left_record_mode",
            "left_library_view",
            "left_library_playlist",
            "left_library_star",
            "left_library_play",
            "right_record_mode",
            "right_library_view",
            "right_library_playlist",
            "right_library_star",
            "right_library_play",
            "mixer_fx_filter",
        ];
        for control in controls {
            match *control {
                Control::Button(name, held) => {
                    if let Some(index) = MOMENTARY.iter().position(|expected| *expected == name) {
                        self.led_held[index] = held;
                    }
                }
                Control::Slider("left_tempo_fader", value) => self.tempos[0] = Some(value),
                Control::Slider("right_tempo_fader", value) => self.tempos[1] = Some(value),
                Control::Slider(_, _) | Control::Encoder(_, _) | Control::Jog(_, _) => {}
            }
        }
    }

    fn translate(
        &mut self,
        controls: &[Control],
        buttons: Option<&[u8; 22]>,
        output: &mut Vec<[u8; 3]>,
    ) -> anyhow::Result<()> {
        let previous_fx = self.fx_positions;
        let previous_targets = self.fx;
        let previous_shift = self.shifted;
        self.observe_analog(controls)?;
        self.observe_led_inputs(controls);
        // All releases precede selectors; selectors precede new presses/movement.
        // Thus descriptor item order cannot choose a simultaneous Shift+Play route.
        for control in controls {
            if let Control::Button(name, false) = control {
                if let Some((side, "wheel_touch")) = side_name(name)
                    && let Some(owner) = &mut self.wheels[side]
                    && (owner.coasting
                        || (self.jog_config.backspin_enabled
                            && owner.action == "jog_scratch"
                            && !self.grid[side]
                            && self.vinyl[side]
                            && self.wheel_motion[side].qualifies(self.jog_now)))
                {
                    if !owner.coasting {
                        owner.coasting = true;
                        info!(side, deck = owner.deck.channel() + 1, "BACKSPIN_COAST");
                    }
                    continue;
                }
                self.release(name, output);
            }
        }
        if pressed(controls, "mixer_fx_filter") && !self.held.contains_key("mixer_fx_filter") {
            self.eq_selecting = !self.eq_selecting;
            self.held.insert("mixer_fx_filter", Vec::new());
        }
        self.selectors(controls, output);
        for (side, previous) in previous_targets.into_iter().enumerate() {
            if self.fx[side] != previous {
                self.fx_pickup[side] = [FxPickup::default(); 6];
            }
            if self.fx[side] != previous || self.shifted[side] != previous_shift[side] {
                self.fx_browse_anchors[side] = std::array::from_fn(|slot| previous_fx[side][slot]);
            }
        }
        self.move_selection(controls);
        // Preview must be held before a simultaneous Browse turn, regardless
        // of the descriptor's item order.
        for control in controls {
            if let Control::Button(name, true) = control
                && is_preview(name)
            {
                self.press(name, output)?;
            }
        }
        if let Some(payload) = buttons {
            self.front_panel(payload, output)?;
        }
        for control in controls {
            match *control {
                Control::Button(name, true) => self.press(name, output)?,
                Control::Button(_, false) => {}
                Control::Slider(name, value) => {
                    let routes = self.routes(name)?;
                    if routes.is_empty() {
                        continue;
                    }
                    ensure!(
                        (0.0..=1.0).contains(&value),
                        "invalid full-profile {name}: {value}"
                    );
                    let value = (value * 127.0)
                        .round()
                        .to_u8()
                        .context("quantize MIDI CC")?;
                    for (binding, channel) in routes {
                        if binding.kind == Kind::Absolute
                            && !(channel == 4 && (4..20).contains(&binding.number))
                        {
                            if let Some(side) = name
                                .strip_prefix("fx1_")
                                .map(|_| 0)
                                .or_else(|| name.strip_prefix("fx2_").map(|_| 1))
                            {
                                let slot = usize::from(binding.number % 8);
                                let knob = if slot < 3 { slot } else { 3 };
                                if slot < 3 && self.shifted[side] {
                                    self.browse_fx(side, slot, value, output)?;
                                    continue;
                                }
                                if !self.fx_pickup[side][slot].accepts(
                                    previous_fx[side][knob],
                                    value,
                                    self.fx_values[usize::from(channel)][slot],
                                ) {
                                    continue;
                                }
                            }
                            output.push([0xb0 | channel, binding.number, value]);
                        }
                    }
                }
                Control::Encoder(name, delta) => {
                    self.encoder(name, delta, output)?;
                }
                Control::Jog(name, delta) => self.jog(name, delta, output)?,
            }
        }
        self.emit_mixer(output)
    }

    fn browse_fx(
        &mut self,
        side: usize,
        slot: usize,
        value: u8,
        output: &mut Vec<[u8; 3]>,
    ) -> anyhow::Result<()> {
        // Four MIDI position units per effect retain sub-step movement
        // and provide hysteresis around the last emitted position.
        let anchor = self.fx_browse_anchors[side][slot].get_or_insert(value);
        let steps = (i32::from(value) - i32::from(*anchor)) / 4;
        *anchor = u8::try_from(i32::from(*anchor) + steps * 4)?;
        self.fx_pickup[side][slot] = FxPickup::default();
        relative(
            Address {
                channel: self.fx[side].channel() + 8,
                number: catalog::FX_BROWSE_BINDINGS[slot].number,
            },
            steps,
            output,
        )
    }

    fn move_selection(&mut self, controls: &[Control]) {
        for (side, name) in [
            (0, "left_move_encoder_press"),
            (1, "right_move_encoder_press"),
        ] {
            if pressed(controls, name) && !self.held.contains_key(name) {
                let deck = usize::from(self.decks[side].channel());
                self.move_selecting[deck] = !self.move_selecting[deck];
                // Own the physical press, even though it emits no MIDI note.
                self.held.insert(name, Vec::new());
                info!(
                    deck = deck + 1,
                    selecting = self.move_selecting[deck],
                    "MOVE_SIZE_SELECT"
                );
            }
        }
    }

    fn encoder(&mut self, name: &str, delta: i32, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        if let Some((side, "move_encoder")) = side_name(name)
            && self.shifted[side]
        {
            if delta == 0 {
                return Ok(());
            }
            let channel = self.decks[side].channel();
            let one_beat = catalog::jump_restore_binding(3).context("one-beat size missing")?;
            pulse(
                Address {
                    channel,
                    number: one_beat.number,
                },
                output,
            );
            // Djay coalesces repeated fixed-jump button pulses in one batch.
            // Its rotary consumes the full delta at the temporarily selected size.
            relative(Address { channel, number: 2 }, delta, output)?;
            let deck = usize::from(channel);
            if let Some(binding) = self.jump_sizes[deck].and_then(catalog::jump_restore_binding) {
                pulse(
                    Address {
                        channel,
                        number: binding.number,
                    },
                    output,
                );
                if self.jump_sizes[deck] != Some(3) {
                    self.jump_restore[deck] = Some(self.jump_revision[deck]);
                }
            }
            return Ok(());
        }
        if let Some((side, "browse_encoder")) = side_name(name) {
            let preview = self.held.contains_key(if side == 0 {
                "left_library_play"
            } else {
                "right_library_play"
            });
            if self.shifted[side] || preview {
                let action = match (preview, delta < 0) {
                    (true, true) => "preview_backward",
                    (true, false) => "preview_forward",
                    (false, true) => "source_previous",
                    (false, false) => "source_next",
                };
                let binding = catalog::DECK_BINDINGS
                    .iter()
                    .find(|binding| binding.input == action)
                    .context("browser pulse binding missing")?;
                for _ in 0..delta.unsigned_abs() {
                    pulse(
                        Address {
                            channel: channel(*binding, self.decks[side], self.shifted[side]),
                            number: binding.number,
                        },
                        output,
                    );
                }
                return Ok(());
            }
        }
        if let Some((side, "move_encoder")) = side_name(name)
            && self.move_selecting[usize::from(self.decks[side].channel())]
            && delta != 0
        {
            let deck = usize::from(self.decks[side].channel());
            self.jump_restore[deck] = None;
            self.jump_sizes[deck] = None;
        }
        for (binding, channel) in self.routes(name)? {
            if binding.kind == Kind::Relative {
                relative(
                    Address {
                        channel,
                        number: binding.number,
                    },
                    delta,
                    output,
                )?;
            }
        }
        Ok(())
    }

    fn selectors(&mut self, controls: &[Control], output: &mut Vec<[u8; 3]>) {
        for (side, prefix) in [(0, "left"), (1, "right")] {
            let old_shift = self.shifted[side];
            let old_grid = self.grid[side];
            let old_vinyl = self.vinyl[side];
            for control in controls {
                if let Control::Button(name, pressed) = *control {
                    if name
                        == if side == 0 {
                            "left_shift"
                        } else {
                            "right_shift"
                        }
                    {
                        self.shifted[side] = pressed;
                    }
                    if name == if side == 0 { "left_grid" } else { "right_grid" } {
                        self.grid[side] = pressed;
                    }
                }
            }
            if old_grid != self.grid[side]
                || (old_shift != self.shifted[side] && self.wheels[side].is_none())
            {
                self.residuals[side] = 0.0;
            }
            for (name, deck) in [
                ("left_deck_switch_a", Deck::A),
                ("left_deck_switch_c", Deck::C),
                ("right_deck_switch_b", Deck::B),
                ("right_deck_switch_d", Deck::D),
            ] {
                if name.starts_with(prefix) && pressed(controls, name) {
                    self.decks[side] = deck;
                    if self.wheels[side].is_none() {
                        self.residuals[side] = 0.0;
                    }
                    let binding = catalog::deck_selection_binding(deck.channel().saturating_add(1));
                    if let Some(binding) = binding {
                        pulse(
                            Address {
                                channel: 5,
                                number: binding.number,
                            },
                            output,
                        );
                    }
                }
            }
            for (suffix, mode) in [
                ("hotcue_mode", PadMode::Hotcue),
                ("samples_mode", PadMode::Samples),
                ("stems_mode", PadMode::Stems),
                ("mute_mode", PadMode::Stems),
            ] {
                if controls.iter().any(|input| {
                    matches!(input, Control::Button(name, true)
                    if name.strip_prefix(prefix).and_then(|s| s.strip_prefix('_')) == Some(suffix))
                }) {
                    self.pads[side] = mode;
                }
            }
            if pressed(
                controls,
                if side == 0 {
                    "left_jog_mode"
                } else {
                    "right_jog_mode"
                },
            ) {
                self.vinyl[side] = false;
            }
            if pressed(
                controls,
                if side == 0 {
                    "left_turntable_mode"
                } else {
                    "right_turntable_mode"
                },
            ) {
                self.vinyl[side] = true;
            }
            if self.wheels[side].is_some_and(|owner| owner.coasting)
                && (self.grid[side] || !self.vinyl[side])
            {
                self.end_backspin(side, output);
            } else if self.grid[side] != old_grid || self.vinyl[side] != old_vinyl {
                self.wheel_motion[side].reset_window();
            }
            self.wheel_touch(side, controls, output);
        }
        self.fx_targets(controls, output);
    }

    fn wheel_touch(&mut self, side: usize, controls: &[Control], output: &mut Vec<[u8; 3]>) {
        let touch = if side == 0 {
            "left_wheel_touch"
        } else {
            "right_wheel_touch"
        };
        for input in controls {
            if let Control::Button(name, touched) = *input
                && name == touch
            {
                let next = WheelOwner {
                    deck: self.decks[side],
                    action: if self.shifted[side] {
                        "jog_seek"
                    } else if self.vinyl[side] {
                        "jog_scratch"
                    } else {
                        "jog_bend"
                    },
                    coasting: false,
                };
                if touched {
                    if let Some(owner) = &mut self.wheels[side] {
                        if !owner.coasting {
                            continue; // Repeated touch snapshots retain their original owner.
                        }
                        if owner.deck == next.deck && owner.action == next.action {
                            owner.coasting = false;
                            self.wheel_motion[side].reset_window();
                            continue;
                        }
                        self.end_backspin(side, output);
                    }
                    self.wheel_motion[side].reset_window();
                    self.wheels[side] = Some(next);
                } else if self.wheels[side].is_none_or(|owner| !owner.coasting) {
                    self.wheels[side] = None;
                    self.wheel_motion[side].reset_window();
                }
                self.residuals[side] = 0.0;
            }
        }
    }

    fn fx_targets(&mut self, controls: &[Control], output: &mut Vec<[u8; 3]>) {
        let choosing_eq = self.eq_selecting;
        for (deck, fx1, fx2, quick) in [
            (
                Deck::A,
                "mixer_ch1_fx1",
                "mixer_ch1_fx2",
                "mixer_ch1_fx_select",
            ),
            (
                Deck::B,
                "mixer_ch2_fx1",
                "mixer_ch2_fx2",
                "mixer_ch2_fx_select",
            ),
            (
                Deck::C,
                "mixer_ch3_fx1",
                "mixer_ch3_fx2",
                "mixer_ch3_fx_select",
            ),
            (
                Deck::D,
                "mixer_ch4_fx1",
                "mixer_ch4_fx2",
                "mixer_ch4_fx_select",
            ),
        ] {
            if pressed(controls, fx1) {
                self.fx[0] = deck;
            }
            if pressed(controls, fx2) {
                self.fx[1] = deck;
            }
            if pressed(controls, quick) && !self.held.contains_key(quick) {
                self.held.insert(quick, Vec::new());
                if choosing_eq {
                    if self.eq_selecting {
                        pulse(
                            Address {
                                channel: deck.channel(),
                                number: catalog::EQ_MODE_NOTE,
                            },
                            output,
                        );
                        self.eq_selecting = false;
                    }
                } else {
                    self.quick = deck;
                }
            }
        }
    }

    fn routes(&self, name: &str) -> anyhow::Result<Vec<(Binding, u8)>> {
        let mut routes = Vec::new();
        if let Some((side, suffix)) = side_name(name) {
            let deck = self.decks[side];
            let suffix = if suffix == "move_encoder"
                && !self.shifted[side]
                && self.move_selecting[usize::from(deck.channel())]
            {
                "move_size"
            } else {
                suffix
            };
            if let Some(index) = suffix.strip_prefix("pad_") {
                let index = index
                    .parse::<u8>()?
                    .checked_sub(1)
                    .context("pad numbering")?;
                let binding = catalog::pad_binding(self.pads[side], index)
                    .context("pad index outside 1-8")?;
                routes.push((binding, channel(binding, deck, self.shifted[side])));
            } else {
                let fx = if side == 0 {
                    catalog::FX_LEFT_BINDINGS
                } else {
                    catalog::FX_RIGHT_BINDINGS
                };
                for binding in fx.iter().filter(|binding| binding.input == suffix) {
                    routes.push((
                        *binding,
                        channel(*binding, self.fx[side], self.shifted[side]),
                    ));
                }
                for binding in catalog::DECK_BINDINGS
                    .iter()
                    .filter(|binding| binding.input == suffix)
                {
                    if suffix != "wheel_touch" || self.vinyl[side] || self.shifted[side] {
                        routes.push((*binding, channel(*binding, deck, self.shifted[side])));
                    }
                }
            }
        } else if let Some((strip, suffix)) = name
            .strip_prefix("fx1_")
            .map(|s| (0, s))
            .or_else(|| name.strip_prefix("fx2_").map(|s| (1, s)))
        {
            let bindings = if strip == 0 {
                catalog::FX_LEFT_BINDINGS
            } else {
                catalog::FX_RIGHT_BINDINGS
            };
            for binding in bindings.iter().filter(|binding| binding.input == suffix) {
                routes.push((
                    *binding,
                    channel(*binding, self.fx[strip], self.shifted[strip]),
                ));
            }
        } else {
            for binding in catalog::MIXER_BINDINGS
                .iter()
                .filter(|binding| binding.input == name)
            {
                routes.push((*binding, 4));
            }
            for binding in catalog::QUICK_BINDINGS
                .iter()
                .filter(|binding| binding.input == name)
            {
                routes.push((*binding, self.quick.channel()));
            }
            if name == "mixer_quantize" {
                for binding in catalog::DECK_BINDINGS
                    .iter()
                    .filter(|binding| binding.input == "quantize")
                {
                    for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
                        routes.push((*binding, deck.channel()));
                    }
                }
            }
        }
        Ok(routes)
    }

    fn press(&mut self, name: &'static str, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        if self.held.contains_key(name) {
            return Ok(());
        }
        if let Some(deck) = catalog::EXT_INPUTS.iter().position(|input| *input == name) {
            // Consume even suppressed presses until release; later feedback or
            // releasing Shift must not turn a held source-select into a switch.
            self.held.insert(name, Vec::new());
            if self.shifted.contains(&true) {
                return Ok(());
            }
            let Some(thru) = self.thru[deck] else {
                warn!(
                    deck = deck + 1,
                    "EXT_STATE_UNKNOWN: select the matching feedback mapping"
                );
                return Ok(());
            };
            let binding = catalog::EXT_BINDINGS[usize::from(thru)];
            pulse(
                Address {
                    channel: u8::try_from(deck)?,
                    number: binding.number,
                },
                output,
            );
            info!(
                deck = deck + 1,
                mode = if thru { "INT" } else { "THRU" },
                "EXT_MODE_REQUEST"
            );
            return Ok(());
        }
        let addresses: Vec<_> = self
            .routes(name)?
            .into_iter()
            .filter(|(binding, _)| binding.kind == Kind::Note)
            .map(|(binding, channel)| Address {
                channel,
                number: binding.number,
            })
            .collect();
        if is_preview(name) {
            if !self.held.keys().any(|held| is_preview(held)) {
                for address in &addresses {
                    pulse(*address, output);
                }
            }
            self.held.insert(name, addresses);
            return Ok(());
        }
        for address in &addresses {
            if !self.held.values().flatten().any(|owned| owned == address) {
                output.push(address.press());
            }
        }
        if !addresses.is_empty() {
            self.held.insert(name, addresses);
        }
        Ok(())
    }

    fn release(&mut self, name: &str, output: &mut Vec<[u8; 3]>) {
        if let Some(addresses) = self.held.remove(name) {
            if is_preview(name) {
                if !self.held.keys().any(|held| is_preview(held)) {
                    for address in addresses {
                        pulse(address, output);
                    }
                }
                return;
            }
            for address in addresses {
                if !self.held.values().flatten().any(|owned| *owned == address) {
                    output.push(address.release());
                }
            }
        }
    }

    fn release_all(&mut self, output: &mut Vec<[u8; 3]>) {
        self.eq_selecting = false;
        self.wheels = [None; 2];
        self.residuals = [0.0; 2];
        for motion in &mut self.wheel_motion {
            motion.reset_window();
        }
        self.release("left_library_play", output);
        self.release("right_library_play", output);
        let mut addresses: Vec<_> = self
            .held
            .drain()
            .flat_map(|(_, addresses)| addresses)
            .collect();
        addresses.sort_unstable_by_key(|address| (address.channel, address.number));
        addresses.dedup();
        output.extend(addresses.into_iter().map(Address::release));
    }

    fn jog(&mut self, name: &str, delta: f64, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        let (side, _) = side_name(name).context("unknown jog side")?;
        ensure!(delta.is_finite(), "non-finite jog movement");
        let owner = self.wheels[side].unwrap_or(WheelOwner {
            deck: self.decks[side],
            action: if self.shifted[side] {
                "jog_seek"
            } else {
                "jog_bend"
            },
            coasting: false,
        });
        // Encdr divides integer S4 counter deltas by 1000 into f32.
        // Recover those integers before truncation can defer a count to a
        // later report (or cancel it when the direction reverses).
        let counts = (delta * 1000.0).round();
        let scale = if self.grid[side] {
            0.125
        } else if self.backspin_deck(side).is_some() {
            1.0 + self.jog_config.backspin_speed_increase / self.jog_config.scratch_speed
        } else {
            1.0
        };
        let accumulated = counts.mul_add(scale, self.residuals[side]);
        let steps = accumulated
            .trunc()
            .to_i32()
            .context("jog delta outside MIDI translation range")?;
        self.residuals[side] = accumulated - f64::from(steps);
        if self.grid[side] {
            let binding = catalog::DECK_BINDINGS
                .iter()
                .find(|binding| binding.input == if steps < 0 { "grid_left" } else { "grid_right" })
                .context("grid binding missing")?;
            for _ in 0..steps.unsigned_abs() {
                pulse(
                    Address {
                        channel: owner.deck.channel(),
                        number: binding.number,
                    },
                    output,
                );
            }
        } else {
            let binding = catalog::DECK_BINDINGS
                .iter()
                .find(|binding| binding.input == owner.action)
                .context("jog binding missing")?;
            relative(
                Address {
                    channel: owner.deck.channel(),
                    number: binding.number,
                },
                steps,
                output,
            )?;
        }
        Ok(())
    }

    fn front_panel(&mut self, payload: &[u8; 22], output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        for (deck, shift) in [(Deck::A, 4), (Deck::B, 2), (Deck::C, 6), (Deck::D, 0)] {
            let value = (payload[17] >> shift) & 3;
            ensure!(value <= 2, "invalid crossfader assignment state {value}");
            let index = usize::from(deck.channel());
            self.switches[index] = Some(value);
        }
        let curve = payload[18] & 3;
        match curve {
            0..=2 => {}
            _ => bail!("invalid crossfader curve state {curve}"),
        }
        self.switches[4] = Some(curve);
        self.emit_front_panel(output)
    }

    fn sync_front_panel(
        &mut self,
        feedback: &feedback::FeedbackState,
        output: &mut Vec<[u8; 3]>,
    ) -> anyhow::Result<()> {
        self.thru = feedback.thru;
        for side in 0..2 {
            let deck = usize::from(self.fx[side].channel());
            for slot in 0..6 {
                self.fx_pickup[side][slot].observe(
                    self.fx_values[deck][slot],
                    feedback.fx_values[deck][slot],
                    self.fx_positions[side][if slot < 3 { slot } else { 3 }],
                );
            }
        }
        self.fx_values = feedback.fx_values;
        for deck in 0..4 {
            let selected = feedback.info.jump_index(deck);
            self.jump_revision[deck] = feedback.info.jump_revision[deck];
            if let Some(revision) = self.jump_restore[deck] {
                // The fixed jump reports one before the following restore.
                // A newer non-one value confirms restoration or an external edit.
                if self.jump_revision[deck] != revision && selected.is_some_and(|index| index != 3)
                {
                    self.jump_restore[deck] = None;
                    self.jump_sizes[deck] = selected;
                }
            } else {
                self.jump_sizes[deck] = selected;
            }
        }
        if !self.front_panel_ready && feedback.playback_received {
            self.front_panel_ready = true;
            info!(
                "CROSSFADER_FEEDBACK_RECEIVED: applying physical switches once; restart bridge after mapping reconnect"
            );
        }
        self.emit_front_panel(output)?;
        self.emit_mixer(output)
    }

    fn emit_front_panel(&mut self, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
        if !self.front_panel_ready {
            return Ok(());
        }
        for index in 0..5 {
            if let Some(value) = self.switches[index]
                && self.delivered_switches[index] != Some(value)
            {
                if index < 4 {
                    let deck = u8::try_from(index)?.saturating_add(1);
                    let number = catalog::assignment_binding(deck, value)
                        .context("crossfader assignment binding missing")?
                        .number;
                    pulse(Address { channel: 4, number }, output);
                } else {
                    let value = [127, 64, 0][usize::from(value)];
                    output.push([0xb4, catalog::CURVE_BINDING.number, value]);
                }
                self.delivered_switches[index] = Some(value);
            }
        }
        Ok(())
    }
}

const fn channel(binding: Binding, deck: Deck, shifted: bool) -> u8 {
    if shifted && binding.shifted.is_some() {
        deck.channel().saturating_add(8)
    } else {
        deck.channel()
    }
}

fn is_preview(name: &str) -> bool {
    matches!(name, "left_library_play" | "right_library_play")
}

fn side_name(name: &str) -> Option<(usize, &str)> {
    name.strip_prefix("left_")
        .map(|name| (0, name))
        .or_else(|| name.strip_prefix("right_").map(|name| (1, name)))
}

fn pressed(controls: &[Control], name: &str) -> bool {
    controls
        .iter()
        .any(|control| matches!(control, Control::Button(input, true) if *input == name))
}

fn pulse(address: Address, output: &mut Vec<[u8; 3]>) {
    output.push(address.press());
    output.push(address.release());
}

fn relative(address: Address, mut steps: i32, output: &mut Vec<[u8; 3]>) -> anyhow::Result<()> {
    while steps != 0 {
        let chunk = steps.clamp(-63, 63);
        let value = u8::try_from(64_i32.saturating_add(chunk))?;
        output.push([0xb0 | address.channel, address.number, value]);
        steps = steps.saturating_sub(chunk);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(state: &mut Translator, controls: &[Control]) -> anyhow::Result<Vec<[u8; 3]>> {
        let mut output = Vec::new();
        state.translate(controls, None, &mut output)?;
        Ok(output)
    }

    #[test]
    fn captured_ext_fields_use_confirmed_modes_on_fixed_decks() -> anyhow::Result<()> {
        use encdr::core::descriptor::InputItemDesc;
        use encdr::device::loader::DescriptorRegistry;

        // Given captured EXT bits, the real descriptor and unrelated selectors.
        let mut registry = DescriptorRegistry::new();
        registry.load_builtins()?;
        let original = registry
            .find(crate::S4_VENDOR_ID, crate::S4_PRODUCT_ID)
            .context("S4 descriptor missing")?;
        let descriptor = crate::full_control_descriptor(original)?;
        let names = registry.intern_descriptor_names(&descriptor);
        for (deck, (byte, mask)) in
            (0_u8..4).zip([(11, 0x01_u8), (11, 0x02), (2, 0x02), (11, 0x04)])
        {
            let button = descriptor
                .input_packets
                .iter()
                .flat_map(|packet| &packet.items)
                .find_map(|item| match item {
                    InputItemDesc::Button(item)
                        if item.byte == byte && item.mask.0 == u16::from(mask) =>
                    {
                        Some(item)
                    }
                    _ => None,
                })
                .context("captured EXT field missing")?;
            let name = *names.get(&button.name).context("EXT name missing")?;
            let mut state = Translator {
                decks: [Deck::C, Deck::D],
                fx: [Deck::D, Deck::C],
                quick: Deck::B,
                ..Translator::default()
            };
            let mut feedback = feedback::FeedbackState::default();
            feedback::Decoder::default().feed(
                &[
                    0xb0 | deck,
                    catalog::EXT_THRU_CC,
                    if deck.is_multiple_of(2) { 0 } else { 127 },
                ],
                |message| {
                    feedback.apply(message, Duration::ZERO);
                },
            );
            state.sync_front_panel(&feedback, &mut Vec::new())?;
            let mut payload = [0_u8; 22];
            payload[byte] = mask;
            // When the physical channel's EXT is pressed.
            let output = run(
                &mut state,
                &[Control::Button(name, payload[byte] & mask != 0)],
            )?;
            // Then confirmed THRU chooses INT, otherwise THRU, on that fixed deck.
            let note = if deck.is_multiple_of(2) { 27 } else { 28 };
            assert_eq!(output, [[0x90 | deck, note, 127], [0x80 | deck, note, 0]]);
            assert_eq!(state.thru, feedback.thru);
        }
        Ok(())
    }

    #[test]
    fn ext_waits_for_feedback_and_consumes_each_press_until_release() -> anyhow::Result<()> {
        // Given an unknown mode and a physical EXT hold.
        let mut state = Translator::default();
        let mut feedback = feedback::FeedbackState::default();
        let name = "mixer_ch1_ext";
        assert!(run(&mut state, &[Control::Button(name, true)])?.is_empty());
        // When feedback arrives, a held snapshot cannot become a new press.
        for thru in [false, true] {
            feedback::Decoder::default().feed(
                &[0xb0, catalog::EXT_THRU_CC, if thru { 127 } else { 0 }],
                |message| {
                    feedback.apply(message, Duration::ZERO);
                },
            );
            state.sync_front_panel(&feedback, &mut Vec::new())?;
            assert!(run(&mut state, &[Control::Button(name, true)])?.is_empty());
            assert!(run(&mut state, &[Control::Button(name, false)])?.is_empty());
            // Then only a fresh press selects the mode; it never guesses the echo.
            let note = if thru { 28 } else { 27 };
            assert_eq!(
                run(&mut state, &[Control::Button(name, true)])?,
                [[0x90, note, 127], [0x80, note, 0]]
            );
            assert_eq!(state.thru, feedback.thru);
        }
        let mut released = Vec::new();
        state.release_all(&mut released);
        assert!(released.is_empty());
        Ok(())
    }

    #[test]
    fn either_shift_consumes_ext_without_a_later_unshifted_switch() -> anyhow::Result<()> {
        for shift in ["left_shift", "right_shift"] {
            for name in catalog::EXT_INPUTS {
                // Given known Internal modes on all decks.
                let mut state = Translator {
                    thru: [Some(false); 4],
                    ..Translator::default()
                };
                // When EXT precedes Shift in the same snapshot, then it is still suppressed.
                assert!(
                    run(
                        &mut state,
                        &[Control::Button(name, true), Control::Button(shift, true)]
                    )?
                    .is_empty()
                );
                assert!(
                    run(
                        &mut state,
                        &[Control::Button(shift, false), Control::Button(name, true)]
                    )?
                    .is_empty()
                );
                assert!(run(&mut state, &[Control::Button(name, false)])?.is_empty());
                assert_eq!(run(&mut state, &[Control::Button(name, true)])?.len(), 2);
            }
        }
        Ok(())
    }

    #[test]
    fn physical_routes_exist_in_the_generated_mapping_on_every_layer() -> anyhow::Result<()> {
        // Given actual physical names from the pinned, corrected descriptor.
        use encdr::core::descriptor::InputItemDesc;
        use encdr::device::loader::DescriptorRegistry;
        let mut registry = DescriptorRegistry::new();
        registry.load_builtins()?;
        let original = registry
            .find(crate::S4_VENDOR_ID, crate::S4_PRODUCT_ID)
            .context("S4 descriptor missing")?;
        let descriptor = crate::full_control_descriptor(original)?;
        let mapping = mapping::generate()?;
        let controls = mapping
            .as_dictionary()
            .and_then(|root| root.get("controls"))
            .and_then(plist::Value::as_array)
            .context("mapping controls missing")?;
        // When each deck, modifier and pad layer resolves these physical names.
        for (left, right) in [(Deck::A, Deck::B), (Deck::C, Deck::D)] {
            for shifted in [false, true] {
                for mode in [PadMode::Hotcue, PadMode::Samples, PadMode::Stems] {
                    let state = Translator {
                        decks: [left, right],
                        fx: [left, right],
                        quick: left,
                        shifted: [shifted; 2],
                        pads: [mode; 2],
                        ..Translator::default()
                    };
                    for item in descriptor
                        .input_packets
                        .iter()
                        .flat_map(|packet| &packet.items)
                    {
                        let name = match item {
                            InputItemDesc::Button(item) => &item.name,
                            InputItemDesc::Slider(item) => &item.name,
                            InputItemDesc::Encoder(item) => &item.name,
                            InputItemDesc::EncoderFine(item) => &item.name,
                            InputItemDesc::Touch(item) => &item.name,
                        };
                        for (binding, channel) in state.routes(name)? {
                            let kind = if binding.kind == Kind::Note { 1 } else { 3 };
                            // Then the generator contains the exact runtime address.
                            assert!(
                                controls.iter().any(|control| {
                                    let Some(row) = control.as_dictionary() else {
                                        return false;
                                    };
                                    row.get("midiChannel")
                                        .and_then(plist::Value::as_unsigned_integer)
                                        == Some(u64::from(channel))
                                        && row
                                            .get("midiData")
                                            .and_then(plist::Value::as_unsigned_integer)
                                            == Some(u64::from(binding.number))
                                        && row
                                            .get("midiMessageType")
                                            .and_then(plist::Value::as_unsigned_integer)
                                            == Some(kind)
                                }),
                                "unmapped runtime address for {name}"
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn captured_encoders_route_rotation_and_press_to_the_physical_knob() -> anyhow::Result<()> {
        use encdr::core::descriptor::InputItemDesc;
        use encdr::device::encoder::EncoderState;
        use encdr::device::loader::DescriptorRegistry;

        // Given independently captured physical fields, not descriptor names.
        for shifted in [false, true] {
            for (byte, increment, press_byte, mask, cc, note, deck) in [
                (19, 1, 6, 0x04, 2, 5, 0),   // Left MOVE
                (19, 16, 6, 0x20, 1, 6, 0),  // Left LOOP
                (20, 16, 15, 0x20, 2, 5, 1), // Right MOVE
                (21, 1, 15, 0x04, 1, 6, 1),  // Right LOOP
            ] {
                let mut registry = DescriptorRegistry::new();
                registry.load_builtins()?;
                let original = registry
                    .find(crate::S4_VENDOR_ID, crate::S4_PRODUCT_ID)
                    .context("S4 descriptor missing")?;
                let descriptor = crate::full_control_descriptor(original)?;
                let names = registry.intern_descriptor_names(&descriptor);
                let encoder = descriptor
                    .input_packets
                    .iter()
                    .flat_map(|packet| &packet.items)
                    .find_map(|item| match item {
                        InputItemDesc::Encoder(item)
                            if item.byte == byte
                                && item.bit_offset == if increment == 1 { 0 } else { 4 } =>
                        {
                            Some(item)
                        }
                        _ => None,
                    })
                    .context("captured encoder field missing")?;
                let encoder_name = *names.get(&encoder.name).context("encoder name missing")?;
                let press = descriptor
                    .input_packets
                    .iter()
                    .flat_map(|packet| &packet.items)
                    .find_map(|item| match item {
                        InputItemDesc::Button(item)
                            if item.byte == press_byte && item.mask.0 == u16::from(mask) =>
                        {
                            Some(item)
                        }
                        _ => None,
                    })
                    .context("captured press field missing")?;
                let press_name = *names.get(&press.name).context("press name missing")?;
                let mut state = Translator::default();
                let mut payload = [0_u8; 22];
                if shifted {
                    let shift = if deck == 0 {
                        "left_shift"
                    } else {
                        "right_shift"
                    };
                    state.translate(&[Control::Button(shift, true)], None, &mut Vec::new())?;
                }
                let mut decoder = EncoderState::default();
                assert_eq!(
                    decoder.update_wrap16((payload[byte] >> encoder.bit_offset) & 15),
                    None
                );

                // When the knob moves forward/backward and is pressed/released.
                let mut output = Vec::new();
                for action in 0..4 {
                    match action {
                        0 => payload[byte] = payload[byte].wrapping_add(increment),
                        1 => payload[byte] = payload[byte].wrapping_sub(increment),
                        2 => payload[press_byte] |= mask,
                        _ => payload[press_byte] &= !mask,
                    }
                    let control = if action < 2 {
                        Control::Encoder(
                            encoder_name,
                            decoder
                                .update_wrap16((payload[byte] >> encoder.bit_offset) & 15)
                                .context("captured rotation produced no delta")?,
                        )
                    } else {
                        Control::Button(press_name, payload[press_byte] & mask != 0)
                    };
                    state.translate(&[control], None, &mut output)?;
                }

                // Then the actual MIDI destinations retain physical semantics.
                assert_eq!(
                    output,
                    captured_encoder_midi(cc, note, deck, shifted),
                    "physical field {byte} increment {increment}, shifted={shifted}"
                );
                assert_eq!(state.move_selecting[usize::from(deck)], note == 5);
            }
        }
        Ok(())
    }

    #[test]
    fn normalized_hid_jog_counts_are_not_delayed_by_float_rounding() -> anyhow::Result<()> {
        // Given integer counter deltas normalized to f32 by Encdr.
        let mut state = Translator::default();
        for counts in [5_i16, -5, 9, -9, 63, -63] {
            let delta = f64::from(f32::from(counts) / 1000.0);
            // When slow motion reverses direction.
            let output = run(&mut state, &[Control::Jog("left_jog_wheel", delta)])?;
            // Then every report retains its exact signed counter movement.
            let decoded: i32 = output
                .iter()
                .map(|message| i32::from(message[2]) - 64)
                .sum();
            assert_eq!(decoded, i32::from(counts));
        }
        Ok(())
    }

    #[test]
    fn cue_release_keeps_the_deck_that_owned_its_press() -> anyhow::Result<()> {
        // Given Cue held on deck A.
        let mut state = Translator::default();
        assert_eq!(
            run(&mut state, &[Control::Button("left_cue", true)])?,
            [[0x90, 1, 127]]
        );
        run(
            &mut state,
            &[
                Control::Button("left_deck_switch_c", true),
                Control::Button("left_shift", true),
            ],
        )?;
        // When the physical Cue button is released after changing deck/layer.
        let output = run(&mut state, &[Control::Button("left_cue", false)])?;
        // Then the old deck receives the release, not deck C.
        assert_eq!(output, [[0x80, 1, 0]]);
        Ok(())
    }

    #[test]
    fn shifted_sync_emits_once_and_releases_original_layer_after_deck_change() -> anyhow::Result<()>
    {
        // Given either physical side and either selected deck on that side.
        for (sync, shift, switch, channel) in [
            ("left_sync", "left_shift", "left_deck_switch_c", 0_u8),
            ("right_sync", "right_shift", "right_deck_switch_d", 1),
        ] {
            for alternate in [false, true] {
                let mut state = Translator::default();
                if alternate {
                    run(&mut state, &[Control::Button(switch, true)])?;
                }
                let deck = channel + if alternate { 2 } else { 0 };
                assert_eq!(
                    run(&mut state, &[Control::Button(sync, true)])?,
                    [[0x90 + deck, 2, 127]]
                );
                run(&mut state, &[Control::Button(sync, false)])?;
                // When Shift and Sync arrive together, even with Sync first.
                assert_eq!(
                    run(
                        &mut state,
                        &[Control::Button(sync, true), Control::Button(shift, true)]
                    )?,
                    [[0x98 + deck, 2, 127]]
                );
                assert!(run(&mut state, &[Control::Button(sync, true)])?.is_empty());
                run(
                    &mut state,
                    &[
                        Control::Button(shift, false),
                        Control::Button(switch, !alternate),
                    ],
                )?;
                // Then release stays on the shifted address that owned the press.
                assert_eq!(
                    run(&mut state, &[Control::Button(sync, false)])?,
                    [[0x88 + deck, 2, 0]]
                );
            }
        }
        Ok(())
    }

    #[test]
    fn simultaneous_shift_and_pad_press_do_not_depend_on_descriptor_order() -> anyhow::Result<()> {
        // Given equivalent HID reports with opposite event ordering.
        for inputs in [
            [
                Control::Button("left_pad_1", true),
                Control::Button("left_shift", true),
            ],
            [
                Control::Button("left_shift", true),
                Control::Button("left_pad_1", true),
            ],
        ] {
            let mut state = Translator::default();
            // When the entire report is translated.
            let output = run(&mut state, &inputs)?;
            // Then the shifted hotcue action always gets the press.
            assert_eq!(output, [[0x98, 40, 127]]);
        }
        Ok(())
    }

    #[test]
    fn held_pad_releases_its_original_mode_before_new_strikes_use_samples() -> anyhow::Result<()> {
        // Given a hotcue pad held while the mode changes.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_pad_2", true)])?;
        run(&mut state, &[Control::Button("left_samples_mode", true)])?;
        // When that pad is released.
        let output = run(&mut state, &[Control::Button("left_pad_2", false)])?;
        // Then only its original hotcue note is released.
        assert_eq!(output, [[0x80, 41, 0]]);
        Ok(())
    }

    #[test]
    fn new_pad_strikes_use_the_selected_pad_mode() -> anyhow::Result<()> {
        // Given Samples mode.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_samples_mode", true)])?;
        // When pad 2 is struck.
        let output = run(&mut state, &[Control::Button("left_pad_2", true)])?;
        // Then its sample binding is emitted.
        assert_eq!(output, [[0x90, 49, 127]]);
        Ok(())
    }

    #[test]
    fn eq_selection_toggles_each_deck_without_retargeting_instant_fx() -> anyhow::Result<()> {
        for (deck, name) in [
            (0, "mixer_ch1_fx_select"),
            (1, "mixer_ch2_fx_select"),
            (2, "mixer_ch3_fx_select"),
            (3, "mixer_ch4_fx_select"),
        ] {
            for reversed in [false, true] {
                // Given Instant FX targeting B and both buttons newly pressed.
                let mut state = Translator {
                    quick: Deck::B,
                    ..Translator::default()
                };
                let mut controls = [
                    Control::Button("mixer_fx_filter", true),
                    Control::Button(name, true),
                ];
                if reversed {
                    controls.reverse();
                }
                // When Filter and ON arrive together in either descriptor order.
                let output = run(&mut state, &controls)?;
                // Then only this deck's EQ toggle pulses; FX selection is preserved.
                assert_eq!(output, [[0x90 | deck, 20, 127], [0x80 | deck, 20, 0]]);
                assert_eq!(state.quick, Deck::B);
                assert!(!state.local_led_state().eq_selecting);
                assert!(run(&mut state, &controls)?.is_empty());
                assert_eq!(state.quick, Deck::B);
            }
        }
        Ok(())
    }

    #[test]
    fn filter_selection_cancels_only_on_a_new_press() -> anyhow::Result<()> {
        // Given a held Filter press and a duplicate snapshot.
        let mut state = Translator::default();
        assert!(run(&mut state, &[Control::Button("mixer_fx_filter", true)])?.is_empty());
        assert!(run(&mut state, &[Control::Button("mixer_fx_filter", true)])?.is_empty());
        assert!(state.local_led_state().eq_selecting);
        run(&mut state, &[Control::Button("mixer_fx_filter", false)])?;
        // When Filter is pressed again.
        let output = run(&mut state, &[Control::Button("mixer_fx_filter", true)])?;
        // Then it cancels without resetting a filter or toggling EQ.
        assert!(output.is_empty());
        assert!(!state.local_led_state().eq_selecting);
        Ok(())
    }

    #[test]
    fn eq_selection_preserves_held_fx_and_subsequent_normal_on_selection() -> anyhow::Result<()> {
        // Given an Instant FX held on A and a pending EQ selection.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("mixer_fx_2", true)])?;
        run(&mut state, &[Control::Button("mixer_fx_filter", true)])?;
        run(&mut state, &[Control::Button("mixer_ch3_fx_select", true)])?;
        // When the FX releases, then C is selected normally after ON releases.
        let release = run(&mut state, &[Control::Button("mixer_fx_2", false)])?;
        run(&mut state, &[Control::Button("mixer_ch3_fx_select", false)])?;
        run(&mut state, &[Control::Button("mixer_ch3_fx_select", true)])?;
        let next = run(&mut state, &[Control::Button("mixer_fx_2", true)])?;
        // Then the old owner releases on A and the new effect targets C.
        assert_eq!(release, [[0x80, 17, 0]]);
        assert_eq!(next, [[0x92, 17, 127]]);
        Ok(())
    }

    #[test]
    fn simultaneous_eq_choices_consume_one_selection_without_changing_fx() -> anyhow::Result<()> {
        // Given Filter awaiting one deck choice with FX targeting D.
        let mut state = Translator {
            quick: Deck::D,
            ..Translator::default()
        };
        run(&mut state, &[Control::Button("mixer_fx_filter", true)])?;
        // When multiple ON presses arrive in the same report.
        let output = run(
            &mut state,
            &[
                Control::Button("mixer_ch2_fx_select", true),
                Control::Button("mixer_ch1_fx_select", true),
            ],
        )?;
        // Then the lowest deck wins deterministically and neither retargets FX.
        assert_eq!(output, [[0x90, 20, 127], [0x80, 20, 0]]);
        assert_eq!(state.quick, Deck::D);
        Ok(())
    }

    #[test]
    fn fx_target_changes_do_not_strand_held_effect_buttons() -> anyhow::Result<()> {
        // Given the left FX1 button held for deck A.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_fx_1", true)])?;
        run(&mut state, &[Control::Button("mixer_ch3_fx1", true)])?;
        // When FX1 releases and FX2 presses after selecting deck C.
        let output = run(
            &mut state,
            &[
                Control::Button("left_fx_1", false),
                Control::Button("left_fx_2", true),
            ],
        )?;
        // Then old ownership releases on A, while the new press targets C.
        assert_eq!(output, [[0x80, 33, 0], [0x92, 34, 127]]);
        Ok(())
    }

    #[test]
    fn touched_jog_keeps_its_owner_through_deck_selection() -> anyhow::Result<()> {
        // Given a touched wheel on A followed by selection of C.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_wheel_touch", true)])?;
        run(&mut state, &[Control::Button("left_deck_switch_c", true)])?;
        // When the touched wheel moves.
        let output = run(&mut state, &[Control::Jog("left_jog_wheel", 0.002)])?;
        // Then scratch movement remains on its touch owner A.
        assert_eq!(output, [[0xb0, 5, 66]]);
        Ok(())
    }

    pub(super) fn jog_snapshot(
        state: &mut Translator,
        side: usize,
        time_ms: u64,
        counter: u32,
    ) -> anyhow::Result<Vec<[u8; 3]>> {
        let mut report = [0_u8; 60];
        report[0] = 3;
        for offset in [8, 36] {
            report[offset..offset + 4].copy_from_slice(&u32::try_from(time_ms + 1)?.to_le_bytes());
        }
        let offset = if side == 0 { 12 } else { 40 };
        report[offset..offset + 4].copy_from_slice(&counter.to_le_bytes());
        let mut output = Vec::new();
        state.observe_jog(Some(&report), Duration::from_millis(time_ms), &mut output)?;
        Ok(output)
    }

    pub(super) fn prime_backspin(state: &mut Translator, side: usize) -> anyhow::Result<()> {
        let (touch, wheel) = if side == 0 {
            ("left_wheel_touch", "left_jog_wheel")
        } else {
            ("right_wheel_touch", "right_jog_wheel")
        };
        jog_snapshot(state, side, 0, 1_000)?;
        run(state, &[Control::Button(touch, true)])?;
        for time in [10, 20, 30] {
            jog_snapshot(state, side, time, 1_000)?;
        }
        for time in [40, 50, 60] {
            jog_snapshot(state, side, time, 1_000 - u32::try_from(time - 30)? * 10)?;
            run(state, &[Control::Jog(wheel, -0.1)])?;
        }
        Ok(())
    }

    #[test]
    fn assist_qualification_follows_the_detected_wheel_and_cancels_on_retouch() -> anyhow::Result<()>
    {
        for (side, touch, deck, shift) in [
            (0, "left_wheel_touch", "left_deck_switch_c", "left_shift"),
            (1, "right_wheel_touch", "right_deck_switch_d", "right_shift"),
        ] {
            for enabled in [false, true] {
                // Given a real qualified gesture and an explicit assist preference.
                let mut state = Translator::default();
                state.jog_config.backspin_assist_enabled = enabled;
                prime_backspin(&mut state, side)?;
                let mut expected = [false; 2];
                expected[side] = enabled;
                assert_eq!(state.backspin_assist(), expected);
                // When release, deck selection and Shift change the presentation.
                run(&mut state, &[Control::Button(touch, false)])?;
                run(
                    &mut state,
                    &[Control::Button(deck, true), Control::Button(shift, true)],
                )?;
                // Then assistance still follows only its original physical wheel.
                assert_eq!(state.backspin_assist(), expected);
                // When physical touch returns, then the old event is not qualified.
                run(&mut state, &[Control::Button(touch, true)])?;
                assert_eq!(state.backspin_assist(), [false; 2]);
            }
        }
        Ok(())
    }

    #[test]
    fn mode_cancellation_removes_motor_qualification_with_the_midi_hold() -> anyhow::Result<()> {
        for mode in ["left_jog_mode", "left_grid"] {
            // Given an opted-in released backspin.
            let mut state = Translator::default();
            state.jog_config.backspin_assist_enabled = true;
            prime_backspin(&mut state, 0)?;
            run(&mut state, &[Control::Button("left_wheel_touch", false)])?;
            assert_eq!(state.backspin_assist(), [true, false]);
            // When a current input cancels its wheel mode,
            let output = run(&mut state, &[Control::Button(mode, true)])?;
            // Then the same event loses motor permission and releases its original note.
            assert_eq!(state.backspin_assist(), [false; 2]);
            assert!(output.contains(&[0x80, 7, 0]));
        }
        Ok(())
    }

    #[test]
    fn released_backspin_keeps_scratch_until_measured_motion_stops() -> anyhow::Result<()> {
        for (side, touch, wheel, channel) in [
            (0, "left_wheel_touch", "left_jog_wheel", 0),
            (1, "right_wheel_touch", "right_jog_wheel", 1),
        ] {
            // Given a touched platter with sustained backward movement.
            let mut state = Translator::default();
            prime_backspin(&mut state, side)?;
            // When the hand releases but the physical platter keeps spinning.
            let release = run(&mut state, &[Control::Button(touch, false)])?;
            // Then no touch-off is sent and real movement remains scratch.
            assert!(
                release.is_empty(),
                "backspin released scratch mode: {release:?}"
            );
            jog_snapshot(&mut state, side, 70, 600)?;
            let movement = run(&mut state, &[Control::Jog(wheel, -0.1)])?;
            assert_eq!(
                movement,
                [
                    [0xb0 | channel, 5, 1],
                    [0xb0 | channel, 5, 1],
                    [0xb0 | channel, 5, 1],
                    [0xb0 | channel, 5, 1],
                    [0xb0 | channel, 5, 9],
                ]
            );
            for time in [80, 90] {
                assert!(jog_snapshot(&mut state, side, time, 600)?.is_empty());
            }
            assert_eq!(
                jog_snapshot(&mut state, side, 100, 600)?,
                [[0x80 | channel, 7, 0]]
            );
        }
        Ok(())
    }

    #[test]
    fn ordinary_scratch_release_still_returns_immediately_to_nudge() -> anyhow::Result<()> {
        // Given an ordinary slow backward scratch.
        let mut state = Translator::default();
        jog_snapshot(&mut state, 0, 0, 1_000)?;
        run(&mut state, &[Control::Button("left_wheel_touch", true)])?;
        for time in [10, 20, 30, 40, 50, 60] {
            jog_snapshot(&mut state, 0, time, 1_000 - u32::try_from(time)?)?;
        }
        // When its top touch releases and movement continues.
        let mut output = run(&mut state, &[Control::Button("left_wheel_touch", false)])?;
        output.extend(run(&mut state, &[Control::Jog("left_jog_wheel", -0.01)])?);
        // Then touch releases normally and only nudge receives the new motion.
        assert_eq!(output, [[0x80, 7, 0], [0xb0, 4, 54]]);
        assert_eq!(state.backspin_deck(0), None);
        Ok(())
    }

    #[test]
    fn immediate_flick_uses_measured_pre_touch_rest_as_its_acceleration_baseline()
    -> anyhow::Result<()> {
        // Given a measured stationary wheel before the hand touches it.
        let mut state = Translator::default();
        for time in [0, 10, 20, 30] {
            jog_snapshot(&mut state, 0, time, 1_000)?;
        }
        run(&mut state, &[Control::Button("left_wheel_touch", true)])?;
        // When a backward flick starts immediately and releases after 30 ms.
        for time in [40, 50, 60] {
            jog_snapshot(&mut state, 0, time, 1_000 - u32::try_from(time - 30)? * 10)?;
        }
        let output = run(&mut state, &[Control::Button("left_wheel_touch", false)])?;
        // Then detection uses the real rest baseline rather than waiting for a second flick.
        assert!(output.is_empty());
        assert_eq!(state.backspin_deck(0), Some(0));
        Ok(())
    }

    #[test]
    fn backspin_detection_respects_disable_threshold_and_wheel_modes() -> anyhow::Result<()> {
        for variant in 0..5 {
            // Given disabled detection, a higher threshold, seek, JOG, or GRID.
            let mut state = Translator::default();
            match variant {
                0 => state.jog_config.backspin_enabled = false,
                1 => state.jog_config.backspin_acceleration = 1_000_000.0,
                2 => {
                    run(&mut state, &[Control::Button("left_shift", true)])?;
                }
                3 => {
                    run(&mut state, &[Control::Button("left_jog_mode", true)])?;
                }
                _ => {
                    run(&mut state, &[Control::Button("left_grid", true)])?;
                }
            }
            prime_backspin(&mut state, 0)?;
            // When an otherwise fast backward gesture releases.
            let release = run(&mut state, &[Control::Button("left_wheel_touch", false)])?;
            // Then it cannot retain a synthetic scratch hold.
            assert_eq!(state.backspin_deck(0), None, "variant {variant}");
            assert!(state.wheels[0].is_none(), "variant {variant}");
            assert_eq!(release.is_empty(), variant == 3);
        }
        Ok(())
    }

    #[test]
    fn coasting_backspin_retains_ownership_across_deck_and_shift_changes() -> anyhow::Result<()> {
        // Given a released backspin owned by A.
        let mut state = Translator::default();
        prime_backspin(&mut state, 0)?;
        run(&mut state, &[Control::Button("left_wheel_touch", false)])?;
        run(
            &mut state,
            &[
                Control::Button("left_deck_switch_c", true),
                Control::Button("left_shift", true),
            ],
        )?;
        jog_snapshot(&mut state, 0, 70, 600)?;
        // When the released wheel continues and the independent right wheel nudges.
        let output = run(
            &mut state,
            &[
                Control::Jog("left_jog_wheel", -0.01),
                Control::Jog("right_jog_wheel", 0.01),
            ],
        )?;
        // Then only the original A scratch owner receives the left motion.
        assert_eq!(output, [[0xb0, 5, 34], [0xb1, 4, 74]]);
        assert_eq!(state.backspin_deck(0), Some(0));
        assert_eq!(state.backspin_deck(1), None);
        Ok(())
    }

    #[test]
    fn retouch_clears_backspin_and_reacquires_the_current_deck_without_stuck_notes()
    -> anyhow::Result<()> {
        for select_c in [false, true] {
            // Given a released A backspin, optionally with C selected afterward.
            let mut state = Translator::default();
            prime_backspin(&mut state, 0)?;
            run(&mut state, &[Control::Button("left_wheel_touch", false)])?;
            if select_c {
                run(&mut state, &[Control::Button("left_deck_switch_c", true)])?;
            }
            // When the hand catches the wheel.
            let output = run(&mut state, &[Control::Button("left_wheel_touch", true)])?;
            // Then the same deck stays held without a blip, or ownership transfers.
            assert_eq!(
                output,
                if select_c {
                    vec![[0x80, 7, 0], [0x92, 7, 127]]
                } else {
                    Vec::new()
                }
            );
            assert_eq!(state.backspin_deck(0), None);
            assert_eq!(
                run(&mut state, &[Control::Button("left_wheel_touch", false)])?,
                [[if select_c { 0x82 } else { 0x80 }, 7, 0]]
            );
        }
        Ok(())
    }

    #[test]
    fn reversal_discontinuity_freshness_and_stop_release_backspin_once() -> anyhow::Result<()> {
        for variant in 0..4 {
            // Given a released and still-moving backspin.
            let mut state = Translator::default();
            prime_backspin(&mut state, 0)?;
            run(&mut state, &[Control::Button("left_wheel_touch", false)])?;
            // When it reverses, jumps, loses fresh input, or the bridge stops.
            let output = match variant {
                0 => jog_snapshot(&mut state, 0, 70, 702)?,
                1 => jog_snapshot(&mut state, 0, 70, 5_000)?,
                2 => {
                    let mut output = Vec::new();
                    state.observe_jog(None, Duration::from_millis(111), &mut output)?;
                    output
                }
                _ => {
                    let mut output = Vec::new();
                    state.release_all(&mut output);
                    output
                }
            };
            // Then scratch releases exactly once and no continuation remains.
            assert_eq!(output, [[0x80, 7, 0]]);
            assert!(state.wheels[0].is_none());
            let mut repeated = Vec::new();
            state.release_all(&mut repeated);
            assert!(repeated.is_empty());
        }
        Ok(())
    }

    #[test]
    fn backspin_speed_increase_is_additive_and_does_not_leak_into_normal_motion()
    -> anyhow::Result<()> {
        // Given an increase equal to the normal speed, hence double event travel.
        let mut state = Translator {
            jog_config: jog::Config {
                backspin_speed_increase: jog::DEFAULT.scratch_speed,
                ..jog::DEFAULT
            },
            ..Translator::default()
        };
        prime_backspin(&mut state, 0)?;
        run(&mut state, &[Control::Button("left_wheel_touch", false)])?;
        jog_snapshot(&mut state, 0, 70, 600)?;
        // When event motion is followed by retouch and an ordinary slow scratch.
        let boosted = run(&mut state, &[Control::Jog("left_jog_wheel", -0.01)])?;
        run(&mut state, &[Control::Button("left_wheel_touch", true)])?;
        let normal = run(&mut state, &[Control::Jog("left_jog_wheel", -0.01)])?;
        // Then only the active backspin receives twice the normal count gain.
        assert_eq!(boosted, [[0xb0, 5, 44]]);
        assert_eq!(normal, [[0xb0, 5, 54]]);
        Ok(())
    }

    #[test]
    fn fractional_grid_motion_is_preserved_without_timing_assumptions() -> anyhow::Result<()> {
        // Given three reports smaller than the eight-count grid step.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_grid", true)])?;
        run(&mut state, &[Control::Jog("left_jog_wheel", 0.003)])?;
        run(&mut state, &[Control::Jog("left_jog_wheel", 0.003)])?;
        // When the accumulated motion passes one step.
        let output = run(&mut state, &[Control::Jog("left_jog_wheel", 0.003)])?;
        // Then one grid pulse is emitted and the remainder retained.
        assert_eq!(output, [[0x90, 30, 127], [0x80, 30, 0]]);
        assert!((state.residuals[0] - 0.125).abs() < 1e-9);
        Ok(())
    }

    fn captured_encoder_midi(cc: u8, note: u8, deck: u8, shifted: bool) -> Vec<[u8; 3]> {
        let channel = if shifted { deck + 8 } else { deck };
        match (note, shifted) {
            (5, false) => vec![[0xb0 | deck, cc, 65], [0xb0 | deck, cc, 63]],
            (5, true) => vec![
                [0x90 | deck, 67, 127],
                [0x80 | deck, 67, 0],
                [0xb0 | deck, 2, 65],
                [0x90 | deck, 67, 127],
                [0x80 | deck, 67, 0],
                [0xb0 | deck, 2, 63],
            ],
            _ => vec![
                [0xb0 | channel, cc, 65],
                [0xb0 | channel, cc, 63],
                [0x90 | channel, note, 127],
                [0x80 | channel, note, 0],
            ],
        }
    }

    #[test]
    fn move_selection_is_per_deck_and_duplicate_presses_do_not_toggle() -> anyhow::Result<()> {
        // Given A selecting size, including a duplicate press and a deck switch while held.
        let mut state = Translator::default();
        assert!(
            run(
                &mut state,
                &[Control::Button("left_move_encoder_press", true)]
            )?
            .is_empty()
        );
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", true)],
        )?;
        run(&mut state, &[Control::Button("left_deck_switch_c", true)])?;
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", false)],
        )?;
        assert_eq!(
            run(&mut state, &[Control::Encoder("left_move_encoder", 1)])?,
            [[0xb2, 2, 65]]
        );
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", true)],
        )?;
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", false)],
        )?;
        run(&mut state, &[Control::Button("left_deck_switch_a", true)])?;
        // When A's knob turns again, its selection mode is retained without a timeout.
        assert_eq!(
            run(&mut state, &[Control::Encoder("left_move_encoder", -2)])?,
            [[0xb0, 7, 62]]
        );
        assert_eq!(
            state.local_led_state().move_selecting,
            [true, false, true, false]
        );
        // Then finishing selection restores jumping, independently of C.
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", true)],
        )?;
        run(
            &mut state,
            &[Control::Button("left_move_encoder_press", false)],
        )?;
        assert_eq!(
            run(&mut state, &[Control::Encoder("left_move_encoder", 1)])?,
            [[0xb0, 2, 65]]
        );
        assert!(state.move_selecting[2]);
        Ok(())
    }

    #[test]
    fn shifted_move_preserves_selection_and_emits_one_beat_per_detent() -> anyhow::Result<()> {
        for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
            for selecting in [false, true] {
                // Given the deck on the left with Shift held, in either local mode.
                let mut state = Translator {
                    decks: [deck, Deck::B],
                    shifted: [true, false],
                    ..Translator::default()
                };
                state.move_selecting[usize::from(deck.channel())] = selecting;
                // When both directions turn through multiple detents.
                for delta in [2_i32, -3, 0] {
                    let output = run(&mut state, &[Control::Encoder("left_move_encoder", delta)])?;
                    // Then one-beat selection precedes a magnitude-preserving native rotary.
                    let address = Address {
                        channel: deck.channel(),
                        number: 67,
                    };
                    let expected = if delta == 0 {
                        Vec::new()
                    } else {
                        vec![
                            address.press(),
                            address.release(),
                            [0xb0 | deck.channel(), 2, u8::try_from(64 + delta)?],
                        ]
                    };
                    assert_eq!(output, expected);
                }
                assert_eq!(state.move_selecting[usize::from(deck.channel())], selecting);
            }
        }
        Ok(())
    }

    #[test]
    fn shifted_move_restores_confirmed_size_after_every_detent_batch() -> anyhow::Result<()> {
        // Given every independently confirmed deck/size and either direction.
        for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
            for index in 0..feedback::info::JUMP_INTERVALS.len() {
                for delta in [-3_i32, 0, 2] {
                    let mut state = Translator {
                        decks: [deck, Deck::B],
                        shifted: [true, false],
                        ..Translator::default()
                    };
                    let mut feedback = feedback::FeedbackState::default();
                    feedback.apply(
                        feedback::Message {
                            status: 0xb0 | deck.channel(),
                            number: 80 + u8::try_from(index)?,
                            value: 127,
                        },
                        Duration::ZERO,
                    );
                    state.sync_front_panel(&feedback, &mut Vec::new())?;
                    // When Shift+MOVE turns, then one-beat rotary movement precedes one restore.
                    let output = run(&mut state, &[Control::Encoder("left_move_encoder", delta)])?;
                    let one = Address {
                        channel: deck.channel(),
                        number: 67,
                    };
                    let mut expected = Vec::new();
                    if delta != 0 {
                        expected.extend([
                            one.press(),
                            one.release(),
                            [0xb0 | deck.channel(), 2, u8::try_from(64 + delta)?],
                        ]);
                        let restore = Address {
                            channel: deck.channel(),
                            number: 64 + u8::try_from(index)?,
                        };
                        expected.extend([restore.press(), restore.release()]);
                    }
                    assert_eq!(output, expected);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn intermediate_one_beat_feedback_cannot_replace_an_inflight_restore() -> anyhow::Result<()> {
        // Given a confirmed eight-beat selection and a fixed jump already sent.
        let mut state = Translator {
            shifted: [true, false],
            ..Translator::default()
        };
        let mut feedback = feedback::FeedbackState::default();
        let mut decoder = feedback::Decoder::default();
        decoder.feed(&[0xb0, 86, 127], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        run(&mut state, &[Control::Encoder("left_move_encoder", 1)])?;
        decoder.feed(&[0xb0, 86, 0, 83, 127], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        // When a new turn arrives before the eight-beat restoration feedback.
        let output = run(&mut state, &[Control::Encoder("left_move_encoder", -1)])?;
        // Then it still restores eight, not the native command's intermediate one.
        assert_eq!(
            output,
            [
                [0x90, 67, 127],
                [0x80, 67, 0],
                [0xb0, 2, 63],
                [0x90, 70, 127],
                [0x80, 70, 0]
            ]
        );
        Ok(())
    }

    #[test]
    fn newer_confirmed_jump_selection_replaces_inflight_ownership() -> anyhow::Result<()> {
        // Given eight beats captured before a fixed jump.
        let mut state = Translator {
            shifted: [true, false],
            ..Translator::default()
        };
        let mut feedback = feedback::FeedbackState::default();
        let mut decoder = feedback::Decoder::default();
        decoder.feed(&[0xb0, 86, 127], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        run(&mut state, &[Control::Encoder("left_move_encoder", 1)])?;
        // When a newer unambiguous sixteen-beat selection is received.
        decoder.feed(&[0xb0, 86, 0, 87, 127], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        let output = run(&mut state, &[Control::Encoder("left_move_encoder", 1)])?;
        // Then the next jump restores that actual value, not the old eight.
        assert_eq!(output.last(), Some(&[0x80, 71, 0]));
        Ok(())
    }

    #[test]
    fn held_preview_browse_emits_mapped_pulses_without_navigation() -> anyhow::Result<()> {
        // Given the generated mapping and either physical side on every deck.
        let mapping = mapping::generate()?;
        let controls = mapping
            .as_dictionary()
            .and_then(|root| root.get("controls"))
            .and_then(plist::Value::as_array)
            .context("mapping controls missing")?;
        for side in 0..2 {
            for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
                let mut state = Translator::default();
                state.decks[side] = deck;
                run(
                    &mut state,
                    &[Control::Button(
                        if side == 0 {
                            "left_library_play"
                        } else {
                            "right_library_play"
                        },
                        true,
                    )],
                )?;
                let name = if side == 0 {
                    "left_browse_encoder"
                } else {
                    "right_browse_encoder"
                };
                // When both directions turn, including multiple and zero detents.
                for (delta, number, key) in [
                    (2_i32, 23_u8, "musicLibrary.skipForwardPreview"),
                    (-3, 24, "musicLibrary.skipBackwardPreview"),
                    (0, 23, "musicLibrary.skipForwardPreview"),
                ] {
                    let output = run(&mut state, &[Control::Encoder(name, delta)])?;
                    let address = Address {
                        channel: deck.channel(),
                        number,
                    };
                    // Then each detent is a button pulse mapped to the correct action.
                    assert_eq!(
                        output,
                        [address.press(), address.release()]
                            .repeat(usize::try_from(delta.unsigned_abs())?)
                    );
                    assert!(controls.iter().any(|control| {
                        let Some(row) = control.as_dictionary() else {
                            return false;
                        };
                        row.get("keyPath").and_then(plist::Value::as_string) == Some(key)
                            && row
                                .get("midiChannel")
                                .and_then(plist::Value::as_unsigned_integer)
                                == Some(u64::from(address.channel))
                            && row
                                .get("midiData")
                                .and_then(plist::Value::as_unsigned_integer)
                                == Some(u64::from(number))
                            && row
                                .get("midiMessageType")
                                .and_then(plist::Value::as_unsigned_integer)
                                == Some(1)
                    }));
                }
            }
        }
        Ok(())
    }

    #[test]
    fn shifted_browse_selects_sources_until_preview_is_held() -> anyhow::Result<()> {
        // Given either side on each deck with Shift held.
        for side in 0..2 {
            for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
                let mut state = Translator::default();
                state.decks[side] = deck;
                state.shifted[side] = true;
                let (browse, preview) = if side == 0 {
                    ("left_browse_encoder", "left_library_play")
                } else {
                    ("right_browse_encoder", "right_library_play")
                };
                for held in [false, true, false] {
                    run(&mut state, &[Control::Button(preview, held)])?;
                    // When the encoder turns through positive, negative and zero detents.
                    for (delta, source, seek) in [(2_i32, 25, 23), (-3, 26, 24), (0, 25, 23)] {
                        let output = run(&mut state, &[Control::Encoder(browse, delta)])?;
                        let address = Address {
                            channel: deck.channel() + 8,
                            number: if held { seek } else { source },
                        };
                        // Then only the selected source/held-preview route receives pulses.
                        assert_eq!(
                            output,
                            [address.press(), address.release()]
                                .repeat(usize::try_from(delta.unsigned_abs())?)
                        );
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn browse_uses_current_preview_hold_and_restores_navigation_on_release() -> anyhow::Result<()> {
        // Given either side, with Preview and rotation in one unordered snapshot.
        for (name, preview, channel) in [
            ("left_browse_encoder", "left_library_play", 0_u8),
            ("right_browse_encoder", "right_library_play", 1),
        ] {
            let mut state = Translator::default();
            // When Preview is pressed or released in the same snapshot as a turn.
            let seek = run(
                &mut state,
                &[Control::Encoder(name, 1), Control::Button(preview, true)],
            )?;
            let browse = run(
                &mut state,
                &[Control::Encoder(name, -1), Control::Button(preview, false)],
            )?;
            // Then toggles bracket the held scrub route and browsing returns immediately.
            assert_eq!(
                seek,
                [
                    [0x90 | channel, 13, 127],
                    [0x80 | channel, 13, 0],
                    [0x90 | channel, 23, 127],
                    [0x80 | channel, 23, 0]
                ]
            );
            assert_eq!(
                browse,
                [
                    [0x90 | channel, 13, 127],
                    [0x80 | channel, 13, 0],
                    [0xb0 | channel, 3, 63],
                ]
            );
        }
        Ok(())
    }

    #[test]
    fn preview_toggles_once_per_hold_and_closes_on_release_or_shutdown() -> anyhow::Result<()> {
        for shutdown in [false, true] {
            // Given Preview held on A, including a duplicate held snapshot.
            let mut state = Translator::default();
            assert_eq!(
                run(&mut state, &[Control::Button("left_library_play", true)])?,
                [[0x90, 13, 127], [0x80, 13, 0]]
            );
            assert!(run(&mut state, &[Control::Button("left_library_play", true)])?.is_empty());
            run(
                &mut state,
                &[
                    Control::Button("left_deck_switch_c", true),
                    Control::Button("left_shift", true),
                ],
            )?;
            // When the physical button releases or the bridge stops.
            let mut output = Vec::new();
            if shutdown {
                state.release_all(&mut output);
            } else {
                state.translate(
                    &[Control::Button("left_library_play", false)],
                    None,
                    &mut output,
                )?;
            }
            // Then a complete second toggle uses the original address, with no stuck note.
            assert_eq!(output, [[0x90, 13, 127], [0x80, 13, 0]]);
            assert!(run(&mut state, &[Control::Button("left_library_play", false)])?.is_empty());
        }
        Ok(())
    }

    #[test]
    fn overlapping_preview_buttons_keep_preview_open_until_last_release() -> anyhow::Result<()> {
        // Given both physical Preview buttons held.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_library_play", true)])?;
        assert!(run(&mut state, &[Control::Button("right_library_play", true)])?.is_empty());
        // When one side releases.
        let output = run(&mut state, &[Control::Button("left_library_play", false)])?;
        // Then it does not stop the other side's preview or suppress its scrub.
        assert!(output.is_empty());
        assert_eq!(
            run(&mut state, &[Control::Encoder("right_browse_encoder", -1)])?,
            [[0x91, 24, 127], [0x81, 24, 0]]
        );
        assert_eq!(
            run(&mut state, &[Control::Button("right_library_play", false)])?,
            [[0x91, 13, 127], [0x81, 13, 0]]
        );
        Ok(())
    }

    #[test]
    fn shift_cue_keeps_original_deck_ownership_on_release() -> anyhow::Result<()> {
        // Given Shift+Cue pressed on A in one unordered snapshot.
        let mut state = Translator::default();
        assert_eq!(
            run(
                &mut state,
                &[
                    Control::Button("left_cue", true),
                    Control::Button("left_shift", true),
                ],
            )?,
            [[0x98, 1, 127]]
        );
        run(
            &mut state,
            &[
                Control::Button("left_shift", false),
                Control::Button("left_deck_switch_c", true),
            ],
        )?;
        // When Cue releases after changing deck and Shift.
        let output = run(&mut state, &[Control::Button("left_cue", false)])?;
        // Then the shifted A address receives the release.
        assert_eq!(output, [[0x88, 1, 0]]);
        Ok(())
    }

    #[test]
    fn move_selection_precedes_rotation_regardless_of_descriptor_order() -> anyhow::Result<()> {
        // Given a size-mode toggle simultaneous with rotation and a right-side D selector.
        let mut state = Translator::default();
        let output = run(
            &mut state,
            &[
                Control::Encoder("right_move_encoder", 1),
                Control::Button("right_move_encoder_press", true),
                Control::Button("right_deck_switch_d", true),
            ],
        )?;
        // Then the movement changes D's size, never jumps B or D.
        assert!(output.contains(&[0xb3, 7, 65]));
        assert!(!output.iter().any(|m| m[0] & 0xf0 == 0xb0 && m[1] == 2));
        assert_eq!(state.move_selecting, [false, false, false, true]);
        Ok(())
    }

    #[test]
    fn changing_untouched_jog_mode_does_not_leak_old_fractional_motion() -> anyhow::Result<()> {
        // Given an incomplete grid step.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_grid", true)])?;
        run(&mut state, &[Control::Jog("left_jog_wheel", 0.007)])?;
        run(
            &mut state,
            &[
                Control::Button("left_grid", false),
                Control::Button("left_shift", true),
            ],
        )?;
        // When a small movement arrives in seek mode.
        let output = run(&mut state, &[Control::Jog("left_jog_wheel", 0.001)])?;
        // Then only the new count is sent, with no old grid remainder.
        assert_eq!(output, [[0xb0, 6, 65]]);
        assert!(state.residuals[0].abs() < f64::EPSILON);
        Ok(())
    }

    #[test]
    fn relative_encoding_preserves_large_positive_and_negative_deltas() -> anyhow::Result<()> {
        for delta in [130_i32, -130] {
            // Given a large signed encoder delta.
            let mut state = Translator::default();
            // When it is translated.
            let output = run(&mut state, &[Control::Encoder("left_loop_encoder", delta)])?;
            // Then bounded chunks preserve its magnitude and sign.
            let decoded: i32 = output
                .iter()
                .map(|message| i32::from(message[2]) - 64)
                .sum();
            assert_eq!(decoded, delta);
            assert_eq!(output.len(), 3);
        }
        Ok(())
    }

    #[test]
    fn front_panel_assignment_changes_are_atomic() -> anyhow::Result<()> {
        // Given all selectors initially Through and an established snapshot.
        let mut state = Translator {
            front_panel_ready: true,
            ..Translator::default()
        };
        let mut payload = [0_u8; 22];
        payload[17] = 0x55;
        let mut ignored = Vec::new();
        state.translate(&[], Some(&payload), &mut ignored)?;
        payload[17] = 0x95; // Channel 3 -> Left, with all other selectors unchanged.
        // When the next whole snapshot arrives.
        let mut output = Vec::new();
        state.translate(&[], Some(&payload), &mut output)?;
        // Then one Left action fires, with no transient Through/Right action.
        assert_eq!(output, [[0x94, 88, 127], [0x84, 88, 0]]);
        Ok(())
    }

    #[test]
    fn startup_crossfader_waits_for_mapping_feedback_and_uses_latest_snapshot() -> anyhow::Result<()>
    {
        // Given two physical snapshots before Djay's mapping sends playback state.
        let mut state = Translator::default();
        let mut feedback = feedback::FeedbackState::default();
        let mut decoder = feedback::Decoder::default();
        let mut payload = [0_u8; 22];
        payload[17] = 0x55;
        assert!(run_front_snapshot(&mut state, &payload)?.is_empty());
        payload[17] = 0x26; // A left, B through, C right, D left.
        payload[18] = 2;
        assert!(run_front_snapshot(&mut state, &payload)?.is_empty());
        let mut output = Vec::new();
        decoder.feed(&[0x90, 0, 127, 0xb6, 0, 0], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut output)?;
        assert!(output.is_empty()); // Lamps/meters are not the playback gate.
        // When real playback feedback arrives, even paused followed by track-unloaded.
        decoder.feed(&[0xb6, 4, 0, 0x96, 0, 0], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut output)?;
        // Then the latest four assignments and curve emit exactly once.
        assert_eq!(
            output,
            [
                [0x94, 82, 127],
                [0x84, 82, 0],
                [0x94, 84, 127],
                [0x84, 84, 0],
                [0x94, 86, 127],
                [0x84, 86, 0],
                [0x94, 91, 127],
                [0x84, 91, 0],
                [0xb4, 25, 0],
            ]
        );
        output.clear();
        decoder.feed(&[0xb6, 4, 127], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut output)?;
        assert!(output.is_empty()); // Later UI/playback changes cannot replay selectors.
        assert!(run_front_snapshot(&mut state, &payload)?.is_empty());
        Ok(())
    }

    #[test]
    fn startup_applies_only_latest_channel_volume_and_eq_in_either_launch_order()
    -> anyhow::Result<()> {
        for feedback_first in [false, true] {
            // Given the full physical analog baseline, including zero and maximum.
            let mut state = Translator::default();
            let mut feedback = feedback::FeedbackState::default();
            let baseline: Vec<_> = catalog::MIXER_BINDINGS
                .iter()
                .filter(|binding| (4..20).contains(&binding.number))
                .map(|binding| Control::Slider(binding.input, 0.0))
                .chain([
                    Control::Slider("fx1_knob_1", 1.0),
                    Control::Slider("left_tempo_fader", 1.0),
                    Control::Slider("crossfader", 1.0),
                    Control::Slider("ch1_gain", 1.0),
                    Control::Slider("ch1_quick_fx", 1.0),
                ])
                .collect();
            let mut output = Vec::new();
            feedback.apply(
                feedback::Message {
                    status: 0xb6,
                    number: 4,
                    value: 0,
                },
                Duration::ZERO,
            );
            if feedback_first {
                state.sync_front_panel(&feedback, &mut output)?;
            }
            state.observe_analog(&baseline)?;
            state.observe_analog(&[Control::Slider("ch2_fader", 1.0)])?;
            // When the other half of startup becomes available.
            if feedback_first {
                state.translate(&[], None, &mut output)?;
            } else {
                state.sync_front_panel(&feedback, &mut output)?;
            }
            // Then exactly sixteen authorized controls apply their latest values.
            let expected: Vec<_> = (4_u8..20)
                .map(|number| [0xb4, number, if number == 17 { 127 } else { 0 }])
                .collect();
            assert_eq!(output, expected);
            output.clear();
            state.sync_front_panel(&feedback, &mut output)?;
            state.translate(&[Control::Slider("ch2_fader", 1.0)], None, &mut output)?;
            assert!(
                output.is_empty(),
                "unchanged physical state must not undo mouse edits"
            );
            state.translate(&[Control::Slider("ch1_fader", 0.5)], None, &mut output)?;
            assert_eq!(output, [[0xb4, 16, 64]]);
        }
        Ok(())
    }

    #[test]
    fn shifted_fx_knobs_browse_relative_steps_on_each_target() -> anyhow::Result<()> {
        for side in 0..2 {
            for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
                for (slot, name) in [
                    ["fx1_knob_1", "fx1_knob_2", "fx1_knob_3"],
                    ["fx2_knob_1", "fx2_knob_2", "fx2_knob_3"],
                ][side]
                    .into_iter()
                    .enumerate()
                {
                    // Given a known physical position, independent FX target and Shift.
                    let mut state = Translator::default();
                    state.fx[side] = deck;
                    state.observe_analog(&[Control::Slider(name, 64.0 / 127.0)])?;
                    run(
                        &mut state,
                        &[Control::Button(
                            if side == 0 {
                                "left_shift"
                            } else {
                                "right_shift"
                            },
                            true,
                        )],
                    )?;
                    // When small movement accumulates, then whole relative steps are emitted.
                    assert!(run(&mut state, &[Control::Slider(name, 67.0 / 127.0)])?.is_empty());
                    let address = [0xb8 | deck.channel(), 32 + u8::try_from(slot)?, 65];
                    assert_eq!(
                        run(&mut state, &[Control::Slider(name, 68.0 / 127.0)])?,
                        [address]
                    );
                    assert!(run(&mut state, &[Control::Slider(name, 68.0 / 127.0)])?.is_empty());
                    assert_eq!(
                        run(&mut state, &[Control::Slider(name, 80.0 / 127.0)])?,
                        [[address[0], address[1], 67]]
                    );
                    assert_eq!(
                        run(&mut state, &[Control::Slider(name, 72.0 / 127.0)])?,
                        [[address[0], address[1], 62]]
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn shifted_fx_browsing_rebases_on_shift_and_target_changes() -> anyhow::Result<()> {
        // Given a partial step on A while Shift is held.
        let mut state = Translator::default();
        state.observe_analog(&[Control::Slider("fx1_knob_1", 64.0 / 127.0)])?;
        run(&mut state, &[Control::Button("left_shift", true)])?;
        assert!(run(&mut state, &[Control::Slider("fx1_knob_1", 67.0 / 127.0)])?.is_empty());
        // When retargeting and moving together, only new travel belongs to C.
        assert!(
            run(
                &mut state,
                &[
                    Control::Slider("fx1_knob_1", 68.0 / 127.0),
                    Control::Button("mixer_ch3_fx1", true),
                ]
            )?
            .is_empty()
        );
        assert_eq!(
            run(&mut state, &[Control::Slider("fx1_knob_1", 71.0 / 127.0)])?,
            [[0xba, 32, 65]]
        );
        // Then releasing Shift permits repositioning; reholding starts a new origin.
        run(&mut state, &[Control::Button("left_shift", false)])?;
        assert!(run(&mut state, &[Control::Slider("fx1_knob_1", 0.0)])?.is_empty());
        run(&mut state, &[Control::Button("left_shift", true)])?;
        assert_eq!(
            run(&mut state, &[Control::Slider("fx1_knob_1", 1.0)])?,
            [[0xba, 32, 95]]
        );
        assert!(run(&mut state, &[Control::Slider("fx1_knob_1", 1.0)])?.is_empty());
        assert_eq!(
            run(&mut state, &[Control::Slider("fx1_knob_1", 0.0)])?,
            [[0xba, 32, 33]]
        );
        Ok(())
    }

    #[test]
    fn shifted_fx_first_sample_only_seeds_browsing_and_dry_wet_stays_absolute() -> anyhow::Result<()>
    {
        // Given Shift before any physical knob baseline and known wet/dry feedback.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("right_shift", true)])?;
        let mut feedback = feedback::FeedbackState::default();
        feedback::Decoder::default().feed(&[0xb1, 35, 0, 36, 0, 37, 0], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        // When initial positions arrive, then no effect is selected by absolute position.
        assert_eq!(
            run(
                &mut state,
                &[
                    Control::Slider("fx2_knob_2", 1.0),
                    Control::Slider("fx2_dry_wet", 0.0),
                ]
            )?,
            [[0xb1, 43, 0], [0xb1, 44, 0], [0xb1, 45, 0]]
        );
        assert_eq!(
            run(&mut state, &[Control::Slider("fx2_knob_2", 123.0 / 127.0)])?,
            [[0xb9, 33, 63]]
        );
        Ok(())
    }

    #[test]
    fn shifted_fx_browsing_rearms_parameter_pickup() -> anyhow::Result<()> {
        // Given an acquired parameter at 64 and a matching real feedback cache.
        let mut state = Translator::default();
        state.observe_analog(&[Control::Slider("fx1_knob_1", 64.0 / 127.0)])?;
        let mut feedback = feedback::FeedbackState::default();
        feedback::Decoder::default().feed(&[0xb0, 32, 64], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        run(&mut state, &[Control::Slider("fx1_knob_1", 64.0 / 127.0)])?;
        // When Shift and movement arrive together, browsing replaces parameter output.
        assert_eq!(
            run(
                &mut state,
                &[
                    Control::Slider("fx1_knob_1", 80.0 / 127.0),
                    Control::Button("left_shift", true),
                ]
            )?,
            [[0xb8, 32, 68]]
        );
        run(&mut state, &[Control::Button("left_shift", false)])?;
        // Then repositioning away from the software value cannot alter the parameter.
        assert!(run(&mut state, &[Control::Slider("fx1_knob_1", 100.0 / 127.0)])?.is_empty());
        assert_eq!(
            run(&mut state, &[Control::Slider("fx1_knob_1", 60.0 / 127.0)])?,
            [[0xb0, 32, 60]]
        );
        Ok(())
    }

    #[test]
    fn fx_retarget_requires_pickup_but_feedback_lag_does_not_block_acquired_motion()
    -> anyhow::Result<()> {
        // Given a physical knob above C's confirmed software position.
        let mut state = Translator::default();
        state.observe_analog(&[Control::Slider("fx1_knob_1", 0.9)])?;
        let mut feedback = feedback::FeedbackState::default();
        let mut decoder = feedback::Decoder::default();
        decoder.feed(&[0xb2, 32, 64], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        run(&mut state, &[Control::Button("mixer_ch3_fx1", true)])?;
        // When movement approaches, crosses, then continues before feedback catches up.
        assert!(run(&mut state, &[Control::Slider("fx1_knob_1", 0.8)])?.is_empty());
        assert_eq!(
            run(&mut state, &[Control::Slider("fx1_knob_1", 0.4)])?,
            [[0xb2, 32, 51]]
        );
        assert_eq!(
            run(&mut state, &[Control::Slider("fx1_knob_1", 0.2)])?,
            [[0xb2, 32, 25]]
        );
        decoder.feed(&[0xb2, 32, 51, 32, 25], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        // Then a subsequent external software edit requires pickup again.
        decoder.feed(&[0xb2, 32, 110], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        assert!(run(&mut state, &[Control::Slider("fx1_knob_1", 0.3)])?.is_empty());
        Ok(())
    }

    #[test]
    fn dry_wet_acquires_each_software_slot_independently() -> anyhow::Result<()> {
        // Given three different real wet/dry positions on A and physical zero.
        let mut state = Translator::default();
        state.observe_analog(&[Control::Slider("fx1_dry_wet", 0.0)])?;
        let mut feedback = feedback::FeedbackState::default();
        feedback::Decoder::default().feed(&[0xb0, 35, 0, 36, 64, 37, 127], |message| {
            feedback.apply(message, Duration::ZERO);
        });
        state.sync_front_panel(&feedback, &mut Vec::new())?;
        // When the shared knob reaches the midpoint, then the maximum slot stays unchanged.
        assert_eq!(
            run(&mut state, &[Control::Slider("fx1_dry_wet", 0.5)])?,
            [[0xb0, 35, 64], [0xb0, 36, 64]]
        );
        Ok(())
    }

    fn run_front_snapshot(
        state: &mut Translator,
        payload: &[u8; 22],
    ) -> anyhow::Result<Vec<[u8; 3]>> {
        let mut output = Vec::new();
        state.translate(&[], Some(payload), &mut output)?;
        Ok(output)
    }

    #[test]
    fn startup_crossfader_handles_both_launch_orders_and_all_selector_positions()
    -> anyhow::Result<()> {
        for feedback_first in [false, true] {
            for (packed, assignments) in [
                (0x00, [0_u8; 4]),
                (0x55, [1; 4]),
                (0xaa, [2; 4]),
                (0x26, [2, 1, 0, 2]),
            ] {
                for (curve, cc) in [(0, 127), (1, 64), (2, 0)] {
                    // Given a fresh bridge with the physical switches already set.
                    let mut state = Translator::default();
                    let mut feedback = feedback::FeedbackState::default();
                    let mut decoder = feedback::Decoder::default();
                    decoder.feed(&[0xb6, 7, 0], |message| {
                        feedback.apply(message, Duration::ZERO);
                    });
                    let mut payload = [0_u8; 22];
                    payload[17] = packed;
                    payload[18] = curve;
                    let mut output = Vec::new();
                    // When playback feedback and the snapshot arrive in either order.
                    if feedback_first {
                        state.sync_front_panel(&feedback, &mut output)?;
                    }
                    state.translate(&[], Some(&payload), &mut output)?;
                    if !feedback_first {
                        state.sync_front_panel(&feedback, &mut output)?;
                    }
                    // Then every assignment and curve is applied once, without moving hardware.
                    let mut expected = Vec::new();
                    for (deck, assignment) in assignments.into_iter().enumerate() {
                        let note = 80 + 3 * u8::try_from(deck)? + assignment;
                        expected.extend([[0x94, note, 127], [0x84, note, 0]]);
                    }
                    expected.push([0xb4, 25, cc]);
                    assert_eq!(output, expected);
                    assert!(run_front_snapshot(&mut state, &payload)?.is_empty());
                }
            }
        }
        Ok(())
    }

    #[test]
    fn shutdown_releases_owned_notes_and_duplicate_edges_do_not_toggle_again() -> anyhow::Result<()>
    {
        // Given a held Play and an ignored duplicate press.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_play", true)])?;
        assert!(run(&mut state, &[Control::Button("left_play", true)])?.is_empty());
        // When shutdown releases ownership.
        let mut output = Vec::new();
        state.release_all(&mut output);
        // Then exactly one release is emitted.
        assert_eq!(output, [[0x80, 0, 0]]);
        assert!(state.held.is_empty());
        Ok(())
    }

    #[test]
    fn reverse_hold_releases_original_deck_after_selector_changes_or_shutdown() -> anyhow::Result<()>
    {
        for deck in [Deck::A, Deck::B, Deck::C, Deck::D] {
            for shutdown in [false, true] {
                // Given REV held on each deck with a duplicate input edge.
                let side = usize::from(deck.channel() % 2);
                let name = if side == 0 {
                    "left_reverse"
                } else {
                    "right_reverse"
                };
                let mut state = Translator::default();
                state.decks[side] = deck;
                assert_eq!(
                    run(&mut state, &[Control::Button(name, true)])?,
                    [[0x90 | deck.channel(), 3, 127]]
                );
                assert!(run(&mut state, &[Control::Button(name, true)])?.is_empty());
                run(
                    &mut state,
                    &[
                        Control::Button(
                            if side == 0 {
                                "left_deck_switch_c"
                            } else {
                                "right_deck_switch_d"
                            },
                            true,
                        ),
                        Control::Button(
                            if side == 0 {
                                "left_shift"
                            } else {
                                "right_shift"
                            },
                            true,
                        ),
                    ],
                )?;
                // When the physical button releases or the bridge shuts down.
                let mut output = Vec::new();
                if shutdown {
                    state.release_all(&mut output);
                } else {
                    state.translate(&[Control::Button(name, false)], None, &mut output)?;
                }
                // Then only its original deck receives release, including with Shift now held.
                assert_eq!(output, [[0x80 | deck.channel(), 3, 0]]);
                assert!(!state.held.contains_key(name));
            }
        }
        Ok(())
    }

    #[test]
    fn led_routing_follows_current_selection_while_held_notes_keep_old_ownership()
    -> anyhow::Result<()> {
        // Given Cue held on A.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_cue", true)])?;
        // When C becomes the visible deck.
        run(&mut state, &[Control::Button("left_deck_switch_c", true)])?;
        // Then lights show C while the held Cue still belongs to A.
        assert_eq!(state.local_led_state().decks[0], 2);
        assert_eq!(state.held["left_cue"][0].channel, 0);
        Ok(())
    }

    #[test]
    fn screen_pages_follow_real_selectors_without_changing_held_note_ownership()
    -> anyhow::Result<()> {
        // Given Cue held on A.
        let mut state = Translator::default();
        run(&mut state, &[Control::Button("left_cue", true)])?;
        let feedback = feedback::FeedbackState::default();
        let initial = screens::pages(&state, &feedback);
        // When C, STEMS, and Shift become the physical left side's current layer.
        run(
            &mut state,
            &[
                Control::Button("left_deck_switch_c", true),
                Control::Button("left_stems_mode", true),
                Control::Button("left_shift", true),
            ],
        )?;
        let changed = screens::pages(&state, &feedback);
        // Then the screen changes only that side while Cue retains its original owner.
        assert!(matches!(
            changed[0],
            s4mk3_hid_midi_bridge::screens::page::Page::Stems { shifted: true, .. }
        ));
        assert_ne!(changed[0], initial[0]);
        assert_eq!(changed[1], initial[1]);
        assert_eq!(state.held["left_cue"][0].channel, 0);
        Ok(())
    }

    #[test]
    fn momentary_led_release_survives_shift_and_deck_changes() -> anyhow::Result<()> {
        // Given a held library preview after a deck/modifier change.
        let mut state = Translator::default();
        run(
            &mut state,
            &[
                Control::Button("left_library_play", true),
                Control::Button("left_deck_switch_c", true),
                Control::Button("left_shift", true),
            ],
        )?;
        // When the physical preview button releases.
        run(&mut state, &[Control::Button("left_library_play", false)])?;
        // Then the momentary light is not stranded on the old layer.
        assert!(!state.local_led_state().held[4]);
        Ok(())
    }
}
