//! Which connection a command is waiting on, and where it goes.
//!
//! `docs/features.md`: *ask, and get [the five answers] — then show the prompt,
//! **inspect the connection**, reveal what is blocking it, or stop.* This is the
//! second of those four, and it follows
//! [`crate::WhyItIsWaiting::ForTheNetwork`] — which `docs/quirks.md` records can
//! only be told from *waiting for the keyboard* by what the command reads from,
//! because both park in the same kernel function.
//!
//! So this answers the question that answer raises: **waiting for which?**
//!
//! # Four unprivileged reads, and the hex is the hard part
//!
//! The descriptor says `socket:[N]`; `N` appears in one of the kernel's own
//! tables; the line holds the two addresses. Nothing here is privileged and
//! nothing here is a guess.
//!
//! What it is easy to get wrong is the encoding, so it was measured before it
//! was written. From `/proc/net/tcp` on 2026-10-17, for a socket bound to
//! loopback on an arbitrary port:
//!
//! ```text
//!  1: 0100007F:AEAB 00000000:0000 0A ... 20249653 1 ...
//!     ^^^^^^^^ ^^^^                      ^^^^^^^^
//!     address  port                      the inode, token ten
//! ```
//!
//! **The address is little-endian and the port is not.** `0100007F` is
//! `127.0.0.1` read backwards a byte at a time, and `AEAB` is 44715 read
//! forwards. Getting one of the two the wrong way round gives an address that
//! looks plausible and is somewhere else entirely — `1.0.0.127`, or port 43950 —
//! which is why both are in the tests as the strings the kernel really wrote.
//!
//! # Why the inode is token ten when the header lists eleven columns
//!
//! Because `tx_queue` and `rx_queue` share one token, `00000000:00000000`, and
//! so do `tr` and `tm->when`. A reader that counted header columns would be two
//! out and would read the uid as an inode — a number that is nothing, matching
//! nothing, for ever.

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Where a socket goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheConnection {
    /// A network socket, with the two ends as this machine records them.
    ///
    /// `away` is [`None`] for a socket that is listening and has no far end
    /// yet, which is a real state and not a failure to read one.
    OverTheNetwork {
        /// Which of the kernel's tables it was found in: `tcp`, `tcp6`, `udp`
        /// or `udp6`.
        kind: &'static str,
        /// This end.
        here: SocketAddr,
        /// The far end, where there is one.
        away: Option<SocketAddr>,
    },
    /// A socket on this machine only, named by its path where it has one.
    ///
    /// An unnamed one is a pair made by a program for itself, which is why the
    /// path is an option rather than an empty string.
    OnThisMachine {
        /// Where it is in the filesystem, where it is anywhere.
        path: Option<String>,
    },
}

/// One end of a network socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SocketAddr {
    /// The address.
    pub address: IpAddr,
    /// The port.
    pub port: u16,
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, into: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.address {
            IpAddr::V4(address) => write!(into, "{address}:{}", self.port),
            IpAddr::V6(address) => write!(into, "[{address}]:{}", self.port),
        }
    }
}

impl fmt::Display for TheConnection {
    fn fmt(&self, into: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OverTheNetwork {
                kind,
                here,
                away: Some(away),
            } => write!(into, "{kind} from {here} to {away}"),
            Self::OverTheNetwork {
                kind,
                here,
                away: None,
            } => write!(into, "{kind} listening at {here}"),
            Self::OnThisMachine { path: Some(path) } => {
                write!(into, "a socket on this machine at {path}")
            }
            Self::OnThisMachine { path: None } => {
                write!(into, "a socket on this machine with no name")
            }
        }
    }
}

/// The kernel's tables of network sockets, and what to call each.
const THE_TABLES: [(&str, &str); 4] = [
    ("tcp", "/proc/net/tcp"),
    ("tcp6", "/proc/net/tcp6"),
    ("udp", "/proc/net/udp"),
    ("udp6", "/proc/net/udp6"),
];

/// Where the kernel lists sockets that never leave this machine.
const ON_THIS_MACHINE: &str = "/proc/net/unix";

/// The connection `process` is reading from, if it is reading one at all.
///
/// Asks about its standard input, which is what
/// [`crate::why_a_command_is_waiting`] decided *waiting for the network* from —
/// so this answers about the same descriptor that produced the answer, and
/// cannot name a different socket the command happens to hold.
///
/// [`None`] when its input is not a socket, or when the machine will not say
/// which — a socket the kernel has already closed, or a table this kernel does
/// not keep.
#[must_use]
pub fn the_connection_being_read_by(process: u32) -> Option<TheConnection> {
    let reading = std::fs::read_link(format!("/proc/{process}/fd/0")).ok()?;
    let inode = reading
        .to_str()?
        .strip_prefix("socket:[")?
        .strip_suffix(']')?
        .to_owned();
    the_connection_numbered(&inode)
}

/// The connection with this inode, from whichever table holds it.
#[must_use]
pub fn the_connection_numbered(inode: &str) -> Option<TheConnection> {
    for (kind, at) in THE_TABLES {
        let Ok(table) = std::fs::read_to_string(at) else {
            continue;
        };
        for line in table.lines().skip(1) {
            let mut said = line.split_whitespace();
            let _index = said.next()?;
            let here = said.next()?;
            let away = said.next()?;
            // st, tx:rx, tr:tm->when, retrnsmt, uid, timeout, then the inode.
            let found = said.nth(6)?;
            if found != inode {
                continue;
            }
            return Some(TheConnection::OverTheNetwork {
                kind,
                here: an_end(here)?,
                away: an_end(away).filter(|end| end.port != 0),
            });
        }
    }

    // Not on the network, so it may be one that never leaves this machine.
    let table = std::fs::read_to_string(ON_THIS_MACHINE).ok()?;
    for line in table.lines().skip(1) {
        let said: Vec<&str> = line.split_whitespace().collect();
        // The inode is the seventh column and a path, where there is one, the
        // eighth and last.
        if said.get(6) != Some(&inode) {
            continue;
        }
        return Some(TheConnection::OnThisMachine {
            path: said.get(7).map(|path| (*path).to_owned()),
        });
    }
    None
}

/// One end, as the kernel writes it: a hex address, a colon, a hex port.
///
/// **The address is little-endian and the port is not**, which this module's own
/// documentation measures. Both orders are wrong in a way that looks right, so
/// each is read deliberately rather than by whichever `from_str_radix` happens
/// to do.
#[must_use]
fn an_end(said: &str) -> Option<SocketAddr> {
    let (address, port) = said.split_once(':')?;
    let port = u16::from_str_radix(port, 16).ok()?;
    Some(SocketAddr {
        address: an_address(address)?,
        port,
    })
}

/// A hex address, however long, as an address.
#[must_use]
fn an_address(said: &str) -> Option<IpAddr> {
    match said.len() {
        8 => {
            let whole = u32::from_str_radix(said, 16).ok()?;
            // Little-endian: the first byte written is the last of the address.
            Some(IpAddr::V4(Ipv4Addr::from(whole.swap_bytes())))
        }
        32 => {
            // Four little-endian words, in order.
            let mut bytes = [0_u8; 16];
            for (which, word) in said.as_bytes().chunks(8).enumerate() {
                let word = std::str::from_utf8(word).ok()?;
                let whole = u32::from_str_radix(word, 16).ok()?;
                let at = which.checked_mul(4)?;
                bytes
                    .get_mut(at..at.checked_add(4)?)?
                    .copy_from_slice(&whole.to_le_bytes());
            }
            Some(IpAddr::V6(Ipv6Addr::from(bytes)))
        }
        _ => None,
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_connection_it_is_waiting_on_tests.rs"]
mod tests;
