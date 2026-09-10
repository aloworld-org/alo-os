//! Every string this crate can say, and the English beside each one.
//!
//! Two of them are refusals — why the agent did not appear when the key was
//! pressed. The rest are **what the overlay shows once it has appeared and
//! before anybody has asked it anything**: the state it is in, and the three
//! readings under it. An empty overlay's states are strings too, and this is
//! where they are declared rather than written into a compositor.
//!
//! The shape is `alo-shortcuts`' and is copied rather than re-decided:
//! constants under one area, `alo_strings::Word` because these are literals
//! in this file, [`Counted`] for the two that count something,
//! [`declare_into`] for the one vocabulary the machine has, and tests at the
//! bottom holding the list to the rules every other list is held to.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no alo machine in front of them. *The
//! agent* is the assistant built into alo OS and not a person; *the desktop*
//! is the graphical session and not a piece of furniture; *a grant* is a
//! person allowing the agent to reach one thing. Where the sentence cannot be
//! translated from its own words, the note says so.
//!
//! # A headline is a whole sentence; a reading is a clause
//!
//! [`crate::Standing`]'s three strings are what a person reads first, so they
//! are whole capitalised sentences and two of them say **what to do**. The
//! readings beneath — what answers, what is granted, what is leaving — are
//! lower-case clauses drawn one under another, the way `alo-keeping` and
//! `alo-egress` write a line in a list. Nothing here joins them: the
//! conjunction between two clauses is not punctuation a program can pick for
//! a language it does not know.

use alo_strings::{Key, Plural, Vocabulary};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// One string this crate can say about a number of things.
///
/// Separate from [`Word`] because a countable string is declared and looked
/// up differently: two English sentences rather than one, and the reader's
/// own language decides which of *its* forms is shown. The shape is
/// `alo-keeping`'s, copied rather than re-decided.
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

/// What [`crate::NotSummoned::NoCompositor`] says: the key was pressed
/// somewhere no desktop is running, so the agent has nowhere to appear.
pub const NO_COMPOSITOR: Word = Word::saying(
    "overlay.summon.no-compositor",
    "The agent has nowhere to appear: the desktop is not running. Sign in to the desktop and \
     press the key again",
)
.noting(
    "The agent is the assistant built into alo OS, not a person. The desktop is the graphical \
     session a person signs into. This is shown when the key that summons the agent is pressed \
     somewhere nothing is drawing a screen — a text console, or a session that is still \
     starting.",
);

/// What [`crate::SurfaceRefused::NothingToShowOn`] says: the desktop is
/// running, and there is no screen to put the overlay on.
pub const NOTHING_TO_SHOW_ON: Word = Word::saying(
    "overlay.summon.nothing-to-show-on",
    "The agent has nowhere to appear: no screen is connected. Connect a screen and press the \
     key again",
)
.noting(
    "The agent is the assistant built into alo OS, not a person. Shown when the desktop is \
     running without any display to draw on — before a monitor is plugged in, or after the \
     last one went away. A screen here is a physical display.",
);

// ---------------------------------------------------------------------------
// What state the overlay is in before a question is asked — [`crate::Standing`].
//
// The first thing a person reads when the key opens the overlay, and the whole
// of what they read when there is nothing to do yet. Two of the three say what
// to do, because an empty state that only reports emptiness has told somebody
// they are stuck.
// ---------------------------------------------------------------------------

/// Nothing has been chosen to answer questions with.
pub const NOTHING_CHOSEN: Word = Word::saying(
    "overlay.at-rest.nothing-chosen",
    "Nothing has been chosen to answer questions yet. Choose a model to run on this machine, or \
     add a provider to send questions to, and the agent can start",
)
.noting(
    "The agent is the assistant built into alo OS, not a person. A provider is a company or \
     service whose model answers questions somewhere other than this machine. Read the first time \
     somebody presses the agent's key on a machine nobody has set up, so the second half matters \
     as much as the first: a person shown only that nothing is chosen has been told they are \
     stuck.",
);

/// Something would answer, and the agent has been granted nothing.
pub const NOTHING_GRANTED: Word = Word::saying(
    "overlay.at-rest.nothing-granted",
    "The agent can reach nothing on this machine yet. Grant it a folder, and it can work with \
     what is in that folder and with nothing else",
)
.noting(
    "The agent is the assistant built into alo OS, not a person. To grant is for the person to \
     allow the agent to reach one thing — alo OS never grants anything by itself, and there is no \
     grant to a whole machine. Read when something can answer questions but nothing has been \
     granted, so the second half is the instruction and the reassurance at once.",
);

/// Something would answer and something is granted: the ordinary machine.
pub const READY: Word = Word::saying("overlay.at-rest.ready", "Ask the agent anything").noting(
    "The agent is the assistant built into alo OS, not a person. Short on purpose: this is what \
     the overlay says on a machine that is set up, and the readings drawn beneath it say what \
     answers, what is granted and whether anything is leaving. An invitation, not a status.",
);

// ---------------------------------------------------------------------------
// The three readings drawn beneath it.
//
// Lower-case clauses on lines of their own, never joined into one sentence:
// what would answer a question, what the agent may reach, and what is leaving
// the machine at this moment.
// ---------------------------------------------------------------------------

/// What would answer, on a machine where nobody has chosen.
pub const ANSWERS_NOTHING: Word = Word::saying(
    "overlay.answers.nothing",
    "no model or provider has been chosen",
)
.noting(
    "One of three readings shown on lines of their own beneath the sentence above them, which is \
     why it is a lower-case clause rather than a sentence. A provider is a company or service \
     whose model answers questions somewhere other than this machine.",
);

/// What would answer, on a machine where somebody has.
pub const ANSWERS_SOMETHING: Word =
    Word::saying("overlay.answers.something", "{model} answers, {where}").noting(
        "A lower-case clause on a line of its own. {model} is a model's name exactly as its own \
         list writes it and is never translated. {where} is a clause this system says elsewhere — \
         \"on this machine\", or \"by Mistral, in the EU\" — and arrives already translated. Read \
         by somebody who is entitled to know where their question would go before they type it.",
    );

/// What is granted, on a machine where nothing is.
pub const GRANTED_NOTHING: Word = Word::saying("overlay.granted.nothing", "nothing is granted")
    .noting(
        "A lower-case clause on a line of its own. To grant is for the person to allow the agent \
         to reach one thing — a folder, a file or an application. This is the honest reading of a \
         machine where the agent can reach nothing, and it is the state every machine starts in.",
    );

/// What is granted, on a machine where something is.
pub const GRANTED_HOW_MANY: Counted = Counted {
    named: "overlay.granted.how-many",
    number: "how_many",
    one: "one thing is granted",
    other: "{how_many} things are granted",
    note: "A lower-case clause on a line of its own. {how_many} is a whole number of things the \
           person has granted the agent: each is a folder, a file or an application, which is why \
           the sentence says things rather than naming one of the three.",
};

/// What is leaving, on a machine where nothing is.
pub const NOTHING_IS_LEAVING: Word = Word::saying(
    "overlay.egress.nothing-is-leaving",
    "nothing is leaving this machine",
)
.noting(
    "A lower-case clause on a line of its own, and the state a machine answering its own \
     questions is in all day. To leave the machine is for something to be sent over the network. \
     Present tense and about this moment: what happened earlier is a different question.",
);

/// What is leaving, on a machine where something is.
pub const HOW_MANY_ARE_LEAVING: Counted = Counted {
    named: "overlay.egress.how-many-are-leaving",
    number: "how_many",
    one: "one thing is leaving this machine right now",
    other: "{how_many} things are leaving this machine right now",
    note: "A lower-case clause on a line of its own. {how_many} is a whole number of connections \
           open at this moment, each of which alo OS or one of its agents caused. \"Right now\" is \
           the point of the sentence and should survive translation: it is about this instant and \
           not about the day.",
};

/// Every plain string this crate can say, in the order a translator meets
/// them.
///
/// The two countable ones are not here — they are declared beneath, because
/// they are declared differently.
pub const EVERY_WORD: [Word; 9] = [
    NO_COMPOSITOR,
    NOTHING_TO_SHOW_ON,
    NOTHING_CHOSEN,
    NOTHING_GRANTED,
    READY,
    ANSWERS_NOTHING,
    ANSWERS_SOMETHING,
    GRANTED_NOTHING,
    NOTHING_IS_LEAVING,
];

/// Every countable string this crate can say, in the same order.
pub const EVERY_COUNTED: [Counted; 2] = [GRANTED_HOW_MANY, HOW_MANY_ARE_LEAVING];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the lists above — the tests at the bottom of
/// this file are what say so. It is a `Result` rather than an unwrap because
/// a library that panics on its own string table takes the shell with it, and
/// because [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note
    /// that could not be attached.
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
pub fn overlay_words() -> Result<Vocabulary, WordsError> {
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
/// nothing is replaced, because a key means one string and whoever declared
/// it first said what that string is.
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
    /// [`Word::key`] does not check, because a key written in this file
    /// cannot arrive from anywhere; this is the test that makes that true.
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
            assert_eq!(word.key().area(), "overlay", "{}", word.named());
        }
        for counted in EVERY_COUNTED {
            assert_eq!(counted.key().area(), "overlay", "{}", counted.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — the plain strings and the two that count something.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = overlay_words().unwrap();
        assert_eq!(
            vocabulary.how_many(),
            EVERY_WORD.len() + EVERY_COUNTED.len()
        );
        assert_eq!(vocabulary.counted().count(), EVERY_COUNTED.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and
    /// nothing is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = overlay_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Both refusals are whole sentences with nothing to fill in** — a gap
    /// in a refusal would put `{}` in front of a person at the exact moment
    /// something already went wrong.
    #[test]
    fn a_refusal_names_nothing_and_needs_nothing_filled() {
        for word in [NO_COMPOSITOR, NOTHING_TO_SHOW_ON] {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().gaps().is_empty(), "{}", word.named());
        }
    }

    /// **The state a person reads first is a whole sentence with nothing to
    /// fill in**, whichever of the three it is. A headline with a gap would
    /// be a headline that could arrive with a hole in it.
    #[test]
    fn no_headline_needs_anything_filled_in() {
        for word in [NOTHING_CHOSEN, NOTHING_GRANTED, READY] {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().gaps().is_empty(), "{}", word.named());
        }
    }

    /// **The two headlines a person can do something about say what to do.**
    /// This is the plan's acceptance for the empty overlay, held at the level
    /// of the string: *the nothing chosen case says what to do rather than
    /// being empty.* Each is two clauses — what is so, and then the action —
    /// so neither can shrink to a bare report without failing here.
    #[test]
    fn the_states_with_something_to_do_say_what_to_do() {
        for (word, verb) in [(NOTHING_CHOSEN, "Choose"), (NOTHING_GRANTED, "Grant")] {
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

    /// **Every word and every counted string carries a note.** None of these
    /// can be translated from its own words alone: each is shown at a moment
    /// the translator has to be able to picture, and several name things —
    /// the agent, a grant, a provider — that have a meaning here.
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
    /// A translator with no alo machine in front of them would otherwise have
    /// to guess, and in several languages the guess decides the grammar of the
    /// whole sentence.
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
    /// English sentence is a sentence that cannot be translated into a
    /// language with three plural forms; the two that count are declared as
    /// plurals, and this is what keeps a third from being written by hand.
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

    /// **Each counted string says the same thing about one as about many**,
    /// and only the plural form carries the gap: a `one` form with
    /// `{how_many}` in it reads *1 things are granted* in the source and is
    /// the mistake this shape exists to prevent.
    #[test]
    fn a_counted_string_counts_in_its_other_form_only() {
        for counted in EVERY_COUNTED {
            assert!(!counted.one().contains("{how_many}"), "{}", counted.named());
            assert!(
                counted.other().contains("{how_many}"),
                "{}",
                counted.named()
            );
            assert!(counted.one().starts_with("one "), "{}", counted.named());
        }
    }
}
