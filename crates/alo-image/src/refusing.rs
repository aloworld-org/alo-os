//! Why a file the image ships would not read at all.
//!
//! `crate::wrong` is the other half and is a different question: these are files
//! that cannot be understood, and a [`Wrong`](crate::Wrong) is files that are
//! understood and disagree with each other.
//!
//! # Every sentence here is English, and stays English
//!
//! `CLAUDE.md` says hardcoded English is a bug, and this crate is the exception
//! `alo-shortcuts`' `DefaultsError` already is: nothing here is ever read by
//! somebody using alo OS. A `sysusers.d` line with no number on it is read by
//! whoever is editing the image, in a build that failed, beside a diff — and
//! `alo-agentd` is what says something to a person, in their own language, on a
//! machine that booted. A translated message here would be a translation nobody
//! can use, checked by a test nobody can read.
//!
//! Every one of them names **where**: the file, and the line inside it. That is
//! the whole of what somebody needs, and it is why none of these carry a
//! sentence about what should have been there instead.

use std::path::PathBuf;

use thiserror::Error;

/// Why a systemd unit file would not read.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NotAUnit {
    /// A line this reader will not join to the next one.
    #[error(
        "line {at} is continued onto the next one, and this reader does not join them — write \
         the value on one line"
    )]
    Continued {
        /// Which line it is.
        at: usize,
    },
    /// A `key=value` before any `[Section]` header.
    #[error(
        "line {at} assigns `{key}` before any section — put it under the section it belongs to"
    )]
    NoSection {
        /// Which line it is.
        at: usize,
        /// The key that was assigned.
        key: String,
    },
    /// A line that is neither a section header nor an assignment.
    #[error("line {at} is neither a section nor a `key=value`: {line}")]
    NotAnAssignment {
        /// Which line it is.
        at: usize,
        /// What was written on it.
        line: String,
    },
}

/// Why a unit file is not a service alo OS could be checked against.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NotAService {
    /// A unit with no `[Service]` section at all.
    #[error("{called} has no [Service] section, so systemd would not start anything from it")]
    NoServiceSection {
        /// The unit file.
        called: String,
    },
    /// A service with no `ExecStart`.
    #[error("{called} has a [Service] section with no ExecStart — say what it runs")]
    NothingToStart {
        /// The unit file.
        called: String,
    },
}

/// Why a `tmpfiles.d` file would not read.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NotMade {
    /// A line with fewer than the five fields that decide anything.
    #[error(
        "line {at} has fewer than five fields — a type, a path, a mode, a user and a group: {line}"
    )]
    TooFewFields {
        /// Which line it is.
        at: usize,
        /// What was written on it.
        line: String,
    },
    /// A mode that is not an octal number, `-` included.
    #[error(
        "line {at} has `{mode}` where the mode belongs — write it in octal, because a mode left \
         to the umask is a directory nobody decided the permissions of"
    )]
    NotAMode {
        /// Which line it is.
        at: usize,
        /// What was written where the mode belongs.
        mode: String,
    },
}

/// Why a `sysusers.d` file would not read.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NotDeclared {
    /// A line with fewer than the three fields every kind of line has.
    #[error(
        "line {at} has fewer than three fields — a type, a name and a number or a group: {line}"
    )]
    TooFewFields {
        /// Which line it is.
        at: usize,
        /// What was written on it.
        line: String,
    },
    /// An identifier that is not a number, `-` included.
    #[error(
        "line {at} has `{written}` where a number belongs — a login whose number the machine \
         picks is a login the machine description cannot name"
    )]
    NotANumber {
        /// Which line it is.
        at: usize,
        /// What was written where a number belongs.
        written: String,
    },
    /// A type letter this reader does not know.
    #[error("line {at} is a `{kind}` line, which this reader does not know — it reads g, u and m")]
    UnknownKind {
        /// Which line it is.
        at: usize,
        /// The type letter.
        kind: String,
    },
}

/// Why the machine description the image ships would not read.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NotDescribed {
    /// A description written for an alo OS this is not.
    #[error(
        "the description says it is format {format}, and this alo OS reads format 1 — a newer \
         one is refused rather than read the parts of that happen to fit"
    )]
    AnotherFormat {
        /// What the file said.
        format: u32,
    },
    /// A missing key, a key nobody declared, or a value of the wrong kind.
    #[error(
        "the description is not the shape docs/contracts/machine-description.md describes: {why}"
    )]
    NotTheShape {
        /// What the reader said about it.
        why: String,
    },
}

/// Why a directory is not an image this crate can check.
#[derive(Debug, Error)]
pub enum NotAnImage {
    /// A file the image has to ship is not there, or would not open.
    #[error("{at} could not be read: {why}")]
    Unreadable {
        /// The file.
        at: PathBuf,
        /// What the machine said.
        why: std::io::Error,
    },
    /// A unit file that would not read.
    #[error("{at}: {why}")]
    NotAUnit {
        /// The file.
        at: PathBuf,
        /// Why.
        why: NotAUnit,
    },
    /// A unit that is not a service.
    #[error("{why}")]
    NotAService {
        /// Why.
        why: NotAService,
    },
    /// A `tmpfiles.d` file that would not read.
    #[error("{at}: {why}")]
    NotMade {
        /// The file.
        at: PathBuf,
        /// Why.
        why: NotMade,
    },
    /// A `sysusers.d` file that would not read.
    #[error("{at}: {why}")]
    NotDeclared {
        /// The file.
        at: PathBuf,
        /// Why.
        why: NotDeclared,
    },
    /// A machine description that would not read.
    #[error("{at}: {why}")]
    NotDescribed {
        /// The file.
        at: PathBuf,
        /// Why.
        why: NotDescribed,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every refusal about a file names the line inside it, because that is
    /// what whoever is editing the image is looking for.
    #[test]
    fn a_refusal_about_a_line_says_which_line() {
        assert!(NotAUnit::Continued { at: 12 }.to_string().contains("12"));
        assert!(
            NotMade::NotAMode {
                at: 3,
                mode: "-".to_owned()
            }
            .to_string()
            .contains("line 3")
        );
        assert!(
            NotDeclared::UnknownKind {
                at: 7,
                kind: "r".to_owned()
            }
            .to_string()
            .contains("line 7")
        );
    }

    /// And a refusal about a unit names the unit, because a build that failed
    /// has two of them in it.
    #[test]
    fn a_refusal_about_a_unit_says_which_unit() {
        assert!(
            NotAService::NothingToStart {
                called: "alo-agentd.service".to_owned()
            }
            .to_string()
            .contains("alo-agentd.service")
        );
    }

    /// A description written for a later alo OS says so as a version rather
    /// than as whichever of its keys this reader happened not to know.
    #[test]
    fn a_description_from_the_future_says_that_is_what_it_is() {
        let said = NotDescribed::AnotherFormat { format: 4 }.to_string();
        assert!(said.contains('4'), "{said}");
        assert!(said.contains("format 1"), "{said}");
    }
}
