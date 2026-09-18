//! Switching desktops from the keyboard, without taking a chord away from
//! `alo-shortcuts`.
//!
//! **Why the chords are here and not there.** This crate's plan reads
//! `alo-shortcuts` and never edits it, and there is a reason stronger than the
//! plan: `alo_shortcuts::Action` is a **closed list of fixed things the system
//! does**, and *go to desktop 4* is not one of them. It is a request about a
//! desktop that may or may not exist, made by a person who may have three
//! desktops today and seven tomorrow. A closed enum cannot hold nine of those
//! without nine variants that are the same variant, and it certainly cannot
//! hold *go to desktop 12*. So what a chord means when it reaches a desktop is
//! this crate's, and stays parameterised by a [`Switch`].
//!
//! **And `alo-shortcuts` still wins every argument.** A system shortcut is a key
//! the compositor takes before any application sees it; a desktop chord is one
//! this crate asks about afterwards. So [`DesktopChords::switch_for`] asks the
//! person's own shortcuts first and answers `None` the moment they say the chord
//! already does something, and [`DesktopChords::bind`] refuses a chord a system
//! action holds rather than shadowing it. A person who binds *the launcher* to
//! `Super+PageDown` has moved the launcher there and lost the desktop chord,
//! which is the answer they asked for.
//!
//! # What is shipped
//!
//! `Super+PageDown` and `Super+PageUp` for the next and previous desktop, and
//! `Super+1` … `Super+9` for the first nine by number. None of the eleven is a
//! chord `alo_shortcuts::Defaults` ships, which the test at the bottom of this
//! file holds against the real list rather than against a copy of it.

use alo_shortcuts::{Chord, ChordError, Key, Modifier, Modifiers, Shortcuts};

use crate::on_a_display::MOST_DESKTOPS;
use crate::position::Position;
use crate::refusing::Refused;
use crate::switching::Switch;

/// Which chord goes to which desktop.
///
/// Held in the order they were bound, which is the order a shortcuts panel lists
/// them. A `Vec` rather than a map because `alo_shortcuts::Chord` is deliberately
/// not `Ord` or `Hash` — it is a thing a person presses, not a key — and eleven
/// of them are not worth a second ordering of somebody else's type.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DesktopChords {
    /// The chords, and what each asks for.
    bound: Vec<(Chord, Switch)>,
}

impl DesktopChords {
    /// No desktop chords at all: switching is then a gesture and nothing else.
    #[must_use]
    pub const fn none() -> Self {
        Self { bound: Vec::new() }
    }

    /// The eleven chords alo OS ships.
    ///
    /// # Errors
    /// [`ChordError`], which the list below cannot cause — every one of them
    /// holds Super, and none of them is the clipboard's. The test at the bottom
    /// of this file is what says so.
    pub fn as_shipped() -> Result<Self, ChordError> {
        let mut chords = Self::none();
        let held = Modifiers::just(Modifier::Super);
        chords
            .bound
            .push((Chord::checked(held, Key::PageDown)?, Switch::Next));
        chords
            .bound
            .push((Chord::checked(held, Key::PageUp)?, Switch::Previous));
        for (number, key) in [
            (1, Key::Digit1),
            (2, Key::Digit2),
            (3, Key::Digit3),
            (4, Key::Digit4),
            (5, Key::Digit5),
            (6, Key::Digit6),
            (7, Key::Digit7),
            (8, Key::Digit8),
            (9, Key::Digit9),
        ] {
            let Some(at) = Position::numbered(number) else {
                continue;
            };
            chords
                .bound
                .push((Chord::checked(held, key)?, Switch::To(at)));
        }
        Ok(chords)
    }

    /// What this chord asks for, pressed on this person's machine — or `None`
    /// when it is not a desktop chord at all.
    ///
    /// `None` also when the person's own shortcuts say the chord already does
    /// something, because the compositor takes a system shortcut first and this
    /// crate would otherwise be claiming a chord it never gets.
    #[must_use]
    pub fn switch_for(&self, shortcuts: &Shortcuts, chord: Chord) -> Option<Switch> {
        if shortcuts.action_for(chord).is_some() {
            return None;
        }
        self.bound
            .iter()
            .find(|(bound, _)| *bound == chord)
            .map(|(_, switch)| *switch)
    }

    /// The chord bound to this, when one is — the row a shortcuts panel draws.
    #[must_use]
    pub fn chord_for(&self, switch: Switch) -> Option<Chord> {
        self.bound
            .iter()
            .find(|(_, bound)| *bound == switch)
            .map(|(chord, _)| *chord)
    }

    /// Every chord bound here, in the order a panel lists them.
    pub fn bound(&self) -> impl Iterator<Item = (Chord, Switch)> + '_ {
        self.bound.iter().copied()
    }

    /// Bind this chord to this switch, moving it off whatever it was on.
    ///
    /// # Errors
    /// - [`Refused::ChordIsTaken`] when one of the person's system shortcuts
    ///   already has the chord — the compositor takes that first, so binding it
    ///   here would be a chord that never arrives;
    /// - [`Refused::ChordIsASwitch`] when another desktop switch already has it;
    /// - [`Refused::NoSuchPosition`] for a desktop beyond [`MOST_DESKTOPS`],
    ///   which no display can ever have.
    pub fn bind(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
        switch: Switch,
    ) -> Result<(), Refused> {
        if let Some(action) = shortcuts.action_for(chord) {
            return Err(Refused::ChordIsTaken(action));
        }
        if let Some((_, already)) = self.bound.iter().find(|(bound, _)| *bound == chord) {
            if *already != switch {
                return Err(Refused::ChordIsASwitch(*already));
            }
            return Ok(());
        }
        if let Switch::To(at) = switch
            && usize::from(at.number()) > MOST_DESKTOPS
        {
            return Err(Refused::NoSuchPosition(at));
        }
        self.unbind(switch);
        self.bound.push((chord, switch));
        Ok(())
    }

    /// Leave this switch with no chord: a person who wants the key back.
    ///
    /// Whether one was bound is the answer, so a settings panel can say nothing
    /// changed.
    pub fn unbind(&mut self, switch: Switch) -> bool {
        let before = self.bound.len();
        self.bound.retain(|(_, bound)| *bound != switch);
        self.bound.len() != before
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_shortcuts::Action;

    /// A chord with Super held, for the tests below.
    fn chord(key: Key) -> Chord {
        Chord::checked(Modifiers::just(Modifier::Super), key).unwrap()
    }

    /// The shipped chords are chords, and none of them is one a system shortcut
    /// already has — held against `alo-shortcuts`' own shipped list, so a
    /// default moved there is a failure here rather than a chord that silently
    /// stops working.
    #[test]
    fn the_shipped_chords_take_nothing_a_shortcut_already_has() {
        let chords = DesktopChords::as_shipped().unwrap();
        let shortcuts = Shortcuts::shipped();
        assert_eq!(chords.bound().count(), 11);
        for (chord, switch) in chords.bound() {
            assert_eq!(shortcuts.action_for(chord), None, "{switch:?}");
            assert_eq!(chords.switch_for(&shortcuts, chord), Some(switch));
        }
        assert_eq!(
            chords.chord_for(Switch::Next),
            Some(chord(Key::PageDown)),
            "the next desktop is Super+PageDown"
        );
        assert_eq!(
            chords.chord_for(Switch::To(Position::first())),
            Some(chord(Key::Digit1))
        );
        // Nothing reaches a tenth desktop from the keyboard, and the row of
        // digits is why: there is no Super+10.
        assert_eq!(
            chords.chord_for(Switch::To(Position::numbered(10).unwrap())),
            None
        );
    }

    /// **A chord a system shortcut holds is not a desktop chord**, however it
    /// got there: bound before the person moved the shortcut onto it, it stops
    /// answering, and binding it in the first place is refused naming the
    /// action.
    #[test]
    fn a_system_shortcut_wins_the_chord() {
        let mut shortcuts = Shortcuts::shipped();
        let mut chords = DesktopChords::as_shipped().unwrap();
        let taken = chord(Key::PageDown);

        // The person moves the launcher onto the desktop chord.
        shortcuts.bind(Action::Launcher, taken).unwrap();
        assert_eq!(chords.switch_for(&shortcuts, taken), None);
        // And binding it again is refused, naming what has it.
        assert_eq!(
            chords.bind(&shortcuts, taken, Switch::Next),
            Err(Refused::ChordIsTaken(Action::Launcher))
        );
        // The rest still answer.
        assert_eq!(
            chords.switch_for(&shortcuts, chord(Key::PageUp)),
            Some(Switch::Previous)
        );
    }

    /// A chord already reaching another desktop is refused rather than moved,
    /// and rebinding a switch moves it off its old chord.
    #[test]
    fn one_chord_reaches_one_desktop() {
        let shortcuts = Shortcuts::shipped();
        let mut chords = DesktopChords::as_shipped().unwrap();
        assert_eq!(
            chords.bind(&shortcuts, chord(Key::Digit2), Switch::Next),
            Err(Refused::ChordIsASwitch(Switch::To(
                Position::numbered(2).unwrap()
            )))
        );
        // Binding a chord to what it already does changes nothing and is not a
        // refusal: a settings panel that re-applies a row is not an error.
        chords
            .bind(
                &shortcuts,
                chord(Key::Digit2),
                Switch::To(Position::numbered(2).unwrap()),
            )
            .unwrap();
        assert_eq!(chords.bound().count(), 11);

        // Moving *next desktop* to another chord leaves the old one free.
        chords
            .bind(&shortcuts, chord(Key::Period), Switch::Next)
            .unwrap();
        assert_eq!(chords.chord_for(Switch::Next), Some(chord(Key::Period)));
        assert_eq!(chords.switch_for(&shortcuts, chord(Key::PageDown)), None);
        assert_eq!(chords.bound().count(), 11);
    }

    /// A chord for a desktop no display can have is refused, and a switch can
    /// be left with no chord at all.
    #[test]
    fn a_desktop_beyond_the_cap_has_no_chord_and_a_switch_may_have_none() {
        let shortcuts = Shortcuts::shipped();
        let mut chords = DesktopChords::as_shipped().unwrap();
        let beyond = Position::numbered(u16::try_from(MOST_DESKTOPS).unwrap() + 1).unwrap();
        assert_eq!(
            chords.bind(&shortcuts, chord(Key::Minus), Switch::To(beyond)),
            Err(Refused::NoSuchPosition(beyond))
        );
        assert_eq!(chords.switch_for(&shortcuts, chord(Key::Minus)), None);

        assert!(chords.unbind(Switch::Next));
        assert!(!chords.unbind(Switch::Next));
        assert_eq!(chords.switch_for(&shortcuts, chord(Key::PageDown)), None);
        assert!(DesktopChords::none().bound().next().is_none());
    }
}
