//! The only file here that talks to the kernel, and the only one with `unsafe`
//! in it.
//!
//! `alo-agentd`'s `unix.rs` is the shape this follows: what the kernel will
//! only be asked for through `unsafe` is asked in one file, named there, and
//! every other file in the crate is held to the rule. There are four such
//! things and no others.
//!
//! **The symbol the kernel attaches to.** `#[lsm(hook = "file_open")]` puts an
//! exported name in a link section the loader looks for. Exporting a symbol is
//! `unsafe` in this edition because the compiler can no longer prove nothing
//! else claims the name.
//!
//! **The argument.** An LSM hook is called with the arguments of the kernel
//! function it stands in front of. `security_file_open(struct file *file)` has
//! one, and a second the kernel adds: whatever the previous LSM on this path
//! decided.
//!
//! **A read of kernel memory.** `bpf_probe_read_kernel` is how a BPF program
//! dereferences anything, and it is unsafe for the ordinary reason: it is given
//! an address. What makes it *safe* here is not a proof by the compiler — it is
//! that the read is a helper call which fails rather than faults, and that the
//! kernel's verifier has already refused to load this program if it could read
//! somewhere it should not.
//!
//! **A lookup in a map.** `HashMap::get` is unsafe in `aya-ebpf` because a map
//! this program only reads and the daemon only writes could, in general, have
//! an entry removed underneath the reference. Here the value is copied out
//! before anything else happens, so the reference does not outlive the lookup.
//!
//! # What is not here
//!
//! No writing. Nothing in this file puts anything into a map, a ring buffer, a
//! counter or a log line, and there is no `bpf_printk`. That is ADR 0015's
//! *the LSM decides and forgets*, and the absence is the whole of it: a program
//! with nowhere to write cannot become a record of somebody's day.
//!
//! The two maps below are read here and written only by the daemon, and there
//! are two. `alo-bounding`'s `tests/the_boundary_decides_and_forgets.rs` counts
//! them on a running kernel, so a third — however it is named and whatever kind
//! it is — fails a test rather than passing a review, and so does a
//! `bpf_printk` added to the hook.

// The one module in this crate the rule is lifted for. Everything the four
// paragraphs above describe is below; nothing else in the package may.
#![allow(
    unsafe_code,
    reason = "a BPF program is a raw pointer from the kernel and helper calls on it; \
              the crate root has the whole argument"
)]

use aya_ebpf::{
    helpers::{bpf_get_current_cgroup_id, bpf_probe_read_kernel},
    macros::{lsm, map},
    maps::{Array, HashMap},
    programs::LsmContext,
};

use alo_bounding_map::{Bounds, Field, WORDS};

use crate::deciding;

/// Which turn may reach where: the cgroup a turn runs in, against the places it
/// was granted.
///
/// The daemon writes an entry when a turn begins and takes it out when the turn
/// ends, and a cgroup with no entry is a program that is not an agent turn —
/// which is every other process on the machine.
///
/// The value is [`WORDS`] words rather than two, because a turn is bound to
/// several places: one execution names more than one path, and
/// `alo_bounding_map::Bounds` is where the layout of them is decided.
#[map(name = "BOUNDS")]
static BOUNDS: HashMap<u64, [u64; WORDS]> = HashMap::with_max_entries(1024, 0);

/// Where the fields this program reads sit in this kernel's own structures.
///
/// Filled by the daemon out of `/sys/kernel/btf/vmlinux` before the program is
/// attached, so nothing here is compiled against a kernel version. `Field` is
/// the agreement about which slot is which.
///
/// Sixteen slots for fourteen fields. The spare ones are read back by
/// `the_boundary_decides_and_forgets` and held at zero, because an array this
/// program can already reach is exactly where a counter would sit.
#[map(name = "FIELDS")]
static FIELDS: Array<u32> = Array::with_max_entries(16, 0);

/// Every open of every file, on this machine, from now until the program is
/// detached.
///
/// It is worth reading that sentence twice, because it is the reason ADR 0015
/// calls this the most dangerous thing in the repository. What the function
/// does with that reach is look up one cgroup and, almost always, find nothing
/// and return.
#[lsm(hook = "file_open")]
pub fn file_open(ctx: LsmContext) -> i32 {
    // `security_file_open(struct file *file)`, and after the hook's own
    // arguments the kernel appends what the previous LSM on this path decided.
    let file: u64 = ctx.arg(0);
    let already: i32 = ctx.arg(1);
    if already != 0 {
        // Somebody else has already refused. Ours is not the decision that
        // matters, and turning a refusal into an allow is not something an
        // additional security module may do.
        return already;
    }
    deciding::decide(file)
}

/// Every rename of every file, on this machine, until the program is detached.
///
/// The second hook, and the reason there is one: what a turn **opens** has been
/// watched since the boundary was first loaded and what it **moves** was not.
/// So a file nobody granted could be renamed into a granted folder, and read
/// from there, past a boundary that had no complaint about either step. ADR
/// 0015 named `inode_rename` beside `file_open` in its own mechanism; only one
/// of the two had been built.
///
/// The hook is
/// `inode_rename(struct inode *old_dir, struct dentry *old_dentry,
/// struct inode *new_dir, struct dentry *new_dentry)`. The two **inodes** are
/// handed over and are not read: an inode does not say where it is, and this
/// program decides by walking up a chain of directory entries. The entries are
/// what carry that chain, so the entries are what it takes.
#[lsm(hook = "inode_rename")]
pub fn inode_rename(ctx: LsmContext) -> i32 {
    let old_entry: u64 = ctx.arg(1);
    let new_entry: u64 = ctx.arg(3);
    // As with `file_open`, the kernel appends what the previous LSM decided
    // after the hook's own arguments — the fifth here, because there are four.
    let already: i32 = ctx.arg(4);
    if already != 0 {
        // Somebody else has already refused, and turning a refusal into an
        // allow is not something an additional security module may do.
        return already;
    }
    deciding::decide_rename(old_entry, new_entry)
}

/// Every removal of every name, on this machine, until the program is detached.
///
/// `inode_unlink(struct inode *dir, struct dentry *dentry)` — two arguments, so
/// the previous module's decision is the third. The directory is handed over
/// and is not read: what is being destroyed is the entry, and the entry is what
/// carries the chain this program walks.
#[lsm(hook = "inode_unlink")]
pub fn inode_unlink(ctx: LsmContext) -> i32 {
    let entry: u64 = ctx.arg(1);
    let already: i32 = ctx.arg(2);
    if already != 0 {
        return already;
    }
    deciding::decide_delete(entry)
}

/// Every hard link made on this machine, until the program is detached.
///
/// `inode_link(struct dentry *old_dentry, struct inode *dir,
/// struct dentry *new_dentry)` — three arguments, so the previous module's
/// decision is the fourth, and the **directory sits between the two entries**
/// rather than beside them. Reading the arguments in a rename's order would
/// take the destination folder's inode for the new entry, which is a pointer to
/// the wrong kind of structure and refuses everything for reasons nobody can
/// see; `docs/quirks.md` has that trap written down.
#[lsm(hook = "inode_link")]
pub fn inode_link(ctx: LsmContext) -> i32 {
    let old_entry: u64 = ctx.arg(0);
    let new_entry: u64 = ctx.arg(2);
    let already: i32 = ctx.arg(3);
    if already != 0 {
        return already;
    }
    deciding::decide_link(old_entry, new_entry)
}

/// Every connection made on this machine, until the program is detached.
///
/// `socket_connect(struct socket *sock, struct sockaddr *address, int addrlen)`
/// — three arguments, so the previous module's decision is the fourth. The
/// socket itself is handed over and is not read: what decides is **where the
/// connection is going**, and that is the address.
///
/// This is the other half of law 1. The four hooks before it are about what a
/// turn reaches on a disk; this is about what it reaches off the machine, and
/// [`crate::deciding::decide_departure`] argues what may and may not be decided
/// here.
#[lsm(hook = "socket_connect")]
pub fn socket_connect(ctx: LsmContext) -> i32 {
    let where_to: u64 = ctx.arg(1);
    let already: i32 = ctx.arg(3);
    if already != 0 {
        return already;
    }
    deciding::decide_departure(where_to)
}

/// Every message sent on every socket on this machine, until the program is
/// detached.
///
/// `socket_sendmsg(struct socket *sock, struct msghdr *msg, int size)` —
/// three arguments, so the previous module's decision is the fourth. The size
/// is not read: what decides is where the bytes are going, never how many
/// there are, and this program does not look at the bytes themselves.
///
/// The sixth hook, and the one that makes the fifth whole. `socket_connect`
/// decides when a connection is *made*, so a socket joined before the turn
/// began was never asked, and a datagram sent without joining anything asks
/// nothing at all. This runs on the message, which is the moment the bytes
/// actually leave — and it is asked of the **sending thread's** control group,
/// so a socket the daemon opened outside any turn is still the turn's to
/// answer for the moment a turn writes on it.
/// [`crate::deciding::decide_message`] says what may be decided here and what
/// may not.
#[lsm(hook = "socket_sendmsg")]
pub fn socket_sendmsg(ctx: LsmContext) -> i32 {
    let socket: u64 = ctx.arg(0);
    let message: u64 = ctx.arg(1);
    let already: i32 = ctx.arg(3);
    if already != 0 {
        return already;
    }
    deciding::decide_message(socket, message)
}

/// Every read and every write of every file, on this machine, until the
/// program is detached.
///
/// `file_permission(struct file *file, int mask)` — two arguments, so the
/// previous module's decision is the third. The mask is not read: a read and
/// a write through a descriptor outside the bound are refused alike, and the
/// two kinds of file that hold no contents of their own — a socket, whose
/// writes another hook decides, and a pipe — are told apart by what they
/// *are* rather than by what is being done to them.
///
/// The seventh hook, and the one that reaches a descriptor the six before it
/// never could. `file_open` decides when a file is *opened*, so a descriptor
/// opened before the turn began was never asked about: the daemon's own
/// record, its way out of a turn, and whatever else it had open. This runs on
/// the use, which is the moment the bytes move — every `read`, `write`,
/// `sendfile`, `splice` and `getdents` on the machine — and it is asked of
/// the **reading or writing thread's** control group, so a descriptor the
/// daemon opened outside any turn is the turn's to answer for the moment the
/// turn reads or writes through it. [`crate::deciding::decide_use`] says what
/// may be decided here and what may not.
#[lsm(hook = "file_permission")]
pub fn file_permission(ctx: LsmContext) -> i32 {
    let file: u64 = ctx.arg(0);
    let already: i32 = ctx.arg(2);
    if already != 0 {
        return already;
    }
    deciding::decide_use(file)
}

/// Every change to a file's size, mode, owner or times, on this machine,
/// until the program is detached.
///
/// `inode_setattr(struct mnt_idmap *idmap, struct dentry *dentry,
/// struct iattr *attr)` — three arguments, so the previous module's decision
/// is the fourth, and **the entry is the second** rather than the first: the
/// mapping the mount applies to owners comes before it, which is a shape the
/// kernel gave every attribute hook in 6.9 and which `docs/quirks.md` records
/// beside the rename hook's trap. The attributes themselves are not read: a
/// change to any of them on a file outside the grant is refused alike, and
/// reading which one would only be a reason to allow some.
///
/// The eighth hook, and the one the attribute hooks were added for: this is
/// the one `truncate(2)` reaches without an open, so it is the one that let a
/// bound turn empty a file it could not read.
/// [`crate::deciding::decide_attribute`] says what is decided here and why.
#[lsm(hook = "inode_setattr")]
pub fn inode_setattr(ctx: LsmContext) -> i32 {
    let entry: u64 = ctx.arg(1);
    let already: i32 = ctx.arg(3);
    if already != 0 {
        return already;
    }
    deciding::decide_attribute(entry)
}

/// Every extended attribute set on this machine, until the program is
/// detached.
///
/// `inode_setxattr(struct mnt_idmap *idmap, struct dentry *dentry,
/// const char *name, const void *value, size_t size, int flags)` — six
/// arguments, so the previous module's decision is the seventh, and the
/// entry is the second. Neither the name nor the value is read.
#[lsm(hook = "inode_setxattr")]
pub fn inode_setxattr(ctx: LsmContext) -> i32 {
    let entry: u64 = ctx.arg(1);
    let already: i32 = ctx.arg(6);
    if already != 0 {
        return already;
    }
    deciding::decide_attribute(entry)
}

/// Every extended attribute taken away on this machine, until the program is
/// detached.
///
/// `inode_removexattr(struct mnt_idmap *idmap, struct dentry *dentry,
/// const char *name)` — three arguments, so the previous module's decision is
/// the fourth, and the entry is the second.
#[lsm(hook = "inode_removexattr")]
pub fn inode_removexattr(ctx: LsmContext) -> i32 {
    let entry: u64 = ctx.arg(1);
    let already: i32 = ctx.arg(3);
    if already != 0 {
        return already;
    }
    deciding::decide_attribute(entry)
}

/// Every POSIX access list set on this machine, until the program is
/// detached.
///
/// `inode_set_acl(struct mnt_idmap *idmap, struct dentry *dentry,
/// const char *acl_name, struct posix_acl *kacl)` — four arguments, so the
/// previous module's decision is the fifth, and the entry is the second. A
/// hook of its own because since Linux 6.2 an access list set with `setxattr`
/// never reaches `inode_setxattr`: a boundary that watched only that hook
/// would refuse `chmod` and allow the same thing spelled as a list.
#[lsm(hook = "inode_set_acl")]
pub fn inode_set_acl(ctx: LsmContext) -> i32 {
    let entry: u64 = ctx.arg(1);
    let already: i32 = ctx.arg(4);
    if already != 0 {
        return already;
    }
    deciding::decide_attribute(entry)
}

/// Every POSIX access list taken away on this machine, until the program is
/// detached.
///
/// `inode_remove_acl(struct mnt_idmap *idmap, struct dentry *dentry,
/// const char *acl_name)` — three arguments, so the previous module's decision
/// is the fourth, and the entry is the second.
#[lsm(hook = "inode_remove_acl")]
pub fn inode_remove_acl(ctx: LsmContext) -> i32 {
    let entry: u64 = ctx.arg(1);
    let already: i32 = ctx.arg(3);
    if already != 0 {
        return already;
    }
    deciding::decide_attribute(entry)
}

/// Every `ioctl` on every descriptor on this machine, until the program is
/// detached.
///
/// `file_ioctl(struct file *file, unsigned int cmd, unsigned long arg)` —
/// three arguments, so the previous module's decision is the fourth. The
/// file is the first, as it is for `file_open` and `file_permission`; the
/// request is the second and is the one thing this hook reads before it
/// decides whether to read anything else; the third is the request's own
/// argument, which is not read.
///
/// The thirteenth hook, and the one that reaches the last thing a descriptor
/// opened before the turn began could still do: `FS_IOC_SETFLAGS` and
/// `FS_IOC_FSSETXATTR` change what a file *is* — its inode flags — and
/// neither is a change to an inode by name, so none of the five attribute
/// hooks sees them. This runs on the request, which is the moment the flags
/// would move, and it is asked of the **requesting thread's** control group,
/// as `file_permission` is. [`crate::deciding::decide_request`] says which
/// requests are decided here and why every other one is let through without
/// a walk.
#[lsm(hook = "file_ioctl")]
pub fn file_ioctl(ctx: LsmContext) -> i32 {
    let file: u64 = ctx.arg(0);
    let request: u32 = ctx.arg(1);
    let already: i32 = ctx.arg(3);
    if already != 0 {
        return already;
    }
    deciding::decide_request(file, request)
}

/// Which turn this open belongs to, or the cgroup of whoever is not in one.
pub fn turn() -> u64 {
    unsafe { bpf_get_current_cgroup_id() }
}

/// The places a turn was granted, or [`None`] if this cgroup is not a turn.
///
/// [`None`] is the answer for every ordinary program on the machine, and it is
/// the answer that costs nothing: one hash lookup, a miss, and the open goes
/// ahead as though this program were not loaded.
///
/// It is therefore the one answer that must not be reachable from an entry that
/// *is* there and cannot be read — which is why `Bounds::of_words` cannot fail
/// and clamps instead. An entry read wrongly bounds a turn to fewer places, and
/// [`None`] here means only that there was no entry at all.
pub fn granted(turn: u64) -> Option<Bounds> {
    // Copied out immediately, so the borrow of the map's value does not outlive
    // the lookup — which is the whole of what `get`'s safety note asks for.
    let words = *unsafe { BOUNDS.get(turn) }?;
    Some(Bounds::of_words(words))
}

/// Where a field sits in this kernel, as the daemon found it.
pub fn offset(field: Field) -> Option<u64> {
    FIELDS.get(field.index()).map(|found| u64::from(*found))
}

/// Eight bytes of kernel memory, or [`None`] if the kernel would not give them.
pub fn word_at(address: u64) -> Option<u64> {
    if address == 0 {
        return None;
    }
    unsafe { bpf_probe_read_kernel(address as *const u64) }.ok()
}

/// Four bytes of kernel memory, or [`None`] if the kernel would not give them.
pub fn half_word_at(address: u64) -> Option<u32> {
    if address == 0 {
        return None;
    }
    unsafe { bpf_probe_read_kernel(address as *const u32) }.ok()
}

/// Two bytes of kernel memory, or [`None`] if the kernel would not give them.
///
/// The width a `sockaddr`'s family and port are, which is why it exists.
pub fn quarter_word_at(address: u64) -> Option<u16> {
    if address == 0 {
        return None;
    }
    unsafe { bpf_probe_read_kernel(address as *const u16) }.ok()
}
