//! Do-not-disturb holds **every** notification and shows none — and it turns
//! itself on while the screen is being read.
//!
//! The plan's clause is *do-not-disturb holds every notification and shows
//! none, turned on by a person or by a schedule, and automatically while the
//! screen is shared or recorded.* Each half of it is easy to get almost right:
//! a rule that showed *important* ones anyway, a rule that only looked at the
//! person's own setting, a rule that asked the screen after the settings. So
//! this walks a day's worth of notifications through each way of being quiet
//! and finds none of them shown.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod common;

use alo_appearance::TimeOfDay;
use alo_notifying::{Because, Missed, Quiet, QuietHours, Settings, Why, arriving, deciding, words};

use common::{
    a_notification_titled, an_open_seat, nothing_is_in_use, the_agent, the_camera_is_on,
    the_machines_words, the_screen_is_being_shared,
};

/// A time on the clock the person is looking at.
fn at(hour: u8, minute: u8) -> TimeOfDay {
    TimeOfDay::checked(hour, minute).unwrap()
}

/// A desk's worth of notifications: an application's, the agent's own, and one
/// alo OS sent about the machine.
fn a_days_notifications() -> Vec<alo_notifying::Notification> {
    vec![
        a_notification_titled("Anna Pärt"),
        a_notification_titled("Bertrand"),
        arriving::from_the_agent(&the_agent(), "The invoices are filed", "", &[]).unwrap(),
        arriving::from_alo_os("An update is ready", "", &[]).unwrap(),
    ]
}

/// **Every notification is held and none is shown**, whichever way the machine
/// is quiet — including the agent's own and alo OS's own, which get no
/// exemption for being ours.
#[test]
fn every_notification_is_held_and_none_is_shown() {
    for because in Because::EVERY {
        let mut seat = an_open_seat();
        let mut missed = Missed::nothing();
        for notification in a_days_notifications() {
            let became =
                deciding::arrives(notification, &mut seat, Quiet::Yes(because), &mut missed);
            assert!(!became.is_shown(), "{because:?} showed one");
            assert_eq!(became.shown(), None, "{because:?} handed something to draw");
            assert_eq!(became.why(), Some(Why::Quiet(because)));
        }
        assert_eq!(missed.how_many(), 4, "{because:?} lost one");
    }
}

/// **A person turning it on holds everything**, and the same machine with it
/// off shows everything — so the setting is doing the work rather than
/// something else.
#[test]
fn a_person_turning_it_on_holds_everything_and_turning_it_off_shows_everything() {
    let quiet_room = nothing_is_in_use();
    let mut on = Settings::shipped();
    on.do_not_disturb = true;
    assert_eq!(
        Quiet::now(&on, Some(&quiet_room), at(15, 0)),
        Quiet::Yes(Because::YouAskedForQuiet)
    );
    assert_eq!(
        Quiet::now(&Settings::shipped(), Some(&quiet_room), at(15, 0)),
        Quiet::No
    );

    let mut seat = an_open_seat();
    let mut missed = Missed::nothing();
    for notification in a_days_notifications() {
        let became = deciding::arrives(
            notification,
            &mut seat,
            Quiet::now(&Settings::shipped(), Some(&quiet_room), at(15, 0)),
            &mut missed,
        );
        assert!(became.is_shown());
    }
    assert!(missed.is_empty(), "nothing shown was missed");
}

/// **The hours a person set aside hold notifications, and only inside them.**
/// The stretch runs through midnight, which is the shape nearly every one of
/// them has.
#[test]
fn the_hours_a_person_set_aside_hold_notifications_and_only_inside_them() {
    let quiet_room = nothing_is_in_use();
    let mut settings = Settings::shipped();
    settings.quiet_hours = Some(QuietHours::these_two_times(at(23, 0), at(7, 0)).unwrap());

    for (hour, minute) in [(23_u8, 0_u8), (23, 59), (0, 0), (3, 15), (6, 59)] {
        assert_eq!(
            Quiet::now(&settings, Some(&quiet_room), at(hour, minute)),
            Quiet::Yes(Because::TheQuietHours),
            "{hour:02}:{minute:02} should have been quiet"
        );
    }
    for (hour, minute) in [(7_u8, 0_u8), (9, 30), (18, 0), (22, 59)] {
        assert_eq!(
            Quiet::now(&settings, Some(&quiet_room), at(hour, minute)),
            Quiet::No,
            "{hour:02}:{minute:02} should not have been"
        );
    }
}

/// **A screen being shared or recorded holds notifications on its own**, with
/// every setting a person could touch turned off — and says it is the
/// machine's doing rather than theirs.
///
/// This is the clause with no switch behind it. A person about to share their
/// screen has to remember nothing, which is the only version of this promise
/// that is worth making.
#[test]
fn a_screen_being_shared_holds_notifications_with_every_setting_turned_off() {
    let settings = Settings {
        do_not_disturb: false,
        quiet_hours: None,
    };
    let quiet = Quiet::now(&settings, Some(&the_screen_is_being_shared()), at(15, 0));
    assert_eq!(quiet, Quiet::Yes(Because::TheScreenIsShared));
    assert!(!Because::TheScreenIsShared.is_the_persons_own());

    let mut seat = an_open_seat();
    let mut missed = Missed::nothing();
    for notification in a_days_notifications() {
        assert!(!deciding::arrives(notification, &mut seat, quiet, &mut missed).is_shown());
    }
    assert_eq!(missed.how_many(), 4);
}

/// **The screen is asked before the person's settings**, so do-not-disturb
/// being off never reaches past a screen that is being read.
#[test]
fn the_screen_is_asked_before_the_persons_settings() {
    let mut settings = Settings::shipped();
    settings.do_not_disturb = true;
    settings.quiet_hours = Some(QuietHours::these_two_times(at(23, 0), at(7, 0)).unwrap());
    assert_eq!(
        Quiet::now(&settings, Some(&the_screen_is_being_shared()), at(2, 0)),
        Quiet::Yes(Because::TheScreenIsShared),
        "the person's own reason was given for the machine's"
    );
}

/// **A camera that is on is not a screen that is being read.** The thing being
/// promised is about what can see the screen, and a rule that held
/// notifications for any use at all would be a rule people turn off.
#[test]
fn a_camera_that_is_on_is_not_a_screen_that_is_being_read() {
    assert_eq!(
        Quiet::now(&Settings::shipped(), Some(&the_camera_is_on()), at(15, 0)),
        Quiet::No
    );
}

/// **A machine that could not ask holds too.** An answer nobody got is not an
/// answer that nothing is watching — and a private sentence shown into a
/// recording cannot be taken back.
#[test]
fn a_machine_that_could_not_ask_holds_too() {
    assert_eq!(
        Quiet::now(&Settings::shipped(), None, at(15, 0)),
        Quiet::Yes(Because::ItCannotTellAboutTheScreen)
    );
}

/// **Nothing said about a held notification names it.** Every reason is a
/// sentence with no gap in it, so there is nowhere for a sender, a subject or a
/// first line to appear on the screen the notifications were kept off.
#[test]
fn nothing_said_about_a_held_notification_names_it() {
    let strings = the_machines_words();
    let mut seat = an_open_seat();
    let mut missed = Missed::nothing();
    for because in Because::EVERY {
        let became = deciding::arrives(
            a_notification_titled("Anna Pärt — the March invoices"),
            &mut seat,
            Quiet::Yes(because),
            &mut missed,
        );
        let said = became.why().unwrap().said(&strings);
        assert!(!said.is_a_bug(), "{because:?} is not declared");
        assert!(said.unfilled().is_empty(), "{because:?}: {said}");
        assert!(!said.text().contains("Anna"), "{said}");
        assert!(!said.text().contains("invoice"), "{said}");
        assert!(
            words::EVERY_WORD.contains(&because.word()),
            "{because:?} says something this crate does not declare"
        );
    }
}
