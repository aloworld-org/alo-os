//! **Which applications this machine has, read from its desktop entries at
//! the moment a verb asks** — so an application that arrived after the person
//! signed in is one an agent can reach.
//!
//! Until 2026-11-14 this daemon read the list once, when it started, and kept
//! it. It starts with the person's session, and on a fresh machine
//! `alo-shipping` installs the first eight applications after that, from the
//! network; one alo installed itself was not on the list either. Both were
//! *not installed* to every application verb until the next sign-in.
//! `alo_turn::the_applications_here` holds the argument.
//!
//! The places are the ones `crate::how_applications_start_here` reads how
//! each application starts from — both flatpak installations, the person's and
//! the machine's, then `XDG_DATA_DIRS` — so the list and the road agree.

use std::path::PathBuf;

use alo_applications::Installed;
use alo_turn::SaysWhatIsInstalled;

/// This machine's desktop entries, read each time they are asked about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatIsInstalledHere {
    /// The `applications/` directories, most specific first.
    entries: Vec<PathBuf>,
}

impl WhatIsInstalledHere {
    /// Read from these places.
    #[must_use]
    pub const fn in_these(entries: Vec<PathBuf>) -> Self {
        Self { entries }
    }
}

impl SaysWhatIsInstalled for WhatIsInstalledHere {
    fn what_is_installed(&self) -> Installed {
        alo_installed::read(&self.entries)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **An entry written after this was made is read**, which is the whole
    /// difference from the list this replaced.
    #[test]
    fn an_application_that_arrives_later_is_seen() {
        let root = tempfile::TempDir::new().unwrap();
        let applications = root.path().join("applications");
        let here = WhatIsInstalledHere::in_these(vec![applications.clone()]);
        assert!(here.what_is_installed().is_empty());
        std::fs::create_dir_all(&applications).unwrap();
        std::fs::write(
            applications.join("com.vscodium.codium.desktop"),
            "[Desktop Entry]\nType=Application\nName=VSCodium\nExec=codium\n",
        )
        .unwrap();
        assert!(here.what_is_installed().has("com.vscodium.codium"));
    }
}
