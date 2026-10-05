//! Shared physical-input catalog for the full S4 MK3 mapping profile.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadMode {
    Hotcue,
    Samples,
    Stems,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Note,
    Absolute,
    Relative,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Binding {
    pub input: &'static str,
    pub number: u8,
    pub kind: Kind,
    pub target: &'static str,
    pub shifted: Option<&'static str>,
    pub pickup: bool,
    pub flipped: bool,
    pub sensitivity: Option<f64>,
}

const fn binding(
    input: &'static str,
    number: u8,
    kind: Kind,
    target: &'static str,
    shifted: Option<&'static str>,
) -> Binding {
    Binding {
        input,
        number,
        kind,
        target,
        shifted,
        pickup: false,
        flipped: false,
        sensitivity: None,
    }
}

const fn cc(
    input: &'static str,
    number: u8,
    target: &'static str,
    shifted: Option<&'static str>,
) -> Binding {
    binding(input, number, Kind::Absolute, target, shifted)
}

const fn relative(
    input: &'static str,
    number: u8,
    target: &'static str,
    shifted: Option<&'static str>,
) -> Binding {
    binding(input, number, Kind::Relative, target, shifted)
}

const fn note(
    input: &'static str,
    number: u8,
    target: &'static str,
    shifted: Option<&'static str>,
) -> Binding {
    binding(input, number, Kind::Note, target, shifted)
}

/// Deck suffixes are prefixed with `left_` / `right_` by the HID bridge.
pub const DECK_BINDINGS: &[Binding] = &[
    note("play", 0, "turntable{deck}.playPause", None),
    note(
        "cue",
        1,
        "turntable{deck}.cuePositionOrJumpConsideringPlayState1",
        None,
    ),
    note("sync", 2, "turntable{deck}.bpmSync", None),
    note("reverse", 3, "turntable{deck}.reverse", None),
    note("flux", 4, "turntable{deck}.deckSlipToggle", None),
    note("move_encoder_press", 5, "turntable{deck}.loopInOut", None),
    note(
        "loop_encoder_press",
        6,
        "turntable{deck}.autoLoopOnOff",
        Some("turntable{deck}.loopInOut"),
    ),
    note(
        "wheel_touch",
        7,
        "turntable{deck}.scratchingMode",
        Some("turntable{deck}.jogSeekMode"),
    ),
    Binding {
        pickup: true,
        flipped: true,
        ..binding(
            "tempo_fader",
            0,
            Kind::Absolute,
            "turntable{deck}.speed",
            Some("turntable{deck}.speedRelative"),
        )
    },
    relative(
        "loop_encoder",
        1,
        "turntable{deck}.autoLoopDurationRotary",
        Some("turntable{deck}.autoLoopMoveRotary"),
    ),
    relative(
        "move_encoder",
        2,
        "turntable{deck}.autoLoopMoveRotary",
        Some("turntable{deck}.skipRotary"),
    ),
    relative(
        "browse_encoder",
        3,
        "musicLibrary.libraryRotary",
        Some("musicLibrary.sectionRotary"),
    ),
    Binding {
        sensitivity: Some(7.0),
        ..relative("jog_bend", 4, "turntable{deck}.pitchBendMove", None)
    },
    Binding {
        sensitivity: Some(25.0),
        ..relative("jog_scratch", 5, "turntable{deck}.scratchingMove", None)
    },
    Binding {
        sensitivity: Some(20.0),
        ..relative("jog_seek", 6, "turntable{deck}.jogSeekMove", None)
    },
    note("master", 8, "turntable{deck}.turntableIsSyncMaster", None),
    note("library_view", 9, "musicLibrary.toggleLibraryVisible", None),
    note("library_playlist", 10, "musicLibrary.focusQueue", None),
    note(
        "library_star",
        11,
        "musicLibrary.markUnmarkSelectedSongs",
        None,
    ),
    note(
        "browse_press",
        12,
        "musicLibrary.load{deck}",
        Some("musicLibrary.libraryBack"),
    ),
    note("library_play", 13, "musicLibrary.togglePreview", None),
    note("quantize", 15, "turntable{deck}.toggleQuantize", None),
    note("record_mode", 22, "turntable{deck}.recordSample", None),
    note("grid_left", 29, "turntable{deck}.shiftDownBeatLeft", None),
    note("grid_right", 30, "turntable{deck}.shiftDownBeatRight", None),
];

/// Left strip suffixes are shared by button and knob event families.
pub const FX_LEFT_BINDINGS: &[Binding] = &[
    note("fx_on", 32, "turntable{deck}.fxActive", None),
    note(
        "fx_1",
        33,
        "turntable{deck}.fx1Enabled",
        Some("turntable{deck}.fx1SelectNext"),
    ),
    note(
        "fx_2",
        34,
        "turntable{deck}.fx2Enabled",
        Some("turntable{deck}.fx2SelectNext"),
    ),
    note(
        "fx_3",
        35,
        "turntable{deck}.fx3Enabled",
        Some("turntable{deck}.fx3SelectNext"),
    ),
    Binding {
        pickup: true,
        ..cc("knob_1", 32, "turntable{deck}.fx1ParameterValue", None)
    },
    Binding {
        pickup: true,
        ..cc("knob_2", 33, "turntable{deck}.fx2ParameterValue", None)
    },
    Binding {
        pickup: true,
        ..cc("knob_3", 34, "turntable{deck}.fx3ParameterValue", None)
    },
    Binding {
        pickup: true,
        ..cc("dry_wet", 35, "turntable{deck}.fx1WetDryValue", None)
    },
    Binding {
        pickup: true,
        ..cc("dry_wet", 36, "turntable{deck}.fx2WetDryValue", None)
    },
    Binding {
        pickup: true,
        ..cc("dry_wet", 37, "turntable{deck}.fx3WetDryValue", None)
    },
];

/// Right-strip button suffixes occupy a distinct MIDI note range.
pub const FX_RIGHT_BINDINGS: &[Binding] = &[
    note("fx_on", 36, "turntable{deck}.fxActive", None),
    note(
        "fx_1",
        37,
        "turntable{deck}.fx1Enabled",
        Some("turntable{deck}.fx1SelectNext"),
    ),
    note(
        "fx_2",
        38,
        "turntable{deck}.fx2Enabled",
        Some("turntable{deck}.fx2SelectNext"),
    ),
    note(
        "fx_3",
        39,
        "turntable{deck}.fx3Enabled",
        Some("turntable{deck}.fx3SelectNext"),
    ),
    Binding {
        pickup: true,
        ..cc("knob_1", 40, "turntable{deck}.fx1ParameterValue", None)
    },
    Binding {
        pickup: true,
        ..cc("knob_2", 41, "turntable{deck}.fx2ParameterValue", None)
    },
    Binding {
        pickup: true,
        ..cc("knob_3", 42, "turntable{deck}.fx3ParameterValue", None)
    },
    Binding {
        pickup: true,
        ..cc("dry_wet", 43, "turntable{deck}.fx1WetDryValue", None)
    },
    Binding {
        pickup: true,
        ..cc("dry_wet", 44, "turntable{deck}.fx2WetDryValue", None)
    },
    Binding {
        pickup: true,
        ..cc("dry_wet", 45, "turntable{deck}.fx3WetDryValue", None)
    },
];

/// `MIXER_BINDINGS` use full Encdr input names and fully qualified native keys.
pub const MIXER_BINDINGS: &[Binding] = &[
    cc("ch1_fader", 16, "mixer.lineVolume1", None),
    cc("ch2_fader", 17, "mixer.lineVolume2", None),
    cc("ch3_fader", 18, "mixer.lineVolume3", None),
    cc("ch4_fader", 19, "mixer.lineVolume4", None),
    cc("crossfader", 20, "mixer.crossfade", None),
    cc("ch1_gain", 0, "turntable1.gain", None),
    cc("ch2_gain", 1, "turntable2.gain", None),
    cc("ch3_gain", 2, "turntable3.gain", None),
    cc("ch4_gain", 3, "turntable4.gain", None),
    cc("ch1_eq_high", 4, "turntable1.highEQ", None),
    cc("ch2_eq_high", 5, "turntable2.highEQ", None),
    cc("ch3_eq_high", 6, "turntable3.highEQ", None),
    cc("ch4_eq_high", 7, "turntable4.highEQ", None),
    cc("ch1_eq_mid", 8, "turntable1.midEQ", None),
    cc("ch2_eq_mid", 9, "turntable2.midEQ", None),
    cc("ch3_eq_mid", 10, "turntable3.midEQ", None),
    cc("ch4_eq_mid", 11, "turntable4.midEQ", None),
    cc("ch1_eq_low", 12, "turntable1.lowEQ", None),
    cc("ch2_eq_low", 13, "turntable2.lowEQ", None),
    cc("ch3_eq_low", 14, "turntable3.lowEQ", None),
    cc("ch4_eq_low", 15, "turntable4.lowEQ", None),
    cc("ch1_quick_fx", 21, "turntable1.filter", None),
    cc("ch2_quick_fx", 22, "turntable2.filter", None),
    cc("ch3_quick_fx", 23, "turntable3.filter", None),
    cc("ch4_quick_fx", 24, "turntable4.filter", None),
    note("mixer_ch1_pfl", 0, "mixer.monitorActive1", None),
    note("mixer_ch2_pfl", 1, "mixer.monitorActive2", None),
    note("mixer_ch3_pfl", 2, "mixer.monitorActive3", None),
    note("mixer_ch4_pfl", 3, "mixer.monitorActive4", None),
];

/// Quick-FX controls use the locally selected deck, not a mixer audio bus.
pub const QUICK_BINDINGS: &[Binding] = &[
    note("mixer_fx_1", 16, "turntable{deck}.instantFx1", None),
    note("mixer_fx_2", 17, "turntable{deck}.instantFx2", None),
    note("mixer_fx_3", 18, "turntable{deck}.instantFx3", None),
    note("mixer_fx_4", 19, "turntable{deck}.instantFx4", None),
    note("mixer_fx_filter", 20, "turntable{deck}.resetFilter", None),
];

pub const CURVE_BINDING: Binding = Binding {
    input: "crossfader_curve",
    number: 25,
    kind: Kind::Absolute,
    target: "mixer.crossfadeStyle",
    shifted: None,
    pickup: false,
    flipped: false,
    sensitivity: None,
};

pub fn assignment_binding(deck: u8, state: u8) -> Option<Binding> {
    let deck_index = deck.checked_sub(1)?;
    let targets = [
        [
            "mixer.crossfadeAssignment1Right",
            "mixer.crossfadeAssignment1Through",
            "mixer.crossfadeAssignment1Left",
        ],
        [
            "mixer.crossfadeAssignment2Right",
            "mixer.crossfadeAssignment2Through",
            "mixer.crossfadeAssignment2Left",
        ],
        [
            "mixer.crossfadeAssignment3Right",
            "mixer.crossfadeAssignment3Through",
            "mixer.crossfadeAssignment3Left",
        ],
        [
            "mixer.crossfadeAssignment4Right",
            "mixer.crossfadeAssignment4Through",
            "mixer.crossfadeAssignment4Left",
        ],
    ];
    let target = *targets
        .get(usize::from(deck_index))?
        .get(usize::from(state))?;
    let number = 80_u8
        .checked_add(deck_index.checked_mul(3)?)?
        .checked_add(state)?;
    Some(note("crossfader_assign", number, target, None))
}

pub fn deck_selection_binding(deck: u8) -> Option<Binding> {
    let index = deck.checked_sub(1)?;
    let target = *[
        "application.turntable1Selected",
        "application.turntable2Selected",
        "application.turntable3Selected",
        "application.turntable4Selected",
    ]
    .get(usize::from(index))?;
    Some(note("deck_select", index, target, None))
}

/// Return the binding for a zero-based physical pad index.
pub const fn pad_binding(mode: PadMode, index: u8) -> Option<Binding> {
    if index >= 8 {
        return None;
    }
    let base = match mode {
        PadMode::Hotcue => 40,
        PadMode::Samples => 48,
        PadMode::Stems => 56,
    };
    let target = match mode {
        PadMode::Hotcue => match index {
            0 => "turntable{deck}.cueOrJumpIfAlreadySet1",
            1 => "turntable{deck}.cueOrJumpIfAlreadySet2",
            2 => "turntable{deck}.cueOrJumpIfAlreadySet3",
            3 => "turntable{deck}.cueOrJumpIfAlreadySet4",
            4 => "turntable{deck}.cueOrJumpIfAlreadySet5",
            5 => "turntable{deck}.cueOrJumpIfAlreadySet6",
            6 => "turntable{deck}.cueOrJumpIfAlreadySet7",
            _ => "turntable{deck}.cueOrJumpIfAlreadySet8",
        },
        PadMode::Samples => match index {
            0 => "sampler.turntable{bank}.player1.playingConsideringHoldSetting",
            1 => "sampler.turntable{bank}.player2.playingConsideringHoldSetting",
            2 => "sampler.turntable{bank}.player3.playingConsideringHoldSetting",
            3 => "sampler.turntable{bank}.player4.playingConsideringHoldSetting",
            4 => "sampler.turntable{bank}.player5.playingConsideringHoldSetting",
            5 => "sampler.turntable{bank}.player6.playingConsideringHoldSetting",
            6 => "sampler.turntable{bank}.player7.playingConsideringHoldSetting",
            _ => "sampler.turntable{bank}.player8.playingConsideringHoldSetting",
        },
        PadMode::Stems => match index {
            0 => "turntable{deck}.unmixerFourTrackChannel1Muted",
            1 => "turntable{deck}.unmixerFourTrackChannel2Muted",
            2 => "turntable{deck}.unmixerFourTrackChannel3Muted",
            3 => "turntable{deck}.unmixerFourTrackChannel4Muted",
            4 => "turntable{deck}.unmixerFourTrackChannel1Solo",
            5 => "turntable{deck}.unmixerFourTrackChannel2Solo",
            6 => "turntable{deck}.unmixerFourTrackChannel3Solo",
            _ => "turntable{deck}.unmixerFourTrackChannel4Solo",
        },
    };
    let shifted = match mode {
        PadMode::Hotcue => match index {
            0 => Some("turntable{deck}.clearCuePoint1"),
            1 => Some("turntable{deck}.clearCuePoint2"),
            2 => Some("turntable{deck}.clearCuePoint3"),
            3 => Some("turntable{deck}.clearCuePoint4"),
            4 => Some("turntable{deck}.clearCuePoint5"),
            5 => Some("turntable{deck}.clearCuePoint6"),
            6 => Some("turntable{deck}.clearCuePoint7"),
            _ => Some("turntable{deck}.clearCuePoint8"),
        },
        PadMode::Samples => match index {
            0 => Some("sampler.turntable{bank}.player1.paused"),
            1 => Some("sampler.turntable{bank}.player2.paused"),
            2 => Some("sampler.turntable{bank}.player3.paused"),
            3 => Some("sampler.turntable{bank}.player4.paused"),
            4 => Some("sampler.turntable{bank}.player5.paused"),
            5 => Some("sampler.turntable{bank}.player6.paused"),
            6 => Some("sampler.turntable{bank}.player7.paused"),
            _ => Some("sampler.turntable{bank}.player8.paused"),
        },
        PadMode::Stems => match index {
            0 => Some("turntable{deck}.unmixerFourTrackChannel1SoloExclusive"),
            1 => Some("turntable{deck}.unmixerFourTrackChannel2SoloExclusive"),
            2 => Some("turntable{deck}.unmixerFourTrackChannel3SoloExclusive"),
            3 => Some("turntable{deck}.unmixerFourTrackChannel4SoloExclusive"),
            _ => None,
        },
    };
    Some(Binding {
        input: match mode {
            PadMode::Hotcue => "hotcue_pad",
            PadMode::Samples => "samples_pad",
            PadMode::Stems => "stems_pad",
        },
        number: base + index,
        kind: Kind::Note,
        target,
        shifted,
        pickup: false,
        flipped: false,
        sensitivity: None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlNote {
    pub name: &'static str,
    pub reason: &'static str,
}

/// Physically recognized controls that select hardware-local bridge state.
pub const LOCAL_CONTROLS: &[ControlNote] = &[
    ControlNote {
        name: "shift",
        reason: "selects shifted binding layer",
    },
    ControlNote {
        name: "deck_select_a_c",
        reason: "selects the active physical deck",
    },
    ControlNote {
        name: "deck_select_b_d",
        reason: "selects the active physical deck",
    },
    ControlNote {
        name: "pad_mode_hotcue",
        reason: "selects the pad binding layer",
    },
    ControlNote {
        name: "pad_mode_samples",
        reason: "selects the pad binding layer",
    },
    ControlNote {
        name: "pad_mode_stems",
        reason: "selects the pad binding layer",
    },
    ControlNote {
        name: "mixer_chN_fx1",
        reason: "selects the FX strip target",
    },
    ControlNote {
        name: "mixer_chN_fx2",
        reason: "selects the FX strip target",
    },
    ControlNote {
        name: "mixer_chN_fx_select",
        reason: "selects the quick-FX deck",
    },
    ControlNote {
        name: "jog",
        reason: "routes jog motion to the selected deck",
    },
    ControlNote {
        name: "crossfader_assign",
        reason: "emits a selector pulse for crossfader assignment",
    },
    ControlNote {
        name: "master_volume",
        reason: "hardware-local output control; avoid double attenuation",
    },
    ControlNote {
        name: "booth_volume",
        reason: "hardware-local output control; avoid double attenuation",
    },
    ControlNote {
        name: "headphone_mix",
        reason: "hardware-local output control; avoid double attenuation",
    },
    ControlNote {
        name: "headphone_gain",
        reason: "hardware-local headphone level; decoded from payload bytes26-27",
    },
];

/// Controls with no established equivalent native Djay mapping action.
pub const UNSUPPORTED_CONTROLS: &[ControlNote] = &[ControlNote {
    name: "EXT",
    reason: "no verified live-input selector action",
}];
