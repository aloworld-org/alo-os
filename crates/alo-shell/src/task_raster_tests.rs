//! That a task window's furniture rasterises, that its shapes are where the
//! layout puts them, and that a window too small is refused rather than drawn
//! wrong.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a window that will not rasterise is the failure being reported"
)]

use alo_access::TurnedOn;
use alo_appearance::{Accent, Appearance, TimeOfDay};
use alo_strings::Direction;
use smithay::utils::Rectangle;

use super::{NoTaskPicture, TaskPicture, furniture};
use crate::desktop_look::DesktopLook;

/// The ordinary look.
fn a_look() -> DesktopLook {
    let mut appearance = Appearance::shipped();
    appearance.set_accent(Accent::Indigo);
    DesktopLook::of(
        &appearance,
        &TurnedOn::nothing(),
        TimeOfDay::checked(10, 0).expect("ten in the morning"),
        Direction::LeftToRight,
        &crate::desktop_testing::a_display(),
    )
}

/// A window at the size the design was drawn at, with two files shared.
fn as_drawn() -> TaskPicture {
    furniture(
        Rectangle::new((120, 80).into(), (1040, 800).into()),
        a_look(),
        2,
    )
    .expect("the size the design was drawn at")
}

/// The four bytes at this point of the panel's raster.
fn pixel(of: &TaskPicture, x: i32, y: i32) -> [u8; 4] {
    let washed = &of.panel;
    let width = washed.area.size.w;
    let at = ((y as usize) * (width as usize) + (x as usize)) * 4;
    washed
        .pixels
        .get(at..at + 4)
        .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
        .expect("four bytes at a point inside the raster")
}

/// Where a point in the window's own coordinates lands in the raster, which is
/// offset by the room the shadow needs.
fn in_the_raster(of: &TaskPicture, x: i32, y: i32) -> (i32, i32) {
    (
        of.at.loc.x + x - of.panel.area.loc.x,
        of.at.loc.y + y - of.panel.area.loc.y,
    )
}

/// **A window at the design's size rasterises, and its raster is larger than
/// the window** — by the room the shadow needs on every side.
#[test]
fn a_window_rasterises_with_room_for_its_shadow() {
    let drawn = as_drawn();

    assert!(
        drawn.panel.area.size.w > drawn.at.size.w,
        "the raster is no wider than the window, so there is no room for a shadow"
    );
    assert!(drawn.panel.area.size.h > drawn.at.size.h);
    assert_eq!(
        drawn.panel.pixels.len(),
        (drawn.panel.area.size.w * drawn.panel.area.size.h * 4) as usize,
        "the raster's bytes do not match its area"
    );
}

/// **The middle of the window is its own surface**, opaque and the palette's.
#[test]
fn the_middle_of_the_window_is_its_surface() {
    let drawn = as_drawn();
    let (x, y) = in_the_raster(&drawn, drawn.at.size.w / 2, 300);
    let [r, g, b, a] = pixel(&drawn, x, y);

    assert_eq!(a, 255, "the middle of a window is opaque");
    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");
    assert_eq!(
        [r, g, b],
        palette.surface,
        "the middle is not the surface colour"
    );
}

/// **The reply field is drawn, and it is the cool colour rather than the
/// surface.**
///
/// The field is the one well at radius 18, so this is also the test that the
/// per-well corner reaches a real window rather than only the rasteriser's own
/// unit test.
#[test]
fn the_reply_field_is_drawn_in_the_cool_colour() {
    let drawn = as_drawn();
    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");
    let rows = alo_tasks::layout::TheRows::of(1040, 800).expect("the design's size");

    // The middle of the field, which is well inside its radius.
    let field_middle_y = rows.composer.from_the_top
        + alo_tasks::measures::ABOVE_THE_FIELD
        + alo_tasks::measures::THE_FIELD_IS_TALL / 4;
    let (x, y) = in_the_raster(
        &drawn,
        drawn.at.size.w / 2,
        i32::try_from(field_middle_y).expect("a small number"),
    );
    let [r, g, b, _] = pixel(&drawn, x, y);

    assert_eq!(
        [r, g, b],
        palette.cool,
        "the reply field is not drawn in the cool colour"
    );
    assert_ne!(
        [r, g, b],
        palette.surface,
        "the field and the window are the same colour, so the field is invisible"
    );
}

/// **The one button is drawn in the ink colour**, which is the inversion that
/// marks it as the thing a person presses to act.
#[test]
fn the_one_button_is_drawn_in_the_ink_colour() {
    let drawn = as_drawn();
    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");
    let rows = alo_tasks::layout::TheRows::of(1040, 800).expect("the design's size");
    let controls = rows.controls();

    let middle = alo_tasks::measures::A_CONTROL_IS / 2;
    let (x, y) = in_the_raster(
        &drawn,
        i32::try_from(controls.the_one_button + middle).expect("a small number"),
        i32::try_from(controls.from_the_top + middle).expect("a small number"),
    );
    let [r, g, b, _] = pixel(&drawn, x, y);

    assert_eq!([r, g, b], palette.the_one_button);
    assert_ne!(
        [r, g, b],
        palette.cool,
        "the one button is the same colour as the controls beside it"
    );
}

/// **A chip is drawn for each shared file, and none when nothing is shared.**
///
/// Asserted by sampling the same point with two files and with none, which
/// catches a chip drawn unconditionally — a row of empty grounds on a task
/// nobody attached anything to.
#[test]
fn a_chip_is_drawn_per_shared_file_and_none_without() {
    let rows = alo_tasks::layout::TheRows::of(1040, 800).expect("the design's size");
    let down = rows.shared_files.from_the_top + alo_tasks::measures::THE_SHARED_FILES_ARE_TALL / 2;
    let across = alo_tasks::measures::BESIDE_A_ROW + alo_tasks::measures::A_CHIP_IS_WIDE / 2;

    let sample = |how_many: usize| {
        let drawn = furniture(
            Rectangle::new((120, 80).into(), (1040, 800).into()),
            a_look(),
            how_many,
        )
        .expect("the design's size");
        let (x, y) = in_the_raster(
            &drawn,
            i32::try_from(across).expect("a small number"),
            i32::try_from(down).expect("a small number"),
        );
        let [r, g, b, _] = pixel(&drawn, x, y);
        [r, g, b]
    };

    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");
    assert_eq!(sample(2), palette.cool, "no chip was drawn for two files");
    assert_eq!(
        sample(0),
        palette.surface,
        "a chip was drawn on a task with nothing shared"
    );
}

/// **A window too small is refused, and the refusal says what it would need.**
#[test]
fn a_window_too_small_is_refused_rather_than_drawn_wrong() {
    let refused = furniture(
        Rectangle::new((0, 0).into(), (200, 100).into()),
        a_look(),
        0,
    )
    .expect_err("a 200 by 100 window was rasterised");

    assert!(
        matches!(refused, NoTaskPicture::TooSmall(_)),
        "{refused:?} is not a refusal about size"
    );
}

/// **More chips than fit are not drawn past the row's edge.**
///
/// Twenty files on a window that holds five: the row stops rather than running
/// chips off the side.
#[test]
fn chips_stop_at_the_rows_edge_rather_than_running_off_it() {
    let many = furniture(
        Rectangle::new((0, 0).into(), (1040, 800).into()),
        a_look(),
        20,
    )
    .expect("the design's size");

    // The last pixel column inside the window must still be the surface: no
    // chip reached it.
    let (x, y) = in_the_raster(
        &many,
        many.at.size.w - 2,
        i32::try_from(
            alo_tasks::layout::TheRows::of(1040, 800)
                .expect("the design's size")
                .shared_files
                .from_the_top
                + alo_tasks::measures::THE_SHARED_FILES_ARE_TALL / 2,
        )
        .expect("a small number"),
    );
    let [r, g, b, _] = pixel(&many, x, y);
    let palette = crate::task_look::TaskPalette::of(a_look()).expect("a palette");

    assert_eq!(
        [r, g, b],
        palette.surface,
        "a chip was drawn against the window's own edge"
    );
}
