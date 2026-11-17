//! What the market says about an application beside its name, when there is
//! something a person choosing it should know.
//!
//! The owner decided on 2026-11-17 that the market lists every kind of
//! application, and three kinds are listed **with a label in plain words on
//! their row**:
//!
//! - *"Proprietary apps are listed, labelled 'not open source'."*
//! - *"Flathub-unverified apps are listed, labelled 'not verified'"* — the
//!   maker did not publish it on Flathub themselves.
//! - The Craft applications are *"added, labelled 'early version'."*
//!
//! # A label is never a warning to dismiss
//!
//! Each says a fact and what it means, and nothing about whether to install:
//! the person chooses (law 5). There is no label meaning *recommended*, and no
//! label that hides an application or asks twice.
//!
//! # Read off the entry, never written beside it
//!
//! [`Label::NotOpenSource`] comes from the licence as `crate::fields::Licence`
//! reads it, so a licence and its label cannot disagree. The other two are the
//! catalogue's own fields (`verified` and `early`), which every entry has to
//! write, so a forgotten one is a refused file rather than a missing label.

use crate::words::{self, Word};

/// One thing the market says about an application beside its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Label {
    /// Its makers do not publish the code it is built from.
    NotOpenSource,
    /// It was not published at its source by the people who make it.
    NotVerified,
    /// It is new and still changing.
    EarlyVersion,
}

impl Label {
    /// Every label, in the order a row shows them.
    pub const EVERY: [Self; 3] = [Self::NotOpenSource, Self::NotVerified, Self::EarlyVersion];

    /// The short words drawn on the row.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::NotOpenSource => words::NOT_OPEN_SOURCE,
            Self::NotVerified => words::NOT_VERIFIED_BY_ITS_MAKER,
            Self::EarlyVersion => words::EARLY_VERSION,
        }
    }

    /// The whole sentence, for a screen reader and for anybody who asks what
    /// the label means.
    #[must_use]
    pub const fn said_in_full(self) -> Word {
        match self {
            Self::NotOpenSource => words::NOT_OPEN_SOURCE_IN_FULL,
            Self::NotVerified => words::NOT_VERIFIED_BY_ITS_MAKER_IN_FULL,
            Self::EarlyVersion => words::EARLY_VERSION_IN_FULL,
        }
    }

    /// The label as the wire and the record name it — what alo is told when it
    /// looks at what can be installed.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::NotOpenSource => "not_open_source",
            Self::NotVerified => "not_verified",
            Self::EarlyVersion => "early_version",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every label has words of its own, short and in full, and a name.
    #[test]
    fn every_label_has_its_own_words_and_name() {
        let mut seen = Vec::new();
        for label in Label::EVERY {
            for key in [
                label.word().named().to_owned(),
                label.said_in_full().named().to_owned(),
                label.named().to_owned(),
            ] {
                assert!(!seen.contains(&key), "{key} is said twice");
                seen.push(key);
            }
        }
        assert_eq!(seen.len(), 9, "the loop read every label");
    }

    /// **A label's short words say the fact in the owner's words**, in English
    /// at least — the sentence the decision named.
    #[test]
    fn the_short_words_are_the_owners_words() {
        assert_eq!(Label::NotOpenSource.word().says(), "Not open source");
        assert_eq!(Label::NotVerified.word().says(), "Not verified");
        assert_eq!(Label::EarlyVersion.word().says(), "Early version");
    }
}
