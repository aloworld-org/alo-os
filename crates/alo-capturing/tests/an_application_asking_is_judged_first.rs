//! An application asking for a picture of the screen is judged against its
//! grant, before anything is taken.
//!
//! The plan's sixth acceptance for this task. Two things are held here, and the
//! second is the one that is easy to get wrong:
//!
//! - **judged by `alo-portals`** — the same `Request::judged` an application's
//!   camera request goes through, against the same `alo_capability::Grants` an
//!   agent's verb is, refused in the same words. Nothing in `alo-capturing`
//!   decides whether an application may photograph a screen, and there is no
//!   list of trusted applications anywhere in it;
//! - **before anything is taken** — the mechanism counts how often it was asked
//!   for a picture, and after every refusal below it is at zero. It is a
//!   property of the types rather than of the order of some lines:
//!   `ForAnApplication` is the only value with a `take` on it for an
//!   application's picture, and `Asked::judged` is its only constructor.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
use alo_capturing::{
    Asked, Grabs, NotGrabbed, Picture, Region, Screen, Screenshot, Session, Taken, What,
    WhereItGoes, WhoseSession,
};
use alo_clipboard::Clipboard;
use alo_portals::Refused;

/// A screen-capture mechanism that keeps count of how often it was asked.
#[derive(Debug, Default)]
struct Counting {
    /// How many pictures it was asked for.
    how_often: usize,
}

impl Grabs for Counting {
    fn grab(&mut self, _across: Region, _on: Screen) -> Result<Picture, NotGrabbed> {
        self.how_often = self.how_often.saturating_add(1);
        Picture::of(b"\x89PNG".to_vec())
    }
}

/// Noon on the sixteenth of September 2026.
fn noon() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_789_560_000)
}

/// How long the grants in these tests last.
fn the_hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A machine on which this application was granted this facility at noon.
fn granted(application: &str, facility: Facility) -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked_for(
            &Applicant::named(application).grantee(),
            Reach::Facility(facility),
            noon(),
            the_hour(),
        )
        .expect("a grant"),
    );
    grants
}

/// The picture every application in these tests asks for.
fn a_picture_of_the_screen() -> Screenshot {
    Screenshot::of(
        What::TheWholeScreen,
        &Session::of(&WhoseSession::of("anna")),
        Screen::measuring(1920, 1080).expect("a screen"),
        WhereItGoes::the_clipboard(),
    )
    .expect("a picture that may be taken")
}

/// **An application the person granted this may take it**, and the grants it
/// was allowed against come back with it for whatever writes the answer down.
#[test]
fn an_application_that_was_granted_this_takes_its_picture() {
    let mut mechanism = Counting::default();
    let mut clipboard = Clipboard::nothing_copied_yet();
    let grants = granted("com.example.Notes", Facility::ScreenOnce);

    let allowed = Asked::of("com.example.Notes")
        .expect("a request")
        .judged(a_picture_of_the_screen(), &grants, noon())
        .expect("its grant covers this");

    assert!(!allowed.against().against().is_empty());
    let taken = allowed
        .take(
            alo_capturing::OnThisDay::at(noon(), 0),
            &mut mechanism,
            &mut clipboard,
        )
        .expect("a picture");

    assert_eq!(taken, Taken::Copied);
    assert_eq!(mechanism.how_often, 1);
}

/// **An application nobody granted anything is refused before anything is
/// taken**, and before any question could reach the person's screen.
#[test]
fn an_application_nobody_granted_anything_is_refused_and_nothing_is_taken() {
    let mechanism = Counting::default();

    let refused = Asked::of("com.example.Stranger")
        .expect("a request")
        .judged(a_picture_of_the_screen(), &Grants::default(), noon())
        .expect_err("nothing is granted on this machine");

    assert!(matches!(
        refused.refused(),
        Some(Refused::NothingGranted { .. })
    ));
    assert_eq!(
        mechanism.how_often, 0,
        "a picture was taken for an application nobody granted anything"
    );
}

/// **A grant to something else is not a grant to this.** A camera grant is a
/// camera grant; ADR 0040's table is `alo-portals`' and is never restated here.
#[test]
fn a_grant_to_the_camera_is_not_a_grant_to_a_picture_of_the_screen() {
    let mechanism = Counting::default();
    let grants = granted("com.example.VideoCall", Facility::Camera);

    let refused = Asked::of("com.example.VideoCall")
        .expect("a request")
        .judged(a_picture_of_the_screen(), &grants, noon())
        .expect_err("the camera is not the screen");

    assert!(matches!(
        refused.refused(),
        Some(Refused::NotAllowed { .. })
    ));
    assert_eq!(mechanism.how_often, 0);
}

/// **A grant to record the screen is not a grant to photograph it either.**
/// The two are separate facilities, and this crate asks for the one it is.
#[test]
fn a_grant_to_record_the_screen_is_not_a_grant_to_photograph_it() {
    let grants = granted("com.example.Meeting", Facility::ScreenContinuously);

    let refused = Asked::of("com.example.Meeting")
        .expect("a request")
        .judged(a_picture_of_the_screen(), &grants, noon())
        .expect_err("recording is not photographing");

    assert!(matches!(
        refused.refused(),
        Some(Refused::NotAllowed { .. })
    ));
}

/// **A grant that has ended is not a grant.** The same application, the same
/// request, an hour later — which is what *grants expire* means when somebody
/// checks rather than reads.
///
/// It is refused as *nothing granted* rather than as *not allowed this*, and
/// that is `alo-portals`' own ordering read back rather than a surprise: an
/// application whose only grant has ended holds no live grant at all at that
/// moment, and a request from one is refused before anything it asked for is
/// looked at. This crate does not reword it either way.
#[test]
fn a_grant_that_has_ended_takes_nothing() {
    let mechanism = Counting::default();
    let grants = granted("com.example.Notes", Facility::ScreenOnce);
    let asked = Asked::of("com.example.Notes").expect("a request");

    assert!(
        asked
            .judged(a_picture_of_the_screen(), &grants, noon())
            .is_ok()
    );

    let later = noon() + the_hour();
    let refused = asked
        .judged(a_picture_of_the_screen(), &grants, later)
        .expect_err("the grant has ended");
    assert!(
        matches!(refused.refused(), Some(Refused::NothingGranted { .. })),
        "{refused:?}"
    );
    assert_eq!(mechanism.how_often, 0);
}

/// **A grant revoked takes effect on the next request**, with nothing to wait
/// for and no cache in front of it — `alo-capability`'s own revocation, reached
/// through the one road this crate has.
#[test]
fn a_grant_revoked_takes_effect_on_the_next_request() {
    let mut grants = Grants::default();
    let to_notes = grants.grant(
        Grant::checked_for(
            &Applicant::named("com.example.Notes").grantee(),
            Reach::Facility(Facility::ScreenOnce),
            noon(),
            the_hour(),
        )
        .expect("a grant"),
    );
    let asked = Asked::of("com.example.Notes").expect("a request");
    assert!(
        asked
            .judged(a_picture_of_the_screen(), &grants, noon())
            .is_ok()
    );

    grants.revoke(to_notes);
    assert!(
        asked
            .judged(a_picture_of_the_screen(), &grants, noon())
            .is_err(),
        "a revoked grant still allowed a picture of the screen"
    );
}
