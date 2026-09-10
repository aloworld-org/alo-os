//! Every string this crate can say, and the English beside each one.
//!
//! Thirteen, and they divide in three. Ten are **what became of one entry** —
//! the short clause read at the head of a line, before the sentence the machine
//! generated when it happened. One is **the answer to a question nothing
//! matches**. Two are **refusals**: the ways an account cannot be put in front
//! of anybody.
//!
//! The shape is `alo-approving`'s and is copied rather than re-decided:
//! constants under one area, `alo_strings::Word` because these are literals in
//! this file, [`declare_into`] for the one vocabulary the machine has, and
//! tests at the bottom holding the list to the rules every other list is held
//! to. Nothing here counts anything, so there is no plural in this file — how
//! many entries an account holds is a number beside it, never inside a
//! sentence.
//!
//! # What is deliberately not on this list
//!
//! **Everything the record already says.** The sentence describing what an
//! agent did is the call's own, generated from the validated arguments and
//! written down at the moment it happened; the reason something was refused is
//! the refusal's own words, kept as they were shown; where something went is
//! `alo-egress`'s. None of them is declared here, and there is no string on
//! this list that describes a change or names a place.
//!
//! That is the whole of what makes this an account rather than a retelling. A
//! surface able to word what happened would be a machine with two descriptions
//! of one moment, and the one a person read afterwards would be the one nothing
//! checked.
//!
//! # Ten clauses, and none of them is a category
//!
//! The ten outcomes are sentences about a machine rather than labels for a
//! column — *the agent asked to do this and the person at this machine said no*
//! rather than *declined*. A person reading their own record is not reading a
//! table of statuses they have to learn, and the difference between *nobody was
//! asked*, *the person said no* and *it was refused at the last moment* is
//! exactly the difference a one-word label loses.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no alo machine in front of them. *The agent*
//! is the assistant built into alo OS and not a person; *the desktop* is the
//! graphical session rather than a piece of furniture; *a screen* is a physical
//! display. Where the sentence cannot be translated from its own words, the
//! note says so.

use alo_strings::Vocabulary;

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What became of one entry — [`crate::Outcome`].
//
// Each is read at the head of one line of an account, with the machine's own
// sentence about what happened after it. They say what the record says and
// nothing more: none of them describes the change itself, because the record
// already carries the words a person was shown.
// ---------------------------------------------------------------------------

/// A verb ran.
pub const RAN: Word = Word::saying("recounting.outcome.ran", "the agent did this").noting(
    "Read at the head of one line of a record somebody is reading back, with a sentence after it \
     describing exactly what was done — the agent being the assistant built into alo OS and not a \
     person. It reports one thing that really happened on this machine and is not a claim that it \
     succeeded at anything beyond doing it.",
);

/// A change the grants already refused, so nobody was interrupted.
pub const NOBODY_WAS_ASKED: Word = Word::saying(
    "recounting.outcome.nobody-was-asked",
    "the agent asked to do this and it was refused before anybody was asked about it",
)
.noting(
    "Read at the head of one line of a record somebody is reading back — the agent being the \
     assistant built into alo OS and not a person. It means the machine refused on its own: what \
     the agent wanted was outside what it had been allowed, so nothing was ever put in front of \
     the person to answer. Nothing happened, and the person was never interrupted.",
);

/// A person said no.
pub const THE_PERSON_SAID_NO: Word = Word::saying(
    "recounting.outcome.the-person-said-no",
    "the agent asked to do this and the person at this machine said no",
)
.noting(
    "Read at the head of one line of a record somebody is reading back — the agent being the \
     assistant built into alo OS and not a person. Somebody was shown the sentence after it and \
     declined it. No reason was asked for and none is recorded, so a translation should not \
     suggest one was given.",
);

/// The grants were asked at the moment it would have run, and said no.
pub const THE_GRANTS_SAID_NO: Word = Word::saying(
    "recounting.outcome.the-grants-said-no",
    "the agent asked to do this and it was refused at the moment it would have happened",
)
.noting(
    "Read at the head of one line of a record somebody is reading back — the agent being the \
     assistant built into alo OS and not a person. Most often this is somebody having taken back \
     what they allowed, between agreeing to something and it being carried out, and the machine \
     honouring that. Nothing happened.",
);

/// Something that never became a call at all.
pub const NEVER_BECAME_A_CALL: Word = Word::saying(
    "recounting.outcome.never-became-a-call",
    "the agent asked for something this machine does not offer, so nothing was done",
)
.noting(
    "Read at the head of one line of a record somebody is reading back — the agent being the \
     assistant built into alo OS and not a person. What was asked for was not on the list of \
     things alo OS will do at all, or what was asked with it made no sense, so it was turned away \
     before it became anything. What the agent typed is shown beside this and is not this \
     machine's words.",
);

/// A question answered on this machine.
pub const ANSWERED_HERE: Word = Word::saying(
    "recounting.outcome.answered-here",
    "the agent asked a question and it was answered on this machine",
)
.noting(
    "Read at the head of one line of a record somebody is reading back — the agent being the \
     assistant built into alo OS and not a person. Nothing left the machine. The question itself \
     is not recorded and never will be, so there is nothing after this line to describe.",
);

/// A question that was refused before it was put anywhere.
pub const NEVER_PUT_ANYWHERE: Word = Word::saying(
    "recounting.outcome.never-put-anywhere",
    "the agent asked a question and it was not put anywhere, so nothing was sent",
)
.noting(
    "Read at the head of one line of a record somebody is reading back — the agent being the \
     assistant built into alo OS and not a person. A rule on this machine refused the place the \
     question would have gone, and no second place was tried. The reason is shown after it in the \
     words the person was shown at the time.",
);

/// The person's grants were not read again.
pub const GRANTS_NOT_READ_AGAIN: Word = Word::saying(
    "recounting.outcome.grants-not-read-again",
    "what you have granted could not be read again, so this machine went on with the list it already had",
)
.noting(
    "Read at the head of one line of a record somebody is reading back. Somebody's side of the      machine said that what they had granted had changed, and the machine could not read its own      list of grants — so nothing was widened and nothing was forgotten either. The reason is shown      after it in the words the person was shown at the time. \"Granted\" is what a person does by      picking a folder for their agent to reach.",
);

/// Something left this machine.
pub const LEFT: Word = Word::saying("recounting.outcome.left", "something left this machine")
    .noting(
        "Read at the head of one line of a record somebody is reading back. Where it went is shown \
         beside it. It is deliberately vague about what went: alo OS records that something left \
         and never what was in it.",
    );

/// Something the egress policy refused to let leave.
pub const HELD_BACK: Word = Word::saying(
    "recounting.outcome.held-back",
    "something would have left this machine and was not allowed to, so nothing left",
)
.noting(
    "Read at the head of one line of a record somebody is reading back. A rule on this machine \
     stopped it, and the rule's own words are shown after it. The second half is the point: this \
     line is not a departure and must not read as one.",
);

/// alo OS reached the network with nobody having asked.
pub const LEFT_ON_ITS_OWN: Word = Word::saying(
    "recounting.outcome.left-on-its-own",
    "alo OS reached the network itself, with nobody having asked it to",
)
.noting(
    "Read at the head of one line of a record somebody is reading back. alo OS is the name of this \
     operating system and is never translated. There is no agent and no person behind this line: \
     the machine did it for one of the few reasons it is allowed to, which is shown beside it. \
     Somebody reading this is checking a promise, so it should read plainly rather than \
     reassuringly.",
);

// ---------------------------------------------------------------------------
// The answer when nothing matches — [`crate::Account::said`].
// ---------------------------------------------------------------------------

/// What an account with nothing in it says.
///
/// Read **above** what the record says about itself, never instead of it: a
/// record that does not go all the way back says so in `alo-keeping`'s own
/// words, and this sentence beside that one is the difference between *the
/// agent did nothing* and *this record does not reach that far*.
pub const NOTHING_TO_TELL: Word = Word::saying(
    "recounting.nothing-to-tell",
    "nothing in this machine's record answers that question",
)
.noting(
    "Read where a list of what happened would have been. It says the record holds no answer to \
     what was asked, which is not the same as saying nothing happened — a sentence beneath it says \
     whether the record goes all the way back. So it must not be softened into \"nothing \
     happened\" or \"the machine has been idle\".",
);

// ---------------------------------------------------------------------------
// The two refusals — [`crate::NotRecounted`].
//
// Read somewhere other than the surface the account would have been on,
// because there is no such surface: a service log while a session starts, or a
// text console. They are still declared and still translated, because a person
// reading a log on their own machine reads their own language.
// ---------------------------------------------------------------------------

/// What [`crate::NotRecounted::NoCompositor`] says: nothing is drawing a
/// screen, so there is nowhere to put the account.
pub const NO_COMPOSITOR: Word = Word::saying(
    "recounting.no-compositor",
    "What this machine's record says cannot be put in front of you: the desktop is not running. \
     Sign in to the desktop, and ask again",
)
.noting(
    "The desktop is the graphical session a person signs into. Read when the part of alo OS that \
     draws a screen is not running at all — during start-up, or on a text console — so it is read \
     in a log or on a console rather than on the surface it is about. Nothing has happened to the \
     record itself.",
);

/// What [`crate::SurfaceRefused::NothingToShowOn`] says: something is drawing,
/// and there is no screen to draw on.
pub const NOTHING_TO_SHOW_ON: Word = Word::saying(
    "recounting.nothing-to-show-on",
    "What this machine's record says cannot be put in front of you: no screen is connected. \
     Connect a screen, and ask again",
)
.noting(
    "A screen here is a physical display. Read when the desktop is running with no display to draw \
     on — before a monitor is plugged in, or after the last one went away. Nothing has happened to \
     the record itself.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 14] = [
    RAN,
    NOBODY_WAS_ASKED,
    THE_PERSON_SAID_NO,
    THE_GRANTS_SAID_NO,
    NEVER_BECAME_A_CALL,
    ANSWERED_HERE,
    NEVER_PUT_ANYWHERE,
    GRANTS_NOT_READ_AGAIN,
    LEFT,
    HELD_BACK,
    LEFT_ON_ITS_OWN,
    NOTHING_TO_TELL,
    NO_COMPOSITOR,
    NOTHING_TO_SHOW_ON,
];

/// What became of one entry, in the order [`crate::Outcome`] declares them.
///
/// Named apart from the rest so the tests below can hold each group to the rule
/// that is actually its own: a clause is read beside a sentence and does not end
/// in a full stop, and a refusal says what to do next.
pub const EVERY_OUTCOME: [Word; 11] = [
    RAN,
    NOBODY_WAS_ASKED,
    THE_PERSON_SAID_NO,
    THE_GRANTS_SAID_NO,
    NEVER_BECAME_A_CALL,
    ANSWERED_HERE,
    NEVER_PUT_ANYWHERE,
    GRANTS_NOT_READ_AGAIN,
    LEFT,
    HELD_BACK,
    LEFT_ON_ITS_OWN,
];

/// What an account says about itself, rather than about one entry.
pub const EVERY_REMARK: [Word; 1] = [NOTHING_TO_TELL];

/// Every way this crate refuses, each of which is read having just asked
/// something.
pub const EVERY_REFUSAL: [Word; 2] = [NO_COMPOSITOR, NOTHING_TO_SHOW_ON];

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
pub fn recounting_words() -> Result<Vocabulary, WordsError> {
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
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "recounting", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(recounting_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = recounting_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **The three groups are the whole of the list.** A word added here and
    /// classified as none of them fails this rather than quietly escaping every
    /// rule below.
    #[test]
    fn every_word_is_an_outcome_a_remark_or_a_refusal() {
        let named: BTreeSet<&str> = EVERY_OUTCOME
            .iter()
            .chain(EVERY_REMARK.iter())
            .chain(EVERY_REFUSAL.iter())
            .map(|word| word.named())
            .collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        for word in EVERY_WORD {
            assert!(named.contains(word.named()), "{}", word.named());
        }
    }

    /// **Every string here is whole, with nothing to fill in** — a gap in one
    /// would put `{}` in front of a person where an account of their own machine
    /// belongs. What varies from line to line is the record's own text, which is
    /// shown beside these rather than poured into them.
    #[test]
    fn nothing_here_needs_anything_filled_in() {
        for word in EVERY_WORD {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{}",
                word.named()
            );
        }
    }

    /// **Every refusal says what to do.** Somebody told only that their record
    /// cannot be shown has been told they are stuck, at the one moment where
    /// being stuck means they cannot check what their machine has been doing.
    /// Each is two clauses — what is so, and then the action.
    #[test]
    fn every_refusal_says_what_to_do() {
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

    /// **The outcomes are clauses read beside a sentence, not labels.** None of
    /// them ends in a full stop, none starts a sentence of its own, and no two
    /// read the same — a person told the same clause for *the machine refused*
    /// and *you refused* would be reading two different afternoons as one.
    #[test]
    fn every_outcome_is_a_clause_read_beside_a_sentence() {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for word in EVERY_OUTCOME {
            assert!(!word.says().ends_with('.'), "{}", word.named());
            assert!(
                word.says().starts_with(char::is_lowercase),
                "{} reads as a sentence of its own",
                word.named()
            );
            assert!(seen.insert(word.says()), "two outcomes say {}", word.says());
        }
        // And the one remark is read where a list would have been, so it is a
        // clause too rather than a heading.
        assert!(!NOTHING_TO_TELL.says().ends_with('.'));
    }

    /// **Every word carries a note for the translator.** None of these can be
    /// translated from its own words alone: every one of them is read in a place
    /// that gives it its meaning, and half of them name something — the agent,
    /// the desktop, a screen, alo OS — that has a meaning here.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Every sentence that names the agent says the agent is not a person.**
    /// The grammar of the whole sentence depends on the answer in a good many
    /// languages, and a translator who guesses wrong writes alo OS a person into
    /// every one of them.
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

    /// **Nothing here counts anything.** A number written into an English
    /// sentence is a sentence that cannot be translated into a language with
    /// three plural forms; how many entries an account holds is a number shown
    /// beside it, and this is what keeps a plural from being written by hand.
    #[test]
    fn nothing_said_here_counts_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }

    /// **Nothing here describes what happened.** The sentence in an account is
    /// the record's own, generated from the arguments a call was validated with;
    /// a string on this list that named a verb or a place would be this crate
    /// starting to retell what it is supposed to be reading out.
    #[test]
    fn nothing_here_describes_a_change() {
        for word in EVERY_WORD {
            for describing in ["move ", "delete ", "rename ", "file ", "folder "] {
                assert!(
                    !word.says().to_lowercase().contains(describing),
                    "{} describes a change",
                    word.named()
                );
            }
        }
    }
}
