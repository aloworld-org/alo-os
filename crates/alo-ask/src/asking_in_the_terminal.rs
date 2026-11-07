//! **"Run this command? [y/N]"** — the person's OK, asked for in the terminal
//! they typed the request in.
//!
//! [ADR 0124](../../../docs/decisions/0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md)
//! and `docs/design/the-terminal-window.md` draw the command proposing, then
//! asking *Run this command? [y/N]*. Until 2026-11-07 nothing asked: the change
//! lapsed the moment the reply was read, and a person who had not switched on
//! acting without asking could not approve anything from a terminal at all.
//!
//! # One key, and only `y` is yes
//!
//! The terminal is put into a mode where one key is read as it is pressed, with
//! nothing echoed and **Ctrl-C arriving as a key rather than as a signal**, so
//! that Ctrl-C is a *no* the record keeps rather than this process dying with
//! the question unanswered. `y` or `Y` is yes. Everything else — `n`, Enter,
//! Ctrl-C, Ctrl-D, any other key — is no, which is what the capital `N` in
//! `[y/N]` promises. The terminal is put back as it was before anything else
//! happens, whichever way the answer went.
//!
//! **Two ways of not answering, and neither is a yes or a no.** The question
//! stands only as long as the service holds it, so the wait is bounded by that;
//! and a terminal that closes has nobody left at it. In both, nothing is sent,
//! and the change lapses with the turn.

use std::io::Write;
use std::os::fd::BorrowedFd;
use std::time::Duration;

use rustix::event::{PollFd, PollFlags, Timespec};
use rustix::termios::{LocalModes, OptionalActions, SpecialCodeIndex};

/// What the person answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheirAnswer {
    /// They pressed `y`.
    Yes,
    /// They pressed anything else: `n`, Enter, Ctrl-C, Ctrl-D.
    No,
    /// They pressed nothing while the question stood.
    NoAnswerInTime,
    /// The terminal went away, or would not be read.
    TheTerminalWentAway,
}

/// What one key, or no key at all, means.
#[must_use]
pub const fn the_answer_in(key: Option<u8>) -> TheirAnswer {
    match key {
        Some(b'y' | b'Y') => TheirAnswer::Yes,
        Some(_) => TheirAnswer::No,
        None => TheirAnswer::TheTerminalWentAway,
    }
}

/// Ask at `terminal`, writing `prompt` to `told`, and wait at most `within`.
///
/// `terminal` is the person's terminal — standard input on a real machine, one
/// end of a pseudo-terminal in a test.
pub fn asked_at(
    terminal: BorrowedFd<'_>,
    prompt: &str,
    told: &mut dyn Write,
    within: Duration,
) -> TheirAnswer {
    let Ok(as_it_was) = rustix::termios::tcgetattr(terminal) else {
        return TheirAnswer::TheTerminalWentAway;
    };
    let mut one_key = as_it_was.clone();
    one_key
        .local_modes
        .remove(LocalModes::ICANON | LocalModes::ECHO | LocalModes::ISIG);
    one_key.special_codes[SpecialCodeIndex::VMIN] = 1;
    one_key.special_codes[SpecialCodeIndex::VTIME] = 0;
    if rustix::termios::tcsetattr(terminal, OptionalActions::Now, &one_key).is_err() {
        return TheirAnswer::TheTerminalWentAway;
    }
    let _ = write!(told, "{prompt}");
    let _ = told.flush();
    let answer = one_key_within(terminal, within);
    // **Put back before anything else**, so a person's shell never inherits a
    // terminal that reads one key at a time.
    let _ = rustix::termios::tcsetattr(terminal, OptionalActions::Now, &as_it_was);
    let echoed = match answer {
        TheirAnswer::Yes => "y",
        TheirAnswer::No => "N",
        TheirAnswer::NoAnswerInTime | TheirAnswer::TheTerminalWentAway => "",
    };
    let _ = writeln!(told, "{echoed}");
    answer
}

/// One key, read within `within`, and what it means.
fn one_key_within(terminal: BorrowedFd<'_>, within: Duration) -> TheirAnswer {
    let mut polling = [PollFd::new(&terminal, PollFlags::IN)];
    let timeout = Timespec::try_from(within).ok();
    match rustix::event::poll(&mut polling, timeout.as_ref()) {
        Ok(0) => return TheirAnswer::NoAnswerInTime,
        Ok(_) => {}
        Err(_) => return TheirAnswer::TheTerminalWentAway,
    }
    let mut key = [0_u8; 1];
    match rustix::io::read(terminal, &mut key) {
        Ok(1) => the_answer_in(key.first().copied()),
        _ => the_answer_in(None),
    }
}

#[cfg(test)]
#[path = "asking_in_the_terminal_tests.rs"]
mod tests;
