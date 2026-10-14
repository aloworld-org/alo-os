//! What a cell looks like, and the colours a program can ask for.
//!
//! # Why a terminal has a palette at all
//!
//! A program does not ask for `#146C43`. It asks for **red**, or for *colour
//! number 9*, and the terminal decides what that looks like. `cargo test` prints
//! its failures in ANSI red and its passes in ANSI green; `ls` colours a
//! directory in ANSI blue. So a terminal with no palette is a terminal in which
//! none of that is readable.
//!
//! **That makes the sixteen a program-facing contract rather than alo's own
//! design.** A program asking for red expects something recognisably red, and
//! `docs/design/` has no terminal palette in it — measured: the design file's
//! variables for every terminal node are the twelve semantic roles and the type
//! styles, and not one colour named for a terminal.
//!
//! So this adopts **the standard palette**, which is what every terminal starts
//! from, and `docs/design/the-terminal-window.md` records that alo's own is
//! owed. It is the same decision as the function keys in
//! [`crate::what_a_person_typed`]: `F1` to `F4` are `SS3` and the rest are
//! `CSI <n> ~` not because anybody chose it but because that is what the
//! terminal database on the machine expects.
//!
//! **What is deliberately not adopted is the default pair.** A program that
//! asks for no colour at all gets [`AColour::TheDefault`], and the shell
//! substitutes its own — `text/primary` on `bg/surface`, which the design does
//! decide. So alo's terminal looks like alo until a program asks otherwise.
//!
//! # Dim and bright, which are the two easy mistakes
//!
//! Eight of the sixteen are **bright** variants, and a program reaches them two
//! ways: by asking for colours 8 to 15 directly, or by asking for 0 to 7 *with
//! bold*. The second is the older convention and still what most programs do —
//! so a terminal that drew bold as weight alone shows `cargo test`'s green in
//! the dim green, which is visibly wrong against every other terminal.
//!
//! `DIM` is the other direction and is **not** a separate eight: it is the
//! ordinary colour drawn darker.

use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::vte::ansi::{Color, NamedColor};

/// A colour a cell can be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AColour {
    /// The program asked for no colour, so whatever is drawing decides.
    ///
    /// **Not a value.** The default pair is the shell's — `text/primary` on
    /// `bg/surface` — and this crate has no business knowing them. Resolving
    /// this to a grey here would be this crate choosing what alo looks like.
    TheDefault,
    /// Exactly these three bytes.
    Exactly {
        /// Red.
        red: u8,
        /// Green.
        green: u8,
        /// Blue.
        blue: u8,
    },
}

impl AColour {
    /// One of the sixteen, by its number.
    ///
    /// **The standard palette, adopted rather than designed.** Numbers 0 to 7
    /// are the ordinary colours and 8 to 15 their bright variants, in the order
    /// the terminal protocol fixes: black, red, green, yellow, blue, magenta,
    /// cyan, white.
    #[must_use]
    pub const fn one_of_the_sixteen(which: u8) -> Self {
        let (red, green, blue) = match which {
            0 => (0x00, 0x00, 0x00),
            1 => (0xcd, 0x00, 0x00),
            2 => (0x00, 0xcd, 0x00),
            3 => (0xcd, 0xcd, 0x00),
            4 => (0x00, 0x00, 0xee),
            5 => (0xcd, 0x00, 0xcd),
            6 => (0x00, 0xcd, 0xcd),
            7 => (0xe5, 0xe5, 0xe5),
            8 => (0x7f, 0x7f, 0x7f),
            9 => (0xff, 0x00, 0x00),
            10 => (0x00, 0xff, 0x00),
            11 => (0xff, 0xff, 0x00),
            12 => (0x5c, 0x5c, 0xff),
            13 => (0xff, 0x00, 0xff),
            14 => (0x00, 0xff, 0xff),
            // **15 and anything past it.** A `u8` reaches 255 and the sixteen
            // stop at 15; the arm is here because a `match` on `u8` must be
            // total, and white is the honest answer for *a colour number this
            // is not*.
            _ => (0xff, 0xff, 0xff),
        };
        Self::Exactly { red, green, blue }
    }

    /// One of the 256, by its number.
    ///
    /// Three ranges, and the protocol fixes all three: 0 to 15 are the sixteen,
    /// 16 to 231 are a 6×6×6 cube, and 232 to 255 are twenty-four greys.
    ///
    /// **The cube's steps are not evenly spaced and that is not a mistake.**
    /// The first step is 0 and the rest are 95, 135, 175, 215 and 255 — a
    /// convention every terminal shares, and a linear ramp would make every
    /// 256-colour program look subtly wrong against every other terminal.
    #[must_use]
    pub const fn one_of_the_256(which: u8) -> Self {
        if which < 16 {
            return Self::one_of_the_sixteen(which);
        }
        if which < 232 {
            let cube = which - 16;
            return Self::Exactly {
                red: a_step_of_the_cube(cube / 36),
                green: a_step_of_the_cube((cube % 36) / 6),
                blue: a_step_of_the_cube(cube % 6),
            };
        }
        // Twenty-four greys from 8 to 238 in steps of ten.
        let grey = 8 + (which - 232) * 10;
        Self::Exactly {
            red: grey,
            green: grey,
            blue: grey,
        }
    }

    /// Darker, for a cell a program asked to be dim.
    ///
    /// **Two thirds, applied to the colour rather than swapped for another
    /// one.** `DIM` is not a separate palette: it is the colour the program
    /// asked for, drawn fainter. [`Self::TheDefault`] stays itself, because
    /// dimming a colour this crate does not know is not something it can do —
    /// the shell dims its own.
    #[must_use]
    pub const fn dimmed(self) -> Self {
        match self {
            Self::TheDefault => Self::TheDefault,
            Self::Exactly { red, green, blue } => Self::Exactly {
                red: red / 3 * 2,
                green: green / 3 * 2,
                blue: blue / 3 * 2,
            },
        }
    }
}

/// One of the six steps along an axis of the 256-colour cube.
///
/// **The steps are not evenly spaced and that is not a mistake.** The first is
/// 0 and the rest are 95, 135, 175, 215 and 255 — a convention every terminal
/// shares. A linear ramp would make every 256-colour program look subtly wrong
/// against every other terminal, which is the kind of difference somebody
/// notices and cannot name.
///
/// A function rather than a closure because [`AColour::one_of_the_256`] is
/// `const`, and a closure cannot be called in one.
const fn a_step_of_the_cube(at: u8) -> u8 {
    match at {
        0 => 0,
        1 => 95,
        2 => 135,
        3 => 175,
        4 => 215,
        _ => 255,
    }
}

/// How a run of cells is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowItLooks {
    /// The colour of the characters.
    pub ink: AColour,
    /// The colour behind them.
    pub ground: AColour,
    /// Whether the program asked for bold.
    ///
    /// **Separate from the colour, although the colour already brightened.**
    /// A program asking for bold red means both things — the bright red *and*
    /// a heavier face — and a terminal that did only one of them is wrong in a
    /// way a person notices by comparison.
    pub bold: bool,
    /// Whether the program asked for italic.
    pub italic: bool,
    /// Whether the program asked for the cell underlined.
    pub underline: bool,
}

impl HowItLooks {
    /// What this cell looks like.
    ///
    /// **Inverse is applied here and not left to a caller**, because it is the
    /// one attribute whose meaning is *swap the two colours* — and a caller that
    /// forgot it would draw selected text invisibly, ink on its own colour.
    ///
    /// **Hidden is applied here too**, as ink the same colour as the ground.
    /// That is what `HIDDEN` means and it is how a password prompt that echoes
    /// hides what it echoes.
    #[must_use]
    pub fn of(cell: &Cell) -> Self {
        let bold = cell.flags.intersects(Flags::BOLD);
        // **Bold brightens an ordinary colour, which is the older convention
        // and still what most programs rely on.** Without it `cargo test`'s
        // green is the dim green and looks wrong beside every other terminal.
        let ink = a_colour(cell.fg, bold);
        let ground = a_colour(cell.bg, false);
        let (ink, ground) = if cell.flags.intersects(Flags::INVERSE) {
            (ground, ink)
        } else {
            (ink, ground)
        };
        let ink = if cell.flags.intersects(Flags::HIDDEN) {
            ground
        } else if cell.flags.intersects(Flags::DIM) {
            ink.dimmed()
        } else {
            ink
        };
        Self {
            ink,
            ground,
            bold,
            italic: cell.flags.intersects(Flags::ITALIC),
            underline: cell.flags.intersects(Flags::UNDERLINE),
        }
    }
}

/// One colour a program asked for, resolved.
///
/// `brighten` is for a foreground with bold on it, and is **false for a
/// background**: bold is about the text, and a program asking for bold on a
/// blue background did not ask for a brighter blue behind it.
fn a_colour(colour: Color, brighten: bool) -> AColour {
    match colour {
        Color::Spec(rgb) => AColour::Exactly {
            red: rgb.r,
            green: rgb.g,
            blue: rgb.b,
        },
        Color::Indexed(which) => AColour::one_of_the_256(which),
        Color::Named(named) => a_named_colour(named, brighten),
    }
}

/// One of the colours the protocol gives a name to.
fn a_named_colour(named: NamedColor, brighten: bool) -> AColour {
    let ordinary = |which: u8| {
        AColour::one_of_the_sixteen(if brighten && which < 8 {
            which + 8
        } else {
            which
        })
    };
    match named {
        NamedColor::Black => ordinary(0),
        NamedColor::Red => ordinary(1),
        NamedColor::Green => ordinary(2),
        NamedColor::Yellow => ordinary(3),
        NamedColor::Blue => ordinary(4),
        NamedColor::Magenta => ordinary(5),
        NamedColor::Cyan => ordinary(6),
        NamedColor::White => ordinary(7),
        NamedColor::BrightBlack => AColour::one_of_the_sixteen(8),
        NamedColor::BrightRed => AColour::one_of_the_sixteen(9),
        NamedColor::BrightGreen => AColour::one_of_the_sixteen(10),
        NamedColor::BrightYellow => AColour::one_of_the_sixteen(11),
        NamedColor::BrightBlue => AColour::one_of_the_sixteen(12),
        NamedColor::BrightMagenta => AColour::one_of_the_sixteen(13),
        NamedColor::BrightCyan => AColour::one_of_the_sixteen(14),
        NamedColor::BrightWhite => AColour::one_of_the_sixteen(15),
        // **The default pair, and the dim and bright versions of it, are all
        // the shell's.** A program asking for *the foreground colour* is asking
        // for whatever this machine's terminal uses, and that is `text/primary`
        // on `bg/surface` — decided in the design and not here.
        NamedColor::Foreground
        | NamedColor::Background
        | NamedColor::BrightForeground
        | NamedColor::DimForeground => AColour::TheDefault,
        // The dim eight are the ordinary eight, dimmed.
        NamedColor::DimBlack => AColour::one_of_the_sixteen(0).dimmed(),
        NamedColor::DimRed => AColour::one_of_the_sixteen(1).dimmed(),
        NamedColor::DimGreen => AColour::one_of_the_sixteen(2).dimmed(),
        NamedColor::DimYellow => AColour::one_of_the_sixteen(3).dimmed(),
        NamedColor::DimBlue => AColour::one_of_the_sixteen(4).dimmed(),
        NamedColor::DimMagenta => AColour::one_of_the_sixteen(5).dimmed(),
        NamedColor::DimCyan => AColour::one_of_the_sixteen(6).dimmed(),
        NamedColor::DimWhite => AColour::one_of_the_sixteen(7).dimmed(),
        // **The cursor's own colours are not a cell's.** A program can set
        // them, and what draws the cursor reads them from the cursor rather
        // than from a cell that happens to be under it.
        NamedColor::Cursor => AColour::TheDefault,
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "what_a_cell_looks_like_tests.rs"]
mod tests;
