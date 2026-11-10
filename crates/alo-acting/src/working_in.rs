//! The words on the compositor's door for **alo working in an application**,
//! so that what the application sends while it does is on the indicator
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q6, the owner's answers of 2026-11-10, and
//! [ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md),
//! whose count this extends to one copy of an application).
//!
//! Like a count, this holds its connection for as long as alo works, so a turn
//! that crashes cannot leave a line up, or a count running.
//!
//! ```text
//! → work-in-alos-copy org.mozilla.firefox      (the default setting)
//! ← counting           alo's copy is started or found, and only it is counted
//!   (or: cannot-count, no-copy-of-its-own, not-installed, could-not-start,
//!    cannot-show)
//!   … alo works in the copy …
//! → done
//! ← sent 52113         (or: stopped 52113 if the desktop ended it first,
//!                       or: uncounted when the answer was cannot-count)
//!
//! → work-in-their-window org.mozilla.firefox   (the setting turned up)
//! ← showing            a line is up for as long as alo works: nothing counted
//!   (or, at the default: not-chosen)
//! → done
//! ← uncounted
//! ```
//!
//! **Which of the two alo may ask for is the person's**, read by the door at
//! each ask from their own settings (`alo_capability::AloInApplications`),
//! never taken from the word alo sent. At the default `work-in-their-window`
//! is answered `not-chosen` and nothing is shown or started. Turned up,
//! `work-in-alos-copy` still uses alo's copy where the application has one —
//! that is counted, and the person chose *may*, not *must* — and the person's
//! own window where it has none, answered `showing` rather than
//! `no-copy-of-its-own`.
//!
//! **`cannot-count` is not a refusal.** alo's copy is running and alo may work
//! in it, and the line that is up says *this machine cannot count what it
//! sends* for as long as it does — never silence (law 1). **`cannot-show` is a
//! refusal**: a desktop with nowhere to draw a line cannot let alo work where
//! something might leave unseen.
//!
//! The person's own windows are **not counted at either setting**: at the
//! default only the units of alo's copy are, and at the turned-up setting
//! nothing is.

use alo_applications::Application;

use crate::request::Request;

/// The word that begins work in alo's own copy, at the default setting.
pub const IN_ALOS_COPY: &str = "work-in-alos-copy";

/// The word that begins work in the person's own window, at the setting turned
/// up.
pub const IN_THEIR_WINDOW: &str = "work-in-their-window";

/// Where alo works.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    /// In a copy of the application started for alo, counted on its own.
    InAlosCopy,
    /// In the person's own window: shown, never counted.
    InTheirWindow,
}

impl Where {
    /// The word on the wire.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::InAlosCopy => IN_ALOS_COPY,
            Self::InTheirWindow => IN_THEIR_WINDOW,
        }
    }
}

/// alo asking to work in one application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToWorkIn {
    /// The application, by its identifier.
    application: Application,
    /// Where.
    place: Where,
}

impl ToWorkIn {
    /// alo works in this application, here.
    #[must_use]
    pub const fn at(application: Application, place: Where) -> Self {
        Self { application, place }
    }

    /// The application.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// Where alo works in it.
    #[must_use]
    pub const fn place(&self) -> Where {
        self.place
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        format!("{} {}", self.place.written(), self.application.identifier())
    }

    /// One line, read — [`None`] when it is not alo asking to work at all, so
    /// the door can read it as one of the other acts.
    ///
    /// A line that begins with one of the two words and does not name exactly
    /// one application is `Some(Err(…))`: a caller with a bug.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotToWorkIn>> {
        let line = line.trim_end_matches(['\r', '\n']);
        let mut words = line.split(' ');
        let place = match words.next() {
            Some(IN_ALOS_COPY) => Where::InAlosCopy,
            Some(IN_THEIR_WINDOW) => Where::InTheirWindow,
            _ => return None,
        };
        if line.len() > Request::THE_MOST {
            return Some(Err(NotToWorkIn::TooLong));
        }
        let (Some(named), None) = (words.next(), words.next()) else {
            return Some(Err(NotToWorkIn::NotOneApplication));
        };
        Some(
            Application::identified(named)
                .map(|application| Self::at(application, place))
                .map_err(|_| NotToWorkIn::NotOneApplication),
        )
    }
}

/// Why a line that began as alo asking to work is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotToWorkIn {
    /// Longer than any line on this door may be.
    #[error("a line on this door is at most {} bytes", Request::THE_MOST)]
    TooLong,
    /// It named no application, more than one, or one no verb could name.
    #[error("alo works in exactly one application, named by its identifier")]
    NotOneApplication,
}

/// What the door answers at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Working {
    /// alo's copy is running, and only it is counted.
    Counting,
    /// alo's copy is running and this machine cannot count it, so the line
    /// says so for as long as alo works in it.
    CannotCount,
    /// alo works in the person's window, and the line says that the
    /// application may be sending.
    Showing,
    /// **Refusal A**: this application cannot open a separate copy for alo.
    NoCopyOfItsOwn,
    /// The application is not installed.
    NotInstalled,
    /// alo's copy could not be started.
    CouldNotStart,
    /// This desktop has nowhere to show a line, so alo may not work there.
    CannotShow,
    /// alo asked to work in the person's own window, and the person has not
    /// chosen to let it (`alo_capability::AloInApplications`): nothing was
    /// shown or started.
    NotChosen,
}

impl Working {
    /// Every answer, for a test that reads each back.
    pub const ALL: [Self; 8] = [
        Self::Counting,
        Self::CannotCount,
        Self::Showing,
        Self::NoCopyOfItsOwn,
        Self::NotInstalled,
        Self::CouldNotStart,
        Self::CannotShow,
        Self::NotChosen,
    ];

    /// The word on the wire.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::Counting => "counting",
            Self::CannotCount => "cannot-count",
            Self::Showing => "showing",
            Self::NoCopyOfItsOwn => "no-copy-of-its-own",
            Self::NotInstalled => "not-installed",
            Self::CouldNotStart => "could-not-start",
            Self::CannotShow => "cannot-show",
            Self::NotChosen => "not-chosen",
        }
    }

    /// One line, read; [`None`] for anything else, which a caller must never
    /// read as alo being allowed to work.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let line = line.trim_end_matches(['\r', '\n']);
        Self::ALL.into_iter().find(|one| one.written() == line)
    }

    /// Whether alo may work in the application now — a line is up, or will be
    /// from the first byte counted.
    #[must_use]
    pub const fn may_work(self) -> bool {
        matches!(self, Self::Counting | Self::CannotCount | Self::Showing)
    }
}

/// What the door answers when alo's work ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Worked {
    /// What alo's copy sent while alo worked in it.
    Sent(u64),
    /// The desktop ended it first, and this is what was counted until then.
    Stopped(u64),
    /// Nothing was counted: the person's window, or a copy this machine could
    /// not count. The line was up the whole time.
    Uncounted,
}

impl Worked {
    /// The line, without its newline.
    #[must_use]
    pub fn written(self) -> String {
        match self {
            Self::Sent(bytes) => format!("sent {bytes}"),
            Self::Stopped(bytes) => format!("stopped {bytes}"),
            Self::Uncounted => "uncounted".to_owned(),
        }
    }

    /// One line, read; [`None`] for anything else.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let line = line.trim_end_matches(['\r', '\n']);
        if line == "uncounted" {
            return Some(Self::Uncounted);
        }
        let (word, number) = line.split_once(' ')?;
        let bytes = number.parse().ok()?;
        match word {
            "sent" => Some(Self::Sent(bytes)),
            "stopped" => Some(Self::Stopped(bytes)),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "working_in_tests.rs"]
mod tests;
