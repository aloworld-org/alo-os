//! Gesture preferences, kept at the path supplied by the session (ADR 0038).
//! Only differences are written. Corrupt files are refused and preserved by
//! `alo_kept`; nothing here reads an environment or watches a file.

use serde::{Deserialize, Serialize};

use crate::gesture_events::Kind;

/// Direction of touchpad scrolling; independent of desktop swipe direction.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scrolling {
    /// Content follows the fingers.
    #[default]
    Natural,
    /// Positive upstream deltas move down or right through the content.
    Traditional,
}

/// Whether a switch has its shipped, enabled value.
fn enabled(value: &bool) -> bool {
    *value
}

/// Whether scrolling has its shipped direction.
fn natural(value: &Scrolling) -> bool {
    *value == Scrolling::Natural
}

/// The person's gesture settings. All gestures ship enabled with natural scroll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct Preferences {
    /// Enable touchpad scrolling.
    #[serde(skip_serializing_if = "enabled")]
    pub scroll: bool,
    /// Enable two-finger pinch zoom.
    #[serde(skip_serializing_if = "enabled")]
    pub pinch: bool,
    /// Enable three-finger desktop swipes.
    #[serde(skip_serializing_if = "enabled")]
    pub three_finger_swipe: bool,
    /// Enable four-finger desktop swipes.
    #[serde(skip_serializing_if = "enabled")]
    pub four_finger_swipe: bool,
    /// Natural or traditional scrolling.
    #[serde(skip_serializing_if = "natural")]
    pub scrolling: Scrolling,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            scroll: true,
            pinch: true,
            three_finger_swipe: true,
            four_finger_swipe: true,
            scrolling: Scrolling::Natural,
        }
    }
}

impl Preferences {
    /// Whether this gesture family is enabled.
    pub const fn enabled(self, kind: Kind) -> bool {
        match kind {
            Kind::Pinch => self.pinch,
            Kind::ThreeFingerSwipe => self.three_finger_swipe,
            Kind::FourFingerSwipe => self.four_finger_swipe,
        }
    }

    /// Whether finger scrolling is enabled.
    pub const fn scroll(self) -> bool {
        self.scroll
    }

    /// The current scrolling direction.
    pub const fn scrolling(self) -> Scrolling {
        self.scrolling
    }
}
