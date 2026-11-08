//! Native local peers only. Run each suite under an external process watchdog:
//! a synchronous peer read must not be mistaken for a bounded async wait.
//! No NI service, device, driver, or hardware is contacted.

use super::*;
use crate::{Interest, Poll, Token};
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::IntoRawHandle;
use std::sync::Weak;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{
    ERROR_IO_INCOMPLETE, ERROR_PIPE_CONNECTED, GetHandleInformation,
};
#[cfg(feature = "s4-cancel-pipe-writes-on-drop")]
use windows_sys::Win32::Foundation::{ERROR_NOT_FOUND, ERROR_OPERATION_ABORTED};
use windows_sys::Win32::System::Pipes::{
    PIPE_READMODE_MESSAGE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_MESSAGE, PIPE_WAIT,
};

const REQUEST_BYTES: usize = 8 * 1024 * 1024;
const REQUEST_BYTE: u8 = 0x5a;
const DEADLINE: Duration = Duration::from_secs(3);
static NEXT_PIPE: AtomicUsize = AtomicUsize::new(0);

// The synchronous peer has no internal read-ahead. Its only reads are the
// explicit read_exact calls in completion-wins / feature-off control tests.
struct Pair {
    client: NamedPipe,
    peer: File,
    poll: Poll,
}

fn pair() -> io::Result<Pair> {
    let name = format!(
        r"\\.\pipe\s4-mio-drop-{}-{}",
        std::process::id(),
        NEXT_PIPE.fetch_add(1, Relaxed)
    );
    let wide: Vec<_> = OsStr::new(&name).encode_wide().chain(Some(0)).collect();
    // SAFETY: wide is NUL-terminated and valid throughout the native call.
    // The returned synchronous handle is transferred to File exactly once.
    let handle = unsafe {
        CreateNamedPipeW(
            wide.as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            4096,
            4096,
            0,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: handle is valid, synchronous, uniquely owned, and not yet wrapped.
    let peer = unsafe { File::from_raw_handle(handle) };
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(FILE_FLAG_OVERLAPPED)
        .security_qos_flags(0x0001_0000)
        .open(&name)?;
    // CreateFile already connected this instance, so this call cannot wait for
    // a future client. ERROR_PIPE_CONNECTED is the documented successful race.
    // SAFETY: peer owns a valid synchronous server handle; no OVERLAPPED is used.
    if unsafe { ConnectNamedPipe(peer.as_raw_handle(), std::ptr::null_mut()) } == 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(i32::try_from(ERROR_PIPE_CONNECTED).unwrap()) {
            return Err(error);
        }
    }
    // SAFETY: file is uniquely owned, opened with FILE_FLAG_OVERLAPPED, and
    // transfers its handle to one Mio owner. The peer remains synchronous.
    let mut client = unsafe { NamedPipe::from_raw_handle(file.into_raw_handle()) };
    let poll = Poll::new()?;
    poll.registry().register(
        &mut client,
        Token(7),
        Interest::READABLE | Interest::WRITABLE,
    )?;
    Ok(Pair { client, peer, poll })
}

fn pending_write(client: &NamedPipe) -> io::Result<()> {
    let request = vec![REQUEST_BYTE; REQUEST_BYTES];
    assert_eq!((&*client).write(&request)?, REQUEST_BYTES);
    {
        let io = client.inner.io.lock().unwrap();
        assert!(
            matches!(&io.write, State::Pending(buf, 0) if buf.len() == REQUEST_BYTES),
            "fixture must submit an OS-pending write, not State::Ok awaiting its callback"
        );
    }
    // SAFETY: this queries the actual write OVERLAPPED just submitted on this
    // live owner. It neither waits nor dequeues a completion.
    let error = unsafe { client.inner.result(client.inner.write.as_ptr().cast_mut()) }
        .expect_err("the non-reading peer must leave the native write incomplete");
    assert_eq!(
        error.raw_os_error(),
        Some(i32::try_from(ERROR_IO_INCOMPLETE).unwrap())
    );
    Ok(())
}

fn ownership(client: &NamedPipe) -> (Weak<Inner>, Arc<CompletionPort>, *const OVERLAPPED) {
    let weak = Arc::downgrade(&client.inner);
    let port = client.inner.io.lock().unwrap().cp.as_ref().unwrap().clone();
    (weak, port, client.inner.write.as_ptr())
}

fn assert_retained_handle_is_live(weak: &Weak<Inner>) {
    let inner = weak.upgrade().expect("completion must retain its owner");
    let mut flags = 0;
    // SAFETY: the upgraded Arc owns this handle; the out-parameter is valid.
    assert_ne!(
        unsafe { GetHandleInformation(inner.handle.raw(), &mut flags) },
        0,
        "a retained operation must retain its native handle"
    );
}

// This is the selector's real named-pipe dispatch path, using packets from a
// real port, not fabricated statuses. The test-owned port permits late dispatch
// after the selector has gone away; production has no such extra drainer.
fn drain_to_release(
    weak: &Weak<Inner>,
    port: &CompletionPort,
    write: *const OVERLAPPED,
) -> io::Result<Option<io::Result<usize>>> {
    let deadline = Instant::now() + DEADLINE;
    let mut write_result = None;
    while weak.strong_count() != 0 {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "owner was not reclaimed"))?;
        let mut statuses = [CompletionStatus::zero(); 16];
        for status in port.get_many(&mut statuses, Some(remaining))? {
            assert!(!status.overlapped().is_null());
            assert_eq!(status.token() % 2, 1, "fixture only associates named pipes");
            if status.overlapped().cast_const() == write {
                assert!(write_result.is_none(), "duplicate write completion");
                let inner = weak.upgrade().expect("packet must own an Arc");
                // SAFETY: this packet is for inner.write on its live handle,
                // and the completion has not yet consumed its retained Arc.
                write_result = Some(unsafe { inner.result(status.overlapped()) });
            }
            // SAFETY: every non-null fixture packet came from Mio's own
            // registered named pipe or schedule_event, each retaining one Arc
            // for its Overlapped callback. No other thread polls this port.
            let callback = unsafe { (*(status.overlapped().cast::<Overlapped>())).callback };
            callback(status.entry(), None);
        }
    }
    assert!(
        weak.upgrade().is_none(),
        "all completion references must be released"
    );
    Ok(write_result)
}

#[cfg(feature = "s4-cancel-pipe-writes-on-drop")]
#[test]
fn pending_write_drop_cancels_and_reclaims_without_peer_reads() -> io::Result<()> {
    let Pair { client, peer, poll } = pair()?;
    pending_write(&client)?;
    let (weak, port, write) = ownership(&client);

    drop(client);

    let error = drain_to_release(&weak, &port, write)?
        .expect("the pending write must complete")
        .expect_err("drop must abort the backpressured write");
    assert_eq!(
        error.raw_os_error(),
        Some(i32::try_from(ERROR_OPERATION_ABORTED).unwrap())
    );
    drop(peer);
    drop(poll);
    Ok(())
}

#[cfg(feature = "s4-cancel-pipe-writes-on-drop")]
#[test]
fn completed_write_waiting_for_callback_wins_drop_cancellation() -> io::Result<()> {
    let Pair {
        client,
        mut peer,
        poll,
    } = pair()?;
    pending_write(&client)?;
    let (weak, port, write) = ownership(&client);
    let mut received = vec![0; REQUEST_BYTES];
    peer.read_exact(&mut received)?;
    assert_eq!(received, vec![REQUEST_BYTE; REQUEST_BYTES]);
    // The handle also has a pending read. A blocking GetOverlappedResult with
    // no operation-specific event can wait on that unrelated operation.
    // Dequeue the actual write packet, withholding only its callback instead.
    let deadline = Instant::now() + DEADLINE;
    let completed = loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "write packet missing"))?;
        let mut statuses = [CompletionStatus::zero(); 16];
        let mut completed = None;
        for status in port.get_many(&mut statuses, Some(remaining))? {
            assert!(!status.overlapped().is_null());
            if status.overlapped().cast_const() == write {
                assert!(completed.is_none(), "duplicate write packet");
                completed = Some(*status);
            } else {
                // SAFETY: the fixture dequeues only this pipe's real Mio
                // packets. Their retained owners stay live through callbacks.
                let callback = unsafe { (*(status.overlapped().cast::<Overlapped>())).callback };
                callback(status.entry(), None);
            }
        }
        if let Some(completed) = completed {
            break completed;
        }
    };
    // SAFETY: the dequeued packet identifies this still-owned write, whose
    // callback has not yet reclaimed its operation storage.
    assert_eq!(
        unsafe { client.inner.result(write.cast_mut()) }?,
        REQUEST_BYTES
    );
    assert!(matches!(
        &client.inner.io.lock().unwrap().write,
        State::Pending(..)
    ));
    // SAFETY: this identifies the completed native write while its owner lives.
    let error = unsafe { cancel(&client.inner.handle, &client.inner.write) }
        .expect_err("completed I/O is no longer cancellable");
    assert_eq!(
        error.raw_os_error(),
        Some(i32::try_from(ERROR_NOT_FOUND).unwrap())
    );

    drop(client);

    write_done(completed.entry(), None);
    assert!(
        drain_to_release(&weak, &port, write)?.is_none(),
        "the completed write must not submit another packet"
    );
    drop(peer);
    drop(poll);
    Ok(())
}

#[cfg(feature = "s4-cancel-pipe-writes-on-drop")]
#[test]
fn cancellation_after_selector_drop_keeps_storage_until_late_dispatch() -> io::Result<()> {
    let Pair { client, peer, poll } = pair()?;
    pending_write(&client)?;
    let (weak, port, write) = ownership(&client);
    // Force the late-completion ordering without relying on kernel timing:
    // selector teardown happens while both native operations are incomplete.
    drop(poll);

    drop(client);

    assert_retained_handle_is_live(&weak);
    let error = drain_to_release(&weak, &port, write)?
        .expect("late write packet")
        .expect_err("late cancellation must abort the native write");
    assert_eq!(
        error.raw_os_error(),
        Some(i32::try_from(ERROR_OPERATION_ABORTED).unwrap())
    );
    drop(peer);
    Ok(())
}

#[cfg(feature = "s4-cancel-pipe-writes-on-drop")]
#[test]
fn pipe_then_selector_drop_preserves_any_late_write_owner() -> io::Result<()> {
    let Pair { client, peer, poll } = pair()?;
    pending_write(&client)?;
    let (weak, port, write) = ownership(&client);

    // The production order is pipe first, then its current-thread reactor.
    drop(client);
    drop(poll);

    let retained = weak.strong_count() != 0;
    if retained {
        assert_retained_handle_is_live(&weak);
    }
    eprintln!("pipe-then-selector drop retained late write owner: {retained}");
    // Do not assert synchronous reclamation: SelectorInner::drop only drains
    // ready packets. The test-owned port reaps anything arriving afterward.
    drain_to_release(&weak, &port, write)?;
    drop(peer);
    Ok(())
}

#[test]
fn pipe_then_selector_drop_read_only_control_reclaims_on_late_dispatch() -> io::Result<()> {
    let Pair { client, peer, poll } = pair()?;
    assert!(matches!(
        &client.inner.io.lock().unwrap().read,
        State::Pending(..)
    ));
    let (weak, port, write) = ownership(&client);

    drop(client);
    drop(poll);

    let retained = weak.strong_count() != 0;
    if retained {
        assert_retained_handle_is_live(&weak);
    }
    eprintln!("pipe-then-selector drop retained late read owner: {retained}");
    assert!(drain_to_release(&weak, &port, write)?.is_none());
    drop(peer);
    Ok(())
}

#[cfg(not(feature = "s4-cancel-pipe-writes-on-drop"))]
#[test]
fn feature_off_drop_preserves_pending_write_for_peer_to_drain() -> io::Result<()> {
    let Pair {
        client,
        mut peer,
        poll,
    } = pair()?;
    pending_write(&client)?;
    let (weak, port, write) = ownership(&client);

    drop(client);

    let mut received = vec![0; REQUEST_BYTES];
    peer.read_exact(&mut received)?;
    assert_eq!(received, vec![REQUEST_BYTE; REQUEST_BYTES]);
    assert_eq!(
        drain_to_release(&weak, &port, write)?.expect("graceful write must complete")?,
        REQUEST_BYTES
    );
    drop(peer);
    drop(poll);
    Ok(())
}
