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
    let address = address_of(&listener);
    let handle =
        thread::spawn(move || one_exchange(&listener, response_body, status, extra_headers));
    (address, handle)
}

/// Several requests in a row, each with the reply that stands beside it.
///
/// [`serving`] is one request because nearly every call in this crate is one
/// call. Finding a runtime is the exception: it asks the runtime a question of
/// its own and the caller then asks it theirs, so a server that closed after
/// the first would make *found* untestable against anything but a refused
/// connection.
///
/// Answers with what the client sent, in the order it sent it.
pub(crate) fn serving_in_turn(
    replies: &'static [&'static str],
    status: u16,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = address_of(&listener);
    let handle = thread::spawn(move || {
        replies
            .iter()
            .map(|reply| one_exchange(&listener, reply, status, ""))
            .collect()
    });
    (address, handle)
}

/// Several requests in a row, each answered with its own status and reply.
///
/// [`serving_in_turn`] answers every request with one status. Handing a file to
/// the runtime is three requests whose statuses are the point — *not held yet*,
/// *taken*, *created* — so a server that could only say `200` would test a road
/// the runtime never takes.
pub(crate) fn serving_each(
    replies: &'static [(u16, &'static str)],
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = address_of(&listener);
    let handle = thread::spawn(move || {
        replies
            .iter()
            .map(|(status, reply)| one_exchange(&listener, reply, *status, ""))
            .collect()
    });
    (address, handle)
}

/// Where a listener is, as an adapter is pointed at it.
fn address_of(listener: &TcpListener) -> String {
    format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port())
}

/// Take one connection, read the whole request, answer it, and say what arrived.
fn one_exchange(
    listener: &TcpListener,
    response_body: &str,
    status: u16,
    extra_headers: &str,
) -> String {
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
}

/// A machine that says all three things a grade needs beside it.
pub(crate) fn a_machine() -> crate::MeasuredOn {
    crate::MeasuredOn {
        machine: "a test machine, 16 GB".to_owned(),
        date: "2026-09-14".to_owned(),
        runtime: "Ollama 0.34.0".to_owned(),
        drove: None,
        of: None,
        loaded_bytes: None,
        on_the_gpu_bytes: None,
        held_to: None,
        instructions: None,
    }
}

/// **A kernel's list of what draws**, written by a test rather than by a
/// kernel, and removed when the test is done with it.
///
/// Here rather than beside one file's tests because two files need it —
/// [`crate::card`] reads it, and [`crate::road`] decides from what it read —
/// and two copies of a fixture drift into two fixtures, which is this file's
/// own rule about the server above.
///
/// It is a folder of real files because that is what
/// [`crate::card::WhatDrawsHere::among`] takes: a machine whose cards could be
/// stated instead of read would be a reader nobody had checked against a
/// machine.
pub(crate) struct ADrmList(std::path::PathBuf);

/// An empty list, under this machine's temporary folder, with a name no other
/// test shares.
pub(crate) fn a_list_of_what_draws() -> ADrmList {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let at = loop {
        let at = std::env::temp_dir().join(format!(
            "alo-models-drm-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&at) {
            Ok(()) => break at,
            Err(why) => {
                assert!(
                    why.kind() == std::io::ErrorKind::AlreadyExists,
                    "a folder for this test: {why}"
                );
            }
        }
    };
    // The kernel's own list carries this file beside the devices, and a reader
    // that counted it as one would report a machine with no card as having one.
    std::fs::write(at.join("version"), "drm 1.1.0 20060810\n").unwrap();
    ADrmList(at)
}

impl ADrmList {
    /// Where the list is.
    pub(crate) fn at(&self) -> &std::path::Path {
        &self.0
    }

    /// One device written the way the kernel writes one: its vendor on the bus,
    /// the driver bound to it where one is, and the memory it publishes where
    /// it publishes any.
    pub(crate) fn holding(
        &self,
        named: &str,
        vendor: &str,
        driver: Option<&str>,
        memory: Option<u64>,
    ) {
        let device = self.0.join(named).join("device");
        std::fs::create_dir_all(&device).unwrap();
        std::fs::write(device.join("vendor"), format!("{vendor}\n")).unwrap();
        let uevent = match driver {
            Some(driver) => format!("DRIVER={driver}\nPCI_ID={vendor}:0001\n"),
            None => format!("PCI_ID={vendor}:0001\n"),
        };
        std::fs::write(device.join("uevent"), uevent).unwrap();
        if let Some(memory) = memory {
            std::fs::write(device.join("mem_info_vram_total"), format!("{memory}\n")).unwrap();
        }
    }
}

impl Drop for ADrmList {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.0));
    }
}
