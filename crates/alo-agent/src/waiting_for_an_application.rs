//! **Waiting for the person to let alo use an application**, after a call for
//! one was refused and the turn paused
//! (`alo_protocol::what_the_agent_says_back::AN_APPLICATION_WAS_REFUSED`; the
//! owner's decision of 2026-11-16, `docs/design/when-alo-asks-for-an-application.md`).
//!
//! The shell writes one line: `alo-granted:` when the person let alo use it,
//! `alo-not-granted:` when they said *Not now* or nothing was wanted. Anything
//! else ends the turn, as a line that is not an answer does after a change.
//! **Nothing here grants anything**: the grant is the person's, made on their
//! own door before the shell writes a word; this only learns whether to ask
//! again.

use std::io::BufRead;
use std::os::fd::BorrowedFd;
use std::time::Duration;

use alo_protocol::what_the_agent_says_back::{IT_WAS_NOT_GIVEN, the_grant_told};

/// What came back while the turn paused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheGrant {
    /// The person let alo use it: ask again, and the call waits for its own
    /// approval.
    Given,
    /// *Not now*, or nothing was wanted: the refusal stands, and the turn goes
    /// on with it as a step.
    NotGiven,
    /// The shell closed or shut its side: the person stopped, or closed the
    /// window.
    TheyLeft,
    /// Nothing came in the time the sheet may stand.
    ItLapsed,
    /// A line that is not one of the two, refused in words. The turn ends.
    NotAnAnswer(String),
}

/// How long the agent waits: the sheet stands five minutes before it is
/// answered *Not now* by itself, and a minute more covers a shell slow to say
/// so.
pub const HOW_LONG_IT_WAITS: Duration = Duration::from_secs(6 * 60);

/// Wait up to `bound` for the shell's one line about the application.
///
/// `socket` is the input's own descriptor, so the wait is bounded by the
/// kernel, as `crate::the_persons_answer`'s is.
pub fn the_persons_grant(
    input: &mut dyn BufRead,
    socket: BorrowedFd<'_>,
    bound: Duration,
) -> TheGrant {
    use rustix::net::sockopt::{Timeout, set_socket_timeout};

    let bound = bound.max(Duration::from_millis(1));
    if set_socket_timeout(socket, Timeout::Recv, Some(bound)).is_err() {
        return TheGrant::TheyLeft;
    }
    // The longer of the two lines and its newline: read no further.
    let most = u64::try_from(IT_WAS_NOT_GIVEN.len() + 2).unwrap_or(u64::MAX);
    let mut line = String::new();
    let read = std::io::Read::take(&mut *input, most).read_line(&mut line);
    let _ = set_socket_timeout(socket, Timeout::Recv, None);
    match read {
        Ok(0) => TheGrant::TheyLeft,
        Ok(_) => match the_grant_told(&line) {
            Some(true) => TheGrant::Given,
            Some(false) => TheGrant::NotGiven,
            None => TheGrant::NotAnAnswer(
                "the line after a call for an application paused was neither `alo-granted:` \
                 nor `alo-not-granted:`, so nothing more was driven"
                    .to_owned(),
            ),
        },
        Err(why)
            if line.is_empty()
                && matches!(
                    why.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
        {
            TheGrant::ItLapsed
        }
        Err(_) => TheGrant::TheyLeft,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::io::{BufReader, Write as _};
    use std::os::fd::AsFd as _;
    use std::os::unix::net::UnixStream;

    /// What the agent hears when the shell writes `line`, and then closes or
    /// waits.
    fn heard(line: Option<&str>, close: bool, bound: Duration) -> TheGrant {
        let (mut shell, agent) = UnixStream::pair().unwrap();
        if let Some(line) = line {
            shell.write_all(line.as_bytes()).unwrap();
        }
        if close {
            drop(shell);
            let mut input = BufReader::new(&agent);
            return the_persons_grant(&mut input, agent.as_fd(), bound);
        }
        let mut input = BufReader::new(&agent);
        let grant = the_persons_grant(&mut input, agent.as_fd(), bound);
        drop(shell);
        grant
    }

    /// **Given and not given read back as themselves.**
    #[test]
    fn given_and_not_given_read_back() {
        assert_eq!(
            heard(Some("alo-granted:\n"), false, Duration::from_secs(5)),
            TheGrant::Given
        );
        assert_eq!(
            heard(Some("alo-not-granted:\n"), false, Duration::from_secs(5)),
            TheGrant::NotGiven
        );
    }

    /// **Any other line is not an answer**, and in particular a change's
    /// answer is not a grant.
    #[test]
    fn any_other_line_is_not_an_answer() {
        for line in ["alo-answered:{}\n", "yes\n", "alo-granted:please\n"] {
            assert!(
                matches!(
                    heard(Some(line), false, Duration::from_secs(5)),
                    TheGrant::NotAnAnswer(_)
                ),
                "{line:?}"
            );
        }
    }

    /// **The shell going is the person leaving**, and silence lapses.
    #[test]
    fn closing_is_leaving_and_silence_lapses() {
        assert_eq!(
            heard(None, true, Duration::from_secs(5)),
            TheGrant::TheyLeft
        );
        assert_eq!(
            heard(None, false, Duration::from_millis(50)),
            TheGrant::ItLapsed
        );
    }
}
