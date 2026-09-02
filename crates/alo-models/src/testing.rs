//! The fixtures this crate's tests are written against.
//!
//! Two of them, because this crate is tested against two things it does not
//! own: a service on the far end of a socket, and the words a person reads.
//!
//! **A real server rather than a mocked HTTP client**, because the thing worth
//! testing is what goes out on the wire and what we make of what comes back —
//! which a mock at the client boundary would assume rather than check. The
//! Ollama adapter and the provider test both need one, and two copies of a
//! fixture drift into two fixtures.
//!
//! **One vocabulary rather than one per file.** Every file here that says
//! something has the same two questions to answer — *what does this say on a
//! machine with no translations* and *what does it say when somebody has
//! translated it* — and answering them from one fixture is what stops six files
//! inventing six vocabularies that resemble the real one. The real one is
//! [`crate::model_words`], and both of these are built from it.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpListener;
use std::thread;

use alo_strings::{Language, Strings, Translation, Word};

use crate::words::model_words;

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(model_words().unwrap())
}

/// The same, with these words translated into German and German preferred.
///
/// German because most of what this crate says is a sentence rather than a
/// label, and German moves the verb — so a translation that came out reading
/// like English with the words swapped would not be exercising anything.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = model_words().unwrap();
    let mut german = Translation::into_language(german_language());
    for (word, says) in words {
        german = german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german_language()]);
    strings
}

/// German, as `alo-strings` names a language.
pub(crate) fn german_language() -> Language {
    Language::written("de").unwrap()
}

/// One request, one canned reply.
///
/// Answers with the address to point an adapter at, and a handle that yields
/// everything the client sent — head and body — so a test can assert on what
/// really went out rather than on what was intended.
pub(crate) fn serving(
    response_body: &'static str,
    status: u16,
) -> (String, thread::JoinHandle<String>) {
    serving_with(response_body, status, "")
}

/// The same, with extra header lines in the reply.
///
/// `extra_headers` is raw, each line ending `\r\n`. It exists for `Location:`,
/// which is the one reply where the head matters as much as the body: a
/// redirect is a provider telling us to go somewhere the policy never answered
/// about.
pub(crate) fn serving_with(
    response_body: &'static str,
    status: u16,
    extra_headers: &'static str,
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        // Read the request head, and the body if one was announced.
        let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
        let mut head = String::new();
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = v.trim().parse().unwrap_or(0);
            }
            let done = line == "\r\n" || line == "\n";
            head.push_str(&line);
            if done {
                break;
            }
        }
        let mut body = vec![0u8; length];
        if length > 0 {
            reader.read_exact(&mut body).unwrap();
        }
        let reply = format!(
            "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{extra_headers}Connection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        stream.write_all(reply.as_bytes()).unwrap();
        stream.flush().unwrap();
        head + &String::from_utf8_lossy(&body)
    });
    (format!("http://127.0.0.1:{port}"), handle)
}
