//! Every string this crate can say, and the English beside each one.
//!
//! Four groups: the three verbs an agent proposes a change to the network
//! through, with the options they offer; what became of a change; and why one
//! was not made. Every refusal says **what did not happen and what a person does
//! about it**.
//!
//! # Nothing a person reads names the machinery
//!
//! Not the network manager, not the part of the machine that makes changes with
//! authority over the whole of it, not a socket, a key, a password field or
//! *root*. A person joins a network, forgets one, turns Wi-Fi on or off and sets
//! a proxy; `docs/features.md` says so in those words, and a test at the bottom
//! of this file reads every sentence for the words that would undo it. A
//! network's name is data, filled into `{network}` exactly as the network
//! announced it, and never written into a sentence.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// The gap holding a network's name, and the argument that names one.
pub const NETWORK: &str = "network";

/// The gap holding whether Wi-Fi is turned on or off, and the argument.
pub const WIRELESS: &str = "wireless";

/// The gap holding what a change does to this conversation's connection, and
/// the argument.
pub const THIS_CONVERSATION: &str = "this_conversation";

// ---------------------------------------------------------------------------
// The three verbs.
// ---------------------------------------------------------------------------

/// What proposing to join a network is for.
pub const JOIN_PURPOSE: Word = Word::saying(
    "changing-network.verb.join-network.purpose",
    "propose joining a Wi-Fi network this machine can see",
)
.noting(
    "What an assistant's ability to propose joining a Wi-Fi network is for, shown in a list of \
     what it can do. It only ever proposes: nothing is joined until the person approves, and a \
     network's password is only ever asked of the person.",
);

/// What proposing to forget a network is for.
pub const FORGET_PURPOSE: Word = Word::saying(
    "changing-network.verb.forget-network.purpose",
    "propose forgetting a Wi-Fi network this machine has saved",
)
.noting(
    "What an assistant's ability to propose forgetting a saved Wi-Fi network is for, shown in a \
     list of what it can do. Nothing is forgotten until the person approves.",
);

/// What proposing to turn Wi-Fi on or off is for.
pub const WIRELESS_PURPOSE: Word = Word::saying(
    "changing-network.verb.switch-wireless.purpose",
    "propose turning this machine's Wi-Fi on or off",
)
.noting(
    "What an assistant's ability to propose turning Wi-Fi on or off is for, shown in a list of \
     what it can do. Nothing changes until the person approves.",
);

/// The argument naming the network.
pub const THE_NETWORK: Word = Word::saying(
    "changing-network.verb.network",
    "the Wi-Fi network, by the name it announces",
)
.noting("Describes one part of a proposal: which Wi-Fi network, by the name it shows in a list.");

/// The argument saying whether Wi-Fi is turned on or off.
pub const THE_WIRELESS: Word = Word::saying(
    "changing-network.verb.wireless",
    "whether Wi-Fi is turned on or off",
)
.noting("Describes one part of a proposal: turning Wi-Fi on, or turning it off.");

/// The argument saying what a change does to this conversation's connection.
pub const THE_CONVERSATION: Word = Word::saying(
    "changing-network.verb.this-conversation",
    "what the change does to this conversation's connection",
)
.noting(
    "Describes one part of a proposal: whether the conversation with the assistant stays \
     connected while the change is made, or loses its connection.",
);

/// The option: this conversation keeps its connection.
pub const KEEPS_ITS_CONNECTION: Word = Word::saying(
    "changing-network.verb.keeps-its-connection",
    "this conversation keeps its connection",
)
.noting(
    "Completes the sentence a person approves for a change to the network, when the change does \
     not cut the connection the conversation with the assistant is using.",
);

/// The option: this conversation loses its connection.
pub const LOSES_ITS_CONNECTION: Word = Word::saying(
    "changing-network.verb.loses-its-connection",
    "this conversation loses its connection until this machine is connected again",
)
.noting(
    "Completes the sentence a person approves for a change to the network, when the change cuts \
     the connection the conversation with the assistant is using. It is said before the person \
     approves, so they know the assistant stops answering until the machine is connected again.",
);

/// The option: on.
pub const ON: Word = Word::saying("changing-network.verb.on", "on")
    .noting("Fills the sentence \"turn Wi-Fi {wireless}\" when Wi-Fi is being turned on.");

/// The option: off.
pub const OFF: Word = Word::saying("changing-network.verb.off", "off")
    .noting("Fills the sentence \"turn Wi-Fi {wireless}\" when Wi-Fi is being turned off.");

/// The sentence a person approves to join a network.
pub const JOIN_SENTENCE: Word = Word::saying(
    "changing-network.verb.join-network.sentence",
    "join the Wi-Fi network {network}, and {this_conversation}",
)
.noting(
    "The sentence a person approves before an assistant's proposal to join a Wi-Fi network is \
     carried out. {network} is the network's name and is not translated. {this_conversation} is \
     one of the two phrases saying whether the conversation keeps its connection. If the network \
     asks for a password, the person is asked for it themselves, never the assistant.",
);

/// The sentence a person approves to forget a network.
pub const FORGET_SENTENCE: Word = Word::saying(
    "changing-network.verb.forget-network.sentence",
    "forget the Wi-Fi network {network}, so this machine no longer joins it on its own, and \
     {this_conversation}",
)
.noting(
    "The sentence a person approves before an assistant's proposal to forget a saved Wi-Fi network \
     is carried out. {network} is the network's name and is not translated. {this_conversation} \
     says whether the conversation keeps its connection.",
);

/// The sentence a person approves to turn Wi-Fi on or off.
pub const WIRELESS_SENTENCE: Word = Word::saying(
    "changing-network.verb.switch-wireless.sentence",
    "turn Wi-Fi {wireless}, and {this_conversation}",
)
.noting(
    "The sentence a person approves before an assistant's proposal to turn Wi-Fi on or off is \
     carried out. {wireless} is the word for on or off. {this_conversation} says whether the \
     conversation keeps its connection.",
);

// ---------------------------------------------------------------------------
// What became of a change.
// ---------------------------------------------------------------------------

/// A network was joined.
pub const JOINED: Word = Word::saying(
    "changing-network.joined",
    "This machine is connected to {network}",
)
.noting("Said once the Wi-Fi network the person chose has been joined. {network} is its name.");

/// A network was forgotten.
pub const FORGOTTEN: Word = Word::saying(
    "changing-network.forgotten",
    "{network} is forgotten. This machine no longer joins it on its own",
)
.noting(
    "Said once a saved Wi-Fi network the person chose to forget is gone. {network} is its name.",
);

/// Wi-Fi is on.
pub const WIRELESS_ON: Word = Word::saying("changing-network.wireless-on", "Wi-Fi is on")
    .noting("Said once Wi-Fi has been turned on.");

/// Wi-Fi is off.
pub const WIRELESS_OFF: Word = Word::saying("changing-network.wireless-off", "Wi-Fi is off")
    .noting("Said once Wi-Fi has been turned off.");

/// The machine's proxy is set.
pub const PROXY_SET: Word = Word::saying(
    "changing-network.proxy-set",
    "This machine's proxy is set, for everything on it that reaches the internet",
)
.noting(
    "Said once the proxy a person chose in Settings has been set for the whole machine. A proxy \
     is the server a company network sends connections to the internet through.",
);

// ---------------------------------------------------------------------------
// Why a change was not made.
// ---------------------------------------------------------------------------

/// No network in range by that name.
pub const NONE_VISIBLE_CALLED: Word = Word::saying(
    "changing-network.refused.none-visible-called",
    "No Wi-Fi network called {network} is in range, so nothing was changed. Move closer to it, \
     then choose it from the networks in Settings",
)
.noting(
    "Said when joining a Wi-Fi network by its name was asked for and no network this machine can \
     see now has that name. {network} is the name that was asked for.",
);

/// No saved network by that name.
pub const NONE_SAVED_CALLED: Word = Word::saying(
    "changing-network.refused.none-saved-called",
    "No Wi-Fi network called {network} is saved on this machine, so nothing was changed. The \
     networks in Settings show the ones that are",
)
.noting(
    "Said when forgetting a Wi-Fi network by its name was asked for and no network saved on this \
     machine has that name. {network} is the name.",
);

/// More than one network has that name.
pub const MORE_THAN_ONE_CALLED: Word = Word::saying(
    "changing-network.refused.more-than-one-called",
    "More than one Wi-Fi network is called {network}, so nothing was changed. Choose the one you \
     mean from the networks in Settings, where each shows whether it asks for a password",
)
.noting(
    "Said when two Wi-Fi networks share a name, for example one that asks for a password and an \
     open one using the same name. Joining either by name alone could join the wrong one. \
     {network} is the name.",
);

/// The network asks for an organisation's sign-in.
pub const ASKS_FOR_A_SIGN_IN: Word = Word::saying(
    "changing-network.refused.asks-for-a-sign-in",
    "{network} asks for an organisation's sign-in, which this machine cannot set up yet, so \
     nothing was changed. Ask whoever runs that network how to connect",
)
.noting(
    "Said when a Wi-Fi network needs a work or school account rather than a password. {network} \
     is the network's name.",
);

/// What the change does to this conversation was not said correctly.
pub const SAYS_THE_WRONG_THING: Word = Word::saying(
    "changing-network.refused.says-the-wrong-thing",
    "This change was not put to you, because it did not say correctly what it does to this \
     conversation's connection. Nothing was changed. Ask for it again",
)
.noting(
    "Said when an assistant proposed a change to the network and said the conversation would keep \
     its connection when it would lose it, or the other way round. The machine checks this before \
     asking the person, so they never approve a sentence that is wrong about it.",
);

/// What the change does to this conversation changed since it was approved.
pub const CONNECTION_CHANGED: Word = Word::saying(
    "changing-network.refused.connection-changed",
    "This machine's connection changed after you approved this, so what the change would do to \
     this conversation is no longer what you approved. Nothing was changed. Ask for it again",
)
.noting(
    "Said when, between the person approving a change to the network and it being made, the \
     machine moved to another connection, so the approved sentence about the conversation's \
     connection would no longer be true.",
);

/// The network settings could not be asked.
pub const NETWORK_NOT_ANSWERING: Word = Word::saying(
    "changing-network.refused.network-not-answering",
    "This machine's network settings are not answering, so nothing was changed. Restart the \
     machine, then try again",
)
.noting("Said when the part of this machine that manages its networks could not be asked.");

/// The approval was not accepted.
pub const APPROVAL_NOT_ACCEPTED: Word = Word::saying(
    "changing-network.refused.approval-not-accepted",
    "This change was not made, because its approval was not accepted: it was used already, came \
     too late, or was not given for this change. Nothing was changed. Ask for it again, and \
     approve it when you are asked",
)
.noting(
    "Said when a person's approval of a change to the network reached the part of the machine \
     that makes such changes and was refused there. An approval works once, for one change, and \
     only for about a minute after it is given.",
);

/// The change was accepted and could not be finished.
pub const COULD_NOT_FINISH: Word = Word::saying(
    "changing-network.refused.could-not-finish",
    "This machine could not finish the change to its network, for example because a password was \
     not accepted. Open the networks in Settings to see how they are now",
)
.noting(
    "Said when an approved change to the network was started and the machine could not complete \
     it: a network went out of range, its password was declined or not accepted, or Wi-Fi is \
     switched off by a key on the machine.",
);

/// The proxy could not be set.
pub const PROXY_NOT_SET: Word = Word::saying(
    "changing-network.refused.proxy-not-set",
    "This machine's proxy was not set. If your organisation manages this machine, it sets the \
     proxy. Otherwise open the proxy in Settings, check it, and set it again",
)
.noting(
    "Said when a proxy a person chose in Settings could not be set for the whole machine: the \
     organisation that manages the machine set one, or what was chosen could not be used.",
);

/// Nothing is being written down, so nothing is changed.
pub const NOT_BEING_KEPT: Word = Word::saying(
    "changing-network.refused.not-being-kept",
    "This machine has stopped keeping its record of changes to its settings, so it makes none. \
     Nothing was changed. Restart the machine",
)
.noting(
    "Said when the machine could not write down a change before making it. It makes no change it \
     cannot write down, so it makes none until it is restarted.",
);

/// Nothing is there to make the change.
pub const NOTHING_MAKES_CHANGES: Word = Word::saying(
    "changing-network.refused.nothing-makes-changes",
    "The part of this machine that changes its settings is not running, so nothing was changed. \
     Restart the machine",
)
.noting(
    "Said when a change a person approved could not be handed to the part of the machine that \
     makes changes to its settings for everybody who uses it.",
);

/// What was handed over was not a change to the network.
pub const NOT_A_NETWORK_CHANGE: Word = Word::saying(
    "changing-network.refused.not-a-network-change",
    "This was not a change to the network, so nothing was changed",
)
.noting(
    "Said when something other than an approved change to the network was handed to the part of \
     the machine that changes it. A person should never see it; if they do, the machine is wrong \
     rather than them.",
);

/// Every word that names something inside another sentence.
pub const THE_NAMES: [Word; 10] = [
    JOIN_PURPOSE,
    FORGET_PURPOSE,
    WIRELESS_PURPOSE,
    THE_NETWORK,
    THE_WIRELESS,
    THE_CONVERSATION,
    KEEPS_ITS_CONNECTION,
    LOSES_ITS_CONNECTION,
    ON,
    OFF,
];

/// Every word that is a sentence of its own.
pub const THE_SENTENCES: [Word; 21] = [
    JOIN_SENTENCE,
    FORGET_SENTENCE,
    WIRELESS_SENTENCE,
    JOINED,
    FORGOTTEN,
    WIRELESS_ON,
    WIRELESS_OFF,
    PROXY_SET,
    NONE_VISIBLE_CALLED,
    NONE_SAVED_CALLED,
    MORE_THAN_ONE_CALLED,
    ASKS_FOR_A_SIGN_IN,
    SAYS_THE_WRONG_THING,
    CONNECTION_CHANGED,
    NETWORK_NOT_ANSWERING,
    APPROVAL_NOT_ACCEPTED,
    COULD_NOT_FINISH,
    PROXY_NOT_SET,
    NOT_BEING_KEPT,
    NOTHING_MAKES_CHANGES,
    NOT_A_NETWORK_CHANGE,
];

/// Every word, the names first.
pub const EVERY_WORD: [Word; 31] = [
    JOIN_PURPOSE,
    FORGET_PURPOSE,
    WIRELESS_PURPOSE,
    THE_NETWORK,
    THE_WIRELESS,
    THE_CONVERSATION,
    KEEPS_ITS_CONNECTION,
    LOSES_ITS_CONNECTION,
    ON,
    OFF,
    JOIN_SENTENCE,
    FORGET_SENTENCE,
    WIRELESS_SENTENCE,
    JOINED,
    FORGOTTEN,
    WIRELESS_ON,
    WIRELESS_OFF,
    PROXY_SET,
    NONE_VISIBLE_CALLED,
    NONE_SAVED_CALLED,
    MORE_THAN_ONE_CALLED,
    ASKS_FOR_A_SIGN_IN,
    SAYS_THE_WRONG_THING,
    CONNECTION_CHANGED,
    NETWORK_NOT_ANSWERING,
    APPROVAL_NOT_ACCEPTED,
    COULD_NOT_FINISH,
    PROXY_NOT_SET,
    NOT_BEING_KEPT,
    NOTHING_MAKES_CHANGES,
    NOT_A_NETWORK_CHANGE,
];

/// Why this crate's words could not be declared.
#[derive(Debug, thiserror::Error)]
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
pub fn changing_network_words() -> Result<Vocabulary, WordsError> {
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

    /// Every key is one of this crate's, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in EVERY_WORD {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "changing-network", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The two groups are the whole list.
    #[test]
    fn the_groups_are_the_whole_list() {
        let mut all: Vec<&str> = THE_NAMES.iter().map(Word::named).collect();
        all.extend(THE_SENTENCES.iter().map(Word::named));
        let every: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(all, every);
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = changing_network_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note, and every gap is one of the three
    /// arguments.**
    #[test]
    fn every_word_carries_a_note_and_every_gap_is_an_argument() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(
                    [NETWORK, WIRELESS, THIS_CONVERSATION].contains(&gap.as_str()),
                    "{} has a gap called {gap}",
                    word.named()
                );
            }
        }
    }

    /// **Nothing here names the machinery, and nothing hedges.**
    #[test]
    fn nothing_here_names_the_machinery_or_hedges() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "networkmanager",
                "network manager",
                "nmcli",
                "d-bus",
                "dbus",
                "broker",
                "socket",
                "root",
                "token",
                "key",
                "digest",
                "identity",
                "ssid",
                "psk",
                "wpa",
                "daemon",
                "service",
                "luks",
                "tpm",
                "cups",
                "vpn",
                "probably",
                "might",
                "perhaps",
                "possibly",
            ] {
                assert!(
                    !said.contains(forbidden),
                    "{} says \"{forbidden}\"",
                    word.named()
                );
            }
        }
    }

    /// **Every refusal says what to do about it** — a second clause after what
    /// happened.
    #[test]
    fn every_refusal_says_what_to_do_about_it() {
        for word in [
            NONE_VISIBLE_CALLED,
            NONE_SAVED_CALLED,
            MORE_THAN_ONE_CALLED,
            ASKS_FOR_A_SIGN_IN,
            SAYS_THE_WRONG_THING,
            CONNECTION_CHANGED,
            NETWORK_NOT_ANSWERING,
            APPROVAL_NOT_ACCEPTED,
            COULD_NOT_FINISH,
            PROXY_NOT_SET,
            NOT_BEING_KEPT,
            NOTHING_MAKES_CHANGES,
        ] {
            let said = word.says();
            assert!(
                said.contains(". ") || said.contains(", then "),
                "{} names what happened and not what to do",
                word.named()
            );
        }
    }
}
