//! Scoped device ownership while the compositor polls seat notifications.

use crate::{SessionError, direct_session::dispatch_session, session_device::SessionDevice};
use smithay::{backend::session::Session, reexports::calloop::EventLoop};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};

/// Independent operation and device-close outcomes from an active session scope.
#[derive(Debug)]
#[must_use = "inspect both the operation outcome and session cleanup result"]
pub struct ActiveSessionResult<T> {
    /// The caller's result, including any rendering or output-retirement failure.
    pub outcome: T,
    /// Session-manager close result; failure permanently retires the session.
    pub cleanup: Result<(), SessionError>,
}

/// Close on normal return and unwinding, after the caller's borrowed target drops.
struct Scope<'a, S: Session> {
    /// Manager remains alive until the scoped descriptor has been consumed.
    device: &'a mut SessionDevice<S>,
    /// Taken before close so drop never retries a failed manager operation.
    fd: Option<OwnedFd>,
}

impl<S: Session> Scope<'_, S> {
    /// Explicit cleanup reports errors; Drop provides the unwind backstop.
    fn close(&mut self) -> Result<(), SessionError> {
        match self.fd.take() {
            Some(fd) => self.device.close_owned(fd),
            None => Ok(()),
        }
    }
}

impl<S: Session> Drop for Scope<'_, S> {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// The generic implementation is also exercised with real calloop notifications.
pub(crate) fn run<S: Session + 'static, T>(
    device: &mut SessionDevice<S>,
    events: &mut EventLoop<'static, SessionDevice<S>>,
    operation: impl FnOnce(BorrowedFd<'_>, &mut dyn FnMut() -> Result<(), SessionError>) -> T,
) -> Result<ActiveSessionResult<T>, SessionError> {
    dispatch_session(events, device)?;
    let fd = device.take_active()?;
    let mut scope = Scope {
        device,
        fd: Some(fd),
    };
    let fd = scope.fd.as_ref().ok_or(SessionError::Failed)?.as_fd();
    let device = &mut scope.device;
    let mut poll = || {
        // Once interrupted, do not dispatch further events or revive this scope.
        device.check_active()?;
        dispatch_session(events, device)?;
        device.check_active()
    };
    let outcome = operation(fd, &mut poll);
    Ok(ActiveSessionResult {
        outcome,
        cleanup: scope.close(),
    })
}
