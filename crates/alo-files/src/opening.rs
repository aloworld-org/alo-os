//! Opening a file, and moving a name, with no gap between the check and the
//! act.
//!
//! Every other file in this crate decides *whether*. This one is the only place
//! that asks the kernel to do it, and it exists because two of those questions
//! cannot be answered honestly by `std`:
//!
//! - a path is resolved, the grants permit it, and then the file is opened **by
//!   that name** — and anything with write access to a folder on the way can
//!   swap a link in between the two;
//! - a destination is checked for and then renamed onto, because `fs::rename`
//!   has no portable no-clobber form, so *nothing is replaced that was not
//!   named* is a check with a gap after it rather than a property.
//!
//! `docs/quirks.md` recorded both as residual races the portable acting half
//! could not close. This file closes them on Linux and says plainly what it
//! does everywhere else.
//!
//! # The one rented thing here
//!
//! `openat` and `renameat2` have no spelling in `std` on a stable compiler, and
//! `CLAUDE.md` forbids `unsafe` workspace-wide, so the choice is a crate or an
//! ADR asking to write the syscalls out by hand. Item 21c already made it once,
//! for `SO_PEERCRED`: **`rustix`**, pinned in the workspace, named in exactly
//! one file per crate. A second wrapper for the same kind of call would be two
//! rented spellings of the same kernel, so this is that file for `alo-files`.
//! If it is ever replaced, this file is the change.
//!
//! # Every component, not only the last one
//!
//! `O_NOFOLLOW` on the file being opened refuses a link *in the final position*
//! and says nothing about the rest of the path. A path is a sequence of
//! lookups, and `/home/anna/Invoices/march.pdf` is four of them: an attacker
//! who can replace `Invoices` does not need to touch `march.pdf` at all.
//!
//! So [`read_only`] is `openat2` with **`RESOLVE_NO_SYMLINKS`**, which is the
//! kernel refusing a link at *any* component of one lookup. There is no moment
//! between the components for a substitution to be made in, because there are
//! no moments: the whole path is resolved inside one syscall or the syscall
//! fails.
//!
//! # Why not walk it a component at a time
//!
//! The obvious alternative — open `/`, then each folder relative to the handle
//! before it — gives the same guarantee and was written first. It cannot be
//! used here, and the reason is one of alo OS's other guarantees rather than a
//! preference.
//!
//! A turn runs inside a kernel boundary (ADR 0013, ADR 0015) whose rule is that
//! this execution may open **what its call named, and what is under it**;
//! [`crate::reaching`] is where those places are decided. A walk from the root
//! opens `/`, then `/home`, then `/home/anna` — every one of them *above* what
//! the call named, and every one of them an open the boundary refuses with
//! `EACCES`. So a bounded turn could not read a file it had been granted. It
//! was caught by `alo-agentd`'s boundary test on a running kernel, which is
//! what that test is for.
//!
//! `openat2` resolves the whole path in the kernel and opens exactly one file:
//! one `file_open` for the boundary to see, and it is the file the call named.
//! The guarantee is the stronger one and the boundary does not have to widen by
//! a single directory to allow it.
//!
//! # Refusing a link here refuses exactly the ones that arrived late
//!
//! That is only safe because of what reaches this file. Every path here has
//! been through [`crate::resolving`] and is a [`crate::Real`] — canonical, with
//! every link already followed — or it was built by [`crate::walking`], which
//! steps over a link at every depth rather than through it. So a path arriving
//! here has no link in it, and one found now is one that appeared *after* the
//! grants were asked. Refusing every link therefore refuses the substitution
//! and nothing a person legitimately named.
//!
//! # What a rename still cannot promise, and why it is not fixed here
//!
//! `renameat2` has no `RESOLVE_NO_SYMLINKS`. A rename never follows a link in
//! the *final* position — it moves the link itself — so the destination cannot
//! be turned into a way of writing somewhere else. The folders **on the way**
//! to either name are resolved by name, and closing that would need handles on
//! the two folders, which needs opening them, which is the boundary again: the
//! folder a `move_file` takes a file *out of* is not a place its call named.
//!
//! So a rename closes the collision race — one call that refuses and moves —
//! and leaves the substitution race on its path components open. That is in
//! `docs/quirks.md` with what it would take to close it, which is a decision
//! about how wide a turn's boundary is and belongs in an ADR rather than here.
//!
//! # Refusing rather than degrading
//!
//! Both calls can be missing. `RENAME_NOREPLACE` is a kernel feature not every
//! filesystem implements, and the ones that do not answer `EINVAL`; `openat2`
//! arrived in Linux 5.6, and an older kernel answers `ENOSYS`. The tempting
//! thing in either case is to notice and fall back to what this file exists to
//! replace, which would mean the guarantee quietly stops holding on exactly the
//! kernels and filesystems nobody tested.
//!
//! Neither falls back, and nothing here asks the machine what it supports:
//! the call is made, and what it answers is the answer. A filesystem or a
//! kernel that cannot promise refuses the work, in its own words, through
//! [`crate::failed::Failed::machine`] — a person is told their file was not
//! read or not moved, which is true, rather than having it done by a code path
//! that only runs where nobody looked.
//!
//! # What the other platforms get
//!
//! The portable half is exactly what it was, and deliberately: a check, then
//! the act, with the gap written down. That is not this file weakening on
//! Windows — it is `std` having no better answer there, and `docs/quirks.md`
//! saying so since 2026-09-02. What changed is that Linux, which is what alo OS
//! ships, no longer has the gap.

use std::fs::File;
use std::io;
use std::path::Path;

/// Open a file for reading without following a link at any point in its path.
///
/// On Linux the whole path is resolved inside one syscall that refuses a link
/// at every component, so nothing on the way can be exchanged between the check
/// and the open. Elsewhere it is `File::open`, which resolves the name once
/// more.
///
/// # Errors
/// Whatever the machine said. A component that has become a symbolic link is
/// `ELOOP`, which reaches a person as a refusal rather than as a file somebody
/// else chose. A kernel too old to promise this is `ENOSYS`, which refuses the
/// read rather than doing it the old way.
pub(crate) fn read_only(path: &Path) -> io::Result<File> {
    platform::read_only(path)
}

/// Move a name, refusing to replace anything already at the destination.
///
/// On Linux this is one syscall that both checks and moves, so there is no
/// moment in between. Elsewhere it is a check and then a move, and the gap is
/// the one `docs/quirks.md` records.
///
/// Neither half promises anything about a folder **on the way** to either name;
/// [`self`] says why that is a boundary question rather than a syscall one, and
/// `docs/quirks.md` keeps it.
///
/// # Errors
/// [`io::ErrorKind::AlreadyExists`] when the destination is taken — which the
/// caller turns into the refusal a person reads — and whatever the machine said
/// otherwise. A filesystem that cannot promise not to replace refuses here; it
/// does not fall back to replacing.
pub(crate) fn rename_no_replace(from: &Path, to: &Path) -> io::Result<()> {
    platform::rename_no_replace(from, to)
}

#[cfg(target_os = "linux")]
mod platform {
    //! The Linux answers, and the only place `rustix` is named in this crate.

    use std::fs::File;
    use std::io;
    use std::path::{Component, Path};

    use rustix::fs::{CWD, Mode, OFlags, RenameFlags, ResolveFlags, openat2, renameat_with};

    /// Opening the file: read-only, closed on exec, and not a link itself.
    const READING: OFlags = OFlags::RDONLY
        .union(OFlags::CLOEXEC)
        .union(OFlags::NOFOLLOW);

    /// **The whole of the guarantee.** No component of this path may be a
    /// symbolic link — not the last one, and not one on the way to it.
    const STRICTLY: ResolveFlags = ResolveFlags::NO_SYMLINKS;

    /// A path this file will not open, said as an [`io::Error`] so that every
    /// caller reports it the same way as anything else the machine refused.
    fn not_ours() -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "a path opened here must be absolute and already resolved",
        )
    }

    /// That a path is the kind this file was written for: absolute, naming
    /// something, and with no `..` left in it.
    ///
    /// A `..` is refused rather than resolved: this crate is handed paths that
    /// [`crate::real::Real`] has already made canonical, so one appearing here
    /// means the caller is not the one this was written for, and following it
    /// would be inventing a second resolver next to the one the grants were
    /// asked about. Refusing costs nothing a real caller wanted.
    ///
    /// **An interior `.` never reaches this.** `Path::components` drops it, so
    /// `/home/anna/./march.pdf` arrives as three names — which is what `.`
    /// means, and not a second answer to where a path leads.
    /// `Component::CurDir` is still refused, because it does arrive for a path
    /// that *is* `.` or begins with `./`, and such a path is relative anyway.
    fn already_resolved(path: &Path) -> io::Result<()> {
        let mut rooted = false;
        let mut names = 0_usize;
        for part in path.components() {
            match part {
                Component::RootDir => rooted = true,
                Component::Normal(_) => names = names.saturating_add(1),
                Component::Prefix(_) | Component::CurDir | Component::ParentDir => {
                    return Err(not_ours());
                }
            }
        }
        if rooted && names > 0 {
            Ok(())
        } else {
            Err(not_ours())
        }
    }

    /// See [`super::read_only`].
    pub(super) fn read_only(path: &Path) -> io::Result<File> {
        already_resolved(path)?;
        let opened =
            openat2(CWD, path, READING, Mode::empty(), STRICTLY).map_err(io::Error::from)?;
        Ok(File::from(opened))
    }

    /// See [`super::rename_no_replace`].
    ///
    /// `CWD` with two absolute paths, and not handles on the two folders. That
    /// is deliberate and [`super`] argues it: handles would mean opening the
    /// folders, and a turn's boundary permits opening only what its call named.
    pub(super) fn rename_no_replace(from: &Path, to: &Path) -> io::Result<()> {
        already_resolved(from)?;
        already_resolved(to)?;
        renameat_with(CWD, from, CWD, to, RenameFlags::NOREPLACE).map_err(io::Error::from)
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    //! What `std` allows, which is less, and the gap is in `docs/quirks.md`.

    use std::fs::{self, File};
    use std::io;
    use std::path::Path;

    /// See [`super::read_only`]. This resolves the name a second time, which is
    /// the race the Linux half exists to close.
    pub(super) fn read_only(path: &Path) -> io::Result<File> {
        File::open(path)
    }

    /// See [`super::rename_no_replace`]. A check and then a move, with a gap
    /// between them that no portable call removes.
    pub(super) fn rename_no_replace(from: &Path, to: &Path) -> io::Result<()> {
        // Anything at all, including a link: `symlink_metadata` answers about
        // the name rather than about what it leads to, which is the question.
        if fs::symlink_metadata(to).is_ok() {
            return Err(io::Error::from(io::ErrorKind::AlreadyExists));
        }
        fs::rename(from, to)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_folder_of_our_own;
    use std::fs;
    use std::io::Read as _;

    /// The ordinary day, on every platform: a file that is there is opened and
    /// holds what was written to it.
    #[test]
    fn a_file_that_is_there_is_opened_and_read() {
        let folder = a_folder_of_our_own("open");
        let file = folder.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();

        let mut held = Vec::new();
        read_only(&file).unwrap().read_to_end(&mut held).unwrap();
        assert_eq!(held, b"an invoice");

        let _ = fs::remove_dir_all(&folder);
    }

    /// A file that is not there is `NotFound` and not some other complaint,
    /// because [`crate::failed::Failed::machine`] turns exactly that into *it
    /// is gone* and everything else into the machine's own words.
    #[test]
    fn a_file_that_is_not_there_is_not_found() {
        let folder = a_folder_of_our_own("open-missing");

        let why = read_only(&folder.join("april.pdf")).unwrap_err();
        assert_eq!(why.kind(), io::ErrorKind::NotFound, "{why:?}");

        let _ = fs::remove_dir_all(&folder);
    }

    /// A move onto a name nothing holds happens, and a move onto one something
    /// holds is `AlreadyExists` — on every platform, by two different means.
    #[test]
    fn a_move_happens_unless_the_name_is_taken() {
        let folder = a_folder_of_our_own("no-replace");
        let file = folder.join("march.pdf");
        let free = folder.join("march-2026.pdf");
        let taken = folder.join("april.pdf");
        fs::write(&file, b"an invoice").unwrap();
        fs::write(&taken, b"somebody else's invoice").unwrap();

        rename_no_replace(&file, &free).unwrap();
        assert!(!file.exists());
        assert_eq!(fs::read(&free).unwrap(), b"an invoice");

        let why = rename_no_replace(&free, &taken).unwrap_err();
        assert_eq!(why.kind(), io::ErrorKind::AlreadyExists, "{why:?}");
        // And the refusal changed nothing at either end.
        assert_eq!(fs::read(&free).unwrap(), b"an invoice");
        assert_eq!(fs::read(&taken).unwrap(), b"somebody else's invoice");

        let _ = fs::remove_dir_all(&folder);
    }

    /// A name a folder already holds is taken whatever is under it — a folder
    /// and a link count, and a link counts even when it leads nowhere.
    ///
    /// Unix only for the link, because making one on Windows needs a privilege
    /// a developer's account may not have.
    #[test]
    fn a_folder_and_a_link_are_things_that_are_already_there() {
        let folder = a_folder_of_our_own("no-replace-kinds");
        let file = folder.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();
        let a_folder = folder.join("Archive");
        fs::create_dir_all(&a_folder).unwrap();

        let why = rename_no_replace(&file, &a_folder).unwrap_err();
        assert_eq!(why.kind(), io::ErrorKind::AlreadyExists, "{why:?}");
        assert!(a_folder.is_dir());

        #[cfg(unix)]
        {
            let dangling = folder.join("nowhere.pdf");
            std::os::unix::fs::symlink(folder.join("gone.pdf"), &dangling).unwrap();
            let why = rename_no_replace(&file, &dangling).unwrap_err();
            assert_eq!(why.kind(), io::ErrorKind::AlreadyExists, "{why:?}");
            assert!(fs::symlink_metadata(&dangling).unwrap().is_symlink());
        }

        assert_eq!(fs::read(&file).unwrap(), b"an invoice");
        let _ = fs::remove_dir_all(&folder);
    }

    /// A path this file will not walk is refused rather than walked as best it
    /// can. Everything reaching [`read_only`] is canonical, so a relative name,
    /// a bare root or a `..` means the caller is not the one this was written
    /// for — and guessing what it meant would be a second resolver beside the
    /// one the grants were asked about.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_path_that_is_not_canonical_is_refused_rather_than_walked() {
        for path in [
            Path::new("march.pdf"),
            Path::new("Invoices/march.pdf"),
            Path::new("./march.pdf"),
            Path::new("/"),
            Path::new("/home/anna/../anna/march.pdf"),
        ] {
            let why = read_only(path).unwrap_err();
            assert_eq!(
                why.kind(),
                io::ErrorKind::InvalidInput,
                "{} was walked",
                path.display()
            );
            let moving = rename_no_replace(path, Path::new("/tmp/nowhere")).unwrap_err();
            assert_eq!(
                moving.kind(),
                io::ErrorKind::InvalidInput,
                "{} was walked",
                path.display()
            );
        }
    }

    /// An interior `.` is not a path this refuses: `std` has already removed it
    /// by the time the walk sees the names, and the file it names is the file
    /// it would have named without it.
    #[cfg(target_os = "linux")]
    #[test]
    fn an_interior_dot_names_the_same_file_it_would_have_without_one() {
        let folder = a_folder_of_our_own("dot");
        let file = folder.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();

        let mut held = Vec::new();
        read_only(&folder.join(".").join("march.pdf"))
            .unwrap()
            .read_to_end(&mut held)
            .unwrap();
        assert_eq!(held, b"an invoice");

        let _ = fs::remove_dir_all(&folder);
    }
}
