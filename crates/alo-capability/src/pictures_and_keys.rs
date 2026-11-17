//! **Pictures and keys, for the applications that tell alo nothing about their
//! windows**: which applications the person has let alo look at and press keys
//! in ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q11, and the owner's *"Yes, build it"* of 2026-11-17).
//!
//! # What this is, and what it is not
//!
//! It is the **answer** and nothing else, beside [`crate::AloInApplications`]
//! for that type's reason: a person's choices live in their own settings and
//! whoever acts is handed the answer. It is kept in the same file
//! (`docs/contracts/how-code-runs-file.md`, `pictures-and-keys-in`), under the
//! same rules about who may have written it, so **an agent cannot switch it on
//! for itself**.
//!
//! It is not a grant. A grant says alo may use an application at all (ADR 0040,
//! ADR 0143 Q10) and has an end; this says **by which road**, once alo is let
//! in, and lasts until the person switches it off. Neither implies the other:
//! a switch on here reaches no application alo was not let into, and a grant
//! reaches no picture of a window where this is off.
//!
//! # Off, application by application
//!
//! ADR 0143's table: *Pictures and keystrokes, for applications nothing
//! describes — default **off**; more: **on, per application**.* So the answer
//! is a set of application identifiers, **empty as a machine arrives**, and
//! **empty for a file that will not read** ([`PicturesAndKeys::as_read`]): a
//! file somebody else may have written cannot speak for the person, and *no
//! application* is the only answer that cannot be one.
//!
//! # Never a terminal
//!
//! **A terminal is never in the set, whatever the file says**
//! ([`PicturesAndKeys::allows`]). What a terminal shows is the person's work
//! and what is typed into one runs; ADR 0111 is the one road to a terminal, and
//! it reads text rather than pictures. A file that names one is refused whole
//! by its reader, and a set built here drops it.
//!
//! # What it costs is said, and only said
//!
//! [`PicturesAndKeys::what_it_costs`] is ADR 0143's own sentence. The
//! indicator, the record and alo's teal border stay on whichever way the switch
//! is: a picture that leaves this computer is shown when it leaves (law 1).

use std::collections::BTreeSet;

use alo_strings::Word;

use crate::words;

/// The applications the person let alo use pictures and keys in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PicturesAndKeys {
    /// Each by the identifier the machine knows it by, matched exactly.
    on_in: BTreeSet<String>,
}

impl PicturesAndKeys {
    /// The set of applications this answer names, keeping only identifiers
    /// one could be: **never a terminal**, nothing empty, nothing with space
    /// around it.
    #[must_use]
    pub fn of<I, S>(applications: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            on_in: applications
                .into_iter()
                .map(Into::into)
                .filter(|it| could_be_one(it))
                .collect(),
        }
    }

    /// Whether alo may use pictures and keys in this application.
    ///
    /// Exact, with no case folding and no trimming, as every identifier in
    /// this crate is matched. **A terminal never**, at any answer.
    #[must_use]
    pub fn allows(&self, application: &str) -> bool {
        could_be_one(application) && self.on_in.contains(application)
    }

    /// This answer with `application` switched on; unchanged for an
    /// identifier [`Self::of`] would not keep.
    #[must_use]
    pub fn switched_on(mut self, application: &str) -> Self {
        if could_be_one(application) {
            self.on_in.insert(application.to_owned());
        }
        self
    }

    /// This answer with `application` switched off.
    #[must_use]
    pub fn switched_off(mut self, application: &str) -> Self {
        self.on_in.remove(application);
        self
    }

    /// The applications it is switched on in, in identifier order: what the
    /// file writes.
    pub fn applications(&self) -> impl Iterator<Item = &str> {
        self.on_in.iter().map(String::as_str)
    }

    /// The answer a reader of the person's file goes by: what it says, or,
    /// for a file that is there and will not read, **no application** — never
    /// what the file might have meant.
    #[must_use]
    pub fn as_read<E>(read: Result<Self, E>) -> Self {
        read.unwrap_or_default()
    }

    /// What switching it on costs: ADR 0143's own words, said beside the
    /// switch whichever way it is.
    #[must_use]
    pub const fn what_it_costs() -> Word {
        words::WHAT_PICTURES_AND_KEYS_COST
    }
}

/// Whether this identifier is one alo may be let use pictures and keys in:
/// **not a terminal**, not empty, and with no space around it — which trimming
/// would turn into a different application.
fn could_be_one(identifier: &str) -> bool {
    !identifier.is_empty()
        && identifier.trim() == identifier
        && !crate::is_a_persons_own(identifier)
}

#[cfg(test)]
#[path = "pictures_and_keys_tests.rs"]
mod tests;
