//! What the shortcuts are before anybody changes them.
//!
//! **Conventional wherever being conventional costs nothing.** The machines
//! this replaces are Windows machines, and fifteen years of muscle memory put
//! maximise on `Super+Up`, the halves of the screen on `Super+Left` and
//! `Super+Right`, closing a window on `Alt+F4` and the next window on `Alt+Tab`.
//! Inventing better ones would be paying for a preference with everybody else's
//! first afternoon. Where there is no convention to follow — the agent, the
//! launcher — the two easiest chords go to the two things a person reaches for
//! most.
//!
//! **The defaults live in the code, not in the file a person's settings are
//! written to.** That is what makes them improvable: a release that changes one
//! reaches every machine that never touched it, and touches nothing on a machine
//! that did. The cost is the thing [`crate::clash`] exists for — a new default
//! can land on a chord somebody already moved something else onto — and that is
//! a report rather than a surprise.

use crate::action::Action;
use crate::chord::Chord;
use crate::clash::Clash;
use crate::key::Key;
use crate::modifier::{Modifier, Modifiers};

/// Super held, and nothing else.
const SUPER: Modifiers = Modifiers::just(Modifier::Super);
/// Super and Shift held.
const SUPER_SHIFT: Modifiers = SUPER.and(Modifier::Shift);
/// Alt held, and nothing else.
const ALT: Modifiers = Modifiers::just(Modifier::Alt);
/// Alt and Shift held.
const ALT_SHIFT: Modifiers = ALT.and(Modifier::Shift);

/// The shortcuts alo OS ships with, in the order a settings panel lists them.
///
/// Built by the compiler rather than checked at runtime, which is why a test
/// puts every one of them back through [`Chord::checked`]: the shipped list is
/// held to the rules a person's own bindings are held to, or the rules are
/// advice.
const SHIPPED: [(Action, Chord); 11] = [
    (Action::TheAgent, Chord::shipped(SUPER, Key::A)),
    (Action::Launcher, Chord::shipped(SUPER, Key::Space)),
    (Action::CloseWindow, Chord::shipped(ALT, Key::F4)),
    (Action::MinimiseWindow, Chord::shipped(SUPER, Key::Down)),
    (Action::MaximiseWindow, Chord::shipped(SUPER, Key::Up)),
    (Action::SnapLeft, Chord::shipped(SUPER, Key::Left)),
    (Action::SnapRight, Chord::shipped(SUPER, Key::Right)),
    (Action::NextWindow, Chord::shipped(ALT, Key::Tab)),
    (Action::PreviousWindow, Chord::shipped(ALT_SHIFT, Key::Tab)),
    (Action::NextApplication, Chord::shipped(SUPER, Key::Tab)),
    (
        Action::PreviousApplication,
        Chord::shipped(SUPER_SHIFT, Key::Tab),
    ),
];

/// Why a set of defaults cannot be used.
///
/// Only reachable through [`Defaults::of`]. The shipped list is a constant and
/// is checked by a test instead, because a program that could fail to start over
/// its own defaults would be a program with a worse problem than a clash.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DefaultsError {
    /// One action was given two defaults, so neither could be said to be it.
    #[error("{0} is in the defaults twice")]
    Twice(Action),
    /// Two actions were given the same chord, which is the thing a person is
    /// refused for doing.
    #[error("{0}")]
    Clash(Clash),
}

/// A set of default bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Defaults {
    /// One entry per action at most, in the order they are shown.
    bound: Vec<(Action, Chord)>,
}

impl Defaults {
    /// What alo OS ships with.
    #[must_use]
    pub fn shipped() -> Self {
        Self {
            bound: SHIPPED.to_vec(),
        }
    }

    /// A set of defaults that is not the shipped one — a release being tried
    /// out against a person's changes, or a test of what a new default would
    /// do to them.
    ///
    /// # Errors
    /// [`DefaultsError`] when the list contradicts itself: one action bound
    /// twice, or two actions on one chord.
    pub fn of(bound: Vec<(Action, Chord)>) -> Result<Self, DefaultsError> {
        for (at, (action, chord)) in bound.iter().enumerate() {
            for (other_action, other_chord) in bound.iter().skip(at.saturating_add(1)) {
                if action == other_action {
                    return Err(DefaultsError::Twice(*action));
                }
                if chord == other_chord {
                    return Err(DefaultsError::Clash(Clash::over(
                        *chord,
                        vec![*action, *other_action],
                    )));
                }
            }
        }
        Ok(Self { bound })
    }

    /// What this action does before anybody changes it, if it has a default at
    /// all.
    #[must_use]
    pub fn chord_for(&self, action: Action) -> Option<Chord> {
        self.bound
            .iter()
            .find(|(bound, _)| *bound == action)
            .map(|(_, chord)| *chord)
    }

    /// Every default, in the order they are shown.
    pub fn iter(&self) -> impl Iterator<Item = (Action, Chord)> {
        self.bound.iter().copied()
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self::shipped()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The list we ship is held to the rules a person is held to.** These
    /// chords are built by the compiler and never go through [`Chord::checked`]
    /// in production, so this is the only thing standing between a shipped
    /// default and a chord nobody would have been allowed to set.
    #[test]
    fn every_shipped_default_is_a_chord_a_person_could_have_set() {
        for (action, chord) in Defaults::shipped().iter() {
            let checked = Chord::checked(chord.modifiers(), chord.key());
            assert!(checked.is_ok(), "{action}: {chord} would be refused");
            assert_eq!(checked.unwrap_or(chord), chord);
        }
    }

    /// **Everything a person can rebind arrives bound.** An action with no
    /// default is a feature nobody discovers, since the way you find out a
    /// system does something is that a key does it.
    #[test]
    fn every_action_ships_with_a_shortcut() {
        let shipped = Defaults::shipped();
        for action in Action::ALL {
            assert!(
                shipped.chord_for(*action).is_some(),
                "{action} ships unbound"
            );
        }
        assert_eq!(shipped.iter().count(), Action::ALL.len());
    }

    /// The shipped list does not contradict itself. It is a constant, so
    /// nothing at runtime would catch it if it did.
    #[test]
    fn the_shipped_list_has_no_clash_in_it() {
        assert_eq!(Defaults::of(SHIPPED.to_vec()).unwrap(), Defaults::shipped());
    }

    /// The conventions this list is here to keep. Somebody moving from Windows
    /// should not have to look any of these up.
    #[test]
    fn the_conventional_ones_are_the_conventional_ones() {
        let shipped = Defaults::shipped();
        let said = |action| {
            shipped
                .chord_for(action)
                .map_or_else(|| "unbound".to_owned(), |chord| chord.to_string())
        };
        assert_eq!(said(Action::CloseWindow), "Alt+F4");
        assert_eq!(said(Action::MaximiseWindow), "Super+Up");
        assert_eq!(said(Action::MinimiseWindow), "Super+Down");
        assert_eq!(said(Action::SnapLeft), "Super+Left");
        assert_eq!(said(Action::SnapRight), "Super+Right");
        assert_eq!(said(Action::NextWindow), "Alt+Tab");
        assert_eq!(said(Action::PreviousWindow), "Alt+Shift+Tab");
        assert_eq!(said(Action::NextApplication), "Super+Tab");
        assert_eq!(said(Action::PreviousApplication), "Super+Shift+Tab");
        assert_eq!(said(Action::TheAgent), "Super+A");
        assert_eq!(said(Action::Launcher), "Super+Space");
    }

    /// A set of defaults that binds one action twice is refused: neither entry
    /// could be called the default.
    #[test]
    fn defaults_that_bind_one_action_twice_are_refused() {
        let refused = Defaults::of(vec![
            (Action::TheAgent, Chord::shipped(SUPER, Key::A)),
            (Action::TheAgent, Chord::shipped(SUPER, Key::B)),
        ])
        .unwrap_err();
        assert_eq!(refused, DefaultsError::Twice(Action::TheAgent));
    }

    /// A set of defaults with two actions on one chord is refused as the clash
    /// it is, in the same words a person's own clash is reported in.
    #[test]
    fn defaults_with_two_actions_on_one_chord_are_refused() {
        let chord = Chord::shipped(SUPER, Key::A);
        let refused =
            Defaults::of(vec![(Action::TheAgent, chord), (Action::Launcher, chord)]).unwrap_err();
        assert_eq!(
            refused,
            DefaultsError::Clash(Clash::over(chord, vec![Action::TheAgent, Action::Launcher]))
        );
        assert!(refused.to_string().contains("more than one thing"));
    }
}
