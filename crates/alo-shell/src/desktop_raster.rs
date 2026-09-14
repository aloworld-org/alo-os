//! The ordinary desktop for one display: the dock, and the two windows a person
//! may have open over the room the dock leaves.
//!
//! The windows are laid out in what is left of the display once the dock has
//! taken its edge, so no window covers the dock or its status area. With one
//! window open it has that room; with both, the room is shared along its longer
//! side, and the window of what is running takes the half a person starts
//! reading from. Each window's rows are `crate::running_rows` and
//! `crate::filling_rows`, set out by `crate::desktop_list`.

use alo_dock::{Dock, Edge};
use alo_strings::{Direction, Strings};
use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle};

use crate::desktop_list::{ListLook, ListPicture, ListShows};
use crate::dock_raster::DockPicture;
use crate::{DesktopLook, FillingShows, FillingWindow, RenderError, RunningShows, RunningWindow};

/// The desktop for one display, ready to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopPicture {
    /// The display size it was laid out for.
    pub(crate) size: (i32, i32),
    /// The dock.
    pub(crate) dock: DockPicture,
    /// The window of what is running.
    pub(crate) running: ListPicture,
    /// The window of what is filling the disk.
    pub(crate) filling: ListPicture,
}

/// What the running window shows, as a panel.
pub(crate) fn running_shows(window: &RunningWindow, strings: &Strings) -> ListShows {
    match window.shows() {
        RunningShows::Nothing => ListShows::Nothing,
        RunningShows::Refusal(why) => ListShows::Refusal(why.said(strings).into_text()),
        RunningShows::Running {
            running,
            moved_past,
        } => ListShows::Rows {
            remarks: crate::running_rows::remarks(running, strings),
            rows: crate::running_rows::rows(running, strings),
            first: moved_past,
            selected: None,
        },
    }
}

/// What the filling window shows, as a panel.
pub(crate) fn filling_shows(window: &FillingWindow, strings: &Strings) -> ListShows {
    match window.shows() {
        FillingShows::Nothing => ListShows::Nothing,
        FillingShows::Refusal(why) => ListShows::Refusal(why.said(strings).into_text()),
        FillingShows::Holding {
            holding,
            opened,
            selected,
        } => ListShows::Rows {
            remarks: crate::filling_rows::remarks(holding, strings),
            rows: crate::filling_rows::rows(holding, opened, strings),
            first: 0,
            selected: Some(selected),
        },
    }
}

/// Lay the desktop out on a display of `size` and rasterise it.
///
/// # Errors
/// Every refusal the dock makes, and [`RenderError::DesktopScene`] when an open
/// window cannot be held whole in the room the dock leaves.
pub(crate) fn picture(
    dock: &Dock,
    look: DesktopLook,
    running: &ListShows,
    filling: &ListShows,
    fonts: &mut FontSystem,
    size: (i32, i32),
) -> Result<DesktopPicture, RenderError> {
    let dock_picture = crate::dock_raster::picture(dock, look, size)?;
    let palette = look.palette().map_err(|_| RenderError::AccentRefused)?;
    let measure = look.measure();
    let list = ListLook {
        measure,
        palette,
        right_to_left: look.reading() == Direction::RightToLeft,
    };
    let room = room_beside(&dock_picture, measure.px(16));
    let both = !matches!(running, ListShows::Nothing) && !matches!(filling, ListShows::Nothing);
    let (running_room, filling_room) = if both {
        shared(room, look.reading())
    } else {
        (room, room)
    };
    let running = crate::desktop_list::picture(running, fonts, size, running_room, list)?;
    let filling = crate::desktop_list::picture(filling, fonts, size, filling_room, list)?;
    Ok(DesktopPicture {
        size,
        dock: dock_picture,
        running,
        filling,
    })
}

/// The room the dock leaves, inset by `margin` on every side.
fn room_beside(dock: &DockPicture, margin: i32) -> Rectangle<i32, Physical> {
    let (width, height) = dock.size;
    let thick = match dock.layout.edge() {
        Edge::Bottom | Edge::Top => dock.band.size.h,
        Edge::Left | Edge::Right => dock.band.size.w,
    };
    let (x, y, w, h) = match dock.layout.edge() {
        Edge::Bottom => (0, 0, width, height - thick),
        Edge::Top => (0, thick, width, height - thick),
        Edge::Left => (thick, 0, width - thick, height),
        Edge::Right => (0, 0, width - thick, height),
    };
    Rectangle::new(
        (x + margin, y + margin).into(),
        ((w - 2 * margin).max(0), (h - 2 * margin).max(0)).into(),
    )
}

/// One room shared by two windows along its longer side: the first for the
/// window of what is running, on the side a person starts reading from.
fn shared(
    room: Rectangle<i32, Physical>,
    reading: Direction,
) -> (Rectangle<i32, Physical>, Rectangle<i32, Physical>) {
    let gap = (room.size.w.min(room.size.h) / 64).max(1);
    if room.size.w >= room.size.h {
        let half = (room.size.w - gap) / 2;
        let left = Rectangle::new(room.loc, (half, room.size.h).into());
        let right = Rectangle::new(
            (room.loc.x + room.size.w - half, room.loc.y).into(),
            (half, room.size.h).into(),
        );
        match reading {
            Direction::LeftToRight => (left, right),
            Direction::RightToLeft => (right, left),
        }
    } else {
        let half = (room.size.h - gap) / 2;
        (
            Rectangle::new(room.loc, (room.size.w, half).into()),
            Rectangle::new(
                (room.loc.x, room.loc.y + room.size.h - half).into(),
                (room.size.w, half).into(),
            ),
        )
    }
}

#[cfg(test)]
#[path = "desktop_raster_tests.rs"]
mod tests;
