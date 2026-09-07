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

//! # What is watched, and what is not
//!
//! Four hooks: what a turn **opens**, **moves**, **removes**, and gives a
//! **second name**. That is not the whole of a filesystem and this file does
//! not pretend it is. Nothing here watches:
//!
//! - **symbolic links** (`inode_symlink`) — a turn can make one pointing
//!   anywhere. It is not a way out on its own: following it to read something
//!   is a `file_open`, which is watched, and `alo-files` refuses a path with a
//!   link in it. It is a way to leave a **name** somewhere, which is not a way
//!   to leave *contents*;
//! - **directories** (`inode_mkdir`, `inode_rmdir`) — a turn can make and
//!   remove empty ones. Removing a directory with anything in it needs the
//!   contents gone first, and that is `inode_unlink`;
//! - **making a file** (`inode_create`) — a turn can create one. Writing to it
//!   is an open, which is watched, so what this leaves is an empty file
//!   somewhere;
//! - **what is inside a file already open** — a boundary on `file_open`
//!   decides at the moment of opening and says nothing afterwards;
//! - **signals and memory**, and everything else that is not a filesystem. What
//!   a turn connects to *is* watched, by `socket_connect`, and
//!   [`decide_departure`] says what that does and does not decide.
//!
//! Each of those is a real gap and each is written down rather than left to be
//! discovered. What they have in common is that none of them moves a byte of
//! somebody's file to somewhere they did not approve, which is the property the
//! four hooks that exist were chosen for.

use alo_bounding_map::{Bounds, Departure, Family, Field, Place, reaches};

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

/// Whether this rename may go ahead.
///
/// A rename gives a file a different name, so it is [`a_name_being_made`] with
/// the file as the source: both ends have to be inside, and a rename out of a
/// granted folder into an ungranted one — or the reverse — is refused.
pub fn decide_rename(old_entry: u64, new_entry: u64) -> i32 {
    a_name_being_made(old_entry, new_entry)
}

/// Whether this hard link may be made.
///
/// A link gives a file a **second** name, and it is the same question a rename
/// asks: this file, into that folder. The difference is only that the first
/// name stays, which makes it the cheaper of the two ways to put somebody's
/// document somewhere they did not approve — it costs no bytes and survives
/// everything the turn does afterwards.
///
/// `docs/quirks.md` records the other end of this: a hard link inside a granted
/// folder is inside every check a path can make, because the granted name
/// genuinely is a name for that file, and `alo-files` answers that by refusing
/// to *read* a file the machine knows by more than one name. This is a turn
/// being unable to **make** one. Neither replaces the other — a link somebody
/// else made before the turn began is still only answered by the reading rule.
pub fn decide_link(old_entry: u64, new_entry: u64) -> i32 {
    a_name_being_made(old_entry, new_entry)
}

/// Whether this turn may put **this file** under a name in **that folder**.
///
/// # Two entries, and they are not asked the same question
///
/// What a turn is bound to for a move is *the file, and the folder it is going
/// into*; for a rename it is *the file, and the folder it sits in*. That is
/// `alo_files::Reaching` read back rather than this file being clever, and it
/// decides the shape:
///
/// - the **source** is asked about the entry itself, because the file is what
///   the call named and what a bound is made of;
/// - the **destination** is asked about the entry's **parent**, because the
///   destination usually does not exist yet — a no-clobber rename and every
///   link are a name nothing is at — and a directory entry with no inode has no
///   place to be asked about. The folder it would be made in is what the call
///   named, and it is there.
///
/// Asking the source's *parent* instead would refuse every legitimate move: the
/// folder a `move_file` takes a file out of is not a place its call named, and
/// widening a bound to include it is the thing this must not do.
fn a_name_being_made(old_entry: u64, new_entry: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every rename and link on the machine.
        return ALLOWED;
    };
    let Some(fields) = Fields::found() else {
        return REFUSED;
    };
    let Some(new_folder) = kernel::word_at(new_entry.wrapping_add(fields.dentry_parent)) else {
        return REFUSED;
    };
    if upwards_from(old_entry, &fields, granted) && upwards_from(new_folder, &fields, granted) {
        ALLOWED
    } else {
        REFUSED
    }
}

/// Where a `sockaddr` keeps the family, which every one of them has first.
const FAMILY_AT: u64 = 0;

/// Where both kinds keep the port, immediately after the family.
const PORT_AT: u64 = 2;

/// Where a `sockaddr_in` keeps its four bytes of address.
const INET_ADDRESS_AT: u64 = 4;

/// Where a `sockaddr_in6` keeps its sixteen, past the flow label.
const INET6_ADDRESS_AT: u64 = 8;

/// Whether this connection may be made.
///
/// # What is decided here, and the much larger thing that is not
///
/// **`alo-egress`'s policy is not enforced here and cannot be.** That policy
/// decides by provider and by region; this program sees a control group, a
/// family and an address. `alo-bounding`'s own documentation argues it at
/// length. What is enforced is the sentence that policy makes true:
///
/// > A turn opens no socket unless the person has been shown that it is about
/// > to.
///
/// So a destination the person was shown is permitted and **one they were not
/// is refused, even while another departure of the same turn is open**. One
/// departure is one address and one port, never permission to connect.
///
/// # Four answers, and the reason each is the answer
///
/// - **Not a turn** — allowed, and nothing is remembered. Every other process
///   on this machine, including the person's own browser and this service's own
///   errands, which are not turns.
/// - **A family this cannot read the address of** — refused. It is a
///   destination that cannot be checked against what somebody was shown, and
///   that is the direction to fail in.
/// - **A family that is not a network address at all** — allowed, because a
///   Unix socket is not egress and refusing it would be enforcing something no
///   policy claims. `alo-egress` decides about what leaves the machine, and a
///   local socket does not.
/// - **Loopback** — allowed, and this is the one that deserves saying out loud.
///   ADR 0007 makes a model on this machine the default; `alo-egress`'
///   `Leaving::asking` answers that a question answered here is not a departure
///   at all, so nothing is shown and nothing would ever be written for it.
///   Refusing loopback would break the ordinary case the whole product is built
///   around.
///
/// **What that last one costs is real and is not closed here.**
/// `docs/quirks.md` records that a proxy listening on loopback would be
/// believed by every type in this repository, and says the place it is caught
/// is egress enforcement at the network boundary. **This is not that place.**
/// This is turn-scoped: it decides what a *turn* connects to, and a proxy
/// somebody else started is not a turn, so its own outward connection passes
/// this program untouched. The quirk's forward reference is corrected rather
/// than left to read as answered.
pub fn decide_departure(where_to: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every connection on the machine.
        return ALLOWED;
    };
    let Some(family) = kernel::quarter_word_at(where_to.wrapping_add(FAMILY_AT)) else {
        return REFUSED;
    };
    let Some(family) = Family::of(family) else {
        // Not a network address, so not egress, so not this program's to
        // decide about.
        return ALLOWED;
    };
    let Some(port) = kernel::quarter_word_at(where_to.wrapping_add(PORT_AT)) else {
        return REFUSED;
    };
    let Some(address) = address_of(family, where_to) else {
        return REFUSED;
    };
    if stays_on_this_machine(family, address) {
        return ALLOWED;
    }
    if granted.may_leave(Departure::of(family, address, u16::from_be(port))) {
        ALLOWED
    } else {
        REFUSED
    }
}

/// The address in a `sockaddr`, in host order, as far as it can be read.
///
/// A `sockaddr_in` and a `sockaddr_in6` have layouts the standard fixes rather
/// than layouts a kernel chooses, so these offsets are not looked up the way
/// `struct file`'s are — there is nothing about them that moves between
/// kernels.
fn address_of(family: Family, where_to: u64) -> Option<u128> {
    match family {
        Family::Four => {
            let four = kernel::half_word_at(where_to.wrapping_add(INET_ADDRESS_AT))?;
            Some(u128::from(u32::from_be(four)))
        }
        Family::Six => {
            let high = kernel::word_at(where_to.wrapping_add(INET6_ADDRESS_AT))?;
            let low = kernel::word_at(where_to.wrapping_add(INET6_ADDRESS_AT + 8))?;
            Some((u128::from(u64::from_be(high)) << 64) | u128::from(u64::from_be(low)))
        }
    }
}

/// Whether this address is one that never leaves the machine.
///
/// `127.0.0.0/8` and `::1`. Nothing here treats `::ffff:127.0.0.1` as loopback,
/// and deliberately: it arrives as an IPv6 address and is compared as one, so a
/// turn reaching it is checked against what somebody was shown like any other
/// destination. Being stricter than necessary about a mapped address is the
/// safe direction.
const fn stays_on_this_machine(family: Family, address: u128) -> bool {
    match family {
        Family::Four => address >> 24 == 127,
        Family::Six => address == 1,
    }
}

/// Whether this name may be removed.
///
/// One entry rather than two, and it is **the entry being removed** — not the
/// folder it is in, for the reason the source of a rename is not judged by its
/// folder either: a grant can be over a single file, and its folder is then not
/// a place the call named.
///
/// # Inside the bound is not the same as authorised, and this is the floor
///
/// **None of the six verbs deletes anything.** A turn removing a name is
/// therefore a verb with a bug in it, and `alo-capability` is what refuses such
/// a call long before a syscall. What this adds is ADR 0013's floor: a bug like
/// that cannot reach outside the places the call itself named. Reading it as
/// *deleting inside a granted folder is allowed* would be reading it as the
/// capability model, which it is not and does not replace.
///
/// One legitimate delete does exist and it is why this is judged by the entry:
/// when writing an archive fails part of the way through, `alo-files` removes
/// the half-written file it made, in the folder the archive was going into —
/// which is a place that call named.
pub fn decide_delete(entry: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every delete on the machine.
        return ALLOWED;
    };
    let Some(fields) = Fields::found() else {
        return REFUSED;
    };
    if upwards_from(entry, &fields, granted) {
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
    upwards_from(entry, &fields, granted)
}

/// Whether a granted place is met walking up from this directory entry.
///
/// The whole of the walk, once, for the two hooks that need it: this entry,
/// the directory it is in, the directory that is in, until either a granted
/// place is met or the top of the filesystem is. Two copies of it would be two
/// answers to *is this inside* waiting to disagree, and one of them would be on
/// the hook nobody was looking at.
fn upwards_from(entry: u64, fields: &Fields, granted: Bounds) -> bool {
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
