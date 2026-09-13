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

use std::io::Write;
use std::net::{SocketAddr, TcpStream};

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

/// Put `body` to `path` at `at`, and bring back what was answered.
///
/// # Errors
///
/// [`NotNearby::TheNetwork`] if the machine could not be reached, would not
/// take the request, or did not answer within [`WHILE_THE_WIRE_ANSWERS`];
/// [`NotNearby::NotAMessage`] if what it answered is not a reply this wire
/// carries.
pub(crate) fn put(at: SocketAddr, path: &str, body: &str) -> Result<Answered, NotNearby> {
    let mut stream = TcpStream::connect_timeout(&at, WHILE_THE_WIRE_ANSWERS)
        .map_err(|why| NotNearby::TheNetwork(because(&why)))?;
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

        let answered = put(at, "/alo-os/1/pairing/proposal", "a line\n").unwrap();
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
            put(at, "/alo-os/1/pairing/proposal", "a line\n").unwrap_err(),
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
            put(at, "/alo-os/1/pairing/proposal", "a line\n").unwrap_err(),
            NotNearby::NotAMessage(_)
        ));
        answering.join().unwrap();
    }
}
