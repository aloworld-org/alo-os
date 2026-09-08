//! Seat-wide transitions after trusted backend event extraction.

use crate::{DirectKeyEvent, DirectPointerEvent, InputError, Server};
use smithay::backend::input::{ButtonState, KeyState};

/// Trusted input data, never an agent or client IPC surface.
pub enum DirectSeatEvent {
    /// A key transition and the count of this key held across the seat afterwards.
    Key(DirectKeyEvent, u32),
    /// A pointer button transition and its seat-wide held count afterwards.
    Button {
        /// Linux evdev mouse button.
        code: u32,
        /// Press or release.
        state: ButtonState,
        /// Monotonic milliseconds, wrapping at the protocol's u32 boundary.
        time: u32,
        /// Number of devices holding this button after the transition.
        count: u32,
    },
    /// Relative/absolute motion or a converted scroll frame.
    Pointer(DirectPointerEvent),
    /// Any device removal conservatively cancels the entire seat's interaction.
    Removed,
}

impl Server {
    /// Route an extracted event only under a fresh trusted seat activity check.
    ///
    /// First press and last release implement one logical keyboard/pointer across
    /// multiple devices. Removal clears all held input, focus and popup/drag grabs:
    /// this deliberately cancels other devices' ongoing interactions too, rather
    /// than risking a stuck modifier or transferable initiating serial. Focus must
    /// be restored explicitly and pointer motion must be fresh after cleanup.
    /// Missing input capabilities refuse normal events but never prevent cleanup.
    /// Events are queued; the caller must flush. This does not acquire devices.
    pub fn direct_seat(
        &mut self,
        active: bool,
        extent: (i32, i32),
        event: DirectSeatEvent,
    ) -> Result<(), InputError> {
        if !active || matches!(event, DirectSeatEvent::Removed) {
            self.clear_input();
            return Ok(());
        }
        match event {
            DirectSeatEvent::Key(event, count) => {
                if !(1..=0x2ff).contains(&event.code)
                    || (event.state == KeyState::Pressed && count == 0)
                {
                    return Err(InputError::InvalidKey);
                }
                if count == u32::from(event.state == KeyState::Pressed) {
                    self.direct_keyboard(true, Some(event))?;
                }
            }
            DirectSeatEvent::Button {
                code,
                state,
                time,
                count,
            } => {
                if !(0x110..=0x117).contains(&code) || (state == ButtonState::Pressed && count == 0)
                {
                    return Err(InputError::InvalidPointer);
                }
                if count == u32::from(state == ButtonState::Pressed) {
                    self.direct_pointer(
                        true,
                        extent,
                        Some(DirectPointerEvent::Button { code, state, time }),
                    )?;
                }
            }
            DirectSeatEvent::Pointer(event) => self.direct_pointer(true, extent, Some(event))?,
            DirectSeatEvent::Removed => {}
        }
        Ok(())
    }
}
