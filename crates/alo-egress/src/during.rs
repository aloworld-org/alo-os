//! An application sending something while a person's own command runs, and the
//! line a person reads while it does
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)).
//!
//! The third kind of line on the one indicator. [`Leaving`](crate::Leaving) is
//! an agent's egress and carries whose authority it is under;
//! [`OnItsOwn`](crate::OnItsOwn) is alo OS's own and carries its reason. This is
//! neither: **the person told an application to do something by typing a
//! command, and the application sent something while it ran.** Nobody's grant
//! is behind it and alo OS did not send it, so it has neither field.
//!
//! **It has no destination, and that is the owner's rule, not a gap.** The
//! counter reads how many bytes the application's own control groups sent and
//! nothing about where they went (ADR 0148 §1), so a line that named a place
//! would be inventing one. What it names is the application, and *during your
//! command* — never *because of* it, since the count includes anything else the
//! application sent in those seconds.

use alo_strings::{Filling, Said, Strings};
use serde::Serialize;

use crate::words;

/// The longest name an application's line may carry.
const LONGEST: usize = 128;

/// An application, sending something during a person's command.
///
/// Serialises so the indicator's contents can be told to a desktop, and does
/// not deserialise, as [`OnItsOwn`](crate::OnItsOwn) does not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DuringTheirCommand {
    /// The application, by the identifier its sandbox gives it.
    application: String,
    /// What a person calls it: its menu entry's name, in their language.
    called: String,
}

/// Why an application could not be put on the indicator's line.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAnApplicationsLine {
    /// The identifier or the name is empty.
    #[error("an application on the indicator needs an identifier and a name")]
    Nothing,
    /// One of them holds a character a person cannot read on one line.
    #[error("an application's name or identifier holds a character that cannot be shown")]
    NotShowable,
    /// One of them is longer than a line can hold.
    #[error("an application's name or identifier is longer than {LONGEST} characters")]
    TooLong,
}

impl DuringTheirCommand {
    /// The application with this identifier, which a person calls `called`.
    ///
    /// # Errors
    /// [`NotAnApplicationsLine`] when either is empty, longer than a line, or
    /// holds a control character — a name that could not be shown honestly on
    /// one line is refused rather than shown wrongly.
    pub fn of(application: &str, called: &str) -> Result<Self, NotAnApplicationsLine> {
        for text in [application, called] {
            if text.trim().is_empty() {
                return Err(NotAnApplicationsLine::Nothing);
            }
            if text.chars().any(char::is_control) {
                return Err(NotAnApplicationsLine::NotShowable);
            }
            if text.chars().count() > LONGEST {
                return Err(NotAnApplicationsLine::TooLong);
            }
        }
        Ok(Self {
            application: application.to_owned(),
            called: called.to_owned(),
        })
    }

    /// The application, by its identifier.
    #[must_use]
    pub fn application(&self) -> &str {
        &self.application
    }

    /// What a person calls it.
    #[must_use]
    pub fn called(&self) -> &str {
        &self.called
    }

    /// The line a person reads: *Thunderbird is sending, during your command.*
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        said_of_their_command(&self.called, strings)
    }
}

/// **The one sentence for an application during a person's command**, for this
/// machine's line and for a [`crate::Reported`] one alike.
pub(crate) fn said_of_their_command(called: &str, strings: &Strings) -> Said {
    strings.say(
        &words::IS_SENDING_DURING_YOUR_COMMAND.key(),
        &Filling::of("application", called),
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    #[test]
    fn the_line_names_the_application_and_says_during_your_command() {
        let line = DuringTheirCommand::of("org.mozilla.Thunderbird", "Thunderbird").unwrap();
        assert_eq!(
            line.said(&in_english()).text(),
            "Thunderbird is sending, during your command"
        );
        assert_eq!(line.application(), "org.mozilla.Thunderbird");
    }

    #[test]
    fn the_line_is_said_in_the_readers_language() {
        let strings = translated(&[(
            words::IS_SENDING_DURING_YOUR_COMMAND,
            "{application} sendet etwas, während dein Befehl läuft",
        )]);
        let line = DuringTheirCommand::of("org.mozilla.Thunderbird", "Thunderbird").unwrap();
        assert_eq!(
            line.said(&strings).text(),
            "Thunderbird sendet etwas, während dein Befehl läuft"
        );
    }

    /// **A name that cannot be shown honestly on one line is refused**, rather
    /// than shown broken or as somebody else's.
    #[test]
    fn a_name_that_cannot_be_shown_is_refused() {
        assert_eq!(
            DuringTheirCommand::of("", "Thunderbird"),
            Err(NotAnApplicationsLine::Nothing)
        );
        assert_eq!(
            DuringTheirCommand::of("org.mozilla.Thunderbird", "  "),
            Err(NotAnApplicationsLine::Nothing)
        );
        assert_eq!(
            DuringTheirCommand::of("org.mozilla.Thunderbird", "Thunder\nbird"),
            Err(NotAnApplicationsLine::NotShowable)
        );
        assert_eq!(
            DuringTheirCommand::of("org.mozilla.Thunderbird", "\u{1b}[31mbird"),
            Err(NotAnApplicationsLine::NotShowable)
        );
        assert_eq!(
            DuringTheirCommand::of("org.mozilla.Thunderbird", &"a".repeat(LONGEST + 1)),
            Err(NotAnApplicationsLine::TooLong)
        );
        assert!(DuringTheirCommand::of("org.mozilla.Thunderbird", &"a".repeat(LONGEST)).is_ok());
    }
}
