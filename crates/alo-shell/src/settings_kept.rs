//! One section of Settings whose file is in the person's folder: drawn from
//! that file at sign-in, changed through its crate's keeper, and put back as
//! shipped through the one door that replaces a file that did not read.
//!
//! This is `docs/contracts/person-settings.md`'s *A Settings surface, from
//! sign-in to the next change*, once, for appearance, the dock and shortcuts
//! alike — so the three cannot walk it three different ways. Each call below
//! is one the contract names, made through `crate::settings_keepers`:
//!
//! - **drawn at sign-in** is the keeper's `at_sign_in`: what the section draws
//!   and, when its file did not read, the refusal beside it;
//! - **a change** is the drawn value changed on a copy and the copy's changes
//!   handed whole to the keeper's `keep`. Only a copy the keeper wrote replaces
//!   what is drawn, so a refused change leaves the screen and the file exactly
//!   as they were, and the section says the keeper's refusal;
//! - **put back as shipped** is offered only while the file did not read, and
//!   is the keeper's own door for it.
//!
//! Nothing here decides a value, a file or a rule: a change is whatever the
//! crate that owns the value makes of the person's act.

use std::path::{Path, PathBuf};

use alo_strings::Strings;

use crate::settings_keepers::Keeper;

/// What a change or a putting back did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsKept {
    /// The keeper wrote it, and it is drawn.
    Written,
    /// The owning crate refused the change before anything was written — a
    /// chord somebody else has — and the section says why.
    Refused,
    /// The keeper did not write it, and the section says why. The file and
    /// what is drawn are as they were.
    NotWritten,
    /// The session has no folder to keep it in, so nothing was changed.
    NowhereToKeep,
    /// Putting back as shipped was asked of a section whose file reads, which
    /// is not offered: it would throw away what the person chose.
    NotOffered,
}

/// One section kept in the person's folder.
#[derive(Debug)]
pub(crate) struct KeptSection<K: Keeper> {
    /// The keeper's file, when the session has a folder.
    at: Option<PathBuf>,
    /// What the section draws.
    drawn: K::Drawn,
    /// Why the file did not read, while it does not.
    not_read: Option<K::NotRead>,
    /// What the section says about the last change, in the keeper's words.
    told: Option<String>,
}

impl<K: Keeper> KeptSection<K> {
    /// The section at sign-in: the file read once, or the release when the
    /// session has no folder.
    pub(crate) fn at_sign_in(at: Option<PathBuf>) -> Self {
        let (drawn, not_read) = match at.as_deref() {
            Some(at) => K::at_sign_in(at),
            None => (K::shipped(), None),
        };
        Self {
            at,
            drawn,
            not_read,
            told: None,
        }
    }

    /// What the section draws.
    pub(crate) fn drawn(&self) -> &K::Drawn {
        &self.drawn
    }

    /// Why the file did not read, while it does not.
    pub(crate) fn not_read(&self) -> Option<&K::NotRead> {
        self.not_read.as_ref()
    }

    /// What the section says about the last change.
    pub(crate) fn told(&self) -> Option<&str> {
        self.told.as_deref()
    }

    /// Where the keeper keeps it.
    pub(crate) fn at(&self) -> Option<&Path> {
        self.at.as_deref()
    }

    /// The person changed the section: `change` is made on a copy of what is
    /// drawn — refused in the owning crate's words when that crate refuses it —
    /// and the copy is kept whole.
    pub(crate) fn changed(
        &mut self,
        change: impl FnOnce(&mut K::Drawn) -> Result<(), String>,
        strings: &Strings,
    ) -> SettingsKept {
        let Some(at) = self.at.clone() else {
            return SettingsKept::NowhereToKeep;
        };
        let mut copy = self.drawn.clone();
        if let Err(said) = change(&mut copy) {
            self.told = Some(said);
            return SettingsKept::Refused;
        }
        match K::keep(&at, &copy) {
            Ok(()) => {
                self.drawn = copy;
                self.not_read = None;
                self.told = None;
                SettingsKept::Written
            }
            Err(refused) => {
                if let Some(why) = K::did_not_read(&refused) {
                    self.not_read = Some(why);
                }
                self.told = Some(K::not_written_said(&refused, strings));
                SettingsKept::NotWritten
            }
        }
    }

    /// The person put a section whose file did not read back as the release
    /// ships it.
    pub(crate) fn put_back_as_shipped(&mut self, strings: &Strings) -> SettingsKept {
        let Some(at) = self.at.clone() else {
            return SettingsKept::NowhereToKeep;
        };
        if self.not_read.is_none() {
            return SettingsKept::NotOffered;
        }
        match K::put_back_as_shipped(&at) {
            Ok(()) => {
                self.drawn = K::shipped();
                self.not_read = None;
                self.told = None;
                SettingsKept::Written
            }
            Err(refused) => {
                self.told = Some(K::not_written_said(&refused, strings));
                SettingsKept::NotWritten
            }
        }
    }
}
