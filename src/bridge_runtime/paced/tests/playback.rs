use super::*;
use crate::{bridge_runtime::replay, full};

fn transcript(config: full::jog::Config) -> anyhow::Result<Vec<serde_json::Value>> {
    let records = RefCell::new(Vec::new());
    replay::execute(config, replay::fixture()?, |record| {
        records.borrow_mut().push(record);
    })?;
    Ok(records.into_inner())
}

fn motors(records: &[serde_json::Value]) -> anyhow::Result<Vec<(u64, [u8; 11])>> {
    records
        .iter()
        .filter(|record| record["event"] == "motor")
        .map(|record| {
            Ok((
                record["now_us"].as_u64().context("motor timestamp")?,
                serde_json::from_value(record["report"].clone())?,
            ))
        })
        .collect()
}

fn force(report: &[u8; 11], side: usize) -> u16 {
    u16::from_le_bytes([report[4 + side * 5], report[5 + side * 5]])
}

#[test]
fn production_replay_gates_motor_force_and_preserves_original_scratch_owner() -> anyhow::Result<()>
{
    // Given the executable's real raw parser, MIDI cache, regulator and pacer.
    // When the deterministic fixture crosses playback, gestures and recovery.
    let records = transcript(full::jog::DEFAULT)?;
    let writes = motors(&records)?;
    // Then unknown/internal/paused/THRU/unloaded and TT-off gates are real,
    // independent on each wheel, with no force summation.
    assert!(
        writes
            .iter()
            .filter(|(at, _)| *at <= 10_001)
            .all(|(_, report)| *report == neutral_report())
    );
    for (start, end, side) in [
        (90_001, 110_001, 0),
        (500_001, 520_001, 0),
        (800_001, 820_001, 0),
        (860_001, 880_001, 1),
        (940_001, 960_001, 0),
    ] {
        let window: Vec<_> = writes
            .iter()
            .filter(|(at, _)| (start..end).contains(at))
            .collect();
        assert!(!window.is_empty());
        assert!(
            window.iter().all(|(_, report)| force(report, side) == 0),
            "{start}..{end}"
        );
        assert!(window.iter().all(|(_, report)| force(report, 1 - side) > 0));
    }
    assert!(
        writes
            .iter()
            .all(|(_, report)| force(report, 0) <= 20_000 && force(report, 1) <= 20_000)
    );
    assert!(writes.iter().all(|(_, report)| report[8] != 0xfe));
    assert!(
        writes
            .iter()
            .filter(|(at, _)| (608_001..642_001).contains(at))
            .all(|(_, report)| *report == neutral_report())
    );
    assert!(
        writes
            .iter()
            .any(|(at, report)| *at == 650_001 && force(report, 0) > 0 && force(report, 1) > 0)
    );
    // Cached real playback permission survives silence beyond 15 seconds.
    assert!(
        writes
            .iter()
            .filter(|(at, _)| *at > 16_000_000)
            .take(100)
            .all(|(_, report)| force(report, 0) > 0 && force(report, 1) > 0)
    );
    assert_eq!(
        writes.last().map(|(_, report)| *report),
        Some(neutral_report())
    );
    let scratch: Vec<_> = records
        .iter()
        .filter(|record| {
            record["event"] == "midi" && record["message"][0] == 0xb0 && record["message"][1] == 5
        })
        .collect();
    assert!(!scratch.is_empty());
    assert!(
        scratch
            .iter()
            .any(|record| record["now_us"].as_u64().is_some_and(|at| at > 282_001))
    );
    assert!(!records.iter().any(|record| record["event"] == "midi"
        && record["now_us"].as_u64().is_some_and(|at| at < 1_000_000)
        && record["message"][0] == 0xb2
        && record["message"][1] == 5));
    // TT touch enters the original deck's Scratch route without cutting drive.
    let touch: Vec<_> = records
        .iter()
        .filter(|record| record["now_us"] == 200_001)
        .collect();
    assert!(
        writes
            .iter()
            .filter(|(at, _)| (200_001..220_001).contains(at))
            .all(|(_, report)| force(report, 0) == 5280)
    );
    let scratch_index = touch
        .iter()
        .position(|record| {
            record["event"] == "midi" && record["message"][0] == 0xb0 && record["message"][1] == 5
        })
        .context("touch scratch movement")?;
    assert!(scratch_index > 0);
    Ok(())
}

#[test]
fn powered_platter_uses_relative_jog_bend_and_continues_assist_on_touch() -> anyhow::Result<()> {
    // Given motor integration with all optional tension/backspin features off.
    let config = full::jog::Config {
        backspin_enabled: false,
        backspin_assist_enabled: false,
        backspin_assist_strength: 0,
        ..full::jog::DEFAULT
    };
    // When the same executable fixture supplies raw nominal and rim movement.
    let records = transcript(config)?;
    let writes = motors(&records)?;
    // Then motorized motion uses relative Pitch Bend CC, and touch owns the wheel.
    assert!(
        writes
            .iter()
            .any(|(at, report)| *at == 16_000_001 && force(report, 0) > 0)
    );
    assert!(!records.iter().any(|record| {
        record["event"] == "midi"
            && record["message"][0]
                .as_u64()
                .is_some_and(|status| (0xb0..=0xb3).contains(&status))
            && record["message"][1] == 8
    }));
    let scratch_mode = records
        .iter()
        .filter(|record| {
            record["event"] == "midi" && record["message"][1] == full::catalog::SCRATCHING_MODE_NOTE
        })
        .collect::<Vec<_>>();
    // Touch or the deliberate speed-floor gesture may hold Scratch, never nominal rotation.
    assert!(scratch_mode.iter().all(|record| {
        record["now_us"].as_u64().is_some_and(|at| {
            (200_001..=282_001).contains(&at)
                || (3_000_000..4_600_000).contains(&at)
                || (6_000_000..7_700_000).contains(&at)
        })
    }));
    assert!(scratch_mode.iter().any(|record| {
        record["now_us"] == 200_001 && record["message"] == serde_json::json!([0x90, 7, 127])
    }));
    let jogs: Vec<_> = records
        .iter()
        .filter(|record| {
            record["event"] == "midi"
                && record["message"][0]
                    .as_u64()
                    .is_some_and(|status| (0xb0..=0xb3).contains(&status))
                && record["message"][1] == 4
        })
        .collect();
    assert!(
        jogs.iter().all(
            |record| record["now_us"].as_u64().is_some_and(|at| at < 1_000_000
                || (3_000_000..4_600_000).contains(&at)
                || (6_000_000..7_700_000).contains(&at))
        ),
        "nominal powered motion emitted relative jog-bend CC: {jogs:?}"
    );
    assert!(jogs.iter().all(|record| {
        record["message"][2]
            .as_u64()
            .is_some_and(|value| (1..=127).contains(&value) && value != 64)
    }));
    let vinyl_off = records
        .iter()
        .find(|record| record["event"] == "local" && record["now_us"] == 130_001)
        .context("vinyl off state")?;
    assert_eq!(vinyl_off["vinyl"], serde_json::json!([false, false]));
    assert_eq!(vinyl_off["motor_enabled"], serde_json::json!([true, true]));
    Ok(())
}

#[test]
fn powered_speed_error_emits_only_the_corresponding_relative_bend() -> anyhow::Result<()> {
    // Given the production parser/translator with a quantized steady sensor speed.
    for (rpm_tenths, direction) in [
        (327_u64, 0_i64),
        (330, 0),
        (316, 0),
        (320, 0),
        (340, 0),
        (300, -1),
        (360, 1),
    ] {
        let mut fixture = replay::fixture()?;
        let mut anchor = None;
        for step in &mut fixture {
            if step.now_us < 2_000_000 {
                continue;
            }
            if let Some(frame) = &mut step.frame {
                let (start, counters) = *anchor.get_or_insert((
                    step.now_us,
                    [
                        u32::from_le_bytes(frame.buffers[1][12..16].try_into()?),
                        u32::from_le_bytes(frame.buffers[1][40..44].try_into()?),
                    ],
                ));
                let travel =
                    u32::try_from((step.now_us - start) * rpm_tenths * 2880 / 600_000_000)?;
                for (side, offset) in [(0, 12), (1, 40)] {
                    let counter = counters[side].wrapping_add(travel);
                    frame.buffers[1][offset..offset + 4].copy_from_slice(&counter.to_le_bytes());
                }
            }
        }
        // When the complete production replay processes that speed.
        let bends = RefCell::new(Vec::new());
        replay::execute(full::jog::DEFAULT, fixture, |record| {
            if record["event"] == "midi"
                && record["now_us"].as_u64().is_some_and(|at| at > 3_000_000)
                && record["message"][0]
                    .as_u64()
                    .is_some_and(|status| (0xb0..=0xb3).contains(&status))
                && record["message"][1] == 4
            {
                bends
                    .borrow_mut()
                    .push((record["now_us"].as_u64(), record["message"][2].as_i64()));
            }
        })?;
        // Then nominal and the captured steady baseline stay silent, while
        // sustained slower/faster travel produces only matching signed bends.
        let bends = bends
            .into_inner()
            .into_iter()
            .map(|(at, value)| {
                Ok((
                    at.context("bend timestamp")?,
                    value.context("bend value")? - 64,
                ))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        if direction == 0 {
            assert!(bends.is_empty(), "{rpm_tenths}: {bends:?}");
        } else {
            assert!(!bends.is_empty(), "{rpm_tenths}");
            assert!(
                bends
                    .iter()
                    .all(|(_, bend)| bend.signum() == direction && bend.abs() == 1)
            );
            assert!(bends.windows(2).all(|pair| pair[1].0 - pair[0].0 <= 10_000));
        }
    }
    Ok(())
}

#[test]
fn duplicate_sensor_snapshots_cannot_refresh_motor_freshness() -> anyhow::Result<()> {
    // Given real playback force and a frozen left sensor in otherwise new reports.
    let mut engine = Engine::new(config(0)?);
    engine.configure_playback(full::jog::DEFAULT.platter_settings());
    assert!(
        engine
            .set_playback(s4mk3_hid_midi_bridge::tension::PlaybackState {
                enabled: [true; 2],
                owned: [false; 2],
                continuous: [false; 2],
            })
            .is_none()
    );
    for index in 0_u32..=15 {
        let mut sample = frame(u64::from(index) * 2000 + 1, [index * 3; 2], 0);
        for offset in [8, 36] {
            sample.buffers[1][offset..offset + 4]
                .copy_from_slice(&(index * 200_000 + 100).to_le_bytes());
        }
        engine.observe(
            sample.report(0),
            sample.report(1),
            sample.host_us,
            sample.host_us,
        )?;
        engine.tick(sample.host_us)?;
    }
    let frozen = frame(30_001, [45; 2], 0);
    // When fresh host reports repeat the same sensor timestamp and counter.
    for index in 1_u32..=20 {
        let at = 30_001 + u64::from(index) * 2000;
        let mut sample = frozen.clone();
        sample.buffers[1][8..12].copy_from_slice(&3_000_100_u32.to_le_bytes());
        sample.buffers[1][36..40].copy_from_slice(&(3_000_100 + index * 200_000).to_le_bytes());
        sample.buffers[1][40..44].copy_from_slice(&(45 + index * 3).to_le_bytes());
        engine.observe(sample.report(0), sample.report(1), at, at)?;
    }
    // Then the frozen wheel's original 50ms deadline is unchanged.
    assert_eq!(engine.input_deadline_us(), 80_001);
    assert!(engine.tick(80_002).is_err());
    Ok(())
}

#[test]
fn sensor_clock_gap_clears_force_and_requires_a_new_measurement_window() -> anyhow::Result<()> {
    // Given both playback regulators already producing forward force.
    let mut engine = Engine::new(config(0)?);
    engine.configure_playback(full::jog::DEFAULT.platter_settings());
    assert!(
        engine
            .set_playback(s4mk3_hid_midi_bridge::tension::PlaybackState {
                enabled: [true; 2],
                owned: [false; 2],
                continuous: [false; 2],
            })
            .is_none()
    );
    let mut outputs = Vec::new();
    // When sensor time skips over 50ms despite fresh host snapshots.
    for index in 0_u32..=30 {
        let at = u64::from(index) * 2000 + 1;
        let mut sample = frame(at, [index * 3; 2], 0);
        let timer = index * 200_000 + if index >= 16 { 6_000_000 } else { 0 };
        for offset in [8, 36] {
            sample.buffers[1][offset..offset + 4].copy_from_slice(&timer.to_le_bytes());
        }
        let immediate = engine.observe(sample.report(0), sample.report(1), at, at)?;
        outputs.push((index, immediate, engine.tick(at)?));
    }
    // Then the discontinuity clears force immediately, automatic recovery uses
    // a new baseline, and no old regulator integral can escape before 20ms.
    assert!(force(&outputs[15].2, 0) > 0);
    assert_eq!(outputs[16].1, Some(neutral_report()));
    assert!(
        outputs[16..26]
            .iter()
            .all(|(_, _, report)| *report == neutral_report())
    );
    assert!(force(&outputs[26].2, 0) > 0);
    Ok(())
}

#[test]
fn powered_history_cannot_become_immediate_tension_when_touch_arrives() -> anyhow::Result<()> {
    // Given strong braking tension beneath an untouched playback owner.
    let mut engine = Engine::new(config(100)?);
    engine.configure_playback(full::jog::DEFAULT.platter_settings());
    assert!(
        engine
            .set_playback(s4mk3_hid_midi_bridge::tension::PlaybackState {
                enabled: [true; 2],
                owned: [false; 2],
                continuous: [false; 2],
            })
            .is_none()
    );
    for index in 0_u32..16 {
        let at = u64::from(index) * 2000 + 1;
        let mut sample = frame(at, [index * 3; 2], 0);
        for offset in [8, 36] {
            sample.buffers[1][offset..offset + 4].copy_from_slice(&(index * 200_000).to_le_bytes());
        }
        engine.observe(sample.report(0), sample.report(1), at, at)?;
        engine.tick(at)?;
    }
    let mut touched = frame(32_001, [48; 2], 0x10);
    for offset in [8, 36] {
        touched.buffers[1][offset..offset + 4].copy_from_slice(&3_200_000_u32.to_le_bytes());
    }
    // When physical touch suspends the powered wheel.
    let immediate = engine
        .observe(touched.report(0), touched.report(1), 32_001, 32_001)?
        .context("touch neutral")?;
    // Then neutral precedes ownership work and tension needs new manual motion.
    assert_eq!(force(&immediate, 0), 0);
    assert!(force(&immediate, 1) > 0);
    assert_eq!(force(&engine.tick(32_001)?, 0), 0);
    Ok(())
}

#[test]
fn touch_across_sensor_rollover_neutralizes_without_exiting() -> anyhow::Result<()> {
    // Given active playback near a 32-bit sensor rollover, with 20ms snapshots.
    let mut engine = Engine::new(config(0)?);
    engine.configure_playback(full::jog::DEFAULT.platter_settings());
    assert!(
        engine
            .set_playback(PlaybackState {
                enabled: [true; 2],
                owned: [false; 2],
                continuous: [false; 2],
            })
            .is_none()
    );
    let origin = u32::MAX - 5_000_000;
    for index in 0_u32..=2 {
        let at = u64::from(index) * 20_000 + 1;
        let mut sample = frame(at, [index * 32; 2], 0);
        for offset in [8, 36] {
            sample.buffers[1][offset..offset + 4]
                .copy_from_slice(&origin.wrapping_add(index * 2_000_000).to_le_bytes());
        }
        engine.observe(sample.report(0), sample.report(1), at, at)?;
        assert!(index == 0 || force(&engine.tick(at)?, 0) > 0);
    }
    // When top touch arrives on the first post-rollover snapshot.
    let mut touched = frame(60_001, [96; 2], 0x10);
    for offset in [8, 36] {
        touched.buffers[1][offset..offset + 4]
            .copy_from_slice(&origin.wrapping_add(6_000_000).to_le_bytes());
    }
    let immediate = engine
        .observe(touched.report(0), touched.report(1), 60_001, 60_001)?
        .context("touch neutral")?;
    // Then touch clears only its wheel and the runtime can keep ticking.
    assert_eq!(force(&immediate, 0), 0);
    assert!(force(&engine.tick(60_001)?, 1) > 0);
    Ok(())
}
