//! What an application was answered is kept after the backend stops.
//!
//! Task 8 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`,
//! each clause of its acceptance a test here:
//!
//! - **the backend's answers are kept durably and read back in the order given,
//!   each with its time, the application named (or nobody), the portal and the
//!   outcome** —
//!   [`the_file::every_kind_of_answer_is_kept_and_read_back_in_order`];
//! - **a backend started again over the same record finds what the last one
//!   wrote** — [`the_file::a_record_opened_again_adds_to_what_was_there`], and
//!   on a real bus,
//!   [`on_the_bus::a_backend_started_again_finds_what_the_last_one_wrote`];
//! - **every refusal kind tasks 5–7 produce — not granted, not identified, a
//!   caller gone — is written and read back**: the first two in both tests
//!   above, and a caller gone in
//!   `a_caller_is_named_by_the_process_the_bus_holds.rs`
//!   (`a_caller_gone_is_read_back_from_the_disk_after_the_backend_stops`);
//! - **an answer the record could not keep is never sent to the application** —
//!   [`on_the_bus::an_answer_the_record_could_not_keep_is_never_sent`];
//! - **where it is kept, under `alo-remembering`'s rules, with its format in
//!   `docs/contracts/`** — the refusals in [`the_file`], and
//!   [`the_contract_describes_the_file`].

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

/// **The contract names the file, its format line, every answer and every
/// portal by the names the file writes.**
#[test]
fn the_contract_describes_the_file() {
    let contract = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/contracts/portal-answers-file.md"),
    )
    .expect("docs/contracts/portal-answers-file.md is there");
    #[cfg(unix)]
    {
        // Where the file is now, and the two variables that decide it — the
        // path is per login since ADR 0052, so the contract names the rule
        // rather than one machine's answer.
        assert!(contract.contains(alo_portals::THE_ANSWERS_FILE), "the file");
        assert!(contract.contains(alo_portals::THE_FOLDER), "the folder");
        assert!(contract.contains(alo_portals::STATE_HOME), "the state home");
        assert!(contract.contains(alo_portals::HOME), "the home");
        assert!(
            contract.contains(alo_portals::WHERE_IT_USED_TO_BE),
            "where the file used to be, which a machine upgraded from a pre-release image still \
             has"
        );
    }
    assert!(contract.contains(r#"{"format":1}"#), "the format line");
    for answer in [
        "secret-handed-over",
        "opened",
        "appearance-read",
        "appearance-sent",
        "refused",
        "not-a-request",
        "nothing-opens",
        "unanswered",
        "nothing-granted",
        "never-granted",
        "lapsed",
        "not-identified",
    ] {
        assert!(contract.contains(&format!("`{answer}`")), "{answer}");
    }
    for portal in alo_portals::Portal::EVERY {
        let named = serde_json::to_string(&portal).unwrap();
        assert!(
            contract.contains(&format!("`{}`", named.trim_matches('"'))),
            "{portal:?}"
        );
    }
}

#[cfg(unix)]
mod the_file {
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    use alo_applications::NothingOpens;
    use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
    use alo_opening::Cannot;
    use alo_portals::{
        Answered, AnswersFile, KeptAnswer, KeptOutcome, NotARequest, NotRecorded, Outcome, Portal,
        ReadBack, Recording, RefusedAs, Request, Unanswered,
    };

    const FRACTAL: &str = "org.gnome.Fractal";
    const STRANGER: &str = "org.example.Stranger";

    /// A folder of this test's own, empty.
    fn a_folder(what: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("alo-portal-answers-{what}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::new(1_760_000_000, 987_654_321)
    }

    /// Every kind of answer the backend gives, each a real decision where a
    /// decision makes it: what the grants allowed and refused came from
    /// `Request::judged`.
    fn every_kind_of_answer() -> Vec<Answered> {
        let mut grants = Grants::default();
        let secrets = grants.grant(
            Grant::checked_for(
                &Applicant::named(FRACTAL).grantee(),
                Reach::Facility(Facility::Secrets),
                noon(),
                Duration::from_secs(60),
            )
            .unwrap(),
        );
        // Held for an hour, so once the secrets grant has lapsed Fractal still
        // holds something, and is refused as lapsed rather than as granted
        // nothing.
        grants.grant(
            Grant::checked_for(
                &Applicant::named(FRACTAL).grantee(),
                Reach::Facility(Facility::Camera),
                noon(),
                Duration::from_secs(60 * 60),
            )
            .unwrap(),
        );
        let secret = Request::of(FRACTAL, Portal::Secret).unwrap();
        let allowed = secret.judged(&grants, noon()).unwrap();
        assert_eq!(allowed.against(), [secrets]);
        let nothing_granted = Request::of(STRANGER, Portal::Secret)
            .unwrap()
            .judged(&grants, noon())
            .unwrap_err();
        let never_granted = Request::of(FRACTAL, Portal::Microphone)
            .unwrap()
            .judged(&grants, noon())
            .unwrap_err();
        let lapsed = secret
            .judged(&grants, noon() + Duration::from_secs(120))
            .unwrap_err();

        let mut outcomes = vec![
            (
                Some(FRACTAL),
                Portal::Secret,
                Outcome::SecretHandedOver(allowed.clone()),
            ),
            (
                Some(FRACTAL),
                Portal::Settings,
                Outcome::AppearanceRead(allowed.clone()),
            ),
            (
                Some(FRACTAL),
                Portal::Settings,
                Outcome::AppearanceSent(allowed),
            ),
            (
                Some(STRANGER),
                Portal::Secret,
                Outcome::Refused(nothing_granted),
            ),
            (
                Some(FRACTAL),
                Portal::Microphone,
                Outcome::Refused(never_granted),
            ),
            (Some(FRACTAL), Portal::Secret, Outcome::Refused(lapsed)),
            (
                Some(FRACTAL),
                Portal::OpenWith,
                Outcome::NotARequest(NotARequest::CouldLeadElsewhere),
            ),
            (
                Some(FRACTAL),
                Portal::OpenWith,
                Outcome::NothingOpens(NothingOpens::TheFile(Cannot::AProgram)),
            ),
        ];
        for unanswered in [
            Unanswered::NotIdentified,
            Unanswered::NotAToken,
            Unanswered::GrantsUnread,
            Unanswered::ApplicationsUnread,
            Unanswered::KeyringUnavailable,
            Unanswered::NotWritten,
            Unanswered::NotAFile,
            Unanswered::NotOpened {
                opener: "org.gnome.Papers".to_owned(),
            },
            Unanswered::NotDecidedHere,
            Unanswered::NoSuchSetting,
            Unanswered::AppearanceUnread,
        ] {
            let nobody = unanswered == Unanswered::NotIdentified;
            outcomes.push((
                (!nobody).then_some(FRACTAL),
                Portal::OpenWith,
                Outcome::Unanswered(unanswered),
            ));
        }
        outcomes
            .into_iter()
            .zip(0_u64..)
            .map(|((application, portal, outcome), second)| {
                Answered::new(
                    noon() + Duration::from_secs(second),
                    application.map(Applicant::named),
                    portal,
                    outcome,
                )
            })
            .collect()
    }

    fn kept(answers: &[Answered]) -> Vec<KeptAnswer> {
        answers.iter().map(KeptAnswer::from).collect()
    }

    fn keep_all(record: &AnswersFile, answers: &[Answered]) {
        for answered in answers {
            record.keep(answered.clone()).unwrap();
        }
    }

    /// **Every kind of answer — the refusals as carefully as the rest — is kept
    /// and read back in the order given**, with its time to the nanosecond, the
    /// application or nobody, the portal and the outcome; in a file only its
    /// owner can write, whose first line says its format.
    #[test]
    fn every_kind_of_answer_is_kept_and_read_back_in_order() {
        let folder = a_folder("every-kind");
        let at = folder.join("portal-answers.jsonl");
        let answers = every_kind_of_answer();
        keep_all(&AnswersFile::opened(&at).unwrap(), &answers);

        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.unreadable, Vec::<usize>::new());
        assert_eq!(read.answers, kept(&answers));
        for (back, given) in read.answers.iter().zip(&answers) {
            assert_eq!(back.at(), given.at());
            assert_eq!(
                back.application(),
                given.application().map(Applicant::as_str)
            );
            assert_eq!(back.portal(), given.portal());
            assert_eq!(back.outcome().was_refused(), given.outcome().was_refused());
        }
        let refusals: Vec<&KeptOutcome> = read
            .answers
            .iter()
            .map(KeptAnswer::outcome)
            .filter(|outcome| matches!(outcome, KeptOutcome::Refused { .. }))
            .collect();
        assert_eq!(
            refusals,
            [
                &KeptOutcome::Refused {
                    why: RefusedAs::NothingGranted
                },
                &KeptOutcome::Refused {
                    why: RefusedAs::NeverGranted
                },
                &KeptOutcome::Refused {
                    why: RefusedAs::Lapsed
                },
            ]
        );
        let not_identified = read
            .answers
            .iter()
            .find(|answer| {
                answer.outcome()
                    == &KeptOutcome::Unanswered {
                        why: Unanswered::NotIdentified,
                    }
            })
            .unwrap();
        assert_eq!(not_identified.application(), None);

        let text = std::fs::read_to_string(&at).unwrap();
        assert_eq!(text.lines().next(), Some(r#"{"format":1}"#));
        assert_eq!(text.lines().count(), answers.len() + 1);
        let mode = std::fs::metadata(&at).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "the answers went down mode {mode:o}");
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **A record opened again adds to what was there** — one format line, and
    /// every answer from before the restart ahead of every answer after it.
    #[test]
    fn a_record_opened_again_adds_to_what_was_there() {
        let folder = a_folder("again");
        let at = folder.join("portal-answers.jsonl");
        let answers = every_kind_of_answer();
        let (before, after) = answers.split_at(4);
        keep_all(&AnswersFile::opened(&at).unwrap(), before);
        assert_eq!(AnswersFile::read_back(&at).unwrap().answers, kept(before));
        keep_all(&AnswersFile::opened(&at).unwrap(), after);

        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.answers, kept(&answers));
        let text = std::fs::read_to_string(&at).unwrap();
        assert_eq!(text.matches("format").count(), 1, "{text}");
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **A line the machine stopped in the middle of stays one line that did not
    /// read**, reported by its number beside everything that did, and the next
    /// answer after a restart is whole.
    #[test]
    fn a_torn_last_line_is_reported_and_the_next_answer_is_whole() {
        let folder = a_folder("torn");
        let at = folder.join("portal-answers.jsonl");
        let answers = every_kind_of_answer();
        keep_all(
            &AnswersFile::opened(&at).unwrap(),
            answers.get(..1).unwrap(),
        );
        let mut text = std::fs::read_to_string(&at).unwrap();
        text.push_str(r#"{"at":{"secs_since_epoch":17600"#);
        std::fs::write(&at, text).unwrap();

        keep_all(
            &AnswersFile::opened(&at).unwrap(),
            answers.get(1..2).unwrap(),
        );
        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(
            read,
            ReadBack {
                answers: kept(answers.get(..2).unwrap()),
                unreadable: vec![3],
                since: None,
                under: None,
            }
        );
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **A file the group or the world could write is neither added to nor
    /// believed**, and a symbolic link is never followed.
    #[test]
    fn a_file_somebody_else_could_have_written_is_refused() {
        let folder = a_folder("others");
        let at = folder.join("portal-answers.jsonl");
        keep_all(
            &AnswersFile::opened(&at).unwrap(),
            every_kind_of_answer().get(..1).unwrap(),
        );
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o666)).unwrap();
        assert!(matches!(
            AnswersFile::opened(&at),
            Err(NotRecorded::WritableByOthers { mode: 0o666, .. })
        ));
        assert!(matches!(
            AnswersFile::read_back(&at),
            Err(NotRecorded::WritableByOthers { .. })
        ));

        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o600)).unwrap();
        let link = folder.join("linked.jsonl");
        std::os::unix::fs::symlink(&at, &link).unwrap();
        assert!(matches!(
            AnswersFile::opened(&link),
            Err(NotRecorded::ALink { .. })
        ));
        assert!(matches!(
            AnswersFile::read_back(&link),
            Err(NotRecorded::ALink { .. })
        ));
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **A file in a newer format, or one that does not begin with its format,
    /// is refused and left byte for byte as it was** — and so is a folder.
    #[test]
    fn a_file_that_is_not_an_answers_file_this_reads_is_left_as_it_was() {
        let folder = a_folder("not-ours");
        let entry = KeptAnswer::from(every_kind_of_answer().first().unwrap());
        let entry = serde_json::to_string(&entry).unwrap();
        for (named, text) in [
            ("newer.jsonl", "{\"format\":2}\n".to_owned()),
            ("headless.jsonl", format!("{entry}\n")),
            ("nothing-like.jsonl", "an answer\n".to_owned()),
            ("format-zero.jsonl", "{\"format\":0}\n".to_owned()),
        ] {
            let at = folder.join(named);
            std::fs::write(&at, &text).unwrap();
            std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o600)).unwrap();
            let opened = AnswersFile::opened(&at);
            let read = AnswersFile::read_back(&at);
            if named == "newer.jsonl" {
                assert!(matches!(
                    opened,
                    Err(NotRecorded::ANewerFormat { format: 2, .. })
                ));
                assert!(matches!(
                    read,
                    Err(NotRecorded::ANewerFormat { format: 2, .. })
                ));
            } else {
                assert!(
                    matches!(opened, Err(NotRecorded::NotAnAnswersFile { .. })),
                    "{named}: {opened:?}"
                );
                assert!(
                    matches!(read, Err(NotRecorded::NotAnAnswersFile { .. })),
                    "{named}: {read:?}"
                );
            }
            assert_eq!(std::fs::read_to_string(&at).unwrap(), text, "{named}");
        }
        assert!(matches!(
            AnswersFile::opened(&folder),
            Err(NotRecorded::NotWritten { .. } | NotRecorded::NotAnAnswersFile { .. })
        ));
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **A folder that is not there is refused, not made**, and a reader asking
    /// where nothing has been kept is told exactly that.
    #[test]
    fn a_missing_folder_is_refused_and_nothing_kept_is_not_there() {
        let folder = a_folder("missing");
        let nowhere = folder.join("not-made").join("portal-answers.jsonl");
        assert!(matches!(
            AnswersFile::opened(&nowhere),
            Err(NotRecorded::NotWritten { .. })
        ));
        assert!(!nowhere.parent().unwrap().exists(), "the folder was made");
        assert!(matches!(
            AnswersFile::read_back(&folder.join("portal-answers.jsonl")),
            Err(NotRecorded::NotThere { .. })
        ));
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **A named pipe where the file should be is refused at once**, rather
    /// than holding the backend at `open` until somebody writes into it.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_named_pipe_is_refused_without_waiting() {
        use rustix::fs::{CWD, FileType, Mode, mknodat};
        let folder = a_folder("pipe");
        let at = folder.join("portal-answers.jsonl");
        mknodat(CWD, &at, FileType::Fifo, Mode::from_raw_mode(0o600), 0).unwrap();
        let started = std::time::Instant::now();
        assert!(matches!(
            AnswersFile::opened(&at),
            Err(NotRecorded::NotAnAnswersFile { .. })
        ));
        assert!(matches!(
            AnswersFile::read_back(&at),
            Err(NotRecorded::NotAnAnswersFile { .. })
        ));
        assert!(started.elapsed() < Duration::from_secs(5));
        std::fs::remove_dir_all(&folder).unwrap();
    }
}

#[cfg(target_os = "linux")]
mod on_the_bus {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Read;
    use std::os::fd::AsFd;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{Receiver, channel};
    use std::sync::{Arc, Mutex, RwLock};
    use std::time::{Duration, SystemTime};

    use alo_appearance::{Following, Shipped};
    use alo_applications::{Application, Chosen, Declared, Installed};
    use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
    use alo_keyring_fixture::AKeyringOfOurOwn;
    use alo_portals::{
        Allowed, Answered, AnswersFile, Appearance, Applications, Backend, KeepsSecrets, Kept,
        KeptAnswer, KeptOutcome, NotKept, NotRecorded, Portal, Recording, RefusedAs, Request,
        Sandboxes, Served, TheMachine, TimeOfDay, Unanswered,
    };
    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::{Fd, OwnedObjectPath, OwnedValue, Value};

    const FRACTAL: &str = "org.gnome.Fractal";
    const STRANGER: &str = "org.example.Stranger";
    const PAPERS: &str = "org.gnome.Papers";

    const DESKTOP: &str = "org.freedesktop.portal.Desktop";
    const PORTALS: &str = "/org/freedesktop/portal/desktop";
    const SETTINGS: &str = "org.freedesktop.portal.Settings";

    const A_PDF: &[u8] = b"%PDF-1.7\n1 0 obj << >> endobj\n%%EOF\n";

    /// The machine as the backend reads it: grants a test changes, Papers
    /// opening PDFs, and a clock a test moves.
    struct ThisMachine {
        grants: RwLock<Grants>,
        time: RwLock<TimeOfDay>,
    }

    impl TheMachine for ThisMachine {
        fn grants(&self) -> Option<Grants> {
            Some(self.grants.read().unwrap().clone())
        }

        fn applications(&self) -> Option<Applications> {
            let papers = Application::called(PAPERS, "Papers").unwrap();
            let mut declared = Declared::nothing();
            declared.declares_media_types(&papers, ["application/pdf"]);
            Some(Applications {
                installed: Installed::holding([papers]),
                declared,
                chosen: Chosen::untouched(),
            })
        }

        fn appearance(&self) -> Option<Appearance> {
            let mut appearance = Appearance::shipped();
            appearance.follow(Following::from(Shipped::the_evening_schedule()));
            Some(appearance)
        }

        fn time_of_day(&self) -> Option<TimeOfDay> {
            Some(*self.time.read().unwrap())
        }
    }

    /// A keyring that judges the request as `alo-secrets` does and hands an
    /// allowed application a secret naming it.
    struct AKeyring;

    impl KeepsSecrets for AKeyring {
        fn hand_over(
            &self,
            request: &Request,
            grants: &Grants,
            now: SystemTime,
            into: &mut dyn std::io::Write,
        ) -> Result<Allowed, NotKept> {
            let allowed = request.judged(grants, now).map_err(NotKept::Refused)?;
            into.write_all(secret_of(request.application().as_str()).as_bytes())
                .map_err(|_| NotKept::NotWritten)?;
            Ok(allowed)
        }
    }

    fn secret_of(application: &str) -> String {
        format!("the secret of {application}")
    }

    /// The answers file, and every answer it kept in memory beside it, so what
    /// is read back can be held to exactly what was given.
    struct OnTheDisk {
        file: AnswersFile,
        given: Arc<Kept>,
    }

    impl Recording for OnTheDisk {
        fn keep(&self, answered: Answered) -> Result<(), NotRecorded> {
            self.file.keep(answered.clone())?;
            self.given.keep(answered)
        }
    }

    /// A record that keeps nothing, and counts what it was asked to keep.
    struct ADiskThatWillNotTakeIt {
        asked: AtomicUsize,
        at: PathBuf,
    }

    impl Recording for ADiskThatWillNotTakeIt {
        fn keep(&self, _answered: Answered) -> Result<(), NotRecorded> {
            self.asked.fetch_add(1, Ordering::SeqCst);
            Err(NotRecorded::NotWritten {
                at: self.at.clone(),
                why: "No space left on device (os error 28)".to_owned(),
            })
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

    /// A bus, a machine, and where sandboxes and files are laid out — which a
    /// backend is started on, stopped, and started on again.
    struct AMachine {
        bus: AKeyringOfOurOwn,
        machine: Arc<ThisMachine>,
        place: PathBuf,
        processes: PathBuf,
        client: Connection,
        papers_opened: Arc<Mutex<Vec<String>>>,
        _papers: Connection,
    }

    impl AMachine {
        fn started(what: &str) -> Self {
            let bus = AKeyringOfOurOwn::a_bus_with_no_keyring_on_it(what);
            let place = std::env::temp_dir().join(format!(
                "alo-portal-answers-bus-{what}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&place);
            let processes = place.join("proc");
            std::fs::create_dir_all(processes.join(std::process::id().to_string()).join("root"))
                .unwrap();
            std::fs::create_dir_all(place.join("Invoices")).unwrap();
            let machine = Arc::new(ThisMachine {
                grants: RwLock::new(Grants::default()),
                time: RwLock::new(TimeOfDay::checked(12, 0).unwrap()),
            });
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
                bus,
                machine,
                place: place.canonicalize().unwrap(),
                processes,
                client,
                papers_opened,
                _papers: papers,
            }
        }

        fn answers(&self) -> PathBuf {
            self.place.join("portal-answers.jsonl")
        }

        /// A backend on this machine's bus, writing into `record`.
        fn serving(&self, record: Arc<dyn Recording>) -> Served {
            Backend::answering_from(
                self.machine.clone(),
                Arc::new(AKeyring),
                Sandboxes::under(&self.processes),
                record,
            )
            .serve_on(&self.bus.address())
            .expect("the portals are served on the test's bus")
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

        /// A request made by `call`, listening on the predicted handle first:
        /// the response code.
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
            let reply = call(&self.client).expect("the portal answers the call");
            let returned: OwnedObjectPath = reply.body().deserialize().unwrap();
            assert_eq!(returned.as_str(), handle);
            let response = responses.next().expect("a response");
            let (code, _results): (u32, HashMap<String, OwnedValue>) =
                response.body().deserialize().unwrap();
            code
        }

        /// `RetrieveSecret`: the response code, and every byte written.
        fn retrieve_secret(&self, token: &str) -> (u32, Vec<u8>) {
            let (mut reader, writer) = std::io::pipe().unwrap();
            let code = self.ask(token, |client| {
                client.call_method(
                    Some(DESKTOP),
                    PORTALS,
                    Some("org.freedesktop.portal.Secret"),
                    "RetrieveSecret",
                    &(Fd::from(writer.as_fd()), options(token)),
                )
            });
            drop(writer);
            let mut written = Vec::new();
            reader.read_to_end(&mut written).unwrap();
            (code, written)
        }

        /// `OpenFile` for the file at `path`: the response code.
        fn open_file(&self, path: &Path, token: &str) -> u32 {
            let file = File::open(path).unwrap();
            self.ask(token, |client| {
                client.call_method(
                    Some(DESKTOP),
                    PORTALS,
                    Some("org.freedesktop.portal.OpenURI"),
                    "OpenFile",
                    &("", Fd::from(file.as_fd()), options(token)),
                )
            })
        }

        fn read_one(&self) -> zbus::Result<OwnedValue> {
            self.client
                .call_method(
                    Some(DESKTOP),
                    PORTALS,
                    Some(SETTINGS),
                    "ReadOne",
                    &("org.freedesktop.appearance", "color-scheme"),
                )?
                .body()
                .deserialize()
        }

        fn read_all(&self, namespaces: &[&str]) -> zbus::Result<zbus::Message> {
            self.client.call_method(
                Some(DESKTOP),
                PORTALS,
                Some(SETTINGS),
                "ReadAll",
                &(namespaces,),
            )
        }

        /// Every `SettingChanged` the client is sent, as they arrive.
        fn changes(&self) -> Receiver<String> {
            let proxy = Proxy::new(&self.client, DESKTOP, PORTALS, SETTINGS).unwrap();
            let signals = proxy.receive_signal("SettingChanged").unwrap();
            let (send, arrived) = channel();
            std::thread::spawn(move || {
                for signal in signals {
                    let (_namespace, key, _value): (String, String, OwnedValue) =
                        signal.body().deserialize().unwrap();
                    if send.send(key).is_err() {
                        return;
                    }
                }
            });
            arrived
        }

        fn papers_opened(&self) -> Vec<String> {
            self.papers_opened.lock().unwrap().clone()
        }
    }

    impl Drop for AMachine {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.place);
        }
    }

    fn options(token: &str) -> HashMap<&'static str, Value<'_>> {
        let mut options = HashMap::new();
        options.insert("handle_token", Value::from(token));
        options
    }

    fn the_error_is(error: &zbus::Error, named: &str) -> bool {
        matches!(error, zbus::Error::MethodError(name, _, _) if name.as_str() == named)
    }

    /// **A backend started again over the same answers file finds what the last
    /// one wrote**: every answer each backend gave on a real bus — handed over,
    /// refused as never granted, refused as nothing granted, and a program
    /// nobody could name — read back from the disk after both have stopped,
    /// in the order given, each with its time, application, portal and outcome.
    #[test]
    fn a_backend_started_again_finds_what_the_last_one_wrote() {
        let machine = AMachine::started("again");
        machine.grant(FRACTAL, Reach::Facility(Facility::Secrets));
        let given = Arc::new(Kept::nothing());

        let first = machine.serving(Arc::new(OnTheDisk {
            file: AnswersFile::opened(&machine.answers()).unwrap(),
            given: given.clone(),
        }));
        machine.become_(Some(FRACTAL));
        assert_eq!(
            machine.retrieve_secret("fractal"),
            (0, secret_of(FRACTAL).into_bytes())
        );
        assert!(the_error_is(
            &machine.read_one().unwrap_err(),
            "org.freedesktop.portal.Error.NotFound"
        ));
        machine.become_(Some(STRANGER));
        assert_eq!(machine.retrieve_secret("stranger"), (2, Vec::new()));
        drop(first);

        let second = machine.serving(Arc::new(OnTheDisk {
            file: AnswersFile::opened(&machine.answers()).unwrap(),
            given: given.clone(),
        }));
        machine.become_(None);
        assert_eq!(machine.retrieve_secret("nobody"), (2, Vec::new()));
        machine.become_(Some(STRANGER));
        assert_eq!(machine.retrieve_secret("stranger_again"), (2, Vec::new()));
        drop(second);

        let read = AnswersFile::read_back(&machine.answers()).unwrap();
        assert!(read.unreadable.is_empty(), "{read:?}");
        let given: Vec<KeptAnswer> = given.everything().iter().map(KeptAnswer::from).collect();
        assert_eq!(read.answers, given);
        let summary: Vec<(Option<&str>, Portal, &KeptOutcome)> = read
            .answers
            .iter()
            .map(|answer| (answer.application(), answer.portal(), answer.outcome()))
            .collect();
        assert!(
            matches!(
                summary.as_slice(),
                [
                    (
                        Some(FRACTAL),
                        Portal::Secret,
                        KeptOutcome::SecretHandedOver { .. }
                    ),
                    (
                        Some(FRACTAL),
                        Portal::Settings,
                        KeptOutcome::Refused {
                            why: RefusedAs::NeverGranted
                        }
                    ),
                    (
                        Some(STRANGER),
                        Portal::Secret,
                        KeptOutcome::Refused {
                            why: RefusedAs::NothingGranted
                        }
                    ),
                    (
                        None,
                        Portal::Secret,
                        KeptOutcome::Unanswered {
                            why: Unanswered::NotIdentified
                        }
                    ),
                    (
                        Some(STRANGER),
                        Portal::Secret,
                        KeptOutcome::Refused {
                            why: RefusedAs::NothingGranted
                        }
                    ),
                ]
            ),
            "{summary:#?}"
        );
        let times: Vec<SystemTime> = read.answers.iter().map(KeptAnswer::at).collect();
        assert!(times.is_sorted(), "{times:?}");
    }

    /// **An answer the record could not keep is never sent to the application.**
    /// Every request here is one the grants allow and a working record would
    /// answer: the secret is not written and the response is `2`; the file is
    /// not opened and the response is `2`; the appearance is not read out and
    /// the method fails; a namespace nobody answers is not answered with an
    /// empty dictionary; and a change is not sent. Each was put to the record
    /// first.
    #[test]
    fn an_answer_the_record_could_not_keep_is_never_sent() {
        let machine = AMachine::started("not-kept");
        machine.grant(FRACTAL, Reach::Facility(Facility::Secrets));
        machine.grant(FRACTAL, Reach::Facility(Facility::AppearanceSettings));
        machine.grant(FRACTAL, Reach::Folder(machine.place.join("Invoices")));
        machine.grant(FRACTAL, Reach::Application(PAPERS.to_owned()));
        let invoice = machine.place.join("Invoices").join("june.pdf");
        std::fs::write(&invoice, A_PDF).unwrap();
        let record = Arc::new(ADiskThatWillNotTakeIt {
            asked: AtomicUsize::new(0),
            at: machine.answers(),
        });
        let _served = machine.serving(record.clone());
        machine.become_(Some(FRACTAL));
        let changes = machine.changes();

        assert_eq!(machine.retrieve_secret("secret"), (2, Vec::new()));
        assert_eq!(record.asked.load(Ordering::SeqCst), 1);

        assert_eq!(machine.open_file(&invoice, "invoice"), 2);
        assert!(
            machine.papers_opened().is_empty(),
            "{:?}",
            machine.papers_opened()
        );
        assert_eq!(record.asked.load(Ordering::SeqCst), 2);

        let refused = machine.read_one().unwrap_err();
        assert!(
            the_error_is(&refused, "org.freedesktop.portal.Error.Failed"),
            "{refused:?}"
        );
        assert_eq!(record.asked.load(Ordering::SeqCst), 3);
        let refused = machine.read_all(&["org.example.nothing"]).unwrap_err();
        assert!(
            the_error_is(&refused, "org.freedesktop.portal.Error.Failed"),
            "{refused:?}"
        );
        assert_eq!(record.asked.load(Ordering::SeqCst), 4);

        *machine.machine.time.write().unwrap() = TimeOfDay::checked(21, 0).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        while record.asked.load(Ordering::SeqCst) < 5 {
            assert!(
                std::time::Instant::now() < deadline,
                "the change was never put to the record"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            changes.recv_timeout(Duration::from_secs(3)).is_err(),
            "a change the record did not keep was sent"
        );
        assert!(machine.papers_opened().is_empty());
        assert!(!machine.answers().exists());
    }

    /// **The same machine with a record that keeps answers answers every one
    /// of those requests** — so the refusals above are the record's and
    /// nothing else.
    #[test]
    fn the_same_requests_are_answered_when_the_record_keeps_them() {
        let machine = AMachine::started("kept");
        machine.grant(FRACTAL, Reach::Facility(Facility::Secrets));
        machine.grant(FRACTAL, Reach::Facility(Facility::AppearanceSettings));
        machine.grant(FRACTAL, Reach::Folder(machine.place.join("Invoices")));
        machine.grant(FRACTAL, Reach::Application(PAPERS.to_owned()));
        let invoice = machine.place.join("Invoices").join("june.pdf");
        std::fs::write(&invoice, A_PDF).unwrap();
        let given = Arc::new(Kept::nothing());
        let _served = machine.serving(Arc::new(OnTheDisk {
            file: AnswersFile::opened(&machine.answers()).unwrap(),
            given: given.clone(),
        }));
        machine.become_(Some(FRACTAL));
        let changes = machine.changes();

        assert_eq!(
            machine.retrieve_secret("secret"),
            (0, secret_of(FRACTAL).into_bytes())
        );
        assert_eq!(machine.open_file(&invoice, "invoice"), 0);
        assert_eq!(machine.papers_opened().len(), 1);
        assert!(machine.read_one().is_ok());
        assert!(machine.read_all(&["org.example.nothing"]).is_ok());
        *machine.machine.time.write().unwrap() = TimeOfDay::checked(21, 0).unwrap();
        changes
            .recv_timeout(Duration::from_secs(30))
            .expect("the change was sent");

        let read = AnswersFile::read_back(&machine.answers()).unwrap();
        assert_eq!(
            read.answers,
            given
                .everything()
                .iter()
                .map(KeptAnswer::from)
                .collect::<Vec<_>>()
        );
        assert!(read.answers.iter().any(|answer| matches!(
            answer.outcome(),
            KeptOutcome::Opened { opener, .. } if opener == PAPERS
        )));
        assert!(
            read.answers
                .iter()
                .any(|answer| matches!(answer.outcome(), KeptOutcome::AppearanceSent { .. }))
        );
    }
}
