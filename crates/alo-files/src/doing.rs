//! The one door from *this may touch the disk* to *this is what happened*.
//!
//! [`crate::Touching`] is the end of everything the capability model can decide
//! without a filesystem: validated, permitted, approved if it changes anything,
//! resolved, and asked about again where it really leads. [`Did`] is what
//! happens next, and it is the only thing in this crate that acts.
//!
//! # A change is asked about once more, for what it would create
//!
//! Every path a call *names* has been asked about twice already. The paths a
//! call **creates** have not been asked about at all: `rename_file` invents a
//! name, `move_file` and `archive_folder` invent a full path inside a folder.
//! A grant can be over one file (ADR 0001 §4 — the document offered at
//! invocation), and under one of those, renaming would put a file at a name
//! nobody granted.
//!
//! So the grants are asked one last question here — *may this be created?* — at
//! the moment the authorisation was made, and a no is an
//! [`alo_capability::Refused`] like every other, in the grants' own words, into
//! the record beside every other refusal. **A grant covers where a file goes,
//! not only where it comes from.**
//!
//! # Two ways of not happening, and they are different facts
//!
//! [`Did::of`] answers `Err(Refused)` when the capability model said no, and a
//! [`Did`] carrying a [`Failed`] when everything said yes and the machine could
//! not. A record that flattened the two would tell a security review that the
//! grants stopped a full disk. The authorisation comes back either way, because
//! either way something is written down.

use std::path::{Path, PathBuf};

use alo_capability::{Ask, Authorised, Grants, Refused, Value};
use alo_strings::{Filling, Strings};

use crate::answer::Answer;
use crate::archiving::{archive, is_an_archive_name};
use crate::changing::{move_into, rename};
use crate::failed::Failed;
use crate::looking::{find, list, read};
use crate::real::Real;
use crate::touching::Touching;
use crate::words;

/// What happened when the machine was asked to do it.
///
/// Not `Clone`, like everything else on this journey: a thing that happened is
/// not a thing that can happen again.
#[derive(Debug)]
pub struct Did {
    /// What ran, and the authority it ran under.
    authorised: Authorised,
    /// What it answered with, or why the machine could not.
    outcome: Result<Answer, Failed>,
}

impl Did {
    /// Do it.
    ///
    /// The grants are asked once more about anything this would create, at the
    /// moment the call was authorised — the same moment [`Touching`] asked
    /// about the paths the call named, because two moments would be two answers
    /// that could disagree.
    ///
    /// The strings are for the refusal, and for nothing else: what a person is
    /// told when the grants do not cover a path this would create is worded
    /// here, and `alo_capability::Refused` carries words. [`crate::touching`]
    /// says why that is one rendering rather than two.
    ///
    /// # Errors
    /// [`Refused`], carrying the call, when the grants do not cover something
    /// the call would create. Nothing has been touched when that happens.
    pub fn of(touching: Touching, grants: &Grants, strings: &Strings) -> Result<Self, Refused> {
        let outcome = match Todo::of(&touching) {
            Err(failed) => Err(failed),
            Ok(todo) => {
                if let Some(creating) = todo.creating() {
                    may_create(&touching, grants, creating, strings)?;
                }
                todo.done()
            }
        };
        Ok(Self {
            authorised: touching.into_authorised(),
            outcome,
        })
    }

    /// What ran, and the authority it ran under — what the record is written
    /// from.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// What it answered with, when the machine did it.
    #[must_use]
    pub fn answer(&self) -> Option<&Answer> {
        self.outcome.as_ref().ok()
    }

    /// Why the machine could not, when it could not.
    #[must_use]
    pub fn failure(&self) -> Option<&Failed> {
        self.outcome.as_ref().err()
    }

    /// The authority and the outcome, taken.
    ///
    /// The authorisation comes back whether or not the machine managed it,
    /// because a call that was permitted, approved and attempted is a thing
    /// that happened and is recorded as one. What the disk made of it is the
    /// answer to whoever asked, not evidence about the capability model.
    pub fn into_parts(self) -> (Authorised, Result<Answer, Failed>) {
        (self.authorised, self.outcome)
    }
}

/// One of the six, with everything it needs and nothing else.
///
/// Made once, before anything is touched, so that what a call *would create* is
/// known before the grants are asked about it and before a single byte is
/// written. Every verb's arguments are read here and nowhere else.
#[derive(Debug)]
enum Todo<'a> {
    /// List what is in a folder.
    List {
        /// The folder.
        folder: &'a Real,
    },
    /// Read what is in a file.
    Read {
        /// The file.
        file: &'a Real,
    },
    /// Find files in a folder by what they are called.
    Find {
        /// The folder to look in.
        folder: &'a Real,
        /// What a name has to contain.
        named: &'a str,
        /// How many to answer with at most.
        most: usize,
    },
    /// Give a file a different name, where it is.
    Rename {
        /// The file.
        file: &'a Real,
        /// What it would then be called, in full.
        to: PathBuf,
    },
    /// Move a file into a folder.
    Move {
        /// The file.
        file: &'a Real,
        /// The folder it goes into.
        into: &'a Real,
        /// Where it would then be, in full.
        to: PathBuf,
    },
    /// Make one archive file out of a folder.
    Archive {
        /// The folder to make an archive of.
        folder: &'a Real,
        /// The folder the archive goes into.
        into: &'a Real,
        /// Where the archive would be, in full.
        to: PathBuf,
    },
}

impl<'a> Todo<'a> {
    /// What this call comes down to.
    fn of(touching: &'a Touching) -> Result<Self, Failed> {
        let verb = touching.verb();
        match verb {
            "list_folder" => Ok(Self::List {
                folder: real(touching, "folder")?,
            }),
            "read_file" => Ok(Self::Read {
                file: real(touching, "file")?,
            }),
            "find_in_folder" => Ok(Self::Find {
                folder: real(touching, "folder")?,
                named: a_name(touching, "named")?,
                most: a_count(touching, "most")?,
            }),
            "rename_file" => {
                let file = real(touching, "file")?;
                let name = a_name(touching, "name")?;
                let held = file.as_path().parent().ok_or_else(|| Failed::NotAFile {
                    path: file.as_path().display().to_string(),
                })?;
                Ok(Self::Rename {
                    file,
                    to: held.join(name),
                })
            }
            "move_file" => {
                let file = real(touching, "file")?;
                let into = real(touching, "into")?;
                let called = file.as_path().file_name().ok_or_else(|| Failed::NotAFile {
                    path: file.as_path().display().to_string(),
                })?;
                Ok(Self::Move {
                    file,
                    into,
                    to: into.as_path().join(called),
                })
            }
            "archive_folder" => {
                let folder = real(touching, "folder")?;
                let into = real(touching, "into")?;
                let name = a_name(touching, "name")?;
                if !is_an_archive_name(name) {
                    return Err(Failed::NotAZipName {
                        name: name.to_owned(),
                    });
                }
                Ok(Self::Archive {
                    folder,
                    into,
                    to: into.as_path().join(name),
                })
            }
            _ => Err(Failed::NotAFileVerb {
                verb: verb.to_owned(),
            }),
        }
    }

    /// The path this would create, for the three that create one.
    ///
    /// A read creates nothing, and answering `None` for one is the whole
    /// difference: the grants are asked about what a change would leave behind,
    /// and a question about what a read would leave behind has no answer.
    fn creating(&self) -> Option<&Path> {
        match self {
            Self::List { .. } | Self::Read { .. } | Self::Find { .. } => None,
            Self::Rename { to, .. } | Self::Move { to, .. } | Self::Archive { to, .. } => Some(to),
        }
    }

    /// Do it.
    fn done(self) -> Result<Answer, Failed> {
        match self {
            Self::List { folder } => list(folder),
            Self::Read { file } => read(file),
            Self::Find {
                folder,
                named,
                most,
            } => find(folder, named, most),
            Self::Rename { file, to } => rename(file, &to),
            Self::Move { file, into, to } => move_into(file, into, &to),
            Self::Archive { folder, into, to } => archive(folder, into, &to),
        }
    }
}

/// Ask the grants about a path this call would create.
///
/// The moment is the authorisation's own, and the words of a refusal are the
/// grants' own, so that this refusal is the same kind of thing as every other
/// and reaches the record by the same road.
fn may_create(
    touching: &Touching,
    grants: &Grants,
    creating: &Path,
    strings: &Strings,
) -> Result<(), Refused> {
    let authorised = touching.authorised();
    let under = authorised.under();
    if let Err(why) = grants.permitting(under, &Ask::Path(creating.to_owned()), authorised.at()) {
        return Err(Refused::worded_elsewhere(
            touching.call().clone(),
            strings.say(
                &words::WOULD_CREATE.key(),
                &Filling::of("verb", authorised.verb())
                    .and("path", creating.display().to_string())
                    // The grants' own refusal, said here so that the sentence
                    // it sits inside and the sentence it is are one language.
                    .and("why", why.said(strings).into_text()),
            ),
        ));
    }
    Ok(())
}

/// Where this argument's path really leads.
fn real<'a>(touching: &'a Touching, argument: &str) -> Result<&'a Real, Failed> {
    touching.real(argument).ok_or_else(|| Failed::Missing {
        verb: touching.verb().to_owned(),
        argument: argument.to_owned(),
    })
}

/// The one name this argument carried.
fn a_name<'a>(touching: &'a Touching, argument: &str) -> Result<&'a str, Failed> {
    match touching.call().value(argument) {
        Some(Value::Name(name)) => Ok(name),
        _ => Err(Failed::Missing {
            verb: touching.verb().to_owned(),
            argument: argument.to_owned(),
        }),
    }
}

/// The count this argument carried.
///
/// A count outside what this machine counts with is treated as an argument that
/// did not arrive: the file verbs declare a range of 1 to 1000, so reaching this
/// means a verb was declared with a range no answer could have, which is a verb
/// to write again rather than a call to make again.
fn a_count(touching: &Touching, argument: &str) -> Result<usize, Failed> {
    match touching.call().value(argument) {
        Some(Value::Count(how_many)) => usize::try_from(*how_many).map_err(|_| Failed::Missing {
            verb: touching.verb().to_owned(),
            argument: argument.to_owned(),
        }),
        _ => Err(Failed::Missing {
            verb: touching.verb().to_owned(),
            argument: argument.to_owned(),
        }),
    }
}
