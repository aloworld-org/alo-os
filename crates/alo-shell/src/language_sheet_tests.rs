//! What the language sheet draws, asked of the picture it produced.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a picture that is not there is the failure being reported"
)]

use super::{LanguageSheet, drawn};
use crate::the_languages_offered::TheLanguagesOffered;

/// The sheet at the reference size, in the designed light palette.
fn sheet(offered: &TheLanguagesOffered) -> crate::sign_in_raster::SignInPicture {
    let mut fonts = crate::window_control_label::WindowControlLabels::new()
        .expect("the shell's own fonts load");
    let words = crate::booting::what_this_machine_can_say().expect("this machine's words");
    drawn(
        &LanguageSheet { offered },
        &mut fonts.fonts,
        &words,
        (1440, 960),
        crate::sign_in_surface::TheColours::of(
            crate::Contrast::AsDesigned,
            alo_appearance::Scheme::Light,
        ),
        &crate::sign_in_raster::Measure::of(alo_appearance::TextScale::ordinary()),
    )
    .expect("the reference size holds a sheet")
}

/// How much ink the picture carries, as summed darkness against the ground.
///
/// A count of inked runs would answer *did anything draw*; this answers *how
/// much*, which is what tells a row with two lines from a row with one.
fn how_much_ink(picture: &crate::sign_in_raster::SignInPicture) -> u64 {
    picture
        .inked
        .iter()
        .map(|inked| inked.pixels.len() as u64)
        .sum()
}

#[test]
fn the_sheet_draws_at_the_reference_size() {
    let offered = TheLanguagesOffered::every_official();
    let picture = sheet(&offered);
    assert_eq!(picture.size, (1440, 960));
    assert!(
        picture.surface.is_some(),
        "the sheet draws its cards and its wallpaper"
    );
    // The control: ink at all. Every assertion below compares amounts, and
    // two equal zeros would read as "no difference" when the truth is
    // "nothing was drawn".
    assert!(
        how_much_ink(&picture) > 0,
        "the sheet drew no text at all, so nothing below measures anything"
    );
}

#[test]
fn a_sheet_too_small_refuses_rather_than_drawing_over_itself() {
    let offered = TheLanguagesOffered::every_official();
    let mut fonts = crate::window_control_label::WindowControlLabels::new()
        .expect("the shell's own fonts load");
    let words = crate::booting::what_this_machine_can_say().expect("this machine's words");
    let refused = drawn(
        &LanguageSheet { offered: &offered },
        &mut fonts.fonts,
        &words,
        (200, 120),
        crate::sign_in_surface::TheColours::of(
            crate::Contrast::AsDesigned,
            alo_appearance::Scheme::Light,
        ),
        &crate::sign_in_raster::Measure::of(alo_appearance::TextScale::ordinary()),
    );
    assert!(
        refused.is_err(),
        "a screen too small for a sheet is refused, not drawn over its own footer"
    );
}

#[test]
fn choosing_a_row_adds_ink_to_it_and_to_nothing_else() {
    // **The word `Selected` is drawn, not just the border.** The design's rule
    // is that a choice is never shown by colour alone, so the chosen row's
    // first line gains ` ·  Selected` -- which is more ink than the same sheet
    // with nothing chosen. A test that only checked the border would pass with
    // the word missing, which is the half a person who cannot see the border
    // depends on.
    let nothing_chosen = TheLanguagesOffered::every_official();
    let before = how_much_ink(&sheet(&nothing_chosen));

    let mut chosen = TheLanguagesOffered::every_official();
    chosen.choose();
    let after = how_much_ink(&sheet(&chosen));

    assert!(
        after > before,
        "choosing drew no more ink than not choosing, so the word `Selected` \
         is missing: {before} then {after}"
    );
}

#[test]
fn every_visible_row_is_drawn_and_moving_the_cursor_changes_which() {
    // Walking the list must change the picture: the rows on screen are
    // different languages, so the ink differs. A window that did not move
    // would draw the same three rows for all twenty-four.
    let mut offered = TheLanguagesOffered::every_official();
    for _ in 0..30 {
        offered.up();
    }
    let at_the_top = how_much_ink(&sheet(&offered));
    for _ in 0..100 {
        offered.down();
    }
    let at_the_bottom = how_much_ink(&sheet(&offered));
    assert_ne!(
        at_the_top, at_the_bottom,
        "the first three rows and the last three drew the same ink, so the \
         window is not moving"
    );
}

#[test]
fn the_sheet_draws_the_same_number_of_rows_the_window_offers() {
    let offered = TheLanguagesOffered::every_official();
    let rows = offered.window().len();
    assert_eq!(
        rows,
        crate::the_languages_offered::ROWS_AT_A_TIME,
        "the window offers the rows the design draws"
    );
    // And the layout gives a card to each, which is what stops a fourth row
    // being drawn into nothing or a third being dropped.
    let at = crate::setup_sheet_layout::setup_sheet_laid_out(
        smithay::utils::Size::from((1440, 960)),
        crate::setup_sheet_layout::TheTextIsTall {
            title: 88,
            introduction: 0,
            consequence: 0,
            choices: rows,
            actions: 2,
        },
    )
    .expect("the reference sheet lays out");
    assert_eq!(at.choices.len(), rows);
    assert_eq!(at.actions.len(), 2, "the way on and the way back");
}

/// **Where the cursor is, visible.** These are the tests whose absence was the
/// defect: the sheet drew the chosen row and nothing else, so pressing an
/// arrow key changed no pixel until the window happened to scroll.
mod the_ring_says_where_a_person_is {
    use super::sheet;
    use crate::the_languages_offered::{TheLanguagesOffered, WhereTheRingIs};

    /// The surface as drawn, to be compared whole.
    ///
    /// **A count of inked pixels is the wrong measure and this test proved
    /// it**: a ring on the first row and a ring on the second have the same
    /// area, so counting said they were identical while the pictures were
    /// not. Where a mark IS is the whole question here, so the bytes are
    /// compared rather than summarised.
    fn every_mark(picture: &crate::sign_in_raster::SignInPicture) -> Vec<u8> {
        picture
            .surface
            .as_ref()
            .map(|washed| washed.pixels.clone())
            .unwrap_or_default()
    }

    #[test]
    fn moving_the_cursor_one_row_changes_the_picture() {
        // **The defect, as a test.** At the top of the list the window cannot
        // scroll up, so moving from the first row to the second leaves the
        // same three rows on screen -- which is exactly where the old sheet
        // drew nothing at all. The ring has to be what differs.
        //
        // The first version of this test started mid-list, where Down DOES
        // scroll; its own guard below caught that and said so.
        let mut before = TheLanguagesOffered::every_official();
        for _ in 0..40 {
            before.up();
        }
        let mut after = before.clone();
        after.down();
        assert_ne!(
            before.cursor(),
            after.cursor(),
            "the cursor did not move, so this test measures nothing"
        );
        assert_eq!(
            before.window().iter().map(|it| it.tag).collect::<Vec<_>>(),
            after.window().iter().map(|it| it.tag).collect::<Vec<_>>(),
            "the window scrolled, so this is not the case the defect was in"
        );
        assert_ne!(
            every_mark(&sheet(&before)),
            every_mark(&sheet(&after)),
            "the same three rows were drawn identically with the cursor on \
             different ones, so nothing marks where a person is"
        );
    }

    #[test]
    fn the_ring_is_drawn_before_anything_is_chosen() {
        // A sheet nobody has chosen on still has a ring: that is the whole
        // point of focus and selection being two marks.
        let offered = TheLanguagesOffered::every_official();
        assert!(offered.chosen().is_none());
        assert_eq!(offered.ring(), WhereTheRingIs::OnTheList);
        assert!(
            offered.cursor_in_the_window().is_some(),
            "there is a row for the ring to sit on"
        );
        assert!(!every_mark(&sheet(&offered)).is_empty());
    }

    #[test]
    fn choosing_moves_the_ring_to_the_way_on_and_changes_the_picture() {
        let before = TheLanguagesOffered::every_official();
        let mut after = TheLanguagesOffered::every_official();
        after.choose();
        after.ring_to_the_way_on();
        assert_eq!(after.ring(), WhereTheRingIs::OnTheWayOn);
        assert_ne!(
            every_mark(&sheet(&before)),
            every_mark(&sheet(&after)),
            "choosing left the picture unchanged, so neither the border nor \
             the ring moved"
        );
    }

    #[test]
    fn moving_after_choosing_brings_the_ring_back_to_the_list() {
        let mut offered = TheLanguagesOffered::every_official();
        offered.choose();
        offered.ring_to_the_way_on();
        offered.down();
        assert_eq!(
            offered.ring(),
            WhereTheRingIs::OnTheList,
            "an arrow key means a different row, so the ring belongs on the list"
        );
        // And the choice survives the move: moving is not unchoosing.
        assert!(offered.chosen().is_some());
    }
}
