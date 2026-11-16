//! The approval card's layout against the approved frame's own numbers
//! (`549:193`, measured 2026-11-01).

use super::TheCard;

/// **The frame's card, with its one line of action, adds up**: 16 above, the
/// 16 line, 6, the action's 30, 16 below it, then the row of choices — a
/// hairline, 14, the 48 answers and 14.
#[test]
fn the_frames_card_adds_up_to_its_own_height() {
    let card = TheCard {
        wide: 920,
        the_action_is_tall: 30,
        rows: 0,
        what_it_does_is_tall: 0,
        the_texts_are_tall: 0,
    };
    assert_eq!(card.who_is_asking(), 16);
    assert_eq!(card.the_action(), 38);
    assert_eq!(card.the_choices(), 84);
    assert_eq!(card.the_answers(), 99);
    assert_eq!(card.tall(), 161);
}

/// **Allow once and Deny are the same width, side by side, and the line beside
/// them ends 20 in from the right.**
#[test]
fn the_answers_are_equal_and_the_line_beside_them_keeps_to_the_card() {
    let card = TheCard {
        wide: 920,
        the_action_is_tall: 30,
        rows: 0,
        what_it_does_is_tall: 0,
        the_texts_are_tall: 0,
    };
    assert_eq!(
        card.deny() - super::BESIDE,
        super::AN_ANSWER_IS_WIDE + super::BETWEEN
    );
    let (starts, room) = card.the_line_beside();
    assert_eq!(starts, 20 + 144 + 8 + 144 + 8);
    assert_eq!(starts + room + super::BESIDE, 920);
    assert_eq!(TheCard::the_actions_room(920), 880);
}

/// **A longer action makes a taller card**, by exactly as much, and moves the
/// answers down with it.
#[test]
fn a_longer_action_moves_the_answers_down() {
    let one = TheCard {
        wide: 920,
        the_action_is_tall: 30,
        rows: 0,
        what_it_does_is_tall: 0,
        the_texts_are_tall: 0,
    };
    let two = TheCard {
        the_action_is_tall: 60,
        ..one
    };
    assert_eq!(two.tall() - one.tall(), 30);
    assert_eq!(two.the_answers() - one.the_answers(), 30);
}

/// **The frame's own card, with its one file row, adds up to the frame**:
/// 16, 16, 6, the action's 30, 12, the 64 row, 16, then the choices' 77 —
/// 237 before the lines saying what will and will not change.
#[test]
fn a_row_naming_a_file_is_where_the_frame_draws_it() {
    let card = TheCard {
        wide: 920,
        the_action_is_tall: 30,
        rows: 1,
        what_it_does_is_tall: 0,
        the_texts_are_tall: 0,
    };
    assert_eq!(card.a_row(0), 38 + 30 + 12);
    assert_eq!(card.the_choices(), 38 + 30 + 12 + 64 + 16);
    assert_eq!(card.tall(), 237);
    let (starts, room) = card.a_rows_words();
    assert_eq!(starts, 20 + 12 + 40 + 12);
    assert_eq!(starts + room + 12 + 20, 920);
    let two = TheCard { rows: 2, ..card };
    assert_eq!(two.a_row(1) - two.a_row(0), 64 + 12);
    assert_eq!(two.tall() - card.tall(), 64 + 12);
}

/// **The frame's whole card adds up**: with its one-line action, its one
/// file row and its one line of what it does — 22, both sentences side by
/// side — the card is 16 + 16 + 6 + 30 + 12 + 64 + 12 + 22 + 16 + 77 = 271,
/// which is the frame's own card.
#[test]
fn the_frames_whole_card_adds_up() {
    let card = TheCard {
        wide: 920,
        the_action_is_tall: 30,
        rows: 1,
        what_it_does_is_tall: 22,
        the_texts_are_tall: 0,
    };
    assert_eq!(card.what_it_does(), 38 + 30 + 12 + 64 + 12);
    assert_eq!(card.tall(), 271);
    let nothing_said = TheCard {
        what_it_does_is_tall: 0,
        ..card
    };
    assert_eq!(
        card.tall() - nothing_said.tall(),
        12 + 22,
        "a verb that says nothing still kept room for it"
    );
}

/// **A text adds what is said about it and its box, and nothing else**: 12
/// above, a line saying how long it is (two when something is marked), 8,
/// then 12 round its words — and the box sits under the rows naming what the
/// change touches, with what it does and the answers moved down by exactly
/// as much.
#[test]
fn a_text_sits_under_the_rows_and_moves_the_rest_down_by_its_height() {
    let card = TheCard {
        wide: 920,
        the_action_is_tall: 30,
        rows: 1,
        what_it_does_is_tall: 22,
        the_texts_are_tall: 0,
    };
    // Three lines of System/Mono 12/18, how long it is said on one line.
    let three = super::a_text_is_tall(1, 3 * 18);
    assert_eq!(three, 12 + 18 + 8 + 12 + 54 + 12);
    // A hidden character adds the line saying so.
    assert_eq!(super::a_text_is_tall(2, 54) - three, 18);
    let with = TheCard {
        the_texts_are_tall: three,
        ..card
    };
    assert_eq!(with.the_texts(), 38 + 30 + 12 + 64, "under the one row");
    assert_eq!(with.what_it_does() - card.what_it_does(), three);
    assert_eq!(with.the_answers() - card.the_answers(), three);
    assert_eq!(with.tall() - card.tall(), three);
    // Twelve lines is the most a box holds before it scrolls.
    assert_eq!(super::THE_MOST_LINES_SHOWN, 12);
    assert_eq!(super::a_texts_box_is_tall(12 * 18), 12 + 216 + 12);
}
