//! Where the two facts this machine keeps across a restart are kept.
//!
//! The base says which build booted and nothing about why. So the machine keeps
//! two builds of its own, beside the record:
//!
//! - **the build it last knew it was running** — so the first start on a
//!   different one is noticed, once ([`crate::last_known`]);
//! - **the build the person chose to go back to**, noted before the base is
//!   told — so that start is written as a return and not as an update
//!   ([`crate::going_back`]).
//!
//! One value naming both, so that nothing which writes one of them can be
//! handed a place where the other is not.

use std::path::{Path, PathBuf};

/// Where the build last known is kept on an alo OS machine, beside the record.
pub const THE_LAST_KNOWN_BUILD: &str = "/var/lib/alo/last-known-build";

/// Where the build the person chose to go back to is kept on an alo OS
/// machine, until the machine starts on it.
pub const THE_BUILD_TO_GO_BACK_TO: &str = "/var/lib/alo/going-back-to";

/// The two files kept across a restart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcrossRestarts {
    /// The build last known.
    last_known: PathBuf,
    /// The build the person chose to go back to.
    going_back_to: PathBuf,
}

impl AcrossRestarts {
    /// Where an alo OS machine keeps them, under `/var`, which no update and
    /// no return replaces.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self {
            last_known: PathBuf::from(THE_LAST_KNOWN_BUILD),
            going_back_to: PathBuf::from(THE_BUILD_TO_GO_BACK_TO),
        }
    }

    /// Both in `folder`, under the names an alo OS machine gives them — a
    /// test's.
    #[must_use]
    pub fn in_folder(folder: &Path) -> Self {
        Self {
            last_known: folder.join("last-known-build"),
            going_back_to: folder.join("going-back-to"),
        }
    }

    /// Where the build last known is kept.
    #[must_use]
    pub fn last_known(&self) -> &Path {
        &self.last_known
    }

    /// Where the build the person chose to go back to is kept.
    #[must_use]
    pub fn going_back_to(&self) -> &Path {
        &self.going_back_to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Both are kept under `/var/lib/alo`, beside the record**, which is
    /// what an update and a return both leave alone.
    #[test]
    fn on_this_machine_both_are_kept_beside_the_record_under_var() {
        let kept = AcrossRestarts::on_this_machine();
        assert_eq!(
            kept.last_known(),
            Path::new("/var/lib/alo/last-known-build")
        );
        assert_eq!(
            kept.going_back_to(),
            Path::new("/var/lib/alo/going-back-to")
        );
        let in_a_folder = AcrossRestarts::in_folder(Path::new("/tmp/x"));
        assert_eq!(
            in_a_folder.last_known().file_name(),
            kept.last_known().file_name()
        );
        assert_eq!(
            in_a_folder.going_back_to().file_name(),
            kept.going_back_to().file_name()
        );
    }
}
