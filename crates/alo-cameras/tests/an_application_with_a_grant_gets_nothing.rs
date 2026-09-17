//! The plan's acceptance: **a person can turn the camera or the microphone off
//! for everyone, and an application with a grant still gets nothing.**
//!
//! This is held against a **real** grant, made with `alo-capability` the way an
//! application's grant is really made, which really does permit the camera at
//! the moment it is asked. Then the switch is off, and the answer is no.
//!
//! A test written against a stand-in for the grant would prove that this crate's
//! own refusal works. What has to be true is stronger and simpler: **the switch
//! is not weighed against the grant.** It is asked first, it is asked about the
//! machine rather than about the application, and there is no application
//! important enough to be an exception — because an exception is a thing a
//! person would have to know about before they could trust the switch, and then
//! it would not be a switch.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_cameras::{Switch, TheSwitches, TurnedOff, Which};
use alo_capability::{Applicant, Ask, Facility, Grant, Grants, Reach};

/// An application, granted the camera for an hour, as one really is granted.
fn an_application_that_was_granted_the_camera() -> (Grants, Applicant, SystemTime) {
    let now = SystemTime::now();
    let application = Applicant::named("world.alo.example.VideoCall");
    let grant = Grant::checked_for(
        &application.grantee(),
        Reach::Facility(Facility::Camera),
        now,
        Duration::from_secs(60 * 60),
    )
    .expect("an application may be granted a facility");
    let mut grants = Grants::default();
    grants.grant(grant);
    (grants, application, now)
}

/// **The grant says yes and the switch says no, and no is the answer.**
#[test]
fn a_real_grant_that_permits_the_camera_is_not_enough_when_the_switch_is_off() {
    let (grants, application, now) = an_application_that_was_granted_the_camera();
    let asking = Ask::Facility(Facility::Camera);

    // The grant is real, and it permits.
    assert!(
        grants.allowing(&application, &asking, now).is_ok(),
        "the fixture's grant does not allow the camera, so this test would prove nothing"
    );

    // And with the switch off, the machine's answer is still no.
    let off = TheSwitches::both_on().switching(Which::Camera);
    assert_eq!(
        off.may_open(Which::Camera),
        Err(TurnedOff {
            which: Which::Camera
        }),
        "an application with a grant was given the camera while a person had it switched off"
    );

    // The grant was not revoked, changed or hidden. A switch is not a
    // withdrawal of permission: it is the machine, and when it goes back on the
    // application has what it had.
    assert!(
        grants.allowing(&application, &asking, now).is_ok(),
        "turning the switch off took an application's grant away, which is a second thing \
         happening behind one switch"
    );
    let on = off.switching(Which::Camera);
    assert!(on.may_open(Which::Camera).is_ok());
    assert_eq!(on.of(Which::Camera), Switch::On);
}

/// **The microphone is its own switch**, and covering a camera does not mute a
/// call.
#[test]
fn the_two_switches_do_not_stand_in_for_each_other() {
    let (grants, application, now) = an_application_that_was_granted_the_camera();
    let camera_off = TheSwitches::both_on().switching(Which::Camera);

    assert!(camera_off.may_open(Which::Microphone).is_ok());
    assert!(camera_off.may_open(Which::Camera).is_err());
    assert!(
        grants
            .allowing(&application, &Ask::Facility(Facility::Camera), now)
            .is_ok()
    );

    let both_off = camera_off.switching(Which::Microphone);
    for which in Which::BOTH {
        assert!(
            both_off.may_open(which).is_err(),
            "{which:?} was still open"
        );
    }
}

/// **An agent cannot hold this grant at all**, which is ADR 0040 and is checked
/// here because this crate's switch would otherwise look like the only thing
/// standing between an agent and a camera.
#[test]
fn a_facility_is_not_something_an_agent_can_be_granted() {
    let why = Grant::checked(
        "an-agent",
        Reach::Facility(Facility::Camera),
        SystemTime::now(),
        Duration::from_secs(60),
    )
    .expect_err("ADR 0040 keeps facilities to applications");
    assert_eq!(why, alo_capability::GrantError::NotForAnAgent);
}

/// **The portal says yes and the switch still says no.**
///
/// The road an application really takes: it asks the portal, the portal judges
/// the request against the grants, and the grant it holds allows it. This is
/// where the switch sits — **after** the portal has said yes, so that an
/// application which did everything right and was allowed everything still gets
/// nothing while a person has the camera off.
#[test]
fn a_request_the_portal_allowed_is_still_refused_while_the_camera_is_off() {
    use alo_portals::{Portal, Request};

    let (grants, application, now) = an_application_that_was_granted_the_camera();
    let asking = Request::of(application.as_str(), Portal::Camera).expect("a request");
    assert!(
        asking.judged(&grants, now).is_ok(),
        "the portal refused the fixture's request, so this test would prove nothing"
    );

    let off = TheSwitches::both_on().switching(Which::Camera);
    assert!(
        off.may_open(Which::Camera).is_err(),
        "an application the portal allowed was given the camera while a person had it off"
    );

    // And the portal's answer did not change, because the switch is not a
    // permission and does not pretend to be one: it is the machine.
    assert!(asking.judged(&grants, now).is_ok());
}
