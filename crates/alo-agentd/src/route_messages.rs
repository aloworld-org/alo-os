//! What the kernel says about this machine's interfaces, read out of its own
//! routing messages.
//!
//! The standard library cannot list a machine's interfaces, and `getifaddrs` has
//! no safe spelling. The kernel's routing socket answers the same question with
//! plain bytes: one request for every link — its index, name and `IFF_*` flags —
//! and one for every IPv4 address with the index of the link it is on. This
//! file writes the two requests and reads the answers; `crate::unix` is the only
//! file that puts them on a socket, and `crate::networks` decides what is
//! joined. Reading is a function of bytes, so a message the kernel never sent in
//! a test is still a test.
//!
//! # What is read, and what is refused
//!
//! A message is read only as far as its own length says, and a length that runs
//! past what arrived, or is shorter than a header, is refused rather than read
//! around — a machine that half-read its interfaces would join on half of them
//! and say nothing. An attribute this file does not read is stepped over by its
//! length. An address for an interface no link message named is stepped over:
//! the two answers are separate dumps, and an interface that appeared between
//! them is picked up by the next look.

use std::net::Ipv4Addr;

use crate::networks::Interface;

/// `RTM_NEWLINK`: one link, in a dump or a notification.
const RTM_NEWLINK: u16 = 16;
/// `RTM_GETLINK`: ask for links.
const RTM_GETLINK: u16 = 18;
/// `RTM_NEWADDR`: one address, in a dump or a notification.
const RTM_NEWADDR: u16 = 20;
/// `RTM_GETADDR`: ask for addresses.
const RTM_GETADDR: u16 = 22;
/// `NLMSG_ERROR`: the kernel refused, or acknowledged.
const NLMSG_ERROR: u16 = 2;
/// `NLMSG_DONE`: the end of a dump.
const NLMSG_DONE: u16 = 3;
/// `NLM_F_REQUEST | NLM_F_DUMP`.
const A_DUMP: u16 = 0x1 | 0x300;
/// `AF_INET`.
const AF_INET: u8 = 2;
/// `IFLA_IFNAME`: a link's name.
const IFLA_IFNAME: u16 = 3;
/// `IFA_ADDRESS`: an address, or on a point-to-point link the far end's.
const IFA_ADDRESS: u16 = 1;
/// `IFA_LOCAL`: the local address, where it differs from `IFA_ADDRESS`.
const IFA_LOCAL: u16 = 2;

/// The length of a message header.
const HEADER: usize = 16;
/// The length of `struct ifinfomsg`.
const LINK: usize = 16;
/// The length of `struct ifaddrmsg`.
const ADDRESS: usize = 8;

/// The request for every link on this machine.
#[must_use]
pub fn every_link() -> Vec<u8> {
    a_request(RTM_GETLINK, &[0; LINK])
}

/// The request for every IPv4 address on this machine.
#[must_use]
pub fn every_address() -> Vec<u8> {
    a_request(RTM_GETADDR, &[AF_INET, 0, 0, 0, 0, 0, 0, 0])
}

/// A dump request of `kind`, carrying `body`.
fn a_request(kind: u16, body: &[u8]) -> Vec<u8> {
    let length = u32::try_from(HEADER + body.len()).unwrap_or(u32::MAX);
    let mut request = Vec::with_capacity(HEADER + body.len());
    request.extend_from_slice(&length.to_ne_bytes());
    request.extend_from_slice(&kind.to_ne_bytes());
    request.extend_from_slice(&A_DUMP.to_ne_bytes());
    request.extend_from_slice(&1_u32.to_ne_bytes());
    request.extend_from_slice(&0_u32.to_ne_bytes());
    request.extend_from_slice(body);
    request
}

/// Why what the kernel answered could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotReported {
    /// A message said it was longer than what arrived, or shorter than itself.
    #[error("the kernel's interface report was cut short")]
    CutShort,
    /// The kernel answered the request with an error.
    #[error("the kernel refused to report its interfaces (error {0})")]
    Refused(i32),
    /// The socket would not open, send or read.
    #[error("the kernel's routing socket: {0}")]
    TheSocket(String),
}

/// Whether this datagram ends a dump: it carries the end, or an error.
///
/// # Errors
///
/// [`NotReported`] when the datagram cannot be read as messages.
pub fn ends_a_dump(datagram: &[u8]) -> Result<bool, NotReported> {
    Ok(messages(datagram)?
        .iter()
        .any(|(kind, _)| *kind == NLMSG_DONE || *kind == NLMSG_ERROR))
}

/// Every interface in what the kernel answered to [`every_link`] and
/// [`every_address`], with the addresses on each.
///
/// # Errors
///
/// [`NotReported::CutShort`] when a message or attribute runs past what
/// arrived, and [`NotReported::Refused`] when the kernel answered with an error.
pub fn interfaces_in(links: &[u8], addresses: &[u8]) -> Result<Vec<Interface>, NotReported> {
    let mut interfaces = Vec::new();
    for (kind, body) in messages(links)? {
        refused_in(kind, body)?;
        if kind != RTM_NEWLINK {
            continue;
        }
        let fixed = body.get(..LINK).ok_or(NotReported::CutShort)?;
        let index = four(fixed, 4)?;
        let flags = four(fixed, 8)?;
        let mut name = String::new();
        for (attribute, value) in attributes(body.get(LINK..).unwrap_or_default())? {
            if attribute == IFLA_IFNAME {
                let until = value
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(value.len());
                name = String::from_utf8_lossy(value.get(..until).unwrap_or_default()).into_owned();
            }
        }
        interfaces.push(Interface {
            index,
            name,
            flags,
            addresses: Vec::new(),
        });
    }
    for (kind, body) in messages(addresses)? {
        refused_in(kind, body)?;
        if kind != RTM_NEWADDR {
            continue;
        }
        let fixed = body.get(..ADDRESS).ok_or(NotReported::CutShort)?;
        if fixed.first() != Some(&AF_INET) {
            continue;
        }
        let index = four(fixed, 4)?;
        let mut address = None;
        let mut local = None;
        for (attribute, value) in attributes(body.get(ADDRESS..).unwrap_or_default())? {
            let Ok(octets) = <[u8; 4]>::try_from(value) else {
                continue;
            };
            match attribute {
                IFA_ADDRESS => address = Some(Ipv4Addr::from(octets)),
                IFA_LOCAL => local = Some(Ipv4Addr::from(octets)),
                _ => {}
            }
        }
        if let (Some(interface), Some(at)) = (
            interfaces
                .iter_mut()
                .find(|interface| interface.index == index),
            local.or(address),
        ) {
            interface.addresses.push(at);
        }
    }
    Ok(interfaces)
}

/// Every interface on this machine, as the kernel reports it now.
///
/// # Errors
///
/// [`NotReported`] when the routing socket will not answer or its answer cannot
/// be read.
pub fn reported_by_the_kernel() -> Result<Vec<Interface>, NotReported> {
    let dumped = |request: &[u8]| {
        crate::unix::a_route_dump(request, |datagram| ends_a_dump(datagram).unwrap_or(true))
            .map_err(|why| NotReported::TheSocket(why.to_string()))
    };
    interfaces_in(&dumped(&every_link())?, &dumped(&every_address())?)
}

/// [`NotReported::Refused`] when this is an error message carrying an error.
fn refused_in(kind: u16, body: &[u8]) -> Result<(), NotReported> {
    if kind != NLMSG_ERROR {
        return Ok(());
    }
    let code = i32::from_ne_bytes(
        body.get(..4)
            .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
            .ok_or(NotReported::CutShort)?,
    );
    if code == 0 {
        Ok(())
    } else {
        Err(NotReported::Refused(code))
    }
}

/// The messages in `said`, each its kind and body, as far as each length says.
fn messages(said: &[u8]) -> Result<Vec<(u16, &[u8])>, NotReported> {
    let mut found = Vec::new();
    let mut at = 0_usize;
    while at < said.len() {
        let header = said.get(at..at + HEADER).ok_or(NotReported::CutShort)?;
        let length = usize::try_from(four(header, 0)?).map_err(|_| NotReported::CutShort)?;
        if length < HEADER {
            return Err(NotReported::CutShort);
        }
        let kind = two(header, 4)?;
        let body = said
            .get(at + HEADER..at + length)
            .ok_or(NotReported::CutShort)?;
        found.push((kind, body));
        at = at.saturating_add(aligned(length));
    }
    Ok(found)
}

/// The attributes in `said`, each its type and value.
fn attributes(said: &[u8]) -> Result<Vec<(u16, &[u8])>, NotReported> {
    let mut found = Vec::new();
    let mut at = 0_usize;
    while at + 4 <= said.len() {
        let length = usize::from(two(said, at)?);
        if length < 4 {
            return Err(NotReported::CutShort);
        }
        let kind = two(said, at + 2)? & 0x3fff;
        let value = said.get(at + 4..at + length).ok_or(NotReported::CutShort)?;
        found.push((kind, value));
        at = at.saturating_add(aligned(length));
    }
    Ok(found)
}

/// Two native-endian bytes at `at`, as a number.
fn two(said: &[u8], at: usize) -> Result<u16, NotReported> {
    said.get(at..at + 2)
        .and_then(|bytes| <[u8; 2]>::try_from(bytes).ok())
        .map(u16::from_ne_bytes)
        .ok_or(NotReported::CutShort)
}

/// Four native-endian bytes at `at`, as a number.
fn four(said: &[u8], at: usize) -> Result<u32, NotReported> {
    said.get(at..at + 4)
        .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
        .map(u32::from_ne_bytes)
        .ok_or(NotReported::CutShort)
}

/// `length` rounded up to the four bytes every message and attribute is padded
/// to.
const fn aligned(length: usize) -> usize {
    length.saturating_add(3) & !3
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::networks::{IFF_LOOPBACK, IFF_MULTICAST, IFF_RUNNING, IFF_UP};

    /// One message of `kind` carrying `body`, padded.
    fn a_message(kind: u16, body: &[u8]) -> Vec<u8> {
        let mut message = a_request(kind, body);
        while !message.len().is_multiple_of(4) {
            message.push(0);
        }
        message
    }

    /// One attribute, padded.
    fn an_attribute(kind: u16, value: &[u8]) -> Vec<u8> {
        let mut attribute = Vec::new();
        attribute.extend_from_slice(&u16::try_from(4 + value.len()).unwrap().to_ne_bytes());
        attribute.extend_from_slice(&kind.to_ne_bytes());
        attribute.extend_from_slice(value);
        while !attribute.len().is_multiple_of(4) {
            attribute.push(0);
        }
        attribute
    }

    /// A link message.
    fn a_link(index: u32, name: &str, flags: u32) -> Vec<u8> {
        let mut body = vec![0_u8; 4];
        body.extend_from_slice(&index.to_ne_bytes());
        body.extend_from_slice(&flags.to_ne_bytes());
        body.extend_from_slice(&[0; 4]);
        // An attribute this file does not read comes first, and is stepped over.
        body.extend(an_attribute(4, &1_500_u32.to_ne_bytes()));
        let mut named = name.as_bytes().to_vec();
        named.push(0);
        body.extend(an_attribute(IFLA_IFNAME, &named));
        a_message(RTM_NEWLINK, &body)
    }

    /// An address message.
    fn an_address(index: u32, family: u8, attributes: &[(u16, [u8; 4])]) -> Vec<u8> {
        let mut body = vec![family, 0, 0, 0];
        body.extend_from_slice(&index.to_ne_bytes());
        for (kind, value) in attributes {
            body.extend(an_attribute(*kind, value));
        }
        a_message(RTM_NEWADDR, &body)
    }

    /// The end of a dump.
    fn the_end() -> Vec<u8> {
        a_message(NLMSG_DONE, &0_i32.to_ne_bytes())
    }

    /// **The interfaces and their addresses are read out of the kernel's two
    /// answers**, the local address preferred over a point-to-point peer's, and
    /// an IPv6 address stepped over.
    #[test]
    fn interfaces_and_their_addresses_are_read_out_of_the_answers() {
        let joinable = IFF_UP | IFF_RUNNING | IFF_MULTICAST;
        let links = [
            a_link(1, "lo", IFF_UP | IFF_RUNNING | IFF_LOOPBACK),
            a_link(2, "eth0", joinable),
            a_link(3, "wlan0", joinable),
            the_end(),
        ]
        .concat();
        let addresses = [
            an_address(1, AF_INET, &[(IFA_ADDRESS, [127, 0, 0, 1])]),
            an_address(2, AF_INET, &[(IFA_ADDRESS, [10, 61, 1, 1])]),
            an_address(
                3,
                AF_INET,
                &[(IFA_ADDRESS, [10, 61, 2, 254]), (IFA_LOCAL, [10, 61, 2, 1])],
            ),
            an_address(2, 10, &[(IFA_ADDRESS, [0xfe, 0x80, 0, 0])]),
            an_address(9, AF_INET, &[(IFA_ADDRESS, [10, 9, 9, 9])]),
            the_end(),
        ]
        .concat();

        let interfaces = interfaces_in(&links, &addresses).unwrap();
        assert_eq!(
            interfaces,
            vec![
                Interface {
                    index: 1,
                    name: "lo".to_owned(),
                    flags: IFF_UP | IFF_RUNNING | IFF_LOOPBACK,
                    addresses: vec![Ipv4Addr::LOCALHOST],
                },
                Interface {
                    index: 2,
                    name: "eth0".to_owned(),
                    flags: joinable,
                    addresses: vec![Ipv4Addr::new(10, 61, 1, 1)],
                },
                Interface {
                    index: 3,
                    name: "wlan0".to_owned(),
                    flags: joinable,
                    addresses: vec![Ipv4Addr::new(10, 61, 2, 1)],
                },
            ]
        );
        assert!(ends_a_dump(&the_end()).unwrap());
        assert!(!ends_a_dump(&a_link(2, "eth0", joinable)).unwrap());
    }

    /// **A message that runs past what arrived is refused**, not read around.
    #[test]
    fn a_message_cut_short_is_refused() {
        let link = a_link(2, "eth0", IFF_UP);
        let cut = link.get(..link.len() - 8).unwrap();
        assert_eq!(interfaces_in(cut, &[]), Err(NotReported::CutShort));

        let link = a_link(2, "eth0", IFF_UP);
        let lying = [&4_u32.to_ne_bytes(), link.get(4..).unwrap()].concat();
        assert_eq!(interfaces_in(&lying, &[]), Err(NotReported::CutShort));
    }

    /// **The kernel refusing the request is refused**, and its acknowledgement
    /// is not.
    #[test]
    fn the_kernel_refusing_is_refused_and_an_acknowledgement_is_not() {
        let refused = a_message(NLMSG_ERROR, &(-13_i32).to_ne_bytes());
        assert_eq!(interfaces_in(&refused, &[]), Err(NotReported::Refused(-13)));
        let acknowledged = a_message(NLMSG_ERROR, &0_i32.to_ne_bytes());
        assert_eq!(interfaces_in(&acknowledged, &[]), Ok(Vec::new()));
    }

    /// **The kernel on this host answers**, and loopback is among what it says —
    /// so the request is one it reads.
    #[test]
    fn the_kernel_on_this_host_reports_its_interfaces() {
        let interfaces = reported_by_the_kernel().unwrap();
        assert!(
            interfaces
                .iter()
                .any(|interface| interface.flags & IFF_LOOPBACK != 0
                    && interface.addresses.contains(&Ipv4Addr::LOCALHOST)),
            "{interfaces:?}"
        );
    }
}
