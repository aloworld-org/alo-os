//! Reaching the Dock when a window is filling the screen.
//!
//! True full screen covers the Dock. Moving to the bottom edge reveals it over
//! the content, and **the pointer can travel onto the revealed Dock and click
//! without it disappearing on the way.**
//!
//! # That last clause is the whole of this file
//!
//! It reads like a detail and it is the entire difficulty. The obvious
//! implementation reveals while the pointer is in a strip at the bottom edge and
//! hides when it leaves — and the Dock is drawn *above* that strip, so moving
//! towards the thing that just appeared makes it vanish. Everybody has used a
//! system that does this.
//!
//! So [`Revealing`] is a small state machine rather than a predicate on a
//! pointer position. Once revealed, what keeps it revealed is **the strip or the
//! Dock**, and the only thing that hides it is leaving both.
//!
//! # Nothing here is a timer
//!
//! No delay before revealing, no grace period after leaving. A timer would make
//! this file's behaviour depend on how fast somebody moves, which is exactly
//! what a person with a tremor or a trackball cannot control — and a grace
//! period is the usual patch for the bug this state machine does not have.
//!
//! Whether the reveal is animated is the compositor's, read from the person's
//! motion preference.

/// Where the pointer is, as far as revealing is concerned.
///
/// Three places rather than a coordinate: this file should not know how tall the
/// Dock is or where the strip ends, because then two files would have opinions
/// about the same edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThePointer {
    /// In the strip along the very bottom edge that asks for the Dock.
    AtTheBottomEdge,
    /// Over the Dock itself, while it is revealed.
    OnTheDock,
    /// Anywhere else.
    Elsewhere,
}

/// Whether the Dock is revealed over a full-screen window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Revealing {
    /// Covered by the full-screen window, as it is at rest.
    #[default]
    Covered,
    /// Drawn over the window.
    Revealed,
}

impl Revealing {
    /// Whether the Dock is on the screen at this moment.
    #[must_use]
    pub const fn is_revealed(self) -> bool {
        matches!(self, Self::Revealed)
    }

    /// Where things stand after the pointer moves here.
    ///
    /// **Once revealed, the Dock itself keeps it revealed.** That is the clause
    /// that stops it vanishing from under a pointer travelling towards it, and
    /// it is why this takes the previous state rather than answering from the
    /// position alone.
    #[must_use]
    pub const fn the_pointer_is(self, now: ThePointer) -> Self {
        match (self, now) {
            // Asking for it, from anywhere.
            (_, ThePointer::AtTheBottomEdge) => Self::Revealed,
            // Already there and the pointer is on it: keep it.
            (Self::Revealed, ThePointer::OnTheDock) => Self::Revealed,
            // On the Dock while it is covered cannot happen — there is nothing
            // to be on — and answering `Covered` says so rather than inventing
            // a reveal nobody asked for.
            (Self::Covered, ThePointer::OnTheDock) | (_, ThePointer::Elsewhere) => Self::Covered,
        }
    }

    /// The window stopped filling the screen, so there is nothing to reveal
    /// over: the Dock is simply there.
    #[must_use]
    pub const fn full_screen_ended() -> Self {
        Self::Covered
    }
}

/// The keyboard's road to the same place.
///
/// **A person who cannot point still reaches the Dock.** Focusing it reveals it
/// and keeps it revealed while it holds focus, and Escape gives it up — which is
/// the same shape as the pointer's *stay while you are on it*, so neither road
/// is a consolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheKeyboard {
    /// The Dock has keyboard focus.
    IsOnTheDock,
    /// It does not.
    IsElsewhere,
}

impl Revealing {
    /// Where things stand when the keyboard moves.
    #[must_use]
    pub const fn the_keyboard(self, now: TheKeyboard) -> Self {
        match now {
            TheKeyboard::IsOnTheDock => Self::Revealed,
            TheKeyboard::IsElsewhere => Self::Covered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The bottom edge reveals it**, from anywhere, which is how a person
    /// asks.
    #[test]
    fn the_bottom_edge_reveals_it() {
        let covered = Revealing::Covered;
        assert!(!covered.is_revealed());
        assert!(
            covered
                .the_pointer_is(ThePointer::AtTheBottomEdge)
                .is_revealed()
        );
    }

    /// **The pointer can travel onto it without it vanishing on the way.** The
    /// whole reason this is a state machine: the obvious version hides the Dock
    /// the moment the pointer leaves the strip, which is the moment it reaches
    /// the Dock.
    #[test]
    fn the_pointer_can_move_from_the_edge_onto_the_dock_and_it_stays() {
        let revealed = Revealing::Covered.the_pointer_is(ThePointer::AtTheBottomEdge);
        let still = revealed.the_pointer_is(ThePointer::OnTheDock);
        assert!(still.is_revealed(), "it vanished from under the pointer");

        // And it goes on staying while the pointer is on it — moving between
        // icons does not dismiss it.
        assert!(
            still
                .the_pointer_is(ThePointer::OnTheDock)
                .the_pointer_is(ThePointer::OnTheDock)
                .is_revealed()
        );
    }

    /// **Leaving both hides it**, which is the only thing that does.
    #[test]
    fn leaving_the_edge_and_the_dock_hides_it() {
        let revealed = Revealing::Covered.the_pointer_is(ThePointer::AtTheBottomEdge);
        assert!(!revealed.the_pointer_is(ThePointer::Elsewhere).is_revealed());

        let from_the_dock = revealed.the_pointer_is(ThePointer::OnTheDock);
        assert!(
            !from_the_dock
                .the_pointer_is(ThePointer::Elsewhere)
                .is_revealed()
        );
    }

    /// **Coming back to the edge reveals it again**, however many times.
    #[test]
    fn it_can_be_asked_for_again_and_again() {
        let mut state = Revealing::Covered;
        for _ in 0..3 {
            state = state.the_pointer_is(ThePointer::AtTheBottomEdge);
            assert!(state.is_revealed());
            state = state.the_pointer_is(ThePointer::Elsewhere);
            assert!(!state.is_revealed());
        }
    }

    /// Being told the pointer is on a Dock that is not revealed answers
    /// *covered* rather than inventing a reveal: there was nothing to be on.
    #[test]
    fn being_on_a_dock_that_is_not_there_reveals_nothing() {
        assert!(
            !Revealing::Covered
                .the_pointer_is(ThePointer::OnTheDock)
                .is_revealed()
        );
    }

    /// **The keyboard reaches the same place.** Focus reveals it and holds it,
    /// and giving focus up gives it back — the same shape as the pointer's, so
    /// neither road is a lesser one.
    #[test]
    fn the_keyboard_reveals_it_and_holds_it_the_same_way() {
        let revealed = Revealing::Covered.the_keyboard(TheKeyboard::IsOnTheDock);
        assert!(revealed.is_revealed());
        assert!(
            revealed
                .the_keyboard(TheKeyboard::IsOnTheDock)
                .is_revealed(),
            "it stays while it holds focus"
        );
        assert!(
            !revealed
                .the_keyboard(TheKeyboard::IsElsewhere)
                .is_revealed()
        );
    }

    /// When the window stops filling the screen there is nothing to reveal over,
    /// so the state goes back to where it starts.
    #[test]
    fn leaving_full_screen_ends_the_whole_question() {
        assert!(!Revealing::full_screen_ended().is_revealed());
        assert_eq!(Revealing::full_screen_ended(), Revealing::default());
    }
}
