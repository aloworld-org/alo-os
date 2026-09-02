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

use crate::real::Real;
use crate::resolving::{OnThisMachine, Resolving};

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
