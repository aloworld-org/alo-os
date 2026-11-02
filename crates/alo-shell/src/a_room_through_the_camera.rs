//! **A window of alo's laid out in a room of its own, seen through the
//! camera**: where a part of the room lands on the display, and where a point
//! on the display is in the room.
//!
//! Settings and the record lay themselves out for a display — a rule of its
//! size — and were drawn fixed to it. On the canvas (ADR 0145) each is laid out
//! in a **room**, the window and the design's least margin round it, and the
//! camera puts the whole room on the display, panned and scaled. These two
//! functions are that arithmetic and its inverse, one place for both, so a
//! window drawn and a window pressed cannot come apart.

use smithay::utils::{Physical, Point, Rectangle};

/// Where `area`, a part of a room `room` big, lands on the display when the
/// whole room lands at `room_on_screen`.
#[must_use]
pub(crate) fn on_the_screen(
    area: Rectangle<i32, Physical>,
    room: (i32, i32),
    room_on_screen: Rectangle<i32, Physical>,
) -> Rectangle<i32, Physical> {
    let (across, down) = (room.0.max(1), room.1.max(1));
    let out = |from: i32, start: i32, seen: i32, all: i32| -> i32 {
        i32::try_from(i64::from(start) + i64::from(from) * i64::from(seen) / i64::from(all))
            .unwrap_or(start)
    };
    let seen = room_on_screen;
    let left = out(area.loc.x, seen.loc.x, seen.size.w, across);
    let top = out(area.loc.y, seen.loc.y, seen.size.h, down);
    let right = out(area.loc.x + area.size.w, seen.loc.x, seen.size.w, across);
    let bottom = out(area.loc.y + area.size.h, seen.loc.y, seen.size.h, down);
    Rectangle::new((left, top).into(), (right - left, bottom - top).into())
}

/// Where a point `at` on the display is in a room `room` big that landed at
/// `room_on_screen`; [`None`] for a room drawn with no area.
#[must_use]
pub(crate) fn in_the_room(
    at: Point<i32, Physical>,
    room: (i32, i32),
    room_on_screen: Rectangle<i32, Physical>,
) -> Option<Point<i32, Physical>> {
    let seen = room_on_screen;
    if seen.size.w <= 0 || seen.size.h <= 0 {
        return None;
    }
    let back = |from: i32, start: i32, seen: i32, all: i32| -> Option<i32> {
        i32::try_from(i64::from(from - start) * i64::from(all) / i64::from(seen)).ok()
    };
    Some(Point::from((
        back(at.x, seen.loc.x, seen.size.w, room.0)?,
        back(at.y, seen.loc.y, seen.size.h, room.1)?,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **At half size and moved, a part lands halved and moved, and a point
    /// in it comes back to where it was** — the two directions agree.
    #[test]
    fn there_and_back() {
        let room = (1000, 800);
        let seen = Rectangle::new((100, 50).into(), (500, 400).into());
        let part = Rectangle::new((200, 100).into(), (400, 200).into());
        assert_eq!(
            on_the_screen(part, room, seen),
            Rectangle::new((200, 100).into(), (200, 100).into())
        );
        assert_eq!(
            in_the_room(Point::from((250, 150)), room, seen),
            Some(Point::from((300, 200)))
        );
        assert_eq!(
            in_the_room(Point::from((1, 1)), room, Rectangle::default()),
            None
        );
    }
}
