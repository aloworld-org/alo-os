//! The document a person had open when they invoked an agent — and the only
//! part of a context that grants anything.
//!
//! # Why this one and not the others
//!
//! ADR 0001 §3: *a grant comes from a deliberate act: a folder chosen in a
//! picker, or the document offered at invocation.* A person working in a file
//! and then asking an agent about it has said which file they mean as plainly
//! as if they had picked it out of a dialogue, and making them pick it again
//! would be a permission prompt for something they had just done.
//!
//! Neither of the other two parts of a context is that. What is *in front of*
//! somebody is what they were looking at, and what they had *selected* is text.
//! Only this one is a thing they opened. [`crate::Turn`] is where that becomes
//! a grant, and it grants **this file** — not the folder it is in, and not the
//! other files beside it.
//!
//! # Checked here, and checked again where the grant is made
//!
//! The rules are the ones `alo_capability::Grant` holds a reach over a single
//! file to, and this file asks them at the moment the document is offered so
//! that a person hears about it while they can still do something about it.
//! `Grant::checked` asks them again when the grant is made, and that is not
//! duplication worth removing: the two answers can only ever disagree by this
//! one being *less* strict, and the grant is what decides. There is a test
//! walking the same bad paths through both.

use std::path::{Path, PathBuf};

use alo_capability::Reach;
use alo_capability::path::{is_a_root, steps_upwards};

use crate::refusing::NotOffered;

/// The document that was open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Where it is. A full path with no `..` in it, so it means the same thing
    /// wherever it is read.
    path: PathBuf,
}

impl Document {
    /// The document at this path, or the refusal saying why it cannot be
    /// offered.
    ///
    /// # Errors
    /// [`NotOffered`], saying what to offer instead.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, NotOffered> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(NotOffered::NoDocument);
        }
        // The whole-machine check comes first so that offering `/` is refused
        // for the reason a person needs to hear, rather than for a technicality
        // about its spelling. `alo_capability::Grant` orders it the same way.
        if is_a_root(&path) {
            return Err(NotOffered::NotADocument {
                offered: shown(&path),
            });
        }
        if !path.has_root() {
            return Err(NotOffered::NotAFullPath {
                offered: shown(&path),
            });
        }
        if steps_upwards(&path) {
            return Err(NotOffered::CouldLeadElsewhere {
                offered: shown(&path),
            });
        }
        Ok(Self { path })
    }

    /// Where the document is.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// What a grant made from this document covers: **this file**, and nothing
    /// else.
    ///
    /// A [`Reach::File`] rather than a [`Reach::Folder`] over the folder it
    /// sits in, which is the difference between *the agent may read the invoice
    /// I am looking at* and *the agent may read every invoice I have ever been
    /// sent*. A person who wanted the second picks the folder.
    #[must_use]
    pub fn reach(&self) -> Reach {
        Reach::File(self.path.clone())
    }
}

/// A path as a refusal shows it.
///
/// `to_string_lossy` because a path that is not valid Unicode still has to be
/// nameable in the sentence that refuses it: showing nothing would leave a
/// person reading *that is not a full path* about no path at all. Nothing is
/// ever acted on by this string — [`Document::path`] is what a grant is made
/// from — so a lossy rendering costs nothing and saying nothing would cost the
/// sentence.
fn shown(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{hour, noon};
    use alo_capability::path::is_usable;
    use alo_capability::{Ask, Grant};

    #[test]
    fn a_document_is_the_full_path_to_the_file_that_was_open() {
        let document = Document::open("/home/anna/Invoices/march.pdf").unwrap();
        assert_eq!(document.path(), Path::new("/home/anna/Invoices/march.pdf"));
        assert_eq!(
            document.reach(),
            Reach::File(PathBuf::from("/home/anna/Invoices/march.pdf"))
        );
    }

    /// **The grant is over that file and not over the folder it is in.** The
    /// whole difference between offering a document and offering a filing
    /// cabinet, asserted through the thing that decides rather than by reading
    /// the variant.
    #[test]
    fn what_is_offered_reaches_that_file_and_nothing_beside_it() {
        let document = Document::open("/home/anna/Invoices/march.pdf").unwrap();
        let grant = Grant::checked("@files", document.reach(), noon(), hour()).unwrap();

        assert!(grant.permits(&Ask::path("/home/anna/Invoices/march.pdf"), noon()));
        assert!(!grant.permits(&Ask::path("/home/anna/Invoices/april.pdf"), noon()));
        assert!(!grant.permits(&Ask::path("/home/anna/Invoices"), noon()));
        assert!(!grant.permits(&Ask::path("/home/anna/Invoices/march.pdf.bak"), noon()));
    }

    /// Each way a document can fail to be one is refused where it is offered,
    /// and named in the refusal.
    #[test]
    fn a_path_that_could_not_be_granted_is_refused_when_it_is_offered() {
        assert_eq!(Document::open(""), Err(NotOffered::NoDocument));
        assert_eq!(
            Document::open("march.pdf"),
            Err(NotOffered::NotAFullPath {
                offered: "march.pdf".to_owned()
            })
        );
        assert!(matches!(
            Document::open("/home/anna/../root/notes.txt"),
            Err(NotOffered::CouldLeadElsewhere { .. })
        ));
        for machine in ["/", "//", "/."] {
            assert!(
                matches!(
                    Document::open(machine),
                    Err(NotOffered::NotADocument { .. })
                ),
                "{machine}"
            );
        }
    }

    /// **The two checks cannot drift apart in the direction that matters.**
    /// Every path this file refuses is one the grants would refuse as well, so
    /// nothing gets offered that could then be granted, and nothing is granted
    /// that was never offered.
    #[test]
    fn everything_refused_here_is_refused_by_the_grants_too() {
        for bad in [
            "march.pdf",
            "/home/anna/../root/notes.txt",
            "/",
            "//",
            "/.",
            "",
        ] {
            assert!(Document::open(bad).is_err(), "{bad}");
            assert!(
                Grant::checked("@files", Reach::File(PathBuf::from(bad)), noon(), hour()).is_err(),
                "{bad} is refused as a document and permitted as a grant"
            );
        }
    }

    /// A path this crate cannot read as text is still named in the refusal,
    /// because *that is not a full path* about no path at all is not a sentence
    /// anybody can act on.
    #[test]
    fn a_path_that_is_not_text_is_still_named_in_the_refusal() {
        assert_eq!(
            Document::open("Rechnungen/März.pdf"),
            Err(NotOffered::NotAFullPath {
                offered: "Rechnungen/März.pdf".to_owned()
            })
        );
    }

    /// A path this crate accepts is one the grants reason about, which is the
    /// other half of the pair above.
    #[test]
    fn everything_offered_here_is_a_path_the_grants_will_reason_about() {
        for good in [
            "/home/anna/march.pdf",
            "/home/anna/Rechnungen/März.pdf",
            "/tmp/a",
        ] {
            let document = Document::open(good).unwrap();
            assert!(is_usable(document.path()), "{good}");
            assert!(
                Grant::checked("@files", document.reach(), noon(), hour()).is_ok(),
                "{good}"
            );
        }
    }
}
