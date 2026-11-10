//! An application alo is working in, and the line a person reads while it
//! does
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q6 and the owner's answers of 2026-11-10).
//!
//! The fourth kind of line on the one indicator. When alo presses *Publish* in
//! an editor, **the bytes are sent by the editor**, not by `alo-agentd`, so
//! [`Leaving`](crate::Leaving) cannot show them; this is what does.
//!
//! - **At the default**, alo works in a separate copy of the application
//!   started for it, and only that copy is counted: the line goes up when the
//!   copy is seen sending ([`HowItIsShown::Counted`]), or, on a machine that
//!   cannot count, for the whole time alo works in it and saying so
//!   ([`HowItIsShown::Uncounted`]).
//! - **Turned up**, alo works in the person's own window, and the line is up for
//!   the whole time saying the application *may* be sending
//!   ([`HowItIsShown::TheirWindow`]) — never how much, because counting the
//!   person's own window would be watching them.
//!
//! **It has no destination**, for [`crate::DuringTheirCommand`]'s reason: the
//! counter reads how much, never where.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::during::{DuringTheirCommand, NotAnApplicationsLine};
use crate::words;

/// Which of the three lines it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HowItIsShown {
    /// alo's own copy, counted, and seen sending.
    Counted,
    /// alo's own copy, on a machine that cannot count it.
    Uncounted,
    /// The person's own window, at the setting turned up; never counted.
    TheirWindow,
}

/// An application alo is working in.
///
/// Serialises so the indicator's contents can be told to another process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AloAtWork {
    /// The application, by its identifier.
    application: String,
    /// What a person calls it.
    called: String,
    /// Which line.
    how: HowItIsShown,
}

impl AloAtWork {
    /// alo working in the application with this identifier, which a person
    /// calls `called`, shown this way.
    ///
    /// # Errors
    /// [`NotAnApplicationsLine`] on exactly the names
    /// [`DuringTheirCommand::of`] refuses, by the same check.
    pub fn of(
        application: &str,
        called: &str,
        how: HowItIsShown,
    ) -> Result<Self, NotAnApplicationsLine> {
        let checked = DuringTheirCommand::of(application, called)?;
        Ok(Self {
            application: checked.application().to_owned(),
            called: checked.called().to_owned(),
            how,
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

    /// Which line.
    #[must_use]
    pub const fn how(&self) -> HowItIsShown {
        self.how
    }

    /// The line a person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        said_of_alo_at_work(&self.called, self.how, strings)
    }
}

/// **The one sentence for each of the three**, for this machine's line and for
/// a [`crate::Reported`] one alike.
pub(crate) fn said_of_alo_at_work(called: &str, how: HowItIsShown, strings: &Strings) -> Said {
    let word = match how {
        HowItIsShown::Counted => words::ALOS_COPY_IS_SENDING,
        HowItIsShown::Uncounted => words::ALOS_COPY_CANNOT_BE_COUNTED,
        HowItIsShown::TheirWindow => words::ALO_IS_IN_YOUR_WINDOW,
    };
    strings.say(&word.key(), &Filling::of("application", called))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    #[test]
    fn each_line_says_what_alo_is_doing_and_what_is_counted() {
        let strings = in_english();
        for (how, said) in [
            (
                HowItIsShown::Counted,
                "Firefox, in the copy alo is working in, is sending",
            ),
            (
                HowItIsShown::Uncounted,
                "alo is working in its own copy of Firefox, and this machine cannot count what it \
                 sends",
            ),
            (
                HowItIsShown::TheirWindow,
                "alo is working in your Firefox window, which may be sending, and alo cannot \
                 count what",
            ),
        ] {
            let line = AloAtWork::of("org.mozilla.firefox", "Firefox", how).unwrap();
            assert_eq!(line.said(&strings).text(), said);
            assert_eq!(line.how(), how);
        }
    }

    /// The same names are refused as for a person's command, by the same check.
    #[test]
    fn a_name_that_cannot_be_shown_is_refused() {
        assert_eq!(
            AloAtWork::of("", "Firefox", HowItIsShown::Counted),
            Err(NotAnApplicationsLine::Nothing)
        );
        assert_eq!(
            AloAtWork::of(
                "org.mozilla.firefox",
                "Fire\nfox",
                HowItIsShown::TheirWindow
            ),
            Err(NotAnApplicationsLine::NotShowable)
        );
    }
}
