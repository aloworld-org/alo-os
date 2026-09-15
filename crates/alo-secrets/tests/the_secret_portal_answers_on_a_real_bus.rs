//! `org.freedesktop.portal.Secret`, served on a real bus and asked by a real
//! client.
//!
//! Task 5 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`,
//! for the Secret portal: the backend `alo-portals` serves answers on a private
//! session bus the test starts, a `zbus` client — not a mock — calls
//! `RetrieveSecret` with a pipe as the specification says, and receives what
//! task 3 decided:
//!
//! - **a granted application is written its own portal secret, the one the
//!   keyring keeps for it, and the response is `0`** —
//!   [`a_granted_application_is_handed_its_own_secret_through_the_portal`];
//! - **an application granted nothing receives the portal's own refusal
//!   response, nothing is written, and the record carries it as refused with
//!   the application named** — and beside it the other refusals: a grant of
//!   something else, a revoked grant, a program with no sandbox and a request
//!   that names itself wrongly —
//!   [`an_application_granted_nothing_is_refused_on_the_bus_and_recorded_by_name`].
//!
//! Who is asking is read from a `.flatpak-info` in a directory laid out as
//! `/proc` is, at this test's own process number: the bus says which process
//! sent each request, and the backend reads that process's sandbox there, as it
//! reads `/proc` on a machine. Changing the file between requests is how one
//! test process is two applications.
//!
//! **Every secret here is synthetic**, in a keyring this test starts and kills.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;
use std::io::Read;
use std::os::fd::AsFd;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use alo_capability::{Applicant, Facility, Grant, GrantId, Grants, Reach};
use alo_keyring_fixture::AKeyringOfOurOwn;
use alo_portals::{
    Answered, Appearance, Applications, Backend, Kept, Outcome, Portal, Refused, Request,
    Sandboxes, Served, TheMachine, TimeOfDay, Unanswered,
};
use alo_secrets::{PORTAL_SECRET_LENGTH, THE_PORTALS, TheKeyring};
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{Fd, OwnedObjectPath, OwnedValue, Value};

const FRACTAL: &str = "org.gnome.Fractal";
const EVOLUTION: &str = "org.gnome.Evolution";
const STRANGER: &str = "org.example.Stranger";

const DESKTOP: &str = "org.freedesktop.portal.Desktop";
const PORTALS: &str = "/org/freedesktop/portal/desktop";

/// The machine as the backend reads it: grants a test can revoke from.
struct ThisMachine {
    grants: RwLock<Grants>,
}

impl TheMachine for ThisMachine {
    fn grants(&self) -> Option<Grants> {
        Some(self.grants.read().unwrap().clone())
    }

    fn applications(&self) -> Option<Applications> {
        Some(Applications::default())
    }

    fn appearance(&self) -> Option<Appearance> {
        Some(Appearance::shipped())
    }

    fn time_of_day(&self) -> Option<TimeOfDay> {
        TimeOfDay::checked(12, 0).ok()
    }
}

/// Everything one test serves and asks through.
struct Serving {
    keyring: AKeyringOfOurOwn,
    machine: Arc<ThisMachine>,
    record: Arc<Kept>,
    processes: PathBuf,
    client: Connection,
    _served: Served,
}

impl Serving {
    fn started(what: &str) -> Self {
        let keyring = AKeyringOfOurOwn::started(what);
        let processes =
            std::env::temp_dir().join(format!("alo-secret-portal-{what}-{}", std::process::id()));
        std::fs::create_dir_all(processes.join(std::process::id().to_string()).join("root"))
            .unwrap();
        let machine = Arc::new(ThisMachine {
            grants: RwLock::new(Grants::default()),
        });
        let record = Arc::new(Kept::nothing());
        let served = Backend::answering_from(
            machine.clone(),
            Arc::new(TheKeyring::opened(&keyring.bus()).expect("the keyring opens")),
            Sandboxes::under(&processes),
            record.clone(),
        )
        .serve_on(&keyring.address())
        .expect("the portals are served on the test's bus");
        let client = zbus::blocking::connection::Builder::address(keyring.address().as_str())
            .unwrap()
            .build()
            .unwrap();
        Self {
            keyring,
            machine,
            record,
            processes,
            client,
            _served: served,
        }
    }

    /// From now on, this test process is `application`'s sandbox — or, for
    /// [`None`], a program with no sandbox at all.
    fn become_(&self, application: Option<&str>) {
        let info = self
            .processes
            .join(std::process::id().to_string())
            .join("root")
            .join(".flatpak-info");
        match application {
            Some(name) => std::fs::write(info, format!("[Application]\nname={name}\n")).unwrap(),
            None => {
                let _ = std::fs::remove_file(info);
            }
        }
    }

    fn grant(&self, application: &str, facility: Facility) -> GrantId {
        self.machine.grants.write().unwrap().grant(
            Grant::checked_for(
                &Applicant::named(application).grantee(),
                Reach::Facility(facility),
                SystemTime::now(),
                Duration::from_secs(60 * 60),
            )
            .unwrap(),
        )
    }

    /// `RetrieveSecret`, as a client following the specification makes it: the
    /// response is listened for on the predicted handle before the call, and
    /// the pipe's own end is closed once the call is made. The response code,
    /// and every byte written.
    fn retrieve_secret(&self, token: &str) -> (u32, Vec<u8>) {
        let sender = self.client.unique_name().unwrap().as_str();
        let handle = format!(
            "{PORTALS}/request/{}/{token}",
            sender.trim_start_matches(':').replace('.', "_")
        );
        let request = Proxy::new(
            &self.client,
            DESKTOP,
            handle.as_str(),
            "org.freedesktop.portal.Request",
        )
        .unwrap();
        let mut responses = request.receive_signal("Response").unwrap();

        let (mut reader, writer) = std::io::pipe().unwrap();
        let mut options: HashMap<&str, Value<'_>> = HashMap::new();
        options.insert("handle_token", Value::from(token));
        let reply = self
            .client
            .call_method(
                Some(DESKTOP),
                PORTALS,
                Some("org.freedesktop.portal.Secret"),
                "RetrieveSecret",
                &(Fd::from(writer.as_fd()), options),
            )
            .expect("the Secret portal answers the call");
        let returned: OwnedObjectPath = reply.body().deserialize().unwrap();
        assert_eq!(returned.as_str(), handle, "the handle is the predicted one");
        drop(writer);

        let response = responses.next().expect("a response");
        let (code, results): (u32, HashMap<String, OwnedValue>) =
            response.body().deserialize().unwrap();
        assert!(results.is_empty(), "{results:?}");
        let mut written = Vec::new();
        reader.read_to_end(&mut written).unwrap();
        (code, written)
    }

    /// Task 3's door, asked directly: the portal secret the keyring keeps for
    /// `application`.
    fn kept_for(&self, application: &str) -> Vec<u8> {
        let grants = self.machine.grants().unwrap();
        TheKeyring::opened(&self.keyring.bus())
            .unwrap()
            .for_the_application(
                &Request::of(application, Portal::Secret).unwrap(),
                &grants,
                SystemTime::now(),
            )
            .unwrap()
            .the_portals_secret()
            .unwrap()
            .bytes()
            .to_vec()
    }

    fn last_answer(&self) -> Answered {
        self.record
            .everything()
            .pop()
            .expect("an answer was recorded")
    }
}

impl Drop for Serving {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.processes);
    }
}

/// How many portal secrets the keyring holds for `application`, read by a
/// client that is not the one under test.
fn portal_secrets_of(keyring: &AKeyringOfOurOwn, application: &str) -> usize {
    let connection = zbus::blocking::connection::Builder::address(keyring.address().as_str())
        .unwrap()
        .build()
        .unwrap();
    let service = secret_service::blocking::SecretService::connect_with_existing(
        secret_service::EncryptionType::Dh,
        connection,
    )
    .unwrap();
    let collection = service.get_default_collection().unwrap();
    let mut attributes = HashMap::new();
    attributes.insert("xdg:schema", THE_PORTALS);
    attributes.insert("application", application);
    collection.search_items(attributes).unwrap().len()
}

/// **A granted application is written its own portal secret — the one task 3's
/// keyring keeps for it — and answered `0`.**
#[test]
fn a_granted_application_is_handed_its_own_secret_through_the_portal() {
    let serving = Serving::started("secret-portal-granted");
    let fractals = serving.grant(FRACTAL, Facility::Secrets);
    let evolutions = serving.grant(EVOLUTION, Facility::Secrets);

    serving.become_(Some(FRACTAL));
    let (code, written) = serving.retrieve_secret("fractal_first");
    assert_eq!(code, 0);
    assert_eq!(written.len(), PORTAL_SECRET_LENGTH);
    assert_eq!(
        written,
        serving.kept_for(FRACTAL),
        "the portal wrote something other than the secret the keyring keeps for Fractal"
    );
    let answered = serving.last_answer();
    assert_eq!(answered.portal(), Portal::Secret);
    assert_eq!(answered.application(), Some(&Applicant::named(FRACTAL)));
    match answered.outcome() {
        Outcome::SecretHandedOver(allowed) => assert_eq!(allowed.against(), [fractals]),
        other => panic!("{other:?}"),
    }

    // Asked again, the same secret, and still one kept.
    let (code, again) = serving.retrieve_secret("fractal_again");
    assert_eq!((code, &again), (0, &written));
    assert_eq!(portal_secrets_of(&serving.keyring, FRACTAL), 1);

    // Another application is handed its own, and never Fractal's.
    serving.become_(Some(EVOLUTION));
    let (code, evolutions_secret) = serving.retrieve_secret("evolution_first");
    assert_eq!(code, 0);
    assert_ne!(evolutions_secret, written);
    assert_eq!(evolutions_secret, serving.kept_for(EVOLUTION));
    match serving.last_answer().outcome() {
        Outcome::SecretHandedOver(allowed) => {
            assert_eq!(allowed.application(), &Applicant::named(EVOLUTION));
            assert_eq!(allowed.against(), [evolutions]);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(serving.record.everything().len(), 3);
}

/// **An application granted nothing receives the portal's refusal, is written
/// nothing, and the record says it was refused and names it** — and so is every
/// other request no grant covers, or that nothing could say the source of.
#[test]
fn an_application_granted_nothing_is_refused_on_the_bus_and_recorded_by_name() {
    let serving = Serving::started("secret-portal-refused");

    // Granted nothing.
    serving.become_(Some(STRANGER));
    let (code, written) = serving.retrieve_secret("stranger");
    assert_eq!(code, 2, "the specification's *ended some other way*");
    assert!(written.is_empty(), "a refused application was written to");
    let answered = serving.last_answer();
    assert_eq!(answered.application(), Some(&Applicant::named(STRANGER)));
    assert_eq!(answered.portal(), Portal::Secret);
    assert!(answered.outcome().was_refused());
    assert_eq!(
        answered.outcome(),
        &Outcome::Refused(Refused::NothingGranted {
            application: Applicant::named(STRANGER),
            portal: Portal::Secret,
        })
    );
    assert_eq!(
        portal_secrets_of(&serving.keyring, STRANGER),
        0,
        "a refused request reached the keyring"
    );

    // Granted the camera, which is not its secrets.
    serving.grant(STRANGER, Facility::Camera);
    let (code, written) = serving.retrieve_secret("stranger_camera");
    assert_eq!((code, written.len()), (2, 0));
    let answered = serving.last_answer();
    assert_eq!(answered.application(), Some(&Applicant::named(STRANGER)));
    assert!(
        matches!(
            answered.outcome(),
            Outcome::Refused(Refused::NotAllowed { .. })
        ),
        "{answered:?}"
    );

    // Granted, and revoked: refused at the very next request.
    serving.become_(Some(FRACTAL));
    let fractals = serving.grant(FRACTAL, Facility::Secrets);
    assert_eq!(serving.retrieve_secret("fractal_granted").0, 0);
    assert!(serving.machine.grants.write().unwrap().revoke(fractals));
    let (code, written) = serving.retrieve_secret("fractal_revoked");
    assert_eq!((code, written.len()), (2, 0));
    let answered = serving.last_answer();
    assert_eq!(answered.application(), Some(&Applicant::named(FRACTAL)));
    assert!(answered.outcome().was_refused(), "{answered:?}");

    // A program with no sandbox is nobody, and named as nobody.
    serving.become_(None);
    let (code, written) = serving.retrieve_secret("no_sandbox");
    assert_eq!((code, written.len()), (2, 0));
    let answered = serving.last_answer();
    assert_eq!(answered.application(), None);
    assert_eq!(
        answered.outcome(),
        &Outcome::Unanswered(Unanswered::NotIdentified)
    );

    // A request that names its handle with a path in it is refused on the call,
    // and recorded with the application that made it.
    serving.become_(Some(FRACTAL));
    let (_reader, writer) = std::io::pipe().unwrap();
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert("handle_token", Value::from("../1_1/theirs"));
    let refused = serving
        .client
        .call_method(
            Some(DESKTOP),
            PORTALS,
            Some("org.freedesktop.portal.Secret"),
            "RetrieveSecret",
            &(Fd::from(writer.as_fd()), options),
        )
        .expect_err("a token with a path in it was answered");
    assert!(
        matches!(&refused, zbus::Error::MethodError(name, _, _) if name.as_str() == "org.freedesktop.DBus.Error.InvalidArgs"),
        "{refused:?}"
    );
    let answered = serving.last_answer();
    assert_eq!(answered.application(), Some(&Applicant::named(FRACTAL)));
    assert_eq!(
        answered.outcome(),
        &Outcome::Unanswered(Unanswered::NotAToken)
    );

    // Every request above, and nothing else, is in the record — each refusal
    // with its application, and none of them a secret handed over but one.
    let everything = serving.record.everything();
    assert_eq!(everything.len(), 6);
    assert_eq!(
        everything
            .iter()
            .filter(|answered| !answered.outcome().was_refused())
            .count(),
        1
    );
}
