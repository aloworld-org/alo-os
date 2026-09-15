//! The Settings portal on a real bus: appearance, from what the person set, to
//! an application granted it — and to nobody else.
//!
//! Task 6 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`.
//! The backend is served on a private session bus the test starts (with no
//! service activation, `alo-keyring-fixture`'s bus), and a `zbus` client — not a
//! mock — asks it as an application does:
//!
//! - **an application granted the `appearance settings` facility receives the
//!   person's values through `ReadAll`, `ReadOne` and the deprecated `Read`** —
//!   [`on_the_bus::a_granted_application_reads_the_persons_appearance`];
//! - **an application not granted it receives the specification's error for a
//!   setting it cannot read, and nothing else, recorded with the application
//!   named** —
//!   [`on_the_bus::an_application_not_granted_receives_not_found_and_is_recorded_by_name`];
//! - **any other namespace is answered as unknown, never with a value** —
//!   [`on_the_bus::any_other_namespace_is_answered_as_unknown_and_never_with_a_value`];
//! - **`SettingChanged` is sent only to an application that could read the
//!   value at the moment it changed** —
//!   [`on_the_bus::a_change_is_sent_only_to_an_application_that_could_read_it`];
//! - **`docs/contracts/portals.md` lists `settings` as answered** —
//!   [`the_contract_lists_settings_as_answered_with_what_it_answers`].

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

/// **`settings` is in the contract's *Answered* table, on its interface, and
/// the row names what is answered and what is not** — so a person building an
/// application against this machine reads that `contrast` is not answered
/// rather than finding out.
#[test]
fn the_contract_lists_settings_as_answered_with_what_it_answers() {
    assert_eq!(
        Portal::Settings.answered_on_the_bus(),
        Some("org.freedesktop.portal.Settings")
    );
    let contract =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(THE_CONTRACT))
            .expect("docs/contracts/portals.md is there");
    let (answered, not_yet) = contract
        .split_once("## Answered")
        .and_then(|(_, rest)| rest.split_once("## Not answered yet"))
        .expect("the contract has an Answered section, then a Not answered yet section");
    let row = answered
        .lines()
        .find(|line| line.starts_with("| settings |"))
        .expect("settings is listed as answered");
    for named in [
        "org.freedesktop.portal.Settings",
        "ReadAll",
        "ReadOne",
        "SettingChanged",
        "org.freedesktop.appearance",
        "color-scheme",
        "accent-color",
        "contrast",
        "appearance-settings",
        "org.freedesktop.portal.Error.NotFound",
    ] {
        assert!(
            row.contains(named),
            "the settings row does not name {named}"
        );
    }
    assert!(
        !not_yet.lines().any(|line| line.starts_with("| settings |")),
        "settings is still listed as not answered"
    );
}

#[cfg(target_os = "linux")]
mod on_the_bus {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{Receiver, channel};
    use std::sync::{Arc, RwLock};
    use std::time::{Duration, Instant, SystemTime};

    use alo_appearance::{Accent, Following, Scheme, Shipped};
    use alo_capability::{Applicant, Facility, Grant, GrantId, Grants, Reach};
    use alo_keyring_fixture::AKeyringOfOurOwn;
    use alo_portals::watching_appearance::WATCHED_EVERY;
    use alo_portals::{
        Allowed, Answered, Appearance, Applications, Backend, KeepsSecrets, Kept, NotKept, Outcome,
        Portal, Refused, Request, Sandboxes, Served, TheMachine, TimeOfDay, Unanswered,
    };
    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::OwnedValue;

    const NEWSFLASH: &str = "io.gitlab.news_flash.NewsFlash";
    const STRANGER: &str = "org.example.Stranger";

    const DESKTOP: &str = "org.freedesktop.portal.Desktop";
    const PORTALS: &str = "/org/freedesktop/portal/desktop";
    const SETTINGS: &str = "org.freedesktop.portal.Settings";
    const APPEARANCE: &str = "org.freedesktop.appearance";
    const NOT_FOUND: &str = "org.freedesktop.portal.Error.NotFound";

    /// The machine as the backend reads it: grants, the person's appearance and
    /// the clock, each a test changes — and how many times the appearance was
    /// read, so a test knows the backend has looked since it changed something.
    struct ThisMachine {
        grants: RwLock<Grants>,
        appearance: RwLock<Appearance>,
        time: RwLock<TimeOfDay>,
        looked: AtomicUsize,
    }

    impl TheMachine for ThisMachine {
        fn grants(&self) -> Option<Grants> {
            Some(self.grants.read().unwrap().clone())
        }

        fn applications(&self) -> Option<Applications> {
            Some(Applications::default())
        }

        fn appearance(&self) -> Option<Appearance> {
            self.looked.fetch_add(1, Ordering::SeqCst);
            Some(self.appearance.read().unwrap().clone())
        }

        fn time_of_day(&self) -> Option<TimeOfDay> {
            Some(*self.time.read().unwrap())
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

    /// Everything one test serves and asks through.
    struct Serving {
        _bus: AKeyringOfOurOwn,
        machine: Arc<ThisMachine>,
        record: Arc<Kept>,
        processes: PathBuf,
        client: Connection,
        _served: Served,
    }

    impl Serving {
        /// The backend on a bus of its own, over a person who chose rose, dark
        /// after six by the shipped schedule, at noon.
        fn started(what: &str) -> Self {
            let bus = AKeyringOfOurOwn::a_bus_with_no_keyring_on_it(what);
            let processes = std::env::temp_dir()
                .join(format!("alo-portal-settings-{what}-{}", std::process::id()))
                .join("proc");
            std::fs::create_dir_all(processes.join(std::process::id().to_string()).join("root"))
                .unwrap();

            let mut appearance = Appearance::shipped();
            appearance.set_accent(Accent::Rose);
            appearance.follow(Following::from(Shipped::the_evening_schedule()));
            let machine = Arc::new(ThisMachine {
                grants: RwLock::new(Grants::default()),
                appearance: RwLock::new(appearance),
                time: RwLock::new(at(12)),
                looked: AtomicUsize::new(0),
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
            let client = zbus::blocking::connection::Builder::address(bus.address().as_str())
                .unwrap()
                .build()
                .unwrap();
            Self {
                _bus: bus,
                machine,
                record,
                processes,
                client,
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

        fn grant(&self, application: &str, reach: Reach, lasting: Duration) -> GrantId {
            self.machine.grants.write().unwrap().grant(
                Grant::checked_for(
                    &Applicant::named(application).grantee(),
                    reach,
                    SystemTime::now(),
                    lasting,
                )
                .unwrap(),
            )
        }

        fn revoke(&self, id: GrantId) {
            assert!(self.machine.grants.write().unwrap().revoke(id));
        }

        fn change(&self, how: impl FnOnce(&mut Appearance)) {
            how(&mut self.machine.appearance.write().unwrap());
        }

        fn set_clock(&self, hour: u8) {
            *self.machine.time.write().unwrap() = at(hour);
        }

        /// Wait until the backend has looked at the appearance twice since
        /// now: once to see what changed and send it, once more after that
        /// sending finished.
        fn until_the_backend_has_looked(&self) {
            let since = self.machine.looked.load(Ordering::SeqCst);
            let deadline = Instant::now() + WATCHED_EVERY * 20;
            while self.machine.looked.load(Ordering::SeqCst) < since + 2 {
                assert!(Instant::now() < deadline, "the backend stopped looking");
                std::thread::sleep(WATCHED_EVERY / 10);
            }
        }

        fn call<B>(&self, method: &str, body: &B) -> zbus::Result<zbus::Message>
        where
            B: zbus::export::serde::Serialize + zbus::zvariant::DynamicType,
        {
            self.client
                .call_method(Some(DESKTOP), PORTALS, Some(SETTINGS), method, body)
        }

        fn read_one(&self, namespace: &str, key: &str) -> zbus::Result<OwnedValue> {
            self.call("ReadOne", &(namespace, key))?
                .body()
                .deserialize()
        }

        fn read_all(
            &self,
            namespaces: &[&str],
        ) -> zbus::Result<HashMap<String, HashMap<String, OwnedValue>>> {
            self.call("ReadAll", &(namespaces,))?.body().deserialize()
        }

        /// Every `SettingChanged` the client is sent, as they arrive.
        fn changes(&self) -> Receiver<(String, String, OwnedValue)> {
            let proxy = Proxy::new(&self.client, DESKTOP, PORTALS, SETTINGS).unwrap();
            let signals = proxy.receive_signal("SettingChanged").unwrap();
            let (send, arrived) = channel();
            std::thread::spawn(move || {
                for signal in signals {
                    let body: (String, String, OwnedValue) = signal.body().deserialize().unwrap();
                    if send.send(body).is_err() {
                        return;
                    }
                }
            });
            arrived
        }

        fn last_answer(&self) -> Answered {
            self.record
                .everything()
                .pop()
                .expect("an answer was recorded")
        }

        fn sent(&self) -> Vec<Answered> {
            self.record
                .everything()
                .into_iter()
                .filter(|answered| matches!(answered.outcome(), Outcome::AppearanceSent(_)))
                .collect()
        }
    }

    impl Drop for Serving {
        fn drop(&mut self) {
            if let Some(place) = self.processes.parent() {
                let _ = std::fs::remove_dir_all(place);
            }
        }
    }

    fn at(hour: u8) -> TimeOfDay {
        TimeOfDay::checked(hour, 0).unwrap()
    }

    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn scheme(value: &OwnedValue) -> u32 {
        u32::try_from(value).unwrap()
    }

    /// The red, green and blue an `accent-color` value carries.
    fn accent(value: OwnedValue) -> [f64; 3] {
        let (red, green, blue) = <(f64, f64, f64)>::try_from(value).unwrap();
        [red, green, blue]
    }

    /// What `accent-color` is for this accent on this ground, worked out here
    /// from the colour `alo-appearance` gives: each channel over 255.
    fn rgb(accent: Accent, on: Scheme) -> [f64; 3] {
        let colour = accent.on(on);
        [colour.red(), colour.green(), colour.blue()].map(|channel| f64::from(channel) / 255.0)
    }

    /// Whether two `accent-color` values are the same colour.
    #[track_caller]
    fn same_colour(sent: [f64; 3], expected: [f64; 3]) {
        for (part, wanted) in sent.into_iter().zip(expected) {
            assert!((0.0..=1.0).contains(&part), "{part} is not from 0 to 1");
            assert!(
                (part - wanted).abs() < 1e-9,
                "sent {sent:?}, and the person's accent is {expected:?}"
            );
        }
    }

    fn is_not_found(refused: &zbus::Error) -> bool {
        matches!(refused, zbus::Error::MethodError(name, _, _) if name.as_str() == NOT_FOUND)
    }

    /// **An application granted the appearance settings receives the person's
    /// values**, resolved at the machine's time of day, through every method
    /// that reads them — and each read is recorded with it named.
    #[test]
    fn a_granted_application_reads_the_persons_appearance() {
        let serving = Serving::started("settings-granted");
        serving.grant(
            NEWSFLASH,
            Reach::Facility(Facility::AppearanceSettings),
            hour(),
        );
        serving.become_(Some(NEWSFLASH));

        // Noon, by a schedule that turns dark at six: light, and rose for light.
        assert_eq!(
            scheme(&serving.read_one(APPEARANCE, "color-scheme").unwrap()),
            2
        );
        same_colour(
            accent(serving.read_one(APPEARANCE, "accent-color").unwrap()),
            rgb(Accent::Rose, Scheme::Light),
        );
        let answered = serving.last_answer();
        assert_eq!(answered.portal(), Portal::Settings);
        assert_eq!(answered.application(), Some(&Applicant::named(NEWSFLASH)));
        match answered.outcome() {
            Outcome::AppearanceRead(allowed) => {
                assert_eq!(allowed.application(), &Applicant::named(NEWSFLASH));
                assert_eq!(allowed.against().len(), 1);
            }
            other => panic!("{other:?}"),
        }

        // Evening, and the person's own choice of dark.
        serving.set_clock(19);
        for asked in [
            vec![APPEARANCE],
            vec![],
            vec![""],
            vec!["org.freedesktop.*"],
        ] {
            let mut everything = serving.read_all(&asked).unwrap();
            assert_eq!(everything.len(), 1, "{asked:?}: {everything:?}");
            let mut appearance = everything.remove(APPEARANCE).expect("the namespace");
            let mut keys: Vec<&str> = appearance.keys().map(String::as_str).collect();
            keys.sort_unstable();
            assert_eq!(
                keys,
                ["accent-color", "color-scheme"],
                "no contrast is invented"
            );
            assert_eq!(scheme(appearance.get("color-scheme").unwrap()), 1);
            same_colour(
                accent(appearance.remove("accent-color").unwrap()),
                rgb(Accent::Rose, Scheme::Dark),
            );
        }

        // The deprecated `Read`, which wraps the value in one more variant.
        let wrapped: OwnedValue = serving
            .call("Read", &(APPEARANCE, "color-scheme"))
            .unwrap()
            .body()
            .deserialize()
            .unwrap();
        let inner = zbus::zvariant::Value::from(wrapped);
        match inner {
            zbus::zvariant::Value::Value(value) => assert_eq!(*value, 1_u32.into()),
            other => panic!("Read did not wrap its value: {other:?}"),
        }

        let version: u32 = Proxy::new(&serving.client, DESKTOP, PORTALS, SETTINGS)
            .unwrap()
            .get_property("version")
            .unwrap();
        assert_eq!(version, 2);
        assert!(
            serving
                .record
                .everything()
                .iter()
                .all(|answered| matches!(answered.outcome(), Outcome::AppearanceRead(_))),
            "{:?}",
            serving.record.everything()
        );
    }

    /// **An application not granted the appearance settings receives
    /// `org.freedesktop.portal.Error.NotFound` and nothing else**, from every
    /// method, and is recorded as refused with its application named — an
    /// application granted nothing, one granted something else, one whose grant
    /// was revoked, one whose grant ran out, and a program with no sandbox.
    #[test]
    fn an_application_not_granted_receives_not_found_and_is_recorded_by_name() {
        let serving = Serving::started("settings-refused");

        let asks_everything = |serving: &Serving| {
            for refused in [
                serving.read_one(APPEARANCE, "color-scheme").unwrap_err(),
                serving.read_one(APPEARANCE, "accent-color").unwrap_err(),
                serving.read_all(&[APPEARANCE]).unwrap_err(),
                serving.read_all(&[]).unwrap_err(),
                serving
                    .call("Read", &(APPEARANCE, "color-scheme"))
                    .unwrap_err(),
            ] {
                assert!(is_not_found(&refused), "{refused:?}");
            }
        };

        // Granted nothing at all.
        serving.become_(Some(STRANGER));
        asks_everything(&serving);
        let answered = serving.last_answer();
        assert_eq!(answered.portal(), Portal::Settings);
        assert_eq!(answered.application(), Some(&Applicant::named(STRANGER)));
        assert_eq!(
            answered.outcome(),
            &Outcome::Refused(Refused::NothingGranted {
                application: Applicant::named(STRANGER),
                portal: Portal::Settings,
            })
        );

        // Granted the camera, and not the appearance settings.
        serving.grant(NEWSFLASH, Reach::Facility(Facility::Camera), hour());
        serving.become_(Some(NEWSFLASH));
        asks_everything(&serving);
        let answered = serving.last_answer();
        assert_eq!(answered.application(), Some(&Applicant::named(NEWSFLASH)));
        assert!(
            matches!(
                answered.outcome(),
                Outcome::Refused(Refused::NotAllowed {
                    portal: Portal::Settings,
                    ..
                })
            ),
            "{answered:?}"
        );

        // Granted, read, and revoked: refused at the next request.
        let granted = serving.grant(
            NEWSFLASH,
            Reach::Facility(Facility::AppearanceSettings),
            hour(),
        );
        assert_eq!(
            scheme(&serving.read_one(APPEARANCE, "color-scheme").unwrap()),
            2
        );
        serving.revoke(granted);
        asks_everything(&serving);
        assert!(
            matches!(
                serving.last_answer().outcome(),
                Outcome::Refused(Refused::NotAllowed { .. })
            ),
            "{:?}",
            serving.last_answer()
        );

        // Granted for a moment that has passed.
        serving.grant(
            NEWSFLASH,
            Reach::Facility(Facility::AppearanceSettings),
            Duration::from_secs(1),
        );
        std::thread::sleep(Duration::from_millis(1_100));
        asks_everything(&serving);
        assert!(
            matches!(
                serving.last_answer().outcome(),
                Outcome::Refused(Refused::NotAllowed { .. })
            ),
            "{:?}",
            serving.last_answer()
        );

        // A program with no sandbox is nobody, and nobody is refused too.
        serving.become_(None);
        asks_everything(&serving);
        let answered = serving.last_answer();
        assert_eq!(answered.application(), None);
        assert_eq!(
            answered.outcome(),
            &Outcome::Unanswered(Unanswered::NotIdentified)
        );

        // Every refusal was recorded, and only the one granted read was not one.
        let everything = serving.record.everything();
        assert_eq!(everything.len(), 5 * 5 + 1, "{everything:#?}");
        assert_eq!(
            everything
                .iter()
                .filter(|answered| !answered.outcome().was_refused())
                .count(),
            1
        );
    }

    /// **Any other namespace, and any key the machine does not hold, is
    /// answered as unknown and never with a value** — to an application granted
    /// the appearance settings as to one granted nothing — and recorded.
    #[test]
    fn any_other_namespace_is_answered_as_unknown_and_never_with_a_value() {
        let serving = Serving::started("settings-unknown");
        serving.grant(
            NEWSFLASH,
            Reach::Facility(Facility::AppearanceSettings),
            hour(),
        );
        for application in [NEWSFLASH, STRANGER] {
            serving.become_(Some(application));
            for (namespace, key) in [
                ("org.gnome.desktop.interface", "color-scheme"),
                ("org.gnome.desktop.interface", "gtk-theme"),
                ("org.kde.kdeglobals.General", "ColorScheme"),
                (APPEARANCE, "contrast"),
                (APPEARANCE, "anything"),
                ("org.freedesktop.appearance.color-scheme", ""),
            ] {
                let refused = serving.read_one(namespace, key).unwrap_err();
                assert!(is_not_found(&refused), "{namespace} {key}: {refused:?}");
                let answered = serving.last_answer();
                assert_eq!(
                    answered.outcome(),
                    &Outcome::Unanswered(Unanswered::NoSuchSetting)
                );
                assert_eq!(answered.application(), Some(&Applicant::named(application)));
            }
            for asked in [
                vec!["org.gnome.desktop.interface"],
                vec!["org.gnome.*", "org.kde.*"],
                vec!["org.freedesktop.appearance.*"],
            ] {
                assert!(
                    serving.read_all(&asked).unwrap().is_empty(),
                    "{application} was answered for {asked:?}"
                );
            }
        }
        assert!(
            serving
                .record
                .everything()
                .iter()
                .all(|answered| answered.outcome()
                    == &Outcome::Unanswered(Unanswered::NoSuchSetting)),
            "{:?}",
            serving.record.everything()
        );
    }

    /// **`SettingChanged` is sent only to an application that could read the
    /// value at the moment it changed.** The client listens throughout; what
    /// it receives, in order, is exactly the changes made while its
    /// application held the grant — the clock crossing six while it was a
    /// stranger, and the evening falling after its grant was revoked, never
    /// reach it, and the first signal after each is the next change it could
    /// read.
    #[test]
    fn a_change_is_sent_only_to_an_application_that_could_read_it() {
        let serving = Serving::started("settings-changed");
        let changes = serving.changes();
        let next = || {
            changes
                .recv_timeout(WATCHED_EVERY * 20)
                .expect("a SettingChanged arrived")
        };

        // A stranger, while the clock crosses six: nothing is sent.
        serving.become_(Some(STRANGER));
        serving.set_clock(19);
        serving.until_the_backend_has_looked();
        assert!(serving.sent().is_empty(), "{:?}", serving.sent());

        // Granted, and the person picks moss: that alone arrives, for dark.
        let granted = serving.grant(
            NEWSFLASH,
            Reach::Facility(Facility::AppearanceSettings),
            hour(),
        );
        serving.become_(Some(NEWSFLASH));
        serving.change(|appearance| appearance.set_accent(Accent::Moss));
        let (namespace, key, value) = next();
        assert_eq!(
            (namespace.as_str(), key.as_str()),
            (APPEARANCE, "accent-color"),
            "the change made while it was a stranger reached it"
        );
        same_colour(accent(value), rgb(Accent::Moss, Scheme::Dark));
        let sent = serving.sent();
        assert_eq!(sent.len(), 1, "{sent:?}");
        let sent = sent.first().unwrap();
        assert_eq!(sent.application(), Some(&Applicant::named(NEWSFLASH)));
        assert_eq!(sent.portal(), Portal::Settings);

        // Morning, by the clock alone and no setting touched: both arrive.
        serving.set_clock(8);
        let (_, key, value) = next();
        assert_eq!((key.as_str(), scheme(&value)), ("color-scheme", 2));
        let (_, key, value) = next();
        assert_eq!(key, "accent-color");
        same_colour(accent(value), rgb(Accent::Moss, Scheme::Light));

        // Revoked, while the evening falls: nothing is sent.
        serving.until_the_backend_has_looked();
        serving.revoke(granted);
        let sent_before = serving.sent().len();
        serving.set_clock(20);
        serving.until_the_backend_has_looked();
        assert_eq!(serving.sent().len(), sent_before);

        // Granted again, and the person picks rose: that is the next signal.
        serving.grant(
            NEWSFLASH,
            Reach::Facility(Facility::AppearanceSettings),
            hour(),
        );
        serving.change(|appearance| appearance.set_accent(Accent::Rose));
        let (_, key, value) = next();
        assert_eq!(
            key, "accent-color",
            "the evening that fell while it was revoked reached it"
        );
        same_colour(accent(value), rgb(Accent::Rose, Scheme::Dark));

        // And a program with no sandbox is sent nothing either.
        serving.until_the_backend_has_looked();
        serving.become_(None);
        let sent_before = serving.sent().len();
        serving.change(|appearance| appearance.follow(Following::from(Scheme::Light)));
        serving.until_the_backend_has_looked();
        assert_eq!(serving.sent().len(), sent_before);
        assert!(
            changes.try_recv().is_err(),
            "a program with no sandbox was sent a change"
        );
    }
}
