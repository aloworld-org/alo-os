//! **What the command is called**: `alo`, and the name it had before.
//!
//! The owner renamed it on 2026-11-06 ([ADR 0124](../../../docs/decisions/0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md),
//! amended): a person types `alo email Sarah that I'm late`. **`alo-ask` keeps
//! working** — it is a public surface, and scripts and fingers already know it
//! (`CLAUDE.md`, *contracts outlive code*). The image installs it as a link to the
//! same program, and typed by that name the command answers exactly as it would
//! as `alo`, with one line on stderr saying the name has changed. Nothing on
//! stdout differs, so `alo-ask … > answer.txt` still holds the answer alone.

use std::ffi::OsStr;
use std::path::Path;

/// The command's name.
pub const ITS_NAME: &str = "alo";

/// The name it had until 2026-11-06, still answered to and deprecated.
pub const ITS_OLD_NAME: &str = "alo-ask";

/// Whether the program was started by its old name, from the first argument
/// the system gave it — a path, or a bare name.
///
/// **Only the old name exactly**: a program at `/usr/bin/alo-ask` was, and one
/// at `/usr/bin/alo`, or called anything else, was not. No first argument is
/// not the old name either, because then nobody typed it.
#[must_use]
pub fn called_by_its_old_name(first: Option<&OsStr>) -> bool {
    first
        .and_then(|first| Path::new(first).file_name())
        .is_some_and(|name| name == ITS_OLD_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The old name, however the system spelled the path, is the old name.**
    #[test]
    fn the_old_name_is_recognised_by_its_path_or_alone() {
        for old in ["alo-ask", "/usr/bin/alo-ask", "./alo-ask"] {
            assert!(called_by_its_old_name(Some(OsStr::new(old))), "{old}");
        }
    }

    /// **Nothing else is**: the new name, a name that only begins like the old
    /// one, a directory named like it, and no name at all.
    #[test]
    fn nothing_else_is_the_old_name() {
        for not in [
            "alo",
            "/usr/bin/alo",
            "alo-asks",
            "my-alo-ask",
            "/opt/alo-ask/alo",
            "",
        ] {
            assert!(!called_by_its_old_name(Some(OsStr::new(not))), "{not}");
        }
        assert!(!called_by_its_old_name(None));
        assert_eq!(ITS_NAME, "alo");
    }
}
