//! Why a message is not one this side can act on, and what whoever sent it is
//! told.
//!
//! Item 21's sentence is *a malformed request is refused in the reader's own
//! language, not dropped*, and this is both halves of it. A privileged service
//! that answers a message it could not read with silence is a service nobody
//! can tell apart from one that has stopped — and one that answers in English
//! is one whose diagnosis is unreadable to the person whose machine it is.
//!
//! # One list, both directions
//!
//! Five of these are about the **envelope** — how long, how many, which format,
//! and whether it could be read at all — and an envelope is an envelope
//! whichever way it is going. Two are about a request arriving on the wrong
//! door, and two about an answer arriving on the wrong one. Splitting them into
//! a type per direction would have meant writing the five shared ones twice, in
//! two places that then have to agree; the 9-series spent six items removing
//! exactly that shape.
//!
//! # A refusal is a value, and it is worded when somebody shows it
//!
//! Item 9e's decision, in the crate furthest out from the model: what a message
//! *is* cannot depend on a vocabulary having loaded, so reading one asks for no
//! [`Strings`] at all and [`NotUnderstood::said`] renders the answer where it
//! is read. A daemon whose protocol stopped working because a translation
//! failed to load would be a daemon nothing else on the machine could reach.
//!
//! # Nothing here quotes the message back
//!
//! What arrived is bytes nobody has checked. Every sentence in
//! [`crate::words`] is gapless, so there is no road for that text into a
//! sentence a person reads — `alo-record`'s *the arguments of a call that never
//! validated are never kept*, met one step earlier, before there is a call to
//! keep anything about.
//!
//! The two numbers a reader might still want are carried as fields:
//! [`NotUnderstood::TooLong`] says how long the message was and how long is
//! allowed, and the two format refusals say what number the message claimed.
//! Both are for whoever draws them beside the sentence, and neither is inside
//! it — item 9f's rule, so that no language has to invent a plural for a
//! quantity English happened to put in a sentence.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why a message is not a request.
///
/// Deliberately **not** a `std::error::Error` and with no `Display`, which is
/// item 9b's rule: a `Display` is one `to_string()` from a screen whose author
/// had no reason to think about language. The only road to words is
/// [`NotUnderstood::said`], and every answer it gives says whether anybody
/// translated it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotUnderstood {
    /// Longer than this machine will hold.
    TooLong {
        /// The most a message may be, in bytes.
        most: usize,
        /// How many bytes arrived.
        was: usize,
    },
    /// More than one message where one was expected.
    MoreThanOneMessage,
    /// A format number this alo OS has never written, and higher than the one
    /// it does.
    FromANewerAloOs {
        /// The number the message claimed.
        format: u32,
    },
    /// A format number no alo OS has ever written.
    NotAFormat {
        /// The number the message claimed.
        format: u32,
    },
    /// Not a message of this shape at all.
    NotReadable,
    /// A request only a person makes, arriving on the agent's side.
    NotForAnAgent,
    /// A request only an agent makes, arriving on the person's side.
    NotForAPerson,
    /// An answer only a person's screen is given, arriving on the agent's side.
    NotAnAnswerForAnAgent,
    /// An answer only an agent is given, arriving on the person's side.
    NotAnAnswerForAPerson,
}

impl NotUnderstood {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::TooLong { .. } => words::TOO_LONG,
            Self::MoreThanOneMessage => words::MORE_THAN_ONE_MESSAGE,
            Self::FromANewerAloOs { .. } => words::FROM_A_NEWER_ALO_OS,
            Self::NotAFormat { .. } => words::NOT_A_FORMAT,
            Self::NotReadable => words::NOT_READABLE,
            Self::NotForAnAgent => words::NOT_FOR_AN_AGENT,
            Self::NotForAPerson => words::NOT_FOR_A_PERSON,
            Self::NotAnAnswerForAnAgent => words::NOT_AN_ANSWER_FOR_AN_AGENT,
            Self::NotAnAnswerForAPerson => words::NOT_AN_ANSWER_FOR_A_PERSON,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::protocol_words`] answers with
    /// the key, marked, and `Said::is_a_bug`. **What is refused never depends
    /// on the string table** — it was decided before this was called, and
    /// calling it cannot change the answer.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }

    /// Whether this refusal is about who asked rather than about the message.
    ///
    /// The two that answer `true` are the ones a daemon has reason to treat
    /// differently from a client that cannot spell: a request arriving on the
    /// wrong side of the socket is well-formed, was understood, and was still
    /// refused — which is the thing worth writing into a log of its own.
    #[must_use]
    pub fn is_about_who_asked(&self) -> bool {
        matches!(self, Self::NotForAnAgent | Self::NotForAPerson)
    }

    /// Whether this is about who an **answer** was for rather than about the
    /// message.
    ///
    /// The twin of [`NotUnderstood::is_about_who_asked`], on the way back. A
    /// separate question because the two mean different things about the
    /// machine: a request on the wrong door is a client asking for something it
    /// may not have, and an answer on the wrong door is a **daemon** that put
    /// one side's answer on the other side's connection — which is a bug in alo
    /// OS, and the more serious of the two.
    #[must_use]
    pub fn is_about_who_it_was_for(&self) -> bool {
        matches!(
            self,
            Self::NotAnAnswerForAnAgent | Self::NotAnAnswerForAPerson
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use alo_strings::Vocabulary;

    /// One example of each, written out by hand: a list derived from the
    /// variants would be derived from the same thing it is checking.
    fn every_refusal() -> Vec<NotUnderstood> {
        vec![
            NotUnderstood::TooLong {
                most: 1_048_576,
                was: 4_000_000,
            },
            NotUnderstood::MoreThanOneMessage,
            NotUnderstood::FromANewerAloOs { format: 2 },
            NotUnderstood::NotAFormat { format: 0 },
            NotUnderstood::NotReadable,
            NotUnderstood::NotForAnAgent,
            NotUnderstood::NotForAPerson,
            NotUnderstood::NotAnAnswerForAnAgent,
            NotUnderstood::NotAnAnswerForAPerson,
        ]
    }

    /// **Every way a message can be refused has a sentence**, and no two of
    /// them are the same sentence: a message that was too long and one that was
    /// unreadable send whoever sent them to two different places.
    #[test]
    fn every_refusal_says_something_of_its_own() {
        let strings = in_english();
        let mut said: Vec<String> = every_refusal()
            .iter()
            .map(|why| why.said(&strings).into_text())
            .collect();
        assert_eq!(said.len(), words::EVERY_WORD.len());
        said.sort();
        said.dedup();
        assert_eq!(said.len(), words::EVERY_WORD.len());
    }

    /// **A refusal never depends on a string table.** With no words at all it
    /// still answers, and it names the refusal by its key so whoever forgot to
    /// declare this crate's words finds out from the sentence rather than from
    /// a blank line.
    #[test]
    fn a_refusal_without_the_words_still_names_itself() {
        let nothing = Strings::of(Vocabulary::empty());
        for why in every_refusal() {
            let said = why.said(&nothing);
            assert!(said.is_a_bug(), "{said}");
            assert!(said.text().contains(why.word().named()), "{said}");
        }
    }

    /// A refusal reads in the person's own language when somebody has
    /// translated it, and says so.
    #[test]
    fn a_refusal_reads_in_the_language_the_person_has() {
        let german = translated(&[(
            words::NOT_FOR_AN_AGENT,
            "ein Assistent kann keine Frage beantworten, die einem Menschen gestellt wurde",
        )]);
        let said = NotUnderstood::NotForAnAgent.said(&german);
        assert!(said.is_translated());
        assert!(said.text().starts_with("ein Assistent"), "{said}");
    }

    /// The numbers are fields rather than words, so a client that wants to show
    /// them beside the sentence can, and no language has to count in ours.
    #[test]
    fn the_numbers_are_carried_beside_the_sentence_and_not_inside_it() {
        let strings = in_english();
        let too_long = NotUnderstood::TooLong {
            most: 1_048_576,
            was: 4_000_000,
        };
        let said = too_long.said(&strings).into_text();
        assert!(!said.contains("1048576"), "{said}");
        assert!(!said.contains("4000000"), "{said}");
        assert_eq!(
            too_long,
            NotUnderstood::TooLong {
                most: 1_048_576,
                was: 4_000_000
            }
        );
    }

    /// **A request on the wrong side of the socket is not a client that cannot
    /// spell**, and the two are answerable apart without reading the sentence.
    #[test]
    fn a_request_from_the_wrong_side_is_answerable_as_that() {
        assert!(NotUnderstood::NotForAnAgent.is_about_who_asked());
        assert!(NotUnderstood::NotForAPerson.is_about_who_asked());
        for why in [
            NotUnderstood::NotReadable,
            NotUnderstood::MoreThanOneMessage,
            NotUnderstood::NotAFormat { format: 9 },
            NotUnderstood::FromANewerAloOs { format: 9 },
            NotUnderstood::TooLong { most: 1, was: 2 },
            NotUnderstood::NotAnAnswerForAnAgent,
            NotUnderstood::NotAnAnswerForAPerson,
        ] {
            assert!(!why.is_about_who_asked(), "{why:?}");
        }
    }

    /// **And an answer on the wrong door is a different thing again**, worth
    /// asking about separately because it is a mistake in this machine rather
    /// than in whatever is talking to it.
    #[test]
    fn an_answer_on_the_wrong_door_is_answerable_as_that() {
        assert!(NotUnderstood::NotAnAnswerForAnAgent.is_about_who_it_was_for());
        assert!(NotUnderstood::NotAnAnswerForAPerson.is_about_who_it_was_for());
        for why in every_refusal() {
            assert!(
                !(why.is_about_who_asked() && why.is_about_who_it_was_for()),
                "{why:?}"
            );
        }
        assert!(!NotUnderstood::NotForAnAgent.is_about_who_it_was_for());
        assert!(!NotUnderstood::NotReadable.is_about_who_it_was_for());
    }
}
