//! While locked, task 1's rule applies and nothing is shown.
//!
//! `crates/alo-locking` decided that a lock screen shows the time, the lock
//! image, the battery and that the machine is locked — and that a notification
//! arriving while locked is **held**, with no preview variant a setting could
//! turn on. This crate is what finally sends one, so this is where that rule
//! either holds or quietly stops being true.
//!
//! It holds structurally rather than by care: `alo_notifying::deciding::arrives`
//! hands the notification to `alo_locking::Seat::arrives` **before it looks at
//! anything else**, and what comes back on a locked seat is
//! `alo_locking::Arrived::Held` — a value that carries nothing to draw. There
//! is no branch in this crate that could draw one because there is nothing for
//! it to draw.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod common;

use alo_locking::Unlocking;
use alo_notifying::{Because, Missed, Quiet, Why, arriving, deciding};

use common::{
    a_notification_titled, an_open_seat, annas_password, locked, the_accounts, the_agent,
    the_machines_words,
};

/// **A notification arriving at a locked machine is held and nothing comes
/// back to draw** — whoever sent it, and whatever else is or is not holding
/// notifications.
#[test]
fn a_notification_arriving_at_a_locked_machine_is_held() {
    for quiet in [
        Quiet::No,
        Quiet::Yes(Because::YouAskedForQuiet),
        Quiet::Yes(Because::TheScreenIsShared),
    ] {
        let mut seat = locked(an_open_seat());
        let mut missed = Missed::nothing();
        for notification in [
            a_notification_titled("Anna Pärt"),
            arriving::from_the_agent(&the_agent(), "The invoices are filed", "", &[]).unwrap(),
            arriving::from_alo_os("An update is ready", "", &[]).unwrap(),
        ] {
            let became = deciding::arrives(notification, &mut seat, quiet, &mut missed);
            assert!(!became.is_shown(), "{quiet:?}");
            assert_eq!(became.shown(), None, "{quiet:?} handed something to draw");
            assert_eq!(became.why(), Some(Why::TheMachineIsLocked));
        }
    }
}

/// **The lock is asked before do-not-disturb**, so what a locked machine says
/// is the one sentence the lock screen already says rather than a second one
/// about the person's settings.
#[test]
fn a_locked_machine_says_the_one_sentence_a_lock_screen_says() {
    let strings = the_machines_words();
    let mut seat = locked(an_open_seat());
    let mut missed = Missed::nothing();
    let became = deciding::arrives(
        a_notification_titled("Anna Pärt — the March invoices"),
        &mut seat,
        Quiet::Yes(Because::YouAskedForQuiet),
        &mut missed,
    );
    let said = became.why().unwrap().said(&strings);
    assert!(!said.is_a_bug());
    assert!(said.unfilled().is_empty(), "{said}");
    assert!(!said.text().contains("Anna"), "{said}");
    assert!(!said.text().contains("invoice"), "{said}");
}

/// **What the lock screen held is handed back at the unlock, in the order it
/// arrived, and only then goes on the list of what was missed.**
///
/// It is task 1's to hold and task 1's to hand over: this crate never reaches
/// into a locked seat, and `alo_locking::Unlocking::Unlocked` is the one road
/// out — which the person reached by proving who they are.
#[test]
fn what_was_held_behind_the_lock_comes_back_at_the_unlock() {
    let mut seat = locked(an_open_seat());
    let mut missed = Missed::nothing();
    for title in ["First", "Second", "Third"] {
        deciding::arrives(
            a_notification_titled(title),
            &mut seat,
            Quiet::No,
            &mut missed,
        );
    }
    assert!(
        missed.is_empty(),
        "what a lock screen holds is not this crate's until the unlock"
    );

    let Unlocking::Unlocked { held, .. } = seat.unlocks(the_accounts(), "anna", annas_password())
    else {
        panic!("the right password did not unlock");
    };
    missed.at_the_unlock(held);
    let titles: Vec<&str> = missed
        .waiting()
        .iter()
        .map(|waiting| waiting.notification().title())
        .collect();
    assert_eq!(titles, ["First", "Second", "Third"]);
}

/// **A machine that stays locked keeps holding them.** A wrong password is not
/// a road to what arrived, and the count on the missed list does not move.
#[test]
fn a_wrong_password_is_not_a_road_to_what_arrived() {
    let mut seat = locked(an_open_seat());
    let mut missed = Missed::nothing();
    deciding::arrives(
        a_notification_titled("Anna Pärt"),
        &mut seat,
        Quiet::No,
        &mut missed,
    );

    let attempt = seat.unlocks(the_accounts(), "anna", "not it");
    assert!(
        matches!(attempt, Unlocking::StillLocked { .. }),
        "a wrong password unlocked"
    );
    assert!(attempt.seat().is_locked());
    assert!(missed.is_empty(), "a refused unlock handed something over");
}
