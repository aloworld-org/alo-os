//! **What a person's shell sent**, read off the connection `alo-asking.socket`
//! hands this process (ADR 0109; `docs/contracts/asking-socket.md`).
//!
//! Up to four lines, each with its own reason to be there:
//!
//! 1. `alo-may-reach:` and the folders this person granted, tab-separated —
//!    only when there are any. A hint for the model, never a capability.
//! 2. The request, in the person's own words.
//! 3. `alo-earlier:` and the conversation so far, when the request is a
//!    follow-up typed in a task window
//!    ([ADR 0138](../../../docs/decisions/0138-a-follow-up-is-sent-with-the-conversation-so-far.md)).
//!    A current shell always sends it, with nothing after the marker for a
//!    first request.
//! 4. `alo-this-window:` and the window the person offered with *Help with
//!    this window* (`alo_instructing::TheWindowOffered`). A current shell
//!    always sends it, with nothing after the marker for a request about no
//!    window. A hint for the model, never a capability, as the folders are.
//! 5. `alo-their-folders:` and where the person's home and standard folders
//!    are on this computer (`alo_instructing::ThePersonsFolders`), read on
//!    their side as they sent the request. A current shell always sends it,
//!    with nothing after the marker where it knows no home. It travels inside
//!    [`WhatThePersonSent::may_reach`], since it is told to the model with
//!    the folders and, like them, permits nothing.
//!
//! # Why the later lines are after the request, and waited for briefly
//!
//! **An older agent must keep working.** One reads the first line, and the
//! second if the first named folders, and never reads again — so a line after
//! the request is a line it ignores, and it answers the follow-up as though it
//! were a first request. Anywhere before the request it would have been read
//! *as* the request. The fourth is after the third for the same reason: an
//! agent written before it reads three lines and never a fourth, and the stop
//! watcher reads and drops anything left unread (`crate::the_person_may_stop`).
//!
//! The cost is on this side. A shell written before this sends no third line
//! and does not close its end — the end stays open while a change waits
//! (`crate::holding_the_turn`) — so reading for one could wait for ever. Each
//! is waited for for [`HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR`] and no longer,
//! and only on a socket: a current shell writes every line at once, so the
//! wait ends as soon as it starts. The fourth is waited for only after a third
//! arrived, so a shell that sent neither waits once, not twice.

use std::io::BufRead;
use std::os::fd::BorrowedFd;
use std::path::Path;
use std::time::Duration;

use alo_instructing::{TheConversationSoFar, ThePersonsFolders, TheWindowOffered, WhatItMayReach};

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
/// are already here; short enough that a shell written before ADR 0138, which
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
    /// The window the person offered with the request, if they offered one
    /// (*Help with this window*). A hint for the model, never a capability.
    pub window: Option<TheWindowOffered>,
    /// Whether the person can still stop this turn: the shell kept its
    /// sending side open after the request. A caller that shut it as soon as it
    /// had written — `alo-ask` does — has said *that was all*, and that is
    /// never read as a Stop (`crate::the_person_may_stop`; ADR 0139).
    pub may_stop: bool,
}

/// **Read what the shell sent.**
///
/// `socket` is the connection, when the input is one: the third and fourth lines are
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
    let (earlier, window, home, may_stop) = match socket {
        Some(socket) => the_lines_after_the_request(input, socket)?,
        None => (TheConversationSoFar::nothing(), None, None, false),
    };
    Ok(WhatThePersonSent {
        request,
        may_reach: may_reach.in_this_home(home),
        earlier,
        window,
        may_stop,
    })
}

/// What reading one line after the request found.
enum ALine {
    /// The line, with its newline.
    Read(String),
    /// The end of the input: the caller has finished writing.
    Ended,
    /// Nothing within the wait: a caller from before that line, still open.
    NotSent,
}

/// One line, read no further than `most` bytes, waited for as long as the
/// socket's timeout says.
fn a_line(input: &mut dyn BufRead, most: usize, what: &str) -> Result<ALine, String> {
    let mut line = String::new();
    // **Read no further than the line could be**, plus its newline: the bound
    // is checked while reading, not after a privileged process has already
    // held whatever was sent.
    let most = u64::try_from(most.saturating_add(2)).unwrap_or(u64::MAX);
    match std::io::Read::take(&mut *input, most).read_line(&mut line) {
        Ok(0) => Ok(ALine::Ended),
        Ok(_) => Ok(ALine::Read(line)),
        Err(why)
            if line.is_empty()
                && matches!(
                    why.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
        {
            Ok(ALine::NotSent)
        }
        Err(why) => Err(format!(
            "{what} stopped arriving partway, so it was not read: {why}"
        )),
    }
}

/// What the lines after the request carried: the conversation so far, the
/// window offered, where the person's folders are, and whether the caller
/// left its sending side open.
type AfterTheRequest = (
    TheConversationSoFar,
    Option<TheWindowOffered>,
    Option<ThePersonsFolders>,
    bool,
);

/// The lines after the request, each waited for briefly, or nothing — and
/// whether the caller left its sending side open.
fn the_lines_after_the_request(
    input: &mut dyn BufRead,
    socket: BorrowedFd<'_>,
) -> Result<AfterTheRequest, String> {
    use rustix::net::sockopt::{Timeout, set_socket_timeout};

    if set_socket_timeout(
        socket,
        Timeout::Recv,
        Some(HOW_LONG_THE_THIRD_LINE_IS_WAITED_FOR),
    )
    .is_err()
    {
        // Not a socket after all: nobody could have sent a third line.
        return Ok((TheConversationSoFar::nothing(), None, None, false));
    }
    let read = the_third_and_fourth(input);
    // Back to waiting as long as it takes; `crate::holding_the_turn` sets its
    // own bound when it needs one.
    let _ = set_socket_timeout(socket, Timeout::Recv, None);
    read
}

/// The conversation so far, then — only after it — the window offered, and
/// only after that, where the person's folders are.
fn the_third_and_fourth(input: &mut dyn BufRead) -> Result<AfterTheRequest, String> {
    let third = a_line(
        input,
        alo_instructing::THE_LONGEST_LINE,
        "the earlier conversation",
    )?;
    let earlier = match third {
        // **It has finished writing**: the end of the input came where a
        // third line could have.
        ALine::Ended => return Ok((TheConversationSoFar::nothing(), None, None, false)),
        ALine::NotSent => return Ok((TheConversationSoFar::nothing(), None, None, true)),
        ALine::Read(line) => TheConversationSoFar::read(&line).map_err(|why| why.to_string())?,
    };
    let window = match a_line(
        input,
        alo_instructing::THE_LONGEST_WINDOW_LINE,
        "the window offered",
    )? {
        ALine::Read(line) => {
            TheWindowOffered::read_the_line(&line).map_err(|why| why.to_string())?
        }
        // A shell from before the fourth line: a request about no window, and
        // nothing after it to wait for.
        ALine::Ended | ALine::NotSent => return Ok((earlier, None, None, true)),
    };
    let home = match a_line(
        input,
        alo_instructing::THE_LONGEST_FOLDERS_LINE,
        "the person's folders",
    )? {
        ALine::Read(line) => {
            ThePersonsFolders::read_the_line(&line).map_err(|why| why.to_string())?
        }
        // A shell from before the fifth line: no home was said.
        ALine::Ended | ALine::NotSent => None,
    };
    Ok((earlier, window, home, true))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "what_the_person_sent_tests.rs"]
mod tests;
