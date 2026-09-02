//! Two actions wanting the same keys, and the refusal that keeps it from
//! happening quietly.
//!
//! **The last binding does not win.** A model where it did would let a person
//! set `Super+Left` on one thing and lose it from another without being told,
//! and they would find out days later when the shortcut they did not change
//! stopped working. So there are two types here, for the two moments a clash can
//! arrive by.
//!
//! [`Taken`] is the refusal: it comes back from [`crate::Shortcuts::bind`] when
//! the chord somebody just pressed is already doing something else, and it names
//! what. Nothing is changed.
//!
//! [`Clash`] is the report, and it exists because refusing at the moment of
//! binding is not enough. **A release can add a default that lands on a chord
//! somebody already moved something else onto**, and no refusal at bind time
//! could have seen it coming — the binding was made before the default existed.
//! So a clash has to be a thing the model can hold and show, not only a thing it
//! can prevent.

use std::fmt;

use crate::action::Action;
use crate::chord::Chord;

/// A chord that more than one action wants.
///
/// The actions are in the order [`Action::ALL`] lists them, so the same clash
/// reads the same way every time it is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clash {
    /// The chord being fought over.
    chord: Chord,
    /// Everything that wants it — always two or more.
    actions: Vec<Action>,
}

impl Clash {
    /// A clash over this chord, between these actions.
    pub(crate) fn over(chord: Chord, actions: Vec<Action>) -> Self {
        Self { chord, actions }
    }

    /// The chord being fought over.
    #[must_use]
    pub fn chord(&self) -> Chord {
        self.chord
    }

    /// Everything that wants it.
    #[must_use]
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }
}

impl fmt::Display for Clash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} is set to do more than one thing:", self.chord)?;
        for action in &self.actions {
            write!(f, " {action};")?;
        }
        f.write_str(" change one of them")
    }
}

/// The refusal a person gets when the chord they pressed is already doing
/// something.
///
/// It carries the action that holds the chord, because a refusal that only said
/// *taken* would leave somebody pressing keys to find out by what.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{chord} is already {by} — change that one first, or use another key")]
pub struct Taken {
    /// The chord that was asked for.
    chord: Chord,
    /// What already does it.
    by: Action,
}

impl Taken {
    /// A refusal of this chord, because this action has it.
    pub(crate) fn new(chord: Chord, by: Action) -> Self {
        Self { chord, by }
    }

    /// The chord that was asked for.
    #[must_use]
    pub fn chord(&self) -> Chord {
        self.chord
    }

    /// What already does it.
    #[must_use]
    pub fn by(&self) -> Action {
        self.by
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

    fn chord() -> Chord {
        Chord::checked(Modifiers::just(Modifier::Super), Key::Left).unwrap()
    }

    /// A refusal names what already has the chord, and says what to do about
    /// it. "That is taken" would leave somebody guessing by what.
    #[test]
    fn a_refusal_names_what_already_has_it() {
        let taken = Taken::new(chord(), Action::SnapLeft);
        assert_eq!(taken.chord(), chord());
        assert_eq!(taken.by(), Action::SnapLeft);
        let said = taken.to_string();
        assert!(said.contains("Super+Left"), "{said}");
        assert!(said.contains("Put the window on the left half"), "{said}");
        assert!(said.contains("or use another key"), "{said}");
    }

    /// A report says everything that wants the chord, not only the first two,
    /// so a person fixing it can see all of what they are fixing.
    #[test]
    fn a_report_says_everything_that_wants_the_chord() {
        let clash = Clash::over(chord(), vec![Action::SnapLeft, Action::NextWindow]);
        assert_eq!(clash.chord(), chord());
        assert_eq!(clash.actions(), [Action::SnapLeft, Action::NextWindow]);
        let said = clash.to_string();
        assert!(said.contains("Put the window on the left half"), "{said}");
        assert!(said.contains("Next window"), "{said}");
        assert!(said.contains("more than one thing"), "{said}");
    }
}
