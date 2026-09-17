//! Switching keyboards: one shortcut, and the mark in the status area.
//!
//! A person who types in two languages switches between them dozens of times an
//! hour, so this is one chord and not a menu. The chord is `Alt+Space`, and it
//! is the rented option [`THE_OPTION`]: the switch happens inside the rented
//! keyboard configuration rather than in a handler of ours, which is what keeps
//! it working in an application that grabs the keyboard.
//!
//! # Why not `Super+Space`, which is what most systems use
//!
//! Because `alo_shortcuts` already binds `Super+Space` to the launcher, and
//! **this crate reads that crate and never edits it** (this plan's own rule).
//! Two things on one chord is one of them silently not working, which is
//! exactly the bug `alo_shortcuts::Clash` exists to catch. `Alt+Space` is the
//! next rented option that nothing in the release binds, and
//! `the_switch_is_not_a_chord_the_release_already_binds` is what holds that
//! true as more shortcuts are added.
//!
//! When `alo_shortcuts` gains an action for switching keyboards, the chord
//! becomes that crate's value like every other, and [`the_chord`] goes. It is
//! here because the action does not exist and inventing one in this crate would
//! put two lists of shortcuts on the machine.
//!
//! # A person who has taken the chord is told, not overruled
//!
//! [`shadowed_by`] answers *which of your shortcuts is on this chord* and the
//! surface says so. alo OS does not quietly move a shortcut a person bound, and
//! it does not pretend the switch still works.

use alo_shortcuts::{Action, Chord, ChordError, Key, Modifier, Modifiers, Shortcuts};
use alo_strings::{Filling, Said, Strings};

/// The rented option that switches keyboards, which is what is written into the
/// keyboard configuration.
pub const THE_OPTION: &str = "grp:alt_space_toggle";

/// The chord [`THE_OPTION`] is, for showing it beside the other shortcuts and
/// for asking whether anything else is on it.
///
/// # Errors
/// [`ChordError`] if `Alt+Space` ever stops being a chord, which
/// `the_switch_is_a_chord` runs.
pub fn the_chord() -> Result<Chord, ChordError> {
    Chord::checked(Modifiers::just(Modifier::Alt), Key::Space)
}

/// Which of a person's shortcuts is on the switch's chord, if one is.
///
/// `None` on a machine nobody has rebound, which the test below holds for the
/// release as it ships.
#[must_use]
pub fn shadowed_by(shortcuts: &Shortcuts) -> Option<Action> {
    shortcuts.action_for(the_chord().ok()?)
}

/// What a person is told when something else is on the switch's chord: the
/// chord, and what it does instead.
///
/// `None` when nothing is. Never fails and never panics.
#[must_use]
pub fn said_if_shadowed(shortcuts: &Shortcuts, strings: &Strings) -> Option<Said> {
    let action = shadowed_by(shortcuts)?;
    let chord = the_chord().ok()?;
    let filling = chord.fills(
        "chord",
        Filling::of("action", action.said(strings).into_text()),
        strings,
    );
    Some(strings.say(&crate::words::SWITCH_SHADOWED.key(), &filling))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// The switch is a chord, and it is the one this file says it is.
    #[test]
    fn the_switch_is_a_chord() {
        let chord = the_chord().unwrap();
        assert_eq!(chord.key(), Key::Space);
        assert!(chord.modifiers().holds(Modifier::Alt));
        assert!(!chord.modifiers().holds(Modifier::Super));
    }

    /// **Nothing the release binds is on the switch's chord.** The one this
    /// would have used, `Super+Space`, is the launcher's, which is why it is
    /// not the one.
    #[test]
    fn the_switch_is_not_a_chord_the_release_already_binds() {
        let shipped = Shortcuts::shipped();
        assert_eq!(shadowed_by(&shipped), None);
        assert_eq!(
            shipped
                .action_for(Chord::checked(Modifiers::just(Modifier::Super), Key::Space).unwrap()),
            Some(Action::Launcher)
        );
    }

    /// **A person who puts something else on the chord is told what is on it**,
    /// in a sentence naming both the chord and what it does.
    #[test]
    fn a_shortcut_bound_onto_the_switch_is_named() {
        let mut shortcuts = Shortcuts::shipped();
        shortcuts
            .bind(Action::TheAgent, the_chord().unwrap())
            .unwrap();
        assert_eq!(shadowed_by(&shortcuts), Some(Action::TheAgent));
        let strings = in_english();
        let said = said_if_shadowed(&shortcuts, &strings).unwrap();
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(said.text().contains("Alt+Space"), "{said}");
        assert!(
            said.text().contains(Action::TheAgent.said(&strings).text()),
            "{said}"
        );
        assert_eq!(said_if_shadowed(&Shortcuts::shipped(), &strings), None);
    }
}
