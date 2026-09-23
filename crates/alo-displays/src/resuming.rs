//! The desk a machine wakes up at: what a resume answers with when the screens
//! in front of the person are not the ones the machine went to sleep with.
//!
//! [`crate::Attached`] learns about screens one cable at a time —
//! [`crate::Attached::unplugged`] and [`crate::Attached::plugged_in`] — and **a
//! machine that was asleep saw none of them**. It wakes holding the set it went
//! to sleep with: two screens that are no longer plugged into anything, or one
//! screen where there are now three. A laptop closed at home and opened at the
//! office is the everyday shape of it.
//!
//! So there is one door that asks the whole question again from **what is
//! reported now** rather than from the events nobody was awake for:
//! [`crate::Attached::resumed_to`], which takes the whole reported set and
//! answers with a [`Resumed`].
//!
//! # Nothing new is decided here
//!
//! Every rule this uses was decided when the screens were: a set the person has
//! arranged is restored, a set nobody has arranged is laid out side by side at
//! each screen's own size, and what was on a screen that has gone belongs on
//! the main screen of what remains. This file does not add a layout rule and
//! must not grow one. What it adds is **asking all of it at once**, and one
//! sentence — [`crate::Note::TheDeskChanged`] — that tells a person the desk
//! changed rather than leaving them to go looking for a window.
//!
//! # A screen that is still there is not touched
//!
//! A machine that wakes to the same screens reporting themselves the same way
//! moves nothing, says nothing and changes nothing: the arrangement it was
//! already holding is the arrangement, down to the places in it. *Nothing
//! happened* is the commonest resume there is, and a sentence about it every
//! morning is a sentence nobody reads.
//!
//! # And a machine that wakes to nothing keeps what it had
//!
//! A resume that reports no screens at all is refused with
//! [`crate::NotArranged::NoScreens`], and the set the machine went to sleep
//! with is left exactly as it was. A machine briefly seeing nothing — a dock
//! that has not woken yet, a monitor still negotiating — must not be a machine
//! that has forgotten which screens it had, because the arrangement it is
//! holding is what the next resume is compared against.
//!
//! # Where a window goes is still the shell's to carry out
//!
//! There is no window identifier in this file, as there is none in
//! [`crate::coming_and_going`]. A [`Moved`] says *what was open on that screen
//! belongs on this one*; moving it is drawing, and drawing is the shell's.

use crate::coming_and_going::{CameBack, Moved};
use crate::notes::Note;

/// What a resume found, and what it means for what was open.
///
/// [`Resumed::the_same_desk`] is the ordinary morning: nothing moved and
/// nothing to say. Anything else names every screen that has gone and where
/// what was on it belongs, every screen that has come back, and the one note
/// the person reads.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "a screen that has gone has windows on it, and the shell is what moves them"]
pub struct Resumed {
    /// Every screen that went while the machine was asleep, and where what was
    /// on each belongs now.
    moved: Vec<Moved>,
    /// Every screen that is back and was away when the machine slept, and
    /// where what was on it has been sitting.
    came_back: Vec<CameBack>,
    /// What the person is told, where there is anything to tell them.
    note: Option<Note>,
}

impl Resumed {
    /// The machine woke to exactly the screens it went to sleep with.
    pub(crate) const fn the_same_desk_as_before() -> Self {
        Self {
            moved: Vec::new(),
            came_back: Vec::new(),
            note: None,
        }
    }

    /// The machine woke to a different set of screens.
    pub(crate) const fn at_another_desk(moved: Vec<Moved>, came_back: Vec<CameBack>) -> Self {
        Self {
            moved,
            came_back,
            note: Some(Note::TheDeskChanged),
        }
    }

    /// Whether the screens are the ones the machine went to sleep with, in
    /// which case nothing moved and nothing is said.
    #[must_use]
    pub const fn the_same_desk(&self) -> bool {
        self.note.is_none()
    }

    /// Every screen that has gone, and where what was open on each belongs
    /// now — the same answer an unplugged cable gives.
    pub fn moved(&self) -> impl Iterator<Item = &Moved> {
        self.moved.iter()
    }

    /// Every screen that is back and had been away, and what goes back to it.
    pub fn came_back(&self) -> impl Iterator<Item = &CameBack> {
        self.came_back.iter()
    }

    /// What the person is told about the desk itself, where there is anything
    /// to tell them.
    ///
    /// The screens' own notes — a screen nobody has sized, an arrangement that
    /// no longer fits — are [`crate::Attached::notes`] as they always are, and
    /// this one is at the front of them.
    #[must_use]
    pub const fn note(&self) -> Option<&Note> {
        self.note.as_ref()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, the_home_screen, the_laptop, the_office_screen};
    use crate::words;

    /// **The ordinary morning has nothing in it**, which is the shape the
    /// caller reads before it reads anything else.
    #[test]
    fn the_same_desk_carries_nothing_and_says_nothing() {
        let resumed = Resumed::the_same_desk_as_before();
        assert!(resumed.the_same_desk());
        assert_eq!(resumed.moved().count(), 0);
        assert_eq!(resumed.came_back().count(), 0);
        assert_eq!(resumed.note(), None);
    }

    /// **A different desk says one declared sentence**, with every gap filled,
    /// and carries the screens that went in the shape an unplug answers with.
    #[test]
    fn another_desk_says_one_declared_sentence_and_names_what_moved() {
        let strings = in_english();
        let resumed = Resumed::at_another_desk(
            vec![Moved::of(the_office_screen(), the_laptop(), Vec::new())],
            vec![CameBack::of(the_home_screen(), Some(the_laptop()))],
        );
        assert!(!resumed.the_same_desk());
        assert_eq!(resumed.note(), Some(&Note::TheDeskChanged));

        let said = resumed.note().unwrap().said(&strings);
        assert!(words::EVERY_WORD.contains(&Note::TheDeskChanged.word()));
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("went to sleep"), "{said}");

        assert_eq!(resumed.moved().count(), 1);
        let moved = resumed.moved().next().unwrap();
        assert_eq!(moved.from(), &the_office_screen());
        assert_eq!(moved.onto(), &the_laptop());
        assert_eq!(resumed.came_back().count(), 1);
    }
}
