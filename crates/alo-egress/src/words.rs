//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! It is the sixth list of its kind — `alo-files` in item 9b, `alo-shortcuts`
//! in 9c, `alo-appearance` in 9d, `alo-capability` in 9e, `alo-models` in 9f —
//! and the last crate in this workspace to hold English. What is different
//! about it is what one of these strings is.
//!
//! # The indicator line is the promise on the box
//!
//! Law 1 says every network egress an agent causes is *visible at the moment it
//! happens*. [`crate::Leaving`]'s three sentences are that visibility: they are
//! the whole of what a person is shown while something is leaving their
//! machine. A person who cannot read them is a person the indicator is not for,
//! which is why *"English plus the big five"* is the same bug as hardcoded
//! English wearing a business case — and why these three are the strings in
//! this repository it would be least acceptable to leave untranslated.
//!
//! They are one sentence each, not a stem with a destination glued on. The
//! preposition before the place is not punctuation a program can pick: English
//! wants *of* after a question and *from* after a fetch, and a language that
//! inflects the place needs the whole sentence in front of it to choose.
//!
//! # A destination is a phrase, and one kind of it is not a string at all
//!
//! Three of the four kinds of [`crate::Destination`] are described here. The
//! fourth — a host a verb's argument named — is shown exactly as it was
//! written, like a filename in `alo-files` and a model's name in `alo-models`:
//! it is somebody's data, and a translation of `alo.example` would be an
//! invention. [`crate::Destination::word`] answers `None` for it, and says so.
//!
//! The three that are here are **twins of `alo-models`' four**, because
//! `Destination::of` maps an [`InferenceSource`](alo_models::InferenceSource)
//! onto one of them in one place. They are not the same strings, and cannot be:
//! `alo-models` says where an answer *came from* — *by someone, which has not
//! said where it runs* — while these name a *place a thing is going to*, which
//! is a different grammatical job in English and in most languages. What must
//! not differ is what they say about the provider, and that is the note on each
//! of them and a test in `tests/what_this_crate_says.rs`.
//!
//! # Nothing here counts anything out loud
//!
//! There is no [`alo_strings::Plural`] in this list, which is `alo-models`'
//! rule from item 9f kept. The one sentence that would have had to count is
//! [`crate::DestinationError::TooLong`], whose English was *an address is at
//! most 253 characters* — English's two shapes standing in for Polish's three.
//! The number stays a field on the refusal, for whoever draws it beside the
//! sentence in the reader's own language.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Lifted into `alo-strings` by item 9d. Re-exported here because this crate's
/// own files, and the tests that read this list, name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Where something is going — [`crate::Destination`].
//
// A phrase rather than a sentence: each of these goes inside an indicator line
// and inside every refusal the policy makes, so that a line and the place named
// in it are one language.
// ---------------------------------------------------------------------------

/// A machine somebody paired with, on their own network (ADR 0003).
pub const A_PAIRED_MACHINE: Word = Word::saying(
    "egress.destination.paired-machine",
    "{machine}, on your network",
)
.noting(
    "{machine} is the name a person gave the other machine when they paired with it, and is never \
     translated. \"Your network\" is the person's own: this left the machine and stayed in the \
     building. It goes inside a longer sentence — \"@mail is asking a question of {machine}, on \
     your network\".",
);

/// A provider that has said where it runs.
pub const A_PROVIDER: Word = Word::saying(
    "egress.destination.provider",
    "{provider}, in {region}",
)
.noting(
    "{provider} is the name a person gave the provider when they added it. {region} is where that \
     provider says it runs, in the provider's own words — \"the EU\", \"Switzerland\". Neither is \
     translated. It goes inside a longer sentence naming where something is going.",
);

/// A provider that has not.
pub const A_PROVIDER_SOMEWHERE: Word = Word::saying(
    "egress.destination.provider-somewhere",
    "{provider}, which has not said where it runs",
)
.noting(
    "This is read while something is on its way out of the machine. It has to sound like what it \
     is — nobody knows where this goes — rather than like a detail that has not been filled in \
     yet. It says the same thing as \"models.source.hosted-unstated\" about the same provider, in \
     the other grammatical position: that one is where an answer came from, this one is where \
     something is going. {provider} is never translated.",
);

// ---------------------------------------------------------------------------
// The indicator line — [`crate::Leaving`].
//
// The sentence law 1 exists to put in front of somebody. Three of them, because
// what an agent can cause to leave is three things and no more (`Why`).
// ---------------------------------------------------------------------------

/// A question put to a model somewhere other than this machine.
pub const IS_ASKING: Word = Word::saying(
    "egress.leaving.asking",
    "{agent} is asking a question of {destination}",
)
.noting(
    "Shown while it is happening, on the indicator every alo OS machine has. {agent} is the name \
     of an agent — \"@mail\" — and is never translated. {destination} is where the question is \
     going and arrives already in the reader's language. Present tense: this is happening now, \
     not something that might.",
);

/// Something retrieved from an address a verb named.
pub const IS_FETCHING: Word = Word::saying(
    "egress.leaving.fetching",
    "{agent} is fetching something from {destination}",
)
.noting(
    "\"Something\" is deliberately unspecific: what is being fetched is not always nameable, and a \
     line that named it wrongly would be worse than one that does not. {agent} is never \
     translated; {destination} arrives already in the reader's language.",
);

/// Something handed to a service outside this machine.
pub const IS_SENDING: Word = Word::saying(
    "egress.leaving.sending",
    "{agent} is sending something to {destination}",
)
.noting(
    "The one of the three that says something of the person's is going out, so it must not read \
     more softly than the other two. {agent} is never translated; {destination} arrives already \
     in the reader's language.",
);

// ---------------------------------------------------------------------------
// What alo OS is doing itself — [`crate::Errand`], and the promise beside it.
//
// The other kind of egress: no agent, and so no sentence with an {agent} in
// it. Three reasons and no more, on the same indicator as an agent's egress,
// because *no telemetry* is only checkable by somebody who can see what the
// machine does when nobody has asked it to.
//
// "alo OS" is the product's name and stays as it is in every language.
// ---------------------------------------------------------------------------

/// Signing a person in against their alo identity.
pub const ALO_IS_SIGNING_YOU_IN: Word = Word::saying(
    "egress.itself.signing-in",
    "alo OS is signing you in at {destination}",
)
.noting(
    "Shown while it is happening, on the same indicator as everything else that leaves. \"You\" is \
     the person at the machine, so a language that distinguishes formal and familiar should use \
     whichever the rest of the shell uses. {destination} is where the sign-in is going and arrives \
     already in the reader's language. \"alo OS\" is the product's name and is never translated.",
);

/// Downloading a model from the catalogue.
pub const ALO_IS_FETCHING_A_MODEL: Word = Word::saying(
    "egress.itself.fetching-a-model",
    "alo OS is fetching a model from {destination}",
)
.noting(
    "This is the download that lets the machine answer questions on its own hardware, so it is the \
     one egress a person is most likely to want to see and least likely to be alarmed by. \"A \
     model\" is the thing that answers questions, not a shape or an example. {destination} arrives \
     already in the reader's language; \"alo OS\" is never translated.",
);

/// Asking whether there is a newer deployment.
pub const ALO_IS_CHECKING_FOR_AN_UPDATE: Word = Word::saying(
    "egress.itself.checking-for-an-update",
    "alo OS is checking for an update at {destination}",
)
.noting(
    "Checking, not installing: this line is about the question, and it is the place where other \
     operating systems send a description of your machine along with it. It must not read as \
     though something is being sent. {destination} arrives already in the reader's language; \"alo \
     OS\" is never translated.",
);

/// The promise that goes with the list of three.
pub const ALO_REACHES_NOTHING_ELSE: Word = Word::saying(
    "egress.itself.nothing-else",
    "alo OS reaches the network for these reasons and no others, and never to say anything about \
     how you use this machine",
)
.noting(
    "★ The no-telemetry promise, shown beside the list of reasons rather than kept in a document \
     nobody reads. Both halves matter and a shorter translation must not drop either: \"and no \
     others\" is what makes the list a list rather than an example, and the second half is the \
     promise itself — no measurement, no diagnostics, no anonymised anything. It is a statement of \
     fact about the machine, not a reassurance, so it should read as flatly as the rest of the \
     shell.",
);

// ---------------------------------------------------------------------------
// What this machine's policy will not let leave — [`crate::Refusal`].
//
// An organisation named the rule (ADR 0004) and this is it in words. A policy
// nobody can understand is a policy people work around, so each one says what
// the machine is set to and where the thing was going, rather than that
// something was not permitted.
// ---------------------------------------------------------------------------

/// Something that would have left the building.
pub const OUTSIDE_THE_BUILDING: Word = Word::saying(
    "egress.policy.outside-the-building",
    "this machine is set to keep everything in the building, and {destination} is outside it",
)
.noting(
    "{destination} is where it was going and arrives already in the reader's language. \"In the \
     building\" means on this machine or on one somebody here paired with — not on the same \
     network, which is not the same thing (ADR 0003).",
);

/// Something that would have left the region an organisation named.
pub const OUTSIDE_THE_REGION: Word = Word::saying(
    "egress.policy.outside-the-region",
    "this machine is set to reach {region} only, and {destination} does not meet that",
)
.noting(
    "{region} is the region the organisation named, in their own words, and is never translated. \
     {destination} arrives already in the reader's language. \"Does not meet that\" covers both a \
     place somewhere else and a place that has not said where it is.",
);

/// Something that would have left at all.
pub const NOTHING_MAY_LEAVE: Word = Word::saying(
    "egress.policy.nothing-leaves",
    "this machine is set to let nothing leave, and {destination} is somewhere else",
)
.noting("{destination} arrives already in the reader's language.");

// ---------------------------------------------------------------------------
// Somewhere that could not be named — [`crate::DestinationError`].
//
// Two of these refuse what a verb's argument said and are read by a person; two
// refuse a mistake in code and are read by whoever is fixing it. They are one
// list because they are one type, shown in one place: splitting them would put
// a branch in front of whoever draws a refusal, and that branch is where an
// English sentence reaches a screen.
// ---------------------------------------------------------------------------

/// Nothing, or only spaces.
pub const NOWHERE_NAMED: Word = Word::saying(
    "egress.destination.nameless",
    "say where this is going — an egress with nowhere named cannot be shown to anybody",
)
.noting(
    "The second half is the reason rather than an apology: the indicator is what law 1 promises, \
     and a line with no place on it would be a line nobody can act on.",
);

/// A character that cannot be read on one line.
pub const NOT_SHOWABLE: Word = Word::saying(
    "egress.destination.not-printable",
    "the address contains a character that cannot be shown — the indicator has to be readable in \
     one line",
)
.noting(
    "It is almost always a line break or a terminal escape inside an address an agent supplied. Do \
     not quote the address: this sentence must never contain the thing it is refusing.",
);

/// Longer than an address can be.
pub const TOO_LONG: Word = Word::saying(
    "egress.destination.too-long",
    "that address is longer than an address can be — check it is a hostname and nothing more",
)
.noting(
    "How many characters an address may be is a number, shown beside this line rather than inside \
     it — see this file's note about counting. \"A hostname\" is the name of a machine, like \
     alo.example.",
);

/// A source that is this machine, which is not a departure at all.
pub const NOTHING_LEAVES: Word = Word::saying(
    "egress.destination.nothing-leaves",
    "an answer on this machine goes nowhere — ask whether the source causes egress before making a \
     destination of it",
)
.noting(
    "Read by whoever is writing the code that caused this, not by the person using the machine: \
     nothing left, so there is nothing for them to be told about. It is translated anyway because \
     it shares a type with the two refusals above, and whoever shows one shows all of them.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
pub const EVERY_WORD: [Word; 17] = [
    A_PAIRED_MACHINE,
    A_PROVIDER,
    A_PROVIDER_SOMEWHERE,
    IS_ASKING,
    IS_FETCHING,
    IS_SENDING,
    ALO_IS_SIGNING_YOU_IN,
    ALO_IS_FETCHING_A_MODEL,
    ALO_IS_CHECKING_FOR_AN_UPDATE,
    ALO_REACHES_NOTHING_ELSE,
    OUTSIDE_THE_BUILDING,
    OUTSIDE_THE_REGION,
    NOTHING_MAY_LEAVE,
    NOWHERE_NAMED,
    NOT_SHOWABLE,
    TOO_LONG,
    NOTHING_LEAVES,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it keeps its English and its `Display` for
/// the reason `alo-capability`'s `VerbError` does. It exists because
/// [`declare_into`] can genuinely fail against a vocabulary that already holds
/// one of these keys.
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
pub fn egress_words() -> Result<Vocabulary, WordsError> {
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
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true, and it is
    /// the same shape as `alo-shortcuts` putting every shipped binding back
    /// through `Chord::checked`.
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
            assert_eq!(word.key().area(), "egress", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = egress_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = egress_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **The indicator line is the string this crate exists for**, so all three
    /// of them carry a note: a translator with no product in front of them
    /// cannot tell that `{agent}` is a name and `{destination}` a clause
    /// somebody else already translated.
    #[test]
    fn every_string_a_person_reads_while_something_leaves_carries_a_note() {
        for word in [
            IS_ASKING,
            IS_FETCHING,
            IS_SENDING,
            ALO_IS_SIGNING_YOU_IN,
            ALO_IS_FETCHING_A_MODEL,
            ALO_IS_CHECKING_FOR_AN_UPDATE,
            ALO_REACHES_NOTHING_ELSE,
            A_PAIRED_MACHINE,
            A_PROVIDER,
            A_PROVIDER_SOMEWHERE,
            OUTSIDE_THE_BUILDING,
            OUTSIDE_THE_REGION,
            NOTHING_MAY_LEAVE,
        ] {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **What alo OS does on its own names the place too.** These lines are
    /// what makes ★ *no telemetry* checkable rather than claimed, and a line
    /// that said only *alo OS is doing something* would be worth nothing to the
    /// person reading it.
    #[test]
    fn every_line_about_the_machines_own_errands_names_where_it_is_reaching() {
        for word in [
            ALO_IS_SIGNING_YOU_IN,
            ALO_IS_FETCHING_A_MODEL,
            ALO_IS_CHECKING_FOR_AN_UPDATE,
        ] {
            assert!(word.says().contains("{destination}"), "{}", word.named());
            // No agent caused these, so no line about one may name an agent.
            assert!(!word.says().contains("{agent}"), "{}", word.named());
        }
    }

    /// **The promise says both halves.** *These reasons and no others* is what
    /// makes the list a list; *nothing about how you use this machine* is the
    /// promise itself. A translation that kept one and dropped the other would
    /// pass every check `alo-strings` makes, because neither half is a gap — so
    /// this is the test, and the note is where a translator is warned.
    #[test]
    fn the_no_telemetry_promise_says_both_halves_and_fills_nothing() {
        assert!(ALO_REACHES_NOTHING_ELSE.says().contains("no others"));
        assert!(
            ALO_REACHES_NOTHING_ELSE
                .says()
                .contains("how you use this machine")
        );
        assert!(!ALO_REACHES_NOTHING_ELSE.says().contains('{'));
        assert!(
            ALO_REACHES_NOTHING_ELSE
                .note()
                .is_some_and(|note| note.contains("drop either"))
        );
    }

    /// **The three lines name all three things.** An indicator that said only
    /// *something is leaving* would be a diagnostic rather than a feature, and
    /// a translation that dropped a gap is refused by `alo-strings` — but only
    /// if the gap was in the English to begin with.
    #[test]
    fn every_indicator_line_names_the_agent_and_the_place() {
        for word in [IS_ASKING, IS_FETCHING, IS_SENDING] {
            assert!(word.says().contains("{agent}"), "{}", word.named());
            assert!(word.says().contains("{destination}"), "{}", word.named());
        }
    }

    /// **Every refusal the policy makes names where the thing was going.** A
    /// person told only that a rule exists cannot tell whether it stopped
    /// something they meant to happen.
    #[test]
    fn every_policy_refusal_names_where_it_was_going() {
        for word in [OUTSIDE_THE_BUILDING, OUTSIDE_THE_REGION, NOTHING_MAY_LEAVE] {
            assert!(word.says().contains("{destination}"), "{}", word.named());
        }
    }

    /// **No sentence here counts anything.** `alo-models` settled it in item 9f
    /// and this list keeps it: a gap named for a quantity would be English's
    /// two shapes standing in for Polish's three, and the plural rules are not
    /// written from memory.
    #[test]
    fn nothing_this_crate_says_counts_something() {
        let counting = egress_words().unwrap();
        assert_eq!(counting.counted().count(), 0);
        for word in EVERY_WORD {
            for gap in ["count", "how-many", "how_many", "number", "longest"] {
                assert!(
                    !word.says().contains(&format!("{{{gap}}}")),
                    "{}: {gap}",
                    word.named()
                );
            }
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }
}
