//! **No share ever overlaps another or leaves a gap on the display** — held over
//! thousands of changes rather than over the few a person thinks to write down.
//!
//! A seeded walk drives one division through every road into it: windows
//! dragged to edges and corners and dropped, proposals abandoned, keyboard
//! splits, halves split again, boundaries dragged anywhere including past a
//! window's minimum and off the display, and windows closed. Some windows state
//! minimum sizes large enough that many of those changes are refused. After
//! every step, accepted or refused, the shares are held to five things:
//!
//! 1. together they cover exactly the display's area;
//! 2. no two of them share a single logical unit;
//! 3. every one is inside the display;
//! 4. no window is smaller than the minimum it stated;
//! 5. at 100 %, 125 %, 150 % and 200 %, the same is true in pixels.
//!
//! And a refused step is held to having changed nothing at all.
//!
//! The walk is deterministic — a fixed seed through a small generator written
//! here — so a failure names the step that caused it and happens again.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;

use alo_dividing::{
    Area, Division, Offer, Point, Refused, Scale, Share, Side, Size, Window, WindowId,
};

/// A small deterministic generator, so the walk needs no dependency and
/// replays exactly.
struct Walk(u64);

impl Walk {
    /// The next number.
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// A number below `bound`.
    fn below(&mut self, bound: u64) -> u32 {
        u32::try_from(self.next() % bound).unwrap()
    }
}

/// The windows of the walk and the minimum each states: most state none, some
/// are wide, one is tall, one is both.
fn windows() -> BTreeMap<u64, Window> {
    let minimums = [
        (1, Size::of(0, 0)),
        (2, Size::of(0, 0)),
        (3, Size::of(700, 0)),
        (4, Size::of(0, 0)),
        (5, Size::of(0, 500)),
        (6, Size::of(300, 200)),
        (7, Size::of(0, 0)),
        (8, Size::of(1000, 700)),
    ];
    minimums
        .into_iter()
        .map(|(id, minimum)| (id, Window::at_least(WindowId::from_compositor(id), minimum)))
        .collect()
}

/// Every promise a division makes about its shares.
fn holds_its_promises(division: &Division, windows: &BTreeMap<u64, Window>, step: usize) {
    let display = division.display();
    let shares = division.shares();
    if shares.is_empty() {
        assert!(division.is_empty(), "step {step}");
        return;
    }
    let covered: u64 = shares.iter().map(|share| share.area().covers()).sum();
    assert_eq!(covered, display.covers(), "step {step}: a gap");
    for (i, one) in shares.iter().enumerate() {
        assert!(
            display.encloses(one.area()),
            "step {step}: {one:?} is off the display"
        );
        let minimum = windows
            .get(&one.window().to_compositor())
            .unwrap()
            .minimum();
        assert!(
            one.area().width() >= minimum.width.max(1)
                && one.area().height() >= minimum.height.max(1),
            "step {step}: {one:?} is squeezed below {minimum:?}"
        );
        for other in shares.iter().skip(i + 1) {
            assert!(
                !one.area().overlaps(other.area()),
                "step {step}: {one:?} overlaps {other:?}"
            );
            assert_ne!(
                one.window(),
                other.window(),
                "step {step}: one window twice"
            );
        }
    }
    for n in [120, 150, 180, 240] {
        in_pixels_too(&shares, display, Scale::in_120ths(n).unwrap(), step);
    }
}

/// The shares still tile the display once each is turned into pixels.
fn in_pixels_too(shares: &[Share], display: Area, scale: Scale, step: usize) {
    let whole = scale.to_pixels(display);
    let pixels: Vec<_> = shares
        .iter()
        .map(|share| scale.to_pixels(share.area()))
        .collect();
    let covered: u64 = pixels.iter().map(|p| p.width * p.height).sum();
    assert_eq!(
        covered,
        whole.width * whole.height,
        "step {step}: a gap in pixels at {scale:?}"
    );
    for (i, one) in pixels.iter().enumerate() {
        for other in pixels.iter().skip(i + 1) {
            let apart = one.x + one.width <= other.x
                || other.x + other.width <= one.x
                || one.y + one.height <= other.y
                || other.y + other.height <= one.y;
            assert!(
                apart,
                "step {step}: {one:?} overlaps {other:?} at {scale:?}"
            );
        }
    }
}

/// One side, chosen.
fn a_side(walk: &mut Walk) -> Side {
    Side::ALL.get(walk.below(4) as usize).copied().unwrap()
}

/// **The walk.** Five thousand steps over a display that does not start at
/// the origin and is an odd size, so no half is exact.
#[test]
fn no_share_overlaps_another_or_leaves_a_gap_whatever_is_done_to_it() {
    let display = Area::of(Point::at(1920, 0), Size::of(2561, 1441)).unwrap();
    let windows = windows();
    let mut division = Division::of(display);
    let mut walk = Walk(0x005e_eda1_0d1d_1de5);
    let mut accepted = 0_usize;
    let mut refused = 0_usize;
    let mut most_shares = 0_usize;

    for step in 0..5_000 {
        let before = division.shares();
        let window = |walk: &mut Walk| *windows.get(&u64::from(walk.below(8) + 1)).unwrap();
        let outcome: Result<(), Refused> = match walk.below(6) {
            0 | 1 => {
                let pointer = Point::at(
                    display.x() + walk.below(u64::from(display.width())),
                    display.y() + walk.below(u64::from(display.height())),
                );
                // Pointers mostly near an edge or a corner, where drops do
                // something.
                let near_left = display.x() + walk.below(20);
                let near_right = display.right() - 1 - walk.below(20);
                let near_top = display.y() + walk.below(20);
                let near_bottom = display.bottom() - 1 - walk.below(20);
                let pointer = match walk.below(6) {
                    0 => pointer,
                    1 => Point::at(near_left, pointer.y),
                    2 => Point::at(near_right, pointer.y),
                    3 => Point::at(pointer.x, near_top),
                    4 => Point::at(pointer.x, near_bottom),
                    _ => Point::at(near_left, near_bottom),
                };
                let dragged = window(&mut walk);
                let in_front = Some(window(&mut walk));
                match division.propose_drop(pointer, dragged, in_front) {
                    Offer::Nothing => Ok(()),
                    Offer::Refused(why) => Err(why),
                    Offer::Proposed(proposal) => {
                        assert_eq!(
                            division.shares(),
                            before,
                            "step {step}: a proposal moved something"
                        );
                        if walk.below(4) == 0 {
                            drop(proposal);
                            Ok(())
                        } else {
                            division.commit(proposal)
                        }
                    }
                }
            }
            2 => {
                let focused = window(&mut walk);
                let next = Some(window(&mut walk));
                division.divide_with_next(focused, next, a_side(&mut walk))
            }
            3 => {
                let target = WindowId::from_compositor(u64::from(walk.below(8) + 1));
                division.divide(target, window(&mut walk), a_side(&mut walk))
            }
            4 => {
                let target = WindowId::from_compositor(u64::from(walk.below(8) + 1));
                let to = walk.below(u64::from(display.right()) + 200);
                division.move_boundary(target, a_side(&mut walk), to)
            }
            _ => division.close(WindowId::from_compositor(u64::from(walk.below(8) + 1))),
        };
        match outcome {
            Ok(()) => accepted += 1,
            Err(_) => {
                refused += 1;
                assert_eq!(
                    division.shares(),
                    before,
                    "step {step}: a refusal changed something"
                );
            }
        }
        holds_its_promises(&division, &windows, step);
        most_shares = most_shares.max(division.shares().len());
    }

    // The walk is only evidence if it went both ways many times.
    assert!(accepted > 1_000, "{accepted} accepted");
    assert!(refused > 500, "{refused} refused");
    assert!(
        most_shares >= 5,
        "the walk never divided the display into more than {most_shares}"
    );
}
