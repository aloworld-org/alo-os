//! **What happens when a new window has no room in what the person is looking
//! at.**
//!
//! The owner, 2026-11-07: *"when the user opens the window of the application
//! it should not open on top of the other, it should smooth move the canvas and
//! open on the empty space. Also this can be put in the settings so the user
//! chooses"*, and asked which a new machine starts with, **glide to empty
//! space**. So the shipped answer is [`WhenThereIsNoRoom::GlideToEmptySpace`],
//! and the rule that came before it — open in front, slightly offset from the
//! window being worked in — is kept as [`WhenThereIsNoRoom::OpenInFront`]
//! rather than withdrawn (`CLAUDE.md`, law 5).
//!
//! `docs/design/where-a-new-window-opens.md`, amended 2026-11-07, is the rule.
//! This file only says which of the two a person chose. Written by name in
//! `appearance.toml`, beside [`crate::titles::WindowTitles`], so a file a person
//! edits by hand says what it means.

use serde::{Deserialize, Serialize};

/// Where a new window goes when there is no room for it in the current view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum WhenThereIsNoRoom {
    /// The view glides to the nearest empty space on the Place, and the window
    /// opens there without covering any other: what this release ships.
    #[default]
    GlideToEmptySpace,
    /// The window opens in front of the one being worked in, slightly offset,
    /// and the view stays where it is: the rule before 2026-11-07.
    OpenInFront,
}

impl WhenThereIsNoRoom {
    /// Both, in the order a settings page offers them.
    pub const ALL: [Self; 2] = [Self::GlideToEmptySpace, Self::OpenInFront];

    /// Whether the view may move to find a new window somewhere to be.
    #[must_use]
    pub const fn glides(self) -> bool {
        matches!(self, Self::GlideToEmptySpace)
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
    fn a_machine_nobody_has_changed_glides_to_empty_space() {
        assert_eq!(
            WhenThereIsNoRoom::default(),
            WhenThereIsNoRoom::GlideToEmptySpace
        );
        assert!(WhenThereIsNoRoom::default().glides());
        assert!(!WhenThereIsNoRoom::OpenInFront.glides());
        assert_eq!(
            WhenThereIsNoRoom::ALL,
            [
                WhenThereIsNoRoom::GlideToEmptySpace,
                WhenThereIsNoRoom::OpenInFront
            ]
        );
    }

    #[test]
    fn a_choice_is_written_by_its_name_and_nothing_else_is_read() {
        let written = serde_json::to_string(&WhenThereIsNoRoom::OpenInFront).unwrap();
        assert_eq!(written, r#""OpenInFront""#);
        assert!(serde_json::from_str::<WhenThereIsNoRoom>(r#""Somewhere""#).is_err());
    }
}
