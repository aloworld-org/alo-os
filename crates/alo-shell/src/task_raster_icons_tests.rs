//! That the reply box's controls are drawn with their icons, and that the one
//! button shows the Stop square while alo is working and the Send arrow
//! otherwise — at 100 per cent and at 200.
//!
//! Drawn with the shell's own fonts, never the machine's, for
//! `crate::dock_raster_scale_tests`' reason.

use alo_tasks::TheOneButton;
use smithay::utils::{Physical, Rectangle};

use super::tests::{a_look, the_words, two_files, what_was_said};
use super::{TaskPicture, furniture};
use crate::display_scale::DisplayScale;
use crate::task_look::TaskPalette;

/// The shell's own faces and nothing else.
fn fonts() -> cosmic_text::FontSystem {
    crate::WindowControlLabels::new()
        .map(|labels| labels.fonts)
        .unwrap_or_else(|_| unreachable!("the shell's own fonts load"))
}

/// The design's window at `scale`, its one button showing `button`.
fn drawn(scale: DisplayScale, button: TheOneButton) -> TaskPicture {
    let files = two_files();
    let said = what_was_said();
    let per_cent = i32::from(scale.per_cent());
    let at = Rectangle::new(
        (120 * per_cent / 100, 80 * per_cent / 100).into(),
        (1040 * per_cent / 100, 800 * per_cent / 100).into(),
    );
    furniture(
        at,
        a_look(),
        2,
        &the_words(&files),
        &said,
        alo_tasks::TheScroll::at_the_newest(),
        button,
        false,
        &mut fonts(),
        scale,
    )
    .unwrap_or_else(|_| unreachable!("the design's size draws"))
}

/// The colour of the raster at a point on the screen.
fn colour_at(window: &TaskPicture, x: i32, y: i32) -> Option<[u8; 3]> {
    let area = window.panel.area;
    let (x, y) = (x - area.loc.x, y - area.loc.y);
    let index = usize::try_from((y * area.size.w + x) * 4).ok()?;
    let pixel = window.panel.pixels.get(index..index + 3)?;
    <[u8; 3]>::try_from(pixel).ok()
}

/// The one button's circle on the screen.
fn the_button(window: &TaskPicture, scale: DisplayScale) -> Rectangle<i32, Physical> {
    crate::a_press_on_the_one_button::the_one_buttons_place(window.at, scale)
        .unwrap_or_else(|| unreachable!("the design's size has a button"))
}

/// A point inside the button, `x` and `y` logical pixels from its corner.
fn in_the_button(window: &TaskPicture, scale: DisplayScale, x: u32, y: u32) -> Option<[u8; 3]> {
    let button = the_button(window, scale);
    colour_at(
        window,
        button.loc.x + scale.px(x),
        button.loc.y + scale.px(y),
    )
}

/// The palette the window is drawn in.
fn palette() -> TaskPalette {
    TaskPalette::of(a_look()).unwrap_or_else(|_| unreachable!("the shipped accent"))
}

/// **Stop is a solid square and Send an arrow, in the same circle.** A point
/// down and right of the middle — inside the square, off the arrow — tells
/// them apart, and the arrow's shaft in the middle shows Send is drawn at all.
#[test]
fn the_one_button_shows_stop_while_working_and_send_otherwise() {
    let palette = palette();
    for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
        let stop = drawn(scale, TheOneButton::Stop);
        let send = drawn(scale, TheOneButton::Send);
        // The icon is 28 centred in 48, so its corner is 10 in; the square
        // runs from 7 to 21 of it, and the arrow's shaft is 13 to 15 across
        // with its head above 13 — so 17, 17 is square and not arrow.
        let off_the_arrow = (10 + 17, 10 + 17);
        assert_eq!(
            in_the_button(&stop, scale, off_the_arrow.0, off_the_arrow.1),
            Some(palette.on_the_one_button),
            "at {} the Stop square is not drawn",
            scale.per_cent()
        );
        assert_eq!(
            in_the_button(&send, scale, off_the_arrow.0, off_the_arrow.1),
            Some(palette.the_one_button),
            "at {} the Send arrow fills the square",
            scale.per_cent()
        );
        // The arrow's shaft is at the middle, two units wide.
        assert_eq!(
            in_the_button(&send, scale, 24, 24),
            Some(palette.on_the_one_button),
            "at {} the Send arrow is not drawn",
            scale.per_cent()
        );
    }
}

/// **A greyed Send and a Stop asked for are drawn as Send and Stop**, because
/// a disabled button's look is not in the design.
#[test]
fn the_greyed_states_draw_their_own_icon() {
    let scale = DisplayScale::ONE;
    assert_eq!(
        drawn(scale, TheOneButton::Stopping).panel,
        drawn(scale, TheOneButton::Stop).panel
    );
    assert_eq!(
        drawn(scale, TheOneButton::NothingToSend).panel,
        drawn(scale, TheOneButton::Send).panel
    );
}

/// How much ink is on a stretch of the control row, `x` to `x + wide`
/// logical pixels across: the sum, over every pixel, of how far its colour is
/// from the cool ground.
///
/// **A sum of ink, not a count of pixels.** A 2-pixel stroke at 100 per cent
/// has smoothed edges a pixel wide on each side, so counting touched pixels
/// overstates a thin line at the small size; summing the ink does not. Kept
/// four logical pixels clear of the row's top and bottom, away from the
/// controls' rounded corners.
fn ink_in(window: &TaskPicture, scale: DisplayScale, x: u32, wide: u32) -> u64 {
    let rows = alo_tasks::layout::TheRows::of(
        scale.logical(window.at.size.w),
        scale.logical(window.at.size.h),
    )
    .unwrap_or_else(|_| unreachable!("the design's size lays out"));
    let controls = rows.controls();
    let left = window.at.loc.x + scale.px(alo_tasks::measures::BORDER + x);
    let top = window.at.loc.y + scale.px(alo_tasks::measures::BORDER + controls.from_the_top);
    let (wide, tall) = (scale.px(wide), scale.px(alo_tasks::measures::A_CONTROL_IS));
    let cool = palette().cool;
    let margin = scale.px(4);
    (left..left + wide)
        .flat_map(|x| (top + margin..top + tall - margin).map(move |y| (x, y)))
        .filter_map(|(x, y)| colour_at(window, x, y))
        .map(|colour| {
            colour
                .iter()
                .zip(cool)
                .map(|(one, other)| u64::from(one.abs_diff(other)))
                .sum::<u64>()
        })
        .sum()
}

/// **Attach and Permissions are no longer empty boxes**, and their icons carry
/// four times the ink at 200 per cent: the same artwork, sharper.
#[test]
fn attach_and_permissions_carry_their_icons_at_every_scale() {
    let controls = alo_tasks::layout::TheRows::of(1040, 800)
        .unwrap_or_else(|_| unreachable!("the design's size lays out"))
        .controls();
    let one = drawn(DisplayScale::ONE, TheOneButton::Stop);
    let two = drawn(DisplayScale::of_per_cent(200), TheOneButton::Stop);
    for (x, named) in [
        (controls.attach, "Attach"),
        (controls.permissions, "Permissions"),
    ] {
        let at_one = ink_in(
            &one,
            DisplayScale::ONE,
            x,
            alo_tasks::measures::A_CONTROL_IS,
        );
        let at_two = ink_in(
            &two,
            DisplayScale::of_per_cent(200),
            x,
            alo_tasks::measures::A_CONTROL_IS,
        );
        // The ink of at least thirty fully inked pixels.
        let navy_on_cool = ink_in_one_pixel();
        assert!(
            at_one > 30 * navy_on_cool,
            "{named} is an empty box: {at_one} of ink"
        );
        #[expect(
            clippy::cast_precision_loss,
            reason = "sums of a few million are exact in f64"
        )]
        let ratio = at_two as f64 / at_one as f64;
        assert!(
            (3.6..=4.4).contains(&ratio),
            "{named} at 200 carries {ratio} times the ink it carries at 100"
        );
    }
}

/// The ink of one pixel fully inked in `text/primary` on the cool ground.
fn ink_in_one_pixel() -> u64 {
    let palette = palette();
    palette
        .ink
        .iter()
        .zip(palette.cool)
        .map(|(one, other)| u64::from(one.abs_diff(other)))
        .sum()
}

/// **The model's control carries its arrow**, centred while it has no name.
#[test]
fn the_models_control_carries_its_arrow() {
    let files = two_files();
    let mut words = the_words(&files);
    words.model = "";
    let window = furniture(
        Rectangle::new((120, 80).into(), (1040, 800).into()),
        a_look(),
        2,
        &words,
        &what_was_said(),
        alo_tasks::TheScroll::at_the_newest(),
        TheOneButton::Stop,
        false,
        &mut fonts(),
        DisplayScale::ONE,
    )
    .unwrap_or_else(|_| unreachable!("the design's size draws"));
    let model = alo_tasks::layout::TheRows::of(1040, 800)
        .unwrap_or_else(|_| unreachable!("the design's size lays out"))
        .controls()
        .model;
    let wide = alo_tasks::measures::THE_MODEL_CONTROL_IS_WIDE;
    let left_half = ink_in(&window, DisplayScale::ONE, model, wide / 2);
    let right_half = ink_in(&window, DisplayScale::ONE, model + wide / 2, wide / 2);
    assert!(
        left_half + right_half > 10 * ink_in_one_pixel(),
        "the model's control has no arrow"
    );
    // Centred: the arrow is symmetrical, so as much ink on each side.
    #[expect(
        clippy::cast_precision_loss,
        reason = "sums of a few thousand are exact in f64"
    )]
    let share = left_half as f64 / (left_half + right_half) as f64;
    assert!(
        (0.45..=0.55).contains(&share),
        "the arrow is not centred: {share}"
    );
}

/// The design's window at `scale`, with or without the cloud.
fn with_a_cloud(scale: DisplayScale, a_cloud: bool) -> TaskPicture {
    let files = two_files();
    let per_cent = i32::from(scale.per_cent());
    furniture(
        Rectangle::new(
            (120 * per_cent / 100, 80 * per_cent / 100).into(),
            (1040 * per_cent / 100, 800 * per_cent / 100).into(),
        ),
        a_look(),
        2,
        &the_words(&files),
        &what_was_said(),
        alo_tasks::TheScroll::at_the_newest(),
        TheOneButton::Stop,
        a_cloud,
        &mut fonts(),
        scale,
    )
    .unwrap_or_else(|_| unreachable!("the design's size draws"))
}

/// **The cloud is drawn before the processing destination while a provider
/// answers, and only then** — the approved window of 2026-10-31 (`549:113`).
/// The two windows differ in the panel's raster only inside the cloud's 16
/// frame at the destination's start, at 100 per cent and at 200.
#[test]
fn the_cloud_is_drawn_beside_a_provider_and_nowhere_else() {
    for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
        let (cloud, none) = (with_a_cloud(scale, true), with_a_cloud(scale, false));
        assert_eq!(cloud.panel.area, none.panel.area);
        let controls = alo_tasks::layout::TheRows::of(1040, 800)
            .unwrap_or_else(|_| unreachable!("the design's size lays out"))
            .controls();
        let frame_is = scale.px(alo_tasks::measures::THE_DESTINATIONS_ICON);
        let left = cloud.at.loc.x + scale.px(alo_tasks::measures::BORDER + controls.destination);
        let top = cloud.at.loc.y
            + scale.px(alo_tasks::measures::BORDER + controls.from_the_top)
            + (scale.px(alo_tasks::measures::A_CONTROL_IS) - frame_is) / 2;
        let the_frame =
            Rectangle::<i32, Physical>::new((left, top).into(), (frame_is, frame_is).into());
        let wide = cloud.panel.area.size.w;
        let mut differing = 0;
        for (at, (one, other)) in cloud
            .panel
            .pixels
            .chunks(4)
            .zip(none.panel.pixels.chunks(4))
            .enumerate()
        {
            if one == other {
                continue;
            }
            differing += 1;
            let at = i32::try_from(at).unwrap_or(i32::MAX);
            let point = smithay::utils::Point::<i32, Physical>::from((
                cloud.panel.area.loc.x + at % wide,
                cloud.panel.area.loc.y + at / wide,
            ));
            assert!(
                the_frame.contains(point),
                "at {}: the cloud drew at {point:?}, outside its frame {the_frame:?}",
                scale.per_cent()
            );
        }
        assert!(
            differing > 10,
            "at {}: no cloud was drawn",
            scale.per_cent()
        );
    }
}
