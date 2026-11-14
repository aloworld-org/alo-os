//! **Hearing from `alo-agentd` that a command alo ran needs a GitHub
//! sign-in**, so the shell can put the sign-in sheet up (the owner's approval
//! of 2026-11-13: the sheet *appears when alo can't publish because you're not
//! signed in*).
//!
//! It connects to the person's own door through the same two checks the
//! indicator's listener makes (`alo_indicator::watching::connected_to_the_persons_service`),
//! asks `watch-what-needs-a-github-sign-in`, and hands over one knock for each
//! `a-github-sign-in-is-needed` it is told. A knock carries nothing: the
//! desktop does not learn which command failed, or what it said — the person
//! read that already, in alo's own answer.
//!
//! # What it never does
//!
//! - **It never opens the sheet itself.** The shell decides, once a frame,
//!   whether a sheet may go up (`alo_shell`'s `the_github_sheet_rises_by_itself`).
//! - **It never retries the command.** Signing in is the person's; asking alo
//!   again is the person's too. A command run again after a sign-in would be a
//!   second run nobody approved.
//! - **It never blocks a frame**: it runs on its own thread and hands its
//!   knocks over a channel the desktop drains once a frame.
//!
//! A service older than this request refuses it; that is said once on the
//! journal and asked again only after the longest wait, so an older service
//! is never knocked on once a second.

use std::io::{BufRead as _, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Duration;

use alo_indicator::watching::{StoppedListening, connected_to_the_persons_service};
use alo_protocol::{FromAPerson, ToAPerson};

/// How long to wait before asking again, at first.
const FIRST_WAIT: Duration = Duration::from_secs(1);
/// The longest wait between two attempts.
const LONGEST_WAIT: Duration = Duration::from_secs(10);

/// One knock: a command needs a GitHub sign-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AGitHubSignInIsNeeded;

/// Why one attempt at hearing ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoppedHearing {
    /// The door could not be reached, or failed a check
    /// (`alo_indicator::watching::StoppedListening`).
    NotReached(StoppedListening),
    /// The service answered something other than that this desktop is
    /// watching — a refusal from a service older than the request.
    NotOffered,
    /// The service ended the connection, or said something that is not a
    /// knock.
    Ended,
    /// The desktop is no longer listening for knocks.
    NobodyToTell,
}

/// Hear once, at the door at `door`, for `person`: every knock is handed to
/// `tell`, and a `tell` that answers `false` stops it. Answers why it stopped.
pub fn heard_once(
    door: &Path,
    person: u32,
    tell: &mut dyn FnMut(AGitHubSignInIsNeeded) -> bool,
) -> StoppedHearing {
    let mut connection = match connected_to_the_persons_service(door, person) {
        Ok(connection) => connection,
        Err(why) => return StoppedHearing::NotReached(why),
    };
    let Ok(mut asked) = FromAPerson::WatchWhatNeedsAGitHubSignIn.written() else {
        return StoppedHearing::Ended;
    };
    asked.push('\n');
    if connection.write_all(asked.as_bytes()).is_err() {
        return StoppedHearing::Ended;
    }
    let mut lines = BufReader::new(connection);
    let mut first = String::new();
    match lines.read_line(&mut first) {
        Ok(0) | Err(_) => return StoppedHearing::Ended,
        Ok(_) => {}
    }
    if ToAPerson::read(&first) != Ok(ToAPerson::WatchingWhatNeedsAGitHubSignIn) {
        return StoppedHearing::NotOffered;
    }
    loop {
        let mut line = String::new();
        match lines.read_line(&mut line) {
            Ok(0) | Err(_) => return StoppedHearing::Ended,
            Ok(_) => {}
        }
        if ToAPerson::read(&line) != Ok(ToAPerson::AGitHubSignInIsNeeded) {
            return StoppedHearing::Ended;
        }
        if !tell(AGitHubSignInIsNeeded) {
            return StoppedHearing::NobodyToTell;
        }
    }
}

/// Keep hearing for as long as the desktop holds the other end of `send`,
/// asking again after a wait that grows from one second to ten — and after the
/// longest wait for a service that does not offer it.
pub fn kept_hearing(door: PathBuf, person: u32, send: &Sender<AGitHubSignInIsNeeded>) {
    let mut wait = FIRST_WAIT;
    let mut said_not_offered = false;
    loop {
        let mut heard_anything = false;
        let stopped = heard_once(&door, person, &mut |knock| {
            heard_anything = true;
            send.send(knock).is_ok()
        });
        match stopped {
            StoppedHearing::NobodyToTell => return,
            StoppedHearing::NotOffered => {
                if !said_not_offered {
                    eprintln!(
                        "alo-desktop: alo-agentd does not tell this desktop when a command needs \
                         a GitHub sign-in, so the sign-in sheet opens only from Settings → \
                         Accounts"
                    );
                    said_not_offered = true;
                }
                wait = LONGEST_WAIT;
            }
            StoppedHearing::NotReached(why) if why != StoppedListening::NotThere => {
                eprintln!(
                    "alo-desktop: alo-agentd's door failed a check, so nothing tells this \
                     desktop when a command needs a GitHub sign-in: {why:?}"
                );
                wait = LONGEST_WAIT;
            }
            StoppedHearing::NotReached(_) | StoppedHearing::Ended => {
                wait = if heard_anything {
                    FIRST_WAIT
                } else {
                    (wait * 2).min(LONGEST_WAIT)
                };
            }
        }
        std::thread::sleep(wait);
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "hearing_that_github_needs_a_sign_in_tests.rs"]
mod tests;
