//! The one folder the broker writes into and the update units read from:
//! root's, and nobody else's.
//!
//! A person hands an update over in `/run/alo-broker/wanted`, which is `0770`
//! in their own group — so they can write there, and so can anything else of
//! theirs. The broker checks what is there against the identity that was
//! approved ([`crate::approved`]), and **then writes those exact bytes here**,
//! where only root can reach them, before starting the unit that acts on them.
//!
//! Copying rather than pointing the unit at the person's folder is the whole
//! of the reason this file exists. Between the moment the broker digests a
//! handed-over file and the moment a privileged program reads it, the person's
//! folder can be written again by anything running as that person — which
//! includes anything that has got hold of their session. A unit reading from
//! there would act on bytes nobody approved, and the check would have proved
//! something about a file that no longer exists.
//!
//! **Nothing is left behind.** The broker removes what it put here whichever
//! way the act went, because a `/run` file naming a build to install is an
//! instruction waiting for somebody to start a unit by hand.

use std::fs::{self, File, OpenOptions};
use std::io::{Read as _, Write as _};
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _};
use std::path::{Path, PathBuf};

use alo_broker::NotCarried;

/// The folder itself: root's, nobody else's, made when the broker starts.
pub const THE_FOLDER: &str = "/run/alo-broker/approved";

/// The mode the folder is made with: root reads, writes and enters; nobody
/// else does anything at all.
pub const THE_FOLDERS_MODE: u32 = 0o700;

/// The mode each file is written with.
const THE_FILES_MODE: u32 = 0o600;

/// Where the broker puts the update a person approved, for
/// `alo-applying-an-update.service` to read.
pub const THE_UPDATE: &str = "/run/alo-broker/approved/update.json";

/// Where the broker puts the identity going back was approved under, for
/// `alo-going-back.service` to check what it decides against.
pub const THE_RETURN: &str = "/run/alo-broker/approved/going-back.identity";

/// Why a unit would not read what it was handed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotHanded(pub String);

impl std::fmt::Display for NotHanded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for NotHanded {}

/// Where these are on a machine, and where a test puts its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handing {
    /// The folder they are written into.
    folder: PathBuf,
}

impl Handing {
    /// The folder on a machine.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::at(Path::new(THE_FOLDER))
    }

    /// Some other folder — a test's.
    #[must_use]
    pub fn at(folder: &Path) -> Self {
        Self {
            folder: folder.to_owned(),
        }
    }

    /// The update a person approved.
    #[must_use]
    pub fn update(&self) -> PathBuf {
        self.named(THE_UPDATE)
    }

    /// The identity going back was approved under.
    #[must_use]
    pub fn going_back(&self) -> PathBuf {
        self.named(THE_RETURN)
    }

    /// One of the two, by the name a machine gives it.
    fn named(&self, on_a_machine: &str) -> PathBuf {
        let name = Path::new(on_a_machine)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("approved"));
        self.folder.join(name)
    }
}

/// Write `bytes` at `at`, whole, root's alone, replacing whatever was there.
///
/// # Errors
/// [`NotCarried`], and the unit is started with nothing.
pub fn put(at: &Path, bytes: &[u8]) -> Result<(), NotCarried> {
    let beside = at.with_extension("writing");
    drop(fs::remove_file(&beside));
    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(THE_FILES_MODE)
        .open(&beside)
        .and_then(|mut file| {
            file.write_all(bytes)?;
            file.sync_all()
        })
        .and_then(|()| fs::rename(&beside, at));
    written.map_err(|why| {
        drop(fs::remove_file(&beside));
        NotCarried(format!(
            "what the person approved could not be handed to the unit that carries it out: {why}"
        ))
    })
}

/// Take away what [`put`] left, whichever way the act went.
pub fn taken_away(at: &Path) {
    drop(fs::remove_file(at));
}

/// Read what the broker handed over, believed only as a plain file of root's
/// that nobody else can read or write.
///
/// The folder is already root's alone, so this is the second lock on the same
/// door — and it is the one that still holds if somebody ever widens the
/// folder. A privileged program that reads whatever is at a path is a
/// privileged program waiting for a link to be left there.
///
/// # Errors
/// [`NotHanded`], and nothing is read.
pub fn handed(at: &Path, longest: u64) -> Result<Vec<u8>, NotHanded> {
    let refused = |why: &str| NotHanded(format!("{}: {why}", at.display()));
    let opened = rustix::fs::open(
        at,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::NONBLOCK
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(|why| refused(&format!("was not opened: {why}")))?;
    let file = File::from(opened);
    let about = file
        .metadata()
        .map_err(|why| refused(&format!("was not looked at: {why}")))?;
    if !about.file_type().is_file() {
        return Err(refused("is not a plain file"));
    }
    if about.uid() != 0 {
        return Err(refused("is not root's, so the broker did not write it"));
    }
    if about.mode() & 0o077 != 0 {
        return Err(refused(
            "can be read or written by somebody other than root, so it is not believed",
        ));
    }
    if about.len() > longest {
        return Err(refused("is longer than it can be"));
    }
    let mut bytes = Vec::new();
    file.take(longest)
        .read_to_end(&mut bytes)
        .map_err(|why| refused(&format!("was not read: {why}")))?;
    Ok(bytes)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A folder of this test's own, empty.
    fn a_folder(named: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!(
            "alo-brokerd-for-the-unit-{}-{named}",
            std::process::id()
        ));
        drop(fs::remove_dir_all(&at));
        fs::create_dir_all(&at).unwrap();
        at
    }

    /// **What the broker puts there is what the unit reads back**, and it is
    /// root's alone.
    #[test]
    fn what_is_put_there_is_what_is_read_back() {
        let folder = a_folder("round-trip");
        let handing = Handing::at(&folder);
        put(&handing.update(), b"{\"from\":\"a\"}\n").unwrap();
        assert_eq!(
            handed(&handing.update(), 1024).unwrap(),
            b"{\"from\":\"a\"}\n"
        );
        let about = fs::metadata(handing.update()).unwrap();
        assert_eq!(about.mode() & 0o777, THE_FILES_MODE);
        taken_away(&handing.update());
        assert!(handed(&handing.update(), 1024).is_err());
    }

    /// **A file anybody else can read is not believed**, even in a folder that
    /// should have made it impossible.
    #[test]
    fn a_file_somebody_else_can_read_is_not_believed() {
        use std::os::unix::fs::PermissionsExt as _;

        let folder = a_folder("wide-open");
        let at = folder.join("update.json");
        fs::write(&at, b"{}\n").unwrap();
        fs::set_permissions(&at, fs::Permissions::from_mode(0o644)).unwrap();
        let refused = handed(&at, 1024).unwrap_err();
        assert!(refused.to_string().contains("other than root"), "{refused}");
    }

    /// **A link where a file should be is not followed**, and neither is a
    /// folder read as one.
    #[test]
    fn a_link_where_a_file_should_be_is_not_followed() {
        let folder = a_folder("a-link");
        let elsewhere = folder.join("somebody-elses.json");
        fs::write(&elsewhere, b"{}\n").unwrap();
        let at = folder.join("update.json");
        std::os::unix::fs::symlink(&elsewhere, &at).unwrap();
        assert!(handed(&at, 1024).is_err());

        let a_folder_instead = folder.join("going-back.identity");
        fs::create_dir(&a_folder_instead).unwrap();
        let refused = handed(&a_folder_instead, 1024).unwrap_err();
        assert!(refused.to_string().contains("plain file"), "{refused}");
    }

    /// **Something enormous is not read into a privileged process.**
    #[test]
    fn something_longer_than_it_can_be_is_not_read() {
        let folder = a_folder("enormous");
        let at = folder.join("update.json");
        put(&at, &vec![b'a'; 4096]).unwrap();
        let refused = handed(&at, 1024).unwrap_err();
        assert!(refused.to_string().contains("longer"), "{refused}");
    }

    /// **The two files a machine hands over are the two this names**, and a
    /// test's folder holds them under the same names.
    #[test]
    fn the_two_files_are_named_the_same_wherever_the_folder_is() {
        let machines = Handing::on_this_machine();
        assert_eq!(machines.update(), Path::new(THE_UPDATE));
        assert_eq!(machines.going_back(), Path::new(THE_RETURN));
        let folder = a_folder("names");
        let mine = Handing::at(&folder);
        assert_eq!(mine.update(), folder.join("update.json"));
        assert_eq!(mine.going_back(), folder.join("going-back.identity"));
    }
}
