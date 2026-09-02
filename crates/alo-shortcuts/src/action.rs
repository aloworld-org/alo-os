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

use serde::{Deserialize, Serialize};

use std::fmt;

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

    /// What this does, said the way it would be said in a list of shortcuts.
    #[must_use]
    pub fn purpose(self) -> &'static str {
        match self {
            Self::TheAgent => "Ask the agent",
            Self::Launcher => "Open the launcher",
            Self::CloseWindow => "Close the window",
            Self::MinimiseWindow => "Minimise the window",
            Self::MaximiseWindow => "Maximise the window, or put it back",
            Self::SnapLeft => "Put the window on the left half",
            Self::SnapRight => "Put the window on the right half",
            Self::NextWindow => "Next window",
            Self::PreviousWindow => "Previous window",
            Self::NextApplication => "Next application",
            Self::PreviousApplication => "Previous application",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.purpose())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
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
        let mut said = BTreeSet::new();
        for action in Action::ALL {
            let purpose = action.purpose();
            assert!(!purpose.is_empty(), "{action:?}");
            assert!(said.insert(purpose), "two actions are both {purpose}");
            assert_eq!(action.to_string(), purpose);
        }
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
