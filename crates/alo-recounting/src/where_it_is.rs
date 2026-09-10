//! Where this machine keeps the record, read off what the machine says about
//! itself.
//!
//! Until this file the only way to an account was for a caller to already know
//! the path, and on a running machine nothing did: the record lives where
//! `docs/contracts/machine-description.md` says it does, which is
//! `[record].path` in [`THE_DESCRIPTION`], and nothing on the person's side of
//! the machine had ever opened that. So an account could be read in a test and
//! nowhere else, and *what did my machine do today* had no answer on the
//! machine it was about.
//!
//! # This is a third reader of that file, and it is one on purpose
//!
//! `alo-agentd`'s own `describing.rs` decides whether a description is
//! **believed** — its owner, its mode, the format number, every value, and the
//! refusals — and `alo-image`'s `description.rs` reads the same file for what
//! the *image* is answerable for. This is neither. It reads one key, for the
//! person's side of the machine, and it exists because neither of the others
//! can be reached from here: the daemon is a Unix socket and the credentials a
//! kernel keeps for one, so on any host that is not Linux it compiles to
//! nothing at all, and a surface that linked it would be a surface linking the
//! privileged service in order to ask where a file is.
//!
//! # It is answerable for one key, so it refuses one thing and ignores the rest
//!
//! **The format number is refused when it is one this alo OS does not know.**
//! `[record].path` is a path today and this reader would go on believing it
//! were one whatever a later shape meant by it, which is the mistake
//! `docs/contracts/record-file.md` names about a record from a newer alo OS,
//! made about the file that says what a machine is.
//!
//! **A key this reader has never heard of is ignored**, and here that is right
//! where `deny_unknown_fields` is right for the other two. They are answerable
//! for the whole description; this is answerable for where the record is. A
//! reader that refused to say where the record is because the description had
//! grown a section about something else would take away a person's account of
//! their own machine over a key that has nothing to do with it.
//!
//! # Nothing here believes the description, and that is deliberate
//!
//! [`THE_DESCRIPTION`] is root's, in `/etc`, written by whoever installs or
//! manages the machine, and the daemon believes it before it will serve
//! anything. What must be believed before it is read is **the record**, which is
//! `alo_keeping::Reading::believed_at` and is the point at which somebody is
//! about to be shown what a file says happened on their machine. A description
//! only names a path; a machine whose `/etc` somebody else can write has lost
//! this argument several steps earlier.

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Where `alo-agentd` reads its description from, and where this looks for the
/// one key it is answerable for.
///
/// Part of `docs/contracts/machine-description.md`. It is spelled here rather
/// than asked of the daemon for this module's reason: the daemon is Linux and
/// this is read on the person's side.
pub const THE_DESCRIPTION: &str = "/etc/alo/agentd.toml";

/// The shape of description this reader knows.
const THE_FORMAT: u32 = 2;

/// The older shape, which says the same thing about where the record is.
const ALSO_READ: [u32; 1] = [1];

/// What the machine description says, as far as this crate is answerable for
/// it.
///
/// No `deny_unknown_fields`: see this module's documentation. Every other
/// section of that file belongs to somebody else and is not read here.
#[derive(Debug, Deserialize)]
struct Description {
    /// Which shape this description is in.
    format: u32,
    /// Where what happened is written down.
    record: TheRecord,
}

/// The `[record]` section, of which only the path is read here — how long a
/// record is kept is the daemon's and the organisation's (ADR 0004).
#[derive(Debug, Deserialize)]
struct TheRecord {
    /// The file `alo-keeping` appends to.
    path: PathBuf,
}

/// Why this machine could not say where it keeps its record.
///
/// Never *there is nothing to show*: a machine that cannot say where its record
/// is has not told anybody that nothing happened. Each is a sentence in
/// [`crate::words`], because a person asks this question about their own
/// machine and reads the answer in their own language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotSaid {
    /// There is no description at that path: this is not a machine that has
    /// been stood up as an alo OS machine.
    NoDescription {
        /// Where it was looked for.
        path: String,
    },
    /// What is there does not say what this machine is — including a
    /// description in a shape this alo OS does not know.
    NotADescription {
        /// What was named.
        path: String,
    },
    /// The machine would not read it.
    NotRead {
        /// What was named.
        path: String,
        /// What the machine said.
        why: String,
    },
}

/// Where this machine keeps the record, according to what it says about itself.
///
/// # Errors
///
/// [`NotSaid`], and every one of them is a machine that will not answer rather
/// than a machine with nothing to say. See this module's documentation for what
/// is refused and what is ignored.
pub fn where_the_record_is(description: &Path) -> Result<PathBuf, NotSaid> {
    let text = match std::fs::read_to_string(description) {
        Ok(text) => text,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
            return Err(NotSaid::NoDescription {
                path: description.display().to_string(),
            });
        }
        Err(why) => {
            return Err(NotSaid::NotRead {
                path: description.display().to_string(),
                why: why.to_string(),
            });
        }
    };
    let described: Description = toml::from_str(&text).map_err(|_| NotSaid::NotADescription {
        path: description.display().to_string(),
    })?;
    if described.format != THE_FORMAT && !ALSO_READ.contains(&described.format) {
        return Err(NotSaid::NotADescription {
            path: description.display().to_string(),
        });
    }
    Ok(described.record.path)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_description_at, somewhere_of_our_own};

    /// **Where the record is, read off what the machine says about itself.**
    /// This is the sentence that makes an account reachable on a running
    /// machine rather than only in a test that already knew the path.
    #[test]
    fn a_machine_says_where_it_keeps_its_record() {
        let at = a_description_at(
            &somewhere_of_our_own("described"),
            2,
            "/var/lib/alo/record.jsonl",
        );
        assert_eq!(
            where_the_record_is(&at),
            Ok(PathBuf::from("/var/lib/alo/record.jsonl"))
        );
    }

    /// The older shape says the same thing about where the record is, so a
    /// machine described before `[questions]` existed still answers.
    #[test]
    fn the_older_shape_says_where_the_record_is_too() {
        let at = a_description_at(&somewhere_of_our_own("older"), 1, "/var/lib/alo/record");
        assert_eq!(
            where_the_record_is(&at),
            Ok(PathBuf::from("/var/lib/alo/record"))
        );
    }

    /// **A section this reader has never heard of is ignored**, because it is
    /// answerable for one key. A reader that refused here would take away a
    /// person's account of their own machine over a policy about something
    /// else.
    #[test]
    fn a_section_this_reader_does_not_know_is_not_a_reason_to_refuse() {
        let at = somewhere_of_our_own("grown");
        std::fs::write(
            &at,
            "format = 2\n[record]\npath = \"/var/lib/alo/record\"\nkeeping = \"forever\"\n\
             [questions]\nanywhere = false\n[something-later]\nkey = 1\n",
        )
        .unwrap();
        assert_eq!(
            where_the_record_is(&at),
            Ok(PathBuf::from("/var/lib/alo/record"))
        );
    }

    /// **A shape this alo OS does not know is refused rather than guessed at.**
    /// What `path` means is fixed by the shape the description is in, and a
    /// reader that took it anyway would be reading a file by guessing.
    #[test]
    fn a_description_from_a_newer_alo_os_is_refused() {
        let at = a_description_at(&somewhere_of_our_own("newer"), 9, "/var/lib/alo/record");
        assert!(matches!(
            where_the_record_is(&at),
            Err(NotSaid::NotADescription { .. })
        ));
    }

    /// A machine with no description at all is told apart from every other
    /// failure: it is not an alo OS machine, rather than a broken one.
    #[test]
    fn a_machine_that_says_nothing_about_itself_is_told_apart() {
        let at = somewhere_of_our_own("none");
        assert!(matches!(
            where_the_record_is(&at),
            Err(NotSaid::NoDescription { .. })
        ));
    }

    /// Anything else at that path is not a description, and nothing is read out
    /// of it — including a description with no `[record]` in it at all, which
    /// is a machine that has not said the one thing this reader came for.
    #[test]
    fn nothing_that_is_not_a_description_is_read_as_one() {
        for written in [
            "notes about the invoice\n",
            "format = 2\n",
            "format = 2\n[record]\nkeeping = \"forever\"\n",
        ] {
            let at = somewhere_of_our_own("not-a-description");
            std::fs::write(&at, written).unwrap();
            assert!(
                matches!(
                    where_the_record_is(&at),
                    Err(NotSaid::NotADescription { .. })
                ),
                "{written:?} was read as a description"
            );
        }
    }
}
