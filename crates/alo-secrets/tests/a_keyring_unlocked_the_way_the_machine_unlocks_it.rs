//! A key goes in and comes back out of a keyring unlocked with **no password**,
//! which is the configuration the image actually ships.
//!
//! The image's unit sends one newline on the daemon's standard input, so its
//! login keyring has a password of no characters — argued in
//! `image/usr/lib/systemd/system/alo-keyring.service`, resting on ADR 0054
//! encrypting the disk underneath it. Since 2026-10-22
//! `AKeyringOfOurOwn::started` feeds exactly that, so **every** keyring test in
//! this repository runs the shipping configuration; this is the file that names
//! it as its subject rather than inheriting it.
//!
//! # Why this is a file rather than one more case in the others
//!
//! **The difference between those two stdin bytes shipped as a bug** on
//! 2026-10-22, and nothing here could see it. The unit said
//! `StandardInput=null`, which is end of file and not an empty password. The
//! daemon started, systemd called the unit `active`, it took
//! `org.freedesktop.secrets` on the bus — and created no collection, so
//! `get_default_collection` resolved to `/` and every key a person tried to
//! keep died on *object does not exist at path …/collection/login*.
//!
//! Three checks were true and none was the thing: `--version` loads the binary,
//! a `grep` finds the activation file, `systemctl` says `active`. And the suite
//! could not help, because **the fixture diverged from the shipped unit in
//! exactly the setting that was broken.** So the subject here is not *does a
//! keyring work*, which is covered — it is *does a keyring unlocked the way
//! this machine unlocks it work*.
//!
//! **What this does not prove.** It does not start the unit, so it says nothing
//! about `StandardInputData=Cg==` being what delivers that byte, nor about the
//! ordering against `user@1000.service`. Those were measured by hand under the
//! image's own systemd 257, and
//! `docs/misreadings/a-service-that-starts-and-serves-nothing.md` records that
//! round trip and says where it does and does not run. This file is the part
//! that runs on every build.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_models::{Secret, SecretRef};
use alo_secrets::{NotStored, TheKeyring};

/// A key that is a credential to nothing.
const A_SYNTHETIC_SECRET: &str = "sk-live-FIXTURE-ONLY-9c2f41";

/// **A key a person added is kept and handed back, with no password on the
/// keyring** — the whole of what Settings needs from the store, against the
/// configuration the image boots with.
///
/// Both halves through [`TheKeyring`] on purpose: a key filed under a name our
/// own lookup does not search for passes every test that stores with one client
/// and reads with another.
#[test]
fn a_key_is_kept_and_handed_back_with_no_password_on_the_keyring() {
    let keyring = AKeyringOfOurOwn::started("no-password");
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens on its own bus");
    let named = SecretRef::named("provider/OpenAI");

    ours.keep(
        &named,
        &Secret::typed(A_SYNTHETIC_SECRET).expect("a synthetic key is one this machine can send"),
    )
    .expect("a key is kept in a keyring unlocked with no password");

    let found = ours
        .look_up(&named)
        .expect("the key that was just kept is handed back");

    // It is a `Secret`, so there is no accessor to compare against — which is
    // the point of the type. What can be said is that it is not the error path
    // and that nothing rendered it.
    assert!(
        !format!("{found:?}").contains(A_SYNTHETIC_SECRET),
        "the key rendered itself"
    );
}

/// **And a name nothing was filed under is still `Missing`**, so the test above
/// cannot pass against a store that answers everything.
///
/// Without this, a keyring handing something back for every reference would
/// satisfy the first test exactly as a correct one does.
#[test]
fn a_name_nothing_was_filed_under_is_still_missing() {
    let keyring = AKeyringOfOurOwn::started("no-password-missing");
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens on its own bus");
    ours.keep(
        &SecretRef::named("provider/OpenAI"),
        &Secret::typed(A_SYNTHETIC_SECRET).expect("a synthetic key is one this machine can send"),
    )
    .expect("a key is kept");

    assert!(
        matches!(
            ours.look_up(&SecretRef::named("provider/NobodyAdded")),
            Err(NotStored::Missing)
        ),
        "a provider nobody added came back with something"
    );
}
