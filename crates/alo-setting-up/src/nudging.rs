//! Nothing setup says asks anybody to buy anything, or leans on one of the four.
//!
//! [ADR 0009](../../../docs/decisions/0009-a-good-computer-without-the-agent.md)
//! gave the fourth choice *the same weight as the other three and no persuasion
//! attached*, and
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! rejected pre-selection because *a pre-selected control is persuasion by
//! geometry*. `crate::setting_up` keeps the geometry half: nothing is selected
//! and pressing on without choosing does not choose. **This file is the other
//! half, which is the copy** — because a flow that selects nothing and then
//! calls one of the four *the recommended option* has moved the persuasion from
//! the geometry into the sentence, where no structural rule can see it.
//!
//! Since every word alo OS says is declared in `alo-strings`, that is checkable
//! rather than a habit — the shape `alo_saying::rented` settled for a different
//! rule about the same vocabulary.
//!
//! # It passes today, and that is why it is worth having
//!
//! Nothing this crate says leans on anything. The check therefore costs nothing
//! now and catches the first one later, which is the only moment it could be
//! useful: the sentence it exists for gets written the day somebody is asked to
//! make setup convert better.
//!
//! # It is asked of this crate's list and not of the machine's
//!
//! Deliberately. *The money ran out* is a sentence alo OS has to be able to say
//! (`docs/features.md`, ADR 0009), and `alo-answering` says it; a check over the
//! whole vocabulary would either fire on that or be watered down until it fired
//! on nothing. The rule is about **the moment a person is choosing between four
//! configurations**, so it is asked of the strings read at that moment.
//!
//! # Two places a word can arrive, and the second is not obvious
//!
//! - **The sentence.** What a person meets. The case the rule is about.
//! - **The note.** Nobody using the machine reads a note; a translator does,
//!   and what they write from it is read in every language alo OS is translated
//!   into. A translator told which of the four is the sensible one writes a
//!   sentence that says so, and no test in this repository can see that file.
//!
//! The key is not checked, and that is the one difference from
//! `alo_saying::rented`: a key is read by a person only when nothing declares
//! it, and a key here that read as persuasion — `setup.choice.recommended` —
//! would be a name for a choice that does not exist rather than a nudge toward
//! one that does. What would catch that is `crate::offered`'s closed list.

use std::fmt;

use alo_strings::{Key, Vocabulary};

/// A word that would lean on a person choosing between the four.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leaning {
    /// The word itself.
    word: &'static str,
    /// What it does to somebody who is choosing. Quoted when the check fails,
    /// because whoever reads that is being asked to write another sentence.
    why: &'static str,
}

impl Leaning {
    /// One word, and what it does.
    const fn leans(word: &'static str, why: &'static str) -> Self {
        Self { word, why }
    }

    /// The word itself.
    #[must_use]
    pub const fn word(&self) -> &'static str {
        self.word
    }

    /// What it does to somebody who is choosing.
    #[must_use]
    pub const fn why(&self) -> &'static str {
        self.why
    }
}

/// Every word that would turn one of the four into the answer.
///
/// Two kinds, and both are named because they are the two ways this promise
/// really gets broken: **a ranking**, which makes three of the four into the
/// wrong answer, and **money**, which is what a machine sold on sovereignty
/// must never ask for at the moment somebody is deciding whether to have an
/// agent at all.
pub const EVERYTHING_THAT_LEANS: [Leaning; 16] = [
    Leaning::leans(
        "recommended",
        "it names one of the four as the one alo OS would pick, which is the default ADR 0016 \
         refused wearing a sentence instead of a setting",
    ),
    Leaning::leans("recommend", "the same, said as a verb"),
    Leaning::leans(
        "best",
        "a ranking, and three of the four are then the wrong answer",
    ),
    Leaning::leans(
        "easiest",
        "a ranking about effort rather than about quality",
    ),
    Leaning::leans(
        "powerful",
        "a claim about one choice that the other three are then measured against",
    ),
    Leaning::leans(
        "fastest",
        "a ranking, and one this crate could not honestly make about anybody's machine",
    ),
    Leaning::leans(
        "popular",
        "what other people did, which is not a fact about this person's machine",
    ),
    Leaning::leans(
        "simply",
        "it says one choice takes less of somebody than another, which is persuasion about effort",
    ),
    Leaning::leans("buy", "money, at the moment a person is choosing"),
    Leaning::leans("purchase", "the same, said the other way"),
    Leaning::leans(
        "free",
        "an argument about money even when it is true, and the one that makes a person feel they \
         are being sold the alternative",
    ),
    Leaning::leans("cheaper", "money, as a comparison between the four"),
    Leaning::leans(
        "subscription",
        "a thing to sign up to, offered where a person is deciding whether to have an agent at all",
    ),
    Leaning::leans("subscribe", "the same, said as a verb"),
    Leaning::leans(
        "upgrade",
        "it makes one of the four into a lesser version of another",
    ),
    Leaning::leans(
        "premium",
        "it makes the other three into the unpaid version of this one",
    ),
];

/// Which part of a declaration a leaning word was found in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    /// The English a person is shown when nobody has translated it.
    Sentence,
    /// What a translator was told about it.
    Note,
}

/// One word that leans, in something setup says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leant {
    /// The string it is in.
    key: Key,
    /// What was found.
    leaning: Leaning,
    /// Which part of the declaration it was in.
    found: Where,
    /// The text it was found in, quoted back so nobody has to go looking.
    text: String,
}

impl Leant {
    /// The string it is in.
    #[must_use]
    pub const fn key(&self) -> &Key {
        &self.key
    }

    /// What was found.
    #[must_use]
    pub const fn leaning(&self) -> Leaning {
        self.leaning
    }

    /// Which part of the declaration it was in.
    #[must_use]
    pub const fn found(&self) -> Where {
        self.found
    }

    /// The text it was found in.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Said as *this would lean on somebody who is choosing*, rather than as *this
/// word is forbidden*. Whoever reads it is looking for another sentence to
/// write, and a list of banned words does not help them find one.
impl fmt::Display for Leant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let word = self.leaning.word();
        let why = self.leaning.why();
        let key = &self.key;
        match self.found {
            Where::Sentence => write!(
                formatter,
                "a person choosing between the four would read \"{word}\" in {key}, and {why}. \
                 The sentence says: \"{}\"",
                self.text
            ),
            Where::Note => write!(
                formatter,
                "a translator working on {key} would be told \"{word}\", and {why} — what they \
                 write from it is read in every language alo OS is translated into, where nothing \
                 here could see it. The note says: \"{}\"",
                self.text
            ),
        }
    }
}

/// Every word that leans, in everything this vocabulary can say.
///
/// Answers with all of them rather than with the first, for the reason
/// `alo_strings::Vocabulary::check` does: somebody fixing a list wants to see
/// the work, not to be told about the next one each time they try again.
///
/// An empty answer is the ordinary case and the one alo OS ships with.
#[must_use]
pub fn what_would_lean_on_a_person(vocabulary: &Vocabulary) -> Vec<Leant> {
    let mut leant = Vec::new();
    for phrase in vocabulary.phrases() {
        look_at(
            &mut leant,
            phrase.key(),
            phrase.source().as_written(),
            Where::Sentence,
        );
        if let Some(note) = phrase.note() {
            look_at(&mut leant, phrase.key(), note, Where::Note);
        }
    }
    leant
}

/// Everything on the list that this text says.
fn look_at(leant: &mut Vec<Leant>, key: &Key, text: &str, found: Where) {
    for leaning in EVERYTHING_THAT_LEANS {
        if says(text, leaning.word()) {
            leant.push(Leant {
                key: key.clone(),
                leaning,
                found,
                text: text.to_owned(),
            });
        }
    }
}

/// Whether this text says this word.
///
/// Whole words, however they are capitalised, with an English plural allowed
/// through — *the premiums* is the same sentence with an `s` on it. A word
/// inside a longer one is not a match, because a check that fired on
/// `bestowed` is one somebody learns to work around, and then the real one goes
/// past them too.
///
/// **Its own matcher rather than `alo-saying`'s**, and that is deliberate: this
/// crate is collected *by* `alo-saying` and reaching back into it would be a
/// dependency running the wrong way through the one vocabulary. What is shared
/// between the two rules is the argument, which is in both files, and not
/// twenty lines that would then have to serve two lists.
fn says(text: &str, word: &str) -> bool {
    let text = text.to_ascii_lowercase();
    let word = word.to_ascii_lowercase();
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(rest) = text.get(from..) {
        let Some(at) = rest.find(&word) else {
            return false;
        };
        let start = from + at;
        let end = start + word.len();
        let after = if bytes.get(end) == Some(&b's') {
            end + 1
        } else {
            end
        };
        let inside_a_longer_word = start
            .checked_sub(1)
            .and_then(|before| bytes.get(before))
            .is_some_and(u8::is_ascii_alphanumeric)
            || bytes.get(after).is_some_and(u8::is_ascii_alphanumeric);
        if !inside_a_longer_word {
            return true;
        }
        // The match started at an ASCII byte, so the next index is a boundary.
        from = start + 1;
    }
    false
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::words::setting_up_words;
    use alo_strings::Phrase;
    use std::collections::BTreeSet;

    /// A vocabulary holding one phrase, for asking about one string at a time.
    fn saying(key: &str, sentence: &str) -> Vocabulary {
        Vocabulary::empty()
            .and(Phrase::says(Key::named(key).unwrap(), sentence).unwrap())
            .unwrap()
    }

    /// **Nothing setup says leans on anybody.** The guarantee, asked of every
    /// string this crate declares — the question, the four names, the four
    /// lines, and the four refusals.
    #[test]
    fn nothing_setup_says_asks_anybody_to_buy_anything_or_leans_on_one_of_the_four() {
        let leant = what_would_lean_on_a_person(&setting_up_words().unwrap());
        assert!(
            leant.is_empty(),
            "{}",
            leant
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// **Every word on the list says what it does**, because a failure quotes
    /// it and a reader who is being asked to write another sentence needs to
    /// know what was wrong with this one. No two entries are the same.
    #[test]
    fn every_word_on_the_list_says_what_it_does() {
        let mut seen = BTreeSet::new();
        for leaning in EVERYTHING_THAT_LEANS {
            assert!(!leaning.word().trim().is_empty());
            assert!(
                leaning.why().trim().len() > "a nudge".len(),
                "{} says nothing about what it does",
                leaning.word()
            );
            assert!(
                seen.insert(leaning.word()),
                "{} is here twice",
                leaning.word()
            );
        }
    }

    /// **A ranking in a sentence is what this exists to catch**, and the
    /// failure is what the person reading it would be leant on by rather than
    /// *forbidden word*.
    #[test]
    fn a_ranking_in_a_sentence_is_found() {
        let leant = what_would_lean_on_a_person(&saying(
            "setup.choice.on-this-machine.is",
            "the best way to run alo OS",
        ));
        assert_eq!(leant.len(), 1, "{leant:?}");
        let only = leant.first().unwrap();
        assert_eq!(only.leaning().word(), "best");
        assert_eq!(only.found(), Where::Sentence);
        let said = only.to_string();
        assert!(
            said.contains("a person choosing between the four"),
            "{said}"
        );
        assert!(said.contains("setup.choice.on-this-machine.is"), "{said}");
    }

    /// **Money in a sentence is the other half of the rule.** A machine sold on
    /// sovereignty must not ask for a card at the moment somebody is deciding
    /// whether to have an agent at all.
    #[test]
    fn an_argument_about_money_is_found() {
        for sentence in [
            "buy credit to get started",
            "the free option",
            "upgrade later for more",
        ] {
            assert!(
                !what_would_lean_on_a_person(&saying("setup.the-question", sentence)).is_empty(),
                "{sentence}"
            );
        }
    }

    /// **A note is checked because a translator reads it**, and what they write
    /// from it is read in every language alo OS is translated into.
    #[test]
    fn a_word_that_leans_in_a_note_is_found() {
        let vocabulary = Vocabulary::empty()
            .and(
                Phrase::says(
                    Key::named("setup.choice.not-at-all").unwrap(),
                    "not at all",
                )
                .unwrap()
                .noting("The one people are least likely to pick; the first is the recommended one.")
                .unwrap(),
            )
            .unwrap();
        let leant = what_would_lean_on_a_person(&vocabulary);
        assert_eq!(leant.len(), 1, "{leant:?}");
        assert_eq!(leant.first().unwrap().found(), Where::Note);
        assert!(
            leant.first().unwrap().to_string().contains("a translator"),
            "{}",
            leant.first().unwrap()
        );
    }

    /// **However it is written.** A sentence that capitalises a word has not
    /// stopped leaning on anybody.
    #[test]
    fn it_does_not_matter_how_the_word_is_capitalised() {
        for written in ["Recommended", "recommended", "RECOMMENDED"] {
            assert!(
                !what_would_lean_on_a_person(&saying(
                    "setup.the-question",
                    &format!("{written} for most machines")
                ))
                .is_empty(),
                "{written}"
            );
        }
    }

    /// **A word inside a longer one is not a leaning.** A check that fired on
    /// `bestowed` is one somebody learns to work around, and then the real one
    /// goes past them too.
    #[test]
    fn a_word_inside_a_longer_word_is_not_a_leaning() {
        for sentence in [
            "a bestowed inheritance",
            "the freeholder of the building",
            "a buyer somewhere else",
        ] {
            assert!(
                what_would_lean_on_a_person(&saying("setup.the-question", sentence)).is_empty(),
                "{sentence}"
            );
        }
    }

    /// A vocabulary that says nothing leans on nobody, which is the state every
    /// process starts in.
    #[test]
    fn a_vocabulary_that_says_nothing_leans_on_nobody() {
        assert!(what_would_lean_on_a_person(&Vocabulary::empty()).is_empty());
    }
}
