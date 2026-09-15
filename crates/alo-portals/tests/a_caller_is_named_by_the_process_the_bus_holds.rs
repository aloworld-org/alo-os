//! A caller is named by the process the bus holds, never by a number that can
//! be given to another process while its sandbox is read.
//!
//! Task 7 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`.
//! The backend is served on a private bus the test starts, and the caller is
//! **another process** — this test binary run again, asking the bus as an
//! application does. Its `.flatpak-info` is a named pipe, so the backend opening
//! it waits until the test writes to it: the test ends the caller and reaps it
//! at exactly that moment, *then* writes a sandbox naming an application that
//! holds every grant the request needs. A backend that trusted the number would
//! name that application. This one must not:
//!
//! - **a caller gone by the time its sandbox is read is `NotIdentified`, and is
//!   recorded that way** — for `OpenURI`
//!   ([`on_the_bus::an_open_file_request_whose_caller_is_gone_is_not_identified`]),
//!   for `Secret`
//!   ([`on_the_bus::a_secret_request_whose_caller_is_gone_is_not_identified`]),
//!   and for `Settings`
//!   ([`on_the_bus::a_settings_read_whose_caller_is_gone_is_not_identified`]);
//! - **a `SettingChanged` whose connection's process is gone is not sent, and is
//!   recorded as not identified** —
//!   [`on_the_bus::a_change_for_a_connection_whose_process_is_gone_is_not_sent`];
//! - **a caller in another process that is still there is named**, for all
//!   three portals and the signal —
//!   [`on_the_bus::a_caller_in_another_process_that_is_still_there_is_named`];
//! - **the contract says how the process is held, and the quirks which bus
//!   daemons hand over a descriptor** —
//!   [`the_contract_and_the_quirks_say_how_a_caller_is_held`].

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

/// **`docs/contracts/portals.md`'s *Who is asking* says the process is held by
/// a descriptor and what is refused, and `docs/quirks.md` records which daemon
/// gives `ProcessFD`.**
#[test]
fn the_contract_and_the_quirks_say_how_a_caller_is_held() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let contract = std::fs::read_to_string(root.join("docs/contracts/portals.md")).unwrap();
    let who = contract
        .split_once("## Who is asking")
        .map(|(_, rest)| {
            rest.split_once("\n## ")
                .map_or(rest, |(section, _)| section)
        })
        .expect("the contract has a Who is asking section");
    for named in ["ProcessFD", "pidfd", "ProcessID", "not identified"] {
        assert!(who.contains(named), "Who is asking does not name {named}");
    }
    let quirks = std::fs::read_to_string(root.join("docs/quirks.md")).unwrap();
    assert!(
        quirks.contains("ProcessFD") && quirks.contains("1.14.10"),
        "docs/quirks.md does not record which dbus-daemon gives ProcessFD"
    );
}

#[cfg(target_os = "linux")]
mod on_the_bus {
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, Write};
    use std::os::fd::AsFd;
    use std::path::PathBuf;
    use std::process::{Child, ChildStdin, Command, Stdio};
    use std::sync::mpsc::channel;
    use std::sync::{Arc, RwLock};
    use std::time::{Duration, Instant, SystemTime};

    use alo_appearance::{Following, Shipped};
    use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
    use alo_keyring_fixture::AKeyringOfOurOwn;
    use alo_portals::{
        Allowed, Answered, Appearance, Applications, Backend, KeepsSecrets, Kept, NotKept, Outcome,
        Portal, Request, Sandboxes, Served, TheMachine, TimeOfDay, Unanswered,
    };
    use rustix::fs::{CWD, FileType, Mode, mknodat};
    use zbus::zvariant::{Fd, Value};

    /// The application every sandbox the test writes names, and which holds
    /// every grant a request here could need.
    const FRACTAL: &str = "org.gnome.Fractal";

    const DESKTOP: &str = "org.freedesktop.portal.Desktop";
    const PORTALS: &str = "/org/freedesktop/portal/desktop";

    /// The bus a caller run as another process asks on.
    const ASKING_ON: &str = "ALO_PORTALS_A_CALLER_ON";
    /// What it asks: `open-file`, `secret`, `settings`, or `listening`.
    const ASKING: &str = "ALO_PORTALS_A_CALLER_ASKS";

    /// How long anything here waits before calling it a failure.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// **The caller, when this binary is run as one** — and nothing at all when
    /// it is run as the test suite, where [`ASKING_ON`] is not set.
    ///
    /// It waits for a line on its input before it touches the bus, so the test
    /// has laid out its sandbox by the time it asks; says `connected` once it is
    /// on the bus; and then asks, or for `listening` waits to be sent a change.
    #[test]
    fn a_caller_run_as_another_process() {
        let (Ok(address), Ok(asking)) = (std::env::var(ASKING_ON), std::env::var(ASKING)) else {
            return;
        };
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let client = zbus::blocking::connection::Builder::address(address.as_str())
            .unwrap()
            .build()
            .unwrap();
        println!("connected");
        std::io::stdout().flush().unwrap();
        let mut options: HashMap<&str, Value<'_>> = HashMap::new();
        options.insert("handle_token", Value::from("gone"));
        match asking.as_str() {
            "open-file" => {
                let file = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
                let _ = client.call_method(
                    Some(DESKTOP),
                    PORTALS,
                    Some("org.freedesktop.portal.OpenURI"),
                    "OpenFile",
                    &("", Fd::from(file.as_fd()), options),
                );
            }
            "secret" => {
                let (_reader, writer) = std::io::pipe().unwrap();
                let _ = client.call_method(
                    Some(DESKTOP),
                    PORTALS,
                    Some("org.freedesktop.portal.Secret"),
                    "RetrieveSecret",
                    &(Fd::from(writer.as_fd()), options),
                );
            }
            "settings" => {
                let _ = client.call_method(
                    Some(DESKTOP),
                    PORTALS,
                    Some("org.freedesktop.portal.Settings"),
                    "ReadOne",
                    &("org.freedesktop.appearance", "color-scheme"),
                );
            }
            "listening" => {
                line.clear();
                let _ = std::io::stdin().read_line(&mut line);
            }
            other => panic!("a caller asked to ask {other}"),
        }
        drop(client);
    }

    /// The machine: Fractal granted the appearance settings and its secrets,
    /// and a clock a test moves to make the appearance change.
    struct ThisMachine {
        grants: Grants,
        time: RwLock<TimeOfDay>,
    }

    impl TheMachine for ThisMachine {
        fn grants(&self) -> Option<Grants> {
            Some(self.grants.clone())
        }

        fn applications(&self) -> Option<Applications> {
            Some(Applications::default())
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

    /// No keyring: a secret request that got as far as the keyring was named.
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

    /// The backend on a bus of its own, and where callers' sandboxes are laid out.
    struct Serving {
        bus: AKeyringOfOurOwn,
        machine: Arc<ThisMachine>,
        record: Arc<Kept>,
        processes: PathBuf,
        _served: Served,
    }

    impl Serving {
        fn started(what: &str) -> Self {
            let bus = AKeyringOfOurOwn::a_bus_with_no_keyring_on_it(what);
            let processes = std::env::temp_dir()
                .join(format!("alo-portal-held-{what}-{}", std::process::id()))
                .join("proc");
            std::fs::create_dir_all(&processes).unwrap();
            let mut grants = Grants::default();
            for facility in [Facility::AppearanceSettings, Facility::Secrets] {
                grants.grant(
                    Grant::checked_for(
                        &Applicant::named(FRACTAL).grantee(),
                        Reach::Facility(facility),
                        SystemTime::now(),
                        Duration::from_secs(60 * 60),
                    )
                    .unwrap(),
                );
            }
            let machine = Arc::new(ThisMachine {
                grants,
                time: RwLock::new(TimeOfDay::checked(12, 0).unwrap()),
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
            Self {
                bus,
                machine,
                record,
                processes,
                _served: served,
            }
        }

        /// This binary, run again as a caller asking `asking`, waiting for its
        /// go-ahead.
        fn a_caller(&self, asking: &str) -> Caller {
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "on_the_bus::a_caller_run_as_another_process",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env(ASKING_ON, self.bus.address())
                .env(ASKING, asking)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let input = child.stdin.take().unwrap();
            let root = self.processes.join(child.id().to_string()).join("root");
            std::fs::create_dir_all(&root).unwrap();
            Caller {
                child,
                input: Some(input),
                sandbox: root.join(".flatpak-info"),
            }
        }

        /// The answers recorded for `portal`, once there are `at_least` of them.
        fn recorded(&self, portal: Portal, at_least: usize) -> Vec<Answered> {
            let deadline = Instant::now() + PATIENCE;
            loop {
                let answers: Vec<Answered> = self
                    .record
                    .everything()
                    .into_iter()
                    .filter(|answered| answered.portal() == portal)
                    .collect();
                if answers.len() >= at_least {
                    return answers;
                }
                assert!(
                    Instant::now() < deadline,
                    "nothing was recorded for {portal:?}"
                );
                std::thread::sleep(Duration::from_millis(20));
            }
        }

        /// Move the clock into the evening, when the shipped schedule is dark,
        /// so the appearance an application reads changes.
        fn evening(&self) {
            *self.machine.time.write().unwrap() = TimeOfDay::checked(21, 0).unwrap();
        }
    }

    impl Drop for Serving {
        fn drop(&mut self) {
            if let Some(place) = self.processes.parent() {
                let _ = std::fs::remove_dir_all(place);
            }
        }
    }

    /// A caller in another process.
    struct Caller {
        child: Child,
        input: Option<ChildStdin>,
        sandbox: PathBuf,
    }

    impl Caller {
        /// Its sandbox names Fractal, as an ordinary file.
        fn sandboxed_as_fractal(&self) {
            std::fs::write(&self.sandbox, sandbox_naming_fractal()).unwrap();
        }

        /// Its sandbox is a named pipe, which the backend waits on when it
        /// opens it.
        fn sandboxed_behind_a_pipe(&self) {
            mknodat(
                CWD,
                &self.sandbox,
                FileType::Fifo,
                Mode::from_raw_mode(0o600),
                0,
            )
            .unwrap();
        }

        /// Let it onto the bus.
        fn go(&mut self) {
            writeln!(self.input.as_mut().unwrap(), "go").unwrap();
        }

        /// Wait until it says it is on the bus.
        fn until_connected(&mut self) {
            let output = self.child.stdout.take().unwrap();
            let (said, heard) = channel();
            std::thread::spawn(move || {
                for line in BufReader::new(output).lines() {
                    let Ok(line) = line else { return };
                    // The harness writes the test's name on the same line.
                    if line.ends_with("connected") {
                        let _ = said.send(());
                        return;
                    }
                }
            });
            heard
                .recv_timeout(PATIENCE)
                .expect("the caller reached the bus");
        }

        /// Wait until the backend opens its sandbox; then end the caller and
        /// reap it, so its number is free to be given away; and only then let
        /// the backend read a sandbox naming Fractal.
        fn gone_while_its_sandbox_is_read(mut self) {
            let pipe = self.sandbox.clone();
            let (opened, is_open) = channel();
            std::thread::spawn(move || {
                let writer = std::fs::OpenOptions::new().write(true).open(pipe);
                let _ = opened.send(writer);
            });
            let mut writer = is_open
                .recv_timeout(PATIENCE)
                .expect("the backend read the caller's sandbox")
                .unwrap();
            self.child.kill().unwrap();
            self.child.wait().unwrap();
            writer
                .write_all(sandbox_naming_fractal().as_bytes())
                .unwrap();
            drop(writer);
        }

        /// Wait for it to finish asking on its own.
        fn finished(mut self) {
            drop(self.input.take());
            let deadline = Instant::now() + PATIENCE;
            while self.child.try_wait().unwrap().is_none() {
                assert!(Instant::now() < deadline, "the caller never finished");
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }

    impl Drop for Caller {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    fn sandbox_naming_fractal() -> String {
        format!("[Application]\nname={FRACTAL}\n")
    }

    /// The one request from a caller that was gone: not identified, and nobody
    /// named — where a backend trusting the number would have named Fractal.
    fn not_identified(answers: &[Answered]) {
        let [answer] = answers else {
            panic!("one answer was not recorded: {answers:?}");
        };
        assert_eq!(
            answer.outcome(),
            &Outcome::Unanswered(Unanswered::NotIdentified),
            "{answer:?}"
        );
        assert_eq!(answer.application(), None, "{answer:?}");
    }

    /// A request from `asking`, whose caller is ended while its sandbox is read.
    fn a_request_whose_caller_is_gone(what: &str, asking: &str, portal: Portal) {
        let serving = Serving::started(what);
        let mut caller = serving.a_caller(asking);
        caller.sandboxed_behind_a_pipe();
        caller.go();
        caller.gone_while_its_sandbox_is_read();
        not_identified(&serving.recorded(portal, 1));
        std::thread::sleep(Duration::from_millis(200));
        not_identified(&serving.recorded(portal, 1));
    }

    /// **An `OpenFile` request whose caller is gone by the time its sandbox is
    /// read is `NotIdentified`**, and recorded naming nobody.
    #[test]
    fn an_open_file_request_whose_caller_is_gone_is_not_identified() {
        a_request_whose_caller_is_gone("held-open-file", "open-file", Portal::OpenWith);
    }

    /// **A `RetrieveSecret` request whose caller is gone is `NotIdentified`** —
    /// never handed to the keyring as Fractal's.
    #[test]
    fn a_secret_request_whose_caller_is_gone_is_not_identified() {
        a_request_whose_caller_is_gone("held-secret", "secret", Portal::Secret);
    }

    /// **A `ReadOne` whose caller is gone is `NotIdentified`** — never answered
    /// with the appearance Fractal is granted.
    #[test]
    fn a_settings_read_whose_caller_is_gone_is_not_identified() {
        a_request_whose_caller_is_gone("held-settings", "settings", Portal::Settings);
    }

    /// **A `SettingChanged` for a connection whose process is gone by the time
    /// its sandbox is read is not sent**, and that is recorded as not
    /// identified rather than as a signal sent to Fractal.
    #[test]
    fn a_change_for_a_connection_whose_process_is_gone_is_not_sent() {
        let serving = Serving::started("held-changed");
        let mut caller = serving.a_caller("listening");
        caller.sandboxed_behind_a_pipe();
        caller.go();
        caller.until_connected();
        serving.evening();
        caller.gone_while_its_sandbox_is_read();
        let answers = serving.recorded(Portal::Settings, 1);
        not_identified(&answers);
        assert!(
            !answers
                .iter()
                .any(|answer| matches!(answer.outcome(), Outcome::AppearanceSent(_))),
            "a change was sent for a process that was gone: {answers:?}"
        );
    }

    /// **A caller in another process that is still there when its sandbox has
    /// been read is named** — for each portal, and for the signal — so the
    /// refusals above are the process being gone and nothing else.
    #[test]
    fn a_caller_in_another_process_that_is_still_there_is_named() {
        let serving = Serving::started("held-named");
        for (asking, portal) in [
            ("open-file", Portal::OpenWith),
            ("secret", Portal::Secret),
            ("settings", Portal::Settings),
        ] {
            let mut caller = serving.a_caller(asking);
            caller.sandboxed_as_fractal();
            caller.go();
            let answers = serving.recorded(portal, 1);
            let [answer] = answers.as_slice() else {
                panic!("one answer was not recorded: {answers:?}");
            };
            assert_eq!(
                answer.application(),
                Some(&Applicant::named(FRACTAL)),
                "{answer:?}"
            );
            caller.finished();
        }
        assert!(matches!(
            serving
                .recorded(Portal::Settings, 1)
                .first()
                .map(Answered::outcome),
            Some(Outcome::AppearanceRead(_))
        ));

        let mut listening = serving.a_caller("listening");
        listening.sandboxed_as_fractal();
        listening.go();
        listening.until_connected();
        serving.evening();
        let answers = serving.recorded(Portal::Settings, 2);
        let sent = answers.get(1).expect("a change was recorded");
        assert!(
            matches!(sent.outcome(), Outcome::AppearanceSent(_)),
            "{sent:?}"
        );
        assert_eq!(sent.application(), Some(&Applicant::named(FRACTAL)));
        drop(listening);
    }
}
