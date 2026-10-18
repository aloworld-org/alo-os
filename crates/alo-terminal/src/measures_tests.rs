//! **The design's own figures, asserted against the arithmetic.**
//!
//! A measures file is a list of numbers somebody typed, and a typo in one is
//! invisible. What is not invisible is a *relation* between them that the design
//! file also states — so every test here checks one of those, and each names the
//! figure the design draws so a reader can go and look.
//!
//! The figures and their provenance are in
//! `docs/design/the-terminal-window.md`; this is where they meet arithmetic.

use super::a_bars_length;
use super::{
    A_BORDER, A_LETTER, A_LINE, A_NOTE, A_STRIP, AN_INSET, BELOW_THE_NOTE, CONTENT_BELOW_THE_TOP,
    THE_WEIGHT, a_source_region_inside, lines_that_fit, transcript_height_in,
};

/// The body the design draws the window at. **Not a measure** — it is the size
/// the design was drawn at, and a window is as big as the person made it. It is
/// here only so the tests can check the arithmetic against the drawing.
const THE_DESIGNS_BODY: u32 = 800;

/// What the design draws the transcript as, in that body.
const THE_DESIGNS_TRANSCRIPT: u32 = 706;

/// The design's two transcript widths, and the source region measured in each.
const THE_DESIGNS_WIDTHS: [(u32, u32); 2] = [(1038, 990), (718, 670)];

/// **The rule reproduces the design's own figure, exactly.**
///
/// This is the test that found the file's first mistake. `transcript_height_in`
/// originally subtracted the border and the identity strip only, answered 742,
/// and was short by exactly the task scope note and the nine rows below it —
/// an element that was missing from the design record as well, until the
/// arithmetic refused to balance.
///
/// So this is not a restatement of the constants. It is the one assertion that
/// catches a *missing* term, which no amount of checking the terms present can
/// do.
#[test]
fn the_rule_reproduces_the_designs_own_transcript() {
    assert_eq!(
        transcript_height_in(THE_DESIGNS_BODY),
        THE_DESIGNS_TRANSCRIPT,
        "the design draws a {THE_DESIGNS_TRANSCRIPT}-tall transcript in an \
         {THE_DESIGNS_BODY}-tall body, and the rule answers {}. Something in the body is \
         missing from this file or has the wrong figure.",
        transcript_height_in(THE_DESIGNS_BODY)
    );
}

/// **Everything in the body adds up to the body**, which is the same check from
/// the other end: it fails if a term is wrong even when two errors would cancel
/// in the subtraction above.
#[test]
fn the_body_is_fully_accounted_for() {
    // **The border once, not twice**: the bottom one is inside
    // `BELOW_THE_NOTE`, which is rows 791 to 800 of the body.
    let accounted = A_BORDER + A_STRIP + THE_DESIGNS_TRANSCRIPT + A_NOTE + BELOW_THE_NOTE;
    assert_eq!(
        accounted,
        THE_DESIGNS_BODY,
        "the parts of the body come to {accounted} and the body is {THE_DESIGNS_BODY}, so \
         {} rows have no owner",
        THE_DESIGNS_BODY.abs_diff(accounted)
    );
}

/// **The transcript begins where the border and the strip leave off.**
///
/// The design places it at row 57, and `1 + 56` is 57. A separate assertion from
/// the height, because a transcript the right height in the wrong place draws
/// perfectly and over the identity strip.
#[test]
fn the_transcript_begins_below_the_strip() {
    assert_eq!(
        A_BORDER + A_STRIP,
        57,
        "the design places the transcript at row 57 of the body"
    );
}

/// **The inset is a fixed measure and not a proportion**, which is why it is a
/// constant at all.
///
/// Measured at two transcript widths 320 apart — 990 inside 1038 and 670 inside
/// 718 — and the same at both. Had it been a proportion, the narrower one would
/// have been inset about 17 rather than 24.
#[test]
fn the_inset_does_not_scale_with_the_width() {
    for (transcript, region) in THE_DESIGNS_WIDTHS {
        assert_eq!(
            a_source_region_inside(transcript),
            region,
            "a {transcript}-wide transcript should hold a {region}-wide source region"
        );
    }
    // And the control: the two widths really are different, so the loop above
    // is testing the claim and not the same case twice.
    let [(wide, _), (narrow, _)] = THE_DESIGNS_WIDTHS;
    assert_ne!(
        wide, narrow,
        "both measured widths are the same, so this test cannot tell a fixed inset from a \
         proportional one"
    );
}

/// **A transcript narrower than its own insets answers zero, not a wrapped
/// number.** A real case on a small display, and `u32` subtraction would wrap.
#[test]
fn a_transcript_too_narrow_for_its_insets_has_no_room() {
    for narrow in [0, 1, AN_INSET, AN_INSET * 2 - 1, AN_INSET * 2] {
        assert_eq!(
            a_source_region_inside(narrow),
            0,
            "a {narrow}-wide transcript has no room for a source region once inset \
             {AN_INSET} each side"
        );
    }
    assert_eq!(a_source_region_inside(AN_INSET * 2 + 1), 1);
}

/// **A body too small for its furniture leaves no transcript**, rather than
/// wrapping to four billion rows.
#[test]
fn a_body_too_small_leaves_no_transcript() {
    let furniture = A_BORDER + A_STRIP + A_NOTE + BELOW_THE_NOTE;
    for small in [0, 1, A_STRIP, furniture - 1, furniture] {
        assert_eq!(
            transcript_height_in(small),
            0,
            "a {small}-tall body cannot hold {furniture} of furniture and a transcript too"
        );
    }
    assert_eq!(transcript_height_in(furniture + A_LINE), A_LINE);
}

/// **Lines are counted, never assumed.**
///
/// The design's own transcript holds 32 whole lines of 22 — `706 / 22` is 32
/// with 2 left over — and **not 24**, which is the figure a test would have
/// chosen. That difference is the point: a terminal's grid comes from the window
/// it was given.
#[test]
fn lines_are_counted_from_the_height_rather_than_assumed() {
    assert_eq!(lines_that_fit(THE_DESIGNS_TRANSCRIPT), 32);
    assert_ne!(
        lines_that_fit(THE_DESIGNS_TRANSCRIPT),
        24,
        "the design's transcript holds 32 lines, not the 24 of a conventional terminal"
    );
    // Exact multiples and the row below one, so the division is not off by one.
    assert_eq!(lines_that_fit(A_LINE), 1);
    assert_eq!(lines_that_fit(A_LINE - 1), 0);
    assert_eq!(lines_that_fit(A_LINE * 10), 10);
    assert_eq!(lines_that_fit(A_LINE * 10 - 1), 9);
}

/// **The type size and the line height are two different numbers**, and a file
/// that let them become one is how descenders get clipped.
///
/// From `Terminal/Command`: size 14, line height 22. The leading is the
/// difference, and a line height below the letter would be a transcript whose
/// rows overlap.
#[test]
fn a_line_is_taller_than_the_letter_in_it() {
    // **Checked when this crate is compiled, not when it is tested.** Both are
    // constants, so clippy's own suggestion is the stronger one: a `const`
    // block makes a line shorter than its letter fail the **build** rather
    // than a test run.
    const {
        assert!(
            A_LINE > A_LETTER,
            "a line must be taller than the letter in it"
        )
    };
    assert_eq!(A_LINE - A_LETTER, 8, "the design's leading");
}

/// **The weight is the design's `Medium` and not a default.**
///
/// Asserted because this is the figure whose being wrong does not look like a
/// weight error: at any other weight the family is thrown away entirely and the
/// transcript is drawn in a proportional face. `docs/quirks.md` carries the
/// mechanism, and the guard that the face really resolves at this weight lives
/// in `alo-shell`, next to the font system that loads it.
#[test]
fn the_weight_is_the_designs_own() {
    assert_eq!(THE_WEIGHT, 500, "`Terminal/Command` is Geist Mono Medium");
    assert_ne!(
        THE_WEIGHT, 400,
        "400 is the weight the variable face happens to register at, which is what made this \
         fault invisible — asking at the default looked correct"
    );
}

/// **The vertical content inset is its own figure**, measured once, and not
/// assumed equal to the horizontal one.
#[test]
fn the_content_insets_are_not_assumed_symmetric() {
    assert_eq!(CONTENT_BELOW_THE_TOP, 20);
    assert_ne!(
        CONTENT_BELOW_THE_TOP, AN_INSET,
        "these two are measured separately and happen to differ; a file that set one from the \
         other would have been wrong by four"
    );
}

/// **The design's own two bars come back at the design's own two lengths.**
///
/// 240 ms and 180 ms on a 706-wide track, which is what `479:28967` has after
/// its label, its value and the two gutters. If this ever stops answering 480
/// and 360, either a measure moved or the scale did.
#[test]
fn the_designs_two_bars_are_the_designs_two_lengths() {
    let longest = 240_000;
    assert_eq!(a_bars_length(240_000, longest, 706), 480);
    assert_eq!(a_bars_length(180_000, longest, 706), 360);
}

/// **A bar never leaves its track.**
#[test]
fn a_bar_past_the_track_is_held_to_it() {
    let longest = 10_000_000;
    assert_eq!(
        a_bars_length(longest, longest, 706),
        706,
        "the slowest run fills the track exactly and does not pass it"
    );
}

/// **Rescaling keeps the ratio**, which is what makes two bars comparable.
///
/// A comparison that overflowed and then clamped would draw two full bars for
/// two different durations — a chart that says the opposite of the numbers
/// beside it.
#[test]
fn rescaling_past_the_track_keeps_the_ratio_between_bars() {
    let (slow, fast) = (1_000_000, 500_000);
    let long = a_bars_length(slow, slow, 706);
    let short = a_bars_length(fast, slow, 706);
    assert_eq!(long, 706);
    assert_eq!(
        short * 2,
        long,
        "half the duration is half the bar, whatever the scale"
    );
}

/// **A comparison of equal runs draws equal bars**, and neither is zero.
#[test]
fn two_equal_durations_draw_two_equal_bars() {
    assert_eq!(
        a_bars_length(240_000, 240_000, 706),
        a_bars_length(240_000, 240_000, 706)
    );
    assert!(a_bars_length(240_000, 240_000, 706) > 0);
}

/// **Nothing to scale against draws nothing**, rather than dividing by zero.
#[test]
fn a_longest_of_nothing_or_a_track_of_nothing_draws_nothing() {
    assert_eq!(a_bars_length(240_000, 0, 706), 0);
    assert_eq!(a_bars_length(240_000, 240_000, 0), 0);
}
