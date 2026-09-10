//! The plan's acceptance for *the agent overlay: one key, from anywhere*,
//! one test per criterion.
//!
//! `docs/autonomy/v0-01-delivery-plan.md`, task 2: *the chord resolves to the
//! action; pressing it asks for the surface exactly once; a second press
//! while it is open does not ask twice; and with no compositor there is a
//! refusal a person could read. No pixels are claimed and none are tested.*
//!
//! These tests cross the seam the way the shell will: `alo-shortcuts`
//! resolves the chord to [`alo_shortcuts::Action::TheAgent`], and the action
//! drives [`alo_overlay::Summoning`] against a compositor that counts what it
//! was asked — so *once* and *not twice* are numbers, not impressions.
//! Nothing here draws anything, and nothing asserts a pixel.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_overlay::{
    Compositor, NotSummoned, Pressed, Summoning, SurfaceRefused, SurfaceRequest, overlay_words,
};
use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts};
use alo_strings::Strings;

/// A compositor that counts every request it is asked to honour.
struct Counting {
    /// How many requests arrived, which the acceptance is about.
    asked: usize,
    /// What each request is answered with.
    answer: Result<(), SurfaceRefused>,
}

impl Compositor for Counting {
    fn honour(&mut self, _asked: SurfaceRequest) -> Result<(), SurfaceRefused> {
        self.asked = self.asked.saturating_add(1);
        self.answer
    }
}

/// The chord a person actually presses, resolved through their bindings the
/// way the shell resolves it: `Super+A` as shipped.
fn the_shipped_chord() -> Chord {
    Shortcuts::shipped().chord_for(Action::TheAgent).unwrap()
}

/// **The chord resolves to the action.** `Super+A`, through the same
/// [`Shortcuts`] every other chord goes through, means summon the agent —
/// and it is the shipped binding, so the key works before anybody changes
/// anything.
#[test]
fn the_chord_resolves_to_summoning_the_agent() {
    let shortcuts = Shortcuts::shipped();
    let super_a = Chord::checked(Modifiers::just(Modifier::Super), Key::A).unwrap();
    assert_eq!(shortcuts.action_for(super_a), Some(Action::TheAgent));
    assert_eq!(shortcuts.chord_for(Action::TheAgent), Some(super_a));
}

/// **Pressing it asks for the surface exactly once.** The resolved action
/// drives the summoning, the compositor honours, and the count of requests
/// is one — not zero (a key that does nothing) and not more.
#[test]
fn pressing_it_asks_for_the_surface_exactly_once() {
    let shortcuts = Shortcuts::shipped();
    assert_eq!(
        shortcuts.action_for(the_shipped_chord()),
        Some(Action::TheAgent),
        "the press below is not the agent's key"
    );

    let mut compositor = Counting {
        asked: 0,
        answer: Ok(()),
    };
    let mut summoning = Summoning::closed();
    assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
    assert_eq!(compositor.asked, 1);
    assert!(summoning.is_open());
}

/// **A second press while it is open does not ask twice.** The count stays
/// at one, and the second press says the overlay is already there rather
/// than raising a duplicate request or failing.
#[test]
fn a_second_press_while_it_is_open_does_not_ask_twice() {
    let mut compositor = Counting {
        asked: 0,
        answer: Ok(()),
    };
    let mut summoning = Summoning::closed();
    assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
    assert_eq!(
        summoning.press(Some(&mut compositor)),
        Pressed::AlreadySummoned
    );
    assert_eq!(compositor.asked, 1, "the overlay was asked for twice");

    // Dismissed and pressed again is a new summons on purpose: once per
    // opening, not once per boot.
    summoning.dismissed().unwrap();
    assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
    assert_eq!(compositor.asked, 2);
}

/// **With no compositor there is a refusal a person could read** — a whole
/// sentence from this crate's own declared words, not a silent nothing and
/// not a debug string. And the refusal is not the end: the same summoning
/// summons the moment a compositor is back.
#[test]
fn with_no_compositor_there_is_a_refusal_a_person_could_read() {
    let mut summoning = Summoning::closed();
    let pressed = summoning.press(None);
    assert_eq!(pressed, Pressed::Refused(NotSummoned::NoCompositor));
    assert!(!summoning.is_open());

    let strings = Strings::of(overlay_words().unwrap());
    let said = NotSummoned::NoCompositor.said(&strings);
    assert!(!said.is_a_bug(), "the refusal is not declared");
    assert_eq!(
        said.text(),
        "The agent has nowhere to appear: the desktop is not running. Sign in to the desktop \
         and press the key again"
    );

    // A compositor arrives; the key was refused, not broken.
    let mut compositor = Counting {
        asked: 0,
        answer: Ok(()),
    };
    assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
    assert_eq!(compositor.asked, 1);
}

/// The other nowhere: **a compositor that refuses is a sentence too**, the
/// request really was made — once — and the summoning stays closed so the
/// next press may try again.
#[test]
fn a_compositor_with_nothing_to_show_on_refuses_in_words() {
    let mut compositor = Counting {
        asked: 0,
        answer: Err(SurfaceRefused::NothingToShowOn),
    };
    let mut summoning = Summoning::closed();
    let pressed = summoning.press(Some(&mut compositor));
    assert_eq!(
        pressed,
        Pressed::Refused(NotSummoned::Surface(SurfaceRefused::NothingToShowOn))
    );
    assert_eq!(compositor.asked, 1, "the compositor was never asked");
    assert!(!summoning.is_open());

    let strings = Strings::of(overlay_words().unwrap());
    let said = NotSummoned::Surface(SurfaceRefused::NothingToShowOn).said(&strings);
    assert!(!said.is_a_bug());
    assert_eq!(
        said.text(),
        "The agent has nowhere to appear: no screen is connected. Connect a screen and press \
         the key again"
    );

    // A screen arrives and the same key summons: refusal is recoverable.
    compositor.answer = Ok(());
    assert_eq!(summoning.press(Some(&mut compositor)), Pressed::Summoned);
    assert_eq!(compositor.asked, 2);
}
