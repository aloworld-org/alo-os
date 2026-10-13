//! That the conversation stacks what was said in order, marks a person's own
//! words by side and width, opens at the newest, and clips to the view.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on a size that will not lay out, or on a block \
              this file placed itself, is the failure being reported"
)]

use super::{TheBlocks, WhoseBlock};
use crate::layout::TheRows;
use crate::measures as m;

/// The conversation row at the size the design was drawn at.
fn as_drawn() -> u32 {
    TheRows::of(1040, 800)
        .expect("the design's size")
        .conversation
        .wide
}

/// The design's own four blocks, with the heights it draws them at.
fn the_designs_blocks() -> Vec<(WhoseBlock, u32)> {
    vec![
        (WhoseBlock::ThePersons, 92),
        (WhoseBlock::Alos, 70),
        (WhoseBlock::AnAccount, 120),
        (WhoseBlock::AnAccount, 152),
    ]
}

/// **The blocks are stacked in order with one gap between each.**
#[test]
fn the_blocks_stack_in_order_with_a_gap_between_them() {
    let blocks = TheBlocks::stacked(&the_designs_blocks(), as_drawn());
    let each = blocks.each();

    assert_eq!(each.len(), 4);
    assert_eq!(each[0].from_the_top, 0, "the first is flush, with no gap");
    for pair in each.windows(2) {
        assert_eq!(
            pair[1].from_the_top,
            pair[0].past_its_bottom() + m::BETWEEN_WHAT_IS_SAID,
            "one gap, and exactly one"
        );
    }
    assert_eq!(
        blocks.how_tall(),
        92 + 70 + 120 + 152 + m::BETWEEN_WHAT_IS_SAID * 3,
        "the content is the blocks and the three gaps between four of them"
    );
}

/// **One block starts flush and adds no gap**, so a conversation with a single
/// request does not begin one gap down from its padding.
#[test]
fn a_single_block_adds_no_gap() {
    let blocks = TheBlocks::stacked(&[(WhoseBlock::ThePersons, 92)], as_drawn());

    assert_eq!(blocks.each()[0].from_the_top, 0);
    assert_eq!(blocks.how_tall(), 92);
}

/// **Nothing said is nothing drawn and no height.**
#[test]
fn an_empty_conversation_has_no_blocks_and_no_height() {
    let blocks = TheBlocks::stacked(&[], as_drawn());

    assert!(blocks.each().is_empty());
    assert_eq!(blocks.how_tall(), 0);
    assert_eq!(blocks.out_of_view(0, 400), (false, false));
    assert_eq!(blocks.scrolled_to_the_newest(400), 0);
}

/// **A person's request is narrower and against the right edge; alo's reply is
/// the full column on the left.**
///
/// The two marks that tell them apart without colour, which is what somebody
/// who cannot distinguish the bubble from the surface behind it relies on.
#[test]
fn a_persons_words_are_marked_by_side_and_width_not_only_colour() {
    let blocks = TheBlocks::stacked(&the_designs_blocks(), as_drawn());
    let each = blocks.each();
    let inside = blocks.inside();

    assert_eq!(each[0].wide, m::A_REQUEST_IS_WIDE);
    assert_eq!(each[0].from_the_left, inside - m::A_REQUEST_IS_WIDE);
    assert_eq!(
        each[0].from_the_left + each[0].wide,
        inside,
        "a request ends against the far edge"
    );

    assert_eq!(each[1].wide, m::ALOS_SIDE_IS_WIDE);
    assert_eq!(each[1].from_the_left, 0, "alo's reply starts at the left");
    assert!(
        each[0].wide < each[1].wide,
        "a request must be narrower than a reply, or the width marks nothing"
    );
}

/// **The design's own figures**: the conversation's inside is 974 less its
/// padding, and a request ends against that edge.
#[test]
fn the_widths_are_the_designs_own() {
    let blocks = TheBlocks::stacked(&the_designs_blocks(), as_drawn());

    assert_eq!(blocks.inside(), 1038 - 64, "the row less 32 each side");
    assert_eq!(
        blocks.inside(),
        974,
        "which is the design's own Person message row"
    );
    assert_eq!(
        blocks.each()[0].from_the_left,
        974 - 660,
        "the request's left edge at the design's size"
    );
    assert_eq!(
        blocks.each()[0].from_the_left,
        314,
        "and 314 is where the design draws the bubble inside that row"
    );
}

/// **A block wider than the column is cut to the column**, so a narrow window
/// does not place a block past its own edge.
#[test]
fn a_block_never_reaches_past_the_column() {
    let narrow = TheRows::of(super::super::layout::the_least_width(), 800)
        .expect("the least width")
        .conversation
        .wide;
    let blocks = TheBlocks::stacked(&the_designs_blocks(), narrow);
    let inside = blocks.inside();

    for block in blocks.each() {
        assert!(
            block.from_the_left + block.wide <= inside,
            "{:?} reaches past the column: {} + {} > {inside}",
            block.whose,
            block.from_the_left,
            block.wide
        );
    }
}

/// **A conversation opens at the newest thing said**, not the oldest.
///
/// Opening at the top would show somebody the request they typed an hour ago
/// and hide what alo just answered.
#[test]
fn a_conversation_opens_at_the_newest() {
    let blocks = TheBlocks::stacked(&the_designs_blocks(), as_drawn());
    let view = 300;

    let scrolled = blocks.scrolled_to_the_newest(view);

    assert_eq!(scrolled, blocks.how_tall() - view);
    let newest = blocks.each().last().expect("a last block");
    assert!(
        newest.is_in_view(scrolled, view),
        "the newest block is not in view when the conversation opens"
    );
    assert_eq!(
        scrolled + view,
        blocks.how_tall(),
        "and the view ends exactly at the end of the content"
    );
}

/// **Everything fitting means no scroll at all**, so a short conversation sits
/// against the top.
#[test]
fn a_short_conversation_does_not_scroll() {
    let blocks = TheBlocks::stacked(&[(WhoseBlock::ThePersons, 92)], as_drawn());

    assert_eq!(blocks.scrolled_to_the_newest(400), 0);
    assert_eq!(blocks.out_of_view(0, 400), (false, false));
}

/// **What is out of view above and below is answered together**, which is what
/// a scroll rail is drawn from.
#[test]
fn what_is_out_of_view_is_answered_for_both_ends_at_once() {
    let blocks = TheBlocks::stacked(&the_designs_blocks(), as_drawn());
    let view = 200;

    assert_eq!(
        blocks.out_of_view(0, view),
        (false, true),
        "at the top: nothing above, something below"
    );
    assert_eq!(
        blocks.out_of_view(blocks.how_tall() - view, view),
        (true, false),
        "at the bottom: something above, nothing below"
    );
    let middle = (blocks.how_tall() - view) / 2;
    assert_eq!(
        blocks.out_of_view(middle, view),
        (true, true),
        "in the middle: both"
    );
}

/// **Only the blocks touching the view are drawn**, and a block exactly at the
/// edge counts as touching it.
#[test]
fn only_the_blocks_touching_the_view_are_drawn() {
    let blocks = TheBlocks::stacked(&the_designs_blocks(), as_drawn());
    let first = blocks.each()[0];

    // A view that ends exactly where the first block ends still shows it.
    assert!(first.is_in_view(0, first.tall));
    // A view that begins exactly where it ends does not.
    assert!(!first.is_in_view(first.past_its_bottom(), 100));

    let in_view: Vec<WhoseBlock> = blocks.in_view(0, 92).map(|block| block.whose).collect();
    assert_eq!(
        in_view,
        [WhoseBlock::ThePersons],
        "a view the height of the first block showed more than it"
    );

    let all: Vec<WhoseBlock> = blocks
        .in_view(0, blocks.how_tall())
        .map(|block| block.whose)
        .collect();
    assert_eq!(all.len(), 4, "a view of the whole content shows everything");
}

/// **Which kinds are in bubbles**, asserted so a later change cannot quietly
/// put alo's replies in one — the thing that would make a person's words and
/// alo's look alike.
#[test]
fn alos_reply_is_the_one_kind_with_no_bubble() {
    assert!(WhoseBlock::ThePersons.is_in_a_bubble());
    assert!(WhoseBlock::AnAccount.is_in_a_bubble());
    assert!(!WhoseBlock::Alos.is_in_a_bubble());
}
