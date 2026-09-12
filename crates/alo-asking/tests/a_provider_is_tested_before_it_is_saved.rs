//! The whole journey: a provider typed in, tested with one request on the
//! indicator, and then saved — or not, and saved anyway if the person says so.
//!
//! `docs/features.md`, v0.5: *test a provider before saving it, so a mistyped
//! key is found now rather than in the middle of a question.* The pieces are
//! three crates' — `alo-models` has the wire, `alo-egress` the indicator and
//! `alo-choosing` the one door that writes a provider — and `alo_asking::Vetting`
//! is where they are joined. This file walks the join the way a settings panel
//! will, which none of those crates' own tests can do, and holds it to the
//! acceptance of lane B's task 2, one test per clause:
//!
//! - one request is made, for the model list, with the key;
//! - the outcome is one of exactly three values, each a sentence the machine
//!   can say;
//! - a refusal or an unreachable provider writes nothing, and the person may
//!   still save it if they say so;
//! - the request goes through `alo-egress`' accounting and the indicator says
//!   so;
//! - a provider this crate does not know is not guessed at;
//! - no retry, and no wait longer than a person would sit at a dialogue.
//!
//! # Owned fixtures, a synthetic secret, nothing external
//!
//! Every service here is a listener this test owns, on loopback, on a port the
//! operating system chose, and counts what reached it. The hosted provider that
//! is never reached is `https://0.0.0.0:1`, which is somewhere else to
//! `alo-models`, needs no name looked up, and has nothing listening. The key is
//! invented here and is a credential to nothing. **No external provider is
//! contacted.**
//!
//! **Why the hosted happy path is not a socket here.** A line goes on the
//! indicator only for an address that is not this machine, and a plain-HTTP
//! listener this test can own is on this machine; a TLS one at another address
//! is `alo-agentd`'s fixture and is not copied here. What is exercised instead
//! is the order: the departure is obtained before the request on the path where
//! nothing answered, which is the same line of code the answered path takes.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, BufReader, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_asking::{Found, NotVetted, Vetting};
use alo_capability::Grantee;
use alo_choosing::{Choosing, Settings};
use alo_egress::{Indicator, Why};
use alo_models::{NotTried, Provider, Region, Secret, SourcePolicy};
use alo_record::{Entry, Only, Record};
use alo_strings::{Said, Strings};

/// Two models, in the shape every OpenAI-compatible provider answers with.
const A_MODEL_LIST: &str = r#"{"object":"list","data":[{"id":"mistral-small-latest","object":"model"},{"id":"mistral-large-latest","object":"model"}]}"#;

/// A key that is not a credential to anything, and never was.
const A_SYNTHETIC_SECRET: &str = "sk-live-DO-NOT-LOG-9f3c1a";

/// What a service of this test's own saw.
struct Saw {
    /// How many connections reached it.
    connections: usize,
    /// The head of the first request, or nothing if none arrived.
    first_request: Option<String>,
}

/// A service that answers every connection the same way and counts them.
///
/// The count is the point: `alo-models`' fixture accepts one connection and
/// returns, which cannot tell one request from two. This keeps accepting until
/// the test says the door has returned, so a retry would be a second
/// connection and a failure rather than something nobody noticed.
fn a_counting_service(
    response_body: &'static str,
    status: u16,
) -> (String, Arc<AtomicBool>, thread::JoinHandle<Saw>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a listener of our own");
    let port = listener
        .local_addr()
        .expect("a listener knows where it is")
        .port();
    listener
        .set_nonblocking(true)
        .expect("a listener can be made not to wait");
    let done = Arc::new(AtomicBool::new(false));
    let stop = Arc::clone(&done);
    let handle = thread::spawn(move || {
        let mut saw = Saw {
            connections: 0,
            first_request: None,
        };
        // Keep listening a little after the door has returned, so a request
        // that arrived late is still counted rather than missed.
        let mut after = None;
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    saw.connections += 1;
                    stream
                        .set_nonblocking(false)
                        .expect("one connection can wait");
                    let mut reader = BufReader::new(stream.try_clone().expect("the same socket"));
                    let mut head = String::new();
                    loop {
                        let mut line = String::new();
                        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                            break;
                        }
                        head.push_str(&line);
                    }
                    saw.first_request.get_or_insert(head);
                    let reply = format!(
                        "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{response_body}",
                        response_body.len()
                    );
                    stream.write_all(reply.as_bytes()).expect("a reply");
                    stream.flush().expect("a reply flushed");
                }
                Err(_) => {
                    if stop.load(Ordering::SeqCst) {
                        let until = after.get_or_insert_with(|| {
                            std::time::Instant::now() + Duration::from_millis(300)
                        });
                        if std::time::Instant::now() >= *until {
                            break;
                        }
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        saw
    });
    (format!("http://127.0.0.1:{port}"), done, handle)
}

/// What a counting service saw, once the door has returned.
fn what_it_saw(done: &AtomicBool, handle: thread::JoinHandle<Saw>) -> Saw {
    done.store(true, Ordering::SeqCst);
    handle.join().expect("the service thread finishes")
}

/// A settings file under a folder only this test uses, made fresh.
fn settings_nobody_has_written(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-asking-vetting-{what}"));
    let _ = std::fs::remove_dir_all(&folder);
    folder.join("alo").join("settings.toml")
}

/// The surface that made the request, named by this test as a shell names
/// itself.
fn settings() -> Grantee {
    Grantee::named("@settings")
}

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

fn key() -> Secret {
    Secret::typed(A_SYNTHETIC_SECRET).expect("a key")
}

/// A provider on this machine, which the counting service stands in for.
fn local(url: &str) -> Provider {
    Provider::checked("Local", url, Region::Unknown, None).expect("a provider on this machine")
}

/// A provider that is genuinely somewhere else, at an address nothing is
/// listening on and that needs no name looked up.
fn far_away() -> Provider {
    Provider::checked(
        "Mistral",
        "https://0.0.0.0:1",
        Region::Declared("the EU".to_owned()),
        Some(alo_models::SecretRef::named("provider/Mistral")),
    )
    .expect("a hosted provider")
}

/// Everything the machine can say, which is what a process really holds.
fn the_machines_words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("alo OS's own words"))
}

/// **One request, for the model list, with the key — and a provider that
/// answered is then saved.** The service counts connections, so a retry would
/// fail this rather than go unnoticed.
#[test]
fn one_request_is_made_with_the_key_and_a_provider_that_answers_is_saved() {
    let (url, done, service) = a_counting_service(A_MODEL_LIST, 200);
    let provider = local(&url);
    let key = key();
    let mut indicator = Indicator::default();
    let at = settings_nobody_has_written("answered");
    let mut choosing = Choosing::at(&at).expect("settings nobody has written");

    let vetted = Vetting::provider(&provider, Some(&key))
        .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
        .expect("a test happened");
    let saw = what_it_saw(&done, service);

    assert_eq!(saw.connections, 1, "one request, once");
    let request = saw.first_request.expect("the request");
    assert!(request.starts_with("GET /v1/models "), "{request}");
    assert!(
        request
            .to_lowercase()
            .contains(&format!("authorization: bearer {A_SYNTHETIC_SECRET}\r\n").to_lowercase()),
        "{request}"
    );
    assert!(
        !request.to_lowercase().contains("content-length"),
        "nothing of the person's leaves: {request}"
    );

    let found = vetted.ended(&mut indicator);
    assert!(found.is_a_working_provider(), "{found:?}");
    assert!(
        matches!(&found, Found::Answered(tried) if tried.models() == ["mistral-small-latest", "mistral-large-latest"])
    );

    // It answered, so it is saved — through the same door as before.
    assert!(!at.exists(), "nothing was written before the person saved");
    choosing
        .adding(provider)
        .expect("a provider that answered is saved");
    assert!(
        Settings::at(&at)
            .expect("settings read back")
            .providers()
            .get("Local")
            .is_some()
    );
}

/// **A key the provider refuses writes nothing — and the person may still
/// save it if they say so.** The mistyped key is found while they are looking
/// at the field; the file is not touched by the test; and *save anyway* is the
/// door that was always there.
#[test]
fn a_key_the_provider_refuses_writes_nothing_and_the_person_may_still_save_it() {
    let (url, done, service) = a_counting_service(r#"{"message":"Unauthorized"}"#, 401);
    let provider = local(&url);
    let key = key();
    let mut indicator = Indicator::default();
    let at = settings_nobody_has_written("refused-key");
    let mut choosing = Choosing::at(&at).expect("settings nobody has written");

    let vetted = Vetting::provider(&provider, Some(&key))
        .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
        .expect("a test happened");
    assert_eq!(what_it_saw(&done, service).connections, 1);

    let found = vetted.ended(&mut indicator);
    assert_eq!(found, Found::RefusedTheKey(NotTried::KeyNotAccepted));
    assert!(!found.is_a_working_provider());
    let said = found.said(&the_machines_words());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("the whole key"), "{said}");
    assert!(!said.text().contains("DO-NOT-LOG"), "{said}");

    // Nothing was written: the file is not there and the settings are untouched.
    assert!(!at.exists(), "a refused key wrote a file");
    assert_eq!(*choosing.settings(), Settings::untouched());

    // And if the person says so, it is saved anyway — a provider whose key is
    // wrong today is still theirs to keep.
    choosing
        .adding(provider)
        .expect("saved anyway, on the person's word");
    assert!(at.exists());
    assert_eq!(
        Settings::at(&at)
            .expect("settings read back")
            .providers()
            .configured
            .len(),
        1
    );
}

/// **A provider somewhere else is shown leaving while it is tested**, the
/// departure comes back so what left is written down, nothing is written to the
/// settings, and a provider that is down today is not a wrong provider — the
/// person may still save it.
#[test]
fn a_provider_that_could_not_be_reached_is_shown_leaving_writes_nothing_and_may_be_saved() {
    let provider = far_away();
    let key = key();
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let at = settings_nobody_has_written("unreachable");
    let mut choosing = Choosing::at(&at).expect("settings nobody has written");

    let vetted = Vetting::provider(&provider, Some(&key))
        .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
        .expect("a test happened");

    // Law 1, first half: the indicator says so, in its own words, while the
    // line is still up.
    assert_eq!(indicator.showing().len(), 1);
    assert_eq!(
        indicator
            .showing()
            .first()
            .map(|shown| shown.said(&the_machines_words()).text().to_owned()),
        Some("@settings is fetching something from Mistral, in the EU".to_owned())
    );
    let departing = vetted.departing().expect("the request left this machine");
    assert_eq!(departing.why(), Why::Fetching);
    assert_eq!(departing.agent(), &settings());

    // Law 1, second half: what left can be written down, and only from the
    // departure the indicator handed back.
    record.keep(Entry::left(departing));
    assert_eq!(
        record
            .answering(&alo_record::Asking::anything().only(Only::Egress))
            .count(),
        1
    );

    let found = vetted.ended(&mut indicator);
    assert!(indicator.is_quiet());
    assert_eq!(found, Found::CouldNotBeReached(NotTried::Unreachable));
    let said = found.said(&the_machines_words());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("check the address"), "{said}");

    assert!(!at.exists(), "an unreachable provider wrote a file");
    choosing
        .adding(provider)
        .expect("saved anyway, on the person's word");
    assert_eq!(
        Settings::at(&at)
            .expect("settings read back")
            .providers()
            .get("Mistral")
            .map(|provider| provider.endpoint.as_str()),
        Some("https://0.0.0.0:1")
    );
}

/// **A rule that forbids reaching the provider sends nothing**, the indicator
/// stays quiet because nothing left, the refusal is the rule's own — and the
/// save proceeds exactly as it does today, because a test that did not happen
/// found nothing wrong.
#[test]
fn a_rule_that_forbids_the_test_sends_nothing_and_the_save_proceeds_as_it_does_today() {
    let provider = far_away();
    let key = key();
    let mut indicator = Indicator::default();
    let at = settings_nobody_has_written("forbidden");
    let mut choosing = Choosing::at(&at).expect("settings nobody has written");

    let not_vetted = Vetting::provider(&provider, Some(&key))
        .under(
            &settings(),
            &SourcePolicy::InTheBuilding,
            &mut indicator,
            noon(),
        )
        .expect_err("the rule refused it");

    assert!(
        matches!(not_vetted, NotVetted::HeldBack(_)),
        "{not_vetted:?}"
    );
    assert!(indicator.is_quiet());
    let said = not_vetted.said(&the_machines_words());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("in the building"), "{said}");
    assert!(said.text().contains("Mistral, in the EU"), "{said}");

    assert!(!at.exists());
    choosing
        .adding(provider)
        .expect("saved as it always could be");
    assert!(at.exists());
}

/// **A provider this crate does not know how to reach is not guessed at.**
/// Nothing is sent, nothing is shown, the sentence says it cannot be tested
/// from here — and the save proceeds as it does today, through the writer's
/// own judgement of the address.
#[test]
fn a_provider_this_crate_does_not_know_is_not_guessed_at_and_the_save_proceeds() {
    // Built by hand, past `Provider::checked`, because that is the only way
    // such an address reaches the door at all.
    let provider = Provider {
        name: "Somewhere".to_owned(),
        endpoint: "ftp://models.example".to_owned(),
        region: Region::Unknown,
        key: None,
        models: Vec::new(),
    };
    let key = key();
    let mut indicator = Indicator::default();
    let at = settings_nobody_has_written("unknown");
    let mut choosing = Choosing::at(&at).expect("settings nobody has written");

    let not_vetted = Vetting::provider(&provider, Some(&key))
        .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
        .expect_err("nothing here knows how to reach it");

    assert_eq!(not_vetted, NotVetted::CannotBeTestedFromHere);
    assert!(indicator.is_quiet());
    let said = not_vetted.said(&the_machines_words());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("cannot be tested from here"), "{said}");

    // The save proceeds as it does today — which for this address is the
    // writer refusing it as not an address, in `alo-models`' own words. The
    // test neither added a road to the file nor took one away.
    let refused = choosing
        .adding(provider)
        .expect_err("the writer judges the address as it always has");
    assert!(
        matches!(
            refused,
            alo_choosing::NotWritten::NotAProvider {
                why: alo_models::ProviderError::NotAnAddress,
                ..
            }
        ),
        "{refused:?}"
    );
    assert!(!at.exists());
}

/// **Every sentence a test can produce is one the whole machine can say.** The
/// three outcomes and the four reasons nothing was sent, each rendered against
/// the vocabulary `alo-saying` collects rather than any crate's own.
#[test]
fn every_sentence_a_test_can_say_is_one_the_machine_can_say() {
    let strings = the_machines_words();
    let mut said: Vec<Said> = Vec::new();

    for (body, status) in [
        (A_MODEL_LIST, 200),
        (r#"{"message":"Unauthorized"}"#, 401),
        (r#"{"error":"upstream capacity exceeded"}"#, 503),
    ] {
        let (url, done, service) = a_counting_service(body, status);
        let provider = local(&url);
        let key = key();
        let mut indicator = Indicator::default();
        let vetted = Vetting::provider(&provider, Some(&key))
            .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
            .expect("a test happened");
        assert_eq!(what_it_saw(&done, service).connections, 1, "{status}");
        let found = vetted.ended(&mut indicator);
        said.extend(found.caveats(&strings));
        said.push(found.said(&strings));
    }

    let mut indicator = Indicator::default();
    let vetted = Vetting::provider(&far_away(), None)
        .under(&settings(), &SourcePolicy::Anywhere, &mut indicator, noon())
        .expect("a test happened");
    said.push(vetted.ended(&mut indicator).said(&strings));

    for not_vetted in [
        Vetting::provider(&far_away(), None)
            .under(
                &settings(),
                &SourcePolicy::ThisMachineOnly,
                &mut indicator,
                noon(),
            )
            .expect_err("held back"),
        NotVetted::CannotBeTestedFromHere,
        NotVetted::CannotBeShown(alo_egress::DestinationError::NotPrintable),
        NotVetted::Forbidden(alo_models::NotAllowed::OutsideTheBuilding {
            source: far_away().source(),
        }),
    ] {
        said.push(not_vetted.said(&strings));
    }

    assert_eq!(said.len(), 8, "{said:?}");
    for sentence in &said {
        assert!(!sentence.is_a_bug(), "{sentence}");
        assert!(!sentence.text().contains("DO-NOT-LOG"), "{sentence}");
    }
}

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// A source file of this repository, read.
fn source(relative: &str) -> String {
    let at = the_repository().join(relative);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// The shipped code of a file: no unit tests, no whole-line comments, no
/// trailing comments, no string literals.
///
/// Coarse on purpose — it has to find a keyword, not parse Rust — and the same
/// shape `nothing_here_keeps_the_key.rs` uses, whose header says why a string
/// spanning lines is state carried from the line before.
fn the_code_of(written: &str) -> String {
    let mut code = String::new();
    let mut inside = false;
    for line in written.lines() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        if !inside && line.trim_start().starts_with("//") {
            continue;
        }
        let mut kept = String::new();
        let mut letters = line.chars().peekable();
        while let Some(letter) = letters.next() {
            if inside {
                if letter == '\\' {
                    letters.next();
                } else if letter == '"' {
                    inside = false;
                }
                continue;
            }
            if letter == '"' {
                inside = true;
                continue;
            }
            if letter == '/' && letters.peek() == Some(&'/') {
                break;
            }
            kept.push(letter);
        }
        code.push_str(&kept);
        code.push('\n');
    }
    code
}

/// Whether this code uses the identifier as a word, rather than as part of a
/// longer one.
fn names(code: &str, word: &str) -> usize {
    let a_word = |letter: Option<char>| {
        letter.is_some_and(|letter| letter.is_alphanumeric() || letter == '_')
    };
    let mut count = 0;
    let mut rest = code;
    while let Some(at) = rest.find(word) {
        let before = rest[..at].chars().next_back();
        let after = rest[at + word.len()..].chars().next();
        if !a_word(before) && !a_word(after) {
            count += 1;
        }
        rest = &rest[at + word.len()..];
    }
    count
}

/// **One request, no retry, and no wait of this door's own.** The door calls
/// the wire exactly once per path, has no loop around it, and the wait is
/// `alo-models`' — which is ten seconds, the time somebody sits at a dialogue.
#[test]
fn the_test_is_one_request_with_no_retry_and_no_long_wait() {
    let door = the_code_of(&source("crates/alo-asking/src/vetting.rs"));
    // One call on the path for this machine and one on the path for a
    // provider elsewhere, and they are on different paths of one function.
    assert_eq!(door.matches(".under(policy)").count(), 2, "{door}");
    for repeating in ["loop", "while", "for", "retry", "sleep", "Duration"] {
        assert_eq!(
            names(&door, repeating),
            0,
            "the door {repeating}s, and a test request is made once: {door}"
        );
    }

    let wire = source("crates/alo-models/src/trying.rs");
    let waits = wire
        .lines()
        .find(|line| line.contains("const WHILE_SOMEBODY_WAITS"))
        .expect("the wire names how long it waits");
    let seconds: u64 = waits
        .split("from_secs(")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .and_then(|digits| digits.trim().parse().ok())
        .expect("the wait is written in whole seconds");
    assert!(
        seconds <= 30,
        "the wire waits {seconds}s, longer than a person sits at a dialogue"
    );
}

/// Every Rust file that ships under a crate's `src`, as a path from the
/// repository — unit-test fixtures excluded, because `#[cfg(test)] mod testing`
/// is not shipped.
fn every_shipped_source() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let crates = the_repository().join("crates");
    let mut looking: Vec<PathBuf> = std::fs::read_dir(&crates)
        .expect("the crates directory")
        .map(|entry| entry.expect("an entry").path().join("src"))
        .filter(|src| src.is_dir())
        .collect();
    while let Some(folder) = looking.pop() {
        for entry in std::fs::read_dir(&folder).expect("a source directory") {
            let at = entry.expect("an entry").path();
            if at.is_dir() {
                looking.push(at);
            } else if at.extension().is_some_and(|kind| kind == "rs")
                && at.file_name().is_none_or(|name| name != "testing.rs")
            {
                let relative = at
                    .strip_prefix(the_repository())
                    .expect("under the repository")
                    .to_string_lossy()
                    .replace('\\', "/");
                found.push((
                    relative,
                    std::fs::read_to_string(&at).expect("a source file"),
                ));
            }
        }
    }
    assert!(found.len() > 40, "this workspace has more files than that");
    found
}

/// **Nothing that ships reaches the wire except through this door.**
/// `alo_models::Trying` is public because `Vetting` needs it, and anything
/// else that named it would be a test request with the indicator quiet — so
/// the workspace is read, and the only code naming it is the wire itself and
/// the door that shows it.
#[test]
fn nothing_that_ships_reaches_the_wire_except_through_the_door_that_shows_it() {
    let allowed = [
        "crates/alo-models/src/trying.rs",
        "crates/alo-models/src/lib.rs",
        "crates/alo-asking/src/vetting.rs",
    ];
    for (at, written) in every_shipped_source() {
        let mentions = names(&the_code_of(&written), "Trying");
        if allowed.contains(&at.as_str()) {
            continue;
        }
        assert_eq!(
            mentions, 0,
            "{at} reaches alo_models::Trying, which makes a test request with nothing on the \
             indicator — use alo_asking::Vetting, which shows it"
        );
    }
    // And the door really does name it, so the check above is a check of
    // something.
    assert!(
        names(
            &the_code_of(&source("crates/alo-asking/src/vetting.rs")),
            "Trying"
        ) >= 2
    );
}
