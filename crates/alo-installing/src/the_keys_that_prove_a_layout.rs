//! The few characters a person types to see whether the keyboard guess is
//! right.
//!
//! # Why the card asks for this rather than for a name
//!
//! `the_screen_that_lists_every_layout` records the design's reasoning, and it
//! is the reason this file exists rather than a list of names:
//!
//! > This card asks the keyboard to prove itself rather than asking the person
//! > to recognise a name: somebody who cannot tell one layout from another in
//! > a list can tell whether what appeared matches what is printed on their
//! > key.
//!
//! So the card shows a few characters, the person presses the keys those
//! characters are printed on, and either the same characters appear or the
//! guess was wrong. No knowledge of layout names is needed at any point.
//!
//! # The fault this file fixes
//!
//! The sentence is `"Press the keys shown to check: {keys}"`, and **nothing
//! ever filled `{keys}`**: every sentence on every screen was said with
//! `Filling::nothing()`. A person read *Press the keys shown to check:
//! \[keys\]* — the placeholder, drawn as a placeholder — on the first screen of
//! the install, and the card that the whole keyboard question rests on asked
//! them to press nothing in particular.
//!
//! It was found by rendering the screen to an image and looking at it. No test
//! saw it: the sentence was fetched correctly, said correctly, laid out
//! correctly and drawn correctly. It was simply not finished.
//!
//! # Where the four positions come from, which is not a choice made here
//!
//! The design's own frame shows **`a z ; m`** for Belgisch. Those four are
//! exactly what the Belgian layout gives at the four physical positions a
//! QWERTY keyboard prints `Q`, `W`, `,` and `;` on — measured from
//! `alo_keyboards::what_a_key_gives`, not chosen to fit:
//!
//! | position | evdev | `be` gives |
//! |---|---|---|
//! | `Q` | 16 | `a` |
//! | `W` | 17 | `z` |
//! | `,` | 51 | `;` |
//! | `;` | 39 | `m` |
//!
//! Reading them in that order reproduces the design's example character for
//! character, which is the argument for these four rather than any other four.
//! They are also where the Latin layouts this environment offers differ most:
//! the AZERTY pair move two letters, and the last position carries `ö`, `ñ`,
//! `ò` or `+` on the German, Spanish, Italian and Dutch layouts.
//!
//! # What it cannot do, said here rather than discovered
//!
//! **English (UK) and English (US) give the same four characters**, so this
//! card cannot tell those two apart. It is honest about what it is: a check
//! that the keyboard is roughly the right family, which is the mistake worth
//! catching — somebody with an AZERTY keyboard handed a QWERTY guess cannot
//! type their own password. The two English layouts differ on `£` and `#`, and
//! a person who picks the wrong one of those can still type every character of
//! a password and a disk's name.

use alo_keyboards::what_a_key_gives::{Held, what_a_key_gives};

/// The four physical positions, named by what a QWERTY keyboard prints on
/// them, and in the order the design's own example reads.
///
/// Held as evdev codes because that is what this environment reads off
/// `/dev/input/event*`, and because a position is a position whatever is
/// printed on it — which is the entire point of the check.
pub const THE_FOUR_POSITIONS: [u16; 4] = [16, 17, 51, 39];

/// How the four characters are joined for a person to read.
///
/// A space between them, because they are four separate keys to press and not
/// a word to type.
const BETWEEN_THEM: &str = " ";

/// What a person should see if this layout is the one their keyboard has.
///
/// [`None`] where the layout is not one this environment knows, so a caller
/// cannot print an empty check and call it a check.
#[must_use]
pub fn the_keys_that_prove_a_layout(layout: &str) -> Option<String> {
    let mut shown = Vec::with_capacity(THE_FOUR_POSITIONS.len());
    for position in THE_FOUR_POSITIONS {
        shown.push(what_a_key_gives(layout, position, Held::nothing())?.to_string());
    }
    Some(shown.join(BETWEEN_THEM))
}

#[cfg(test)]
#[path = "the_keys_that_prove_a_layout_tests.rs"]
mod tests;
