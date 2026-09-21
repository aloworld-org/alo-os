//! The machine these tests are written against: one person, one mail client,
//! one grant, and a screen that is or is not being read.
//!
//! Shared by this crate's integration tests so that each of them is an
//! argument rather than a fixture with an argument at the end. Every value
//! here is a real one — a real sign-in, a real grant judged by `alo-portals`,
//! a real reading of a media server — because a test written against a value
//! nothing could produce proves something about that value and nothing about
//! this machine.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]
// Every integration test compiles the whole of this module and uses the part
// of it its own argument needs, so every one of them has some of it left over.
// `allow` rather than `expect` because which part is left over differs per test
// binary, and an `expect` fulfilled in four of them and unfulfilled in the fifth
// would be a warning about where a fixture is used rather than about anything
// being wrong.
#![allow(dead_code, reason = "each test binary uses a different part of this")]

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_accounts::{Accounts, Session};
use alo_applications::Application;
use alo_capability::{Applicant, Facility, Grant, Grantee, Grants, Reach};
use alo_in_use::{By, InUse, NotHeard, Streams, Use, UseId, Used};
use alo_locking::Seat;
use alo_notifying::{Notification, arriving};
use alo_overlay::Summoning;
use alo_portals::{Allowed, Over, Portal, Request};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;

/// The number this machine's person runs as.
const PERSON: u32 = 1000;

/// Anna's password, and it is right.
const ANNAS: &str = "correct horse battery staple";

/// The identifier the mail client is granted by.
pub const THE_MAIL_CLIENT: &str = "org.example.Mail";

/// Everything alo OS can say, which is what a person actually reads.
pub fn the_machines_words() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// A moment, fixed, so nothing here depends on when it runs.
pub fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The agent on this machine.
pub fn the_agent() -> Grantee {
    Grantee::named("@alo")
}

/// The mail client's request to the notification portal, judged against a
/// person's grants and allowed.
pub fn allowed_to_notify() -> Allowed {
    let applicant = Applicant::named(THE_MAIL_CLIENT);
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked_for(
            &applicant.grantee(),
            Reach::Facility(Facility::Notifications),
            noon(),
            Duration::from_secs(60 * 60),
        )
        .unwrap(),
    );
    Request::of(THE_MAIL_CLIENT, Portal::Notifications)
        .unwrap()
        .judged(&grants, noon())
        .unwrap()
}

/// The mail client's request to any portal, judged against a person who
/// granted it everything, and allowed.
///
/// For the test that walks every portal and finds that only one of them is a
/// permission to notify. A path portal's request names a file under a folder
/// the person granted; a facility portal's names nothing.
pub fn allowed_to(portal: Portal) -> Allowed {
    let applicant = Applicant::named(THE_MAIL_CLIENT);
    let hour = Duration::from_secs(60 * 60);
    let mut grants = Grants::default();
    for facility in Facility::EVERY {
        grants.grant(
            Grant::checked_for(
                &applicant.grantee(),
                Reach::Facility(facility),
                noon(),
                hour,
            )
            .unwrap(),
        );
    }
    grants.grant(
        Grant::checked_for(
            &applicant.grantee(),
            Reach::Folder(PathBuf::from("/home/anna/Documents")),
            noon(),
            hour,
        )
        .unwrap(),
    );
    let request = match portal.over() {
        Over::Facility(_) => Request::of(THE_MAIL_CLIENT, portal).unwrap(),
        Over::APath => Request::over(
            THE_MAIL_CLIENT,
            portal,
            Path::new("/home/anna/Documents/march.pdf"),
        )
        .unwrap(),
    };
    request.judged(&grants, noon()).unwrap()
}

/// One notification from the mail client, with this title.
pub fn a_notification_titled(title: &str) -> Notification {
    arriving::from_an_application(&allowed_to_notify(), title, "", &[]).unwrap()
}

/// One notification from the mail client offering these things to do.
pub fn a_notification_offering(title: &str, offering: &[(&str, &str)]) -> Notification {
    arriving::from_an_application(&allowed_to_notify(), title, "", offering).unwrap()
}

/// A media server answering with whatever it was given.
struct AServer(Vec<Use>);

impl Streams for AServer {
    fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
        Ok(self.0.clone())
    }
}

/// A quiet room: nothing is watching or listening.
pub fn nothing_is_in_use() -> InUse {
    InUse::read_from(&mut AServer(Vec::new())).unwrap()
}

/// Something is reading the screen — a meeting, sharing it.
pub fn the_screen_is_being_shared() -> InUse {
    InUse::read_from(&mut AServer(vec![Use::of(
        UseId::recorded(42),
        Used::Screen,
        By::an_application(Application::called("com.example.Meeting", "Meeting").unwrap()),
    )]))
    .unwrap()
}

/// The camera is on and the screen is not being read — the case a lazier rule
/// would have got wrong.
pub fn the_camera_is_on() -> InUse {
    InUse::read_from(&mut AServer(vec![Use::of(
        UseId::recorded(43),
        Used::Camera,
        By::an_application(Application::called("com.example.Meeting", "Meeting").unwrap()),
    )]))
    .unwrap()
}

/// Anna's session, as a sign-in really opens one.
pub fn anna() -> Session {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", PERSON, ANNAS).unwrap();
    let signed_in = accounts.signs_in("anna", ANNAS).unwrap();
    Session::opened(signed_in, PERSON).unwrap()
}

/// Anna's seat, open.
pub fn an_open_seat() -> Seat<Notification> {
    Seat::opened(anna())
}

/// That seat, locked.
pub fn locked(seat: Seat<Notification>) -> Seat<Notification> {
    seat.locked(&mut Summoning::closed())
}

/// The machine's accounts, for an unlock.
pub fn the_accounts() -> Accounts {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", PERSON, ANNAS).unwrap();
    accounts
}

/// Anna's password, for an unlock.
pub const fn annas_password() -> &'static str {
    ANNAS
}
