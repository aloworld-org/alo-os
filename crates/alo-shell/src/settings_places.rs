//! Where Settings reads and writes, worked out once when a session opens it.
//!
//! Nothing here names a file. The person's folder is `alo_choosing`'s rule,
//! asked of the two variables the session hands over rather than read from the
//! environment; each section's file inside it is the name its own crate
//! declares; and the machine's grants, the daemon's pairings and the daemon's
//! own door are wherever the host was told they are. So there is no path in
//! this crate a Settings surface could keep a setting at, or knock at, that its
//! owning crate does not also name — `alo_changing::this_persons_door` is where
//! the real socket is spelled, and this crate is handed the answer.
//!
//! A login with no folder — no home directory, or a relative one — has no
//! places for the person's own sections at all, and every one of them is drawn
//! as the release ships it with nothing written. The grants and pairings are
//! the machine's rather than the person's folder's, so they are still read.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Where each section of Settings is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsPlaces {
    /// The person's folder, when the session has one.
    folder: Option<PathBuf>,
    /// The person's choice of what answers their questions, when the session
    /// has a folder.
    choosing: Option<PathBuf>,
    /// The machine's grants.
    grants: PathBuf,
    /// The pairings the daemon keeps.
    pairings: PathBuf,
    /// The socket the person's daemon listens on, which a revocation knocks at.
    daemon: PathBuf,
}

impl SettingsPlaces {
    /// The places for a session whose `$XDG_CONFIG_HOME` and `$HOME` are these,
    /// with the machine's grants, the daemon's pairings and the daemon's own
    /// door where the host was told they are.
    ///
    /// **The door is asked for here rather than separately**, so there is no
    /// session that has somewhere to read a grant from and nowhere to say it
    /// was revoked. A surface with a revoke button and no door would write the
    /// person's file and leave the running daemon serving the old list — which
    /// is worse than no button, and is a state this signature makes
    /// unreachable: the one telling carries all four.
    #[must_use]
    pub fn of(
        config_home: Option<&OsStr>,
        home: Option<&OsStr>,
        grants: &Path,
        pairings: &Path,
        daemon: &Path,
    ) -> Self {
        Self {
            folder: alo_choosing::where_the_folder_is(config_home, home),
            choosing: alo_choosing::where_it_is(config_home, home),
            grants: grants.to_owned(),
            pairings: pairings.to_owned(),
            daemon: daemon.to_owned(),
        }
    }

    /// The person's folder, or [`None`] for a login that has none.
    #[must_use]
    pub fn folder(&self) -> Option<&Path> {
        self.folder.as_deref()
    }

    /// The file a keeper declares, inside the person's folder.
    pub(crate) fn kept(&self, file: &str) -> Option<PathBuf> {
        self.folder.as_ref().map(|folder| folder.join(file))
    }

    /// The person's choice of what answers their questions.
    pub(crate) fn choosing(&self) -> Option<&Path> {
        self.choosing.as_deref()
    }

    /// The machine's grants.
    pub(crate) fn grants(&self) -> &Path {
        &self.grants
    }

    /// The pairings the daemon keeps.
    pub(crate) fn pairings(&self) -> &Path {
        &self.pairings
    }

    /// The socket this person's daemon listens on.
    ///
    /// A door is built at it per key press rather than kept open: `door.rs`
    /// makes one connection, one line and one answer, and a daemon that is not
    /// there is the ordinary case rather than a failure.
    pub(crate) fn daemon(&self) -> &Path {
        &self.daemon
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The folder and every file in it are the owning crates' answers**, and a
    /// login with no home directory has none of them — while the machine's own
    /// grants and pairings are still where the host said.
    #[test]
    fn the_places_are_the_owning_crates_and_a_login_with_no_home_has_none() {
        let grants = Path::new("/var/lib/alo/grants");
        let pairings = Path::new("/var/lib/alo/pairings");
        let door = Path::new("/run/alo/1000/agentd.sock");
        let places =
            SettingsPlaces::of(None, Some(OsStr::new("/home/ada")), grants, pairings, door);
        assert_eq!(places.daemon(), door);
        let folder = alo_choosing::where_the_folder_is(None, Some(OsStr::new("/home/ada")));
        assert_eq!(places.folder(), folder.as_deref());
        assert_eq!(
            places.kept(alo_dock::keeping::THE_FILE),
            folder.map(|folder| folder.join(alo_dock::keeping::THE_FILE))
        );
        assert_eq!(
            places.choosing(),
            alo_choosing::where_it_is(None, Some(OsStr::new("/home/ada"))).as_deref()
        );
        assert!(
            places
                .choosing()
                .unwrap()
                .starts_with(places.folder().unwrap())
        );

        for (config_home, home) in [
            (None, None),
            (Some(OsStr::new("relative/config")), None),
            (None, Some(OsStr::new("relative-home"))),
        ] {
            let nowhere = SettingsPlaces::of(config_home, home, grants, pairings, door);
            assert_eq!(nowhere.folder(), None);
            assert_eq!(nowhere.kept(alo_dock::keeping::THE_FILE), None);
            assert_eq!(nowhere.choosing(), None);
            assert_eq!(nowhere.grants(), grants);
            assert_eq!(nowhere.pairings(), pairings);
            // **A login with no folder still has a door.** It is the machine's
            // and the daemon's, like the grants and the pairings beside it, so
            // a person with no home directory can still revoke what they
            // granted.
            assert_eq!(nowhere.daemon(), door);
        }
    }
}
