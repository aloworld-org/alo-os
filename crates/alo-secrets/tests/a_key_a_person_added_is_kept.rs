//! **A key a person added is kept, and this machine finds it again.**
//!
//! The other half of `a_real_keyring_answers.rs`, which proves reading. Until
//! 2026-10-22 there was no writing half at all: `TheKeyring` could `look_up` a
//! key and had no way to put one there, so every test of the reading side had
//! to store its key through a client of its own — and a person adding a
//! provider on their own machine had nowhere for its key to go.
//!
//! # Read back through a client of its own
//!
//! Every assertion below reads through a **separate** Secret Service client,
//! never through `TheKeyring`. A test that stored and read through one object
//! would pass just as well if both halves agreed on the wrong attributes, which
//! is the one way this can appear to work and leave a key the machine cannot
//! find — exactly what `look_up` would then report as `Missing`, sending a
//! person to add a key they already added.
//!
//! The same reason `a_real_keyring_answers.rs` gives for storing through its
//! own client, pointed the other way.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;

use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_models::{Secret, SecretRef};
use alo_secrets::{NotStored, TheKeyring};
use secret_service::EncryptionType;
use secret_service::blocking::SecretService;

/// What alo OS files its provider keys under. Spelled here on purpose: a test
/// that imported the constant would agree with the code by construction and
/// could not catch it changing.
const OURS: &str = "xdg:schema";
/// The schema value, spelled for the same reason.
const ALO: &str = "dev.alo.Provider";
/// What the reference is filed under, spelled for the same reason.
const REFERRED_TO_AS: &str = "reference";

/// A key that is obviously not a real one, and would be recognisable in any
/// output that leaked it.
const A_SYNTHETIC_SECRET: &str = "sk-live-DO-NOT-LOG-9f3c1a";

/// A client of the fixture's own, separate from the one under test.
fn a_client(keyring: &AKeyringOfOurOwn) -> SecretService<'static> {
    let connection = zbus::blocking::connection::Builder::address(keyring.address().as_str())
        .expect("the fixture's address is an address")
        .build()
        .expect("the fixture's bus accepts a connection");
    SecretService::connect_with_existing(EncryptionType::Dh, connection)
        .expect("the fixture serves the Secret Service")
}

/// What is filed under this reference, read through a client of its own.
///
/// `None` when nothing is, which is what makes the control at the other end of
/// each test mean something.
fn what_is_filed_under(keyring: &AKeyringOfOurOwn, reference: &str) -> Option<String> {
    let service = a_client(keyring);
    let mut asked = HashMap::new();
    asked.insert(OURS, ALO);
    asked.insert(REFERRED_TO_AS, reference);
    let found = service.search_items(asked).unwrap();
    let item = found.unlocked.first()?;
    Some(String::from_utf8(item.get_secret().unwrap()).unwrap())
}

#[test]
fn a_key_that_was_kept_is_filed_where_this_machine_looks_for_it() {
    let keyring = AKeyringOfOurOwn::started("kept");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();
    let named = SecretRef::named("provider/OpenAI");

    // The control at the start: nothing is there before, so what is found
    // afterwards was put there by the call under test and not by the fixture.
    assert_eq!(what_is_filed_under(&keyring, "provider/OpenAI"), None);

    ours.keep(&named, &Secret::typed(A_SYNTHETIC_SECRET).unwrap())
        .expect("a key is kept in a keyring that is open");

    assert_eq!(
        what_is_filed_under(&keyring, "provider/OpenAI").as_deref(),
        Some(A_SYNTHETIC_SECRET),
        "filed under the attributes look_up searches by -- a key filed anywhere else is a key \
         this machine reports as Missing, which sends a person to add one they already added"
    );
}

#[test]
fn what_was_kept_is_what_look_up_hands_back() {
    // The round trip, which is the claim a person cares about: they paste a
    // key, and later the machine can ask a provider with it.
    let keyring = AKeyringOfOurOwn::started("round-trip");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();
    let named = SecretRef::named("provider/OpenAI");

    assert!(
        matches!(ours.look_up(&named), Err(NotStored::Missing)),
        "the control: nothing is filed under it yet"
    );

    ours.keep(&named, &Secret::typed(A_SYNTHETIC_SECRET).unwrap())
        .unwrap();

    let found = ours
        .look_up(&named)
        .expect("the key that was kept is found");
    // `Secret` has no accessor, which is the point of the type. What can be
    // said here is that it is not the error path and that nothing rendered it;
    // the bytes themselves are compared above, through the other client.
    assert!(
        !format!("{found:?}").contains(A_SYNTHETIC_SECRET),
        "the key rendered itself"
    );
}

#[test]
fn a_second_key_under_one_reference_replaces_the_first() {
    // Two items under one reference would make `look_up` answer with whichever
    // the service returned first: a machine that works today and stops
    // tomorrow for no reason the person could see.
    let keyring = AKeyringOfOurOwn::started("replaced");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();
    let named = SecretRef::named("provider/OpenAI");

    ours.keep(&named, &Secret::typed("sk-the-first-one").unwrap())
        .unwrap();
    ours.keep(&named, &Secret::typed("sk-the-second-one").unwrap())
        .unwrap();

    assert_eq!(
        what_is_filed_under(&keyring, "provider/OpenAI").as_deref(),
        Some("sk-the-second-one"),
        "the key they pasted last is the key the machine has"
    );

    let service = a_client(&keyring);
    let mut asked = HashMap::new();
    asked.insert(OURS, ALO);
    asked.insert(REFERRED_TO_AS, "provider/OpenAI");
    let found = service.search_items(asked).unwrap();
    assert_eq!(
        found.unlocked.len(),
        1,
        "and there is one item, not two: the first was replaced rather than added beside"
    );
}

#[test]
fn two_providers_keep_their_own_keys() {
    // The control that the reference is actually used: two keys kept on one
    // machine must not become one.
    let keyring = AKeyringOfOurOwn::started("two");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();

    ours.keep(
        &SecretRef::named("provider/OpenAI"),
        &Secret::typed("sk-open").unwrap(),
    )
    .unwrap();
    ours.keep(
        &SecretRef::named("provider/Mistral"),
        &Secret::typed("sk-mistral").unwrap(),
    )
    .unwrap();

    assert_eq!(
        what_is_filed_under(&keyring, "provider/OpenAI").as_deref(),
        Some("sk-open")
    );
    assert_eq!(
        what_is_filed_under(&keyring, "provider/Mistral").as_deref(),
        Some("sk-mistral")
    );
}

#[test]
fn a_locked_keyring_is_refused_rather_than_prompting_the_person() {
    // Unlocking is a person answering a prompt, not a daemon deciding for
    // them — which is the reading side's own rule, and a settings screen that
    // triggered a prompt mid-save would be the daemon deciding.
    let keyring = AKeyringOfOurOwn::started("locked");
    let ours = TheKeyring::opened(&keyring.bus()).unwrap();

    // The control first: it writes while unlocked, so the refusal below is the
    // lock and not something else about this keyring.
    ours.keep(
        &SecretRef::named("provider/OpenAI"),
        &Secret::typed(A_SYNTHETIC_SECRET).unwrap(),
    )
    .expect("an unlocked keyring takes a key");

    a_client(&keyring)
        .get_default_collection()
        .unwrap()
        .lock()
        .expect("the fixture's collection can be locked");

    assert!(
        matches!(
            ours.keep(
                &SecretRef::named("provider/Mistral"),
                &Secret::typed("sk-mistral").unwrap()
            ),
            Err(NotStored::Locked)
        ),
        "a locked keyring is told to the caller, not worked around"
    );
}
