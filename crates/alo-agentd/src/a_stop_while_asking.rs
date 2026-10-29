//! **An agent that hangs up while its question is out has stopped it.**
//!
//! [ADR 0139](../../../docs/decisions/0139-stop-reaches-alo-and-cancels-what-it-sent.md):
//! when a person presses Stop, `alo-agent` shuts its sending side of this
//! connection. If the daemon is, at that moment, putting the agent's question
//! to a provider, the provider's connection is shut too —
//! `alo_asking::stop_the_question_in_flight` — so **nothing more leaves after
//! Stop**, and the question's departure, and the egress indicator's line for
//! it, end.
//!
//! # A new meaning, not a new message
//!
//! Nothing on the wire changes: a half-close is something a client could
//! always do, and until now the daemon only noticed it at the next read. This
//! notices it *during* a message, and only one that began with the connection
//! open — a client that shuts its sending side as soon as it has written, which
//! is a way of saying *that was all*, is never read as a Stop.
//!
//! # Beside the loop, and touching nothing of the machine
//!
//! `crate::serving` is one thread that owns the machine. This watcher is a
//! second, for the length of one message: it holds a copy of the connection's
//! descriptor, asks the kernel whether the agent hung up, and on that shuts
//! one socket in `alo-asking`. It reads nothing, writes nothing, and holds no
//! grant, record or turn.

use std::os::fd::{AsFd as _, BorrowedFd, OwnedFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use rustix::event::{PollFd, PollFlags, Timespec};

/// How often the watcher looks, and so the longest a Stop waits to be seen.
const HOW_OFTEN: Duration = Duration::from_millis(50);

/// A watcher over one agent's connection, for the length of one message.
#[derive(Debug)]
pub(crate) struct Watching {
    /// Set when the message has been answered, so the watcher ends.
    answered: Arc<AtomicBool>,
    /// The watcher.
    watcher: Option<JoinHandle<bool>>,
}

/// **Watch this agent's connection** while one of its messages is answered.
///
/// [`None`] when the connection was already half-closed as the message began —
/// that is a client saying it has finished writing, never a Stop — or when its
/// descriptor could not be copied, in which case the message is answered as it
/// always was.
pub(crate) fn watching(agent: BorrowedFd<'_>) -> Option<Watching> {
    if hung_up(agent, Some(Duration::ZERO)) {
        return None;
    }
    let held: OwnedFd = rustix::io::dup(agent).ok()?;
    let answered = Arc::new(AtomicBool::new(false));
    let seen = Arc::clone(&answered);
    let watcher = std::thread::Builder::new()
        .name("alo-agentd-stop".to_owned())
        .spawn(move || {
            while !seen.load(Ordering::Acquire) {
                if hung_up(held.as_fd(), Some(HOW_OFTEN)) {
                    alo_asking::stop_the_question_in_flight();
                    return true;
                }
            }
            false
        })
        .ok()?;
    Some(Watching {
        answered,
        watcher: Some(watcher),
    })
}

impl Watching {
    /// The message has been answered: the watcher ends, and says whether the
    /// agent stopped it.
    pub(crate) fn answered(mut self) -> bool {
        self.ended()
    }

    /// End the watcher and read what it saw.
    fn ended(&mut self) -> bool {
        self.answered.store(true, Ordering::Release);
        self.watcher
            .take()
            .and_then(|watcher| watcher.join().ok())
            .unwrap_or(false)
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        drop(self.ended());
    }
}

/// Whether the other end has shut its sending side, or gone.
fn hung_up(connection: BorrowedFd<'_>, within: Option<Duration>) -> bool {
    let mut polling = [PollFd::from_borrowed_fd(connection, PollFlags::RDHUP)];
    let timeout = within.and_then(|within| Timespec::try_from(within).ok());
    match rustix::event::poll(&mut polling, timeout.as_ref()) {
        Ok(_) => polling.first().is_some_and(|polled| {
            polled
                .revents()
                .intersects(PollFlags::RDHUP | PollFlags::HUP)
        }),
        Err(_) => false,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;

    /// **A connection that shuts its sending side while a message is being
    /// answered is seen as a Stop.**
    #[test]
    fn an_agent_that_hangs_up_while_asking_is_seen() {
        let (agent, daemon) = UnixStream::pair().unwrap();
        let watching = watching(daemon.as_fd()).unwrap();
        agent.shutdown(std::net::Shutdown::Write).unwrap();
        std::thread::sleep(HOW_OFTEN * 4);
        assert!(watching.answered());
    }

    /// **A connection that stays open is not**, and the watcher ends when the
    /// message is answered.
    #[test]
    fn an_agent_that_waits_is_not_seen_to_stop() {
        let (_agent, daemon) = UnixStream::pair().unwrap();
        let watching = watching(daemon.as_fd()).unwrap();
        std::thread::sleep(HOW_OFTEN * 2);
        assert!(!watching.answered());
    }

    /// **A client that had already shut its sending side when the message
    /// began is not watched**: that is *I have finished writing*, never a Stop.
    #[test]
    fn a_client_that_finished_writing_before_the_message_is_never_a_stop() {
        let (agent, daemon) = UnixStream::pair().unwrap();
        agent.shutdown(std::net::Shutdown::Write).unwrap();
        assert!(watching(daemon.as_fd()).is_none());
    }
}
