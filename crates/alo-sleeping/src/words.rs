//! Every string this crate can say, and the English beside each one.
//!
//! They fall into five groups a person meets in five places: **why this machine
//! is staying awake**, which is the answer to *why won't this laptop sleep* and
//! the reason this crate names every keeper; **what alo OS itself is holding**,
//! which is what the machine's own list of holds says beside alo OS's name;
//! **what became of the agent's work** when the machine woke; the **refusals**;
//! and **the person's settings**, with what is said about the file they are
//! kept in.
//!
//! # What is deliberately not here
//!
//! **Anything we rent.** No sentence names `logind`, an inhibitor, a suspend
//! state or a lid switch: a person is told that their machine is staying awake
//! and why, not what keeps it so. `tests/every_sentence_here_is_collected.rs`
//! holds that against the machine's list of rented names.
//!
//! **What an application said about itself.** The inhibit portal carries a
//! reason an application wrote, and it is never shown: a system surface quoting
//! an application's own text is a surface an application can put words on.
//! What is shown is the application's identifier, which the sandbox vouches for.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Why this machine is staying awake — [`crate::Keeper`] and
// [`crate::StaysAwake`]. Each is a whole sentence, because each is the whole
// answer somebody asked for.
// ---------------------------------------------------------------------------

/// The person's own setting is keeping the machine awake.
pub const KEPT_AWAKE_BY_YOUR_SETTING: Word = Word::saying(
    "sleeping.kept-awake.by-your-setting",
    "This machine is staying awake because you set it to stay awake",
)
.noting(
    "Shown when somebody asks why their machine has not gone to sleep on its own, and in the list \
     of what is holding it awake. The person turned on a setting that keeps the machine awake; \
     the sentence reminds them it was their own choice, so it should not sound like a fault.",
);

/// An application holding the inhibit portal is keeping the machine awake.
pub const KEPT_AWAKE_BY_AN_APPLICATION: Word = Word::saying(
    "sleeping.kept-awake.by-an-application",
    "This machine is staying awake because {application} asked to keep it awake",
)
.noting(
    "{application} is the identifier of an application, like org.example.Slides, and is \
     never translated. Typically a presentation or a video that should not be interrupted by the \
     screen going to sleep. The application was allowed to ask by a permission the person gave \
     it. Shown so the person can tell which program is keeping their machine awake.",
);

/// A turn that is running is keeping the machine awake.
pub const KEPT_AWAKE_BY_THE_AGENT: Word = Word::saying(
    "sleeping.kept-awake.by-the-agent",
    "This machine is staying awake because the agent is still working on what you asked",
)
.noting(
    "The agent is the assistant built into alo OS, and not a person. The person asked it to do something and it is \
     not finished; the machine stays awake only until that piece of work ends. Shown beside the \
     agent's own mark and name, so do not add a name for the assistant here.",
);

/// The lid is closed and the machine is awake, as the person chose.
pub const AWAKE_WITH_THE_LID_CLOSED: Word = Word::saying(
    "sleeping.lid.stays-awake",
    "This machine is staying awake with its lid closed because another display is attached and \
     you chose that",
)
.noting(
    "Said when a laptop's lid is closed while an external monitor is plugged in, and the person \
     has chosen in Settings that the laptop keeps running in that case. A display here is a \
     monitor or a projector.",
);

// ---------------------------------------------------------------------------
// What alo OS itself holds — [`crate::Inhibit`]. Written into the machine's own
// list of what is holding it awake, beside alo OS's name, in the person's
// language.
// ---------------------------------------------------------------------------

/// alo OS decides what closing the lid does.
pub const HOLDING_THE_LID: Word = Word::saying(
    "sleeping.holding.the-lid",
    "Deciding what closing the lid does",
)
.noting(
    "One line in the machine's list of what is currently holding off sleep, read by somebody \
     checking why their machine behaves as it does. alo OS holds this for as long as a person is \
     signed in, so that closing the lid follows their own setting.",
);

/// alo OS locks the machine before it sleeps.
pub const HOLDING_UNTIL_LOCKED: Word = Word::saying(
    "sleeping.holding.until-locked",
    "Locking this machine before it sleeps",
)
.noting(
    "One line in the machine's list of what is currently holding off sleep. Whenever the machine \
     is about to sleep, alo OS waits the moment it takes to lock the screen first, so the machine \
     never wakes up unlocked.",
);

// ---------------------------------------------------------------------------
// What became of the agent's work when the machine woke —
// [`crate::AfterSleep`].
// ---------------------------------------------------------------------------

/// A turn's time ran out while the machine was asleep.
pub const STOPPED_WHILE_ASLEEP: Word = Word::saying(
    "sleeping.after.stopped",
    "The agent stopped working on what you asked, because its time ran out while this machine \
     was asleep. Ask again to start over.",
)
.noting(
    "Shown after the person unlocks a machine that went to sleep while the agent — the assistant \
     built into alo OS, and not a person — was working on something. Each request to the assistant has a limited time, and that time passed while \
     the machine slept, so the work did not carry on. Nothing was done without the person; they \
     can simply ask again.",
);

// ---------------------------------------------------------------------------
// The refusals — [`crate::NotHeld`] and [`crate::NotAsleep`].
// ---------------------------------------------------------------------------

/// What an application was allowed is not keeping the machine awake.
pub const NOT_ASKING_TO_STAY_AWAKE: Word = Word::saying(
    "sleeping.not-held.not-asking-to-stay-awake",
    "{application} did not ask to keep this machine awake, so it is not keeping it awake",
)
.noting(
    "{application} is the identifier of an application and is never translated. An application \
     was allowed something else — the camera, say — and that permission was offered as though it \
     were permission to keep the machine awake. It was refused.",
);

/// A turn that is over cannot hold the machine awake.
pub const THE_AGENT_IS_NOT_WORKING: Word = Word::saying(
    "sleeping.not-held.the-agent-is-not-working",
    "The agent is no longer working on what you asked, so it is not keeping this machine awake",
)
.noting(
    "The agent is the assistant built into alo OS, and not a person. Its piece of work had already ended, or had stopped, when something tried to \
     keep the machine awake for it. Only work that is still running can keep a machine awake.",
);

/// The machine would not take a hold.
pub const THE_MACHINE_WOULD_NOT_HOLD: Word = Word::saying(
    "sleeping.not-held.by-the-machine",
    "This machine would not agree to stay awake, so it may still go to sleep",
)
.noting(
    "Something asked the machine to stay awake and the operating system underneath refused. \
     Nothing is keeping the machine awake as a result. Do not name any part of the system.",
);

/// The machine did not go to sleep.
pub const DID_NOT_SLEEP: Word = Word::saying(
    "sleeping.not-asleep",
    "This machine did not go to sleep. It is locked and still running.",
)
.noting(
    "The person closed the lid or chose Sleep, the screen locked first as it always does, and \
     then the machine did not go to sleep. It is safe — locked — and still using power. Read by \
     somebody finding out why their battery went down; never shown on the lock screen itself.",
);

// ---------------------------------------------------------------------------
// The person's settings — [`crate::Lid`] and [`crate::Settings`].
// ---------------------------------------------------------------------------

/// Closing the lid puts the machine to sleep.
pub const LID_SLEEPS: Word = Word::saying(
    "sleeping.setting.lid-sleeps",
    "Sleep when the lid is closed",
)
.noting(
    "One of two choices in Settings for what closing a laptop's lid does. This one always puts \
     the machine to sleep, even with a monitor plugged in.",
);

/// Closing the lid with a display attached leaves the machine awake.
pub const LID_STAYS_AWAKE_WITH_A_DISPLAY: Word = Word::saying(
    "sleeping.setting.lid-stays-awake-with-a-display",
    "Stay awake when the lid is closed and another display is attached",
)
.noting(
    "The other of the two choices for the lid. With a monitor or projector plugged in, the laptop \
     keeps running with its lid closed; with none attached, closing the lid still puts it to \
     sleep.",
);

/// The person keeps the machine awake.
pub const KEEP_AWAKE: Word = Word::saying("sleeping.setting.keep-awake", "Keep this machine awake")
    .noting(
        "A switch in Settings. When it is on, the machine does not go to sleep on its own when \
         nobody is using it. Closing the lid or choosing Sleep still puts it to sleep.",
    );

// ---------------------------------------------------------------------------
// The person's own file, `sleeping.toml` — [`crate::FileNotRead`] and
// [`crate::FileNotWritten`]. Each names the file, and each says what the
// machine does instead.
// ---------------------------------------------------------------------------

/// The disk would not give the file up.
pub const KEPT_NOT_READ: Word = Word::saying(
    "sleeping.kept.not-read",
    "your sleep settings at {path} could not be read, so this machine sleeps as alo OS ships it",
)
.noting(
    "{path} is a file on this machine and is never translated. A disk or a permission rather than \
     anything a person typed. \"alo OS\" is the product's name and is never translated.",
);

/// The file is there and is not sleep settings.
pub const KEPT_NOT_UNDERSTOOD: Word = Word::saying(
    "sleeping.kept.not-understood",
    "your sleep settings at {path} are not settings alo OS can read, so nothing in the file has \
     been used and this machine sleeps as alo OS ships it",
)
.noting(
    "{path} is a file on this machine and is never translated. The important clause is that \
     nothing in the file was used: alo OS did not take the half it understood.",
);

/// The file stopped being settings at a line.
pub const KEPT_NOT_UNDERSTOOD_AT: Word = Word::saying(
    "sleeping.kept.not-understood-at",
    "your sleep settings at {path} stop making sense at line {line}, so nothing in the file has \
     been used and this machine sleeps as alo OS ships it",
)
.noting(
    "{path} is a file on this machine and is never translated. {line} is a plain whole number, \
     counted from one the way a text editor counts lines.",
);

/// The file says it is a shape this alo OS does not read.
pub const KEPT_ANOTHER_FORMAT: Word = Word::saying(
    "sleeping.kept.another-format",
    "your sleep settings at {path} were written for a different alo OS than this one, so nothing \
     in the file has been used and this machine sleeps as alo OS ships it",
)
.noting(
    "{path} is a file on this machine and is never translated. Most often a newer alo OS wrote \
     the file.",
);

/// The file names something that is not a sleep setting.
pub const KEPT_UNKNOWN_KEY: Word = Word::saying(
    "sleeping.kept.unknown-key",
    "your sleep settings at {path} say {key}, which is not something alo OS can change about \
     sleep, so nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and {key} is a word as it was typed into it; neither is \
     translated. The key is named because it is what a person has to find in the file to fix it.",
);

/// The disk would not take the changed file.
pub const KEPT_NOT_WRITTEN: Word = Word::saying(
    "sleeping.kept.not-written",
    "your sleep settings at {path} could not be written, so nothing about sleep has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A full disk or a folder the person \
     cannot write. The second clause is what they act on: the file is exactly as it was.",
);

/// The change could not be written as a file this alo OS reads back.
pub const KEPT_NOT_EXPRESSIBLE: Word = Word::saying(
    "sleeping.kept.not-expressible",
    "this alo OS could not write that change into sleep settings at {path} it can read back \
     again, so nothing about sleep has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A fault in alo OS rather than \
     anything the person did, said plainly: the change was refused before the file was touched.",
);

/// The file a change would replace is there and did not read, so it was kept.
pub const KEPT_NOT_REPLACED: Word = Word::saying(
    "sleeping.kept.not-replaced",
    "your sleep settings at {path} could not be read, so that change has not been written over \
     them and nothing about sleep has been changed — correct the file, or put sleep back as alo \
     OS ships it",
)
.noting(
    "{path} is a file on this machine and is never translated. Said when a person changes \
     something in Settings while the file, most often one they edited by hand, does not read: alo \
     OS keeps their file rather than replacing it, so a typing mistake in it is not lost. The last \
     clause gives the two ways on.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 22] = [
    KEPT_AWAKE_BY_YOUR_SETTING,
    KEPT_AWAKE_BY_AN_APPLICATION,
    KEPT_AWAKE_BY_THE_AGENT,
    AWAKE_WITH_THE_LID_CLOSED,
    HOLDING_THE_LID,
    HOLDING_UNTIL_LOCKED,
    STOPPED_WHILE_ASLEEP,
    NOT_ASKING_TO_STAY_AWAKE,
    THE_AGENT_IS_NOT_WORKING,
    THE_MACHINE_WOULD_NOT_HOLD,
    DID_NOT_SLEEP,
    LID_SLEEPS,
    LID_STAYS_AWAKE_WITH_A_DISPLAY,
    KEEP_AWAKE,
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
/// are what say so.
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
pub fn sleeping_words() -> Result<Vocabulary, WordsError> {
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
    use std::collections::BTreeSet;

    use super::*;

    /// What we ship is held to the rule everybody else is held to.
    #[test]
    fn every_key_is_a_key_under_this_crates_area() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "sleeping", "{}", word.named());
        }
    }

    /// The list declares once, each key once, and a second declaration
    /// replaces nothing.
    #[test]
    fn the_whole_list_declares_once() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let mut vocabulary = sleeping_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// Every word carries a note: none of these can be translated from its own
    /// words alone.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Only an application's identifier and a file's place are ever filled
    /// in.** No sentence here has room for the reason an application gave, a
    /// document, or anything the agent was working on.
    #[test]
    fn nothing_but_an_identifier_a_path_a_line_or_a_key_is_filled_in() {
        let allowed: BTreeSet<&str> = ["application", "path", "line", "key"].into();
        for word in EVERY_WORD {
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(allowed.contains(gap.as_str()), "{}: {gap}", word.named());
            }
        }
    }
}
