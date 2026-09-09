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
// `clippy::panic` is deliberately not expected here any more: the `panic!`s
// were the fixture's, and the fixture is `alo-keyring-fixture` now.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;

use alo_models::SecretRef;
use alo_secrets::{NotStored, TheKeyring};
use secret_service::EncryptionType;
use secret_service::blocking::SecretService;

use alo_keyring_fixture::AKeyringOfOurOwn;

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

/// **A real bus with no Secret Service on it is `Unavailable`.**
///
/// This is the machine whose image ships no keyring, and it is the case
/// `TheKeyring::opened` asks `get_all_collections` for: building the client is
/// a proxy and a session handshake, which can succeed against a name nobody
/// owns, so a store that cannot answer must be refused at opening rather than
/// at the first key somebody wanted.
///
/// **The control matters more than the assertion here.** `bus::tests` already
/// proves `Unavailable` for a socket that is absent, is not a socket, is not the
/// person's, or cannot be said as an address. Without proving the bus is up and
/// answering, this would be a fifth spelling of those and would pass just as
/// well against a dead socket — so the bus is asked something first, and asked
/// whether anybody owns the name.
///
/// It reaches the state by **not starting the keyring**, rather than by killing
/// one. An earlier version tried to stop the daemon and something went on
/// serving the name, so the test asserted a state it had not produced; it was
/// withdrawn rather than left passing for the wrong reason.
#[test]
fn a_real_bus_with_no_secret_service_on_it_is_unavailable() {
    let bare = AKeyringOfOurOwn::a_bus_with_no_keyring_on_it("bare");

    // Control one: the bus is alive and answers its own calls.
    let connection = zbus::blocking::connection::Builder::address(bare.address().as_str())
        .expect("the fixture's address is an address")
        .build()
        .expect("the fixture's bus accepts a connection");
    let names: Vec<String> = connection
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

    // Control two: and nothing on it is the Secret Service.
    assert!(
        !names.iter().any(|name| name == "org.freedesktop.secrets"),
        "something owns org.freedesktop.secrets on a bus that should have \
         nothing on it, so this test would not prove what it says: {names:?}"
    );

    let began = std::time::Instant::now();
    let answered = TheKeyring::opened(&bare.bus()).err();
    let took = began.elapsed();

    assert_eq!(
        answered,
        Some(NotStored::Unavailable),
        "a live bus with no Secret Service on it was not reported as unavailable"
    );

    // **And it says so promptly**, which is a promise to a person and not a
    // preference about test runtime. A proxy can be built for a name nobody
    // owns and every call then waits out the D-Bus method timeout: this took
    // **125 seconds** before `store.rs` asked the bus who owns the name, which
    // on a machine shipping no keyring is a daemon that appears to have hung.
    // The bound is loose because it guards against the timeout coming back, not
    // against a slow machine.
    assert!(
        took < std::time::Duration::from_secs(10),
        "opening a bus with no Secret Service took {took:?}, which means it is \
         waiting out a method timeout again rather than asking who owns the name"
    );
}
