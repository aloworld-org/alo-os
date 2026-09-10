//! Every string this crate can say, and the English beside each one.
//!
//! Two sentences, and both are refusals: the only thing the summoning seam
//! ever says to a person is why the agent did not appear. What the overlay
//! shows once it *has* appeared is the next task in the plan and will be
//! declared here beside these when it exists — an empty overlay's states are
//! strings too.
//!
//! The shape is `alo-shortcuts`' and is copied rather than re-decided:
//! constants under one area, `alo_strings::Word` because these are literals
//! in this file, `declare_into` for the one vocabulary the machine has, and
//! tests at the bottom holding the list to the rules every other list is
//! held to.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no alo machine in front of them. *The
//! agent* is the assistant built into alo OS and not a person; *the desktop*
//! is the graphical session and not a piece of furniture. Where the sentence
//! cannot be translated from its own words, the note says so.

use alo_strings::Vocabulary;

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

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

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 2] = [NO_COMPOSITOR, NOTHING_TO_SHOW_ON];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the list above — the tests at the bottom of
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
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "overlay", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it. Nothing here counts anything, so no plurals.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = overlay_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
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
        for word in EVERY_WORD {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().gaps().is_empty(), "{}", word.named());
        }
    }

    /// **Every word carries a note**, because none of these can be translated
    /// from its own words alone: each names the agent, and each is shown at a
    /// moment the translator has to be able to picture.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            assert!(
                word.note().unwrap().contains("not a person"),
                "{} does not say what the agent is",
                word.named()
            );
        }
    }
}
