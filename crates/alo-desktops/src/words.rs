//! Every string this crate can say, and the English beside each one.
//!
//! Four kinds: the three surfaces that are on every desktop at once, the ways a
//! person asks for another desktop, what a desktop nobody has named is called,
//! and why nothing changed.
//!
//! The shape is `alo-shortcuts`' and is copied rather than re-decided.
//!
//! # No sentence here names a desktop, a window or a chord
//!
//! A refusal is about one of those, and none of these sentences says which. The
//! shell marks it beside the sentence instead, through [`crate::Refused`]'s own
//! accessors, for the reason `alo-dividing` gives: the only name this crate
//! could put in a sentence is one it would have to be handed, and a window's
//! title is exactly what an arrangement does not hold.
//!
//! # The two sentences a person never reads are not here
//!
//! [`crate::NotADisplay`] and [`crate::TwoPromises`] keep their English and
//! their `Display`: both mean the shell and this crate disagree about what
//! exists, and there is nothing to ask a person about it.

use alo_strings::Vocabulary;

/// One string a crate can say.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The three that are on every desktop at once — [`crate::Always`]. Names, for
// the row a settings panel or a screen reader reads.
// ---------------------------------------------------------------------------

/// [`crate::Always::EgressIndicator`].
pub const THE_EGRESS_INDICATOR: Word = Word::saying(
    "desktops.always.egress-indicator",
    "What is leaving this machine",
)
.noting(
    "The name of the part of the screen that shows whatever alo OS is sending out over the \
     network as it is sent. \"This machine\" is the computer in front of the person. It is on \
     every desktop at once and cannot be moved to one, which is what this row in a list of \
     desktops says.",
);

/// [`crate::Always::ApprovalSurface`].
pub const THE_APPROVAL_SURFACE: Word = Word::saying(
    "desktops.always.approval-surface",
    "What is waiting for your approval",
)
.noting(
    "The name of the part of the screen that shows changes the agent has proposed and is waiting \
     for the person to allow or refuse. Approval as in permission, one change at a time — see \
     the note on desktops.always.egress-indicator for where this row is read.",
);

/// [`crate::Always::AgentOverlay`].
pub const THE_AGENT_OVERLAY: Word = Word::saying("desktops.always.the-agent", "The agent").noting(
    "The name of the part of the screen a person brings the agent up in. \"The agent\" is \
         what alo OS calls the assistant a person talks to; translate it as the ordinary word \
         for something that acts on your behalf, not as a brand name.",
);

// ---------------------------------------------------------------------------
// Moving to another desktop — [`crate::Switch`]. Rows in a shortcuts panel,
// beside the combination each answers to.
// ---------------------------------------------------------------------------

/// [`crate::Switch::Next`].
pub const NEXT_DESKTOP: Word = Word::saying("desktops.switch.next", "Next desktop").noting(
    "A row in the list of keyboard shortcuts: the combination that moves to the desktop after \
     the one a person is on. A desktop here is one of several screenfuls of windows a person \
     keeps separately and moves between; use whatever your language already calls those.",
);

/// [`crate::Switch::Previous`].
pub const PREVIOUS_DESKTOP: Word = Word::saying("desktops.switch.previous", "Previous desktop")
    .noting("The desktop before the one a person is on — see the note on desktops.switch.next.");

// ---------------------------------------------------------------------------
// What a desktop nobody has named is called — [`crate::Desktop::numbered`],
// and the same sentence for a shortcut that reaches one by number.
// ---------------------------------------------------------------------------

/// A desktop the person has not named, read from where it is in their order.
pub const DESKTOP_NUMBERED: Word = Word::saying("desktops.desktop.numbered", "Desktop {number}")
    .noting(
        "What a desktop nobody has given a name to is called, and the row in the shortcuts list \
         for the combination that goes straight to it. {number} is a plain whole number counted \
         from 1. A desktop here is one of several screenfuls of windows — see the note on \
         desktops.switch.next.",
    );

// ---------------------------------------------------------------------------
// Why a name was not taken — [`crate::NameError`]. Read in a rename box, by
// somebody who has just typed something.
// ---------------------------------------------------------------------------

/// [`crate::NameError::Nothing`].
pub const NAME_IS_NOTHING: Word = Word::saying(
    "desktops.name.nothing",
    "a desktop's name needs at least one letter, so it has been left as it was",
)
.noting(
    "Said when a person confirms a rename box that is empty or holds nothing but spaces. \"At \
     least one letter\" means at least one character of writing in any script.",
);

/// [`crate::NameError::TooLong`].
pub const NAME_TOO_LONG: Word = Word::saying(
    "desktops.name.too-long",
    "a desktop's name can be {most} characters at most, so it has been left as it was",
)
.noting(
    "{most} is a plain whole number. Characters as a person counts them, so an accented letter \
     counts once however many bytes it takes.",
);

/// [`crate::NameError::NotWords`].
pub const NAME_IS_NOT_WORDS: Word = Word::saying(
    "desktops.name.not-words",
    "that name has something in it that is not writing, so it has been left as it was — it would \
     not read as what you typed",
)
.noting(
    "Said when a name holds control characters — a line break pasted in with the text, for \
     instance — which are not letters in any script and would draw as something other than what \
     was typed. Any script is a name; this is not about which alphabet.",
);

// ---------------------------------------------------------------------------
// Why the desktops were left as they were — [`crate::Refused`]. Each says what
// did **not** happen, because a person reading it has just tried something.
// ---------------------------------------------------------------------------

/// [`crate::Refused::NoSuchDesktop`].
pub const NO_SUCH_DESKTOP: Word = Word::saying(
    "desktops.refused.no-such-desktop",
    "that desktop is no longer there, so nothing has changed",
)
.noting(
    "Said when a person acts on a desktop that was removed while they were looking at it — from \
     a list that had not caught up, for instance.",
);

/// [`crate::Refused::NoSuchPosition`].
pub const NO_SUCH_POSITION: Word = Word::saying(
    "desktops.refused.no-such-position",
    "there is no desktop at that number on this screen, so nothing has changed",
)
.noting(
    "Said when a person presses the combination for, say, the fourth desktop while they have \
     three. \"This screen\" is the display they are looking at: desktops belong to one screen, \
     and another screen has its own.",
);

/// [`crate::Refused::TheLastDesktop`].
pub const THE_LAST_DESKTOP: Word = Word::saying(
    "desktops.refused.the-last-desktop",
    "this is the only desktop on this screen, and a screen always has one — add another before \
     removing this",
)
.noting(
    "Said when a person tries to remove their only desktop. Removing it would leave the screen \
     with nowhere to put a window.",
);

/// [`crate::Refused::TooManyDesktops`].
pub const TOO_MANY_DESKTOPS: Word = Word::saying(
    "desktops.refused.too-many-desktops",
    "there is no room for another desktop on this screen — alo OS holds {most}",
)
.noting(
    "{most} is a plain whole number. \"alo OS\" is the name of the system and is never \
     translated.",
);

/// [`crate::Refused::NoDesktopThatWay`].
pub const NO_DESKTOP_THAT_WAY: Word = Word::saying(
    "desktops.refused.no-desktop-that-way",
    "there is no desktop that way — this is the end of the row",
)
.noting(
    "Said when a person swipes or presses past the first or the last desktop. alo OS does not \
     come back round to the other end; \"the row\" is the desktops in the order the person put \
     them in.",
);

/// [`crate::Refused::NameIsTaken`].
pub const NAME_IS_TAKEN: Word = Word::saying(
    "desktops.refused.name-is-taken",
    "another desktop on this screen is already called that, so the name has been left as it was",
)
.noting(
    "Said when a person renames a desktop to a name one of their other desktops on the same \
     screen already has. Two desktops with one name would be a list nobody could choose from.",
);

/// [`crate::Refused::APromise`].
pub const A_PROMISE: Word = Word::saying(
    "desktops.refused.a-promise",
    "this is part of alo OS itself and is on every desktop, so it cannot be put on one or closed",
)
.noting(
    "Said when something tries to move or close one of the three things alo OS keeps on every \
     desktop: what is leaving the machine, what is waiting for approval, and the agent. They are \
     promises the system makes, and one that could only be seen on one desktop would be no \
     promise at all. \"alo OS\" is never translated.",
);

/// [`crate::Refused::NoSuchWindow`].
pub const NO_SUCH_WINDOW: Word = Word::saying(
    "desktops.refused.no-such-window",
    "that window is not on any desktop of this screen, so nothing has changed",
)
.noting(
    "Said when something acts on a window that has already closed, or that is on another screen. \
     See the note on desktops.refused.no-such-position for \"this screen\".",
);

/// [`crate::Refused::ChordIsTaken`].
pub const CHORD_IS_TAKEN: Word = Word::saying(
    "desktops.refused.chord-is-taken",
    "one of your keyboard shortcuts already uses that combination, so it has been left as it was \
     — change that shortcut first, or choose another combination",
)
.noting(
    "Said when a person sets a combination for switching desktops that one of the system's own \
     keyboard shortcuts already uses. The shortcut takes the keys first, so the desktop would \
     never get them. Which shortcut has it is marked beside this sentence.",
);

/// [`crate::Refused::ChordIsASwitch`].
pub const CHORD_IS_A_SWITCH: Word = Word::saying(
    "desktops.refused.chord-is-a-switch",
    "another desktop already answers to that combination, so it has been left as it was — choose \
     another",
)
.noting(
    "Said when a person sets a combination that one of their other desktop combinations already \
     uses. Which one is marked beside this sentence.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 19] = [
    THE_EGRESS_INDICATOR,
    THE_APPROVAL_SURFACE,
    THE_AGENT_OVERLAY,
    NEXT_DESKTOP,
    PREVIOUS_DESKTOP,
    DESKTOP_NUMBERED,
    NAME_IS_NOTHING,
    NAME_TOO_LONG,
    NAME_IS_NOT_WORDS,
    NO_SUCH_DESKTOP,
    NO_SUCH_POSITION,
    THE_LAST_DESKTOP,
    TOO_MANY_DESKTOPS,
    NO_DESKTOP_THAT_WAY,
    NAME_IS_TAKEN,
    A_PROMISE,
    NO_SUCH_WINDOW,
    CHORD_IS_TAKEN,
    CHORD_IS_A_SWITCH,
];

/// Why this crate's own words could not be declared.
///
/// Nothing in the list above can cause one; the tests at the bottom of this file
/// are what say so.
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
pub fn desktop_words() -> Result<Vocabulary, WordsError> {
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

    /// Every key written here is a key by the rule every other key is held to.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "desktops", "{}", word.named());
        }
    }

    /// No two words share a key, and the whole list declares.
    #[test]
    fn the_whole_list_declares_and_no_two_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let vocabulary = desktop_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        let again = declare_into(&mut desktop_words().unwrap()).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// Every word carries a note, and only the two that are about a number have
    /// a gap in them.
    #[test]
    fn every_word_carries_a_note_and_only_two_have_a_gap() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| !note.trim().is_empty()),
                "{}",
                word.named()
            );
            let phrase = word.phrase().unwrap();
            let gaps = phrase.source().gaps();
            let expected: &[&str] = match word.named() {
                "desktops.desktop.numbered" => &["number"],
                "desktops.name.too-long" | "desktops.refused.too-many-desktops" => &["most"],
                _ => &[],
            };
            assert_eq!(gaps.len(), expected.len(), "{}", word.named());
            for gap in expected {
                assert!(phrase.source().has(gap), "{} wants {gap}", word.named());
            }
        }
    }
}
