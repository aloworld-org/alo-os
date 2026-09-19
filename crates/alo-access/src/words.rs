//! Every sentence this crate can say, and the notes a translator writes from.
//!
//! These are the sentences a person reads **while looking for the setting they
//! need**, sometimes without being able to see the screen, so they say what the
//! setting does rather than name a feature: *read the screen aloud*, not
//! *screen reader*, because somebody who has never used one is looking for the
//! first and not the second.

use alo_strings::Vocabulary;

pub use alo_strings::Word;

/// Read the screen aloud.
pub const SCREEN_READER: Word = Word::saying("access.screen-reader", "read the screen aloud")
    .noting(
        "A setting a person turns on. It starts the screen reader this machine ships with, which \
     reads what is on the screen and what the keyboard is on. Say it as what it does, because \
     somebody who has not used one before is looking for that rather than for its name.",
    );

/// Magnify what is under the pointer.
pub const MAGNIFIER: Word = Word::saying(
    "access.magnifier",
    "magnify what is under the pointer",
)
.noting(
    "A setting a person turns on. How much larger is a number shown beside this line rather than \
     inside it.",
);

/// Draw in high contrast.
pub const HIGH_CONTRAST: Word = Word::saying(
    "access.high-contrast",
    "use strong colours that are easier to read",
)
.noting(
    "A setting a person turns on. It replaces the machine's quiet colours with a palette of black \
     and white and one strong accent. Do not translate it as \"high contrast mode\" — nothing is \
     switched off, only the colours change.",
);

/// Draw text larger.
pub const LARGER_TEXT: Word = Word::saying("access.larger-text", "make all text larger").noting(
    "A setting a person turns on, which makes text half again as large everywhere. A person who \
     wants to choose a size themselves does that in the appearance settings; this is one switch \
     for somebody who cannot read the ordinary size.",
);

/// Move nothing that need not move.
pub const REDUCED_MOTION: Word =
    Word::saying("access.reduced-motion", "stop things sliding and fading").noting(
        "A setting a person turns on, for somebody who is made unwell by movement on a screen. \
     Nothing is removed — what would have slid appears instead.",
    );

/// A modifier held by pressing it.
pub const STICKY_KEYS: Word = Word::saying(
    "access.sticky-keys",
    "press keys one at a time instead of holding them together",
)
.noting(
    "A setting a person turns on, for somebody who cannot hold two keys at once. After it, a \
     modifier such as shift stays held until the next key is pressed.",
);

/// A key counts once it has been held.
pub const SLOW_KEYS: Word = Word::saying(
    "access.slow-keys",
    "ignore a key unless it is held for a moment",
)
.noting(
    "A setting a person turns on, so that a key brushed by accident does nothing. How long is a \
     number shown beside this line.",
);

/// The same key twice in a moment is one press.
pub const BOUNCE_KEYS: Word = Word::saying(
    "access.bounce-keys",
    "ignore the same key pressed twice quickly",
)
.noting(
    "A setting a person turns on, for a hand that repeats a key without meaning to. How long is a \
     number shown beside this line.",
);

/// Focus, always drawn.
pub const FOCUS_ALWAYS_VISIBLE: Word = Word::saying(
    "access.focus-always-visible",
    "always show where the keyboard is",
)
.noting(
    "A setting a person turns on. \"Where the keyboard is\" is the outline around whatever a key \
     would act on. Ordinarily it appears once somebody uses the keyboard; this draws it always.",
);

/// The screen where somebody signs in.
pub const SIGN_IN: Word = Word::saying("access.sign-in", "signing in")
    .noting("The screen where somebody signs in. Read aloud as the name of the whole screen.");

/// A list of the people who have an account on this machine.
pub const WHO_IS_SIGNING_IN: Word = Word::saying("access.who-is-signing-in", "who is signing in")
    .noting("A list of the people who have an account on this machine.");

/// Where a password is typed.
pub const THE_PASSWORD: Word = Word::saying(
    "access.the-password",
    "password",
)
.noting(
    "Where a password is typed. Never read back, and a reader says only that it is a password field.",
);

/// The button that signs the chosen person in.
pub const SIGN_IN_NOW: Word = Word::saying("access.sign-in-now", "sign in")
    .noting("The button that signs the chosen person in.");

/// The button at the sign-in screen that opens the settings in this crate, before anybody has an account.
pub const ACCESS_SETTINGS: Word = Word::saying(
    "access.settings-here",
    "settings for seeing, hearing and typing",
)
.noting(
    "The button at the sign-in screen that opens the settings in this crate, before anybody has an account. Somebody who needs one of those settings to use the machine must be able to reach it from here.",
);

/// The whole working area, read as the name of a window.
pub const THE_DESKTOP: Word = Word::saying("access.the-desktop", "the desktop")
    .noting("The whole working area, read as the name of a window.");

/// A list of what is open, which a person moves through with the keyboard.
pub const THE_WINDOWS_OPEN: Word =
    Word::saying("access.the-windows-open", "the windows that are open")
        .noting("A list of what is open, which a person moves through with the keyboard.");

/// The agent, reached without knowing a chord.
pub const ASK_THE_AGENT: Word = Word::saying("access.ask-the-agent", "ask the agent").noting(
    "A control on the desktop that opens the agent. It is here because a chord is not a road \
         for somebody who has never been told the chord: the agent answers to a key, and it must \
         also be somewhere the keyboard arrives at by pressing Tab.",
);

/// The launcher, reached the same way.
pub const THE_LAUNCHER: Word = Word::saying("access.the-launcher", "open something").noting(
    "A control in the dock that opens the launcher. Named for what a person wants rather than \
         for the thing — somebody looking for a program is looking to open something.",
);

/// The strip of applications a person starts things from.
pub const THE_DOCK: Word = Word::saying("access.the-dock", "the dock")
    .noting("The strip of applications a person starts things from.");

/// One entry in the dock.
pub const AN_APPLICATION: Word = Word::saying("access.an-application", "an application")
    .noting("One entry in the dock. The application's own name is read after this.");

/// The strip that shows what is running, what the agent is doing and what is leaving the machine.
pub const THE_STATUS_AREA: Word = Word::saying(
    "access.the-status-area",
    "what this machine is doing",
)
.noting(
    "The strip that shows what is running, what the agent is doing and what is leaving the machine.",
);

/// Announced the moment it changes, not found by looking.
pub const SOMETHING_IS_LEAVING: Word = Word::saying(
    "access.something-is-leaving",
    "something is leaving this machine",
)
.noting(
    "Announced the moment it changes, not found by looking. This is the indicator that makes the first law visible, so a person who cannot see the screen is told in words.",
);

/// Announced when it changes.
pub const THE_AGENT_IS_WORKING: Word =
    Word::saying("access.the-agent-is-working", "the agent is working")
        .noting("Announced when it changes. The agent is the assistant that acts on this machine.");

/// The name of the surface where a change waits for one approval.
pub const SOMETHING_IS_ASKED: Word = Word::saying(
    "access.something-is-asked",
    "the machine is asking you something",
)
.noting(
    "The name of the surface where a change waits for one approval. Read before the sentence itself.",
);

/// The sentence describing the change, written by the turn that asked.
pub const WHAT_THE_TURN_WROTE: Word = Word::saying(
    "access.what-the-turn-wrote",
    "what will happen if you approve",
)
.noting(
    "The sentence describing the change, written by the turn that asked. It is what the person approves, so it is read whole and never summarised.",
);

/// The answer that changes nothing.
pub const SAY_NO: Word = Word::saying("access.say-no", "no")
    .noting("The answer that changes nothing. Read first, and nothing is chosen for the person.");

/// The answer that carries the change out, once.
pub const APPROVE_IT: Word = Word::saying("access.approve-it", "approve")
    .noting("The answer that carries the change out, once.");

/// The window listing what has happened on this machine.
pub const THE_RECORD: Word = Word::saying("access.the-record", "the record")
    .noting("The window listing what has happened on this machine.");

/// The list itself.
pub const WHAT_HAPPENED: Word =
    Word::saying("access.what-happened", "what has happened").noting("The list itself.");

/// One entry in the record.
pub const ONE_THING_THAT_HAPPENED: Word =
    Word::saying("access.one-thing-that-happened", "one thing that happened")
        .noting("One entry in the record. What it says is read after this.");

/// The settings window.
pub const SETTINGS: Word =
    Word::saying("access.settings", "settings").noting("The settings window.");

/// The list of settings.
pub const WHAT_CAN_BE_CHANGED: Word =
    Word::saying("access.what-can-be-changed", "what can be changed")
        .noting("The list of settings.");

/// One setting, which is on or off.
pub const A_SETTING: Word = Word::saying("access.a-setting", "a setting")
    .noting("One setting, which is on or off. Which it is now is read after the name.");

/// The button that closes a window.
pub const CLOSE_THIS_WINDOW: Word = Word::saying("access.close-this-window", "close this window")
    .noting("The button that closes a window.");

/// The button that moves a window to one side of the screen.
pub const ARRANGE_THIS_WINDOW: Word =
    Word::saying("access.arrange-this-window", "move this window")
        .noting("The button that moves a window to one side of the screen.");

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 35] = [
    SCREEN_READER,
    MAGNIFIER,
    HIGH_CONTRAST,
    LARGER_TEXT,
    REDUCED_MOTION,
    STICKY_KEYS,
    SLOW_KEYS,
    BOUNCE_KEYS,
    FOCUS_ALWAYS_VISIBLE,
    SIGN_IN,
    WHO_IS_SIGNING_IN,
    THE_PASSWORD,
    SIGN_IN_NOW,
    ACCESS_SETTINGS,
    THE_DESKTOP,
    THE_WINDOWS_OPEN,
    ASK_THE_AGENT,
    THE_LAUNCHER,
    THE_DOCK,
    AN_APPLICATION,
    THE_STATUS_AREA,
    SOMETHING_IS_LEAVING,
    THE_AGENT_IS_WORKING,
    SOMETHING_IS_ASKED,
    WHAT_THE_TURN_WROTE,
    SAY_NO,
    APPROVE_IT,
    THE_RECORD,
    WHAT_HAPPENED,
    ONE_THING_THAT_HAPPENED,
    SETTINGS,
    WHAT_CAN_BE_CHANGED,
    A_SETTING,
    CLOSE_THIS_WINDOW,
    ARRANGE_THIS_WINDOW,
];

/// Why this crate's own list could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
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
pub fn access_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put every sentence this crate says into a vocabulary somebody else holds.
///
/// # Errors
/// [`WordsError`] where the vocabulary already has one of these keys.
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
    use crate::setting::Setting;

    /// **Every setting has a sentence, and no two share one.**
    #[test]
    fn every_setting_says_what_it_does_in_words_of_its_own() {
        let vocabulary = access_words().unwrap();
        assert_eq!(
            EVERY_WORD.len(),
            Setting::ALL.len() + crate::tree::EVERY_NAME_A_READER_SAYS,
            "a word was added to this crate and to neither list it belongs to"
        );
        for setting in Setting::ALL {
            let word = setting.word();
            assert!(
                EVERY_WORD.iter().any(|every| every.key() == word.key()),
                "{setting:?} says something this crate does not declare"
            );
            assert!(vocabulary.phrase(&word.key()).is_some(), "{setting:?}");
        }
        let mut keys: Vec<String> = EVERY_WORD
            .iter()
            .map(|word| word.key().to_string())
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(
            keys.len(),
            EVERY_WORD.len(),
            "two settings share a sentence"
        );
    }

    /// **No sentence names a disability**, and none says *mode*: a person is
    /// reading a list of what the machine can do, not a diagnosis.
    #[test]
    fn no_sentence_names_a_person_or_calls_itself_a_mode() {
        for word in EVERY_WORD {
            let said = format!("{} {}", word.says(), word.note().unwrap_or_default());
            for never in [
                "disabled", "impair", "handicap", "blind", "deaf", "sufferer",
            ] {
                assert!(
                    !said.to_lowercase().contains(never),
                    "{}: {never}",
                    word.key()
                );
            }
            assert!(
                !word.says().to_lowercase().contains("mode"),
                "{}",
                word.key()
            );
        }
    }
}
