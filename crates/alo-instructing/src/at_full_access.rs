//! **Whether the person chose full access**, as one line between their shell
//! and the agent
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! Measured on a booted machine on 2026-11-11 (the walk of `57b98c15`): the
//! person chose **full access** on the first-start sheet, offered Dolphin
//! showing their home with *Help with this window*, and asked *what folders are
//! in this window*. alo answered *nothing has been granted, a person grants a
//! folder by picking it in Settings* — and asked the machine nothing. The
//! service would have listed the folder: since #1134 a read at full access
//! reaches whatever the person's account can. **The model had been told the
//! opposite.** The shell sends the folders granted, there are none at full
//! access, and an empty list is told as *every verb that reads a file will be
//! refused* (`crate::what_it_may_reach`).
//!
//! ```text
//! alo-full-access:yes
//! ```
//!
//! It is the sixth line on the asking socket, after where the person's folders
//! are (`docs/contracts/asking-socket.md`), for the reason every line is after
//! the last one an older agent reads. A current shell always sends it — the
//! marker alone at every other level — so a current agent never waits for it.
//!
//! # Why the shell says it
//!
//! The level is in `/var/lib/alo/how-code-runs.toml`, `0700 alo alo`, which the
//! agent's own user cannot read — the separation ADR 0109 rests on, and not
//! worth weakening for a hint. The person's shell wrote it, reads it already to
//! draw Settings, and is the side the choice was made on, exactly as it is for
//! the folders granted (`alo-shell`'s `what_the_agent_may_reach`).
//!
//! # A hint, never a capability
//!
//! **Saying full access allows nothing.** `alo-agentd` reads the level itself,
//! at the moment of each read, proposal and OK, and a line claiming full access
//! that the file does not hold sends the model to a path the daemon refuses: a
//! round spent, and nothing reached.

/// The marker the line begins with.
pub const AT_FULL_ACCESS: &str = "alo-full-access:";

/// What follows the marker when the person chose full access.
pub const YES: &str = "yes";

/// The most a line may be before it is read at all, in bytes.
pub const THE_LONGEST_FULL_ACCESS_LINE: usize = AT_FULL_ACCESS.len() + YES.len() + 2;

/// Why a line could not be read as whether the person chose full access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotTheLevel {
    /// The line does not begin with [`AT_FULL_ACCESS`].
    NotMarked,
    /// Something after the marker other than [`YES`].
    NotSaid(String),
}

impl std::fmt::Display for NotTheLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotMarked => write!(
                f,
                "the line after the person's folders is not whether they chose full access"
            ),
            Self::NotSaid(what) => write!(
                f,
                "whether the person chose full access was sent as {what:?}, which a shell does \
                 not write"
            ),
        }
    }
}

impl std::error::Error for NotTheLevel {}

/// **The line a shell writes**, without its newline: [`YES`] after the marker
/// at full access, and the marker alone at every other level.
#[must_use]
pub fn the_line_for(at_full_access: bool) -> String {
    if at_full_access {
        format!("{AT_FULL_ACCESS}{YES}")
    } else {
        AT_FULL_ACCESS.to_owned()
    }
}

/// **The line read back**, with or without its newline.
///
/// # Errors
/// [`NotTheLevel`], for a line a current shell would not write: another
/// marker, or anything after it but [`YES`]. **Refused rather than read as
/// no**, as every other line is — a shell that wrote something else is one
/// this agent does not understand.
pub fn read_the_line(line: &str) -> Result<bool, NotTheLevel> {
    let said = line
        .trim_end_matches(['\n', '\r'])
        .strip_prefix(AT_FULL_ACCESS)
        .ok_or(NotTheLevel::NotMarked)?;
    match said {
        "" => Ok(false),
        YES => Ok(true),
        other => Err(NotTheLevel::NotSaid(other.chars().take(64).collect())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What a shell writes is what an agent reads, at both answers.
    #[test]
    fn both_answers_are_read_back_as_written() {
        for at in [true, false] {
            assert_eq!(read_the_line(&the_line_for(at)), Ok(at));
            assert_eq!(read_the_line(&format!("{}\n", the_line_for(at))), Ok(at));
        }
    }

    /// The marker alone is every level but full access.
    #[test]
    fn the_marker_alone_is_not_full_access() {
        assert_eq!(the_line_for(false), "alo-full-access:");
        assert_eq!(read_the_line("alo-full-access:\n"), Ok(false));
    }

    /// **Refused, never guessed at**: a word a shell does not write is not
    /// read as yes, and not read as no either.
    #[test]
    fn anything_else_is_refused_rather_than_read_as_an_answer() {
        for line in [
            "alo-full-access:true",
            "alo-full-access:YES",
            "alo-full-access: yes",
            "alo-full-access:no",
        ] {
            assert!(
                matches!(read_the_line(line), Err(NotTheLevel::NotSaid(_))),
                "{line}"
            );
        }
        assert_eq!(
            read_the_line("alo-their-folders:"),
            Err(NotTheLevel::NotMarked)
        );
    }

    /// The bound holds the longest line a shell writes, with its newline.
    #[test]
    fn the_bound_holds_what_a_shell_writes() {
        assert!(the_line_for(true).len() < THE_LONGEST_FULL_ACCESS_LINE);
    }
}
