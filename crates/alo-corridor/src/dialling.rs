//! The one file in this crate that opens a connection, and what it takes the
//! address from.
//!
//! The address is [`alo_nearby::Found::where_it_answers`], carried in by
//! `crossing.rs`, and no address is spelt here — the integration test holds
//! that nothing in this file parses one or makes one, as `alo-nearby`'s
//! guard does of its own dial, so the next hop is what discovery measured
//! rather than what somebody typed.
//!
//! **Private, and called from one place.** `crossing.rs` calls this after
//! the indicator has handed it an `alo_egress::Departing` and from nowhere
//! before, which is `alo-asking`'s arrangement for the same reason: a public
//! dial would be a way to reach another machine with law 1 having shown
//! nothing. Nothing here decides anything. What to send and what a reply
//! means are `crossing.rs`'; this puts bytes on a connection and brings
//! bytes back.

use std::io::{Cursor, Read as _, Write as _};
use std::net::{SocketAddr, TcpStream};

use alo_asking::THE_PROOF_HEADER;
use alo_nearby::http::{self, WHILE_THE_WIRE_ANSWERS};

use crate::answered::AT_MOST_AN_ANSWER;
use crate::refusing::WentBack;

/// The most bytes a reply is read from: the head, and an answer.
const AT_MOST_A_REPLY: u64 = 8 * 1024 + AT_MOST_AN_ANSWER as u64;

/// What the other machine answered: a status and a body.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Answered {
    /// The status it answered with.
    pub(crate) status: u16,
    /// What it said.
    pub(crate) body: String,
}

/// Put `body` to `path` at `at`, with `proof` in the header, and bring back
/// what was answered.
///
/// # Errors
///
/// [`WentBack::TheNetwork`] if the machine could not be reached, would not
/// take the request, did not answer within [`WHILE_THE_WIRE_ANSWERS`], or
/// closed the connection with nothing on it — which is what a machine whose
/// rule held the answer back does; [`WentBack::NotAnAnswer`] if what it
/// answered is not a reply this wire carries.
pub(crate) fn put(
    at: SocketAddr,
    path: &str,
    proof: &str,
    body: &str,
) -> Result<Answered, WentBack> {
    let network = |why: std::io::Error| WentBack::TheNetwork(why.to_string());
    let mut stream = TcpStream::connect_timeout(&at, WHILE_THE_WIRE_ANSWERS).map_err(network)?;
    stream
        .set_read_timeout(Some(WHILE_THE_WIRE_ANSWERS))
        .map_err(network)?;
    stream
        .set_write_timeout(Some(WHILE_THE_WIRE_ANSWERS))
        .map_err(network)?;
    let request =
        http::a_request_carrying(path, &at.to_string(), &[(THE_PROOF_HEADER, proof)], body);
    stream.write_all(request.as_bytes()).map_err(network)?;
    // Read to the end first: the reply says `connection: close`, and a
    // connection closed with nothing on it is the other machine having said
    // nothing, not the other machine having said something unreadable.
    let mut reply = Vec::new();
    (&stream)
        .take(AT_MOST_A_REPLY)
        .read_to_end(&mut reply)
        .map_err(network)?;
    if reply.is_empty() {
        return Err(WentBack::TheNetwork(
            "the connection closed with nothing on it".to_owned(),
        ));
    }
    let message = http::read_message_of_at_most(Cursor::new(reply), AT_MOST_AN_ANSWER)
        .map_err(|why| WentBack::NotAnAnswer(why.to_string()))?;
    let status =
        http::status_of(&message.first).map_err(|why| WentBack::NotAnAnswer(why.to_string()))?;
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
    use std::io::Write as _;
    use std::net::TcpListener;

    use alo_asking::THE_PROOF_HEADER;
    use alo_nearby::http;

    use super::{Answered, put};
    use crate::refusing::WentBack;

    /// What is put arrives as it was written, with the proof in its header,
    /// and what is answered comes back as it was written.
    #[test]
    fn what_is_put_arrives_with_its_proof_and_what_is_answered_comes_back() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let at = listener.local_addr().unwrap();
        let answering = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let message = http::read_message(&stream).unwrap();
            stream
                .write_all(http::a_reply(200, "OK", r#"{"waits":{"number":1}}"#).as_bytes())
                .unwrap();
            message
        });

        let answered = put(at, "/alo-os/1/verb/change", "alo-os/1 a b 1 2 ff", "{}").unwrap();
        assert_eq!(
            answered,
            Answered {
                status: 200,
                body: r#"{"waits":{"number":1}}"#.to_owned()
            }
        );
        let arrived = answering.join().unwrap();
        assert_eq!(arrived.first, "POST /alo-os/1/verb/change HTTP/1.1");
        assert_eq!(
            arrived.header(THE_PROOF_HEADER),
            Some("alo-os/1 a b 1 2 ff")
        );
        assert_eq!(arrived.body, "{}");
    }

    /// A machine that cannot be reached is the network, and one that answers
    /// with something else is not an answer.
    #[test]
    fn a_machine_that_cannot_be_reached_or_answers_nonsense_is_said_so() {
        let nobody = TcpListener::bind("127.0.0.1:0").unwrap();
        let at = nobody.local_addr().unwrap();
        drop(nobody);
        assert!(matches!(
            put(at, "/alo-os/1/verb/read", "p", "{}").unwrap_err(),
            WentBack::TheNetwork(_)
        ));

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let at = listener.local_addr().unwrap();
        let answering = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            drop(http::read_message(&stream));
            stream.write_all(b"220 mail.example.org ESMTP\r\n").unwrap();
        });
        assert!(matches!(
            put(at, "/alo-os/1/verb/read", "p", "{}").unwrap_err(),
            WentBack::NotAnAnswer(_)
        ));
        answering.join().unwrap();
    }
}
