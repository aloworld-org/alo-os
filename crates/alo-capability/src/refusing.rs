//! Why the grants refused something, and what a person is told.
//!
//! [`Grants::permitting`](crate::Grants::permitting) answers with the grant
//! that said yes or with one of these. It is a **value rather than a sentence**,
//! and that is the decision this file exists for.
//!
//! # A refusal is decided without words and worded afterwards
//!
//! Item 9b put the words of a refusal where the refusal was made: `alo-files`
//! renders its own, in the reader's language, and hands the text to
//! [`crate::Refused`] so that what a person was told is what the record keeps.
//! The rule was right and the shape it took would not survive this crate. To
//! word a refusal here, [`crate::Grants`] would have to be handed a
//! `Strings` — and then *deciding whether an agent may touch a folder* would
//! depend on somebody having loaded a vocabulary. That is a dependency the
//! deciding crate must not have.
//!
//! So the refusal is a value that carries what it refused, and [`NotGranted::said`]
//! renders it when somebody shows it or writes it down. The guarantee 9b was
//! after is kept in the stronger form: the screen and the record render **the
//! same value**, so they cannot be two different accounts of one moment — one
//! of them cannot be English while the other is German, because neither is a
//! language until it is asked for.
//!
//! # The two halves say different things on purpose
//!
//! *It expired* and *you never granted it* need different things from the
//! person reading them. The first is fixed by granting again; the second is
//! not, and a message that covered both would tell somebody to check something
//! they already know.

use alo_strings::{Filling, Said, Strings};

use crate::reach::{Ask, Reach};
use crate::words;

/// Why no grant permitted something.
///
/// Carries what was asked for and, where there was one, the grant that has run
/// out — so the sentence can name the folder a person would go and grant again
/// rather than telling them that something, somewhere, was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotGranted {
    /// A grant covered this and has expired.
    Lapsed {
        /// The agent that asked.
        agent: String,
        /// What the expired grant was over.
        reach: Reach,
        /// What was asked for.
        wanted: Ask,
    },
    /// Nothing this agent holds has ever covered it.
    Never {
        /// The agent that asked.
        agent: String,
        /// What was asked for.
        wanted: Ask,
    },
}

impl NotGranted {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::Lapsed { .. } => words::HAS_EXPIRED,
            Self::Never { .. } => words::NEVER_GRANTED,
        }
    }

    /// What was asked for, whichever refusal this is.
    #[must_use]
    pub fn wanted(&self) -> &Ask {
        match self {
            Self::Lapsed { wanted, .. } | Self::Never { wanted, .. } => wanted,
        }
    }

    /// The agent that asked.
    #[must_use]
    pub fn agent(&self) -> &str {
        match self {
            Self::Lapsed { agent, .. } | Self::Never { agent, .. } => agent,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// What was asked for and what the expired grant was over both go in as
    /// clauses rather than as text ([`Ask::fills`] and [`Reach::said`]), so
    /// this refusal is only as translated as the pieces named inside it. The
    /// agent's name and the path are the machine's and stay as they are, which
    /// is exactly the distinction a gap holding text alone could not make.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::capability_words`] answers with
    /// the key, marked, and `Said::is_a_bug`. **What is refused never depends
    /// on the string table** — the refusal was decided before this method was
    /// called, and calling it cannot change the answer.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = self.wanted().fills(
            "wanted",
            Filling::of("agent", self.agent().to_owned()),
            strings,
        );
        let filling = match self {
            Self::Lapsed { reach, .. } => filling.and_said("reach", &reach.said(strings)),
            Self::Never { .. } => filling,
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use std::path::PathBuf;

    fn never() -> NotGranted {
        NotGranted::Never {
            agent: "@files".to_owned(),
            wanted: Ask::path("/home/anna/Taxes/2024.pdf"),
        }
    }

    fn lapsed() -> NotGranted {
        NotGranted::Lapsed {
            agent: "@files".to_owned(),
            reach: Reach::Folder(PathBuf::from("/home/anna/Invoices")),
            wanted: Ask::path("/home/anna/Invoices/march.pdf"),
        }
    }

    /// The refusal says which of the two it was, names the agent, and names
    /// what was asked for — everything a person needs to decide what to do.
    #[test]
    fn a_refusal_says_which_of_the_two_reasons_it_was() {
        let strings = in_english();
        let never = never().said(&strings);
        assert!(never.text().contains("has not been granted"), "{never}");
        assert!(
            never.text().contains("/home/anna/Taxes/2024.pdf"),
            "{never}"
        );
        assert!(never.text().contains("@files"), "{never}");

        let lapsed = lapsed().said(&strings);
        assert!(lapsed.text().contains("has expired"), "{lapsed}");
        assert!(lapsed.text().contains("/home/anna/Invoices"), "{lapsed}");
    }

    /// **A refusal and the thing named inside it are in one language.** The
    /// grant that expired is described by this crate too, so a German machine
    /// does not read a German sentence with an English clause in the middle of
    /// it.
    #[test]
    fn a_refusal_and_what_it_names_are_in_one_language() {
        let strings = translated(&[
            (
                words::HAS_EXPIRED,
                "{agent} durfte {reach} erreichen, und das ist abgelaufen — erteilen Sie es \
                 erneut, damit {agent} {wanted} erreichen kann",
            ),
            (words::A_FOLDER, "{path} und alles darin"),
        ]);
        let said = lapsed().said(&strings);
        assert!(said.is_translated());
        assert!(said.text().contains("und alles darin"), "{said}");
        assert!(!said.text().contains("everything in it"), "{said}");
        // The agent's name and the path are the machine's, not the language's.
        assert!(said.text().contains("@files"), "{said}");
        assert!(
            said.text().contains("/home/anna/Invoices/march.pdf"),
            "{said}"
        );
    }

    /// **A refusal is only as translated as the clause inside it.** A German
    /// sentence naming the grant in English is a line half its reader cannot
    /// read, and the half they cannot read is the one saying which grant to go
    /// and make again.
    #[test]
    fn a_refusal_naming_an_untranslated_grant_does_not_claim_to_be_translated() {
        let half = translated(&[(
            words::HAS_EXPIRED,
            "{agent} durfte {reach} erreichen, und das ist abgelaufen — erteilen Sie es erneut, \
             damit {agent} {wanted} erreichen kann",
        )]);
        let said = lapsed().said(&half);
        assert!(!said.is_translated(), "{said}");
        assert!(said.text().starts_with("@files durfte"), "{said}");
        assert!(said.text().contains("and everything in it"), "{said}");
    }

    /// **A path is data, and data cannot make a refusal untranslated.** Nobody
    /// translates `/home/anna/Taxes/2024.pdf`, so a German refusal naming one
    /// is a German refusal — and a rule that said otherwise would put a release
    /// note's count out by the number of files anybody happened to ask about.
    #[test]
    fn a_refusal_naming_a_path_is_as_translated_as_it_reads() {
        let strings = translated(&[(
            words::NEVER_GRANTED,
            "{agent} hat keine Berechtigung für {wanted} — erteilen Sie eine, wenn das gewollt ist",
        )]);
        let said = never().said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("/home/anna/Taxes/2024.pdf"), "{said}");
    }

    /// **A refusal never depends on a string table.** With no words at all it
    /// refuses exactly as firmly and answers with the key, marked, so whoever
    /// forgot to declare this crate's words finds out from the answer rather
    /// than from a blank line.
    #[test]
    fn a_refusal_without_the_words_still_names_the_rule() {
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let said = never().said(&strings);
        assert!(said.is_a_bug());
        assert!(
            said.text().contains("capability.refused.never-granted"),
            "{said}"
        );
    }
}
