//! The setup sheet's geometry, held against `the-first-start-sheets.xml`.
//!
//! **Every figure here is the capture's**, and the node id is in the assertion
//! that uses it, so the next person checks the file rather than re-deriving a
//! number from a picture.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a sheet that would not lay out is the failure being reported"
)]

use super::{
    THE_ACTION_IS, THE_CHOICE_IS, THE_COLUMN_IS_WIDE, THE_SMALLEST_SHEET_IS, TheTextIsTall,
    setup_sheet_laid_out,
};
use smithay::utils::Size;

/// *10 · Your name*: a title of 88 and an introduction of 25, two choices and
/// two actions — `371:30193`, `371:30194`, `371:30195`, `371:30203`.
fn your_name() -> TheTextIsTall {
    TheTextIsTall {
        title: 88,
        introduction: 25,
        consequence: 22,
        choices: 2,
        actions: 2,
        rows: crate::setup_sheet_layout::TheRowsAre::Apart,
    }
}

/// *11a · Add a PIN*: a title of 44 and an introduction of 50 — `371:30400`
/// and `371:30401`. The same sheet with different words in it.
fn add_a_pin() -> TheTextIsTall {
    TheTextIsTall {
        title: 44,
        introduction: 50,
        consequence: 22,
        choices: 2,
        actions: 2,
        rows: crate::setup_sheet_layout::TheRowsAre::Apart,
    }
}

fn laid_out(w: i32, h: i32, tall: TheTextIsTall) -> super::SetupSheetLayout {
    setup_sheet_laid_out(Size::from((w, h)), tall).expect("this sheet lays out")
}

/// **At the reference size, *Your name* is where the capture puts it.**
#[test]
fn your_name_is_where_the_file_puts_it() {
    let at = laid_out(1440, 960, your_name());

    // `371:30191` — the column at (160, 164), 600 wide.
    assert_eq!(at.column.loc.x, 160, "the column is not 160 in");
    assert_eq!(at.column.loc.y, 164, "the column does not begin at 164");
    assert_eq!(at.column.size.w, THE_COLUMN_IS_WIDE);

    // `371:30192` — the eyebrow at the column's own top, 16 tall.
    assert_eq!(at.eyebrow.loc.y - at.column.loc.y, 0);
    assert_eq!(at.eyebrow.size.h, 16);

    // `371:30193` — the title at y=32 inside the column.
    assert_eq!(
        at.title.loc.y - at.column.loc.y,
        32,
        "`371:30193` is at y=32"
    );
    assert_eq!(at.title.size.h, 88);

    // `371:30194` — the introduction at y=136.
    assert_eq!(
        at.introduction.loc.y - at.column.loc.y,
        136,
        "`371:30194` is at y=136"
    );

    // `371:30195` — the choices at y=177, two rows 95 apart.
    assert_eq!(
        at.choices.first().expect("there are two choices").loc.y - at.column.loc.y,
        177,
        "`371:30195` is at y=177"
    );
    assert_eq!(at.choices.len(), 2);
    let apart = at.choices.get(1).expect("two").loc.y - at.choices.first().expect("two").loc.y;
    assert_eq!(apart, 95, "`371:30199` is 95 below `371:30196`");
    assert_eq!(at.choices.first().expect("two").size.h, THE_CHOICE_IS.1);

    // `371:30202` — the consequence at y=373.
    assert_eq!(
        at.consequence.loc.y - at.column.loc.y,
        373,
        "`371:30202` is at y=373"
    );

    // `371:30203` — the actions at y=411, 172 apart.
    assert_eq!(
        at.actions.first().expect("there are two actions").loc.y - at.column.loc.y,
        411,
        "`371:30203` is at y=411"
    );
    let apart = at.actions.get(1).expect("two").loc.x - at.actions.first().expect("two").loc.x;
    assert_eq!(apart, 172, "`371:30206` is 172 right of `371:30204`");
    assert_eq!(
        at.actions.first().expect("two").size,
        Size::from(THE_ACTION_IS)
    );
}

/// **The same sheet with a shorter title puts everything higher**, which is
/// what the capture's two differing column heights say: 459 and 440.
///
/// A layout that fixed the title's height would draw the choices on top of the
/// introduction the first time a heading was translated into a longer
/// language.
#[test]
fn a_shorter_title_lifts_everything_below_it() {
    let long = laid_out(1440, 960, your_name());
    let short = laid_out(1440, 960, add_a_pin());

    assert!(
        short.choices.first().expect("two").loc.y < long.choices.first().expect("two").loc.y,
        "a shorter title did not lift the choices"
    );

    // `371:30402` — on *Add a PIN* the choices sit at y=158.
    assert_eq!(
        short.choices.first().expect("two").loc.y - short.column.loc.y,
        158,
        "`371:30402` is at y=158"
    );
    // `371:30410` — and its actions at y=392.
    assert_eq!(
        short.actions.first().expect("two").loc.y - short.column.loc.y,
        392,
        "`371:30410` is at y=392"
    );
}

/// **The stage, accessibility and footer are pinned to the edges**, not placed
/// at the reference size's numbers.
#[test]
fn the_edges_are_pinned_to_whatever_sheet_this_is() {
    for (w, h) in [(1440, 960), (1366, 768), (1920, 1080), (1024, 768)] {
        let at = laid_out(w, h, add_a_pin());
        assert_eq!(
            h - at.accessibility.loc.y,
            84,
            "accessibility is not 84 above the bottom of {w}x{h}"
        );
        assert_eq!(
            h - at.footer.loc.y,
            68,
            "the footer is not 68 above the bottom of {w}x{h}"
        );
        assert_eq!(
            w - (at.stage.loc.x + at.stage.size.w),
            70,
            "the stage line is not 70 in from the right of {w}x{h}"
        );
    }
    // At the reference size they land on the capture's own numbers.
    let at = laid_out(1440, 960, add_a_pin());
    assert_eq!(at.accessibility.loc, (40, 876).into(), "`371:30415`");
    assert_eq!(at.footer.loc, (736, 892).into(), "`371:30417`");
    assert_eq!(at.stage.loc, (950, 52).into(), "`371:30397`");
}

/// **Every part stays on the sheet.**
#[test]
fn nothing_falls_off_any_sheet_that_lays_out() {
    for (w, h) in [
        THE_SMALLEST_SHEET_IS,
        (1024, 768),
        (1024, 768),
        (1440, 960),
        (2560, 1440),
    ] {
        let at = laid_out(w, h, add_a_pin());
        let mut every = vec![
            ("the stage", at.stage),
            ("the eyebrow", at.eyebrow),
            ("the title", at.title),
            ("the introduction", at.introduction),
            ("the consequence", at.consequence),
            ("accessibility", at.accessibility),
            ("the footer", at.footer),
        ];
        every.extend(at.choices.iter().map(|it| ("a choice", *it)));
        every.extend(at.actions.iter().map(|it| ("an action", *it)));
        for (what, rectangle) in every {
            assert_eq!(
                at.sheet.intersection(rectangle),
                Some(rectangle),
                "{what} is not wholly on a {w}x{h} sheet: {rectangle:?}"
            );
        }
    }
}

/// **The actions never reach the accessibility control.**
///
/// Two buttons at the same point is the fault this refuses: a sheet whose
/// column has grown down to the bottom edge is refused rather than drawn with
/// one control under another.
#[test]
fn the_column_never_sits_on_the_controls_below_it() {
    for (w, h) in [THE_SMALLEST_SHEET_IS, (1024, 768), (1280, 800), (1440, 960)] {
        let at = laid_out(w, h, add_a_pin());
        let lowest = at.actions.last().map_or(0, |last| last.loc.y + last.size.h);
        assert!(
            lowest < at.accessibility.loc.y,
            "the actions reach accessibility on {w}x{h}"
        );
    }
}

/// **A sheet too small, or too full, is refused** rather than laid out with
/// its parts on top of each other.
#[test]
fn a_sheet_that_cannot_hold_its_contents_is_refused() {
    for (w, h) in [(1, 1), (639, 700), (640, 699), (200, 900)] {
        assert!(
            setup_sheet_laid_out(Size::from((w, h)), add_a_pin()).is_none(),
            "{w}x{h} was laid out and should have been refused"
        );
    }
    // A title long enough to push the actions into the footer is refused too.
    assert!(
        setup_sheet_laid_out(
            Size::from(THE_SMALLEST_SHEET_IS),
            TheTextIsTall {
                title: 4000,
                introduction: 25,
                consequence: 22,
                choices: 2,
                actions: 2,
                rows: crate::setup_sheet_layout::TheRowsAre::Apart,
            }
        )
        .is_none(),
        "a sheet whose title fills it was laid out anyway"
    );
    // The control: the smallest allowed sheet with ordinary text does lay out,
    // so the refusals above are boundaries and not a function that always
    // refuses.
    assert!(
        setup_sheet_laid_out(Size::from(THE_SMALLEST_SHEET_IS), add_a_pin()).is_some(),
        "the smallest sheet with ordinary text was refused"
    );
}

/// **A sheet with one choice and one action lays out too.**
///
/// The journey has eleven frames and only two are captured; a composition that
/// only worked for two-and-two would have to be rewritten at the third.
#[test]
fn a_sheet_of_any_shape_lays_out() {
    for (choices, actions) in [(0, 1), (1, 1), (1, 2), (3, 2), (2, 3)] {
        let at = laid_out(
            1440,
            960,
            TheTextIsTall {
                title: 44,
                introduction: 25,
                consequence: 22,
                choices,
                actions,
                rows: crate::setup_sheet_layout::TheRowsAre::Apart,
            },
        );
        assert_eq!(at.choices.len(), choices);
        assert_eq!(at.actions.len(), actions);
        for action in &at.actions {
            assert_eq!(
                at.sheet.intersection(*action),
                Some(*action),
                "an action of a {choices}-choice {actions}-action sheet is off it"
            );
        }
    }
}

/// **A sheet with nothing to say puts its actions straight under its
/// choices.**
///
/// `185:10770` — *Sign-in options* — has three choices, two actions and no
/// consequence between them: Main is 516 tall, the choices end at 452 and the
/// actions begin at 468. The composition had been read from two sheets that
/// both carry a consequence, so it always placed one, and this frame's buttons
/// would have landed thirty-eight units low.
#[test]
fn a_sheet_with_no_consequence_puts_its_actions_under_its_choices() {
    let at = laid_out(
        1440,
        960,
        TheTextIsTall {
            title: 88,
            introduction: 25,
            consequence: 0,
            choices: 3,
            actions: 2,
            rows: crate::setup_sheet_layout::TheRowsAre::Apart,
        },
    );
    assert_eq!(at.choices.len(), 3, "`371:30583` holds three choices");
    // The third sits 190 below the first, so the 95 spacing holds for a third.
    assert_eq!(
        at.choices.get(2).expect("three").loc.y - at.choices.first().expect("three").loc.y,
        190,
        "`371:30590` is 190 below `371:30584`"
    );
    // `371:30593` — the actions at y=468 inside the column.
    assert_eq!(
        at.actions.first().expect("two").loc.y - at.column.loc.y,
        468,
        "`371:30593` is at y=468"
    );
    assert_eq!(
        at.consequence.size.h, 0,
        "a sheet with nothing to say reserved a box anyway"
    );

    // The control: the same sheet *with* a consequence puts them lower, so the
    // assertion above is about the absence and not about the number 468.
    let with = laid_out(
        1440,
        960,
        TheTextIsTall {
            title: 88,
            introduction: 25,
            consequence: 22,
            choices: 3,
            actions: 2,
            rows: crate::setup_sheet_layout::TheRowsAre::Apart,
        },
    );
    assert!(
        with.actions.first().expect("two").loc.y > at.actions.first().expect("two").loc.y,
        "a consequence did not move the actions down"
    );
}
