//! Decide one touchpad's event stream. Pinch and swipe commit only at a normal
//! end, so cancellation cannot leave a zoom or desktop change behind. Scroll is
//! continuous. Reset on focus changes, seat pause and device removal.

use crate::Switch;
use crate::gesture_events::{Event, Intent, Kind};
use crate::gesture_settings::{Preferences, Scrolling};

/// Minimum horizontal travel, in libinput's normalized 1000-dpi units.
pub const SWIPE_DISTANCE: f64 = 100.0;

/// An unfinished gesture, bounded in memory regardless of stream length.
#[derive(Debug, Clone, Copy)]
struct Pending {
    /// Family selected at begin.
    kind: Kind,
    /// Last absolute pinch scale.
    scale: f64,
    /// Accumulated horizontal swipe displacement.
    x: f64,
    /// Accumulated vertical swipe displacement.
    y: f64,
}

/// One device's decisions, with no effect until the caller routes an intent.
#[derive(Debug, Default)]
pub struct Gestures {
    /// This person's current preferences.
    preferences: Preferences,
    /// At most one in-flight gesture.
    pending: Option<Pending>,
}

impl Gestures {
    /// Install settings immediately and discard an unfinished gesture. Re-enabling
    /// a gesture cannot resurrect an old begin event.
    pub fn configure(&mut self, preferences: Preferences) {
        self.preferences = preferences;
        self.pending = None;
    }

    /// Decide an extracted event. Disabled, unsupported, malformed, orphaned,
    /// vertical and too-short gestures produce no intent. Malformed sequences
    /// are discarded whole; a fresh begin is required afterwards.
    pub fn decide(&mut self, event: Event) -> Option<Intent> {
        match event {
            Event::Reset => self.pending = None,
            Event::Scroll {
                horizontal,
                vertical,
            } => {
                self.pending = None;
                if !self.preferences.scroll()
                    || horizontal
                        .into_iter()
                        .chain(vertical)
                        .any(|v| !v.is_finite())
                    || (horizontal.is_none() && vertical.is_none())
                {
                    return None;
                }
                let sign = match self.preferences.scrolling() {
                    Scrolling::Natural => -1.0,
                    Scrolling::Traditional => 1.0,
                };
                return Some(Intent::Scroll {
                    horizontal: horizontal.map(|v| v * sign),
                    vertical: vertical.map(|v| v * sign),
                });
            }
            Event::Begin(kind) => {
                // Overlapping begins invalidate both sequences.
                if self.pending.take().is_none() && self.preferences.enabled(kind) {
                    self.pending = Some(Pending {
                        kind,
                        scale: 1.0,
                        x: 0.0,
                        y: 0.0,
                    });
                }
            }
            Event::Pinch(scale) => {
                let mut pending = self.pending.take()?;
                if pending.kind == Kind::Pinch && scale.is_finite() && scale > 0.0 {
                    pending.scale = scale;
                    self.pending = Some(pending);
                }
            }
            Event::Swipe { dx, dy } => {
                let mut pending = self.pending.take()?;
                pending.x += dx;
                pending.y += dy;
                if pending.kind != Kind::Pinch && pending.x.is_finite() && pending.y.is_finite() {
                    self.pending = Some(pending);
                }
            }
            Event::End { kind, cancelled } => {
                let pending = self.pending.take()?;
                if cancelled || pending.kind != kind {
                    return None;
                }
                if kind == Kind::Pinch {
                    return (pending.scale != 1.0).then_some(Intent::Zoom(pending.scale));
                }
                if pending.x.abs() >= SWIPE_DISTANCE && pending.x.abs() > pending.y.abs() * 2.0 {
                    return Some(Intent::Desktop(if pending.x < 0.0 {
                        Switch::Next
                    } else {
                        Switch::Previous
                    }));
                }
            }
        }
        None
    }
}
