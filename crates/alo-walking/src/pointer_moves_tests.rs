//! A place on the screen as qemu's absolute pointer takes it.

use serde_json::json;

use super::{ABS_MOST, absolute, button, command};

/// **The first pixel is nought, the last is the most, and the middle is the
/// middle**, on screens of different sizes.
#[test]
fn the_ends_and_the_middle_map_onto_qemus_range() {
    for extent in [800_usize, 1280, 1920, 3] {
        assert_eq!(absolute(0, extent), 0);
        assert_eq!(absolute(extent - 1, extent), ABS_MOST);
        let middle = absolute((extent - 1) / 2, extent);
        let one_pixel = ABS_MOST / u32::try_from(extent).unwrap_or(1);
        assert!(
            middle.abs_diff(ABS_MOST / 2) <= one_pixel,
            "{extent}: {middle}"
        );
    }
}

/// **A place past the edge is the edge**, never past qemu's range.
#[test]
fn past_the_edge_is_the_edge() {
    assert_eq!(absolute(5000, 1280), ABS_MOST);
    assert!(absolute(1, 1) <= ABS_MOST);
}

/// **One command carries both axes**, absolute.
#[test]
fn a_place_is_one_absolute_command() {
    assert_eq!(
        command((0, 799), (1280, 800)),
        json!({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    { "type": "abs", "data": { "axis": "x", "value": 0 } },
                    { "type": "abs", "data": { "axis": "y", "value": ABS_MOST } }
                ]
            }
        })
    );
}

/// **A click is the left button down, then up**, each its own command.
#[test]
fn a_click_is_the_left_button_down_then_up() {
    for down in [true, false] {
        assert_eq!(
            button(down),
            json!({
                "execute": "input-send-event",
                "arguments": {
                    "events": [ { "type": "btn", "data": { "down": down, "button": "left" } } ]
                }
            })
        );
    }
}
