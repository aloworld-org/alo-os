//! Whether a question a turn puts to a provider runs inside the boundary.
//!
//! **It does not, and this file is that fact with a test around it.**
//!
//! `alo-bounding`'s `socket_connect` refuses a bound turn any destination the
//! person was not shown. What decides whether a connection is a *bound turn's*
//! is the control group it is made from, and the only thing that puts a thread
//! into one is `Bounding::carrying_out`. So the question this file asks is
//! narrow and answerable: **is `carrying_out` called around a question?**
//!
//! It is called around a file verb, by `crate::carrying`. It is not called
//! around a question: `Turning::asking` reaches `alo_asking::Asking` directly,
//! on whichever thread the daemon is running on, which is in no turn's control
//! group. A programme that decides by control group therefore sees a provider
//! request as *not a turn* — the answer that allows everything — and the
//! enforcement built for exactly this case never runs.
//!
//! # Why a test rather than a paragraph in a report
//!
//! Because the paragraph would be true today and silently false later, in
//! either direction. If somebody bounds the asking, this fails and they have to
//! come here and say so. If somebody *unbounds* the file verbs, the other half
//! fails. Neither is a change anybody would make on purpose without noticing,
//! and both are changes that would leave the reports around them wrong.
//!
//! It is measurement, not approval. `docs/autonomy/updates/end-to-end-network-enforcement.md`
//! is where what to do about it is argued, and doing it changes the turn
//! lifecycle, which is a decision rather than a commit.

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

/// A boundary that counts how many executions were carried out inside it.
///
/// The whole instrument. It bounds nothing — `alo_turn::bounding` says why no
/// library here may ship an implementation that does — and it remembers one
/// number, which is the number this file is about.
#[derive(Debug, Default)]
struct Counting {
    /// How many times a thread was asked to go inside.
    times: u32,
}

impl alo_turn::Bounding for Counting {
    fn carrying_out(
        &mut self,
        _reaching: &alo_files::Reaching,
        doing: alo_turn::Doing<'_>,
    ) -> Result<alo_turn::Done, alo_turn::NoBoundary> {
        self.times = self.times.saturating_add(1);
        Ok(doing.done())
    }
}

/// **A file verb goes inside the boundary and a question does not.**
///
/// One turn, one of each. The count is one afterwards, and the one it counted
/// was the read.
///
/// What follows from it is the whole of
/// `docs/autonomy/updates/end-to-end-network-enforcement.md`: the destination
/// enforcement in `alo-bounding` decides by control group, a question is put
/// from a thread that is in none, and so **no provider request on this machine
/// is subject to it**. The mechanism is real and the production path does not
/// reach it.
#[test]
fn a_file_verb_is_carried_out_inside_a_boundary_and_a_question_is_not() {
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

        bounding.times
    };

    assert_eq!(
        counted, 1,
        "one execution was carried out inside a boundary and it was the file verb. If this \
         is now two, a question is being bounded — say so in \
         docs/autonomy/updates/end-to-end-network-enforcement.md and in the kernel plan, \
         because the destination enforcement in alo-bounding then finally applies to \
         provider requests."
    );

    let _ = fs::remove_dir_all(&folder);
}
