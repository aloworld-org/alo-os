//! Hearing from the kernel that a program let go of the port presence advertises.
//!
//! *Machines find each other with zero configuration.* The port is listened on
//! once per network (`crate::listeners`), and on a network where another program
//! already holds it the kernel refuses that listener `EADDRINUSE`. The service
//! log then says a machine on that network cannot reach this one **until it can
//! be** — and before this file nothing tried again when it could be: the
//! listeners are moved when the kernel says a network changed, and a program
//! closing a socket is not a network change. An installer that took the port for
//! a moment at start-up, or a service restarting, left this machine found on that
//! network and unreachable there until some cable somewhere changed.
//!
//! # How the service learns the port is free
//!
//! Four readings were weighed, and the constraint was the plan's: no interval, no
//! polling, and a kernel reading the service **as the unit runs it** really gets —
//! as the person, with an empty capability bounding set (ADR 0001 §2, ADR 0018).
//!
//! - **The kernel's own message that a TCP socket was destroyed** — this file.
//!   `NETLINK_SOCK_DIAG` has had multicast groups for it since Linux 4.10
//!   (`SKNLGRP_INET_TCP_DESTROY`, `SKNLGRP_INET6_TCP_DESTROY`), and the kernel
//!   lets a process with **no capabilities at all** join them: measured
//!   2026-09-17 on `6.18.33.2`, as uid 1000 with `CapEff` zero, in the network
//!   namespace that process is in (`docs/quirks.md`). The message is sent after
//!   the socket has left the kernel's bind tables, so a bind made on hearing it
//!   is not refused by the socket it describes — measured the same day.
//! - **Waiting on the program that holds the port** (a `pidfd`). Which process
//!   holds a socket is read from another process's `/proc/<pid>/fd`, which a
//!   person's service may not read for a program running as somebody else — the
//!   installer is root — and a program can close the socket and go on running.
//! - **Asking again on an interval.** What the constraint rules out, and a wake
//!   on a machine where nothing happened.
//! - **Knocking on the other program's port and waiting for the reset.** A
//!   connection delivered into somebody else's service, which it may accept,
//!   log, or answer; and a program that only bound the port and never listened
//!   refuses the knock outright.
//!
//! # And it hears only its own port
//!
//! Joined as it is, the group carries the destruction of **every** TCP socket in
//! this network namespace: every connection any program on the machine closes,
//! with its addresses. The service has no business reading that, and would be
//! woken for each. So before the socket joins, a classic socket filter is
//! attached to it ([`only_the_port`]) that the kernel runs on every message and
//! that keeps a message only where the destroyed socket's **local port** is this
//! one — the sixteen bits at `inet_diag_msg.id.idiag_sport`, in network order.
//! Everything else is dropped in the kernel and never reaches this process.
//! Attaching a classic filter needs no capability.
//!
//! What still arrives is not *the port is free*: a connection the wire itself
//! accepted and closed is destroyed at this port too, and so is one of the other
//! program's while its listener goes on holding the port. **The message is a
//! reason to try again, never proof the try will bind**; a bind still refused is
//! still refused, and `crate::listeners` says so once and not on every try.
//!
//! # What is subscribed, and when
//!
//! The socket is opened **before** the first listener is bound, so a program
//! letting go between the kernel refusing a bind and the service subscribing
//! cannot be missed. A message the kernel dropped because the socket was not read
//! in time (`ENOBUFS`) is any port having been let go of, as
//! `crate::interfaces_that_went` reads a dropped routing message.

use std::os::fd::OwnedFd;

use rustix::net::netlink::SocketAddrNetlink;
use socket2::SockFilter;

/// `SKNLGRP_INET_TCP_DESTROY`, the group's number.
const SKNLGRP_INET_TCP_DESTROY: u32 = 1;

/// `SKNLGRP_INET6_TCP_DESTROY`, the group's number.
const SKNLGRP_INET6_TCP_DESTROY: u32 = 3;

/// `SKNLGRP_INET_TCP_DESTROY`, as a bit in the groups a netlink socket binds to.
const IPV4_TCP_DESTROYED: u32 = 1 << (SKNLGRP_INET_TCP_DESTROY - 1);

/// `SKNLGRP_INET6_TCP_DESTROY`, as a bit in the groups a netlink socket binds to
/// — a dual-stack IPv6 listener holds the IPv4 port as well.
const IPV6_TCP_DESTROYED: u32 = 1 << (SKNLGRP_INET6_TCP_DESTROY - 1);

/// Where `inet_diag_msg.id.idiag_sport` begins in a message the kernel sends:
/// after the sixteen-byte `nlmsghdr`, and the family, state, timer and
/// retransmit bytes.
const THE_LOCAL_PORT_AT: u32 = 16 + 4;

/// `BPF_LD | BPF_H | BPF_ABS`: load sixteen bits at an offset, in network order.
const LOAD_SIXTEEN_BITS: u16 = 0x28;

/// `BPF_JMP | BPF_JEQ | BPF_K`: compare with a constant.
const IF_EQUAL: u16 = 0x15;

/// `BPF_RET | BPF_K`: keep this many bytes of the message, and none for zero.
const KEEP: u16 = 0x06;

/// The classic socket filter that keeps a message only where the socket it
/// describes was at `port`.
///
/// A message too short to hold a port fails the load, which a classic filter
/// answers by keeping nothing.
#[must_use]
pub fn only_the_port(port: u16) -> [SockFilter; 4] {
    [
        SockFilter::new(LOAD_SIXTEEN_BITS, 0, 0, THE_LOCAL_PORT_AT),
        SockFilter::new(IF_EQUAL, 0, 1, u32::from(port)),
        SockFilter::new(KEEP, 0, 0, u32::MAX),
        SockFilter::new(KEEP, 0, 0, 0),
    ]
}

/// A socket the kernel writes to whenever a TCP socket at `port` is destroyed,
/// in either family, in this network namespace — and never for any other port.
///
/// Not blocking, so the service waits on it beside everything else and empties
/// it without sleeping (`crate::unix::emptied`). The filter is attached before
/// the socket joins, so not one message about another port is ever queued on it.
///
/// # Errors
///
/// Whatever the kernel answered: a kernel built without `CONFIG_INET_DIAG`
/// refuses the join, and everything opening a socket can fail with.
pub fn told_when_let_go_of(port: u16) -> Result<OwnedFd, std::io::Error> {
    let socket = rustix::net::socket_with(
        rustix::net::AddressFamily::NETLINK,
        rustix::net::SocketType::DGRAM,
        rustix::net::SocketFlags::NONBLOCK | rustix::net::SocketFlags::CLOEXEC,
        Some(rustix::net::netlink::SOCK_DIAG),
    )?;
    socket2::SockRef::from(&socket).attach_filter(&only_the_port(port))?;
    rustix::net::bind(
        &socket,
        &SocketAddrNetlink::new(0, IPV4_TCP_DESTROYED | IPV6_TCP_DESTROYED),
    )?;
    Ok(socket)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr, TcpListener, TcpStream};
    use std::os::fd::{AsFd as _, OwnedFd};
    use std::time::Duration;

    use super::told_when_let_go_of;
    use crate::unix::{emptied, ready};

    /// A port nothing else on this machine is on — and, while the listener
    /// handed back is held, nothing else can be.
    fn a_port_held() -> (u16, TcpListener) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        (listener.local_addr().unwrap().port(), listener)
    }

    /// Whether the kernel said anything on `told` within a moment, emptying it.
    fn told_anything(told: &OwnedFd) -> bool {
        let said = ready(&[Some(told.as_fd())], Some(Duration::from_secs(2))).unwrap();
        let mut heard = false;
        emptied(told.as_fd(), &mut |_| heard = true).unwrap();
        said == [true] && heard
    }

    /// **A listener at the port let go of is heard at once, and the port binds
    /// on hearing it** — over IPv4.
    #[test]
    fn a_listener_let_go_of_is_heard_and_the_port_then_binds() {
        let (port, holding) = a_port_held();
        let told = told_when_let_go_of(port).unwrap();
        assert!(
            TcpListener::bind((Ipv4Addr::LOCALHOST, port)).is_err(),
            "the port was not held"
        );
        drop(holding);
        assert!(
            told_anything(&told),
            "nothing was heard when the port was let go of"
        );
        drop(TcpListener::bind((Ipv4Addr::LOCALHOST, port)).unwrap());
    }

    /// **The same over IPv6**, the family a dual-stack program holds an IPv4
    /// port through.
    #[test]
    fn a_listener_let_go_of_over_ipv6_is_heard() {
        let Ok(holding) = TcpListener::bind((Ipv6Addr::LOCALHOST, 0)) else {
            // A host with no IPv6 loopback has no IPv6 listener to let go of.
            return;
        };
        let port = holding.local_addr().unwrap().port();
        let told = told_when_let_go_of(port).unwrap();
        drop(holding);
        assert!(
            told_anything(&told),
            "an IPv6 listener let go of was not heard"
        );
    }

    /// **Every other port is dropped in the kernel**: a listener at another
    /// port let go of, and a connection at another port closed, never reach the
    /// socket at all.
    #[test]
    fn a_socket_at_any_other_port_is_never_heard() {
        let (port, _holding) = a_port_held();
        let told = told_when_let_go_of(port).unwrap();

        let (_, elsewhere) = a_port_held();
        let at = elsewhere.local_addr().unwrap();
        let connection = TcpStream::connect(at).unwrap();
        let (accepted, _) = elsewhere.accept().unwrap();
        drop(connection);
        drop(accepted);
        drop(elsewhere);

        assert!(
            !told_anything(&told),
            "the destruction of a socket at another port reached the service"
        );
    }

    /// **A connection at the port closing is heard too**, while the listener
    /// goes on holding the port: which is why hearing is a reason to try again
    /// and never proof the port is free.
    #[test]
    fn a_connection_at_the_port_is_heard_while_the_port_stays_held() {
        let (port, holding) = a_port_held();
        let told = told_when_let_go_of(port).unwrap();
        let connection = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        let (accepted, _) = holding.accept().unwrap();
        drop(accepted);
        drop(connection);
        assert!(told_anything(&told));
        assert!(
            TcpListener::bind((Ipv4Addr::LOCALHOST, port)).is_err(),
            "the port was free while its listener held it"
        );
    }
}
