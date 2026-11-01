//! Every string this crate can say, and the English beside each one.
//!
//! The list a person reads, the sentence they approve, what they are told
//! afterwards, and every way going back is refused. Each refusal in
//! [`crate::NotGoingBack`] has one here, because a person who asked to go back
//! and did not is the person who most needs a sentence — and `Display` on an
//! error is written for a log, not for them. [`crate::said`] is the mapping, in
//! one place, so that a variant added without a sentence does not compile.
//!
//! # Lower case, and nothing counted
//!
//! The quiet labels are lower case, which is this product's rule for them;
//! capitals inside a translatable string are an i18n fault rather than a matter
//! of style. Nothing here counts anything — a list is shown and never totalled,
//! which is also
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s second
//! term, *a person is told which, never how many*. So there is no plural in this
//! file and no gap for one.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string this crate can say — `alo-strings`' type, re-exported because
/// this crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// There is nothing kept to go back to.
pub const NOTHING_IS_KEPT: Word = Word::saying(
    "as-it-was.nothing-is-kept",
    "alo has not kept any place for you to return to on this machine",
)
.noting(
    "The ordinary state of a machine on which nobody has turned this on, and of one whose \
     filesystem cannot keep a place at all. Not a fault: it is an answer. \"alo\" is the \
     assistant's name and is never translated.",
);

/// The heading above the list of places.
pub const WHAT_YOU_CAN_RETURN_TO: Word = Word::saying(
    "as-it-was.what-you-can-return-to",
    "places you can return to, newest first",
)
.noting(
    "A heading over a list in a terminal. Lower case on purpose: a quiet label rather than a \
     title. \"newest first\" tells the reader the order so that they do not have to work it out.",
);

/// How to ask for one of them.
pub const HOW_TO_ASK: Word = Word::saying(
    "as-it-was.how-to-ask",
    "to put one folder back as it was, name a place from this list and the folder",
)
.noting(
    "Printed under the list. \"put back as it was\" rather than \"restore\": the words \
     docs/features.md promises the person, kept the same here so that they recognise what they \
     were promised.",
);

/// Something in the folder was not read and was left where it was.
pub const SOMETHING_WAS_NOT_READ: Word = Word::saying(
    "as-it-was.something-was-not-read",
    "something kept on this machine could not be read, and has been left exactly where it is: \
     {why}",
)
.noting(
    "Shown rather than hidden, because a place that cannot be read is a place a person cannot \
     return to and they would otherwise simply not see it in the list. {why} is a technical \
     sentence in English about a file, which is deliberate: it is what somebody fixing it needs.",
);

/// The place named is not one of the places kept.
pub const NO_SUCH_PLACE: Word = Word::saying(
    "as-it-was.no-such-place",
    "{named} is not one of the places kept on this machine",
)
.noting(
    "The person mistyped, or named a place that has since been let go of. {named} is what they \
     typed and is never translated.",
);

/// What going back would do — the sentence a person approves.
pub const THIS_IS_WHAT_WOULD_HAPPEN: Word = Word::saying(
    "as-it-was.this-is-what-would-happen",
    "{folder} will be as it was at {taken}, and what is in it now will be at {aside}",
)
.noting(
    "The one sentence a person approves before anything is changed, and both halves matter: what \
     goes back, and where what is there now can be found afterwards. {folder} and {aside} are \
     paths on this machine and are never translated. {taken} is a date and a time, already \
     formatted. Nothing is deleted by this act, which is what the second half says.",
);

/// How a person says yes.
pub const TYPE_ITS_NAME: Word = Word::saying(
    "as-it-was.type-its-name",
    "type the folder's own name to go on, or anything else to stop",
)
.noting(
    "The approval. Typing the folder's own name rather than \"yes\" is the same consent alo OS \
     already takes before it writes a removable drive, and it catches the other mistake as well: \
     a person who meant a different folder types a name that does not match. \"alo OS\" is the \
     product's name and is never translated.",
);

/// What was typed is not the folder's name.
pub const THAT_IS_NOT_ITS_NAME: Word = Word::saying(
    "as-it-was.that-is-not-its-name",
    "that is not the folder's name, so nothing has been changed",
)
.noting(
    "What a person reads when they typed something else, or pressed return. The second half is \
     the half they need: it says the machine is as it was.",
);

/// It went back.
pub const IT_IS_AS_IT_WAS: Word = Word::saying(
    "as-it-was.it-is-as-it-was",
    "{folder} is as it was. what was in it is at {aside}",
)
.noting(
    "Said after it worked. The second sentence is what makes this undoable in turn, and it starts \
     in lower case because it continues the same quiet report. {folder} and {aside} are paths on \
     this machine and are never translated.",
);

/// The folder named is not inside this person's home.
pub const NOT_IN_YOUR_HOME: Word = Word::saying(
    "as-it-was.not-in-your-home",
    "{folder} is not inside your own home, and this puts back nothing else",
)
.noting(
    "A refusal. The reason is the product's: a place kept is a copy of this person's home, so \
     there is nothing in it to put anywhere else. {folder} is a path on this machine and is never \
     translated.",
);

/// The folder named is the whole home.
pub const THE_WHOLE_HOME: Word = Word::saying(
    "as-it-was.the-whole-home",
    "your whole home cannot go back while you are signed in — one folder at a time can",
)
.noting(
    "A refusal, and a design decision rather than a limitation of this machine: the home is what \
     the person is working in at that moment, so swapping it under them is not something any \
     operating system does. The second half says what they can do instead.",
);

/// What was named is not a folder.
pub const NOT_A_FOLDER: Word = Word::saying(
    "as-it-was.not-a-folder",
    "{folder} is not a folder on this machine",
)
.noting(
    "The person mistyped, or named a file rather than a folder. {folder} is a path on this machine \
     and is never translated.",
);

/// The place holds no copy of that folder.
pub const NOT_IN_THAT_PLACE: Word = Word::saying(
    "as-it-was.not-in-that-place",
    "{folder} was not there when that place was kept, so there is nothing to put back",
)
.noting(
    "A refusal rather than a deletion. Putting back a folder that did not exist then would mean \
     removing it now, which is a different and larger act, and alo does not do it on the strength \
     of a request to go back. {folder} is a path on this machine and is never translated.",
);

/// The copy inside the place is a link.
pub const A_LINK_RATHER_THAN_A_FOLDER: Word = Word::saying(
    "as-it-was.a-link-rather-than-a-folder",
    "what was kept at {at} is a link rather than a folder, and this does not follow one",
)
.noting(
    "A refusal. A link inside a kept copy still points wherever it pointed, so following one would \
     read from outside the copy. {at} is a path on this machine and is never translated.",
);

/// A previous go-back was interrupted.
pub const ONE_IS_HALF_DONE: Word = Word::saying(
    "as-it-was.one-is-half-done",
    "{at} is still there from a go-back that did not finish — look at what is in it, then move it \
     out of the way",
)
.noting(
    "A refusal that asks the person to decide. alo will not remove it: whatever is in it was \
     copied out of one of their own kept places, and it is not alo's to throw away. {at} is a path \
     on this machine and is never translated.",
);

/// There is nowhere to set aside what is in the folder.
pub const NOWHERE_TO_SET_ASIDE: Word = Word::saying(
    "as-it-was.nowhere-to-set-aside",
    "{at} is already there, so what is in the folder now has nowhere to go — move it out of the \
     way first",
)
.noting(
    "A refusal, reached when two go-backs of one folder land in the same second. It is refused \
     before anything is copied, because nothing may be changed until there is somewhere for what \
     is there to go. {at} is a path on this machine and is never translated.",
);

/// The copy out of the place did not finish.
pub const NOT_COPIED: Word = Word::saying(
    "as-it-was.not-copied",
    "the folder could not be copied out of that place, and nothing has been changed: {why}",
)
.noting(
    "A refusal. The clause that matters is that nothing changed. {why} is a technical sentence in \
     English from the program that copies — the disk is full, or a permission — and it is passed \
     through rather than reworded.",
);

/// The swap did not happen.
pub const NOT_SWAPPED: Word = Word::saying(
    "as-it-was.not-swapped",
    "the folder has not been put back and nothing has been changed: {why}",
)
.noting(
    "A refusal at the one moment that has to be all-or-nothing. {why} is a technical sentence in \
     English from this machine's kernel.",
);

/// It went back and only the last rename failed.
pub const SET_ASIDE_UNDER_THE_WORKING_NAME: Word = Word::saying(
    "as-it-was.set-aside-under-the-working-name",
    "{folder} is as it was, but what was in it is still at {at} rather than where this said it \
     would be: {why}",
)
.noting(
    "Half worked, and said as such. The folder did go back; what failed is giving the old contents \
     their final name, so they are still under the name alo was building under. Nothing is lost. \
     {folder} and {at} are paths on this machine and are never translated; {why} is a technical \
     sentence in English from this machine's kernel.",
);

/// Two folders to go back together are one inside the other.
pub const ONE_INSIDE_ANOTHER: Word = Word::saying(
    "as-it-was.one-inside-another",
    "{folder} is inside another folder being put back with it, so none of them has been changed",
)
.noting(
    "Folders asked to go back together go back together or not at all, and one inside another \
     cannot: putting the outer one back carries the inner one with it. Nothing was changed. \
     {folder} is a path on this machine and is never translated.",
);

/// Putting back is always written down, and the service that writes it could
/// not be reached.
pub const NOT_WITHOUT_THE_RECORD: Word = Word::saying(
    "as-it-was.not-without-the-record",
    "the service that keeps this machine's record could not be reached, so nothing has been put \
     back: putting back is always written down. {why}",
)
.noting(
    "Said in the person's own terminal. Putting a folder back is written into the list of what \
     happened on this machine, by the one service that writes that list, and when it cannot be \
     reached nothing is changed rather than changed without a record. {why} is a technical \
     sentence in English from this machine.",
);

/// Everything this crate can say.
pub const EVERY_WORD: [Word; 21] = [
    NOTHING_IS_KEPT,
    WHAT_YOU_CAN_RETURN_TO,
    HOW_TO_ASK,
    SOMETHING_WAS_NOT_READ,
    NO_SUCH_PLACE,
    THIS_IS_WHAT_WOULD_HAPPEN,
    TYPE_ITS_NAME,
    THAT_IS_NOT_ITS_NAME,
    IT_IS_AS_IT_WAS,
    NOT_IN_YOUR_HOME,
    THE_WHOLE_HOME,
    NOT_A_FOLDER,
    NOT_IN_THAT_PLACE,
    A_LINK_RATHER_THAN_A_FOLDER,
    ONE_IS_HALF_DONE,
    NOWHERE_TO_SET_ASIDE,
    NOT_COPIED,
    NOT_SWAPPED,
    SET_ASIDE_UNDER_THE_WORKING_NAME,
    ONE_INSIDE_ANOTHER,
    NOT_WITHOUT_THE_RECORD,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above, and the tests at the bottom of this
/// file are what say so.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
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
pub fn as_it_was_words() -> Result<Vocabulary, WordsError> {
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
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::collections::BTreeSet;

    use super::{EVERY_WORD, as_it_was_words};

    /// **Every word on the list is a phrase, and no two share a key.**
    ///
    /// The two things [`WordsError`] exists for. A duplicate key is what a
    /// copied declaration leaves behind, and it is the one of the two that has
    /// actually happened in this repository.
    #[test]
    fn every_word_declares() {
        let vocabulary = as_it_was_words().unwrap();
        assert_eq!(
            vocabulary.how_many(),
            EVERY_WORD.len(),
            "a word on the list did not reach the vocabulary, which means two of them share a key"
        );
    }

    /// **Only a path, a name, a moment or a program's own sentence is ever
    /// filled in.**
    ///
    /// No sentence here has room for a count of places, a command a person
    /// typed, or anything about what they were doing. The first would be a
    /// plural ADR 0045's second term refuses; the second is something alo
    /// deliberately never learns.
    #[test]
    fn nothing_but_a_path_a_name_a_moment_or_a_reason_is_filled_in() {
        let allowed: BTreeSet<&str> = ["folder", "aside", "at", "taken", "named", "why"].into();
        for word in EVERY_WORD {
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(allowed.contains(gap.as_str()), "{}: {gap}", word.named());
            }
        }
    }

    /// **Nothing a person reads starts with a capital.**
    ///
    /// The owner's rule for these quiet labels, and in a translatable string a
    /// capital is a fault rather than a matter of style: a language that does
    /// not capitalise the way English does inherits the wrong shape from the
    /// source.
    #[test]
    fn no_sentence_starts_with_a_capital() {
        for word in EVERY_WORD {
            let text = word.phrase().unwrap().source().as_written().to_owned();
            let first = text.chars().next().unwrap();
            assert!(
                !first.is_uppercase(),
                "{} starts with {first:?}: {text}",
                word.named()
            );
        }
    }
}
