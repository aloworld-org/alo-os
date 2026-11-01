//! The part of the screen *Show all* fits the windows into: what the fixed
//! controls leave clear, with a margin inside it.
//!
//! **Found on a booted machine on 2026-11-01**, and it is why this file exists.
//! `⊞`+0 with one window open made that window fill the whole screen and run
//! under the Dock, because the fit was given the output's full size. The
//! promise is *fit the windows on the current Place into view*
//! (`docs/design/the-canvas-as-a-workspace.md` §4), and a window behind the
//! Dock is not in view. **This makes an existing promise true rather than
//! adding one**: no setting, no surface and no shortcut is new.
//!
//! # Clear of every fixed control, not only the Dock
//!
//! The room starts as the whole output and gives way to each control the last
//! draw laid out — the Dock's band, the put-aside panel's column, the top
//! controls and the egress indicator, the same four
//! `Server::the_fixed_controls_were_drawn` keeps. Each control takes the side of
//! the room it sits on: the room keeps the largest of the four pieces left
//! beside it. So a Dock on the bottom edge leaves the room above it, a Dock on
//! the left edge leaves the room to its right, and nothing here knows which edge
//! the person chose.
//!
//! # The margin, and where its number comes from
//!
//! **The design gives no figure for it**, and that is stated rather than
//! hidden. The design file's *Show all* frames (`377:26182`, `377:26232`) are a
//! tour's sample arrangement, and `where-a-new-window-opens.md` already warns
//! that a drawn arrangement is not an algorithm. So the margin is the largest
//! step of the design's own spacing scale, `space/8` = **32 logical pixels**
//! (`docs/design/the-canvas-in-numbers.md`), converted through the display's
//! scale like every other logical measure. It is a value the owner may change;
//! it is not a value read off a frame.
//!
//! # Never a reason to refuse
//!
//! The clear room is a preference about how the windows are shown, not a new
//! limit on what *Show all* can reach. `docs/design/the-canvas-as-a-workspace.md`
//! §4 makes *Show all* a recovery tool whose reach must match where a window may
//! be put, and `crate::canvas_never_lost` holds a drag to the **whole output's**
//! reach. So a span too wide for the clear room at the furthest zoom out is shown
//! across the whole screen, exactly as before, rather than refused — see
//! `Server::show_all_on_the_canvas`.

use smithay::utils::{Physical, Rectangle};

/// How far inside the clear room the windows stop, in logical pixels.
///
/// `space/8` of the design's spacing scale. See the module's note for why it is
/// that step and not a figure read off a frame.
pub(crate) const THE_MARGIN_SHOW_ALL_LEAVES: u32 = 32;

/// The largest part of `output` that none of `controls` covers, as each control
/// takes the side of the room it sits on.
///
/// A control that does not reach the room changes nothing. A control that covers
/// the whole room leaves it as it was, because there is no clear piece to keep
/// and an empty room would fit nothing.
#[must_use]
pub(crate) fn the_room_left_clear(
    output: Rectangle<i32, Physical>,
    controls: &[Rectangle<i32, Physical>],
) -> Rectangle<i32, Physical> {
    controls
        .iter()
        .fold(output, |room, control| beside(room, *control))
}

/// `room` with `margin` taken off every side, or [`None`] where nothing would be
/// left.
#[must_use]
pub(crate) fn inside_the_margin(
    room: Rectangle<i32, Physical>,
    margin: i32,
) -> Option<Rectangle<i32, Physical>> {
    let twice = margin.checked_mul(2)?;
    let width = room.size.w.checked_sub(twice)?;
    let height = room.size.h.checked_sub(twice)?;
    (width > 0 && height > 0).then(|| {
        Rectangle::new(
            (room.loc.x + margin, room.loc.y + margin).into(),
            (width, height).into(),
        )
    })
}

/// The largest piece of `room` beside `control`.
fn beside(
    room: Rectangle<i32, Physical>,
    control: Rectangle<i32, Physical>,
) -> Rectangle<i32, Physical> {
    // **A meeting is not a cover.** smithay's `intersection` answers a rectangle
    // of no width or no height for two that only touch, and for a control of no
    // extent inside the room — an empty panel's column — and either would split
    // the room around nothing. The unit test caught the second on the gate.
    let Some(covered) = room
        .intersection(control)
        .filter(|it| it.size.w > 0 && it.size.h > 0)
    else {
        return room;
    };
    let (left, top) = (room.loc.x, room.loc.y);
    let (right, bottom) = (left + room.size.w, top + room.size.h);
    let (from_x, from_y) = (covered.loc.x, covered.loc.y);
    let (to_x, to_y) = (from_x + covered.size.w, from_y + covered.size.h);
    // Above, below, left of and right of what the control covers. The order is
    // the tie-break: a control in a corner leaves equal pieces, and taking the
    // one above or below first keeps the room's full width, which is the
    // longer side of almost every screen.
    let pieces = [
        Rectangle::new((left, top).into(), (room.size.w, from_y - top).into()),
        Rectangle::new((left, to_y).into(), (room.size.w, bottom - to_y).into()),
        Rectangle::new((left, top).into(), (from_x - left, room.size.h).into()),
        Rectangle::new((to_x, top).into(), (right - to_x, room.size.h).into()),
    ];
    let area = |it: &Rectangle<i32, Physical>| i64::from(it.size.w) * i64::from(it.size.h);
    // Reversed, because `max_by_key` keeps the last of equals and the tie-break
    // above wants the first.
    pieces
        .into_iter()
        .filter(|it| it.size.w > 0 && it.size.h > 0)
        .rev()
        .max_by_key(area)
        .unwrap_or(room)
}

#[cfg(test)]
#[path = "canvas_the_room_show_all_fits_into_tests.rs"]
mod tests;
