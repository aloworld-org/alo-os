//! Libseat notifier integration for a single direct-display device.

use crate::{
    SessionError,
    session_device::{SessionDevice, backend_error},
};
use smithay::{
    backend::session::{Event, Session, libseat::LibSeatSession},
    reexports::calloop::EventLoop,
};
use std::{io, os::fd::BorrowedFd, path::PathBuf, time::Duration};

/// A trusted shell session, with no direct-open fallback or VT-switch interface.
///
/// Poll regularly in the compositor loop, even while idle. Each access also
/// dispatches pending seat events. Activation lazily reacquires a fresh device;
/// callers must rebuild DRM resources and never duplicate the scoped descriptor.
/// `with_device` supports short discovery operations. `with_active_device` keeps
/// a renderer's descriptor alive across polls until its retirement scope returns.
pub struct DirectSession {
    /// Declared first so device cleanup precedes notifier/seat destruction.
    device: SessionDevice<LibSeatSession>,
    /// Owns the notifier (and therefore the live libseat connection).
    events: EventLoop<'static, SessionDevice<LibSeatSession>>,
}

impl DirectSession {
    /// Connect to the login seat, without opening the requested device yet.
    ///
    /// `path` is trusted shell configuration. libseat decides device permissions;
    /// never pass an agent-supplied path here. Uses pinned Smithay's unmodified
    /// libseat backend; its internal panic behavior is an upstream limitation.
    pub fn new(path: PathBuf) -> Result<Self, SessionError> {
        let (session, notifier) =
            LibSeatSession::new().map_err(|error| backend_error("connect", error))?;
        let events = EventLoop::try_new().map_err(|error| SessionError::Backend {
            stage: "create event loop",
            source: io::Error::other(error.to_string()),
        })?;
        events
            .handle()
            .insert_source(notifier, |event, _, device| session_event(event, device))
            .map_err(|error| SessionError::Backend {
                stage: "register notifier",
                source: io::Error::other(error.to_string()),
            })?;
        Ok(Self {
            device: SessionDevice::new(session, path),
            events,
        })
    }

    /// Dispatch ready notifications without blocking; retire access on failure.
    pub fn poll(&mut self) -> Result<(), SessionError> {
        dispatch_session(&mut self.events, &mut self.device)
    }

    /// Dispatch seat events, then lend the active descriptor for one synchronous
    /// operation. Re-query DRM resources each time; snapshots are not reservations.
    /// The kernel can revoke permission during the operation, so handle ioctl errors.
    pub fn with_device<T>(
        &mut self,
        operation: impl FnOnce(BorrowedFd<'_>) -> T,
    ) -> Result<T, SessionError> {
        self.poll()?;
        Ok(operation(self.device.device()?))
    }

    /// Lend one device lifetime and a nonblocking seat poll to a trusted loop.
    ///
    /// Poll before every frame and while idle. On any poll error, stop submitting,
    /// call `Server::retire_output`, drop the target and return from the closure.
    /// Pause remains latched even if activation arrives in the same batch. The
    /// descriptor stays open until the closure returns (also during unwinding),
    /// then closes exactly once; a subsequent scope reacquires fresh resources.
    /// Both the caller's outcome and close failure are returned independently.
    ///
    /// This is trusted shell control flow, not authority to continue after pause.
    /// Kernel revocation may precede notification, so retirement can still fail.
    /// Pinned Smithay acknowledges libseat disable before delivering the event;
    /// this API does not fix that upstream ordering or certify physical scanout.
    /// A borrowed descriptor (and thus a target borrowing it) cannot escape:
    ///
    /// ```compile_fail
    /// # fn cannot_escape(session: &mut alo_shell::DirectSession) {
    /// let _escaped = session.with_active_device(|fd, _poll| fd);
    /// # }
    /// ```
    pub fn with_active_device<T>(
        &mut self,
        operation: impl FnOnce(BorrowedFd<'_>, &mut dyn FnMut() -> Result<(), SessionError>) -> T,
    ) -> Result<crate::ActiveSessionResult<T>, SessionError> {
        crate::active_session::run(&mut self.device, &mut self.events, operation)
    }

    /// Close the device while the seat connection is alive and report errors.
    pub fn shutdown(mut self) -> Result<(), SessionError> {
        self.device.shutdown()
    }
}

/// Preserve notification cleanup errors across the callback's unit return type.
pub(crate) fn session_event<S: Session>(event: Event, device: &mut SessionDevice<S>) {
    if let Err(error) = device.event(event) {
        device.pending = Some(error);
    }
}

/// Drain the seat-to-channel handoff before allowing synchronous device access.
pub(crate) fn dispatch_session<S: Session + 'static>(
    events: &mut EventLoop<'static, SessionDevice<S>>,
    device: &mut SessionDevice<S>,
) -> Result<(), SessionError> {
    // Smithay dispatches libseat callbacks into a calloop channel. A second
    // readiness pass delivers events queued by the first pass's seat fd.
    for _ in 0..2 {
        events.dispatch(Duration::ZERO, device).map_err(|error| {
            device.fail();
            SessionError::Backend {
                stage: "dispatch notifier",
                source: io::Error::other(error.to_string()),
            }
        })?;
    }
    device.check()
}
