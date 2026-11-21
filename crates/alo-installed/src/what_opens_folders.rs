//! **Which installed applications open folders** — what a file manager is, on
//! this machine, as its own entry says it.
//!
//! The owner's decision of 2026-11-17: a person asking alo to open a file *in*
//! a file manager means what a double-click there means. So the turn has to
//! know which applications are file managers, and that is **declared once,
//! here**, from the one thing every file manager's entry says: it opens
//! `inode/directory`, the media type of a folder. Never a list of identifiers
//! written by hand: a file manager the person installs tomorrow says the same
//! line in its entry, and is one.
//!
//! Read off the same walk as [`crate::read`], so the two answers agree about
//! which entries count, and earlier places win.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// The media type of a folder, which a file manager's entry declares it opens.
pub const A_FOLDER: &str = "inode/directory";

/// The identifiers of every installed application whose entry says it opens
/// folders, read from these places, earlier places winning.
#[must_use]
pub fn what_opens_folders(places: &[PathBuf]) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut opening = BTreeSet::new();
    for place in places {
        for entry in crate::entries_under(place) {
            // **The first entry for an identifier is the one**, the walk's own
            // rule: a person's copy that no longer opens folders is not one.
            if !seen.insert(entry.identifier.clone()) {
                continue;
            }
            if entry.said.opens.iter().any(|kind| kind == A_FOLDER) {
                opening.insert(entry.identifier);
            }
        }
    }
    opening
}

#[cfg(test)]
#[path = "what_opens_folders_tests.rs"]
mod tests;
