//! The person, the machine, the application and the screen this crate's own
//! tests are written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # The grant is a real grant
//!
//! [`allowed_to`] is a real `alo_portals::Request` judged against real
//! `alo_capability::Grants`, so a notification this crate accepts is one a
//! person's own grant allowed. Nothing in these tests builds an
//! `alo_portals::Allowed` by hand, because there is no way to.
//!
//! # And the session is a real sign-in
//!
//! [`anna`] is what `alo-accounts` hands back when a password verifies and the
//! machine description agrees about the number, which is the only way an
//! `alo_accounts::Session` exists. A seat that holds a notification behind its
//! lock screen is therefore a session somebody really opened.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_accounts::{Accounts, Session};
use alo_appearance::TimeOfDay;
use alo_applications::Application;
use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
use alo_in_use::{By, InUse, NotHeard, Streams, Use, UseId, Used};
use alo_locking::Seat;
use alo_overlay::Summoning;
use alo_portals::{Allowed, Over, Portal, Request};
use alo_strings::{Strings, Vocabulary};

use crate::arriving::from_an_application;
use crate::changes::Settings;
use crate::notification::Notification;

/// The number this machine's person runs as.
const PERSON: u32 = 1000;

/// Anna's password, and it is right.
const ANNAS: &str = "correct horse battery staple";

/// The identifier the mail client is granted by.
pub(crate) const THE_MAIL_CLIENT: &str = "org.example.Mail";

/// This crate's words, in English — and `alo-applications`', because a
/// sentence saying who a notification is from puts an application's own clause
/// inside it (`applications.called`), and a whole sentence is only as
/// translated as its least translated piece.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::declare_into(&mut vocabulary).unwrap();
    alo_applications::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A moment, fixed, so nothing in these tests depends on when they run.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A time on the clock the person is looking at.
pub(crate) fn at(hour: u8, minute: u8) -> TimeOfDay {
    TimeOfDay::checked(hour, minute).unwrap()
}

/// What alo OS ships.
pub(crate) const fn what_alo_os_ships() -> Settings {
    Settings::shipped()
}

/// The mail client, by the identifier it is granted by and the name it calls
/// itself.
pub(crate) fn a_mail_client() -> Application {
    Application::called(THE_MAIL_CLIENT, "Mail").unwrap()
}

/// The agent on this machine.
pub(crate) fn the_agent() -> alo_capability::Grantee {
    alo_capability::Grantee::named("@alo")
}

/// The mail client's request to this portal, judged against a person's grants
/// and allowed.
pub(crate) fn allowed_to(portal: Portal) -> Allowed {
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
    grants.grant(
        Grant::checked_for(
            &applicant.grantee(),
            Reach::Application("org.example.Viewer".to_owned()),
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
pub(crate) fn a_notification_titled(title: &str) -> Notification {
    from_an_application(&allowed_to(Portal::Notifications), title, "", &[]).unwrap()
}

/// A media server answering with whatever it was given.
struct AServer(Vec<Use>);

impl Streams for AServer {
    fn in_use_now(&mut self) -> Result<Vec<Use>, NotHeard> {
        Ok(self.0.clone())
    }
}

/// A quiet room: nothing is watching or listening.
pub(crate) fn nothing_is_in_use() -> InUse {
    InUse::read_from(&mut AServer(Vec::new())).unwrap()
}

/// Something is reading the screen.
pub(crate) fn the_screen_is_being_recorded() -> InUse {
    InUse::read_from(&mut AServer(vec![Use::of(
        UseId::recorded(42),
        Used::Screen,
        By::an_application(Application::called("com.example.Meeting", "Meeting").unwrap()),
    )]))
    .unwrap()
}

/// The machine's accounts: Anna, at this machine's number.
fn the_machine() -> Accounts {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", PERSON, ANNAS).unwrap();
    accounts
}

/// Anna's session, as a sign-in really opens one. Made once: hashing a
/// password is deliberately slow.
pub(crate) fn anna() -> Session {
    static ANNA: OnceLock<Session> = OnceLock::new();
    ANNA.get_or_init(|| {
        let signed_in = the_machine().signs_in("anna", ANNAS).unwrap();
        Session::opened(signed_in, PERSON).unwrap()
    })
    .clone()
}

/// Anna's seat, open.
pub(crate) fn an_open_seat() -> Seat<Notification> {
    Seat::opened(anna())
}

/// That seat, locked.
pub(crate) fn locked(seat: Seat<Notification>) -> Seat<Notification> {
    seat.locked(&mut Summoning::closed())
}

/// A folder under the temporary directory that is this test's alone, emptied.
pub(crate) fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-notifying-{what}-{}-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}
