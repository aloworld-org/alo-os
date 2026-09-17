//! **Where a person's answers file is**, which is in their own state and
//! nobody else's.
//!
//! [ADR 0052](../../../docs/decisions/0052-what-a-persons-applications-asked-for-is-the-persons-record.md):
//! the record of what somebody's applications asked for and were told is
//! theirs. It is one file per login, in the person's own state directory, and
//! not the machine-wide file beside the agent's record that this crate began
//! with — that file would have held two people's applications on a machine with
//! two people on it, each readable by the other.
//!
//! # The rule is the base directory specification's
//!
//! `$XDG_STATE_HOME` when it is set **and absolute**; otherwise
//! `$HOME/.local/state`, which is the specification's own default. A login with
//! neither has **nowhere** for this file, and that is [`None`] rather than a
//! guess — `/` is not somebody's state directory, and a backend that invented
//! one would keep a record that belongs to nobody.
//!
//! *State* rather than *data* or *config* is the specification's own
//! distinction and the right one here: this is a log of what was asked and
//! answered on one machine, which is exactly what it says belongs in state.
//! `alo-choosing` makes the same reading for a person's settings under
//! `$XDG_CONFIG_HOME`, and [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
//! says why a person's own things are not in `/etc` or `/var`.
//!
//! # Nothing here reads the environment
//!
//! The two variables arrive as arguments, as they do in `alo-choosing` and for
//! the same reason: a login with neither, a relative `$XDG_STATE_HOME`, and a
//! service started with an empty environment are then three tests rather than
//! three things somebody has to arrange on a real machine.
//!
//! # One directory is made, and only under one that is there
//!
//! The machine-wide file was in a folder the image makes, so nothing here made
//! folders: a typo in a path would have become a second record nobody reads. A
//! person's state directory is nobody's to make but the programs that use it, so
//! [`ThePlace::made`] makes **`alo`** — and, under `$HOME`, the specification's
//! own `.local/state` before it — inside a base directory **that already
//! exists**. A base that is not there is refused, so a typo is still a refusal
//! rather than a record nobody reads.

use std::ffi::OsStr;
use std::fs::DirBuilder;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};

use crate::not_recorded::NotRecorded;

/// The variable that names where a person's state is kept.
pub const STATE_HOME: &str = "XDG_STATE_HOME";

/// The variable that names a person's home directory.
pub const HOME: &str = "HOME";

/// What `$HOME` is followed by when `$XDG_STATE_HOME` says nothing.
pub const DOT_LOCAL_STATE: &str = ".local/state";

/// The directory alo OS keeps this login's answers in.
pub const THE_FOLDER: &str = "alo";

/// The file itself, inside that directory. Part of
/// `docs/contracts/portal-answers-file.md`.
pub const THE_ANSWERS_FILE: &str = "portal-answers.jsonl";

/// The mode the folder is made with: this login, and nobody else.
const OURS_ALONE: u32 = 0o700;

/// **Where this login's answers file is.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThePlace {
    /// The directory that must already be there: the state home, or the home
    /// directory the default was taken under.
    base: PathBuf,
    /// The folder this crate keeps its file in.
    folder: PathBuf,
    /// The file.
    file: PathBuf,
}

impl ThePlace {
    /// **Where the answers of the login these variables describe are**, or
    /// [`None`] for a login with nowhere to put them.
    #[must_use]
    pub fn for_this_login(state_home: Option<&OsStr>, home: Option<&OsStr>) -> Option<Self> {
        let state = named(state_home)
            .map(|state| (state.clone(), state))
            .or_else(|| {
                let home = named(home)?;
                let state = home.join(DOT_LOCAL_STATE);
                Some((home, state))
            })?;
        let (base, state) = state;
        let folder = state.join(THE_FOLDER);
        let file = folder.join(THE_ANSWERS_FILE);
        Some(Self { base, folder, file })
    }

    /// The file, whether or not anything has been written to it.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// The folder it is in.
    #[must_use]
    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// The directory this login already had, which everything else is made
    /// inside.
    #[must_use]
    pub fn base(&self) -> &Path {
        &self.base
    }

    /// **Make the folder if it is not there**, and answer with the file's path.
    ///
    /// # Errors
    /// [`NotRecorded::NotWritten`] naming the base directory when that is not
    /// there — a login whose state directory is missing is a login this is not
    /// the machine for — and naming the folder for everything else the machine
    /// said.
    pub fn made(&self) -> Result<&Path, NotRecorded> {
        if !self.base.is_dir() {
            return Err(NotRecorded::NotWritten {
                at: self.base.clone(),
                why: "this login has no such directory to keep a record in".to_owned(),
            });
        }
        DirBuilder::new()
            .recursive(true)
            .mode(OURS_ALONE)
            .create(&self.folder)
            .map_err(|why| NotRecorded::NotWritten {
                at: self.folder.clone(),
                why: why.to_string(),
            })?;
        Ok(&self.file)
    }
}

/// A variable that names a place: set, not empty, and absolute.
///
/// Absolute is asked as [`Path::has_root`] rather than [`Path::is_absolute`],
/// for the reason `alo-choosing` gives about the same question: the rule being
/// stated is Linux's, and `is_absolute` answers about the machine this was
/// compiled for.
fn named(variable: Option<&OsStr>) -> Option<PathBuf> {
    let named = Path::new(variable?);
    (named.has_root()).then(|| named.to_path_buf())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn a_folder() -> PathBuf {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let at = std::env::temp_dir().join(format!(
            "alo-portals-place-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&at).expect("a folder for this test");
        at
    }

    /// **The state home wins when it is set and absolute.**
    #[test]
    fn a_login_with_a_state_home_keeps_its_answers_there() {
        let place = ThePlace::for_this_login(Some(OsStr::new("/home/ada/.local/state")), None)
            .expect("a login with a state home has somewhere");
        assert_eq!(
            place.file(),
            Path::new("/home/ada/.local/state/alo/portal-answers.jsonl")
        );
        assert_eq!(place.folder(), Path::new("/home/ada/.local/state/alo"));
        assert_eq!(place.base(), Path::new("/home/ada/.local/state"));
    }

    /// **And without one, the specification's own default under `$HOME`.**
    #[test]
    fn a_login_with_only_a_home_keeps_them_where_the_specification_says() {
        let place = ThePlace::for_this_login(None, Some(OsStr::new("/home/ada")))
            .expect("a login with a home has somewhere");
        assert_eq!(
            place.file(),
            Path::new("/home/ada/.local/state/alo/portal-answers.jsonl")
        );
        assert_eq!(
            place.base(),
            Path::new("/home/ada"),
            "the directory that must already be there is the home, not the state directory the \
             specification says to make inside it"
        );
    }

    /// **A relative variable is ignored**, which is the specification's rule and
    /// matters more than it reads: a relative path would put a person's record
    /// wherever the backend happened to be started from, and somewhere else the
    /// next time.
    #[test]
    fn a_relative_state_home_is_not_a_place() {
        let place =
            ThePlace::for_this_login(Some(OsStr::new("state")), Some(OsStr::new("/home/ada")))
                .expect("the home is still a place");
        assert_eq!(place.base(), Path::new("/home/ada"));

        assert_eq!(
            ThePlace::for_this_login(Some(OsStr::new("state")), Some(OsStr::new("home"))),
            None,
            "two relative variables are nowhere, not a path under whatever this was started from"
        );
    }

    /// **A login with neither variable has nowhere**, and is told so rather
    /// than given `/`.
    #[test]
    fn a_login_with_neither_has_nowhere_rather_than_the_root_of_the_machine() {
        assert_eq!(ThePlace::for_this_login(None, None), None);
        assert_eq!(
            ThePlace::for_this_login(Some(OsStr::new("")), Some(OsStr::new(""))),
            None
        );
    }

    /// **The folder is made, `0700`, inside a base that is there.**
    #[test]
    fn the_folder_is_made_for_this_login_and_nobody_else() {
        let home = a_folder();
        let place =
            ThePlace::for_this_login(None, Some(home.as_os_str())).expect("a home is somewhere");
        let file = place.made().expect("the folder was made");

        assert_eq!(file, place.file());
        assert!(place.folder().is_dir());
        assert!(
            !file.exists(),
            "making the folder wrote a record nobody kept"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(place.folder())
                .expect("the folder is there")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(
                mode, OURS_ALONE,
                "somebody else can read this person's record"
            );
        }

        // And making it again is not an error: a backend starting twice is an
        // ordinary day.
        place.made().expect("the folder is still there");
    }

    /// **A base that is not there is refused**, so a typo is a refusal rather
    /// than a second record nobody reads.
    #[test]
    fn a_state_directory_that_is_not_there_is_refused_rather_than_built() {
        let nowhere = a_folder().join("not-a-login");
        let place = ThePlace::for_this_login(Some(nowhere.as_os_str()), None)
            .expect("an absolute path is a place");
        let why = place.made().expect_err("there is no such directory");
        assert!(matches!(why, NotRecorded::NotWritten { .. }));
        assert!(
            !place.folder().exists(),
            "a folder was made under a directory that was not there"
        );
    }
}
