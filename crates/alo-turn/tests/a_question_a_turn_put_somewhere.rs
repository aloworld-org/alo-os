//! A question a turn put somewhere, against a real socket and a real record
//! file.
//!
//! `a_turn_on_this_machine.rs` is the other half of this bargain: a turn that
//! touches the disk in front of the person. This is the half that may not stay
//! on the machine at all — the same crate, the same record, and law 1 rather
//! than ADR 0001 §5 deciding what has to be true.
//!
//! **What it is really asking** is the sentence `CLAUDE.md` puts on this
//! product's front page: *nothing leaves silently*. A question goes to a
//! provider on a real socket and is findable afterwards in a file that has been
//! closed and read back; a question the rule refuses goes nowhere and is
//! findable as a refusal rather than as a departure. Neither can be
//! demonstrated by one crate, because the indicator, the decision, the door and
//! the record are four of them.
//!
//! It is not the hardware verification `CLAUDE.md` asks for: that is a
//! certified machine, and this is whatever the tests were run on. **Nothing
//! here has been run against a provider anybody pays for.**

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::Answering;
use alo_asking::Hosted;
use alo_capability::Grants;
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Resolving, file_words};
use alo_keeping::{Reading, Writing};
use alo_models::{InferenceSource, Provider, Region, Secret, SourcePolicy};
use alo_record::{Asking, Only};
use alo_strings::Strings;
use alo_turn::{Answers, Machine, Places, Turning};

/// The moment the person pressed the key.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the turn stands.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A folder of this test's own, resolved.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-turn-asked-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// The words this machine reads: every crate a question passes through, in one
/// vocabulary, which is the arrangement a shell is really in.
fn on_this_machine() -> Strings {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_answering::declare_into(&mut vocabulary).unwrap();
    alo_asking::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// One answer, in the shape every OpenAI-compatible provider replies with.
const AN_ANSWER: &str = r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"No, not without written consent."}}]}"#;

/// How many entries are on the disk at this moment.
fn lines_on_the_disk(path: &Path) -> usize {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .count()
}

/// One request, one canned reply, on a real socket.
///
/// A real server rather than a mocked client, for `alo-asking`'s reason: what
/// is worth asserting is what really crossed the wire. It yields everything the
/// client sent, head and body.
fn serving(reply: &'static str) -> (String, thread::JoinHandle<String>) {
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
            if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = value.trim().parse().unwrap_or(0);
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
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{reply}",
            reply.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        head + &String::from_utf8_lossy(&body)
    });
    (format!("http://127.0.0.1:{port}"), handle)
}

/// Where an answer from the provider these two tests use says it came from.
fn mistral() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// **A question that leaves, on a real socket, with the record on a real
/// disk.**
///
/// `@mail` asks a provider something, and the file on the disk
/// answers law 1's second question — *what left this machine today* — after the
/// record has been closed and read back by the crate that reads records.
///
/// Two things it asks that no unit test can. **The departure is on the disk
/// before the answer reaches the caller**, because `alo_keeping::Writing::keep`
/// syncs before it answers, so the count taken here is what a security review
/// would find if the machine lost power in the same instant. And **what was
/// asked and what came back are nowhere in the bytes**, which is ADR 0001 §7
/// asserted against the file rather than against a type.
///
/// It is not the hardware verification `CLAUDE.md` asks for, and nothing here
/// has been run against a provider anybody pays for.
#[test]
fn a_question_that_leaves_is_on_the_disk_before_the_answer_is_handed_back() {
    let strings = on_this_machine();
    let kept_at = a_folder_of_our_own("asked-record").join("record.jsonl");
    let (url, server) = serving(AN_ANSWER);
    let provider =
        Provider::checked("Mistral", &url, Region::Declared("the EU".to_owned()), None).unwrap();
    let key = Secret::typed("sk-live-0123456789").unwrap();

    {
        let mut writing = Writing::opening(&kept_at).unwrap();
        let mut indicator = Indicator::default();
        let mut grants = Grants::default();
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut indicator,
            &mut writing,
        )
        .unwrap();
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@mail",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();

        // Nothing was granted, and nothing needs to be: putting a question to a
        // model is what an agent is rather than something it is granted.
        assert!(grants.is_empty());
        assert!(turning.showing().is_quiet());

        let permitted = Answering::chosen(mistral(), &SourcePolicy::Anywhere).unwrap();
        let answer = turning
            .asking(
                "may the tenant sublet?",
                "mistral-small-latest",
                permitted,
                &Answers::Provider(Hosted::provider(&provider, Some(&key))),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
            .unwrap();

        // The departure was on the disk before the answer got here, and the
        // line is off the indicator because the connection is over.
        assert_eq!(lines_on_the_disk(&kept_at), 1);
        assert_eq!(answer.source(), &mistral());
        assert_eq!(answer.text(), "No, not without written consent.");
        assert!(turning.showing().is_quiet());
        assert!(!turning.ending(&mut grants));
    }

    // The question really crossed the wire, with the key on it.
    let sent = server.join().unwrap();
    assert!(sent.contains("may the tenant sublet?"), "{sent}");
    assert!(sent.contains("sk-live-0123456789"), "{sent}");

    // And the record, read back by the crate that reads records, answers law
    // 1's question: one thing left this machine, under this agent, to this
    // provider — and nothing left that nobody asked for.
    let read = Reading::at(&kept_at).unwrap();
    let record = read.record();
    assert_eq!(record.answering(&Asking::anything()).count(), 1);
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Egress))
            .count(),
        1
    );
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::OnItsOwn))
            .count(),
        0
    );
    let by_mail = Asking::anything().by("@mail");
    let entry = record.answering(&by_mail).next().unwrap();
    assert!(entry.happened().caused_egress());
    assert!(!entry.happened().was_stopped());
    assert_eq!(
        entry
            .happened()
            .destination()
            .map(|going_to| going_to.shown(&strings)),
        Some("Mistral, in the EU".to_owned())
    );

    // What was asked, what came back, and the key are nowhere in the file.
    let written = fs::read_to_string(&kept_at).unwrap();
    assert!(!written.contains("sublet"), "{written}");
    assert!(!written.contains("written consent"), "{written}");
    assert!(!written.contains("sk-live"), "{written}");
}

/// **A question the organisation's rule will not let leave**, on the same
/// arrangement: nothing is sent, the refusal is on the disk in the rule's own
/// words, and the record does not report it as something that left.
///
/// The provider is at an address nothing is listening on, so a rule that failed
/// to stop it would make this test fail rather than pass.
#[test]
fn a_question_the_rule_refuses_is_on_the_disk_as_a_refusal_and_not_as_egress() {
    let strings = on_this_machine();
    let kept_at = a_folder_of_our_own("refused-record").join("record.jsonl");
    let provider = Provider::checked(
        "Mistral",
        "https://127.0.0.2:1",
        Region::Declared("the EU".to_owned()),
        None,
    )
    .unwrap();

    {
        let mut writing = Writing::opening(&kept_at).unwrap();
        let mut indicator = Indicator::default();
        let mut grants = Grants::default();
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut indicator,
            &mut writing,
        )
        .unwrap();
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@mail",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();

        // The person chose this provider while the rule allowed it, and the
        // organisation has tightened the rule since.
        let permitted = Answering::chosen(mistral(), &SourcePolicy::Anywhere).unwrap();
        let refused = turning
            .asking(
                "may the tenant sublet?",
                "mistral-small-latest",
                permitted,
                &Answers::Provider(Hosted::provider(&provider, None)),
                &Places::under(&SourcePolicy::ThisMachineOnly),
                noon(),
            )
            .unwrap_err();

        assert!(refused.nothing_left());
        assert!(turning.showing().is_quiet());
        assert_eq!(lines_on_the_disk(&kept_at), 1);
        assert!(!turning.ending(&mut grants));
    }

    let read = Reading::at(&kept_at).unwrap();
    let record = read.record();
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Refusals))
            .count(),
        1
    );
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::Egress))
            .count(),
        0,
        "a question the rule refused is not a question that left"
    );
    let anything = Asking::anything();
    let entry = record.answering(&anything).next().unwrap();
    assert!(
        entry
            .happened()
            .why_stopped()
            .is_some_and(|why| why.as_str().contains("nothing leave")),
        "{:?}",
        entry.happened()
    );
}
