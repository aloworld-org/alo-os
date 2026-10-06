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
    /// **libseat has no backend at all: no login session, and no `seatd`.**
    ///
    /// Told apart from [`Self::Backend`] because the errno alone sends a
    /// reader to the wrong place. *Function not implemented* reads as a
    /// missing kernel feature or an unsupported device; what it means here is
    /// that libseat tried every way it has of asking for a seat and found
    /// none of them available.
    ///
    /// **Measured on a booted machine, 2026-10-05.** `alo-compositor` failed
    /// this way four times out of four on the first alo OS image to carry a
    /// compositor, and the sentence it printed —
    /// *display session connect failed: Function not implemented (os error
    /// 38)* — was accurate and cost an evening of somebody else's time to
    /// interpret. This is that interpretation, written where the next person
    /// meets it.
    ///
    /// **It does not say how to fix it**, and that is deliberate: a greeter
    /// may be given a seat by a login session or by running `seatd`, and
    /// [ADR 0088](../../../docs/decisions/0088-a-machines-grants-belong-to-a-person.md)
    /// reserves that choice to the owner. An error that recommended one would
    /// be this crate deciding something it was told not to.
    #[error(
        "there is no seat to take a display from: libseat found no login session for this \
         process and no seatd to ask instead. A compositor started as a system service is in \
         no login session — ordering after logind is not membership in one"
    )]
    NoSeat,
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
/// What libseat answers when **every** backend it has is unavailable.
///
/// `ENOSYS`. Written as its number because that is what reaches a log and what
/// a person searches for, and named because `38` in a comparison is the kind
/// of figure nobody can check.
const NO_BACKEND_AT_ALL: i32 = 38;

/// The refusal a seat connection makes, with `ENOSYS` told apart from the rest.
///
/// Only at `connect`: an `ENOSYS` from a later stage is a real unsupported
/// operation on a seat this process already has, which is a different fault
/// and keeps the errno that names it.
pub(crate) fn connect_error(error: impl AsErrno) -> SessionError {
    match backend_error("connect", error) {
        SessionError::Backend { source, .. }
            if source.raw_os_error() == Some(NO_BACKEND_AT_ALL) =>
        {
            SessionError::NoSeat
        }
        other => other,
    }
}

/// Preserve the backend's errno where available, naming the stage it failed at.
///
/// **Its doc comment was lost when `connect_error` was inserted above it** and
/// CI caught it as *missing documentation for a function*. That is the third
/// time in one day a new item has been written directly above an existing one
/// and taken its doc with it — the others were in `alo-arranging` and
/// `alo-access`. The fault is not the insertion, it is inserting *between* a
/// doc comment and the thing it documents, which reads as correct in a diff
/// because both halves look untouched.
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

    /// Share only the manager connection, never the scoped display descriptor.
    pub(crate) fn input_session(&self) -> S
    where
        S: Clone,
    {
        self.session.clone()
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

#[cfg(test)]
mod no_seat_tests {
    use super::*;

    /// A backend error carrying exactly the errno libseat gave.
    ///
    /// `Debug` because `backend_error` falls back to formatting the error when
    /// it carries no errno — a path this fixture never takes, and the bound
    /// does not know that.
    #[derive(Debug)]
    struct Said(i32);
    impl AsErrno for Said {
        fn as_errno(&self) -> Option<i32> {
            Some(self.0)
        }
    }

    /// **`ENOSYS` at connect is *no seat*, and says so without an errno.**
    ///
    /// The sentence a person reads is the whole point of this: *Function not
    /// implemented* sends a reader to the kernel or the device, and the fault
    /// is neither.
    #[test]
    fn enosys_at_connect_is_a_seat_that_does_not_exist() {
        let said = connect_error(Said(NO_BACKEND_AT_ALL)).to_string();
        assert!(
            matches!(connect_error(Said(NO_BACKEND_AT_ALL)), SessionError::NoSeat),
            "ENOSYS at connect was left as a bare backend error"
        );
        assert!(
            said.contains("no seat") && said.contains("no login session"),
            "the refusal does not say what is missing: {said}"
        );
        assert!(
            !said.contains("38") && !said.contains("not implemented"),
            "the refusal still sends a reader to the errno: {said}"
        );
    }

    /// **Every other errno keeps the errno**, because every other errno means
    /// something the number names better than a sentence would.
    #[test]
    fn any_other_refusal_at_connect_keeps_its_errno() {
        for errno in [1, 2, 13, 19, 110] {
            assert!(
                matches!(
                    connect_error(Said(errno)),
                    SessionError::Backend {
                        stage: "connect",
                        ..
                    }
                ),
                "errno {errno} was turned into a seat refusal it does not mean"
            );
        }
    }
}
