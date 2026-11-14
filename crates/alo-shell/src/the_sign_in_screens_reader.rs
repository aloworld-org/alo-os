//! **The sign-in screen's side of the door to the reader that speaks it**
//! ([ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md) §2).
//!
//! The compositor runs as root with no capabilities and never reaches the
//! sign-in login's buses. It binds one socket, in a folder only it and the
//! sign-in login can enter, and the reader — `alo-reading-the-sign-in-screen`,
//! running as that login — connects to it. **The kernel says who connected**
//! (`SO_PEERCRED`), and anybody but the sign-in login is shut out before a
//! byte is sent.
//!
//! What this does with a reader that is in:
//!
//! - **it sends the screen whenever it changes**, whole, as
//!   `crate::the_screens_before_sign_in_as_read` built it — asked for only
//!   while a reader is connected, so a machine nobody reads costs nothing;
//! - **it sends whether to read aloud, and in which language**: when the person
//!   turns the reader over (`Ctrl`+`Alt`+`S` or *Accessibility*) and when the
//!   language they chose changes;
//! - **it hears what starting or stopping came to**, for the screen to say;
//! - **and when somebody signs in it asks the reader to step aside** and waits
//!   a short while for it to say it has, so the person's own reader does not
//!   start over the sign-in screen's.
//!
//! # Never in the way of the screen
//!
//! Nothing here blocks a frame. The socket is non-blocking both ways, what
//! cannot be written yet waits for the next frame, and a reader that goes away
//! is simply not there: the screen goes on, and a reader that connects again is
//! sent the whole screen and what the person last chose. **A person who turns
//! the reader on when no reader can be reached is told so** —
//! [`TheScreenReaderAnswered::DidNotStart`] — and the choice stays as they made
//! it (law 5), so a reader that arrives later is told to read.

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::{Duration, Instant};

use crate::the_sign_in_screen_on_the_wire::{FromTheReader, THE_LONGEST_LINE, ToTheReader};
use crate::{ReadAloudTree, TheScreenReaderAnswered};

/// Where the door is on this machine: a folder `0750 root:alo-greeter`, made
/// at every start by `image/usr/lib/tmpfiles.d/alo.conf`.
pub const THE_READING_DOOR: &str = "/run/alo-sign-in-read-aloud/reading";

/// **The sign-in login, by number**: `alo-greeter`, the only uid let through
/// the door (`image/usr/lib/sysusers.d/alo.conf`, which the image's checks
/// hold to this number).
pub const THE_SIGN_IN_LOGIN: u32 = 60990;

/// How long a person signing in waits, at most, for the sign-in screen's
/// reader to stop — after which the session opens whether it has or not.
pub const PATIENCE_AT_SIGN_IN: Duration = Duration::from_secs(2);

/// **The door, and the reader on the other side of it, if one is.**
#[derive(Debug)]
pub struct TheSignInScreensReader {
    /// The socket readers connect to, where it could be bound.
    listening: Option<UnixListener>,
    /// The reader that is in, if one is.
    reader: Option<AReader>,
    /// The only uid let in.
    the_readers_uid: u32,
    /// The screen last sent to the reader that is in.
    sent: Option<ReadAloudTree>,
    /// Whether the person asked for the screen to be read aloud.
    reading: bool,
    /// The language the screen speaks now, as a tag.
    language: Option<String>,
    /// What the reader answered, oldest first, not yet asked for.
    answers: VecDeque<TheScreenReaderAnswered>,
}

/// A reader that is in: its connection, what it said that is not yet a whole
/// line, and what is waiting to be written to it.
#[derive(Debug)]
struct AReader {
    /// The connection.
    stream: UnixStream,
    /// What it said that is not yet a whole line.
    heard: Vec<u8>,
    /// What is waiting to be written.
    waiting: Vec<u8>,
    /// Whether it has said it stepped aside.
    stepped_aside: bool,
}

impl TheSignInScreensReader {
    /// A door at `at`, letting in `the_readers_uid` and nobody else.
    ///
    /// **A door that cannot be opened is not a screen that cannot be drawn**:
    /// the reason is said on the journal and the screen goes on without a
    /// reader, which is what it did before there was one.
    #[must_use]
    pub fn at(at: &Path, the_readers_uid: u32) -> Self {
        // A socket left by an earlier screen is in the way of this one's.
        match std::fs::remove_file(at) {
            Ok(()) => {}
            Err(why) if why.kind() == io::ErrorKind::NotFound => {}
            Err(why) => eprintln!(
                "alo-compositor: the old reading door at {} could not be cleared: {why}",
                at.display()
            ),
        }
        let listening = UnixListener::bind(at)
            .and_then(|listening| listening.set_nonblocking(true).map(|()| listening));
        let listening = match listening {
            Ok(listening) => Some(listening),
            Err(why) => {
                eprintln!(
                    "alo-compositor: no screen reader can reach the sign-in screen, because its \
                     door at {} would not open: {why}",
                    at.display()
                );
                None
            }
        };
        Self {
            listening,
            reader: None,
            the_readers_uid,
            sent: None,
            reading: false,
            language: None,
            answers: VecDeque::new(),
        }
    }

    /// The same door, with the screen already to be read aloud — what a
    /// machine whose sign-in screen was left reading starts as
    /// (`crate::the_reader_at_the_sign_in_screen_kept`). Nothing is said yet:
    /// no reader has had time to connect, and the first that does is told.
    #[must_use]
    pub fn reading_from_the_start(mut self, reading: bool) -> Self {
        self.reading = reading;
        self
    }

    /// **Once a frame**: let a reader in if one is knocking, hear what it said,
    /// and send it the screen if the screen changed. `screen` is asked only
    /// while a reader is in.
    pub fn following(&mut self, screen: impl FnOnce() -> ReadAloudTree) {
        self.let_in();
        self.hear();
        if self.reader.is_none() {
            return;
        }
        let now = screen();
        if self.sent.as_ref() != Some(&now) {
            self.send(&ToTheReader::TheScreen(now.clone()));
            self.sent = Some(now);
        }
        self.flush();
    }

    /// **The person turned the reader over**, by `Ctrl`+`Alt`+`S` or the
    /// *Accessibility* button, reading in `language`.
    pub fn turned_over(&mut self, language: Option<String>) {
        self.reading = !self.reading;
        self.language = language;
        if self.reader.is_none() {
            if self.reading {
                eprintln!(
                    "alo-compositor: the screen reader was turned on at the sign-in screen and no \
                     reader is connected to it, so nothing reads the screen aloud; it will be \
                     told to as soon as one is"
                );
                self.answers.push_back(TheScreenReaderAnswered::DidNotStart);
            } else {
                self.answers.push_back(TheScreenReaderAnswered::Stopped);
            }
            return;
        }
        self.send(&ToTheReader::Read {
            on: self.reading,
            language: self.language.clone(),
        });
        self.flush();
    }

    /// **The language the screen speaks is now `language`** — told to a
    /// reader that is reading, so it speaks it from the press that chose it.
    pub fn the_language_is(&mut self, language: Option<String>) {
        if self.language == language {
            return;
        }
        self.language = language;
        if self.reading && self.reader.is_some() {
            self.send(&ToTheReader::Read {
                on: true,
                language: self.language.clone(),
            });
            self.flush();
        }
    }

    /// Whether the person asked for the screen to be read aloud.
    #[must_use]
    pub const fn is_reading(&self) -> bool {
        self.reading
    }

    /// The next thing the reader answered, oldest first.
    pub fn answered(&mut self) -> Option<TheScreenReaderAnswered> {
        self.answers.pop_front()
    }

    /// **Somebody is signing in**: ask the reader to step aside, and wait at
    /// most `patience` for it to say it has. Answers whether it did; a reader
    /// that is not there has nothing to step aside from, and answers yes.
    pub fn steps_aside(&mut self, patience: Duration) -> bool {
        if self.reader.is_none() {
            return true;
        }
        self.send(&ToTheReader::SomebodyIsSigningIn);
        let until = Instant::now() + patience;
        loop {
            self.flush();
            self.hear();
            match self.reader.as_ref() {
                None => return true,
                Some(reader) if reader.stepped_aside => return true,
                Some(_) => {}
            }
            if Instant::now() >= until {
                eprintln!(
                    "alo-compositor: the sign-in screen's reader did not say it had stopped within \
                     {patience:?}, so the session opens anyway"
                );
                return false;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Let a knocking reader in, if it is the sign-in login.
    fn let_in(&mut self) {
        if self.reader.is_some() {
            return;
        }
        let Some(listening) = self.listening.as_ref() else {
            return;
        };
        let stream = match listening.accept() {
            Ok((stream, _)) => stream,
            Err(why) if why.kind() == io::ErrorKind::WouldBlock => return,
            Err(why) => {
                eprintln!("alo-compositor: a screen reader knocked and could not be let in: {why}");
                return;
            }
        };
        let who = rustix::net::sockopt::socket_peercred(&stream).map(|peer| peer.uid.as_raw());
        match who {
            Ok(uid) if uid == self.the_readers_uid => {}
            Ok(uid) => {
                eprintln!(
                    "alo-compositor: uid {uid} knocked at the sign-in screen's reading door, and \
                     only the sign-in login ({}) is let in",
                    self.the_readers_uid
                );
                return;
            }
            Err(why) => {
                eprintln!(
                    "alo-compositor: the kernel would not say who knocked at the reading door, so \
                     nobody was let in: {why}"
                );
                return;
            }
        }
        if let Err(why) = stream.set_nonblocking(true) {
            eprintln!("alo-compositor: the screen reader's connection could not be set up: {why}");
            return;
        }
        self.reader = Some(AReader {
            stream,
            heard: Vec::new(),
            waiting: Vec::new(),
            stepped_aside: false,
        });
        // **A reader that arrives is told everything**: the whole screen on the
        // next frame, and what the person last asked for.
        self.sent = None;
        if self.reading {
            self.send(&ToTheReader::Read {
                on: true,
                language: self.language.clone(),
            });
        }
    }

    /// Queue `message` for the reader that is in.
    fn send(&mut self, message: &ToTheReader) {
        if let Some(reader) = self.reader.as_mut() {
            reader
                .waiting
                .extend_from_slice(message.written().as_bytes());
        }
    }

    /// Write what is waiting, as much as the socket takes now.
    fn flush(&mut self) {
        let Some(reader) = self.reader.as_mut() else {
            return;
        };
        while !reader.waiting.is_empty() {
            match reader.stream.write(&reader.waiting) {
                Ok(0) => {
                    self.gone("its connection took nothing");
                    return;
                }
                Ok(written) => {
                    reader.waiting.drain(..written);
                }
                Err(why) if why.kind() == io::ErrorKind::WouldBlock => return,
                Err(why) if why.kind() == io::ErrorKind::Interrupted => {}
                Err(why) => {
                    self.gone(&why.to_string());
                    return;
                }
            }
        }
    }

    /// Hear what the reader said, a whole line at a time.
    fn hear(&mut self) {
        let mut lines = Vec::new();
        let mut went = None;
        if let Some(reader) = self.reader.as_mut() {
            let mut buffer = [0_u8; 4096];
            loop {
                match reader.stream.read(&mut buffer) {
                    Ok(0) => {
                        went = Some("it closed its connection".to_owned());
                        break;
                    }
                    Ok(read) => reader
                        .heard
                        .extend_from_slice(buffer.get(..read).unwrap_or(&[])),
                    Err(why) if why.kind() == io::ErrorKind::WouldBlock => break,
                    Err(why) if why.kind() == io::ErrorKind::Interrupted => {}
                    Err(why) => {
                        went = Some(why.to_string());
                        break;
                    }
                }
            }
            while let Some(end) = reader.heard.iter().position(|byte| *byte == b'\n') {
                let line: Vec<u8> = reader.heard.drain(..=end).collect();
                lines.push(String::from_utf8_lossy(&line).into_owned());
            }
            if reader.heard.len() > THE_LONGEST_LINE {
                went = Some("it said more on one line than either side reads".to_owned());
            }
        }
        for line in lines {
            match FromTheReader::read(&line) {
                Ok(FromTheReader::Answered(answered)) => self.answers.push_back(answered),
                Ok(FromTheReader::SteppedAside) => {
                    if let Some(reader) = self.reader.as_mut() {
                        reader.stepped_aside = true;
                    }
                }
                Err(why) => eprintln!(
                    "alo-compositor: the screen reader said something that is not a message, and \
                     it was ignored: {why}"
                ),
            }
        }
        if let Some(why) = went {
            self.gone(&why);
        }
    }

    /// The reader is gone, for `why`.
    fn gone(&mut self, why: &str) {
        if self.reader.take().is_some() {
            eprintln!("alo-compositor: the sign-in screen's reader went away: {why}");
        }
        self.sent = None;
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_sign_in_screens_reader_tests.rs"]
mod tests;
