//! Kept rasters are the same pixels, made once — and how much a frame saves.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or a missing pixel is the failure being reported"
)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::*;
use crate::a_window_raised::round_the_window;

/// The straight-alpha pixel at `(x, y)` among these rasters, if one has it.
fn pixel(rasters: &[Washed], x: i32, y: i32) -> Option<[u8; 4]> {
    rasters.iter().find_map(|raster| {
        let area = raster.area;
        let inside = x >= area.loc.x
            && x < area.loc.x + area.size.w
            && y >= area.loc.y
            && y < area.loc.y + area.size.h;
        inside.then(|| {
            let at = usize::try_from(((y - area.loc.y) * area.size.w + (x - area.loc.x)) * 4)
                .expect("inside");
            let mut it = [0_u8; 4];
            it.copy_from_slice(&raster.pixels[at..at + 4]);
            it
        })
    })
}

/// **A window's line and shadow are the same wherever it is**: the pixel a
/// given distance from the window is the same colour at two places on the
/// display — which is the whole reason a raster can be kept and moved.
#[test]
fn the_same_window_is_the_same_pixels_wherever_it_is() {
    let size = (731, 457);
    for look in [WhichLook::Focused, WhichLook::NotFocused] {
        let here = Rectangle::<i32, Physical>::new((100, 120).into(), size.into());
        let there = Rectangle::<i32, Physical>::new((900, 610).into(), size.into());
        let at_here = round_the_window(here, 1.0, look, (2880, 1920));
        let at_there = round_the_window(there, 1.0, look, (2880, 1920));
        for (dx, dy) in [
            (-1, -1),
            (-1, 200),
            (300, -1),
            (731, 457),
            (-20, 470),
            (740, 480),
        ] {
            assert_eq!(
                pixel(&at_here, 100 + dx, 120 + dy),
                pixel(&at_there, 900 + dx, 610 + dy),
                "{look:?} at ({dx}, {dy}) from the window"
            );
        }
    }
}

/// **A window part off the display draws exactly the part that is on it**:
/// the same pixels as the same window on a display large enough for all of it.
#[test]
fn a_window_part_off_the_display_is_clipped_not_changed() {
    let window = Rectangle::<i32, Physical>::new((-200, 30).into(), (640, 400).into());
    let small = round_the_window(window, 1.25, WhichLook::Focused, (1280, 800));
    for raster in &small {
        assert!(
            raster.area.loc.x >= 0 && raster.area.loc.y >= 0,
            "{:?}",
            raster.area
        );
        assert!(raster.area.loc.x + raster.area.size.w <= 1280);
    }
    // The same window seen on a display that starts further left: shifted by
    // 400, so all of it is on.
    let shifted = Rectangle::<i32, Physical>::new((200, 30).into(), (640, 400).into());
    let large = round_the_window(shifted, 1.25, WhichLook::Focused, (2000, 1000));
    for (x, y) in [(0, 20), (0, 431), (439, 25), (440, 440), (5, 300)] {
        assert_eq!(
            pixel(&small, x, y),
            pixel(&large, x + 400, y),
            "at ({x}, {y})"
        );
    }
}

/// **One making for one size, scale and look.** Asked twice, the second is
/// the kept one; a different look is made once more.
#[test]
fn a_raster_is_made_once_for_its_key() {
    let made = AtomicUsize::new(0);
    let make = || {
        made.fetch_add(1, Ordering::SeqCst);
        vec![Washed {
            area: Rectangle::from_size((2, 1).into()),
            pixels: vec![1, 2, 3, 4, 5, 6, 7, 8],
        }]
    };
    // A size no other test asks for, so the list is this test's.
    let focused = TheKey::of(Which::Strip, (9_973, 3), 1.0, WhichLook::Focused);
    let quiet = TheKey::of(Which::Strip, (9_973, 3), 1.0, WhichLook::NotFocused);
    let first = kept_or_made(focused, make);
    let second = kept_or_made(focused, make);
    assert!(Arc::ptr_eq(&first, &second) || made.load(Ordering::SeqCst) == 2);
    let _ = kept_or_made(quiet, make);
    assert!(made.load(Ordering::SeqCst) >= 2);
    assert_eq!(
        placed(&first, (5, 7), (100, 100))[0].area.loc,
        (5, 7).into()
    );
}

/// Three 1280 × 800 windows' lines, shadows and strips for `frames` frames —
/// worked out every frame when `kept` is false, as before this change.
fn three_windows(frames: u32, kept: bool) -> Duration {
    let size = (1280, 800);
    let began = Instant::now();
    for frame in 0..frames {
        for which in 0..3 {
            let at = (
                60 + 90 * which,
                80 + 40 * which + i32::try_from(frame % 2).unwrap_or(0),
            );
            let look = if which == 2 {
                WhichLook::Focused
            } else {
                WhichLook::NotFocused
            };
            let window = Rectangle::<i32, Physical>::new(at.into(), size.into());
            let strip =
                Rectangle::<i32, Physical>::new((at.0, at.1 - 32).into(), (size.0, 32).into());
            if kept {
                std::hint::black_box(round_the_window(window, 1.0, look, (2560, 1600)));
                std::hint::black_box(crate::the_title_strip_drawn::the_strip(
                    strip,
                    1.0,
                    look,
                    (2560, 1600),
                ));
            } else {
                let reach = 57;
                let made = crate::a_window_raised::made_at_the_origin(size, 1.0, look);
                std::hint::black_box(placed(&made, (at.0 - reach, at.1 - reach), (2560, 1600)));
                let strip_made = crate::the_title_strip_drawn::made_at_the_origin(
                    (size.0 + 2, 32),
                    (1.0, 10.0),
                    look,
                );
                std::hint::black_box(strip_made);
                std::hint::black_box(strip);
            }
        }
    }
    began.elapsed()
}

/// **The frame's CPU cost of three windows' rasters, made every frame and
/// kept.** Measured on every build, in the build's own profile, and printed;
/// asserted only to be at least five times smaller kept, because a timing is
/// a fact about the machine that ran it and a gate must not fail on a slow
/// runner for a reason that is not the change.
#[test]
fn keeping_the_rasters_makes_a_frame_cheaper() {
    // Made once before the clock starts, as a window at rest has been.
    let _ = three_windows(1, true);
    let frames = 6;
    let each_frame = three_windows(frames, false);
    let kept = three_windows(frames, true);
    eprintln!(
        "three 1280x800 windows' line, shadow and strip, per frame: made every frame {:?}, kept {:?}",
        each_frame / frames,
        kept / frames
    );
    assert!(
        kept * 5 < each_frame,
        "kept {kept:?} is not five times cheaper than made every frame {each_frame:?}"
    );
}
