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
//! Twenty-one hooks on the filesystem — `file_open`, `file_permission`,
//! `inode_rename`, `inode_unlink`, `inode_link`, `inode_setattr`,
//! `inode_setxattr`, `inode_removexattr`, `inode_set_acl`,
//! `inode_remove_acl`, `file_ioctl`, `inode_create`, `inode_mknod`,
//! `inode_mkdir`, `inode_rmdir`, `inode_symlink`, `inode_getattr`,
//! `inode_getxattr`, `inode_listxattr`, `inode_readlink` and
//! `inode_get_acl` — which is what
//! a turn **opens**, **reads and writes**, **moves**, **removes**, gives a
//! **second name**, **changes about a file that is not its contents** — its
//! size, mode, owner, times, extended attributes and access lists, which
//! [`decide_attribute`] decides as one question, and its inode flags, which
//! [`decide_request`] decides for the two `ioctl` requests that set them —
//! since 2026-09-13 what it **makes**: a file, with an open or without one,
//! a directory, and a symbolic link, each decided by the folder the name is
//! being made in, which [`decide_making`] argues; and a directory it
//! removes, which [`decide_delete`] decides as it decides a file — and,
//! since the same day, what it **learns about** a file it may not open: its
//! size, mode, owner and times, the value of an extended attribute, the
//! names of its attributes, where a symbolic link points, and its access
//! list — which the kernel routes to `inode_get_acl` past `inode_getxattr`
//! since Linux 6.2, so it is a hook of its own as the write is — which
//! [`decide_asking`] and [`decide_question`] decide by the walk every other
//! file hook makes. That is not the whole of a filesystem and this file does
//! not pretend it is. Nothing here watches:
//!
//! - **whether a name exists** (`inode_permission`) — `access(2)` and every
//!   path the kernel resolves ask it, so a hook there would be paid for every
//!   component of every open on the machine, twice, to refuse a turn the one
//!   bit `stat` no longer gives it. Named rather than closed, deliberately;
//! - **a mapping of a file** (`mmap_file`) — a file mapped into memory is read
//!   by the processor rather than by a syscall, so a mapping is the one way
//!   left to the contents of a descriptor that was **opened before a turn
//!   began**: every read and write through such a descriptor is decided by
//!   `file_permission` since 2026-09-12 — [`decide_use`] has the whole of it,
//!   and `alo-bounding/tests/what_a_turn_inherits.rs` measures each refusal
//!   beside the use inside the grant it must not break — but a mapping of
//!   one made inside the turn asks no hook this program has. It is named in
//!   `docs/quirks.md` with the reason it is not reproduced in the committed
//!   suite: there is no safe spelling of `mmap` in Rust, and `unsafe` is
//!   forbidden outside this package's one file;
//! - **signals and memory**, and everything else that is not a filesystem. What
//!   a turn connects to *is* watched, by `socket_connect`, and what it sends by
//!   `socket_sendmsg`; [`crate::departing::decide_departure`] and
//!   [`crate::departing::decide_message`] say what those do and do not decide.
//!
//! Each of those is a real gap and each is written down rather than left to be
//! discovered. The mapping is the remainder of a gap that was closed rather
//! than one that was chosen, and what it and the rest have in common is that
//! none of them moves a byte of somebody's file to somewhere they did not
//! approve, which is the property the hooks that exist were chosen for.
//! **Until 2026-09-12 this list also held attributes**, and one of them went
//! further than the promise: `truncate(2)` reaches `inode_setattr` without an
//! open, so a turn could empty a file it could not read. [`decide_attribute`]
//! closed it, and
//! `alo-bounding/tests/the_kernel_refuses_an_attribute_change.rs` measures
//! every attribute refused outside the grant beside the same change landing
//! inside it. **Until 2026-09-13 it held a file's flags** beside the mapping:
//! `FS_IOC_SETFLAGS` on a descriptor opened before the turn began met none of
//! the five attribute hooks, because an `ioctl` is not a change to an inode
//! by name. [`decide_request`] closed it, on `file_ioctl`, and the same test
//! file measures the flag refused outside the grant beside the same flag
//! landing inside it. **And until 2026-09-13 it held what a turn makes** —
//! a symbolic link, a file with or without an open, a directory made or
//! removed — with the honest argument that none of them moves a byte, which
//! is the argument that had been made for attributes until the size broke
//! it. [`decide_making`] and [`decide_delete`] closed the five, and
//! `alo-bounding/tests/the_kernel_refuses_what_a_turn_makes.rs` measures
//! each refused outside the grant beside the same thing made inside it.
//! **And until 2026-09-13 nothing decided what a turn learned *about* a file
//! it could not open**: `stat(2)` answered with its size, owner, mode and
//! times, `getxattr(2)` with the value of an attribute an application put
//! there, `listxattr(2)` with their names and `readlink(2)` with where a link
//! points — no byte of contents, and everything the machine knew about files
//! nobody granted, which is *context is offered, never watched* failing by
//! another road. [`decide_asking`] and [`decide_question`] closed the four,
//! and `alo-bounding/tests/the_kernel_refuses_what_a_turn_reads_about_a_file.rs`
//! measures each refused outside the grant beside the same question answered
//! inside it, with the right answer.
//!
//! What a bound turn can still change on a filesystem is therefore
//! reproduced in `alo-bounding/tests/what_a_bound_turn_can_still_change.rs`
//! as the two things that file has left to say: a program outside the grant
//! cannot be started, and a write inside the grant lands. `docs/quirks.md`
//! carries the list of what was unwatched, with the date each row closed,
//! and `alo-bounding/tests/the_unwatched_mutations_are_written_down.rs`
//! fails the day a hook lands and the documents still call it unwatched —
//! or a hook arrives that no document names.
//!
//! One thing that is **not** a way round any of it: a turn cannot start a
//! program to make the calls for it, because starting one opens the program's
//! own file and that is a `file_open`.

use alo_bounding_map::{Bounds, Field, Place, reaches};

use crate::kernel;

/// The kernel's answer for an open that may go ahead.
pub(crate) const ALLOWED: i32 = 0;

/// The kernel's answer for an open that may not: `EACCES`.
///
/// A number rather than a constant from a header, because this program has no
/// headers — and it is the number ADR 0013 names, so a verb that overreached
/// fails the way a permission failure has always looked rather than in a way a
/// program would have to learn.
pub(crate) const REFUSED: i32 = -13;

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

/// The bits of a mode that say what kind of file it is: `S_IFMT`.
const A_KIND: u16 = 0o170_000;

/// The kind that is a socket: `S_IFSOCK`.
const A_SOCKET: u16 = 0o140_000;

/// The kind that is a pipe: `S_IFIFO`.
const A_PIPE: u16 = 0o010_000;

/// Whether this read or write may go ahead.
///
/// # A descriptor is asked about every time it is used
///
/// `file_open` decides once, when a file is opened, and a descriptor
/// opened before a turn began was never opened inside it — so the record
/// the daemon holds open, the way out of a turn, and any file a verb with a
/// bug in it was handed were all readable and writable past the grant, and
/// were measured being so. This runs on every `read`, `write`, `sendfile`,
/// `splice` and `getdents` on the machine, and for every process that is not
/// a turn it is one hash lookup and a return — the price the six hooks before
/// it charge and for the same reason. Inside a turn it reads the file's kind
/// and walks up from its directory entry exactly as an open would; it reads
/// none of the bytes, and it writes nothing down.
///
/// # What is asked, and of whom
///
/// The control group is the **current** thread's, which is what closes the
/// inherited case rather than restating it: a descriptor the daemon opened
/// outside any turn is, at the moment a turn reads through it, being used by
/// the turn, and the turn is what is asked.
/// [`crate::departing::decide_message`] gives the same reason for a socket,
/// and this is the file half of the same fact — the moment that matters is
/// when the bytes move, not when the handle was made.
///
/// # The answers
///
/// - **Not a turn** — allowed, and nothing is remembered.
/// - **A socket** — allowed here, because
///   [`crate::departing::decide_message`] decides about every message on one
///   by reading where the bytes are going, and a hook that refused a socket
///   by its place in the filesystem would refuse the daemon its answer to the
///   person and a question its provider. The kind
///   is read from the inode's mode; a mode that cannot be read is a file
///   that cannot be checked, and is refused.
/// - **A pipe** — allowed, for the reason a Unix socket is: it holds no
///   contents of its own. What comes through it, a process outside the
///   boundary put there, and what goes into it reaches a process this service
///   already talks to; neither is a file at rest, and neither is a place a
///   grant is over. A test that binds a child process talks to it over one
///   from inside the turn, and so could a service.
/// - **Anything else** — a granted place is met walking up from the file's
///   own directory entry, or `EACCES` before a byte has moved. A terminal, a
///   device, the cgroup filesystem and the record are all *anything else*:
///   none is a place a grant is over, so none is reachable from inside a turn,
///   which is what `file_open` already answered for the same things opened by
///   name.
///
/// # What that costs the turn, and how it is paid
///
/// Leaving a boundary used to be the turn's own write into `cgroup.threads`
/// through a descriptor opened before it began — the same property as the gap,
/// used on purpose. That write is now refused like any other, so a turn's
/// thread cannot end its own boundary at all, and it is brought home by a
/// thread of the service that is not in one. `alo-bounding`'s `inside.rs` has
/// the arrangement; `what_a_turn_inherits.rs` measures both the refusal and
/// the turn still ending.
///
/// # What this does not decide
///
/// A mapping. `mmap` of a file is `mmap_file`, not a read, and it is not
/// hooked: a file mapped into memory is read by the processor rather than by
/// a syscall, so a mapping of an inherited descriptor made inside a turn is a
/// way to its contents this hook does not see. `docs/quirks.md` names it
/// beside what closed here, and why the committed suite cannot reproduce it.
pub fn decide_use(file: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every read and write on the machine.
        return ALLOWED;
    };
    match kind_of(file) {
        Some(A_SOCKET | A_PIPE) => ALLOWED,
        Some(_) if inside(file, granted) => ALLOWED,
        _ => REFUSED,
    }
}

/// What kind of file this is — the `S_IFMT` bits of its inode's mode — or
/// [`None`] if the kernel would not say.
///
/// # Why this fetches its own offsets rather than using [`Fields`]
///
/// The verifier's stack. A BPF program has 512 bytes of stack across every
/// call it makes at once; a bound is 128 of them, the walk's fields 64, and
/// reading the kind inside the walk's frame was measured putting this program
/// 64 bytes over, so the kernel refused to load it. Reading it first, in a
/// frame of its own that has returned before the walk begins, costs three more
/// reads of kernel memory per use inside a turn and keeps the walk exactly the
/// shape [`decide`] has already verified.
fn kind_of(file: u64) -> Option<u16> {
    let file_path = kernel::offset(Field::FilePath)?;
    let path_dentry = kernel::offset(Field::PathDentry)?;
    let entry = kernel::word_at(file.wrapping_add(file_path).wrapping_add(path_dentry))?;
    kind_at(entry)
}

/// What kind of file this directory entry names — the `S_IFMT` bits of its
/// inode's mode — or [`None`] if the kernel would not say.
///
/// The half of [`kind_of`] that starts from an entry, so that a `stat` handed
/// a `struct path` and a read handed a `struct file` ask the same question of
/// the same inode. Its own frame for the reason [`kind_of`] has one.
fn kind_at(entry: u64) -> Option<u16> {
    let dentry_inode = kernel::offset(Field::DentryInode)?;
    let inode_mode = kernel::offset(Field::InodeMode)?;
    let inode = kernel::word_at(entry.wrapping_add(dentry_inode))?;
    let mode = kernel::quarter_word_at(inode.wrapping_add(inode_mode))?;
    Some(mode & A_KIND)
}

/// Whether this question about a file — its size, mode, owner and times,
/// put by a `struct path` rather than by an entry — may be answered.
///
/// # What `stat` reveals, and why it is decided
///
/// Every hook before this one decides what a turn does *to* a file. None
/// decided what a turn found out *about* one it could not open, and `stat`
/// is the whole of that: a turn refused a folder's listing by
/// `file_permission` could still ask each name in it whether it existed and
/// how big it was, and for a private folder that is most of what a listing
/// would have said. No byte of contents moves; what moves is what the machine
/// knows about files nobody granted, and *context is offered, never watched*
/// forbids exactly that by another road.
///
/// # The path, and then the entry, and then the same walk
///
/// `inode_getattr` is handed a `struct path` — the one an open reaches its
/// entry through as `f_path`, handed here on its own — so the entry is one
/// read further in, and from there the decision is [`decide`]'s exactly: the
/// file's own entry, upwards, until a granted place is met or the top of the
/// filesystem is. `alo-files` asks `symlink_metadata` of every path it was
/// given before it opens one, and reads a file's size and link count through
/// the descriptor it opened; every one of those is among the turn's places,
/// and was measured answered before this hook was written.
///
/// # A socket and a pipe are stepped aside from
///
/// `fstat` reaches this hook through the descriptor's own path, and a
/// descriptor can be a socket or a pipe. Both are stepped aside from for the
/// reason [`decide_use`] steps aside from them: neither holds contents of its
/// own, neither is a place a grant is over, and a copy in the standard
/// library asks the kind of both its ends before it moves a byte — so a
/// refusal here would break a verb copying a file inside its grant towards a
/// process this service already talks to, while looking like a boundary. The
/// kind is read from the inode's mode; a mode that cannot be read is a file
/// that cannot be checked, and is refused. A terminal, a device and the
/// cgroup filesystem are not stepped aside from, and are refused as they are
/// by name.
///
/// # What it costs, and what is deliberately not paid
///
/// This runs on every `stat`, `lstat`, `fstat` and `statx` on the machine,
/// which is the busiest hook here after reads and writes; for a process that
/// is not a turn it is one hash lookup and a miss, as everywhere. What is
/// **not** hooked is `inode_permission`: it runs on every component of every
/// path the kernel resolves, so the walk would be paid for every open on the
/// machine twice, and what it would add is refusing `access(2)`, which
/// reveals only whether a name exists. Named in `docs/quirks.md` rather than
/// closed.
///
/// # Not a turn
///
/// Allowed, and nothing is remembered.
pub fn decide_asking(path: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every `stat` on the machine.
        return ALLOWED;
    };
    let Some(entry) = entry_at(path) else {
        return REFUSED;
    };
    match kind_at(entry) {
        Some(A_SOCKET | A_PIPE) => ALLOWED,
        Some(_) if entry_inside(entry, granted) => ALLOWED,
        _ => REFUSED,
    }
}

/// Whether this question about a file — the value of an extended attribute,
/// the names of its attributes, where a symbolic link points, or its access
/// list — may be answered.
///
/// # One answer for four hooks
///
/// `inode_getxattr`, `inode_listxattr`, `inode_readlink` and `inode_get_acl`
/// are each handed the directory entry of the file being asked about. The
/// file exists, so this is the question [`decide_attribute`] asks of a change
/// and the same walk answers it, from the entry rather than its folder: a
/// grant can be over a single file, and its folder is then not a place the
/// call named.
///
/// # Why a read of what is not the contents is decided
///
/// A `user.*` attribute is somewhere a person's application keeps bytes that
/// are not the file's contents — a comment, an origin, a checksum — and a
/// byte somebody put there is theirs whether or not it is in the file. The
/// names say which files carry one. And where a link points is somebody's
/// filesystem laid out in words: a turn that could read every link on the
/// machine could map it without opening a file. None of that moves a byte of
/// contents past a grant, which is the argument that was made for attribute
/// changes until the size broke it, and it has the same remainder — nothing
/// of somebody's contents, all of somebody's files.
///
/// # Why the access list is a fourth hook and not the second
///
/// A file's access list is read with `getxattr` under
/// `system.posix_acl_access`, and since Linux 6.2 the kernel routes that read
/// to `inode_get_acl` before `inode_getxattr` is ever asked — the same
/// routing that made `inode_set_acl` a hook beside `inode_setxattr` for the
/// write. So a boundary on `inode_getxattr` alone refused a turn the names of
/// a file's attributes and answered it the list of who may read the file,
/// which task 19's own reproduction measured on 2026-09-13 and task 20
/// closed the same day. The list is what the mode bits cannot say — a named
/// user, a mask — and a file nobody granted is not one whose readers a turn
/// may learn.
///
/// # Not a turn
///
/// Allowed, and nothing is remembered.
pub fn decide_question(entry: u64) -> i32 {
    this_entry(entry)
}

/// The directory entry a `struct path` leads to, or [`None`] if it cannot be
/// read.
///
/// The second half of [`entry_of`]: a `struct file` embeds a path, and this
/// is the step from the path to its entry, so a `stat` handed the path alone
/// and an open handed the file start their walks from the same place.
fn entry_at(path: u64) -> Option<u64> {
    let path_dentry = kernel::offset(Field::PathDentry)?;
    kernel::word_at(path.wrapping_add(path_dentry))
}

/// The `ioctl` request that sets a file's inode flags: `FS_IOC_SETFLAGS`,
/// which is `_IOW('f', 2, long)` — how `chattr` spells `nodump`, `noatime`,
/// `append-only` and `immutable`.
///
/// A number rather than a constant from a header, for the reason [`REFUSED`]
/// is one: this program has no headers. The encoding is the kernel's own —
/// direction, size, type and number — and a `long` is eight bytes on every
/// machine this program can be loaded on.
const SETTING_FLAGS: u32 = 0x4008_6602;

/// The same request in its 32-bit width: `FS_IOC32_SETFLAGS`, which is
/// `_IOW('f', 2, int)`.
///
/// Not a different request. It is what a 32-bit program spells when it means
/// [`SETTING_FLAGS`], and a kernel before 6.8 hands it to this same hook with
/// the width unchanged — so a boundary that recognised only the 64-bit
/// spelling would be one a 32-bit program could walk past. On this kernel a
/// 32-bit program's requests reach `file_ioctl_compat` instead, which is
/// named in `docs/quirks.md` with why it is bounded already; recognising the
/// number here costs one comparison and closes the older kernels.
const SETTING_FLAGS_32: u32 = 0x4004_6602;

/// The `ioctl` request that sets a file's extended inode attributes:
/// `FS_IOC_FSSETXATTR`, which is `_IOW('X', 32, struct fsxattr)`, twenty-eight
/// bytes of it.
///
/// The other spelling of the same change: the `fsxattr` structure carries the
/// same flags as [`SETTING_FLAGS`] in a different layout, with a project
/// identifier beside them, and a boundary that refused one spelling and let
/// the other through would be refusing `chattr` and allowing the change made
/// by hand.
const SETTING_EXTENDED_FLAGS: u32 = 0x401c_5820;

/// Whether this `ioctl` request may go ahead.
///
/// # The request is read first, and almost every request ends here
///
/// `ioctl` is how a terminal is asked its size, a socket its state, a device
/// anything at all — and a person's editor, shell and compositor make those
/// requests constantly. So this is the one hook whose first question is not
/// *is this a turn*: the request number is compared against the three that
/// change what a file **is**, and a request that is not one of them is
/// allowed before any map is looked up or any word of kernel memory read.
/// `TIOCGWINSZ` inside a turn costs three comparisons; it is never walked.
/// That order is the plan's constraint and the honest shape of the cost: a
/// boundary that walked a filesystem for every request on the machine would
/// be a tax on everybody for a flag nobody was changing.
///
/// # What is decided, and why it is these three
///
/// [`SETTING_FLAGS`], its 32-bit width, and [`SETTING_EXTENDED_FLAGS`] — the
/// two ways a file's inode flags are set, one of them in two widths. Flags
/// are the last thing about a file that is not its contents and that a
/// descriptor opened **before** the turn began could still change: an open
/// inside the turn on a file outside the grant is refused at the open, so
/// what this reaches is exactly the inherited descriptor `file_permission`
/// closed for reads and writes and `inode_setattr` closed for the size.
/// What the kernel already bounds is real and is not this program's:
/// `append-only` and `immutable` need `CAP_LINUX_IMMUTABLE`, which
/// `alo-agentd` does not hold, so before this hook what a turn could do was
/// set `nodump` or `noatime` on a file it could not read. Small, and a row is
/// still a row.
///
/// Reading a file's flags — `FS_IOC_GETFLAGS`, `FS_IOC_FSGETXATTR` — is not
/// decided here and is let through like every other request: the flags are
/// not the contents, and what `file_open` and `file_permission` refuse is
/// reading the file.
///
/// # And then the same question every file hook asks
///
/// For one of the three, the decision is [`decide`]'s exactly: the file's own
/// directory entry, walked upwards until a granted place is met or the top of
/// the filesystem is. The same function and the same walk, so an open and a
/// flag change on the same descriptor cannot come to different answers about
/// where it is.
///
/// # Not a turn
///
/// Allowed, and nothing is remembered — for one of the three requests it is
/// one hash lookup and a miss, and for every other request it is not even
/// that.
pub fn decide_request(file: u64, request: u32) -> i32 {
    match request {
        SETTING_FLAGS | SETTING_FLAGS_32 | SETTING_EXTENDED_FLAGS => decide(file),
        _ => ALLOWED,
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
///
/// # A directory removed is a name removed
///
/// `inode_rmdir` asks this as well, since 2026-09-13, and it is the same
/// question: the directory being removed exists, so it is judged by its own
/// entry, and a grant over a single directory is a grant over removing it.
/// Removing a directory that is not empty needs its contents unlinked first,
/// which is this function again, one name at a time — so what was left
/// before the hook was the removal of an **empty** directory nobody granted,
/// and that is what is refused now.
pub fn decide_delete(entry: u64) -> i32 {
    this_entry(entry)
}

/// Whether this name may be **made** — a file, with `open(O_CREAT)` or with
/// `mknod`, a directory, or a symbolic link.
///
/// # The folder, because the entry is not there yet
///
/// Every one of the four hooks that ask this is handed the directory entry
/// for the name being made, and that entry is **negative**: it names a place
/// in a folder rather than a file, and it has no inode to be asked about.
/// The rename hook's destination has the same shape, and
/// `a_name_being_made` answers it the same way: the folder the name is
/// being made in is what the call named, it exists, and its entry is walked
/// upwards exactly as an open's would be. `alo_files::Reaching` puts *the
/// folder above anything a call would create* among a turn's places for
/// exactly this, so a verb that writes an archive is bound to the folder the
/// archive goes into and the create inside it lands.
///
/// # Why this exists, when none of it moves a byte
///
/// The argument for leaving these unwatched was honest and was measured:
/// what a bound turn could leave was an empty file, a device node, a folder
/// or a link — no byte of anybody's document in any of them, because putting
/// one there is an open and an open is watched. It is the same argument that
/// was made for attributes until `truncate(2)` broke it, and it has the same
/// shape of remainder: none of that is somebody's contents, all of it is
/// somebody's filesystem. A boundary that stopped a turn changing a file's
/// mode outside the grant while letting it fill the same folder with names
/// of its choosing was one that had to be explained, and a name a turn
/// leaves outlives the turn.
///
/// # The target of a link is not read
///
/// A link inside the grant may point anywhere, and the turn is no better off
/// for it: opening through it is a `file_open` on the file it leads to, and
/// the walk starts there. What this decides is where the *name* goes, which
/// is the folder — a link made outside the grant is refused whatever it
/// points at, and one made inside it is allowed whatever it points at.
///
/// # Not a turn
///
/// Allowed, and nothing is remembered — this is every file, folder and link
/// made on the machine, and for all of them it is one hash lookup and a
/// return.
pub fn decide_making(entry: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every file and folder made on the
        // machine.
        return ALLOWED;
    };
    let Some(fields) = Fields::found() else {
        return REFUSED;
    };
    let Some(folder) = kernel::word_at(entry.wrapping_add(fields.dentry_parent)) else {
        return REFUSED;
    };
    if upwards_from(folder, &fields, granted) {
        ALLOWED
    } else {
        REFUSED
    }
}

/// Whether this change to what a file **is** — rather than to what it holds —
/// may go ahead.
///
/// # One answer for five hooks
///
/// `inode_setattr` is a file's size, mode, owner and times; `inode_setxattr`
/// and `inode_removexattr` are an extended attribute set and taken away;
/// `inode_set_acl` and `inode_remove_acl` are an access list set and taken
/// away, which since Linux 6.2 the kernel routes past the extended-attribute
/// hooks even though the call that makes one is `setxattr`. Every one of them
/// is handed the directory entry of the file being changed, and every one of
/// them asks this: is that entry inside the grant. Five hooks answered by one
/// function so that `chmod` and the same change spelled as an access list
/// cannot come to different answers about the same file.
///
/// # Why this exists at all, when none of it moves a byte
///
/// Until 2026-09-12 none of these was watched, and the argument for leaving
/// them was honest: a mode, an owner or an attribute on a file nobody granted
/// moves no contents past a grant, and what decides here is where a file is
/// and not what its mode says. **Size broke that argument.** `truncate(2)`
/// reaches `inode_setattr` without an open, so a turn refused `open` on a file
/// could still empty it — nothing left the grant and the contents were
/// destroyed where they sat, which is worse. The rest came with it because
/// they share the hook and because what a person has is not only what their
/// files hold: a mode left wide open outlives the turn, an owner changed is a
/// file they no longer own, and a boundary that stopped a turn reading a file
/// but let it strip the file's access list would be one that had to be
/// explained.
///
/// # The entry, not its folder
///
/// The same question `decide_delete` asks and for the same reason: a grant
/// can be over a single file, and its folder is then not a place the call
/// named. Judging by the parent would refuse every legitimate change inside
/// a one-file grant.
///
/// # Not a turn
///
/// Allowed, and nothing is remembered — this is every `chmod`, every
/// `truncate` and every attribute on the machine, and for all of them it is
/// one hash lookup and a return.
pub fn decide_attribute(entry: u64) -> i32 {
    this_entry(entry)
}

/// Whether **this** directory entry is inside the grant, for the hooks that
/// are handed one entry and are asking about the file it names.
///
/// One function for the two, so that a delete and an attribute change cannot
/// walk from different places or come to different answers about the same
/// entry.
fn this_entry(entry: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every delete, attribute change and
        // question about a file on the machine.
        return ALLOWED;
    };
    if entry_inside(entry, granted) {
        ALLOWED
    } else {
        REFUSED
    }
}

/// Whether this directory entry lies at or under a granted place.
///
/// The fetch of the fields and the walk, for the hooks that already hold an
/// entry; [`inside`] is the same for the hooks that hold a file.
fn entry_inside(entry: u64, granted: Bounds) -> bool {
    let Some(fields) = Fields::found() else {
        return false;
    };
    upwards_from(entry, &fields, granted)
}

/// Whether the file this open is for lies at or under a granted place.
fn inside(file: u64, granted: Bounds) -> bool {
    let Some(fields) = Fields::found() else {
        return false;
    };
    let Some(entry) = entry_of(file, &fields) else {
        return false;
    };
    upwards_from(entry, &fields, granted)
}

/// The directory entry a `struct file` was opened through, or [`None`] if it
/// cannot be read.
///
/// One fetch for the two hooks that are handed a file, so that an open and a
/// use of the same descriptor cannot start their walks from different places.
fn entry_of(file: u64, fields: &Fields) -> Option<u64> {
    // `f_path` is embedded in `struct file` rather than pointed at, so its
    // offset gives the address of the `struct path` itself, and the entry is
    // one step further in.
    let path = file.wrapping_add(fields.file_path);
    kernel::word_at(path.wrapping_add(fields.path_dentry))
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
