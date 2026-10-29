//! **The person may stop this turn**, and this is how the agent hears it.
//!
//! [ADR 0139](../../../docs/decisions/0139-stop-reaches-alo-and-cancels-what-it-sent.md):
//! pressing Stop in a task window makes the person's shell shut its sending
//! side of the request's connection — this process's input. While the turn is
//! driven, a watcher looks for exactly that. When it comes it remembers that
//! the person stopped, so the loop asks for nothing more
//! (`crate::driving`), and it shuts this process's own sending side to the
//! daemon, which cancels a question the daemon is putting to a provider at that
//! moment, so nothing more leaves.
//!
//! # A caller that finished writing is not stopping
//!
//! A caller that shuts its sending side as soon as it has written — `alo-ask`
//! does — is saying *that was all*, and is never watched: that half-close was
//! already there when the turn began (`crate::what_the_person_sent`).
//!
//! # Watching ends before the turn is held
//!
//! After a proposal, `crate::holding_the_turn` reads this same input to learn
//! that the person answered. The watcher is ended and joined first, so two
//! readers never share the input; a Stop that arrived meanwhile is still there
//! for holding to read, as the end of the connection.

use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use rustix::event::{PollFd, PollFlags, Timespec};

/// How often the watcher looks, and so the longest a Stop waits to be heard.
const HOW_OFTEN: Duration = Duration::from_millis(50);

/// Whether the person has stopped this turn, as a watcher hears it.
#[derive(Debug)]
pub struct ThePersonMayStop {
    /// Set by the watcher when the person stopped.
    stopped: Arc<AtomicBool>,
    /// Set to end the watcher.
    ended: Arc<AtomicBool>,
    /// The watcher, while it runs.
    watcher: Option<JoinHandle<()>>,
}

impl ThePersonMayStop {
    /// **Nobody can stop this turn**: a request typed on a command line, or a
    /// caller that had already finished writing.
    #[must_use]
    pub fn cannot() -> Self {
        Self {
            stopped: Arc::new(AtomicBool::new(false)),
            ended: Arc::new(AtomicBool::new(true)),
            watcher: None,
        }
    }

    /// **Watch `input` for the person's Stop**, and on it shut the sending
    /// side of `to_the_daemon` — a second handle on the daemon connection.
    ///
    /// A watcher that cannot be started leaves a turn nobody can stop, which
    /// is what every turn was before this; it is never a reason to refuse one.
    #[must_use]
    pub fn watching(input: OwnedFd, to_the_daemon: UnixStream) -> Self {
        let stopped = Arc::new(AtomicBool::new(false));
        let ended = Arc::new(AtomicBool::new(false));
        let (heard, finished) = (Arc::clone(&stopped), Arc::clone(&ended));
        let watcher = std::thread::Builder::new()
            .name("alo-agent-stop".to_owned())
            .spawn(move || {
                while !finished.load(Ordering::Acquire) {
                    if the_input_ended(&input) {
                        heard.store(true, Ordering::Release);
                        drop(to_the_daemon.shutdown(std::net::Shutdown::Write));
                        return;
                    }
                }
            })
            .ok();
        Self {
            stopped,
            ended,
            watcher,
        }
    }

    /// Whether the person has stopped this turn.
    #[must_use]
    pub fn has_stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }

    /// **End the watcher**, before anything else reads the input, and answer
    /// whether the person stopped.
    pub fn ended(mut self) -> bool {
        self.finish();
        self.has_stopped()
    }

    /// End and join the watcher.
    fn finish(&mut self) {
        self.ended.store(true, Ordering::Release);
        if let Some(watcher) = self.watcher.take() {
            drop(watcher.join());
        }
    }
}

impl Drop for ThePersonMayStop {
    fn drop(&mut self) {
        self.finish();
    }
}

/// Whether the person's side of the input has ended, within [`HOW_OFTEN`].
///
/// Anything that arrives instead is read and dropped: nothing is said on this
/// connection after the request (`docs/contracts/asking-socket.md`), and an
/// unread byte would keep the input looking ready for ever.
fn the_input_ended(input: &OwnedFd) -> bool {
    let mut polling = [PollFd::new(input, PollFlags::IN | PollFlags::RDHUP)];
    let timeout = Timespec::try_from(HOW_OFTEN).ok();
    if rustix::event::poll(&mut polling, timeout.as_ref()).is_err() {
        return false;
    }
    let said = polling
        .first()
        .map_or_else(PollFlags::empty, PollFd::revents);
    if said.intersects(PollFlags::RDHUP | PollFlags::HUP) {
        return true;
    }
    if said.contains(PollFlags::IN) {
        let mut ignored = [0_u8; 64];
        return matches!(rustix::io::read(input, &mut ignored), Ok(0));
    }
    false
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::io::Read as _;

    /// A watcher over one end of a pair, standing in for the shell's
    /// connection, and the far end of a second pair, standing in for the
    /// daemon's.
    fn a_turn() -> (UnixStream, UnixStream, ThePersonMayStop) {
        let (shell, input) = UnixStream::pair().unwrap();
        let (to_the_daemon, daemon) = UnixStream::pair().unwrap();
        let watching = ThePersonMayStop::watching(OwnedFd::from(input), to_the_daemon);
        (shell, daemon, watching)
    }

    /// **The shell's half-close is heard as a Stop**, and the daemon is told
    /// by this side's own half-close.
    #[test]
    fn a_shell_that_shuts_its_side_stops_the_turn_and_the_daemon_hears() {
        let (shell, mut daemon, watching) = a_turn();
        shell.shutdown(std::net::Shutdown::Write).unwrap();
        std::thread::sleep(HOW_OFTEN * 4);
        assert!(watching.has_stopped());
        daemon
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        assert_eq!(
            daemon.read(&mut [0_u8; 1]).unwrap(),
            0,
            "the daemon was not told"
        );
        assert!(watching.ended());
    }

    /// **A shell that keeps its side open has not stopped**, and ending the
    /// watcher leaves the daemon's connection as it was.
    #[test]
    fn a_shell_that_waits_has_not_stopped() {
        let (_shell, daemon, watching) = a_turn();
        std::thread::sleep(HOW_OFTEN * 2);
        assert!(!watching.ended());
        daemon.set_nonblocking(true).unwrap();
        assert!(
            (&daemon).read(&mut [0_u8; 1]).is_err(),
            "the daemon's connection was shut without a Stop"
        );
    }

    /// **A turn nobody can stop never says it was stopped.**
    #[test]
    fn a_turn_nobody_can_stop_has_not_stopped() {
        assert!(!ThePersonMayStop::cannot().ended());
    }
}
