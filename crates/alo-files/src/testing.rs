//! The folders and the real paths this crate's tests are written against.
//!
//! The acting half is tested against a real filesystem, because there is only
//! one thing that acts and abstracting it would be inventing a second answer to
//! *what happened when the machine was asked*. So every test here makes a
//! folder of its own under whatever this machine calls its temporary
//! directory, and takes it away afterwards.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use alo_capability::Refused;
use alo_strings::Strings;

use crate::failed::Failed;
use crate::real::Real;
use crate::resolving::{OnThisMachine, Resolving};
use crate::words::file_words;

/// A folder of this test's own, resolved, so that what is granted and what is
/// asked about are spelled the way this machine spells them.
///
/// That is not a test convenience — `docs/quirks.md` records why. A grant is
/// over a place, so a person picking a folder grants the *real* one; on Windows
/// a resolved path carries a `\\?\` prefix that the path it was typed from does
/// not, and a grant made over the unresolved spelling would match nothing.
///
/// Named after the test rather than at random, because a leftover folder should
/// say which test left it.
pub(crate) fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-files-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// Where this path really leads, as the one resolver on this machine says.
pub(crate) fn really(path: &Path) -> Real {
    OnThisMachine.real(path).unwrap()
}

/// The words this machine reads, with nothing translated: what a machine that
/// has no translations shows, which is what most of these tests are about.
///
/// **Both crates' lists.** A shell has one vocabulary and every crate declares
/// into it, and these tests need that arrangement rather than a smaller one: a
/// refusal met here can have been worded by the file half — *this path really
/// leads elsewhere* — or by the capability model, which is where *nobody
/// granted it* comes from.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// What a failure says on such a machine.
pub(crate) fn said(failed: &Failed) -> String {
    failed.said(&in_english()).into_text()
}

/// What a refusal says on such a machine.
///
/// `alo_capability::Refused` has no `Display` either, for the reason
/// [`crate::Failed`] has none: the only road to words goes past the strings the
/// person reads, whichever crate worded it.
pub(crate) fn refusal(refused: &Refused) -> String {
    refused.said(&in_english()).into_text()
}

/// One example of every way the machine can fail to do something it was
/// allowed to do.
///
/// Written out by hand, because the point of the list is that a variant added
/// without a word for it is caught — and a list derived from the variants
/// would be derived from the same thing it is checking.
pub(crate) fn every_failure() -> Vec<Failed> {
    vec![
        Failed::NotAFileVerb {
            verb: "open_application".to_owned(),
        },
        Failed::Missing {
            verb: "move_file".to_owned(),
            argument: "into".to_owned(),
        },
        Failed::NotAFolder {
            path: "/home/anna/Invoices/march.pdf".to_owned(),
        },
        Failed::NotAFile {
            path: "/home/anna/Invoices".to_owned(),
        },
        Failed::Gone {
            path: "/home/anna/Invoices/march.pdf".to_owned(),
        },
        Failed::TooBig {
            path: "/home/anna/Invoices/scan.tiff".to_owned(),
            bytes: 200_000_000,
            most: 1_048_576,
        },
        Failed::NotText {
            path: "/home/anna/Invoices/scan.tiff".to_owned(),
        },
        Failed::AlreadyThere {
            path: "/home/anna/Archive/march.pdf".to_owned(),
        },
        Failed::AlreadyIn {
            path: "/home/anna/Archive/march.pdf".to_owned(),
        },
        Failed::IntoItself {
            folder: "/home/anna/Invoices".to_owned(),
        },
        Failed::NotAZipName {
            name: "invoices".to_owned(),
        },
        Failed::TooMany {
            folder: "/home/anna/Invoices".to_owned(),
            most: 20_000,
        },
        Failed::TooMuch {
            folder: "/home/anna/Invoices".to_owned(),
            most: 2_147_483_648,
        },
        Failed::TheMachineSaidNo {
            path: "/home/anna/Invoices".to_owned(),
            doing: "listed".to_owned(),
            why: "permission denied".to_owned(),
        },
    ]
}
