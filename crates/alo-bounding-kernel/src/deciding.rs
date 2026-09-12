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
//! Five hooks on the filesystem — `file_open`, `file_permission`,
//! `inode_rename`, `inode_unlink` and `inode_link` — which is what a turn
//! **opens**, **reads and writes**, **moves**, **removes**, and gives a
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
//! - **making a file** (`inode_create`, and `inode_mknod` for the call that
//!   makes one without opening it) — a turn can create one. Writing to it is an
//!   open, which is watched, so what this leaves is an empty file somewhere;
//! - **attributes** (`inode_setattr`, `inode_setxattr`) — a turn can change the
//!   mode, owner, times and extended attributes of a file nobody granted it,
//!   and is no better off for it: what decides here is where a file is and not
//!   what its mode says. **The exception is size.** `truncate(2)` reaches
//!   `inode_setattr` without an open, so a turn can empty a file it cannot
//!   read — which moves no contents anywhere and destroys them where they are.
//!   It is the sharpest thing on this list and `docs/quirks.md` says so;
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
//!   `socket_sendmsg`; [`decide_departure`] and [`decide_message`] say what
//!   those do and do not decide.
//!
//! Each of those is a real gap and each is written down rather than left to be
//! discovered. What they have in common — size aside, which is named above
//! rather than filed under it, and a mapping aside, which is the remainder of
//! a gap that was closed rather than one that was chosen — is that none of
//! them moves a byte of somebody's file to somewhere they did not approve,
//! which is the property the hooks that exist were chosen for.
//!
//! **All of them are reproduced** against this programme on a running kernel, in
//! `alo-bounding/tests/what_a_bound_turn_can_still_change.rs`, each with a
//! refused open beside it proving the boundary was in force. `docs/quirks.md`
//! carries the same list with the release that owns closing each — every one of
//! them v0.5 — and `alo-bounding/tests/the_unwatched_mutations_are_written_down.rs`
//! fails the day one of these hooks lands and the documents still call it
//! unwatched.
//!
//! One thing that is **not** a way round any of it: a turn cannot start a
//! program to make the calls for it, because starting one opens the program's
//! own file and that is a `file_open`.

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
/// the turn, and the turn is what is asked. [`decide_message`] gives the same
/// reason for a socket, and this is the file half of the same fact — the
/// moment that matters is when the bytes move, not when the handle was made.
///
/// # The answers
///
/// - **Not a turn** — allowed, and nothing is remembered.
/// - **A socket** — allowed here, because [`decide_message`] decides about
///   every message on one by reading where the bytes are going, and a hook
///   that refused a socket by its place in the filesystem would refuse the
///   daemon its answer to the person and a question its provider. The kind
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
    let dentry_inode = kernel::offset(Field::DentryInode)?;
    let inode_mode = kernel::offset(Field::InodeMode)?;
    let entry = kernel::word_at(file.wrapping_add(file_path).wrapping_add(path_dentry))?;
    let inode = kernel::word_at(entry.wrapping_add(dentry_inode))?;
    let mode = kernel::quarter_word_at(inode.wrapping_add(inode_mode))?;
    Some(mode & A_KIND)
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

/// Somewhere bytes are about to go, as far as this program can read it.
///
/// One shape for the two places a destination is found — the `sockaddr` a
/// `connect` or a `sendto` names, and the peer a joined socket remembers — so
/// that the two hooks that read them cannot come to different answers about
/// the same address.
#[derive(Clone, Copy)]
enum Destination {
    /// Not a network address at all: a Unix socket, a netlink socket, and
    /// anything else whose family is not one this bound can describe. Not
    /// egress, and not this program's to decide about.
    NotEgress,

    /// A network address, with the port in host order.
    Network(Family, u128, u16),
}

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
///
/// # What this hook alone could not decide, and what now does
///
/// A connection is made once and written on many times, and this hook sees
/// only the once. A socket joined before the turn began, and a datagram sent
/// on a socket joined to nothing, were both reproduced reaching past a bound
/// turn's boundary — and both are now decided by [`decide_message`], on the
/// message, which is the moment the bytes go. The proxy above is the one of
/// the three that remains, reproduced in
/// `alo-bounding/tests/what_a_bound_turn_can_still_reach.rs` beside the two
/// that are closed.
pub fn decide_departure(where_to: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every connection on the machine.
        return ALLOWED;
    };
    match destination_named_at(where_to) {
        Some(going) if may_go(granted, going) => ALLOWED,
        _ => REFUSED,
    }
}

/// Whether this message may be sent.
///
/// # Why a hook on the message, and what it costs
///
/// `socket_connect` runs once, when a socket is joined. Two things never pass
/// it: a socket joined **before** the turn began — the network's version of an
/// inherited descriptor — and a datagram sent with `sendto` on a socket joined
/// to nothing, which makes no `connect` at all. Both were reproduced against
/// this programme moving bytes past a bound turn's boundary, and both are the
/// same fact: the moment that matters is when the bytes go, not when the
/// socket was joined.
///
/// So this runs on every message the machine sends, and for every process that
/// is not a turn it is one hash lookup and a return — the same price the other
/// five hooks charge and for the same reason. Inside a turn it reads a handful
/// of words of kernel memory and compares numbers; it does not read the bytes
/// of the message, and it writes nothing down.
///
/// # What is asked, and it is asked of the sending thread
///
/// The control group is the **current** thread's, which is what makes this
/// close the inherited-socket gap rather than restate it: a socket the daemon
/// opened outside any turn is, at the moment a turn writes on it, being used by
/// the turn, and the turn is what is asked. ADR 0013's *attribution of every
/// one to the turn that caused it*, on the message.
///
/// # Where a message goes, and both places are asked
///
/// A message has up to two destinations, and either one is enough to send the
/// bytes somewhere:
///
/// - **the address it names** — `msg_name`, which `sendto` fills and `send`
///   and `write` leave null. A datagram socket sends there;
/// - **the peer the socket is joined to** — `skc_daddr`, `skc_v6_daddr` and
///   `skc_dport` on the `struct sock`, filled by a `connect` whenever it
///   happened. A stream socket sends there whatever the message names.
///
/// **Both are checked when both are there**, because the kernel decides which
/// one the bytes follow by protocol, and a program that guessed the protocol
/// would be a program somebody could arrange to guess wrong. A message that
/// names nothing on a socket joined to nobody has no destination this program
/// can read, and is refused: the kernel would refuse it too, and a destination
/// that cannot be checked is refused in every hook here.
///
/// # The answers, and where they come from
///
/// - **Not a turn** — allowed, and nothing is remembered.
/// - **A socket whose family is not a network address** — allowed. The daemon
///   answers the person on a Unix socket, and a turn writing on it is not
///   egress; `decide_departure` says the same of a `connect` to one.
/// - **A network socket whose message names an address this cannot read** —
///   refused. `AF_UNSPEC` on a datagram is read by the kernel as an IPv4
///   address; here it is a family this program does not enforce, and on a
///   network socket that is a destination that cannot be checked rather than
///   one that is not egress.
/// - **Loopback**, whether named or joined — allowed, for ADR 0007's reason
///   and with the same cost: a proxy on loopback is still believed.
/// - **A destination the person was shown** — allowed. **Any other** —
///   refused, with `EACCES`, before a byte leaves. Withdrawal and the end of
///   the turn are the same map entry `socket_connect` reads, so a destination
///   withdrawn while a connection to it is open is refused on the next message
///   — which closes the connection-reuse gap the connect hook named.
pub fn decide_message(socket: u64, message: u64) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every message on the machine.
        return ALLOWED;
    };
    let Some(fields) = NetworkFields::found() else {
        return REFUSED;
    };
    let Some(sock) = kernel::word_at(socket.wrapping_add(fields.socket_sock)) else {
        return REFUSED;
    };
    let Some(family) = kernel::quarter_word_at(sock.wrapping_add(fields.sock_family)) else {
        return REFUSED;
    };
    let Some(family) = Family::of(family) else {
        // Not a network socket, so not egress, so not this program's to decide
        // about — whatever the message names.
        return ALLOWED;
    };
    let Some(named) = kernel::word_at(message.wrapping_add(fields.message_name)) else {
        return REFUSED;
    };
    let Some(joined_to) = peer_of(sock, family, &fields) else {
        return REFUSED;
    };

    let mut somewhere = false;
    if let Some(peer) = joined_to {
        if !may_go(granted, peer) {
            return REFUSED;
        }
        somewhere = true;
    }
    if named != 0 {
        match destination_named_at(named) {
            // A network socket naming an address that is not a network address
            // is naming one this program cannot check, not one that stays home.
            Some(Destination::Network(family, address, port))
                if may_go(granted, Destination::Network(family, address, port)) => {}
            _ => return REFUSED,
        }
        somewhere = true;
    }
    if somewhere { ALLOWED } else { REFUSED }
}

/// Whether a bound turn may send bytes there.
fn may_go(granted: Bounds, going: Destination) -> bool {
    match going {
        Destination::NotEgress => true,
        Destination::Network(family, address, port) => {
            stays_on_this_machine(family, address)
                || granted.may_leave(Departure::of(family, address, port))
        }
    }
}

/// The destination a `sockaddr` names, or [`None`] if it cannot be read.
fn destination_named_at(where_to: u64) -> Option<Destination> {
    let family = kernel::quarter_word_at(where_to.wrapping_add(FAMILY_AT))?;
    let Some(family) = Family::of(family) else {
        return Some(Destination::NotEgress);
    };
    let port = kernel::quarter_word_at(where_to.wrapping_add(PORT_AT))?;
    let address = address_of(family, where_to)?;
    Some(Destination::Network(family, address, u16::from_be(port)))
}

/// The peer a network socket is joined to, [`None`] if it cannot be read, and
/// [`Some`] of nothing for a socket joined to nobody.
///
/// A socket joined to nobody has a port of zero, which no peer has: the kernel
/// clears the port on disconnect and never assigns port zero to a peer, so it
/// is the honest reading of *there is no peer* rather than a sentinel this
/// program chose.
fn peer_of(sock: u64, family: Family, fields: &NetworkFields) -> Option<Option<Destination>> {
    let port = kernel::quarter_word_at(sock.wrapping_add(fields.sock_port))?;
    if port == 0 {
        return Some(None);
    }
    let address = match family {
        Family::Four => {
            let four = kernel::half_word_at(sock.wrapping_add(fields.sock_address))?;
            u128::from(u32::from_be(four))
        }
        Family::Six => {
            let high = kernel::word_at(sock.wrapping_add(fields.sock_address6))?;
            let low = kernel::word_at(sock.wrapping_add(fields.sock_address6 + 8))?;
            (u128::from(u64::from_be(high)) << 64) | u128::from(u64::from_be(low))
        }
    };
    Some(Some(Destination::Network(
        family,
        address,
        u16::from_be(port),
    )))
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

/// Where this kernel keeps the fields the message hook reads.
///
/// Its own set rather than six more on [`Fields`], because the two hooks that
/// read them share nothing: a walk up a filesystem fetches no socket, and a
/// message fetches no directory entry. Six lookups a turn does not need would
/// be six the verifier still has to account for.
struct NetworkFields {
    /// `struct socket`'s `sk`.
    socket_sock: u64,
    /// `struct sock`'s `skc_family`, through `__sk_common`.
    sock_family: u64,
    /// `struct sock`'s `skc_dport`, through `__sk_common`.
    sock_port: u64,
    /// `struct sock`'s `skc_daddr`, through `__sk_common`.
    sock_address: u64,
    /// `struct sock`'s `skc_v6_daddr`, through `__sk_common`.
    sock_address6: u64,
    /// `struct msghdr`'s `msg_name`.
    message_name: u64,
}

impl NetworkFields {
    /// The six offsets, or [`None`] if the daemon did not put them there.
    ///
    /// Several of these are legitimately zero on a real kernel —
    /// `__sk_common` is the first thing in a `struct sock` and the peer's
    /// address is the first thing in it, and `msg_name` opens a `struct
    /// msghdr` — so zero is not read as *missing* here any more than it is for
    /// the walk. [`None`] is a slot the map does not have, which is a program
    /// and a daemon built from different sources.
    fn found() -> Option<Self> {
        Some(Self {
            socket_sock: kernel::offset(Field::SocketSock)?,
            sock_family: kernel::offset(Field::SockFamily)?,
            sock_port: kernel::offset(Field::SockPort)?,
            sock_address: kernel::offset(Field::SockAddress)?,
            sock_address6: kernel::offset(Field::SockAddress6)?,
            message_name: kernel::offset(Field::MessageName)?,
        })
    }
}
