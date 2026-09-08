//! Borrow libinput events without retaining device/context references.

use crate::{DirectKeyEvent, DirectPointerEvent, DirectSeatEvent, InputError, InputUpdate, Server};
use smithay::reexports::input::{
    Event,
    event::{
        DeviceEvent, KeyboardEvent, PointerEvent, keyboard::KeyboardEventTrait,
        pointer::PointerEventTrait,
    },
};

impl Server {
    /// Deliver a SeatInput update synchronously after its authority poll.
    ///
    /// Use inside `SeatInput::dispatch`; map errors to io::Error so dispatch
    /// retires the context and calls Reset. Reset queues infallible whole-seat
    /// cleanup even with no input capabilities or an invalid output extent.
    /// The caller must flush client events, including after dispatch refusal.
    /// Device additions do not select focus. Unsupported touch/tablet/gesture/
    /// switch events are ignored; v0.01 exposes keyboard and pointer only.
    /// Deprecated axis events are ignored because libinput also emits modern
    /// scroll events. Processing both would scroll twice.
    /// This callback does not wire the live DirectSession poll or frame loop.
    pub fn libinput_update(
        &mut self,
        update: InputUpdate<'_>,
        extent: (i32, i32),
    ) -> Result<(), InputError> {
        match update {
            InputUpdate::Reset => self.clear_input(),
            InputUpdate::Event(event) => {
                if let Some(event) = translate(event)? {
                    self.direct_seat(true, extent, event)?;
                }
            }
        }
        Ok(())
    }
}

/// Extract evdev codes (not Smithay's already-offset XKB keycodes).
fn translate(event: &Event) -> Result<Option<DirectSeatEvent>, InputError> {
    let pointer = match event {
        Event::Device(DeviceEvent::Removed(_)) => return Ok(Some(DirectSeatEvent::Removed)),
        Event::Keyboard(KeyboardEvent::Key(key)) => {
            return Ok(Some(DirectSeatEvent::Key(
                DirectKeyEvent {
                    code: key.key(),
                    state: key.key_state().into(),
                    time: key.time(),
                },
                key.seat_key_count(),
            )));
        }
        Event::Pointer(PointerEvent::Motion(event)) => DirectPointerEvent::Relative {
            dx: event.dx(),
            dy: event.dy(),
            time: event.time(),
        },
        Event::Pointer(PointerEvent::MotionAbsolute(event)) => DirectPointerEvent::Absolute {
            x: event.absolute_x_transformed(1),
            y: event.absolute_y_transformed(1),
            time: event.time(),
        },
        Event::Pointer(PointerEvent::Button(event)) => {
            return Ok(Some(DirectSeatEvent::Button {
                code: event.button(),
                state: event.button_state().into(),
                time: event.time(),
                count: event.seat_button_count(),
            }));
        }
        Event::Pointer(event) => match crate::libinput_scroll::scroll(event)? {
            Some(frame) => DirectPointerEvent::Axis(frame),
            None => return Ok(None),
        },
        _ => return Ok(None),
    };
    Ok(Some(DirectSeatEvent::Pointer(pointer)))
}
