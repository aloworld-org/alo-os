//! What the image makes at boot, which is the two directories the daemon
//! refuses to make itself.
//!
//! `systemd-tmpfiles` reads these lines. What matters here is that both
//! refusals in `alo-agentd` — [ADR 0017](../../../docs/decisions/0017-the-agents-door-is-ours-and-not-in-the-session.md)'s
//! `/run/alo` and the machine description's record folder — are answered by
//! something, because on a machine where they are not the service names a
//! directory and stops.
//!
//! # Only the five fields that decide anything
//!
//! A `tmpfiles.d` line has seven: type, path, mode, user, group, age and an
//! argument. The last two are read past rather than into, because nothing alo OS
//! promises depends on them and a field this crate had an opinion about would be
//! an opinion nobody decided.

use std::path::{Path, PathBuf};

use crate::refusing::NotMade;

/// The type letter that means an ordinary directory, made if it is not there.
pub const A_DIRECTORY: &str = "d";

/// One line of a `tmpfiles.d` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Made {
    /// What kind of thing it is: `d` for a directory, and the rest.
    kind: String,
    /// Where it goes.
    at: PathBuf,
    /// The mode it is made with.
    mode: u32,
    /// The login that owns it.
    owner: String,
    /// The group it is in.
    group: String,
}

impl Made {
    /// What kind of thing this line makes.
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Where it goes.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// The mode it is made with.
    #[must_use]
    pub const fn mode(&self) -> u32 {
        self.mode
    }

    /// The login that owns it.
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// The group it is in.
    #[must_use]
    pub fn group(&self) -> &str {
        &self.group
    }

    /// Whether this line makes a directory at this path.
    #[must_use]
    pub fn is_a_directory_at(&self, path: &Path) -> bool {
        self.kind == A_DIRECTORY && self.at == path
    }
}

/// Everything a `tmpfiles.d` file makes.
///
/// # Errors
///
/// [`NotMade::TooFewFields`] for a line with no owner or group on it, and
/// [`NotMade::NotAMode`] for a mode that is not octal — including `-`, which
/// systemd reads as *whatever the umask says* and which this crate refuses,
/// because the mode of `/run/alo` is a decision rather than an inheritance.
pub fn everything_made(text: &str) -> Result<Vec<Made>, NotMade> {
    let mut made = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let at = index + 1;
        let mut fields = line.split_whitespace();
        let (Some(kind), Some(path), Some(mode), Some(owner), Some(group)) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            return Err(NotMade::TooFewFields {
                at,
                line: line.to_owned(),
            });
        };
        let mode = u32::from_str_radix(mode, 8).map_err(|_ignored| NotMade::NotAMode {
            at,
            mode: mode.to_owned(),
        })?;
        made.push(Made {
            kind: kind.to_owned(),
            at: PathBuf::from(path),
            mode,
            owner: owner.to_owned(),
            group: group.to_owned(),
        });
    }
    Ok(made)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The two lines alo OS's own image ships, with a comment above them.
    const BOTH: &str = "\
# the door every person's is beneath
d /run/alo 0755 root root -
d /var/lib/alo 0700 alo alo -
";

    /// The ordinary case: two directories, each with the mode and the owner
    /// somebody chose.
    #[test]
    fn both_directories_read_back_as_they_were_written() {
        let made = everything_made(BOTH).unwrap();

        assert_eq!(made.len(), 2);
        let door = made.first().unwrap();
        assert!(door.is_a_directory_at(Path::new("/run/alo")));
        assert_eq!(door.mode(), 0o755);
        assert_eq!(door.owner(), "root");
        assert_eq!(door.group(), "root");
    }

    /// **The mode is read as octal**, which is the only way it is ever written
    /// and the only way it means what somebody meant: `0700` read as seven
    /// hundred is a directory nobody could open.
    #[test]
    fn a_mode_is_octal() {
        let made = everything_made("d /var/lib/alo 0700 alo alo -\n").unwrap();
        assert_eq!(made.first().unwrap().mode(), 0o700);
    }

    /// **A mode that is not octal is refused**, and `-` is the one that really
    /// happens: systemd reads it as *whatever the umask leaves*, and the mode of
    /// the directory every person's door goes in is a decision.
    #[test]
    fn a_mode_left_to_the_umask_is_refused() {
        let refused = everything_made("d /run/alo - root root -\n").unwrap_err();
        assert!(
            matches!(&refused, NotMade::NotAMode { at: 1, mode } if mode == "-"),
            "{refused}"
        );
    }

    /// **A line with no group on it is refused**, rather than read as a
    /// directory belonging to nobody in particular.
    #[test]
    fn a_line_missing_its_group_is_refused() {
        let refused = everything_made("d /run/alo 0755 root\n").unwrap_err();
        assert!(
            matches!(refused, NotMade::TooFewFields { at: 1, .. }),
            "{refused}"
        );
    }

    /// A line that is not about the path being asked about is not about it,
    /// whatever else it makes.
    #[test]
    fn a_line_about_somewhere_else_is_about_somewhere_else() {
        let made = everything_made(BOTH).unwrap();
        assert!(
            !made
                .iter()
                .any(|it| it.is_a_directory_at(Path::new("/run/alo/1000"))),
            "the per-person directory is the daemon's, not the image's"
        );
    }

    /// A file with nothing but comments makes nothing, and that is an answer
    /// rather than a refusal — a check above it is what notices.
    #[test]
    fn a_file_of_comments_makes_nothing() {
        assert!(everything_made("# nothing here\n\n").unwrap().is_empty());
    }
}
