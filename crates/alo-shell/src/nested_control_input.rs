//! Parent pointer position and native/client routing for the nested event pump.
use crate::{
    InputError, NestedPointerEvent, Server, WindowControlPointerEvent, WindowControlRouteError,
};
use smithay::backend::input::ButtonState;

/// One nested backend's pointer history, independent of the client seat position.
///
/// Keep this with its backend and server for their entire lifetime. Native grabs
/// intentionally freeze client motion, so subsequent buttons must use this actual
/// parent position instead. Reader transactions retain publication-bound authority
/// and cancelled release ownership; use `route_reader` throughout a reader session.
#[derive(Default)]
pub struct NestedControlInput {
    /// Last finite parent motion since activation; never inferred from client focus.
    pub(crate) position: Option<(f64, f64)>,
    /// Reader ownership survives removal, deactivation and routing failures.
    pub(crate) reader: crate::WindowControlReaderInput,
    /// Pending native F1 opening, including cancelled release ownership.
    pub(crate) opening: crate::nested_reader_session::NameOpening,
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
        if let Some(NestedPointerEvent::Button {
            code,
            state: ButtonState::Released,
            ..
        }) = &event
            && server.control_overlay.release(*code)
        {
            return Ok(());
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
                if let Some(position) = self.position {
                    server.route_presented_window_control_axis(position, frame)?;
                }
            }
            None => {}
        }
        Ok(())
    }
}
