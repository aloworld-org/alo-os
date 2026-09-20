//! The NetworkMonitor portal on a real bus: how far this machine reaches, to an
//! application granted it — and to nobody else.
//!
//! Task 12 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`.
//! The backend is served on a private session bus the test starts (with no
//! service activation, `alo-keyring-fixture`'s bus), and a `zbus` client — not a
//! mock — asks it as an application does:
//!
//! - **a granted application is answered what the machine reads, in the
//!   specification's numbering** —
//!   [`on_the_bus::a_granted_application_is_answered_what_the_machine_reads`];
//! - **an application not granted it is refused, and learns nothing a machine
//!   with no network would not tell it** —
//!   [`on_the_bus::an_ungranted_application_is_refused_and_told_nothing_more`];
//! - **`CanReach` is refused to a granted application too**, because answering
//!   sends a packet to a host it named —
//!   [`on_the_bus::can_reach_is_refused_to_a_granted_application_as_well`];
//! - **a machine that cannot read its own network refuses rather than claiming
//!   to be online** —
//!   [`on_the_bus::a_machine_that_cannot_read_its_network_refuses`];
//! - **`docs/contracts/portals.md` lists the network monitor as answered** —
//!   [`the_contract_lists_the_network_monitor_as_answered`].
//!
//! **No machine in this lane has NetworkManager.** What the machine reports is
//! this test's own `Reaching`, so nothing here claims what NetworkManager says
//! on real hardware. What is proven is what the portal does with a reading.

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

/// **The network monitor is in the contract's *Answered* table, on its
/// interface, and the row names what is answered and what is not** — so a
/// person building an application against this machine reads that `CanReach` is
/// never answered, and that the numbering is GLib's, rather than finding out.
#[test]
fn the_contract_lists_the_network_monitor_as_answered() {
    assert_eq!(
        Portal::NetworkMonitor.answered_on_the_bus(),
        Some("org.freedesktop.portal.NetworkMonitor")
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
        .find(|line| line.starts_with("| network monitor |"))
        .expect("the network monitor is listed as answered");
    for named in [
        "org.freedesktop.portal.NetworkMonitor",
        "GetAvailable",
        "GetMetered",
        "GetConnectivity",
        "GetStatus",
        "CanReach",
        "network-state",
        "org.freedesktop.portal.Error.NotAllowed",
    ] {
        assert!(
            row.contains(named),
            "the network monitor row does not name {named}"
        );
    }
    assert!(
        !not_yet
            .lines()
            .any(|line| line.starts_with("| network monitor |")),
        "the network monitor is still listed as not answered"
    );
}

#[cfg(target_os = "linux")]
mod on_the_bus {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};
    use std::time::{Duration, SystemTime};

    use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
    use alo_keyring_fixture::AKeyringOfOurOwn;
    use alo_networks::{HowFar, Metered};
    use alo_portals::{
        Allowed, Appearance, Applications, Backend, KeepsSecrets, Kept, NotKept, Outcome, Reaching,
        Request, Sandboxes, Served, TheMachine, TimeOfDay, Unanswered,
    };
    use zbus::blocking::Connection;
    use zbus::zvariant::OwnedValue;

    const NEWSFLASH: &str = "io.gitlab.news_flash.NewsFlash";
    const STRANGER: &str = "org.example.Stranger";

    const DESKTOP: &str = "org.freedesktop.portal.Desktop";
    const PORTALS: &str = "/org/freedesktop/portal/desktop";
    const MONITOR: &str = "org.freedesktop.portal.NetworkMonitor";
    const NOT_ALLOWED: &str = "org.freedesktop.portal.Error.NotAllowed";

    /// The machine as the backend reads it: grants and one network reading,
    /// both a test changes.
    struct ThisMachine {
        grants: RwLock<Grants>,
        reaching: RwLock<Option<Reaching>>,
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
        fn reaching(&self) -> Option<Reaching> {
            *self.reaching.read().unwrap()
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
        /// The backend on a bus of its own, over a machine reaching a café's
        /// sign-in page on a connection guessed to be metered — both
        /// deliberately mid-range, so a portal that rounded either is caught.
        fn started(what: &str) -> Self {
            let bus = AKeyringOfOurOwn::a_bus_with_no_keyring_on_it(what);
            let processes = std::env::temp_dir()
                .join(format!("alo-portal-network-{what}-{}", std::process::id()))
                .join("proc");
            std::fs::create_dir_all(processes.join(std::process::id().to_string()).join("root"))
                .unwrap();

            let machine = Arc::new(ThisMachine {
                grants: RwLock::new(Grants::default()),
                reaching: RwLock::new(Some(Reaching::reported(
                    HowFar::APageInTheWay,
                    Metered::ProbablyMetered,
                ))),
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

        fn grant(&self, application: &str, reach: Reach) {
            self.machine.grants.write().unwrap().grant(
                Grant::checked_for(
                    &Applicant::named(application).grantee(),
                    reach,
                    SystemTime::now(),
                    Duration::from_secs(3600),
                )
                .unwrap(),
            );
        }

        /// What the machine can say about its network from now on.
        fn reads(&self, reaching: Option<Reaching>) {
            *self.machine.reaching.write().unwrap() = reaching;
        }

        fn call<B>(&self, method: &str, body: &B) -> zbus::Result<zbus::Message>
        where
            B: zbus::export::serde::Serialize + zbus::zvariant::DynamicType,
        {
            self.client
                .call_method(Some(DESKTOP), PORTALS, Some(MONITOR), method, body)
        }

        fn available(&self) -> zbus::Result<bool> {
            self.call("GetAvailable", &())?.body().deserialize()
        }

        fn metered(&self) -> zbus::Result<bool> {
            self.call("GetMetered", &())?.body().deserialize()
        }

        fn connectivity(&self) -> zbus::Result<u32> {
            self.call("GetConnectivity", &())?.body().deserialize()
        }

        fn status(&self) -> zbus::Result<HashMap<String, OwnedValue>> {
            self.call("GetStatus", &())?.body().deserialize()
        }
    }

    impl Drop for Serving {
        fn drop(&mut self) {
            if let Some(place) = self.processes.parent() {
                let _ = std::fs::remove_dir_all(place);
            }
        }
    }

    /// The name and sentence of a `zbus` error, for a test that cares which one
    /// arrived rather than only that one did.
    fn refusal(why: &zbus::Error) -> String {
        match why {
            zbus::Error::MethodError(name, said, _) => {
                format!("{}: {}", name.as_str(), said.clone().unwrap_or_default())
            }
            other => panic!("not a refusal from the portal: {other}"),
        }
    }

    /// **A granted application is answered what the machine reads**, in the
    /// specification's numbering and not the network manager's.
    ///
    /// The machine reaches a café's sign-in page on a connection *guessed* to
    /// be metered. So `available` is true — an application that wants to show
    /// the sign-in page needs the connection — `connectivity` is `3`, GLib's
    /// *portal*, and `metered` is true, because a guess is enough to hold off
    /// spending somebody's data.
    #[test]
    fn a_granted_application_is_answered_what_the_machine_reads() {
        let serving = Serving::started("granted");
        serving.become_(Some(NEWSFLASH));
        serving.grant(NEWSFLASH, Reach::Facility(Facility::NetworkState));

        assert!(serving.available().unwrap());
        assert!(serving.metered().unwrap());
        assert_eq!(serving.connectivity().unwrap(), 3, "GLib's portal, not 2");

        let status = serving.status().unwrap();
        let value = |key: &str| status.get(key).expect("the key is in the status").clone();
        assert!(bool::try_from(value("available")).unwrap());
        assert!(bool::try_from(value("metered")).unwrap());
        assert_eq!(u32::try_from(value("connectivity")).unwrap(), 3);

        // A machine reaching everything on a connection somebody marked as not
        // metered is answered the other way round, off the same road.
        serving.reads(Some(Reaching::reported(
            HowFar::AllOfIt,
            Metered::Unmetered,
        )));
        assert!(serving.available().unwrap());
        assert!(!serving.metered().unwrap());
        assert_eq!(serving.connectivity().unwrap(), 4);

        // And a machine reaching this network and no further is GLib's
        // *limited*, 2 — where the network manager's own number was 3.
        serving.reads(Some(Reaching::reported(
            HowFar::OnlyThisNetwork,
            Metered::NotSaid,
        )));
        assert_eq!(serving.connectivity().unwrap(), 2);
        assert!(serving.metered().unwrap(), "nothing said holds off");

        // Every answer is in the record, named.
        let answers = serving.record.everything();
        assert!(answers.len() >= 8);
        assert!(
            answers
                .iter()
                .all(|answered| matches!(answered.outcome(), Outcome::NetworkRead(_))),
            "a granted application's reads are all recorded as network reads"
        );
        assert!(
            answers.iter().all(|answered| answered
                .application()
                .is_some_and(|who| who.as_str() == NEWSFLASH)),
            "every answer names the application"
        );
    }

    /// **An application not granted the network state is refused, and learns
    /// nothing a machine with no network would not tell it.**
    ///
    /// The same error and the same sentence for both, so being refused is not
    /// itself a way of finding out that this machine has a working connection.
    /// The record tells the two apart; the application is not told.
    #[test]
    fn an_ungranted_application_is_refused_and_told_nothing_more() {
        let serving = Serving::started("ungranted");
        serving.become_(Some(STRANGER));

        let refused = refusal(&serving.available().unwrap_err());
        assert!(refused.starts_with(NOT_ALLOWED), "{refused}");
        for asked in [
            serving.metered().err(),
            serving.connectivity().err(),
            serving.status().err(),
        ] {
            assert_eq!(refusal(&asked.expect("refused")), refused);
        }

        // A machine that cannot see its own network tells a granted
        // application exactly the same thing.
        let elsewhere = Serving::started("ungranted-vs-unread");
        elsewhere.become_(Some(NEWSFLASH));
        elsewhere.grant(NEWSFLASH, Reach::Facility(Facility::NetworkState));
        elsewhere.reads(None);
        assert_eq!(refusal(&elsewhere.available().unwrap_err()), refused);

        // The record knows which was which.
        assert!(matches!(
            serving.record.everything().first().map(|a| a.outcome()),
            Some(Outcome::Refused(_))
        ));
        assert!(matches!(
            elsewhere.record.everything().first().map(|a| a.outcome()),
            Some(Outcome::Unanswered(Unanswered::NetworkUnread))
        ));

        // A program with no sandbox at all is refused the same way.
        serving.become_(None);
        assert_eq!(refusal(&serving.available().unwrap_err()), refused);
    }

    /// **`CanReach` is refused to a granted application as well**, because
    /// answering it means sending a packet to a host the application named —
    /// on a machine whose first law is that nothing leaves silently.
    ///
    /// Refusing it to the granted and the ungranted alike also means the
    /// refusal says nothing about the grants.
    #[test]
    fn can_reach_is_refused_to_a_granted_application_as_well() {
        let serving = Serving::started("can-reach");
        serving.become_(Some(NEWSFLASH));
        serving.grant(NEWSFLASH, Reach::Facility(Facility::NetworkState));
        assert!(serving.available().unwrap(), "it is granted and answered");

        let granted = refusal(
            &serving
                .call("CanReach", &("example.org", 443_u32))
                .unwrap_err(),
        );
        assert!(granted.starts_with(NOT_ALLOWED), "{granted}");

        serving.become_(Some(STRANGER));
        let ungranted = refusal(
            &serving
                .call("CanReach", &("example.org", 443_u32))
                .unwrap_err(),
        );
        assert_eq!(granted, ungranted, "the refusal says nothing about grants");
    }

    /// **A machine that cannot read its own network refuses rather than
    /// claiming to be online**, and the record says why.
    ///
    /// An application told it is connected would keep retrying something that
    /// cannot work, on a person's battery.
    #[test]
    fn a_machine_that_cannot_read_its_network_refuses() {
        let serving = Serving::started("unread");
        serving.become_(Some(NEWSFLASH));
        serving.grant(NEWSFLASH, Reach::Facility(Facility::NetworkState));
        serving.reads(None);

        assert!(serving.available().is_err());
        assert!(serving.connectivity().is_err());
        assert!(matches!(
            serving.record.everything().last().map(|a| a.outcome()),
            Some(Outcome::Unanswered(Unanswered::NetworkUnread))
        ));

        // The network manager answering again is answered again: nothing is
        // held between requests.
        serving.reads(Some(Reaching::reported(
            HowFar::AllOfIt,
            Metered::Unmetered,
        )));
        assert_eq!(serving.connectivity().unwrap(), 4);
    }

    /// **A grant of some other facility is not a grant of the network state.**
    #[test]
    fn a_grant_of_something_else_does_not_reach_the_network() {
        let serving = Serving::started("other-facility");
        serving.become_(Some(NEWSFLASH));
        serving.grant(NEWSFLASH, Reach::Facility(Facility::Camera));
        let refused = refusal(&serving.available().unwrap_err());
        assert!(refused.starts_with(NOT_ALLOWED), "{refused}");
    }
}
