//! **What a command alo ran did**, in the shape the record, the model and the
//! person are told it in.
//!
//! The crate that starts the program (`alo-running`, ADR 0117) makes one of
//! these, `alo-turn` writes it down, and `alo-protocol` carries it to the agent.
//! It lives here because all three already depend on this crate and none of
//! them should depend on each other for it.

/// What a command did once it was running.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatACommandDid {
    /// Its exit status, and [`None`] when a signal ended it — a program killed
    /// by a signal has no exit status, and any number put here would be one
    /// nothing said.
    returned: Option<i32>,
    /// What it wrote, standard output and standard error together in the order
    /// they arrived, as text.
    said: String,
    /// Whether it wrote more than was kept.
    cut_short: bool,
    /// Whether it was stopped because it ran longer than a command is let run.
    took_too_long: bool,
    /// How many bytes it sent off this machine while it ran, as the kernel
    /// counted them — [`None`] where nothing counted them (ADR 0148, as
    /// amended 2026-11-13).
    ///
    /// Absent rather than zero where there was no count, because *it sent
    /// nothing* and *nobody could tell* are two answers a person would act on
    /// differently.
    sent: Option<u64>,
}

impl WhatACommandDid {
    /// What a command did, as the crate that ran it saw it.
    #[must_use]
    pub fn of(returned: Option<i32>, said: String, cut_short: bool, took_too_long: bool) -> Self {
        Self {
            returned,
            said,
            cut_short,
            took_too_long,
            sent: None,
        }
    }

    /// This, with what the kernel counted it sending — [`None`] where nothing
    /// could count it.
    #[must_use]
    pub const fn having_sent(mut self, sent: Option<u64>) -> Self {
        self.sent = sent;
        self
    }

    /// How many bytes it sent off this machine while it ran, or [`None`] where
    /// nothing counted them.
    #[must_use]
    pub const fn sent(&self) -> Option<u64> {
        self.sent
    }

    /// Its exit status, and [`None`] when a signal ended it.
    #[must_use]
    pub fn returned(&self) -> Option<i32> {
        self.returned
    }

    /// What it wrote.
    #[must_use]
    pub fn said(&self) -> &str {
        &self.said
    }

    /// Whether it wrote more than was kept.
    #[must_use]
    pub fn cut_short(&self) -> bool {
        self.cut_short
    }

    /// Whether it was stopped for running too long.
    #[must_use]
    pub fn took_too_long(&self) -> bool {
        self.took_too_long
    }
}

/// **Why a command that ran did not work**, or nothing when it did.
///
/// A command that ran is not a command that did what was asked: `mkdir` of a
/// folder whose parent is not there runs, writes one line and ends with
/// status 1. Until 2026-11-10 every surface answered a run as *done* whatever it ended
/// with, and a booted machine told a person *Done without asking you first …
/// mkdir /Users/username/Documents/demo* about a folder that was never made.
/// So *did it work* is decided here, once, from what the run itself said — and
/// every surface that tells a person about a run asks this rather than its own
/// opinion of an exit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItDidNotWork {
    /// It ended by itself, with a status other than 0.
    EndedWith(i32),
    /// A signal ended it, so it has no status at all.
    WasStopped,
    /// It ran longer than a command is let run, and was stopped.
    TookTooLong,
    /// It ended with a failure, and what it said is that nobody is signed in
    /// to GitHub (`crate::not_signed_in_to_github`) — so the person is told
    /// that, and how to sign in, rather than a number.
    NotSignedInToGitHub,
}

impl ItDidNotWork {
    /// Why a command that ended this way did not work, or [`None`] for one
    /// that ended by itself with status 0 in time — the only end that is
    /// *done*.
    ///
    /// Running too long is asked first: a command stopped for it was ended by
    /// a signal too, and the person is owed the reason rather than the means.
    #[must_use]
    pub const fn of(returned: Option<i32>, took_too_long: bool) -> Option<Self> {
        if took_too_long {
            return Some(Self::TookTooLong);
        }
        match returned {
            Some(0) => None,
            Some(status) => Some(Self::EndedWith(status)),
            None => Some(Self::WasStopped),
        }
    }

    /// [`Self::of`], and — for a command that ended by itself with a failure —
    /// [`Self::NotSignedInToGitHub`] where what it `said` is that nobody is
    /// signed in to GitHub. **Whether it worked is still [`Self::of`]'s
    /// answer**: what a command said never turns a failure into *done*, nor
    /// *done* into a failure.
    #[must_use]
    pub fn of_what_it_said(returned: Option<i32>, took_too_long: bool, said: &str) -> Option<Self> {
        match Self::of(returned, took_too_long)? {
            Self::EndedWith(_) if crate::not_signed_in_to_github::not_signed_in_to_github(said) => {
                Some(Self::NotSignedInToGitHub)
            }
            why => Some(why),
        }
    }

    /// Why, as the person reads it: one clause, to go after *it did not work:*.
    #[must_use]
    pub fn said(self, strings: &alo_strings::Strings) -> alo_strings::Said {
        match self {
            Self::EndedWith(status) => strings.say(
                &crate::words::A_COMMAND_ENDED_WITH_A_STATUS.key(),
                &alo_strings::Filling::of("status", status.to_string()),
            ),
            Self::WasStopped => strings.say(
                &crate::words::A_COMMAND_WAS_STOPPED.key(),
                &alo_strings::Filling::nothing(),
            ),
            Self::TookTooLong => strings.say(
                &crate::words::A_COMMAND_RAN_TOO_LONG.key(),
                &alo_strings::Filling::nothing(),
            ),
            Self::NotSignedInToGitHub => strings.say(
                &crate::words::NOT_SIGNED_IN_TO_GITHUB.key(),
                &alo_strings::Filling::nothing(),
            ),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod it_did_not_work {
    use super::ItDidNotWork;

    /// **Only status 0, in time, is done.** Every other end is a reason,
    /// and running too long is named even though a signal ended it.
    #[test]
    fn only_an_end_with_status_zero_in_time_worked() {
        assert_eq!(ItDidNotWork::of(Some(0), false), None);
        assert_eq!(
            ItDidNotWork::of(Some(1), false),
            Some(ItDidNotWork::EndedWith(1))
        );
        assert_eq!(
            ItDidNotWork::of(Some(-3), false),
            Some(ItDidNotWork::EndedWith(-3))
        );
        assert_eq!(
            ItDidNotWork::of(None, false),
            Some(ItDidNotWork::WasStopped)
        );
        assert_eq!(
            ItDidNotWork::of(None, true),
            Some(ItDidNotWork::TookTooLong)
        );
        assert_eq!(
            ItDidNotWork::of(Some(0), true),
            Some(ItDidNotWork::TookTooLong)
        );
    }

    /// **What a command said names the reason and never decides whether it
    /// worked**: *not signed in* is said only for a failure, and a run that
    /// worked, was stopped or ran too long keeps its own answer whatever it
    /// printed.
    #[test]
    fn not_signed_in_is_said_only_for_a_failure_that_says_so() {
        let nobody = "fatal: could not read Username for 'https://github.com': terminal prompts \
                      disabled";
        assert_eq!(
            ItDidNotWork::of_what_it_said(Some(128), false, nobody),
            Some(ItDidNotWork::NotSignedInToGitHub)
        );
        assert_eq!(ItDidNotWork::of_what_it_said(Some(0), false, nobody), None);
        assert_eq!(
            ItDidNotWork::of_what_it_said(None, false, nobody),
            Some(ItDidNotWork::WasStopped)
        );
        assert_eq!(
            ItDidNotWork::of_what_it_said(Some(1), true, nobody),
            Some(ItDidNotWork::TookTooLong)
        );
        assert_eq!(
            ItDidNotWork::of_what_it_said(Some(1), false, "mkdir: cannot create directory"),
            Some(ItDidNotWork::EndedWith(1))
        );
    }

    /// **The person is told how to sign in, and what that sign-in can do** —
    /// the command to type, and that alo can use it at full access.
    #[test]
    fn not_signed_in_says_how_to_sign_in_and_what_it_lets_alo_do() {
        let strings = alo_strings::Strings::of(crate::words::capability_words().unwrap());
        let said = ItDidNotWork::NotSignedInToGitHub.said(&strings);
        for part in [
            "gh auth login --web",
            "full access",
            "gh auth logout",
            "every repository",
        ] {
            assert!(said.text().contains(part), "{part:?} is not in {said:?}");
        }
    }
}

/// A command that never started — the shell would not start, or the machine
/// lost track of it before it could say anything.
///
/// The English is for whoever administers the machine; the person is told
/// [`crate::words::NOTHING_RAN_WOULD_NOT_START`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItDidNotStart(String);

impl ItDidNotStart {
    /// Why, in English, for the service log.
    #[must_use]
    pub fn because(why: impl Into<String>) -> Self {
        Self(why.into())
    }

    /// Why, in English.
    #[must_use]
    pub fn why(&self) -> &str {
        &self.0
    }
}
