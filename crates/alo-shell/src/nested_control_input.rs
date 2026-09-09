//! Parent pointer position and native/client routing for the nested event pump.
use crate::{
    InputError, NestedPointerEvent, Server, WindowControlPointerEvent, WindowControlRouteError,
};
use smithay::backend::input::ButtonState;

/// One nested backend's pointer history, independent of the client seat position.
///
/// Keep this with its backend and server for their entire lifetime. Native grabs
/// intentionally freeze client motion, so subsequent buttons must use this actual
/// parent position instead. This contains no window or execution authority.
#[derive(Default)]
pub struct NestedControlInput {
    /// Last finite parent motion since activation; never inferred from client focus.
    position: Option<(f64, f64)>,
}

impl NestedControlInput {
    /// Current parent position for fresh hover feedback, absent after input loss.
    pub fn position(&self) -> Option<(f64, f64)> {
        self.position
    }

    /// Route one ordered parent event exactly once against published controls.
    ///
    /// Supply None on activation changes/close. Deactivation retires presentation
    /// and requires fresh motion, but still drains owned native primary releases.
    /// Invalid motion cancels input and forgets position before refusal. Errors
    /// must not be retried or forwarded. Scroll and non-primary buttons retain
    /// ordinary seat routing; keyboard input is handled separately by the pump.
    pub fn route(
        &mut self,
        server: &mut Server,
        active: bool,
        event: Option<NestedPointerEvent>,
    ) -> Result<(), WindowControlRouteError> {
        if !active {
            self.position = None;
            server.pointer_leave()?;
        }
        // A cancelled release is still ours, even with no new motion or while
        // inactive. NaN cannot hit a control; retirement already disarmed it.
        if let Some(NestedPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Released,
            time,
        }) = &event
            && server.control_press.is_some()
        {
            server.route_presented_window_control_pointer(
                self.position.unwrap_or((f64::NAN, f64::NAN)),
                WindowControlPointerEvent::Button(0x110, ButtonState::Released),
                *time,
            )?;
            return Ok(());
        }
        if !active {
            return Ok(());
        }
        match event {
            Some(NestedPointerEvent::Motion { x, y, time }) => {
                if !crate::pointer::bounded(x) || !crate::pointer::bounded(y) {
                    self.position = None;
                    server.pointer_leave()?;
                    return Err(InputError::InvalidPointer.into());
                }
                if server.surfaces.pointer.is_none() {
                    return Err(InputError::PointerUnavailable.into());
                }
                server.route_presented_window_control_pointer(
                    (x, y),
                    WindowControlPointerEvent::Motion,
                    time,
                )?;
                self.position = Some((x, y));
            }
            Some(NestedPointerEvent::Button { code, state, time }) => {
                if let Some(position) = self.position {
                    server.route_presented_window_control_pointer(
                        position,
                        WindowControlPointerEvent::Button(code, state),
                        time,
                    )?;
                }
            }
            Some(NestedPointerEvent::Axis(frame)) => {
                server.focus_window_control(None);
                if self.position.is_some() {
                    server.pointer_axis(frame)?;
                }
            }
            None => {}
        }
        Ok(())
    }
}
