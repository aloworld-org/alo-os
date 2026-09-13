//! The last word of *point alo OS at weights you already have* — **and it runs
//! them.**
//!
//! `a_model_we_never_catalogued.rs` asks the first half: a file on this machine
//! is offered as a model with no catalogue entry, its licence the person's and
//! its size measured off the disk. That much is settings. Until the runtime has
//! been told, the first question put to those weights fails as *no model
//! there* — honest, and not the promise.
//!
//! This one walks the rest of the road from outside: a file, the weights a
//! person's settings would carry for it, the door, and a runtime fixture on the
//! other side of a real socket answering by the file's own name.
//!
//! | The acceptance | The test |
//! |---|---|
//! | the runtime is told, and afterwards answers by that id | [`a_brought_file_is_told_to_the_runtime_and_then_answers_by_its_own_name`] |
//! | nothing downloads: the Modelfile names a path on this disk | [`what_the_runtime_is_told_names_a_path_on_this_disk_and_asks_for_no_download`] |
//! | the door is not the choosing door — a runtime that is down costs nothing typed | [`a_runtime_that_is_down_costs_the_person_nothing_they_typed`] |
//!
//! # It needs no runtime
//!
//! The far side is a socket this test opens and answers on, which is how every
//! other adapter test in this crate is written. What it shows is that the
//! **request alo OS sends** is the right one and that the answer is carried
//! back; whether the pinned runtime accepts that request is a question for a
//! machine with the runtime on it, and the report says so rather than this
//! file implying otherwise.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;

use alo_models::{Catalogue, ModelRuntime, Ollama, RuntimeError, Weights};

/// A file on this machine's own disk that only this test uses.
fn a_file_of_our_own(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-brought-road-{what}-{}", std::process::id()));
    std::fs::write(&at, b"GGUF\0\0\0\0").unwrap();
    at
}

/// A runtime that answers two requests in turn and hands back what it was
/// sent, so the road can be walked rather than described.
///
/// Written here rather than borrowed from the crate's own fixtures because
/// those are `pub(crate)`: an integration test is outside, which is the point
/// of it.
fn a_runtime_answering(replies: [&'static str; 2]) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let mut sent = Vec::new();
        for reply in replies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut head = String::new();
            let mut length = 0usize;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if let Some(said) = line.to_lowercase().strip_prefix("content-length:") {
                    length = said.trim().parse().unwrap_or(0);
                }
                let done = line == "\r\n" || line.is_empty();
                head.push_str(&line);
                if done {
                    break;
                }
            }
            let mut body = vec![0u8; length];
            std::io::Read::read_exact(&mut reader, &mut body).unwrap();
            head.push_str(&String::from_utf8_lossy(&body));
            sent.push(head);
            write!(
                stream,
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{reply}",
                reply.len()
            )
            .unwrap();
            stream.flush().unwrap();
        }
        sent
    });
    (address, handle)
}

/// **The runtime is told about the file, and afterwards answers by its id.**
///
/// Two requests over one socket: the door, and then a question put to the name
/// the door taught it. That second request is the whole point — an id the
/// runtime did not know before this road was walked.
#[test]
fn a_brought_file_is_told_to_the_runtime_and_then_answers_by_its_own_name() {
    let file = a_file_of_our_own("answers");
    let weights = Weights::at(&file).unwrap();
    let (address, runtime) = a_runtime_answering([
        r#"{"status":"success"}"#,
        r#"{"message":{"role":"assistant","content":"Yes, and here is why."}}"#,
    ]);
    let ollama = Ollama::at(&address, Catalogue::built_in().unwrap());

    ollama.bring(&weights).unwrap();
    let said = ollama.answers("Does this run?", &weights.id).unwrap();

    let sent = runtime.join().unwrap();
    assert_eq!(sent.len(), 2, "the runtime was not asked twice: {sent:?}");
    let taught = sent.first().unwrap();
    let asked = sent.get(1).unwrap();
    assert_eq!(said, "Yes, and here is why.");
    assert!(taught.contains("POST /api/create"), "{taught}");
    assert!(asked.contains("POST /api/chat"), "{asked}");
    assert!(
        asked.contains(&weights.id),
        "the question was not put to the id the door taught the runtime: {asked}"
    );
    drop(std::fs::remove_file(&file));
}

/// **Nothing downloads.** The Modelfile is one `FROM` naming a path on this
/// disk, because to the runtime a bare name is an instruction to fetch from a
/// publisher — which would turn *run the weights you already have* into an
/// egress nobody asked for.
#[test]
fn what_the_runtime_is_told_names_a_path_on_this_disk_and_asks_for_no_download() {
    let file = a_file_of_our_own("no-download");
    let weights = Weights::at(&file).unwrap();
    let (address, runtime) = a_runtime_answering([r#"{"status":"success"}"#, r#"{}"#]);
    let ollama = Ollama::at(&address, Catalogue::built_in().unwrap());

    ollama.bring(&weights).unwrap();
    drop(ollama.answers("anything", &weights.id));

    let sent = runtime.join().unwrap();
    let taught = sent.first().unwrap();
    let body = taught.split_once("\r\n\r\n").map(|(_, body)| body).unwrap();
    let body: serde_json::Value = serde_json::from_str(body).unwrap();
    let modelfile = body.get("modelfile").unwrap().as_str().unwrap();

    assert_eq!(modelfile, format!("FROM {}", file.display()));
    assert!(
        !taught.contains("/api/pull"),
        "the road went near a download: {taught}"
    );
    drop(std::fs::remove_file(&file));
}

/// **The door is not the choosing door.** It is called after the person's
/// choice is already written, so a runtime that is down refuses here and costs
/// them nothing they typed — the file stays chosen, and telling the runtime is
/// something that can be done again.
#[test]
fn a_runtime_that_is_down_costs_the_person_nothing_they_typed() {
    let file = a_file_of_our_own("down");
    let weights = Weights::at(&file).unwrap();

    // Port 1 is not listening.
    let refused = Ollama::at("http://127.0.0.1:1", Catalogue::built_in().unwrap())
        .bring(&weights)
        .unwrap_err();

    assert_eq!(refused, RuntimeError::Unreachable);
    assert!(
        file.is_file(),
        "the file a person pointed at was disturbed by a runtime being down"
    );
    assert_eq!(
        weights.file.as_deref(),
        Some(file.as_path()),
        "the weights stopped naming the file they were made from"
    );
    drop(std::fs::remove_file(&file));
}
