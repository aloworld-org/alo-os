//! **The desktops told when a command alo ran needs a GitHub sign-in.**
//!
//! The owner approved the GitHub sign-in sheet on 2026-11-13 *as drawn*, and
//! it *appears when alo can't publish because you're not signed in*
//! (`docs/design/github-sign-in.md`). This service runs every command alo
//! runs, and `alo_capability::ItDidNotWork::of_what_it_said` is where *nobody
//! is signed in to GitHub* is recognised. Until this file the recognition
//! became a sentence in the command's answer and nothing more, so the desktop
//! — which draws the sheet — never heard of it.
//!
//! A desktop asks `watch-what-needs-a-github-sign-in` on the person's door.
//! That connection is answered `watching-what-needs-a-github-sign-in`, given
//! over to this list, and answers nothing else — `crate::watchers`' road, and
//! for its reason: the person's door is free again for Settings.
//!
//! # Once per failure, and only for a run on this machine
//!
//! Each command that ended with a failure and said nobody is signed in is
//! told **once**, as `a-github-sign-in-is-needed`, which carries nothing. A
//! command that worked, one that failed for any other reason, and one that
//! was stopped are told nothing.
//!
//! **Only a command this machine ran is told**, and only through the person's
//! own level: every road a paired machine reaches this one by runs commands
//! as `crate::running_a_command::CommandsHere::where_nothing_was_chosen`,
//! which tells nobody — and nothing that crosses from a paired machine is
//! told here at all. Signing in here would not help a command that ran on
//! another machine, under another person's gh.
//!
//! # A watcher never holds a command up
//!
//! Each write is non-blocking, and a watcher that cannot take the line at
//! once is dropped rather than waited for — `crate::watchers`' rule. What the
//! command answered is told to the agent and the person either way; the
//! sheet is a convenience on top of the sentence, never instead of it.

use std::io::Write as _;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

use alo_capability::ItDidNotWork;
use alo_protocol::{Done, ToAPerson};

/// How many desktops are told at once — `crate::watchers::AT_MOST`'s number,
/// for its reason. **The oldest goes** when another arrives.
pub const AT_MOST: usize = crate::watchers::AT_MOST;

/// The desktops told when a command needs a GitHub sign-in.
///
/// Shared between the serving loop, which hands it connections, and every
/// turn's commands, which tell it. Cloning shares the one list.
#[derive(Debug, Clone, Default)]
pub struct TheDesktopsToldGitHubNeedsASignIn(Arc<Mutex<Vec<UnixStream>>>);

impl PartialEq for TheDesktopsToldGitHubNeedsASignIn {
    /// The same list, and never two lists that happen to hold as many: what
    /// is compared is who would be told.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TheDesktopsToldGitHubNeedsASignIn {}

impl TheDesktopsToldGitHubNeedsASignIn {
    /// Nobody watching yet.
    #[must_use]
    pub fn none_yet() -> Self {
        Self::default()
    }

    /// What a desktop that asked is answered with, before its connection is
    /// given over.
    #[must_use]
    pub const fn answered() -> ToAPerson {
        ToAPerson::WatchingWhatNeedsAGitHubSignIn
    }

    /// Tell this connection each time a command needs a sign-in, from now on.
    ///
    /// A connection that cannot be made non-blocking is dropped: a blocking
    /// one could hold a command's answer up.
    pub fn watching(&self, connection: UnixStream) {
        if connection.set_nonblocking(true).is_err() {
            return;
        }
        if let Ok(mut listening) = self.0.lock() {
            listening.push(connection);
            while listening.len() > AT_MOST {
                listening.remove(0);
            }
        }
    }

    /// How many desktops are being told.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.0.lock().map(|listening| listening.len()).unwrap_or(0)
    }

    /// **What a command did, looked at once**: where it failed because nobody
    /// is signed in to GitHub, every desktop watching is told so, once.
    /// Anything else is told to nobody.
    ///
    /// Takes the answer as it goes on the wire, so the desktop is told exactly
    /// when the agent and the person are told *you are not signed in to
    /// GitHub*, and never on a guess of its own.
    pub fn told_of(&self, done: &Done) {
        if needs_a_github_sign_in(done) {
            self.a_sign_in_is_needed();
        }
    }

    /// **What a refused change was, looked at once**: a publication refused
    /// because nobody is signed in to GitHub tells every desktop watching,
    /// once. Any other refusal is told to nobody.
    pub fn told_of_a_refusal(&self, why: &alo_turn::NotDone) {
        if a_refusal_needs_a_github_sign_in(why) {
            self.a_sign_in_is_needed();
        }
    }

    /// Tell every desktop watching that a command needs a GitHub sign-in.
    ///
    /// Public for an errand whose failure is not a command's answer: a
    /// publication to GitHub Pages refused because nobody is signed in is told
    /// here directly, once, by whatever refused it.
    pub fn a_sign_in_is_needed(&self) {
        let Ok(mut written) = ToAPerson::AGitHubSignInIsNeeded.written() else {
            return;
        };
        written.push('\n');
        if let Ok(mut listening) = self.0.lock() {
            listening.retain_mut(|connection| connection.write_all(written.as_bytes()).is_ok());
        }
    }
}

/// Whether this answer is a command that failed because nobody is signed in
/// to GitHub — `alo_capability::ItDidNotWork::of_what_it_said`'s answer, the
/// one every surface asks.
#[must_use]
pub fn needs_a_github_sign_in(done: &Done) -> bool {
    match done {
        Done::RanACommand {
            returned,
            said,
            took_too_long,
            ..
        } => {
            ItDidNotWork::of_what_it_said(*returned, *took_too_long, said)
                == Some(ItDidNotWork::NotSignedInToGitHub)
        }
        _ => false,
    }
}

/// Whether this refusal is a publication refused because nobody is signed in
/// to GitHub (`alo_publishing::NotPublished::NotSignedIn`) — which comes back
/// as a refusal, not as a command's answer, so [`needs_a_github_sign_in`]
/// never sees it.
#[must_use]
pub fn a_refusal_needs_a_github_sign_in(why: &alo_turn::NotDone) -> bool {
    matches!(
        why,
        alo_turn::NotDone::CouldNotPublish(alo_publishing::NotPublished::NotSignedIn)
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_desktops_told_github_needs_a_sign_in_tests.rs"]
mod tests;
