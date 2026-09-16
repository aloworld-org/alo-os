//! Every string this crate can say, and the English beside each one.
//!
//! Four groups: what the machine's proxy is, whose it is, why a person could not
//! change it, and why a road out was not taken. Every refusal says **what did
//! not happen and what a person does about it** — *nothing was sent* on its own
//! leaves somebody staring at a window that did nothing.
//!
//! # Not one sentence has a gap in it
//!
//! Deliberately. What could go into a gap here is a proxy's address, a host a
//! road was going to, or what a program said — and each of those is either
//! somebody else's text or a fact a person cannot act on. The address of the
//! proxy belongs in the settings panel beside the sentence, where it is data
//! shown as it was written; the host is what the egress indicator already names;
//! and what the program said is kept for whoever administers the machine and
//! never shown (`refusing.rs`).
//!
//! # Nothing a person reads names the machinery
//!
//! Not the script's own name, not the notation it is written in, not the
//! variables an application is given, not the portal and not the bus it is on. A
//! person sets *the proxy for this machine*; the rest is how. A test at the
//! bottom of this file reads every sentence and every note for the words that
//! would undo it.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What the machine's proxy is.
// ---------------------------------------------------------------------------

/// No proxy is set.
pub const THE_PROXY_NONE: Word = Word::saying(
    "proxy.the-proxy.none",
    "This machine reaches the network directly",
)
.noting(
    "Shown in settings where a person chooses how this machine reaches the network. It is a \
     statement of fact about the machine, not an instruction.",
);

/// An address was set by hand.
pub const THE_PROXY_MANUAL: Word = Word::saying(
    "proxy.the-proxy.manual",
    "This machine reaches the network through the proxy set here",
)
.noting(
    "Shown in settings beside the address itself, which is data and appears as it was written \
     rather than inside this sentence.",
);

/// An address a configuration is at was set.
pub const THE_PROXY_AUTOMATIC: Word = Word::saying(
    "proxy.the-proxy.automatic",
    "This machine asks the network where each connection should go",
)
.noting(
    "Shown in settings where a person has given the address of an automatic configuration. \
     \"the network\" means the network this machine is on — whoever runs it publishes the rule. \
     Do not name the notation the rule is written in.",
);

// ---------------------------------------------------------------------------
// Whose it is.
// ---------------------------------------------------------------------------

/// The organisation that manages this machine set it.
pub const SET_BY_AN_ORGANISATION: Word = Word::saying(
    "proxy.set-by.an-organisation",
    "Your organisation set this",
)
.noting(
    "Shown beside the proxy on a machine an organisation manages. The person is told rather than \
     shown a setting that quietly does nothing.",
);

/// The person whose machine it is set it.
pub const SET_BY_THIS_PERSON: Word = Word::saying("proxy.set-by.this-person", "You set this")
    .noting(
        "Shown beside the proxy on a person's own machine. \"You\" is the person in front of the \
         machine.",
    );

/// A person tried to change a proxy their organisation set.
pub const NOT_CHANGED_AN_ORGANISATION_SET_IT: Word = Word::saying(
    "proxy.not-changed.an-organisation-set-it",
    "Your organisation set the proxy for this machine, so it cannot be changed here. Ask whoever \
     manages it",
)
.noting(
    "Said when somebody changes the proxy on a machine an organisation manages. The second \
     sentence is the useful half: it says who can change it.",
);

// ---------------------------------------------------------------------------
// Why a road out was not taken.
// ---------------------------------------------------------------------------

/// The rule could not be worked out, so nothing was sent.
pub const THE_PROXY_COULD_NOT_BE_WORKED_OUT: Word = Word::saying(
    "proxy.refused.could-not-be-worked-out",
    "This machine could not work out how to reach that, so nothing was sent. Ask whoever runs \
     this network",
)
.noting(
    "Said when the machine asks the network where a connection should go and gets no answer it \
     can use. \"nothing was sent\" is the part that must survive translation: the person needs to \
     know the connection did not quietly happen another way.",
);

/// This machine has nothing that works an automatic configuration out.
pub const NOTHING_WORKS_OUT_THE_PROXY: Word = Word::saying(
    "proxy.refused.nothing-works-it-out",
    "This machine cannot read the rule this network publishes, so nothing was sent. Set the proxy \
     here by hand, or ask whoever runs this network for its address",
)
.noting(
    "Said on a machine that has nothing able to read an automatic configuration. \"nothing was \
     sent\" must survive translation. The second sentence gives the person two things they can \
     actually do.",
);

// ---------------------------------------------------------------------------
// What somebody typed that was not a proxy.
// ---------------------------------------------------------------------------

/// Nothing where the proxy goes.
pub const ADDRESS_NOWHERE: Word = Word::saying(
    "proxy.address.nowhere",
    "Type the address of the proxy, or choose to reach the network directly",
)
.noting("Said when the address field is empty. It says what to do rather than what went wrong.");

/// The address is not one line.
pub const ADDRESS_NOT_ONE_LINE: Word = Word::saying(
    "proxy.address.not-one-line",
    "That proxy address is not one line. Type the name of the proxy on its own",
)
.noting(
    "Said when what was typed or pasted holds a line break or a control character. The sentence \
     deliberately does not repeat what was typed.",
);

/// The address is longer than a host can be.
pub const ADDRESS_TOO_LONG: Word = Word::saying(
    "proxy.address.too-long",
    "That proxy address is longer than any host can be. Check it against what you were given",
)
.noting("Said when the host is longer than a hostname may be.");

/// The port names nothing.
pub const ADDRESS_NO_PORT: Word = Word::saying(
    "proxy.address.no-port",
    "Type the port the proxy answers on, beside its address",
)
.noting(
    "Said when the port is zero, which names no service. \"port\" is the number beside a proxy's \
     address, such as 8080.",
);

/// A name and a password were written into the address.
pub const ADDRESS_CARRIES_A_PASSWORD: Word = Word::saying(
    "proxy.address.carries-a-password",
    "That proxy address has a name and a password written into it. Type the address on its own, \
     and the name and the password in their own fields — the password is then kept safely and \
     never written into a file",
)
.noting(
    "Said when somebody pastes an address of the form name:password@host. The last clause is the \
     reason and is worth keeping: it is why this is refused rather than accepted.",
);

/// A name was given with nowhere for its password.
pub const ADDRESS_NO_PLACE_FOR_THE_PASSWORD: Word = Word::saying(
    "proxy.address.no-place-for-the-password",
    "Type the name the proxy asks for, or leave both the name and the password empty",
)
.noting("Said when a password was given with no name to go with it, or a name with no password.");

/// The address is not the name of a host.
pub const ADDRESS_NOT_A_HOST: Word = Word::saying(
    "proxy.address.not-a-host",
    "That is not the name of a proxy. Type the name of the machine on its own, without http:// in 
     front of it and without anything after it",
)
.noting(
    "Said when what was typed is not a host name or an address — usually a whole web address 
     pasted into the field, or a path. The advice is the useful half.",
);

/// Nothing was typed into the password field.
pub const PASSWORD_BLANK: Word = Word::saying(
    "proxy.password.blank",
    "Type the password the proxy asks for, or leave the name empty as well",
)
.noting(
    "Said when the password field is empty. It never repeats what was typed, because a password \
     in a message is a password in a log.",
);

/// The password holds something that cannot be sent.
pub const PASSWORD_NOT_SENDABLE: Word = Word::saying(
    "proxy.password.not-sendable",
    "That password holds something that cannot be sent. Type it again rather than pasting it",
)
.noting(
    "Said when the password holds a control character — usually pasted along with it. It never \
     repeats what was typed.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// string left off it is a string nothing can say.
pub const EVERY_WORD: [Word; 17] = [
    THE_PROXY_NONE,
    THE_PROXY_MANUAL,
    THE_PROXY_AUTOMATIC,
    SET_BY_AN_ORGANISATION,
    SET_BY_THIS_PERSON,
    NOT_CHANGED_AN_ORGANISATION_SET_IT,
    THE_PROXY_COULD_NOT_BE_WORKED_OUT,
    NOTHING_WORKS_OUT_THE_PROXY,
    ADDRESS_NOWHERE,
    ADDRESS_NOT_ONE_LINE,
    ADDRESS_TOO_LONG,
    ADDRESS_NO_PORT,
    ADDRESS_CARRIES_A_PASSWORD,
    ADDRESS_NO_PLACE_FOR_THE_PASSWORD,
    ADDRESS_NOT_A_HOST,
    PASSWORD_BLANK,
    PASSWORD_NOT_SENDABLE,
];

/// Why this crate's own list would not declare.
///
/// Either a word that is not a phrase, or a key the vocabulary already holds.
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
pub fn proxy_words() -> Result<Vocabulary, WordsError> {
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

    /// Everything this crate declares, as one list.
    fn everything() -> Vec<Word> {
        EVERY_WORD.to_vec()
    }

    /// Every key is a key, in this crate's area, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in everything() {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "proxy", "{}", word.named());
        }
        let named: BTreeSet<&str> = everything().iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), everything().len());
    }

    /// **Not one sentence has a gap in it**, which is this file's decision: a
    /// gap here would hold an address, a host or what a program said, and none
    /// of those belongs inside a sentence.
    #[test]
    fn no_sentence_has_a_gap_in_it() {
        for word in everything() {
            assert!(
                !word.says().contains('{'),
                "{}: {}",
                word.named(),
                word.says()
            );
        }
    }

    /// **Nothing a person reads names the machinery.** Not the notation a
    /// network's rule is written in, not the variables an application is given,
    /// not the portal and not the bus.
    #[test]
    fn nothing_a_person_reads_names_the_machinery() {
        for word in everything() {
            let read =
                format!("{} {}", word.says(), word.note().unwrap_or_default()).to_lowercase();
            for machinery in [
                "pac",
                "wpad",
                "javascript",
                "d-bus",
                "dbus",
                "portal",
                "socks",
                "keyring",
                "environment variable",
                "http_proxy",
            ] {
                assert!(
                    !read.contains(machinery),
                    "{} names {machinery}: {read}",
                    word.named()
                );
            }
        }
    }

    /// Every string a translator is handed has the note they cannot work
    /// without: these are refusals and statements about somebody's network, and
    /// none of them is self-explanatory out of context.
    #[test]
    fn every_string_carries_a_note_for_whoever_translates_it() {
        for word in everything() {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// The list declares, and nothing is replaced if it is declared twice.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = proxy_words().unwrap();
        assert_eq!(vocabulary.how_many(), everything().len());
        assert!(matches!(
            declare_into(&mut vocabulary).unwrap_err(),
            WordsError::List(_)
        ));
        assert_eq!(vocabulary.how_many(), everything().len());
    }
}
