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
    /// How code the agent writes or fetches runs: `alo_remembering::THE_LEVEL`.
    ///
    /// The machine's, beside the grants, because ADR 0064 makes running code a
    /// grant and its contract holds the file to the same three rules about who
    /// may have written it. **Handed over rather than taken from the
    /// constant**, which is this file's rule for every other path here, and is
    /// what lets a test point the section at a file of its own rather than at
    /// the machine's.
    how_code_runs: PathBuf,
    /// The shell this account uses, as the session was told it.
    ///
    /// **Handed over rather than read here**, which is this file's rule for
    /// everything else: the session knows its own environment and this crate is
    /// given the answer. `None` where the session did not say, which is a
    /// different answer from a shell alo ships nothing for.
    shell: Option<String>,
    /// The person's home folder, where the picker starts.
    ///
    /// **Separate from `folder` above**, which is where their settings are
    /// kept — usually `.config/alo` inside this. A picker that opened in the
    /// settings folder would put somebody two levels below their own files
    /// before they had pressed anything.
    home: Option<PathBuf>,
    /// The agent this machine has, by the name its grants use.
    ///
    /// **Handed over rather than read here**, which is this file's rule for
    /// everything else. It is the machine description's `name`, and it has to
    /// be that exact name: a grant written under any other is one the daemon
    /// will never match. [`None`] where the session did not say, and then **no
    /// folder can be given** — a picker that granted to a guess would walk a
    /// person through choosing a folder and hand them a grant that does
    /// nothing at all.
    agent: Option<String>,
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
    #[expect(
        clippy::too_many_arguments,
        reason = "every place arrives together on purpose; the doc above says why"
    )]
    pub fn of(
        config_home: Option<&OsStr>,
        home: Option<&OsStr>,
        shell: Option<&OsStr>,
        grants: &Path,
        pairings: &Path,
        daemon: &Path,
        how_code_runs: &Path,
        agent: Option<&str>,
    ) -> Self {
        Self {
            folder: alo_choosing::where_the_folder_is(config_home, home),
            choosing: alo_choosing::where_it_is(config_home, home),
            grants: grants.to_owned(),
            pairings: pairings.to_owned(),
            daemon: daemon.to_owned(),
            how_code_runs: how_code_runs.to_owned(),
            // Lossy on purpose: a shell whose path is not text is a shell this
            // crate cannot match against a name anyway, and losing a byte is
            // better than refusing to draw the rest of Settings.
            shell: shell.map(|shell| shell.to_string_lossy().into_owned()),
            // Lossy for the shell's reason, and the picker refuses a name it
            // cannot use anyway.
            home: home.map(PathBuf::from),
            agent: agent.map(str::to_owned),
        }
    }

    /// The person's home folder, where the picker starts.
    #[must_use]
    pub fn home(&self) -> Option<&Path> {
        self.home.as_deref()
    }

    /// The agent a folder would be given to, by the name the grants use.
    ///
    /// [`None`] where the session did not say, which is the one state in which
    /// Settings offers no way to give a folder at all.
    #[must_use]
    pub fn agent(&self) -> Option<&str> {
        self.agent.as_deref()
    }

    /// The shell this account uses, as the session was told it.
    ///
    /// [`None`] where the session did not say — which the shell-integration
    /// section tells apart from a shell alo ships nothing for.
    #[must_use]
    pub fn shell(&self) -> Option<&str> {
        self.shell.as_deref()
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

    /// Where this machine's level of running code is kept.
    pub(crate) fn how_code_runs(&self) -> &Path {
        &self.how_code_runs
    }
}

/// **The names file the daemon keeps beside the pairings at `pairings`**:
/// `alo_remembering::THE_MACHINE_NAMES` sits in the same folder as
/// `alo_remembering::THE_PAIRINGS` (*beside the pairings*, that crate says),
/// so it is found from wherever the host said the pairings are — which keeps
/// a test's own folder and the machine's in step without a second place to
/// name.
pub(crate) fn the_names_beside(pairings: &Path) -> PathBuf {
    let named = Path::new(alo_remembering::THE_MACHINE_NAMES)
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("machine-names.toml"));
    pairings.with_file_name(named)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **On the machine, the names beside the pairings are the daemon's names
    /// file**, so a Settings reading them reads what the daemon wrote.
    #[test]
    fn the_names_beside_the_machines_pairings_are_the_daemons_names_file() {
        assert_eq!(
            the_names_beside(Path::new(alo_remembering::THE_PAIRINGS)),
            Path::new(alo_remembering::THE_MACHINE_NAMES)
        );
    }

    /// **The folder and every file in it are the owning crates' answers**, and a
    /// login with no home directory has none of them — while the machine's own
    /// grants and pairings are still where the host said.
    #[test]
    fn the_places_are_the_owning_crates_and_a_login_with_no_home_has_none() {
        let grants = Path::new("/var/lib/alo/grants");
        let pairings = Path::new("/var/lib/alo/pairings");
        let door = Path::new("/run/alo/1000/agentd.sock");
        let level = Path::new("/var/lib/alo/how-code-runs.toml");
        let places = SettingsPlaces::of(
            None,
            Some(OsStr::new("/home/ada")),
            None,
            grants,
            pairings,
            door,
            level,
            None,
        );
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
            let nowhere = SettingsPlaces::of(
                config_home,
                home,
                None,
                grants,
                pairings,
                door,
                Path::new("/var/lib/alo/how-code-runs.toml"),
                None,
            );
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
