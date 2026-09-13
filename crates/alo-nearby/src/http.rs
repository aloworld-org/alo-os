//! The framing both ends of a wire between two machines share: the least of
//! HTTP/1.1 that carries one message there and one message back.
//!
//! # Why HTTP, and why this little of it
//!
//! The port a machine advertises is the one the corridor already puts
//! questions to over HTTP, and the verb wire (`alo-corridor`) carries its
//! proof in a header. One port, one shape: a daemon that answers on it can
//! tell a proposal from a verb from a question by the path, rather than by
//! guessing which protocol a stranger's first bytes are in.
//!
//! But only this much of it. A request is a `POST` with a `content-length`
//! and `connection: close`; a reply is a status line with the same two. No
//! chunked bodies, no keep-alive, no pipelining, a stated bound on the body,
//! nothing that is not text. What is not read cannot be got wrong, and
//! everything on a network can reach this port.
//!
//! Hand-written rather than rented for the reason the DNS packets in
//! `advertising.rs` are: what this crate reads off a port is small and
//! closed, and a dependency that read more would be more to be wrong in.
//!
//! # Public, and for whom
//!
//! The pairing wire in this crate and the verb wire in `alo-corridor` are one
//! port and one shape, so they are one framing. That crate reads and writes
//! through this module rather than carrying a second copy of it, which is the
//! whole of why the module is public: a second parser of what a stranger sends
//! would be a second place to be wrong in. The headers are kept because the
//! verb wire's proof travels in one; the pairing wire reads none.

use std::io::{BufRead, BufReader, Read};
use std::time::Duration;

use crate::refusing::{NotNearby, because};

/// How long either end waits for the other on one connection.
///
/// Ten seconds. The other machine is in the building, and what it has to do
/// before answering is show a value to a surface or walk a verb through its
/// grants, not think.
pub const WHILE_THE_WIRE_ANSWERS: Duration = Duration::from_secs(10);

/// The most bytes the head of a message — its first line and its headers —
/// is read from.
const AT_MOST_A_HEAD: u64 = 8 * 1024;

/// The most bytes a body on the pairing wire may say it is.
///
/// Four kibibytes: a proposal and a confirmation are each one short line, and
/// the verb wire, which carries more, says how much through
/// [`read_message_of_at_most`].
pub const AT_MOST_A_BODY: usize = 4 * 1024;

/// The most headers a message here has.
const AT_MOST_HEADERS: usize = 32;

/// The one header that says how long the body is.
const CONTENT_LENGTH: &str = "content-length";

/// The header that would mean a body this wire does not carry.
const TRANSFER_ENCODING: &str = "transfer-encoding";

/// What every message here says its body is.
const TEXT: &str = "text/plain; charset=utf-8";

/// One message, read: its first line, its headers and its body.
#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    /// The request line or the status line.
    pub first: String,
    /// Every header, name lowercased and value trimmed, in the order sent.
    pub headers: Vec<(String, String)>,
    /// The body, which is text.
    pub body: String,
}

impl Message {
    /// The value of the first header called `name`, if one was sent.
    ///
    /// Names are matched lowercased, because HTTP does; values are as they
    /// were sent, trimmed.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(sent, _)| *sent == name)
            .map(|(_, value)| value.as_str())
    }
}

/// One message off a connection, refused if it is not the shape the pairing
/// wire carries.
///
/// # Errors
///
/// [`NotNearby::NotAMessage`] for anything but a first line, at most
/// thirty-two headers, a `content-length` of at most [`AT_MOST_A_BODY`], and
/// that many bytes of text; [`NotNearby::TheNetwork`] if the connection would
/// not read.
pub fn read_message<R: Read>(from: R) -> Result<Message, NotNearby> {
    read_message_of_at_most(from, AT_MOST_A_BODY)
}

/// One message off a connection, with a body of at most `most_body` bytes.
///
/// The same reading as [`read_message`] with the bound named by the caller,
/// for the verb wire, whose answers carry a folder's listing rather than one
/// line. The bound is on the body a message *says* it has and on what is read
/// for it, so a stranger who says four kibibytes and sends a gigabyte is read
/// for four kibibytes and refused.
///
/// # Errors
///
/// As [`read_message`].
pub fn read_message_of_at_most<R: Read>(from: R, most_body: usize) -> Result<Message, NotNearby> {
    let at_most = AT_MOST_A_HEAD.saturating_add(u64::try_from(most_body).unwrap_or(u64::MAX));
    let mut lines = BufReader::new(from.take(at_most));
    let first = a_line(&mut lines)?;
    if first.is_empty() {
        return Err(NotNearby::NotAMessage(
            "it began with an empty line".to_owned(),
        ));
    }
    let mut length = None;
    let mut headers = Vec::new();
    loop {
        let line = a_line(&mut lines)?;
        if line.is_empty() {
            break;
        }
        if headers.len() >= AT_MOST_HEADERS {
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
        let value = value.trim().to_owned();
        if name == TRANSFER_ENCODING {
            return Err(NotNearby::NotAMessage(
                "a body this wire does not carry".to_owned(),
            ));
        }
        if name == CONTENT_LENGTH {
            length = Some(value.parse::<usize>().map_err(|_| {
                NotNearby::NotAMessage("a content-length that is not a number".to_owned())
            })?);
        }
        headers.push((name, value));
    }
    let Some(length) = length else {
        return Err(NotNearby::NotAMessage("no content-length".to_owned()));
    };
    if length > most_body {
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
    Ok(Message {
        first,
        headers,
        body,
    })
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
#[must_use]
pub fn a_request(path: &str, host: &str, body: &str) -> String {
    a_request_carrying(path, host, &[], body)
}

/// A request to `path` at `host`, carrying `body` and these headers besides
/// the four every request here has, as the bytes go.
///
/// For the verb wire, whose proof travels in a header. A header's value is
/// one line: whatever the caller hands over is written as it is, and the one
/// caller writes a proof, which is hexadecimal and decimal and nothing else.
#[must_use]
pub fn a_request_carrying(path: &str, host: &str, headers: &[(&str, &str)], body: &str) -> String {
    let carried: String = headers
        .iter()
        .map(|(name, value)| format!("{name}: {value}\r\n"))
        .collect();
    format!(
        "POST {path} HTTP/1.1\r\nhost: {host}\r\ncontent-type: {TEXT}\r\ncontent-length: {}\r\n\
         connection: close\r\n{carried}\r\n{body}",
        body.len()
    )
}

/// A reply with `status`, carrying `body`, as the bytes go.
#[must_use]
pub fn a_reply(status: u16, reason: &str, body: &str) -> String {
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
pub fn asked_for(first: &str) -> Result<(String, String), NotNearby> {
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
pub fn status_of(first: &str) -> Result<u16, NotNearby> {
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

    use super::{
        AT_MOST_A_BODY, Message, a_reply, a_request, a_request_carrying, asked_for, read_message,
        read_message_of_at_most, status_of,
    };
    use crate::refusing::NotNearby;

    /// A request written here is read back as its line, its headers and its
    /// body.
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
                headers: vec![
                    ("host".to_owned(), "192.168.1.20:7610".to_owned()),
                    (
                        "content-type".to_owned(),
                        "text/plain; charset=utf-8".to_owned()
                    ),
                    ("content-length".to_owned(), "9".to_owned()),
                    ("connection".to_owned(), "close".to_owned()),
                ],
                body: "one line\n".to_owned(),
            }
        );
        assert_eq!(
            asked_for(&message.first).unwrap(),
            ("POST".to_owned(), "/alo-os/1/pairing/proposal".to_owned())
        );
        assert_eq!(message.header("Host"), Some("192.168.1.20:7610"));
        assert_eq!(message.header("alo-pairing"), None);
    }

    /// A header the caller carries is read back by name, whatever case it
    /// was sent in, and a request carrying none is the plain request.
    #[test]
    fn a_header_carried_is_read_back_by_name() {
        let bytes = a_request_carrying(
            "/alo-os/1/verb/read",
            "192.168.1.20:7610",
            &[("Alo-Pairing", "alo-os/1 a b 1 2 ff")],
            "{}",
        );
        let message = read_message(Cursor::new(bytes)).unwrap();
        assert_eq!(message.header("alo-pairing"), Some("alo-os/1 a b 1 2 ff"));
        assert_eq!(message.body, "{}");
        assert_eq!(
            a_request_carrying("/p", "h", &[], "b"),
            a_request("/p", "h", "b")
        );
    }

    /// The body bound is the caller's on the verb wire and four kibibytes on
    /// the pairing wire, and either way a body that says more is refused
    /// before it is read.
    #[test]
    fn the_body_bound_is_the_callers_and_is_enforced_before_the_body_is_read() {
        let five = "x".repeat(5 * 1024);
        let bytes = a_request("/p", "h", &five);
        assert!(matches!(
            read_message(Cursor::new(bytes.clone())).unwrap_err(),
            NotNearby::NotAMessage(_)
        ));
        assert_eq!(
            read_message_of_at_most(Cursor::new(bytes), 8 * 1024)
                .unwrap()
                .body,
            five
        );
        let says_more_than_it_sends = format!(
            "POST / HTTP/1.1\r\ncontent-length: {}\r\n\r\nshort",
            AT_MOST_A_BODY + 1
        );
        assert!(matches!(
            read_message(Cursor::new(says_more_than_it_sends)).unwrap_err(),
            NotNearby::NotAMessage(_)
        ));
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
