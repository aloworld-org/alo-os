//! Owned fields borrowed from the input library; no device or agent authority.

use crate::Switch;

/// An event from one touchpad. Keep a separate recognizer per device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// Modern finger scroll, with absent axes distinct from stop (zero).
    /// Values are in traditional direction, before this crate's preference.
    Scroll {
        /// Horizontal delta, or no horizontal report.
        horizontal: Option<f64>,
        /// Vertical delta, or no vertical report.
        vertical: Option<f64>,
    },
    /// Start a pinch or swipe.
    Begin(Kind),
    /// Absolute scale since pinch begin, not a delta.
    Pinch(f64),
    /// Unaccelerated normalized swipe displacement since the last event.
    Swipe {
        /// Horizontal displacement.
        dx: f64,
        /// Vertical displacement.
        dy: f64,
    },
    /// Finish the named sequence; cancellation never commits it.
    End {
        /// The sequence that ended.
        kind: Kind,
        /// Whether the library cancelled recognition.
        cancelled: bool,
    },
    /// Device removal, seat pause, focus change or unsupported gesture.
    Reset,
}

/// Gesture families the person can independently disable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Two-finger pinch.
    Pinch,
    /// Three-finger swipe.
    ThreeFingerSwipe,
    /// Four-finger swipe.
    FourFingerSwipe,
}

impl Kind {
    /// Decide the library's swipe finger count; all other counts are unsupported.
    pub const fn swipe(fingers: i32) -> Option<Self> {
        match fingers {
            3 => Some(Self::ThreeFingerSwipe),
            4 => Some(Self::FourFingerSwipe),
            _ => None,
        }
    }

    /// Decide the library's pinch finger count; only two-finger zoom is offered.
    pub const fn pinch(fingers: i32) -> Option<Self> {
        if fingers == 2 {
            Some(Self::Pinch)
        } else {
            None
        }
    }
}

impl Event {
    /// Normalize library scroll fields to traditional direction before the
    /// person's preference is applied. This prevents double natural inversion
    /// when the upstream device already has natural scrolling enabled.
    pub fn finger_scroll(
        horizontal: Option<f64>,
        vertical: Option<f64>,
        upstream_natural: bool,
    ) -> Self {
        let sign = if upstream_natural { -1.0 } else { 1.0 };
        Self::Scroll {
            horizontal: horizontal.map(|value| value * sign),
            vertical: vertical.map(|value| value * sign),
        }
    }
}

/// The complete set of effects a gesture may request. No agent invocation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intent {
    /// Scroll the focused application; zero stops the reported axis.
    Scroll {
        /// Horizontal delta or no report.
        horizontal: Option<f64>,
        /// Vertical delta or no report.
        vertical: Option<f64>,
    },
    /// Multiply the application's current zoom by this positive factor.
    Zoom(f64),
    /// Pass through the ordinary desktop switching road, including edge refusal.
    Desktop(Switch),
}
