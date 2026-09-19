//! What is reopened at the next sign-in, which is nothing unless the person
//! asked for it.
//!
//! One function and one value. [`at_sign_in`] reads the person's own file and
//! answers with the settings a session runs by, what is to be reopened, and the
//! refusal when the file did not read — the three things whoever starts a session
//! needs, from one reading of one file.
//!
//! # Three ways to get nothing, and all three are the same answer
//!
//! - **The setting is off**, which is how alo OS ships.
//! - **The file did not read**, so nothing in it is honoured — including a list
//!   that happens to be well formed in a file that is not.
//! - **The setting is off and a list is there anyway**, which is a hand-edited
//!   file or a much older release: the choice wins, and the list is ignored. It
//!   is also taken out at the next log-out (`crate::keeping::at_sign_out`).
//!
//! The third is the belt beside the braces, and it is why [`Restoring`] is
//! worked out here rather than read straight off [`crate::Changes`].
//!
//! # What reopening is not
//!
//! It is not this crate opening anything. [`Restoring::These`] is a list, and
//! whoever draws a desktop opens each application on the screen and in the split
//! it names — under the grants that application already has, and with no
//! knowledge of what was in it, because there is none to have.

use std::path::Path;

use crate::changes::Settings;
use crate::keeping;
use crate::open::WasOpen;
use crate::unkept::FileNotRead;

/// What is reopened at this sign-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Restoring {
    /// Nothing. The person did not ask for it, or their file did not read.
    NothingIsReopened,
    /// These, oldest first, each on the screen and in the split it names.
    These(WasOpen),
}

impl Restoring {
    /// What was open, or an empty list — for a caller that would rather walk
    /// than match.
    #[must_use]
    pub fn what_was_open(&self) -> WasOpen {
        match self {
            Self::NothingIsReopened => WasOpen::nothing(),
            Self::These(was_open) => was_open.clone(),
        }
    }
}

/// What a person's own file says at a sign-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtSignIn {
    /// The settings this session runs by — the release's, where the file did not
    /// read.
    pub settings: Settings,
    /// What is reopened.
    pub restoring: Restoring,
    /// Why the file did not read, when it did not, for Settings to say in that
    /// section.
    pub refused: Option<FileNotRead>,
}

/// The person is signing in: what they chose, and what is reopened.
#[must_use]
pub fn at_sign_in(at: &Path) -> AtSignIn {
    let (changes, refused) = match keeping::read(at) {
        Ok(changes) => (Some(changes), None),
        Err(refused) => (None, Some(refused)),
    };
    let settings = changes.as_ref().map_or_else(Settings::shipped, |changes| {
        Settings::shipped().with(changes)
    });
    let restoring = match changes {
        // The choice decides, whatever a list in the file says.
        Some(changes) if settings.reopen && !changes.what_was_open().is_nothing() => {
            Restoring::These(changes.what_was_open())
        }
        Some(_) | None => Restoring::NothingIsReopened,
    };
    AtSignIn {
        settings,
        restoring,
        refused,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::changes::Changes;
    use crate::ending::MayEnd;
    use crate::keeping::THE_FILE;
    use crate::testing::{a_desk, a_folder_of_our_own};

    /// **A person who asked for it signs in to what they left.**
    #[test]
    fn a_person_who_asked_for_it_signs_in_to_what_they_left() {
        let at = a_folder_of_our_own("restoring-asked").join(THE_FILE);
        let mut changes = Changes::untouched();
        changes.set_reopen(true);
        keeping::keep(&at, &changes).unwrap();
        keeping::at_sign_out(&at, &MayEnd::everything_closed(), &a_desk()).unwrap();

        let signing_in = at_sign_in(&at);
        assert_eq!(signing_in.refused, None);
        assert!(signing_in.settings.reopen);
        assert_eq!(signing_in.restoring, Restoring::These(a_desk()));
        assert_eq!(signing_in.restoring.what_was_open(), a_desk());
    }

    /// **A machine nobody configured reopens nothing**, and has no file to read.
    #[test]
    fn a_machine_nobody_configured_reopens_nothing() {
        let at = a_folder_of_our_own("restoring-untouched").join(THE_FILE);
        let signing_in = at_sign_in(&at);
        assert_eq!(signing_in.settings, Settings::shipped());
        assert_eq!(signing_in.restoring, Restoring::NothingIsReopened);
        assert_eq!(signing_in.refused, None);
        assert_eq!(signing_in.restoring.what_was_open(), WasOpen::nothing());
    }

    /// **A list in a file whose owner did not ask for one is ignored.** The
    /// choice decides, and a hand-edited list does not reopen itself.
    #[test]
    fn a_list_nobody_asked_for_is_ignored() {
        let at = a_folder_of_our_own("restoring-not-asked").join(THE_FILE);
        std::fs::write(
            &at,
            "format = 1\n\n[[was-open]]\napplication = \"org.example.Editor\"\non = \"eDP-1 \
             Built-in display\"\nsplit = \"the-whole-screen\"\n",
        )
        .unwrap();
        let signing_in = at_sign_in(&at);
        assert_eq!(signing_in.refused, None, "the file itself is fine");
        assert!(!signing_in.settings.reopen);
        assert_eq!(signing_in.restoring, Restoring::NothingIsReopened);
    }

    /// **A file that did not read reopens nothing**, whatever is in it.
    #[test]
    fn a_file_that_did_not_read_reopens_nothing() {
        let at = a_folder_of_our_own("restoring-refused").join(THE_FILE);
        std::fs::write(
            &at,
            "format = 1\nreopen = true\nwhat-i-was-writing = \"march.odt\"\n\n[[was-open]]\n\
             application = \"org.example.Editor\"\non = \"eDP-1 Built-in display\"\nsplit = \
             \"the-whole-screen\"\n",
        )
        .unwrap();
        let signing_in = at_sign_in(&at);
        assert!(signing_in.refused.is_some());
        assert_eq!(signing_in.settings, Settings::shipped());
        assert_eq!(signing_in.restoring, Restoring::NothingIsReopened);
    }
}
