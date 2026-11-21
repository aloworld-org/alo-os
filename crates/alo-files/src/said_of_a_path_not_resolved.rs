//! **What the daemon says on the journal when a path would not resolve**:
//! which way, and the path's shape — never the path.
//!
//! At full access only resolving a path can refuse opening a file
//! (`crate::touching`: the path as written and where it really leads are both
//! answered by full access), and main's walks saw exactly that refusal for
//! VSCodium with nothing anywhere saying which way it failed — the refusal
//! crosses as a sentence, and its sentence names the path (run 44912175590,
//! 1109 s: *"it was decided where the path was resolved"*). This says the kind
//! and the shape of what was written, so a walk can tell a path nothing
//! expands, like `~/hello.txt`, from one the machine would not read.
//!
//! Written as the daemon's own stderr, which is this machine's journal.

use std::path::Path;

use crate::real::RealError;

/// The words a line saying a path would not resolve begins with.
pub const A_PATH_WOULD_NOT_RESOLVE: &str = "a path would not resolve:";

/// The line saying `given` would not resolve, and why, in kind.
pub(crate) fn said(given: &Path, why: &RealError) -> String {
    let kind = match why {
        RealError::Nothing { .. } => "nothing is there",
        RealError::Unreadable { .. } => "it could not be read",
    };
    let shape = if given.starts_with("~") {
        "beginning with ~"
    } else if given.is_absolute() {
        "absolute"
    } else {
        "relative"
    };
    format!("{A_PATH_WOULD_NOT_RESOLVE} {kind}, the path written {shape}")
}

#[cfg(test)]
#[path = "said_of_a_path_not_resolved_tests.rs"]
mod tests;
