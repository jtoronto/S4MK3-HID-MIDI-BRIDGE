use super::*;

#[test]
fn tempo_warning_needs_motion_and_clears_on_return_or_pickup() -> anyhow::Result<()> {
    let mut translator = Translator {
        decks: [Deck::C, Deck::B],
        tempos: [Some(0.5), None],
        ..Translator::default()
    };
    let mut feedback = FeedbackState::default();
    let visible =
        |translator: &Translator, feedback: &FeedbackState| match pages(translator, feedback)[0] {
            Page::Info(info) => info.tempo_pickup,
            Page::Empty | Page::Stems { .. } => None,
        };
    feed(&mut feedback, &[0x96, 2, 127, 0xb2, 113, 127]);
    translator.update_tempo_pickup(&feedback, Duration::ZERO);
    translator.update_tempo_pickup(&feedback, Duration::from_millis(75));
    assert_eq!(visible(&translator, &feedback), None);
    for (tick, (physical, expected)) in [
        (0.75, TempoPickup::new(95, 127)),
        (0.5, None),
        (0.75, TempoPickup::new(95, 127)),
    ]
    .into_iter()
    .enumerate()
    {
        translator.translate(
            &[Control::Slider("left_tempo_fader", physical)],
            None,
            &mut Vec::new(),
        )?;
        translator.update_tempo_pickup(&feedback, Duration::from_millis(80 + u64::try_from(tick)?));
        assert_eq!(visible(&translator, &feedback), expected);
    }
    feed(&mut feedback, &[0xb2, 113, 95]);
    translator.update_tempo_pickup(&feedback, Duration::from_millis(85));
    assert_eq!(visible(&translator, &feedback), None);
    feed(
        &mut feedback,
        &[0x96, 0, 127, 0xb2, 113, 127, 0xb0, 113, 64],
    );
    translator.update_tempo_pickup(&feedback, Duration::from_millis(90));
    translator.translate(
        &[Control::Button("left_deck_switch_a", true)],
        None,
        &mut Vec::new(),
    )?;
    assert_eq!(translator.decks[0], Deck::A);
    translator.update_tempo_pickup(&feedback, Duration::from_millis(165));
    assert_eq!(visible(&translator, &feedback), None);
    Ok(())
}

#[test]
fn numeric_packets_route_to_each_selected_deck_and_global_range() -> anyhow::Result<()> {
    for (side, deck) in [(0, Deck::A), (1, Deck::B), (0, Deck::C), (1, Deck::D)] {
        // Given either physical side on a chosen logical deck.
        let mut translator = Translator::default();
        translator.decks[side] = deck;
        let mut feedback = FeedbackState::default();
        let before = pages(&translator, &feedback);
        // When its real wire feedback arrives.
        let cc = 0xb0 | deck.channel();
        feed(&mut feedback, &[cc, 86, 127, 98, 127, 112, 81]);
        feed(&mut feedback, &[0x90 | deck.channel(), 6, 127]);
        // Then values follow that deck, not the selected-deck alias or the other side.
        let Page::Info(info) = pages(&translator, &feedback)[side] else {
            anyhow::bail!("expected information page");
        };
        assert_eq!(
            (info.jump, info.loop_size, info.key),
            (Some("8"), Some("8"), Some("9A"))
        );
        assert_eq!(info.loop_active, Some(true));
        assert_eq!(pages(&translator, &feedback)[side ^ 1], before[side ^ 1]);
        feed(&mut feedback, &[0xb6, 8, 95]);
        for page in pages(&translator, &feedback) {
            let Page::Info(info) = page else {
                anyhow::bail!("expected information page");
            };
            assert_eq!(info.tempo_range, Some(75));
        }
    }
    Ok(())
}

#[test]
fn move_selection_is_per_deck_and_stems_does_not_flash() -> anyhow::Result<()> {
    // Given MOVE size selection latched by real translator presses on A.
    let mut translator = Translator::default();
    translator.translate(
        &[Control::Button("left_move_encoder_press", true)],
        None,
        &mut Vec::new(),
    )?;
    translator.translate(
        &[Control::Button("left_move_encoder_press", false)],
        None,
        &mut Vec::new(),
    )?;
    let feedback = FeedbackState::default();
    assert!(pages(&translator, &feedback)[0].selecting());
    // When the physical side changes decks, then returns to A in STEMS.
    translator.translate(
        &[Control::Button("left_deck_switch_c", true)],
        None,
        &mut Vec::new(),
    )?;
    assert!(!pages(&translator, &feedback)[0].selecting());
    translator.translate(
        &[
            Control::Button("left_deck_switch_a", true),
            Control::Button("left_stems_mode", true),
        ],
        None,
        &mut Vec::new(),
    )?;
    // Then the local latch survives but the separate STEMS page stays steady.
    assert!(translator.move_selecting[0]);
    assert!(!pages(&translator, &feedback)[0].selecting());
    Ok(())
}

#[test]
fn key_feedback_before_loading_false_survives_next_loaded_state() -> anyhow::Result<()> {
    // Given Deck 2's previously loaded 2A track, as captured at startup.
    let translator = Translator::default();
    let mut feedback = FeedbackState::default();
    feed(&mut feedback, &[0x96, 1, 127, 0xb1, 112, 71]);
    let left = pages(&translator, &feedback)[0];

    // When Djay reports the new 11A key before loading-false.
    // Exact wire order captured while loading William Kiss - Jacker.
    feed(&mut feedback, &[0xb1, 112, 102]);
    feed(&mut feedback, &[0x96, 1, 0]);
    feed(&mut feedback, &[0x96, 1, 127]);

    // Then the screen uses the new key, not unknown or the previous track's key.
    let Page::Info(info) = pages(&translator, &feedback)[1] else {
        anyhow::bail!("expected info");
    };
    assert_eq!(feedback.loaded[1], Some(true));
    assert_eq!(info.key, Some("11A"));
    assert_eq!(pages(&translator, &feedback)[0], left);
    Ok(())
}

#[test]
fn eject_key_feedback_clears_the_screen_independently_of_loading_order() -> anyhow::Result<()> {
    for packets in [[0xb1, 112, 0, 0x96, 1, 0], [0x96, 1, 0, 0xb1, 112, 0]] {
        // Given the 11A track from the captured failure.
        let translator = Translator::default();
        let mut feedback = FeedbackState::default();
        feed(&mut feedback, &[0x96, 1, 127, 0xb1, 112, 102]);

        // When eject reports no-key zero before loading-false, or in reverse order.
        feed(&mut feedback, &packets);

        // Then an empty deck shows no key, never the previous track's 11A.
        let Page::Info(info) = pages(&translator, &feedback)[1] else {
            anyhow::bail!("expected info");
        };
        assert_eq!(feedback.loaded[1], Some(false));
        assert_eq!(info.key, Some("--"));
    }
    Ok(())
}

#[test]
fn calibrated_key_bytes_reach_the_page_and_unknown_values_do_not_guess() -> anyhow::Result<()> {
    // Given the complete label/wire pairs recorded by the paused-track calibration.
    let calibration = [
        (5, "8B"),
        (10, "8A"),
        (15, "3B"),
        (20, "3A"),
        (25, "10B"),
        (30, "10A"),
        (36, "5B"),
        (41, "5A"),
        (46, "12B"),
        (51, "12A"),
        (56, "7B"),
        (61, "7A"),
        (66, "2B"),
        (71, "2A"),
        (76, "9B"),
        (81, "9A"),
        (86, "4B"),
        (91, "4A"),
        (97, "11B"),
        (102, "11A"),
        (107, "6B"),
        (112, "6A"),
        (117, "1B"),
        (122, "1A"),
        (0, "--"),
    ];
    let translator = Translator::default();
    let mut feedback = FeedbackState::default();
    for (value, label) in calibration {
        // When the observed byte enters through the actual decoder/cache.
        feed(&mut feedback, &[0xb0, 112, value]);
        // Then the screen model carries its observed label.
        let Page::Info(info) = pages(&translator, &feedback)[0] else {
            anyhow::bail!("expected info");
        };
        assert_eq!(info.key, Some(label));
    }
    feed(&mut feedback, &[0xb0, 112, 127]);
    let Page::Info(info) = pages(&translator, &feedback)[0] else {
        anyhow::bail!("expected info");
    };
    assert_eq!(info.key, None);
    Ok(())
}
