//! The three promises the lane B plan names for the local account, each
//! measured rather than stated.
//!
//! 1. An account is **created and authenticated against the machine's own
//!    store** — the store on the disk, through a keep and a find, not a value
//!    that never left memory.
//! 2. A wrong password is **refused in words** and is **not distinguishable
//!    by timing from an unknown name** — the words checked as words, and the
//!    timing measured against the store's own work, because the promise is
//!    about what an attacker at the sign-in surface can learn.
//! 3. The session **carries the uid `alo-agentd` is told about** — compared
//!    against the description the image really ships, read by the reader the
//!    image is already held to, so this test and the machine cannot drift
//!    apart on a copied number.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, Instant};

use alo_accounts::{Accounts, NotSignedIn, Session};

/// The password every test signs in with.
const HERS: &str = "correct horse battery staple";

/// A store with one account: the login the image itself declares, at the
/// image's own number.
fn a_machine_with_one_account(uid: u32) -> Accounts {
    let mut store = Accounts::none().unwrap();
    store.created("alo", uid, HERS).unwrap();
    store
}

/// **Criterion 1: created, kept on the disk, found again, and signed in.**
/// The machine's own store is a file the machine keeps, so the round trip
/// through the disk is the claim — a store that only ever lived in memory
/// would not survive the reboot between creating an account and using it.
#[cfg(unix)]
#[test]
fn an_account_is_created_and_signs_in_against_the_machines_own_store() {
    let folder = std::env::temp_dir().join(format!("alo-accounts-signs-in-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    let at = folder.join("accounts.toml");

    alo_accounts::kept(&at, &a_machine_with_one_account(1000)).unwrap();

    let store = alo_accounts::found(&at).unwrap();
    let signed_in = store.signs_in("alo", HERS).unwrap();
    assert_eq!(signed_in.name(), "alo");
    assert_eq!(signed_in.uid(), 1000);

    // And the refusal path on the same store from the same disk: the file
    // holds a hash, never the password, so a wrong one is refused after the
    // round trip too.
    assert_eq!(
        store.signs_in("alo", "not her password"),
        Err(NotSignedIn::Refused)
    );
}

/// **Criterion 2: one refusal, in words and in time.**
///
/// The refusal is one value with one sentence for a wrong password and an
/// unknown name; and the unknown name is measured to cost what the wrong
/// password costs. The failure this guards against is not a small skew — it
/// is the obvious implementation, which answers an unknown name without
/// running the hash at all and is therefore *hundreds* of times faster. The
/// medians here may differ by a factor of three before this fails, which is
/// noise-proof on a loaded machine and still catches that failure by orders
/// of magnitude.
#[test]
fn a_wrong_password_and_an_unknown_name_are_one_refusal_in_words_and_in_time() {
    let store = a_machine_with_one_account(1000);

    // In shape: the same value, nothing on it to compare.
    let wrong = store.signs_in("alo", "not her password").unwrap_err();
    let unknown = store.signs_in("nobody", HERS).unwrap_err();
    assert_eq!(wrong, unknown);
    assert_eq!(wrong, NotSignedIn::Refused);

    // In words: one sentence, and it names neither the account tried nor
    // which of the two facts refused it.
    let strings = alo_strings::Strings::of(alo_accounts::accounts_words().unwrap());
    let said_of_wrong = wrong.said(&strings);
    let said_of_unknown = unknown.said(&strings);
    assert_eq!(said_of_wrong.text(), said_of_unknown.text());
    assert!(!said_of_wrong.is_a_bug(), "{said_of_wrong}");
    assert!(said_of_wrong.text().contains("name and password"));
    assert!(!said_of_wrong.text().contains("alo"));
    assert!(!said_of_unknown.text().contains("nobody"));

    // In time: interleaved so drift hits both alike, medians so one
    // scheduler hiccup decides nothing.
    let mut wrong_took = Vec::new();
    let mut unknown_took = Vec::new();
    for _ in 0..9 {
        wrong_took.push(refused_in(&store, "alo", "not her password"));
        unknown_took.push(refused_in(&store, "nobody", HERS));
    }
    let wrong_median = median(wrong_took);
    let unknown_median = median(unknown_took);
    assert!(
        unknown_median * 3 >= wrong_median,
        "an unknown name ({unknown_median:?}) answers faster than a wrong password \
         ({wrong_median:?}), so names can be enumerated by timing"
    );
    assert!(
        wrong_median * 3 >= unknown_median,
        "a wrong password ({wrong_median:?}) answers faster than an unknown name \
         ({unknown_median:?}), so names can be enumerated by timing"
    );
}

/// **Criterion 3: the session carries the uid `alo-agentd` is told about.**
///
/// The number is read from the machine description this repository actually
/// ships (`image/etc/alo/agentd.toml`), through `alo-image`'s reader — the
/// same file the daemon is told, so the description and the session cannot
/// disagree about who is signed in. And the refusal beside it: an account at
/// any other number opens no session at all.
#[test]
fn the_session_carries_the_uid_the_daemon_is_told_about() {
    let image = alo_image::Image::at(std::path::Path::new(alo_image::THE_IMAGE)).unwrap();
    let person = image.description().person();

    let store = a_machine_with_one_account(person);
    let signed_in = store.signs_in("alo", HERS).unwrap();
    let session = Session::opened(signed_in, person).unwrap();
    assert_eq!(
        session.uid(),
        person,
        "the session and the shipped machine description disagree about who is signed in"
    );

    // The refusal path: right password, wrong number — authenticated, and
    // still no session, with both numbers kept for whoever reconciles them.
    let elsewhere = a_machine_with_one_account(person + 1);
    let signed_in_elsewhere = elsewhere.signs_in("alo", HERS).unwrap();
    assert_eq!(
        Session::opened(signed_in_elsewhere, person),
        Err(NotSignedIn::NotThePerson {
            signed_in: person + 1,
            described: person,
        })
    );
}

/// One refusal, timed — and asserted to be the indistinguishable one, so a
/// measurement of some other failure cannot pass as this measurement.
fn refused_in(store: &Accounts, name: &str, password: &str) -> Duration {
    let began = Instant::now();
    let refused = store.signs_in(name, password);
    let took = began.elapsed();
    assert_eq!(refused, Err(NotSignedIn::Refused));
    took
}

/// The middle value, which one scheduler hiccup cannot move.
fn median(mut took: Vec<Duration>) -> Duration {
    took.sort_unstable();
    took.get(took.len() / 2).copied().unwrap()
}
