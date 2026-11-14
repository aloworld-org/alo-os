//! **A chord as the presses and releases that make it**, with the time held
//! between each, and the QMP command that sends one of them.
//!
//! qemu's monitor `sendkey` presses a whole chord in one command and lets go
//! of it after a fixed hold. Sent quickly, under the lag of a slow machine,
//! shifted characters arrived garbled: the release of Shift and the press of
//! the letter raced. QMP's `input-send-event` takes **one edge of one key at a
//! time**, so here every modifier goes down first, then the key, then the key
//! comes up, then the modifiers in reverse — each its own command, each
//! followed by a wait the walk chose. Nothing about the order is left to qemu.

use std::time::Duration;

use serde_json::{Value, json};

use crate::keys::Chord;

/// How long a walk holds and waits between keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
    /// After each edge inside a chord: a modifier down, the key down, the key
    /// up.
    pub hold: Duration,
    /// After the whole chord, before the next one.
    pub between: Duration,
}

impl Pace {
    /// The pace for a machine under KVM: a key held for 60 ms, as a person's
    /// is, and 150 ms before the next.
    pub const UNDER_KVM: Self = Self {
        hold: Duration::from_millis(60),
        between: Duration::from_millis(150),
    };
}

/// One edge of one key: it goes down, or it comes up — then a wait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stroke {
    /// The key, as a `qcode`.
    pub key: &'static str,
    /// Down, or up.
    pub down: bool,
    /// How long to wait once it is sent.
    pub then: Duration,
}

/// The strokes that press one chord.
#[must_use]
pub fn of(chord: &Chord, pace: Pace) -> Vec<Stroke> {
    let mut strokes = Vec::new();
    for &modifier in chord.held() {
        strokes.push(Stroke {
            key: modifier,
            down: true,
            then: pace.hold,
        });
    }
    strokes.push(Stroke {
        key: chord.the_key(),
        down: true,
        then: pace.hold,
    });
    strokes.push(Stroke {
        key: chord.the_key(),
        down: false,
        then: pace.hold,
    });
    for &modifier in chord.held().iter().rev() {
        strokes.push(Stroke {
            key: modifier,
            down: false,
            then: pace.hold,
        });
    }
    if let Some(last) = strokes.last_mut() {
        last.then = pace.between;
    }
    strokes
}

/// The QMP command that sends one stroke.
#[must_use]
pub fn command(stroke: &Stroke) -> Value {
    json!({
        "execute": "input-send-event",
        "arguments": {
            "events": [{
                "type": "key",
                "data": {
                    "down": stroke.down,
                    "key": { "type": "qcode", "data": stroke.key }
                }
            }]
        }
    })
}

#[cfg(test)]
#[path = "strokes_tests.rs"]
mod tests;
