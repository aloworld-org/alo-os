//! Converting a document an agent was approved to convert — the verb, carried
//! out.
//!
//! In this order, and the first *no* is the answer:
//!
//! 1. **Is this converting?** An authorisation for any other verb is refused.
//! 2. **May the copy be created where it would go?** The grants have answered
//!    for the document and the folder already ([`alo_files::Touching`]); the
//!    copy's own path is asked about here, at the moment the call was
//!    authorised, and a no is an [`alo_capability::Refused`] like any other.
//! 3. **Can the document be read?** Opened read-only, following no link, and
//!    not read at all if it has other names.
//! 4. **Is it something this machine converts?** Decided from its bytes by
//!    `alo-opening`; a program named `.docx` never reaches the engine.
//! 5. **Is the copy's name free?** Created with `O_EXCL`: a file already there
//!    is a refusal, never an overwrite and never a quiet second name.
//! 6. **Did the service convert it, and say what the copy carried?** If not,
//!    the copy this created is removed, and only that copy.
//!
//! **Nothing here opens a path the grants were not asked about.** Every path
//! opened is one [`alo_files::Touching`] resolved or the copy's, asked about
//! in step 2.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use alo_capability::{Ask, Authorised, Grants, Refused};
use alo_files::Touching;
use alo_opening::{Cannot, Decided, Kind, Outcome, ThisMachine, decide};
use alo_strings::{Filling, Said, Strings};

use crate::carried::Carried;
use crate::conversion::Conversion;
use crate::opening::{Folder, NotOpened, original};
use crate::service::{ConvertingService, Unconverted};
use crate::verbs::{CONVERT_DOCUMENT, FILE, INTO};
use crate::wire::Refusal;
use crate::words::{self, Word};

/// A conversion that was carried out, whatever came of it.
///
/// Not `Clone`, like the [`Authorised`] inside it: a thing that ran is not a
/// thing that can run again.
#[derive(Debug)]
pub struct Done {
    /// What ran, and the authority it ran under.
    authorised: Authorised,
    /// The copy, or why there is none.
    outcome: Result<Converted, NotConverted>,
}

/// A copy, made, and what it carried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Converted {
    /// The document that was converted, as the grants resolved it.
    file: PathBuf,
    /// The copy.
    copy: PathBuf,
    /// What it carried.
    carried: Carried,
}

/// Why no copy was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotConverted {
    /// What was approved was not converting a document.
    NotConverting,
    /// The document is a kind this machine does not convert.
    NotAKindThisConverts(Kind),
    /// The document cannot be opened at all.
    CannotBeOpened(Cannot),
    /// The document's name says it is something it is not.
    NotWhatItsNameSays(Decided),
    /// The document could not be read.
    Unreadable,
    /// The document has other names on this machine, so it was not read.
    MoreThanOneName,
    /// A file with the copy's name is already in the folder.
    NameTaken {
        /// The name.
        copy: String,
    },
    /// The copy could not be created in the folder.
    CopyNotCreated,
    /// Nothing on this machine that converts answered.
    NothingHereConverts,
    /// The converting service refused.
    Refused(Refusal),
}

impl Done {
    /// What ran, and the authority it ran under — what the record is written
    /// from.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// The copy and what it carried, when there is one.
    #[must_use]
    pub fn converted(&self) -> Option<&Converted> {
        self.outcome.as_ref().ok()
    }

    /// Why there is no copy, when there is none.
    #[must_use]
    pub fn not_converted(&self) -> Option<&NotConverted> {
        self.outcome.as_ref().err()
    }

    /// What a person reads, in order.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        match &self.outcome {
            Ok(converted) => converted.said(strings),
            Err(not) => not.said(strings),
        }
    }

    /// The authority and the outcome, taken.
    pub fn into_parts(self) -> (Authorised, Result<Converted, NotConverted>) {
        (self.authorised, self.outcome)
    }
}

impl Converted {
    /// The document that was converted.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// The copy.
    #[must_use]
    pub fn copy(&self) -> &Path {
        &self.copy
    }

    /// What it carried.
    #[must_use]
    pub fn carried(&self) -> &Carried {
        &self.carried
    }

    /// What a person reads: the copy is made and the original is unchanged,
    /// then what the copy carried.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        let mut said =
            vec![
                strings.say(
                    &words::CONVERTED.key(),
                    &Filling::of(words::COPY, name_of(&self.copy))
                        .and(words::FILE, name_of(&self.file)),
                ),
            ];
        said.extend(self.carried.said(strings));
        said
    }
}

impl NotConverted {
    /// What a person reads, in order: what `alo-opening` says about the file
    /// where it has something to say, then why nothing was converted.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        match self {
            Self::NotConverting => vec![plainly(strings, words::NOT_CONVERTING)],
            Self::NotAKindThisConverts(kind) => vec![strings.say(
                &words::NOT_A_KIND_THIS_CONVERTS.key(),
                &Filling::nothing().and_said(words::WHAT, &kind.said(strings)),
            )],
            Self::CannotBeOpened(cannot) => {
                let mut said = Outcome::CannotOpen(*cannot).said(strings);
                said.push(plainly(strings, words::NOTHING_WAS_CONVERTED));
                said
            }
            Self::NotWhatItsNameSays(decided) => {
                let mut said = decided.said(strings);
                said.truncate(1);
                said.push(plainly(strings, words::NOTHING_WAS_CONVERTED));
                said
            }
            Self::Unreadable => vec![plainly(strings, words::UNREADABLE)],
            Self::MoreThanOneName => vec![plainly(strings, words::MORE_THAN_ONE_NAME)],
            Self::NameTaken { copy } => vec![strings.say(
                &words::NAME_TAKEN.key(),
                &Filling::of(words::COPY, copy.clone()),
            )],
            Self::CopyNotCreated => vec![plainly(strings, words::COPY_NOT_CREATED)],
            Self::NothingHereConverts | Self::Refused(Refusal::NotUnderstood) => {
                vec![plainly(strings, words::NOTHING_HERE_CONVERTS)]
            }
            Self::Refused(refusal) => vec![plainly(strings, refused(*refusal))],
        }
    }
}

/// The sentence for a refusal the service gave.
const fn refused(refusal: Refusal) -> Word {
    match refusal {
        Refusal::NotUnderstood => words::NOTHING_HERE_CONVERTS,
        Refusal::NotTheKindAsked | Refusal::CouldNotConvert => words::COULD_NOT_CONVERT,
        Refusal::TooLarge => words::TOO_LARGE,
        Refusal::OriginalNotChecked => words::ORIGINAL_NOT_CHECKED,
        Refusal::TooSlow => words::TOO_SLOW,
        Refusal::CopyNotChecked => words::COPY_NOT_CHECKED,
        Refusal::CopyNotWritten => words::COPY_NOT_WRITTEN,
    }
}

/// Convert the document this call was approved for.
///
/// # Errors
/// [`Refused`], carrying the call, when the grants do not cover the copy's
/// path. Nothing has been opened or created when that happens.
pub fn convert(
    service: &ConvertingService,
    touching: Touching,
    grants: &Grants,
) -> Result<Done, Refused> {
    let outcome = carry_out(service, &touching, grants)?;
    Ok(Done {
        authorised: touching.into_authorised(),
        outcome,
    })
}

/// Everything after the call is known to be one this may carry out.
fn carry_out(
    service: &ConvertingService,
    touching: &Touching,
    grants: &Grants,
) -> Result<Result<Converted, NotConverted>, Refused> {
    if touching.verb() != CONVERT_DOCUMENT {
        return Ok(Err(NotConverted::NotConverting));
    }
    let (Some(file), Some(into)) = (touching.real(FILE), touching.real(INTO)) else {
        return Ok(Err(NotConverted::NotConverting));
    };
    let (file, into) = (file.as_path(), into.as_path());
    let Some(copy_name) = copy_name(file) else {
        return Ok(Err(NotConverted::NotConverting));
    };
    let copy = into.join(&copy_name);

    // 2. Where the copy would go, asked about at the call's own moment.
    let authorised = touching.authorised();
    if let Err(why) = grants.permitting(
        authorised.under(),
        &Ask::Path(copy.clone()),
        authorised.at(),
    ) {
        return Err(Refused::not_granted(authorised.call().clone(), why));
    }
    Ok(converting(service, file, into, &copy, &copy_name))
}

/// Open, decide, create and convert.
fn converting(
    service: &ConvertingService,
    file: &Path,
    into: &Path,
    copy: &Path,
    copy_name: &OsString,
) -> Result<Converted, NotConverted> {
    // 3.
    let mut opened = original(file).map_err(|not| match not {
        NotOpened::MoreThanOneName => NotConverted::MoreThanOneName,
        NotOpened::Taken | NotOpened::NotThatKind | NotOpened::TheMachine(_) => {
            NotConverted::Unreadable
        }
    })?;

    // 4.
    let conversion = what_converts(&mut opened, file)?;

    // 5.
    let folder = Folder::open(into).map_err(|_| NotConverted::CopyNotCreated)?;
    let created = folder.create(copy_name).map_err(|not| match not {
        NotOpened::Taken => NotConverted::NameTaken {
            copy: copy_name.to_string_lossy().into_owned(),
        },
        _ => NotConverted::CopyNotCreated,
    })?;

    // 6.
    match service.convert(conversion, &opened, created.file()) {
        Ok(carried) => Ok(Converted {
            file: file.to_owned(),
            copy: copy.to_owned(),
            carried,
        }),
        Err(unconverted) => {
            // Removed whether or not the machine manages it: a copy that could
            // not be removed is empty or unwritten, and saying so would change
            // nothing about what the person does next.
            let _removed = folder.remove(created);
            Err(match unconverted {
                Unconverted::NotAnswering => NotConverted::NothingHereConverts,
                Unconverted::Refused(refusal) => NotConverted::Refused(refusal),
            })
        }
    }
}

/// Which conversion a document is for, from its bytes and then its name.
fn what_converts(opened: &mut std::fs::File, file: &Path) -> Result<Conversion, NotConverted> {
    let mut machine = ThisMachine::with_nothing()
        .opens(Kind::Pdf)
        .map_err(|_| NotConverted::NotConverting)?;
    for conversion in Conversion::EVERY {
        machine = machine
            .converts(conversion.from(), conversion.into())
            .map_err(|_| NotConverted::NotConverting)?;
    }
    let name = file.file_name().unwrap_or_default();
    match decide(opened, name, &machine).map_err(|_| NotConverted::Unreadable)? {
        decided @ Decided::NotWhatItsNameSays { .. } => {
            Err(NotConverted::NotWhatItsNameSays(decided))
        }
        Decided::AsItIs(Outcome::Converts { from, .. }) => {
            Conversion::of(from).ok_or(NotConverted::NotAKindThisConverts(from))
        }
        Decided::AsItIs(Outcome::OpensAsItIs { kind, .. })
        | Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(kind))) => {
            Err(NotConverted::NotAKindThisConverts(kind))
        }
        Decided::AsItIs(Outcome::CannotOpen(cannot)) => Err(NotConverted::CannotBeOpened(cannot)),
    }
}

/// The copy's name: the document's own, ending in `.pdf` instead.
fn copy_name(file: &Path) -> Option<OsString> {
    let mut name = file.file_stem()?.to_owned();
    name.push(".pdf");
    Some(name)
}

/// A file's own name, for a sentence.
fn name_of(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// A sentence with no gaps.
fn plainly(strings: &Strings, word: Word) -> Said {
    strings.say(&word.key(), &Filling::nothing())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **The copy's name is the document's own**, and never anything a model
    /// chose.
    #[test]
    fn the_copys_name_is_the_documents_own() {
        assert_eq!(
            copy_name(Path::new("/home/anna/Invoices/March figures.docx")),
            Some(OsString::from("March figures.pdf"))
        );
        assert_eq!(
            copy_name(Path::new("/home/anna/.xlsx")),
            Some(OsString::from(".xlsx.pdf"))
        );
        assert_eq!(copy_name(Path::new("/")), None);
    }

    /// **Every reason no copy was made is its own sentence**, and none of them
    /// is the sentence for a copy that was made.
    #[test]
    fn every_reason_no_copy_was_made_reads_differently() {
        let strings = in_english();
        let mut reasons = vec![
            NotConverted::NotConverting,
            NotConverted::NotAKindThisConverts(Kind::GifImage),
            NotConverted::CannotBeOpened(Cannot::Empty),
            NotConverted::Unreadable,
            NotConverted::MoreThanOneName,
            NotConverted::NameTaken {
                copy: "report.pdf".to_owned(),
            },
            NotConverted::CopyNotCreated,
            NotConverted::NothingHereConverts,
        ];
        reasons.extend(
            Refusal::EVERY
                .into_iter()
                .filter(|refusal| {
                    !matches!(refusal, Refusal::NotUnderstood | Refusal::NotTheKindAsked)
                })
                .map(NotConverted::Refused),
        );
        let mut said: Vec<String> = reasons
            .iter()
            .map(|reason| {
                reason
                    .said(&strings)
                    .iter()
                    .map(Said::text)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();
        let converted = Converted {
            file: PathBuf::from("/home/anna/report.docx"),
            copy: PathBuf::from("/home/anna/report.pdf"),
            carried: Carried::Everything,
        }
        .said(&strings);
        assert!(
            converted
                .first()
                .is_some_and(|said| said.text().contains("report.pdf"))
        );
        let before = said.len();
        said.sort();
        said.dedup();
        assert_eq!(said.len(), before);
    }
}
