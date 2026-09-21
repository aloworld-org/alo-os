//! Every string this crate can say, and the English beside each one.
//!
//! Twenty-three, and they fall into four groups: who a notification is from,
//! why one is being held rather than shown, why one was never accepted at all,
//! and what a person is told when `notifying.toml` did not read.
//!
//! **Nothing a sender wrote is declared here.** A title and a body are the
//! sender's own text, in whatever language it was written in, and translating
//! them is neither possible nor ours — the rule `alo-applications` holds an
//! application's name to and `alo-egress` holds a host to. What is declared is
//! everything alo OS itself puts around them.
//!
//! **And no sentence about a held notification has a gap in it.**
//! [`HELD_LOCKED`] and its four companions say why nothing was shown and say
//! nothing about what did not appear, because the screen one of them may be
//! read on is the screen a stranger is guaranteed to be standing in front of.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// The gap a sentence naming who a notification is from fills.
pub const WHO: &str = "who";

/// The gap the agent's own clause fills with the name it is known by.
pub const AGENT: &str = "agent";

/// An agent sent it, named as ADR 0010 requires: the word *agent*, whatever
/// the agent is called.
pub const THE_AGENT: Word = Word::saying(
    "notifying.sender.the-agent",
    "{agent}, the agent on this machine",
)
.noting(
    "{agent} is a name like @alo and is never translated. The agent is the assistant built into \
     alo OS, and not a person. This clause goes inside the sentence saying who a notification is \
     from, and the words \"the agent\" are what tells somebody who cannot see the colour it is \
     drawn in that the machine is speaking on their behalf — so keep a word for the assistant in \
     every language.",
);

/// alo OS itself sent it.
pub const ALO_OS_ITSELF: Word = Word::saying("notifying.sender.alo-os-itself", "alo OS itself")
    .noting(
        "\"alo OS\" is the product's name and is never translated. The clause goes inside the \
         sentence saying who a notification is from, for a notification the machine sent about \
         itself rather than one an application or the agent sent.",
    );

/// Who a notification is from, which every notification says.
pub const SENT_BY: Word = Word::saying("notifying.sent-by", "Sent by {who}").noting(
    "{who} is a clause this same file declares for the agent and for alo OS, or the name and \
     identifier of an application, which are never translated. Shown beside a notification, and \
     read aloud by a screen reader before its title, so that who is speaking is never carried by \
     a colour alone.",
);

/// Nothing was shown because the machine is locked.
pub const HELD_LOCKED: Word = Word::saying(
    "notifying.held.locked",
    "Held until you unlock this machine",
)
.noting(
    "Said about a notification that arrived while the machine was locked. It is deliberately \
     about nothing in particular: it never names who sent it or what it said, because whoever is \
     reading a locked screen may not be the person the machine belongs to. Not an error.",
);

/// Nothing was shown because the person turned notifications off.
pub const HELD_YOU_ASKED_FOR_QUIET: Word = Word::saying(
    "notifying.held.you-asked-for-quiet",
    "Held because you turned notifications off",
)
.noting(
    "The person switched do-not-disturb on themselves. The sentence reminds them it was their \
     own choice, so it should not sound like a fault or a failure to deliver.",
);

/// Nothing was shown because it is inside the hours the person set aside.
pub const HELD_QUIET_HOURS: Word = Word::saying(
    "notifying.held.quiet-hours",
    "Held because this is inside the hours you set aside",
)
.noting(
    "The person set a stretch of the clock — usually the night — during which notifications \
     wait. Say \"hours\" the way a person says it about their own day, not as a technical \
     schedule.",
);

/// Nothing was shown because the screen is being shared or recorded.
pub const HELD_SCREEN_IS_SHARED: Word = Word::saying(
    "notifying.held.screen-is-shared",
    "Held because your screen is being shared or recorded",
)
.noting(
    "alo OS holds notifications on its own while anything is reading the screen, so that a \
     private message does not appear in front of a meeting or inside a recording. The person did \
     not ask for this and cannot turn it off; the sentence explains rather than apologises.",
);

/// Nothing was shown because the machine could not tell whether the screen is
/// being read.
pub const HELD_CANNOT_TELL_ABOUT_THE_SCREEN: Word = Word::saying(
    "notifying.held.cannot-tell-about-the-screen",
    "Held because this machine cannot tell whether your screen is being shared or recorded",
)
.noting(
    "Said when alo OS could not find out what is reading the screen. It holds notifications \
     rather than showing them, because a machine that does not know whether it is being recorded \
     must not put somebody's private message on the screen. Say it as a plain statement of what \
     the machine does not know; it is not the person's fault and not something they can fix.",
);

/// There is nothing waiting for the person.
pub const NOTHING_WAITING: Word = Word::saying("notifying.nothing-waiting", "Nothing is waiting")
    .noting(
        "Shown where the notifications a person missed would be listed, when there are none. Say \
         it as a calm statement rather than as an absence of something expected.",
    );

/// What arrived was not a request to send a notification.
pub const NOT_A_NOTIFICATION: Word = Word::saying(
    "notifying.not-sent.not-a-notification",
    "{application} did not ask to send a notification, so nothing has been shown",
)
.noting(
    "{application} is an identifier like org.example.Mail and is never translated. Said when an \
     application was allowed to do something else entirely — use the camera, print — and that \
     permission arrived where a permission to notify belongs. A fault in the program rather than \
     anything the person did.",
);

/// A notification with nothing to read at the top of it.
pub const NO_TITLE: Word = Word::saying(
    "notifying.not-sent.no-title",
    "A notification from {application} had nothing to read on it, so nothing has been shown",
)
.noting(
    "{application} is an identifier like org.example.Mail and is never translated. The program \
     sent a notification with an empty title, or one alo OS could not show in a single line. A \
     fault in the program.",
);

/// A notification offering something with nothing written on it.
pub const NO_LABEL: Word = Word::saying(
    "notifying.not-sent.no-label",
    "A notification from {application} offered something with no words on it, so nothing has \
     been shown",
)
.noting(
    "{application} is an identifier like org.example.Mail and is never translated. A \
     notification can offer a person things to do — a button — and this one offered one with \
     nothing readable on it. A fault in the program.",
);

/// A notification offering something its sender has no name for.
pub const NO_ACTION_NAME: Word = Word::saying(
    "notifying.not-sent.no-action-name",
    "A notification from {application} offered something it has no name for, so nothing has been \
     shown",
)
.noting(
    "{application} is an identifier like org.example.Mail and is never translated. The name here \
     is the program's own private name for what it offered, which alo OS hands back to it when \
     the person picks that; it is never shown to anybody. A fault in the program.",
);

/// A notification offering more than a person can be asked to read.
pub const TOO_MANY_THINGS_TO_DO: Word = Word::saying(
    "notifying.not-sent.too-many-things-to-do",
    "A notification from {application} offered {offered} things to do and a notification may \
     offer {most}, so nothing has been shown",
)
.noting(
    "{application} is an identifier like org.example.Mail and is never translated. {offered} and \
     {most} are plain whole numbers. A notification is a small thing that arrives uninvited; a \
     program trying to put a menu into one is a fault in the program.",
);

/// Quiet hours that begin and end at the same moment.
pub const BEGINS_AND_ENDS_AT_ONCE: Word = Word::saying(
    "notifying.quiet-hours.begins-and-ends-at-once",
    "hours that begin and end at the same time would say nothing about any part of the day",
)
.noting(
    "Shown while somebody is setting the stretch of the clock during which notifications wait, \
     if they put the same time in both boxes. Lower case at the start: it is said inside a \
     settings panel beside the two times, not as a sentence on its own.",
);

/// The file could not be read at all.
pub const KEPT_NOT_READ: Word = Word::saying(
    "notifying.kept.not-read",
    "your notification settings at {path} could not be read, so notifications behave as alo OS \
     ships them",
)
.noting(
    "{path} is a file on this machine and is never translated. A disk or a permission rather \
     than anything a person typed. \"alo OS\" is the product's name and is never translated.",
);

/// The file is there and is not notification settings.
pub const KEPT_NOT_UNDERSTOOD: Word = Word::saying(
    "notifying.kept.not-understood",
    "your notification settings at {path} are not settings alo OS can read, so nothing in the \
     file has been used and notifications behave as alo OS ships them",
)
.noting(
    "{path} is a file on this machine and is never translated. The important clause is that \
     nothing in the file was used: alo OS did not take the half it understood.",
);

/// The file stopped being settings at a line.
pub const KEPT_NOT_UNDERSTOOD_AT: Word = Word::saying(
    "notifying.kept.not-understood-at",
    "your notification settings at {path} stop making sense at line {line}, so nothing in the \
     file has been used and notifications behave as alo OS ships them",
)
.noting(
    "{path} is a file on this machine and is never translated. {line} is a plain whole number, \
     counted from one the way a text editor counts lines.",
);

/// The file says it is a shape this alo OS does not read.
pub const KEPT_ANOTHER_FORMAT: Word = Word::saying(
    "notifying.kept.another-format",
    "your notification settings at {path} were written for a different alo OS than this one, so \
     nothing in the file has been used and notifications behave as alo OS ships them",
)
.noting(
    "{path} is a file on this machine and is never translated. Most often a newer alo OS wrote \
     the file.",
);

/// The file names something that is not a notification setting.
pub const KEPT_UNKNOWN_KEY: Word = Word::saying(
    "notifying.kept.unknown-key",
    "your notification settings at {path} say {key}, which is not something alo OS can change \
     about notifications, so nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and {key} is a word as it was typed into it; neither is \
     translated. The key is named because it is what a person has to find in the file to fix it.",
);

/// The disk would not take the changed file.
pub const KEPT_NOT_WRITTEN: Word = Word::saying(
    "notifying.kept.not-written",
    "your notification settings at {path} could not be written, so nothing about notifications \
     has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A full disk or a folder the \
     person cannot write. The second clause is what they act on: the file is exactly as it was.",
);

/// The change could not be written as a file this alo OS reads back.
pub const KEPT_NOT_EXPRESSIBLE: Word = Word::saying(
    "notifying.kept.not-expressible",
    "this alo OS could not write that change into notification settings at {path} it can read \
     back again, so nothing about notifications has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A fault in alo OS rather than \
     anything the person did, said plainly: the change was refused before the file was touched.",
);

/// The file a change would replace is there and did not read, so it was kept.
pub const KEPT_NOT_REPLACED: Word = Word::saying(
    "notifying.kept.not-replaced",
    "your notification settings at {path} could not be read, so that change has not been written \
     over them and nothing about notifications has been changed — correct the file, or put \
     notifications back as alo OS ships them",
)
.noting(
    "{path} is a file on this machine and is never translated. Said when a person changes \
     something in Settings while the file, most often one they edited by hand, does not read: \
     alo OS keeps their file rather than replacing it, so a typing mistake in it is not lost. \
     The last clause gives the two ways on.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 23] = [
    THE_AGENT,
    ALO_OS_ITSELF,
    SENT_BY,
    HELD_LOCKED,
    HELD_YOU_ASKED_FOR_QUIET,
    HELD_QUIET_HOURS,
    HELD_SCREEN_IS_SHARED,
    HELD_CANNOT_TELL_ABOUT_THE_SCREEN,
    NOTHING_WAITING,
    NOT_A_NOTIFICATION,
    NO_TITLE,
    NO_LABEL,
    NO_ACTION_NAME,
    TOO_MANY_THINGS_TO_DO,
    BEGINS_AND_ENDS_AT_ONCE,
    KEPT_NOT_READ,
    KEPT_NOT_UNDERSTOOD,
    KEPT_NOT_UNDERSTOOD_AT,
    KEPT_ANOTHER_FORMAT,
    KEPT_UNKNOWN_KEY,
    KEPT_NOT_WRITTEN,
    KEPT_NOT_EXPRESSIBLE,
    KEPT_NOT_REPLACED,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so. It is a `Result` rather than an unwrap because a library
/// that panics on its own string table takes the surface with it, and because
/// [`declare_into`] can genuinely fail against a vocabulary that already holds
/// the key.
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
pub fn notifying_words() -> Result<Vocabulary, WordsError> {
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

    /// What we ship is held to the rule everybody else is held to.
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

    /// Every one of them sorts under this crate's area.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "notifying", "{}", word.named());
        }
    }

    /// The list declares, and a second declaration replaces nothing.
    #[test]
    fn the_whole_list_declares_once() {
        let mut vocabulary = notifying_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **No sentence about a notification being held has a gap in it.** A gap
    /// is the road a sender's name, a subject line or a first sentence could
    /// take onto a screen that is holding notifications precisely so that none
    /// of them appears.
    #[test]
    fn nothing_said_about_a_held_notification_has_a_gap_in_it() {
        for word in [
            HELD_LOCKED,
            HELD_YOU_ASKED_FOR_QUIET,
            HELD_QUIET_HOURS,
            HELD_SCREEN_IS_SHARED,
            HELD_CANNOT_TELL_ABOUT_THE_SCREEN,
            NOTHING_WAITING,
        ] {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{} has a gap in it",
                word.named()
            );
        }
    }

    /// Every word carries a note: a translator has to know that a title is the
    /// sender's own text and that *the agent* is not a person's name.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **The agent's clause says *the agent* in words.** ADR 0010's rule: the
    /// colour is never the only thing carrying it.
    #[test]
    fn the_agents_clause_says_the_agent_in_words() {
        assert!(THE_AGENT.says().contains("the agent"));
        assert_eq!(
            THE_AGENT.phrase().unwrap().source().gaps(),
            [AGENT.to_owned()]
        );
        assert_eq!(SENT_BY.phrase().unwrap().source().gaps(), [WHO.to_owned()]);
    }
}
