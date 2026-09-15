//! One keyring behind the Secret portal.
//!
//! Task 3 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`,
//! each clause of its acceptance a test here, against a real `gnome-keyring`
//! on a bus of the test's own:
//!
//! - **an application's request for the Secret portal is a grant naming the
//!   application, and a secret it stores is kept in the same keyring, in a
//!   collection the application's name owns — stored as one application, it
//!   cannot be read as another** —
//!   [`a_secret_kept_as_one_application_cannot_be_read_as_another`];
//! - the refusals beside it: no grant, the wrong grant, an expired one, the
//!   wrong portal, a locked keyring, a name or a size that is not one —
//!   [`a_request_the_grants_do_not_cover_never_reaches_the_keyring`] and
//!   [`the_keyring_refuses_what_it_should_and_never_unlocks_itself`];
//! - **the daemon's own keys and an application's are separated by collection
//!   and by grant, never by hoping** —
//!   [`the_daemons_keys_and_an_applications_never_answer_for_each_other`];
//! - **the keyring is the machine's one keyring — a test that starts the
//!   fixture finds no second Secret Service on the bus** —
//!   [`the_fixture_is_the_one_secret_service_on_its_bus`];
//! - **a secret stored through the portal is revocable with the application's
//!   grant, gone at the next request** —
//!   [`revoking_the_grant_ends_the_reach_at_the_next_request`].
//!
//! **Every secret here is synthetic**, kept in a keyring this test starts in a
//! temporary directory and kills afterwards.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};

use alo_capability::{Applicant, Facility, Grant, GrantId, Grants, Reach};
use alo_granted::{Listing, Revoked};
use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_models::SecretRef;
use alo_portals::{Portal, Refused, Request};
use alo_secrets::{
    AN_APPLICATIONS, LARGEST_SECRET, NotStored, PORTAL_SECRET_LENGTH, TheKeyring, Withheld,
};
use secret_service::EncryptionType;
use secret_service::blocking::SecretService;

const FRACTAL: &str = "org.gnome.Fractal";
const EVOLUTION: &str = "org.gnome.Evolution";

/// A password that is a credential to nothing.
const FRACTALS_TOKEN: &[u8] = b"syt_FIXTURE-ONLY-fractal-4b19c7";
const EVOLUTIONS_TOKEN: &[u8] = b"imap-FIXTURE-ONLY-evolution-9d02";

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// A grant to an application, made the way a person's is.
fn granted(application: &str, facility: Facility) -> Grant {
    Grant::checked_for(
        &Applicant::named(application).grantee(),
        Reach::Facility(facility),
        noon(),
        hour(),
    )
    .unwrap()
}

/// A request to the Secret portal, as the backend would make one.
fn asking(application: &str) -> Request {
    Request::of(application, Portal::Secret).unwrap()
}

/// A client of the fixture's own, separate from the one under test, so that
/// what this file checks about the keyring is never read back through the
/// object it is checking.
fn a_client(keyring: &AKeyringOfOurOwn) -> SecretService<'static> {
    let connection = zbus::blocking::connection::Builder::address(keyring.address().as_str())
        .expect("the fixture's address is an address")
        .build()
        .expect("the fixture's bus accepts a connection");
    SecretService::connect_with_existing(EncryptionType::Dh, connection)
        .expect("the fixture serves the Secret Service")
}

/// The attributes of every item in the fixture's default collection, read by
/// the fixture's own client.
fn everything_in_the_default_collection(
    keyring: &AKeyringOfOurOwn,
) -> Vec<HashMap<String, String>> {
    let service = a_client(keyring);
    let collection = service.get_default_collection().unwrap();
    collection
        .get_all_items()
        .unwrap()
        .iter()
        .map(|item| item.get_attributes().unwrap())
        .collect()
}

/// **A secret kept as one application is kept in the one keyring, filed under
/// that application, and cannot be read as another.**
#[test]
fn a_secret_kept_as_one_application_cannot_be_read_as_another() {
    let keyring = AKeyringOfOurOwn::started("apps-apart");
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens on its own bus");

    let mut grants = Grants::default();
    let fractals = grants.grant(granted(FRACTAL, Facility::Secrets));
    grants.grant(granted(EVOLUTION, Facility::Secrets));

    // A request for the Secret portal is a grant naming the application.
    let fractal = ours
        .for_the_application(&asking(FRACTAL), &grants, noon())
        .expect("Fractal was granted its secrets");
    assert_eq!(fractal.allowed().application(), &Applicant::named(FRACTAL));
    assert_eq!(fractal.allowed().portal(), Portal::Secret);
    assert_eq!(fractal.allowed().against(), [fractals]);
    fractal
        .keep("matrix", FRACTALS_TOKEN)
        .expect("Fractal keeps a token");

    // Read back as itself.
    let read = ours
        .for_the_application(&asking(FRACTAL), &grants, noon())
        .unwrap()
        .kept("matrix")
        .expect("Fractal reads back what it kept");
    assert_eq!(read.bytes(), FRACTALS_TOKEN);
    assert!(
        !format!("{read:?}").contains("FIXTURE"),
        "a kept secret rendered itself"
    );

    // And not as Evolution, which asks for the very same name.
    assert_eq!(
        ours.for_the_application(&asking(EVOLUTION), &grants, noon())
            .unwrap()
            .kept("matrix")
            .unwrap_err(),
        Withheld::NotStored(NotStored::Missing),
        "Evolution read a secret Fractal kept"
    );
    assert_eq!(
        ours.for_the_application(&asking(EVOLUTION), &grants, noon())
            .unwrap()
            .forget("matrix")
            .unwrap_err(),
        Withheld::NotStored(NotStored::Missing),
        "Evolution could remove a secret Fractal kept"
    );

    // Evolution's own, under the same name, is its own and leaves Fractal's.
    ours.for_the_application(&asking(EVOLUTION), &grants, noon())
        .unwrap()
        .keep("matrix", EVOLUTIONS_TOKEN)
        .unwrap();
    for (application, token) in [(FRACTAL, FRACTALS_TOKEN), (EVOLUTION, EVOLUTIONS_TOKEN)] {
        let read = ours
            .for_the_application(&asking(application), &grants, noon())
            .unwrap()
            .kept("matrix")
            .unwrap();
        assert_eq!(read.bytes(), token, "{application}");
    }

    // Keeping again replaces the application's own and nothing else.
    ours.for_the_application(&asking(FRACTAL), &grants, noon())
        .unwrap()
        .keep("matrix", b"syt_FIXTURE-ONLY-fractal-rotated")
        .unwrap();

    // In the same keyring, the person's default collection, each filed under
    // the application's name — read by a client that is not the one under test.
    let items = everything_in_the_default_collection(&keyring);
    let mine: Vec<&HashMap<String, String>> = items
        .iter()
        .filter(|attributes| {
            attributes.get("xdg:schema").map(String::as_str) == Some(AN_APPLICATIONS)
        })
        .collect();
    assert_eq!(mine.len(), 2, "{items:?}");
    let whose: Vec<&str> = mine
        .iter()
        .map(|attributes| attributes.get("application").unwrap().as_str())
        .collect();
    assert!(
        whose.contains(&FRACTAL) && whose.contains(&EVOLUTION),
        "{whose:?}"
    );

    // The portal's one secret: made once, the same bytes every time after, and
    // each application's its own.
    let first = ours
        .for_the_application(&asking(FRACTAL), &grants, noon())
        .unwrap()
        .the_portals_secret()
        .unwrap();
    let again = ours
        .for_the_application(&asking(FRACTAL), &grants, noon())
        .unwrap()
        .the_portals_secret()
        .unwrap();
    let evolutions = ours
        .for_the_application(&asking(EVOLUTION), &grants, noon())
        .unwrap()
        .the_portals_secret()
        .unwrap();
    assert_eq!(first.bytes().len(), PORTAL_SECRET_LENGTH);
    assert_eq!(first.bytes(), again.bytes());
    assert_ne!(first.bytes(), evolutions.bytes());
    assert_ne!(first.bytes(), [0_u8; PORTAL_SECRET_LENGTH]);

    // Forgetting is the application's own act on its own secret.
    ours.for_the_application(&asking(FRACTAL), &grants, noon())
        .unwrap()
        .forget("matrix")
        .unwrap();
    assert_eq!(
        ours.for_the_application(&asking(FRACTAL), &grants, noon())
            .unwrap()
            .kept("matrix")
            .unwrap_err(),
        Withheld::NotStored(NotStored::Missing)
    );
    assert_eq!(
        ours.for_the_application(&asking(EVOLUTION), &grants, noon())
            .unwrap()
            .kept("matrix")
            .unwrap()
            .bytes(),
        EVOLUTIONS_TOKEN
    );
}

/// **A request the grants do not cover never reaches the keyring**: an
/// application granted nothing, one granted only the camera, one whose grant
/// has expired, and a request to another portal — and nothing is filed for
/// any of them.
#[test]
fn a_request_the_grants_do_not_cover_never_reaches_the_keyring() {
    let keyring = AKeyringOfOurOwn::started("apps-refused");
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens");

    let mut grants = Grants::default();
    grants.grant(granted(FRACTAL, Facility::Secrets));
    grants.grant(granted(EVOLUTION, Facility::Camera));

    // Granted nothing: refused before any dialog, in alo-portals' words.
    let stranger = "org.example.Stranger";
    assert!(matches!(
        ours.for_the_application(&asking(stranger), &grants, noon()),
        Err(Withheld::Refused(Refused::NothingGranted { .. }))
    ));

    // Granted the camera, not its secrets.
    assert!(matches!(
        ours.for_the_application(&asking(EVOLUTION), &grants, noon()),
        Err(Withheld::Refused(Refused::NotAllowed {
            portal: Portal::Secret,
            ..
        }))
    ));

    // Granted, and expired.
    assert!(matches!(
        ours.for_the_application(&asking(FRACTAL), &grants, noon() + hour()),
        Err(Withheld::Refused(_))
    ));

    // Granted, and asking another portal through this door.
    let camera = Request::of(FRACTAL, Portal::Camera).unwrap();
    assert!(matches!(
        ours.for_the_application(&camera, &grants, noon()),
        Err(Withheld::NotTheSecretPortal(Portal::Camera))
    ));

    // None of them put anything in the keyring.
    let items = everything_in_the_default_collection(&keyring);
    assert!(
        items.iter().all(
            |attributes| attributes.get("xdg:schema").map(String::as_str) != Some(AN_APPLICATIONS)
        ),
        "a refused request left something in the keyring: {items:?}"
    );
}

/// **The keyring refuses what it should, and never unlocks itself**: a name
/// that is not one, a secret too large to be a password, and a locked
/// keyring — which stays locked.
#[test]
fn the_keyring_refuses_what_it_should_and_never_unlocks_itself() {
    let keyring = AKeyringOfOurOwn::started("apps-locked");
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens");
    let mut grants = Grants::default();
    grants.grant(granted(FRACTAL, Facility::Secrets));
    let fractal = || {
        ours.for_the_application(&asking(FRACTAL), &grants, noon())
            .unwrap()
    };

    assert_eq!(fractal().keep("", FRACTALS_TOKEN), Err(Withheld::NotAName));
    assert_eq!(fractal().kept("matrix\n").unwrap_err(), Withheld::NotAName);
    assert_eq!(
        fractal().keep("matrix", &vec![7_u8; LARGEST_SECRET + 1]),
        Err(Withheld::TooLarge)
    );
    fractal()
        .keep("largest", &vec![7_u8; LARGEST_SECRET])
        .expect("a secret of exactly the largest size is kept");
    fractal().keep("matrix", FRACTALS_TOKEN).unwrap();

    let theirs = a_client(&keyring);
    let collection = theirs.get_default_collection().unwrap();
    collection.lock().expect("the collection can be locked");
    assert!(
        collection.is_locked().unwrap(),
        "the fixture did not lock it"
    );

    assert_eq!(
        fractal().keep("matrix", FRACTALS_TOKEN),
        Err(Withheld::NotStored(NotStored::Locked))
    );
    assert_eq!(
        fractal().kept("matrix").unwrap_err(),
        Withheld::NotStored(NotStored::Locked)
    );
    assert_eq!(
        fractal().the_portals_secret().unwrap_err(),
        Withheld::NotStored(NotStored::Locked)
    );
    assert!(
        collection.is_locked().unwrap(),
        "asking unlocked the person's keyring on their behalf"
    );
}

/// **The daemon's own keys and an application's never answer for each other**,
/// even under the same name.
#[test]
fn the_daemons_keys_and_an_applications_never_answer_for_each_other() {
    let keyring = AKeyringOfOurOwn::started("apps-and-providers");
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens");
    let mut grants = Grants::default();
    grants.grant(granted(FRACTAL, Facility::Secrets));

    // An application keeps a secret under the name a provider's key has.
    ours.for_the_application(&asking(FRACTAL), &grants, noon())
        .unwrap()
        .keep("provider/Mistral", FRACTALS_TOKEN)
        .unwrap();
    // The daemon's lookup does not find it.
    assert_eq!(
        ours.look_up(&SecretRef::named("provider/Mistral")).err(),
        Some(NotStored::Missing),
        "the daemon was handed an application's secret as a provider's key"
    );

    // The provider's key, filed the way the daemon's are, under a name the
    // application has not used.
    let theirs = a_client(&keyring);
    let mut attributes = HashMap::new();
    attributes.insert("xdg:schema", "dev.alo.Provider");
    attributes.insert("reference", "provider/Anthropic");
    theirs
        .get_default_collection()
        .unwrap()
        .create_item(
            "a provider key this test invented",
            attributes,
            b"sk-FIXTURE-ONLY-provider",
            true,
            "text/plain",
        )
        .unwrap();
    assert!(
        ours.look_up(&SecretRef::named("provider/Anthropic"))
            .is_ok()
    );

    // The application cannot reach it by that name.
    assert_eq!(
        ours.for_the_application(&asking(FRACTAL), &grants, noon())
            .unwrap()
            .kept("provider/Anthropic")
            .unwrap_err(),
        Withheld::NotStored(NotStored::Missing),
        "an application read the daemon's provider key"
    );
    assert!(
        ours.look_up(&SecretRef::named("provider/Anthropic"))
            .is_ok(),
        "the provider's key went missing"
    );
}

/// **Revoking the grant ends the application's reach at its next request**,
/// through the one list a person revokes from — and an allowance already in
/// hand does its one thing and cannot be used again.
#[test]
fn revoking_the_grant_ends_the_reach_at_the_next_request() {
    let keyring = AKeyringOfOurOwn::started("apps-revoked");
    let ours = TheKeyring::opened(&keyring.bus()).expect("the keyring opens");
    let mut grants = Grants::default();
    let fractals = grants.grant(granted(FRACTAL, Facility::Secrets));
    grants.grant(granted(EVOLUTION, Facility::Secrets));

    ours.for_the_application(&asking(FRACTAL), &grants, noon())
        .unwrap()
        .keep("matrix", FRACTALS_TOKEN)
        .unwrap();
    ours.for_the_application(&asking(EVOLUTION), &grants, noon())
        .unwrap()
        .keep("imap", EVOLUTIONS_TOKEN)
        .unwrap();

    // A request allowed before the revocation, in flight. Its judgement was
    // made against the grants as they were, and it does its one thing: every
    // operation takes it by value, so it cannot be used for a second.
    let already_allowed = ours
        .for_the_application(&asking(FRACTAL), &grants, noon())
        .unwrap();

    // Revoked the way a person revokes: the row on the one list.
    assert_eq!(revoke_from_the_list(&mut grants, fractals), Revoked::Now);

    assert_eq!(
        already_allowed.kept("matrix").unwrap().bytes(),
        FRACTALS_TOKEN
    );

    // The next request is refused — and every operation there is starts with
    // one, so none of them is reachable.
    assert!(
        matches!(
            ours.for_the_application(&asking(FRACTAL), &grants, noon()),
            Err(Withheld::Refused(Refused::NothingGranted { .. }))
        ),
        "Fractal reached its secrets after its grant was revoked"
    );

    // Evolution's grant, and its secret, are untouched.
    assert_eq!(
        ours.for_the_application(&asking(EVOLUTION), &grants, noon())
            .unwrap()
            .kept("imap")
            .unwrap()
            .bytes(),
        EVOLUTIONS_TOKEN
    );

    // The password itself is not destroyed by revoking — the person's keyring
    // still holds it — so granting again finds the sign-in where it was.
    let items = everything_in_the_default_collection(&keyring);
    assert!(
        items
            .iter()
            .any(|attributes| attributes.get("application").map(String::as_str) == Some(FRACTAL)),
        "revoking deleted what the application kept: {items:?}"
    );
    grants.grant(granted(FRACTAL, Facility::Secrets));
    assert_eq!(
        ours.for_the_application(&asking(FRACTAL), &grants, noon())
            .unwrap()
            .kept("matrix")
            .unwrap()
            .bytes(),
        FRACTALS_TOKEN
    );
}

/// Revoke a grant through the row the one list shows for it.
fn revoke_from_the_list(grants: &mut Grants, id: GrantId) -> Revoked {
    let listing = Listing::of(grants, noon());
    let row = listing
        .rows()
        .iter()
        .find(|row| row.id() == id)
        .expect("the grant is on the list")
        .clone();
    assert_eq!(row.to(), FRACTAL);
    row.revoke(grants)
}

/// Set on the child that checks the fixture with decoys in its environment.
const WITH_DECOYS: &str = "ALO_ONE_KEYRING_WITH_DECOYS";

/// What the child prints once it has checked, so a child that ran nothing
/// cannot pass for one that did.
const CHECKED: &str = "alo:one-keyring-checked";

/// The name the Secret Service owns on a bus.
const THE_SECRET_SERVICE: &str = "org.freedesktop.secrets";

/// **The fixture is the one Secret Service on its bus**, with a decoy Secret
/// Service activation file in every place a session bus looks for one.
///
/// This is the race `docs/quirks.md` recorded on 2026-09-13: a bus the fixture
/// started with `--session` listed `org.freedesktop.secrets` as activatable,
/// and a slow fixture keyring lost its name to the machine's. On the build
/// machine that file is diverted, so an assertion made against the machine as
/// it is would pass whether or not the fixture were fixed. So the decoys are
/// put where the old bus read them — `XDG_DATA_HOME` and `XDG_DATA_DIRS` — on a
/// child process (setting the environment of this one is `unsafe` in this
/// edition), and the child first proves they are live on a plain session bus.
#[test]
fn the_fixture_is_the_one_secret_service_on_its_bus() {
    if std::env::var_os(WITH_DECOYS).is_some() {
        with_decoys_in_the_environment();
        println!("{CHECKED}");
        return;
    }

    let place = std::env::temp_dir().join(format!("alo-one-keyring-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&place));
    for under in ["home", "dirs"] {
        let services = place.join(under).join("dbus-1").join("services");
        std::fs::create_dir_all(&services).unwrap();
        std::fs::write(
            services.join("org.freedesktop.secrets.service"),
            "[D-BUS Service]\nName=org.freedesktop.secrets\nExec=/bin/false\n",
        )
        .unwrap();
    }

    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "the_fixture_is_the_one_secret_service_on_its_bus",
            "--nocapture",
        ])
        .env(WITH_DECOYS, "1")
        .env("XDG_DATA_HOME", place.join("home"))
        .env("XDG_DATA_DIRS", place.join("dirs"))
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&child.stdout);
    let complained = String::from_utf8_lossy(&child.stderr);
    drop(std::fs::remove_dir_all(&place));
    assert!(
        child.status.success() && said.contains(CHECKED),
        "the child did not find one keyring on the fixture's bus:\n{said}\n{complained}"
    );
}

/// The child's half.
fn with_decoys_in_the_environment() {
    // The decoys are live: a session bus started the old way lists them.
    let control =
        std::env::temp_dir().join(format!("alo-one-keyring-control-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&control));
    std::fs::create_dir_all(&control).unwrap();
    let at = control.join("bus");
    let mut old_way = Command::new("dbus-daemon")
        .arg("--session")
        .arg("--address")
        .arg(format!("unix:path={}", at.display()))
        .arg("--nofork")
        .arg("--nopidfile")
        .spawn()
        .expect("dbus-daemon is installed");
    let activatable = wait_to_ask(&at, activatable_on);
    drop(old_way.kill());
    drop(old_way.wait());
    drop(std::fs::remove_dir_all(&control));
    assert!(
        activatable.iter().any(|name| name == THE_SECRET_SERVICE),
        "the decoy is not live, so this test would measure nothing: {activatable:?}"
    );

    // The fixture's bus, in the same environment, can start no Secret Service…
    let keyring = AKeyringOfOurOwn::started("one-keyring");
    let bus = zbus::blocking::connection::Builder::address(keyring.address().as_str())
        .unwrap()
        .build()
        .unwrap();
    let activatable = activatable_on(&bus);
    assert!(
        !activatable.iter().any(|name| name == THE_SECRET_SERVICE),
        "the fixture's bus can start a second Secret Service: {activatable:?}"
    );

    // …has exactly one owner of the name, waiting behind nobody…
    let owners: Vec<String> = asked(&bus, "ListQueuedOwners").deserialize().unwrap();
    assert_eq!(owners.len(), 1, "{owners:?}");

    // …and that owner is the fixture's own keyring.
    let serving: u32 = asked(&bus, "GetConnectionUnixProcessID")
        .deserialize()
        .unwrap();
    assert_eq!(Some(serving), keyring.keyring_process());

    // And what alo-secrets opens is that keyring.
    assert!(TheKeyring::opened(&keyring.bus()).is_ok());
}

/// What a bus says it could start.
fn activatable_on(bus: &zbus::blocking::Connection) -> Vec<String> {
    bus.call_method(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        Some("org.freedesktop.DBus"),
        "ListActivatableNames",
        &(),
    )
    .unwrap()
    .body()
    .deserialize()
    .unwrap()
}

/// One question to the bus about the Secret Service's name.
fn asked(bus: &zbus::blocking::Connection, method: &str) -> zbus::message::Body {
    bus.call_method(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        Some("org.freedesktop.DBus"),
        method,
        &(THE_SECRET_SERVICE,),
    )
    .unwrap()
    .body()
}

/// Connect to a bus once its socket is there, and ask it something.
fn wait_to_ask<T>(at: &Path, asking: impl Fn(&zbus::blocking::Connection) -> T) -> T {
    let until = std::time::Instant::now() + Duration::from_secs(15);
    while std::time::Instant::now() < until {
        if let Ok(bus) = zbus::blocking::connection::Builder::address(
            format!("unix:path={}", at.display()).as_str(),
        )
        .and_then(zbus::blocking::connection::Builder::build)
        {
            return asking(&bus);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("the control bus never appeared at {}", at.display());
}
