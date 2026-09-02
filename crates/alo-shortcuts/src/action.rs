//! What a shortcut does, and the closed list of what it can do.
//!
//! An [`Action`] is a thing the system itself does — summon the agent, close a
//! window, move to the next application. It is not a thing an *application*
//! does: `Ctrl+B` is bold in a document because the document received the keys,
//! and nothing in this crate knows or cares. The distinction is the whole reason
//! this list is short. Every action here costs a chord that no application on
//! the machine can ever see again, and a system that took thirty of them would
//! be a system whose applications behave strangely for reasons nobody can find.
//!
//! **The list is what `docs/features.md` promises at v0.01** and nothing else:
//! the agent overlay, the launcher, the window management a person does with the
//! keyboard, and switching between windows and applications. Dividing the screen
//! from the keyboard is v0.5 and is not here; screenshots are v0.5 and are not
//! here. An action arrives with the feature it belongs to, because a shortcut
//! bound to something that does not happen yet is a bug report waiting to be
//! filed.
//!
//! What each action *does* is the compositor's, which does not exist yet. What
//! it is *called* is here, because a person rebinding a shortcut is choosing
//! from this list and needs to read it.
//!
//! **In the language they read.** An [`Action`] has no `Display`: the only road
//! to words is [`Action::said`], which takes the strings this machine reads and
//! answers with a `Said` that says whether anybody translated it. A `Display`
//! would be an English row one `to_string()` away from a settings panel whose
//! author had no reason to think about it, and *hardcoded English is a bug*
//! rather than a preference. What the code holds instead is [`Action::word`] —
//! the declaration in [`crate::words`], which is the sentence a translator is
//! handed.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// Something the system does when a chord is pressed.
///
/// The names are a stored format — a settings file holds them beside the chord
/// a person chose — so they change additively or not at all. Removing one is
/// removing a feature, and a person's binding for it goes with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Action {
    /// Bring up the agent over whatever is in front of you.
    TheAgent,
    /// Open the launcher.
    Launcher,
    /// Close the window in front.
    CloseWindow,
    /// Put the window in front out of the way.
    MinimiseWindow,
    /// Make the window in front fill the screen, or put it back.
    MaximiseWindow,
    /// Snap the window in front to the left half of the screen.
    SnapLeft,
    /// Snap the window in front to the right half of the screen.
    SnapRight,
    /// Move to the next window.
    NextWindow,
    /// Move to the previous window.
    PreviousWindow,
    /// Move to the next application.
    NextApplication,
    /// Move to the previous application.
    PreviousApplication,
}

impl Action {
    /// Everything a shortcut can do, in the order a settings panel lists them:
    /// the two that summon something, then the window in front, then moving
    /// between windows.
    pub const ALL: &'static [Self] = &[
        Self::TheAgent,
        Self::Launcher,
        Self::CloseWindow,
        Self::MinimiseWindow,
        Self::MaximiseWindow,
        Self::SnapLeft,
        Self::SnapRight,
        Self::NextWindow,
        Self::PreviousWindow,
        Self::NextApplication,
        Self::PreviousApplication,
    ];

    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::TheAgent => words::THE_AGENT,
            Self::Launcher => words::LAUNCHER,
            Self::CloseWindow => words::CLOSE_WINDOW,
            Self::MinimiseWindow => words::MINIMISE_WINDOW,
            Self::MaximiseWindow => words::MAXIMISE_WINDOW,
            Self::SnapLeft => words::SNAP_LEFT,
            Self::SnapRight => words::SNAP_RIGHT,
            Self::NextWindow => words::NEXT_WINDOW,
            Self::PreviousWindow => words::PREVIOUS_WINDOW,
            Self::NextApplication => words::NEXT_APPLICATION,
            Self::PreviousApplication => words::PREVIOUS_APPLICATION,
        }
    }

    /// What this does, in the language the person reads — the row a settings
    /// panel draws.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not:
    /// there is always something to put on the screen, and what there was to
    /// say about where it came from is on the [`Said`]. A `Strings` that was
    /// never given [`crate::shortcut_words`] answers with the key, marked, and
    /// `Said::is_a_bug` — which is the honest answer to *the shell forgot to
    /// declare what this crate can say*.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use std::collections::BTreeSet;

    /// The list a settings panel walks holds every action once. An action
    /// missing from it is an action a person cannot rebind, which is the
    /// feature this crate exists for going quietly missing.
    #[test]
    fn the_list_holds_each_action_once() {
        let unique: BTreeSet<Action> = Action::ALL.iter().copied().collect();
        assert_eq!(unique.len(), Action::ALL.len());
        assert!(Action::ALL.contains(&Action::TheAgent));
    }

    /// Every action says what it does, and no two of them say the same thing —
    /// two rows reading "Next window" would be a list nobody could set from.
    #[test]
    fn every_action_says_what_it_does_and_no_two_say_the_same() {
        let strings = in_english();
        let mut said = BTreeSet::new();
        for action in Action::ALL {
            let row = action.said(&strings);
            assert!(!row.text().is_empty(), "{action:?}");
            assert!(!row.is_a_bug(), "{action:?} is not declared");
            assert!(
                said.insert(row.text().to_owned()),
                "two actions are both {row}"
            );
            assert_eq!(row.text(), action.word().says());
        }
    }

    /// **A row a person reads is the translation when there is one**, and says
    /// so — which is the whole of what moving this crate onto `alo-strings`
    /// bought.
    #[test]
    fn a_row_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(words::SNAP_LEFT, "Fenster auf die linke Hälfte legen")]);
        let said = Action::SnapLeft.said(&strings);
        assert_eq!(said.text(), "Fenster auf die linke Hälfte legen");
        assert!(said.is_translated());

        // And the one nobody translated is still English, and says it is.
        let untranslated = Action::SnapRight.said(&strings);
        assert_eq!(untranslated.text(), "Put the window on the right half");
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }

    /// A settings file holds the name, so a person's binding survives a release
    /// that adds an action above it in the list.
    #[test]
    fn a_settings_file_holds_the_name() {
        assert_eq!(
            serde_json::to_string(&Action::SnapLeft).unwrap(),
            r#""SnapLeft""#
        );
        assert_eq!(
            serde_json::from_str::<Action>(r#""TheAgent""#).unwrap(),
            Action::TheAgent
        );
        assert!(serde_json::from_str::<Action>(r#""RunAnything""#).is_err());
    }
}
