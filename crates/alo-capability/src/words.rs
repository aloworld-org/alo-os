//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! It is the fourth list of its kind — `alo-files` in item 9b, `alo-shortcuts`
//! in 9c, `alo-appearance` in 9d — and it is deliberately the same shape as
//! those three. What is different is who reads it.
//!
//! # These are the sentences a person reads when they are told no
//!
//! Almost nothing in this crate is a label. A grant that could not be made, an
//! argument that did not survive the boundary, a change that was never put to
//! anybody, a call refused at the moment it would have run: every string here
//! is somebody being told that something did not happen. Two things follow.
//!
//! **They say what to do, not what went wrong.** That was true of this crate's
//! English before it moved here, and the notes say so where a translation could
//! lose it — *invalid argument* is not a translation of *give {argument} as a
//! full path*.
//!
//! **They are read twice.** A refusal is shown to a person and then written
//! into the record (ADR 0001 §7), where a security review reads it afterwards.
//! Both readings come from one rendering of one string, which is what
//! [`crate::Refused`] carrying a refusal rather than a sentence is for.
//!
//! # What is not here, and why that is not an oversight
//!
//! [`crate::VerbError`], [`crate::VerbsError`] and [`crate::SentenceError`]
//! keep their English and their `Display`. They are refusals of a **verb
//! declaration**: they are read by whoever is writing an adapter against
//! `docs/contracts/agent-verbs.md` at the moment their declaration fails to
//! compile past its own tests, never by whoever is using the machine. It is
//! `alo-shortcuts`' `DefaultsError` one crate on — a release's own list
//! contradicting itself is read by whoever is fixing it, and translating it
//! would hand that person a sentence in whichever language happened to be
//! loaded.

use alo_strings::{Key, Plural, Vocabulary};

/// One string a crate can say.
///
/// Lifted into `alo-strings` by item 9d. Re-exported here because this crate's
/// own files, and the tests that read this list, name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// One string this crate can say about a number of things.
///
/// **The second copy of this struct**, `alo-files`' being the first. Two copies
/// are not a pattern and a third would be, which is the rule [`Word`] itself
/// moved under in item 9d: a third lifts it into `alo-strings` beside `Word`.
/// It is written out rather than borrowed from `alo-files` because this crate
/// does not depend on that one and must not start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counted {
    /// What names it.
    named: &'static str,
    /// The gap the number goes in.
    number: &'static str,
    /// What it says about one thing.
    one: &'static str,
    /// What it says about any other number of things.
    other: &'static str,
    /// What a translator needs to know.
    note: &'static str,
}

impl Counted {
    /// What names it.
    #[must_use]
    pub fn key(&self) -> Key {
        Key::unchecked(self.named)
    }
}

// ---------------------------------------------------------------------------
// A grant that could not be made — [`crate::GrantError`].
//
// Read by somebody who has just picked the wrong thing in a file chooser.
// ---------------------------------------------------------------------------

/// A grant naming no agent.
pub const ANONYMOUS: Word = Word::saying(
    "capability.grant.anonymous",
    "say which agent this grant is for — a grant to nobody reaches nothing",
);

/// A grant over nothing at all.
pub const NOTHING_NAMED: Word = Word::saying(
    "capability.grant.nothing-named",
    "choose the folder, file or application this grant is over",
);

/// A grant to `/`, which ADR 0001 §3 does not allow.
pub const THE_WHOLE_MACHINE: Word = Word::saying(
    "capability.grant.the-whole-machine",
    "there is no grant to the whole machine — pick the folder you actually mean",
);

/// A grant over a path that is not a full path.
pub const GRANT_NOT_A_FULL_PATH: Word = Word::saying(
    "capability.grant.not-a-full-path",
    "grant a folder by its full path, so it means the same thing wherever it is asked about",
);

/// A grant over a path with `..` in it.
pub const GRANT_COULD_LEAD_ELSEWHERE: Word = Word::saying(
    "capability.grant.could-lead-elsewhere",
    "a path with .. in it can lead somewhere else — grant the folder you mean by its own path",
)
.noting("\"..\" is how a path says \"the folder above\" and is never translated.");

/// A grant lasting no time.
pub const GRANT_NO_TIME: Word = Word::saying(
    "capability.grant.no-time",
    "say how long the grant should last — a grant for no time reaches nothing",
);

/// A grant with no end this machine can represent.
pub const GRANT_NO_END: Word = Word::saying(
    "capability.grant.no-end",
    "a grant has to end — choose how long this one should last",
);

// ---------------------------------------------------------------------------
// What a grant is over — [`crate::Reach`], and the applications half of
// [`crate::Ask`].
//
// These are the line a person reads in their list of grants, and they are also
// what goes inside a refusal. A refusal and the thing named inside it are in
// one language, which is why these are strings rather than something the caller
// assembles.
// ---------------------------------------------------------------------------

/// A folder, and everything under it.
pub const A_FOLDER: Word = Word::saying("capability.reach.folder", "{path} and everything in it")
    .noting(
        "{path} is a folder on the person's own disk. It is never translated, and the sentence \
         reads better with it at the front in most languages.",
    );

/// Exactly one file.
pub const A_FILE: Word = Word::saying("capability.reach.file", "the file {path}")
    .noting("{path} is a file on the person's own disk and is never translated.");

/// One installed application.
pub const AN_APPLICATION: Word = Word::saying(
    "capability.reach.application",
    "the application {application}",
)
.noting(
    "{application} is the identifier the system knows an application by, like \
     org.blender.Blender. It is never translated.",
);

// ---------------------------------------------------------------------------
// What the grants say when they refuse — [`crate::NotGranted`] and
// [`crate::NotAuthorised`].
//
// The two halves of the grants' answer say different things because they need
// different things from the reader: an expiry is fixed by granting again, and a
// path nobody granted is not.
// ---------------------------------------------------------------------------

/// A grant that covered it and has run out.
pub const HAS_EXPIRED: Word = Word::saying(
    "capability.refused.expired",
    "{agent} had a grant to {reach} and it has expired — grant it again to let {agent} reach \
     {wanted}",
)
.noting(
    "{agent} is the name of an agent, written the way it was granted, and is never translated. \
     {reach} is what the grant was over and {wanted} is what was asked for; both arrive already \
     in the reader's language.",
);

/// Nothing has ever covered it.
pub const NEVER_GRANTED: Word = Word::saying(
    "capability.refused.never-granted",
    "{agent} has not been granted {wanted} — grants are made by picking a folder, never by asking \
     for one",
)
.noting(
    "The second half is the whole capability model in one clause: a person grants by choosing \
     something, and no amount of asking widens what an agent may reach. {agent} is never \
     translated; {wanted} arrives already in the reader's language.",
);

/// A change offered where only a read may go.
pub const CHANGE_WAITS: Word = Word::saying(
    "capability.refused.change-waits",
    "{verb} changes something — propose it with the sentence describing it and let one person \
     approve that, rather than running it in the turn",
)
.noting(
    "{verb} is the name of a capability, like move_file, and is never translated. \"In the turn\" \
     means \"while the person is waiting for the answer\".",
);

// ---------------------------------------------------------------------------
// An argument that did not survive the boundary — [`crate::ArgError`].
//
// Every one of them names the argument and says what to send instead. They are
// read by a person looking at a refusal, and by whoever is writing an adapter
// against the contract, and "invalid argument" would tell neither anything.
// ---------------------------------------------------------------------------

/// Text where a number was declared.
pub const WANTED_NUMBER: Word = Word::saying(
    "capability.argument.wanted-number",
    "give {argument} as a number, not as text",
)
.noting("{argument} is the name a verb declares an argument under and is never translated.");

/// A number where text was declared.
pub const WANTED_TEXT: Word = Word::saying(
    "capability.argument.wanted-text",
    "give {argument} as text, not as a number",
);

/// Nothing, or only spaces.
pub const ARGUMENT_EMPTY: Word = Word::saying(
    "capability.argument.empty",
    "say what {argument} is — it cannot be blank",
);

/// A relative path.
pub const ARGUMENT_NOT_A_FULL_PATH: Word = Word::saying(
    "capability.argument.not-a-full-path",
    "give {argument} as a full path, so it means the same thing wherever it is read",
);

/// A path with `..` in it.
pub const ARGUMENT_COULD_LEAD_ELSEWHERE: Word = Word::saying(
    "capability.argument.could-lead-elsewhere",
    "a path with .. in it can lead somewhere else — give {argument} as the path you mean",
)
.noting("\"..\" is how a path says \"the folder above\" and is never translated.");

/// A path where one name was declared.
pub const NOT_ONE_NAME: Word = Word::saying(
    "capability.argument.not-one-name",
    "{argument} is one name, not a path — give the name on its own, without folders in it",
);

/// Something that is not an application identifier.
pub const NOT_AN_IDENTIFIER: Word = Word::saying(
    "capability.argument.not-an-identifier",
    "give {argument} as an application identifier, like org.blender.Blender",
)
.noting("org.blender.Blender is an example identifier and is not translated.");

/// Longer than the verb allows.
///
/// Countable, because a length is a number of characters and how a language
/// counts is that language's business (item 9a).
pub const TOO_LONG: Counted = Counted {
    named: "capability.argument.too-long",
    number: "longest",
    one: "{argument} is longer than one character — shorten it",
    other: "{argument} is longer than {longest} characters — shorten it",
    note: "{longest} is the most characters the argument may be, so the sentence means \"longer \
           than this many\". {argument} is the name a verb declares an argument under and is \
           never translated.",
};

/// A character that cannot be read in a sentence.
pub const NOT_PRINTABLE: Word = Word::saying(
    "capability.argument.not-printable",
    "{argument} contains a character that cannot be shown — retype it in ordinary text",
)
.noting(
    "The character is left out of the sentence on purpose: it is what made the text unreadable, \
     and showing it would do the same to this refusal.",
);

/// A number outside the range the verb declared.
pub const OUT_OF_RANGE: Word = Word::saying(
    "capability.argument.out-of-range",
    "give {argument} as a number between {least} and {most}",
)
.noting("Both ends are included: {least} and {most} are themselves allowed.");

/// Something that is not one of the options.
pub const NOT_ON_THE_LIST: Word = Word::saying(
    "capability.argument.not-on-the-list",
    "{argument} has to be one of: {options}",
)
.noting(
    "{options} is a list of values a verb wrote down, separated by commas. They are values rather \
     than words and are never translated; the punctuation before them is yours to place.",
);

// ---------------------------------------------------------------------------
// Something that never became a call at all — [`crate::CallError`].
// ---------------------------------------------------------------------------

/// A verb that is not on the list.
pub const NO_SUCH_VERB: Word = Word::saying(
    "capability.call.no-such-verb",
    "there is no verb called {verb} — the list is closed, so a verb that is not on it does not \
     exist",
)
.noting(
    "{verb} is the name that was asked for, which came from outside and is never translated. \
     \"The list is closed\" is law 2: a capability nobody wrote down does not exist.",
);

/// An argument the verb does not take.
pub const NO_SUCH_ARGUMENT: Word = Word::saying(
    "capability.call.no-such-argument",
    "{verb} does not take {argument}",
)
.noting("Both gaps are names in the contract and are never translated.");

/// An argument the verb takes, that nothing gave.
pub const ARGUMENT_MISSING: Word = Word::saying(
    "capability.call.missing",
    "{verb} needs {argument} — {purpose}",
)
.noting(
    "{purpose} is what the verb says the argument is for. It arrives in the language the verb was \
     declared in, which is English until the crate that declares the verb hands its own words \
     over.",
);

/// The same argument given twice.
pub const SAME_ARGUMENT_TWICE: Word = Word::saying(
    "capability.call.same-argument-twice",
    "{argument} was given twice — a call gives each argument one value",
);

/// A call whose sentence could not be filled in.
pub const UNSAYABLE: Word = Word::saying(
    "capability.call.unsayable",
    "{verb} could not be put into a sentence to approve, so nothing was done — this is a verb to \
     declare again, not a call to make again",
)
.noting(
    "This is read by whoever wrote the verb rather than by whoever asked for it: a verb that \
     passed its own declaration checks cannot cause it.",
);

// ---------------------------------------------------------------------------
// A change that was never put to anybody — [`crate::ProposalError`].
// ---------------------------------------------------------------------------

/// A read offered for approval.
pub const READ_DOES_NOT_WAIT: Word = Word::saying(
    "capability.proposal.read-does-not-wait",
    "{verb} answers a question rather than changing anything — run it in the turn instead of \
     asking about it",
)
.noting(
    "\"In the turn\" means \"while the person is waiting for the answer\". Asking somebody to \
     approve a question is how approving becomes a reflex, which is what this refusal exists to \
     prevent.",
);

/// A question that stands for no time at all.
pub const PROPOSAL_NO_TIME: Word = Word::saying(
    "capability.proposal.no-time",
    "say how long the question stands — one that lapses at once cannot be answered",
);

/// A question standing longer than this machine can represent.
pub const PROPOSAL_NO_END: Word = Word::saying(
    "capability.proposal.no-end",
    "a question has to lapse — choose how long this one stands for",
);

// ---------------------------------------------------------------------------
// A question that could not be answered — [`crate::AnswerError`].
// ---------------------------------------------------------------------------

/// A number that is not waiting for an answer.
pub const NOTHING_WAITING: Word = Word::saying(
    "capability.answer.nothing-waiting",
    "nothing is waiting to be approved under number {number} — it has been answered already, or \
     it was never asked",
)
.noting("{number} is the number a person answers a proposal by, as it is shown to them.");

/// A question that stood too long.
pub const LAPSED: Word = Word::saying(
    "capability.answer.lapsed",
    "\"{sentence}\" was proposed too long ago to answer — ask again if it is still wanted",
)
.noting(
    "The quotation marks are part of the sentence: use the ones your language writes. {sentence} \
     is the change that was proposed. It arrives in the language the verb was declared in, which \
     is not yet the reader's — see docs/quirks.md.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
pub const EVERY_WORD: [Word; 33] = [
    ANONYMOUS,
    NOTHING_NAMED,
    THE_WHOLE_MACHINE,
    GRANT_NOT_A_FULL_PATH,
    GRANT_COULD_LEAD_ELSEWHERE,
    GRANT_NO_TIME,
    GRANT_NO_END,
    A_FOLDER,
    A_FILE,
    AN_APPLICATION,
    HAS_EXPIRED,
    NEVER_GRANTED,
    CHANGE_WAITS,
    WANTED_NUMBER,
    WANTED_TEXT,
    ARGUMENT_EMPTY,
    ARGUMENT_NOT_A_FULL_PATH,
    ARGUMENT_COULD_LEAD_ELSEWHERE,
    NOT_ONE_NAME,
    NOT_AN_IDENTIFIER,
    NOT_PRINTABLE,
    OUT_OF_RANGE,
    NOT_ON_THE_LIST,
    NO_SUCH_VERB,
    NO_SUCH_ARGUMENT,
    ARGUMENT_MISSING,
    SAME_ARGUMENT_TWICE,
    UNSAYABLE,
    READ_DOES_NOT_WAIT,
    PROPOSAL_NO_TIME,
    PROPOSAL_NO_END,
    NOTHING_WAITING,
    LAPSED,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it keeps its English and its `Display` for
/// the reason [`crate::VerbError`] does. It exists because [`declare_into`] can
/// genuinely fail against a vocabulary that already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A countable string that could not be declared.
    #[error(transparent)]
    Counting(#[from] alo_strings::PluralError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn capability_words() -> Result<Vocabulary, WordsError> {
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
    vocabulary.counts(
        Plural::counting(
            TOO_LONG.key(),
            TOO_LONG.number,
            TOO_LONG.one,
            TOO_LONG.other,
        )?
        .noting(TOO_LONG.note)?,
    )?;
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
        assert_eq!(Key::named(TOO_LONG.named), Ok(TOO_LONG.key()));
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        assert!(!named.contains(TOO_LONG.named));
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "capability", "{}", word.named());
        }
        assert_eq!(TOO_LONG.key().area(), "capability");
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = capability_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 1);
        assert_eq!(vocabulary.counted().count(), 1);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = capability_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// The sentences that carry something off the machine — a name, an
    /// identifier, a number, a list of options — say so, because a translator
    /// with no product in front of them cannot tell a gap that is a word from
    /// a gap that is a value.
    #[test]
    fn the_ones_a_translator_cannot_work_out_carry_a_note() {
        for word in [
            A_FOLDER,
            AN_APPLICATION,
            HAS_EXPIRED,
            NEVER_GRANTED,
            CHANGE_WAITS,
            NOT_ON_THE_LIST,
            NO_SUCH_VERB,
            ARGUMENT_MISSING,
            LAPSED,
        ] {
            assert!(word.note().is_some(), "{}", word.named());
        }
        assert!(!TOO_LONG.note.is_empty());
    }
}
