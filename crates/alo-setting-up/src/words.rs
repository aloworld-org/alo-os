//! Every string this crate can say, and the English beside each one.
//!
//! Fourteen: the question, the four choices, the line beside each of the four,
//! and the five ways an answer goes nowhere that nobody else has a sentence
//! for. Everything a person's own settings refuse is `alo-choosing`'s own
//! words, carried rather than reworded, because two accounts of one moment is
//! one account too many.
//!
//! # Every sentence says what a choice **is**, and none says what it is worth
//!
//! The four lines answer one question each — where does my question go, and
//! what does this need — because that is what a person weighing four
//! configurations has to know. None of them says what a choice costs, what it
//! is good for, what most people do, or which one alo OS would pick.
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! made that a property of the flow rather than of the copy, and `crate::nudging`
//! is the check that keeps it one.
//!
//! **The fourth line is written to the same length and the same shape as the
//! other three.** ADR 0009 gave *not at all* the same weight, and a line half
//! as long as its neighbours is the greyed-out panel that ADR already refused,
//! wearing a word count.
//!
//! # Nothing here names anything alo OS rents
//!
//! The pinned model runtime is on every machine this repository builds, and *on
//! this machine* is the choice its name most wants to appear in. It does not:
//! `alo_saying::rented` walks this list with everybody else's, and what a person
//! reads is *a model on this machine*.
//!
//! # Nothing here counts anything
//!
//! There is no [`alo_strings::Plural`] in this list. The four are four in the
//! code and are drawn as a list rather than counted out loud, so no sentence
//! here has to know that Polish counts in three.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported as every declaring crate does, so this crate's own files name it
/// as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The question, and the four answers to it.
// ---------------------------------------------------------------------------

/// What setup asks.
pub const THE_QUESTION: Word = Word::saying(
    "setup.the-question",
    "where should your questions be answered?",
)
.noting(
    "Asked once, the first time somebody signs in to this machine. \"Your questions\" are the ones \
     they would put to the agent. The four answers follow it as a list, in a fixed order, with \
     none of them chosen until the person chooses one — so this is a question rather than a \
     confirmation, and it must not be translated as one.",
);

/// The first choice's name.
pub const ON_THIS_MACHINE: Word = Word::saying("setup.choice.on-this-machine", "on this machine")
    .noting(
        "One of four answers to `setup.the-question`, and the first in the list. Short, because it \
         is read as a label beside three others of the same shape; the sentence under it is \
         `setup.choice.on-this-machine.is`. \"This machine\" is the computer the person is sitting \
         at.",
    );

/// What the first choice is.
pub const ON_THIS_MACHINE_IS: Word = Word::saying(
    "setup.choice.on-this-machine.is",
    "a model already on this computer answers, and your question does not leave it",
)
.noting(
    "The line under `setup.choice.on-this-machine`. It says two things and no others: which thing \
     answers, and where the question goes. It must not be translated as praise, as advice, or as \
     a statement about what this choice is good for — each of the four lines answers the same two \
     questions so that a person can weigh them against each other.",
);

/// The second choice's name.
pub const ON_A_MACHINE_ON_THIS_NETWORK: Word = Word::saying(
    "setup.choice.on-a-machine-on-this-network",
    "on a machine on your network",
)
.noting(
    "One of four answers to `setup.the-question`. \"Your network\" is the one the person's \
     computer is on — an office or a home, not the internet. The sentence under it is \
     `setup.choice.on-a-machine-on-this-network.is`.",
);

/// What the second choice is.
pub const ON_A_MACHINE_ON_THIS_NETWORK_IS: Word = Word::saying(
    "setup.choice.on-a-machine-on-this-network.is",
    "another computer you have paired with this one answers, and your question leaves this \
     computer",
)
.noting(
    "The line under `setup.choice.on-a-machine-on-this-network`. \"Paired\" means a deliberate \
     link made on both computers by the people who own them. The second clause is the one that \
     must survive translation exactly: the question does leave, and a wording that softened it \
     would be alo OS understating what it does with somebody's words.",
);

/// The third choice's name.
pub const FROM_A_PROVIDER: Word =
    Word::saying("setup.choice.with-a-provider", "with a provider you add").noting(
        "One of four answers to `setup.the-question`. A \"provider\" is a service somewhere else \
         that answers questions, added by the person under a name they pick, with an address and \
         usually a key. The sentence under it is `setup.choice.with-a-provider.is`.",
    );

/// What the third choice is.
pub const FROM_A_PROVIDER_IS: Word = Word::saying(
    "setup.choice.with-a-provider.is",
    "a service you name and give an address and a key answers, and your question leaves this \
     network",
)
.noting(
    "The line under `setup.choice.with-a-provider`. It says what the person has to supply and \
     where the question goes, and nothing about price, speed or quality. The second clause must \
     not be softened: a question put to a provider leaves the building.",
);

/// The fourth choice's name.
pub const NOT_AT_ALL: Word = Word::saying("setup.choice.not-at-all", "not at all").noting(
    "One of four answers to `setup.the-question`, and it has exactly the weight of the other \
     three. It means the person wants no agent on this computer. The sentence under it is \
     `setup.choice.not-at-all.is`.",
);

/// What the fourth choice is.
pub const NOT_AT_ALL_IS: Word = Word::saying(
    "setup.choice.not-at-all.is",
    "nothing answers questions, and everything else on this computer works the same way",
)
.noting(
    "The line under `setup.choice.not-at-all`. The second clause is a fact rather than \
     reassurance: files, windows, printing, settings and updates are unchanged, and every one of \
     them can be reached without an agent. Written to the same length and shape as the other \
     three lines on purpose — a shorter one would read as the lesser answer.",
);

// ---------------------------------------------------------------------------
// Why an answer went nowhere — [`crate::NotSetUp`].
//
// Five, and every one of them ends by saying setup is still waiting — or, for
// the one about setup already being over, where to go instead. A person who
// answered a question and was told nothing would conclude they had answered it.
// ---------------------------------------------------------------------------

/// Answered with nothing selected.
pub const NOTHING_SELECTED: Word = Word::saying(
    "setup.not-answered.nothing-selected",
    "nothing is selected, so nothing has been chosen and setup is still waiting for an answer",
)
.noting(
    "Said when setup is answered with no choice selected — pressing continue without picking one \
     of the four. It is not a scolding: nothing is selected because alo OS selects nothing, and \
     the person has not picked one yet.",
);

/// Answered with the material for a different choice.
pub const ANOTHER_CHOICE: Word = Word::saying(
    "setup.not-answered.another-choice",
    "that answer is not the choice that is selected, so nothing has been chosen and setup is \
     still waiting for an answer",
)
.noting(
    "Said when what was selected and what was answered with are two different ones of the four — \
     a surface with a stale selection, or two things clicked at once. This is a defect in the \
     screen rather than anything the person did, and it is said plainly because the alternative \
     is alo OS deciding which of the two they meant.",
);

/// The paired-machine choice, on a machine that has paired with none.
pub const NO_PAIRED_MACHINE: Word = Word::saying(
    "setup.not-answered.no-paired-machine",
    "this computer has not been paired with another one, so nothing has been chosen and setup is \
     still waiting for an answer",
)
.noting(
    "Said when somebody answers with `setup.choice.on-a-machine-on-this-network` and this \
     computer knows of no other. Pairing is made deliberately on both computers, and alo OS does \
     not make one on anybody's behalf — so this is a true sentence about the machine rather than \
     a fault.",
);

/// A provider answered with no model to ask it for.
pub const NOTHING_TO_ASK_FOR: Word = Word::saying(
    "setup.not-answered.nothing-to-ask-for",
    "that service has not been given a model to answer with, so nothing has been chosen and setup \
     is still waiting for an answer",
)
.noting(
    "Said when somebody answers with `setup.choice.with-a-provider` and leaves the model empty. A \
     \"model\" here is the name the service answers to, which the person copies from whoever runs \
     it. Held apart from `setup.not-answered.nothing-selected` because the two send a person to \
     two different boxes on the same screen.",
);

/// Setup answered a second time.
pub const ALREADY_ANSWERED: Word = Word::saying(
    "setup.not-answered.already-answered",
    "setup has already been answered on this computer, so it was not answered again — what \
     answers your questions is in settings",
)
.noting(
    "Said when setup is answered twice. Setup is asked once; everything it decides can be changed \
     afterwards, and the last clause says where. \"Settings\" is the name of the place on this \
     computer where a person changes what they chose, and is capitalised or not as that name is \
     in the language being translated into.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 14] = [
    THE_QUESTION,
    ON_THIS_MACHINE,
    ON_THIS_MACHINE_IS,
    ON_A_MACHINE_ON_THIS_NETWORK,
    ON_A_MACHINE_ON_THIS_NETWORK_IS,
    FROM_A_PROVIDER,
    FROM_A_PROVIDER_IS,
    NOT_AT_ALL,
    NOT_AT_ALL_IS,
    NOTHING_SELECTED,
    ANOTHER_CHOICE,
    NO_PAIRED_MACHINE,
    NOTHING_TO_ASK_FOR,
    ALREADY_ANSWERED,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it keeps its English and its `Display` for
/// the reason every other crate's does, which is that whoever reads it is
/// whoever is fixing the list.
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
pub fn setting_up_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
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

    /// What we ship is held to the rule everybody else is held to: `Word::key`
    /// does not check, because a key written in this file cannot arrive from
    /// anywhere, and this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
        }
    }

    /// A key names one string.
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
            assert_eq!(word.key().area(), "setup", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = setting_up_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = setting_up_words().unwrap();
        assert!(matches!(
            declare_into(&mut vocabulary).unwrap_err(),
            WordsError::List(_)
        ));
    }

    /// **Every one of them tells a translator what it is about.** Four of these
    /// are labels of one or four words, and a label handed over without a note
    /// is the string a translator has to guess at.
    #[test]
    fn every_word_tells_a_translator_what_it_is_about() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| !note.trim().is_empty()),
                "{} has nothing to say to a translator",
                word.named()
            );
        }
    }

    /// **The four lines are of a kind.** ADR 0009 gave the fourth choice the
    /// same weight as the other three, and a line half the length of its
    /// neighbours is the greyed-out panel that ADR refused, wearing a word
    /// count. Held loosely — a range rather than a number — because languages
    /// differ and the thing being prevented is one line being conspicuously the
    /// runt.
    #[test]
    fn no_one_of_the_four_lines_is_conspicuously_shorter_than_the_others() {
        let lines = [
            ON_THIS_MACHINE_IS,
            ON_A_MACHINE_ON_THIS_NETWORK_IS,
            FROM_A_PROVIDER_IS,
            NOT_AT_ALL_IS,
        ];
        let longest = lines.iter().map(|word| word.says().len()).max().unwrap();
        for word in lines {
            assert!(
                word.says().len() * 2 >= longest,
                "{} is less than half the length of the longest of the four: {}",
                word.named(),
                word.says()
            );
        }
    }

    /// **Every way an answer goes nowhere says setup is still waiting**, which
    /// is the half the person acts on: they have not chosen anything yet, and
    /// they are still being asked.
    #[test]
    fn every_refusal_says_the_question_is_still_waiting() {
        for word in [
            NOTHING_SELECTED,
            ANOTHER_CHOICE,
            NO_PAIRED_MACHINE,
            NOTHING_TO_ASK_FOR,
            ALREADY_ANSWERED,
        ] {
            assert!(
                word.says().contains("setup is still waiting")
                    || word.says().contains("already been answered"),
                "{}: {}",
                word.named(),
                word.says()
            );
        }
    }

    /// **Nothing here counts anything out loud**, which is `alo-models`' rule
    /// from item 9f: a gap named for a quantity would be English's two shapes
    /// standing in for Polish's three.
    #[test]
    fn nothing_this_crate_says_counts_something() {
        assert_eq!(setting_up_words().unwrap().counted().count(), 0);
        for word in EVERY_WORD {
            assert!(
                !word
                    .says()
                    .chars()
                    .any(|character| character.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }

    /// **Nothing here has a gap in it.** Every other crate's sentences carry a
    /// path or a name; these are read before anything on this machine has a
    /// name, so a `{gap}` here would be a sentence with nothing to fill it.
    #[test]
    fn nothing_this_crate_says_has_anything_to_fill_in() {
        for word in EVERY_WORD {
            assert!(
                !word.says().contains('{'),
                "{}: {}",
                word.named(),
                word.says()
            );
        }
    }
}
