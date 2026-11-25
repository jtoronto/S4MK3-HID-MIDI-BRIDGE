use super::*;
use crate::full::{Control, Deck, feedback::Decoder};
use std::time::Duration;

fn feed(feedback: &mut FeedbackState, bytes: &[u8]) {
    Decoder::default().feed(bytes, |message| {
        feedback.apply(message, Duration::ZERO);
    });
}

fn deck_c_fx_three_routes(
    translator: &Translator,
    feedback: &FeedbackState,
) -> anyhow::Result<[Option<bool>; 5]> {
    match pages(translator, feedback)[0] {
        Page::StemFx {
            deck: 2,
            slot: 2,
            layout: Some(2),
            routes,
            ..
        } => Ok(routes),
        other => anyhow::bail!("expected deck C / FX 3 / two stems, got {other:?}"),
    }
}

#[test]
fn stem_fx_second_press_routes_selected_deck_and_waits_for_djay() -> anyhow::Result<()> {
    use crate::full::leds::{Color, LedConfig, render_buttons};

    let mut translator = Translator::default();
    let mut feedback = FeedbackState::default();
    let mut output = Vec::new();
    let stems = "left_stems_mode";
    translator.translate(&[Control::Button(stems, true)], None, &mut output)?;
    assert!(matches!(
        pages(&translator, &feedback)[0],
        Page::Stems { .. }
    ));
    translator.translate(
        &[Control::Button(stems, false), Control::Button(stems, true)],
        None,
        &mut output,
    )?;
    translator.translate(
        &[
            Control::Button(stems, true),
            Control::Button("left_pad_2", true),
        ],
        None,
        &mut output,
    )?;
    assert!(output.is_empty(), "unknown layout must not send a route");
    assert!(matches!(
        pages(&translator, &feedback)[0],
        Page::StemFx { layout: None, .. }
    ));
    assert!(matches!(pages(&translator, &feedback)[1], Page::Info(_)));

    // Deck C has two stems; switch to FX 3 and request its second stem.
    translator.translate(
        &[
            Control::Button("left_pad_2", false),
            Control::Button("left_deck_switch_c", true),
        ],
        None,
        &mut output,
    )?;
    feed(
        &mut feedback,
        &[0xbe, 47, 127, 48, 127, 49, 0, 50, 0, 42, 127, 44, 0],
    );
    translator.sync_front_panel(&feedback, &mut output)?;
    output.clear();
    translator.translate(
        &[
            Control::Button("left_pad_7", true),
            Control::Button("left_pad_3", true),
            Control::Button("left_pad_2", true),
        ],
        None,
        &mut output,
    )?;
    assert_eq!(output, [[0x9e, 44, 127], [0x8e, 44, 0]]);
    assert_eq!(
        deck_c_fx_three_routes(&translator, &feedback)?[2],
        Some(false),
        "a command is not confirmation"
    );
    // Interleaved feedback for another deck is not applied to deck C.
    feed(
        &mut feedback,
        &[0xbc, 42, 127, 44, 0, 0xbe, 42, 0, 43, 127, 44, 127],
    );
    assert_eq!(
        deck_c_fx_three_routes(&translator, &feedback)?,
        [Some(false), Some(true), Some(true), None, None]
    );
    let config = LedConfig::default();
    let leds = render_buttons(&config, &feedback, &translator.local_led_state());
    assert_eq!(
        leds[11],
        Color::Purple.base() + config.palette_active_intensity
    );
    assert_eq!([leds[2], leds[3]], [leds[11], 0]);

    output.clear();
    translator.translate(
        &[
            Control::Button(stems, false),
            Control::Button(stems, true),
            Control::Button("left_pad_2", false),
            Control::Button("left_pad_2", true),
        ],
        None,
        &mut output,
    )?;
    assert!(matches!(
        pages(&translator, &feedback)[0],
        Page::Stems { .. }
    ));
    assert_eq!(output, [[0x92, 57, 127]]);
    Ok(())
}

#[test]
fn fx_warnings_follow_target_decks_on_info_and_stems_without_cached_value_replay()
-> anyhow::Result<()> {
    // Given both physical strips seeded without any MIDI output.
    let mut translator = Translator::default();
    translator.observe_analog(&[
        Control::Slider("fx1_knob_1", 0.0),
        Control::Slider("fx2_dry_wet", 1.0),
    ])?;
    let mut feedback = FeedbackState::default();
    feed(&mut feedback, &[0xb2, 32, 127, 35, 0, 36, 0, 37, 0]);
    // When both strips target hidden C, then no stale values are replayed.
    let mut output = Vec::new();
    translator.translate(
        &[
            Control::Button("mixer_ch3_fx1", true),
            Control::Button("mixer_ch3_fx2", true),
        ],
        None,
        &mut output,
    )?;
    assert!(output.is_empty());
    let Page::Info(info) = pages(&translator, &feedback)[0] else {
        anyhow::bail!("expected info");
    };
    assert_eq!(info.fx.mismatch, [0; 2]);
    translator.decks[0] = Deck::C;
    for mode in [PadMode::Hotcue, PadMode::Stems] {
        translator.pads[0] = mode;
        let warning = match pages(&translator, &feedback)[0] {
            Page::Info(info) => info.fx,
            Page::Stems { fx, .. } => fx,
            Page::Empty | Page::StemFx { .. } => anyhow::bail!("expected deck page"),
        };
        // Then the visible deck owns both strips' parameter and dry/wet warnings.
        assert_eq!(warning.deck, 2);
        assert_eq!(warning.mismatch, [1, 8]);
        assert_eq!(warning.unknown, [0; 2]);
    }
    Ok(())
}

#[test]
fn acquired_fx_turn_does_not_warn_while_software_feedback_catches_up() -> anyhow::Result<()> {
    // Given a physical knob and its target deck start at the same confirmed value.
    let mut translator = Translator::default();
    translator.observe_analog(&[
        Control::Slider("fx1_knob_1", 64.0 / 127.0),
        Control::Slider("fx1_knob_2", 64.0 / 127.0),
        Control::Slider("fx1_knob_3", 64.0 / 127.0),
        Control::Slider("fx1_dry_wet", 64.0 / 127.0),
    ])?;
    let mut feedback = FeedbackState::default();
    let Page::Info(info) = pages(&translator, &feedback)[0] else {
        anyhow::bail!("expected info");
    };
    assert_eq!(info.fx.unknown, [15, 0]);
    feed(
        &mut feedback,
        &[0xb0, 32, 64, 33, 64, 34, 64, 35, 64, 36, 64, 37, 64],
    );
    translator.sync_front_panel(&feedback, &mut Vec::new())?;

    // When the acquired knob turns, Djay has not yet returned its own new value.
    let mut output = Vec::new();
    translator.translate(
        &[Control::Slider("fx1_knob_1", 80.0 / 127.0)],
        None,
        &mut output,
    )?;

    // Then the emitted control is not presented as a pickup warning during feedback lag.
    assert_eq!(output, [[0xb0, 32, 80]]);
    let Page::Info(info) = pages(&translator, &feedback)[0] else {
        anyhow::bail!("expected info");
    };
    assert_eq!(info.fx.mismatch, [0; 2]);
    assert_eq!(info.fx.unknown, [0; 2]);

    // When dry/wet moves, all three independent slots acquire without warning.
    output.clear();
    translator.translate(
        &[Control::Slider("fx1_dry_wet", 80.0 / 127.0)],
        None,
        &mut output,
    )?;
    assert_eq!(output, [[0xb0, 35, 80], [0xb0, 36, 80], [0xb0, 37, 80]]);
    let Page::Info(info) = pages(&translator, &feedback)[0] else {
        anyhow::bail!("expected info");
    };
    assert_eq!(info.fx.mismatch, [0; 2]);

    // When feedback catches up one message at a time, including partial wet echoes.
    for message in [
        [0xb0, 32, 80],
        [0xb0, 35, 80],
        [0xb0, 36, 80],
        [0xb0, 37, 80],
    ] {
        feed(&mut feedback, &message);
        translator.sync_front_panel(&feedback, &mut Vec::new())?;
        let Page::Info(info) = pages(&translator, &feedback)[0] else {
            anyhow::bail!("expected info");
        };
        // Then no intermediate mixture of old/new echoes invents a warning.
        assert_eq!(info.fx.mismatch, [0; 2]);
    }

    // But a mouse edit to one wet slot re-arms only that slot's pickup.
    feed(&mut feedback, &[0xb0, 35, 100]);
    translator.sync_front_panel(&feedback, &mut Vec::new())?;
    let Page::Info(info) = pages(&translator, &feedback)[0] else {
        anyhow::bail!("expected info");
    };
    assert_eq!(info.fx.mismatch, [8, 0]);
    let mut output = Vec::new();
    translator.translate(
        &[Control::Slider("fx1_dry_wet", 85.0 / 127.0)],
        None,
        &mut output,
    )?;
    assert_eq!(output, [[0xb0, 36, 85], [0xb0, 37, 85]]);
    Ok(())
}

fn pads(
    translator: &Translator,
    feedback: &FeedbackState,
    side: usize,
) -> anyhow::Result<[PadState; 8]> {
    match pages(translator, feedback)[side] {
        Page::Stems { pads, .. } => Ok(pads),
        Page::Empty | Page::Info(_) | Page::StemFx { .. } => anyhow::bail!("expected STEMS"),
    }
}

#[test]
fn construction_never_opens_screen_output_before_hid_initialization() {
    // Given either screen setting at bridge construction time.
    for enabled in [false, true] {
        // When the output lifecycle is constructed, before any initialized input.
        let output = Output::new(enabled);
        // Then no worker can yet claim USB or send a frame.
        assert!(output.worker.is_none());
    }
}

#[test]
fn disabled_screen_output_stays_off_when_input_arrives() {
    // Given an ordinary full-profile bridge without the screen opt-in.
    let mut output = Output::new(false);
    // When initialized input publishes its current local state.
    output.publish([Page::Empty; 2]);
    // Then it still does not acquire a display interface.
    assert!(output.worker.is_none());
}

#[test]
fn feedback_before_initialized_input_never_starts_usb() {
    // Given opted-in screen output before the first initialized input.
    let mut output = Output::new(true);
    // When Djay feedback refreshes the current page.
    output.refresh([Page::Empty; 2]);
    // Then screen startup still waits for initialized HID input.
    assert!(output.worker.is_none());
}

#[test]
fn feedback_alone_updates_the_selected_deck_without_changing_the_other_side() -> anyhow::Result<()>
{
    for (side, deck) in [(0, Deck::A), (1, Deck::B), (0, Deck::C), (1, Deck::D)] {
        // Given a selected deck with no physical pad event or software snapshot.
        let mut translator = Translator {
            pads: [PadMode::Stems; 2],
            ..Translator::default()
        };
        translator.decks[side] = deck;
        let mut feedback = FeedbackState::default();
        let before = pages(&translator, &feedback);
        // When real note feedback arrives, including running status and realtime.
        feed(
            &mut feedback,
            &[0x90 | deck.channel(), 56, 127, 60, 0xf8, 0],
        );
        // Then only that deck's page changes, using confirmed mute/solo values.
        let after = pages(&translator, &feedback);
        assert_ne!(after[side], before[side]);
        assert_eq!(after[side ^ 1], before[side ^ 1]);
        let visible = pads(&translator, &feedback, side)?;
        assert_eq!(visible[0].active, Some(true));
        assert_eq!(visible[4].active, Some(false));
        assert!(!visible[0].held);
        assert_eq!(visible[1].active, None);
    }
    Ok(())
}

#[test]
fn a_physical_press_does_not_invent_confirmed_activation() -> anyhow::Result<()> {
    // Given STEMS with no software feedback.
    let mut translator = Translator {
        pads: [PadMode::Stems; 2],
        ..Translator::default()
    };
    let feedback = FeedbackState::default();
    let mut sent = Vec::new();
    // When the real translator receives a physical mute press.
    translator.translate(&[Control::Button("left_pad_1", true)], None, &mut sent)?;
    // Then the press is visible but the mute state remains unknown.
    assert_eq!(sent, [[0x90, 56, 127]]);
    assert_eq!(
        pads(&translator, &feedback, 0)?[0],
        PadState {
            active: None,
            held: true
        }
    );
    Ok(())
}

#[test]
fn a_held_press_is_not_relabelled_as_a_new_deck_or_shift_action() -> anyhow::Result<()> {
    for change in ["left_deck_switch_c", "left_shift"] {
        // Given a mute press owned by deck A's normal channel.
        let mut translator = Translator {
            pads: [PadMode::Stems; 2],
            ..Translator::default()
        };
        translator.translate(
            &[Control::Button("left_pad_1", true)],
            None,
            &mut Vec::new(),
        )?;
        // When the visible deck or action layer changes while it is held.
        translator.translate(&[Control::Button(change, true)], None, &mut Vec::new())?;
        // Then the new visible action is not marked held; the old note keeps ownership.
        assert!(!pads(&translator, &FeedbackState::default(), 0)?[0].held);
        assert_eq!(
            translator.held["left_pad_1"],
            [Address {
                channel: 0,
                number: 56
            }]
        );
    }
    Ok(())
}

#[test]
fn pad_release_removes_the_outline_without_clearing_software_state() -> anyhow::Result<()> {
    // Given a held solo pad with confirmed active feedback.
    let mut translator = Translator {
        pads: [PadMode::Stems; 2],
        ..Translator::default()
    };
    let mut feedback = FeedbackState::default();
    feed(&mut feedback, &[0x90, 60, 127]);
    translator.translate(
        &[Control::Button("left_pad_5", true)],
        None,
        &mut Vec::new(),
    )?;
    // When the physical pad releases.
    let mut sent = Vec::new();
    translator.translate(&[Control::Button("left_pad_5", false)], None, &mut sent)?;
    // Then the outline clears, but the returned toggle state stays on.
    assert_eq!(sent, [[0x80, 60, 0]]);
    assert_eq!(
        pads(&translator, &feedback, 0)?[4],
        PadState {
            active: Some(true),
            held: false
        }
    );
    Ok(())
}

#[test]
fn exclusive_solo_requires_one_confirmed_solo_and_all_others_confirmed_off() -> anyhow::Result<()> {
    for (values, expected) in [
        ([Some(0), Some(127), Some(0), Some(0)], Some(true)),
        ([Some(127), Some(127), Some(0), Some(0)], Some(false)),
        ([Some(0), Some(127), None, Some(0)], None),
    ] {
        // Given Shift+STEMS and Djay's normal-channel solo states.
        let translator = Translator {
            pads: [PadMode::Stems; 2],
            shifted: [true; 2],
            ..Translator::default()
        };
        let mut feedback = FeedbackState::default();
        for (note, value) in (60_u8..64).zip(values) {
            if let Some(value) = value {
                feed(&mut feedback, &[0x90, note, value]);
            }
        }
        // When the shifted page derives the upper exclusive-solo row.
        let visible = pads(&translator, &feedback, 0)?;
        // Then ONLY never claims exclusivity from just one positive solo packet.
        assert_eq!(visible[1].active, expected);
        assert_eq!(visible[5].active, Some(true));
    }
    Ok(())
}

#[test]
fn hotcue_and_samples_share_info_without_leaking_stem_feedback() {
    for mode in [PadMode::Hotcue, PadMode::Samples] {
        // Given ordinary pad modes with active stem feedback and modifiers.
        let translator = Translator {
            pads: [mode; 2],
            shifted: [true; 2],
            ..Translator::default()
        };
        let mut feedback = FeedbackState::default();
        feed(&mut feedback, &[0x90, 56, 127]);
        // When pages are selected.
        let visible = pages(&translator, &feedback);
        // Then both modes use the same unknown info fields, not a pad legend.
        assert_eq!(
            visible,
            [
                Page::Info(Info::default()),
                Page::Info(Info {
                    fx: FxWarning {
                        deck: 1,
                        ..FxWarning::default()
                    },
                    ..Info::default()
                }),
            ]
        );
    }
}

#[test]
fn backspin_projects_original_owner_across_deck_and_page_switches_and_coexists_with_fx()
-> anyhow::Result<()> {
    for (side, initial_deck, switched_deck, switch_button, touch) in [
        (
            0,
            Deck::A,
            Deck::C,
            "left_deck_switch_c",
            "left_wheel_touch",
        ),
        (
            1,
            Deck::D,
            Deck::B,
            "right_deck_switch_b",
            "right_wheel_touch",
        ),
    ] {
        // Given a physical wheel on its initial deck with an FX mismatch on the switched deck.
        let mut translator = Translator::default();
        translator.decks[side] = initial_deck;
        translator.fx = [switched_deck; 2];
        translator.observe_analog(&[Control::Slider("fx1_knob_1", 0.0)])?;
        let mut feedback = FeedbackState::default();
        feed(
            &mut feedback,
            &[
                0xb0 | switched_deck.channel(),
                32,
                127,
                0x90 | initial_deck.channel(),
                0,
                127,
            ],
        );
        // Playback feedback alone never invents a backspin indicator.
        let Page::Info(before) = pages(&translator, &feedback)[side] else {
            anyhow::bail!("expected info page");
        };
        assert_eq!(before.backspin, None);

        // When a qualified backspin starts on the initial deck (touched and then released).
        super::super::tests::prime_backspin(&mut translator, side)?;
        let Page::Info(touched_info) = pages(&translator, &feedback)[side] else {
            anyhow::bail!("expected info page while touched");
        };
        assert_eq!(touched_info.backspin, Some(initial_deck.channel()));

        // And when released and the physical side switches deck and pad modes.
        translator.translate(
            &[
                Control::Button(touch, false),
                Control::Button(switch_button, true),
            ],
            None,
            &mut Vec::new(),
        )?;
        super::super::tests::jog_snapshot(&mut translator, side, 70, 600)?;
        for mode in [PadMode::Hotcue, PadMode::Samples, PadMode::Stems] {
            translator.pads[side] = mode;
            let visible = pages(&translator, &feedback);
            // Then the active side shows the original backspin owner alongside the switched deck's FX warning,
            // while the other physical screen remains untouched.
            match visible[side] {
                Page::Info(info) => {
                    assert_eq!(info.backspin, Some(initial_deck.channel()));
                    assert_eq!(info.fx.deck, switched_deck.channel());
                    assert_eq!(info.fx.mismatch, [1, 0]);
                }
                Page::Stems { backspin, fx, .. } => {
                    assert_eq!(backspin, Some(initial_deck.channel()));
                    assert_eq!(fx.deck, switched_deck.channel());
                    assert_eq!(fx.mismatch, [1, 0]);
                }
                Page::Empty | Page::StemFx { .. } => anyhow::bail!("expected active page"),
            }
            let Page::Info(other) = visible[side ^ 1] else {
                anyhow::bail!("expected info on other side");
            };
            assert_eq!(other.backspin, None);
        }

        // When motion stops, the indicator clears on both page types.
        for time in [80, 90, 100] {
            super::super::tests::jog_snapshot(&mut translator, side, time, 600)?;
        }
        for mode in [PadMode::Hotcue, PadMode::Stems] {
            translator.pads[side] = mode;
            match pages(&translator, &feedback)[side] {
                Page::Info(info) => assert_eq!(info.backspin, None),
                Page::Stems { backspin, .. } => assert_eq!(backspin, None),
                Page::Empty | Page::StemFx { .. } => anyhow::bail!("expected active page"),
            }
        }
    }
    Ok(())
}

mod info;
