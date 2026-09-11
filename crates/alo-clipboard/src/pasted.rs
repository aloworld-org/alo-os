//! What a completed transfer is: the bytes, the form they are in, and whether
//! the owner has just given something up.
//!
//! Made by [`crate::Clipboard::paste`] and by nothing else — no public field, no
//! constructor, no `From` and no deserialiser — so a value of this type is
//! always something an owner really produced, in a form that owner really
//! offered. A paste that could be assembled beside the clipboard would be a
//! paste of bytes nobody copied, which is the whole of what this crate exists to
//! make impossible.
//!
//! # The form travels with the bytes
//!
//! Because the bytes alone do not say what they are. A paster that asked for
//! `text/html` and is handed a `Vec<u8>` has to remember what it asked for; a
//! paster handed a [`Pasted`] does not, and cannot be wrong about it.
//!
//! # And whether anything moved
//!
//! [`Pasted::the_owner_gave_it_up`] is a cut completing. It is a fact reported
//! rather than an action taken: this crate removes nobody's files and deletes
//! nobody's text. What it does is retire the offer, so nothing can paste the
//! same move a second time — see `clipboard.rs`.

use crate::kind::Kind;

/// What was pasted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pasted {
    /// The form it is in, which is one the owner offered.
    form: Kind,
    /// What the owner produced.
    bytes: Vec<u8>,
    /// Whether this transfer completed a cut.
    gave_it_up: bool,
}

impl Pasted {
    /// A completed transfer. `pub(crate)` — [`crate::Clipboard`] is the only
    /// thing that has ever asked an owner for anything.
    pub(crate) fn of(form: Kind, bytes: Vec<u8>, gave_it_up: bool) -> Self {
        Self {
            form,
            bytes,
            gave_it_up,
        }
    }

    /// The form it is in.
    #[must_use]
    pub fn form(&self) -> &Kind {
        &self.form
    }

    /// What was pasted.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The same, given away, for a paster that is about to write it somewhere.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// How much of it there is.
    #[must_use]
    pub fn how_many_bytes(&self) -> usize {
        self.bytes.len()
    }

    /// Whether this transfer completed a cut — the owner no longer has what it
    /// offered, and the offer is retired.
    ///
    /// A fact about what just happened, not an instruction: nothing in this
    /// crate removes anything of anybody's.
    #[must_use]
    pub fn the_owner_gave_it_up(&self) -> bool {
        self.gave_it_up
    }
}
