//! Opaque native label pointer exclusion, independent of client grabs.
use std::collections::BTreeSet;

use crate::{InputError, Server, WindowControlPointerEvent};
use smithay::{
    backend::input::ButtonState,
    utils::{Physical, Rectangle},
};

/// Pixels may disappear before their owned button releases arrive.
#[derive(Default)]
pub(crate) struct Overlay {
    /// Last successfully submitted label rectangle.
    pub(crate) bounds: Option<Rectangle<i32, Physical>>,
    /// Every button pressed over native pixels, including secondary buttons.
    buttons: BTreeSet<u32>,
}

impl Overlay {
    /// No new label may appear during an owned gesture.
    pub(crate) fn held(&self) -> bool {
        !self.buttons.is_empty()
    }

    /// Drain even while the backend is inactive or has forgotten its position.
    pub(crate) fn release(&mut self, code: u32) -> bool {
        self.buttons.remove(&code)
    }

    /// Half-open physical bounds, with no conversion of invalid coordinates.
    fn contains(&self, (x, y): (f64, f64)) -> bool {
        self.bounds.is_some_and(|r| {
            x >= f64::from(r.loc.x)
                && y >= f64::from(r.loc.y)
                && x < f64::from(r.loc.x + r.size.w)
                && y < f64::from(r.loc.y + r.size.h)
        })
    }
}

impl Server {
    /// Exclude submitted opaque pixels without stealing existing client grabs.
    /// Keep bounds until a subsequent successful frame removes them: an event
    /// batch can contain motion and press before the host has repainted.
    pub(crate) fn route_control_overlay(
        &mut self,
        position: (f64, f64),
        event: WindowControlPointerEvent,
    ) -> Result<bool, InputError> {
        if let WindowControlPointerEvent::Button(code, ButtonState::Released) = event
            && self.control_overlay.release(code)
        {
            return Ok(true);
        }
        if self.window_control_input_busy() || self.control_press.is_some() {
            return Ok(false);
        }
        if self.control_overlay.bounds.is_none() && !self.control_overlay.held() {
            return Ok(false);
        }
        if !crate::pointer::bounded(position.0) || !crate::pointer::bounded(position.1) {
            return Err(InputError::InvalidPointer);
        }
        if !self.control_overlay.held() && !self.control_overlay.contains(position) {
            return Ok(false);
        }
        self.surfaces.clear_pointer()?;
        if let WindowControlPointerEvent::Button(code, ButtonState::Pressed) = event {
            self.control_overlay.buttons.insert(code);
        }
        Ok(true)
    }

    /// Scroll over opaque labels never reaches the client underneath.
    pub(crate) fn control_overlay_axis(
        &mut self,
        position: (f64, f64),
    ) -> Result<bool, InputError> {
        self.route_control_overlay(position, WindowControlPointerEvent::Motion)
    }
}
