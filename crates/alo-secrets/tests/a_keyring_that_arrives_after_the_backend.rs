//! A portal backend that started before the keyring still answers from it.
//!
//! # The machine this is about
//!
//! The session bus, `alo-portald` and the keyring are three user units, and
//! **nothing makes the backend wait for the keyring.** Either may win. Until
//! this test, only the order where the keyring wins was ever exercised, and
//! the other one was broken: `alo-portald` opened the keyring as it started
//! and, finding none, installed `NoKeyringHere` for the life of the process.
//! Every application that later asked for its secret was told *this machine
//! has no keyring* — on a machine that had one, by then, and would until it
//! was switched off.
//!
//! # What is measured, and the control at each end
//!
//! [`alo_secrets::TheKeyringWhenItIsAsked`] is made against a bus with **no**
//! Secret Service on it, which is the losing order exactly. Then:
//!
//! - **the near control** — asked straight away, it answers
//!   [`NotKept::Unavailable`] and writes nothing. Without this the test could
//!   pass on a machine where a keyring was there all along, and would be
//!   measuring nothing.
//! - **the measurement** — a keyring arrives, and the *same object*, built
//!   when there was none, hands a secret over.
//!
//! Both ends matter. One alone is a half this cannot tell apart from a fixture
//! that never changed state.
//!
//! **Shown to fail against the bug before it was trusted.** Against a keyring
//! fixed at construction the first assertion passes and the second cannot: the
//! object has already decided, and `a_keyring_arrives` changes nothing it will
//! look at again.
//!
//! # Why this runs forever rather than once
//!
//! This is a test in the suite, not a reading taken by hand — the order it
//! guards is a race, and a race that is right today is not a property. It runs
//! on every build of this crate.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_portals::{KeepsSecrets, NotKept, Portal, Request};
use alo_secrets::{PORTAL_SECRET_LENGTH, TheKeyringWhenItIsAsked};

/// The application asking for its own secret.
const NOTES: &str = "org.example.Notes";

/// One fixed moment, so a grant's life is arithmetic rather than the clock.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A grant to an application, made the way a person's is.
fn granted(application: &str, facility: Facility) -> Grant {
    Grant::checked_for(
        &Applicant::named(application).grantee(),
        Reach::Facility(facility),
        noon(),
        Duration::from_secs(60 * 60),
    )
    .expect("an hour from noon is a grant")
}

/// The machine's grants, with this application allowed its secrets.
fn the_grants() -> Grants {
    let mut grants = Grants::default();
    grants.grant(granted(NOTES, Facility::Secrets));
    grants
}

/// A request to the Secret portal, as the backend would make one.
fn asking() -> Request {
    Request::of(NOTES, Portal::Secret).expect("the Secret portal takes an application")
}

#[test]
fn a_keyring_that_was_not_there_when_this_was_made_answers_once_it_is() {
    let mut machine = AKeyringOfOurOwn::a_bus_with_no_keyring_on_it("arrives-after-the-backend");
    // Made in the losing order: the bus is up, the keyring is not.
    let keyring = TheKeyringWhenItIsAsked::on(machine.bus());
    let grants = the_grants();

    // The near control. Nothing is there, so nothing is handed over — and
    // nothing is written into what the application gave us either, because a
    // refusal that had written bytes first would be a different fault wearing
    // this one's answer.
    let mut nothing = Vec::new();
    assert_eq!(
        keyring.hand_over(&asking(), &grants, noon(), &mut nothing),
        Err(NotKept::Unavailable),
        "a bus with no Secret Service on it has no secret to hand over"
    );
    assert!(
        nothing.is_empty(),
        "a refusal wrote {} bytes into the application's own file",
        nothing.len()
    );

    // The keyring comes up, as a user unit does, after the backend.
    machine.a_keyring_arrives();

    // The measurement: the same object, which decided nothing when it was made.
    let mut secret = Vec::new();
    let allowed = keyring
        .hand_over(&asking(), &grants, noon(), &mut secret)
        .expect(
            "the keyring is on the bus now. An object that asks when it is asked answers from \
             the machine as it is, not as it was when this service happened to start",
        );
    assert_eq!(allowed.application(), &Applicant::named(NOTES));
    assert_eq!(allowed.portal(), Portal::Secret);
    assert_eq!(
        secret.len(),
        PORTAL_SECRET_LENGTH,
        "the application was handed a secret of the length the portal contract names"
    );
}

#[test]
fn asking_twice_on_a_machine_with_a_keyring_answers_twice() {
    // The far control. The fault above would also be hidden by an object that
    // works exactly once, so the ordinary order is exercised twice over — and
    // each answer is its own secret's worth of bytes rather than a second
    // reply to the first question.
    let machine = AKeyringOfOurOwn::started("asked-more-than-once");
    let keyring = TheKeyringWhenItIsAsked::on(machine.bus());
    let grants = the_grants();

    for time in 1..=2 {
        let mut secret = Vec::new();
        keyring
            .hand_over(&asking(), &grants, noon(), &mut secret)
            .unwrap_or_else(|why| panic!("ask {time} of 2 was refused: {why:?}"));
        assert_eq!(secret.len(), PORTAL_SECRET_LENGTH, "ask {time} of 2");
    }
}
