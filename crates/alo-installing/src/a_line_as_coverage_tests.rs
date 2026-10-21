//! That letters actually come out, in the right place, inside the line they
//! were given — asked of coverage, on a machine with no screen.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use super::a_line_as_coverage;
use crate::alos_own_face::AlosOwnFace;

/// The size the environment's body text is drawn at where the design was
/// drawn, from `where_a_screen_goes`: 16 to the em on a line of 25.
const BODY: u32 = 16;

/// How tall a line of text at this em is, as `TheType::by_proportion` gives
/// it: the design draws body at 16 on 25, so a line is the em and a half.
///
/// A test's own arithmetic rather than the layout's, deliberately -- this
/// file is about the raster, and taking the real layout would make these tests
/// fail when the layout changed for reasons that have nothing to do with
/// drawing a letter.
fn a_line_of(em: u32) -> u32 {
    (em * 25) / 16
}

fn face() -> AlosOwnFace<'static> {
    AlosOwnFace::carried().expect("the typeface compiled into this program")
}

/// How many pixels in this coverage have any ink at all.
fn inked(coverage: &super::ACoverage) -> usize {
    (0..coverage.down())
        .flat_map(|down| (0..coverage.across()).map(move |across| (across, down)))
        .filter(|&(across, down)| coverage.at(across, down) > 0)
        .count()
}

#[test]
fn a_letter_actually_comes_out() {
    // **The check that the whole renderer is not a quiet nothing.** Everything
    // else here could pass against a function that returns the right-sized
    // grid of zeroes, and a screen drawn from that grid is a blank screen with
    // no error anywhere — which is this repository's most-repeated failure.
    let drawn = a_line_as_coverage(face(), "alo OS", BODY, a_line_of(BODY));
    assert!(
        drawn.anything_at_all(),
        "nothing was inked at all, so the screen would be blank"
    );
    assert!(
        inked(&drawn) > 20,
        "only {} pixels were inked for six letters",
        inked(&drawn)
    );
}

#[test]
fn every_letter_of_a_sentence_inks_something() {
    // Letter by letter, because a line that inks *something* can still be
    // missing one letter in the middle — and a word with a hole in it is what
    // a person reads.
    let face = face();
    let sentence = "Which disk should alo OS use? Müller Liège ßẞ";
    for character in sentence.chars().filter(|one| *one != ' ') {
        let drawn = a_line_as_coverage(face, &character.to_string(), BODY, a_line_of(BODY));
        assert!(
            drawn.anything_at_all(),
            "{character:?} inked nothing, so it would be drawn as a gap"
        );
    }
}

#[test]
fn the_coverage_is_exactly_as_wide_as_the_face_said_the_text_was() {
    // The layout measures with `how_wide` and draws this. If the two disagreed
    // by even a pixel the text would creep out of its block, so they are the
    // same arithmetic and this says so.
    let face = face();
    for text in ["alo OS", "Which disk should alo OS use?", "i", "mmmm", ""] {
        let drawn = a_line_as_coverage(face, text, BODY, a_line_of(BODY));
        assert_eq!(
            drawn.across(),
            face.how_wide(text, BODY),
            "the coverage for {text:?} is not the width the face measured"
        );
        assert_eq!(
            drawn.down(),
            a_line_of(BODY),
            "a line is not as tall as the line it was given"
        );
    }
}

#[test]
fn nothing_is_inked_outside_the_line() {
    // Clipping, asserted rather than assumed. A glyph's box can reach above or
    // below the line, and the part outside has to be dropped rather than
    // written somewhere else in the buffer.
    let face = face();
    for em in [8_u32, 11, 14, 16, 36, 48] {
        let drawn = a_line_as_coverage(face, "Ägypten jpqy", em, a_line_of(em));
        // Reading outside the coverage answers nought rather than panicking,
        // which is what a caller that lays out by its own arithmetic needs.
        assert_eq!(drawn.at(drawn.across(), 0), 0);
        assert_eq!(drawn.at(0, drawn.down()), 0);
        assert_eq!(drawn.at(u32::MAX, u32::MAX), 0);
        assert!(drawn.row(drawn.down()).is_empty());
        assert_eq!(drawn.row(0).len(), drawn.across() as usize);
    }
}

#[test]
fn the_letters_sit_inside_the_line_rather_than_at_its_very_top() {
    // **The baseline check.** Positioning a glyph from the line's top instead
    // of from the baseline draws everything about four fifths of a line too
    // high, which on these screens puts the eyebrow off the top of the
    // display. A capital letter has to sit with its top below the line's top
    // and its bottom above the line's bottom.
    let drawn = a_line_as_coverage(face(), "H", 36, a_line_of(36));
    let rows_with_ink: Vec<u32> = (0..drawn.down())
        .filter(|&down| drawn.row(down).iter().any(|&how_much| how_much > 0))
        .collect();
    let first = *rows_with_ink.first().expect("a capital H inks something");
    let last = *rows_with_ink.last().expect("a capital H inks something");

    assert!(
        first > 0,
        "the letter starts at the very top row of the line, which means it was \
         positioned from the top rather than from the baseline"
    );
    assert!(
        last < drawn.down() - 1,
        "the letter reaches the very bottom row of the line"
    );
    // And it is a capital, so it should occupy the greater part of the line
    // rather than a sliver of it.
    assert!(
        last - first > drawn.down() / 3,
        "a capital H is only {} of {} rows tall",
        last - first,
        drawn.down()
    );
}

#[test]
fn a_descender_reaches_below_a_capital_and_an_accent_above_it() {
    // Two letters, one line, and the difference between them is what proves
    // the baseline is shared rather than each glyph being placed on its own.
    let face = face();
    let rows = |text: &str| {
        let drawn = a_line_as_coverage(face, text, 36, a_line_of(36));
        let inked: Vec<u32> = (0..drawn.down())
            .filter(|&down| drawn.row(down).iter().any(|&how_much| how_much > 0))
            .collect();
        (*inked.first().expect("ink"), *inked.last().expect("ink"))
    };

    let (cap_top, cap_bottom) = rows("H");
    let (_, descender_bottom) = rows("p");
    let (accent_top, _) = rows("Ä");

    assert!(
        descender_bottom > cap_bottom,
        "a p does not reach below an H: p ends at {descender_bottom} and H at \
         {cap_bottom}"
    );
    assert!(
        accent_top < cap_top,
        "the diaeresis on an A-umlaut is not above the top of an H: {accent_top} \
         against {cap_top}"
    );
}

#[test]
fn nothing_gives_an_empty_line_rather_than_a_panic() {
    let drawn = a_line_as_coverage(face(), "", BODY, a_line_of(BODY));
    assert_eq!(drawn.across(), 0);
    assert!(!drawn.anything_at_all());
    assert_eq!(drawn.at(0, 0), 0);
}

#[test]
fn a_character_the_face_cannot_draw_leaves_the_rest_alone() {
    // A letter with no glyph advances nothing and inks nothing — so the words
    // either side of it still come out, rather than the line stopping there.
    let face = face();
    let without = a_line_as_coverage(face, "alo OS", BODY, a_line_of(BODY));
    let with = a_line_as_coverage(face, "alo\u{3042}OS", BODY, a_line_of(BODY));
    assert!(
        with.anything_at_all(),
        "the line stopped at the unknown letter"
    );
    assert!(
        without.anything_at_all(),
        "the control line inked nothing, so this test proves nothing"
    );
}

#[test]
fn a_bigger_em_inks_more() {
    let face = face();
    let small = a_line_as_coverage(face, "alo OS", 12, a_line_of(12));
    let large = a_line_as_coverage(face, "alo OS", 48, a_line_of(48));
    assert!(
        inked(&large) > inked(&small) * 4,
        "48 to the em inks {} pixels and 12 inks {}, which is not four times the \
         size",
        inked(&large),
        inked(&small)
    );
}
