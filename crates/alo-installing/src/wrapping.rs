//! Breaking a sentence into the lines it is drawn on.
//!
//! # Why this is here at all
//!
//! The install environment draws on `/dev/fb0` with no compositor and no
//! toolkit, so nothing above it does line breaking. The rasteriser it will use
//! — `swash`, the same one `alo-shell` draws glyphs with, so the installer and
//! the desktop put text on screen through one engine — gives glyphs and
//! advances and not layout. This is the layout.
//!
//! # Why it has to be right rather than approximately right
//!
//! Measured on this crate's own `words.rs`: the longest sentence a person reads
//! in the environment is **120 characters**, and the block it goes in is
//! `600 / 1440` of the display — 600 px where the design was drawn and
//! **267 px at the 640-wide floow this crate still lays out for**. So the hard
//! case is five or six lines, not one, and a wrapper tested on *Choose a
//! network* is a wrapper tested on the wrong subject.
//!
//! # What it breaks on, and what it does not
//!
//! | | |
//! |---|---|
//! | a space | breaks, and the space is dropped rather than drawn at the end of a line |
//! | after a hyphen | breaks, keeping the hyphen on the line above, which is where a reader expects it |
//! | a non-breaking space | **never breaks**, which is the whole reason that character exists |
//! | a word longer than the line | broken between characters, because a word drawn past the edge is worse than a word broken |
//!
//! **No hyphenation**, and that is a decision rather than an omission:
//! hyphenating correctly needs a dictionary per language, this product promises
//! 24 of them, and a wrong hyphen is a misspelling. German compounds will give
//! short lines, and a short line is legible where a wrong break is not.
//!
//! **No bidirectional reordering.** None of the 24 EU languages this product
//! promises is written right to left, so there is nothing here to get wrong
//! yet; a language contributed later that is would need more than this file,
//! and this sentence is where whoever adds it should start.
//!
//! # How width is asked
//!
//! By a function the caller passes in, never by counting characters. Manrope is
//! proportional — an `i` is about a third the width of an `m` — so counting
//! characters would be wrong by a factor of two on the same line, which is how
//! a wrapper passes its tests against a monospace measure and overflows on a
//! real screen. The measure is the renderer's, and this file never guesses it.

/// The character that exists in order not to break.
const A_SPACE_THAT_DOES_NOT_BREAK: char = '\u{00A0}';

/// One line of a sentence, as a slice of the sentence itself.
///
/// A slice rather than a `String` because nothing here rewrites any text: a
/// wrapper that returned new strings could silently change what is drawn, and
/// the one thing this must not do is alter a sentence a translator wrote.
pub type ALine<'a> = &'a str;

/// Break this sentence into lines that each fit inside `wide`.
///
/// `measure` answers how wide a piece of text is drawn, in the same units as
/// `wide`. It is called on candidate lines rather than on words, so that a
/// renderer whose advances depend on neighbouring characters stays correct.
///
/// # What is guaranteed
///
/// - **No line is wider than `wide`**, unless one single character is, which
///   nothing can fix and which the caller sees as a line with one character on
///   it;
/// - **every word appears once, in order**, so nothing is dropped or repeated;
/// - **no line begins or ends with a breaking space.**
///
/// An empty sentence gives no lines at all rather than one empty line, because
/// a drawn empty line takes up room on a screen whose block is already tight.
///
/// # Examples
///
/// ```
/// use alo_installing::wrapped;
///
/// // A measure of five units per character stands in for a renderer here.
/// let lines = wrapped("one two three", 40, |text| text.chars().count() as u32 * 5);
/// assert_eq!(lines, ["one two", "three"]);
/// ```
#[must_use]
pub fn wrapped<'a>(sentence: &'a str, wide: u32, measure: impl Fn(&str) -> u32) -> Vec<ALine<'a>> {
    let mut lines = Vec::new();
    let mut rest = sentence.trim();

    while !rest.is_empty() {
        if measure(rest) <= wide {
            lines.push(rest);
            break;
        }
        let (line, after) = the_longest_that_fits(rest, wide, &measure);
        lines.push(line);
        rest = after.trim_start_matches(' ');
        if line.is_empty() {
            // Nothing fitted and nothing was consumed, which would spin. One
            // character goes on the line instead, and it is wider than `wide`
            // — the honest outcome rather than a hang.
            let mut characters = rest.char_indices();
            characters.next();
            let at = characters.next().map_or(rest.len(), |(at, _)| at);
            let (one, after) = rest.split_at(at);
            lines.pop();
            lines.push(one);
            rest = after;
        }
    }
    lines
}

/// The longest prefix of `rest` that fits, and what is left after it.
///
/// Breaks at the last place a break is allowed. Returns an empty line where no
/// break helps, which is a word longer than the whole line.
fn the_longest_that_fits<'a>(
    rest: &'a str,
    wide: u32,
    measure: &impl Fn(&str) -> u32,
) -> (&'a str, &'a str) {
    let mut best: Option<usize> = None;
    for at in where_a_break_is_allowed(rest) {
        if measure(rest[..at].trim_end()) <= wide {
            best = Some(at);
        } else {
            // The places come in order, so once one is too wide every later
            // one is too. Stopping here is what keeps this linear in the
            // number of breaks rather than quadratic.
            break;
        }
    }
    match best {
        Some(at) => (rest[..at].trim_end(), &rest[at..]),
        // A single word wider than the line: break it between characters, at
        // the last character that fits. A word drawn past the edge of the
        // block is worse than a word broken, because the part past the edge is
        // not merely ugly — it is not there.
        None => break_the_word(rest, wide, measure),
    }
}

/// Every place in this text where a line may be broken, in order.
///
/// After a space, and after a hyphen. A non-breaking space is not one of them.
fn where_a_break_is_allowed(text: &str) -> impl Iterator<Item = usize> + '_ {
    text.char_indices().filter_map(move |(at, character)| {
        let after = at + character.len_utf8();
        if after >= text.len() {
            return None;
        }
        match character {
            ' ' => Some(after),
            '-' => Some(after),
            A_SPACE_THAT_DOES_NOT_BREAK => None,
            _ => None,
        }
    })
}

/// Break a word that is wider than the whole line, between characters.
fn break_the_word<'a>(
    word: &'a str,
    wide: u32,
    measure: &impl Fn(&str) -> u32,
) -> (&'a str, &'a str) {
    let mut fits = 0;
    for (at, character) in word.char_indices() {
        let after = at + character.len_utf8();
        if after >= word.len() {
            break;
        }
        if measure(&word[..after]) <= wide {
            fits = after;
        } else {
            break;
        }
    }
    word.split_at(fits)
}

/// How many lines this sentence is drawn on.
///
/// What the layout needs, and the reason it is its own function: a screen asks
/// *how tall is this introduction* before it asks *what does line three say*,
/// and asking for the lines and counting them would make the layout depend on
/// a `Vec` it throws away.
#[must_use]
pub fn how_many_lines(sentence: &str, wide: u32, measure: impl Fn(&str) -> u32) -> u32 {
    let how_many = wrapped(sentence, wide, measure).len();
    // Never nought: a sentence with nothing in it still occupies no line, but
    // a screen asking how tall its introduction is wants at least one line of
    // room wherever there is an introduction at all, and `None` for whether
    // there is one is the caller's own question.
    u32::try_from(how_many).unwrap_or(u32::MAX)
}

#[cfg(test)]
#[path = "wrapping_tests.rs"]
mod tests;
