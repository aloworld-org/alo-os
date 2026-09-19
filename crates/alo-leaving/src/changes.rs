//! What a person changed about leaving, which is one setting — and the list
//! that only exists because they turned it on.
//!
//! The shape `alo-dock`, `alo-appearance` and `alo-sleeping` keep, for the same
//! reason: the defaults live in the running release and the file holds the
//! difference, so a release that moves a default reaches every machine that
//! never touched it. An untouched machine has no `leaving.toml` at all
//! (`crate::keeping`).
//!
//! # One setting, and it ships off
//!
//! *Open my applications again when I sign in.* What alo OS ships is **not** to,
//! and that is a decision rather than a default nobody thought about: the list
//! of what was open is written only for a person who asked for it, so a machine
//! nobody configured keeps no record of what anybody had open. Turning the
//! setting off again takes the list with it, at the next log-out and through
//! [`Changes::forget`] at once.
//!
//! # The list is in this file, and that is deliberate
//!
//! It would be tidier to keep what was open somewhere other than a settings
//! file, and it would be worse. One file has one writer, one format number and
//! one rule — `alo-kept`'s: refused whole when wrong, written whole or not at
//! all, read back before it counts (ADR 0038) — and a second file holding
//! session state would be a second set of those decisions made again. It also
//! puts the choice and the consequence of the choice in one place a person can
//! open in an editor and understand.

use serde::{Deserialize, Serialize};

use crate::open::{Open, WasOpen};

/// One thing a person can change about leaving, for a settings panel that
/// offers *put it back*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Setting {
    /// Whether their applications open again when they sign in.
    Reopen,
    /// The list itself, which is not something a person sets but is something
    /// they may want gone.
    WhatWasOpen,
}

/// Everything a person has changed about leaving, and everything that was open
/// when they last left.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Written", into = "Written")]
pub struct Changes {
    /// Whether they asked for their applications to open again.
    reopen: Option<bool>,
    /// What was open when they last logged out, oldest first. Empty unless the
    /// setting above is on.
    was_open: Vec<Open>,
}

impl Changes {
    /// Nothing changed yet, and nothing kept.
    #[must_use]
    pub fn untouched() -> Self {
        Self::default()
    }

    /// Whether nothing has been changed or kept at all.
    #[must_use]
    pub const fn is_untouched(&self) -> bool {
        self.reopen.is_none() && self.was_open.is_empty()
    }

    /// Open my applications again when I sign in, or do not.
    pub fn set_reopen(&mut self, on: bool) {
        self.reopen = Some(on);
    }

    /// What was open, kept for the next sign-in.
    ///
    /// `pub(crate)`: the one caller is [`crate::keeping::at_sign_out`], which is
    /// the one moment a list is written and the one place the rule *only if the
    /// person asked for it* is applied.
    pub(crate) fn set_what_was_open(&mut self, was_open: WasOpen) {
        self.was_open = was_open.into_windows();
    }

    /// What was open, as this file holds it.
    #[must_use]
    pub fn what_was_open(&self) -> WasOpen {
        WasOpen::of(self.was_open.clone())
    }

    /// Forget that this was ever changed — or, for the list, that it was ever
    /// kept. Says whether there was anything to forget.
    pub fn forget(&mut self, setting: Setting) -> bool {
        match setting {
            Setting::Reopen => self.reopen.take().is_some(),
            Setting::WhatWasOpen => !std::mem::take(&mut self.was_open).is_empty(),
        }
    }
}

/// The settings a session runs by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// Whether the person's applications open again when they sign in.
    pub reopen: bool,
}

impl Settings {
    /// What alo OS ships: nothing is reopened, and nothing is written down.
    #[must_use]
    pub const fn shipped() -> Self {
        Self { reopen: false }
    }

    /// These settings with a person's changes over them.
    #[must_use]
    pub const fn with(self, changes: &Changes) -> Self {
        Self {
            reopen: match changes.reopen {
                Some(chosen) => chosen,
                None => self.reopen,
            },
        }
    }
}

/// Changes as a settings file holds them: anything untouched is absent.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Written {
    /// Whether applications open again at the next sign-in, if that was chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reopen: Option<bool>,
    /// What was open, if anything was kept.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    was_open: Vec<Open>,
}

impl From<Written> for Changes {
    fn from(written: Written) -> Self {
        Self {
            reopen: written.reopen,
            was_open: written.was_open,
        }
    }
}

impl From<Changes> for Written {
    fn from(changes: Changes) -> Self {
        Self {
            reopen: changes.reopen,
            was_open: changes.was_open,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::split::Split;
    use crate::testing::{the_editor, the_laptop};

    /// **What alo OS ships reopens nothing**, and a person's choice lands over
    /// it.
    #[test]
    fn what_ships_reopens_nothing_and_a_choice_lands_over_it() {
        assert!(!Settings::shipped().reopen);
        assert_eq!(
            Settings::shipped().with(&Changes::untouched()),
            Settings::shipped()
        );

        let mut changes = Changes::untouched();
        changes.set_reopen(true);
        assert!(Settings::shipped().with(&changes).reopen);
        changes.set_reopen(false);
        assert!(!Settings::shipped().with(&changes).reopen);
    }

    /// **Forgetting the setting and forgetting the list are two acts**, and
    /// either one alone leaves the other where it was.
    #[test]
    fn the_setting_and_the_list_are_forgotten_separately() {
        let mut changes = Changes::untouched();
        assert!(changes.is_untouched());
        changes.set_reopen(true);
        changes.set_what_was_open(
            WasOpen::nothing()
                .and(Open::of(the_editor(), the_laptop(), Split::TheWholeScreen).unwrap()),
        );
        assert!(!changes.is_untouched());

        assert!(changes.forget(Setting::WhatWasOpen));
        assert!(!changes.forget(Setting::WhatWasOpen));
        assert!(changes.what_was_open().is_nothing());
        assert!(
            Settings::shipped().with(&changes).reopen,
            "the choice stayed"
        );

        assert!(changes.forget(Setting::Reopen));
        assert!(!changes.forget(Setting::Reopen));
        assert!(changes.is_untouched());
    }
}
