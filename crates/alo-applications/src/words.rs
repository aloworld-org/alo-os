//! Every string this crate can say, and the English beside each one.
//!
//! The same shape as `alo-files`' list, and for the same reason: somebody has
//! to write the first sentence and it is written in the language this code is
//! written in, but **no sentence reaches a person without something having
//! asked whether anybody translated it**. `alo-strings` is the machinery;
//! this file is what this crate hands it.
//!
//! **The four verbs are declared from here, not copied from here.**
//! [`crate::verbs`] passes these constants to `alo_capability::Verb::checked`,
//! so the sentence a person approves and the sentence a translator is handed
//! are one string rather than two a test hopes are equal (item 9g).
//!
//! # The gap that is never translated
//!
//! Every sentence here has `{application}` in it, and it is filled with an
//! identifier off this machine — `org.blender.Blender` — never with the name a
//! person would say. That is the decision [`crate::Application`] exists to
//! carry, and it is written into the note on each of those strings, because a
//! translator with no product in front of them would otherwise reasonably
//! assume the gap holds a word they should have translated around.
//!
//! # And the one gap that arrives already translated
//!
//! `arrange_application`'s `{where}` is the other kind, and this crate is the
//! first to have one: it holds an *arrangement*, which is one of the three
//! strings at the bottom of this file rendered in the reader's own language
//! (item 11a). So the three are written as they will read **inside** the
//! sentence rather than as labels — the preposition lives in the arrangement
//! where a translator can move it, which is `alo-egress`' decision about its
//! indicator line met from the other side.
//!
//! A sentence and a word put into it that were translated by two people at two
//! times can disagree, and nothing here can stop that. What alo OS does instead
//! is refuse to hide it: a sentence holding a word nobody has translated
//! answers `alo_strings::Said::is_translated` with `false`, so a machine
//! showing a German approval with an English arrangement in it is a fact
//! something can see.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read its
/// list, name it as `crate::words::Word`. It lives in `alo-strings` since item
/// 9d.
pub use alo_strings::Word;

/// What a translator has to be told about the one gap that is not a word.
const AN_IDENTIFIER: &str = "{application} is the identifier this machine knows an application by, \
                             like org.blender.Blender. It is never translated, and it is not the \
                             name a person would say — the name is shown beside it, not inside \
                             this sentence.";

// ---------------------------------------------------------------------------
// What this crate says when it cannot reach an application.
// ---------------------------------------------------------------------------

/// Nothing on this machine goes by that identifier.
///
/// The one sentence here that a person meets while an agent is working, and it
/// is worded here rather than by the grants because only this crate holds the
/// list of what is installed. It travels into the record as it was said.
pub const NOT_INSTALLED: Word = Word::saying(
    "applications.not-installed",
    "nothing installed on this machine is {application} — install it, or name one that is here",
)
.noting(AN_IDENTIFIER);

// ---------------------------------------------------------------------------
// What could not join the list of what is installed — [`crate::NotAnApplication`].
// ---------------------------------------------------------------------------

/// An entry with no identifier at all.
pub const NO_IDENTIFIER: Word = Word::saying(
    "applications.entry.no-identifier",
    "an application with no identifier cannot be reached — an identifier is what a grant is made \
     over and what a verb names",
);

/// An entry whose identifier could never be named by a verb.
pub const NOT_AN_IDENTIFIER: Word = Word::saying(
    "applications.entry.not-an-identifier",
    "{application} is not an identifier — an identifier has no spaces and no folders in it, so \
     this one could never arrive as an argument",
)
.noting(AN_IDENTIFIER);

// ---------------------------------------------------------------------------
// How an application is shown, when a shell shows one.
// ---------------------------------------------------------------------------

/// One application, as a list of them is read.
pub const CALLED: Word = Word::saying("applications.called", "{called} ({application})").noting(
    "{called} is the name the application gives itself, in whatever language it was packaged in, \
     and is not ours to translate. {application} is the identifier — the thing a grant is made \
     over. Both are shown because two applications can call themselves the same thing and no two \
     share an identifier. Move the brackets if your language writes them differently.",
);

// ---------------------------------------------------------------------------
// The four verbs: what each does, what a person approves, and what its
// argument is for. `crate::verbs` declares them from these.
// ---------------------------------------------------------------------------

/// What `open_application` does.
pub const OPEN_APPLICATION: Word = Word::saying(
    "applications.verb.open-application.purpose",
    "start an application, so it is running and on the screen",
);
/// **The sentence a person approves before an application is started.**
pub const OPEN_APPLICATION_SENTENCE: Word = Word::saying(
    "applications.verb.open-application.sentence",
    "open {application}",
)
.noting(AN_IDENTIFIER);
/// `open_application`'s only argument.
pub const OPEN_APPLICATION_APPLICATION: Word = Word::saying(
    "applications.verb.open-application.argument.application",
    "which application to start",
);

/// What `focus_application` does.
pub const FOCUS_APPLICATION: Word = Word::saying(
    "applications.verb.focus-application.purpose",
    "bring an application that is already running to the front",
);
/// **The sentence a person approves before what is in front of them changes.**
pub const FOCUS_APPLICATION_SENTENCE: Word = Word::saying(
    "applications.verb.focus-application.sentence",
    "bring {application} to the front",
)
.noting(AN_IDENTIFIER);
/// `focus_application`'s only argument.
pub const FOCUS_APPLICATION_APPLICATION: Word = Word::saying(
    "applications.verb.focus-application.argument.application",
    "which application to bring to the front",
);

/// What `close_application` does.
pub const CLOSE_APPLICATION: Word = Word::saying(
    "applications.verb.close-application.purpose",
    "ask an application to close, exactly as pressing its close button does",
);
/// **The sentence a person approves before an application is asked to close.**
///
/// *Ask* is the word the whole decision rests on, and it is in the string a
/// person reads rather than only in this crate's documentation. See
/// [`crate::verbs`] for why nothing here kills anything.
pub const CLOSE_APPLICATION_SENTENCE: Word = Word::saying(
    "applications.verb.close-application.sentence",
    "ask {application} to close",
)
.noting(
    "\"Ask\" is not politeness and must survive translation: this verb does what pressing the \
     close button does, so an application with unsaved work still gets to put its own question up \
     and the person still answers it. A translation reading \"close {application}\" would promise \
     something alo OS deliberately does not do. {application} is the identifier this machine knows \
     an application by, and is never translated.",
);
/// `close_application`'s only argument.
pub const CLOSE_APPLICATION_APPLICATION: Word = Word::saying(
    "applications.verb.close-application.argument.application",
    "which application to ask to close",
);

/// What `arrange_application` does.
pub const ARRANGE_APPLICATION: Word = Word::saying(
    "applications.verb.arrange-application.purpose",
    "put an application's window somewhere on the screen",
);
/// **The sentence a person approves before a window moves.**
///
/// The one sentence in this crate with two gaps in it, and they are different
/// in kind: `{application}` is an identifier off this machine and `{where}` is
/// one of the three arrangements below, which arrives **already translated**.
/// The note says both, because a translator has nothing else to tell them apart
/// by.
pub const ARRANGE_APPLICATION_SENTENCE: Word = Word::saying(
    "applications.verb.arrange-application.sentence",
    "put {application} {where}",
)
.noting(
    "{application} is the identifier this machine knows an application by, like \
     org.blender.Blender. It is never translated. {where} is one of the arrangements below — \
     applications.where.left-half and its two neighbours — and it arrives already translated, so \
     write this sentence and those three as one sentence between them: whatever preposition or \
     case your language needs goes into the arrangement rather than here, and the gap can move to \
     wherever your language puts the place.",
);
/// `arrange_application`'s first argument.
pub const ARRANGE_APPLICATION_APPLICATION: Word = Word::saying(
    "applications.verb.arrange-application.argument.application",
    "which application's window to move",
);
/// `arrange_application`'s second argument.
pub const ARRANGE_APPLICATION_WHERE: Word = Word::saying(
    "applications.verb.arrange-application.argument.where",
    "where on the screen the window goes",
);

// ---------------------------------------------------------------------------
// The three arrangements. Each of them completes the sentence above, so they
// are written as they will read inside it rather than as labels.
// ---------------------------------------------------------------------------

/// What a translator has to know about all three: they are not labels.
const COMPLETES_THE_SENTENCE: &str = "This is not a label; it finishes applications.verb.arrange-application.sentence — read the \
     two together and write them so the whole line reads as one sentence in your language. The \
     preposition and the case belong here rather than in the sentence, because a language that \
     inflects the place needs the whole phrase in front of it. It is also what a person reads \
     beside the option when they are shown the choice.";

/// The window takes the left half of the screen.
pub const LEFT_HALF: Word = Word::saying(
    "applications.where.left-half",
    "on the left half of the screen",
)
.noting(COMPLETES_THE_SENTENCE);
/// The window takes the right half of the screen.
pub const RIGHT_HALF: Word = Word::saying(
    "applications.where.right-half",
    "on the right half of the screen",
)
.noting(COMPLETES_THE_SENTENCE);
/// The window takes the whole screen.
pub const WHOLE_SCREEN: Word =
    Word::saying("applications.where.whole-screen", "across the whole screen")
        .noting(COMPLETES_THE_SENTENCE);

/// Every string this crate can say, in the order a translator meets them: what
/// it could not reach, what could not join the list, how one is shown, the four
/// verbs, and the arrangements one of them offers.
pub const EVERY_WORD: [Word; 20] = [
    NOT_INSTALLED,
    NO_IDENTIFIER,
    NOT_AN_IDENTIFIER,
    CALLED,
    OPEN_APPLICATION,
    OPEN_APPLICATION_SENTENCE,
    OPEN_APPLICATION_APPLICATION,
    FOCUS_APPLICATION,
    FOCUS_APPLICATION_SENTENCE,
    FOCUS_APPLICATION_APPLICATION,
    CLOSE_APPLICATION,
    CLOSE_APPLICATION_SENTENCE,
    CLOSE_APPLICATION_APPLICATION,
    ARRANGE_APPLICATION,
    ARRANGE_APPLICATION_SENTENCE,
    ARRANGE_APPLICATION_APPLICATION,
    ARRANGE_APPLICATION_WHERE,
    LEFT_HALF,
    RIGHT_HALF,
    WHOLE_SCREEN,
];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the list above — the tests at the bottom of this
/// file are what say so. It is a `Result` rather than an unwrap because a
/// library that panics on its own string table takes the daemon with it, and
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
pub fn application_words() -> Result<Vocabulary, WordsError> {
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
    use alo_strings::Key;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// `Word::key` does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                Key::named(word.named()),
                Ok(word.key()),
                "{}: {}",
                word.named(),
                Key::named(word.named()).unwrap_err()
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
            assert_eq!(word.key().area(), "applications", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = application_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = application_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every sentence with the identifier gap in it carries the note that
    /// says so.** A translator who took `{application}` for a word would
    /// translate the sentence around it wrongly in every language that inflects
    /// a name, and there is nothing in the sentence itself to warn them.
    #[test]
    fn every_sentence_holding_an_identifier_says_what_the_gap_is() {
        for word in EVERY_WORD {
            if word.says().contains("{application}") {
                assert!(
                    word.note().is_some_and(|note| note.contains("identifier")),
                    "{} has an identifier gap and no note about it",
                    word.named()
                );
            }
        }
    }

    /// **The word *ask* is load-bearing**, so the string that carries it says
    /// why to whoever translates it. A translation promising that the
    /// application closes would be promising something alo OS does not do.
    #[test]
    fn the_close_sentence_tells_a_translator_that_asking_is_the_point() {
        let note = CLOSE_APPLICATION_SENTENCE.note().unwrap();
        assert!(note.contains("close button"), "{note}");
        assert!(note.contains("unsaved"), "{note}");
        assert!(CLOSE_APPLICATION_SENTENCE.says().starts_with("ask "));
    }
}
