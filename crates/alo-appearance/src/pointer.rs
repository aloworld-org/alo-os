//! **What the person's pointer looks like**: its shape, its size and its colour.
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

/// What colour the pointer is.
///
/// **The owner's yes of 2026-11-06** to three high-contrast choices rather than
/// the accents: navy, as it ships; white with a navy edge, for dark backgrounds;
/// and black. *Cursor size and colour* is promised (`docs/features.md`). **Deep
/// teal is never one of them** — it is the agent's (ADR 0067) — and
/// `alo-cursors` holds every colour to that.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PointerColour {
    /// Navy with a white edge: what this release ships.
    #[default]
    Navy,
    /// White with a navy edge, for dark backgrounds.
    White,
    /// Black with a white edge.
    Black,
}

impl PointerColour {
    /// All three, in the order a settings page offers them.
    pub const ALL: [Self; 3] = [Self::Navy, Self::White, Self::Black];
}

/// How large the pointer is drawn, over the size every pointer has.
///
/// *Cursor size* is promised (`docs/features.md`), and the three are the
/// design's own (`551:1136`). **Whole multiples of a half**, so each lands on a
/// size alo's cursor theme already carries: 24, 36 and 48 logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PointerSize {
    /// As it ships.
    #[default]
    Default,
    /// Half as large again.
    Large,
    /// Twice as large.
    Larger,
}

impl PointerSize {
    /// All three, in the order the frame draws them.
    pub const ALL: [Self; 3] = [Self::Default, Self::Large, Self::Larger];

    /// How large, as a per cent of the size it ships at.
    #[must_use]
    pub const fn per_cent(self) -> u16 {
        match self {
            Self::Default => 100,
            Self::Large => 150,
            Self::Larger => 200,
        }
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
    fn a_machine_nobody_has_changed_shows_the_kite() {
        assert_eq!(PointerShape::default(), PointerShape::Kite);
        assert_eq!(
            PointerShape::ALL,
            [PointerShape::Kite, PointerShape::Classic]
        );
    }

    #[test]
    fn a_choice_is_written_by_its_name_and_nothing_else_is_read() {
        let written = serde_json::to_string(&PointerShape::Classic).unwrap();
        assert_eq!(written, r#""Classic""#);
        assert!(serde_json::from_str::<PointerShape>(r#""Pointy""#).is_err());
    }

    #[test]
    fn the_shipped_colour_is_navy_and_a_colour_is_written_by_its_name() {
        assert_eq!(PointerColour::default(), PointerColour::Navy);
        assert_eq!(
            serde_json::to_string(&PointerColour::White).unwrap(),
            r#""White""#
        );
        assert!(serde_json::from_str::<PointerColour>(r#""Teal""#).is_err());
    }

    #[test]
    fn each_size_is_larger_than_the_last_and_the_default_is_as_shipped() {
        assert_eq!(PointerSize::default(), PointerSize::Default);
        assert_eq!(PointerSize::ALL.map(PointerSize::per_cent), [100, 150, 200]);
        assert_eq!(
            serde_json::to_string(&PointerSize::Larger).unwrap(),
            r#""Larger""#
        );
        assert!(serde_json::from_str::<PointerSize>(r#""Huge""#).is_err());
    }
}
