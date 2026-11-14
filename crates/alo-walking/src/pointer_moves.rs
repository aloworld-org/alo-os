//! **A pointer move as the steps qemu's PS/2 mouse carries whole**, and the QMP
//! command that sends one of them.
//!
//! The walk machine has the q35 PS/2 mouse and no tablet (`crate::booting`), so
//! a move is relative: *this far right, this far down*. qemu 8.2's PS/2 mouse
//! sends at most 508 units a command and carries the rest forward as a debt it
//! pays toward the corner on every later move
//! (`docs/quirks/qemu-ps2-carries-a-huge-move-forward.md`). So a move here is
//! cut into steps of at most [`LARGEST_STEP`] on each axis, and each is its own
//! command.
//!
//! **Where the pointer lands is never read from what was asked.** libinput
//! accelerates the motion, so a commanded distance is not a measured one; the
//! walk reads the pointer back off the screen after every move
//! (`crate::steering`).

use serde_json::{Value, json};

/// The most one step moves on either axis: well inside what one PS/2 packet
/// carries, and small enough, a step every 30 ms, that libinput sees a slow
/// hand and accelerates it little.
pub const LARGEST_STEP: i32 = 20;

/// The steps that make a move of `dx` right and `dy` down, each at most
/// [`LARGEST_STEP`] on each axis, adding up to exactly the move. No move is no
/// steps.
#[must_use]
pub fn in_steps(dx: i32, dy: i32) -> Vec<(i32, i32)> {
    let longest = dx.unsigned_abs().max(dy.unsigned_abs());
    if longest == 0 {
        return Vec::new();
    }
    let largest = LARGEST_STEP.unsigned_abs();
    let count = i64::from(longest.div_ceil(largest));
    let share = |whole: i32, step: i64| {
        let whole = i64::from(whole);
        let to = whole * (step + 1) / count;
        let from = whole * step / count;
        i32::try_from(to - from).unwrap_or(0)
    };
    (0..count)
        .map(|step| (share(dx, step), share(dy, step)))
        .collect()
}

/// The QMP command that sends one step.
#[must_use]
pub fn command(dx: i32, dy: i32) -> Value {
    json!({
        "execute": "input-send-event",
        "arguments": {
            "events": [
                { "type": "rel", "data": { "axis": "x", "value": dx } },
                { "type": "rel", "data": { "axis": "y", "value": dy } }
            ]
        }
    })
}

#[cfg(test)]
#[path = "pointer_moves_tests.rs"]
mod tests;
