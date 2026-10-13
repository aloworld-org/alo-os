//! That a task window's colours come from roles, that alo's teal is never the
//! person's accent, and that the contrast figures this file records still hold.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a look that will not make a palette is the failure being reported"
)]

use alo_access::{Setting, TurnedOn};
use alo_appearance::{Accent, Appearance, Role, TimeOfDay};
use alo_strings::Direction;

use super::TaskPalette;
use crate::desktop_look::DesktopLook;

/// A look with this accent, and these accessibility settings turned on.
fn with(accent: Accent, turned_on: &TurnedOn) -> DesktopLook {
    let mut appearance = Appearance::shipped();
    appearance.set_accent(accent);
    DesktopLook::of(
        &appearance,
        turned_on,
        TimeOfDay::checked(10, 0).expect("ten in the morning"),
        Direction::LeftToRight,
        &crate::desktop_testing::a_display(),
    )
}

/// The palette for this accent, at the ordinary contrast.
fn palette(accent: Accent) -> TaskPalette {
    TaskPalette::of(with(accent, &TurnedOn::nothing())).expect("an accent a person may choose")
}

/// The palette for this accent with high contrast turned on.
fn at_high_contrast(accent: Accent) -> TaskPalette {
    let mut turned_on = TurnedOn::nothing();
    turned_on.turn_on(Setting::HighContrast);
    TaskPalette::of(with(accent, &turned_on)).expect("an accent a person may choose")
}

/// **Alo's colour is never the accent a person chose.**
///
/// ADR 0067 reserves deep teal for alo acting. The window would look right on a
/// machine whose owner happened to pick something tealish, which is exactly why
/// this is held by a test: drawn for every accent there is, and the teal does
/// not move.
#[test]
fn alos_colour_is_never_the_accent_a_person_chose() {
    let teal = Role::AccentDefault.colour();
    let teal = [teal.red(), teal.green(), teal.blue()];

    let mut seen = Vec::new();
    for accent in Accent::ALL {
        let drawn = palette(accent);
        assert_eq!(
            drawn.alo, teal,
            "{accent:?} moved alo's colour off the reserved teal"
        );
        seen.push(drawn);
    }

    assert_eq!(
        seen.len(),
        4,
        "all four accents must be drawn, or this proves little"
    );
    let first = seen.first().expect("four accents were drawn");
    assert!(
        seen.iter().all(|drawn| drawn.alo == first.alo),
        "alo's colour differed between two accents"
    );
}

/// **Deep teal is not an accent at all**, which is the mechanism the test above
/// relies on: a person cannot choose it, so there is no accent for alo's colour
/// to collide with.
#[test]
fn deep_teal_is_not_a_colour_a_person_may_choose() {
    let teal = alo_appearance::Token::DeepTeal.colour();

    assert!(
        Accent::of_colour(teal).is_err(),
        "deep teal became choosable, so ADR 0067's reservation is gone"
    );
}

/// **Every colour comes from a role, so a contrast setting reaches them.**
///
/// Asserted by changing the setting and watching the quiet ones move: a palette
/// built from hexes would be identical at both contrasts.
#[test]
fn a_contrast_setting_reaches_the_quiet_colours() {
    let ordinary = palette(Accent::Indigo);

    let high = at_high_contrast(Accent::Indigo);

    assert_ne!(
        ordinary.edge, high.edge,
        "the hairline did not change at high contrast, so it is not a role"
    );
    assert_ne!(
        ordinary.ink_quiet, high.ink_quiet,
        "the quiet ink did not change at high contrast"
    );
    assert_eq!(
        ordinary.alo, high.alo,
        "alo's teal must not be substituted: it measures 6.24:1 on white and \
         carries the one signal that says alo is acting"
    );
}

/// **The one button inverts, and nothing else does.**
///
/// The ink on the surface everywhere, except the one thing a person presses to
/// act, where it is the surface on the ink.
#[test]
fn only_the_one_button_inverts() {
    let drawn = palette(Accent::Moss);

    assert_eq!(drawn.the_one_button, drawn.ink);
    assert_eq!(drawn.on_the_one_button, drawn.surface);
    assert_ne!(
        drawn.the_one_button, drawn.on_the_one_button,
        "the button and its arrow are the same colour, so the arrow is invisible"
    );
}

/// **The contrast figures this file records still hold.**
///
/// Recomputed here rather than trusted, because the module's table is a
/// measurement taken once and a change to any of three roles makes it stale
/// with nothing to say so. This is that *something to say so*, for the two
/// pairs that matter most: alo's teal, and the narrowest pair in the window.
#[test]
fn the_contrast_ratios_the_module_records_are_still_true() {
    fn luminance([r, g, b]: [u8; 3]) -> f64 {
        let channel = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
    }
    fn ratio(a: [u8; 3], b: [u8; 3]) -> f64 {
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    let drawn = palette(Accent::Indigo);

    let teal_on_surface = ratio(drawn.alo, drawn.surface);
    assert!(
        teal_on_surface >= 4.5,
        "alo's teal fell below AA on the window's ground: {teal_on_surface:.2}:1"
    );
    assert!(
        (teal_on_surface - 6.24).abs() < 0.01,
        "the module says alo's teal measures 6.24:1 on white and it now \
         measures {teal_on_surface:.2}:1 — the table is stale"
    );

    let quiet_on_cool = ratio(drawn.ink_quiet, drawn.cool);
    assert!(
        quiet_on_cool >= 4.5,
        "the narrowest pair in the window fell below AA: {quiet_on_cool:.2}:1"
    );
    assert!(
        (quiet_on_cool - 4.91).abs() < 0.01,
        "the module says the quiet ink on bg/cool measures 4.91:1 and it now \
         measures {quiet_on_cool:.2}:1 — the table is stale"
    );

    // And the ink itself, which everything else is read against.
    assert!(
        ratio(drawn.ink, drawn.surface) >= 7.0,
        "the ink fell below AAA"
    );
}
