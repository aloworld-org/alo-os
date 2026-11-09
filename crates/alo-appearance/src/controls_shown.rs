//! Whether a window's controls are always on its band, or appear as the
//! pointer comes near.
//!
//! **The owner's ruling of 2026-11-09** (`docs/design/the-window-controls.md`):
//! a window's title and application name are on its band all the time, its
//! three controls appear as the pointer approaches the band, and Settings
//! offers **Always show window controls**. So the shipped answer is
//! [`WindowControls::WhenApproached`].
//!
//! **This replaced a different choice.** From 2026-11-01 to 2026-11-09 the
//! setting here was *show the title only when I point at it*, written in
//! `appearance.toml` as `window_titles`. The owner withdrew it — *"Replace it"* —
//! and `docs/features.md` records the withdrawal. A file a person kept from
//! before still reads: [`WithdrawnTitles`] accepts the old key's two words and
//! nothing is done with them, because the title is now always shown whatever
//! they said, and refusing the whole file over a withdrawn key would take every
//! other choice in it away from them.
//!
//! Written by name in `appearance.toml`, like the accent, so a file a person
//! edits by hand says what it means.

use serde::{Deserialize, Serialize};

/// When a window's controls are drawn on its band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum WindowControls {
    /// As the pointer comes near the band, or the keyboard reaches them: what
    /// this release ships.
    #[default]
    WhenApproached,
    /// On every window, all the time: *Always show window controls*.
    Always,
}

impl WindowControls {
    /// Both, in the order a settings page offers them.
    pub const ALL: [Self; 2] = [Self::WhenApproached, Self::Always];

    /// Whether the controls are drawn whether or not anything points at them.
    #[must_use]
    pub const fn are_always_shown(self) -> bool {
        matches!(self, Self::Always)
    }
}

/// The two words the withdrawn `window_titles` key could hold, read so that a
/// file written before 2026-11-09 still opens, and never written again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum WithdrawnTitles {
    /// The title on every window, all the time — now the only way.
    Always,
    /// The title only while pointed at — the choice the owner withdrew.
    WhenPointedAt,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn a_machine_nobody_has_changed_shows_the_controls_on_approach() {
        assert_eq!(WindowControls::default(), WindowControls::WhenApproached);
        assert!(!WindowControls::default().are_always_shown());
        assert!(WindowControls::Always.are_always_shown());
    }

    #[test]
    fn a_choice_is_written_by_its_name_and_nothing_else_is_read() {
        let written = serde_json::to_string(&WindowControls::Always).unwrap();
        assert_eq!(written, r#""Always""#);
        assert!(serde_json::from_str::<WindowControls>(r#""Sometimes""#).is_err());
    }

    #[test]
    fn the_withdrawn_words_still_read_and_no_others_do() {
        assert_eq!(
            serde_json::from_str::<WithdrawnTitles>(r#""WhenPointedAt""#).unwrap(),
            WithdrawnTitles::WhenPointedAt
        );
        assert_eq!(
            serde_json::from_str::<WithdrawnTitles>(r#""Always""#).unwrap(),
            WithdrawnTitles::Always
        );
        assert!(serde_json::from_str::<WithdrawnTitles>(r#""Sometimes""#).is_err());
    }
}
