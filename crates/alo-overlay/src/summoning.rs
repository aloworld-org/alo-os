//! The key's half of the seam: what one press does, and what a second one
//! does not.
//!
//! [`Summoning`] is the state the chord drives. It exists so that the two
//! promises in the plan's acceptance are a machine rather than a habit:
//! pressing the key asks the compositor for the surface **exactly once**, and
//! a second press while the overlay is open asks **nothing**. Both are easy to
//! get right on the happy path and easy to lose in a refactor, which is why
//! they live in one small type with its own tests instead of inline in an
//! input handler.
//!
//! # What a press can come back as
//!
//! Three things, and every one of them is something rather than nothing: the
//! overlay opened, the overlay was already open, or a refusal with a sentence
//! in it. A key that silently does nothing is the worst outcome available —
//! the plan says so in as many words — so there is no path through
//! [`Summoning::press`] that returns nothing to act on.

use crate::refusing::NotSummoned;
use crate::surface::{Compositor, SurfaceRequest};

/// Where the overlay is, from the key's point of view.
///
/// Two states rather than three: the compositor answers inside the press
/// (see [`crate::surface::Compositor`] for why it can), so there is no
/// *asked and waiting* to be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Nothing is summoned; the next press asks.
    Closed,
    /// The compositor honoured a request and the overlay is up; a press asks
    /// nothing until it is dismissed.
    Open,
}

/// What one press of the chord did.
#[derive(Debug, PartialEq, Eq)]
pub enum Pressed {
    /// The compositor was asked — once — and said yes: the overlay is open.
    Summoned,
    /// The overlay was already open, so nothing was asked. Not an error: the
    /// person's wish is already granted, and asking again would be the double
    /// request the acceptance forbids.
    AlreadySummoned,
    /// The agent could not be summoned, and this is what to put in front of
    /// the person — in their language, through [`NotSummoned::said`].
    Refused(NotSummoned),
}

/// A dismissal was reported for an overlay that was not open.
///
/// A compositor and this model disagreeing about whether the overlay exists
/// is a bug in the wiring, not a thing a person did, so this keeps its
/// English and its `Display` the way `alo_shortcuts::DefaultsError` does:
/// whoever reads it is fixing the shell.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("the agent overlay was reported dismissed while it was not open")]
pub struct NotOpen;

/// The agent overlay's summoning: the state one key drives.
///
/// One per seat that can press the chord — in v0.01 that is one per machine.
/// It holds no compositor and draws nothing; the compositor is handed in at
/// the moment of the press, because whether there is one is a fact about the
/// session that can change while the machine runs.
#[derive(Debug)]
pub struct Summoning {
    /// Open or closed; private so that every way in is a method with a rule.
    state: State,
}

impl Summoning {
    /// Where every session starts: nothing summoned.
    #[must_use]
    pub fn closed() -> Self {
        Self {
            state: State::Closed,
        }
    }

    /// Whether the overlay is up — what the compositor was last told to do,
    /// not a claim about pixels.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.state == State::Open
    }

    /// The chord was pressed.
    ///
    /// With a compositor and the overlay closed, this asks it for the surface
    /// — creating the one [`SurfaceRequest`] this press will ever make — and
    /// opens on a yes. With the overlay already open, it asks nothing at all.
    /// With no compositor there is nowhere for the agent to appear, so the
    /// answer is a refusal with a sentence in it rather than silence —
    /// and anything this model believed was open went with the compositor, so
    /// it closes: the alternative is a machine whose key is dead forever
    /// because a compositor crashed while the overlay was up.
    pub fn press(&mut self, compositor: Option<&mut dyn Compositor>) -> Pressed {
        let Some(compositor) = compositor else {
            self.state = State::Closed;
            return Pressed::Refused(NotSummoned::NoCompositor);
        };
        if self.state == State::Open {
            return Pressed::AlreadySummoned;
        }
        match compositor.honour(SurfaceRequest::asked()) {
            Ok(()) => {
                self.state = State::Open;
                Pressed::Summoned
            }
            Err(refused) => Pressed::Refused(NotSummoned::Surface(refused)),
        }
    }

    /// The overlay went away — the person closed it, or the compositor took
    /// it down — so the next press asks again.
    ///
    /// # Errors
    /// [`NotOpen`] when nothing was open: the compositor and this model
    /// disagree, which is the shell's bug to hear about rather than swallow.
    pub fn dismissed(&mut self) -> Result<(), NotOpen> {
        match self.state {
            State::Open => {
                self.state = State::Closed;
                Ok(())
            }
            State::Closed => Err(NotOpen),
        }
    }
}

impl Default for Summoning {
    fn default() -> Self {
        Self::closed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::SurfaceRefused;

    /// A compositor that counts what it was asked, so the once-only promises
    /// are numbers rather than impressions.
    struct Counting {
        /// How many requests arrived.
        asked: usize,
        /// What every request is answered with.
        answer: Result<(), SurfaceRefused>,
    }

    impl Counting {
        /// A compositor with room for the overlay.
        fn with_room() -> Self {
            Self {
                asked: 0,
                answer: Ok(()),
            }
        }

        /// A compositor with no screen to show anything on.
        fn with_no_screen() -> Self {
            Self {
                asked: 0,
                answer: Err(SurfaceRefused::NothingToShowOn),
            }
        }
    }

    impl Compositor for Counting {
        fn honour(&mut self, _asked: SurfaceRequest) -> Result<(), SurfaceRefused> {
            self.asked = self.asked.saturating_add(1);
            self.answer
        }
    }

    /// **One press, one request, and the overlay is open.** The happy path,
    /// first, so the refusals below are about their own conditions.
    #[test]
    fn a_press_asks_once_and_opens() {
        let mut compositor = Counting::with_room();
        let mut summoning = Summoning::closed();
        assert!(!summoning.is_open());
        assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
        assert_eq!(compositor.asked, 1);
        assert!(summoning.is_open());
    }

    /// **A second press while it is open asks nothing at all** — the count
    /// stays where it was, which is the acceptance's *does not ask twice*
    /// measured rather than assumed.
    #[test]
    fn a_second_press_while_open_asks_nothing() {
        let mut compositor = Counting::with_room();
        let mut summoning = Summoning::closed();
        assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
        assert_eq!(
            summoning.press(Some(&mut compositor)),
            Pressed::AlreadySummoned
        );
        assert_eq!(compositor.asked, 1, "a second request was made");
        assert!(summoning.is_open());
    }

    /// After a dismissal the key works again: the next press is a new
    /// summons with a new request, not a stale *already open*.
    #[test]
    fn after_a_dismissal_the_next_press_asks_again() {
        let mut compositor = Counting::with_room();
        let mut summoning = Summoning::closed();
        assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
        assert_eq!(summoning.dismissed(), Ok(()));
        assert!(!summoning.is_open());
        assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
        assert_eq!(compositor.asked, 2);
    }

    /// **A dismissal for an overlay that is not open is refused**, in the
    /// diagnostic English the shell's own bugs are reported in.
    #[test]
    fn a_dismissal_with_nothing_open_is_refused() {
        let mut summoning = Summoning::closed();
        let refused = summoning.dismissed();
        assert_eq!(refused, Err(NotOpen));
        assert_eq!(
            NotOpen.to_string(),
            "the agent overlay was reported dismissed while it was not open"
        );
    }

    /// **With no compositor the answer is a refusal, not silence**, and the
    /// overlay is closed — so nothing is left half-open for the next press to
    /// trip over.
    #[test]
    fn no_compositor_refuses_in_words_and_closes_nothing_quietly() {
        let mut summoning = Summoning::closed();
        assert_eq!(
            summoning.press(None),
            Pressed::Refused(NotSummoned::NoCompositor)
        );
        assert!(!summoning.is_open());
    }

    /// An overlay that was open when its compositor went away is closed by
    /// the press that discovers it: the person is refused in words this once,
    /// and the key is alive again the moment a compositor is back — rather
    /// than dead forever behind a stale `Open`.
    #[test]
    fn an_open_overlay_whose_compositor_went_is_refused_and_reset() {
        let mut compositor = Counting::with_room();
        let mut summoning = Summoning::closed();
        assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);

        // The compositor goes away; the next press finds nobody to ask.
        assert_eq!(
            summoning.press(None),
            Pressed::Refused(NotSummoned::NoCompositor)
        );
        assert!(!summoning.is_open());

        // And one is back: the key works, with a fresh request.
        assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
        assert_eq!(compositor.asked, 2);
    }

    /// **A refused surface leaves the summoning closed**: the overlay never
    /// opened, so the next press must ask again rather than believe it is up.
    #[test]
    fn a_refused_surface_leaves_it_closed_so_the_next_press_asks_again() {
        let mut compositor = Counting::with_no_screen();
        let mut summoning = Summoning::closed();
        assert_eq!(
            summoning.press(Some(&mut compositor)),
            Pressed::Refused(NotSummoned::Surface(SurfaceRefused::NothingToShowOn))
        );
        assert!(!summoning.is_open());

        // A screen arrives; the same summoning summons.
        let mut with_room = Counting::with_room();
        assert_eq!(summoning.press(Some(&mut with_room)), Pressed::Summoned);
        assert_eq!(with_room.asked, 1);
    }

    /// A fresh summoning is a closed one, whichever way it is built.
    #[test]
    fn a_summoning_starts_closed() {
        assert!(!Summoning::default().is_open());
        assert!(!Summoning::closed().is_open());
    }
}
