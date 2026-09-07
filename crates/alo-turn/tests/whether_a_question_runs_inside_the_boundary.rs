//! Whether a question a turn puts to a provider runs inside the boundary.
//!
//! **It does now, and this file is that fact with a test around it.** When it
//! was written the answer was no, and its own message said that whoever bounded
//! the asking had to come here and say what changed. This is that.
//!
//! `alo-bounding`'s `socket_connect` refuses a bound turn any destination the
//! person was not shown. What decides whether a connection is a *bound turn's*
//! is the control group it is made from, and what puts a thread into one is
//! `Bounding`. So the question this file asks is narrow and answerable: **is a
//! boundary entered around each of the two things a turn does?**
//!
//! A file verb goes through `Bounding::carrying_out`, by `crate::carrying`. A
//! question goes through `Bounding::carrying_out_a_departure`, by
//! `crate::asking`, with the addresses it resolved before entering — ADR 0020.
//! Both are counted here, and both have to happen.
//!
//! # Why a test rather than a paragraph in a report
//!
//! Because the paragraph would be true today and silently false later, in
//! either direction. If somebody bounds the asking, this fails and they have to
//! come here and say so. If somebody *unbounds* the file verbs, the other half
//! fails. Neither is a change anybody would make on purpose without noticing,
//! and both are changes that would leave the reports around them wrong.
//!
//! `docs/autonomy/updates/end-to-end-network-enforcement.md` is where the whole
//! of it is argued, and ADR 0020 is the decision it rests on.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_answering::Answering;
use alo_asking::Hosted;
use alo_capability::{Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Resolving, file_words};
use alo_keeping::Writing;
use alo_models::{InferenceSource, Provider, Region, Secret, SourcePolicy};
use alo_strings::Strings;
use alo_turn::{Answers, Machine, Places, Turning};

/// The moment the person pressed the key.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a turn and its grants last here.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A folder of this test's own, resolved.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let at = std::env::temp_dir().join(format!(
        "alo-inside-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&at);
    fs::create_dir_all(&at).unwrap();
    OnThisMachine.real(&at).unwrap().into_path_buf()
}

/// The words this machine reads, with nothing translated.
fn on_this_machine() -> Strings {
    let mut vocabulary = file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_asking::declare_into(&mut vocabulary).unwrap();
    alo_answering::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// Where an answer would have said it came from.
fn mistral() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// A boundary that counts what was carried out inside it, of each kind.
///
/// The whole instrument. It bounds nothing — `alo_turn::bounding` says why no
/// library here may ship an implementation that does — and it remembers two
/// numbers, which are the numbers this file is about.
#[derive(Debug, Default)]
struct Counting {
    /// How many file verbs went inside.
    verbs: u32,

    /// How many network requests went inside, and where they were let reach.
    departures: Vec<Vec<std::net::SocketAddr>>,
}

impl alo_turn::Bounding for Counting {
    fn carrying_out(
        &mut self,
        _reaching: &alo_files::Reaching,
        doing: alo_turn::Doing<'_>,
    ) -> Result<alo_turn::Done, alo_turn::NoBoundary> {
        self.verbs = self.verbs.saturating_add(1);
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), alo_turn::NoBoundary> {
        // What it was let reach is kept, because *that a boundary was entered*
        // and *what it permitted* are two different claims and only the second
        // one is worth much.
        self.departures.push(to.to_vec());
        doing();
        Ok(())
    }
}

/// **Both a file verb and a question are carried out inside a boundary.**
///
/// One turn, one of each, and the counts say one apiece. The question's also
/// says *where it was let reach*: the addresses the provider's endpoint
/// resolved to, resolved before the boundary was entered and registered so the
/// machine refuses everywhere else (ADR 0020).
///
/// This asserted the opposite when it was written, and its message named the
/// documents to change on the day it stopped being true. That day was the
/// commit that added `Bounding::carrying_out_a_departure`.
#[test]
fn a_file_verb_and_a_question_are_both_carried_out_inside_a_boundary() {
    let folder = a_folder_of_our_own("counting");
    fs::write(folder.join("march.pdf"), b"an invoice").unwrap();
    let kept_at = folder.join("record.jsonl");
    let strings = on_this_machine();
    // An address on loopback with nothing listening: what is measured happens
    // before a connection, so the connection failing is expected.
    let provider = Provider::checked(
        "Mistral",
        "http://127.0.0.1:1",
        Region::Declared("the EU".to_owned()),
        None,
    )
    .unwrap();
    let key = Secret::typed("sk-live-0123456789").unwrap();

    let counted = {
        let mut writing = Writing::opening(&kept_at).unwrap();
        let mut indicator = Indicator::default();
        let mut grants = Grants::default();
        grants
            .grant(Grant::checked("@mail", Reach::Folder(folder.clone()), noon(), hour()).unwrap());
        let mut bounding = Counting::default();
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
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

        // A read of a granted file. This is the path that is bounded.
        let read = turning.reading(
            "read_file",
            &[(
                "file",
                alo_capability::Given::text(
                    folder.join("march.pdf").to_string_lossy().into_owned(),
                ),
            )],
            &grants,
            noon(),
        );
        assert!(read.is_ok(), "the granted read did not happen: {read:?}");

        // And a question to a provider. The address is one nothing listens on,
        // because what is being measured happens *before* a connection: whether
        // the thread that is about to make it was put inside a boundary. The
        // question failing to connect is expected and is not the subject.
        let permitted = Answering::chosen(mistral(), &SourcePolicy::Anywhere).unwrap();
        let asked = turning.asking(
            "may the tenant sublet?",
            "mistral-small-latest",
            permitted,
            &Answers::Provider(Hosted::provider(&provider, Some(&key))),
            &Places::under(&SourcePolicy::Anywhere),
            noon(),
        );
        assert!(
            asked.is_err(),
            "a question to an address nothing listens on answered: {asked:?}"
        );

        (bounding.verbs, bounding.departures.clone())
    };
    let (verbs, departures) = counted;

    assert_eq!(
        verbs, 1,
        "the file verb was not carried out inside a boundary"
    );
    assert_eq!(
        departures.len(),
        1,
        "the question was not carried out inside a boundary, so the destination          enforcement in alo-bounding does not apply to provider requests"
    );
    // And it was let reach what its endpoint resolves to, and only that.
    assert_eq!(
        departures.first().map(Vec::as_slice),
        Some([std::net::SocketAddr::from(([127, 0, 0, 1], 1))].as_slice()),
        "the question was bounded to something other than what its endpoint resolves to"
    );

    let _ = fs::remove_dir_all(&folder);
}
