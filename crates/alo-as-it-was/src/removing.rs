//! Removing a half-made copy, including one that cannot be read.
//!
//! `std::fs::remove_dir_all` descends, so a directory it cannot read stops it —
//! and that is exactly the directory this crate has to remove. **Measured
//! 2026-10-22:** `cp -a` copying out of a folder the process may not read
//! creates the destination directory, copies the source's mode onto it, and
//! then fails on the contents. What it leaves behind is a directory with no
//! permissions on it, and `remove_dir_all` will not touch one — so
//! [`crate::putting_back`]'s cleanup silently did nothing, the working name stayed
//! on the disk, and the next attempt was refused as a *go-back that did not
//! finish* over a directory the failed attempt had just made.
//!
//! Found by the test that asserts no half-copy is left behind, which was written
//! because the comment above the cleanup claimed it.
//!
//! # It only ever removes a name this crate chose
//!
//! Nothing here takes a path from a person. The one caller passes
//! [`crate::putting_back`]'s own working name, built from a folder that was
//! already checked to be inside the person's home. A recursive remove that
//! makes things writable as it goes is a sharp tool, and this is the whole of
//! its reach.

use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Remove a directory and everything in it, making each one traversable first.
///
/// # Errors
/// Whatever the filesystem refuses. A path that is not there at all is `Ok`:
/// the caller wants it gone, and it is.
pub(crate) fn removed(at: &Path) -> io::Result<()> {
    let what = match std::fs::symlink_metadata(at) {
        Ok(what) => what,
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(why) => return Err(why),
    };
    if !what.is_dir() {
        // A link is removed as a link and never followed, which is what
        // `remove_file` does for one.
        return std::fs::remove_file(at);
    }
    // **Traversable before it is read.** The mode is the thing stopping the
    // ordinary remover, and this directory is one this crate made a moment ago.
    let _ = std::fs::set_permissions(at, std::fs::Permissions::from_mode(0o700));
    for entry in std::fs::read_dir(at)? {
        removed(&entry?.path())?;
    }
    std::fs::remove_dir(at)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::removed;

    /// A directory of this test's own.
    fn a_directory(named: &str) -> std::path::PathBuf {
        let at = std::env::temp_dir().join(format!(
            "alo-removing-{named}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// **A directory with no permissions on it is still removed**, which is the
    /// case the ordinary remover will not do and the reason this file exists.
    ///
    /// # It is declared unrunnable on the gate's hosted runner, and why
    ///
    /// That runner runs **as root**, and root ignores a mode — so
    /// `remove_dir_all` succeeds there and the control below fires. It is in
    /// `.github/the-tests-a-hosted-runner-cannot-run.txt` with that
    /// measurement, which is a statement about the runner and not about this
    /// test: it passes on every lane's machine, as the person, which is who
    /// runs it on a real one. **The control is not dropped to make the runner
    /// green**, because a test that passes there while measuring nothing is the
    /// trade that file exists to refuse.
    #[test]
    fn a_directory_with_no_permissions_is_removed() {
        let at = a_directory("no-permissions");
        let inside = at.join("inside");
        std::fs::create_dir_all(inside.join("deeper")).unwrap();
        std::fs::write(inside.join("deeper").join("a-file"), b"x").unwrap();
        std::fs::set_permissions(
            inside.join("deeper"),
            std::fs::Permissions::from_mode(0o000),
        )
        .unwrap();
        std::fs::set_permissions(&inside, std::fs::Permissions::from_mode(0o000)).unwrap();

        // The control: the ordinary remover cannot, which is what makes the
        // assertion below about this function rather than about the filesystem.
        assert!(
            std::fs::remove_dir_all(&inside).is_err(),
            "if the ordinary remover can do this, this file has no reason to exist"
        );

        removed(&inside).unwrap();
        assert!(!inside.exists(), "and this one did");
        let _ = std::fs::remove_dir_all(&at);
    }

    /// **A path that is not there is already removed.**
    #[test]
    fn a_path_that_is_not_there_is_already_gone() {
        let at = a_directory("not-there");
        removed(&at.join("never-existed")).unwrap();
        let _ = std::fs::remove_dir_all(&at);
    }

    /// **A link is removed and never followed.**
    ///
    /// A remover that followed one would descend into whatever it pointed at,
    /// which for this crate's working name would be somewhere in the person's
    /// home.
    #[test]
    fn a_link_is_removed_and_what_it_points_at_is_not() {
        let at = a_directory("a-link");
        let real = at.join("real");
        std::fs::create_dir_all(&real).unwrap();
        std::fs::write(real.join("a-file"), b"keep me").unwrap();
        let pointing = at.join("pointing");
        std::os::unix::fs::symlink(&real, &pointing).unwrap();

        removed(&pointing).unwrap();
        assert!(!pointing.exists(), "the link is gone");
        assert!(
            real.join("a-file").exists(),
            "and what it pointed at was not touched"
        );
        let _ = std::fs::remove_dir_all(&at);
    }
}
