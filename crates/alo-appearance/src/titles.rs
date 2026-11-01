//! Whether a window's title is always on its edge, or only while a person
//! points at it.
//!
//! **The owner's answer of 2026-11-01**: *window titles are always visible on
//! every window's edge, with the app's icon; Settings offers "show the title
//! only when I point at it".* So the shipped answer is [`WindowTitles::Always`],
//! and the other is the edge as it was drawn before that day — concealed at
//! rest, revealed under the pointer or the keyboard — kept as a choice rather
//! than withdrawn, because a person who preferred the quieter edge was promised
//! it (`CLAUDE.md`, law 5).
//!
//! Written by name in `appearance.toml`, like the accent, so a file a person
//! edits by hand says what it means.

use serde::{Deserialize, Serialize};

/// When a window's title strip is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum WindowTitles {
    /// On every window, all the time: what this release ships.
    #[default]
    Always,
    /// Only while the pointer or the keyboard is on the window's edge.
    WhenPointedAt,
}

impl WindowTitles {
    /// Both, in the order a settings page offers them.
    pub const ALL: [Self; 2] = [Self::Always, Self::WhenPointedAt];

    /// Whether the edge is drawn whether or not anything points at it.
    #[must_use]
    pub const fn are_always_shown(self) -> bool {
        matches!(self, Self::Always)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn a_machine_nobody_has_changed_shows_every_title() {
        assert_eq!(WindowTitles::default(), WindowTitles::Always);
        assert!(WindowTitles::default().are_always_shown());
        assert!(!WindowTitles::WhenPointedAt.are_always_shown());
    }

    #[test]
    fn a_choice_is_written_by_its_name_and_nothing_else_is_read() {
        let written = serde_json::to_string(&WindowTitles::WhenPointedAt).unwrap();
        assert_eq!(written, r#""WhenPointedAt""#);
        assert!(serde_json::from_str::<WindowTitles>(r#""Sometimes""#).is_err());
    }
}
