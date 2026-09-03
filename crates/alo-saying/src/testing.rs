//! What this crate's own tests are written against.
//!
//! **A vocabulary of three strings, not the machine's.** The machine's is
//! several hundred and grows every time somebody adds a sentence anywhere in
//! the workspace, so a test written against it would be a test about whichever
//! crate last declared something. What this crate does is take a vocabulary it
//! is handed and put translations onto it, and three strings exercise that as
//! completely as three hundred: one plain sentence, one with a gap in it, and
//! one nothing translated. That the real one can be collected at all is
//! `crate::collecting`'s test, and what the real strings are is each declaring
//! crate's.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use alo_strings::{Key, Language, Phrase, Strings, Vocabulary};

use crate::failing::NotSpoken;

/// A vocabulary of three strings, standing in for a machine's.
pub(crate) fn a_small_machine() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    for (named, says) in [
        ("files.gone", "It is not there any more"),
        ("files.not-a-folder", "{path} is not a folder"),
        ("files.too-big", "{path} is too big"),
    ] {
        vocabulary
            .says(Phrase::says(Key::named(named).unwrap(), says).unwrap())
            .unwrap();
    }
    vocabulary
}

/// The same, as a lookup with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(a_small_machine())
}

/// A translation file for the fixture above, as somebody would write one.
pub(crate) fn german() -> &'static str {
    "format = 1\nlanguage = \"de\"\n\n[says]\n\"files.gone\" = \"Es ist nicht mehr da\"\n"
}

/// A folder of this test's own, on the disk the tests are running on.
///
/// `alo-files` and `alo-keeping` have the same fixture and for the same reason:
/// a test about a file has to be about a real one, and two of them must not
/// meet.
pub(crate) fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-saying-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    drop(fs::remove_dir_all(&folder));
    fs::create_dir_all(&folder).unwrap();
    folder
}

/// One file in a folder, with this in it.
pub(crate) fn wrote(folder: &Path, name: &str, what: &str) {
    fs::write(folder.join(name), what).unwrap();
}

/// One example of every way a translation can fail to be spoken, so that a
/// variant added without a sentence for it is caught by the tests that walk
/// this.
pub(crate) fn every_way_it_can_fail() -> Vec<NotSpoken> {
    vec![
        NotSpoken::NoneHere {
            at: "/usr/share/alo/translations".to_owned(),
            why: "no such file or directory".to_owned(),
        },
        NotSpoken::NotRead {
            file: "de.toml".to_owned(),
            why: "permission denied".to_owned(),
        },
        NotSpoken::NotWritten {
            file: "de.toml".to_owned(),
            why: "TOML parse error at line 4".to_owned(),
        },
        NotSpoken::FromANewerAlo {
            file: "de.toml".to_owned(),
            format: 2,
            reads: 1,
        },
        NotSpoken::NotALanguage {
            file: "de.toml".to_owned(),
            tag: "deutsch".to_owned(),
            why: "deutsch is not a language".to_owned(),
        },
        NotSpoken::NotAKey {
            file: "de.toml".to_owned(),
            named: "gone".to_owned(),
            why: "call it something.gone".to_owned(),
        },
        NotSpoken::GaveNothing {
            file: "de.toml".to_owned(),
            why: "the de translation cannot be shown as it is".to_owned(),
        },
        NotSpoken::AlreadySpoken {
            file: "de.toml".to_owned(),
            language: Language::written("de").unwrap(),
            already: "deutsch.toml".to_owned(),
        },
    ]
}
