//! The one row that offers to add alo's shell integration, and nothing it says.
//!
//! [ADR 0123](../../../docs/decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
//! term 6: *alo OS ships the snippet, the sentence saying plainly what it
//! reveals, and a one-press way to add it. alo does not write to anybody's
//! shell startup unprompted.* The snippet and the sentence shipped with the
//! terminal; this row is the one-press way, and the press is the prompt.
//!
//! # Why there is almost nothing here
//!
//! `tests/settings_source.rs` holds every Settings file to containing **no
//! string literal at all** — *Settings words nothing; every sentence is a
//! crate's*. The first version of this file broke that rule four times over, in
//! `Filling::of("file", …)` placeholder names, and the rule was right: a crate
//! that knows what a shell snippet is and which file it belongs in is the crate
//! that should say so. So the state, the sentences and the write all live in
//! `alo_terminal::where_the_integration_stands`, and this forwards.
//!
//! # Why this is not a [`crate::settings_keepers::Keeper`]
//!
//! Every other section keeps a file of alo's own, whose whole content alo
//! decides. This one **appends to a file that is the person's** — a `.bashrc`
//! that may have been theirs for years. So there is no *put back as shipped*
//! here and there never will be: `crate::settings_window` refuses it for this
//! section, because putting a person's shell startup back as alo ships it is
//! not a thing alo has any business doing.

use std::path::Path;

use alo_strings::Strings;
use alo_terminal::where_the_integration_stands::{WhatThePressDid, WhereItStands, WhichOne};

use crate::settings_kept::SettingsKept;

/// What Settings found about alo's shell integration when the window opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheShellIntegration {
    /// The terminal's answer, which carries the state and every sentence.
    stands: WhereItStands,
}

impl TheShellIntegration {
    /// Read once when Settings opens, like every other section.
    pub(crate) fn at_sign_in(shell: Option<&str>, folder: Option<&Path>) -> Self {
        Self::about(WhichOne::TheIntegration, shell, folder)
    }

    /// The same, for a named one of the two things a person may add.
    ///
    /// **One forwarder and not two.** The two rows differ in the words they say
    /// and the lines they append, both of which `alo-terminal` decides; what
    /// this file does — carry the state Settings read when it opened, and turn
    /// a press into a `SettingsKept` — is identical for them, and a second copy
    /// of it is the fault `docs/misreadings/a-check-fixed-in-one-place-leaves-its-copy.md`
    /// is named after.
    pub(crate) fn about(which: WhichOne, shell: Option<&str>, folder: Option<&Path>) -> Self {
        Self {
            stands: WhereItStands::read_about(which, shell, folder),
        }
    }

    /// What the section says above its row.
    pub(crate) fn said(&self, strings: &Strings) -> Vec<String> {
        self.stands.said(strings)
    }

    /// Whether there is a row to draw and act on.
    pub(crate) const fn offers_a_row(&self) -> bool {
        self.stands.offers_a_row()
    }

    /// The row's words.
    pub(crate) fn row(&self, strings: &Strings) -> Vec<String> {
        self.stands.row(strings)
    }

    /// Whether it is already there, which is what the row's mark says.
    pub(crate) const fn is_already_there(&self) -> bool {
        self.stands.is_already_there()
    }

    /// The person pressed the row.
    ///
    /// The terminal's three outcomes become this window's: there, refused, or
    /// nothing to add. **`NowhereToKeep` for the last** — it is the answer
    /// every other section gives when there is no file to write in, and a row
    /// that cannot be acted on is not drawn at all.
    pub(crate) fn add(&mut self, strings: &Strings) -> SettingsKept {
        match self.stands.add(strings) {
            WhatThePressDid::ItIsThere => SettingsKept::Written,
            WhatThePressDid::ItWouldNotTakeIt => SettingsKept::NotWritten,
            WhatThePressDid::ThereWasNothingToAdd => SettingsKept::NowhereToKeep,
        }
    }
}

#[cfg(test)]
#[path = "settings_shell_integration_tests.rs"]
mod tests;
