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
        && record["message"][0] == 0xb2
        && record["message"][1] == 5));
    // Raw physical touch zero precedes the corresponding MIDI work.
    let touch: Vec<_> = records
        .iter()
        .filter(|record| record["now_us"] == 200_001)
        .collect();
    assert_eq!(touch[1]["event"], "motor");
    assert_eq!(touch[1]["report"][4], 0);
    assert_eq!(touch[2]["message"], serde_json::json!([0x90, 7, 127]));
    Ok(())
}

#[test]
fn powered_nominal_motion_is_silent_and_rim_deviations_remain_midi() -> anyhow::Result<()> {
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
    // Then playback still runs, untouched powered counts never become jog MIDI,
    // while the measured extra rim movement bends the track.
    assert!(
        writes
            .iter()
            .any(|(at, report)| *at == 16_000_001 && force(report, 0) > 0)
    );
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
    assert!(jogs.iter().any(|record| {
        record["now_us"] == 70_001
            && record["message"][2]
                .as_u64()
                .is_some_and(|value| value > 64)
    }));
    assert!(!jogs.iter().any(|record| {
        record["now_us"]
            .as_u64()
            .is_some_and(|at| (10_001..60_001).contains(&at) || at > 1_000_000)
    }));
    let vinyl_off = records
        .iter()
        .find(|record| record["event"] == "local" && record["now_us"] == 130_001)
        .context("vinyl off state")?;
    assert_eq!(vinyl_off["vinyl"], serde_json::json!([false, true]));
    assert_eq!(vinyl_off["motor_enabled"], serde_json::json!([true, true]));
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
