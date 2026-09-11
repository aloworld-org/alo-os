//! What a keyring connection costs, how many there may be at once, and what
//! happens to one when the session it belongs to ends.
//!
//! ADR 0022 records a **shared-singleton** limitation, and records it as
//! *libsecret's*: `secret_service_get_sync()` hands back one service proxy for
//! the whole process, so dropping a wrapper is no evidence the bus connection
//! closed. The amendment of 2026-09-08 replaced libsecret with a client this
//! crate constructs per connection, and said in as many words that this makes
//! connection lifetime **something to measure rather than inherit**.
//!
//! This is that measurement. Nothing here assumes the libsecret answer and
//! nothing here assumes the opposite either.
//!
//! **Every credential is synthetic**, in a keyring started in a temporary
//! directory and killed afterwards.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;
use std::time::{Duration, Instant};

use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_models::SecretRef;
use alo_secrets::{NotStored, TheKeyring};
use secret_service::EncryptionType;
use secret_service::blocking::SecretService;

/// A key that is a credential to nothing.
const A_SYNTHETIC_SECRET: &str = "sk-live-LIFETIME-FIXTURE-ONLY-2f70";

/// What this crate files its items under.
const OURS: &str = "xdg:schema";

/// Its value.
const ALO: &str = "dev.alo.Provider";

/// What the reference is filed under.
const REFERRED_TO_AS: &str = "reference";

/// Where the fixture's key lives.
const THE_REFERENCE: &str = "provider/Mistral";

/// Put the synthetic key in, through a client of the fixture's own.
fn stored(keyring: &AKeyringOfOurOwn) {
    let connection = zbus::blocking::connection::Builder::address(keyring.address().as_str())
        .expect("an address")
        .build()
        .expect("a connection");
    let service = SecretService::connect_with_existing(EncryptionType::Dh, connection)
        .expect("the fixture serves");
    let mut attributes = HashMap::new();
    attributes.insert(OURS, ALO);
    attributes.insert(REFERRED_TO_AS, THE_REFERENCE);
    service
        .get_default_collection()
        .expect("a collection")
        .create_item(
            "a provider key this test invented",
            attributes,
            A_SYNTHETIC_SECRET.as_bytes(),
            true,
            "text/plain",
        )
        .expect("an item can be stored");
}

/// A connection of this test's own, kept for the length of the test.
///
/// **Kept**, because counting connections from a connection that is itself made
/// and dropped for every count would measure the measuring.
fn a_watcher(keyring: &AKeyringOfOurOwn) -> zbus::blocking::Connection {
    zbus::blocking::connection::Builder::address(keyring.address().as_str())
        .expect("an address")
        .build()
        .expect("a connection")
}

/// How many client connections the bus currently has.
///
/// A unique name — `:1.7` and the like — is one per connection, so counting them
/// counts connections. This is the bus's own answer rather than ours.
fn connections(watching: &zbus::blocking::Connection) -> usize {
    let names: Vec<String> = watching
        .call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "ListNames",
            &(),
        )
        .expect("a live bus answers ListNames")
        .body()
        .deserialize()
        .expect("ListNames returns names");
    names.iter().filter(|name| name.starts_with(':')).count()
}

/// Wait for the bus to settle at a count, or give up and report what it was.
///
/// A connection closing is the client's socket shutting and the bus noticing,
/// which is two things and not one instant. Polling for a bounded moment is the
/// honest way to ask; asserting immediately would make this a test about
/// scheduling.
fn settles_at(watching: &zbus::blocking::Connection, expected: usize) -> usize {
    let until = Instant::now() + Duration::from_secs(5);
    let mut last = connections(watching);
    while Instant::now() < until && last != expected {
        std::thread::sleep(Duration::from_millis(50));
        last = connections(watching);
    }
    last
}

/// The count once the bus has gone quiet: the same answer for a whole second.
///
/// A beginning is only worth measuring against if it has stopped moving. The
/// fixture's readiness probe closes its own connection as `started` returns,
/// and the bus goes on listing a closed connection until it notices — so a
/// count taken immediately can include one already on its way out, and every
/// later count would be compared against a beginning that no longer exists.
fn once_it_is_quiet(watching: &zbus::blocking::Connection) -> usize {
    let until = Instant::now() + Duration::from_secs(10);
    let mut last = connections(watching);
    let mut unchanged_since = Instant::now();
    while Instant::now() < until {
        std::thread::sleep(Duration::from_millis(50));
        let now = connections(watching);
        if now != last {
            last = now;
            unchanged_since = Instant::now();
        } else if unchanged_since.elapsed() >= Duration::from_secs(1) {
            break;
        }
    }
    last
}

/// **Each keyring is a connection of its own, and giving one up closes it.**
///
/// This is the measurement ADR 0022 asked for, and the answer is **not**
/// libsecret's: there is no shared singleton here. Four keyrings opened at once
/// are four connections on the bus, and dropping them returns the count to
/// where it started — so a `TheKeyring` that goes out of scope really does stop
/// holding a session on the person's bus.
#[test]
fn each_keyring_is_a_connection_of_its_own_and_giving_it_up_closes_it() {
    let fixture = AKeyringOfOurOwn::started("lifetime");
    let watching = a_watcher(&fixture);
    let before = once_it_is_quiet(&watching);

    let opened: Vec<TheKeyring> = (0..4)
        .map(|_| TheKeyring::opened(&fixture.bus()).expect("the keyring opens"))
        .collect();

    assert_eq!(
        settles_at(&watching, before + 4),
        before + 4,
        "four keyrings did not make four connections, so they are sharing one — which is \
         the libsecret behaviour ADR 0022 says must be measured rather than assumed"
    );

    drop(opened);

    assert_eq!(
        settles_at(&watching, before),
        before,
        "giving up every keyring left connections on the bus, so a `TheKeyring` that goes out \
         of scope goes on holding a session"
    );
}

/// **Several retrievals at once each get the key, and none disturbs another.**
///
/// Eight threads, each opening its own keyring and asking for the same
/// reference. The encrypted session is negotiated per connection, so this is
/// also eight handshakes at once against one `gnome-keyring-daemon`.
#[test]
fn keys_can_be_fetched_concurrently_and_every_one_arrives() {
    let fixture = AKeyringOfOurOwn::started("concurrent");
    stored(&fixture);
    let bus = fixture.bus();

    let asking: Vec<std::thread::JoinHandle<Result<(), NotStored>>> = (0..8)
        .map(|_| {
            let bus = bus.clone();
            std::thread::spawn(move || {
                TheKeyring::opened(&bus)?
                    .look_up(&SecretRef::named(THE_REFERENCE))
                    .map(|_| ())
            })
        })
        .collect();

    for (which, asked) in asking.into_iter().enumerate() {
        let got = asked.join().expect("the asking thread finishes");
        assert!(
            got.is_ok(),
            "asking {which} of eight at once was refused with {got:?}, so concurrent retrieval \
             is not safe"
        );
    }
}

/// **A keyring outlives nothing: when the session ends, it refuses.**
///
/// Logout is `logind` taking `/run/user/<uid>` away, and what is held here is a
/// connection to a bus inside it. A keyring opened while somebody was signed in
/// must not go on answering afterwards, and must not hang either — the person
/// is gone and their key is not this daemon's to keep serving.
///
/// The fixture is dropped, which kills the bus and the keyring exactly as
/// ending a session does. What is asserted is that the next lookup **refuses,
/// promptly, and hands back no key.**
#[test]
fn a_keyring_whose_session_ended_refuses_rather_than_answering() {
    let held = {
        let fixture = AKeyringOfOurOwn::started("logout");
        stored(&fixture);
        let held = TheKeyring::opened(&fixture.bus()).expect("the keyring opens while signed in");

        // It works while the session is there, so what follows is the session
        // ending rather than the key never having been reachable.
        held.look_up(&SecretRef::named(THE_REFERENCE))
            .expect("the key is there while somebody is signed in");
        held
    };

    let began = Instant::now();
    let after = held.look_up(&SecretRef::named(THE_REFERENCE));
    let took = began.elapsed();

    assert!(
        after.is_err(),
        "a keyring went on handing over a key after the session it belonged to ended"
    );
    assert!(
        took < Duration::from_secs(10),
        "refusing after logout took {took:?}, which is a daemon that appears to have hung \
         rather than one that has answered"
    );
}
