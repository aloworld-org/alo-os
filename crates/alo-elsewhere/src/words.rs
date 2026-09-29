//! Every string this crate can say, and the English beside each one.
//!
//! Three of them are the states in [`crate::Reaching`], read in a list of the
//! person's own machines. The fourth and fifth are the two ways adding a machine
//! is refused.
//!
//! # The one that matters is the one nobody plans for
//!
//! [`DID_NOT_ANSWER`] and [`NOT_ASKED_YET`] are two sentences a careless
//! translation would merge, and merging them is the whole fault this crate
//! exists to prevent. *It did not answer* is a machine that may be off, may be
//! elsewhere, may have a broken network — something to act on. *Not asked yet*
//! is a machine added thirty seconds ago and nothing is wrong at all.
//!
//! A translator handed both at once will reasonably wonder whether one word
//! would do. Both notes say outright that it would not, because the person
//! reading the merged version goes looking for a fault that does not exist.
//!
//! The shape is `alo-dock`'s, one crate on: constants, `alo_strings::Word`, and
//! a test at the bottom putting every key back through the vocabulary.

use alo_strings::Vocabulary;

/// One string a crate can say. Re-exported so this crate's own files name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// How a machine is reaching — [`crate::Reaching`]. Read beside each machine in
// the person's list of their own machines.
// ---------------------------------------------------------------------------

/// The key of [`ANSWERED_WORD`], so [`crate::Reaching`] names it without the
/// whole string.
pub const ANSWERED: &str = "elsewhere.reaching.answered";

/// The key of [`DID_NOT_ANSWER_WORD`].
pub const DID_NOT_ANSWER: &str = "elsewhere.reaching.did-not-answer";

/// The key of [`NOT_ASKED_YET_WORD`].
pub const NOT_ASKED_YET: &str = "elsewhere.reaching.not-asked-yet";

/// A machine that answered when this one last asked.
pub const ANSWERED_WORD: Word = Word::saying(ANSWERED, "Answered").noting(
    "Beside one machine in a list of the person's own machines. It means this machine reached \
     that one when it last asked, and nothing more — not that the other machine is idle, healthy \
     or free. Keep it to one or two words: it sits in a row beside a name, not in a sentence.",
);

/// A machine that was asked and did not answer.
pub const DID_NOT_ANSWER_WORD: Word = Word::saying(DID_NOT_ANSWER, "Did not answer").noting(
    "Beside one machine in a list of the person's own machines, and the one row they may need to \
     act on: the machine may be off, may be somewhere else, may have lost its network. It must \
     NOT be merged with the *not asked yet* string into one word for *unavailable*. Somebody \
     reading the merged version for a machine nobody has asked goes looking for a fault that \
     does not exist. The machine is still listed and still theirs; this says what happened when \
     it was asked, not that it is gone.",
);

/// A machine nobody has asked since it was added.
pub const NOT_ASKED_YET_WORD: Word = Word::saying(NOT_ASKED_YET, "Not asked yet").noting(
    "Beside one machine in a list of the person's own machines, just after they add it. Nothing \
     is wrong: adding a machine is not asking it anything. It must NOT be merged with the *did \
     not answer* string — this one is reassuring and that one is not, and a translation that \
     makes them the same word turns a calm state into an alarming one thirty seconds after \
     somebody added a machine that is fine.",
);

// ---------------------------------------------------------------------------
// The two ways adding a machine is refused — [`crate::NotElsewhere`]. Read at
// the moment a person is adding one.
// ---------------------------------------------------------------------------

/// The key of [`UNNAMED_WORD`].
pub const UNNAMED: &str = "elsewhere.not-added.unnamed";

/// The key of [`NAME_TOO_LONG_WORD`].
pub const NAME_TOO_LONG: &str = "elsewhere.not-added.name-too-long";

/// The key of [`ALREADY_ADDED_WORD`].
pub const ALREADY_ADDED: &str = "elsewhere.not-added.already-added";

/// A machine cannot be added without a name.
pub const UNNAMED_WORD: Word = Word::saying(UNNAMED, "Give this machine a name you will recognise")
    .noting(
        "Shown where a person is adding one of their own machines and has left the name empty. \
         It asks rather than scolds: the name is for them, not for the machine, and nobody else \
         ever sees it. Not *the name is required* — that says whose rule it is rather than what \
         it is for.",
    );

/// A name past what a list can show.
pub const NAME_TOO_LONG_WORD: Word = Word::saying(
    NAME_TOO_LONG,
    "That name is longer than {at_most} characters, so it will not fit in the list",
)
.noting(
    "Shown while a person is typing a name for one of their own machines. It says what will \
     happen rather than that a rule was broken, because the limit exists so a list of machines \
     stays scannable. {at_most} is a plain whole number with no sign or separator on it — how a \
     number is written belongs to the region rather than to the language.",
);

/// A machine already on the person's list.
pub const ALREADY_ADDED_WORD: Word = Word::saying(
    ALREADY_ADDED,
    "You have already added this machine, under the name {called}",
)
.noting(
    "Shown when a person adds a machine that is already on their list. It names what they called \
     it the first time, because the likeliest reason they are adding it again is that they did \
     not recognise it under that name. Not an error about duplicates — it is the answer to *have \
     I added this one?* {called} is the name they gave it, which is theirs and never translated.",
);

// ---------------------------------------------------------------------------
// Letting the agent drive a machine — [`crate::Driving`]. Read where the person
// grants it, which ADR 0079 makes a term of the decision rather than a choice
// about presentation.
// ---------------------------------------------------------------------------

/// The key of [`WHAT_IT_CANNOT_ENUMERATE_WORD`].
pub const WHAT_IT_CANNOT_ENUMERATE: &str = "elsewhere.driving.what-it-cannot-enumerate";

/// The key of [`MAY_DRIVE_WORD`].
pub const MAY_DRIVE: &str = "elsewhere.driving.may-drive";

/// The key of [`MAY_NOT_DRIVE_WORD`].
pub const MAY_NOT_DRIVE: &str = "elsewhere.driving.may-not-drive";

/// The key of [`ENDS_BEFORE_IT_BEGINS_WORD`].
pub const ENDS_BEFORE_IT_BEGINS: &str = "elsewhere.not-given.ends-before-it-begins";

/// The key of [`ALREADY_DRIVING_WORD`].
pub const ALREADY_DRIVING: &str = "elsewhere.not-given.already-driving";

/// **The sentence ADR 0079 requires**, read where the person grants this.
pub const WHAT_IT_CANNOT_ENUMERATE_WORD: Word = Word::saying(
    WHAT_IT_CANNOT_ENUMERATE,
    "This lets the agent do anything on that machine that you could do there. Unlike every other \
     permission you give, this one cannot list what it allows",
)
.noting(
    "THE most important string in this crate, and ADR 0079 makes showing it a term of the \
     decision rather than a presentation choice. Read at the moment a person is letting their \
     agent use another of their machines. Every other permission in alo OS is a list of things; \
     this one is not, because a pointer and a keyboard reach everything a machine can do. Both \
     halves must survive translation: what it allows (anything you could do there) AND that it \
     is unlike the others (it cannot list what it allows). A translation that keeps only the \
     first reads as ordinary permission, which is exactly the belief this sentence exists to \
     prevent. It is not a warning and not a question — do not translate it as *are you sure*. \
     It is the plain fact, told to somebody who is entitled to decide.",
);

/// A machine the agent may currently use.
pub const MAY_DRIVE_WORD: Word = Word::saying(MAY_DRIVE, "Your agent may use this machine").noting(
    "Beside one machine in a list of what the person has granted. Present tense, about right \
         now: the grant has an end, and this says it has not been reached. Keep it short enough \
         to sit in a row beside a machine's name.",
);

/// A machine the agent may not use.
pub const MAY_NOT_DRIVE_WORD: Word =
    Word::saying(MAY_NOT_DRIVE, "Your agent may not use this machine").noting(
        "Beside one machine in a list of what the person has granted, when the grant has ended or \
         was never given. It is the ordinary state and must not read as a fault or a refusal — \
         nothing has gone wrong, the agent simply does not have this machine.",
    );

/// A grant whose end is not after its beginning.
pub const ENDS_BEFORE_IT_BEGINS_WORD: Word = Word::saying(
    ENDS_BEFORE_IT_BEGINS,
    "Choose a time after now for this to end",
)
.noting(
    "Shown while a person is granting their agent a machine and has picked an end that is not in \
     the future. It asks for the thing it needs rather than naming the rule that was broken. \
     Every grant in alo OS ends; there is no wording here for a permanent one because there is \
     no permanent one.",
);

/// A machine already granted.
pub const ALREADY_DRIVING_WORD: Word = Word::saying(
    ALREADY_DRIVING,
    "Your agent already has this machine — change when it ends instead",
)
.noting(
    "Shown when a person grants a machine their agent already has. The second half matters more \
     than the first: it tells them the act they actually want. Granting again is refused rather \
     than silently making the grant longer, because somebody who thinks they are granting and is \
     in fact widening has not been told what they did.",
);

// ---------------------------------------------------------------------------
// How work sent to another machine is going — [`crate::HowItIsGoing`]. Read in
// a list of what the person's machines are doing for them.
// ---------------------------------------------------------------------------

/// The key of [`WORK_SENT_WORD`].
pub const WORK_SENT: &str = "elsewhere.work.sent";

/// The key of [`WORK_WORKING_WORD`].
pub const WORK_WORKING: &str = "elsewhere.work.working";

/// The key of [`WORK_CAME_BACK_WORD`].
pub const WORK_CAME_BACK: &str = "elsewhere.work.came-back";

/// The key of [`WORK_STOPPED_WORD`].
pub const WORK_STOPPED: &str = "elsewhere.work.stopped";

/// The key of [`WORK_COULD_NOT_BE_SENT_WORD`].
pub const WORK_COULD_NOT_BE_SENT: &str = "elsewhere.work.could-not-be-sent";

/// The key of [`NO_GOAL_WORD`].
pub const NO_GOAL: &str = "elsewhere.not-sent.no-goal";

/// The key of [`GOAL_TOO_LONG_WORD`].
pub const GOAL_TOO_LONG: &str = "elsewhere.not-sent.goal-too-long";

/// The key of [`IT_WAS_STOPPED_WORD`].
pub const IT_WAS_STOPPED: &str = "elsewhere.work.stopped-before-it-came-back";

/// Work handed over, with the other machine not yet saying it has started.
pub const WORK_SENT_WORD: Word = Word::saying(WORK_SENT, "Sent").noting(
    "Beside one piece of work in a list of what the person's machines are doing for them. It \
     means handed over and not yet begun, as far as this machine knows. One or two words: it \
     sits in a row, not in a sentence.",
);

/// Work the other machine has started.
pub const WORK_WORKING_WORD: Word = Word::saying(WORK_WORKING, "Working").noting(
    "Beside one piece of work in a list of what the person's machines are doing for them. The \
     other machine has begun. Present continuous if the language has one — it is happening now, \
     and the person may still stop it.",
);

/// Work that finished, with its result here.
pub const WORK_CAME_BACK_WORD: Word = Word::saying(WORK_CAME_BACK, "Done").noting(
    "Beside one piece of work in a list of what the person's machines are doing for them. It \
     finished and the result is on THIS machine, the one that asked — never on the machine that \
     did the work. It says nothing about whether the result is good, only that it is here.",
);

/// Work the person stopped.
pub const WORK_STOPPED_WORD: Word = Word::saying(WORK_STOPPED, "Stopped").noting(
    "Beside one piece of work the person stopped. Stopped by them, not failed — the translation \
     must not read as an error or a crash. Nothing went wrong; they changed their mind.",
);

/// Work that never reached the other machine.
pub const WORK_COULD_NOT_BE_SENT_WORD: Word =
    Word::saying(WORK_COULD_NOT_BE_SENT, "Could not be sent").noting(
        "Beside one piece of work that never reached the machine it was meant for. It stays in \
         the list rather than disappearing, because a list that drops what went wrong reads as \
         *everything is fine*. It is about the sending, not about the other machine being \
         broken.",
    );

/// Work cannot be sent without saying what is wanted.
pub const NO_GOAL_WORD: Word = Word::saying(NO_GOAL, "Say what you want this machine to do")
    .noting(
        "Shown where a person is sending work to another of their machines and has said nothing. \
         It asks for the thing it needs. What crosses is a goal in the person's own words — not \
         a command, not a program name — so the wording must invite a sentence rather than an \
         instruction.",
    );

/// A goal longer than what crosses.
pub const GOAL_TOO_LONG_WORD: Word = Word::saying(
    GOAL_TOO_LONG,
    "That is longer than {at_most} characters — say the goal, and let the machine work out the \
     rest",
)
.noting(
    "Shown while a person types what they want another of their machines to do. The second half \
     is the half that matters: the limit is not meanness, it is that a goal is what is wanted \
     rather than instructions for doing it. {at_most} is a plain whole number with no sign or \
     separator on it.",
);

/// A result that arrived after the person stopped the work.
pub const IT_WAS_STOPPED_WORD: Word = Word::saying(
    IT_WAS_STOPPED,
    "You stopped this work, so the result that arrived afterwards was not kept",
)
.noting(
    "Read by somebody who stopped a piece of work and is being told a result turned up anyway \
     and was discarded. It must read as the machine keeping their decision, not as something \
     lost by accident: they ruled this out, and it stayed ruled out. Do not translate it as an \
     error.",
);

// ---------------------------------------------------------------------------
// What a window says about where it is — [`crate::Marked`]. Read on the window
// itself, in every state including filling the screen.
// ---------------------------------------------------------------------------

/// The key of [`ON_MACHINE_WORD`].
pub const ON_MACHINE: &str = "elsewhere.window.on-machine";

/// The key of [`ON_ANOTHER_MACHINE_WORD`].
pub const ON_ANOTHER_MACHINE: &str = "elsewhere.window.on-another-machine";

/// A window showing a named machine of the person's.
pub const ON_MACHINE_WORD: Word = Word::saying(ON_MACHINE, "On {called}").noting(
    "Carried by a window that is showing another of the person's machines, in every state the \
     window can be in — including filling the screen, which is when every other cue is gone and \
     this is the only thing left saying the screen is not this computer. It is a WORD and not a \
     colour: a person who cannot rely on hue must still be able to tell, and a screen reader \
     must be able to speak it. {called} is the name that person gave the machine, which is \
     theirs and is never translated. Keep it short — it sits on a window, not in a sentence.",
);

/// A window showing a machine the person has removed, or never added.
pub const ON_ANOTHER_MACHINE_WORD: Word = Word::saying(ON_ANOTHER_MACHINE, "On another machine")
    .noting(
        "Carried by a window showing a machine that is no longer on the person's list. It must \
         still say the window is elsewhere: a window that quietly stopped saying so would look \
         like this computer's own, and somebody could type a password into a machine they did \
         not mean to. It is not an error and nothing has gone wrong — the machine is simply no \
         longer one they keep, so there is no name to give.",
    );

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 21] = [
    ANSWERED_WORD,
    DID_NOT_ANSWER_WORD,
    NOT_ASKED_YET_WORD,
    UNNAMED_WORD,
    NAME_TOO_LONG_WORD,
    ALREADY_ADDED_WORD,
    WHAT_IT_CANNOT_ENUMERATE_WORD,
    MAY_DRIVE_WORD,
    MAY_NOT_DRIVE_WORD,
    ENDS_BEFORE_IT_BEGINS_WORD,
    ALREADY_DRIVING_WORD,
    WORK_SENT_WORD,
    WORK_WORKING_WORD,
    WORK_CAME_BACK_WORD,
    WORK_STOPPED_WORD,
    WORK_COULD_NOT_BE_SENT_WORD,
    NO_GOAL_WORD,
    GOAL_TOO_LONG_WORD,
    IT_WAS_STOPPED_WORD,
    ON_MACHINE_WORD,
    ON_ANOTHER_MACHINE_WORD,
];

/// What can go wrong declaring this crate's words.
///
/// A `Result` rather than an unwrap because a library that panics on its own
/// string table takes the shell with it, and because [`declare_into`] can
/// genuinely fail against a vocabulary that already holds one of these keys.
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

/// A vocabulary holding only this crate's words.
///
/// # Errors
/// [`WordsError::List`] if two of these declared the same key, which the test
/// at the bottom of this file exists to catch first.
pub fn elsewhere_words() -> Result<Vocabulary, WordsError> {
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
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    /// No two words share a key, which a vocabulary would refuse at run time
    /// and this catches at test time.
    #[test]
    fn every_key_is_its_own() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len(), "two words share a key");
    }

    /// Every key is in this crate's own area, so adding them to the shell's one
    /// vocabulary cannot collide with another crate's.
    #[test]
    fn every_key_is_in_this_crates_area() {
        for word in EVERY_WORD {
            assert!(
                word.named().starts_with("elsewhere."),
                "{} is outside this crate's area",
                word.named()
            );
        }
    }

    /// Every string carries a note, because a word with no note is where a
    /// translation goes wrong quietly.
    #[test]
    fn every_word_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **The two sentences a translation must not merge say so to the
    /// translator.** This is the fault this crate exists to prevent, one layer
    /// out: if the distinction only lives in Rust, the shipped product loses it
    /// in every language but English.
    #[test]
    fn the_two_states_that_must_not_be_merged_tell_the_translator_so() {
        for word in [DID_NOT_ANSWER_WORD, NOT_ASKED_YET_WORD] {
            let note = word.note().expect("a note");
            assert!(
                note.contains("must NOT be merged"),
                "{} does not warn the translator",
                word.named()
            );
        }
    }

    /// **Both halves of the sentence ADR 0079 requires are in the string**, not
    /// one in the string and one added by whatever draws it. A translation that
    /// kept only *anything you could do there* would read as ordinary
    /// permission, which is the belief the sentence exists to prevent.
    #[test]
    fn the_grant_that_cannot_list_itself_says_both_halves() {
        let says = WHAT_IT_CANNOT_ENUMERATE_WORD.says();
        assert!(says.contains("anything on that machine that you could do there"));
        assert!(says.contains("cannot list what it allows"));

        let note = WHAT_IT_CANNOT_ENUMERATE_WORD.note().expect("a note");
        assert!(
            note.contains("Both halves must survive translation"),
            "the note does not tell a translator which halves matter"
        );
        assert!(
            note.contains("are you sure"),
            "the note does not warn against turning a fact into a question"
        );
    }

    /// The sentences that take a value name it, so a translator knows what will
    /// be put there.
    #[test]
    fn the_sentences_with_a_gap_in_them_name_it() {
        assert!(NAME_TOO_LONG_WORD.says().contains("{at_most}"));
        assert!(ALREADY_ADDED_WORD.says().contains("{called}"));
        assert!(GOAL_TOO_LONG_WORD.says().contains("{at_most}"));
        assert!(ON_MACHINE_WORD.says().contains("{called}"));
    }

    /// **A window that is elsewhere says so with a word, never with a colour.**
    /// Both notes say it outright, because a translator handed a two-word
    /// string on a window has no way to know it is the only thing standing
    /// between somebody and a password typed into the wrong machine.
    #[test]
    fn the_window_markings_tell_the_translator_they_are_not_decoration() {
        let named = ON_MACHINE_WORD.note().expect("a note");
        assert!(
            named.contains("WORD and not a colour"),
            "the note does not say the marking must not rely on hue"
        );
        assert!(
            named.contains("filling the screen"),
            "the note does not say the marking survives full screen"
        );

        let unnamed = ON_ANOTHER_MACHINE_WORD.note().expect("a note");
        assert!(
            unnamed.contains("password"),
            "the note does not say what getting it wrong costs"
        );
    }

    /// **The result comes back to the machine that asked**, and the sentence
    /// for *done* tells a translator so. A translation implying the result is
    /// on the machine that did the work would send somebody looking for it in
    /// the wrong place.
    #[test]
    fn the_done_sentence_says_where_the_result_is() {
        let note = WORK_CAME_BACK_WORD.note().expect("a note");
        assert!(
            note.contains("the one that asked"),
            "the note does not say where the result lands"
        );
    }

    /// **Stopped is a decision, not a failure**, and the note says so — a
    /// translation reading as an error would tell somebody something broke when
    /// they simply changed their mind.
    #[test]
    fn stopped_is_not_translated_as_a_failure() {
        for word in [WORK_STOPPED_WORD, IT_WAS_STOPPED_WORD] {
            let note = word.note().expect("a note");
            assert!(
                note.contains("not failed")
                    || note.contains("not as an error")
                    || note.contains("Do not translate it as an error"),
                "{} does not warn against reading a decision as a fault",
                word.named()
            );
        }
    }

    /// They all go into one vocabulary, which is what the shell does with them.
    #[test]
    fn they_all_go_into_one_vocabulary() {
        let vocabulary = elsewhere_words().expect("no key is declared twice");
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
    }
}
