//! Why the machine could not do something it was allowed to do.
//!
//! **This is not a refusal, and keeping the two apart is the point of the
//! file.** [`alo_capability::Refused`] means the capability model said no: no
//! grant covered it, nobody approved it, the grants changed under it. A
//! [`Failed`] means everything said yes and the disk could not — the file was
//! gone, the folder was a file, the machine denied it, there was already
//! something at that name.
//!
//! A record that called a full disk a refusal would tell a security review that
//! the grants stopped something they did not. So the two are different types,
//! they leave [`crate::Did`] by different doors, and only one of them is
//! evidence about the capability model.
//!
//! Every message says what to do next, because every one of them is read by
//! somebody holding a call that did not happen. "IO error 13" tells that person
//! nothing they can act on.

/// Why the machine could not do it.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum Failed {
    /// A verb that is not one of the six.
    #[error(
        "nothing in the file half does {verb} — it lists, reads, finds, renames, moves and archives, and anything else is somewhere else's to do"
    )]
    NotAFileVerb {
        /// The verb that was asked for.
        verb: String,
    },

    /// A verb was performed without an argument it declares.
    ///
    /// Unreachable for a call that came through [`alo_capability::Verbs`],
    /// which refuses a call missing an argument — and still a `Result` rather
    /// than an `unwrap`, because a library that panics on a verb somebody
    /// declared wrongly is a library that takes the daemon with it.
    #[error(
        "{verb} was performed without {argument}, which it declares — this is a verb to write again, not a call to make again"
    )]
    Missing {
        /// The verb.
        verb: String,
        /// The argument it declared and did not get.
        argument: String,
    },

    /// Something is there, and it is not a folder.
    #[error("{path} is not a folder — read it if it is a file, or list the folder it is in")]
    NotAFolder {
        /// What was named.
        path: String,
    },

    /// Something is there, and it is not a file.
    #[error("{path} is not a file — list it if it is a folder")]
    NotAFile {
        /// What was named.
        path: String,
    },

    /// It was there when it was checked, and it is not there now.
    #[error(
        "{path} was there when it was checked and is not there now — nothing was done, so ask again"
    )]
    Gone {
        /// What was named.
        path: String,
    },

    /// A file too large to answer with.
    #[error(
        "{path} holds {bytes} bytes and a verb reads at most {most} — open it in an application, or ask for part of the folder instead"
    )]
    TooBig {
        /// What was named.
        path: String,
        /// How big it is.
        bytes: u64,
        /// The most a read answers with.
        most: u64,
    },

    /// A file that is not text.
    #[error(
        "{path} is not text — this verb answers with what a person could read, and reading this one needs an application that knows what it is"
    )]
    NotText {
        /// What was named.
        path: String,
    },

    /// Something is already at the name a change would create.
    #[error(
        "there is already something at {path} — nothing here replaces a file that was not named, so choose another name"
    )]
    AlreadyThere {
        /// Where the change would have put something.
        path: String,
    },

    /// A file asked to move into the folder it is already in.
    #[error("{path} is already in that folder — nothing was moved")]
    AlreadyIn {
        /// The file.
        path: String,
    },

    /// An archive asked to be written inside the folder it is an archive of.
    #[error(
        "the archive would be written inside {folder}, which is the folder it is an archive of — put it somewhere that is not inside it"
    )]
    IntoItself {
        /// The folder being archived.
        folder: String,
    },

    /// An archive asked for under a name that does not say what it is.
    #[error(
        "call the archive something ending in .zip — alo OS makes zip archives, and a name saying otherwise would be a file whose name lies about what is in it"
    )]
    NotAZipName {
        /// The name that was asked for.
        name: String,
    },

    /// More in a folder than one archive holds.
    #[error(
        "{folder} holds more than {most} things — archive one of the folders inside it instead, so that what is in the archive is something a person can still recognise"
    )]
    TooMany {
        /// The folder.
        folder: String,
        /// The most one archive holds.
        most: usize,
    },

    /// More bytes than one archive holds.
    #[error(
        "{folder} holds more than {most} bytes and one archive holds at most that — archive one of the folders inside it instead"
    )]
    TooMuch {
        /// The folder.
        folder: String,
        /// The most one archive holds.
        most: u64,
    },

    /// The machine said no, in its own words.
    #[error("{path} could not be {doing} ({why}) — nothing else was attempted")]
    TheMachineSaidNo {
        /// What was being touched.
        path: String,
        /// What was being done to it, in one word.
        doing: String,
        /// What the machine said.
        why: String,
    },
}

impl Failed {
    /// The machine said no while doing this to this path.
    ///
    /// One place turns an [`std::io::Error`] into words, so that *it went away*
    /// is always the same answer wherever it happened, rather than one answer
    /// per call site.
    pub(crate) fn machine(path: &std::path::Path, doing: &str, why: &std::io::Error) -> Self {
        if why.kind() == std::io::ErrorKind::NotFound {
            return Self::Gone {
                path: path.display().to_string(),
            };
        }
        Self::TheMachineSaidNo {
            path: path.display().to_string(),
            doing: doing.to_owned(),
            why: why.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    /// A file that went away is answered as having gone away, whatever it was
    /// being asked to do at the time. The alternative is three messages for one
    /// fact, differing only in which verb was unlucky.
    #[test]
    fn a_file_that_went_away_says_so_rather_than_reporting_an_error_number() {
        let gone = Failed::machine(
            Path::new("/home/anna/Invoices/march.pdf"),
            "read",
            &Error::new(ErrorKind::NotFound, "no such file or directory"),
        );
        assert!(
            matches!(gone, Failed::Gone { .. }),
            "{gone}: a missing file is not a machine refusal"
        );
        assert!(gone.to_string().contains("ask again"), "{gone}");
    }

    /// Anything else keeps the machine's own words, because they are the only
    /// thing that says whether this is a full disk, a read-only mount or a
    /// permission a person can change.
    #[test]
    fn anything_else_keeps_what_the_machine_said() {
        let denied = Failed::machine(
            Path::new("/home/anna/Invoices"),
            "listed",
            &Error::new(ErrorKind::PermissionDenied, "permission denied"),
        );
        assert!(denied.to_string().contains("permission denied"), "{denied}");
        assert!(denied.to_string().contains("listed"), "{denied}");
        assert!(
            denied.to_string().contains("nothing else was attempted"),
            "{denied}"
        );
    }

    /// Every message says what to do about it. A refusal a person cannot act on
    /// is a refusal they will ask somebody else about.
    #[test]
    fn every_failure_says_what_to_do_next() {
        let messages = [
            Failed::NotAFileVerb {
                verb: "open_application".to_owned(),
            }
            .to_string(),
            Failed::NotAFolder {
                path: "/home/anna/Invoices/march.pdf".to_owned(),
            }
            .to_string(),
            Failed::NotAFile {
                path: "/home/anna/Invoices".to_owned(),
            }
            .to_string(),
            Failed::TooBig {
                path: "/home/anna/Invoices/scan.tiff".to_owned(),
                bytes: 200_000_000,
                most: 1_048_576,
            }
            .to_string(),
            Failed::NotText {
                path: "/home/anna/Invoices/scan.tiff".to_owned(),
            }
            .to_string(),
            Failed::AlreadyThere {
                path: "/home/anna/Archive/march.pdf".to_owned(),
            }
            .to_string(),
            Failed::NotAZipName {
                name: "invoices".to_owned(),
            }
            .to_string(),
        ];
        for message in messages {
            assert!(
                message.contains(" — "),
                "{message}: a failure that does not say what to do is half a message"
            );
        }
    }
}
