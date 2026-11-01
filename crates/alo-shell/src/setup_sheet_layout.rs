//! Where everything on a first-start sheet goes, in logical pixels.
//!
//! Read from `docs/design/figma-snapshot/the-first-start-sheets.xml`, captured
//! from the live file on 2026-10-13. Every number below appears there.
//!
//! # One composition, filled eleven times
//!
//! `ROOTS.md` inventories the whole first start — Welcome, language, region,
//! keyboard, *Your name*, *Protect your computer*, *Sign-in options*, *Add a
//! PIN*, fingerprint, recovery and the AI choice — and says the rule is
//! structural rather than a list: **a frame carries the strip when it is a
//! setup sheet**. The two account sheets captured so far are the same
//! composition with different contents, so this is laid out once and filled
//! many times, exactly as `crate::sign_in_layout` serves both the sign-in and
//! the unlock screen.
//!
//! # Geometry only, so it can be checked against the file
//!
//! Nothing here knows a colour, a font or a word. That is the raster's, and
//! the split is what lets a test assert *the column is 600 wide at (160, 164)*
//! against the capture without rendering a pixel.
//!
//! # The column is read, not answered
//!
//! It is left-aligned at x=160, where the sign-in screen's group is centred.
//! That is the design's own distinction and worth keeping: a setup sheet is
//! read top to bottom and a sign-in screen is answered in one place.
//!
//! # The content decides the height
//!
//! *What should we call you?* wraps to two lines and *Add a PIN?* does not, so
//! the captured columns are 459 and 440 tall and every block below the title
//! moves with it. So the title's height is **given** to this function rather
//! than assumed, and everything after it is placed relative to what came
//! before. A layout that fixed the title's height would put the choices on top
//! of the introduction the first time somebody translated a heading into a
//! longer language.

use smithay::utils::{Logical, Point, Rectangle, Size};

/// How wide the column of content is.
pub const THE_COLUMN_IS_WIDE: i32 = 600;

/// Where that column's corner sits on the reference sheet.
const THE_COLUMN_SITS_AT: (i32, i32) = (160, 164);

/// A choice row: `600 × 85`, and the gap to the next one.
pub const THE_CHOICE_IS: (i32, i32) = (600, 85);

/// From the top of one choice to the top of the next: 95 in the capture.
const THE_CHOICES_ARE_APART_BY: i32 = 95;

/// An action: `160 × 48`, and `172` from one to the next.
pub const THE_ACTION_IS: (i32, i32) = (160, 48);

/// From the left of one action to the left of the next.
const THE_ACTIONS_ARE_APART_BY: i32 = 172;

/// *Accessibility*, at the bottom left: at `(40, 876)`.
const ACCESSIBILITY_SITS_AT: (i32, i32) = (40, 876);

/// *Accessibility*'s own size: `140 × 44`, a white control in a hairline with
/// its icon, as the approved first start draws it (`562:350`, 2026-11-01). It
/// was `160 × 48` and a word alone before that.
pub const ACCESSIBILITY_IS: (i32, i32) = (140, 44);

/// How a sheet's rows sit: each its own card, or together in one.
///
/// **Grouped is the approved first start of 2026-11-01** (`562:342`, `562:462`,
/// `562:508`): one card in a hairline at corner 12, its rows touching and
/// divided by a hairline — 72 tall for the overview's three, 106 for a row
/// that holds a field. Every other sheet keeps the cards the earlier capture
/// drew, 85 tall and 95 apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheRowsAre {
    /// Each its own card, 95 apart.
    Apart,
    /// Together in one card, each this tall.
    Grouped(i32),
}

/// The field inside a grouped row: 15 in from the row's leading side, 17 from
/// its trailing one, and 41 down,
/// 48 tall — `562:486` at (16, 42) in a card whose rows start one in.
#[must_use]
pub fn the_field_in(row: Rectangle<i32, Logical>) -> Rectangle<i32, Logical> {
    Rectangle::new(
        Point::from((row.loc.x + 15, row.loc.y + 41)),
        Size::from(((row.size.w - 32).max(0), 48)),
    )
}

/// The gaps of a grouped sheet, measured off `562:356`: 12 under the eyebrow
/// and under the title, 28 under the introduction, 12 under the card and 28
/// under the note.
mod grouped {
    /// Eyebrow to title: `28 - 16 = 12`.
    pub(super) const AFTER_THE_EYEBROW: i32 = 12;
    /// Title to introduction: `84 - 72 = 12`.
    pub(super) const AFTER_THE_TITLE: i32 = 12;
    /// Introduction to the card: `137 - 109 = 28`.
    pub(super) const AFTER_THE_INTRODUCTION: i32 = 28;
    /// The card to the note: `367 - 355 = 12`.
    pub(super) const AFTER_THE_CHOICES: i32 = 12;
    /// The note to the actions: `417 - 389 = 28`.
    pub(super) const AFTER_THE_CONSEQUENCE: i32 = 28;
}

/// How far the accessibility control's top sits above the bottom edge:
/// `960 - 876 = 84`.
const ACCESSIBILITY_SITS_ABOVE_THE_EDGE_BY: i32 = 84;

/// The footer: `640 × 16` at `(736, 892)`.
const FOOTER_IS: (i32, i32) = (640, 16);

/// How far the footer's right edge sits in from the sheet's: `1440 - 736 -
/// 640 = 64`.
///
/// **Its own inset, not the column's margin.** Placing it at the column's 160
/// put it at x=640 where the capture says 736, and the assertion against
/// `371:30417` caught it.
const FOOTER_SITS_IN_FROM_THE_RIGHT_BY: i32 = 64;

/// How far the footer's top sits above the bottom edge: `960 - 892 = 68`.
const FOOTER_SITS_ABOVE_THE_EDGE_BY: i32 = 68;

/// The stage line: `420 × 18` at `(950, 52)`, which is 70 in from the right.
const STAGE_IS: (i32, i32) = (420, 18);

/// How far the stage line's right edge sits in from the sheet's: `1440 - 950 -
/// 420 = 70`.
const STAGE_SITS_IN_FROM_THE_RIGHT_BY: i32 = 70;

/// How far down the sheet the stage line sits.
const STAGE_SITS_DOWN_BY: i32 = 52;

/// The gaps inside the column, each read from the capture.
mod gaps {
    /// Eyebrow to title: `32 - 16 = 16`.
    pub(super) const AFTER_THE_EYEBROW: i32 = 16;
    /// Title to introduction, on *Your name*: `136 - 32 - 88 = 16`.
    pub(super) const AFTER_THE_TITLE: i32 = 16;
    /// Introduction to choices, on *Your name*: `177 - 136 - 25 = 16`.
    pub(super) const AFTER_THE_INTRODUCTION: i32 = 16;
    /// Choices to consequence, on *Your name*: `373 - 177 - 180 = 16`.
    pub(super) const AFTER_THE_CHOICES: i32 = 16;
    /// Consequence to actions, on *Your name*: `411 - 373 - 22 = 16`.
    pub(super) const AFTER_THE_CONSEQUENCE: i32 = 16;
}

/// **How close the blocks of a sheet sit**: as the account sheets were
/// captured, or tightened for a screen too short for that.
///
/// The tightening is the design's own first answer to a short screen,
/// measured on `385:26174` (*12k · Arrival / 1280 × 720*) and written in
/// `docs/design/the-first-start.md` under *Responsive · Height*: **the gaps
/// between blocks close to 8 and the cards come to 8 apart**, and the heading
/// comes down to 28 on 36 (`crate::sign_in_raster::Measure::tight_title_metrics`).
/// The readable text never shrinks, and nor does a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowClose {
    /// The capture's own gaps: 16 between blocks, cards 95 apart.
    AsCaptured,
    /// The design's short-screen gaps: 8 between blocks, cards 93 apart.
    Tightened,
}

impl HowClose {
    /// The gap between one block of the column and the next.
    const fn between_blocks(self) -> i32 {
        match self {
            Self::AsCaptured => gaps::AFTER_THE_TITLE,
            Self::Tightened => 8,
        }
    }

    /// From the top of one choice to the top of the next.
    const fn choices_apart(self) -> i32 {
        match self {
            Self::AsCaptured => THE_CHOICES_ARE_APART_BY,
            Self::Tightened => THE_CHOICE_IS.1 + 8,
        }
    }
}

/// How far the footer keeps from *Accessibility* where a narrow screen brings
/// them side by side: the 24 the design leaves between a control and the text
/// beside it on the sign-in screen's row of actions.
const THE_FOOTER_KEEPS_FROM_ACCESSIBILITY: i32 = 24;

/// The eyebrow's own height.
const THE_EYEBROW_IS_TALL: i32 = 16;

/// The consequence's own height.
const THE_CONSEQUENCE_IS_TALL: i32 = 22;

/// The smallest sheet this lays out on.
///
/// **Derived rather than chosen**, the way `crate::sign_in_layout`'s is, and
/// the first figure here was wrong. 520 tall was chosen from the width's
/// argument alone — a sheet narrower than 640 cannot hold a 600-wide column
/// with any margin — and the test that lays a sheet out and asks whether every
/// part is on it caught that the *content* does not fit in 520.
///
/// An ordinary sheet is 440 tall below the top of its column: eyebrow, title,
/// introduction, two choices, a consequence and a row of actions, with their
/// gaps. The column begins at `min(164, height / 4)` and the accessibility
/// control sits 84 above the bottom edge, so a sheet needs
/// `height / 4 + 440 < height - 84` — which is `height > 698`.
///
/// Sheets between that and this are refused by `setup_sheet_laid_out` itself
/// rather than drawn with their actions under the accessibility control; this
/// constant is the size at which an ordinary sheet is known to fit.
pub const THE_SMALLEST_SHEET_IS: (i32, i32) = (640, 700);

/// How tall a block of text is, which only the caller knows.
///
/// The title wraps and the introduction wraps, in a language this code has
/// never seen. **Measured by whoever shaped the words**, and handed here, so a
/// longer heading moves what is below it instead of being drawn over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheTextIsTall {
    /// The title's height, after wrapping.
    pub title: i32,
    /// The introduction's height, after wrapping.
    pub introduction: i32,
    /// The consequence's height, after wrapping, or **zero where the sheet
    /// has none**.
    ///
    /// One line in the design and two in a language that needs more words. It
    /// was a fixed 22 until a fresh machine drew its sentence straight through
    /// the button below it.
    ///
    /// **Zero means there is no consequence**, not a consequence of no height:
    /// no box is placed and no gap is left after it. `185:10770` — *Sign-in
    /// options* — has three choices, two actions and nothing between them, and
    /// a layout that always placed one would have put that sheet's buttons
    /// thirty-eight units low. A sheet says what its answer means when it has
    /// something to say.
    pub consequence: i32,
    /// How many choice rows this sheet has.
    pub choices: usize,
    /// How many actions this sheet offers.
    pub actions: usize,
    /// How the rows sit: apart, or grouped in one card.
    pub rows: TheRowsAre,
}

/// Everything on a setup sheet, placed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupSheetLayout {
    /// The whole sheet, which the wallpaper fills.
    pub sheet: Rectangle<i32, Logical>,
    /// The stage line at the top right.
    pub stage: Rectangle<i32, Logical>,
    /// The column of content.
    pub column: Rectangle<i32, Logical>,
    /// The small line above the title.
    pub eyebrow: Rectangle<i32, Logical>,
    /// The heading.
    pub title: Rectangle<i32, Logical>,
    /// The sentence under it.
    pub introduction: Rectangle<i32, Logical>,
    /// Each choice row, in order.
    pub choices: Vec<Rectangle<i32, Logical>>,
    /// The one card a grouped sheet's rows are in, and [`None`] where each row
    /// is its own card.
    pub group: Option<Rectangle<i32, Logical>>,
    /// What this sheet's answer means, under the choices.
    pub consequence: Rectangle<i32, Logical>,
    /// Each action, in order: the first is the one that goes on.
    pub actions: Vec<Rectangle<i32, Logical>>,
    /// *Accessibility*, at the bottom left.
    pub accessibility: Rectangle<i32, Logical>,
    /// The footer line at the bottom right.
    pub footer: Rectangle<i32, Logical>,
}

/// Lay a setup sheet out for a screen of this size, with text of this height.
///
/// [`None`] for a screen smaller than [`THE_SMALLEST_SHEET_IS`], and for a
/// sheet whose blocks would not fit above its own footer — which is a refusal
/// rather than a column drawn over the bottom of the screen.
#[cfg(test)]
#[must_use]
pub fn setup_sheet_laid_out(
    screen: Size<i32, Logical>,
    tall: TheTextIsTall,
) -> Option<SetupSheetLayout> {
    setup_sheet_laid_out_as(screen, tall, HowClose::AsCaptured)
}

/// `setup_sheet_laid_out`, with the blocks as close as `close` says.
#[must_use]
pub fn setup_sheet_laid_out_as(
    screen: Size<i32, Logical>,
    tall: TheTextIsTall,
    close: HowClose,
) -> Option<SetupSheetLayout> {
    // The capture's gaps are all 16, so one figure stands for each of them;
    // a grouped sheet has the approved first start's own, unless tightened.
    let between = close.between_blocks();
    let tight = close == HowClose::Tightened;
    let gap = |captured: i32, approved: i32| match (tall.rows, tight) {
        (TheRowsAre::Grouped(_), false) => approved,
        _ => between.min(captured),
    };
    if screen.w < THE_SMALLEST_SHEET_IS.0 || screen.h < THE_SMALLEST_SHEET_IS.1 {
        return None;
    }
    let sheet = Rectangle::from_size(screen);

    // **The column, never wider than the screen less a margin each side.**
    let margin = THE_COLUMN_SITS_AT.0.min(screen.w / 8);
    let column_wide = THE_COLUMN_IS_WIDE.min(screen.w - 2 * margin);
    if column_wide <= 0 {
        return None;
    }
    let left = margin;
    let top = THE_COLUMN_SITS_AT.1.min(screen.h / 4);

    let line = |y: i32, h: i32| {
        Rectangle::new(Point::from((left, y)), Size::from((column_wide, h.max(0))))
    };

    let eyebrow = line(top, THE_EYEBROW_IS_TALL);
    let title = line(
        eyebrow.loc.y + eyebrow.size.h + gap(gaps::AFTER_THE_EYEBROW, grouped::AFTER_THE_EYEBROW),
        tall.title,
    );
    let introduction = line(
        title.loc.y + title.size.h + gap(between, grouped::AFTER_THE_TITLE),
        tall.introduction,
    );

    let choices_begin = introduction.loc.y
        + introduction.size.h
        + gap(
            gaps::AFTER_THE_INTRODUCTION,
            grouped::AFTER_THE_INTRODUCTION,
        );
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "a sheet offers a handful of choices, never two billion"
    )]
    let how_many = tall.choices as i32;
    let (choices, group): (Vec<_>, _) = match tall.rows {
        TheRowsAre::Apart => (
            (0..how_many)
                .map(|which| {
                    Rectangle::new(
                        Point::from((left, choices_begin + which * close.choices_apart())),
                        Size::from((column_wide, THE_CHOICE_IS.1)),
                    )
                })
                .collect(),
            None,
        ),
        // **One card, its rows inside its hairline and touching**: a row is
        // the card's width less the hairline each side, and the card is the
        // rows and the hairline above and below them.
        TheRowsAre::Grouped(row_tall) => (
            (0..how_many)
                .map(|which| {
                    Rectangle::new(
                        Point::from((left + 1, choices_begin + 1 + which * row_tall)),
                        Size::from((column_wide - 2, row_tall)),
                    )
                })
                .collect(),
            (how_many > 0).then(|| {
                Rectangle::new(
                    Point::from((left, choices_begin)),
                    Size::from((column_wide, how_many * row_tall + 2)),
                )
            }),
        ),
    };
    let choices_end = group.map_or_else(
        || {
            choices
                .last()
                .map_or(choices_begin, |last| last.loc.y + last.size.h)
        },
        |group| group.loc.y + group.size.h,
    );

    let consequence = line(
        choices_end + gap(gaps::AFTER_THE_CHOICES, grouped::AFTER_THE_CHOICES),
        if tall.consequence > 0 {
            tall.consequence.max(THE_CONSEQUENCE_IS_TALL)
        } else {
            0
        },
    );
    // A sheet with nothing to say skips both the box and the gap after it,
    // which is what puts `185:10770`'s actions 16 below its choices.
    let actions_top = if tall.consequence > 0 {
        consequence.loc.y
            + consequence.size.h
            + gap(gaps::AFTER_THE_CONSEQUENCE, grouped::AFTER_THE_CONSEQUENCE)
    } else {
        choices_end + gap(gaps::AFTER_THE_CHOICES, grouped::AFTER_THE_CONSEQUENCE)
    };
    let actions: Vec<_> = (0..tall.actions)
        .map(|which| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_possible_wrap,
                reason = "a sheet offers a handful of actions, never two billion"
            )]
            let which = which as i32;
            Rectangle::new(
                Point::from((left + which * THE_ACTIONS_ARE_APART_BY, actions_top)),
                Size::from(THE_ACTION_IS),
            )
        })
        .collect();

    // **Pinned to the bottom edge**, not placed at 876 and 892: a fixed number
    // is off the bottom of a shorter screen and halfway up a taller one, which
    // is the fault `crate::sign_in_layout`'s actions row records.
    let accessibility = Rectangle::new(
        Point::from((
            ACCESSIBILITY_SITS_AT.0.min(screen.w / 8),
            screen.h - ACCESSIBILITY_SITS_ABOVE_THE_EDGE_BY,
        )),
        Size::from(ACCESSIBILITY_IS),
    );
    // **Never under *Accessibility*.** On a narrow screen the footer's 640
    // reached back over the control at the bottom left — at 640 wide it began
    // at 64, inside a control that ends at 200 — so it starts no further left
    // than 24 past that control's right edge, and is narrower for it.
    let footer_right = screen.w - FOOTER_SITS_IN_FROM_THE_RIGHT_BY;
    let footer_left = (footer_right - FOOTER_IS.0)
        .max(accessibility.loc.x + accessibility.size.w + THE_FOOTER_KEEPS_FROM_ACCESSIBILITY);
    let footer = Rectangle::new(
        Point::from((footer_left, screen.h - FOOTER_SITS_ABOVE_THE_EDGE_BY)),
        Size::from(((footer_right - footer_left).max(0), FOOTER_IS.1)),
    );
    let stage_wide = STAGE_IS.0.min(screen.w - 2 * margin).max(0);
    let stage = Rectangle::new(
        Point::from((
            screen.w - STAGE_SITS_IN_FROM_THE_RIGHT_BY - stage_wide,
            STAGE_SITS_DOWN_BY.min(screen.h / 8),
        )),
        Size::from((stage_wide, STAGE_IS.1)),
    );

    // **Refused rather than overlapped.** A sheet whose actions reach the
    // accessibility control has nowhere to put them, and drawing it anyway
    // would put a button under a button.
    let lowest = actions
        .last()
        .map_or(actions_top, |last| last.loc.y + last.size.h);
    if lowest >= accessibility.loc.y {
        return None;
    }
    let column = Rectangle::new(
        Point::from((left, top)),
        Size::from((column_wide, lowest - top)),
    );

    Some(SetupSheetLayout {
        sheet,
        stage,
        column,
        eyebrow,
        title,
        introduction,
        choices,
        group,
        consequence,
        actions,
        accessibility,
        footer,
    })
}

#[cfg(test)]
#[path = "setup_sheet_layout_tests.rs"]
mod tests;
