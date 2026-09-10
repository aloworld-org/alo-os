//! What is on the screen, and keeping it in step with what is leaving.
//!
//! [`Drawn`] is one moment. This is the thing that lives as long as the
//! session: it remembers what the compositor was last given, hands it a new
//! picture when the machine's own indicator has moved on, and asks it for
//! nothing when nothing has changed.
//!
//! # Three promises, and they are the same three the key's seam makes
//!
//! - **Nothing is ever shown that did not come off the machine's own
//!   indicator.** [`Indicating::show`] takes an `alo_egress::Indicator` and
//!   makes the picture itself; there is no way to hand it one.
//! - **One change, one redraw.** A picture identical to the last is not sent
//!   again, so a shell may call this on every frame and a compositor is only
//!   asked when the answer moved. That is the same *asks exactly once* the
//!   agent overlay's summoning holds, in the shape a surface that is always up
//!   needs it.
//! - **Nowhere to show it is a sentence, not silence.** An indicator that
//!   failed to appear and said nothing would leave a machine looking exactly
//!   like a machine on which nothing is leaving.
//!
//! # Why a refusal forgets what was drawn
//!
//! On any refusal this forgets the last picture, so the next call redraws from
//! nothing. The alternative is a shell that believes an old picture is still on
//! a screen that has gone — and then, when the screen comes back showing the
//! same count as before, sends nothing, and leaves a person looking at a dark
//! indicator that has not been asked a question since the display was
//! unplugged. Forgetting costs one redraw; remembering costs the guarantee.

use alo_egress::Indicator;

use crate::drawn::Drawn;
use crate::refusing::NotShown;
use crate::surface::Compositor;

/// What one call to [`Indicating::show`] did.
///
/// Three answers, and every one of them is something rather than nothing.
#[derive(Debug, PartialEq, Eq)]
pub enum Drew {
    /// The compositor was given a new picture: what is leaving this machine
    /// changed, or it was not on the screen at all.
    Shown,
    /// Nothing changed since the last picture, so the compositor was asked
    /// nothing. Not an error: the screen is already right.
    Unchanged,
    /// It could not be shown, and this is what to put in front of the person —
    /// in their language, through [`NotShown::said`].
    Refused(NotShown),
}

/// The egress indicator's place on the screen, for as long as a session lasts.
///
/// One per screen the indicator appears on; in v0.01 that is one per machine.
/// It holds no compositor, because whether there is one is a fact about the
/// session that can change while the machine runs — so it is handed in at the
/// moment of the call, exactly as the agent overlay's summoning takes one.
///
/// ```
/// use alo_capability::Grantee;
/// use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
/// use alo_indicator::{Compositor, Drawn, Drew, Indicating, NotShown, SurfaceRefused};
///
/// /// A compositor with a screen, keeping whatever it was last given.
/// #[derive(Default)]
/// struct Screen {
///     showing: Option<Drawn>,
/// }
/// impl Compositor for Screen {
///     fn show(&mut self, drawn: Drawn) -> Result<(), SurfaceRefused> {
///         self.showing = Some(drawn);
///         Ok(())
///     }
/// }
///
/// let mut indicating = Indicating::nowhere();
/// let mut screen = Screen::default();
/// let mut indicator = Indicator::default();
///
/// // A quiet machine puts up a dark indicator, once.
/// assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
/// assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Unchanged);
/// assert!(screen.showing.as_ref().is_some_and(|drawn| drawn.lamp().is_dark()));
///
/// // Something leaves, and the next call lights it.
/// let departing = indicator.beginning(
///     &EgressPolicy::Anywhere,
///     Leaving::because(
///         &Grantee::named("@files"),
///         Why::Fetching,
///         Destination::at("alo.example").expect("a host that can be shown"),
///     ),
///     std::time::SystemTime::UNIX_EPOCH,
/// )?;
/// assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
/// assert!(screen.showing.as_ref().is_some_and(|drawn| drawn.lamp().is_lit()));
///
/// // And with no compositor, the answer is a sentence rather than silence.
/// assert_eq!(
///     indicating.show(None, &indicator),
///     Drew::Refused(NotShown::NoCompositor),
/// );
/// # indicator.ended(departing);
/// # Ok::<(), alo_egress::NotPermitted>(())
/// ```
#[derive(Debug, Default)]
pub struct Indicating {
    /// The picture the compositor was last given, or [`None`] when the
    /// indicator is not on a screen.
    showing: Option<Drawn>,
}

impl Indicating {
    /// Where every session starts: the indicator is on no screen.
    #[must_use]
    pub fn nowhere() -> Self {
        Self { showing: None }
    }

    /// Whether the indicator is on a screen — what the compositor was last
    /// told to show, not a claim about pixels.
    #[must_use]
    pub fn is_showing(&self) -> bool {
        self.showing.is_some()
    }

    /// What the compositor was last given, if it is showing anything.
    ///
    /// For a shell that has to answer *what does the screen say right now*
    /// without asking the machine's indicator again — a screen reader moving
    /// onto the light, or a test.
    #[must_use]
    pub fn showing(&self) -> Option<&Drawn> {
        self.showing.as_ref()
    }

    /// Put what is leaving this machine in front of the person.
    ///
    /// Takes the machine's own `alo_egress::Indicator` and makes the picture
    /// here, so nothing a caller assembled can reach a screen. With a
    /// compositor and something to say that has not been said, it hands the
    /// picture over; with nothing changed it asks nothing at all; with no
    /// compositor, or one that refuses, it answers with a refusal that has a
    /// sentence in it and forgets whatever it believed was on the screen.
    pub fn show(&mut self, compositor: Option<&mut dyn Compositor>, indicator: &Indicator) -> Drew {
        let Some(compositor) = compositor else {
            self.showing = None;
            return Drew::Refused(NotShown::NoCompositor);
        };
        let drawn = Drawn::of(indicator);
        if self.showing.as_ref() == Some(&drawn) {
            return Drew::Unchanged;
        }
        match compositor.show(drawn.clone()) {
            Ok(()) => {
                self.showing = Some(drawn);
                Drew::Shown
            }
            Err(refused) => {
                self.showing = None;
                Drew::Refused(NotShown::Surface(refused))
            }
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
    use crate::surface::SurfaceRefused;
    use crate::testing::{asking_a_provider, fetching, noon};
    use alo_egress::EgressPolicy;

    /// A compositor that counts what it was given and keeps the last of it, so
    /// *one change, one redraw* is a number rather than an impression.
    struct Counting {
        /// How many pictures arrived.
        shown: usize,
        /// The last one, when it accepted any.
        last: Option<Drawn>,
        /// What every request is answered with.
        answer: Result<(), SurfaceRefused>,
    }

    impl Counting {
        /// A compositor with a screen.
        fn with_a_screen() -> Self {
            Self {
                shown: 0,
                last: None,
                answer: Ok(()),
            }
        }

        /// A compositor with nothing to show anything on.
        fn with_no_screen() -> Self {
            Self {
                shown: 0,
                last: None,
                answer: Err(SurfaceRefused::NothingToShowOn),
            }
        }
    }

    impl Compositor for Counting {
        fn show(&mut self, drawn: Drawn) -> Result<(), SurfaceRefused> {
            self.shown = self.shown.saturating_add(1);
            if self.answer.is_ok() {
                self.last = Some(drawn);
            }
            self.answer
        }
    }

    /// **The indicator goes up on a quiet machine, dark**, which is the state
    /// this product is sold on being in all day. It is put up rather than left
    /// off: an indicator that only appeared when something was leaving would
    /// have nothing to say on the day it mattered, because a person cannot tell
    /// a dark light from a light that is not there.
    #[test]
    fn a_quiet_machine_shows_a_dark_indicator() {
        let mut screen = Counting::with_a_screen();
        let mut indicating = Indicating::nowhere();
        assert!(!indicating.is_showing());

        assert_eq!(
            indicating.show(Some(&mut screen), &Indicator::default()),
            Drew::Shown
        );
        assert_eq!(screen.shown, 1);
        assert!(indicating.is_showing());
        assert!(screen.last.unwrap().lamp().is_dark());
    }

    /// **Nothing changed, nothing redrawn.** A shell may ask on every frame;
    /// the compositor is only asked when the answer moved.
    #[test]
    fn a_machine_that_has_not_changed_is_not_redrawn() {
        let mut screen = Counting::with_a_screen();
        let mut indicating = Indicating::nowhere();
        let mut indicator = Indicator::default();

        assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
        for _ in 0..8 {
            assert_eq!(
                indicating.show(Some(&mut screen), &indicator),
                Drew::Unchanged
            );
        }
        assert_eq!(
            screen.shown, 1,
            "the screen was redrawn with the same thing"
        );

        // And the moment something leaves, it is redrawn exactly once.
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon())
            .unwrap();
        assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
        assert_eq!(
            indicating.show(Some(&mut screen), &indicator),
            Drew::Unchanged
        );
        assert_eq!(screen.shown, 2);
        assert!(indicator.ended(departing));
    }

    /// **A second departure is a change**, even though the lamp was already
    /// lit: the list under it grew, and a light that stopped redrawing once it
    /// was on would leave that list a moment behind the machine.
    #[test]
    fn a_second_departure_redraws_even_though_the_lamp_was_already_lit() {
        let mut screen = Counting::with_a_screen();
        let mut indicating = Indicating::nowhere();
        let mut indicator = Indicator::default();
        drop(indicator.beginning(&EgressPolicy::Anywhere, fetching(), noon()));
        assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);

        drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
        assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);
        assert_eq!(screen.shown, 2);
        assert_eq!(screen.last.unwrap().lines().len(), 2);
    }

    /// **With no compositor the answer is a refusal, not silence**, and nothing
    /// is left believing it is on a screen.
    #[test]
    fn no_compositor_refuses_in_words_and_believes_nothing() {
        let mut screen = Counting::with_a_screen();
        let mut indicating = Indicating::nowhere();
        assert_eq!(
            indicating.show(Some(&mut screen), &Indicator::default()),
            Drew::Shown
        );

        assert_eq!(
            indicating.show(None, &Indicator::default()),
            Drew::Refused(NotShown::NoCompositor)
        );
        assert!(!indicating.is_showing());
        assert_eq!(indicating.showing(), None);

        // A compositor is back: the indicator is drawn again rather than
        // treated as already up.
        assert_eq!(
            indicating.show(Some(&mut screen), &Indicator::default()),
            Drew::Shown
        );
        assert_eq!(screen.shown, 2);
    }

    /// **A compositor with no screen is refused in words**, and the next call
    /// asks again rather than believing an indicator is up.
    #[test]
    fn a_compositor_with_no_screen_is_refused_and_asked_again() {
        let mut nowhere = Counting::with_no_screen();
        let mut indicating = Indicating::nowhere();
        assert_eq!(
            indicating.show(Some(&mut nowhere), &Indicator::default()),
            Drew::Refused(NotShown::Surface(SurfaceRefused::NothingToShowOn))
        );
        assert!(!indicating.is_showing());
        assert_eq!(nowhere.shown, 1);

        // Asked again with the same unchanged machine, it really asks: a
        // refusal is not a picture that is still up.
        assert_eq!(
            indicating.show(Some(&mut nowhere), &Indicator::default()),
            Drew::Refused(NotShown::Surface(SurfaceRefused::NothingToShowOn))
        );
        assert_eq!(nowhere.shown, 2);

        // And a screen arrives: the same state is drawn without waiting for
        // anything about the machine to change.
        let mut screen = Counting::with_a_screen();
        assert_eq!(
            indicating.show(Some(&mut screen), &Indicator::default()),
            Drew::Shown
        );
        assert_eq!(screen.shown, 1);
    }

    /// **A screen that went away while something was leaving is redrawn
    /// lit**, and not left dark because the count happens to match what was on
    /// it before. This is why a refusal forgets: the alternative is a person
    /// looking at an indicator that has not been asked a question since their
    /// display was unplugged.
    #[test]
    fn a_screen_that_came_back_is_redrawn_rather_than_assumed() {
        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, fetching(), noon())
            .unwrap();

        let mut screen = Counting::with_a_screen();
        let mut indicating = Indicating::nowhere();
        assert_eq!(indicating.show(Some(&mut screen), &indicator), Drew::Shown);

        // The display goes away and comes back with nothing about the machine
        // having changed.
        assert_eq!(
            indicating.show(None, &indicator),
            Drew::Refused(NotShown::NoCompositor)
        );
        let mut back = Counting::with_a_screen();
        assert_eq!(indicating.show(Some(&mut back), &indicator), Drew::Shown);
        assert!(back.last.unwrap().lamp().is_lit());
        assert!(indicator.ended(departing));
    }

    /// A fresh [`Indicating`] is on no screen, whichever way it is built.
    #[test]
    fn a_fresh_indicating_is_on_no_screen() {
        assert!(!Indicating::default().is_showing());
        assert!(!Indicating::nowhere().is_showing());
    }
}
