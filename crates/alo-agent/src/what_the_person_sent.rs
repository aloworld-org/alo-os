//! **What a person's shell sent**, read off the connection `alo-asking.socket`
//! hands this process (ADR 0109; `docs/contracts/asking-socket.md`).
//!
//! Up to three lines, each with its own reason to be there:
//!
//! 1. `alo-may-reach:` and the folders this person granted, tab-separated —
//!    only when there are any. A hint for the model, never a capability.
//! 2. The request, in the person's own words.
//! 3. `alo-earlier:` and the conversation so far, when the request is a
//!    follow-up typed in a task window
//!    ([ADR 0136](../../../docs/decisions/0136-a-follow-up-is-sent-with-the-conversation-so-far.md)).
//!    A current shell always sends it, with nothing after the marker for a
//!    first request.
//!
//! # Why the third line is after the request, and waited for briefly
//!
//! **An older agent must keep working.** One reads the first line, and the
//! second if the first named folders, and never reads again — so a line after
//! the request is a line it ignores, and it answers the follow-up as though it
//! were a first request. Anywhere before the request it would have been read
//! *as* the request.
//!
//! The cost is on this side. A shell written before this sends no third line
//! and does not close its end — the end stays open while a change waits
//! (`crate::holding_the_turn`) — so reading for one could wait for ever. It is
//! waited for for [`HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR`] and no longer, and
//! only on a socket: a current shell writes all three lines at once, so the
//! wait ends as soon as it starts.

use std::io::BufRead;
use std::os::fd::BorrowedFd;
use std::path::Path;
use std::time::Duration;

use alo_instructing::{TheConversationSoFar, WhatItMayReach};

/// The marker a line of granted folders begins with.
///
/// **A marker rather than a second channel**, because the connection
/// `alo-asking.socket` hands over is one stream and a request is one line.
/// Anything that does not begin with this *is* the request, so every caller
/// written before this one keeps working unchanged.
pub const THE_FOLDERS: &str = "alo-may-reach:";

/// How long a line after the request is waited for, on a socket.
///
/// Long enough for bytes a shell wrote in the same call as the request, which
/// are already here; short enough that a shell written before ADR 0136, which
/// sends nothing more, costs a person half a second.
pub const HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR: Duration = Duration::from_millis(500);

/// What the person's shell sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatThePersonSent {
    /// The request, in the person's own words.
    pub request: String,
    /// The folders the shell says this person granted.
    pub may_reach: WhatItMayReach,
    /// What was said before, for a follow-up; nothing for a first request.
    pub earlier: TheConversationSoFar,
}

/// **Read what the shell sent.**
///
/// `socket` is the connection, when the input is one: the third line is
/// waited for only there. [`None`] — a pipe, a terminal — reads two lines at
/// most, as this did before.
///
/// # Errors
/// A sentence, when there is no request, or when the line after it is not a
/// conversation a shell would send — **refused, never guessed at**, because a
/// conversation half-read would be shown to a model as though it were whole.
pub fn read_what_the_person_sent(
    input: &mut dyn BufRead,
    socket: Option<BorrowedFd<'_>>,
) -> Result<WhatThePersonSent, String> {
    let nothing_read = |why: std::io::Error| format!("nothing could be read from the input: {why}");
    let mut line = String::new();
    input.read_line(&mut line).map_err(nothing_read)?;
    let mut may_reach = WhatItMayReach::nothing();
    if let Some(folders) = line.trim_end().strip_prefix(THE_FOLDERS) {
        may_reach = WhatItMayReach::these(folders.split('\t').map(Path::new));
        line.clear();
        input.read_line(&mut line).map_err(nothing_read)?;
    }
    let request = line.trim().to_owned();
    if request.is_empty() {
        return Err(
            "there is no request to drive. Say what to do, or pipe one line in; `--help` shows \
             how"
            .to_owned(),
        );
    }
    let earlier = match socket {
        Some(socket) => the_earlier_conversation(input, socket)?,
        None => TheConversationSoFar::nothing(),
    };
    Ok(WhatThePersonSent {
        request,
        may_reach,
        earlier,
    })
}

/// The line after the request, waited for briefly, or nothing.
fn the_earlier_conversation(
    input: &mut dyn BufRead,
    socket: BorrowedFd<'_>,
) -> Result<TheConversationSoFar, String> {
    use rustix::net::sockopt::{Timeout, set_socket_timeout};

    if set_socket_timeout(
        socket,
        Timeout::Recv,
        Some(HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR),
    )
    .is_err()
    {
        // Not a socket after all: nobody could have sent a third line.
        return Ok(TheConversationSoFar::nothing());
    }
    let mut line = String::new();
    // **Read no further than a question could carry**, plus its newline: the
    // bound is checked while reading, not after a privileged process has
    // already held whatever was sent.
    let most = u64::try_from(alo_instructing::THE_LONGEST_LINE + 2).unwrap_or(u64::MAX);
    let read = std::io::Read::take(&mut *input, most).read_line(&mut line);
    // Back to waiting as long as it takes; `crate::holding_the_turn` sets its
    // own bound when it needs one.
    let _ = set_socket_timeout(socket, Timeout::Recv, None);
    match read {
        Ok(0) => Ok(TheConversationSoFar::nothing()),
        Ok(_) => TheConversationSoFar::read(&line).map_err(|why| why.to_string()),
        Err(why)
            if line.is_empty()
                && matches!(
                    why.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
        {
            Ok(TheConversationSoFar::nothing())
        }
        Err(why) => Err(format!(
            "the earlier conversation stopped arriving partway, so it was not read: {why}"
        )),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "what_the_person_sent_tests.rs"]
mod tests;
