//! One request to this machine's printing service, and its answer, as HTTP/1.1
//! carries them.
//!
//! The service reads IPP out of the body of a `POST`, so this file is exactly
//! that much of HTTP and no more: one request per connection, a length on the
//! way out, and on the way back either a length or the chunks the service
//! sends a long answer in. It is not a client for anything else and would be a
//! poor one — there are no redirects, no cookies, no keep-alive and no
//! password authentication. Only the broker socket asks for PeerCred root;
//! the service decides who may
//! do what from the socket it was reached on.
//!
//! **An answer is bounded.** A list of devices is the largest thing this crate
//! asks for and is a few kilobytes; a service answering with more than
//! [`MOST`] is refused rather than read into memory until something runs out.

use std::io::{self, Read, Write};

/// The most bytes an answer's head may be.
const MOST_HEAD: usize = 16 * 1024;

/// The most bytes an answer's body may be.
pub const MOST: usize = 1024 * 1024;

/// Why an exchange did not produce an answer.
#[derive(Debug)]
pub enum Failed {
    /// Writing or reading failed part way — the service went away, or did not
    /// answer in time.
    Io,
    /// What came back was not an HTTP answer this file can read.
    NotUnderstood,
    /// The service answered, and would not let this machine do it.
    NotPermitted,
    /// The service answered with a status that is not success.
    Status,
    /// The answer was larger than any this crate asks for.
    TooLarge,
}

impl From<io::Error> for Failed {
    fn from(_: io::Error) -> Self {
        Self::Io
    }
}

/// Send one request whose body is `head` followed by `length` bytes of
/// `document`, and read the answer's body.
///
/// # Errors
/// [`Failed`].
pub fn exchange<S: Read + Write, D: Read>(
    stream: &mut S,
    path: &str,
    head: &[u8],
    document: &mut D,
    length: u64,
    as_root: bool,
) -> Result<Vec<u8>, Failed> {
    if !path.starts_with('/')
        || !path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.'))
    {
        return Err(Failed::NotUnderstood);
    }
    let total = u64::try_from(head.len())
        .ok()
        .and_then(|head| head.checked_add(length))
        .ok_or(Failed::TooLarge)?;
    let authentication = if as_root {
        "Authorization: PeerCred root\r\n"
    } else {
        ""
    };
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/ipp\r\n\
         Content-Length: {total}\r\n{authentication}Connection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes())?;
    stream.write_all(head)?;
    let copied = io::copy(&mut document.take(length), stream)?;
    if copied != length {
        return Err(Failed::Io);
    }
    stream.flush()?;
    read_answer(stream)
}

/// The body of the answer on this stream.
fn read_answer<S: Read>(stream: &mut S) -> Result<Vec<u8>, Failed> {
    let mut bytes = Vec::with_capacity(4096);
    let mut chunk = [0_u8; 4096];
    let end_of_head = loop {
        if let Some(at) = find(&bytes, b"\r\n\r\n") {
            break at;
        }
        if bytes.len() > MOST_HEAD {
            return Err(Failed::NotUnderstood);
        }
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(Failed::NotUnderstood);
        }
        bytes.extend(chunk.get(..read).unwrap_or_default());
    };
    let head = std::str::from_utf8(bytes.get(..end_of_head).unwrap_or_default())
        .map_err(|_| Failed::NotUnderstood)?;
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .and_then(|line| {
            let mut words = line.split(' ');
            let version = words.next()?;
            version.starts_with("HTTP/1.").then_some(())?;
            words.next()?.parse::<u16>().ok()
        })
        .ok_or(Failed::NotUnderstood)?;
    let mut length = None;
    let mut chunked = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            return Err(Failed::NotUnderstood);
        };
        let value = value.trim();
        if name.eq_ignore_ascii_case("content-length") {
            length = Some(value.parse::<usize>().map_err(|_| Failed::NotUnderstood)?);
        } else if name.eq_ignore_ascii_case("transfer-encoding") {
            chunked = value.eq_ignore_ascii_case("chunked");
        }
    }
    match status {
        200 => {}
        401 | 403 => return Err(Failed::NotPermitted),
        _ => return Err(Failed::Status),
    }
    let mut body: Vec<u8> = bytes
        .get(end_of_head.saturating_add(4)..)
        .unwrap_or_default()
        .to_vec();
    if chunked {
        read_until_chunks_end(stream, &mut body)?;
        return unchunked(&body);
    }
    match length {
        Some(length) if length > MOST => Err(Failed::TooLarge),
        Some(length) => {
            while body.len() < length {
                let read = stream.read(&mut chunk)?;
                if read == 0 {
                    return Err(Failed::NotUnderstood);
                }
                body.extend(chunk.get(..read).unwrap_or_default());
            }
            body.truncate(length);
            Ok(body)
        }
        None => {
            stream
                .take(u64::try_from(MOST).unwrap_or(u64::MAX).saturating_add(1))
                .read_to_end(&mut body)?;
            if body.len() > MOST {
                return Err(Failed::TooLarge);
            }
            Ok(body)
        }
    }
}

/// Read until the last chunk, or the connection closes.
fn read_until_chunks_end<S: Read>(stream: &mut S, body: &mut Vec<u8>) -> Result<(), Failed> {
    let mut chunk = [0_u8; 4096];
    while unchunked(body).is_err() {
        if body.len() > MOST.saturating_mul(2) {
            return Err(Failed::TooLarge);
        }
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(Failed::NotUnderstood);
        }
        body.extend(chunk.get(..read).unwrap_or_default());
    }
    Ok(())
}

/// A chunked body, joined — or refused if it has not reached its last chunk.
fn unchunked(body: &[u8]) -> Result<Vec<u8>, Failed> {
    let mut joined = Vec::new();
    let mut at = 0_usize;
    loop {
        let rest = body.get(at..).ok_or(Failed::NotUnderstood)?;
        let line_end = find(rest, b"\r\n").ok_or(Failed::NotUnderstood)?;
        let size_line = std::str::from_utf8(rest.get(..line_end).unwrap_or_default())
            .map_err(|_| Failed::NotUnderstood)?;
        let size_text = size_line.split(';').next().unwrap_or_default().trim();
        let size = usize::from_str_radix(size_text, 16).map_err(|_| Failed::NotUnderstood)?;
        let start = at.saturating_add(line_end).saturating_add(2);
        if size == 0 {
            return Ok(joined);
        }
        let end = start.checked_add(size).ok_or(Failed::TooLarge)?;
        joined.extend(body.get(start..end).ok_or(Failed::NotUnderstood)?);
        if joined.len() > MOST {
            return Err(Failed::TooLarge);
        }
        at = end.saturating_add(2);
    }
}

/// Where `needle` first begins in `haystack`.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// A stream whose reads come from one buffer and whose writes go to
    /// another.
    struct Wire {
        /// What the service answers.
        answer: Cursor<Vec<u8>>,
        /// What was sent.
        sent: Vec<u8>,
    }

    impl Read for Wire {
        fn read(&mut self, into: &mut [u8]) -> io::Result<usize> {
            self.answer.read(into)
        }
    }

    impl Write for Wire {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.sent.extend(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    /// One exchange against a canned answer.
    fn against(answer: &[u8]) -> (Result<Vec<u8>, Failed>, Vec<u8>) {
        let mut wire = Wire {
            answer: Cursor::new(answer.to_vec()),
            sent: Vec::new(),
        };
        let got = exchange(
            &mut wire,
            "/admin/",
            b"head",
            &mut Cursor::new(b"document"),
            8,
            false,
        );
        (got, wire.sent)
    }

    /// The request says how long it is and carries the head and the document.
    #[test]
    fn a_request_says_its_length_and_carries_both_halves() {
        let (got, sent) = against(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc");
        assert_eq!(got.unwrap(), b"abc");
        let sent = String::from_utf8(sent).unwrap();
        assert!(sent.starts_with("POST /admin/ HTTP/1.1\r\n"), "{sent}");
        assert!(sent.contains("Content-Length: 12\r\n"), "{sent}");
        assert!(sent.ends_with("\r\n\r\nheaddocument"), "{sent}");
    }

    /// A chunked answer is joined, and one that stops before its last chunk is
    /// refused rather than read as what arrived.
    #[test]
    fn a_chunked_answer_is_joined_and_one_cut_short_is_refused() {
        let (got, _) = against(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n2;x\r\nde\r\n0\r\n\r\n",
        );
        assert_eq!(got.unwrap(), b"abcde");
        let (got, _) =
            against(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n");
        assert!(matches!(got, Err(Failed::NotUnderstood)), "{got:?}");
    }

    /// The service saying no is told apart from the service failing.
    #[test]
    fn not_permitted_is_told_apart_from_a_failure() {
        let (got, _) = against(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n");
        assert!(matches!(got, Err(Failed::NotPermitted)), "{got:?}");
        let (got, _) = against(b"HTTP/1.1 500 Internal\r\nContent-Length: 0\r\n\r\n");
        assert!(matches!(got, Err(Failed::Status)), "{got:?}");
        let (got, _) = against(b"not http at all\r\n\r\n");
        assert!(matches!(got, Err(Failed::NotUnderstood)), "{got:?}");
    }

    /// An answer larger than anything asked for is refused before it is read.
    #[test]
    fn an_answer_larger_than_anything_asked_for_is_refused() {
        let (got, _) = against(b"HTTP/1.1 200 OK\r\nContent-Length: 99999999\r\n\r\n");
        assert!(matches!(got, Err(Failed::TooLarge)), "{got:?}");
    }

    /// A path with anything in it but a path's characters is never sent.
    #[test]
    fn a_path_that_is_not_a_plain_path_is_never_sent() {
        let mut wire = Wire {
            answer: Cursor::new(Vec::new()),
            sent: Vec::new(),
        };
        let got = exchange(
            &mut wire,
            "/a b\r\nHost: x",
            b"",
            &mut io::empty(),
            0,
            false,
        );
        assert!(matches!(got, Err(Failed::NotUnderstood)), "{got:?}");
        assert!(wire.sent.is_empty());
    }
}
