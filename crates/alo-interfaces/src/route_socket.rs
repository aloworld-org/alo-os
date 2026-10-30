//! The kernel's routing socket: the one place in this crate that opens one.
//!
//! `rustix` is named here and nowhere else in this crate. What the bytes mean is
//! `crate::route_messages`'s; this file opens the socket, sends one request and
//! reads the answer back.

use std::time::Duration;

/// Put one request on the kernel's routing socket and read everything it
/// answers, until `ended` says a datagram was the last.
///
/// The standard library has no netlink socket, and
/// `crate::route_messages::reported_by_the_kernel` needs one to ask which
/// interfaces this machine has. A kernel that says nothing for two seconds is
/// an error rather than a hang, and so is a dump that goes on past a thousand
/// datagrams. The socket is closed when this returns.
///
/// # Errors
///
/// Whatever the machine said, as a `std::io::Error`.
pub(crate) fn a_route_dump(
    request: &[u8],
    ended: impl Fn(&[u8]) -> bool,
) -> Result<Vec<u8>, std::io::Error> {
    use rustix::net::netlink::SocketAddrNetlink;

    let socket = rustix::net::socket(
        rustix::net::AddressFamily::NETLINK,
        rustix::net::SocketType::DGRAM,
        // `NETLINK_ROUTE` is protocol zero, which is how `None` is spelt.
        None,
    )?;
    rustix::net::sockopt::set_socket_timeout(
        &socket,
        rustix::net::sockopt::Timeout::Recv,
        Some(Duration::from_secs(2)),
    )?;
    rustix::net::sendto(
        &socket,
        request,
        rustix::net::SendFlags::empty(),
        &SocketAddrNetlink::new(0, 0),
    )?;
    let mut answered = Vec::new();
    let mut datagram = vec![0_u8; 32_768];
    for _ in 0..1_000 {
        let (read, _) = rustix::net::recv(
            &socket,
            datagram.as_mut_slice(),
            rustix::net::RecvFlags::empty(),
        )?;
        let said = datagram
            .get(..read)
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidData))?;
        answered.extend_from_slice(said);
        if ended(said) {
            return Ok(answered);
        }
    }
    Err(std::io::Error::from(std::io::ErrorKind::InvalidData))
}
