use super::*;

fn session_with_backend(profile: &'static str, backend: Backend) -> Session {
    let (sender, receiver) = crossbeam_channel::bounded(1);
    let shared = Arc::new(Mutex::new(Status {
        version: 1,
        state: State::Starting,
        profile,
        backend,
        request_id: None,
        readiness: Readiness {
            hid: false,
            midi_input: (profile == "full").then_some(false),
            midi_output: (profile != "probe").then_some(false),
            controller_initialized: false,
            feedback: if profile == "full" {
                Feedback::Unknown
            } else {
                Feedback::NotApplicable
            },
        },
        error: None,
        started: true,
        stop_requested: false,
    }));
    Session {
        stop: Stop { shared, sender },
        receiver,
        screens: None,
        led_config: None,
    }
}

fn session(profile: &'static str) -> Session {
    session_with_backend(profile, Backend::Loopbe)
}

#[test]
fn stop_before_initialization_prevents_ready_and_keeps_first_request() -> anyhow::Result<()> {
    for profile in ["full", "minimal", "probe"] {
        let session = session(profile);
        session.stop.request(Some(u64::MAX))?;
        session.stop.request(Some(2))?;
        session.ready()?;
        let status = session
            .stop
            .shared
            .lock()
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let wire = serde_json::to_value(&*status)?;
        drop(status);
        assert_eq!(wire["state"], "stopping");
        assert_eq!(wire["request_id"], "18446744073709551615");
        assert_eq!(session.receiver.try_iter().count(), 1);
    }
    Ok(())
}

#[test]
fn readiness_requires_actual_profile_handles_and_initialization() -> anyhow::Result<()> {
    for (profile, backend, backend_name) in
        ["full", "minimal", "probe"]
            .into_iter()
            .flat_map(|profile| {
                [
                    (profile, Backend::Loopbe, "loopbe"),
                    (profile, Backend::Bloop, "bloop"),
                ]
            })
    {
        let session = session_with_backend(profile, backend);
        assert!(session.ready().is_err());
        session.milestone(|ready| ready.hid = true)?;
        if profile != "probe" {
            assert!(session.ready().is_err());
        }
        session.milestone(|ready| {
            if let Some(output) = &mut ready.midi_output {
                *output = true;
            }
            if let Some(input) = &mut ready.midi_input {
                *input = true;
            }
        })?;
        session.ready()?;
        let status = session
            .stop
            .shared
            .lock()
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let wire = serde_json::to_value(&*status)?;
        drop(status);
        assert_eq!(wire["version"], 1);
        assert_eq!(wire["state"], "ready");
        assert_eq!(wire["profile"], profile);
        assert_eq!(wire["backend"], backend_name);
        assert!(wire["request_id"].is_null());
        assert!(wire["error"].is_null());
        assert_eq!(wire["readiness"]["controller_initialized"], true);
        assert_eq!(wire["readiness"]["midi_input"].is_null(), profile != "full");
        assert_eq!(
            wire["readiness"]["midi_output"].is_null(),
            profile == "probe"
        );
    }
    Ok(())
}

#[test]
fn eof_and_duration_stop_have_no_request_id_and_failure_is_terminal() -> anyhow::Result<()> {
    let session = session("full");
    session.stop.request(None)?;
    assert!(
        session
            .finish(Err(anyhow::anyhow!("reader shutdown failed")))
            .is_err()
    );
    session.stop.request(Some(10))?;
    session.ready()?;
    let status = session
        .stop
        .shared
        .lock()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    let wire = serde_json::to_value(&*status)?;
    drop(status);
    assert_eq!(wire["state"], "failed");
    assert!(wire["request_id"].is_null());
    assert_eq!(wire["error"]["stage"], "runtime");
    assert_eq!(wire["error"]["message"], "reader shutdown failed");
    Ok(())
}
