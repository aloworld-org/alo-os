//! **The person's two choices for the screen road, read from their file each
//! time a picture is asked for** — `alo_turn::TheScreenRoadAsChosen` on a real
//! machine ([ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)).
//!
//! Both are kept in `/var/lib/alo/how-code-runs.toml`
//! (`docs/contracts/how-code-runs-file.md`, `pictures-and-keys-in` and
//! `pictures-read-by`), beside the level, under the rules about who may write
//! that file — so an agent can switch neither on for itself.
//!
//! **Read on every call and never held**: a switch turned off between a
//! proposal and its OK stops the picture. **A file that will not read is no
//! application and no one** — the two `as_read` rules — so a damaged file never
//! lets a picture leave.

use std::path::{Path, PathBuf};

use alo_capability::{PicturesAndKeys, WhoReadsPictures};
use alo_turn::TheScreenRoadAsChosen;

/// The screen road's choices, kept at one path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheScreenRoadHere {
    /// The person's file.
    at: PathBuf,
}

impl TheScreenRoadHere {
    /// The choices kept at `at`.
    #[must_use]
    pub fn at(at: &Path) -> Self {
        Self { at: at.to_owned() }
    }

    /// The choices kept where a machine keeps them.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::at(Path::new(alo_remembering::THE_LEVEL))
    }
}

impl TheScreenRoadAsChosen for TheScreenRoadHere {
    fn pictures_and_keys(&self) -> PicturesAndKeys {
        PicturesAndKeys::as_read(alo_remembering::pictures_and_keys_remembered(&self.at))
    }

    fn who_reads_pictures(&self) -> WhoReadsPictures {
        WhoReadsPictures::as_read(alo_remembering::who_reads_pictures_remembered(&self.at))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn a_file_saying(name: &str, said: &str) -> PathBuf {
        let folder = crate::testing::a_directory_of_our_own(name);
        let at = folder.join("how-code-runs.toml");
        std::fs::write(&at, said).unwrap();
        at
    }

    /// **No file is no application and no one** — how a machine arrives.
    #[test]
    fn no_file_is_nothing_switched_on_and_no_reader() {
        let here = TheScreenRoadHere::at(Path::new("/proc/alo-nothing-here/how-code-runs.toml"));
        assert_eq!(here.pictures_and_keys(), PicturesAndKeys::default());
        assert_eq!(here.who_reads_pictures(), WhoReadsPictures::NoOne);
    }

    /// **What the file says now is what is answered**, read again each time.
    #[test]
    fn the_file_is_read_again_each_time() {
        // **No file yet**, which is how a machine arrives: the choices write it.
        let at = crate::testing::a_directory_of_our_own("screen-road-read-again")
            .join("how-code-runs.toml");
        let here = TheScreenRoadHere::at(&at);
        assert_eq!(here.who_reads_pictures(), WhoReadsPictures::NoOne);
        alo_remembering::who_reads_pictures_chosen(&at, WhoReadsPictures::ThisComputer).unwrap();
        alo_remembering::pictures_and_keys_switched(&at, "net.supertuxkart.SuperTuxKart", true)
            .unwrap();
        assert_eq!(here.who_reads_pictures(), WhoReadsPictures::ThisComputer);
        assert!(
            here.pictures_and_keys()
                .allows("net.supertuxkart.SuperTuxKart")
        );
    }

    /// **A file that will not read lets no picture leave**: no one reads, and
    /// nothing is switched on.
    #[test]
    fn a_file_that_will_not_read_is_no_one_and_nothing() {
        let at = a_file_saying("screen-road-unreadable", "this is = = not toml [");
        let here = TheScreenRoadHere::at(&at);
        assert_eq!(here.who_reads_pictures(), WhoReadsPictures::NoOne);
        assert_eq!(here.pictures_and_keys(), PicturesAndKeys::default());
    }
}
