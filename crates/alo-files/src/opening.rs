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
//! # A rename holds its two folders instead, and why that is allowed
//!
//! `renameat2` has no `RESOLVE_NO_SYMLINKS`, so the trick above does not work
//! twice. What it does take is two directory handles, and the folders on the
//! way to either name cannot be exchanged once they are held rather than named.
//!
//! Taking a handle means opening something, which is the boundary again — and
//! the folder a `move_file` takes a file *out of* is not a place its call
//! named. **`O_PATH` is what makes it possible, and it was measured rather than
//! assumed** (item 6c). `alo-bounding`'s `what_an_o_path_handle_is` puts the
//! question to a running kernel with the real programme loaded, and the answer
//! has four parts:
//!
//! - an `O_PATH` open of a folder nobody granted **succeeds** — Linux does not
//!   run `security_file_open` for one, so the boundary never sees it;
//! - opening a file **through** that handle is refused with `EACCES`, because
//!   the boundary walks up from the *file's own* directory entry and does not
//!   care which handle it was reached from;
//! - so is turning the handle back into a readable one through `/proc/self/fd`,
//!   which is the way round somebody would actually try;
//! - and `renameat2` accepts the handles.
//!
//! So an `O_PATH` handle is a reference to a place that confers no reading —
//! exactly the authority needed to move a name, and none of the authority the
//! boundary exists to withhold. **Nothing here widens what a turn may reach**,
//! and the test asserts all four parts so that a kernel which changes any of
//! them fails rather than quietly downgrades.
//!
//! **The argument does not rest on renames being unwatched, and they no longer
//! are.** When this was written the kernel had no hook on `inode_rename` and a
//! rename of an ungranted file simply succeeded; the boundary now watches both
//! ends of one, refusing a source or a destination outside the bound. Nothing
//! here changed when it did, because what makes an `O_PATH` handle acceptable
//! is that it confers no reading — not that nobody was looking. The handles
//! this file takes are on folders the call named, so the moves it makes are the
//! ones the kernel permits.
//!
//! The final component of each name is not followed, which is `renameat2`'s own
//! behaviour and the right one: a link put where the file was is moved as the
//! link it is, rather than being reached through.
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

/// Why a file was not opened for reading.
///
/// Two different facts, and they are not both the machine's: one is what a
/// syscall answered, and the other is this crate declining to read a file whose
/// contents may also live somewhere nobody granted.
#[derive(Debug)]
pub(crate) enum Opening {
    /// The machine said no, in its own words.
    TheMachine(io::Error),

    /// The file has more than one name on this machine, and where the others
    /// are cannot be asked of it.
    MoreThanOneName,
}

impl From<io::Error> for Opening {
    fn from(why: io::Error) -> Self {
        Self::TheMachine(why)
    }
}

impl Opening {
    /// Which kind of thing the machine said, when it was the machine.
    ///
    /// Only for this file's own tests. Everywhere else the whole point is that
    /// these two are not the same kind of fact and are not asked about as
    /// though they were: [`crate::failed::Failed::opening`] is the one place
    /// that tells them apart, and it does it by matching rather than by asking.
    #[cfg(test)]
    fn what_the_machine_said(&self) -> Option<io::ErrorKind> {
        match self {
            Self::TheMachine(why) => Some(why.kind()),
            Self::MoreThanOneName => None,
        }
    }
}

/// Open a file for reading: not through a link, and not one of several names.
///
/// On Linux the whole path is resolved inside one syscall that refuses a link
/// at every component, so nothing on the way can be exchanged between the check
/// and the open. Elsewhere it is `File::open`, which resolves the name once
/// more.
///
/// **Then the open file is asked how many names it has**, which is a question
/// only a handle can answer honestly — the same discipline as asking a handle
/// how big a file is rather than asking its name again. More than one, and it
/// is not read: see [`Opening::MoreThanOneName`] and [`more_than_one_name`].
///
/// # Errors
/// [`Opening::TheMachine`] with whatever the machine said. A component that has
/// become a symbolic link is `ELOOP`, which reaches a person as a refusal
/// rather than as a file somebody else chose; a kernel too old to promise this
/// is `ENOSYS`, which refuses the read rather than doing it the old way. Or
/// [`Opening::MoreThanOneName`], which is this crate's own answer rather than
/// the machine's.
pub(crate) fn read_only(path: &Path) -> Result<File, Opening> {
    let opened = platform::read_only(path)?;
    if more_than_one_name(&opened)? {
        return Err(Opening::MoreThanOneName);
    }
    Ok(opened)
}

/// Whether this machine knows this open file by more than one name.
///
/// # Why it is asked of the open file rather than of the path
///
/// A **hard link** is a second real name for one file. Nothing about a path
/// reveals it: `alo-files` resolves every path a verb names and asks the grants
/// where it really leads, and a hard link inside a granted folder to a file
/// that also lives outside it resolves to the granted name and passes, because
/// the granted name *genuinely is* a real name for that file. There is nothing
/// a cleverer path comparison could do, and `docs/quirks.md` has said so since
/// 2026-09-02.
///
/// What a file will answer is how many names it has, and a handle is what to
/// ask: the count is read from the file that was opened rather than from the
/// name it was opened by, so nothing swapped in afterwards changes the answer.
///
/// # It cannot say where the other names are, and that decides the policy
///
/// There is no way from a file to its own names — that would be a scan of every
/// filesystem it could be on. So *more than one name* is as much as is known,
/// and the two ways of being wrong are: read a file whose contents also live
/// outside the grant, or refuse a file whose second name is inside the grant
/// and harmless. The second is the safe one and it is the one taken. What that
/// costs is in `docs/quirks.md`.
///
/// # Only regular files
///
/// Every directory has at least two names — its own and the `.` inside it —
/// and one with subdirectories has one more for each. Counting names on a
/// directory would refuse every folder on the machine, which is why this asks
/// what the file is first.
#[cfg(unix)]
fn more_than_one_name(opened: &File) -> Result<bool, Opening> {
    use std::os::unix::fs::MetadataExt as _;

    let what = opened.metadata().map_err(Opening::TheMachine)?;
    Ok(what.is_file() && what.nlink() > 1)
}

/// See the Unix half. `std` cannot count a file's names on Windows without a
/// call it does not expose, so this answers *not that we can tell* — which is
/// the honest answer and is the gap `docs/quirks.md` keeps.
#[cfg(not(unix))]
fn more_than_one_name(_: &File) -> Result<bool, Opening> {
    Ok(false)
}

/// Move a name, refusing to replace anything already at the destination.
///
/// On Linux this is one syscall that both checks and moves, so there is no
/// moment in between, and it is made from handles on the two folders rather
/// than from their names, so neither can be exchanged either. Elsewhere it is a
/// check and then a move, by name, and the gap is the one `docs/quirks.md`
/// records.
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

    use std::ffi::OsStr;
    use std::fs::File;
    use std::io;
    use std::os::fd::OwnedFd;
    use std::path::{Component, Path};

    use rustix::fs::{CWD, Mode, OFlags, RenameFlags, ResolveFlags, openat2, renameat_with};

    /// Opening the file: read-only, closed on exec, and not a link itself.
    const READING: OFlags = OFlags::RDONLY
        .union(OFlags::CLOEXEC)
        .union(OFlags::NOFOLLOW);

    /// Referring to a folder without opening it: a place a name can be moved in
    /// or out of, and nothing that can be read, written or listed.
    const REFERRING: OFlags = OFlags::PATH.union(OFlags::CLOEXEC).union(OFlags::DIRECTORY);

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

    /// A reference to the folder a path's last name is in, and that name.
    ///
    /// `O_PATH` is the whole point: it is a handle on a *place* rather than an
    /// open file — nothing can be read, written or listed through one — and
    /// [`super`] gives the measured reason that matters, which is that the
    /// boundary neither sees it nor needs to. `RESOLVE_NO_SYMLINKS` is what
    /// makes it worth taking: the folder is reached by a path no component of
    /// which was a link, and from then on it is held rather than named.
    fn folder_holding(path: &Path) -> io::Result<(OwnedFd, &OsStr)> {
        let (Some(folder), Some(name)) = (path.parent(), path.file_name()) else {
            return Err(not_ours());
        };
        let held =
            openat2(CWD, folder, REFERRING, Mode::empty(), STRICTLY).map_err(io::Error::from)?;
        Ok((held, name))
    }

    /// See [`super::rename_no_replace`].
    ///
    /// Both folders are held rather than named, so neither can be exchanged
    /// between being resolved and being renamed in — and the two names are the
    /// last components, which `renameat2` never follows.
    pub(super) fn rename_no_replace(from: &Path, to: &Path) -> io::Result<()> {
        already_resolved(from)?;
        already_resolved(to)?;
        let (from_folder, from_name) = folder_holding(from)?;
        let (to_folder, to_name) = folder_holding(to)?;
        renameat_with(
            &from_folder,
            from_name,
            &to_folder,
            to_name,
            RenameFlags::NOREPLACE,
        )
        .map_err(io::Error::from)
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

    /// **A file the machine knows by two names is not read.**
    ///
    /// The second name is made *outside* the folder being read from, which is
    /// the shape that matters: what a person granted is one of the names, and
    /// the file's contents are equally reachable by the other.
    #[cfg(unix)]
    #[test]
    fn a_file_with_a_second_name_is_not_read() {
        let folder = a_folder_of_our_own("two-names");
        let granted = folder.join("Invoices");
        let elsewhere = folder.join("Elsewhere");
        fs::create_dir_all(&granted).unwrap();
        fs::create_dir_all(&elsewhere).unwrap();

        let outside = elsewhere.join("secret.txt");
        fs::write(&outside, b"not an invoice").unwrap();
        let inside = granted.join("notes.txt");
        fs::hard_link(&outside, &inside).unwrap();

        let why = read_only(&inside).unwrap_err();
        assert!(
            matches!(why, Opening::MoreThanOneName),
            "a file with another name was opened: {why:?}"
        );
        // And it is this crate's own answer rather than something the machine
        // said, which is the distinction `Opening` exists to keep.
        assert_eq!(why.what_the_machine_said(), None);

        let _ = fs::remove_dir_all(&folder);
    }

    /// The two names taken apart again: the moment one of them is gone the file
    /// has one name and is read, so what is refused is the *sharing* rather
    /// than anything about the file itself.
    #[cfg(unix)]
    #[test]
    fn the_same_file_is_read_once_it_has_only_one_name() {
        let folder = a_folder_of_our_own("one-name-again");
        let outside = folder.join("secret.txt");
        let inside = folder.join("notes.txt");
        fs::write(&outside, b"an invoice").unwrap();
        fs::hard_link(&outside, &inside).unwrap();
        assert!(read_only(&inside).is_err());

        fs::remove_file(&outside).unwrap();

        let mut held = Vec::new();
        read_only(&inside).unwrap().read_to_end(&mut held).unwrap();
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
        assert_eq!(
            why.what_the_machine_said(),
            Some(io::ErrorKind::NotFound),
            "{why:?}"
        );

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
                why.what_the_machine_said(),
                Some(io::ErrorKind::InvalidInput),
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
