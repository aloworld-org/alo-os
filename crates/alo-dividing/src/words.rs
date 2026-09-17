//! Every string this crate can say, and the English beside each one.
//!
//! Two kinds: what a proposed place on the screen is called — the label a drop
//! shows before it is committed, and what a screen reader announces while a
//! window is dragged — and why a division was left as it was.
//!
//! The shape is `alo-shortcuts`' and is copied rather than re-decided.
//!
//! # No sentence here names a window
//!
//! A refusal is about a window, and none of these sentences say which. A
//! division knows a window only as a number the compositor gave it (see
//! [`crate::window`]), so the only name this crate could put in a sentence is
//! one it would have to be handed — a title, which is exactly what this plan
//! keeps out of a division. The shell marks the window beside the sentence
//! instead, through [`crate::Refused::window`].

use alo_strings::Vocabulary;

/// One string a crate can say.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Where a dragged window would go — [`crate::Place`]. Labels, shown while the
// window is still held so a person can let go somewhere else.
// ---------------------------------------------------------------------------

/// [`crate::Place::LeftHalf`].
pub const LEFT_HALF: Word = Word::saying("dividing.place.left-half", "Left half").noting(
    "Shown while a person drags a window to the left edge of the screen: letting go puts the \
     window on the left half of the screen. Nothing has moved yet when this is shown.",
);

/// [`crate::Place::RightHalf`].
pub const RIGHT_HALF: Word = Word::saying("dividing.place.right-half", "Right half")
    .noting("The right half of the screen — see the note on dividing.place.left-half.");

/// [`crate::Place::TopHalf`].
pub const TOP_HALF: Word = Word::saying("dividing.place.top-half", "Top half")
    .noting("The top half of the screen — see the note on dividing.place.left-half.");

/// [`crate::Place::BottomHalf`].
pub const BOTTOM_HALF: Word = Word::saying("dividing.place.bottom-half", "Bottom half")
    .noting("The bottom half of the screen — see the note on dividing.place.left-half.");

/// [`crate::Place::TopLeftQuarter`].
pub const TOP_LEFT_QUARTER: Word =
    Word::saying("dividing.place.top-left-quarter", "Top-left quarter").noting(
        "Shown while a person drags a window to a corner of the screen: letting go puts the \
         window in that quarter of the screen. A quarter is a fourth of the whole screen.",
    );

/// [`crate::Place::TopRightQuarter`].
pub const TOP_RIGHT_QUARTER: Word =
    Word::saying("dividing.place.top-right-quarter", "Top-right quarter")
        .noting("See the note on dividing.place.top-left-quarter.");

/// [`crate::Place::BottomLeftQuarter`].
pub const BOTTOM_LEFT_QUARTER: Word =
    Word::saying("dividing.place.bottom-left-quarter", "Bottom-left quarter")
        .noting("See the note on dividing.place.top-left-quarter.");

/// [`crate::Place::BottomRightQuarter`].
pub const BOTTOM_RIGHT_QUARTER: Word = Word::saying(
    "dividing.place.bottom-right-quarter",
    "Bottom-right quarter",
)
.noting("See the note on dividing.place.top-left-quarter.");

/// [`crate::Place::Part`].
pub const PART: Word = Word::saying("dividing.place.part", "Part of the screen").noting(
    "Shown when letting go would give the window a piece of the screen that is neither a half \
     nor a quarter, such as half of a quarter. The outline drawn on the screen shows which piece.",
);

// ---------------------------------------------------------------------------
// Why the screen was left as it was — [`crate::Refused`]. Each says what did
// not happen, because a person reading it has just tried to move something.
// ---------------------------------------------------------------------------

/// [`crate::Refused::NotDivided`].
pub const NOT_DIVIDED: Word = Word::saying(
    "dividing.refused.not-divided",
    "this window is not part of the divided screen, so nothing has moved — drag it to an edge of \
     the screen to add it",
)
.noting(
    "A divided screen is one where windows share the screen side by side or one above the other. \
     This window is floating over them rather than being one of them.",
);

/// [`crate::Refused::NothingToShareWith`].
pub const NOTHING_TO_SHARE_WITH: Word = Word::saying(
    "dividing.refused.nothing-to-share-with",
    "there is no other window open to share the screen with, so nothing has moved",
)
.noting("Said when a person tries to divide the screen while only one window is open.");

/// [`crate::Refused::NoNeighbour`].
pub const NO_NEIGHBOUR: Word = Word::saying(
    "dividing.refused.no-neighbour",
    "that edge is the edge of the screen, so there is no window beside it to make room for",
)
.noting(
    "Said when a person drags the outer edge of a window that fills a share of the screen. Only \
     an edge between two windows can be moved, and moving it makes one larger and the other \
     smaller.",
);

/// [`crate::Refused::TooNarrow`].
pub const TOO_NARROW: Word = Word::saying(
    "dividing.refused.too-narrow",
    "this window cannot be made that narrow, so the screen has been left as it was",
)
.noting(
    "The application has said how narrow it can be drawn, and its share of the screen would have \
     been narrower. alo OS does not squeeze it or let it cover its neighbour; the window is marked \
     beside this sentence.",
);

/// [`crate::Refused::TooShort`].
pub const TOO_SHORT: Word = Word::saying(
    "dividing.refused.too-short",
    "this window cannot be made that short, so the screen has been left as it was",
)
.noting(
    "Short as in height, top to bottom — see the note on dividing.refused.too-narrow, which is \
     the same sentence about width.",
);

/// [`crate::Refused::Changed`].
pub const CHANGED: Word = Word::saying(
    "dividing.refused.changed",
    "the screen changed before the window was let go, so nothing has moved — drag it again",
)
.noting(
    "Said when another window opened, closed or moved while a person was dragging, so where the \
     outline showed the window would go is no longer true.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 15] = [
    LEFT_HALF,
    RIGHT_HALF,
    TOP_HALF,
    BOTTOM_HALF,
    TOP_LEFT_QUARTER,
    TOP_RIGHT_QUARTER,
    BOTTOM_LEFT_QUARTER,
    BOTTOM_RIGHT_QUARTER,
    PART,
    NOT_DIVIDED,
    NOTHING_TO_SHARE_WITH,
    NO_NEIGHBOUR,
    TOO_NARROW,
    TOO_SHORT,
    CHANGED,
];

/// Why this crate's own words could not be declared.
///
/// Nothing in the list above can cause one; the tests at the bottom of this file
/// are what say so.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
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
pub fn dividing_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
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

    /// Every key written here is a key by the rule every other key is held to.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "dividing", "{}", word.named());
        }
    }

    /// No two words share a key, and the whole list declares.
    #[test]
    fn the_whole_list_declares_and_no_two_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let vocabulary = dividing_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        let again = declare_into(&mut dividing_words().unwrap()).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// Every word carries a note, and none has a gap: nothing here is filled in.
    #[test]
    fn every_word_carries_a_note_and_names_nothing() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| !note.trim().is_empty()),
                "{}",
                word.named()
            );
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{}",
                word.named()
            );
        }
    }
}
