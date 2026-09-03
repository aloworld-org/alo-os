//! The fixtures this crate's tests are written against.
//!
//! Three of them, because this crate is tested against three things it does not
//! own: a service on the far end of a socket, the words a person reads, and the
//! places a question may go.
//!
//! **A real server rather than a mocked HTTP client**, for the reason
//! `alo-models`' own fixture gives: the thing worth testing is what goes out on
//! the wire and what is made of what comes back, and a mock at the client
//! boundary would assume both. That fixture is `pub(crate)` in a crate that
//! ships, as it should be, so this is a second one rather than a shared one —
//! and the difference between them is the reason it is not a copy: this one
//! reads a request **body**, because a question is a `POST`.
//!
//! **One vocabulary rather than one per file**, and it holds four crates'
//! words. What a person reads around a question is mostly not this crate's
//! (`words.rs` says why), so a fixture holding only this crate's two strings
//! would be a fixture that cannot render a single line the product actually
//! shows.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpListener;
use std::thread;

use alo_models::{InferenceSource, Provider, Region};
use alo_strings::{Language, Strings, Translation, Vocabulary, Word};

/// Everything the crates in this journey can say, in one vocabulary.
///
/// A shell has one of these and every crate declares into it, which is what the
/// area at the front of a key is for.
pub(crate) fn vocabulary() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    crate::words::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_answering::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// Those words with nothing translated: what a machine that has no translations
/// shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(vocabulary())
}

/// The same, in German, with the lines a question actually produces translated.
///
/// The three are here rather than in each test because they are the sentence a
/// person sees while their question is leaving, the clause naming where it went,
/// and the one beside the answer when it comes back. A test that translated only
/// its own string would be asserting about a line half of which nobody could
/// read, and passing.
pub(crate) fn translated(extra: &[(Word, &str)]) -> Strings {
    let vocabulary = vocabulary();
    let mut translation = Translation::into_language(german())
        .says(
            alo_models::words::BY_A_PROVIDER.key(),
            "von {provider}, in {region}",
        )
        .says(
            alo_egress::words::A_PROVIDER.key(),
            "{provider}, in {region}",
        )
        .says(
            alo_egress::words::IS_ASKING.key(),
            "{agent} stellt {destination} eine Frage",
        );
    for (word, says) in extra {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);
    strings
}

/// German, as `alo-strings` names a language.
pub(crate) fn german() -> Language {
    Language::written("de").unwrap()
}

/// A provider a person added, at this address.
pub(crate) fn mistral(endpoint: &str) -> Provider {
    Provider::checked(
        "Mistral",
        endpoint,
        Region::Declared("the EU".to_owned()),
        None,
    )
    .unwrap()
}

/// Where an answer from that provider says it came from.
pub(crate) fn mistral_source() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// One request, one canned reply.
///
/// Answers with the address to point a provider at, and a handle that yields
/// everything the client sent — head **and body** — so a test can assert on what
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
/// redirect is a provider telling this machine to go somewhere its rule never
/// answered about.
pub(crate) fn serving_with(
    response_body: &'static str,
    status: u16,
    extra_headers: &'static str,
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
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
