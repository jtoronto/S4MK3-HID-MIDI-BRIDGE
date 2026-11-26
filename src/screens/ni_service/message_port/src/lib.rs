//! Owned macOS Core Foundation message ports. Native calls do not escape this crate.

#![cfg(target_os = "macos")]

use std::ffi::{c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::{NonNull, null};
use std::sync::Arc;
use std::time::Duration;

use core_foundation_sys::base::CFRelease;
use core_foundation_sys::data::{CFDataCreate, CFDataGetBytePtr, CFDataGetLength, CFDataRef};
use core_foundation_sys::messageport::{
    __CFMessagePort, CFMessagePortContext, CFMessagePortCreateLocal, CFMessagePortCreateRemote,
    CFMessagePortInvalidate, CFMessagePortIsValid, CFMessagePortRef, CFMessagePortSendRequest,
};
use core_foundation_sys::string::{__CFString, CFStringCreateWithBytes, kCFStringEncodingUTF8};

unsafe extern "C" {
    fn dispatch_queue_create(label: *const c_char, attributes: *const c_void) -> *mut c_void;
    fn dispatch_release(object: *mut c_void);
    fn CFMessagePortSetDispatchQueue(port: CFMessagePortRef, queue: *mut c_void);
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    Unavailable,
    NameInUse,
    Allocation,
    Payload,
    Status(i32),
}

// A +1 reference returned by a Create/Copy function or out parameter.
struct Owned<T>(NonNull<T>);

impl<T> Owned<T> {
    fn new(pointer: *const T) -> Option<Self> {
        NonNull::new(pointer.cast_mut()).map(Self)
    }

    fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T> Drop for Owned<T> {
    fn drop(&mut self) {
        // SAFETY: Stored references are non-null, live CF objects owned by this wrapper.
        unsafe { CFRelease(self.as_ptr().cast()) };
    }
}

fn string(value: &str) -> Result<Owned<__CFString>, Error> {
    let length = isize::try_from(value.len()).map_err(|_| Error::Payload)?;
    // SAFETY: `value` supplies `length` readable UTF-8 bytes for the call.
    let value = unsafe {
        CFStringCreateWithBytes(null(), value.as_ptr(), length, kCFStringEncodingUTF8, 0)
    };
    Owned::new(value).ok_or(Error::Allocation)
}

fn data(value: &[u8]) -> Result<CFDataRef, Error> {
    let length = isize::try_from(value.len()).map_err(|_| Error::Payload)?;
    // SAFETY: The slice is readable for the specified length; CFDataCreate copies it.
    let value = unsafe { CFDataCreate(null(), value.as_ptr(), length) };
    if value.is_null() {
        return Err(Error::Allocation);
    }
    Ok(value)
}

// Use a scoped borrow so none of CF's storage can escape its owner or callback.
unsafe fn with_bytes<R>(
    data: CFDataRef,
    limit: usize,
    read: impl FnOnce(&[u8]) -> R,
) -> Result<R, Error> {
    if data.is_null() {
        return Ok(read(&[]));
    }
    // SAFETY: Caller supplies a live immutable CFData or null.
    let length = usize::try_from(unsafe { CFDataGetLength(data) }).map_err(|_| Error::Payload)?;
    if length > limit {
        return Err(Error::Payload);
    }
    if length == 0 {
        return Ok(read(&[]));
    }
    // SAFETY: For a live nonempty CFData, CF returns `length` readable bytes.
    let pointer = unsafe { CFDataGetBytePtr(data) };
    if pointer.is_null() {
        return Err(Error::Payload);
    }
    Ok(read(unsafe { std::slice::from_raw_parts(pointer, length) }))
}

pub struct Remote(Owned<__CFMessagePort>);

// SAFETY: CFMessagePort operations and refcounts are thread-safe. Its allocation
// is live until the last Rust owner drops it, which cannot overlap a borrow.
unsafe impl Send for Remote {}
unsafe impl Sync for Remote {}

impl Remote {
    pub fn open(name: &str) -> Result<Self, Error> {
        let name = string(name)?;
        // SAFETY: `name` owns a valid CFString; returned object is +1.
        let port = unsafe { CFMessagePortCreateRemote(null(), name.as_ptr()) };
        Owned::new(port).map(Self).ok_or(Error::Unavailable)
    }

    pub fn is_valid(&self) -> bool {
        // SAFETY: Self owns the port.
        unsafe { CFMessagePortIsValid(self.0.as_ptr()) != 0 }
    }

    pub fn send(&self, id: i32, payload: &[u8]) -> Result<(), Error> {
        self.exchange(id, payload, None).map(|_| ())
    }

    pub fn request(
        &self,
        id: i32,
        payload: &[u8],
        wait: Duration,
        max_reply: usize,
    ) -> Result<Vec<u8>, Error> {
        self.exchange(id, payload, Some((wait, max_reply)))
    }

    fn exchange(
        &self,
        id: i32,
        payload: &[u8],
        reply: Option<(Duration, usize)>,
    ) -> Result<Vec<u8>, Error> {
        let message = Owned::new(data(payload)?).ok_or(Error::Allocation)?;
        let mode = reply
            .map(|_| string("com.native-instruments.ipc.wait.for.reply"))
            .transpose()?;
        let mut output = null();
        // SAFETY: Port, message and mode live through the call. `output` is writable
        // and CF transfers a +1 reference when it supplies data.
        let status = unsafe {
            CFMessagePortSendRequest(
                self.0.as_ptr(),
                id,
                message.as_ptr(),
                0.025,
                reply.map_or(0.0, |(wait, _)| wait.as_secs_f64()),
                mode.as_ref()
                    .map_or(null(), |value| value.as_ptr().cast_const()),
                &mut output,
            )
        };
        let output = Owned::new(output);
        if status != 0 {
            return Err(Error::Status(status));
        }
        let Some((_, limit)) = reply else {
            return Ok(Vec::new());
        };
        // SAFETY: The reply data remains owned throughout the copy.
        unsafe {
            with_bytes(
                output.as_ref().map_or(null(), |v| v.as_ptr().cast_const()),
                limit,
                <[u8]>::to_vec,
            )
        }
    }
}

impl Drop for Remote {
    fn drop(&mut self) {
        // SAFETY: Releases the CF-owned reply Mach port and its retained context
        // before releasing our reference. This adapter owns the named remote.
        unsafe { CFMessagePortInvalidate(self.0.as_ptr()) };
    }
}

type Handler = Box<dyn Fn(i32, Result<&[u8], Error>) -> Option<Vec<u8>> + Send + Sync>;

unsafe extern "C" fn retain(info: *const c_void) -> *const c_void {
    // SAFETY: CF supplies the `Arc::as_ptr` context while it has a live reference.
    unsafe { Arc::increment_strong_count(info.cast::<(usize, Handler)>()) };
    info
}

unsafe extern "C" fn release(info: *const c_void) {
    // SAFETY: Matches a reference acquired by the retain callback.
    unsafe { Arc::decrement_strong_count(info.cast::<(usize, Handler)>()) };
}

unsafe extern "C" fn receive(
    _port: CFMessagePortRef,
    id: i32,
    data_ref: CFDataRef,
    info: *mut c_void,
) -> CFDataRef {
    // SAFETY: CF keeps a context reference and a live CFData for this callback.
    let (limit, handler) = unsafe { &*info.cast::<(usize, Handler)>() };
    // Never unwind into CF. The production callback only enqueues a bounded copy.
    catch_unwind(AssertUnwindSafe(|| {
        let response = unsafe { with_bytes(data_ref, *limit, |bytes| handler(id, Ok(bytes))) }
            .unwrap_or_else(|error| handler(id, Err(error)));
        response
            .as_deref()
            .and_then(|bytes| data(bytes).ok())
            .unwrap_or(null())
    }))
    .unwrap_or(null())
}

pub struct Local(Owned<__CFMessagePort>);

// SAFETY: CF delivers on its own serial queue; Arc-managed callback data
// remains live during delivery even if another thread invalidates the port.
unsafe impl Send for Local {}

impl Local {
    pub fn bind(
        name: &str,
        max_message: usize,
        handler: impl Fn(i32, Result<&[u8], Error>) -> Option<Vec<u8>> + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        let name = string(name)?;
        let handler: Arc<(usize, Handler)> = Arc::new((max_message, Box::new(handler)));
        let context = CFMessagePortContext {
            version: 0,
            info: Arc::as_ptr(&handler).cast_mut().cast(),
            retain: Some(retain),
            release: Some(release),
            copyDescription: None,
        };
        let mut name_in_use = 1;
        // SAFETY: CF copies `context` and invokes retain if it takes ownership;
        // our reference remains live throughout creation, including failure.
        let port = unsafe {
            CFMessagePortCreateLocal(
                null(),
                name.as_ptr(),
                Some(receive),
                &context,
                &mut name_in_use,
            )
        };
        let port = Owned::new(port).ok_or(Error::Unavailable)?;
        if name_in_use != 0 {
            // CF can return an existing local port. Only release the extra ref.
            return Err(Error::NameInUse);
        }
        let local = Self(port);
        // SAFETY: Valid NUL-terminated label, NULL attributes select a serial queue.
        let queue =
            NonNull::new(unsafe { dispatch_queue_create(c"s4-ni-msgport".as_ptr(), null()) })
                .ok_or(Error::Allocation)?;
        // SAFETY: Dispatch queue and port are live. CF retains the queue.
        unsafe {
            CFMessagePortSetDispatchQueue(local.0.as_ptr(), queue.as_ptr());
            dispatch_release(queue.as_ptr());
        }
        Ok(local)
    }
}

impl Drop for Local {
    fn drop(&mut self) {
        // SAFETY: This wrapper created the port and owns the +1 reference.
        unsafe { CFMessagePortInvalidate(self.0.as_ptr()) };
    }
}
