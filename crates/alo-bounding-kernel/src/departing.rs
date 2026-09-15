//! What happens when a turn connects or sends, in ordinary Rust.
//!
//! The two network hooks, `socket_connect` and `socket_sendmsg`, arrive here.
//! The first thing asked is the same as on an open — whether this control group
//! is a turn at all — and for every other process on the machine the answer is
//! no and the function is over.
//!
//! Its own file rather than the second half of `crate::deciding`, because what
//! changes a destination and what changes a walk up a filesystem are two
//! different reasons to open a file: ADR 0041 changed what a destination is and
//! touched nothing a file hook reads.
//!
//! # A destination is an address, a port, and for a link-local address its interface
//!
//! [ADR 0041](../../../docs/decisions/0041-a-link-local-departure-names-its-interface.md).
//! `fe80::1` is one machine on each link this machine is on, so the interface
//! a link-local destination leaves by is part of *where* — and it is read from
//! the same two places the kernel itself takes it from, in the kernel's order:
//!
//! 1. **the scope the address names**, when the caller's `sockaddr_in6` was long
//!    enough to carry one and it is not zero;
//! 2. otherwise **the interface the socket is held to** (`skc_bound_dev_if`),
//!    which a scoped `connect`, a bind to a scoped address and
//!    `SO_BINDTODEVICE` all set.
//!
//! Neither is a guess about a third: when both are zero the kernel would pick
//! the interface from a socket option this program does not read
//! (`IPV6_UNICAST_IF`, `IPV6_MULTICAST_IF`) or from a control message, so the
//! destination is one that names no network and
//! `alo_bounding_map::Departures::holds` refuses it whatever was shown.
//! `alo_bounding_map::Departure::on` decides which addresses need an interface
//! at all, for the daemon and for this program alike.
//!
//! # An IPv4 destination is also decided on the interface its socket is held to
//!
//! [ADR 0044](../../../docs/decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md).
//! `192.168.1.20` can be a different machine on each of two networks, and a
//! `sockaddr_in` has no scope to say which. What does say it is the socket: one
//! held to an interface (`SO_BINDTOIFINDEX`, `SO_BINDTODEVICE`) leaves by it
//! whatever the route says. So an IPv4 destination carries **the interface the
//! socket is held to** — zero for a socket held to none — and a departure held
//! to an interface permits only that one, while a departure held to none (a
//! provider's) permits any, as it always did.
//!
//! **One road around the socket, and it is closed here.** On IPv4 an
//! `IP_PKTINFO` control message names the interface a datagram leaves by ahead
//! of the one the socket is held to, and the kernel does not check that the two
//! agree — IPv6's `IPV6_PKTINFO` is refused by the kernel when they differ,
//! `docs/quirks.md` records both. So an IPv4 message that carries any control
//! message is decided as **held to no interface**: permitted where a departure
//! held to none was shown, and refused by every departure held to one. The
//! control messages themselves are not read.

use alo_bounding_map::{Bounds, Departure, Family, Field, keeps_its_interface, needs_an_interface};

use crate::deciding::{ALLOWED, REFUSED};
use crate::kernel;

/// Where a `sockaddr` keeps the family, which every one of them has first.
const FAMILY_AT: u64 = 0;

/// Where both kinds keep the port, immediately after the family.
const PORT_AT: u64 = 2;

/// Where a `sockaddr_in` keeps its four bytes of address.
const INET_ADDRESS_AT: u64 = 4;

/// Where a `sockaddr_in6` keeps its sixteen, past the flow label.
const INET6_ADDRESS_AT: u64 = 8;

/// Where a `sockaddr_in6` keeps the interface a link-local address is on.
const INET6_SCOPE_AT: u64 = 24;

/// How long a `sockaddr_in6` has to be for the kernel to read its scope.
///
/// RFC 2133's form of the structure is twenty-four bytes and has no scope; the
/// kernel accepts it, and reads a scope only from an address of the full
/// twenty-eight. Past the length the caller gave is whatever was on the kernel's
/// stack, so a scope is read only where the kernel would read one.
const INET6_WITH_ITS_SCOPE: i64 = 28;

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

    /// A network address, with the port in host order and the interface it
    /// leaves by where the address needs one.
    Network(Departure),
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
/// departure is one address and one port — and one interface, for an address
/// that is only somewhere on one — never permission to connect.
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
pub fn decide_departure(socket: u64, where_to: u64, length: i32) -> i32 {
    let Some(granted) = kernel::granted(kernel::turn()) else {
        // Not a turn, and this is almost every connection on the machine.
        return ALLOWED;
    };
    let held_to = || {
        let fields = NetworkFields::found()?;
        let sock = kernel::word_at(socket.wrapping_add(fields.socket_sock))?;
        interface_held_to(sock, &fields)
    };
    match destination_named_at(where_to, i64::from(length), held_to) {
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
/// A link-local peer's interface is the one the socket is held to, which the
/// `connect` that joined it set; a link-local address a message names takes its
/// interface as [`decide_departure`]'s does — this module's own documentation
/// has the order. An IPv4 destination, named or joined, is on the interface the
/// socket is held to, **unless the message carries control messages**, when it
/// is on none: `IP_PKTINFO` can send it by another (ADR 0044).
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
    let Some(control) = kernel::word_at(message.wrapping_add(fields.message_control_length)) else {
        return REFUSED;
    };
    // Where the socket is held to, for a destination that is decided on it — and
    // for an IPv4 message carrying control messages, nowhere, because one of
    // them can name another interface and the kernel would follow it.
    let held_to = || {
        if control != 0 && family == Family::Four {
            Some(0)
        } else {
            interface_held_to(sock, &fields)
        }
    };
    let Some(joined_to) = peer_of(sock, family, held_to, &fields) else {
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
        let Some(length) = kernel::half_word_at(message.wrapping_add(fields.message_name_length))
        else {
            return REFUSED;
        };
        // `msg_namelen` is an `int`; the kernel has already refused a negative
        // one, and reading it as signed keeps a nonsense one short.
        let length = i64::from(length.cast_signed());
        match destination_named_at(named, length, held_to) {
            // A network socket naming an address that is not a network address
            // is naming one this program cannot check, not one that stays home.
            Some(going @ Destination::Network(_)) if may_go(granted, going) => {}
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
        Destination::Network(departure) => {
            stays_on_this_machine(departure) || granted.may_leave(departure)
        }
    }
}

/// The destination a `sockaddr` of `length` bytes names, or [`None`] if it
/// cannot be read.
///
/// `held_to` is asked only where the socket decides: a link-local address whose
/// `sockaddr` names no scope (ADR 0041), and an IPv4 address (ADR 0044). A
/// global IPv6 address reads nothing more of the kernel than it did before
/// either.
fn destination_named_at(
    where_to: u64,
    length: i64,
    held_to: impl FnOnce() -> Option<u32>,
) -> Option<Destination> {
    let family = kernel::quarter_word_at(where_to.wrapping_add(FAMILY_AT))?;
    let Some(family) = Family::of(family) else {
        return Some(Destination::NotEgress);
    };
    let port = u16::from_be(kernel::quarter_word_at(where_to.wrapping_add(PORT_AT))?);
    let address = address_of(family, where_to)?;
    if !keeps_its_interface(family, address) {
        return Some(Destination::Network(Departure::of(family, address, port)));
    }
    // Only an IPv6 address has a scope of its own to name; an IPv4 one is on
    // whatever interface the socket is held to.
    let named = if needs_an_interface(family, address) && length >= INET6_WITH_ITS_SCOPE {
        kernel::half_word_at(where_to.wrapping_add(INET6_SCOPE_AT))?
    } else {
        0
    };
    let interface = if named == 0 { held_to()? } else { named };
    Some(Destination::Network(Departure::on(
        family, address, port, interface,
    )))
}

/// The interface a socket is held to, and zero for one held to none.
fn interface_held_to(sock: u64, fields: &NetworkFields) -> Option<u32> {
    kernel::half_word_at(sock.wrapping_add(fields.sock_bound_interface))
}

/// The peer a network socket is joined to, [`None`] if it cannot be read, and
/// [`Some`] of nothing for a socket joined to nobody.
///
/// A socket joined to nobody has a port of zero, which no peer has: the kernel
/// clears the port on disconnect and never assigns port zero to a peer, so it
/// is the honest reading of *there is no peer* rather than a sentinel this
/// program chose.
///
/// A link-local peer is on the interface the socket is held to, because that is
/// where the `connect` that joined it put the scope; one held to none names no
/// network and is refused by the check rather than here. An IPv4 peer is on the
/// interface `held_to` answers, which is the socket's — or none, for a message
/// whose control messages could move it (ADR 0044).
fn peer_of(
    sock: u64,
    family: Family,
    held_to: impl FnOnce() -> Option<u32>,
    fields: &NetworkFields,
) -> Option<Option<Destination>> {
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
    let port = u16::from_be(port);
    let departure = if keeps_its_interface(family, address) {
        Departure::on(family, address, port, held_to()?)
    } else {
        Departure::of(family, address, port)
    };
    Some(Some(Destination::Network(departure)))
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

/// Whether this destination is one that never leaves the machine.
///
/// `127.0.0.0/8` and `::1`. Nothing here treats `::ffff:127.0.0.1` as loopback,
/// and deliberately: it arrives as an IPv6 address and is compared as one, so a
/// turn reaching it is checked against what somebody was shown like any other
/// destination. Being stricter than necessary about a mapped address is the
/// safe direction.
const fn stays_on_this_machine(departure: Departure) -> bool {
    match departure.family() {
        Some(Family::Four) => departure.address() >> 24 == 127,
        Some(Family::Six) => departure.address() == 1,
        None => false,
    }
}

/// Where this kernel keeps the fields the two network hooks read.
///
/// Its own set rather than more on `crate::deciding`'s `Fields`, because the
/// hooks that read them share nothing: a walk up a filesystem fetches no socket,
/// and a message fetches no directory entry. Lookups a turn does not need would
/// be lookups the verifier still has to account for.
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
    /// `struct sock`'s `skc_bound_dev_if`, through `__sk_common`.
    sock_bound_interface: u64,
    /// `struct msghdr`'s `msg_name`.
    message_name: u64,
    /// `struct msghdr`'s `msg_namelen`.
    message_name_length: u64,
    /// `struct msghdr`'s `msg_controllen`.
    message_control_length: u64,
}

impl NetworkFields {
    /// The nine offsets, or [`None`] if the daemon did not put them there.
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
            sock_bound_interface: kernel::offset(Field::SockBoundInterface)?,
            message_name: kernel::offset(Field::MessageName)?,
            message_name_length: kernel::offset(Field::MessageNameLength)?,
            message_control_length: kernel::offset(Field::MessageControlLength)?,
        })
    }
}
