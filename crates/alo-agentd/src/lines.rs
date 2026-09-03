//! One connection, as a sequence of messages in and answers out.
//!
//! `alo-protocol` settled that a message is one line of bounded length. This is
//! the half of that which touches the socket: finding where a line ends without
//! letting whoever is sending it decide how much of this machine's memory to
//! use, and putting an answer back the same way.
//!
//! # The bound is applied while reading, not after
//!
//! `alo_protocol::frame` refuses a message longer than [`LONGEST`] — but it is
//! handed a line, and by then the line has been held in memory. A client that
//! opens a connection and sends a gigabyte with no line ending in it has taken
//! the person's machine without being granted anything, and it would have done
//! so before any refusal could be reached. So the read stops at the bound
//! itself, and what happens next depends on what was found there:
//!
//! - a line ending inside the bound: an ordinary message, handed on, and the
//!   length rule is applied once more by the crate that owns it;
//! - no line ending inside the bound: [`NotHeard::TooLong`], and the connection
//!   is closed after it is answered — everything read so far is the middle of
//!   something, and there is no way to find the start of the next message.
//!
//! # A refusal here borrows the protocol's words
//!
//! Neither of the two things that can be wrong with a line is a new fact:
//! *longer than this machine will read* and *that message could not be read*
//! are already `alo_protocol::NotUnderstood`, in twenty-four languages, and
//! saying them again here would be `alo-strings`' whole failure mode. So
//! [`NotHeard::what_to_say`] answers with the protocol's own refusal and this
//! file declares no words.
//!
//! # Reading and writing are two handles onto one socket
//!
//! A buffered reader owns what it reads from, and an answer has to go out while
//! one exists. Both handles are the same connection — the kernel's peer
//! credentials, and therefore which door this is, were decided when it was
//! accepted and cannot change.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::os::fd::{AsFd, BorrowedFd};
use std::os::unix::net::UnixStream;

use alo_protocol::{LONGEST, NotUnderstood};

use crate::refusing::NotHeard;

/// One connection, read a line at a time.
#[derive(Debug)]
pub struct Line {
    /// The connection, with somewhere to keep what has arrived and not yet been
    /// read to the end of a line.
    reading: BufReader<UnixStream>,
    /// The same connection, to answer on.
    writing: UnixStream,
}

impl Line {
    /// Take a connection over.
    ///
    /// # Errors
    ///
    /// Whatever the machine said, as a `std::io::Error`, if it would not give a
    /// second handle onto the connection. Nothing is served on a connection
    /// that cannot be answered on: a service that read a request it had no way
    /// to reply to would silently do what it was asked and tell nobody.
    pub fn over(connection: UnixStream) -> Result<Self, std::io::Error> {
        let writing = connection.try_clone()?;
        Ok(Self {
            reading: BufReader::new(connection),
            writing,
        })
    }

    /// What the service waits on while this connection is quiet.
    pub(crate) fn waiting_on(&self) -> BorrowedFd<'_> {
        self.reading.get_ref().as_fd()
    }

    /// The next message, or nothing at all if the caller has gone.
    ///
    /// `Ok(None)` is the ordinary end of a connection: the other side closed
    /// it, which for an agent is the end of its turn.
    ///
    /// Named for what it does rather than `next`, because a connection is not
    /// an iterator and must not be read as one: an iterator that answered
    /// `None` and then something again would be a broken iterator, and that is
    /// exactly what a socket does between two messages.
    ///
    /// # Errors
    ///
    /// [`NotHeard::TooLong`] and [`NotHeard::NotText`] for a line that is not
    /// one this service can act on, both of which are answered in words and
    /// then closed; [`NotHeard::Broken`] for a machine that would not read.
    pub fn heard(&mut self) -> Result<Option<String>, NotHeard> {
        let mut line = String::new();
        let read = self
            .reading
            .by_ref()
            .take(u64::try_from(AS_FAR_AS_IT_READS).unwrap_or(u64::MAX))
            .read_line(&mut line)
            .map_err(NotHeard::from)?;

        if read == 0 {
            return Ok(None);
        }
        if !line.ends_with('\n') && read >= AS_FAR_AS_IT_READS {
            return Err(NotHeard::TooLong { was: read });
        }
        // A line with no ending that stopped short of the bound is a client
        // that closed while it was talking. What it managed to send is handed
        // on: if it was a whole message it is answered, and if it was half of
        // one it is refused in words — and either way the next read is the one
        // that says the caller has gone.
        Ok(Some(line))
    }

    /// Say one thing back.
    ///
    /// The line ending is this method's, because it is part of the envelope
    /// rather than part of the answer: `alo_protocol::ToAnAgent::written` and
    /// its twin hand back one line without one, and every road onto a socket
    /// goes through here.
    ///
    /// # Errors
    ///
    /// [`NotHeard::Broken`] — a caller that has gone, most often, which is not
    /// a fault of this machine's and is the reason it is not reported as one.
    pub fn say(&mut self, line: &str) -> Result<(), NotHeard> {
        self.writing
            .write_all(line.as_bytes())
            .and_then(|()| self.writing.write_all(b"\n"))
            .and_then(|()| self.writing.flush())
            .map_err(NotHeard::from)
    }
}

/// How far a read goes before it gives up on finding a line ending.
///
/// One byte past [`LONGEST`], so that a line right at the bound is still read
/// and can be **refused in words** by the crate that owns the bound — rather
/// than closed on here, which is what a client whose message is one byte too
/// long would otherwise get instead of a sentence.
const AS_FAR_AS_IT_READS: usize = LONGEST + 1;

impl From<std::io::Error> for NotHeard {
    /// A machine that would not read, and the one thing that is not that.
    ///
    /// `InvalidData` from a line reader means the bytes were not text, which is
    /// something the caller did rather than something the machine did — and it
    /// is answered in words rather than treated as a broken socket.
    fn from(why: std::io::Error) -> Self {
        if why.kind() == std::io::ErrorKind::InvalidData {
            Self::NotText
        } else {
            Self::Broken(why)
        }
    }
}

impl NotHeard {
    /// What to say to whoever sent it, when there is anything to say.
    ///
    /// `None` for a connection the machine could not read at all: there is
    /// nobody left to say it to, and inventing a sentence for a socket that has
    /// gone would be a string nothing ever shows anybody.
    #[must_use]
    pub fn what_to_say(&self) -> Option<NotUnderstood> {
        match self {
            Self::TooLong { was } => Some(NotUnderstood::TooLong {
                most: LONGEST,
                was: *was,
            }),
            Self::NotText => Some(NotUnderstood::NotReadable),
            Self::Broken(_) => None,
        }
    }

    /// Whether this connection can go on being served.
    ///
    /// Never: all three leave a connection whose next byte is the middle of
    /// something, so each of them is answered once and then closed. It is a
    /// method rather than a comment because the service asks it, and because
    /// the day one of these becomes recoverable this is where that is decided.
    #[must_use]
    pub const fn is_the_end_of_the_connection(&self) -> bool {
        true
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A connection and the other end of it, to write onto and read back from.
    fn talking() -> (Line, UnixStream) {
        let (ours, theirs) = UnixStream::pair().unwrap();
        (Line::over(ours).unwrap(), theirs)
    }

    /// **A message is one line**, and the line ending is not part of it: what
    /// comes back is what the client wrote, ready for the crate that reads it.
    #[test]
    fn one_line_is_one_message() {
        let (mut line, mut client) = talking();
        client.write_all(b"{\"format\":1}\n").unwrap();

        assert_eq!(line.heard().unwrap().as_deref(), Some("{\"format\":1}\n"));
    }

    /// **Two messages arrive as two**, so nothing sent while the service was
    /// busy is lost: the buffered reader holds what was written past the first
    /// line ending.
    #[test]
    fn two_messages_sent_at_once_are_read_one_at_a_time() {
        let (mut line, mut client) = talking();
        client.write_all(b"one\ntwo\n").unwrap();

        assert_eq!(line.heard().unwrap().as_deref(), Some("one\n"));
        assert_eq!(line.heard().unwrap().as_deref(), Some("two\n"));
    }

    /// **A caller that goes away answers with nothing at all**, which is the
    /// ordinary end of a connection rather than a failure of one.
    #[test]
    fn a_caller_that_closed_the_connection_says_nothing() {
        let (mut line, client) = talking();
        drop(client);

        assert_eq!(line.heard().unwrap(), None);
    }

    /// **A message with no end to it is refused at the bound**, rather than
    /// after this machine has held all of it. The refusal carries how much was
    /// read and borrows the protocol's own sentence.
    #[test]
    fn a_line_that_never_ends_is_refused_before_it_is_all_held() {
        let (mut line, mut client) = talking();
        std::thread::spawn(move || {
            let flood = vec![b'a'; 64 * 1024];
            // More than the bound, and not one line ending anywhere in it. The
            // write stops early once the reader has given up, which is the
            // point: nothing here waits for a client to finish.
            while client.write_all(&flood).is_ok() {}
        });

        let refused = line.heard().unwrap_err();
        assert!(matches!(refused, NotHeard::TooLong { .. }));
        assert!(refused.is_the_end_of_the_connection());
        assert!(matches!(
            refused.what_to_say(),
            Some(NotUnderstood::TooLong { most: LONGEST, .. })
        ));
    }

    /// **Bytes that are not text are the caller's doing and are answered in
    /// words**, with the sentence `alo-protocol` already has for a message it
    /// could not read.
    #[test]
    fn bytes_that_are_not_text_are_answered_rather_than_treated_as_a_fault() {
        let (mut line, mut client) = talking();
        client.write_all(&[0xff, 0xfe, b'\n']).unwrap();

        let refused = line.heard().unwrap_err();
        assert!(matches!(refused, NotHeard::NotText));
        assert_eq!(refused.what_to_say(), Some(NotUnderstood::NotReadable));
    }

    /// **An answer goes back as one line**, with the ending this file adds:
    /// every road onto a socket goes through one place, so a client reading
    /// lines cannot be handed two answers joined together.
    #[test]
    fn an_answer_goes_back_as_one_line() {
        let (mut line, client) = talking();
        line.say("{\"format\":1}").unwrap();

        let mut back = String::new();
        BufReader::new(client).read_line(&mut back).unwrap();
        assert_eq!(back, "{\"format\":1}\n");
    }

    /// **A caller that has gone is not a machine that broke**, and nothing to
    /// say to it: the refusal carries no sentence, because there is nobody to
    /// read one.
    #[test]
    fn answering_a_caller_that_has_gone_says_nothing_to_anybody() {
        let (mut line, client) = talking();
        drop(client);

        // The first write may be taken into a buffer nobody will read; what
        // matters is that the failure, when it comes, has no sentence in it.
        let mut refused = None;
        for _ in 0..8 {
            if let Err(why) = line.say("{\"format\":1}") {
                refused = Some(why);
                break;
            }
        }
        let refused = refused.unwrap();
        assert!(matches!(refused, NotHeard::Broken(_)));
        assert_eq!(refused.what_to_say(), None);
    }

    /// **A line right at the bound is still read**, so that the crate that owns
    /// the bound is the one that refuses it — in words, on a connection that
    /// goes on being served. Closing on it here would answer a message one byte
    /// too long with silence.
    #[test]
    fn a_line_at_the_bound_is_read_so_it_can_be_refused_in_words() {
        let (mut line, mut client) = talking();
        let longest = "x".repeat(LONGEST);
        std::thread::spawn(move || {
            drop(client.write_all(longest.as_bytes()));
            drop(client.write_all(b"\n"));
        });

        let read = line.heard().unwrap().unwrap();
        assert_eq!(read.len(), LONGEST + 1, "the line ending is still on it");
    }
}
