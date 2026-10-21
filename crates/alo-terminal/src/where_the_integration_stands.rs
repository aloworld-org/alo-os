//! Whether alo's shell integration is in this person's startup file, and every
//! sentence said about it.
//!
//! # Why the wording is here and not in Settings
//!
//! `alo-shell`'s `settings_source.rs` holds every Settings file to **containing
//! no string literal at all** — *Settings words nothing; every sentence is a
//! crate's*. That rule is blunt on purpose and it caught the first version of
//! this: four `Filling::of("file", …)` placeholder names, which are not text a
//! person reads, were refused along with everything else.
//!
//! The rule is right and the first version was wrong. A crate that knows what a
//! shell snippet is, which file it goes in, and what adding it reveals is the
//! crate that should say so — and then there is one place to change when `zsh`
//! arrives, rather than a sentence here and a placeholder name there.
//!
//! So this module answers in whole sentences and `alo-shell` forwards them.

use std::path::{Path, PathBuf};

use alo_strings::{Filling, Strings};

use crate::the_snippet_a_person_installs::{self as snippet, Added, NotAdded};

/// What Settings found about the integration, and what the last press did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereItStands {
    /// What there is to offer this person.
    what: What,
    /// What the last press said, in this crate's words.
    told: Option<String>,
}

/// What there is to offer, which is four things and not one.
///
/// **Told apart rather than collapsed into *no*.** A shell alo ships nothing
/// for, a session with no folder, a file that already holds the snippet and a
/// session that never said which shell are four different answers, and one row
/// that did nothing in all four would be the same surface lying four ways.
#[derive(Debug, Clone, PartialEq, Eq)]
enum What {
    /// A shell alo ships for, its startup file, and whether it is already there.
    Offered {
        /// The shell, kept because the press needs it and deriving it twice is
        /// two places that could disagree.
        shell: String,
        /// The startup file, inside the person's folder.
        at: PathBuf,
        /// Whether the snippet is already in it, read when Settings opened.
        already: bool,
    },
    /// The session has no folder, so there is nowhere to add it.
    Nowhere,
    /// A shell alo ships nothing for, named so a person reads which.
    NoSuchShell(String),
    /// The session did not say which shell this account uses.
    ///
    /// **No row and no sentence.** Saying *we have none for your shell* would
    /// be a claim about a shell nobody named.
    NotSaid,
}

/// What a press did, for the surface to report in its own vocabulary of
/// outcomes.
///
/// **Not `alo-shell`'s `SettingsKept`**, which is that crate's type: this crate
/// does not know what a Settings section is, and a shared type would make it
/// know.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatThePressDid {
    /// The snippet is in the file, whether this press put it there or found it.
    ItIsThere,
    /// The file would not take the change, and nothing was written.
    ItWouldNotTakeIt,
    /// There was nothing to add, which the row's absence already says.
    ThereWasNothingToAdd,
}

impl WhereItStands {
    /// What there is to offer, read once when Settings opens.
    ///
    /// `shell` is what the session was told this account's shell is, and
    /// `folder` the person's folder — both handed over rather than read from
    /// the environment.
    #[must_use]
    pub fn read(shell: Option<&str>, folder: Option<&Path>) -> Self {
        let what = match (shell, folder) {
            (None, _) => What::NotSaid,
            (Some(shell), folder) => match (snippet::the_startup_file_of(shell), folder) {
                (None, _) => What::NoSuchShell(shell.to_owned()),
                (Some(_), None) => What::Nowhere,
                (Some(named), Some(folder)) => {
                    let at = folder.join(named);
                    let already = std::fs::read_to_string(&at)
                        .ok()
                        .and_then(|held| snippet::already_in(&held, shell))
                        .unwrap_or(false);
                    What::Offered {
                        shell: shell.to_owned(),
                        at,
                        already,
                    }
                }
            },
        };
        Self { what, told: None }
    }

    /// The sentences said above the row: why there is none, and what the last
    /// press did.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<String> {
        let mut said = Vec::new();
        match &self.what {
            What::Nowhere => said.push(
                strings
                    .say(&crate::words::NOWHERE_TO_ADD_IT.key(), &Filling::nothing())
                    .into_text(),
            ),
            What::NoSuchShell(shell) => said.push(
                strings
                    .say(
                        &crate::words::NO_INTEGRATION_FOR_THIS_SHELL.key(),
                        &Filling::of("shell", shell.clone()),
                    )
                    .into_text(),
            ),
            What::Offered { .. } | What::NotSaid => {}
        }
        said.extend(self.told.clone());
        said
    }

    /// Whether there is a row to draw and act on.
    ///
    /// **One answer for both questions**, because a surface's row list and its
    /// drawing are two readers of it, and a row drawn but not acted on — or
    /// acted on but not drawn — is a fault somebody has to be able to catch.
    #[must_use]
    pub const fn offers_a_row(&self) -> bool {
        matches!(self.what, What::Offered { .. })
    }

    /// Whether the snippet is already in the person's startup file.
    #[must_use]
    pub const fn is_already_there(&self) -> bool {
        matches!(self.what, What::Offered { already: true, .. })
    }

    /// The row's words: what it gives, what it reveals, and where it would go.
    ///
    /// Empty where there is no row, which [`Self::offers_a_row`] answers first.
    /// **The cost is on the row rather than only in a confirmation afterwards**,
    /// because a person decides before pressing.
    #[must_use]
    pub fn row(&self, strings: &Strings) -> Vec<String> {
        let What::Offered { at, already, .. } = &self.what else {
            return Vec::new();
        };
        vec![
            strings
                .say(
                    &crate::words::ALO_CAN_READ_WHAT_YOU_TYPE.key(),
                    &Filling::nothing(),
                )
                .into_text(),
            strings
                .say(&crate::words::WHAT_IT_REVEALS.key(), &Filling::nothing())
                .into_text(),
            strings
                .say(
                    &if *already {
                        crate::words::IT_IS_ALREADY_THERE.key()
                    } else {
                        crate::words::IT_WOULD_BE_ADDED_TO.key()
                    },
                    &Filling::of("file", named(at)),
                )
                .into_text(),
        ]
    }

    /// The person asked for it to be added.
    ///
    /// **The only thing in alo OS that writes to a person's shell startup
    /// file**, reached from a key they pressed on a row that says what it
    /// reveals. Nothing calls it otherwise, which is
    /// [ADR 0123](../../../docs/decisions/0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
    /// term 6's *alo does not write to anybody's shell startup unprompted*.
    pub fn add(&mut self, strings: &Strings) -> WhatThePressDid {
        let What::Offered { shell, at, already } = &mut self.what else {
            return WhatThePressDid::ThereWasNothingToAdd;
        };
        let file = named(at);
        match snippet::add_to(at, shell) {
            // **Appended and already-there are one answer to a person.** They
            // pressed, and now it is there; which of the two it was is this
            // crate's business and not news.
            Ok(Added::Appended | Added::AlreadyThere) => {
                *already = true;
                self.told = Some(
                    strings
                        .say(
                            &crate::words::IT_WAS_ADDED.key(),
                            &Filling::of("file", file),
                        )
                        .into_text(),
                );
                WhatThePressDid::ItIsThere
            }
            Err(why) => {
                self.told = Some(
                    strings
                        .say(
                            &crate::words::IT_WOULD_NOT_TAKE_THE_CHANGE.key(),
                            &Filling::of("file", file).and("why", said_plainly(&why)),
                        )
                        .into_text(),
                );
                WhatThePressDid::ItWouldNotTakeIt
            }
        }
    }
}

/// A startup file's name, as it is on the person's disk.
///
/// The name rather than the whole path: a person knows what `.bashrc` is, and
/// the full path is their home directory spelled out in a sentence.
fn named(at: &Path) -> String {
    at.file_name().map_or_else(
        || at.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// What the machine said, for the `{why}` a refusal carries.
fn said_plainly(why: &NotAdded) -> String {
    why.to_string()
}

#[cfg(all(test, target_os = "linux"))]
#[path = "where_the_integration_stands_tests.rs"]
mod tests;
