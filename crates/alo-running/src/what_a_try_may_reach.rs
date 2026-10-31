//! What a command a person said *try* may reach, and nothing else.
//!
//! `docs/features.md`, starred: *Try a command without letting it touch
//! anything. Paste the install script from a README and **run it**.* The owner
//! decided the gesture on 2026-10-18 — **a word before the command, plus alo
//! offering when it sees a risk** — so this is the places half of that word.
//!
//! # *Touch anything* cannot mean literally nothing, and this says why
//!
//! `alo-bounding`'s boundary permits a path **at or under** a granted place and
//! refuses everything else. A box given only `./install.sh` therefore refuses
//! `/bin/sh`, refuses the dynamic loader, and the script does not start — so a
//! sealed box bound to nothing is not a strict promise kept, it is a command
//! that cannot run.
//!
//! **What a person actually wants from `curl … | sh` is that it cannot touch
//! *their* things**, and that is what this gives:
//!
//! | granted | why |
//! |---|---|
//! | the command itself | it has to be read to be run |
//! | [`WHERE_PROGRAMS_ARE`] | so an interpreter named on a `#!` line can be found and read |
//! | the library folder, from [`where_the_libraries_are`] | so a dynamically linked program can start at all |
//! | [`THE_EMPTY_DEVICE`] | so `>/dev/null` works, which is the most ordinary line in any script |
//!
//! **And nothing of the person's.** Not their home, not the folder they are
//! standing in, not the disk. A script run this way can read the system it
//! needs in order to run and can reach no file anybody wrote.
//!
//! *These three are the set `alo-running`'s own kernel test has used since
//! #729* — `a_command_reaches_what_its_box_was_given` grants the target,
//! `/usr/bin` and the library folder. This file is that set promoted out of a
//! test, because a set a product relies on and only a test states is a set
//! nobody maintains.
//!
//! # What is deliberately not here
//!
//! **A writable place.** Many install scripts want somewhere to unpack, and
//! giving them `/tmp` would hand them a folder other programs keep things in.
//! A box with nowhere to write refuses the write, which is the honest first
//! version of *touch anything*: a person sees the script fail saying it cannot
//! write, which is true and is the thing they asked for. **Somewhere of its own
//! to write is a promise to add deliberately**, not a default to slip in, and
//! `docs/features.md` says nothing about one.
//!
//! **The network — and it turns out there is nothing to add.** *`--offline` so
//! it may not reach the network* is the neighbouring promise, and **a boxed
//! command already cannot**. `alo_bounding_map::Bounds` holds where a turn may
//! connect to and *begins as nowhere*: default-deny, so every connection is
//! refused without anybody writing it down.
//!
//! **Measured on 2026-10-18 rather than taken from the comment.** The same
//! `curl` to an IP address, one difference between the two runs:
//!
//! ```text
//! unboxed   http=301   exit 0
//! in a box  http=000   exit 7     <- curl: could not connect
//! ```
//!
//! So a flag for it would be a flag that does nothing. What the promise asks
//! for beyond this is *and the indicator shows that it did not*, which is the
//! compositor's surface and is not reachable from a command — the same gap
//! [ADR 0125](../../../docs/decisions/0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md)
//! settles for an egress that happens, pointing the other way.

use std::path::{Path, PathBuf};

/// Where a machine keeps the programs an interpreter line can name.
pub const WHERE_PROGRAMS_ARE: &str = "/usr/bin";

/// The empty device, which almost every script opens.
///
/// **Measured on a real kernel on 2026-10-18 and not reasoned about.** A box
/// without it refused `>/dev/null`, which is the most ordinary line in any
/// shell script — and the first version of this crate's own `try` test
/// contained one, so the redirect failed, the command after it never ran, and
/// the test passed while measuring nothing. The run printed
/// `cannot create /dev/null: Permission denied` and that one line is the whole
/// evidence.
///
/// **`/dev/null` by name and never `/dev`.** The folder holds the disks, the
/// memory and every input device on the machine, and the boundary grants a
/// place and everything under it.
pub const THE_EMPTY_DEVICE: &str = "/dev/null";

/// The folders a machine's libraries are in, every one of which a dynamically
/// linked command may need.
///
/// **Both, and this used to be the first of them.** It read
/// `["/usr/lib", "/lib"]` and took whichever was a directory first — `/usr/lib`
/// everywhere. That is where Ubuntu keeps its loader and its C library, so the
/// kernel test passed on the development machine; **Fedora, which alo OS
/// ships on, keeps both in `/usr/lib64`**. Measured inside the image on
/// 2026-10-31, once, by hand: `/usr/bin/id` asks for
/// `/lib64/ld-linux-x86-64.so.2`, which resolves to
/// `/usr/lib64/ld-linux-x86-64.so.2`, outside the one folder granted. So on a
/// booted machine every boxed dynamically linked command was refused its own
/// loader and never started.
const WHERE_LIBRARIES_ARE: [&str; 2] = ["/usr/lib", "/usr/lib64"];

/// Where a machine whose `/usr` holds no libraries keeps them instead.
const WHERE_LIBRARIES_ONCE_WERE: &str = "/lib";

/// Why a machine cannot run a boxed command at all.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NoPlacesForIt {
    /// The command is not an absolute path.
    ///
    /// Resolved by whoever took it from the person, because only they know
    /// which folder the person was standing in. A relative path arriving here
    /// would be resolved against this process's folder, which is not theirs.
    #[error("{named} does not say where it is from the root of the machine")]
    NotAFullPath {
        /// What was asked for, as it was asked.
        named: String,
    },
    /// This machine has no library folder.
    ///
    /// Cannot happen on a machine that runs anything dynamically linked, which
    /// is every machine this product ships on. It is an error rather than a
    /// panic because a library inside a command must not take the command down
    /// with it.
    #[error("this machine has no library folder, so no dynamically linked command can run")]
    NoLibraryFolder,
}

/// The one folder every library this machine keeps is under, resolved.
///
/// **One place, because a box holds four** (`alo_bounding::PLACES`) and the
/// other three are spoken for. So where a machine has both `/usr/lib` and
/// `/usr/lib64` the place is the folder they share, `/usr` — the system's
/// own files, read-only on a booted alo OS, and nothing anybody wrote. Where it
/// has one, it is that one.
///
/// **Never `/`.** Only folders under `/usr` are joined, so the widest this can
/// be is `/usr`; a machine with no library under `/usr` falls back to `/lib`
/// alone, as it always did.
///
/// # Errors
/// [`NoPlacesForIt::NoLibraryFolder`] on a machine with none, which is not a
/// machine that runs anything.
pub fn where_the_libraries_are() -> Result<PathBuf, NoPlacesForIt> {
    // **Resolved, because the boundary compares inodes.** `/lib` as a link to
    // `/usr/lib` is a different place from `/usr/lib` as far as the kernel is
    // concerned, and granting the link grants nothing the loader opens.
    let found: Vec<PathBuf> = WHERE_LIBRARIES_ARE
        .into_iter()
        .map(Path::new)
        .filter(|path| path.is_dir())
        .filter_map(|path| path.canonicalize().ok())
        .collect();
    match found.split_first() {
        Some((first, rest)) => {
            let shared = rest
                .iter()
                .fold(first.clone(), |shared, next| the_folder_both_are_under(&shared, next));
            // Two folders that resolved somewhere unrelated share only `/`,
            // and granting `/` is granting everything. The first alone, then.
            Ok(if shared.parent().is_none() {
                first.clone()
            } else {
                shared
            })
        }
        None => Path::new(WHERE_LIBRARIES_ONCE_WERE)
            .canonicalize()
            .ok()
            .filter(|resolved| resolved.is_dir())
            .ok_or(NoPlacesForIt::NoLibraryFolder),
    }
}

/// The deepest folder two resolved paths are both at or under.
fn the_folder_both_are_under(one: &Path, other: &Path) -> PathBuf {
    one.components()
        .zip(other.components())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a)
        .collect()
}

/// Everywhere a command a person said *try* may reach.
///
/// Four places, in a settled order: the command, the programs folder, the
/// library folder, the empty device. See this module's header for why it is
/// four and not none.
///
/// **Four is `alo_bounding::PLACES`, so this is exactly full.** A fifth
/// place — somewhere of its own to write, most likely — cannot be added without
/// raising that bound, which is a change to the kernel side of the boundary.
/// Said here because the next person to want one will look at this function
/// first.
///
/// # Errors
/// [`NoPlacesForIt`] — a command that is not a full path, or a machine with no
/// library folder.
pub fn what_a_try_may_reach(program: &Path) -> Result<Vec<PathBuf>, NoPlacesForIt> {
    if !program.is_absolute() {
        return Err(NoPlacesForIt::NotAFullPath {
            named: program.display().to_string(),
        });
    }
    Ok(vec![
        program.to_path_buf(),
        PathBuf::from(WHERE_PROGRAMS_ARE),
        where_the_libraries_are()?,
        PathBuf::from(THE_EMPTY_DEVICE),
    ])
}

/// Everywhere a command a person said *try `--only-here`* may reach.
///
/// `docs/features.md`, starred: *Scope one command without changing a setting.
/// `--only-here` so it may not write outside this folder.*
///
/// The same three system places as [`what_a_try_may_reach`], with **the folder
/// the person is standing in** in place of the command. Four places, which is
/// `alo_bounding::PLACES` — so this is a swap and not an addition, and the
/// command itself needs no place of its own when it lives in that folder.
///
/// # It is a loosening, and `try` alone is the strict one
///
/// Said plainly because the flag's name suggests the opposite. `try` on its own
/// grants the command and nothing else of the person's; `--only-here` **adds**
/// their current folder, so a build can write its output where they are
/// standing. A person reaching for it is asking for more, not less, and what
/// they are narrowing is their whole session down to one folder.
///
/// # And it is stricter than the promise says
///
/// The promise is *may not **write** outside this folder*. A place in
/// `alo-bounding` is read and write together, so what this actually gives is
/// *may not **touch** outside this folder* — the command cannot read the
/// person's other files either. **Stricter than promised rather than looser**,
/// and recorded here rather than left for somebody to notice: a reader
/// comparing the two will otherwise think one of them is wrong.
///
/// # Errors
/// [`NoPlacesForIt`] — a folder that is not a full path, or a machine with no
/// library folder.
pub fn what_only_here_may_reach(folder: &Path) -> Result<Vec<PathBuf>, NoPlacesForIt> {
    if !folder.is_absolute() {
        return Err(NoPlacesForIt::NotAFullPath {
            named: folder.display().to_string(),
        });
    }
    Ok(vec![
        folder.to_path_buf(),
        PathBuf::from(WHERE_PROGRAMS_ARE),
        where_the_libraries_are()?,
        PathBuf::from(THE_EMPTY_DEVICE),
    ])
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected Err, or an index past a length the test just \
              asserted, is the failure being reported"
)]
#[path = "what_a_try_may_reach_tests.rs"]
mod tests;
