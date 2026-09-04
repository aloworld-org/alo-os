//! What happens on an open, in ordinary Rust.
//!
//! Every open on the machine arrives here. The first thing asked is whether
//! this cgroup is a turn at all, and for a person's editor, browser and
//! terminal the answer is no and the function is over — one hash lookup, no
//! reads, nothing written down. Only when a turn is running does anything walk
//! anywhere.
//!
//! That order is deliberate and is the shape ADR 0015's discipline takes in
//! code: the expensive, revealing work is behind the question *is this an
//! agent*, so it cannot happen to somebody who is not being one.
//!
//! # The walk
//!
//! From the `struct file` the hook was handed, down to the directory entry the
//! open went through, then upwards: this entry, the directory it is in, the
//! directory that is in, until either a granted place is met or the top of
//! the filesystem is. [`alo_bounding_map::reaches`] is the rule; this file is
//! the fetching.
//!
//! There is **one** walk however many places a turn was bound to. Every step is
//! four reads of kernel memory and every comparison is two numbers, so the
//! places are asked at each step rather than walked for one at a time —
//! `alo_bounding_map::reaches` has the argument.
//!
//! **Anything unreadable refuses.** A missing offset, a null pointer, a read
//! the kernel declines — each ends the walk with [`None`], and
//! [`alo_bounding_map::reaches`] turns that into a refusal. A boundary that
//! guessed when it could not see would open under exactly the conditions
//! somebody would arrange on purpose.

use alo_bounding_map::{Bounds, Field, Place, reaches};

use crate::kernel;

/// The kernel's answer for an open that may go ahead.
const ALLOWED: i32 = 0;

/// The kernel's answer for an open that may not: `EACCES`.
///
/// A number rather than a constant from a header, because this program has no
/// headers — and it is the number ADR 0013 names, so a verb that overreached
/// fails the way a permission failure has always looked rather than in a way a
/// program would have to learn.
const REFUSED: i32 = -13;

/// Whether this open may go ahead.
pub fn decide(file: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn. This is the answer for every other process on the
        // machine, and nothing about it is remembered.
        return ALLOWED;
    };
    if inside(file, granted) {
        ALLOWED
    } else {
        REFUSED
    }
}

/// Whether the file this open is for lies at or under a granted place.
fn inside(file: u64, granted: Bounds) -> bool {
    let Some(fields) = Fields::found() else {
        return false;
    };
    // `f_path` is embedded in `struct file` rather than pointed at, so its
    // offset gives the address of the `struct path` itself, and the entry is
    // one step further in.
    let path = file.wrapping_add(fields.file_path);
    let Some(entry) = kernel::word_at(path.wrapping_add(fields.path_dentry)) else {
        return false;
    };

    let mut at = entry;
    let mut ended = false;
    reaches(granted, || {
        if ended {
            return None;
        }
        let here = fields.place_of(at)?;
        match kernel::word_at(at.wrapping_add(fields.dentry_parent)) {
            // A directory entry whose parent is itself is the top of a
            // filesystem: this is the last place there is to look at.
            Some(above) if above != 0 && above != at => at = above,
            _ => ended = true,
        }
        Some(here)
    })
}

/// Where this kernel keeps the fields the walk reads.
///
/// Fetched once per open rather than once per step, because a map lookup per
/// field per directory is the difference between a program the verifier is
/// comfortable with and one it is not.
struct Fields {
    /// `struct file`'s `f_path`.
    file_path: u64,
    /// `struct path`'s `dentry`.
    path_dentry: u64,
    /// `struct dentry`'s `d_parent`.
    dentry_parent: u64,
    /// `struct dentry`'s `d_inode`.
    dentry_inode: u64,
    /// `struct dentry`'s `d_sb`.
    dentry_super: u64,
    /// `struct inode`'s `i_ino`.
    inode_number: u64,
    /// `struct super_block`'s `s_dev`.
    super_device: u64,
}

impl Fields {
    /// The seven offsets, or [`None`] if the daemon did not put them there.
    ///
    /// [`None`] cannot happen on a machine whose boundary loaded — the daemon
    /// fills the map before it attaches the program — and it refuses rather
    /// than defaulting to zero, because zero is a real offset and would have
    /// the walk read the beginning of a `struct file` as though it were a
    /// directory entry.
    fn found() -> Option<Self> {
        Some(Self {
            file_path: kernel::offset(Field::FilePath)?,
            path_dentry: kernel::offset(Field::PathDentry)?,
            dentry_parent: kernel::offset(Field::DentryParent)?,
            dentry_inode: kernel::offset(Field::DentryInode)?,
            dentry_super: kernel::offset(Field::DentrySuper)?,
            inode_number: kernel::offset(Field::InodeNumber)?,
            super_device: kernel::offset(Field::SuperDevice)?,
        })
    }

    /// Which filesystem, and which inode, a directory entry names.
    fn place_of(&self, entry: u64) -> Option<Place> {
        let inode = kernel::word_at(entry.wrapping_add(self.dentry_inode))?;
        let filesystem = kernel::word_at(entry.wrapping_add(self.dentry_super))?;
        let number = kernel::word_at(inode.wrapping_add(self.inode_number))?;
        let device = kernel::half_word_at(filesystem.wrapping_add(self.super_device))?;
        Some(Place::of(u64::from(device), number))
    }
}
