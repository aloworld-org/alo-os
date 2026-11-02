//! **The record is a window on the canvas**: how big it is, where it is on the
//! plane, and where a display draws it.
//!
//! [ADR 0145](../../../docs/decisions/0145-alos-own-windows-live-on-the-canvas.md),
//! stage 4 of 4. Until then the record window was drawn fixed to the display,
//! over every application and Settings' neighbours, on every Place, and zooming
//! the canvas left it full size. The ADR names why it is not the one left
//! fixed: *see what alo did* is something a person reads beside the work it
//! describes.
//!
//! # Its size is the design's rule; its place is the canvas's
//!
//! **How big** stays the rule the owner approved (`crate::record_room`): 880
//! across and 796 down at most, smaller where the display has less room, through
//! the person's text size and the display's scale. **Where** is the canvas's: it
//! is placed once, where a new window goes, on the Place being looked at, and
//! stays there until it is shut. It is laid out in a room of its own
//! (`Room::around`) — everything inside where the display rule put it, #1071's
//! layout unchanged — and drawn where the camera puts that room.

use alo_strings::Direction;
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use crate::display_scale::DisplayScale;
use crate::record_room::{RecordLook, Room};

/// How big the record window is on a display of `size` pixels drawn at
/// `display`, by the approved rule; [`None`] where the display has no room for
/// it.
#[must_use]
pub(crate) fn how_big_the_record_is(
    size: (i32, i32),
    look: RecordLook,
    display: DisplayScale,
) -> Option<(i32, i32)> {
    let room = Room::on(size, look, display).ok()?;
    Some((room.window.size.w, room.window.size.h))
}

impl crate::Server {
    /// Whether the record is open on the Place being looked at — one not placed
    /// yet is on this one.
    pub(crate) fn the_record_is_open_here(&self) -> bool {
        self.record.is_open()
            && self
                .the_record_on_the_plane
                .is_none_or(|(_, place)| place == self.the_place_now())
    }

    /// Where the record is on the canvas, while it is open on the Place being
    /// looked at and has been placed.
    #[must_use]
    pub(crate) fn where_the_record_is_on_the_plane(&self) -> Option<Rectangle<i32, Logical>> {
        if !self.record.is_open() {
            return None;
        }
        let (at, place) = self.the_record_on_the_plane?;
        (place == self.the_place_now()).then(|| Rectangle::new(at, self.how_big_the_record_was))
    }

    /// Give the record its place on the canvas if it is open and has none, and
    /// forget its place once it is shut. Answers whether it placed it.
    pub(crate) fn the_record_is_placed(
        &mut self,
        window: (i32, i32),
        display: (i32, i32),
        reading: Direction,
    ) -> bool {
        if !self.record.is_open() {
            self.the_record_on_the_plane = None;
            return false;
        }
        self.how_big_the_record_was = Size::from(window);
        if self.the_record_on_the_plane.is_some() {
            return false;
        }
        let camera = self.the_camera();
        let zoom = crate::scene::drawn_at(camera);
        if zoom <= 0.0 {
            return false;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a display divided by a zoom of at least a twentieth fits an i32"
        )]
        let view = Rectangle::<i32, Logical>::new(
            (camera.at().x, camera.at().y).into(),
            (
                (f64::from(display.0) / zoom) as i32,
                (f64::from(display.1) / zoom) as i32,
            )
                .into(),
        );
        let place = self.the_place_now();
        let open: Vec<Rectangle<i32, Logical>> = self
            .mapped_surfaces()
            .filter(|surface| crate::canvas_place::the_place_of(surface) == Some(place))
            .map(crate::where_a_window_is)
            .chain(self.where_the_terminal_is_on_the_plane())
            .chain(self.where_settings_is_on_the_plane())
            .collect();
        let at = crate::where_a_window_opens(
            view,
            &open,
            open.first().copied(),
            Size::from(window),
            reading,
            &[],
        );
        self.the_record_on_the_plane = Some((at, place));
        true
    }

    /// The record as a frame on the canvas, for *Show all* and *fit the window
    /// in front* to count.
    #[must_use]
    pub(crate) fn the_record_as_a_frame(&self) -> Option<alo_canvas::Frame> {
        let window = self.where_the_record_is_on_the_plane()?;
        let at = alo_canvas::At::checked(window.loc.x, window.loc.y)?;
        let size = alo_canvas::Size::checked(
            u32::try_from(window.size.w).ok()?,
            u32::try_from(window.size.h).ok()?,
        )?;
        Some(alo_canvas::Frame::of(
            self.the_records_number,
            self.the_place_now(),
            at,
            size,
        ))
    }

    /// Where the record's room lands on the display named: the room's corner
    /// on the plane is the window's less the room the shadow has, `reach`.
    pub(crate) fn where_the_records_room_is_drawn(
        &self,
        named: &str,
        room: (i32, i32),
    ) -> Option<Rectangle<i32, Physical>> {
        let window = self.where_the_record_is_on_the_plane()?;
        let reach = ((room.0 - window.size.w) / 2, (room.1 - window.size.h) / 2);
        let corner: Point<i32, Logical> =
            Point::from((window.loc.x - reach.0, window.loc.y - reach.1));
        Some(crate::the_terminal_on_the_plane::on_the_screen(
            Rectangle::new(corner, room.into()),
            self.the_camera_of_the_display(named),
        ))
    }

    /// **The person went to the record**: it comes in front with the keys, and
    /// the view goes to it where it is out of sight, its Place first, without
    /// moving it.
    pub(crate) fn the_person_went_to_the_record(&mut self) {
        self.the_composer.it_gave_the_keyboard_up();
        self.the_person_went_to(crate::which_window_is_in_front::AKindOfWindow::TheRecord);
        if let Some((at, place)) = self.the_record_on_the_plane
            && self.record.is_open()
        {
            let _ =
                self.the_view_travels_to(Rectangle::new(at, self.how_big_the_record_was), place);
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "record_on_the_plane_tests.rs"]
mod tests;
