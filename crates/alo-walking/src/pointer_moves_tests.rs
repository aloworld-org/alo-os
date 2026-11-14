//! A move cut into steps qemu's PS/2 mouse carries whole.

use serde_json::json;

use super::{LARGEST_STEP, command, in_steps};

/// **Every step is inside the largest, and the steps add up to the move** —
/// for moves either way, on either axis, and much longer on one than the other.
#[test]
fn the_steps_add_up_to_the_move_and_none_is_too_long() {
    for (dx, dy) in [
        (0, 1),
        (1, 0),
        (100, 100),
        (101, -3),
        (-640, 400),
        (7, -1999),
        (-1, -1),
    ] {
        let steps = in_steps(dx, dy);
        assert!(!steps.is_empty(), "({dx}, {dy}) made no steps");
        for &(x, y) in &steps {
            assert!(
                x.abs() <= LARGEST_STEP && y.abs() <= LARGEST_STEP,
                "({dx}, {dy}) has a step ({x}, {y})"
            );
        }
        let summed = steps
            .iter()
            .fold((0, 0), |(x, y), &(sx, sy)| (x + sx, y + sy));
        assert_eq!(summed, (dx, dy));
    }
}

/// **No move is no command at all**, rather than a command of nothing.
#[test]
fn no_move_is_no_steps() {
    assert!(in_steps(0, 0).is_empty());
}

/// **A step is one command carrying both axes**, relative, which is what a
/// PS/2 mouse takes.
#[test]
fn a_step_is_one_relative_command() {
    assert_eq!(
        command(12, -7),
        json!({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    { "type": "rel", "data": { "axis": "x", "value": 12 } },
                    { "type": "rel", "data": { "axis": "y", "value": -7 } }
                ]
            }
        })
    );
}
