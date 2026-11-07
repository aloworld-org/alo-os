//! **What the person's pointer looks like.**
//!
//! The owner chose the precision kite on 2026-11-06, in one word: *"B"*. So the
//! shipped answer is [`PointerShape::Kite`], and the familiar arrow is kept as
//! [`PointerShape::Classic`] — *the familiar arrow stays a choice in Settings for
//! anybody who prefers it* (`docs/features.md`, *The alo Cursor*). A person who
//! knows the old arrow by heart is not made to learn a new one (`CLAUDE.md`,
//! law 5).
//!
//! Written by name in `appearance.toml`, like the accent, so a file a person
//! edits by hand says what it means. The drawing itself is `alo-cursors`'.

use serde::{Deserialize, Serialize};

/// Which arrow the pointer is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PointerShape {
    /// The precision kite: what this release ships.
    #[default]
    Kite,
    /// The familiar arrow.
    Classic,
}

impl PointerShape {
    /// Both, in the order a settings page offers them.
    pub const ALL: [Self; 2] = [Self::Kite, Self::Classic];
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn a_machine_nobody_has_changed_shows_the_kite() {
        assert_eq!(PointerShape::default(), PointerShape::Kite);
        assert_eq!(PointerShape::ALL, [PointerShape::Kite, PointerShape::Classic]);
    }

    #[test]
    fn a_choice_is_written_by_its_name_and_nothing_else_is_read() {
        let written = serde_json::to_string(&PointerShape::Classic).unwrap();
        assert_eq!(written, r#""Classic""#);
        assert!(serde_json::from_str::<PointerShape>(r#""Pointy""#).is_err());
    }
}
