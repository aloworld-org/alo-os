//! Validating and painting laid-out Settings into a frame.
//!
//! Painted above clients, popups, window controls, the desktop and the record
//! window, and below the approval surface, the egress indicator and the
//! cursor: changing a setting is never a reason to hide a question waiting for
//! an answer or what is leaving the machine.
//!
//! **The window is one raster** since the redesign of 2026-10-31 — its shadow,
//! its rounded corners and every ring and switch are partial coverage — so it
//! is uploaded before the frame opens, the way the terminal's body is
//! (`crate::scene_drawing`), and drawn here from that texture.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Rectangle, Size},
};

use crate::RenderError;
use crate::painted::Washed;
use crate::settings_raster::SettingsPicture;

impl SettingsPicture {
    /// Refuse a frame Settings was not laid out for, before anything is
    /// imported or drawn.
    ///
    /// **A window on the canvas is laid out in a room of its own** and may sit
    /// partly off the display, so it is held to its own room rather than to the
    /// output: every pixel of the raster inside the room it was laid out for.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        let room: Rectangle<i32, Physical> = if self.is_on_the_canvas() {
            Rectangle::from_size(self.size.into())
        } else {
            Rectangle::from_size(size)
        };
        let inside = self
            .washed
            .as_ref()
            .is_none_or(|washed| room.contains_rect(washed.area));
        if (self.is_on_the_canvas() || (size.w, size.h) == self.size) && inside {
            Ok(())
        } else {
            Err(RenderError::SettingsScene)
        }
    }

    /// The raster to upload before a frame opens: the whole window.
    pub(crate) const fn washed(&self) -> Option<&Washed> {
        self.washed.as_ref()
    }

    /// Draw the window from the texture [`Self::washed`] asked for, already
    /// uploaded; [`None`] draws nothing, which is what a renderer that could
    /// not take the raster leaves.
    pub(crate) fn paint<F: Frame>(
        &self,
        frame: &mut F,
        window: Option<&(F::TextureId, Rectangle<i32, Physical>)>,
    ) -> Result<(), RenderError> {
        let Some((texture, area)) = window else {
            return Ok(());
        };
        // **Through the camera**, for a window on the canvas: the raster is a
        // part of its room, so where it lands is that part of where the room
        // lands, scaled by the zoom.
        match self.drawn_at {
            Some(room) => crate::painted::wash_scaled(
                frame,
                texture,
                (area.size.w, area.size.h),
                crate::a_room_through_the_camera::on_the_screen(*area, self.size, room),
            ),
            None => crate::painted::wash(frame, texture, *area),
        }
    }
}
