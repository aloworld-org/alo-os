//! The key that recovers the disk when nothing else opens it.
//!
//! The rented tool makes it — `systemd-cryptenroll --recovery-key`, which on a
//! virtual disk in the pinned base wrote **72 bytes to stdout**: eight groups of
//! eight characters separated by `-`, every character from a sixteen-letter
//! alphabet, and a newline
//! ([ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md),
//! measurement 4). This crate does not run it and does not know how to; what it
//! does is read those bytes, refuse anything that is not them, and hold the key
//! for exactly as long as it takes a person to write it down and type it back.
//!
//! # Why that alphabet matters
//!
//! Sixteen letters, and not a hexadecimal sixteen: `cbdefghijklnrtuv` has no `0`
//! to be mistaken for `O`, no `1` for `l`, and every letter of it is in the same
//! place on QWERTY, AZERTY and QWERTZ. For an operating system that promises 24
//! languages, that is the difference between a key somebody copies onto paper
//! and a key somebody photographs — and a photograph of a recovery key is a
//! recovery key stored on a device.
//!
//! # What this type will not do
//!
//! It does not implement [`Clone`], [`std::fmt::Display`] or any serialisation,
//! and its [`std::fmt::Debug`] says nothing. [`RecoveryKey::written_back`]
//! **consumes** it, so after the person has confirmed they kept it there is no
//! value left anywhere to store, log or send. That is the whole of *the key is
//! shown once and is never stored on the disk it recovers*, held by the type
//! system rather than by a rule somebody has to follow.

use std::fmt;

use crate::written_down::{Again, NotWhatWasShown, WrittenDown};

/// The sixteen characters a recovery key is written in.
pub const THE_ALPHABET: [char; 16] = [
    'c', 'b', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'n', 'r', 't', 'u', 'v',
];

/// How many groups a recovery key is written in.
pub const GROUPS: usize = 8;

/// How many characters are in each group.
///
/// Eight of them, eight times, from sixteen characters: 256 bits, which is the
/// key the volume is opened with rather than something a key is derived from.
pub const IN_A_GROUP: usize = 8;

/// The key the rented tool made, for the length of one screen.
pub struct RecoveryKey {
    /// Exactly the characters the tool printed, in its groups, so that what a
    /// person is shown is what was enrolled and not a re-rendering of it.
    shown: String,
    /// The same characters with the dashes taken out, which is what a typing is
    /// compared against.
    plain: String,
}

/// What the rented tool printed was not a recovery key.
///
/// No variant carries any part of what was read. A refusal here means the tool
/// changed or something else answered, and the bytes are still a secret either
/// way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotARecoveryKey {
    /// Nothing was printed.
    Nothing,
    /// Not eight groups.
    NotEightGroups {
        /// How many groups there were.
        found: usize,
    },
    /// A group was not eight characters long.
    AGroupIsNotEight {
        /// How long the first wrong group was.
        found: usize,
    },
    /// A character was not one of the sixteen.
    NotTheAlphabet,
}

impl fmt::Display for NotARecoveryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nothing => f.write_str("nothing was printed where a recovery key was expected"),
            Self::NotEightGroups { found } => write!(
                f,
                "a recovery key is {GROUPS} groups and {found} were printed"
            ),
            Self::AGroupIsNotEight { found } => write!(
                f,
                "a recovery key's groups are {IN_A_GROUP} characters and one was {found}"
            ),
            Self::NotTheAlphabet => {
                f.write_str("a recovery key's characters are not the ones that were printed")
            }
        }
    }
}

impl std::error::Error for NotARecoveryKey {}

impl RecoveryKey {
    /// The key, from what the rented tool printed.
    ///
    /// Surrounding whitespace is taken off, because the tool ends its line and
    /// a line ending is not part of a key. Nothing else is forgiven: this is
    /// alo OS reading a program's output, not a person typing, and a program
    /// whose output has changed shape is a program we should stop rather than
    /// guess about.
    ///
    /// # Errors
    /// [`NotARecoveryKey`], which names the shape that was wrong and no part of
    /// what was read.
    pub fn as_printed(printed: &str) -> Result<Self, NotARecoveryKey> {
        let shown = printed.trim();
        if shown.is_empty() {
            return Err(NotARecoveryKey::Nothing);
        }
        let groups = shown.split('-').count();
        if groups != GROUPS {
            return Err(NotARecoveryKey::NotEightGroups { found: groups });
        }
        for group in shown.split('-') {
            let long = group.chars().count();
            if long != IN_A_GROUP {
                return Err(NotARecoveryKey::AGroupIsNotEight { found: long });
            }
            if !group.chars().all(|letter| THE_ALPHABET.contains(&letter)) {
                return Err(NotARecoveryKey::NotTheAlphabet);
            }
        }
        Ok(Self {
            shown: shown.to_owned(),
            plain: shown.replace('-', ""),
        })
    }

    /// The key as it goes on the screen the person copies it from.
    ///
    /// In its groups, because eight groups of eight is what a person can copy
    /// without losing their place, and because it is what was enrolled.
    #[must_use]
    pub fn as_it_is_shown(&self) -> &str {
        &self.shown
    }

    /// The person typed it back, and this is what came of that.
    ///
    /// **Consumes the key**, whether they got it right or not: on a refusal it
    /// comes back inside [`Again`], so the only way to show it a second time is
    /// to say out loud that they did not get it the first time. There is no road
    /// on which a `RecoveryKey` outlives the screen it was shown on.
    ///
    /// Dashes, spacing and case are forgiven; the characters are not. A person
    /// copying 64 characters onto paper and back will put the dashes somewhere
    /// else, and refusing that teaches them to photograph the screen instead —
    /// which is the one outcome this whole file exists to prevent.
    ///
    /// The comparison is an ordinary one. There is no remote attacker here and
    /// nothing to time: the key is on the screen beside the person typing it,
    /// once, during an install.
    ///
    /// # Errors
    /// [`Again`], holding the key and why it did not match.
    pub fn written_back(self, typed: &str) -> Result<WrittenDown, Again> {
        let typed: String = typed
            .chars()
            .filter(|letter| *letter != '-' && !letter.is_whitespace())
            .flat_map(char::to_lowercase)
            .collect();
        if typed.is_empty() {
            return Err(Again::of(self, NotWhatWasShown::Nothing));
        }
        if typed == self.plain {
            return Ok(WrittenDown::because_they_typed_it_back());
        }
        Err(Again::of(self, NotWhatWasShown::NotThisKey))
    }
}

impl fmt::Debug for RecoveryKey {
    /// Says that there is a recovery key and never what it is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RecoveryKey(not put in this line)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the rented tool printed on a virtual disk in the pinned base on
    /// 2026-09-17, with the trailing newline it ended the line with.
    const AS_THE_TOOL_PRINTED_IT: &str =
        "lrhdhrji-vtdlgdkr-ulukcjff-rgckhvrd-kguhibgc-gvihrfgc-ghedrdlr-uihctkkt\n";

    /// The same key, minus its line ending.
    fn the_key() -> String {
        AS_THE_TOOL_PRINTED_IT.trim().to_owned()
    }

    /// **What the tool really printed is read, and shown back in its groups.**
    #[test]
    fn what_the_tool_printed_is_a_recovery_key() {
        let key = RecoveryKey::as_printed(AS_THE_TOOL_PRINTED_IT);
        assert!(matches!(&key, Ok(key) if key.as_it_is_shown() == the_key()));
    }

    /// Sixty-four characters in eight groups of eight, from sixteen letters,
    /// is 256 bits. The numbers are written down so that changing one is a
    /// decision somebody made rather than a typo.
    #[test]
    fn the_key_is_two_hundred_and_fifty_six_bits() {
        assert_eq!(GROUPS * IN_A_GROUP * 4, 256);
        assert_eq!(THE_ALPHABET.len(), 16);
        let mut sorted: Vec<char> = THE_ALPHABET.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 16, "a letter is in the alphabet twice");
    }

    /// Nothing, the wrong number of groups, a group of the wrong length, and a
    /// character that is not one of the sixteen: each refused.
    #[test]
    fn anything_that_is_not_the_shape_the_tool_prints_is_refused() {
        assert_eq!(
            RecoveryKey::as_printed("  \n").err(),
            Some(NotARecoveryKey::Nothing)
        );
        assert_eq!(
            RecoveryKey::as_printed("lrhdhrji-vtdlgdkr").err(),
            Some(NotARecoveryKey::NotEightGroups { found: 2 })
        );
        assert_eq!(
            RecoveryKey::as_printed(
                "lrhdhrj-vtdlgdkr-ulukcjff-rgckhvrd-kguhibgc-gvihrfgc-ghedrdlr-uihctkkt"
            )
            .err(),
            Some(NotARecoveryKey::AGroupIsNotEight { found: 7 })
        );
        assert_eq!(
            RecoveryKey::as_printed(
                "lrhdhrjz-vtdlgdkr-ulukcjff-rgckhvrd-kguhibgc-gvihrfgc-ghedrdlr-uihctkkt"
            )
            .err(),
            Some(NotARecoveryKey::NotTheAlphabet)
        );
        assert_eq!(
            RecoveryKey::as_printed(
                "LRHDHRJI-vtdlgdkr-ulukcjff-rgckhvrd-kguhibgc-gvihrfgc-ghedrdlr-uihctkkt"
            )
            .err(),
            Some(NotARecoveryKey::NotTheAlphabet),
            "the tool prints lower case, and a program that stopped doing so is a program to stop"
        );
    }

    /// **A key typed back exactly, or with the dashes and the case somewhere
    /// else, is the same key.**
    #[test]
    fn the_dashes_and_the_case_are_forgiven() {
        for typed in [
            the_key(),
            the_key().replace('-', ""),
            the_key().replace('-', " "),
            the_key().to_uppercase(),
            format!("  {}  ", the_key()),
            the_key().replace('-', "\n"),
        ] {
            let Ok(key) = RecoveryKey::as_printed(AS_THE_TOOL_PRINTED_IT) else {
                unreachable!("the tool's own output is a recovery key")
            };
            assert!(key.written_back(&typed).is_ok(), "{typed}");
        }
    }

    /// **A key one character wrong is not the key**, and nothing else is
    /// either.
    #[test]
    fn a_key_that_is_not_the_one_shown_is_refused() {
        for (typed, why) in [
            ("", NotWhatWasShown::Nothing),
            ("   - -  ", NotWhatWasShown::Nothing),
            (
                "brhdhrji-vtdlgdkr-ulukcjff-rgckhvrd-kguhibgc-gvihrfgc-ghedrdlr-uihctkkt",
                NotWhatWasShown::NotThisKey,
            ),
            (
                "lrhdhrji-vtdlgdkr-ulukcjff-rgckhvrd-kguhibgc-gvihrfgc-ghedrdlr",
                NotWhatWasShown::NotThisKey,
            ),
            ("no", NotWhatWasShown::NotThisKey),
        ] {
            let Ok(key) = RecoveryKey::as_printed(AS_THE_TOOL_PRINTED_IT) else {
                unreachable!("the tool's own output is a recovery key")
            };
            match key.written_back(typed) {
                Ok(_) => unreachable!("{typed} was accepted as the key"),
                Err(again) => {
                    assert_eq!(again.why(), why, "{typed}");
                    assert_eq!(again.shown_again().as_it_is_shown(), the_key());
                }
            }
        }
    }

    /// **A recovery key is not in the line that mentions it.**
    #[test]
    fn a_recovery_key_is_never_in_its_own_debug() {
        let said = RecoveryKey::as_printed(AS_THE_TOOL_PRINTED_IT)
            .map(|key| format!("{key:?}"))
            .unwrap_or_default();
        assert!(!said.is_empty(), "the tool's own output is a recovery key");
        assert!(!said.contains("lrhdhrji"), "{said}");
    }

    /// The refusals name the shape that was wrong and no part of what was read.
    #[test]
    fn a_refusal_names_the_shape_and_never_the_bytes() {
        for refusal in [
            NotARecoveryKey::Nothing,
            NotARecoveryKey::NotEightGroups { found: 2 },
            NotARecoveryKey::AGroupIsNotEight { found: 7 },
            NotARecoveryKey::NotTheAlphabet,
        ] {
            let said = refusal.to_string();
            assert!(!said.contains("lrhdhrji"), "{said}");
            assert!(!said.is_empty());
        }
    }
}
