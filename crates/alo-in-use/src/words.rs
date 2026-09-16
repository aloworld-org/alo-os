//! Every string this crate can say, and the English beside each one.
//!
//! Ten of them, in three groups. **Four are what the indicator reads as** — one
//! line for each of the three things a machine can watch or listen with, and
//! the sentence for a room in which nothing is. **Three are who** — the clauses
//! that go where a line says by whom. **Three are refusals**: why what is
//! watching or listening could not be answered at all.
//!
//! The shape is `alo-indicator`'s and is copied rather than re-decided:
//! constants under one area, `alo_strings::Word` because these are literals in
//! this file, [`declare_into`] for the one vocabulary the machine has, and
//! tests at the bottom holding the list to the rules every other list is held
//! to. There is no countable string here, and that is the one difference: the
//! egress indicator is a light with a number under it, and this one is a line
//! for each use, because *the microphone is on* is not a quantity.
//!
//! # Why these are not `alo-capability`'s clauses, which name the same things
//!
//! `capability.facility.camera` and its neighbours are what a **grant** is over
//! — *the camera*, *one picture of the screen*, *the screen, continuously* —
//! worded to sit inside the sentence a person approves, and split the way a
//! grant is split. This crate's are what is **happening**, and the split does
//! not survive the journey: a screen being photographed and a screen being
//! recorded are one thing to somebody sitting in the room, and an indicator
//! that distinguished them would be answering a question about permission while
//! a person was asking a question about now.
//!
//! So there are three here and twelve there, they are read at different
//! moments, and neither is a fallback for the other.
//!
//! # And they are not `alo-indicator`'s, which is the other indicator
//!
//! *Something left this machine* and *the microphone is on* are different
//! warnings, and the plan's constraint for this task says so outright. Nothing
//! in this list says *leaving*, nothing in `alo-indicator`'s says *in use*, and
//! `tests/two_lines_and_neither_is_the_other.rs` is what holds that true as the
//! two lists grow.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no alo machine in front of them. *The agent*
//! is the assistant built into alo OS and not a person; *the screen* is the
//! physical display in front of somebody; *in use* means something is reading
//! from it at this instant. Where the sentence cannot be translated from its
//! own words, the note says so.
//!
//! **No sentence and no note here names the media server**, or any other part
//! alo OS rents. `alo_saying::what_a_person_would_have_to_learn` is the check,
//! and the plan's task 7 asks it of every sentence these crates say.

use alo_strings::Vocabulary;

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What the indicator reads as — [`crate::Line`], and an [`crate::InUse`] with
// nothing on it.
//
// Each of these is the whole of what one line says, announced on its own by a
// screen reader, so each is a capitalised sentence. A line nobody can read is a
// line that does not exist for some people, and the promise this indicator
// makes is not one alo OS keeps for people who can see a mark and breaks for
// people who cannot.
// ---------------------------------------------------------------------------

/// The screen is being read, by whoever the line names.
pub const THE_SCREEN: Word = Word::saying("in-use.the-screen", "The screen is in use by {who}")
    .noting(
        "The whole of what one line of the indicator says, announced on its own by a screen \
         reader, which is why it is a capitalised sentence rather than a clause. It is read \
         while something is reading the screen at this instant — recording it, sharing it in a \
         call, or taking one picture of it; the line does not distinguish those, because to \
         somebody sitting in the room they are the same fact. The screen is the physical display \
         in front of them. {who} arrives already worded: a name such as \"Cheese \
         (org.gnome.Cheese)\", or one of the clauses under in-use.the-agent, \
         in-use.alo-os-itself and in-use.something-on-this-machine.",
    );

/// The camera is being used, by whoever the line names.
pub const THE_CAMERA: Word = Word::saying("in-use.the-camera", "The camera is in use by {who}")
    .noting(
        "The whole of what one line of the indicator says, announced on its own by a screen \
         reader, which is why it is a capitalised sentence rather than a clause. It is read \
         while something is reading the camera at this instant. The camera is whichever camera \
         the machine has, not a numbered device. {who} arrives already worded: a name such as \
         \"Cheese (org.gnome.Cheese)\", or one of the clauses under in-use.the-agent, \
         in-use.alo-os-itself and in-use.something-on-this-machine.",
    );

/// The microphone is being used, by whoever the line names.
pub const THE_MICROPHONE: Word =
    Word::saying("in-use.the-microphone", "The microphone is in use by {who}").noting(
        "The whole of what one line of the indicator says, announced on its own by a screen \
         reader, which is why it is a capitalised sentence rather than a clause. It is read \
         while something is listening through the microphone at this instant. {who} arrives \
         already worded: a name such as \"Cheese (org.gnome.Cheese)\", or one of the clauses \
         under in-use.the-agent, in-use.alo-os-itself and in-use.something-on-this-machine.",
    );

/// Nothing is watching or listening at this moment.
pub const NOTHING_IS_IN_USE: Word = Word::saying(
    "in-use.nothing-is-in-use",
    "Nothing is watching or listening right now",
)
.noting(
    "The whole of what the indicator says while the screen, the camera and the microphone are \
     all idle — the state a machine is in for most of a day, and the one a person glancing at it \
     is checking for. Announced on its own by a screen reader, which is why it is a capitalised \
     sentence. \"Watching or listening\" covers all three: the screen being read, the camera \
     being read, and the microphone being listened to. \"Right now\" is the point of the \
     sentence and should survive translation: it is about this instant and not about the day.",
);

// ---------------------------------------------------------------------------
// Who is using it — [`crate::By`], filling the {who} gap above.
//
// An application fills the gap with its own name, which is not translated and
// is not declared here. These three are for the ones with no name of their own
// to show.
// ---------------------------------------------------------------------------

/// An agent, by the name the machine knows it by.
pub const THE_AGENT: Word = Word::saying("in-use.the-agent", "{agent}, the agent on this machine")
    .noting(
        "Goes inside another sentence in place of {who}, so it stays lowercase and must read as \
         a noun phrase rather than as a sentence. {agent} is the name the machine knows the \
         agent by, such as @files; it is not translated. The agent is the AI assistant built \
         into alo OS, not a person, and the words \"the agent\" must stay in the translation: \
         this clause is the only thing on the line that says an agent rather than an \
         application is using the camera, and a person who cannot tell one colour from another \
         has nothing else to read it from (ADR 0010).",
    );

/// alo OS itself, with no agent and no application behind it.
pub const ALO_OS_ITSELF: Word = Word::saying("in-use.alo-os-itself", "alo OS itself").noting(
    "Goes inside another sentence in place of {who}, so it stays lowercase and must read as a \
     noun phrase rather than as a sentence. Read when the operating system is the thing using \
     the screen, the camera or the microphone with nobody having asked it to — and it appears on \
     the same indicator as everybody else's use, which is the point of the sentence. \"alo OS\" \
     is a product name and is not translated.",
);

/// Something the machine can see but cannot name.
pub const SOMETHING_ON_THIS_MACHINE: Word = Word::saying(
    "in-use.something-on-this-machine",
    "something on this machine",
)
.noting(
    "Goes inside another sentence in place of {who}, so it stays lowercase and must read as a \
     noun phrase rather than as a sentence. Read when the machine can see that the screen, the \
     camera or the microphone is being used but cannot say by what — the use is shown anyway, \
     because a use nobody can name is the one a person most needs to be told about. It must not \
     read as though nothing is happening.",
);

// ---------------------------------------------------------------------------
// Why it could not be answered at all — [`crate::NotHeard`].
//
// Read on the indicator's own surface, in place of the lines: a person looking
// at the one control that says whether their room is being watched is told that
// it cannot answer, rather than being shown an empty list that looks exactly
// like a quiet room.
// ---------------------------------------------------------------------------

/// What [`crate::NotHeard::NothingHandlesSoundAndVideo`] says.
pub const NOTHING_HANDLES_SOUND_AND_VIDEO: Word = Word::saying(
    "in-use.nothing-handles-sound-and-video",
    "What is watching or listening cannot be shown: the part of alo OS that handles sound and \
     video is not running. Sign out and back in, and this comes back with it",
)
.noting(
    "Read on the indicator itself, in place of the lines. \"Watching or listening\" covers the \
     screen, the camera and the microphone. The part of alo OS that handles sound and video is \
     one component and is deliberately not named: naming it would ask the reader to learn what \
     it is before they could understand why the machine cannot answer. The second sentence must \
     survive translation — it is the only thing the person can do.",
);

/// What [`crate::NotHeard::NoAnswer`] says.
pub const NO_ANSWER: Word = Word::saying(
    "in-use.no-answer",
    "What is watching or listening cannot be shown: the part of alo OS that handles sound and \
     video did not answer. Sign out and back in to start it again",
)
.noting(
    "Read on the indicator itself, in place of the lines. Different from \
     in-use.nothing-handles-sound-and-video: that one is not running at all, this one is there \
     and did not reply. A person fixes them the same way and is not being told the same thing. \
     \"Watching or listening\" covers the screen, the camera and the microphone.",
);

/// What [`crate::NotHeard::NotUnderstood`] says.
pub const NOT_UNDERSTOOD: Word = Word::saying(
    "in-use.not-understood",
    "What is watching or listening cannot be shown: the part of alo OS that handles sound and \
     video answered something this machine could not read. Report this, because what is watching \
     or listening cannot be trusted until it is fixed",
)
.noting(
    "Read on the indicator itself, in place of the lines. It is alo OS's own bug rather than \
     anything the person did, and the second sentence says the serious half: until it is fixed, \
     the machine cannot promise that a camera in use would be shown. \"Watching or listening\" \
     covers the screen, the camera and the microphone. Do not soften it into \"something went \
     wrong\".",
);

/// Every line the indicator can read as, in the order a translator meets them.
pub const EVERY_LINE: [Word; 4] = [THE_SCREEN, THE_CAMERA, THE_MICROPHONE, NOTHING_IS_IN_USE];

/// Every clause that says who is using something.
pub const EVERY_WHO: [Word; 3] = [THE_AGENT, ALO_OS_ITSELF, SOMETHING_ON_THIS_MACHINE];

/// Every refusal: why what is watching or listening could not be answered.
pub const EVERY_REFUSAL: [Word; 3] = [NOTHING_HANDLES_SOUND_AND_VIDEO, NO_ANSWER, NOT_UNDERSTOOD];

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 10] = [
    THE_SCREEN,
    THE_CAMERA,
    THE_MICROPHONE,
    NOTHING_IS_IN_USE,
    THE_AGENT,
    ALO_OS_ITSELF,
    SOMETHING_ON_THIS_MACHINE,
    NOTHING_HANDLES_SOUND_AND_VIDEO,
    NO_ANSWER,
    NOT_UNDERSTOOD,
];

/// The area every key in this crate is under.
///
/// Named once so the tests below and anybody reading a translator's file are
/// looking at the same string.
pub const AREA: &str = "in-use";

/// The gap a line leaves for whoever is using something.
pub const WHO: &str = "who";

/// The gap [`THE_AGENT`] leaves for the agent's own name.
pub const AGENT: &str = "agent";

/// Why this crate's own words could not be declared.
///
/// Neither of these can happen to the list above — the tests at the bottom of
/// this file are what say so. It is a `Result` rather than an unwrap because a
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
pub fn in_use_words() -> Result<Vocabulary, WordsError> {
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
    use alo_strings::Key;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
        }
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// **The three groups add up to the whole list**, so a word added to one
    /// and forgotten in the other is a failure here rather than a string
    /// nothing declares.
    #[test]
    fn the_groups_are_the_whole_list() {
        let grouped: BTreeSet<&str> = EVERY_LINE
            .iter()
            .chain(EVERY_WHO.iter())
            .chain(EVERY_REFUSAL.iter())
            .map(|word| word.named())
            .collect();
        let everything: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(grouped, everything);
        assert_eq!(
            EVERY_LINE.len() + EVERY_WHO.len() + EVERY_REFUSAL.len(),
            EVERY_WORD.len()
        );
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), AREA, "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = in_use_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0, "nothing here counts");
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = in_use_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Each of the three lines leaves exactly one gap, and it is the same
    /// gap.** A line whose gap were named something else would be filled with
    /// nothing and read *The camera is in use by {who}* on somebody's screen.
    #[test]
    fn every_line_about_a_use_leaves_one_gap_for_who() {
        for word in [THE_SCREEN, THE_CAMERA, THE_MICROPHONE] {
            let phrase = word.phrase().unwrap();
            assert_eq!(phrase.source().gaps(), [WHO.to_owned()], "{}", word.named());
        }
        assert!(
            NOTHING_IS_IN_USE
                .phrase()
                .unwrap()
                .source()
                .gaps()
                .is_empty()
        );
        assert_eq!(
            THE_AGENT.phrase().unwrap().source().gaps(),
            [AGENT.to_owned()]
        );
    }

    /// **Every refusal is a whole sentence with nothing to fill in** — a gap in
    /// a refusal would put `{}` in front of a person at the exact moment
    /// something already went wrong.
    #[test]
    fn a_refusal_names_nothing_and_needs_nothing_filled() {
        for word in EVERY_REFUSAL {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().gaps().is_empty(), "{}", word.named());
        }
    }

    /// **Every refusal says what to do.** Somebody told only that the machine
    /// cannot say whether their camera is on has been told they are stuck,
    /// about the one thing this indicator exists for. Each is two clauses —
    /// what is so, and then the action — so none can shrink to a bare report
    /// without failing here.
    #[test]
    fn every_refusal_says_what_to_do() {
        for (word, verb) in [
            (NOTHING_HANDLES_SOUND_AND_VIDEO, "Sign out"),
            (NO_ANSWER, "Sign out"),
            (NOT_UNDERSTOOD, "Report this"),
        ] {
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

    /// **What the indicator reads as stands alone.** Every line is the whole of
    /// what one control says, announced with nothing above or beside it, so
    /// each is a capitalised sentence — the difference between this group and
    /// the clauses beneath it.
    #[test]
    fn every_line_is_a_sentence_that_stands_alone() {
        for word in EVERY_LINE.iter().chain(EVERY_REFUSAL.iter()) {
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_uppercase(),
                "{} does not begin a sentence",
                word.named()
            );
        }
    }

    /// **And every clause that says who does not.** These are read inside a
    /// line, never on their own; one that arrived as a capitalised sentence
    /// would read *The camera is in use by Alo OS itself* in the middle of
    /// somebody's screen.
    #[test]
    fn every_clause_saying_who_goes_inside_a_line() {
        for word in EVERY_WHO {
            let first = word.says().chars().next().unwrap();
            assert!(
                !first.is_uppercase(),
                "{} begins a sentence and belongs inside one",
                word.named()
            );
            assert!(
                !word.says().ends_with('.'),
                "{} ends a sentence and belongs inside one",
                word.named()
            );
        }
    }

    /// **Every word carries a note for the translator.** None of these can be
    /// translated from its own words alone: each is read at a moment the
    /// translator has to be able to picture, and each names something — the
    /// screen, the agent, being in use — that has a meaning here.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Every sentence that names the agent says the agent is not a person.**
    /// One does, and it is the clause ADR 0010 leans on: it is what a person
    /// reads when they cannot tell terracotta from anything else.
    #[test]
    fn every_word_that_names_the_agent_says_what_the_agent_is() {
        let mut found = 0_usize;
        for word in EVERY_WORD {
            if !word.says().contains("agent") {
                continue;
            }
            found = found.saturating_add(1);
            assert!(
                word.note()
                    .is_some_and(|note| note.contains("not a person")),
                "{} does not say what the agent is",
                word.named()
            );
        }
        assert_eq!(found, 1, "the clause naming the agent has gone");
    }

    /// **Nothing here counts anything.** A number written into an English
    /// sentence is one that cannot be translated into a language with three
    /// plural forms, and this indicator has no number to write: it shows a line
    /// for each use rather than a total.
    #[test]
    fn nothing_here_counts_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }

    /// **No line and no refusal says what is leaving this machine.** That is
    /// the other indicator, and the plan's constraint for this task is that the
    /// two are never confused: *the microphone is on* and *something left this
    /// machine* are different warnings.
    #[test]
    fn nothing_here_says_anything_about_what_is_leaving() {
        for word in EVERY_WORD {
            assert!(
                !word.says().contains("leaving"),
                "{} reads as the egress indicator",
                word.named()
            );
        }
    }
}
