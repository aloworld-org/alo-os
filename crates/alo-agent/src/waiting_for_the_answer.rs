//! **Waiting for the person's answer on the request's own socket**, when the
//! shell said it would tell (`docs/contracts/asking-socket.md`, *The turn goes
//! on after an answer*).
//!
//! `crate::holding_the_turn` is the older road and is still taken by every
//! caller that did not say so: the agent shuts its sending side and waits for
//! the socket to close. On this road the sending side stays open, the reply
//! ends with `alo_protocol::what_the_agent_says_back::THE_REPLY_WAITS`, and the
//! shell writes one line once the daemon has answered the person: the marker
//! and the daemon's own answer, as the daemon wrote it.
//!
//! # What the line can and cannot do
//!
//! **It approves nothing.** The approval was given on the person's door, to the
//! daemon, before the line was written. The line is what the model is shown
//! next, and a shell that lied in it would only mislead the model about a
//! change it could not have made happen. Only the four answers an approve or a
//! decline earns are read; anything else ends the turn rather than being
//! guessed at.

use std::io::BufRead;
use std::os::fd::BorrowedFd;
use std::time::Duration;

use alo_protocol::ToAPerson;
use alo_protocol::what_the_agent_says_back::{THE_ANSWER, the_answer_told};

/// What came back while the change waited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheAnswer {
    /// The person answered, and this is what the daemon told them it did.
    Told(ToAPerson),
    /// The shell closed or shut its side: the person stopped, or closed the
    /// window. The change lapses with the turn.
    TheyLeft,
    /// The question's standing time passed with nothing said.
    ItLapsed,
    /// A line that is not the answer, refused in words. The turn ends.
    NotAnAnswer(String),
}

/// Wait up to `stands_for` for the shell's one line about the person's answer.
///
/// `socket` is the input's own descriptor, so the wait is bounded by the kernel
/// rather than by a thread; a zero bound is raised to one millisecond, because
/// zero means *for ever* to the kernel.
pub fn the_persons_answer(
    input: &mut dyn BufRead,
    socket: BorrowedFd<'_>,
    stands_for: Duration,
) -> TheAnswer {
    use rustix::net::sockopt::{Timeout, set_socket_timeout};

    let bound = stands_for.max(Duration::from_millis(1));
    if set_socket_timeout(socket, Timeout::Recv, Some(bound)).is_err() {
        return TheAnswer::TheyLeft;
    }
    // At most the marker and the longest answer a daemon may write, and its
    // newline: read no further than the line could be.
    let most = THE_ANSWER.len() + alo_protocol::LONGEST_ANSWER + 2;
    let most = u64::try_from(most).unwrap_or(u64::MAX);
    let mut line = String::new();
    let read = std::io::Read::take(&mut *input, most).read_line(&mut line);
    let _ = set_socket_timeout(socket, Timeout::Recv, None);
    match read {
        Ok(0) => TheAnswer::TheyLeft,
        Ok(_) => the_answer_told(&line).map_or_else(
            || {
                TheAnswer::NotAnAnswer(format!(
                    "the line after a change waited was not `{THE_ANSWER}` and an approve's or a \
                     decline's answer, so nothing more was driven"
                ))
            },
            TheAnswer::Told,
        ),
        Err(why)
            if line.is_empty()
                && matches!(
                    why.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
        {
            TheAnswer::ItLapsed
        }
        Err(_) => TheAnswer::TheyLeft,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_protocol::what_the_agent_says_back::the_answer_line;
    use std::io::{BufReader, Write as _};
    use std::os::fd::AsFd as _;
    use std::os::unix::net::UnixStream;
    use std::time::Instant;

    fn waited(shell_writes: Option<&str>, close: bool, stands_for: Duration) -> TheAnswer {
        let (mut shell, agent) = UnixStream::pair().unwrap();
        if let Some(line) = shell_writes {
            shell.write_all(line.as_bytes()).unwrap();
        }
        if close {
            drop(shell);
            let mut input = BufReader::new(&agent);
            return the_persons_answer(&mut input, agent.as_fd(), stands_for);
        }
        let mut input = BufReader::new(&agent);
        let answer = the_persons_answer(&mut input, agent.as_fd(), stands_for);
        drop(shell);
        answer
    }

    /// **A decline the daemon kept reaches the agent as a decline.**
    #[test]
    fn the_daemons_answer_is_read_back() {
        let line = format!("{}\n", the_answer_line(&ToAPerson::Declined).unwrap());
        assert_eq!(
            waited(Some(&line), false, Duration::from_secs(5)),
            TheAnswer::Told(ToAPerson::Declined)
        );
    }

    /// **The shell going is the person leaving**, not an answer.
    #[test]
    fn a_closed_socket_is_the_person_leaving() {
        assert_eq!(
            waited(None, true, Duration::from_secs(5)),
            TheAnswer::TheyLeft
        );
    }

    /// **Nothing said is a question that lapsed**, at its standing time.
    #[test]
    fn nothing_said_lapses_at_the_standing_time() {
        let started = Instant::now();
        assert_eq!(
            waited(None, false, Duration::from_millis(200)),
            TheAnswer::ItLapsed
        );
        assert!(started.elapsed() >= Duration::from_millis(150));
    }

    /// **Anything but the answer is refused**, and an approval cannot be
    /// smuggled in as one.
    #[test]
    fn a_line_that_is_not_an_answer_is_refused() {
        for line in [
            "approve 7\n",
            "alo-answered:{\"format\":1,\"asks\":{\"approve\":{\"number\":7}}}\n",
        ] {
            assert!(
                matches!(
                    waited(Some(line), false, Duration::from_secs(5)),
                    TheAnswer::NotAnAnswer(_)
                ),
                "{line}"
            );
        }
    }
}
