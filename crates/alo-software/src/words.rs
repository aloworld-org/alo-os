//! Every string this crate can say, and the English beside each one.
//!
//! Four groups: what became of an installation, an update or a removal; why one
//! was refused; the update a person is offered; and the one verb an agent
//! proposes an installation through. Every refusal says **what did not happen
//! and what a person does about it**, because *nothing was installed* on its
//! own leaves somebody holding a question.
//!
//! # Nothing a person reads names the machinery
//!
//! Not the tool that installs, not the service an application arrives from by
//! its tooling, not a repository, a remote, a key, a branch or a sandbox. A
//! person installs an application — `docs/features.md` says so in those words
//! — and a test at the bottom of this file reads every sentence and every note
//! for the words that would undo it. The name of a place applications come from
//! is data, filled into `{source}` exactly as this machine knows it, and never
//! written into a sentence.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// The gap holding an application's identifier.
pub const APPLICATION: &str = "application";

/// The gap holding the name of a place applications come from.
pub const SOURCE: &str = "source";

// ---------------------------------------------------------------------------
// What became of it.
// ---------------------------------------------------------------------------

/// An application was installed.
pub const INSTALLED: Word = Word::saying(
    "software.installed",
    "{application} is installed. It has been given nothing: it asks when it needs a file, the \
     camera or anything else, and you answer",
)
.noting(
    "Said once an application a person chose has arrived on this machine. {application} is its \
     identifier, like org.gnome.TextEditor, and is not translated. The second sentence is the \
     promise that matters: a new application can reach nothing until the person allows it \
     something.",
);

/// An update is ready and waits for the person.
pub const UPDATE_OFFERED: Word = Word::saying(
    "software.update-offered",
    "A newer version of {application} is ready. It is updated when you choose, and never while it \
     is open",
)
.noting(
    "Shown after this machine found a newer version of an application. Nothing has changed yet: \
     the person chooses when, and an application that is open is never updated underneath the \
     person using it. {application} is the application's identifier and is not translated.",
);

/// An application was updated.
pub const UPDATED: Word = Word::saying(
    "software.updated",
    "{application} is updated. Nothing else was closed or restarted",
)
.noting(
    "Said once a newer version of an application is in place. \"Nothing else\" means no other \
     application and not the computer itself: an application's update never restarts the machine. \
     {application} is the application's identifier.",
);

/// An application was removed, and its grants with it.
pub const REMOVED: Word = Word::saying(
    "software.removed",
    "{application} is removed, and everything it had been allowed has ended",
)
.noting(
    "Said once an application has been taken off this machine. Everything the person had allowed \
     it — a folder, the camera — ended in the same moment, so installing it again later starts \
     with nothing allowed. {application} is the application's identifier.",
);

// ---------------------------------------------------------------------------
// Why it was refused.
// ---------------------------------------------------------------------------

/// The place named is not one this machine installs from.
pub const NOT_ENABLED: Word = Word::saying(
    "software.refused.not-enabled",
    "{source} is not a place this machine installs applications from, so nothing was installed. \
     Choose one of the places listed under Software in Settings",
)
.noting(
    "Said when an application was asked for from a place that is not set up on this machine. \
     {source} is the name of that place exactly as it was given, and is not translated. Software \
     and Settings are the names of places in this machine's own interface and should be the words \
     used there.",
);

/// An organisation does not permit this place.
pub const OUTSIDE_THE_BOUND_BY_AN_ADMINISTRATOR: Word = Word::saying(
    "software.refused.outside-the-bound.administrator",
    "The organisation that manages this machine does not permit installing applications from \
     {source}, so nothing was installed or updated from it. Someone who looks after this machine \
     can say which places are permitted",
)
.noting(
    "Said on a machine an organisation manages, when a place applications come from is set up but \
     the organisation's rule does not include it. The person is told who made the rule, because a \
     refusal nobody can account for is worse than none. {source} is the name of the place.",
);

/// This machine's own description does not permit this place.
pub const OUTSIDE_THE_BOUND_BY_THIS_PERSON: Word = Word::saying(
    "software.refused.outside-the-bound.this-person",
    "This machine's own settings do not permit installing applications from {source}, so nothing \
     was installed or updated from it. No organisation set this: it was set on this machine, and \
     can be changed there",
)
.noting(
    "Said on a machine nobody else manages, when the machine's owner limited where applications may \
     come from and this place is not included. It says plainly that no organisation is involved. \
     {source} is the name of the place.",
);

/// The place is set up without proving what it sends is its own.
pub const NOT_VERIFIED: Word = Word::saying(
    "software.refused.not-verified",
    "{source} is set up without checking that what it sends really comes from it, so this machine \
     does not install or update anything from it. Nothing was installed",
)
.noting(
    "Said when a place applications come from was set up with its signature checking turned off. \
     This machine refuses such a place rather than trusting it. {source} is the name of the place.",
);

/// What arrived could not be shown to come from the place.
pub const SIGNATURE_NOT_SHOWN: Word = Word::saying(
    "software.refused.signature-not-shown",
    "What {source} sent for {application} could not be shown to come from {source}, so nothing was \
     installed and nothing it sent was kept. Try again later, or ask whoever runs {source}",
)
.noting(
    "Said when an application or its update arrived without a valid signature from the place it \
     should have come from — it may have been damaged or replaced on the way. {source} is the name \
     of the place and {application} the application's identifier; neither is translated.",
);

/// The place does not offer this application.
pub const NOT_OFFERED: Word = Word::saying(
    "software.refused.not-offered",
    "{source} does not offer {application}, so nothing was installed. Check the application's \
     name, or choose another place it comes from",
)
.noting(
    "Said when the place named has no application by that identifier. {application} is the \
     identifier exactly as it was given, and {source} the name of the place.",
);

/// The application is already on this machine.
pub const ALREADY_INSTALLED: Word = Word::saying(
    "software.refused.already-installed",
    "{application} is already installed on this machine, so it was not installed again. Updates \
     for it are offered in Software",
)
.noting(
    "Said when a person or an assistant asked to install an application that is already here. \
     {application} is its identifier. Software is the name of a place in this machine's own \
     interface.",
);

/// The application is not on this machine.
pub const NOT_INSTALLED: Word = Word::saying(
    "software.refused.not-installed",
    "{application} is not installed on this machine, so there is nothing to update or remove. \
     Check the application's name",
)
.noting(
    "Said when an update or a removal named an application that is not here. {application} is the \
     identifier exactly as it was given.",
);

/// The application is open, so its update waits.
pub const STILL_OPEN: Word = Word::saying(
    "software.refused.still-open",
    "{application} is open, so it was not updated. Close it and choose to update it again: an \
     update never closes an application by itself",
)
.noting(
    "Said when a person chose an update for an application they are still using. Nothing was \
     closed and nothing was lost; the update simply waits. {application} is the application's \
     identifier.",
);

/// The place has no address this machine can reach.
pub const NOWHERE_TO_REACH: Word = Word::saying(
    "software.refused.nowhere-to-reach",
    "{source} is not set up with an address on the network this machine can reach, so nothing was \
     installed or updated from it. Whoever set it up can give it one",
)
.noting(
    "Said when a place applications come from is set up in a way this machine cannot use — for \
     example with no address, or one written in a form it does not read. {source} is the name of \
     the place.",
);

/// The part of the machine that installs did not respond.
pub const DID_NOT_ANSWER: Word = Word::saying(
    "software.refused.did-not-answer",
    "The part of this machine that installs applications did not respond, so nothing was changed. \
     Restarting this machine starts it again",
)
.noting(
    "Said when installing, updating or removing could not begin or did not finish because the \
     machine's own installing component failed. \"The part of this machine that installs \
     applications\" is deliberate: the person never needs to know what it is called.",
);

// ---------------------------------------------------------------------------
// The verb an agent proposes an installation through.
// ---------------------------------------------------------------------------

/// What the verb is for.
pub const VERB_PURPOSE: Word = Word::saying(
    "software.verb.install-application.purpose",
    "install an application from a place this machine installs applications from",
)
.noting(
    "What an assistant's ability to propose installing an application is for, shown in a list of \
     what it can do. It only ever proposes: nothing is installed until the person approves.",
);

/// The argument naming the application.
pub const VERB_APPLICATION: Word = Word::saying(
    "software.verb.install-application.application",
    "the application to install, by its identifier",
)
.noting(
    "Describes one part of the proposal: which application. An identifier is the exact name this \
     machine knows an application by, like org.gnome.TextEditor.",
);

/// The argument naming the place.
pub const VERB_SOURCE: Word = Word::saying(
    "software.verb.install-application.source",
    "the place to install it from, by the name this machine gives that place",
)
.noting(
    "Describes one part of the proposal: where the application comes from. It is one of the places \
     listed under Software in Settings.",
);

/// The sentence a person approves.
pub const VERB_SENTENCE: Word = Word::saying(
    "software.verb.install-application.sentence",
    "install {application} from {source}",
)
.noting(
    "The sentence a person approves before an assistant's proposal to install an application is \
     carried out. {application} is the application's identifier and {source} the name of the place \
     it comes from; neither is translated. Approving it installs once, and the application arrives \
     allowed nothing.",
);

/// Every word that names something, read inside another sentence.
pub const THE_NAMES: [Word; 3] = [VERB_PURPOSE, VERB_APPLICATION, VERB_SOURCE];

/// Every word that is a line or a sentence of its own.
pub const THE_SENTENCES: [Word; 16] = [
    INSTALLED,
    UPDATE_OFFERED,
    UPDATED,
    REMOVED,
    NOT_ENABLED,
    OUTSIDE_THE_BOUND_BY_AN_ADMINISTRATOR,
    OUTSIDE_THE_BOUND_BY_THIS_PERSON,
    NOT_VERIFIED,
    SIGNATURE_NOT_SHOWN,
    NOT_OFFERED,
    ALREADY_INSTALLED,
    NOT_INSTALLED,
    STILL_OPEN,
    NOWHERE_TO_REACH,
    DID_NOT_ANSWER,
    VERB_SENTENCE,
];

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 19] = [
    VERB_PURPOSE,
    VERB_APPLICATION,
    VERB_SOURCE,
    INSTALLED,
    UPDATE_OFFERED,
    UPDATED,
    REMOVED,
    NOT_ENABLED,
    OUTSIDE_THE_BOUND_BY_AN_ADMINISTRATOR,
    OUTSIDE_THE_BOUND_BY_THIS_PERSON,
    NOT_VERIFIED,
    SIGNATURE_NOT_SHOWN,
    NOT_OFFERED,
    ALREADY_INSTALLED,
    NOT_INSTALLED,
    STILL_OPEN,
    NOWHERE_TO_REACH,
    DID_NOT_ANSWER,
    VERB_SENTENCE,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// say so — and [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
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
pub fn software_words() -> Result<Vocabulary, WordsError> {
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
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every key is a key, in this crate's area, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in EVERY_WORD {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "software", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The two groups are the whole list.
    #[test]
    fn the_two_groups_are_the_whole_list() {
        let mut both: Vec<&str> = THE_NAMES.iter().map(Word::named).collect();
        both.extend(THE_SENTENCES.iter().map(Word::named));
        let every: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(both, every);
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = software_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note**, and every gap is one this crate fills.
    #[test]
    fn every_word_carries_a_note_and_only_gaps_this_crate_fills() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(
                    [APPLICATION, SOURCE].contains(&gap.as_str()),
                    "{} has a gap called {gap}",
                    word.named()
                );
            }
        }
    }

    /// **Nothing here hedges and nothing names the machinery** — not in the
    /// sentence and not in the note a translator reads, because a translator
    /// handed the tool's name has been handed our plumbing.
    #[test]
    fn nothing_here_hedges_or_names_the_machinery() {
        for word in EVERY_WORD {
            for text in [word.says(), word.note().unwrap_or_default()] {
                let text = text.to_lowercase();
                for forbidden in [
                    "probably",
                    "might",
                    "perhaps",
                    "possibly",
                    "flatpak",
                    "flathub",
                    "ostree",
                    "remote",
                    "repository",
                    "repo ",
                    "gpg",
                    "branch",
                    "sandbox",
                    "package",
                    "error",
                    "d-bus",
                    "portal",
                ] {
                    assert!(
                        !text.contains(forbidden),
                        "{} says \"{forbidden}\"",
                        word.named()
                    );
                }
            }
        }
    }

    /// **Every refusal says what did not happen and what to do about it** — a
    /// second sentence or clause after the trouble, never the trouble alone.
    #[test]
    fn every_refusal_says_what_to_do_about_it() {
        for word in [
            NOT_ENABLED,
            OUTSIDE_THE_BOUND_BY_AN_ADMINISTRATOR,
            OUTSIDE_THE_BOUND_BY_THIS_PERSON,
            NOT_VERIFIED,
            SIGNATURE_NOT_SHOWN,
            NOT_OFFERED,
            ALREADY_INSTALLED,
            NOT_INSTALLED,
            STILL_OPEN,
            NOWHERE_TO_REACH,
            DID_NOT_ANSWER,
        ] {
            let said = word.says();
            assert!(
                said.contains("nothing") || said.contains("not"),
                "{}",
                word.named()
            );
            assert!(
                said.contains(". ") || said.contains(": "),
                "{} names the trouble and not what to do",
                word.named()
            );
        }
    }
}
