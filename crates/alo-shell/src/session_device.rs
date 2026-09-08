//! Own one session-mediated descriptor and retire it whenever authority pauses.

use smithay::backend::session::{AsErrno, Event, Session};
use std::{
    io,
    os::fd::{AsFd, BorrowedFd, OwnedFd},
    path::PathBuf,
};

#[cfg(test)]
#[path = "session_device_tests.rs"]
mod tests;

/// Direct-session failure; diagnostics are not person-facing UI strings.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// The session is paused; no device operation was attempted.
    #[error("display session is inactive")]
    Inactive,
    /// A previous transport or cleanup error requires a fresh session.
    #[error("display session failed; create a new session")]
    Failed,
    /// Preserve the backend's errno where available.
    #[error("display session {stage} failed: {source}")]
    Backend {
        /// Operation that failed.
        stage: &'static str,
        /// Underlying backend error.
        #[source]
        source: io::Error,
    },
}

/// Convert Smithay's errno-bearing backend error without losing its diagnostic.
pub(crate) fn backend_error(stage: &'static str, error: impl AsErrno) -> SessionError {
    SessionError::Backend {
        stage,
        source: error
            .as_errno()
            .map(io::Error::from_raw_os_error)
            .unwrap_or_else(|| io::Error::other(format!("{error:?}"))),
    }
}

/// Session lifetime owns the descriptor; callers receive only scoped borrows.
pub(crate) struct SessionDevice<S: Session> {
    /// Backend handle, kept alive until device cleanup finishes.
    session: S,
    /// Trusted compositor configuration, never an agent argument.
    path: PathBuf,
    /// Only acquired while active; never retained over a pause.
    fd: Option<OwnedFd>,
    /// Notification state independently prevents stale backend-active access.
    enabled: bool,
    /// Terminal failures cannot be erased by a later activation notification.
    failed: bool,
    /// A pause cannot be erased within a borrowed rendering lifetime.
    interrupted: bool,
    /// Callback error retained for the event-loop caller.
    pub(crate) pending: Option<SessionError>,
}

impl<S: Session> SessionDevice<S> {
    /// Start without opening a device, including for an initially inactive seat.
    pub(crate) fn new(session: S, path: PathBuf) -> Self {
        let enabled = session.is_active();
        Self {
            session,
            path,
            fd: None,
            enabled,
            failed: false,
            interrupted: false,
            pending: None,
        }
    }

    /// Close through the session manager, consuming the descriptor even on error.
    fn release(&mut self) -> Result<(), SessionError> {
        if let Some(fd) = self.fd.take() {
            self.close_owned(fd)?;
        }
        Ok(())
    }

    /// Process every notification in order, including pause/activate in one batch.
    pub(crate) fn event(&mut self, event: Event) -> Result<(), SessionError> {
        match event {
            Event::PauseSession => {
                self.interrupted = true;
                self.enabled = false;
                self.release()
            }
            Event::ActivateSession => {
                self.enabled = true;
                Ok(())
            }
        }
    }

    /// Retire device access permanently after notifier loss.
    pub(crate) fn fail(&mut self) {
        self.interrupted = true;
        self.failed = true;
        self.enabled = false;
        let _ = self.release();
    }

    /// Surface a callback's original error once, then retain terminal refusal.
    pub(crate) fn check(&mut self) -> Result<(), SessionError> {
        if let Some(error) = self.pending.take() {
            return Err(error);
        }
        if self.failed {
            Err(SessionError::Failed)
        } else {
            Ok(())
        }
    }

    /// Check both notification and backend state before acquiring or borrowing.
    pub(crate) fn device(&mut self) -> Result<BorrowedFd<'_>, SessionError> {
        if self.failed {
            return Err(SessionError::Failed);
        }
        if !self.enabled || !self.session.is_active() {
            self.release()?;
            return Err(SessionError::Inactive);
        }
        if self.fd.is_none() {
            self.fd = Some(
                self.session
                    .open(
                        &self.path,
                        rustix::fs::OFlags::RDWR
                            | rustix::fs::OFlags::CLOEXEC
                            | rustix::fs::OFlags::NOCTTY
                            | rustix::fs::OFlags::NONBLOCK,
                    )
                    .map_err(|error| {
                        self.failed = true;
                        backend_error("open device", error)
                    })?,
            );
        }
        self.fd
            .as_ref()
            .map(AsFd::as_fd)
            .ok_or(SessionError::Failed)
    }

    /// Transfer ownership to a scope which closes after renderer retirement.
    pub(crate) fn take_active(&mut self) -> Result<OwnedFd, SessionError> {
        self.device()?;
        self.interrupted = false;
        self.fd.take().ok_or(SessionError::Failed)
    }

    /// Check authority without closing the scope's still-borrowed descriptor.
    pub(crate) fn check_active(&mut self) -> Result<(), SessionError> {
        self.check()?;
        if !self.session.is_active() {
            self.interrupted = true;
        }
        if self.interrupted || !self.enabled {
            Err(SessionError::Inactive)
        } else {
            Ok(())
        }
    }

    /// Consume the scope's descriptor through the manager, including on failure.
    pub(crate) fn close_owned(&mut self, fd: OwnedFd) -> Result<(), SessionError> {
        self.session.close(fd).map_err(|error| {
            self.failed = true;
            backend_error("close device", error)
        })
    }

    /// Explicit shutdown reports cleanup failure; drop remains a final backstop.
    pub(crate) fn shutdown(&mut self) -> Result<(), SessionError> {
        self.enabled = false;
        self.failed = true;
        self.release()
    }
}

impl<S: Session> Drop for SessionDevice<S> {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
