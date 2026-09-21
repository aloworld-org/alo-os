//! What the disk refused, read off what the rented tool answered.
//!
//! A person standing at an install or at a machine that will not open needs to
//! be told one of a very small number of things, and *the program exited 2* is
//! none of them. This turns what the rented tools answer into the refusals alo
//! OS has sentences for — `alo-enrolling` holds the sentences, because this
//! crate depends on nothing and so cannot say a word in anybody's language.
//!
//! # The numbers are `cryptsetup`'s own, and they were measured
//!
//! `cryptsetup` answers 1 for arguments it did not understand, **2 for a secret
//! that opens nothing**, 3 when it ran out of memory, 4 for a device that is not
//! what it was told it was, and 5 for a volume that is already open or busy. The
//! one that matters is 2, and it was measured on a real LUKS2 volume in the
//! pinned base three times over: the installer's wiped first key, a key that was
//! never this disk's, and the old secret after it was replaced, each *No key
//! available with this passphrase* and each exit 2.
//!
//! `systemd-cryptenroll` answers 1 for anything that went wrong, which is why
//! [`TheDiskRefused::TheRentedToolWouldNotDoIt`] exists and why it does not
//! pretend to know more than it does.

use std::fmt;

/// What `cryptsetup` answers when the secret it was given opens nothing.
const NO_KEY_OPENS_IT: i32 = 2;

/// What it answers for a device that is not the kind of device it was told.
const NOT_THAT_KIND_OF_DEVICE: i32 = 4;

/// What it answers for a volume that is already open, or busy.
const ALREADY_OPEN_OR_BUSY: i32 = 5;

/// What the disk refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheDiskRefused {
    /// The secret that was given opens nothing on this volume.
    ///
    /// The one a person meets: a passphrase typed wrong, a recovery key copied
    /// wrong, or a secret that belonged to a machine this disk is no longer in.
    WhatWasGivenDoesNotOpenIt,
    /// What was named is not an encrypted volume.
    ThisIsNotAnEncryptedVolume,
    /// It is already open, or something else is using it.
    ItIsAlreadyOpen,
    /// The rented tool would not do it, and did not say which of its own
    /// reasons it was.
    TheRentedToolWouldNotDoIt,
    /// The rented tool is not on this machine, or could not be started.
    ///
    /// Not a refusal about the disk at all: it is alo OS's own image being
    /// wrong, and a person is told so rather than being asked to try their
    /// secret again.
    TheRentedToolIsNotOnThisMachine,
}

impl TheDiskRefused {
    /// Every one of them, so that whoever writes the sentences has a list
    /// rather than a memory.
    pub const ALL: [Self; 5] = [
        Self::WhatWasGivenDoesNotOpenIt,
        Self::ThisIsNotAnEncryptedVolume,
        Self::ItIsAlreadyOpen,
        Self::TheRentedToolWouldNotDoIt,
        Self::TheRentedToolIsNotOnThisMachine,
    ];

    /// What the disk refused, from how the rented tool ended.
    ///
    /// `None` is a tool that was stopped by a signal rather than by finishing,
    /// which says nothing about the disk and is read as the tool refusing.
    #[must_use]
    pub const fn from_how_it_ended(ended: Option<i32>) -> Option<Self> {
        match ended {
            Some(0) => None,
            Some(NO_KEY_OPENS_IT) => Some(Self::WhatWasGivenDoesNotOpenIt),
            Some(NOT_THAT_KIND_OF_DEVICE) => Some(Self::ThisIsNotAnEncryptedVolume),
            Some(ALREADY_OPEN_OR_BUSY) => Some(Self::ItIsAlreadyOpen),
            _ => Some(Self::TheRentedToolWouldNotDoIt),
        }
    }

    /// The tool could not be started at all.
    #[must_use]
    pub const fn because_the_tool_would_not_start() -> Self {
        Self::TheRentedToolIsNotOnThisMachine
    }
}

impl fmt::Display for TheDiskRefused {
    /// English for a service log, the way `alo-drives`' is. What a person reads
    /// is `alo-enrolling`'s, in their own language.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::WhatWasGivenDoesNotOpenIt => "what was given does not open this volume",
            Self::ThisIsNotAnEncryptedVolume => "what was named is not an encrypted volume",
            Self::ItIsAlreadyOpen => "the volume is already open, or something else is using it",
            Self::TheRentedToolWouldNotDoIt => "the rented tool refused and did not say why",
            Self::TheRentedToolIsNotOnThisMachine => {
                "the rented tool is not on this machine, which is this image being wrong"
            }
        })
    }
}

impl std::error::Error for TheDiskRefused {}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A tool that finished is not a refusal**, and every other answer is.
    #[test]
    fn nothing_is_refused_when_the_tool_finished() {
        assert_eq!(TheDiskRefused::from_how_it_ended(Some(0)), None);
    }

    /// **The measured answer means the measured thing**: exit 2 is a secret
    /// that opens nothing, which is what a wiped key, a stranger's key and a
    /// replaced secret each answered on a real volume.
    #[test]
    fn the_answer_that_was_measured_is_the_one_a_person_meets() {
        assert_eq!(
            TheDiskRefused::from_how_it_ended(Some(2)),
            Some(TheDiskRefused::WhatWasGivenDoesNotOpenIt)
        );
    }

    /// The rest of `cryptsetup`'s own numbers, and everything else read as the
    /// tool refusing rather than as something about the person's secret.
    #[test]
    fn every_other_answer_says_what_it_can_and_no_more() {
        assert_eq!(
            TheDiskRefused::from_how_it_ended(Some(4)),
            Some(TheDiskRefused::ThisIsNotAnEncryptedVolume)
        );
        assert_eq!(
            TheDiskRefused::from_how_it_ended(Some(5)),
            Some(TheDiskRefused::ItIsAlreadyOpen)
        );
        for ended in [Some(1), Some(3), Some(127), Some(-1), None] {
            assert_eq!(
                TheDiskRefused::from_how_it_ended(ended),
                Some(TheDiskRefused::TheRentedToolWouldNotDoIt),
                "{ended:?}"
            );
        }
    }

    /// **A missing tool is alo OS's bug and not the person's**, and it is kept
    /// apart from every refusal about a secret for that reason.
    #[test]
    fn a_tool_that_is_not_there_is_its_own_refusal() {
        assert_eq!(
            TheDiskRefused::because_the_tool_would_not_start(),
            TheDiskRefused::TheRentedToolIsNotOnThisMachine
        );
        assert!(
            !TheDiskRefused::TheRentedToolIsNotOnThisMachine
                .to_string()
                .contains("secret")
        );
    }

    /// Each says something, and none of them says a secret or a slot number.
    #[test]
    fn each_refusal_says_what_is_wrong_and_names_nothing_else() {
        assert_eq!(TheDiskRefused::ALL.len(), 5);
        for refusal in TheDiskRefused::ALL {
            let said = refusal.to_string();
            assert!(!said.is_empty(), "{refusal:?}");
            assert!(!said.contains("slot"), "{said}");
        }
    }
}
