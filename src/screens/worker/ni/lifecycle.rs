//! Request generations and rendering are independent of helper process I/O.

use super::super::presentation::Presentation;
use super::*;
use crate::screen_motor_diag::{self as diag, Case, Kind, Role};

fn release<T: Transport>(
    transport: &mut Option<T>,
    unconfirmed: &mut bool,
    generation: u64,
) -> bool {
    if let Some(mut session) = transport.take() {
        let begin = diag::record(
            Kind::ScreenReleaseBegin,
            [generation, 0, 0, 0, 0, 0, 0, 0],
            &[],
        );
        let result = session.close();
        diag::record(
            Kind::ScreenReleaseEnd,
            [begin, generation, u64::from(result.is_ok()), 0, 0, 0, 0, 0],
            &[],
        );
        match result {
            Ok(()) => *unconfirmed = false,
            Err(error) => {
                *unconfirmed = true;
                tracing::warn!(%error, "SCREEN_RELEASE_UNCONFIRMED: NI ownership restoration not established");
                return false;
            }
        }
    }
    true
}

fn pending_failure(
    shared: &Shared,
    generation: u64,
    unconfirmed: bool,
) -> anyhow::Result<Option<u64>> {
    if !unconfirmed {
        return Ok(None);
    }
    let mailbox = shared
        .0
        .lock()
        .map_err(|error| anyhow::anyhow!("{error}"))?;
    Ok(mailbox
        .latest
        .filter(|request| request.generation != generation)
        .map(|request| request.generation))
}

fn connect<F, T>(
    open: &mut F,
    selected: (Identity, Cancellation),
    generation: u64,
) -> anyhow::Result<T>
where
    F: FnMut(Identity, Cancellation) -> anyhow::Result<T>,
{
    let begin = diag::record(
        Kind::ScreenConnectBegin,
        [generation, 0, 0, 0, 0, 0, 0, 0],
        &[],
    );
    let result = open(selected.0, selected.1);
    diag::record(
        Kind::ScreenReady,
        [begin, generation, u64::from(result.is_ok()), 0, 0, 0, 0, 0],
        &[],
    );
    result
}

pub(in crate::screens::worker) fn run<F, T, S>(
    shared: &Shared,
    open: F,
    enabled: bool,
    status: &S,
) -> anyhow::Result<()>
where
    F: FnMut(Identity, Cancellation) -> anyhow::Result<T>,
    T: Transport,
    S: Fn(u64, State),
{
    let _guard = diag::enter(Role::Screen);
    run_case(shared, open, enabled, status, diag::case())
}

fn ready(case: Case, generation: u64) {
    if !diag::enabled() {
        return;
    }
    let label = match case {
        Case::Normal => "normal",
        Case::Off => "off",
        Case::Acquired => "acquired",
        Case::Static => "static",
        Case::Updates => "updates",
    };
    diag::record(
        Kind::ScreenCaseReady,
        [generation, 0, 0, 0, 0, 0, 0, 0],
        label.as_bytes(),
    );
    println!("S4_SCREEN_DIAG_READY {label}");
}

pub(super) fn run_case<F, T, S>(
    shared: &Shared,
    open: F,
    enabled: bool,
    status: &S,
    case: Case,
) -> anyhow::Result<()>
where
    F: FnMut(Identity, Cancellation) -> anyhow::Result<T>,
    T: Transport,
    S: Fn(u64, State),
{
    run_clock(shared, open, enabled, status, case, Instant::now)
}

pub(super) fn run_clock<F, T, S>(
    shared: &Shared,
    mut open: F,
    mut enabled: bool,
    status: &S,
    case: Case,
    mut clock: impl FnMut() -> Instant,
) -> anyhow::Result<()>
where
    F: FnMut(Identity, Cancellation) -> anyhow::Result<T>,
    T: Transport,
    S: Fn(u64, State),
{
    let mut transport: Option<T> = None;
    let mut current = None;
    let mut owner = (0, 0);
    let mut active = false;
    let mut release_unconfirmed = false;
    let mut failed_release = None;
    let mut presentation = Presentation::new(case);
    if case == Case::Off {
        enabled = false;
        emit(shared, status, owner, State::Inactive);
        ready(case, owner.0);
    }
    loop {
        let (request, stopping) = work(
            shared,
            &mut current,
            owner.0,
            transport.is_some(),
            presentation.deadline(),
        )?;
        if stopping {
            release(&mut transport, &mut release_unconfirmed, owner.0);
            return Ok(());
        }
        if let Some(request) = request {
            let released = release(&mut transport, &mut release_unconfirmed, owner.0)
                && failed_release.take() != Some(request.generation);
            if !released {
                failed_release = pending_failure(shared, request.generation, release_unconfirmed)?;
            }
            active = false;
            owner = (request.generation, request.id);
            presentation.reset(owner.0);
            enabled = case != Case::Off
                && matches!(request.action, Action::On | Action::Retry)
                && released;
            let state = if !released || (!enabled && release_unconfirmed) {
                State::Failed
            } else if enabled {
                State::Pending
            } else {
                State::Inactive
            };
            emit(shared, status, owner, state);
            if case == Case::Off && released {
                ready(case, owner.0);
            }
        }
        let Some(pages) = current.filter(|_| enabled) else {
            continue;
        };
        let result = (|| -> anyhow::Result<()> {
            if transport.is_none() {
                let Some((identity, cancellation)) = acquire(shared, owner.0)? else {
                    return Ok(());
                };
                transport = Some(connect(&mut open, (identity, cancellation), owner.0)?);
                if interrupted(shared, owner.0)? {
                    return Ok(());
                }
                if case == Case::Acquired {
                    ready(case, owner.0);
                }
            }
            let transport = transport.as_mut().context("NI screen session absent")?;
            transport.poll_control()?;
            if interrupted(shared, owner.0)? {
                return Ok(());
            }
            if presentation.publish(pages, clock(), |frames| transport.publish(frames))? {
                if interrupted(shared, owner.0)? {
                    return Ok(());
                }
                if !active {
                    active = true;
                    emit(shared, status, owner, State::Active);
                    ready(case, owner.0);
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            diag::record(Kind::Lifecycle, [100, owner.0, 0, 0, 0, 0, 0, 0], &[]);
            release_unconfirmed |= error.is::<ipc::Unconfirmed>();
            release(&mut transport, &mut release_unconfirmed, owner.0);
            failed_release = pending_failure(shared, owner.0, release_unconfirmed)?;
            presentation.reset(owner.0);
            active = false;
            enabled = false;
            tracing::warn!(%error, "SCREENS_DISABLED: isolated screen helper failed; controls remain active");
            emit(shared, status, owner, State::Failed);
        }
    }
}
