//! What a machine can watch or listen with: the screen, the camera, the
//! microphone.
//!
//! Three, and the list is closed. `docs/features.md` promises ★ *a visible
//! indicator whenever the screen, camera or microphone is in use — by any
//! application, including ours*, and this is that sentence as a type. Anything
//! else a machine has — a disk, a network, a printer — is a different question
//! with a different answer; law 1's indicator is what says whether something
//! left the machine, and this one is about the room the person is sitting in.
//!
//! # Three here and twelve in a grant
//!
//! `alo_capability::Facility` (ADR 0040) has twelve, and three of them name
//! these: `Camera`, `Microphone`, `ScreenOnce` and `ScreenContinuously` — four,
//! because a grant to photograph the screen is not a grant to record it. That
//! distinction is right where it is and wrong here. Somebody glancing at their
//! machine is asking *is my screen being read*, and an indicator that answered
//! *continuously, rather than once* would be answering a question about
//! permission while they were asking a question about now.
//!
//! So the two lists are not the same list, neither is derived from the other,
//! and this one does not grow: a fourth thing here would be a fourth thing a
//! person has to watch for, and `docs/features.md` promises three.
//!
//! # It shows; it never decides
//!
//! Nothing here asks whether something *may* use the camera. That is
//! `alo-portals` judging a grant, and the plan's constraint for this task says
//! so outright. A [`Used`] is a fact about what is happening, and a fact does
//! not have a policy.

use alo_strings::Word;

use crate::mark::Mark;
use crate::position::Position;
use crate::words;

/// One thing a machine can watch or listen with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Used {
    /// The screen: something is reading what is on it.
    Screen,
    /// The camera.
    Camera,
    /// The microphone.
    Microphone,
}

impl Used {
    /// All three, in the order their lines are ordered.
    pub const EVERY: [Self; 3] = [Self::Screen, Self::Camera, Self::Microphone];

    /// The name this is written down by, where something has to be written
    /// down.
    ///
    /// Not something a person reads — what a person reads is the line's
    /// sentence, which names who as well as what.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::Camera => "camera",
            Self::Microphone => "microphone",
        }
    }

    /// The shape drawn beside a line about this, so that which of the three is
    /// in use never depends on telling one colour from another (ADR 0010).
    #[must_use]
    pub const fn mark(self) -> Mark {
        match self {
            Self::Screen => Mark::Rectangle,
            Self::Camera => Mark::Lens,
            Self::Microphone => Mark::Capsule,
        }
    }

    /// Where lines about this sit on the indicator.
    #[must_use]
    pub const fn position(self) -> Position {
        Position::of(self)
    }

    /// The string this crate declares for a line about this, with a gap for
    /// whoever is using it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Screen => words::THE_SCREEN,
            Self::Camera => words::THE_CAMERA,
            Self::Microphone => words::THE_MICROPHONE,
        }
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

    /// **The three are told apart three ways**: a name, a shape and a place.
    /// Any one of them collapsing would leave the colour of a line doing work
    /// ADR 0010 says a colour may never do alone.
    #[test]
    fn the_three_are_told_apart_by_name_by_shape_and_by_place() {
        let names: BTreeSet<&str> = Used::EVERY.iter().map(|used| used.named()).collect();
        assert_eq!(names.len(), Used::EVERY.len());

        let marks: BTreeSet<Mark> = Used::EVERY.iter().map(|used| used.mark()).collect();
        assert_eq!(marks.len(), Used::EVERY.len());

        let places: BTreeSet<Position> = Used::EVERY.iter().map(|used| used.position()).collect();
        assert_eq!(places.len(), Used::EVERY.len());
    }

    /// **Each has its own sentence**, and each of those sentences leaves a gap
    /// for who — so no line about a use can be shown without saying by what.
    #[test]
    fn each_has_its_own_sentence_with_a_gap_for_who() {
        let mut said = BTreeSet::new();
        for used in Used::EVERY {
            let word = used.word();
            assert!(said.insert(word.named()), "{used:?} shares a sentence");
            assert_eq!(
                word.phrase().unwrap().source().gaps(),
                [words::WHO.to_owned()],
                "{used:?}"
            );
        }
    }

    /// **The list is the one `docs/features.md` promised**, and it is closed —
    /// a fourth thing here is a fourth thing a person has to watch for, and
    /// this is where somebody adding one meets the promise.
    #[test]
    fn the_list_is_the_three_the_product_promised() {
        assert_eq!(Used::EVERY.len(), 3);
        assert_eq!(
            Used::EVERY.map(Used::named),
            ["screen", "camera", "microphone"]
        );
    }
}
