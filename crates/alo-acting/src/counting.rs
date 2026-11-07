//! The words on the compositor's door for **counting what an application sends
//! while a person's command runs**
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)).
//!
//! Every other act on this door is one line in and one line out. This one holds
//! its connection for as long as the command runs, because the count is *for as
//! long as the command runs* and a connection is the one thing a command cannot
//! forget to end: it ends when the command does, crashed or not.
//!
//! ```text
//! → count org.mozilla.Thunderbird
//! ← counting                     (or: not-installed, cannot-count)
//!   … the command does its work …
//! → done
//! ← sent 52113                   (or, if the desktop ended it first: stopped 52113)
//! ```
//!
//! The number is bytes, counted at the socket by the kernel, headers included.

use alo_applications::Application;

use crate::request::Request;

/// The word that begins a count.
pub const COUNT: &str = "count";

/// The word a command says when its work is finished.
pub const DONE: &str = "done";

/// A request to count what one application sends until the connection says
/// [`DONE`] or closes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToCount {
    /// The application, by the identifier its sandbox gives it.
    application: Application,
}

impl ToCount {
    /// Count what this application sends.
    #[must_use]
    pub const fn what(application: Application) -> Self {
        Self { application }
    }

    /// The application to count.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// The line, without its newline.
    #[must_use]
    pub fn written(&self) -> String {
        format!("{COUNT} {}", self.application.identifier())
    }

    /// One line, read as a count — [`None`] when it is not a count at all, so
    /// the door can read it as one of the other acts instead.
    ///
    /// A line that begins with [`COUNT`] and names no application, or names one
    /// no verb could name, is `Some(Err(…))`: it was meant as a count and is a
    /// caller with a bug.
    #[must_use]
    pub fn read(line: &str) -> Option<Result<Self, NotACount>> {
        let line = line.trim_end_matches(['\r', '\n']);
        let mut words = line.split(' ');
        if words.next() != Some(COUNT) {
            return None;
        }
        if line.len() > Request::THE_MOST {
            return Some(Err(NotACount::TooLong));
        }
        let (Some(named), None) = (words.next(), words.next()) else {
            return Some(Err(NotACount::NotOneApplication));
        };
        Some(
            Application::identified(named)
                .map(Self::what)
                .map_err(|_| NotACount::NotOneApplication),
        )
    }
}

/// Why a line that began as a count is not one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotACount {
    /// Longer than any line on this door may be.
    #[error("a count is one line of at most {} bytes", Request::THE_MOST)]
    TooLong,
    /// It named no application, more than one, or one no verb could name.
    #[error("a count names exactly one application, by its identifier")]
    NotOneApplication,
}

/// What the door answers to a count, at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Begun {
    /// The count has begun.
    Counting,
    /// The application is not installed, so there is nothing to count.
    NotInstalled,
    /// The desktop cannot count on this machine: no counter was pinned at boot,
    /// or every entry it holds is in use.
    CannotCount,
}

impl Begun {
    /// The word on the wire.
    #[must_use]
    pub const fn written(self) -> &'static str {
        match self {
            Self::Counting => "counting",
            Self::NotInstalled => "not-installed",
            Self::CannotCount => "cannot-count",
        }
    }

    /// One line, read; [`None`] for anything else, which a caller must never
    /// read as a count having begun.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        match line.trim_end_matches(['\r', '\n']) {
            "counting" => Some(Self::Counting),
            "not-installed" => Some(Self::NotInstalled),
            "cannot-count" => Some(Self::CannotCount),
            _ => None,
        }
    }
}

/// What the door answers when a count ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counted {
    /// The command said it was done, and this is what the application sent.
    Sent(u64),
    /// The desktop ended the count first — it holds one for at most a fixed
    /// time — and this is what the application sent until then.
    Stopped(u64),
}

impl Counted {
    /// The line, without its newline.
    #[must_use]
    pub fn written(self) -> String {
        match self {
            Self::Sent(bytes) => format!("sent {bytes}"),
            Self::Stopped(bytes) => format!("stopped {bytes}"),
        }
    }

    /// One line, read; [`None`] for anything else.
    #[must_use]
    pub fn read(line: &str) -> Option<Self> {
        let line = line.trim_end_matches(['\r', '\n']);
        let (word, number) = line.split_once(' ')?;
        let bytes = number.parse().ok()?;
        match word {
            "sent" => Some(Self::Sent(bytes)),
            "stopped" => Some(Self::Stopped(bytes)),
            _ => None,
        }
    }

    /// How many bytes, whichever way it ended.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        match self {
            Self::Sent(bytes) | Self::Stopped(bytes) => bytes,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn a_count_is_written_and_read_back_whole() {
        let asked = ToCount::what(Application::identified("org.mozilla.Thunderbird").unwrap());
        assert_eq!(asked.written(), "count org.mozilla.Thunderbird");
        assert_eq!(ToCount::read(&asked.written()), Some(Ok(asked.clone())));
        assert_eq!(
            ToCount::read("count org.mozilla.Thunderbird\n"),
            Some(Ok(asked))
        );
    }

    /// **Another act is not a count**, so the door reads it as what it is.
    #[test]
    fn another_act_is_not_read_as_a_count() {
        assert_eq!(ToCount::read("open org.mozilla.Thunderbird"), None);
        assert_eq!(ToCount::read("counting"), None);
        assert_eq!(ToCount::read(""), None);
    }

    /// **A count with no application, two, or a path is refused**, not guessed.
    #[test]
    fn a_count_that_names_no_single_application_is_refused() {
        for line in [
            "count",
            "count ",
            "count a.b c.d",
            "count ../etc/passwd",
            "count org.example/App",
        ] {
            assert_eq!(
                ToCount::read(line),
                Some(Err(NotACount::NotOneApplication)),
                "{line}"
            );
        }
        let long = format!("count {}", "a".repeat(Request::THE_MOST));
        assert_eq!(ToCount::read(&long), Some(Err(NotACount::TooLong)));
    }

    #[test]
    fn the_answers_read_back_and_nothing_else_does() {
        for begun in [Begun::Counting, Begun::NotInstalled, Begun::CannotCount] {
            assert_eq!(Begun::read(begun.written()), Some(begun));
        }
        assert_eq!(Begun::read("acted"), None);
        for counted in [Counted::Sent(0), Counted::Sent(52_113), Counted::Stopped(7)] {
            assert_eq!(Counted::read(&counted.written()), Some(counted));
        }
        assert_eq!(Counted::Stopped(7).bytes(), 7);
        for not in ["sent", "sent -1", "sent many", "sent 1 2", "gone 4", ""] {
            assert_eq!(Counted::read(not), None, "{not}");
        }
    }
}
