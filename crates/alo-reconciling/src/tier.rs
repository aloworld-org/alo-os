//! Which release a promise is made for, read from the line it is written on.
//!
//! `docs/features.md` marks every promise with a tier — `[v0.01]`, `[v0.5]`,
//! `[v1]`, `[v2]` — and that marker is the only place a promise says which
//! release it belongs to. This crate reconciled v0.01 alone until 2026-09-27;
//! the v0.5 gate was read by hand instead, seven times over three months, and
//! `ROADMAP.md`'s reconciliation log records what each reading cost.
//!
//! # Why the tier is a type rather than a string a caller passes
//!
//! Because the markers are one character apart. `[v0.5]` and `[v0.01]` differ by
//! a zero, and `ROADMAP.md` has a whole section — *Two numbers that look alike*
//! — about an evening lost to `v0.5` and `0.0.5` meaning different things. A
//! caller that assembled the marker itself would be one typo away from
//! reconciling an empty set and reporting that every promise was answered.
//!
//! [`Tier::marker`] is the only place a marker is written, and
//! [`Tier::EVERY`] is what a test walks to hold each of them to a promise that
//! really exists in the definition.

/// A release a promise can be made for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// It boots and the agent acts.
    V0_01,
    /// A person can work on it all day.
    V0_5,
    /// An organisation can buy it.
    V1,
    /// Carried beyond v1.
    V2,
}

impl Tier {
    /// Every tier `docs/features.md` uses, so a test can hold each marker to
    /// the definition rather than trusting the four strings below.
    pub const EVERY: [Self; 4] = [Self::V0_01, Self::V0_5, Self::V1, Self::V2];

    /// How `docs/features.md` marks a promise at this tier.
    ///
    /// The leading `- ` is part of it, so that the file's own preamble
    /// explaining what the tiers mean is not read as a promise.
    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::V0_01 => "- [v0.01]",
            Self::V0_5 => "- [v0.5]",
            Self::V1 => "- [v1]",
            Self::V2 => "- [v2]",
        }
    }

    /// What this tier is called where a person reads it.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::V0_01 => "v0.01",
            Self::V0_5 => "v0.5",
            Self::V1 => "v1",
            Self::V2 => "v2",
        }
    }

    /// The heading this tier's ledger puts its entries under, and the file the
    /// ledger is.
    ///
    /// A tier without a ledger answers [`None`], which is how a check asked for
    /// one says *there is nothing to read* rather than reading an empty document
    /// and reporting that every promise was answered.
    #[must_use]
    pub const fn ledger(self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::V0_01 => Some((
                "docs/autonomy/v0-01-evidence.md",
                "## Every v0.01 promise, one at a time",
            )),
            Self::V0_5 => Some((
                "docs/autonomy/v0-5-evidence.md",
                "## Every v0.5 promise, one at a time",
            )),
            Self::V1 | Self::V2 => None,
        }
    }

    /// The heading `ROADMAP.md` puts this tier's gate under.
    ///
    /// Only the milestones have one. A promise carried to v2 has no section in
    /// that file, which is why [`Self::V2`] answers [`None`] rather than a
    /// heading nobody wrote — and a check looking for a v2 promise's box would
    /// otherwise report every one of them as missing.
    #[must_use]
    pub const fn heading(self) -> Option<&'static str> {
        match self {
            Self::V0_01 => Some("## v0.01 — it boots and the agent acts"),
            Self::V0_5 => Some("## v0.5 — a person can work on it all day"),
            Self::V1 => Some("## v1 — an organisation can buy it"),
            Self::V2 => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **No two tiers share a marker, and none is a prefix of another.**
    ///
    /// The prefix half is the one that matters: `- [v1]` is not a prefix of
    /// `- [v0.01]`, but a marker written `- [v0` would be a prefix of two and
    /// would read one tier's promises as another's.
    #[test]
    fn no_marker_is_another_markers_prefix() {
        for one in Tier::EVERY {
            for another in Tier::EVERY {
                if one == another {
                    continue;
                }
                assert!(
                    !one.marker().starts_with(another.marker()),
                    "{} starts with {}",
                    one.marker(),
                    another.marker()
                );
            }
        }
    }

    /// **Every tier is named, marked and distinct.**
    #[test]
    fn every_tier_is_named_and_marked() {
        let mut markers = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        for tier in Tier::EVERY {
            assert!(tier.marker().starts_with("- ["));
            assert!(!tier.named().is_empty());
            assert!(markers.insert(tier.marker()));
            assert!(names.insert(tier.named()));
        }
        assert_eq!(markers.len(), Tier::EVERY.len());
    }

    /// **Only the milestones have a gate**, and v2 says so by answering
    /// nothing rather than by naming a heading nobody wrote.
    #[test]
    fn a_tier_that_is_not_a_milestone_has_no_heading() {
        assert!(Tier::V2.heading().is_none());
        for tier in [Tier::V0_01, Tier::V0_5, Tier::V1] {
            assert!(tier.heading().is_some_and(|h| h.starts_with("## ")));
        }
    }
}
