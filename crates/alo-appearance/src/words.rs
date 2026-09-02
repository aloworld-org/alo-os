//! Every string this crate can say, and the English beside each one.
//!
//! A person opens the appearance panel and reads two kinds of thing: the names
//! of the colours they are choosing between, and the sentence that comes back
//! when a value cannot be used. `CLAUDE.md` calls hardcoded English a bug, and
//! this is where the answer `alo-files` and `alo-shortcuts` gave to that (items
//! 9b and 9c) reaches the wallpaper.
//!
//! The shape is theirs: constants under one area, `alo_strings::Word` — which
//! moved into that crate in this change, rather than being written a third time
//! — and a test at the bottom putting every key back through `Key::named`.
//!
//! # Eleven of these are one word each, and that is the hard part
//!
//! Most of what a crate declares is a sentence, and a sentence carries enough of
//! itself for a translator to work from. A **colour name** does not. *Verdigris*
//! is the blue-green of weathered copper, which is two words in some languages
//! and none in others; several languages have no ordinary word for *terracotta*
//! at all and the nearest loanword may name a different colour; *rose* is a
//! flower before it is a shade of pink. These are names a person picks from a
//! list rather than reads once, so getting one wrong is not a sentence that
//! reads oddly — it is a row that does not match the colour beside it.
//!
//! So every one of the eleven carries a note that describes the colour rather
//! than assuming the word travels, and says outright that describing it is
//! allowed. That is the whole reason `alo_strings::Phrase` has a note at all:
//! `docs/autonomy/QUEUE.md` named terracotta as the example before this crate
//! existed.
//!
//! # What is deliberately not here
//!
//! **How a number is written.** [`crate::TextScale`] shows itself as `200%` and
//! [`crate::TimeOfDay`] as `18:00`, and neither is a string in this list. A time
//! written that way is what the *settings file* holds, whatever the region does;
//! how a person is shown one belongs to their region rather than to their
//! language, and a person reading Swedish in Finland writes a time the Finnish
//! way. The two refusals that carry a percentage say so in their notes, because
//! where the sign goes — and whether a space goes before it — is the one part of
//! it a translator does decide.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Lifted into `alo-strings` in this change, because this crate would have been
/// the third to write the same four fields. Re-exported here because this
/// crate's own files, and the tests that read its list, name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The colours alo OS is built out of — [`crate::Token`]. Rows in a list a
// person picks a plain background from.
// ---------------------------------------------------------------------------

/// What [`crate::Token::Navy`] is called.
pub const NAVY: Word = Word::saying("appearance.token.navy", "Navy").noting(
    "A very dark blue, near enough to black to read text on. Named after naval uniforms in \
     English; if that is not how your language names the colour, describe the colour. This is a \
     row in a list somebody picks a background from.",
);

/// What [`crate::Token::Terracotta`] is called.
pub const TERRACOTTA: Word = Word::saying("appearance.token.terracotta", "Terracotta").noting(
    "The colour of fired clay: an orange-brown. Several languages have no ordinary word for it \
     and the nearest loanword may name a different colour, so describe the colour rather than \
     borrowing the word.",
);

/// What [`crate::Token::Cream`] is called.
pub const CREAM: Word = Word::saying("appearance.token.cream", "Cream").noting(
    "A very pale off-white with a little yellow in it, named after the food. It is the ground \
     text is read on.",
);

/// What [`crate::Token::Porcelain`] is called.
pub const PORCELAIN: Word = Word::saying("appearance.token.porcelain", "Porcelain").noting(
    "A pale off-white, slightly cooler and greyer than cream, named after the china. Name the \
     material or describe the colour, whichever your readers would recognise as a colour.",
);

/// What [`crate::Token::Charcoal`] is called.
pub const CHARCOAL: Word = Word::saying("appearance.token.charcoal", "Charcoal").noting(
    "A very dark grey, named after burnt wood. Not black: it is the colour of the rail down the \
     side of the screen, which sits against black badly.",
);

/// What [`crate::Token::WarmStone`] is called.
pub const WARM_STONE: Word = Word::saying("appearance.token.warm-stone", "Warm stone").noting(
    "A grey-brown, the colour of dry stone in sunlight. Two words in English and not necessarily \
     two in yours; \"warm\" is about the colour leaning towards brown rather than about \
     temperature.",
);

// ---------------------------------------------------------------------------
// The five a person can make their machine — [`crate::Accent`]. The same
// problem as the tokens, five more times.
// ---------------------------------------------------------------------------

/// What [`crate::Accent::Verdigris`] is called.
pub const VERDIGRIS: Word = Word::saying("appearance.accent.verdigris", "Verdigris").noting(
    "The blue-green of weathered copper — a church roof, an old statue. Two words in some \
     languages and none in others; this is a colour a person picks from a list, so name it in \
     whatever way they would recognise it.",
);

/// What [`crate::Accent::Indigo`] is called.
pub const INDIGO: Word = Word::saying("appearance.accent.indigo", "Indigo").noting(
    "A deep blue with a little violet in it, named after the dye. Where the loanword names a \
     different colour in your language, describe the colour instead.",
);

/// What [`crate::Accent::Violet`] is called.
pub const VIOLET: Word = Word::saying("appearance.accent.violet", "Violet").noting(
    "A purple, named after the flower. If your word for the flower would not read as a colour in \
     a list, use your ordinary word for purple.",
);

/// What [`crate::Accent::Moss`] is called.
pub const MOSS: Word = Word::saying("appearance.accent.moss", "Moss").noting(
    "A soft green, the colour of the plant that grows on damp stone and bark. Describe the green \
     if the plant's name would not read as a colour.",
);

/// What [`crate::Accent::Rose`] is called.
pub const ROSE: Word = Word::saying("appearance.accent.rose", "Rose").noting(
    "A deep pink, named after the flower. Darker than the pink somebody might picture from the \
     word, so name the colour they would recognise beside the other four.",
);

// ---------------------------------------------------------------------------
// Why a colour is not an accent — [`crate::AccentError`]. Read by somebody in a
// settings panel wanting a colour, so each says what to choose instead.
// ---------------------------------------------------------------------------

/// Terracotta, which is the agent's and nobody else's.
pub const RESERVED: Word = Word::saying(
    "appearance.accent.reserved",
    "terracotta is how this machine says alo is present or acting, so it is not offered as a \
     personal accent — choose verdigris, indigo, violet, moss or rose",
)
.noting(
    "\"alo\" is the name of the system and is never translated. Terracotta is the colour named in \
     appearance.token.terracotta and the five at the end are the appearance.accent names: use the \
     same words for them here, or a person will be sent to a list they cannot find.",
);

/// A ground or a structure colour, asked for as an accent.
pub const NOT_AN_ACCENT: Word = Word::saying(
    "appearance.accent.not-an-accent",
    "{colour} is a ground or a structure colour rather than an accent — choose verdigris, indigo, \
     violet, moss or rose",
)
.noting(
    "{colour} arrives as one of the appearance.token names, already in your language. A ground is \
     what the shell is drawn on and a structure colour is what it is drawn with, so neither would \
     be visible as an accent.",
);

/// A colour from somewhere else entirely.
pub const NOT_OFFERED: Word = Word::saying(
    "appearance.accent.not-offered",
    "{colour} is not one of the accents this system offers — choose verdigris, indigo, violet, \
     moss or rose, each of which is drawn to read on a light ground and on a dark one",
)
.noting(
    "{colour} arrives as a hash and six hexadecimal digits, such as #123456, and is never \
     translated. The last clause is the reason there is a list rather than a colour wheel: each \
     of the five has been measured against the grounds it is drawn on.",
);

// ---------------------------------------------------------------------------
// Why a piece of text is not a colour — [`crate::ColourError`]. Read by
// somebody looking at a file they typed into.
// ---------------------------------------------------------------------------

/// Not a hash and six digits.
pub const NOT_A_COLOUR: Word = Word::saying(
    "appearance.colour.not-a-colour",
    "a colour is a hash and six hexadecimal digits, as in #102A43 — {text} is not",
)
.noting(
    "\"#102A43\" is an example of the shape and is never translated. {text} is what was actually \
     written, which came off somebody's own settings file.",
);

/// A character that is not a hexadecimal digit.
pub const NOT_A_DIGIT: Word = Word::saying(
    "appearance.colour.not-a-digit",
    "{character} is not a hexadecimal digit — a colour uses 0 to 9 and A to F, as in #102A43",
)
.noting(
    "{character} is the one character that was wrong. \"0 to 9 and A to F\" is what is written in \
     the file and is not translated; hexadecimal is the counting the digits are in.",
);

// ---------------------------------------------------------------------------
// Why a piece of text does not name a display — [`crate::DisplayError`].
// ---------------------------------------------------------------------------

/// A display with no name.
pub const DISPLAY_UNNAMED: Word = Word::saying(
    "appearance.display.unnamed",
    "name the display — it is the name the shell shows for the screen",
)
.noting(
    "A display is one screen. The name is whatever the shell calls it, such as \"DP-1 Dell \
     U2720Q\", and a person setting a background for one screen only picks it from a list.",
);

/// A name with a space at one end, which reads as the same name and is not.
pub const DISPLAY_SPACED: Word = Word::saying(
    "appearance.display.spaced",
    "{name} begins or ends with a space, and a display is matched exactly — give the name without \
     it",
)
.noting(
    "{name} arrives in quotation marks, because the whole problem is a space nobody can see. It \
     is a screen's own name and is never translated.",
);

// ---------------------------------------------------------------------------
// Why a picture cannot be a background — [`crate::PictureError`]. The person is
// in the middle of picking one, so each says what to give instead.
// ---------------------------------------------------------------------------

/// A shipped wallpaper with no name.
pub const PICTURE_UNNAMED: Word = Word::saying(
    "appearance.picture.unnamed",
    "name the wallpaper — it is how the image is asked for the picture it shipped",
)
.noting(
    "A wallpaper that came with alo OS is asked for by name rather than by where it sits on the \
     disk. \"The image\" is the installed system itself, not a picture.",
);

/// A shipped wallpaper whose name is a path.
pub const NAME_IS_A_PATH: Word = Word::saying(
    "appearance.picture.name-is-a-path",
    "{name} is a path, and a wallpaper that came with alo OS is named rather than pathed — use \
     your own picture instead if it is a file on this disk",
)
.noting(
    "{name} is what was written in the settings file and is never translated. A path says where a \
     file is; a name says which wallpaper the system shipped.",
);

/// A person's picture, given as a relative path.
pub const PICTURE_NOT_A_WHOLE_PATH: Word = Word::saying(
    "appearance.picture.not-a-whole-path",
    "give the whole path to {path}, starting from the top of the disk — where a relative path \
     leads depends on where the shell was started from",
)
.noting(
    "{path} came off somebody's own disk and is never translated. \"The top of the disk\" is the \
     root the whole path is written from.",
);

// ---------------------------------------------------------------------------
// Why a folder of pictures cannot rotate — [`crate::RotatingError`].
// ---------------------------------------------------------------------------

/// A folder given as a relative path.
pub const FOLDER_NOT_A_WHOLE_PATH: Word = Word::saying(
    "appearance.rotating.not-a-whole-path",
    "give the whole path to {folder}, starting from the top of the disk — where a relative path \
     leads depends on where the shell was started from",
)
.noting(
    "{folder} came off somebody's own disk and is never translated. It is a separate string from \
     appearance.picture.not-a-whole-path because one is a folder and one is a file, and several \
     languages word the two differently.",
);

/// A rotation faster than a minute.
pub const TOO_QUICK: Word = Word::saying(
    "appearance.rotating.too-quick",
    "leave each picture up for at least a minute — anything quicker is a flicker",
)
.noting(
    "This is about a background made of a folder of pictures that take turns. A screen changing \
     faster than that flickers, which is an accessibility problem rather than a matter of taste.",
);

// ---------------------------------------------------------------------------
// Why two times are not a schedule — [`crate::ScheduleError`].
// ---------------------------------------------------------------------------

/// The same time twice.
pub const THE_SAME_MOMENT: Word = Word::saying(
    "appearance.schedule.the-same-moment",
    "give two different times — a day that turns dark and light at {time} is a day that never \
     changes",
)
.noting(
    "{time} arrives as hours and minutes on a twenty-four hour clock, 18:00, which is how the \
     settings file holds it. Dark and light are the two schemes the screen is drawn in, not the \
     weather.",
);

// ---------------------------------------------------------------------------
// Why a size cannot be used — [`crate::TextError`]. Somebody setting this may
// be doing it because they cannot read the screen as it is.
// ---------------------------------------------------------------------------

/// Smaller than the shell can be read at.
pub const TOO_SMALL: Word = Word::saying(
    "appearance.text.too-small",
    "{percent}% is smaller than this screen can be read at — {smallest}% is as small as it goes",
)
.noting(
    "Both gaps are plain whole numbers with no sign on them, so the percent sign in this sentence \
     is yours to place: put it where your language puts it, with a space before it if that is how \
     percentages are written.",
);

/// Larger than the shell has room for.
pub const TOO_LARGE: Word = Word::saying(
    "appearance.text.too-large",
    "{percent}% is larger than the shell has room for — {largest}% is as large as it goes",
)
.noting(
    "Both gaps are plain whole numbers with no sign on them — see the note on \
     appearance.text.too-small. \"The shell\" is the system's own screen furniture, which the \
     text has to fit inside.",
);

// ---------------------------------------------------------------------------
// Why an hour and a minute are not a time of day — [`crate::TimeError`].
// ---------------------------------------------------------------------------

/// An hour past 23.
pub const NO_SUCH_HOUR: Word = Word::saying(
    "appearance.time.no-such-hour",
    "{hour} is not an hour of the day — the day runs from 0 to 23",
)
.noting(
    "{hour} is the number that was given. 0 to 23 is how the settings file counts the hours \
     whatever the region does, so those two numbers are not translated.",
);

/// A minute past 59.
pub const NO_SUCH_MINUTE: Word = Word::saying(
    "appearance.time.no-such-minute",
    "{minute} is not a minute of the hour — an hour runs from 0 to 59",
)
.noting("{minute} is the number that was given. 0 to 59 is not translated.");

/// Every string this crate can say, in the order a translator meets them: the
/// colours the system is built out of, the five a person can choose, and then
/// the refusals, file by file.
pub const EVERY_WORD: [Word; 28] = [
    NAVY,
    TERRACOTTA,
    CREAM,
    PORCELAIN,
    CHARCOAL,
    WARM_STONE,
    VERDIGRIS,
    INDIGO,
    VIOLET,
    MOSS,
    ROSE,
    RESERVED,
    NOT_AN_ACCENT,
    NOT_OFFERED,
    NOT_A_COLOUR,
    NOT_A_DIGIT,
    DISPLAY_UNNAMED,
    DISPLAY_SPACED,
    PICTURE_UNNAMED,
    NAME_IS_A_PATH,
    PICTURE_NOT_A_WHOLE_PATH,
    FOLDER_NOT_A_WHOLE_PATH,
    TOO_QUICK,
    THE_SAME_MOMENT,
    TOO_SMALL,
    TOO_LARGE,
    NO_SUCH_HOUR,
    NO_SUCH_MINUTE,
];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the list above — the tests at the bottom of this
/// file are what say so. It is a `Result` rather than an unwrap because a
/// library that panics on its own string table takes the shell with it, and
/// because [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
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
pub fn appearance_words() -> Result<Vocabulary, WordsError> {
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
    /// arrive from anywhere; this is the test that makes that true, and it is
    /// the same shape as this crate putting its shipped wallpaper back through
    /// `Picture::shipped`.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}: {}",
                word.named(),
                alo_strings::Key::named(word.named()).unwrap_err()
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

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "appearance", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    /// Nothing here counts anything, so the vocabulary holds no plurals.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = appearance_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = appearance_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **A colour is a name, and a name has nothing to fill in.** A label with a
    /// gap in it would be a row in a picker with `{}` printed in the middle of
    /// it, which is the failure a person choosing a colour would see first.
    #[test]
    fn every_colour_name_names_nothing() {
        for word in [
            NAVY, TERRACOTTA, CREAM, PORCELAIN, CHARCOAL, WARM_STONE, VERDIGRIS, INDIGO, VIOLET,
            MOSS, ROSE,
        ] {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().gaps().is_empty(), "{}", word.named());
        }
    }

    /// **Every refusal that is about a particular value names it.** A sentence
    /// saying only that something was wrong leaves a person editing a file to
    /// find out which line by deleting them one at a time.
    #[test]
    fn the_refusals_that_are_about_something_have_a_gap_for_it() {
        for (word, gap) in [
            (NOT_AN_ACCENT, "colour"),
            (NOT_OFFERED, "colour"),
            (NOT_A_COLOUR, "text"),
            (NOT_A_DIGIT, "character"),
            (DISPLAY_SPACED, "name"),
            (NAME_IS_A_PATH, "name"),
            (PICTURE_NOT_A_WHOLE_PATH, "path"),
            (FOLDER_NOT_A_WHOLE_PATH, "folder"),
            (THE_SAME_MOMENT, "time"),
            (TOO_SMALL, "percent"),
            (TOO_LARGE, "percent"),
            (NO_SUCH_HOUR, "hour"),
            (NO_SUCH_MINUTE, "minute"),
        ] {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().has(gap), "{} wants {gap}", word.named());
        }

        // And the two that say the range say both ends of it.
        assert!(TOO_SMALL.phrase().unwrap().source().has("smallest"));
        assert!(TOO_LARGE.phrase().unwrap().source().has("largest"));
    }

    /// The three refusals that are about the whole rule rather than about a
    /// value deliberately have no gap: repeating what was asked for back at
    /// somebody reads like an instruction to use it.
    #[test]
    fn the_refusals_about_a_rule_name_nothing() {
        for word in [RESERVED, DISPLAY_UNNAMED, PICTURE_UNNAMED, TOO_QUICK] {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().gaps().is_empty(), "{}", word.named());
        }
    }

    /// **Every string in this crate carries a note**, which is not true of the
    /// other two crates that declare words and is the thing that makes this list
    /// different: eleven of them are one word naming a colour, and the rest
    /// carry either a value that must not be translated or a number whose sign a
    /// translator has to place.
    #[test]
    fn every_word_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
        assert!(
            TERRACOTTA
                .note()
                .is_some_and(|note| note.contains("orange-brown")),
            "the colour is described rather than named"
        );
        assert!(
            VERDIGRIS
                .note()
                .is_some_and(|note| note.contains("weathered copper"))
        );
    }
}
