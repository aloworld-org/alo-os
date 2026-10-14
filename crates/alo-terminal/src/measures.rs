//! The terminal's own numbers, and what each of them answers to.
//!
//! Every figure here was measured off the design file and is recorded, with how
//! it was established, in `docs/design/the-terminal-window.md`. This is that
//! document turned into arithmetic, in one file, so that changing one of them
//! is a visible act rather than a discovery.
//!
//! # The unit, and why nothing here is a pixel
//!
//! Everything is in **logical pixels**: the units a compositor lays a screen
//! out in, before it is scaled for a dense display. A machine with twice the
//! pixels draws the same terminal twice as sharp rather than half as big.
//!
//! `CLAUDE.md` names the fault these are shaped to avoid: *nothing is built to
//! one screen size — the design's frames are 1440×960 and no machine is obliged
//! to be.* The terminal's frames are 1440×1040, which makes the point for
//! itself. **So a figure here reaches a display only through that display's own
//! scale**, and the usual way the rule is broken is a number scaled by the
//! person's text size *only*, which looks derived and is half the size it should
//! be on a dense screen.
//!
//! # What is here, and what deliberately is not
//!
//! Here: the figures the design states and the ones that follow from them by
//! arithmetic. **Not here: the window's own size.** The design draws it at
//! 1040×800, and that is the size it was drawn at rather than the size a window
//! is — a terminal is as big as the person made it. The figures below are what
//! stay fixed as it changes.
//!
//! That is the distinction `CLAUDE.md` calls the fourth kind of number, *another
//! surface's current extent*, which can only be asked for. The transcript's
//! height is one: it is not 706, it is **what the body leaves after the identity
//! strip**, and 706 is what that came to at 800.

/// How tall one line of the transcript is.
///
/// **The design states it twice, from two independent directions**, which is
/// the reason to trust it rather than the figure itself:
///
/// - every one of the **123** `Terminal / Source region` frames in the snapshot
///   is 22, 44, 66 or 88 tall — one, two, three or four lines — with no
///   exceptions;
/// - and the `Terminal/Command` text style, read from the live file, is *Geist
///   Mono Medium, size 14, weight 500, **line height 22***.
///
/// Geometry alone could have been a coincidence of the frames, and a token alone
/// could have been unused. The two agreeing is the measurement.
pub const A_LINE: u32 = 22;

/// The type size the transcript's text is drawn at.
///
/// From `Terminal/Command`. Separate from [`A_LINE`] because they are different
/// questions — how big a letter is drawn, and how far apart the lines sit — and
/// a single number serving both is how a terminal ends up with its descenders
/// clipped.
pub const A_LETTER: u32 = 14;

/// The weight the transcript's text is drawn at: `Medium`.
///
/// **It is in this file because getting it wrong does not look like a weight
/// error.** Measured 2026-10-14: asking for Geist Mono at this weight drew
/// **Inter**, proportional, with no error at all — fontdb registers a variable
/// font as one face at its default instance, and cosmic-text honours a family
/// only among faces whose weight matches *exactly*, so missing it by one step
/// throws the family away. `docs/quirks.md` carries the mechanism.
///
/// So this is not a nicety to be rounded to whatever is loaded. A terminal drawn
/// at some other weight is a terminal drawn in another typeface.
pub const THE_WEIGHT: u16 = 500;

/// How far a source region is inset from each side of the transcript.
///
/// **Measured at two different transcript widths and the same at both** — 990
/// inside 1038, and 670 inside 718 — so it is a fixed measure and not a
/// proportion of the width. That is the distinction worth having checked:
/// `the_inset_does_not_scale_with_the_width` is the test.
pub const AN_INSET: u32 = 24;

/// The border the window body carries on every side.
///
/// Derived, and stated as derived: the design's children begin at (1, 1) and are
/// 1038 wide in a 1040 body, so the border is `(1040 - 1038) / 2`. The file does
/// not label it.
pub const A_BORDER: u32 = 1;

/// How tall the strip above the transcript is.
///
/// The design's `Project and session identity`, 1038×56 occupying rows 1 to 57
/// of the body — so it sits inside the border and spans the body.
///
/// **The sessions strip is the same height and is not here.** The design places
/// it over the transcript's own rows (73 to 129, inside the transcript's 57 to
/// 763), so which of the two is drawn, and when, is not settled by the file.
/// `docs/design/the-terminal-window.md` records that silence; a measure for
/// something nobody has decided to draw would be a guess wearing a constant's
/// clothes.
pub const A_STRIP: u32 = 56;

/// How tall the note under the transcript is.
///
/// The design's `Task scope note`, 1038×28, holding the shell and the encoding
/// — `zsh · UTF-8` in the frames measured.
///
/// **It was found by arithmetic, not by reading the list.** The transcript ends
/// at row 763 of an 800-tall body, so 37 rows had no owner; this is 28 of them.
/// That is why [`transcript_height_in`] below balances the whole body rather
/// than subtracting the two things somebody remembered.
pub const A_NOTE: u32 = 28;

/// How much of the body sits below the note, bottom border included.
///
/// Rows `791` to `800`. Measured rather than explained: the design does not
/// label it, and it is here so that [`transcript_height_in`] adds up exactly
/// instead of being nine short.
///
/// **It includes the bottom border**, which is why [`A_BORDER`] is subtracted
/// once and not twice. Counting the border at both ends made the body come to
/// 801 and the transcript to 705 — caught by
/// `the_body_is_fully_accounted_for`, which is the assertion that exists for
/// exactly this kind of double count.
pub const BELOW_THE_NOTE: u32 = 9;

/// How far the transcript's own content begins below its top.
///
/// The transcript's single text child sits at y 20 inside it. Separate from
/// [`AN_INSET`], which is horizontal and measured at two widths; this one is
/// vertical and measured once, and the doc says so rather than implying a
/// symmetry the file does not state.
pub const CONTENT_BELOW_THE_TOP: u32 = 20;

/// How wide the terminal's own gutter is, at the left of every source region.
///
/// **The column a program cannot write into** —
/// [ADR 0113](../../../docs/decisions/0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md).
/// The design's annotation states the rule in words: *source gutter:
/// terminal-owned, **outside clipped program content***. So this is not an
/// overlay drawn on top of the cells; it is room taken away from them before
/// they are laid out, which is what makes the promise structural rather than
/// cosmetic.
///
/// Measured twice, on two different nodes, and the second was the one that
/// mattered:
///
/// - inside `Program content boundary / protected source outside` (990 wide),
///   the frame named `Rich test result / untrusted program data` is 894 wide at
///   x **96**, and `96 + 894 = 990` exactly;
/// - and in a **plain** `Terminal / Source region` — no rich result anywhere in
///   it — the gutter holds `you` and the content's ink resumes at x **97**.
///
/// The second is why this is unconditional. A figure taken only from the rich
/// frame would have been a measure of rich results, and a plain transcript
/// would have drawn its cells straight through the gutter.
pub const A_GUTTER: u32 = 96;

/// How much of [`A_GUTTER`] the label itself is given.
///
/// The design's text node is 80 wide in the 96-wide column, so **16 separates
/// the label from the program's first character**. The word drawn in it is
/// narrower again — `program` measures 51 — so the 80 is the box the label is
/// laid out in and not the width of any particular word.
///
/// It is here rather than derived from the longest word because the longest
/// word changes with the language, and `CLAUDE.md` asks for all 24 official EU
/// languages. A box the design set is a measure; the widest translation is a
/// race.
pub const THE_LABELS_ROOM: u32 = 80;

/// The type size the gutter's label is drawn at.
///
/// The design's `Label/Medium`, asked of the live file for the label node
/// itself rather than its parent: *Manrope SemiBold, size 13, weight 600, line
/// height 18, letter spacing 0.1*.
///
/// **Deliberately not [`A_LETTER`].** The label is 13 where the transcript is
/// 14, and in another family — alo's own rather than the mono — which is what
/// makes a glance tell them apart before any word is read.
pub const THE_LABEL: u32 = 13;

/// The line height the gutter's label is laid out on, from `Label/Medium`.
///
/// Eighteen in a row that is [`A_LINE`] tall, so the label sits inside the
/// line it labels with two above and two below.
pub const THE_LABELS_LINE: u32 = 18;

/// The weight the gutter's label is drawn at: `SemiBold`, from `Label/Medium`.
///
/// The same weight as alo's own face everywhere else, and in this file for the
/// reason [`THE_WEIGHT`] is: a weight missed by one step does not look like a
/// weight error, it silently throws the family away.
pub const THE_LABELS_WEIGHT: u16 = 600;

/// How wide a program's own content is, inside a transcript this wide.
///
/// [`a_source_region_inside`] less [`A_GUTTER`]: the region, less the column
/// the terminal keeps. Saturating for the same reason, and it is the figure the
/// grid is counted in — **a column of cells that reached the gutter would be a
/// program writing where the design says it cannot**.
#[must_use]
pub const fn a_programs_room_inside(transcript_width: u32) -> u32 {
    a_source_region_inside(transcript_width).saturating_sub(A_GUTTER)
}

/// How wide a source region is, inside a transcript this wide.
///
/// The arithmetic the two measured pairs satisfy, rather than either of the
/// numbers they were measured as. Saturating, so a transcript narrower than its
/// own insets answers zero rather than wrapping — which is a real case on a
/// small display and not a defensive flourish.
#[must_use]
pub const fn a_source_region_inside(transcript_width: u32) -> u32 {
    transcript_width.saturating_sub(AN_INSET * 2)
}

/// How many whole lines fit in a transcript this tall.
///
/// **A count, never a constant.** `80×24` is a figure a test chooses and not one
/// a window has: how many lines a terminal shows is how many times [`A_LINE`]
/// fits in what the window gave it.
#[must_use]
pub const fn lines_that_fit(transcript_height: u32) -> u32 {
    transcript_height / A_LINE
}

/// What the transcript's height is, given the body it sits in.
///
/// The body, less everything else in it: the border top and bottom, the
/// identity strip above, the task scope note below, and the nine rows below
/// that.
///
/// **It reproduces the design's own figure exactly** — at a body of 800,
/// `800 − 1 − 56 − 28 − 9` is **706**, which is what the design draws.
///
/// That exactness is the check that this file knows what is in the body rather
/// than what somebody remembered being in it, and it caught two mistakes while
/// being written. The first version subtracted the border and the identity
/// strip only, answered **742**, and was short by the task scope note and the
/// rows under it — an element missing from the design record as well, until the
/// arithmetic refused to balance. The second counted the border at both ends,
/// answered **705**, because the bottom border is already inside
/// [`BELOW_THE_NOTE`].
#[must_use]
pub const fn transcript_height_in(body_height: u32) -> u32 {
    body_height
        .saturating_sub(A_BORDER)
        .saturating_sub(A_STRIP)
        .saturating_sub(A_NOTE)
        .saturating_sub(BELOW_THE_NOTE)
}

#[cfg(test)]
#[path = "measures_tests.rs"]
mod tests;
