//! A real Secret Service, asked for a real key over an encrypted session.
//!
//! `which_bus_is_reached.rs` proves the connection goes where this crate sends
//! it. This proves what happens when something is listening: a key stored by
//! one client is read back by ours, and the three refusals that need a real
//! store to exist at all.
//!
//! **Every credential here is synthetic**, made in a keyring this test starts
//! in a temporary directory and kills afterwards. No existing keyring is read,
//! written or unlocked, and nothing is registered as the machine's own.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;

use alo_models::SecretRef;
use alo_secrets::{NotStored, TheKeyring};
use secret_service::EncryptionType;
use secret_service::blocking::SecretService;

mod a_keyring_of_our_own;

use a_keyring_of_our_own::AKeyringOfOurOwn;

/// A key that is a credential to nothing.
const A_SYNTHETIC_SECRET: &str = "sk-live-FIXTURE-ONLY-4b19c7";

/// What alo OS files its items under, matching `crate::store`.
const OURS: &str = "xdg:schema";

/// Its value.
const ALO: &str = "dev.alo.Provider";

/// What the reference is filed under.
const REFERRED_TO_AS: &str = "reference";

/// Put a secret in the fixture's keyring, the way a settings panel eventually
/// will — through a client of its own, so that what this file tests is our
/// *reading* rather than a round trip through one object.
fn stored(keyring: &AKeyringOfOurOwn, reference: &str, secret: &str) {
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
    attributes.insert(OURS, ALO);
    attributes.insert(REFERRED_TO_AS, reference);
    collection
        .create_item(
            "a provider key this test invented",
            attributes,
            secret.as_bytes(),
            true,
            "text/plain",
        )
        .expect("an item can be stored");
}

/// **A key stored in a real keyring is read back over an encrypted session.**
///
/// The session is `EncryptionType::Dh` on both sides — the fixture's client and
/// ours — so what crosses the bus is not the key. That is not a detail this
/// test arranged to make itself easier: `Plain` would have worked and was not
/// used.
#[test]
fn a_key_that_is_there_is_handed_over() {
    let keyring = AKeyringOfOurOwn::started("there");
    stored(&keyring, "provider/Mistral", A_SYNTHETIC_SECRET);

    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens on its own bus");
    let found = ours
        .look_up(&SecretRef::named("provider/Mistral"))
        .expect("the key that was stored is handed over");

    // It is a `Secret`, so there is no accessor to compare against — which is
    // the point of the type. What can be said is that it is not the error path,
    // and that nothing rendered it.
    assert!(
        !format!("{found:?}").contains(A_SYNTHETIC_SECRET),
        "the key rendered itself"
    );
}

/// **A provider with no entry is `Missing`**, not an error and not a guess.
#[test]
fn a_reference_nothing_was_filed_under_is_missing() {
    let keyring = AKeyringOfOurOwn::started("missing");
    stored(&keyring, "provider/Mistral", A_SYNTHETIC_SECRET);

    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens");
    assert_eq!(
        ours.look_up(&SecretRef::named("provider/Nobody")).err(),
        Some(NotStored::Missing),
        "a provider nobody stored a key for was not reported as missing"
    );
}

/// **A key filed under somebody else's schema is not ours to read.**
///
/// The search is on `xdg:schema` as well as the reference, so an item another
/// application stored under the same name is not matched — which is why the
/// schema is in the attributes at all.
#[test]
fn an_item_that_is_not_ours_is_missing_rather_than_borrowed() {
    let keyring = AKeyringOfOurOwn::started("theirs");

    let connection = zbus::blocking::connection::Builder::address(keyring.address().as_str())
        .expect("an address")
        .build()
        .expect("a connection");
    let service = SecretService::connect_with_existing(EncryptionType::Dh, connection)
        .expect("the fixture serves");
    let mut attributes = HashMap::new();
    attributes.insert(OURS, "org.example.SomebodyElse");
    attributes.insert(REFERRED_TO_AS, "provider/Mistral");
    service
        .get_default_collection()
        .expect("a collection")
        .create_item(
            "somebody else's key, under our reference",
            attributes,
            b"not-ours",
            true,
            "text/plain",
        )
        .expect("an item can be stored");

    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens");
    assert_eq!(
        ours.look_up(&SecretRef::named("provider/Mistral")).err(),
        Some(NotStored::Missing),
        "an item filed under another application's schema was read as ours"
    );
}

// **Not here yet: a real bus with no Secret Service on it.**
//
// `bus::tests` proves `Unavailable` for a socket that is absent, is not a
// socket, is not the person's, or could not be said as an address. The case
// this file would add is a *real bus that answers* with no keyring on it — a
// machine whose image ships none.
//
// It is not here because the fixture cannot yet stop only the keyring:
// signalling the child this test started leaves something still serving
// `org.freedesktop.secrets` on the bus, so the test asserted a state it had not
// actually produced. A test whose fixture does not reach the state it names
// proves nothing, and one that passed by accident would be worse.
//
// `TheKeyring::opened` does now ask the service for its collections before
// answering, so a store that cannot answer is refused there rather than at the
// first key somebody wanted — but **that path is unproven** until the fixture
// can produce it. `docs/autonomy/updates/` carries it as open work.
