//! An application asked for the person to choose what opens a link, and
//! nothing was chosen — the *Open with* sheet the owner approved on 2026-11-12
//! (`docs/design/default-apps-and-open-with.md`, frame `25:52`), answered
//! without a choice.
//!
//! **Chosen or cancelled, the link is in the record** — the frame's own note.
//! This is the *cancelled* half and its three siblings: nobody answered, the
//! desktop could not show the sheet, or there was no desktop to ask. Nothing
//! opens in any of them, and **the default is never used instead**: an
//! application that asked for the person's choice is not answered with one
//! nobody made.
//!
//! **Only the person cancelling is response `1`.** The other four are `2`,
//! because the person did not say no.

use alo_applications::Link;
use alo_capability::Applicant;
use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words;

/// Why nothing was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotChosen {
    /// The person pressed *Cancel* or Escape.
    Cancelled,
    /// Nobody answered while the sheet was up, and the desktop took it down.
    Unanswered,
    /// The desktop could not show the sheet now.
    CannotShow,
    /// No desktop was there to ask.
    NoDesktop,
    /// The desktop was reached and could not be asked, or answered with
    /// something that is not one of the applications offered.
    CouldNotAsk,
}

impl NotChosen {
    /// The response it is sent as: `1` for the person's own cancel, `2`
    /// otherwise.
    #[must_use]
    pub const fn response(self) -> u32 {
        match self {
            Self::Cancelled => 1,
            Self::Unanswered | Self::CannotShow | Self::NoDesktop | Self::CouldNotAsk => 2,
        }
    }

    /// The sentence for it.
    const fn word(self) -> alo_strings::Word {
        match self {
            Self::Cancelled => words::LINK_NOT_CHOSEN_CANCELLED,
            Self::Unanswered => words::LINK_NOT_CHOSEN_UNANSWERED,
            Self::CannotShow => words::LINK_NOT_CHOSEN_CANNOT_SHOW,
            Self::NoDesktop => words::LINK_NOT_CHOSEN_NO_DESKTOP,
            Self::CouldNotAsk => words::LINK_NOT_CHOSEN_COULD_NOT_ASK,
        }
    }
}

/// A link nobody chose for, who asked, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkNotChosen {
    /// The application that asked.
    from: Applicant,
    /// The link.
    link: Link,
    /// Why nothing was chosen.
    why: NotChosen,
}

impl LinkNotChosen {
    /// `from` asked for the person to choose what opens `link`, and nothing
    /// was chosen, for `why`.
    #[must_use]
    pub const fn of(from: Applicant, link: Link, why: NotChosen) -> Self {
        Self { from, link, why }
    }

    /// The application that asked.
    #[must_use]
    pub const fn from(&self) -> &Applicant {
        &self.from
    }

    /// The link.
    #[must_use]
    pub const fn link(&self) -> &Link {
        &self.link
    }

    /// Why nothing was chosen.
    #[must_use]
    pub const fn why(&self) -> NotChosen {
        self.why
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        said_as_not_chosen(
            Filling::of("application", self.from.as_str()),
            self.link.as_str(),
            self.why,
            strings,
        )
    }
}

/// The sentence for a link nobody chose for, from what was decided or from
/// what the record kept — one function, so the two cannot be worded apart.
pub(crate) fn said_as_not_chosen(
    asked: Filling,
    address: &str,
    why: NotChosen,
    strings: &Strings,
) -> Said {
    strings.say(&why.word().key(), &asked.and("address", address))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_strings::Vocabulary;

    fn in_english() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        crate::declare_into(&mut vocabulary).unwrap();
        alo_applications::words::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **Only the person's cancel is `1`**, and each of the five says
    /// something different, naming who asked and the address.
    #[test]
    fn only_the_persons_cancel_is_one_and_each_says_why() {
        let strings = in_english();
        let mut seen = Vec::new();
        for why in [
            NotChosen::Cancelled,
            NotChosen::Unanswered,
            NotChosen::CannotShow,
            NotChosen::NoDesktop,
            NotChosen::CouldNotAsk,
        ] {
            assert_eq!(why.response(), u32::from(why != NotChosen::Cancelled) + 1);
            let not = LinkNotChosen::of(
                Applicant::named("org.gnome.Papers"),
                Link::checked("https://example.com/invoice").unwrap(),
                why,
            );
            let said = not.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            for named in ["org.gnome.Papers", "https://example.com/invoice"] {
                assert!(said.text().contains(named), "{said}");
            }
            seen.push(said.text().to_owned());
        }
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 5, "two reasons said the same thing");
    }

    /// **The file keeps the reason by its spelling.**
    #[test]
    fn the_reason_is_kept_by_its_spelling() {
        assert_eq!(
            serde_json::to_string(&NotChosen::CannotShow).unwrap(),
            r#""cannot-show""#
        );
    }
}
