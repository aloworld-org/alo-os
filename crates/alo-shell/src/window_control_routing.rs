//! Native strip interception and ordinary client fallback at one trusted boundary.
use smithay::{
    backend::input::ButtonState, reexports::wayland_server::protocol::wl_surface::WlSurface,
};

use crate::{
    InputError, Server, WindowControlPressError, WindowControlRelease, WindowControlReleaseError,
};

/// Current painted strip supplied by the trusted compositor, never an agent.
/// Recreate from current presentation on every event; this is not stored authority.
pub struct PaintedWindowControls<'a> {
    /// Exact visible root whose controls were painted.
    pub surface: &'a WlSurface,
    /// Current scale-one output dimensions.
    pub viewport: (i32, i32),
    /// Current output-local strip origin.
    pub origin: (i32, i32),
}

/// Scale-one backend event, with position supplied separately on every call.
#[derive(Debug, Clone, Copy)]
pub enum WindowControlPointerEvent {
    /// Pointer motion at the supplied output-local position.
    Motion,
    /// Linux evdev mouse button transition.
    Button(u32, ButtonState),
}

/// Routing has already occurred; never forward this event again.
#[derive(Debug, PartialEq, Eq)]
pub enum WindowControlRoute {
    /// Ordinary client route ran; bool reports button delivery (true for motion).
    Client(bool),
    /// Native press or held motion consumed without client delivery.
    Consumed,
    /// Native release consumed, including cancellation.
    Released(WindowControlRelease),
}

/// Routing refusal; no automatic fallback or retry is permitted.
#[derive(Debug, thiserror::Error)]
pub enum WindowControlRouteError {
    /// Ordinary client input validation failed.
    #[error(transparent)]
    Input(#[from] InputError),
    /// Native hit refused without taking new ownership.
    #[error(transparent)]
    Press(#[from] WindowControlPressError),
    /// Native release consumed before live operation refusal.
    #[error(transparent)]
    Release(#[from] WindowControlReleaseError),
}

impl Server {
    /// Intercept native primary gestures, otherwise use ordinary pointer routing.
    ///
    /// Supply the current painted target (None when absent), actual output-local
    /// position and monotonic event time. Target replacement/removal permanently
    /// cancels a held transaction, even with identical geometry. Owned motion and
    /// primary transitions never reach a client, including disabled/cancelled hits.
    /// Existing client grabs refuse native acquisition. Other buttons retain their
    /// ordinary route; keyboard and axis routing are separate and unchanged.
    ///
    /// Unowned motion updates ordinary seat position/focus; owned motion deliberately
    /// does neither. Button fallback uses existing seat focus, as `pointer_button`
    /// does: hosts must supply every motion. Continue routing the matching release
    /// after input loss and cancel immediately when presentation is removed between
    /// events. Backends must opt into this API when composing native controls.
    pub fn route_window_control_pointer(
        &mut self,
        painted: Option<PaintedWindowControls<'_>>,
        position: (f64, f64),
        event: WindowControlPointerEvent,
        time: u32,
    ) -> Result<WindowControlRoute, WindowControlRouteError> {
        if self.control_press.as_ref().is_some_and(|press| {
            !painted
                .as_ref()
                .is_some_and(|view| press.targets(view.surface))
        }) {
            self.cancel_window_control();
        }
        let (viewport, origin) = painted
            .as_ref()
            .map(|view| (view.viewport, view.origin))
            .unwrap_or(((0, 0), (0, 0)));
        match event {
            WindowControlPointerEvent::Motion => {
                if self.window_control_motion(viewport, origin, position) {
                    return Ok(WindowControlRoute::Consumed);
                }
                self.pointer_motion(position.0, position.1, time)?;
                Ok(WindowControlRoute::Client(true))
            }
            WindowControlPointerEvent::Button(0x110, ButtonState::Released)
                if self.control_press.is_some() =>
            {
                Ok(WindowControlRoute::Released(
                    self.release_window_control(viewport, origin, position)?,
                ))
            }
            WindowControlPointerEvent::Button(button, state) => {
                if button == 0x110 && state == ButtonState::Pressed {
                    if self.control_press.is_some() {
                        return Ok(WindowControlRoute::Consumed);
                    }
                    if let Some(view) = painted
                        && self.press_window_control(view.surface, viewport, origin, position)?
                    {
                        return Ok(WindowControlRoute::Consumed);
                    }
                }
                Ok(WindowControlRoute::Client(
                    self.pointer_button(button, state, time)?,
                ))
            }
        }
    }
}
