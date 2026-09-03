//! Every string this crate can say, and the English beside each one.
//!
//! Nine strings, and they fall into three groups a person meets in three
//! different places: the four edges, which are a list in Settings; what the dock
//! is doing with its names, which is a line under that list; and the two ways a
//! screen can fail to be one, which is read by whoever is looking at a machine
//! that reported something impossible.
//!
//! The shape is `alo-appearance`'s, one crate on: constants, `alo_strings::Word`,
//! and a test at the bottom putting every key back through `Key::named`.
//!
//! # The one that matters is the one nobody plans for
//!
//! [`NAMES_GAVE_WAY`] is read by somebody who has just made their text bigger
//! for a reason and watched the names in their dock disappear. It is the string
//! in this crate a bad translation would do the most damage to, because the half
//! that matters is not *why* — it is **the name is still there**: resting on an
//! icon still gives it, and a screen reader still reads it. A translation that
//! kept the reason and dropped the reassurance would leave somebody believing
//! that making text bigger had cost them the ability to tell their applications
//! apart. Its note says so outright.
//!
//! # What is deliberately not here
//!
//! **How a number is written.** The percentage and the two screen measurements
//! arrive as plain whole numbers with no sign or separator on them, exactly as
//! `alo_appearance::TextScale`'s two refusals do, because how a number is
//! written belongs to the region rather than to the language. Where the percent
//! sign goes *is* the translator's, so it is inside the sentence.

use alo_strings::Vocabulary;

/// One string a crate can say. Re-exported so this crate's own files name it as
/// `crate::words::Word`, as `alo-appearance` does.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The four edges — [`crate::Edge`]. A list of four in Settings, and the whole of
// what a person chooses about the dock at v0.01.
// ---------------------------------------------------------------------------

/// What [`crate::Edge::Bottom`] is called.
pub const BOTTOM: Word = Word::saying("dock.edge.bottom", "Bottom").noting(
    "One of four rows in a list a person picks the dock's position from. It names the bottom edge \
     of the screen, not a direction to move something in.",
);

/// What [`crate::Edge::Left`] is called.
pub const LEFT: Word = Word::saying("dock.edge.left", "Left").noting(
    "One of four rows in a list a person picks the dock's position from — the left edge of the \
     screen. It is the physical left of the screen and does not swap around in a language read \
     right to left.",
);

/// What [`crate::Edge::Right`] is called.
pub const RIGHT: Word = Word::saying("dock.edge.right", "Right").noting(
    "One of four rows in a list a person picks the dock's position from — the right edge of the \
     screen. As with the left, it is the physical side of the screen.",
);

/// What [`crate::Edge::Top`] is called.
pub const TOP: Word = Word::saying("dock.edge.top", "Top").noting(
    "One of four rows in a list a person picks the dock's position from — the top edge of the \
     screen.",
);

// ---------------------------------------------------------------------------
// What the dock is doing with its names — [`crate::Labels`]. One line under the
// list above, so that choosing an edge shows what it did.
// ---------------------------------------------------------------------------

/// A dock that runs across the screen, with room for its names.
pub const NAMES_UNDER: Word = Word::saying("dock.labels.under", "each icon has its name under it")
    .noting(
        "The dock is along the bottom or the top of the screen, so a name sits below the picture \
         it belongs to. Shown in Settings under the list of edges, so somebody can see what \
         choosing an edge did.",
    );

/// A dock that runs down the screen, with room for its names.
pub const NAMES_BESIDE: Word =
    Word::saying("dock.labels.beside", "each icon has its name beside it").noting(
        "The dock is down the left or the right of the screen, so a name sits next to the picture \
         it belongs to and still reads the ordinary way round — it is never turned on its side.",
    );

/// A dock with no room for names at the size the text has been set to.
pub const NAMES_GAVE_WAY: Word = Word::saying(
    "dock.labels.gave-way",
    "there is no room for names at {percent}% text, so the dock shows icons — resting on one \
     still gives its name, and a screen reader still reads it",
)
.noting(
    "Read by somebody who has just made their text bigger and watched the names in their dock \
     disappear, so the second half is the half that matters: nothing has been taken away, the \
     names have moved. Keep it. {percent} is a plain whole number with no sign on it, so the \
     percent sign is yours to place — put it where your language puts it, with a space before it \
     if that is how percentages are written.",
);

// ---------------------------------------------------------------------------
// Why something is not a screen — [`crate::ScreenError`]. Read by whoever is
// looking at a machine that reported an impossible display.
// ---------------------------------------------------------------------------

/// A screen with no width or no height.
pub const NOT_A_SCREEN: Word = Word::saying(
    "dock.screen.not-a-screen",
    "a screen has a width and a height — {width} by {height} is not one",
)
.noting(
    "{width} and {height} are plain whole numbers with nothing on them, and one of the two is \
     zero. \"by\" joins two measurements, as in \"1366 by 768\"; use whatever your language puts \
     between the two numbers of a size.",
);

/// A screen too small for a dock to sit on without taking it.
pub const SCREEN_TOO_SMALL: Word = Word::saying(
    "dock.screen.too-small",
    "{width} by {height} is smaller than alo OS lays out for — a screen needs at least {least} \
     each way, or the dock would take the screen rather than sit on it",
)
.noting(
    "All three gaps are plain whole numbers. \"by\" joins two measurements — see the note on \
     dock.screen.not-a-screen. \"alo OS\" is the name of the system and is never translated.",
);

/// Every string this crate can say, in the order a translator meets them: the
/// four edges a person picks between, what the dock did with its names, and then
/// the two refusals.
pub const EVERY_WORD: [Word; 9] = [
    BOTTOM,
    LEFT,
    RIGHT,
    TOP,
    NAMES_UNDER,
    NAMES_BESIDE,
    NAMES_GAVE_WAY,
    NOT_A_SCREEN,
    SCREEN_TOO_SMALL,
];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the list above — the tests at the bottom of this
/// file are what say so. It is a `Result` rather than an unwrap because a
/// library that panics on its own string table takes the shell with it, and
/// because [`declare_into`] can genuinely fail against a vocabulary that already
/// holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
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
pub fn dock_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The shell has one vocabulary and every crate adds its own to it, which is
/// what the area at the front of a key is for.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
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
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what lets
    /// one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "dock", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it. Nothing here counts anything, so the vocabulary holds no
    /// plurals.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = dock_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = dock_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **An edge is a name, and a name has nothing to fill in.** A row in a
    /// picker with `{}` printed in the middle of it is the failure a person
    /// choosing where their dock goes would see first.
    #[test]
    fn every_edge_names_nothing() {
        for word in [BOTTOM, LEFT, RIGHT, TOP] {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{word:?}"
            );
        }
    }

    /// Every string that is about a particular value names it. A sentence saying
    /// only that something was wrong leaves whoever is reading it to guess which
    /// number was the problem.
    #[test]
    fn the_strings_that_are_about_something_have_a_gap_for_it() {
        for (word, gaps) in [
            (NAMES_GAVE_WAY, &["percent"][..]),
            (NOT_A_SCREEN, &["width", "height"][..]),
            (SCREEN_TOO_SMALL, &["width", "height", "least"][..]),
        ] {
            let phrase = word.phrase().unwrap();
            for gap in gaps {
                assert!(phrase.source().has(gap), "{} wants {gap}", word.named());
            }
        }
    }

    /// **The reassurance in [`NAMES_GAVE_WAY`] is part of the string**, not a
    /// thing a shell adds beside it — so a translator is handed it and a checked
    /// translation cannot lose it without somebody deciding to.
    #[test]
    fn the_string_about_names_disappearing_says_the_name_is_still_there() {
        assert!(NAMES_GAVE_WAY.says().contains("screen reader"));
        assert!(NAMES_GAVE_WAY.says().contains("still gives its name"));
        assert!(
            NAMES_GAVE_WAY
                .note()
                .is_some_and(|note| note.contains("Keep it")),
            "the note tells a translator which half matters"
        );
    }

    /// Every string here carries a note. Four of them are one word naming an
    /// edge, and a word with no note is where a translation goes wrong quietly.
    #[test]
    fn every_word_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }
}
