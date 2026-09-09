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

/// A client of the fixture's own, separate from the one under test.
///
/// Everything this file arranges — storing, locking — goes through this rather
/// than through [`TheKeyring`], so that a test never proves itself by driving
/// the same object it is checking.
fn a_client(keyring: &AKeyringOfOurOwn) -> SecretService<'static> {
    let connection = zbus::blocking::connection::Builder::address(keyring.address().as_str())
        .expect("the fixture's address is an address")
        .build()
        .expect("the fixture's bus accepts a connection");
    SecretService::connect_with_existing(EncryptionType::Dh, connection)
        .expect("the fixture serves the Secret Service")
}

/// Put a secret in the fixture's keyring, the way a settings panel eventually
/// will — through a client of its own, so that what this file tests is our
/// *reading* rather than a round trip through one object.
fn stored(keyring: &AKeyringOfOurOwn, reference: &str, secret: &str) {
    let service = a_client(keyring);
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

    let service = a_client(&keyring);
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

/// **A key behind a locked collection is `Locked`, not `Missing`.**
///
/// The distinction is the whole point of having four refusals: `Missing` sends
/// a person to add a key they already have, while `Locked` sends them to
/// unlock. The collection here is locked by a real `gnome-keyring-daemon`
/// through a client of its own — nothing about the state is simulated.
#[test]
fn a_key_behind_a_locked_collection_is_locked_rather_than_missing() {
    let keyring = AKeyringOfOurOwn::started("locked");
    stored(&keyring, "provider/Mistral", A_SYNTHETIC_SECRET);

    let theirs = a_client(&keyring);
    let collection = theirs
        .get_default_collection()
        .expect("the fixture has a default collection");
    collection.lock().expect("the collection can be locked");
    assert!(
        collection.is_locked().expect("the collection answers"),
        "the fixture did not actually lock the collection, so this test would \
         have proved nothing"
    );

    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens");
    assert_eq!(
        ours.look_up(&SecretRef::named("provider/Mistral")).err(),
        Some(NotStored::Locked),
        "a key that is present but locked was not reported as locked"
    );
}

/// **Looking a key up never unlocks anything.**
///
/// `store.rs` says unlocking is a person answering a prompt and not a daemon
/// deciding for them. This is that sentence as a check: after a refused lookup
/// the collection is still locked, so nothing on our side quietly opened it to
/// get an answer.
#[test]
fn a_refused_lookup_leaves_the_collection_locked() {
    let keyring = AKeyringOfOurOwn::started("stays-locked");
    stored(&keyring, "provider/Mistral", A_SYNTHETIC_SECRET);

    let theirs = a_client(&keyring);
    let collection = theirs
        .get_default_collection()
        .expect("the fixture has a default collection");
    collection.lock().expect("the collection can be locked");

    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens");
    let refused = ours.look_up(&SecretRef::named("provider/Mistral"));
    assert!(refused.is_err(), "the locked key was handed over");

    assert!(
        collection.is_locked().expect("the collection answers"),
        "looking a key up unlocked the collection"
    );
}

/// **A caller the bus refuses is `Denied`, not `Unavailable`.**
///
/// The service is running and holding the key throughout; what changes is that
/// the bus stops carrying messages to it. That is what a refusal is, and it is
/// a different thing for a person to do about than an absent store: `Denied`
/// sends them to find out why, `Unavailable` sends them looking for a service
/// that was there the whole time.
///
/// **This is a real refusal, not an injected transport failure.** Nothing is
/// mocked, no error is constructed: a `<deny>` rule is added to the running
/// bus's policy and reloaded, and the denial comes back from `dbus-daemon`.
#[test]
fn a_caller_the_bus_refuses_is_denied_rather_than_unavailable() {
    let keyring = AKeyringOfOurOwn::started_where_the_bus_can_refuse("denied");
    stored(&keyring, "provider/Mistral", A_SYNTHETIC_SECRET);

    // Opened while still allowed, so what the refusal interrupts is the lookup
    // and not the connection.
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens while allowed");

    keyring.stop_letting_anyone_reach_the_keyring();

    assert_eq!(
        ours.look_up(&SecretRef::named("provider/Mistral")).err(),
        Some(NotStored::Denied),
        "a caller the bus refused was not reported as denied"
    );
}

/// **A refusal sends nothing to a provider, and discloses nothing.**
///
/// [`NotStored::nothing_was_sent`] is the claim the type exists to make. This
/// holds it up against the one refusal that could plausibly have carried
/// something: the store had the key, and said no.
#[test]
fn a_denied_lookup_carries_neither_the_key_nor_what_the_bus_said() {
    let keyring = AKeyringOfOurOwn::started_where_the_bus_can_refuse("denied-quietly");
    stored(&keyring, "provider/Mistral", A_SYNTHETIC_SECRET);
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens while allowed");
    keyring.stop_letting_anyone_reach_the_keyring();

    let refused = ours
        .look_up(&SecretRef::named("provider/Mistral"))
        .expect_err("the key was handed over by a bus that refused the message");

    assert!(refused.nothing_was_sent(), "a refusal reached a provider");

    // The refusal is four states and no prose. Neither the synthetic key nor
    // the bus's own message may be recoverable from it.
    let rendered = format!("{refused:?}");
    assert!(
        !rendered.contains(A_SYNTHETIC_SECRET),
        "the refusal rendered the key"
    );
    assert!(
        !rendered.to_ascii_lowercase().contains("denied by")
            && !rendered.contains("org.freedesktop"),
        "the refusal repeated what the bus said: {rendered}"
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
