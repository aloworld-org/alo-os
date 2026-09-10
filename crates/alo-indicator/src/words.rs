//! Every string this crate can say, and the English beside each one.
//!
//! Four of them, and they divide in two. Two are **what the indicator itself
//! reads as** — the sentence a screen reader is given when the light is dark,
//! and the counted one it is given when the light is lit. Two are **refusals**:
//! why what is leaving this machine could not be put on a screen at all.
//!
//! The shape is `alo-overlay`'s and is copied rather than re-decided:
//! constants under one area, `alo_strings::Word` because these are literals in
//! this file, [`Counted`] for the one that counts something, [`declare_into`]
//! for the one vocabulary the machine has, and tests at the bottom holding the
//! list to the rules every other list is held to.
//!
//! # Why these are not `alo-overlay`'s two, which say nearly the same thing
//!
//! `overlay.egress.nothing-is-leaving` and `overlay.egress.how-many-are-leaving`
//! are the same fact worded for a different place, and the difference is not
//! decoration. Those are **lower-case clauses in a list**, drawn under two
//! other clauses inside a panel a person opened; they are read with the lines
//! above them and are never read alone. These are the **whole of what one
//! control says**: the indicator is a light with nothing else on it, so its
//! name is its state, it is announced on its own by a screen reader, and it is
//! a capitalised sentence for the same reason every other control's name is.
//!
//! A translator meets both and must be able to tell them apart, so each note
//! says which of the two places it is for. What neither is, is a fallback for
//! the other: the two are shown at the same time on the same machine, and a
//! machine on which they disagreed would have two answers to *has anything left
//! this machine* — which is the one thing law 1 does not survive.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no alo machine in front of them. *To leave
//! the machine* is for something to be sent over the network; *the desktop* is
//! the graphical session and not a piece of furniture; *a screen* is a physical
//! display. Where the sentence cannot be translated from its own words, the
//! note says so.

use alo_strings::{Key, Plural, Vocabulary};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// One string this crate can say about a number of things.
///
/// Separate from [`Word`] because a countable string is declared and looked up
/// differently: two English sentences rather than one, and the reader's own
/// language decides which of *its* forms is shown. The shape is
/// `alo-overlay`'s, copied rather than re-decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counted {
    /// What names it.
    named: &'static str,
    /// The gap the number goes in.
    number: &'static str,
    /// What it says about one thing.
    one: &'static str,
    /// What it says about any other number of things.
    other: &'static str,
    /// What a translator needs to know.
    note: &'static str,
}

impl Counted {
    /// What names it.
    #[must_use]
    pub fn key(&self) -> Key {
        Key::unchecked(self.named)
    }

    /// The key as it was written, for a test that has to name it in a failure.
    #[must_use]
    pub const fn named(&self) -> &'static str {
        self.named
    }

    /// The gap the number goes in.
    #[must_use]
    pub const fn number(&self) -> &'static str {
        self.number
    }

    /// What it says about one thing.
    #[must_use]
    pub const fn one(&self) -> &'static str {
        self.one
    }

    /// What it says about any other number of things.
    #[must_use]
    pub const fn other(&self) -> &'static str {
        self.other
    }

    /// What a translator needs to know that the two sentences do not say.
    #[must_use]
    pub const fn note(&self) -> &'static str {
        self.note
    }
}

// ---------------------------------------------------------------------------
// What the light reads as — [`crate::Lamp`].
//
// The whole of what the control says, on its own, to somebody who cannot see
// whether it is dark. A light nobody can read is a light that does not exist
// for some people, and law 1 is not a promise we keep for people who can see
// it and break for people who cannot.
// ---------------------------------------------------------------------------

/// A dark indicator: nothing is leaving this machine at this moment.
pub const NOTHING_IS_LEAVING: Word = Word::saying(
    "indicator.nothing-is-leaving",
    "Nothing is leaving this machine right now",
)
.noting(
    "The whole of what one control says, announced on its own by a screen reader, which is why it \
     is a capitalised sentence rather than a clause. It is the name of the indicator that shows \
     what is being sent over the network, and it is what that indicator reads as while it is \
     dark. To leave the machine is for something to be sent over the network. \"Right now\" is the \
     point of the sentence and should survive translation: it is about this instant and not about \
     the day. There is a lower-case clause saying the same thing under the key \
     overlay.egress.nothing-is-leaving, for a list inside a panel; the two are shown at the same \
     time on one machine and must not disagree.",
);

/// A lit indicator, with what it is lit for counted.
pub const HOW_MANY_ARE_LEAVING: Counted = Counted {
    named: "indicator.how-many-are-leaving",
    number: "how_many",
    one: "One thing is leaving this machine right now",
    other: "{how_many} things are leaving this machine right now",
    note: "The whole of what one control says, announced on its own by a screen reader, which is \
           why it is a capitalised sentence rather than a clause. It is what the indicator that \
           shows network activity reads as while it is lit. {how_many} is a whole number of \
           connections open at this moment, each of which alo OS or one of its agents caused. \
           \"Right now\" is the point of the sentence and should survive translation: it is about \
           this instant and not about the day. There is a lower-case clause saying the same thing \
           under the key overlay.egress.how-many-are-leaving, for a list inside a panel; the two \
           are shown at the same time on one machine and must not disagree.",
};

// ---------------------------------------------------------------------------
// Why what is leaving could not be shown at all — [`crate::NotShown`].
//
// Both of these are read somewhere other than the screen the indicator would
// have been on, because there is no such screen: a service log while a session
// starts, or a text console. They are still declared and still translated,
// because the person reading a log on their own machine reads their own
// language.
// ---------------------------------------------------------------------------

/// What [`crate::NotShown::NoCompositor`] says: nothing is drawing a screen,
/// so there is nowhere to put the indicator.
pub const NO_COMPOSITOR: Word = Word::saying(
    "indicator.no-compositor",
    "What is leaving this machine cannot be shown: the desktop is not running. Sign in to the \
     desktop, and the indicator appears with it",
)
.noting(
    "The desktop is the graphical session a person signs into. To leave the machine is for \
     something to be sent over the network. Read when the part of alo OS that draws a screen is \
     not running at all — during start-up, or on a text console — so it is read in a log or on a \
     console rather than on the screen it is about.",
);

/// What [`crate::SurfaceRefused::NothingToShowOn`] says: something is drawing,
/// and there is no screen to draw on.
pub const NOTHING_TO_SHOW_ON: Word = Word::saying(
    "indicator.nothing-to-show-on",
    "What is leaving this machine cannot be shown: no screen is connected. Connect a screen, and \
     the indicator appears on it",
)
.noting(
    "To leave the machine is for something to be sent over the network. A screen here is a \
     physical display. Read when the desktop is running with no display to draw on — before a \
     monitor is plugged in, or after the last one went away.",
);

/// Every plain string this crate can say, in the order a translator meets them.
///
/// The countable one is not here — it is declared beneath, because it is
/// declared differently.
pub const EVERY_WORD: [Word; 3] = [NOTHING_IS_LEAVING, NO_COMPOSITOR, NOTHING_TO_SHOW_ON];

/// Every countable string this crate can say.
pub const EVERY_COUNTED: [Counted; 1] = [HOW_MANY_ARE_LEAVING];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the lists above — the tests at the bottom of
/// this file are what say so. It is a `Result` rather than an unwrap because a
/// library that panics on its own string table takes the shell with it, and
/// because [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A countable string that could not be declared.
    #[error(transparent)]
    Counting(#[from] alo_strings::PluralError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn indicator_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The machine has one vocabulary and every crate adds its own to it —
/// `alo-saying` is the one place that calls all of these.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    for counted in EVERY_COUNTED {
        vocabulary.counts(
            Plural::counting(
                counted.key(),
                counted.number(),
                counted.one(),
                counted.other(),
            )?
            .noting(counted.note())?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
        }
        for counted in EVERY_COUNTED {
            assert_eq!(
                alo_strings::Key::named(counted.named()),
                Ok(counted.key()),
                "{}",
                counted.named()
            );
        }
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD
            .iter()
            .map(|word| word.named())
            .chain(EVERY_COUNTED.iter().map(|counted| counted.named()))
            .collect();
        assert_eq!(named.len(), EVERY_WORD.len() + EVERY_COUNTED.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "indicator", "{}", word.named());
        }
        for counted in EVERY_COUNTED {
            assert_eq!(counted.key().area(), "indicator", "{}", counted.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — the plain strings and the one that counts something.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = indicator_words().unwrap();
        assert_eq!(
            vocabulary.how_many(),
            EVERY_WORD.len() + EVERY_COUNTED.len()
        );
        assert_eq!(vocabulary.counted().count(), EVERY_COUNTED.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = indicator_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every refusal is a whole sentence with nothing to fill in** — a gap in
    /// a refusal would put `{}` in front of a person at the exact moment
    /// something already went wrong.
    #[test]
    fn a_refusal_names_nothing_and_needs_nothing_filled() {
        for word in [NO_COMPOSITOR, NOTHING_TO_SHOW_ON] {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().gaps().is_empty(), "{}", word.named());
        }
    }

    /// **Both refusals say what to do.** Somebody told only that what is
    /// leaving their machine cannot be shown has been told they are stuck,
    /// about the one guarantee this product is sold on. Each is two clauses —
    /// what is so, and then the action — so neither can shrink to a bare report
    /// without failing here.
    #[test]
    fn both_refusals_say_what_to_do() {
        for (word, verb) in [(NO_COMPOSITOR, "Sign in"), (NOTHING_TO_SHOW_ON, "Connect")] {
            assert!(
                word.says().contains(verb),
                "{} does not tell anybody to {verb}",
                word.named()
            );
            assert!(
                word.says().split_once(". ").is_some(),
                "{} is one clause, so it reports without instructing",
                word.named()
            );
        }
    }

    /// **What the light reads as stands alone.** Every one of these is the
    /// whole of what one control says, announced with nothing above or beside
    /// it, so each is a capitalised sentence — which is the difference between
    /// this list and `alo-overlay`'s pair of clauses, and the reason both
    /// exist.
    #[test]
    fn what_the_light_reads_as_is_a_sentence_that_stands_alone() {
        for says in [
            NOTHING_IS_LEAVING.says(),
            HOW_MANY_ARE_LEAVING.one(),
            HOW_MANY_ARE_LEAVING.other(),
        ] {
            let first = says.chars().next().unwrap();
            assert!(
                first.is_uppercase() || first == '{',
                "{says} does not begin a sentence"
            );
        }
    }

    /// **Every word and every counted string carries a note.** None of these
    /// can be translated from its own words alone: each is shown at a moment
    /// the translator has to be able to picture, and each names something — the
    /// desktop, a screen, leaving the machine — that has a meaning here.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
        for counted in EVERY_COUNTED {
            assert!(!counted.note().trim().is_empty(), "{}", counted.named());
        }
    }

    /// **Every sentence that names the agent says the agent is not a person.**
    /// Nothing on this list names it today; the test is what holds the next one
    /// added here to the rule the rest of the workspace keeps, in a language
    /// where the answer decides the grammar of the whole sentence.
    #[test]
    fn every_word_that_names_the_agent_says_what_the_agent_is() {
        for word in EVERY_WORD {
            if !word.says().contains("agent") {
                continue;
            }
            assert!(
                word.note()
                    .is_some_and(|note| note.contains("not a person")),
                "{} does not say what the agent is",
                word.named()
            );
        }
    }

    /// **Nothing said plainly counts anything.** A number written into an
    /// English sentence is a sentence that cannot be translated into a language
    /// with three plural forms; the one that counts is declared as a plural,
    /// and this is what keeps a second from being written by hand.
    #[test]
    fn nothing_said_plainly_counts_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
            assert!(
                !word.says().contains("{how_many}"),
                "{} counts something without being a plural",
                word.named()
            );
        }
    }

    /// **The counted string counts in its other form only**, and only that form
    /// carries the gap: a `one` form with `{how_many}` in it reads *1 things
    /// are leaving* in the source and is the mistake this shape exists to
    /// prevent.
    #[test]
    fn a_counted_string_counts_in_its_other_form_only() {
        for counted in EVERY_COUNTED {
            assert!(!counted.one().contains("{how_many}"), "{}", counted.named());
            assert!(
                counted.other().contains("{how_many}"),
                "{}",
                counted.named()
            );
            assert!(counted.one().starts_with("One "), "{}", counted.named());
        }
    }
}
