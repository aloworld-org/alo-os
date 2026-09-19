//! Night light pulls every colour on a screen towards orange. Terracotta is
//! already orange, and it is the one colour in this system that means
//! something.
//!
//! ADR 0010 reserves terracotta for the agent — present, acting, or waiting for
//! an approval — and keeps all five personal accents thirty degrees of hue away
//! from it so that none of them can be mistaken for it. That measurement is
//! made on a screen nobody has warmed. Turn night light on and the whole
//! palette moves: blue goes first, then green, and every hue on the screen
//! slides towards the red end. The rose accent is the one that slides furthest,
//! because it is the accent nearest terracotta already.
//!
//! So *terracotta means the agent* is a promise that has to be re-measured at
//! every warmth a person can choose, and that is what this file does.
//!
//! # Hue distance is the wrong measure here, and this file says so rather than
//! quietly using it
//!
//! `alo_appearance::accent`'s own test asks how far each accent sits from
//! terracotta on the colour wheel, and thirty degrees is its floor. Under night
//! light that measure collapses for everybody: warming takes blue away, so
//! magenta, pink and red all converge on orange and the *wheel* distance
//! between terracotta and rose goes to nothing. What does **not** collapse is
//! that they are still plainly different colours — a bright warm orange and a
//! dark dull one — and the measure for that is a distance in a space built so
//! that equal steps look equally different. This file uses CIE76 ΔE\*ab, whose
//! just-noticeable difference is about 2.3; [`FAR_ENOUGH`] is 5.0, which is
//! twice that and is the distance at which two colours are routinely called
//! two colours rather than two printings of one.
//!
//! # And this is one of the two things bounding the warm end of the range
//!
//! `alo_displays::Warmth::WARMEST` is a number this test is entitled to refuse.
//! It does not refuse today's: at the warmest setting a person can choose, the
//! nearest accent is 9.5 ΔE\*ab from the agent's colour — rose on a dark ground
//! at 2000 K — against the 5.0 below. What it would refuse is a warmer one,
//! because below the warmth at which the agent's colour stops being plainly
//! different from every accent there is no night light setting worth having:
//! the machine would be trading a signal a person relies on for a screen that
//! is slightly kinder at midnight.
//!
//! # None of which makes the colour sufficient, warmed or not
//!
//! ADR 0010's measured note says terracotta on cream is 2.87:1 — under the 3.0
//! that WCAG 2.1 §1.4.11 asks of a shape carrying meaning, and under the 4.5 it
//! asks of text. The agent's mark and the agent's word were never optional, and
//! [`night_light_does_not_make_the_colour_sufficient_because_it_never_was`]
//! holds that night light changes nothing about that either way.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_appearance::{Accent, Colour, ENOUGH_FOR_A_SHAPE, Scheme, Token};
use alo_displays::{Warming, Warmth};

/// How far apart two colours have to stay, as CIE76 ΔE\*ab.
///
/// Twice the just-noticeable difference of about 2.3. See this file's header
/// for why it is this measure rather than a distance round the colour wheel.
const FAR_ENOUGH: f64 = 5.0;

/// The step this file walks the range of warmths in, in kelvin. Fine enough
/// that no gap in it could hide a warmth at which the promise fails.
const A_STEP: u16 = 50;

/// One channel, 0 to 255, as the fraction of light it stands for — the sRGB
/// curve, as `alo_appearance::contrast` straightens it.
fn straightened(channel: u8) -> f64 {
    let part = f64::from(channel) / 255.0;
    if part <= 0.03928 {
        part / 12.92
    } else {
        ((part + 0.055) / 1.055).powf(2.4)
    }
}

/// This colour in CIELAB, against the white every screen in this system draws
/// against.
fn lab(colour: Colour) -> (f64, f64, f64) {
    let red = straightened(colour.red());
    let green = straightened(colour.green());
    let blue = straightened(colour.blue());
    let x = 0.4124f64.mul_add(red, 0.3576f64.mul_add(green, 0.1805 * blue)) / 0.95047;
    let y = 0.2126f64.mul_add(red, 0.7152f64.mul_add(green, 0.0722 * blue));
    let z = 0.0193f64.mul_add(red, 0.1192f64.mul_add(green, 0.9505 * blue)) / 1.08883;
    let (x, y, z) = (bent(x), bent(y), bent(z));
    (116.0f64.mul_add(y, -16.0), 500.0 * (x - y), 200.0 * (y - z))
}

/// The curve CIELAB bends each of the three through, which is what makes equal
/// steps in it look equally different.
fn bent(part: f64) -> f64 {
    if part > 0.008_856 {
        part.cbrt()
    } else {
        7.787f64.mul_add(part, 16.0 / 116.0)
    }
}

/// How far apart these two colours look, as CIE76 ΔE\*ab.
fn apart(one: Colour, other: Colour) -> f64 {
    let (light, green_red, blue_yellow) = lab(one);
    let (their_light, their_green_red, their_blue_yellow) = lab(other);
    let lightness = light - their_light;
    let first = green_red - their_green_red;
    let second = blue_yellow - their_blue_yellow;
    lightness
        .mul_add(lightness, first.mul_add(first, second * second))
        .sqrt()
}

/// Every warmth a person can choose, from the warmest to neutral.
fn every_warmth() -> Vec<Warmth> {
    let mut warmths = Vec::new();
    let mut kelvin = Warmth::WARMEST;
    while kelvin < Warmth::NEUTRAL {
        let Ok(warmth) = Warmth::kelvin(kelvin) else {
            panic!("{kelvin} K is inside the range this walk is over");
        };
        warmths.push(warmth);
        kelvin = kelvin.saturating_add(A_STEP);
    }
    warmths.push(Warmth::neutral());
    warmths
}

/// **At every warmth a person can choose, the agent's colour is still plainly
/// a different colour from all ten values of the five accents.**
///
/// This is ADR 0010 held under night light: not *the accents are far away on
/// the wheel*, which warming destroys for everybody, but *the agent's colour
/// and this accent are two colours*, which is the thing the decision is
/// actually about.
#[test]
fn the_agents_colour_stays_apart_from_every_accent_at_every_warmth() {
    let mut closest = f64::INFINITY;
    let mut where_it_was = String::new();
    for warmth in every_warmth() {
        let warming = Warming::at(warmth);
        let agent = warming.applied_to(Token::Terracotta.colour());
        for accent in Accent::ALL {
            for scheme in [Scheme::Light, Scheme::Dark] {
                let measured = apart(agent, warming.applied_to(accent.on(scheme)));
                if measured < closest {
                    closest = measured;
                    where_it_was = format!(
                        "{} on {scheme:?} at {} K",
                        accent.word().says(),
                        warmth.as_kelvin()
                    );
                }
                assert!(
                    measured >= FAR_ENOUGH,
                    "{} on {scheme:?} at {} K is {measured} from the agent's colour",
                    accent.word().says(),
                    warmth.as_kelvin()
                );
            }
        }
    }
    assert!(
        closest.is_finite(),
        "the walk over every warmth measured nothing"
    );
    // Printed rather than asserted at a number: the margin belongs in the task
    // report as a measurement, and pinning it here would fail the gate for a
    // designer improving an accent.
    println!("the closest any accent comes to the agent's colour is {closest} — {where_it_was}");
}

/// **And no warmth turns an accent into the agent's colour exactly**, which is
/// the crude failure the measurement above would also catch but which is worth
/// its own sentence: a screen at some particular warmth showing a moss that is
/// terracotta byte for byte.
#[test]
fn no_warmth_draws_an_accent_as_the_agents_colour() {
    let agent = Token::Terracotta.colour();
    for warmth in every_warmth() {
        let warming = Warming::at(warmth);
        for accent in Accent::ALL {
            for scheme in [Scheme::Light, Scheme::Dark] {
                let drawn = warming.applied_to(accent.on(scheme));
                assert_ne!(
                    drawn,
                    warming.applied_to(agent),
                    "{} on {scheme:?} at {} K is the agent's colour",
                    accent.word().says(),
                    warmth.as_kelvin()
                );
                assert_ne!(
                    drawn,
                    agent,
                    "{} on {scheme:?} at {} K is drawn as the agent's unwarmed colour",
                    accent.word().says(),
                    warmth.as_kelvin()
                );
            }
        }
    }
}

/// **The mark and the word are not optional, and night light does not change
/// that.**
///
/// ADR 0010 measured terracotta on cream at 2.87:1, which is under the 3.0 that
/// a shape carrying meaning has to reach — so the agent's colour never carried
/// its meaning by itself, on a warmed screen or a cold one. This holds that at
/// every warmth, so that nobody can read the test above as *the colour is
/// enough now*.
#[test]
fn night_light_does_not_make_the_colour_sufficient_because_it_never_was() {
    for warmth in every_warmth() {
        let warming = Warming::at(warmth);
        let agent = warming.applied_to(Token::Terracotta.colour());
        let ground = warming.applied_to(Token::Cream.colour());
        let measured = agent.contrast_with(ground);
        assert!(
            measured < ENOUGH_FOR_A_SHAPE,
            "at {} K the agent's colour on the reading ground measured {measured}, which would be \
             a change to ADR 0010 rather than a change to this crate",
            warmth.as_kelvin()
        );
    }
}

/// **With night light off, the palette is exactly the palette.** A warmed
/// comparison that started from colours nobody would recognise would prove
/// nothing, so this is the anchor: at neutral every token and every accent
/// comes back byte for byte.
#[test]
fn the_neutral_warmth_is_the_palette_itself() {
    let warming = Warming::at(Warmth::neutral());
    for token in Token::ALL {
        assert_eq!(warming.applied_to(token.colour()), token.colour());
    }
    for accent in Accent::ALL {
        for scheme in [Scheme::Light, Scheme::Dark] {
            assert_eq!(warming.applied_to(accent.on(scheme)), accent.on(scheme));
        }
    }
    assert!(
        apart(Token::Terracotta.colour(), Accent::Rose.on(Scheme::Dark)) >= FAR_ENOUGH,
        "the measure itself agrees with ADR 0010 on an unwarmed screen"
    );
}
