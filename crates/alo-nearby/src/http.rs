//! The framing both ends of the pairing wire share: the least of HTTP/1.1
//! that carries one line there and one line back.
//!
//! # Why HTTP, and why this little of it
//!
//! The port a machine advertises is the one the corridor already puts
//! questions to over HTTP, and the verb wire after this one carries its proof
//! in a header. One port, one shape: a daemon that answers on it can tell a
//! proposal from a question by the path, rather than by guessing which
//! protocol a stranger's first bytes are in.
//!
//! But only this much of it. A request is a `POST` with a `content-length`
//! and `connection: close`; a reply is a status line with the same two. No
//! chunked bodies, no keep-alive, no pipelining, nothing over eight
//! kibibytes, nothing that is not text. What is not read cannot be got
//! wrong, and everything on a network can reach this port.
//!
//! Hand-written rather than rented for the reason the DNS packets in
//! `advertising.rs` are: what this crate reads off a port is small and
//! closed, and a dependency that read more would be more to be wrong in.

use std::io::{BufRead, BufReader, Read};
use std::time::Duration;

use crate::refusing::{NotNearby, because};

/// How long either end waits for the other on one connection.
///
/// Ten seconds. The other machine is in the building, and what it has to do
/// before answering is show a value to a surface, not think.
pub(crate) const WHILE_THE_WIRE_ANSWERS: Duration = Duration::from_secs(10);

/// The most bytes one message is read from.
const AT_MOST_A_MESSAGE: u64 = 8 * 1024;

/// The most bytes a body may say it is.
const AT_MOST_A_BODY: usize = 4 * 1024;

/// The most headers a message here has.
const AT_MOST_HEADERS: usize = 32;

/// The one header that says how long the body is.
const CONTENT_LENGTH: &str = "content-length";

/// The header that would mean a body this wire does not carry.
const TRANSFER_ENCODING: &str = "transfer-encoding";

/// What every message here says its body is.
const TEXT: &str = "text/plain; charset=utf-8";

/// One message, read: its first line and its body.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Message {
    /// The request line or the status line.
    pub(crate) first: String,
    /// The body, which is text.
    pub(crate) body: String,
}

/// One message off a connection, refused if it is not the shape this wire
/// carries.
///
/// # Errors
///
/// [`NotNearby::NotAMessage`] for anything but a first line, at most
/// [`AT_MOST_HEADERS`] headers, a `content-length` of at most
/// [`AT_MOST_A_BODY`], and that many bytes of text; [`NotNearby::TheNetwork`]
/// if the connection would not read.
pub(crate) fn read_message<R: Read>(from: R) -> Result<Message, NotNearby> {
    let mut lines = BufReader::new(from.take(AT_MOST_A_MESSAGE));
    let first = a_line(&mut lines)?;
    if first.is_empty() {
        return Err(NotNearby::NotAMessage(
            "it began with an empty line".to_owned(),
        ));
    }
    let mut length = None;
    let mut how_many = 0_usize;
    loop {
        let line = a_line(&mut lines)?;
        if line.is_empty() {
            break;
        }
        how_many = how_many.saturating_add(1);
        if how_many > AT_MOST_HEADERS {
            return Err(NotNearby::NotAMessage(
                "more headers than a message here has".to_owned(),
            ));
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(NotNearby::NotAMessage(
                "a header without a colon".to_owned(),
            ));
        };
        let name = name.trim().to_ascii_lowercase();
        if name == TRANSFER_ENCODING {
            return Err(NotNearby::NotAMessage(
                "a body this wire does not carry".to_owned(),
            ));
        }
        if name == CONTENT_LENGTH {
            length = Some(value.trim().parse::<usize>().map_err(|_| {
                NotNearby::NotAMessage("a content-length that is not a number".to_owned())
            })?);
        }
    }
    let Some(length) = length else {
        return Err(NotNearby::NotAMessage("no content-length".to_owned()));
    };
    if length > AT_MOST_A_BODY {
        return Err(NotNearby::NotAMessage(
            "a body longer than any message here".to_owned(),
        ));
    }
    let mut body = vec![0_u8; length];
    lines
        .read_exact(&mut body)
        .map_err(|_| NotNearby::NotAMessage("it ended before its body did".to_owned()))?;
    let body = String::from_utf8(body)
        .map_err(|_| NotNearby::NotAMessage("a body that is not text".to_owned()))?;
    Ok(Message { first, body })
}

/// One line, without its ending.
fn a_line<R: BufRead>(from: &mut R) -> Result<String, NotNearby> {
    let mut line = String::new();
    let read = from.read_line(&mut line).map_err(|why| {
        if why.kind() == std::io::ErrorKind::InvalidData {
            NotNearby::NotAMessage("a line that is not text".to_owned())
        } else {
            NotNearby::TheNetwork(because(&why))
        }
    })?;
    if read == 0 {
        return Err(NotNearby::NotAMessage(
            "it ended before it finished".to_owned(),
        ));
    }
    Ok(line.trim_end_matches(['\r', '\n']).to_owned())
}

/// A request to `path` at `host`, carrying `body`, as the bytes go.
pub(crate) fn a_request(path: &str, host: &str, body: &str) -> String {
    format!(
        "POST {path} HTTP/1.1\r\nhost: {host}\r\ncontent-type: {TEXT}\r\ncontent-length: {}\r\n\
         connection: close\r\n\r\n{body}",
        body.len()
    )
}

/// A reply with `status`, carrying `body`, as the bytes go.
pub(crate) fn a_reply(status: u16, reason: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} {reason}\r\ncontent-type: {TEXT}\r\ncontent-length: {}\r\n\
         connection: close\r\n\r\n{body}",
        body.len()
    )
}

/// The method and the path a request line asks for.
///
/// # Errors
///
/// [`NotNearby::NotAMessage`] for a line that is not `METHOD path HTTP/1.x`.
pub(crate) fn asked_for(first: &str) -> Result<(String, String), NotNearby> {
    let parts: Vec<&str> = first.split(' ').collect();
    let &[method, path, version] = parts.as_slice() else {
        return Err(NotNearby::NotAMessage(
            "a request line that is not three words".to_owned(),
        ));
    };
    if !version.starts_with("HTTP/1.") {
        return Err(NotNearby::NotAMessage(format!(
            "`{version}` is not a version spoken here"
        )));
    }
    Ok((method.to_owned(), path.to_owned()))
}

/// The status a reply's first line carries.
///
/// # Errors
///
/// [`NotNearby::NotAMessage`] for a line that is not `HTTP/1.x status ...`.
pub(crate) fn status_of(first: &str) -> Result<u16, NotNearby> {
    let mut parts = first.split(' ');
    let (Some(version), Some(status)) = (parts.next(), parts.next()) else {
        return Err(NotNearby::NotAMessage(
            "a status line that is not two words or more".to_owned(),
        ));
    };
    if !version.starts_with("HTTP/1.") {
        return Err(NotNearby::NotAMessage(format!(
            "`{version}` is not a version spoken here"
        )));
    }
    status
        .parse()
        .map_err(|_| NotNearby::NotAMessage("a status that is not a number".to_owned()))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::io::Cursor;

    use super::{Message, a_reply, a_request, asked_for, read_message, status_of};
    use crate::refusing::NotNearby;

    /// A request written here is read back as its line and its body.
    #[test]
    fn a_request_written_here_is_read_back() {
        let bytes = a_request(
            "/alo-os/1/pairing/proposal",
            "192.168.1.20:7610",
            "one line\n",
        );
        let message = read_message(Cursor::new(bytes)).unwrap();
        assert_eq!(
            message,
            Message {
                first: "POST /alo-os/1/pairing/proposal HTTP/1.1".to_owned(),
                body: "one line\n".to_owned(),
            }
        );
        assert_eq!(
            asked_for(&message.first).unwrap(),
            ("POST".to_owned(), "/alo-os/1/pairing/proposal".to_owned())
        );
    }

    /// A reply written here is read back, and an empty body is a body.
    #[test]
    fn a_reply_written_here_is_read_back() {
        let message = read_message(Cursor::new(a_reply(204, "No Content", ""))).unwrap();
        assert_eq!(status_of(&message.first).unwrap(), 204);
        assert!(message.body.is_empty());
        let message = read_message(Cursor::new(a_reply(200, "OK", "an offer\n"))).unwrap();
        assert_eq!(status_of(&message.first).unwrap(), 200);
        assert_eq!(message.body, "an offer\n");
    }

    /// Everything that is not the shape this wire carries is refused as not a
    /// message.
    #[test]
    fn what_is_not_a_message_here_is_refused() {
        let too_long = format!(
            "POST / HTTP/1.1\r\ncontent-length: {}\r\n\r\n",
            4 * 1024 + 1
        );
        let too_many_headers = format!(
            "POST / HTTP/1.1\r\n{}content-length: 0\r\n\r\n",
            "x-one: 1\r\n".repeat(33)
        );
        for not_one in [
            String::new(),
            "\r\n".to_owned(),
            "POST / HTTP/1.1\r\n\r\n".to_owned(),
            "POST / HTTP/1.1\r\ncontent-length: soon\r\n\r\n".to_owned(),
            "POST / HTTP/1.1\r\ntransfer-encoding: chunked\r\n\r\n".to_owned(),
            "POST / HTTP/1.1\r\ncontent-length: 10\r\n\r\nshort".to_owned(),
            "POST / HTTP/1.1\r\nno colon here\r\ncontent-length: 0\r\n\r\n".to_owned(),
            "POST / HTTP/1.1\r\ncontent-length: 0".to_owned(),
            too_long,
            too_many_headers,
        ] {
            assert!(
                matches!(
                    read_message(Cursor::new(not_one.clone())).unwrap_err(),
                    NotNearby::NotAMessage(_)
                ),
                "`{not_one}` was read as a message"
            );
        }
        let not_text = b"POST / HTTP/1.1\r\ncontent-length: 2\r\n\r\n\xff\xfe".to_vec();
        assert!(matches!(
            read_message(Cursor::new(not_text)).unwrap_err(),
            NotNearby::NotAMessage(_)
        ));
        let a_line_that_is_not_text = b"POST / \xff\r\ncontent-length: 0\r\n\r\n".to_vec();
        assert!(matches!(
            read_message(Cursor::new(a_line_that_is_not_text)).unwrap_err(),
            NotNearby::NotAMessage(_)
        ));
    }

    /// A request line or a status line that is not one is refused.
    #[test]
    fn a_first_line_that_is_not_one_is_refused() {
        for not_one in ["", "POST /", "POST / SPDY/3", "GET / HTTP/1.1 extra"] {
            assert!(asked_for(not_one).is_err(), "`{not_one}` was read");
        }
        for not_one in ["", "HTTP/1.1", "HTTP/1.1 OK", "SPDY/3 200 OK"] {
            assert!(status_of(not_one).is_err(), "`{not_one}` was read");
        }
        assert_eq!(status_of("HTTP/1.0 404 Not Found").unwrap(), 404);
    }
}
