//! **The room a maximised or full-screen window fills, display by display.**
//!
//! `docs/design/the-window-controls.md`: *Maximise fills the work area and
//! keeps the band and the shell's areas; Full screen fills the display.* Until
//! 2026-11-09 both were the last drawn output at the plane's origin — so the
//! band, which sits **above** a window (`crate::window_edge`), was off the top
//! of the screen, a maximised window lay under the Dock and the panel, and on a
//! canvas looked at from anywhere but the origin the window went somewhere the
//! person was not looking.
//!
//! # What a display's room is
//!
//! **Whole:** what the display shows of the plane through its own camera —
//! each display is a view onto the one canvas, so this is where a full-screen
//! window goes.
//!
//! **Work:** the whole less what the shell reserves on it, as last drawn — the
//! Dock's band, the put-aside panel's column, the top controls and the egress
//! indicator (`crate::canvas_fixed_controls::FixedControlsDrawn`). The Dock's
//! band and the top controls are **kept when they stop being drawn**, because
//! `docs/design/when-the-dock-gives-way.md` settles that the work area does not
//! depend on the Dock giving way: a work area that grew when the Dock hid would
//! be the oscillation that note was written to prevent.
//!
//! A maximised window's **band** goes at the top of the work area and the
//! window under it, so its title and controls are on the screen.
//!
//! # Which display's room
//!
//! **The display showing most of the window**, through each display's own
//! camera — the rule `crate::which_display_a_window_is_on` gives for the desk,
//! asked here on the plane, where windows are. A window no display shows takes
//! the first room known, which is the display drawn first.

use smithay::utils::{Logical, Physical, Rectangle};

use crate::window_mode::Share;

/// One display's room on the plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ADisplaysRoom {
    /// What the display shows of the plane.
    pub(crate) whole: Rectangle<i32, Logical>,
    /// The whole less what the shell reserves on it.
    pub(crate) work: Rectangle<i32, Logical>,
    /// How tall a window's band is here, on the plane.
    pub(crate) band: i32,
}

/// What the shell reserved on one display, as last drawn, in its pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct WhatIsReserved {
    /// The Dock's band, kept while the Dock gives way.
    pub(crate) dock: Option<Rectangle<i32, Physical>>,
    /// The put-aside panel's column.
    pub(crate) panel: Option<Rectangle<i32, Physical>>,
    /// The top controls, kept while a window fills the screen.
    pub(crate) top: Option<Rectangle<i32, Physical>>,
    /// The egress indicator, while it draws.
    pub(crate) leaving: Option<Rectangle<i32, Physical>>,
}

impl WhatIsReserved {
    /// This draw's controls, keeping the Dock's band and the top controls
    /// from `before` where this draw had none.
    pub(crate) fn drawn(
        drawn: Option<crate::canvas_fixed_controls::FixedControlsDrawn>,
        before: Self,
    ) -> Self {
        let Some(drawn) = drawn else {
            return before;
        };
        Self {
            dock: drawn.dock_band.or(before.dock),
            panel: Some(drawn.panel_reserved),
            top: drawn.top_controls.or(before.top),
            leaving: drawn.what_is_leaving,
        }
    }

    fn each(self) -> impl Iterator<Item = Rectangle<i32, Physical>> {
        [self.dock, self.panel, self.top, self.leaving]
            .into_iter()
            .flatten()
            .filter(|it| it.size.w > 0 && it.size.h > 0)
    }
}

/// The display less what is reserved on it, in its own pixels.
///
/// **Each reserved rectangle takes the edge it stands against**: one at least
/// half the display tall is a column and takes the side its middle is on —
/// the panel's, on the right or, reading right to left, the left; any other
/// takes the top or the bottom its middle is nearer — the top controls and the
/// indicator the top, the Dock the bottom. The Dock floats clear of the
/// bottom edge, and everything below its top is still the Dock's.
pub(crate) fn the_work_area(
    size: (i32, i32),
    reserved: WhatIsReserved,
) -> Rectangle<i32, Physical> {
    let (width, height) = (size.0.max(0), size.1.max(0));
    let (mut left, mut top, mut right, mut bottom) = (0, 0, width, height);
    for it in reserved.each() {
        let (x, y, w, h) = (it.loc.x, it.loc.y, it.size.w, it.size.h);
        if h.saturating_mul(2) >= height {
            if x.saturating_add(w / 2) < width / 2 {
                left = left.max(x.saturating_add(w));
            } else {
                right = right.min(x);
            }
        } else if y.saturating_add(h / 2) < height / 2 {
            top = top.max(y.saturating_add(h));
        } else {
            bottom = bottom.min(y);
        }
    }
    Rectangle::new(
        (left, top).into(),
        ((right - left).max(0), (bottom - top).max(0)).into(),
    )
}

/// A rectangle of a display's pixels as the plane holds it, through this
/// camera — **rounded inward**, so a window sized to it never reaches into
/// what was reserved.
pub(crate) fn onto_the_plane(
    on_the_display: Rectangle<i32, Physical>,
    camera: alo_canvas::Camera,
) -> Option<Rectangle<i32, Logical>> {
    let zoom = crate::scene::drawn_at(camera);
    if !zoom.is_finite() || zoom <= 0.0 {
        return None;
    }
    let at = camera.at();
    let plane = |pixels: i32, from: i32| f64::from(pixels) / zoom + f64::from(from);
    let left = plane(on_the_display.loc.x, at.x).ceil();
    let top = plane(on_the_display.loc.y, at.y).ceil();
    let right = plane(on_the_display.loc.x + on_the_display.size.w, at.x).floor();
    let bottom = plane(on_the_display.loc.y + on_the_display.size.h, at.y).floor();
    let fits = |it: f64| it.is_finite() && it.abs() < 1_000_000.0;
    if ![left, top, right, bottom].into_iter().all(fits) || right < left || bottom < top {
        return None;
    }
    Some(Rectangle::new(
        (left as i32, top as i32).into(),
        ((right - left) as i32, (bottom - top) as i32).into(),
    ))
}

impl ADisplaysRoom {
    /// One display's room: its size in pixels, what is reserved on it, the
    /// camera it looks through and its scale in per cent.
    pub(crate) fn of(
        size: (i32, i32),
        reserved: WhatIsReserved,
        camera: alo_canvas::Camera,
        scale: u16,
    ) -> Option<Self> {
        let whole = onto_the_plane(Rectangle::new((0, 0).into(), size.into()), camera)?;
        let work = onto_the_plane(the_work_area(size, reserved), camera)?;
        // The band is laid out in logical units at the display's scale and is
        // the same size on the plane at every zoom, because it zooms with its
        // window (`crate::a_window_and_its_strip`).
        let band = (i32::from(scale) * crate::window_edge::THE_REGION_IS_TALL).div_ceil(100);
        Some(Self { whole, work, band })
    }

    /// Where a maximised window goes: the work area, less its band at the top.
    ///
    /// [`None`] where the work area is too short to hold a band and anything
    /// under it — a refusal, never a window whose band is off the screen.
    pub(crate) fn maximised(self) -> Option<Share> {
        let height = self.work.size.h - self.band;
        (height > 0 && self.work.size.w > 0).then_some(Share {
            at: (self.work.loc.x, self.work.loc.y + self.band),
            size: (self.work.size.w, height),
        })
    }

    /// Where a full-screen window goes: everything the display shows.
    pub(crate) fn full_screen(self) -> Option<Share> {
        (self.whole.size.w > 0 && self.whole.size.h > 0).then_some(Share {
            at: (self.whole.loc.x, self.whole.loc.y),
            size: (self.whole.size.w, self.whole.size.h),
        })
    }
}

/// The room of the display showing most of `window`, or the first room where
/// none shows any of it.
pub(crate) fn the_room_of(
    rooms: &[ADisplaysRoom],
    window: Rectangle<i32, Logical>,
) -> Option<ADisplaysRoom> {
    let shown = |room: &ADisplaysRoom| {
        room.whole.intersection(window).map_or(0, |it| {
            i64::from(it.size.w.max(0)) * i64::from(it.size.h.max(0))
        })
    };
    // `max_by_key` keeps the last of equals, so the order is reversed to keep
    // the first: at equal shares the display drawn first wins.
    rooms
        .iter()
        .rev()
        .max_by_key(|room| shown(room))
        .copied()
        .filter(|room| shown(room) > 0)
        .or_else(|| rooms.first().copied())
}

impl crate::Server {
    /// Note this display's room, as its camera and its last drawn controls
    /// make it now.
    pub(crate) fn this_displays_room_was_seen(&mut self, named: &str, size: (i32, i32)) {
        // The camera is the one this display is drawn through: its own with
        // an arrangement, the session's one without (`crate::canvas_camera`).
        let looking = if self.screens.is_some() { named } else { "" };
        let camera = self.surfaces.camera_of(looking);
        let before = self
            .surfaces
            .rooms
            .iter()
            .find(|(name, _, _)| name == named)
            .map(|(_, _, reserved)| *reserved)
            .unwrap_or_default();
        let reserved = WhatIsReserved::drawn(
            self.fixed_controls
                .get(named)
                .and_then(crate::canvas_fixed_controls::FixedControls::labelled),
            before,
        );
        let scale = self.the_scale_of_display(named);
        self.surfaces.rooms.retain(|(name, _, _)| name != named);
        if let Some(room) = ADisplaysRoom::of(size, reserved, camera, scale) {
            self.surfaces
                .rooms
                .push((named.to_owned(), room, reserved));
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "the_room_a_window_fills_tests.rs"]
mod tests;
