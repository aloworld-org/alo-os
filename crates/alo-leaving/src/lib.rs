//! Leaving a machine: logging out, switching user, and what was open when you
//! left.
//!
//! Three doors and one file. Logging out asks every application to close in an
//! order that lets each save, names any that would not, and kills nothing.
//! Switching user locks this session and hands the screen to the sign-in.
//! Locking is `alo-locking`'s and is not repeated here. And what was open — a
//! list of applications and where each was, **never what was in them** — is kept
//! in the person's own folder, written once as they leave, and read back only if
//! they asked for it.
//!
//! | | |
//! |---|---|
//! | [`logging_out::asked`] | Every application asked to close, last opened first |
//! | [`LoggingOut`], [`WouldNotClose`] | What came of it, and the names of whatever stayed |
//! | [`MayEnd`], [`AfterWhat`] | The one value that says a session may end, and how it got there |
//! | [`switching::asked`] | This session locked, and the screen handed to the sign-in |
//! | [`Open`], [`WasOpen`], [`Split`] | What was open: an application, a screen, a split — and no fourth thing |
//! | [`keeping`] | `leaving.toml`: the one setting, and the list, kept by [`alo_kept`]'s rule |
//! | [`restoring::at_sign_in`] | What is reopened, which is nothing unless the person asked |
//! | [`words`] | The twelve sentences this crate says |
//!
//! # The rules, one clause each
//!
//! 1. **Logging out ends applications in an order that lets each save.** One at
//!    a time, last opened first, each answered before the next is asked
//!    ([`WasOpen::asked_to_close_in_order`]).
//! 2. **Any that refused to close are named** ([`WouldNotClose::said`]), one
//!    sentence each, by the identifier this machine knows them by.
//! 3. **Nothing is ever killed silently.** Nothing in this crate signals or
//!    terminates anything at all; the only road past an application that stayed
//!    is [`WouldNotClose::even_so`], which cannot be reached without the list of
//!    names in hand.
//! 4. **Switch user locks this session first** and only then hands the screen to
//!    the sign-in ([`switching::asked`]). It is the same session afterwards:
//!    nothing is signed out and nothing is closed.
//! 5. **What was open is a list of applications and which display and split each
//!    was on** — never a document's contents, a window's title or a URL
//!    ([`Open`] has three fields and there is no fourth).
//! 6. **It is kept in the person's folder** by this crate at a path it is handed
//!    (ADR 0038), written at one moment — a log-out — and only for a person who
//!    asked for it ([`keeping::at_sign_out`]).
//! 7. **It is restored at the next sign-in only if the person chose it**
//!    ([`restoring::at_sign_in`]), and what alo OS ships is not to.
//! 8. **The agent never reads this list.** No crate an agent's request is
//!    answered in depends on this one, and nothing in `alo-agentd` reads
//!    `leaving.toml` — `tests/the_agent_never_reads_what_was_open.rs` reads the
//!    workspace's manifests and the daemon's own source for it. *What were you
//!    doing yesterday* is not context an agent is offered: ADR 0001's context
//!    arrives at the moment of invocation and is never harvested.
//!
//! # Nothing here draws, and nothing here ends a session
//!
//! The dialogue that shows what would not close is the shell's; so is the
//! greeter the screen is handed to. And the session itself ends where it was
//! started — this crate hands over a [`MayEnd`] and has no road to `logind`, no
//! signal and no process. What it fixes is the order, which is the part that is
//! wrong on every system that gets this wrong.
//!
//! # What an application does with its own documents is its own
//!
//! No application's session files are read or written here. An editor that
//! reopens the document it had open does that itself, under its own grants —
//! which is also why this crate can keep a list with no titles in it and still
//! keep the promise a person cares about.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod changes;
mod ending;
pub mod keeping;
pub mod logging_out;
mod open;
mod refusing;
pub mod restoring;
mod split;
pub mod switching;
mod unkept;
pub mod words;

#[cfg(test)]
mod testing;

pub use changes::{Changes, Setting, Settings};
pub use ending::{AfterWhat, MayEnd};
pub use logging_out::{Closing, LoggingOut, TheApplications};
pub use open::{Open, WasOpen};
pub use refusing::WouldNotClose;
pub use restoring::{AtSignIn, Restoring};
pub use split::Split;
pub use switching::Switching;
pub use unkept::{FileNotRead, FileNotWritten};
pub use words::{EVERY_WORD, Word, WordsError, declare_into, leaving_words};
