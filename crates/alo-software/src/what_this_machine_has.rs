//! Reading this machine's installed applications off the machine itself.
//!
//! `alo_applications::Installed` is the list a call is checked against, and its
//! own header leaves this seam open in as many words: *whatever builds this list
//! reads it off the machine: desktop entries, the sandbox's own list of what is
//! installed. That is Linux's, and this crate holds no opinion about where the
//! entries come from.* **Nothing had ever plugged into that seam.** Every
//! production reference to building an `Installed` was a test, so on every
//! machine the list was empty and `WhatOpensWhat` was asked about nothing.
//!
//! `docs/features.md` [v0.01] is what needs it: a fresh Dock starts with Files,
//! Browser, Apps and Settings *resolved to applications that are actually
//! installed*, and its own note says `alo_dock::pinning` answers this and
//! **nothing produces the discovery it needs, so no machine has ever drawn it**.
//!
//! # Why this is the sandbox's list and not the desktop entries
//!
//! Both are real sources and they answer different questions. Desktop entries
//! are *what can be launched*, including anything a person dropped into
//! `~/.local/share/applications` by hand. The rented tool's list is *what this
//! machine installed*, which is the question a pin has to be resolved against:
//! ADR 0005 says applications arrive as Flatpaks from a place a person or an
//! organisation chose, so what the tool knows about is what alo OS put there.
//!
//! It is also the list already readable. `crate::asked::installed` builds the
//! arguments, `crate::heard::identifiers` parses the answer, and
//! `crate::Tool::installed` joins them — all three were built and had **no
//! caller**. This file is the caller, and it adds the one step none of them do:
//! turning identifiers into [`Installed`].
//!
//! # A failed scan is not an empty machine
//!
//! The promise says it outright: *an unfinished scan and a failed one are not
//! evidence that an application is absent.* So this returns
//! [`Result`] and **a failure yields no list at all** — there is no value a
//! caller could mistake for *this machine has nothing*. That is the shape
//! `absent is stronger than defaulted` asks for: a default can be failed to
//! override, an absence cannot be overridden wrongly.
//!
//! A scan that **succeeded and found nothing** is `Ok` with an empty list, and
//! that is a different fact from a failure. A machine really can have no
//! applications — before `alo-shipping` has run, which is exactly when a fresh
//! Dock is first drawn.

use alo_applications::{Application, Installed};

use crate::tool::{Failed, Tool};

/// What one scan found, and what it would not take.
///
/// **The refusals are carried rather than dropped**, because
/// `Installed::add`'s own rule is that it *says which happened rather than
/// swallowing it* — two entries claiming one identifier is a packaging fault
/// and the list has to do something about it. A reading that quietly discarded
/// what it could not use would be a reading nobody could audit, and the
/// identifiers come from outside this repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatThisMachineHas {
    /// The applications, by identifier.
    installed: Installed,
    /// Lines the tool printed that are not identifiers this crate will take, in
    /// the order it printed them.
    not_taken: Vec<String>,
    /// Identifiers the tool printed more than once, in the order of the second
    /// and later sightings.
    ///
    /// Separate from [`Self::not_taken`], because they are different faults
    /// with different owners: one is a line this crate does not understand, the
    /// other is one machine claiming an identifier twice.
    twice: Vec<String>,
}

impl WhatThisMachineHas {
    /// The applications this machine has.
    #[must_use]
    pub const fn installed(&self) -> &Installed {
        &self.installed
    }

    /// Lines that were not identifiers this crate will take.
    #[must_use]
    pub fn not_taken(&self) -> &[String] {
        &self.not_taken
    }

    /// Identifiers the tool printed more than once.
    #[must_use]
    pub fn twice(&self) -> &[String] {
        &self.twice
    }

    /// How many applications were taken.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.installed.how_many()
    }

    /// Whether the reading took everything the tool printed.
    ///
    /// **Not whether it found anything.** A machine with no applications read
    /// cleanly is whole; a machine whose tool printed a line this crate could
    /// not use is not, however many it also printed.
    #[must_use]
    pub fn is_whole(&self) -> bool {
        self.not_taken.is_empty() && self.twice.is_empty()
    }
}

/// Read what this machine has installed, through the rented tool.
///
/// # Errors
///
/// [`Failed`], exactly as the tool reported it. **A failure carries no list**,
/// so a caller cannot read a machine that would not answer as a machine with
/// nothing on it — `docs/features.md` requires that distinction and says why:
/// *an unfinished scan and a failed one are not evidence that an application is
/// absent.*
pub fn what_this_machine_has(tool: &impl Tool) -> Result<WhatThisMachineHas, Failed> {
    let printed = tool.installed()?;
    let mut installed = Installed::nothing();
    let mut not_taken = Vec::new();
    let mut twice = Vec::new();
    for line in printed {
        // The tool is started in the C locale and prints one identifier per
        // line, so a line is either an identifier or something this crate does
        // not understand. It is never a sentence to be interpreted.
        let Ok(application) = Application::identified(&line) else {
            not_taken.push(line);
            continue;
        };
        if !installed.add(application) {
            twice.push(line);
        }
    }
    Ok(WhatThisMachineHas {
        installed,
        not_taken,
        twice,
    })
}

#[cfg(test)]
#[path = "what_this_machine_has_tests.rs"]
mod tests;
