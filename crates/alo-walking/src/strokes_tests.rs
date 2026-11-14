//! The order a chord's edges are sent in, and what each one says to qemu.

use std::time::Duration;

use serde_json::json;

use super::{Pace, Stroke, command, of};
use crate::keys::Chord;

/// A pace that is easy to read in an assertion.
const PACE: Pace = Pace {
    hold: Duration::from_millis(10),
    between: Duration::from_millis(99),
};

/// **Shift goes down before the letter and comes up after it**, each edge its
/// own command — the order that keeps a capital a capital.
#[test]
fn shift_is_held_around_the_letter() {
    let strokes = of(&Chord::with("shift", "w"), PACE);
    let edges: Vec<(&str, bool)> = strokes.iter().map(|it| (it.key, it.down)).collect();
    assert_eq!(
        edges,
        vec![("shift", true), ("w", true), ("w", false), ("shift", false)]
    );
}

/// Each edge inside the chord waits the hold, and only the last waits the gap
/// before the next chord.
#[test]
fn the_hold_is_inside_and_the_gap_is_after() {
    let strokes = of(&Chord::key("tab"), PACE);
    let waits: Vec<Duration> = strokes.iter().map(|it| it.then).collect();
    assert_eq!(
        waits,
        vec![Duration::from_millis(10), Duration::from_millis(99)]
    );
}

/// The command is QMP's `input-send-event`, one key event, by `qcode`.
#[test]
fn the_command_is_one_key_event_by_qcode() {
    let sent = command(&Stroke {
        key: "ret",
        down: false,
        then: Duration::ZERO,
    });
    assert_eq!(
        sent,
        json!({
            "execute": "input-send-event",
            "arguments": { "events": [{
                "type": "key",
                "data": { "down": false, "key": { "type": "qcode", "data": "ret" } }
            }]}
        })
    );
}
