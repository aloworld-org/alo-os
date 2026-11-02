//! **Settings is a window on the canvas**: where it is on the plane, how big
//! it is, and where a display draws it.
//!
//! [ADR 0145](../../../docs/decisions/0145-alos-own-windows-live-on-the-canvas.md),
//! stage 3 of 4, after the terminal and the task windows. Until then Settings
//! was drawn fixed to the display, centred, over every application and the
//! Dock, on every Place; zooming the canvas left it full size, and *Show all*
//! could not see it.
//!
//! # Its size is the design's rule; its place is the canvas's
//!
//! **How big** stays what `crate::settings_frame` says for the display it is
//! on — at most the design's 1200 × 816, never closer to the screen's edge than
//! the design leaves — through the person's text size and the display's scale.
//! So making the text larger in Settings still makes the window larger. It is
//! worked out each frame, and the last size is kept for *Show all*.
//!
//! **Where** is the canvas's: on the first frame after it opens it is placed
//! where a new window goes (`docs/design/where-a-new-window-opens.md`) — in the
//! view, beside the window in front, never exactly on top of another — on the
//! Place being looked at, and it stays there. It pans and zooms with the
//! canvas, and on another Place it is not drawn, not pressed and takes no keys.
//!
//! # Drawn in a room of its own
//!
//! `crate::settings_raster::picture` lays the window out for a display. It is
//! handed a **room** instead: the window and the design's least margin round
//! it, which that rule turns back into exactly this window at the room's
//! margin. The room is drawn where the camera puts it, as the terminal's
//! picture is; nothing in the layout had to learn about a camera.

use alo_appearance::TextScale;
use alo_strings::Direction;
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::settings_frame::TheFrame;
use crate::settings_measures as m;

/// How big Settings is on one display, and the room it is laid out in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HowBigSettingsIs {
    /// The window, in the display's pixels.
    pub(crate) window: (i32, i32),
    /// The room around it it is laid out in: the window and the design's
    /// least margin on every side.
    pub(crate) room: (i32, i32),
    /// Where the window is in its room.
    pub(crate) margin: (i32, i32),
}

/// How big Settings is on a display of `size` pixels drawn at `display`, for a
/// person whose text is `text` large: the design's rule, in the display's
/// pixels. [`None`] for a display with no room for it.
///
/// **The room is worked out in design units and rounded up**, so the rule
/// `crate::settings_raster::picture` applies to it — a room's pixels floored
/// into design units — gives back exactly this window at exactly the margin.
pub(crate) fn how_big_settings_is(
    size: (i32, i32),
    text: TextScale,
    display: DisplayScale,
) -> Option<HowBigSettingsIs> {
    let measure = Measure::of(text).on(display);
    let factor = measure.factor();
    if factor <= 0.0 {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "a display's pixels over a factor near one are a few thousand design units"
    )]
    let design = |pixels: i32| -> u32 { (pixels.max(0) as f32 / factor).floor() as u32 };
    let frame = TheFrame::on(design(size.0), design(size.1)).ok()?;
    let px = |logical: u32| measure.px(i32::try_from(logical).unwrap_or(i32::MAX));
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "a room of a few thousand design units through a factor near one fits an i32"
    )]
    let up = |logical: u32| -> i32 { (logical as f32 * factor).ceil() as i32 };
    Some(HowBigSettingsIs {
        window: (px(frame.wide), px(frame.tall)),
        room: (
            up(frame.wide + 2 * m::BESIDE_THE_WINDOW),
            up(frame.tall + 2 * m::ABOVE_THE_WINDOW),
        ),
        margin: (px(m::BESIDE_THE_WINDOW), px(m::ABOVE_THE_WINDOW)),
    })
}

impl crate::Server {
    /// Whether Settings is open on the Place being looked at — one not placed
    /// yet is on this one.
    pub(crate) fn settings_is_open_here(&self) -> bool {
        self.settings.is_open()
            && self
                .settings_on_the_plane
                .is_none_or(|(_, place)| place == self.the_place_now())
    }

    /// Where Settings is on the canvas, while it is open on the Place being
    /// looked at and has been placed.
    #[must_use]
    pub(crate) fn where_settings_is_on_the_plane(&self) -> Option<Rectangle<i32, Logical>> {
        if !self.settings.is_open() {
            return None;
        }
        let (at, place) = self.settings_on_the_plane?;
        (place == self.the_place_now()).then(|| Rectangle::new(at, self.how_big_settings_was))
    }

    /// Give Settings its place on the canvas if it is open and has none, and
    /// forget its place once it is shut. Answers whether it placed it.
    ///
    /// `window` is how big it is on this display (`how_big_settings_is`), and
    /// `display` the display's size.
    pub(crate) fn settings_is_placed(
        &mut self,
        window: (i32, i32),
        display: (i32, i32),
        reading: Direction,
    ) -> bool {
        if !self.settings.is_open() {
            self.settings_on_the_plane = None;
            self.settings_as_drawn = None;
            return false;
        }
        self.how_big_settings_was = Size::from(window);
        if self.settings_on_the_plane.is_some() {
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
            .collect();
        let at = crate::where_a_window_opens(
            view,
            &open,
            open.first().copied(),
            Size::from(window),
            reading,
            &[],
        );
        self.settings_on_the_plane = Some((at, place));
        true
    }

    /// Settings as a frame on the canvas, for *Show all* and *fit the window
    /// in front* to count.
    #[must_use]
    pub(crate) fn settings_as_a_frame(&self) -> Option<alo_canvas::Frame> {
        let window = self.where_settings_is_on_the_plane()?;
        let at = alo_canvas::At::checked(window.loc.x, window.loc.y)?;
        let size = alo_canvas::Size::checked(
            u32::try_from(window.size.w).ok()?,
            u32::try_from(window.size.h).ok()?,
        )?;
        Some(alo_canvas::Frame::of(
            self.the_settings_number,
            self.the_place_now(),
            at,
            size,
        ))
    }

    /// Where Settings' room lands on the display named, through its camera:
    /// the room's corner on the plane is the window's less its margin.
    pub(crate) fn where_the_settings_room_is_drawn(
        &self,
        named: &str,
        sizes: HowBigSettingsIs,
    ) -> Option<Rectangle<i32, Physical>> {
        let window = self.where_settings_is_on_the_plane()?;
        let corner: Point<i32, Logical> =
            Point::from((window.loc.x - sizes.margin.0, window.loc.y - sizes.margin.1));
        Some(crate::the_terminal_on_the_plane::on_the_screen(
            Rectangle::new(corner, sizes.room.into()),
            self.the_camera_of_the_display(named),
        ))
    }

    /// **The person went to Settings**: it comes in front, and the view goes
    /// to it where it is out of sight — its Place first — without moving it.
    pub(crate) fn the_person_went_to_settings(&mut self) {
        self.the_composer.it_gave_the_keyboard_up();
        self.the_person_went_to(crate::which_window_is_in_front::AKindOfWindow::Settings);
        let _ = self.the_view_goes_to_settings();
    }

    /// Bring Settings into view if it is out of it, travelling to where it is
    /// rather than pulling it to where the person is looking
    /// (`docs/design/the-canvas-as-a-workspace.md` §11). Answers whether the
    /// view moved.
    pub(crate) fn the_view_goes_to_settings(&mut self) -> bool {
        if !self.settings.is_open() {
            return false;
        }
        let Some((_, its_place)) = self.settings_on_the_plane else {
            return false;
        };
        let somewhere_else = its_place != self.the_place_now();
        if somewhere_else {
            self.surfaces.place = its_place;
        }
        let Some(window) = self.where_settings_is_on_the_plane() else {
            return somewhere_else;
        };
        let Some(room) = self.surfaces.popups.output_size else {
            return somewhere_else;
        };
        let camera = self.the_camera();
        let seen = crate::the_terminal_on_the_plane::on_the_screen(window, camera);
        if Rectangle::<i32, Physical>::from_size(room).contains_rect(seen) {
            return somewhere_else;
        }
        let zoom = crate::scene::drawn_at(camera);
        if zoom <= 0.0 {
            return somewhere_else;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "half a display divided by a zoom of at least a twentieth fits an i32"
        )]
        let at = alo_canvas::At::checked(
            window.loc.x + window.size.w / 2 - (f64::from(room.w) / 2.0 / zoom) as i32,
            window.loc.y + window.size.h / 2 - (f64::from(room.h) / 2.0 / zoom) as i32,
        );
        at.and_then(|at| self.look_at_the_canvas(at)).is_some() || somewhere_else
    }

    /// What this frame drew of Settings, for the press after it.
    pub(crate) fn settings_was_drawn(
        &mut self,
        drawn: Option<crate::settings_as_drawn::SettingsAsDrawn>,
    ) {
        self.settings_as_drawn = drawn;
    }

    /// Whether Settings is drawn under `at`, a point on the display.
    pub(crate) fn settings_is_under(&self, at: Point<i32, Physical>) -> bool {
        self.settings_is_open_here()
            && self
                .settings_as_drawn
                .as_ref()
                .is_some_and(|drawn| drawn.is_under(at))
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "settings_on_the_plane_tests.rs"]
mod tests;
