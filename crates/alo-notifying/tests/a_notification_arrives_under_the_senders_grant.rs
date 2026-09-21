//! A notification is who sent it, a title, a body and its actions — arriving
//! through the notification portal under the sender's grant.
//!
//! The plan's first clause for this task, and the whole of it is about
//! provenance. An application does not send a notification by reaching a socket
//! and saying who it is; it sends one by having asked `alo-portals` for the
//! notification portal and having been allowed by a grant the person made over
//! `alo_capability::Facility::Notifications` (ADR 0040). What
//! `alo_notifying::arriving::from_an_application` takes is the `Allowed` that
//! judging produced, and there is no other constructor for an application's
//! notification anywhere in the crate.
//!
//! So this walks the whole road: no grant, a grant for something else, a grant
//! that has been revoked, a grant that has run out — and then the one that
//! works.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod common;

use std::time::Duration;

use alo_applications::Application;
use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
use alo_notifying::{MOST_THINGS_TO_DO, NotSent, arriving};
use alo_portals::{Portal, Refused, Request};

use common::{THE_MAIL_CLIENT, allowed_to_notify, noon, the_machines_words};

/// An hour.
fn an_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// **A notification is who, a title, a body and what it offers** — and nothing
/// else, because there is nothing else to give it.
#[test]
fn a_notification_is_who_a_title_a_body_and_what_it_offers() {
    let strings = the_machines_words();
    let notification = arriving::from_an_application(
        &allowed_to_notify(),
        "Anna Pärt",
        "Are we still on for Thursday?",
        &[("reply", "Reply"), ("mark-as-read", "Mark as read")],
    )
    .unwrap();

    assert_eq!(
        notification
            .from()
            .application()
            .map(Application::identifier),
        Some(THE_MAIL_CLIENT)
    );
    assert_eq!(notification.title(), "Anna Pärt");
    assert_eq!(notification.body(), "Are we still on for Thursday?");
    assert_eq!(notification.offering().len(), 2);
    assert_eq!(
        notification.sent_by(&strings).text(),
        "Sent by org.example.Mail"
    );
}

/// **An application nobody granted anything cannot send one**, because
/// judging its request never produces the `Allowed` this crate takes.
#[test]
fn an_application_nobody_granted_anything_cannot_send_one() {
    let grants = Grants::default();
    let refused = Request::of(THE_MAIL_CLIENT, Portal::Notifications)
        .unwrap()
        .judged(&grants, noon())
        .unwrap_err();
    assert!(matches!(refused, Refused::NothingGranted { .. }));
}

/// **A grant over something else is not a grant to notify.** The camera is
/// granted, the notification portal is asked, and `alo-portals` refuses before
/// this crate is reached at all.
#[test]
fn a_grant_over_something_else_is_not_a_grant_to_notify() {
    let applicant = Applicant::named(THE_MAIL_CLIENT);
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked_for(
            &applicant.grantee(),
            Reach::Facility(Facility::Camera),
            noon(),
            an_hour(),
        )
        .unwrap(),
    );
    let refused = Request::of(THE_MAIL_CLIENT, Portal::Notifications)
        .unwrap()
        .judged(&grants, noon())
        .unwrap_err();
    assert!(matches!(refused, Refused::NotAllowed { .. }));
}

/// **A revoked grant stops the next notification, and an expired one is
/// gone.** Judging keeps nothing, so the list as it now is decides.
#[test]
fn a_revoked_grant_stops_the_next_notification_and_an_expired_one_is_gone() {
    let applicant = Applicant::named(THE_MAIL_CLIENT);
    let mut grants = Grants::default();
    let id = grants.grant(
        Grant::checked_for(
            &applicant.grantee(),
            Reach::Facility(Facility::Notifications),
            noon(),
            an_hour(),
        )
        .unwrap(),
    );
    let asking = Request::of(THE_MAIL_CLIENT, Portal::Notifications).unwrap();

    // While it holds, a notification goes out.
    let allowed = asking.judged(&grants, noon()).unwrap();
    assert!(arriving::from_an_application(&allowed, "Anna Pärt", "", &[]).is_ok());

    // An hour and a minute later it has run out.
    let later = noon() + an_hour() + Duration::from_secs(60);
    assert!(asking.judged(&grants, later).is_err());

    // And revoked is refused at the very next request.
    assert!(grants.revoke(id));
    assert!(asking.judged(&grants, noon()).is_err());
}

/// **An `Allowed` for any other portal is refused here.** The grant was real
/// and the person made it; it was a grant to print, or to use the camera, and
/// this crate will not read it as a grant to put text on a screen.
#[test]
fn an_allowed_for_any_other_portal_is_refused_here() {
    for portal in Portal::EVERY {
        if portal == Portal::Notifications {
            continue;
        }
        let allowed = common::allowed_to(portal);
        assert_eq!(
            arriving::from_an_application(&allowed, "Anna Pärt", "", &[]),
            Err(NotSent::NotANotification {
                application: THE_MAIL_CLIENT.to_owned(),
            }),
            "{portal:?} was read as a permission to notify"
        );
    }
}

/// **A notification that is not one is refused with the program named**, and
/// every refusal is a sentence a person can act on.
#[test]
fn a_notification_that_is_not_one_is_refused_with_the_program_named() {
    let strings = the_machines_words();
    let allowed = allowed_to_notify();
    let too_many: Vec<(&str, &str)> = (0..=MOST_THINGS_TO_DO)
        .map(|_| ("reply", "Reply"))
        .collect();

    for refused in [
        arriving::from_an_application(&allowed, "  ", "", &[]).unwrap_err(),
        arriving::from_an_application(&allowed, "Anna", "", &[("", "Reply")]).unwrap_err(),
        arriving::from_an_application(&allowed, "Anna", "", &[("reply", " ")]).unwrap_err(),
        arriving::from_an_application(&allowed, "Anna", "", &too_many).unwrap_err(),
    ] {
        assert_eq!(refused.application(), THE_MAIL_CLIENT);
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{refused:?} is not declared");
        assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
        assert!(said.text().contains(THE_MAIL_CLIENT), "{said}");
    }
}
