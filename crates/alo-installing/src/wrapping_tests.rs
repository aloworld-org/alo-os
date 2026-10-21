//! That no line is drawn past the edge of the block, that nothing is dropped,
//! and that the hard case — the longest sentence this crate says, in the
//! narrowest block it lays out for — comes out legible.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::{how_many_lines, wrapped};

/// The longest sentence a person reads in this environment, measured off
/// `words.rs`: S0's, which names the two things that will be typed.
const THE_LONGEST: &str = "You will type a network password and the name of a disk. Both have to \
                           be exact, so the keyboard has to be right first";

/// A measure that is **not** one unit per character, because a wrapper tested
/// against a monospace measure passes and then overflows on a real screen.
///
/// The proportions are Manrope's rough shape rather than its exact advances:
/// narrow letters about a third of wide ones. Exactness is not the point — the
/// point is that the measure disagrees with character counting, so a wrapper
/// that counts characters cannot pass.
fn proportional(text: &str) -> u32 {
    text.chars()
        .map(|character| match character {
            'i' | 'l' | 'j' | '.' | ',' | '\'' | '!' | '|' => 3,
            'm' | 'w' | 'M' | 'W' => 14,
            ' ' => 4,
            character if character.is_uppercase() => 11,
            _ => 8,
        })
        .sum()
}

/// Five units a character, for the cases where a readable expectation matters
/// more than realism.
fn five_each(text: &str) -> u32 {
    u32::try_from(text.chars().count()).unwrap_or(u32::MAX) * 5
}

#[test]
fn no_line_is_ever_wider_than_the_block() {
    // The one property everything else rests on, swept across the widths this
    // crate lays out for. A line past the edge of the block is not ugly — the
    // part past the edge is simply not on the screen.
    for wide in (40..=600).step_by(7) {
        for sentence in [
            THE_LONGEST,
            "Which disk should alo OS use?",
            "This installer cannot tell which disk it is running from, so it will not erase \
             anything",
            "Donaudampfschifffahrtsgesellschaftskapitänsmütze",
        ] {
            for line in wrapped(sentence, wide, proportional) {
                let drawn = proportional(line);
                // One character wider than the block is the only allowed
                // excess, and only where that single character does not fit.
                let one_character = line.chars().count() <= 1;
                assert!(
                    drawn <= wide || one_character,
                    "at {wide} wide, the line {line:?} is drawn {drawn} wide"
                );
            }
        }
    }
}

#[test]
fn every_letter_survives_once_and_in_order() {
    // A wrapper that drops a word from a sentence about erasing a disk is
    // worse than one that wraps badly, and a dropped word is invisible in a
    // test that only checks widths.
    //
    // **Asserted on letters rather than words, and that is the correct
    // property rather than a weaker one.** This was written as *the words come
    // back the same* and failed at narrow widths — rightly: below about 70
    // units no single word fits, so words are broken between characters and
    // *password* legitimately comes back as two pieces. Comparing letters with
    // the spaces discounted holds in both cases and still catches a dropped
    // or repeated one, which is the fault being guarded against.
    let letters = |text: &str| text.chars().filter(|one| *one != ' ').collect::<String>();

    for wide in (20..=600).step_by(11) {
        let lines = wrapped(THE_LONGEST, wide, proportional);
        assert_eq!(
            letters(&lines.join("")),
            letters(THE_LONGEST),
            "at {wide} wide the sentence came back altered"
        );
    }
}

#[test]
fn no_line_begins_or_ends_with_a_space() {
    // A trailing space is drawn, so it pushes the last word of a line past the
    // edge; a leading one makes a ragged left margin that reads as a mistake.
    for wide in (40..=600).step_by(9) {
        for line in wrapped(THE_LONGEST, wide, proportional) {
            assert!(
                !line.starts_with(' ') && !line.ends_with(' '),
                "at {wide} wide, the line {line:?} has a space on an end"
            );
        }
    }
}

#[test]
fn a_sentence_that_fits_is_left_on_one_line() {
    let lines = wrapped("Which disk should alo OS use?", 600, proportional);
    assert_eq!(lines, ["Which disk should alo OS use?"]);
    assert_eq!(
        how_many_lines("Which disk should alo OS use?", 600, proportional),
        1
    );
}

#[test]
fn nothing_gives_no_lines_rather_than_one_empty_one() {
    assert!(wrapped("", 600, proportional).is_empty());
    assert!(wrapped("   ", 600, proportional).is_empty());
    assert_eq!(how_many_lines("", 600, proportional), 0);
}

#[test]
fn it_breaks_at_a_space_and_drops_it() {
    let lines = wrapped("one two three", 40, five_each);
    assert_eq!(lines, ["one two", "three"]);
}

#[test]
fn it_breaks_after_a_hyphen_and_keeps_the_hyphen_above() {
    // Where a reader expects it. A hyphen moved to the start of the next line
    // reads as a dash.
    let lines = wrapped("alo-OS-installer", 40, five_each);
    assert!(
        lines.first().expect("at least one line").ends_with('-'),
        "the hyphen did not stay on the line above: {lines:?}"
    );
    assert_eq!(lines.join(""), "alo-OS-installer");
}

#[test]
fn a_non_breaking_space_never_breaks() {
    // The whole reason that character exists. "340 GB" with one in it must not
    // be split across two lines, because a number and its unit read as two
    // different things when they are.
    let joined = "about 340\u{00A0}GB of files";
    for wide in 30..=200 {
        for line in wrapped(joined, wide, five_each) {
            let halves = line.contains("340") && !line.contains('\u{00A0}');
            assert!(
                !halves,
                "at {wide} wide, {line:?} split a non-breaking space"
            );
        }
    }
}

#[test]
fn a_word_longer_than_the_line_is_broken_rather_than_drawn_past_the_edge() {
    // The German case, and it is not hypothetical: this product promises 24
    // languages and one of them makes compounds.
    let long = "Donaudampfschifffahrtsgesellschaftskapitänsmütze";
    let lines = wrapped(long, 60, five_each);
    assert!(lines.len() > 1, "the word was not broken at all");
    assert_eq!(lines.join(""), long, "the word was altered by being broken");
    for line in &lines {
        assert!(
            five_each(line) <= 60 || line.chars().count() <= 1,
            "{line:?} is still too wide"
        );
    }
}

#[test]
fn a_block_narrower_than_one_character_still_terminates() {
    // The case that would hang rather than fail, which is the worst outcome on
    // a screen a person is waiting at. One character goes on the line and is
    // wider than the block, which is the honest result.
    let lines = wrapped("abc", 1, five_each);
    assert_eq!(lines.join(""), "abc");
    assert!(lines.len() == 3, "expected one character a line: {lines:?}");
}

#[test]
fn the_longest_sentence_in_the_narrowest_block_is_six_lines_or_fewer() {
    // **The case this file exists for.** 600 / 1440 of a 640-wide display is
    // 267 px, which is the narrowest block this crate lays out for, and the
    // sentence is the longest one it says. Six lines is what the flow has to
    // make room for; more than that and the block cannot hold the screen.
    let narrowest = 600 * 640 / 1440;
    let how_many = how_many_lines(THE_LONGEST, narrowest, proportional);
    assert!(
        (2..=6).contains(&how_many),
        "the longest sentence takes {how_many} lines in {narrowest} px, and the \
         flow was built for six"
    );

    // And at the size the design was drawn, it is two — which is the finding
    // that moved the layout to a flow: the frame gives the introduction one
    // line of room and sixteen units of clearance, so the second line would
    // have been drawn over the first card.
    assert_eq!(how_many_lines(THE_LONGEST, 600, proportional), 2);
}

#[test]
fn narrowing_the_block_never_reduces_the_number_of_lines() {
    // A sanity property that would catch an off-by-one in the break search:
    // lines can only go up as the block narrows.
    let mut last = 0;
    for wide in (60..=600).rev().step_by(10) {
        let how_many = how_many_lines(THE_LONGEST, wide, proportional);
        assert!(
            how_many >= last,
            "at {wide} wide it takes {how_many} lines, fewer than the {last} it \
             took in a wider block"
        );
        last = how_many;
    }
}
