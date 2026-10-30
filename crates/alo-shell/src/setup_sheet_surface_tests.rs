//! What a setup sheet's surface draws, asked of the pixels it produced.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on a raster that is not there is the failure being reported"
)]

use super::setup_sheet_surface;
use crate::setup_sheet_layout::{TheTextIsTall, setup_sheet_laid_out};
use crate::sign_in_surface::TheColours;
use smithay::utils::{Scale, Size};

/// *Add a PIN* at the reference size, in the designed light palette.
fn drawn() -> (
    crate::painted::Washed,
    crate::setup_sheet_layout::SetupSheetLayout,
) {
    let at = setup_sheet_laid_out(
        Size::from((1440, 960)),
        TheTextIsTall {
            title: 44,
            introduction: 50,
            consequence: 22,
            choices: 2,
            actions: 2,
        },
    )
    .expect("the reference sheet lays out");
    let washed = setup_sheet_surface(
        &at,
        (at.sheet.size.w, at.sheet.size.h),
        designed(),
        crate::setup_sheet_surface::TheMarks::default(),
        Scale::from(1.0),
    )
    .expect("a sheet with area draws");
    (washed, at)
}

fn designed() -> TheColours {
    TheColours::of(crate::Contrast::AsDesigned, alo_appearance::Scheme::Light)
}

/// The pixel at a point, as red, green, blue.
fn at(washed: &crate::painted::Washed, x: i32, y: i32) -> [u8; 3] {
    let w = washed.area.size.w;
    let i = ((y * w + x) * 4) as usize;
    [washed.pixels[i], washed.pixels[i + 1], washed.pixels[i + 2]]
}

/// **The raster is the whole sheet**, four bytes a pixel.
#[test]
fn the_raster_covers_the_sheet_exactly() {
    let (washed, layout) = drawn();
    assert_eq!(washed.area.size.w, layout.sheet.size.w);
    assert_eq!(
        washed.pixels.len(),
        (washed.area.size.w * washed.area.size.h * 4) as usize
    );
}

/// **A choice is a card on the sheet**, not the sheet itself.
#[test]
fn a_choice_is_drawn_as_a_card() {
    let (washed, layout) = drawn();
    let card = layout.choices.first().expect("there are choices");
    let middle = at(
        &washed,
        card.loc.x + card.size.w / 2,
        card.loc.y + card.size.h / 2,
    );
    assert_eq!(
        middle,
        designed().entry,
        "a choice's middle is not its surface"
    );
    assert_ne!(middle, designed().ground);
}

/// **The cards have corners**, which is why this file exists.
#[test]
fn a_choice_has_corners_rather_than_being_a_rectangle() {
    let (washed, layout) = drawn();
    let card = layout.choices.first().expect("there are choices");
    assert_ne!(
        at(&washed, card.loc.x + 1, card.loc.y + 1),
        designed().entry,
        "the card's corner is filled, so it was drawn as a rectangle"
    );
    // The control: the same distance in along the flat top edge *is* the card.
    assert_ne!(
        at(&washed, card.loc.x + card.size.w / 2, card.loc.y + 2),
        designed().ground,
        "the card's top edge is not drawn at all"
    );
}

/// **The way on is filled and the way back is not.**
///
/// The design draws *Save PIN* on a navy ground and *Use password only* as an
/// outline, which is how a person tells them apart without reading either.
#[test]
fn the_first_action_is_filled_and_the_second_is_not() {
    let (washed, layout) = drawn();
    let on = layout.actions.first().expect("two actions");
    let back = layout.actions.get(1).expect("two actions");
    assert_eq!(
        at(&washed, on.loc.x + on.size.w / 2, on.loc.y + on.size.h / 2),
        designed().ink,
        "the way on is not filled with the ink"
    );
    assert_eq!(
        at(
            &washed,
            back.loc.x + back.size.w / 2,
            back.loc.y + back.size.h / 2
        ),
        designed().entry,
        "the way back is filled when it should be an outline"
    );
}

/// **The wallpaper is behind it**, which is the artwork the sign-in screen
/// already draws — the capture shows both sheets carry the same five vectors.
#[test]
fn the_wallpaper_is_drawn_behind_the_sheet() {
    let (washed, layout) = drawn();
    // The right-hand folds, well away from the column.
    assert_ne!(
        at(&washed, layout.sheet.size.w - 60, 200),
        designed().ground,
        "the folded light is not drawn on the sheet"
    );
    // And the middle left, which the design leaves bare. **Not (60, 700)**:
    // the bottom-left fold reaches there, which the first version of this
    // assertion did not allow for and the test caught.
    assert_eq!(at(&washed, 60, 450), designed().ground);
}

/// **A sheet with no area draws nothing**, rather than a panic.
#[test]
fn a_sheet_with_no_area_draws_nothing() {
    let at_ = setup_sheet_laid_out(
        Size::from((1440, 960)),
        TheTextIsTall {
            title: 44,
            introduction: 50,
            consequence: 22,
            choices: 1,
            actions: 1,
        },
    )
    .expect("lays out");
    // **An output of no pixels**, which is what the area is measured from.
    assert!(
        setup_sheet_surface(
            &at_,
            (0, 0),
            designed(),
            crate::setup_sheet_surface::TheMarks::default(),
            Scale::from(1.0)
        )
        .is_none()
    );
    // The control: the same sheet with area does draw.
    assert!(
        setup_sheet_surface(
            &at_,
            (1440, 960),
            designed(),
            crate::setup_sheet_surface::TheMarks::default(),
            Scale::from(1.0)
        )
        .is_some()
    );
}

/// What a chosen row looks like, read from the raster rather than from the
/// call that asked for it.
mod a_chosen_row_is_two_pixels_of_ink {
    use super::{designed, setup_sheet_surface};
    use crate::setup_sheet_layout::{TheTextIsTall, setup_sheet_laid_out};
    use smithay::utils::{Scale, Size};

    /// The reference sheet, with `chosen` as given.
    fn sheet(
        chosen: Option<usize>,
    ) -> (
        crate::painted::Washed,
        crate::setup_sheet_layout::SetupSheetLayout,
    ) {
        let at = setup_sheet_laid_out(
            Size::from((1440, 960)),
            TheTextIsTall {
                title: 44,
                introduction: 50,
                consequence: 0,
                choices: 3,
                actions: 2,
            },
        )
        .expect("the reference sheet lays out");
        let washed = setup_sheet_surface(
            &at,
            (at.sheet.size.w, at.sheet.size.h),
            designed(),
            crate::setup_sheet_surface::TheMarks { chosen, ring: None },
            Scale::from(1.0),
        )
        .expect("a sheet with area draws");
        (washed, at)
    }

    /// The colour at a point, in the sheet's own coordinates.
    fn at(washed: &crate::painted::Washed, x: i32, y: i32) -> [u8; 3] {
        let w = washed.area.size.w;
        let i = ((y * w + x) * 4) as usize;
        [washed.pixels[i], washed.pixels[i + 1], washed.pixels[i + 2]]
    }

    /// How many rows of the sheet, down this column, are the ink.
    ///
    /// The border's own thickness, measured rather than asserted: a row's top
    /// edge is one band of ink and its bottom edge another, so a 1px border
    /// gives bands of 1 and a 2px border gives bands of 2.
    fn the_thickest_band_of_ink(
        washed: &crate::painted::Washed,
        x: i32,
        from: i32,
        to: i32,
    ) -> i32 {
        let ink = designed().ink;
        let (mut most, mut run) = (0, 0);
        for y in from..to {
            if at(washed, x, y) == ink {
                run += 1;
                most = most.max(run);
            } else {
                run = 0;
            }
        }
        most
    }

    #[test]
    fn an_unchosen_sheet_draws_no_ink_edge_on_any_row() {
        let (washed, layout) = sheet(None);
        let first = layout.choices[0];
        let x = first.loc.x + first.size.w / 2;
        // The ordinary edge is `border`, not `ink`, so a hairline row has no
        // band of ink at all. This is the control: the measurement below has
        // to be able to come back zero.
        assert_eq!(
            the_thickest_band_of_ink(&washed, x, first.loc.y - 4, first.loc.y + first.size.h + 4),
            0,
            "nothing is chosen, so no row should carry an ink edge"
        );
    }

    #[test]
    fn the_chosen_row_carries_two_pixels_where_the_others_carry_none() {
        let (washed, layout) = sheet(Some(1));
        let chosen = layout.choices[1];
        let x = chosen.loc.x + chosen.size.w / 2;
        assert_eq!(
            the_thickest_band_of_ink(
                &washed,
                x,
                chosen.loc.y - 4,
                chosen.loc.y + chosen.size.h + 4
            ),
            2,
            "the design settled on 2 logical pixels of navy for the chosen row"
        );

        // And only that row. A sheet that drew the border on every row would
        // pass the assertion above and be the wrong picture.
        for other in [0_usize, 2] {
            let row = layout.choices[other];
            let x = row.loc.x + row.size.w / 2;
            assert_eq!(
                the_thickest_band_of_ink(&washed, x, row.loc.y - 4, row.loc.y + row.size.h + 4),
                0,
                "row {other} was not chosen and must carry no ink edge"
            );
        }
    }

    #[test]
    fn the_chosen_row_grows_one_pixel_on_every_edge() {
        // The design file measures 85 for an ordinary row and 87 for the
        // chosen one. The layout does not move — the growth is the border
        // reaching outwards — so the ink is found one pixel ABOVE where the
        // row begins, which is the whole content of "87 rather than 85".
        let (washed, layout) = sheet(Some(0));
        let row = layout.choices[0];
        let x = row.loc.x + row.size.w / 2;
        assert_eq!(
            at(&washed, x, row.loc.y - 1),
            designed().ink,
            "the chosen row's border reaches one pixel above the row"
        );
        assert_eq!(
            at(&washed, x, row.loc.y + row.size.h),
            designed().ink,
            "and one pixel below it"
        );
        assert_eq!(
            row.size.h, 85,
            "the ordinary row is 85 logical pixels, which is what makes the \
             chosen one 87"
        );
    }
}
