//! The one file in this crate that opens a connection, and what it takes the
//! address from.
//!
//! Task 1 of the local-network plan held that discovery dials nothing, and it
//! still does not: `advertising.rs`, `looking.rs`, `reading.rs` and
//! `presence.rs` open no connection, and
//! `the_local_network_says_no_more_than_a_machine_exists.rs` still fails if
//! one appears in them. What task 7 adds is this file and `receiving.rs`,
//! and the address this file dials is never typed: it is
//! [`crate::Found::where_it_answers`] on the asking side and
//! [`crate::Waiting::where_the_other_answers`] — made from a `Found` — on the
//! asked side, both measured by discovery.
//!
//! Nothing here decides anything. What to send and what a reply means are
//! `proposals.rs`'; this puts bytes on a connection and brings bytes back.
//!
//! # And it is held to the network the machine was found on
//!
//! [ADR 0044](../../../docs/decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md).
//! `192.168.1.20` on the wired network and `192.168.1.20` on the Wi-Fi are two
//! machines whenever two routers hand out the same private range, and a socket
//! held to nothing leaves by whatever the route says at the moment it connects.
//! So a proposal and a confirmation are dialled from a socket **held to the
//! interface of the network the other machine was heard on**
//! (`SO_BINDTOIFINDEX`, as `alo_asking`'s corridor already holds a question and
//! `alo_agentd`'s discovery already holds a look): the kernel sends it out of
//! that interface or not at all.
//!
//! Without it, two machines on a network the route does not point at can be
//! found, can propose, and can never pair — the confirmation the asked machine
//! sends back would reach whoever the route reaches at the same address, which
//! is either nobody or the wrong machine.
//!
//! A machine heard at an address that names its own network is dialled from a
//! socket held to nothing, as it always was: a global address is one machine
//! wherever the route goes, and a link-local one carries its interface in the
//! address already.

use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::num::NonZeroU32;

use socket2::{Domain, SockAddr, Socket, Type};

use crate::http::{self, WHILE_THE_WIRE_ANSWERS};
use crate::refusing::{NotNearby, because};

/// What the other machine answered: a status and a body.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Answered {
    /// The status it answered with.
    pub(crate) status: u16,
    /// What it said, which is one line or nothing.
    pub(crate) body: String,
}

/// Put `body` to `path` at `at`, from a socket held to `on` where discovery
/// measured a network, and bring back what was answered.
///
/// # Errors
///
/// [`NotNearby::TheNetwork`] if the machine could not be reached, would not
/// take the request, or did not answer within [`WHILE_THE_WIRE_ANSWERS`];
/// [`NotNearby::NotAMessage`] if what it answered is not a reply this wire
/// carries.
pub(crate) fn put(
    at: SocketAddr,
    on: Option<NonZeroU32>,
    path: &str,
    body: &str,
) -> Result<Answered, NotNearby> {
    let mut stream = connected(at, on).map_err(|why| NotNearby::TheNetwork(because(&why)))?;
    stream
        .set_read_timeout(Some(WHILE_THE_WIRE_ANSWERS))
        .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
    stream
        .set_write_timeout(Some(WHILE_THE_WIRE_ANSWERS))
        .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
    stream
        .write_all(http::a_request(path, &at.to_string(), body).as_bytes())
        .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
    let message = http::read_message(&stream)?;
    let status = http::status_of(&message.first)?;
    Ok(Answered {
        status,
        body: message.body,
    })
}

/// A connection to `at`, held to the interface the kernel numbers `on` where
/// there is one — and the standard library's own connection where there is not.
fn connected(at: SocketAddr, on: Option<NonZeroU32>) -> Result<TcpStream, std::io::Error> {
    let Some(interface) = on else {
        return TcpStream::connect_timeout(&at, WHILE_THE_WIRE_ANSWERS);
    };
    let socket = Socket::new(Domain::for_address(at), Type::STREAM, None)?;
    held(&socket, interface, at)?;
    socket.connect_timeout(&SockAddr::from(at), WHILE_THE_WIRE_ANSWERS)?;
    Ok(socket.into())
}

/// Hold `socket` to `interface`, before it connects to `at`.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn held(socket: &Socket, interface: NonZeroU32, at: SocketAddr) -> Result<(), std::io::Error> {
    match at {
        SocketAddr::V4(_) => socket.bind_device_by_index_v4(Some(interface)),
        SocketAddr::V6(_) => socket.bind_device_by_index_v6(Some(interface)),
    }
}

/// Hold `socket` to `interface` — which a kernel alo OS does not run on is not
/// asked to do, so nothing is dialled rather than dialled by the route.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn held(_socket: &Socket, _interface: NonZeroU32, _at: SocketAddr) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "a socket can be held to an interface only on Linux",
    ))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::io::Write;
    use std::net::TcpListener;

    use super::{Answered, put};
    use crate::http;
    use crate::refusing::NotNearby;

    /// **A proposal is dialled from a socket held to the network the machine
    /// was found on**, and arrives — loopback is interface one in every Linux
    /// network namespace, and the one network a test on one machine has.
    #[test]
    #[cfg(target_os = "linux")]
    fn what_is_put_held_to_a_network_arrives_on_it() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let at = listener.local_addr().unwrap();
        let answering = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            drop(http::read_message(&stream));
            stream
                .write_all(http::a_reply(204, "No Content", "").as_bytes())
                .unwrap();
        });
        let loopback = std::num::NonZeroU32::new(1);
        let answered = put(
            at,
            loopback,
            "/alo-os/1/pairing/confirmation",
            "a line
",
        )
        .unwrap();
        assert_eq!(answered.status, 204);
        answering.join().unwrap();
    }

    /// **A proposal held to a network that is not there reaches nobody**: the
    /// kernel refuses it rather than sending it by the route, which is the whole
    /// of what holding it buys — the same address on another network is another
    /// machine.
    #[test]
    #[cfg(target_os = "linux")]
    fn what_is_put_held_to_a_network_that_is_not_there_reaches_nobody() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let at = listener.local_addr().unwrap();
        let nowhere = std::num::NonZeroU32::new(9_999);
        assert!(matches!(
            put(
                at,
                nowhere,
                "/alo-os/1/pairing/proposal",
                "a line
"
            )
            .unwrap_err(),
            NotNearby::TheNetwork(_)
        ));
        drop(listener);
    }

    /// What is put arrives as it was written, and what is answered comes back
    /// as it was written.
    #[test]
    fn what_is_put_arrives_and_what_is_answered_comes_back() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let at = listener.local_addr().unwrap();
        let answering = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let message = http::read_message(&stream).unwrap();
            stream
                .write_all(http::a_reply(200, "OK", "an answer\n").as_bytes())
                .unwrap();
            message
        });

        let answered = put(at, None, "/alo-os/1/pairing/proposal", "a line\n").unwrap();
        assert_eq!(
            answered,
            Answered {
                status: 200,
                body: "an answer\n".to_owned()
            }
        );
        let arrived = answering.join().unwrap();
        assert_eq!(arrived.first, "POST /alo-os/1/pairing/proposal HTTP/1.1");
        assert_eq!(arrived.body, "a line\n");
    }

    /// A machine that cannot be reached is the network refusing, said as
    /// such, and a machine that answers with something else is not a message.
    #[test]
    fn a_machine_that_cannot_be_reached_or_answers_nonsense_is_refused() {
        let nobody = TcpListener::bind("127.0.0.1:0").unwrap();
        let at = nobody.local_addr().unwrap();
        drop(nobody);
        assert!(matches!(
            put(at, None, "/alo-os/1/pairing/proposal", "a line\n").unwrap_err(),
            NotNearby::TheNetwork(_)
        ));

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let at = listener.local_addr().unwrap();
        let answering = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            drop(http::read_message(&stream));
            stream.write_all(b"220 mail.example.org ESMTP\r\n").unwrap();
        });
        assert!(matches!(
            put(at, None, "/alo-os/1/pairing/proposal", "a line\n").unwrap_err(),
            NotNearby::NotAMessage(_)
        ));
        answering.join().unwrap();
    }
}
