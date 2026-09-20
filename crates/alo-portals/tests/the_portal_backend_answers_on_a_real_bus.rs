//! The portal backend on a real bus: `OpenURI` answered from what opens what,
//! and every portal nothing here decides not registered at all.
//!
//! Task 5 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`.
//! The backend is served on a private session bus the test starts (with no
//! service activation, `alo-keyring-fixture`'s bus), and a `zbus` client — not a
//! mock — asks it as an application does:
//!
//! - **an `OpenFile` request is answered from what task 4 decided: the file is
//!   opened in the application that opens its kind, which receives it over the
//!   bus, and the response is `0`** —
//!   [`a_granted_file_is_opened_in_what_opens_it_and_answered_as_done`];
//! - **an application granted nothing receives the portal's refusal response,
//!   nothing is opened, and the record carries it as refused with the
//!   application named** — beside it a file not granted, an opener not granted,
//!   a file nothing opens, a program with no sandbox, something that is not a
//!   file, an opener that does not answer, and a web link —
//!   [`every_request_no_grant_covers_is_refused_on_the_bus_and_recorded_by_name`];
//! - **the file chooser and everything that draws is not served: the backend
//!   declines by not registering** (the Settings portal, answered since task 6,
//!   is among those registered, and has its own test) —
//!   [`only_the_portals_decided_here_are_registered_on_the_bus`];
//! - **`docs/contracts/` gains the one page describing which portals this
//!   machine answers and which it does not yet** —
//!   [`the_contract_names_every_portal_answered_and_every_one_not`].
//!
//! The "opener" is a real service on the bus, serving
//! `org.freedesktop.Application` as a D-Bus-activatable application does, so
//! what it was asked to open is what it received and not what the backend
//! claims to have sent.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_portals::Portal;

/// `docs/contracts/portals.md`, from this crate.
const THE_CONTRACT: &str = "../../docs/contracts/portals.md";

/// **The contract lists every portal, each in the table it belongs in**: the
/// ones this machine answers, with their interface, and the ones it does not
/// yet.
#[test]
fn the_contract_names_every_portal_answered_and_every_one_not() {
    let contract =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(THE_CONTRACT))
            .expect("docs/contracts/portals.md is there");
    let (answered, rest) = contract
        .split_once("## Answered")
        .and_then(|(_, rest)| rest.split_once("## Not answered yet"))
        .expect("the contract has an Answered section, then a Not answered yet section");
    let not_yet = rest.split("\n## ").next().unwrap_or(rest);
    let row_of = |table: &str, portal: Portal| {
        table
            .lines()
            .find(|line| {
                line.starts_with('|') && line.contains(&format!("| {} |", portal.promised_as()))
            })
            .map(str::to_owned)
    };
    for portal in Portal::EVERY {
        match portal.answered_on_the_bus() {
            Some(interface) => {
                let row = row_of(answered, portal)
                    .unwrap_or_else(|| panic!("{portal:?} is answered and not listed as answered"));
                assert!(row.contains(interface), "{row}");
                assert!(
                    row_of(not_yet, portal).is_none(),
                    "{portal:?} is listed twice"
                );
            }
            None => {
                assert!(
                    row_of(not_yet, portal).is_some(),
                    "{portal:?} is not answered and not listed as not answered yet"
                );
                assert!(
                    row_of(answered, portal).is_none(),
                    "{portal:?} is listed as answered"
                );
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod on_the_bus {
    use std::collections::HashMap;
    use std::fs::File;
    use std::os::fd::AsFd;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex, RwLock};
    use std::time::{Duration, SystemTime};

    use alo_applications::{Application, Chosen, Declared, Installed};
    use alo_capability::{Applicant, Grant, Grants, Reach};
    use alo_keyring_fixture::AKeyringOfOurOwn;
    use alo_portals::{
        Allowed, Answered, Appearance, Applications, Backend, KeepsSecrets, Kept, NotKept, Outcome,
        Portal, Refused, Request, Sandboxes, Served, TheMachine, TimeOfDay, Unanswered,
    };
    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::{Fd, OwnedObjectPath, OwnedValue, Value};

    const MAIL: &str = "org.gnome.Geary";
    const STRANGER: &str = "org.example.Stranger";
    const PAPERS: &str = "org.gnome.Papers";
    const OKULAR: &str = "org.kde.okular";

    const DESKTOP: &str = "org.freedesktop.portal.Desktop";
    const PORTALS: &str = "/org/freedesktop/portal/desktop";
    const OPEN_URI: &str = "org.freedesktop.portal.OpenURI";

    const A_PDF: &[u8] = b"%PDF-1.7\n1 0 obj << >> endobj\n%%EOF\n";
    const A_PROGRAM: &[u8] = b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0";

    /// The machine as the backend reads it: grants a test changes, and Papers
    /// and Okular installed, each declaring PDFs, Papers first.
    struct ThisMachine {
        grants: RwLock<Grants>,
    }

    impl TheMachine for ThisMachine {
        fn grants(&self) -> Option<Grants> {
            Some(self.grants.read().unwrap().clone())
        }

        fn applications(&self) -> Option<Applications> {
            let papers = Application::called(PAPERS, "Papers").unwrap();
            let okular = Application::called(OKULAR, "Okular").unwrap();
            let mut declared = Declared::nothing();
            declared.declares_media_types(&papers, ["application/pdf"]);
            declared.declares_media_types(&okular, ["application/pdf"]);
            Some(Applications {
                installed: Installed::holding([papers, okular]),
                declared,
                chosen: Chosen::untouched(),
            })
        }

        fn appearance(&self) -> Option<Appearance> {
            Some(Appearance::shipped())
        }

        fn time_of_day(&self) -> Option<TimeOfDay> {
            TimeOfDay::checked(12, 0).ok()
        }

        /// This test asks nothing of the network monitor portal, and a machine
        /// that cannot say how far it reaches says so.
        fn reaching(&self) -> Option<alo_portals::Reaching> {
            None
        }
    }

    /// No keyring: this test asks nothing of the Secret portal.
    struct NoKeyring;

    impl KeepsSecrets for NoKeyring {
        fn hand_over(
            &self,
            _request: &Request,
            _grants: &Grants,
            _now: SystemTime,
            _into: &mut dyn std::io::Write,
        ) -> Result<Allowed, NotKept> {
            Err(NotKept::Unavailable)
        }
    }

    /// An activatable application on the bus, keeping every URI it is sent.
    struct AnApplication {
        opened: Arc<Mutex<Vec<String>>>,
    }

    #[zbus::interface(name = "org.freedesktop.Application")]
    impl AnApplication {
        fn open(&self, uris: Vec<String>, _platform_data: HashMap<String, OwnedValue>) {
            self.opened.lock().unwrap().extend(uris);
        }
    }

    /// Everything one test serves and asks through.
    struct Serving {
        _bus: AKeyringOfOurOwn,
        machine: Arc<ThisMachine>,
        record: Arc<Kept>,
        processes: PathBuf,
        files: PathBuf,
        client: Connection,
        papers_opened: Arc<Mutex<Vec<String>>>,
        papers: Connection,
        _served: Served,
    }

    impl Serving {
        fn started(what: &str) -> Self {
            let bus = AKeyringOfOurOwn::a_bus_with_no_keyring_on_it(what);
            let place = std::env::temp_dir()
                .join(format!("alo-portal-backend-{what}-{}", std::process::id()));
            let processes = place.join("proc");
            std::fs::create_dir_all(processes.join(std::process::id().to_string()).join("root"))
                .unwrap();
            let files = place.join("Invoices");
            std::fs::create_dir_all(&files).unwrap();
            let files = files.canonicalize().unwrap();

            let machine = Arc::new(ThisMachine {
                grants: RwLock::new(Grants::default()),
            });
            let record = Arc::new(Kept::nothing());
            let served = Backend::answering_from(
                machine.clone(),
                Arc::new(NoKeyring),
                Sandboxes::under(&processes),
                record.clone(),
            )
            .serve_on(&bus.address())
            .expect("the portals are served on the test's bus");

            let papers_opened = Arc::new(Mutex::new(Vec::new()));
            let papers = zbus::blocking::connection::Builder::address(bus.address().as_str())
                .unwrap()
                .name(PAPERS)
                .unwrap()
                .serve_at(
                    "/org/gnome/Papers",
                    AnApplication {
                        opened: papers_opened.clone(),
                    },
                )
                .unwrap()
                .build()
                .expect("Papers is on the bus");
            let client = zbus::blocking::connection::Builder::address(bus.address().as_str())
                .unwrap()
                .build()
                .unwrap();
            Self {
                _bus: bus,
                machine,
                record,
                processes,
                files,
                client,
                papers_opened,
                papers,
                _served: served,
            }
        }

        /// From now on this test process is `application`'s sandbox, or for
        /// [`None`] a program with none.
        fn become_(&self, application: Option<&str>) {
            let info = self
                .processes
                .join(std::process::id().to_string())
                .join("root")
                .join(".flatpak-info");
            match application {
                Some(name) => {
                    std::fs::write(info, format!("[Application]\nname={name}\n")).unwrap();
                }
                None => {
                    let _ = std::fs::remove_file(info);
                }
            }
        }

        fn grant(&self, application: &str, reach: Reach) {
            self.machine.grants.write().unwrap().grant(
                Grant::checked_for(
                    &Applicant::named(application).grantee(),
                    reach,
                    SystemTime::now(),
                    Duration::from_secs(60 * 60),
                )
                .unwrap(),
            );
        }

        fn a_file(&self, named: &str, bytes: &[u8]) -> PathBuf {
            let path = self.files.join(named);
            std::fs::write(&path, bytes).unwrap();
            path
        }

        /// A request made by `call`, as the specification says to make one:
        /// listening on the predicted handle first. The response code.
        fn ask(
            &self,
            token: &str,
            call: impl FnOnce(&Connection) -> zbus::Result<zbus::Message>,
        ) -> u32 {
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
            let reply = call(&self.client).expect("OpenURI answers the call");
            let returned: OwnedObjectPath = reply.body().deserialize().unwrap();
            assert_eq!(returned.as_str(), handle);
            let response = responses.next().expect("a response");
            let (code, _results): (u32, HashMap<String, OwnedValue>) =
                response.body().deserialize().unwrap();
            code
        }

        /// `OpenFile` for the file at `path`, handed over `O_PATH`-like: opened
        /// as the application would, and passed by descriptor.
        fn open_file(&self, path: &Path, token: &str) -> u32 {
            let file = File::open(path).unwrap();
            self.ask(token, |client| {
                client.call_method(
                    Some(DESKTOP),
                    PORTALS,
                    Some(OPEN_URI),
                    "OpenFile",
                    &("", Fd::from(file.as_fd()), options(token)),
                )
            })
        }

        fn last_answer(&self) -> Answered {
            self.record
                .everything()
                .pop()
                .expect("an answer was recorded")
        }

        fn papers_opened(&self) -> Vec<String> {
            self.papers_opened.lock().unwrap().clone()
        }
    }

    impl Drop for Serving {
        fn drop(&mut self) {
            if let Some(place) = self.processes.parent() {
                let _ = std::fs::remove_dir_all(place);
            }
        }
    }

    fn options(token: &str) -> HashMap<&'static str, Value<'_>> {
        let mut options = HashMap::new();
        options.insert("handle_token", Value::from(token));
        options
    }

    /// **A granted file is opened in the application that opens its kind, which
    /// receives it, and the application is answered `0`.**
    #[test]
    fn a_granted_file_is_opened_in_what_opens_it_and_answered_as_done() {
        let serving = Serving::started("open-file-granted");
        let invoice = serving.a_file("march 2026.pdf", A_PDF);
        serving.grant(MAIL, Reach::Folder(serving.files.clone()));
        serving.grant(MAIL, Reach::Application(PAPERS.to_owned()));
        serving.become_(Some(MAIL));

        assert_eq!(serving.open_file(&invoice, "open_invoice"), 0);
        let expected = format!("file://{}", invoice.to_str().unwrap().replace(' ', "%20"));
        assert_eq!(serving.papers_opened(), [expected]);

        let answered = serving.last_answer();
        assert_eq!(answered.portal(), Portal::OpenWith);
        assert_eq!(answered.application(), Some(&Applicant::named(MAIL)));
        match answered.outcome() {
            Outcome::Opened(opens) => {
                assert_eq!(opens.opener().application().identifier(), PAPERS);
                assert_eq!(
                    opens.allowed().against().len(),
                    2,
                    "the file and its opener"
                );
            }
            other => panic!("{other:?}"),
        }
    }

    /// **Every request no grant covers is answered with the portal's refusal,
    /// opens nothing, and is recorded as refused with its application named.**
    #[test]
    fn every_request_no_grant_covers_is_refused_on_the_bus_and_recorded_by_name() {
        let serving = Serving::started("open-file-refused");
        let invoice = serving.a_file("march.pdf", A_PDF);

        // Granted nothing at all.
        serving.become_(Some(STRANGER));
        assert_eq!(serving.open_file(&invoice, "stranger"), 2);
        let answered = serving.last_answer();
        assert_eq!(answered.application(), Some(&Applicant::named(STRANGER)));
        assert_eq!(
            answered.outcome(),
            &Outcome::Refused(Refused::NothingGranted {
                application: Applicant::named(STRANGER),
                portal: Portal::OpenWith,
            })
        );

        // Granted the opener, not the file.
        serving.become_(Some(MAIL));
        serving.grant(MAIL, Reach::Application(PAPERS.to_owned()));
        assert_eq!(serving.open_file(&invoice, "file_not_granted"), 2);
        let answered = serving.last_answer();
        assert_eq!(answered.application(), Some(&Applicant::named(MAIL)));
        assert!(
            matches!(
                answered.outcome(),
                Outcome::Refused(Refused::NotAllowed { .. })
            ),
            "{answered:?}"
        );

        // Granted the folder too: a file whose kind nothing opens is refused as
        // that, and never opened in whatever is granted.
        serving.grant(MAIL, Reach::Folder(serving.files.clone()));
        let program = serving.a_file("setup.bin", A_PROGRAM);
        assert_eq!(serving.open_file(&program, "a_program"), 2);
        assert!(
            matches!(serving.last_answer().outcome(), Outcome::NothingOpens(_)),
            "{:?}",
            serving.last_answer()
        );

        // Granted the file, and not the application that opens it.
        let serving_okular = Serving::started("open-file-opener-not-granted");
        let other_invoice = serving_okular.a_file("april.pdf", A_PDF);
        serving_okular.grant(MAIL, Reach::Folder(serving_okular.files.clone()));
        serving_okular.grant(MAIL, Reach::Application(OKULAR.to_owned()));
        serving_okular.become_(Some(MAIL));
        assert_eq!(
            serving_okular.open_file(&other_invoice, "opener_not_granted"),
            2
        );
        let answered = serving_okular.last_answer();
        assert_eq!(answered.application(), Some(&Applicant::named(MAIL)));
        assert!(
            matches!(
                answered.outcome(),
                Outcome::Refused(Refused::NotAllowed { .. })
            ),
            "{answered:?}"
        );
        assert!(serving_okular.papers_opened().is_empty());

        // A program with no sandbox is nobody.
        serving.become_(None);
        assert_eq!(serving.open_file(&invoice, "no_sandbox"), 2);
        let answered = serving.last_answer();
        assert_eq!(answered.application(), None);
        assert_eq!(
            answered.outcome(),
            &Outcome::Unanswered(Unanswered::NotIdentified)
        );

        // Something that is not a file.
        serving.become_(Some(MAIL));
        let folder = File::open(&serving.files).unwrap();
        let code = serving.ask("a_folder", |client| {
            client.call_method(
                Some(DESKTOP),
                PORTALS,
                Some(OPEN_URI),
                "OpenFile",
                &("", Fd::from(folder.as_fd()), options("a_folder")),
            )
        });
        assert_eq!(code, 2);
        assert_eq!(
            serving.last_answer().outcome(),
            &Outcome::Unanswered(Unanswered::NotAFile)
        );

        // A web link, and a folder: nothing here decides what opens them.
        let code = serving.ask("a_link", |client| {
            client.call_method(
                Some(DESKTOP),
                PORTALS,
                Some(OPEN_URI),
                "OpenURI",
                &("", "https://example.org/", options("a_link")),
            )
        });
        assert_eq!(code, 2);
        let answered = serving.last_answer();
        assert_eq!(answered.application(), Some(&Applicant::named(MAIL)));
        assert_eq!(
            answered.outcome(),
            &Outcome::Unanswered(Unanswered::NotDecidedHere)
        );
        let in_it = File::open(&invoice).unwrap();
        let code = serving.ask("the_folder", |client| {
            client.call_method(
                Some(DESKTOP),
                PORTALS,
                Some(OPEN_URI),
                "OpenDirectory",
                &("", Fd::from(in_it.as_fd()), options("the_folder")),
            )
        });
        assert_eq!(code, 2);
        assert_eq!(
            serving.last_answer().outcome(),
            &Outcome::Unanswered(Unanswered::NotDecidedHere)
        );

        // Nothing above was opened.
        assert!(
            serving.papers_opened().is_empty(),
            "{:?}",
            serving.papers_opened()
        );
        let everything = serving.record.everything();
        assert_eq!(everything.len(), 7);
        assert!(
            everything
                .iter()
                .all(|answered| answered.outcome().was_refused())
        );
    }

    /// **An opener that does not answer is not claimed to have opened
    /// anything.**
    #[test]
    fn an_opener_that_does_not_answer_is_recorded_and_refused() {
        let serving = Serving::started("open-file-opener-absent");
        let invoice = serving.a_file("may.pdf", A_PDF);
        serving.grant(MAIL, Reach::Folder(serving.files.clone()));
        serving.grant(MAIL, Reach::Application(PAPERS.to_owned()));
        serving.become_(Some(MAIL));
        assert!(serving.papers.release_name(PAPERS).unwrap());

        assert_eq!(serving.open_file(&invoice, "nobody_opens"), 2);
        assert_eq!(
            serving.last_answer().outcome(),
            &Outcome::Unanswered(Unanswered::NotOpened {
                opener: PAPERS.to_owned()
            })
        );
        assert!(serving.papers_opened().is_empty());
    }

    /// **Only the portals decided here are registered.** The object serves
    /// `Secret`, `OpenURI` and `Settings` and no other portal, and asking the file chooser
    /// is answered by the bus as a portal nothing answers — never with yes.
    #[test]
    fn only_the_portals_decided_here_are_registered_on_the_bus() {
        let serving = Serving::started("only-what-was-decided");
        let introspected: String = serving
            .client
            .call_method(
                Some(DESKTOP),
                PORTALS,
                Some("org.freedesktop.DBus.Introspectable"),
                "Introspect",
                &(),
            )
            .unwrap()
            .body()
            .deserialize()
            .unwrap();
        let portals: Vec<&str> = introspected
            .split("<interface name=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .filter(|name| name.starts_with("org.freedesktop.portal."))
            .collect();
        let mut expected: Vec<&str> = Portal::EVERY
            .iter()
            .filter_map(|portal| portal.answered_on_the_bus())
            .collect();
        let mut found = portals.clone();
        found.sort_unstable();
        expected.sort_unstable();
        assert_eq!(found, expected, "{introspected}");

        let refused = serving
            .client
            .call_method(
                Some(DESKTOP),
                PORTALS,
                Some("org.freedesktop.portal.FileChooser"),
                "OpenFile",
                &("", "Open", HashMap::<&str, Value<'_>>::new()),
            )
            .expect_err("the file chooser was answered");
        assert!(
            matches!(&refused, zbus::Error::MethodError(name, _, _)
                if name.as_str() == "org.freedesktop.DBus.Error.UnknownInterface"
                    || name.as_str() == "org.freedesktop.DBus.Error.UnknownMethod"),
            "{refused:?}"
        );
        assert!(
            serving.record.everything().is_empty(),
            "an unserved portal was answered"
        );
    }
}
