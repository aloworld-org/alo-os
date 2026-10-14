//! **The palette's own arithmetic, and the two conventions a person notices.**
//!
//! A colour table is a list somebody copied, and a wrong entry is invisible
//! until a program uses it. So these check the **rules** the table follows —
//! the cube's uneven steps, bold brightening an ordinary colour, inverse
//! swapping the pair — rather than reciting the entries back.

use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};

use super::{AColour, HowItLooks};

/// A cell asking for these colours and flags.
fn a_cell(fg: Color, bg: Color, flags: Flags) -> Cell {
    Cell {
        fg,
        bg,
        flags,
        ..Cell::default()
    }
}

/// The three bytes of a colour, for a test that compares them.
///
/// [`None`] for [`AColour::TheDefault`], which has no bytes **here** — the
/// shell decides them — so a test reaching that asked the wrong question and
/// says so rather than comparing zeroes.
fn bytes_of(colour: AColour) -> Option<(u8, u8, u8)> {
    match colour {
        AColour::Exactly { red, green, blue } => Some((red, green, blue)),
        AColour::TheDefault => None,
    }
}

/// Shorthand for one of the sixteen.
fn sixteen(which: u8) -> AColour {
    AColour::one_of_the_sixteen(which)
}

/// **A program that asks for nothing gets the default**, which the shell
/// decides and this crate does not.
///
/// Resolving it to a grey here would be this crate choosing what alo looks
/// like, and the design decides that — `text/primary` on `bg/surface`.
#[test]
fn a_program_asking_for_nothing_gets_the_default() {
    let looks = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::Foreground),
        Color::Named(NamedColor::Background),
        Flags::empty(),
    ));
    assert_eq!(looks.ink, AColour::TheDefault);
    assert_eq!(looks.ground, AColour::TheDefault);
    assert!(!looks.bold);
}

/// **Bold brightens an ordinary colour as well as asking for weight.**
///
/// The older convention and still what most programs rely on: `cargo test`
/// prints its passes as bold green, and a terminal that drew only the weight
/// shows the dim green — visibly wrong beside every other terminal.
///
/// Both halves are asserted, because a terminal that did only the brightening
/// would be wrong the other way.
#[test]
fn bold_brightens_the_colour_and_asks_for_weight() {
    let plain = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::Green),
        Color::Named(NamedColor::Background),
        Flags::empty(),
    ));
    let bold = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::Green),
        Color::Named(NamedColor::Background),
        Flags::BOLD,
    ));

    assert_eq!(plain.ink, sixteen(2), "ordinary green is colour 2");
    assert_eq!(bold.ink, sixteen(10), "bold green is the bright green, 10");
    assert_ne!(plain.ink, bold.ink, "bold did not brighten the colour");
    assert!(bold.bold && !plain.bold, "bold did not ask for weight");
}

/// **Bold does not brighten a background.**
///
/// Bold is about the text. A program asking for bold on a blue background did
/// not ask for a brighter blue behind it, and a terminal that brightened both
/// would make bold text on a coloured ground change the ground.
#[test]
fn bold_leaves_the_background_alone() {
    let looks = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::White),
        Color::Named(NamedColor::Blue),
        Flags::BOLD,
    ));
    assert_eq!(looks.ground, sixteen(4), "the background brightened too");
}

/// **A bright colour asked for directly is already bright**, and bold does not
/// take it further — there are sixteen and not twenty-four.
#[test]
fn bold_on_an_already_bright_colour_changes_nothing() {
    let looks = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::BrightRed),
        Color::Named(NamedColor::Background),
        Flags::BOLD,
    ));
    assert_eq!(looks.ink, sixteen(9));
}

/// **Inverse swaps the pair**, which is the one attribute whose whole meaning
/// is a swap.
///
/// Applied here rather than left to a caller, because a caller that forgot it
/// would draw selected text invisibly — ink on its own colour.
#[test]
fn inverse_swaps_the_two_colours() {
    let plain = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::Red),
        Color::Named(NamedColor::Blue),
        Flags::empty(),
    ));
    let inverted = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::Red),
        Color::Named(NamedColor::Blue),
        Flags::INVERSE,
    ));
    assert_eq!((plain.ink, plain.ground), (sixteen(1), sixteen(4)));
    assert_eq!(
        (inverted.ink, inverted.ground),
        (sixteen(4), sixteen(1)),
        "inverse did not swap the pair"
    );
}

/// **Hidden is ink the colour of the ground**, which is what hides it — and
/// how a prompt that echoes hides what it echoes.
#[test]
fn hidden_draws_the_ink_in_the_grounds_colour() {
    let looks = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::Red),
        Color::Named(NamedColor::Blue),
        Flags::HIDDEN,
    ));
    assert_eq!(
        looks.ink, looks.ground,
        "a hidden cell's ink is still a different colour from its ground"
    );
}

/// **Dim is the colour the program asked for, drawn fainter** — not a
/// different colour.
#[test]
fn dim_darkens_the_colour_asked_for() {
    let plain = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::Red),
        Color::Named(NamedColor::Background),
        Flags::empty(),
    ));
    let dim = HowItLooks::of(&a_cell(
        Color::Named(NamedColor::Red),
        Color::Named(NamedColor::Background),
        Flags::DIM,
    ));
    let (was, _, _) = bytes_of(plain.ink).expect("an exact colour");
    let (now, _, _) = bytes_of(dim.ink).expect("an exact colour");
    assert!(now < was, "dim did not darken: {now} against {was}");
    assert!(now > 0, "dim took the colour to nothing");
}

/// **The default stays itself when dimmed**, because dimming a colour this
/// crate does not know is not something it can do.
#[test]
fn the_default_cannot_be_dimmed_here() {
    assert_eq!(AColour::TheDefault.dimmed(), AColour::TheDefault);
}

/// **A program asking for exact bytes gets exactly those bytes.**
///
/// Truecolour, which modern programs use and which must not be rounded into
/// the palette.
#[test]
fn exact_bytes_are_passed_through_untouched() {
    let looks = HowItLooks::of(&a_cell(
        Color::Spec(Rgb {
            r: 0x12,
            g: 0x34,
            b: 0x56,
        }),
        Color::Named(NamedColor::Background),
        Flags::empty(),
    ));
    assert_eq!(
        looks.ink,
        AColour::Exactly {
            red: 0x12,
            green: 0x34,
            blue: 0x56
        }
    );
}

/// **The 256-colour cube's steps are the conventional uneven ones.**
///
/// Colour 16 is the cube's black corner and 231 its white one; a linear ramp
/// would put the second step at 51 rather than 95, which is the difference
/// somebody notices and cannot name.
#[test]
fn the_cube_follows_the_conventional_uneven_steps() {
    assert_eq!(
        AColour::one_of_the_256(16),
        AColour::Exactly {
            red: 0,
            green: 0,
            blue: 0
        },
        "colour 16 is the cube's black corner"
    );
    assert_eq!(
        AColour::one_of_the_256(231),
        AColour::Exactly {
            red: 255,
            green: 255,
            blue: 255
        },
        "colour 231 is the cube's white corner"
    );
    // The second step along the red axis: 16 + 36 is one step of red.
    assert_eq!(
        AColour::one_of_the_256(52),
        AColour::Exactly {
            red: 95,
            green: 0,
            blue: 0
        },
        "the second step is 95 and not a linear 51"
    );
}

/// **The first sixteen of the 256 are the sixteen**, rather than a second
/// table that could drift from the first.
#[test]
fn the_first_sixteen_of_the_256_are_the_sixteen() {
    for which in 0..16_u8 {
        assert_eq!(
            AColour::one_of_the_256(which),
            AColour::one_of_the_sixteen(which),
            "colour {which} differs between the two tables"
        );
    }
}

/// **The greys are twenty-four steps and neither end is the cube's.**
#[test]
fn the_greys_are_twenty_four_steps() {
    let (first, _, _) = bytes_of(AColour::one_of_the_256(232)).expect("an exact colour");
    let (last, _, _) = bytes_of(AColour::one_of_the_256(255)).expect("an exact colour");
    assert_eq!(first, 8, "the greys start at 8, not at 0");
    assert_eq!(last, 238, "the greys end at 238, not at 255");
    for which in 232..=255_u8 {
        let (red, green, blue) = bytes_of(AColour::one_of_the_256(which)).expect("an exact colour");
        assert!(
            red == green && green == blue,
            "colour {which} should be a grey and is {red},{green},{blue}"
        );
    }
}

/// **Every one of the sixteen is a distinct colour.**
///
/// A sweep rather than a recital: it catches a copied line, which is how a
/// table like this goes wrong.
#[test]
fn the_sixteen_are_sixteen_different_colours() {
    let all: Vec<AColour> = (0..16_u8).map(AColour::one_of_the_sixteen).collect();
    for (which, colour) in all.iter().enumerate() {
        for (other, against) in all.iter().enumerate() {
            if which != other {
                assert_ne!(
                    colour, against,
                    "colours {which} and {other} are the same, so one was copied"
                );
            }
        }
    }
}

/// **A bright colour is brighter than its ordinary counterpart**, for all
/// eight pairs — the relation the table exists to express.
#[test]
fn each_bright_colour_is_brighter_than_its_own() {
    for which in 0..8_u8 {
        let (r, g, b) = bytes_of(sixteen(which)).expect("an exact colour");
        let (br, bg, bb) = bytes_of(sixteen(which + 8)).expect("an exact colour");
        let ordinary = u32::from(r) + u32::from(g) + u32::from(b);
        let bright = u32::from(br) + u32::from(bg) + u32::from(bb);
        assert!(
            bright > ordinary,
            "colour {} is not brighter than colour {which}: {bright} against {ordinary}",
            which + 8
        );
    }
}
