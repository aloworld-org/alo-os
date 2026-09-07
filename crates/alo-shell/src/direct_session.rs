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
/// dispatches pending seat events. Pause closes the old device; activation lazily
/// reacquires it. Callers must rebuild DRM resources after reacquisition, and must
/// not duplicate or retain the scoped descriptor or scanout objects across polls.
/// This discovery-stage owner does not yet coordinate a running DRM renderer.
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
