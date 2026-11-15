//! **Putting the pointer at a place on the screen, and clicking there**, as the
//! QMP commands that do it on the walk machine.
//!
//! The walk machine has the q35 defaults: a PS/2 mouse and qemu's VMware mouse
//! port beside it, and no tablet (`crate::booting`). **Relative moves did not
//! reach the pointer.** Measured on the first walk run with pointer checks
//! (GitHub run 43646031636, 2026-11-14): `input-send-event` with `rel` events
//! of 24 and 16 changed no pixel of the screen, and the picture kept shows alo's
//! pointer at the top-left corner, where it started. The guest's kernel had
//! bound the mouse as `VirtualPS/2 VMware VMMouse` (`serial.log`, 0.83 s), and
//! that driver takes **absolute** positions from qemu's mouse port — which fits
//! the relative packets going nowhere; it was not taken apart further.
//!
//! So a move here is an **absolute** position, which qemu hands to the mouse
//! port: `0` to [`ABS_MOST`] across and down, whatever the screen's size.
//! libinput delivers it as an absolute motion and the shell maps it onto the
//! display (`alo_shell::direct_pointer`).
//!
//! **Where the pointer lands is still read back off the screen**, never assumed
//! from what was asked (`crate::steering`): the mapping from qemu's range to
//! pixels is the guest's, and a pixel or two off is the guest's to decide.

use serde_json::{Value, json};

/// The largest absolute position qemu takes on either axis.
pub const ABS_MOST: u32 = 0x7FFF;

/// The absolute value for pixel `at` on an axis `extent` pixels long.
#[must_use]
pub fn absolute(at: usize, extent: usize) -> u32 {
    let last = u64::try_from(extent.saturating_sub(1).max(1)).unwrap_or(u64::MAX);
    let at = u64::try_from(at).unwrap_or(u64::MAX).min(last);
    let value = (at * u64::from(ABS_MOST) + last / 2) / last;
    u32::try_from(value).unwrap_or(ABS_MOST).min(ABS_MOST)
}

/// The QMP command that puts the pointer at pixel `(x, y)` of a screen
/// `size` pixels across and down.
#[must_use]
pub fn command((x, y): (usize, usize), size: (usize, usize)) -> Value {
    json!({
        "execute": "input-send-event",
        "arguments": {
            "events": [
                { "type": "abs", "data": { "axis": "x", "value": absolute(x, size.0) } },
                { "type": "abs", "data": { "axis": "y", "value": absolute(y, size.1) } }
            ]
        }
    })
}

/// The QMP command that presses the left button down, or lets it up.
#[must_use]
pub fn button(down: bool) -> Value {
    json!({
        "execute": "input-send-event",
        "arguments": {
            "events": [
                { "type": "btn", "data": { "down": down, "button": "left" } }
            ]
        }
    })
}

#[cfg(test)]
#[path = "pointer_moves_tests.rs"]
mod tests;
