//! Every string this crate can say, and the English beside each one.
//!
//! Eight, in two groups.
//!
//! **Five are the ways a paste does not happen.** Every one of them is a moment
//! where a person pressed a key and nothing appeared, and a machine that lets
//! that happen in silence is indistinguishable from one that is broken —
//! `crates/alo-overlay` made the same argument about a key that summons
//! nothing. So each of the five says which of the five it was: *nothing has been
//! copied* and *what you copied has been copied over* send a person to two
//! different places, and one sentence covering both would send them to neither.
//!
//! **Three are the forms the promise names.** `docs/features.md` says *text,
//! images and files*, and those three words are what a person recognises; the
//! media type underneath is not. They are the only strings here that are not
//! whole sentences, because each is read **inside** [`NOT_THAT_FORM`]'s gap, and
//! the note on each says so — a translator handed the word *text* with no
//! context cannot know whether it is a heading, a button or the middle of a
//! sentence, and in a language that inflects it is three different words.
//!
//! # There is no word here for what was copied
//!
//! Nothing in this crate ever says what is on the clipboard. Not a preview, not
//! a first line, not a byte count. A person's clipboard holds the password they
//! just moved out of a manager, and a sentence quoting it would put it in a
//! notification, a log, and every screen recording taken while it was up.
//! `alo-context` refuses to write down what was on somebody's screen for the
//! same reason, and the test at the bottom of this file is what keeps the next
//! string added here to the rule.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The five ways a paste does not happen — [`crate::NotPasted`].
// ---------------------------------------------------------------------------

/// Nothing has ever been copied, so there is nothing to paste from.
pub const NOTHING_COPIED: Word = Word::saying(
    "clipboard.nothing-copied",
    "There is nothing to paste: nothing has been copied yet",
)
.noting(
    "Shown when somebody pastes and nothing has been copied on this machine since it started. It \
     is deliberately not \"the clipboard is empty\" — a person who has just copied something in \
     another window would read that as their copy having been lost. It says what is so: no copy \
     has been made.",
);

/// The form asked for is not one the owner offered.
pub const NOT_THAT_FORM: Word = Word::saying(
    "clipboard.not-that-form",
    "This cannot be pasted as {as_what}: the application it was copied from did not offer it in \
     that form",
)
.noting(
    "Shown when somebody pastes into a place that can only accept a form the copier did not \
     offer — an image into a text field, say. The gap is a form, and arrives already translated: \
     it is one of \"text\", \"an image\" or \"files\" from this same file, or a media type such as \
     \"application/pdf\" which is a technical name and is never translated. The sentence must read \
     as a fact about the two applications rather than as a fault of the person: nothing they did \
     was wrong, and nothing they can do here will change it.",
);

/// Something else has been copied since, so the offer is gone.
pub const COPIED_OVER: Word = Word::saying(
    "clipboard.copied-over",
    "What you copied is no longer there to paste: something else has been copied since",
)
.noting(
    "Shown when somebody pastes a selection that has been replaced — they copied, then copied \
     something else in another window, and came back to the first. It says which of the two \
     things happened, because the repair is different: this one is copy it again.",
);

/// The owner has given the selection up.
pub const NO_LONGER_OFFERED: Word = Word::saying(
    "clipboard.no-longer-offered",
    "What you copied is no longer there to paste: the application it was copied from has given it \
     up",
)
.noting(
    "Shown when somebody pastes a selection whose owner has withdrawn it — usually because that \
     application was closed. On alo OS what a person copied lives in the application they copied \
     it from until they paste it, so closing that application really does take it away; the \
     sentence says so plainly rather than implying the machine lost it.",
);

/// The owner was asked and did not hand it over.
pub const COULD_NOT_GIVE: Word = Word::saying(
    "clipboard.could-not-give",
    "The application this was copied from did not hand it over. Try copying it again",
)
.noting(
    "Shown when the application that copied something was asked for it and the transfer did not \
     complete — it stopped part way, or the application quit while it was happening. The second \
     sentence is the only advice this crate gives anybody and it is the whole of what there is to \
     do.",
);

// ---------------------------------------------------------------------------
// The three forms the promise names — [`crate::Form`].
// ---------------------------------------------------------------------------

/// Text of any flavour, as a person would name it.
pub const THE_FORM_TEXT: Word = Word::saying("clipboard.form.text", "text").noting(
    "The name of a form something can be pasted in, read inside another sentence: \"This \
         cannot be pasted as {as_what}\". Not a sentence and not a heading — it is lowercase \
         because it sits in the middle of one, and it should be whatever a person in this \
         language calls written characters as a kind of content, rather than a technical word.",
);

/// A picture, as a person would name it.
pub const THE_FORM_AN_IMAGE: Word = Word::saying("clipboard.form.an-image", "an image").noting(
    "The name of a form something can be pasted in, read inside another sentence: \"This \
         cannot be pasted as {as_what}\". The English carries its article because the sentence \
         around it reads \"pasted as an image\"; a language that handles this with a case ending \
         or no article at all should do that instead of translating the word \"an\". A picture of \
         any sort — a photograph, a drawing, a screenshot.",
);

/// A set of files, as a person would name it.
pub const THE_FORM_FILES: Word = Word::saying("clipboard.form.files", "files").noting(
    "The name of a form something can be pasted in, read inside another sentence: \"This \
         cannot be pasted as {as_what}\". It names the kind of content — files rather than their \
         contents — and is not a count: it is said the same way whether one file was copied or \
         twenty, so a language that would otherwise need a number should use whatever form names \
         the category.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// There is no counted one, and there is deliberately nothing here that says
/// how much was copied: a number would mean this crate had looked at what is on
/// the clipboard, which is the one thing it never does.
pub const EVERY_WORD: [Word; 8] = [
    NOTHING_COPIED,
    NOT_THAT_FORM,
    COPIED_OVER,
    NO_LONGER_OFFERED,
    COULD_NOT_GIVE,
    THE_FORM_TEXT,
    THE_FORM_AN_IMAGE,
    THE_FORM_FILES,
];

/// The gap in [`NOT_THAT_FORM`], which is the only gap this crate has.
pub const AS_WHAT: &str = "as_what";

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so. It is a `Result` rather than an unwrap because a library
/// that panics on its own string table takes the shell with it, and because
/// [`declare_into`] can genuinely fail against a vocabulary that already holds
/// one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn clipboard_words() -> Result<Vocabulary, WordsError> {
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

    /// The five sentences, apart from the three form names, which are read
    /// inside one of them.
    const THE_SENTENCES: [Word; 5] = [
        NOTHING_COPIED,
        NOT_THAT_FORM,
        COPIED_OVER,
        NO_LONGER_OFFERED,
        COULD_NOT_GIVE,
    ];

    /// The three form names.
    const THE_FORMS: [Word; 3] = [THE_FORM_TEXT, THE_FORM_AN_IMAGE, THE_FORM_FILES];

    /// **What we ship is held to the rule everybody else is held to.**
    /// `Word::key` does not check, because a key written in this file cannot
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
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "clipboard", "{}", word.named());
        }
    }

    /// The two groups are the whole list, so a word added to one of them
    /// without a test being told about it fails here.
    #[test]
    fn the_two_groups_are_the_whole_list() {
        let mut both: Vec<&str> = THE_SENTENCES.iter().map(Word::named).collect();
        both.extend(THE_FORMS.iter().map(Word::named));
        both.sort_unstable();
        let mut every: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        every.sort_unstable();
        assert_eq!(both, every);
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(clipboard_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = clipboard_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **The only gap in this crate is the form**, and it is in the one
    /// sentence that names one. A gap anywhere else would be somewhere this
    /// crate could put what was copied, which is the one thing it must never
    /// say.
    #[test]
    fn the_only_gap_is_the_form() {
        for word in EVERY_WORD {
            let gaps = word.phrase().unwrap().source().gaps().to_vec();
            if word.named() == NOT_THAT_FORM.named() {
                assert_eq!(gaps, [AS_WHAT.to_owned()], "{}", word.named());
            } else {
                assert!(gaps.is_empty(), "{} has a gap in it", word.named());
            }
        }
    }

    /// **Nothing here quotes what was copied.** A clipboard holds the password
    /// somebody just moved out of a manager; a sentence with a preview in it
    /// would put that into a notification and into every screen recording taken
    /// while it was up. The notes are searched as well as the sentences,
    /// because a note is what a translator writes the sentence from.
    #[test]
    fn nothing_here_quotes_what_was_copied() {
        for word in EVERY_WORD {
            let read =
                format!("{} {}", word.says(), word.note().unwrap_or_default()).to_ascii_lowercase();
            for showing in [
                "preview",
                "first line",
                "begins with",
                "starts with \"",
                "contents of the clipboard",
                "characters long",
            ] {
                assert!(
                    !read.contains(showing),
                    "{} says \"{showing}\", which is this crate showing what was copied",
                    word.named()
                );
            }
        }
    }

    /// **Nothing here counts anything.** A number written into an English
    /// sentence cannot be translated into a language with three plural forms —
    /// and the only number this crate could count is how much was copied, which
    /// it must not look at.
    #[test]
    fn nothing_here_counts_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|letter| letter.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }

    /// **Every word carries a note.** None of the eight can be translated from
    /// its own words: the five are read at a moment a translator has to be able
    /// to picture, and the three are single words read inside somebody else's
    /// sentence, which is where a translation goes wrong silently.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **The five stand on their own and the three do not.** A refusal is read
    /// by a screen reader with nothing above it, so it is a capitalised
    /// sentence; a form name is read inside one, so it is not.
    #[test]
    fn the_sentences_begin_a_sentence_and_the_forms_do_not() {
        for word in THE_SENTENCES {
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_uppercase(),
                "{} does not begin a sentence",
                word.named()
            );
        }
        for word in THE_FORMS {
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_lowercase(),
                "{} is read inside a sentence and begins one",
                word.named()
            );
        }
    }

    /// **Every form name's note says it is read inside another sentence.** It
    /// is the one thing a translator cannot see from the string, and getting it
    /// wrong produces a capitalised word in the middle of a line in every
    /// language that was translated carefully.
    #[test]
    fn every_form_name_says_it_is_read_inside_a_sentence() {
        for word in THE_FORMS {
            assert!(
                word.note()
                    .is_some_and(|note| note.contains("inside another sentence")),
                "{} does not tell a translator where it is read",
                word.named()
            );
        }
    }
}
