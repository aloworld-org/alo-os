//! What a person changed, which is the only part that is written down.
//!
//! **The defaults are not in the settings file.** A file that held every
//! binding would freeze the day it was written: a release that improved a
//! default would reach nobody who had ever opened the shortcuts panel, and a
//! release that added an action would reach them with it unbound. So what is
//! stored is the difference — the actions a person moved, and the actions a
//! person cleared — and everything else comes from the code that is running.
//!
//! **Clearing is a change, and it has to be written down as one.** [`Changed`]
//! holds `Option<Chord>` rather than `Chord` because *I do not want this
//! shortcut* is a decision, and a file that could not express it would hand the
//! default back at the next sign-in.

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::chord::Chord;

/// One action a person moved or cleared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changed {
    /// What was changed.
    pub action: Action,
    /// What it is now — or `None`, meaning the person wants no shortcut for it.
    pub chord: Option<Chord>,
}

/// Everything a person has changed, in the order they changed it.
///
/// This is what a settings file holds and nothing else, so a file written by an
/// older release still says exactly what its owner decided, however much the
/// defaults have moved since.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Vec<Changed>", into = "Vec<Changed>")]
pub struct Changes {
    /// One entry per action at most, oldest first.
    made: Vec<Changed>,
}

impl Changes {
    /// Nothing changed yet.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Record that this action is now this chord, or — with `None` — that the
    /// person wants no shortcut for it.
    ///
    /// Replaces any earlier change to the same action: two rows for one action
    /// would be a file that disagrees with itself.
    pub fn set(&mut self, action: Action, chord: Option<Chord>) {
        self.made.retain(|changed| changed.action != action);
        self.made.push(Changed { action, chord });
    }

    /// Forget that this action was ever changed, which puts it back to the
    /// default the running release ships.
    ///
    /// Says whether there was anything to forget.
    pub fn forget(&mut self, action: Action) -> bool {
        let before = self.made.len();
        self.made.retain(|changed| changed.action != action);
        self.made.len() != before
    }

    /// Forget everything, putting every action back to its default.
    pub fn forget_everything(&mut self) {
        self.made.clear();
    }

    /// What this action was changed to, if it was changed. The outer `Option`
    /// is *was it changed*; the inner one is *to a chord, or to nothing*.
    #[must_use]
    pub fn made_to(&self, action: Action) -> Option<Option<Chord>> {
        self.made
            .iter()
            .find(|changed| changed.action == action)
            .map(|changed| changed.chord)
    }

    /// Everything that was changed, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = Changed> {
        self.made.iter().copied()
    }

    /// How many actions have been changed.
    #[must_use]
    pub fn len(&self) -> usize {
        self.made.len()
    }

    /// Whether nothing has been changed at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.made.is_empty()
    }
}

impl From<Vec<Changed>> for Changes {
    /// Normalising, because a file is a thing a person can edit: an action
    /// named twice means what the file says last, which is the only reading
    /// that does not throw one of the two away at random.
    fn from(made: Vec<Changed>) -> Self {
        let mut changes = Self::none();
        for changed in made {
            changes.set(changed.action, changed.chord);
        }
        changes
    }
}

impl From<Changes> for Vec<Changed> {
    fn from(changes: Changes) -> Self {
        changes.made
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::key::Key;
    use crate::modifier::{Modifier, Modifiers};

    fn chord(key: Key) -> Chord {
        Chord::checked(Modifiers::just(Modifier::Super), key).unwrap()
    }

    /// Setting, changing again, and forgetting — and the outer `Option` says
    /// whether the person touched the action at all.
    #[test]
    fn a_change_is_recorded_replaced_and_forgotten() {
        let mut changes = Changes::none();
        assert!(changes.is_empty());
        assert_eq!(changes.made_to(Action::NextWindow), None);

        changes.set(Action::NextWindow, Some(chord(Key::J)));
        changes.set(Action::NextWindow, Some(chord(Key::K)));
        assert_eq!(changes.len(), 1, "one action, one row");
        assert_eq!(
            changes.made_to(Action::NextWindow),
            Some(Some(chord(Key::K)))
        );

        assert!(changes.forget(Action::NextWindow));
        assert!(!changes.forget(Action::NextWindow));
        assert_eq!(changes.made_to(Action::NextWindow), None);
    }

    /// **Clearing a shortcut is a decision and is stored as one.** Without the
    /// inner `None` a person who wanted no shortcut would be handed the default
    /// back at the next sign-in.
    #[test]
    fn wanting_no_shortcut_is_a_change_like_any_other() {
        let mut changes = Changes::none();
        changes.set(Action::TheAgent, None);
        assert_eq!(changes.made_to(Action::TheAgent), Some(None));
        assert_eq!(changes.len(), 1);

        changes.forget_everything();
        assert!(changes.is_empty());
        assert_eq!(changes.made_to(Action::TheAgent), None);
    }

    /// The file holds the differences and nothing else, so an untouched machine
    /// stores an empty list rather than a copy of the defaults.
    #[test]
    fn the_file_holds_only_what_was_changed() {
        assert_eq!(serde_json::to_string(&Changes::none()).unwrap(), "[]");

        let mut changes = Changes::none();
        changes.set(Action::TheAgent, Some(chord(Key::Space)));
        changes.set(Action::Launcher, None);
        let written = serde_json::to_string(&changes).unwrap();
        assert_eq!(
            written,
            r#"[{"action":"TheAgent","chord":{"modifiers":["Super"],"key":"Space"}},{"action":"Launcher","chord":null}]"#
        );
        assert_eq!(serde_json::from_str::<Changes>(&written).unwrap(), changes);
    }

    /// A hand-edited file that names one action twice means what it says last,
    /// rather than holding two rows one of which never applies.
    #[test]
    fn a_file_that_repeats_itself_means_what_it_says_last() {
        let read: Changes = serde_json::from_str(
            r#"[{"action":"NextWindow","chord":{"modifiers":["Super"],"key":"J"}},
                {"action":"NextWindow","chord":null}]"#,
        )
        .unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read.made_to(Action::NextWindow), Some(None));
    }
}
