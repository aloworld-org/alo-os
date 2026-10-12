//! **Does the agent itself ask a real provider, with a key it looked up?**
//!
//! `alo-asking`'s own example drove the provider road directly, which proved
//! the road and not the daemon. This drives it the way a machine does: a key
//! in a real Secret Service, a provider in the person's own `settings.toml`,
//! the daemon's own lookup of both, and the ask inside the kernel boundary.
//!
//! # Three things this adds over driving the road directly
//!
//! | | |
//! |---|---|
//! | the key | kept in a real keyring and found by `Questions::a_key_from`, not handed in |
//! | the provider | read from the person's settings, not built in code |
//! | the ask | inside `ByTheKernel`, so the boundary decides what may be reached |
//!
//! # Why it is skipped by default, and loudly
//!
//! **It costs money and needs a key, so it cannot be part of the gate.** With
//! no `ALO_PROVIDER_KEY` it prints what it would have done and passes, which
//! is the one shape of pass worth being suspicious of — so it says so in as
//! many words rather than passing quietly. A reader of a green run should be
//! able to tell *skipped* from *proved* without knowing this file.
//!
//! The sibling `a_question_is_bounded_by_the_kernel` proves the same road
//! against a loopback server it starts itself, and that one **does** run in
//! the gate. This is the half that no server of ours can stand in for:
//! whether a real provider accepts what we send.
//!
//! # What it still does not prove
//!
//! **That no other traffic occurred.** The indicator showing one thing means
//! one thing was reported through it; it is not a measurement at the network
//! boundary, which is what law 1 actually promises. The kernel boundary is a
//! stronger statement — `inside_a_bounded_request_only_the_registered_address
//! _can_be_reached` holds that only the registered address is reachable during
//! the turn — and it is still about reachability rather than about what left.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime};

use alo_agentd::questions::{Questions, TheBound, WhatAnswers, WhoseKeyring};
use alo_agentd::{ByTheKernel, starting, what_an_agent_said};
use alo_bounding::{Imposed, Pinned};
use alo_capability::Grants;
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_keeping::Writing;
use alo_models::{Catalogue, SourcePolicy};
use alo_turn::{Machine, Turning};
use secret_service::EncryptionType;
use secret_service::blocking::SecretService;

/// The provider this names, in the settings and in the keyring.
const THE_PROVIDER: &str = "the-owners-test-provider";

/// **One of these on this machine at a time, and nothing forced.**
///
/// The same guard the sibling `a_question_is_bounded_by_the_kernel` takes, and
/// for the same two reasons: a mutex so two tests in this binary do not impose
/// a boundary at once, and `Waited::on_this_kernel` so another checkout's
/// kernel tests wait rather than collide.
///
/// Without it the second run of this test failed with *cannot make a control
/// group at /sys/fs/cgroup/init.scope/home, File exists* — the first run's
/// cgroup, still there. The guard is what makes the cleanup somebody else's
/// turn rather than a race.
fn the_only_one_on_this_machine() -> (MutexGuard<'static, ()>, alo_bounding::Waited) {
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    let ours = ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner);
    let kernel = alo_bounding::Waited::on_this_kernel()
        .expect("this kernel can be taken, and nothing is forced if it cannot");
    (ours, kernel)
}

/// The boundary, imposed and taken away with this value.
struct AsAMachineHasIt {
    pinned: Pinned,
    _loaded: Imposed,
}

impl AsAMachineHasIt {
    fn on_this_kernel(what: &str) -> Self {
        let pinned = Pinned::beneath(
            &PathBuf::from("/sys/fs/bpf")
                .join(format!("alo-provider-{}-{what}", std::process::id())),
        );
        pinned.taken_away();
        pinned
            .made()
            .expect("this machine has a BPF filesystem at /sys/fs/bpf");
        let loaded = Imposed::once(&pinned).unwrap_or_else(|why| {
            panic!("no boundary could be imposed, so nothing here is being tested: {why}")
        });
        Self {
            pinned,
            _loaded: loaded,
        }
    }
}

impl Drop for AsAMachineHasIt {
    fn drop(&mut self) {
        self.pinned.taken_away();
    }
}

/// **The agent asks, the daemon finds the key, and a real provider answers.**
#[test]
fn the_agent_asks_a_real_provider_with_a_key_it_looked_up() {
    let Ok(key) = std::env::var("ALO_PROVIDER_KEY") else {
        println!(
            "SKIPPED, and this is not a pass: ALO_PROVIDER_KEY is not set, so no provider was \
             asked anything. This test sends a real question to a real provider and costs \
             money, so the gate runs it with no key and it skips. \
             `a_question_is_bounded_by_the_kernel` covers the same road against a loopback \
             server and does run."
        );
        return;
    };
    let endpoint =
        std::env::var("ALO_PROVIDER").unwrap_or_else(|_| "https://api.openai.com".to_owned());
    let model = std::env::var("ALO_PROVIDER_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_owned());
    let asked = std::env::var("ALO_ASK")
        .unwrap_or_else(|_| "In one short sentence: what is an operating system?".to_owned());

    // A real Secret Service on a bus nobody else can reach. The key goes here
    // and nowhere else this test writes.
    //
    // Stored through a client of its own rather than through `TheKeyring`, the
    // way `alo-secrets`' own test does and for its reason: a test that stored
    // and read through one object would be proving itself. The attributes are
    // `crate::store`'s, which is what makes the item one alo OS will find.
    let keyring = alo_keyring_fixture::AKeyringOfOurOwn::started("the-agent-asks");
    let named = format!("provider/{THE_PROVIDER}");
    {
        let connection = zbus::blocking::connection::Builder::address(keyring.address().as_str())
            .expect("the fixture's address is an address")
            .build()
            .expect("the fixture's bus accepts a connection");
        let service = SecretService::connect_with_existing(EncryptionType::Dh, connection)
            .expect("the fixture serves the Secret Service");
        let collection = service
            .get_default_collection()
            .expect("the fixture has a default collection");
        let mut attributes = HashMap::new();
        attributes.insert("xdg:schema", "dev.alo.Provider");
        attributes.insert("reference", named.as_str());
        collection
            .create_item(
                "the owner's test provider key",
                attributes,
                key.as_bytes(),
                true,
                "text/plain",
            )
            .expect("an item can be stored");
    }
    println!("the key is kept under {named}");
    // The provider in the person's own settings, written rather than built, so
    // what the daemon parses is what a person would have.
    let folder = std::env::temp_dir().join(format!("alo-agent-asks-{}", std::process::id()));
    let config = folder.join(alo_choosing::THE_FOLDER);
    std::fs::create_dir_all(&config).expect("a directory for the person's settings");
    std::fs::write(
        config.join(alo_choosing::THE_SETTINGS),
        format!(
            "format = 2\n\n[answers]\nprovider = {{ name = \"{THE_PROVIDER}\", model = \
             \"{model}\" }}\n\n[[provider]]\nname = \"{THE_PROVIDER}\"\nendpoint = \
             \"{endpoint}\"\nregion = \"not stated for a test key\"\n"
        ),
    )
    .expect("the person's settings can be written");

    let mut questions = Questions::of_a_session(
        Some(folder.clone().into_os_string()),
        None,
        Catalogue::built_in().expect("the built-in catalogue reads"),
        TheBound::ThePersons(SourcePolicy::Anywhere),
        WhoseKeyring::On(keyring.bus()),
    );

    // **The wiring itself, asserted before anything is asked**, so a key that
    // never reached the daemon is told apart from a provider that refused.
    let WhatAnswers::FromAProvider { provider, .. } = questions.what_answers() else {
        panic!("the daemon did not read the provider choice from the person's settings");
    };
    let reference = provider
        .key
        .clone()
        .expect("the provider in the settings names a key");
    assert_eq!(
        reference.as_str(),
        named,
        "the daemon looked for the key under a different name than it was kept under"
    );
    Questions::a_key_from(&WhoseKeyring::On(keyring.bus()), &reference)
        .expect("the daemon can find the key in the keyring");
    println!("the daemon found it - the key is wired");

    // And now the ask, inside the boundary. The guard is taken before the
    // boundary is imposed and held for the rest of the test.
    let _one = the_only_one_on_this_machine();
    let kernel = AsAMachineHasIt::on_this_kernel("asks");
    let strings = starting::what_this_machine_says()
        .expect("this machine's own words")
        .into_strings();
    let mut writing =
        Writing::opening(&folder.join("record.jsonl")).expect("a record can be opened");
    let mut indicator = Indicator::default();
    let mut grants = Grants::default();
    let mut bounding =
        ByTheKernel::beneath(&kernel.pinned).expect("a service can open the map a loader pinned");

    let now = SystemTime::now();
    let said = {
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut writing,
        )
        .expect("the six declare");
        let mut turning = Turning::beginning(
            Context::at_invocation(now),
            "@mail",
            Duration::from_secs(60 * 60),
            &mut grants,
            &mut machine,
        )
        .expect("a turn can begin");

        let line = format!(
            r#"{{"format":1,"asks":{{"ask":{{"question":{}}}}}}}"#,
            serde_json::to_string(&asked).expect("a question is a string")
        );
        println!("the agent sends: {line}");
        what_an_agent_said(
            &line,
            &mut turning,
            &mut questions,
            None,
            &Grants::default(),
            &strings,
            Duration::from_secs(60 * 60),
            now,
        )
    };

    println!("what the agent was told: {said:?}");
    println!(
        "the indicator showed {} thing(s) during the turn",
        indicator.showing().len()
    );

    let answered = format!("{said:?}");
    assert!(
        !answered.contains("refused"),
        "the agent asked a provider and was refused: {answered}"
    );
    assert!(
        indicator.showing().is_empty(),
        "the indicator is read after the turn, so what it showed during it has ended"
    );

    // The record is what law 1's *afterwards* means, and it is on disk.
    let record = std::fs::read_to_string(folder.join("record.jsonl")).unwrap_or_default();
    assert!(
        !record.is_empty(),
        "a question went to a provider and the record holds nothing"
    );
    assert!(
        !record.contains(&key),
        "the record holds the key, which it must never"
    );
    // **It names the egress**, which is the half of law 1 that is about
    // afterwards. Non-empty was too weak: a record holding anything at all
    // would have passed, including a line about something else entirely.
    assert!(
        record.contains(THE_PROVIDER),
        "the record does not name the provider the question went to: {record}"
    );
    println!(
        "the record holds {} byte(s), names the provider, and not the key",
        record.len()
    );
    println!("the record: {record}");

    let _ = std::fs::remove_dir_all(&folder);
}
