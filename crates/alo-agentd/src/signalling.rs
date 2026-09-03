//! The signal that asks this service to stop, and the one call a handler may
//! be.
//!
//! `crate::stopping` says why a stop is a byte on a socket rather than a flag in
//! memory: the service sleeps inside one `poll`, and the way something asleep on
//! a set of descriptors is woken is by one of them becoming readable. This file
//! is the other end of that argument — the thing that really asks, on a running
//! machine, is `SIGTERM` from whatever supervises the service.
//!
//! # A handler may do almost nothing, and this one does the least
//!
//! A signal handler runs between two instructions of whatever the process was
//! doing, so it may not allocate, take a lock, or call most of the standard
//! library: any of those can be interrupted while already holding the thing the
//! handler would need. Writing one byte to an already-open descriptor is one of
//! the few things it *may* do, and it is the whole of what is installed here.
//! Everything else — noticing, ending the turn, giving the grant back, taking
//! the socket away — happens on the ordinary path afterwards, in
//! `crate::serving`.
//!
//! # Why this is somebody else's crate
//!
//! Installing a handler is `sigaction`, and there is no safe spelling of it in
//! Rust: `rustix`'s is in its `runtime` module and is `unsafe`, and so is every
//! other. `CLAUDE.md` forbids `unsafe` in this workspace, so the choice is a
//! crate or an exception to the rule, and it is the same choice `crate::unix`
//! made about peer credentials and waiting on several descriptors at once.
//! `signal_hook::low_level::pipe` is the one safe API for exactly this shape,
//! and it is a small dependency doing a small thing: it registers a handler
//! that writes one byte to a descriptor it owns.
//!
//! # `SIGTERM` and nothing else
//!
//! `SIGINT` is a terminal's — Ctrl-C, from somebody who started the service by
//! hand — and it is deliberately left alone. Not handling it costs nothing that
//! is not already paid: the default action ends the process, which is what a
//! service with no handler at all does, and the one thing left behind is a
//! socket nobody is listening on, which `crate::listening` removes the next time
//! the service starts. A machine runs this under systemd, and systemd asks with
//! `SIGTERM`.
//!
//! `SIGKILL` cannot be handled by anything, which is the reason the record is
//! appended to one entry at a time rather than held and written at the end.

use signal_hook::consts::SIGTERM;
use signal_hook::low_level::pipe;

use crate::refusing::NotStarted;
use crate::stopping::Stop;

/// Make `SIGTERM` ask this service to stop.
///
/// The stop is given away rather than borrowed: the registration outlives every
/// scope in the process, and a handler writing to a descriptor that had been
/// dropped would be writing to whatever was opened next. A process with a second
/// reason to stop makes a second handle first — `crate::Stop::again`.
///
/// # Errors
///
/// [`NotStarted::NoHandler`], carrying what the machine said. It is a refusal to
/// start rather than something to go on without: a service that cannot be asked
/// to stop can only be killed, and a killed service is one whose turn ended
/// somewhere nobody chose.
pub fn on_sigterm(stop: Stop) -> Result<(), NotStarted> {
    pipe::register(SIGTERM, stop.into_stream())
        .map(drop)
        .map_err(|why| NotStarted::NoHandler { why })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::stopping::Waking;
    use crate::unix::ready;
    use std::time::Duration;

    /// How long a test waits for a signal it has just sent itself.
    ///
    /// Long enough that a loaded machine is not a failing one, and short enough
    /// that a test which is really broken does not hold the suite up.
    const LONG_ENOUGH: Duration = Duration::from_secs(5);

    /// **A `SIGTERM` reaches the service the way a stop does**, which is the
    /// whole of what this file is for: the handler writes the byte, the `poll`
    /// the service is asleep in returns, and nothing else in the process had to
    /// be interrupted to make it happen.
    ///
    /// The signal is sent to this very process, so the registration is asserted
    /// before anything sends one — an unhandled `SIGTERM` ends the test runner
    /// rather than failing a test.
    #[test]
    fn a_sigterm_wakes_the_service() {
        let (waking, stop) = Waking::made().unwrap();
        on_sigterm(stop).unwrap();

        assert_eq!(
            ready(&[Some(waking.waiting_on())], Some(Duration::ZERO)).unwrap(),
            [false],
            "nothing had asked this service to stop yet"
        );

        rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::TERM)
            .unwrap();

        assert_eq!(
            ready(&[Some(waking.waiting_on())], Some(LONG_ENOUGH)).unwrap(),
            [true],
            "SIGTERM did not reach the end the service waits on"
        );
    }
}
