//! **The language a person reads, given them at sign-in from the language the
//! machine was set up in** — where they have not said one themselves.
//!
//! The owner's decision of 2026-10-28: the language picked on the first-start
//! sheet is the person's **reading language**, so alo OS speaks and writes
//! dates in it from the first minute. The sheet keeps it in
//! [`alo_formats::the_machines_language::THE_MACHINES_LANGUAGE`] (it cannot
//! reach anybody's home, and there is nobody yet); this is the half that runs
//! **as the person**, in their own desktop, and writes it into their
//! `settings.toml` through the door that file already has for it,
//! `alo_choosing::Choosing::reading`.
//!
//! # Only where they have said nothing
//!
//! A person whose `[reading]` names a language has said what they read, and
//! nothing here touches it. A person whose `[reading]` is empty has not said —
//! `alo_choosing` holds an empty list to mean exactly that — and is given the
//! language the machine was set up in, which is the language the sheet was
//! answered in. It is written down rather than only used, so Settings and every
//! other process that reads `settings.toml` (the agent's own answers among
//! them) find the same language.
//!
//! # Nothing here can stop a sign-in
//!
//! A file that did not read, or a write the disk refused, is a sentence for the
//! session's log and a desktop in whatever language could still be found —
//! never a desktop that did not appear.

use std::path::Path;

use alo_strings::Language;

/// **What the person reads, and what happened finding out.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheyRead {
    /// Their languages, best first; empty is the machine's source language.
    pub languages: Vec<Language>,
    /// A sentence for the session's log, where there is something to say.
    pub for_the_log: Option<String>,
}

/// **The languages this person reads**: their own `settings.toml` at
/// `settings`, and where it names none, the language the machine at `machine`
/// was set up in, written into it.
#[must_use]
pub fn the_languages_they_read(settings: &Path, machine: &Path) -> TheyRead {
    let mut choosing = match alo_choosing::Choosing::at(settings) {
        Ok(choosing) => choosing,
        Err(why) => {
            return TheyRead {
                languages: Vec::new(),
                for_the_log: Some(format!(
                    "this person's settings at {} did not read ({why:?}), so no language is \
                     given them and the machine's source language is shown",
                    settings.display()
                )),
            };
        }
    };
    let already = choosing.settings().languages().to_vec();
    if !already.is_empty() {
        return TheyRead {
            languages: already,
            for_the_log: None,
        };
    }
    let set_up_in = match alo_formats::the_machines_language::read(machine) {
        Ok(Some(language)) => language,
        Ok(None) => {
            return TheyRead {
                languages: Vec::new(),
                for_the_log: None,
            };
        }
        Err(why) => {
            return TheyRead {
                languages: Vec::new(),
                for_the_log: Some(format!(
                    "the language this machine was set up in, at {}, did not read ({:?}), so \
                     this person is given none",
                    why.at().display(),
                    why.why()
                )),
            };
        }
    };
    let tag = set_up_in.tag().to_owned();
    let for_the_log = match choosing.reading(vec![set_up_in.clone()]) {
        Ok(()) => format!(
            "this person had not said what they read, so they are given {tag}, the language \
             this machine was set up in, kept in {}",
            settings.display()
        ),
        // **Used for this sign-in all the same.** The language is the one the
        // sheet was answered in; a disk that would not take the line is no
        // reason to show the person a language they did not pick.
        Err(why) => format!(
            "this person is given {tag}, the language this machine was set up in, for this \
             sign-in only: it could not be kept in {} ({why:?})",
            settings.display()
        ),
    };
    TheyRead {
        languages: vec![set_up_in],
        for_the_log: Some(for_the_log),
    }
}

#[cfg(test)]
#[path = "the_language_set_up_in_tests.rs"]
mod tests;
