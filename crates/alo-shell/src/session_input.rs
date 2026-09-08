//! Session-mediated libinput descriptor acquisition and observable close failures.

use smithay::{
    backend::session::{AsErrno, Session},
    reexports::input::LibinputInterface,
};
use std::{cell::RefCell, io, os::fd::OwnedFd, path::Path, rc::Rc};

#[cfg(test)]
#[path = "session_input_tests.rs"]
mod tests;

/// Shared failure latch for callbacks whose upstream interface cannot return errors.
///
/// Keep this handle beside the libinput context and check it before delivering
/// input. A failure requires retiring that context, clearing the server's input,
/// and creating a fresh bridge. Cloning never clears the first failure.
#[derive(Clone, Default)]
pub struct SessionInputStatus(Rc<RefCell<Option<(&'static str, i32)>>>);

impl SessionInputStatus {
    /// Return the first acquisition/close failure without consuming the latch.
    pub fn check(&self) -> io::Result<()> {
        match *self.0.borrow() {
            Some((stage, errno)) => Err(io::Error::new(
                io::Error::from_raw_os_error(errno).kind(),
                format!(
                    "input session {stage}: {}",
                    io::Error::from_raw_os_error(errno)
                ),
            )),
            None => Ok(()),
        }
    }

    /// Retain the original stage and errno across later cleanup failures.
    fn fail(&self, stage: &'static str, errno: i32) {
        self.0.borrow_mut().get_or_insert((stage, errno));
    }
}

/// Restricted device callbacks for a trusted compositor's libinput context.
///
/// Every open and close goes through the supplied seat manager. No filesystem
/// fallback or agent-facing path interface exists. The caller must poll its seat
/// notifier and suspend/drop libinput on pause before dispatching more input;
/// checking `Session::is_active` here alone cannot retire existing descriptors.
/// The session/notifier must outlive the context so libinput can close its devices.
pub struct SessionInput<S: Session> {
    /// Manager handle remains owned until libinput releases the interface.
    session: S,
    /// Shared because libinput owns this interface and close returns no result.
    status: SessionInputStatus,
}

impl<S: Session> SessionInput<S> {
    /// Build without opening devices; return a separate observer for callback errors.
    pub fn new(session: S) -> (Self, SessionInputStatus) {
        let status = SessionInputStatus::default();
        (
            Self {
                session,
                status: status.clone(),
            },
            status,
        )
    }
}

impl<S: Session> LibinputInterface for SessionInput<S> {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        if self.status.check().is_err() {
            return Err(rustix::io::Errno::IO.raw_os_error());
        }
        if !self.session.is_active() {
            // Inactivity is recoverable, but must not attempt an acquisition.
            return Err(rustix::io::Errno::ACCESS.raw_os_error());
        }
        let flags = rustix::fs::OFlags::from_bits_retain(flags as u32)
            | rustix::fs::OFlags::CLOEXEC
            | rustix::fs::OFlags::NOCTTY
            | rustix::fs::OFlags::NONBLOCK;
        self.session.open(path, flags).map_err(|error| {
            let errno = error
                .as_errno()
                .unwrap_or(rustix::io::Errno::PERM.raw_os_error());
            self.status.fail("open device", errno);
            errno
        })
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        // Always consume through the manager, even after pause or an earlier error.
        if let Err(error) = self.session.close(fd) {
            self.status.fail(
                "close device",
                error
                    .as_errno()
                    .unwrap_or(rustix::io::Errno::PERM.raw_os_error()),
            );
        }
    }
}
