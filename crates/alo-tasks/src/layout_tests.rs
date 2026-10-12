//! That a task window lays itself out from its own size, that it matches the
//! design's frame where the design has an opinion, and that it refuses a size
//! it cannot honestly fill.
#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic naming the size it was laying out is the failure being reported"
)]

use super::{TheRows, TooSmall, the_least_height, the_least_width};
use crate::measures as m;

/// The size the design's frames were drawn at.
const AS_DRAWN: (u32, u32) = (1040, 800);

/// The design's own figures for the five rows, from
/// `docs/design/the-ask-alo-task-window.md`.
const AS_DESIGNED: [(&str, u32); 5] = [
    ("heading", 76),
    ("shared files", 52),
    ("conversation", 484),
    ("composer", 150),
    ("scope note", 28),
];

/// Lay the design's own size out, which must succeed.
fn as_drawn() -> TheRows {
    let (wide, tall) = AS_DRAWN;
    TheRows::of(wide, tall).expect("the size the design was drawn at")
}

/// **Four of the five rows are exactly the design's.**
#[test]
fn the_fixed_rows_are_the_designs_own_figures() {
    let rows = as_drawn();

    assert_eq!(rows.heading.tall, AS_DESIGNED[0].1);
    assert_eq!(rows.shared_files.tall, AS_DESIGNED[1].1);
    assert_eq!(rows.composer.tall, AS_DESIGNED[3].1);
    assert_eq!(rows.scope_note.tall, AS_DESIGNED[4].1);
}

/// **The conversation is eight taller than the frame, deliberately.**
///
/// The design's five heights sum to 790 inside an inner 798, and the eight are
/// unlabelled — trailing space below the scope note, because the frame's
/// children stack from the top. This gives them to the scrolling row instead.
/// The test asserts the decision *and* its size, so a later change that quietly
/// moved the residue somewhere else fails here rather than looking like a
/// rounding difference.
#[test]
fn the_conversation_takes_the_eight_the_design_leaves_at_the_bottom() {
    let rows = as_drawn();
    let designed = AS_DESIGNED[2].1;

    assert_eq!(
        rows.conversation.tall,
        designed + 8,
        "the conversation absorbs the residue"
    );

    let summed: u32 = AS_DESIGNED.iter().map(|(_, tall)| tall).sum();
    assert_eq!(summed, 790, "the design's own five heights");
    assert_eq!(
        AS_DRAWN.1 - (m::BORDER * 2) - summed,
        8,
        "and this is where the eight came from"
    );
}

/// **The rows meet exactly: no gap and no overlap, and they fill the window.**
///
/// The property that catches an arithmetic slip anywhere in the chain, at any
/// size — which a comparison against the design's figures alone cannot do,
/// because the design has one size.
#[test]
fn the_rows_meet_exactly_and_fill_the_window_at_every_size() {
    for (wide, tall) in [
        AS_DRAWN,
        (600, 400),
        (1440, 900),
        (1920, 1080),
        (3840, 2160),
        (the_least_width(), the_least_height()),
    ] {
        let rows = TheRows::of(wide, tall).unwrap_or_else(|why| {
            panic!("{wide}x{tall} should lay out: {why:?}");
        });
        let every = [
            rows.heading,
            rows.shared_files,
            rows.conversation,
            rows.composer,
            rows.scope_note,
        ];

        assert_eq!(every[0].from_the_top, 0, "{wide}x{tall}: the first row");
        for pair in every.windows(2) {
            let (above, below) = (pair[0], pair[1]);
            assert_eq!(
                above.past_its_bottom(),
                below.from_the_top,
                "{wide}x{tall}: a gap or an overlap between rows"
            );
        }
        assert_eq!(
            every[4].past_its_bottom(),
            tall - (m::BORDER * 2),
            "{wide}x{tall}: the rows do not fill the window"
        );
        for row in every {
            assert_eq!(
                row.wide,
                wide - (m::BORDER * 2),
                "{wide}x{tall}: a row is not the window's width"
            );
        }
    }
}

/// **Only the conversation changes with the window's height.**
#[test]
fn making_the_window_taller_only_grows_the_conversation() {
    let short = TheRows::of(1040, 800).expect("800");
    let tall = TheRows::of(1040, 1200).expect("1200");

    assert_eq!(tall.conversation.tall, short.conversation.tall + 400);
    assert_eq!(tall.heading.tall, short.heading.tall);
    assert_eq!(tall.shared_files.tall, short.shared_files.tall);
    assert_eq!(tall.composer.tall, short.composer.tall);
    assert_eq!(tall.scope_note.tall, short.scope_note.tall);
}

/// **A window too short to hold a conversation is refused**, and the refusal
/// says what the least is.
///
/// Refused rather than laid out with a conversation of nothing, and refused
/// rather than silently clamped: a caller that asked for an impossible size has
/// a bug, and a layout that quietly produced its own size would hide it.
#[test]
fn a_window_too_short_is_refused_and_says_the_least() {
    let least = the_least_height();

    assert_eq!(
        TheRows::of(1040, least - 1),
        Err(TooSmall::NotTallEnough { the_least: least })
    );
    assert!(
        TheRows::of(1040, least).is_ok(),
        "and the least itself is allowed"
    );
    assert_eq!(
        TheRows::of(1040, least)
            .expect("the least")
            .conversation
            .tall,
        m::THE_LEAST_CONVERSATION,
        "at the least height the conversation is exactly its floor"
    );
}

/// **A window too narrow for the composer's controls is refused.**
#[test]
fn a_window_too_narrow_is_refused_and_says_the_least() {
    let least = the_least_width();

    assert_eq!(
        TheRows::of(least - 1, 800),
        Err(TooSmall::NotWideEnough { the_least: least })
    );
    assert!(TheRows::of(least, 800).is_ok());
}

/// **Width is refused before height**, so a window that is too small in both
/// gets one answer rather than depending on the order of the tests.
#[test]
fn a_window_too_small_in_both_names_its_width() {
    assert_eq!(
        TheRows::of(1, 1),
        Err(TooSmall::NotWideEnough {
            the_least: the_least_width()
        })
    );
}

/// **The controls are the design's own positions at the design's own size.**
///
/// Measured off the frame: attach at 44, model at 100, destination at 284,
/// permissions at 440, the button at 944, with 440 of flexible space between
/// the last control and the button.
#[test]
fn the_controls_sit_where_the_design_puts_them() {
    let rows = as_drawn();
    let controls = rows.controls();

    assert_eq!(controls.attach, 44);
    assert_eq!(controls.model, 100);
    assert_eq!(controls.destination, 284);
    assert_eq!(controls.permissions, 440);
    assert_eq!(controls.the_one_button, 944);
    assert_eq!(controls.flexible_space, 440);
    // **Eight lower than the frame draws it**, which is the cost of giving the
    // residue to the conversation. Asserted as the figure *and* as the
    // relationship, so the reason is visible beside the number.
    assert_eq!(controls.from_the_top, 696);
    assert_eq!(
        controls.from_the_top,
        rows.composer.from_the_top + m::ABOVE_THE_FIELD + m::THE_FIELD_IS_TALL
            - m::BELOW_THE_FIELDS_CONTENTS
            - m::A_CONTROL_IS,
        "the row sits at the bottom of the field"
    );
    assert_eq!(
        controls.from_the_top + m::A_CONTROL_IS + m::BELOW_THE_FIELDS_CONTENTS,
        rows.composer.from_the_top + m::ABOVE_THE_FIELD + m::THE_FIELD_IS_TALL,
        "and the field's bottom padding is below it"
    );
}

/// **Only the flexible space absorbs a change in width**, so the controls stay
/// put on the left and the one button stays at the right-hand end.
#[test]
fn widening_the_window_moves_only_the_one_button() {
    let narrow = as_drawn().controls();
    let wide = TheRows::of(1440, 800).expect("1440").controls();

    assert_eq!(wide.attach, narrow.attach);
    assert_eq!(wide.model, narrow.model);
    assert_eq!(wide.destination, narrow.destination);
    assert_eq!(wide.permissions, narrow.permissions);
    assert_eq!(wide.flexible_space, narrow.flexible_space + 400);
    assert_eq!(wide.the_one_button, narrow.the_one_button + 400);
}

/// **A popover opens upward, anchored to its control.**
///
/// Both of the design's popovers end 18 above the control row and sit one right
/// of their control's left edge. Asserted as those two relationships rather
/// than as the design's `top` of 274, which holds only at 800 tall.
#[test]
fn a_popover_opens_upward_from_its_control() {
    let rows = as_drawn();
    let controls = rows.controls();

    // **The design's `left` holds and its `top` does not**, and that asymmetry
    // is the whole point. Horizontal positions are unaffected by the residue;
    // vertical ones are eight lower by the decision in `layout`'s own
    // docstring. An earlier version of this test asserted the design's top of
    // 274 and failed at 282 — having been written straight after the docstring
    // saying a hard-coded top would be wrong.
    for (named, control_left, tall, design_left, design_top) in [
        ("model", controls.model, 396, 101, 274),
        ("permissions", controls.permissions, 398, 441, 272),
    ] {
        let (left, top) = rows.a_popover_from(control_left, tall);

        assert_eq!(left, design_left, "{named}: the design's left still holds");
        assert_eq!(
            top,
            design_top + 8,
            "{named}: eight below the design's top, by the residue decision"
        );
        assert_eq!(
            top + tall,
            controls.from_the_top - m::A_POPOVER_STANDS_OFF,
            "{named}: it ends above the control row"
        );
        assert!(
            top + tall < controls.from_the_top,
            "{named}: it must not reach the composer"
        );
    }
}

/// **A taller popover grows upward, not downward over the composer.**
#[test]
fn a_taller_popover_grows_away_from_the_composer() {
    let rows = as_drawn();
    let controls = rows.controls();

    let (_, short) = rows.a_popover_from(controls.model, 300);
    let (_, tall) = rows.a_popover_from(controls.model, 500);

    assert!(tall < short, "the taller one starts higher up");
    assert_eq!(short - tall, 200, "by exactly the difference in height");
}

/// **The floors are derived, not stated.** If a control's width changes, the
/// least width moves with it — so the two cannot drift apart.
#[test]
fn the_floors_follow_from_the_measures() {
    let controls = m::A_CONTROL_IS * 3 + m::THE_MODEL_CONTROL_IS_WIDE + m::THE_DESTINATION_IS_WIDE;
    // Four borders: two for the window and two for the reply field inside it.
    let paddings = (m::INSIDE_THE_FIELD * 2) + (m::BESIDE_A_ROW * 2) + (m::BORDER * 4);

    assert_eq!(the_least_width(), controls + m::GAP * 5 + paddings);
    assert_eq!(
        the_least_height(),
        m::THE_ROWS_THAT_DO_NOT_STRETCH + m::THE_LEAST_CONVERSATION + m::BORDER * 2
    );
    assert_eq!(
        m::THE_ROWS_THAT_DO_NOT_STRETCH,
        306,
        "the four rows that do not stretch, as the design draws them"
    );
}
