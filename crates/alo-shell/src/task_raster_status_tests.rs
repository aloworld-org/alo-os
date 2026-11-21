//! **Every status a task window can show is a pill**: a rounded ground of its
//! own, 32 tall and centred in the heading, its right end at the row's padding,
//! holding the status's icon where the design gives one and then its words.
//!
//! The owner's decision of 2026-11-21, after a design audit found the status
//! drawn as plain words: the approved frames draw it as a pill (`549:135`,
//! *Waiting for your permission*, with a clock; `549:45`, *alo is working*,
//! with alo's mark on `accent/soft`; `549:280`, *Complete*, with a circled tick
//! in `status/positive`). The three statuses no frame draws — *Stopping…*,
//! *Stopped* and *Stop requested · not confirmed* — are held to the same pill
//! with no icon, which is what this file says rather than leaves to be found.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on a window that will not rasterise is the failure being reported, \
              and a formatted panic names the case it was on"
)]

use alo_appearance::Role;
use alo_tasks::TheStatus;
use alo_tasks::measures as m;
use smithay::utils::Rectangle;

use super::furniture;
use super::tests::a_look;
use crate::display_scale::DisplayScale;
use crate::painted::Inked;
use crate::task_text::TheWordsInATaskWindow;

/// A role as three bytes.
fn rgb(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// What the design draws for each status: the pill's ground, and its icon's
/// size and colour where it has one. **The design's table, written here from
/// the frames and not from the code.**
fn the_design_of(status: TheStatus) -> ([u8; 3], Option<(u32, [u8; 3])>) {
    let cool = rgb(Role::BgCool);
    match status {
        TheStatus::AloIsWorking => (
            rgb(Role::AccentSoft),
            Some((m::ALOS_STATUS_MARK_IS, rgb(Role::AccentDefault))),
        ),
        TheStatus::WaitingForYou => (cool, Some((m::A_STATUS_ICON_IS, rgb(Role::TextPrimary)))),
        TheStatus::Complete => (cool, Some((m::A_STATUS_ICON_IS, rgb(Role::StatusPositive)))),
        TheStatus::Stopping | TheStatus::Stopped | TheStatus::StopNotConfirmed => (cool, None),
    }
}

/// The window, drawn with nothing to say but `status`, so its one piece of ink
/// is the status's words.
fn drawn(
    status: TheStatus,
    scale: DisplayScale,
) -> (super::TaskPicture, Rectangle<i32, smithay::utils::Physical>) {
    let strings = alo_strings::Strings::of(
        alo_saying::everything_this_machine_can_say().expect("the vocabulary"),
    );
    let said = strings
        .say(&status.word().key(), &alo_strings::Filling::nothing())
        .into_text();
    let words = TheWordsInATaskWindow {
        title: "",
        kind: "",
        status: &said,
        status_is: status,
        model: "",
        destination: "",
        invitation: "",
        scope_note: "",
        files: &[],
        how_many_files: "",
        you: "",
        alo: "",
    };
    let per_cent = i32::from(scale.per_cent());
    let at = Rectangle::new(
        (120 * per_cent / 100, 80 * per_cent / 100).into(),
        (1040 * per_cent / 100, 800 * per_cent / 100).into(),
    );
    let picture = furniture(
        at,
        a_look(),
        0,
        &words,
        &[],
        alo_tasks::TheScroll::at_the_newest(),
        alo_tasks::TheOneButton::Send,
        false,
        &mut crate::WindowControlLabels::new()
            .expect("the shell's own fonts")
            .fonts,
        scale,
    )
    .expect("the design's size");
    (picture, at)
}

/// The panel's colour at a point of the display.
fn at(picture: &super::TaskPicture, x: i32, y: i32) -> [u8; 3] {
    let panel = &picture.panel;
    let (x, y) = (x - panel.area.loc.x, y - panel.area.loc.y);
    let from = usize::try_from(y * panel.area.size.w + x).expect("inside the raster") * 4;
    panel
        .pixels
        .get(from..from + 3)
        .and_then(|bytes| <[u8; 3]>::try_from(bytes).ok())
        .unwrap_or_else(|| panic!("no pixel at ({x}, {y})"))
}

/// How far apart two colours are.
fn apart(one: [u8; 3], other: [u8; 3]) -> i32 {
    one.iter()
        .zip(&other)
        .map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs())
        .sum()
}

/// How far `point` is from the line that runs from `ground` to `ink`: a pixel
/// of a stroke in `ink` on `ground` is a blend of the two, and lies on it.
fn off_the_line(point: [u8; 3], ground: [u8; 3], ink: [u8; 3]) -> f64 {
    let (p, g, c) = (
        point.map(f64::from),
        ground.map(f64::from),
        ink.map(f64::from),
    );
    let along: Vec<f64> = c.iter().zip(&g).map(|(c, g)| c - g).collect();
    let here: Vec<f64> = p.iter().zip(&g).map(|(p, g)| p - g).collect();
    let length: f64 = along.iter().map(|d| d * d).sum();
    let how_far =
        (here.iter().zip(&along).map(|(h, d)| h * d).sum::<f64>() / length).clamp(0.0, 1.0);
    here.iter()
        .zip(&along)
        .map(|(h, d)| (h - how_far * d).powi(2))
        .sum()
}

/// The status's words: the one piece of ink.
fn the_words_of(picture: &super::TaskPicture, status: TheStatus) -> &Inked {
    let [words] = picture.inked.as_slice() else {
        panic!(
            "{status:?}: a window given only its status drew {} pieces of ink",
            picture.inked.len()
        );
    };
    words
}

/// **The rule**: each status's pill, its ground, its ends, its height and its
/// icon, at one to one and at twice the pixels.
#[test]
fn every_status_is_a_pill_with_its_icon_and_its_words() {
    let surface = rgb(Role::BgSurface);
    for scale in [DisplayScale::ONE, DisplayScale::of_per_cent(200)] {
        let px = |logical: u32| scale.px(logical);
        for status in TheStatus::EVERY {
            let (picture, window) = drawn(status, scale);
            let words = the_words_of(&picture, status);
            let (ground, icon) = the_design_of(status);
            let case = format!("{status:?} at {scale:?}");

            // Where the pill must be, from the window and the design alone.
            let right = window.loc.x + window.size.w - px(m::BORDER) - px(m::BESIDE_A_ROW);
            let middle = window.loc.y + px(m::BORDER) + px(m::THE_HEADING_IS_TALL) / 2;
            let half = px(m::A_STATUS_IS_TALL) / 2;
            let inside = px(m::INSIDE_A_STATUS);
            let left = words.area.loc.x
                - inside
                - icon.map_or(0, |(size, _)| px(size) + px(m::BESIDE_A_STATUS_ICON));

            // Its ground, at each end inside it, and the window's surface just
            // past each end: a pill that stops where the design says.
            for (x, wanted, what) in [
                (
                    left + inside / 2,
                    ground,
                    "the pill's ground inside its left end",
                ),
                (
                    right - inside / 2,
                    ground,
                    "the pill's ground inside its right end",
                ),
                (left - 3, surface, "the window's surface past its left end"),
                (
                    right + 3,
                    surface,
                    "the window's surface past its right end",
                ),
            ] {
                assert_eq!(at(&picture, x, middle), wanted, "{case}: {what}");
            }
            // And its height, just before the words, where neither their box
            // nor a round end reaches: ground just inside the top and the
            // bottom, surface just outside.
            let across = words.area.loc.x - 2;
            for (y, wanted, what) in [
                (middle - half + 2, ground, "inside its top"),
                (middle + half - 3, ground, "inside its bottom"),
                (middle - half - 3, surface, "above it"),
                (middle + half + 2, surface, "below it"),
            ] {
                assert_eq!(
                    at(&picture, across, y),
                    wanted,
                    "{case}: the pill's ground {what}"
                );
            }

            // **The words are written on the pill's ground**, not on the
            // window's: their opaque box is filled with it.
            assert!(
                words.pixels.contains(&ground) && !words.pixels.contains(&surface),
                "{case}: the status's words are not written on the pill's ground"
            );

            // **Its icon, where the design gives one**: in the box before the
            // words, something is drawn, and it is drawn in the icon's own
            // colour rather than the words' or the ground's.
            if let Some((size, colour)) = icon {
                let size = px(size);
                let from_x = words.area.loc.x - px(m::BESIDE_A_STATUS_ICON) - size;
                let from_y = middle - size / 2;
                let inked = (from_y..from_y + size)
                    .flat_map(|y| (from_x..from_x + size).map(move |x| (x, y)))
                    .map(|(x, y)| at(&picture, x, y))
                    .max_by_key(|pixel| apart(*pixel, ground))
                    .unwrap_or(ground);
                assert!(
                    apart(inked, ground) > 60,
                    "{case}: nothing is drawn where the icon goes"
                );
                // Asked of the blend from the ground towards each colour, since
                // a thin stroke may cover no pixel wholly.
                let others = [
                    rgb(Role::TextPrimary),
                    rgb(Role::AccentDefault),
                    rgb(Role::StatusPositive),
                ];
                let nearest = others
                    .into_iter()
                    .min_by(|one, other| {
                        off_the_line(inked, ground, *one)
                            .total_cmp(&off_the_line(inked, ground, *other))
                    })
                    .unwrap_or(colour);
                assert_eq!(
                    nearest, colour,
                    "{case}: the icon is drawn nearer another colour than its own"
                );
            }
        }
    }
}
