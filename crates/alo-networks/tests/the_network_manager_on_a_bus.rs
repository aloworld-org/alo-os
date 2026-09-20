//! The client and the secret agent, speaking the network manager's own interface
//! on a real bus.
//!
//! A private `dbus-daemon` this test starts, with no service activation, and on
//! it a stand-in network manager served with `zbus` under the network manager's
//! own name, objects and interfaces. It is **not** NetworkManager, and nothing
//! here claims what NetworkManager itself does on a machine — the report for the
//! network task says what was not measured. What it proves is that the client
//! asks for exactly what it says it asks for and changes exactly what it was
//! told to, over a real bus, and that the secret agent answers the network
//! manager's connection and refuses everybody else's.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;
use std::os::unix::fs::MetadataExt as _;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use alo_networks::network_manager::NetworkManager;
use alo_networks::secret_agent::{SecretAgent, ThePersonsOwnSurface, registered};
use alo_networks::{
    HowFar, Metered, NetworkName, NetworkService, Networks, NotDone, Primary, Protection,
    TheNetworks, WhatIsReached, WifiPassword,
};
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

/// A bus of this test's own.
struct ABus {
    /// Where everything it made is.
    place: PathBuf,
    /// The daemon.
    daemon: Child,
}

impl ABus {
    /// Start one, with no service activation.
    fn started(what: &str) -> Self {
        let place =
            std::env::temp_dir().join(format!("alo-networks-{}-{what}", std::process::id()));
        drop(std::fs::remove_dir_all(&place));
        std::fs::create_dir_all(&place).unwrap();
        let socket = place.join("bus");
        let config = place.join("bus.conf");
        std::fs::write(
            &config,
            format!(
                r#"<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <listen>unix:path={}</listen>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
"#,
                socket.display()
            ),
        )
        .unwrap();
        let daemon = Command::new("dbus-daemon")
            .arg("--config-file")
            .arg(&config)
            .arg("--nofork")
            .arg("--nopidfile")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("this machine has `dbus-daemon`");
        let until = Instant::now() + Duration::from_secs(15);
        while !socket.exists() {
            assert!(Instant::now() < until, "the bus never appeared");
            std::thread::sleep(Duration::from_millis(50));
        }
        Self { place, daemon }
    }

    /// Its address.
    fn address(&self) -> String {
        format!("unix:path={}", self.place.join("bus").display())
    }

    /// A connection to it.
    fn connection(&self) -> Connection {
        zbus::blocking::connection::Builder::address(self.address().as_str())
            .unwrap()
            .method_timeout(Duration::from_secs(20))
            .build()
            .unwrap()
    }

    /// The user every connection to it runs as, which is this test's.
    fn our_user(&self) -> u32 {
        std::fs::metadata(self.place.join("bus")).unwrap().uid()
    }
}

impl Drop for ABus {
    fn drop(&mut self) {
        drop(self.daemon.kill());
        drop(self.daemon.wait());
        drop(std::fs::remove_dir_all(&self.place));
    }
}

/// What the stand-in was asked to do, in order.
type Asked = Arc<Mutex<Vec<String>>>;

/// A connection's settings.
type Settings = HashMap<String, HashMap<String, OwnedValue>>;

/// An object path.
fn object(at: &str) -> OwnedObjectPath {
    OwnedObjectPath::try_from(at).unwrap()
}

/// A value.
fn value<'a>(of: impl Into<Value<'a>>) -> OwnedValue {
    OwnedValue::try_from(of.into()).unwrap()
}

/// The manager.
struct Manager {
    /// What it was asked.
    asked: Asked,
    /// Whether the radio is on.
    wireless: bool,
}

#[zbus::interface(name = "org.freedesktop.NetworkManager")]
impl Manager {
    /// The devices: a wired one, then a wireless one.
    fn get_devices(&self) -> Vec<OwnedObjectPath> {
        vec![
            object("/org/freedesktop/NetworkManager/Devices/0"),
            object("/org/freedesktop/NetworkManager/Devices/1"),
        ]
    }

    /// Activate a saved connection.
    fn activate_connection(
        &self,
        connection: OwnedObjectPath,
        device: OwnedObjectPath,
        specific: OwnedObjectPath,
    ) -> OwnedObjectPath {
        self.asked.lock().unwrap().push(format!(
            "activate {} on {} through {}",
            connection.as_str(),
            device.as_str(),
            specific.as_str()
        ));
        object("/org/freedesktop/NetworkManager/ActiveConnection/1")
    }

    /// Add a connection and activate it.
    fn add_and_activate_connection(
        &self,
        settings: Settings,
        device: OwnedObjectPath,
        specific: OwnedObjectPath,
    ) -> (OwnedObjectPath, OwnedObjectPath) {
        self.asked.lock().unwrap().push(format!(
            "add {} settings and activate on {} through {}",
            settings.len(),
            device.as_str(),
            specific.as_str()
        ));
        (
            object("/org/freedesktop/NetworkManager/Settings/9"),
            object("/org/freedesktop/NetworkManager/ActiveConnection/1"),
        )
    }

    /// Whether the radio is on.
    #[zbus(property, name = "WirelessEnabled")]
    fn wireless_enabled(&self) -> bool {
        self.wireless
    }

    /// Turning the radio on or off.
    #[zbus(property, name = "WirelessEnabled")]
    fn set_wireless_enabled(&mut self, on: bool) {
        self.asked.lock().unwrap().push(format!("wireless {on}"));
        self.wireless = on;
    }

    /// The connection sent through.
    #[zbus(property, name = "PrimaryConnection")]
    fn primary_connection(&self) -> OwnedObjectPath {
        object("/org/freedesktop/NetworkManager/ActiveConnection/0")
    }

    /// How far the machine reaches: `NM_CONNECTIVITY_PORTAL`, a sign-in page in
    /// the way. Chosen because it is neither end of the range, so a client that
    /// rounded to *online* or to *offline* would be caught.
    #[zbus(property, name = "Connectivity")]
    fn connectivity(&self) -> u32 {
        2
    }

    /// Whether the way out is a meter: `NM_METERED_GUESS_YES`, a guess rather
    /// than somebody's answer — the two the client is required to keep apart.
    #[zbus(property, name = "Metered")]
    fn metered(&self) -> u32 {
        3
    }
}

/// A device.
struct Device(u32);

#[zbus::interface(name = "org.freedesktop.NetworkManager.Device")]
impl Device {
    /// Its type.
    #[zbus(property, name = "DeviceType")]
    fn device_type(&self) -> u32 {
        self.0
    }
}

/// The wireless device.
struct Wireless;

#[zbus::interface(name = "org.freedesktop.NetworkManager.Device.Wireless")]
impl Wireless {
    /// Every access point it sees.
    fn get_all_access_points(&self) -> Vec<OwnedObjectPath> {
        (1..=4)
            .map(|n| object(&format!("/org/freedesktop/NetworkManager/AccessPoint/{n}")))
            .collect()
    }
}

/// An access point.
struct AccessPoint {
    /// Its name.
    ssid: Vec<u8>,
    /// `Flags`.
    flags: u32,
    /// `RsnFlags`.
    rsn: u32,
    /// `Strength`.
    strength: u8,
}

#[zbus::interface(name = "org.freedesktop.NetworkManager.AccessPoint")]
impl AccessPoint {
    /// Its name.
    #[zbus(property, name = "Ssid")]
    fn ssid(&self) -> Vec<u8> {
        self.ssid.clone()
    }
    /// Its flags.
    #[zbus(property, name = "Flags")]
    fn flags(&self) -> u32 {
        self.flags
    }
    /// Its WPA flags.
    #[zbus(property, name = "WpaFlags")]
    fn wpa_flags(&self) -> u32 {
        0
    }
    /// Its RSN flags.
    #[zbus(property, name = "RsnFlags")]
    fn rsn_flags(&self) -> u32 {
        self.rsn
    }
    /// Its strength.
    #[zbus(property, name = "Strength")]
    fn strength(&self) -> u8 {
        self.strength
    }
}

/// The saved connections.
struct SettingsList;

#[zbus::interface(name = "org.freedesktop.NetworkManager.Settings")]
impl SettingsList {
    /// Both of them.
    fn list_connections(&self) -> Vec<OwnedObjectPath> {
        vec![
            object("/org/freedesktop/NetworkManager/Settings/0"),
            object("/org/freedesktop/NetworkManager/Settings/1"),
        ]
    }
}

/// One saved connection.
struct SavedConnection {
    /// What it was asked.
    asked: Asked,
    /// Where it is.
    at: &'static str,
    /// Its type.
    kind: &'static str,
    /// Its identifier.
    uuid: &'static str,
    /// Its network's name, for a Wi-Fi one.
    ssid: &'static [u8],
}

#[zbus::interface(name = "org.freedesktop.NetworkManager.Settings.Connection")]
impl SavedConnection {
    /// Its settings.
    fn get_settings(&self) -> Settings {
        let mut settings = HashMap::from([(
            "connection".to_owned(),
            HashMap::from([
                ("type".to_owned(), value(self.kind)),
                ("uuid".to_owned(), value(self.uuid)),
            ]),
        )]);
        if !self.ssid.is_empty() {
            settings.insert(
                "802-11-wireless".to_owned(),
                HashMap::from([("ssid".to_owned(), value(self.ssid.to_vec()))]),
            );
        }
        settings
    }

    /// Forget it.
    fn delete(&self) {
        self.asked
            .lock()
            .unwrap()
            .push(format!("delete {}", self.at));
    }
}

/// A connection in use.
struct ActiveConnection {
    /// Its type.
    kind: &'static str,
    /// Its identifier.
    uuid: &'static str,
    /// Its state.
    state: Arc<Mutex<u32>>,
}

#[zbus::interface(name = "org.freedesktop.NetworkManager.Connection.Active")]
impl ActiveConnection {
    /// Its type.
    #[zbus(property, name = "Type")]
    fn kind(&self) -> String {
        self.kind.to_owned()
    }
    /// Its identifier.
    #[zbus(property, name = "Uuid")]
    fn uuid(&self) -> String {
        self.uuid.to_owned()
    }
    /// Its state.
    #[zbus(property, name = "State")]
    fn state(&self) -> u32 {
        *self.state.lock().unwrap()
    }
}

/// Where secret agents register.
struct AgentManager(Asked);

#[zbus::interface(name = "org.freedesktop.NetworkManager.AgentManager")]
impl AgentManager {
    /// Register an agent.
    fn register(&self, identifier: String) {
        self.0
            .lock()
            .unwrap()
            .push(format!("register {identifier}"));
    }
}

/// The stand-in network manager, on `bus`, joining with `joined` as its state.
fn a_network_manager(bus: &ABus, asked: &Asked, joined: u32) -> Connection {
    let at = |n: &str| format!("/org/freedesktop/NetworkManager/{n}");
    let point = |ssid: &[u8], flags, rsn, strength| AccessPoint {
        ssid: ssid.to_vec(),
        flags,
        rsn,
        strength,
    };
    zbus::blocking::connection::Builder::address(bus.address().as_str())
        .unwrap()
        .name("org.freedesktop.NetworkManager")
        .unwrap()
        .serve_at(
            "/org/freedesktop/NetworkManager",
            Manager {
                asked: asked.clone(),
                wireless: true,
            },
        )
        .unwrap()
        .serve_at(at("Devices/0"), Device(1))
        .unwrap()
        .serve_at(at("Devices/1"), Device(2))
        .unwrap()
        .serve_at(at("Devices/1"), Wireless)
        .unwrap()
        .serve_at(at("AccessPoint/1"), point(b"Home", 1, 0x100, 40))
        .unwrap()
        .serve_at(at("AccessPoint/2"), point(b"Home", 1, 0x100, 80))
        .unwrap()
        .serve_at(at("AccessPoint/3"), point(b"Cafe", 0, 0, 60))
        .unwrap()
        .serve_at(at("AccessPoint/4"), point(b"", 0, 0, 90))
        .unwrap()
        .serve_at(at("Settings"), SettingsList)
        .unwrap()
        .serve_at(
            at("Settings/0"),
            SavedConnection {
                asked: asked.clone(),
                at: "/org/freedesktop/NetworkManager/Settings/0",
                kind: "802-3-ethernet",
                uuid: "wired-uuid",
                ssid: b"",
            },
        )
        .unwrap()
        .serve_at(
            at("Settings/1"),
            SavedConnection {
                asked: asked.clone(),
                at: "/org/freedesktop/NetworkManager/Settings/1",
                kind: "802-11-wireless",
                uuid: "home-uuid",
                ssid: b"Home",
            },
        )
        .unwrap()
        .serve_at(
            at("ActiveConnection/0"),
            ActiveConnection {
                kind: "802-11-wireless",
                uuid: "home-uuid",
                state: Arc::new(Mutex::new(2)),
            },
        )
        .unwrap()
        .serve_at(
            at("ActiveConnection/1"),
            ActiveConnection {
                kind: "802-11-wireless",
                uuid: "joining",
                state: Arc::new(Mutex::new(joined)),
            },
        )
        .unwrap()
        .serve_at(at("AgentManager"), AgentManager(asked.clone()))
        .unwrap()
        .build()
        .unwrap()
}

/// A name.
fn named(name: &str) -> NetworkName {
    NetworkName::announced(name.as_bytes()).unwrap()
}

/// **How far the machine reaches is read from the network manager, and is
/// neither guessed nor rounded.**
///
/// The stand-in reports a sign-in page in the way and a *guess* that the
/// connection is metered — both deliberately in the middle of their ranges. A
/// client that inferred reachability from having a primary connection would say
/// the machine reaches everything, and one that flattened the guess to a plain
/// yes or no would lose the distinction a person is owed before their data is
/// spent. This reads the two properties off the network manager's own object.
///
/// What this does **not** show is what NetworkManager itself reports on a real
/// machine: this is a stand-in on a private bus, and no machine in this lane has
/// a network manager. What is proven is the asking and the reading.
#[test]
fn how_far_the_machine_reaches_is_read_from_the_network_manager_and_not_guessed() {
    let bus = ABus::started("reaching");
    let asked = Asked::default();
    let _service = a_network_manager(&bus, &asked, 2);
    let client = NetworkManager::at(&bus.address()).unwrap();

    let reaching = client.reaching_now().unwrap();
    assert_eq!(reaching.how_far, HowFar::APageInTheWay);
    assert_eq!(reaching.metered, Metered::ProbablyMetered);

    // The machine has a primary connection, and still does not reach past the
    // sign-in page: the two questions are answered apart.
    assert!(client.now().unwrap().primary.is_some());
    assert!(reaching.how_far.reaches_anything());
    assert_ne!(reaching.how_far, HowFar::AllOfIt);

    // A guess that it is metered holds off what would spend a person's data.
    assert!(reaching.metered.should_hold_off());
    assert_ne!(reaching.metered, Metered::Metered);
}

/// **What the network manager reports is what the client says**: one network
/// per name and protection through its strongest access point, a network with
/// no name left out, the saved Wi-Fi networks and not the wired one, the radio,
/// and the connection the machine sends through.
#[test]
fn the_network_manager_reports_what_it_sees_has_saved_and_sends_through() {
    let bus = ABus::started("reports");
    let asked = Asked::default();
    let _service = a_network_manager(&bus, &asked, 2);
    let now = NetworkManager::at(&bus.address()).unwrap().now().unwrap();

    let seen: Vec<(Option<&str>, Protection, u8)> = now
        .visible
        .iter()
        .map(|network| {
            (
                network.name().called(),
                network.protection(),
                network.strength(),
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            (Some("Home"), Protection::Password, 80),
            (Some("Cafe"), Protection::Open, 60),
        ]
    );
    let saved: Vec<(Option<&str>, &str)> = now
        .saved
        .iter()
        .map(|saved| (saved.name().called(), saved.uuid()))
        .collect();
    assert_eq!(saved, [(Some("Home"), "home-uuid")]);
    assert!(now.wireless_on);
    assert_eq!(now.primary, Some(Primary::reported(true, "home-uuid")));
    assert_eq!(now.joined().map(|saved| saved.uuid()), Some("home-uuid"));
    assert!(
        asked.lock().unwrap().is_empty(),
        "reading changed something"
    );
}

/// **Each change is asked of the network manager against exactly what it
/// reported**: a network with no saved connection is added from its access
/// point with no settings of ours at all — so no password — a saved one is
/// activated rather than added twice, forgetting deletes that one saved
/// connection, and the radio is switched through the manager's own property.
#[test]
fn each_change_is_made_against_exactly_what_the_network_manager_reported() {
    let bus = ABus::started("changes");
    let asked = Asked::default();
    let _service = a_network_manager(&bus, &asked, 2);
    let client = NetworkManager::at(&bus.address()).unwrap();
    let TheNetworks { visible, saved, .. } = client.now().unwrap();
    let cafe = visible
        .iter()
        .find(|network| network.name() == &named("Cafe"))
        .unwrap();
    let home = visible
        .iter()
        .find(|network| network.name() == &named("Home"))
        .unwrap();

    client.join(cafe).unwrap();
    client.join(home).unwrap();
    client.forget(saved.first().unwrap()).unwrap();
    client.switch_wireless(false).unwrap();
    assert!(!client.now().unwrap().wireless_on);

    assert_eq!(
        *asked.lock().unwrap(),
        [
            "add 0 settings and activate on /org/freedesktop/NetworkManager/Devices/1 through \
             /org/freedesktop/NetworkManager/AccessPoint/3",
            "activate /org/freedesktop/NetworkManager/Settings/1 on \
             /org/freedesktop/NetworkManager/Devices/1 through \
             /org/freedesktop/NetworkManager/AccessPoint/2",
            "delete /org/freedesktop/NetworkManager/Settings/1",
            "wireless false",
        ]
    );
}

/// **A join the network manager gave up on is not a join, and neither is one
/// that never finished** — nor a network it never reported, nor one asking for
/// an organisation's sign-in.
#[test]
fn a_join_that_did_not_happen_is_not_said_to_have_happened() {
    let bus = ABus::started("gave-up");
    let asked = Asked::default();
    let _service = a_network_manager(&bus, &asked, 4);
    let client = NetworkManager::at(&bus.address()).unwrap();
    let cafe = client
        .now()
        .unwrap()
        .visible
        .into_iter()
        .find(|network| network.name() == &named("Cafe"))
        .unwrap();
    assert!(matches!(client.join(&cafe), Err(NotDone(_))));

    let still_joining = ABus::started("still-joining");
    let _slow = a_network_manager(&still_joining, &Asked::default(), 1);
    let impatient = NetworkManager::at(&still_joining.address())
        .unwrap()
        .waiting(Duration::from_millis(600));
    let cafe = impatient
        .now()
        .unwrap()
        .visible
        .into_iter()
        .find(|network| network.name() == &named("Cafe"))
        .unwrap();
    assert!(matches!(impatient.join(&cafe), Err(NotDone(_))));

    let unasked = asked.lock().unwrap().len();
    let invented = alo_networks::Visible::reported(named("Cafe"), Protection::Open, 60);
    assert!(client.join(&invented).is_err());
    let sign_in = alo_networks::Visible::reported(named("Work"), Protection::Enterprise, 60);
    assert!(client.join(&sign_in).is_err());
    assert_eq!(asked.lock().unwrap().len(), unasked);
}

/// Where the person is asked, counting.
struct Asking {
    /// How many times.
    asked: Arc<Mutex<Vec<String>>>,
    /// What they type, or nothing to decline.
    typing: Option<&'static str>,
}

impl ThePersonsOwnSurface for Asking {
    fn password_for(&self, network: &NetworkName) -> Option<WifiPassword> {
        self.asked
            .lock()
            .unwrap()
            .push(network.called().unwrap_or("?").to_owned());
        self.typing.map(|typed| WifiPassword::typed(typed).unwrap())
    }
}

/// Ask the agent on `person` for secrets, from `caller`.
fn get_secrets(
    caller: &Connection,
    person: &Connection,
    setting: &str,
    flags: u32,
) -> Result<Settings, zbus::Error> {
    let connection: HashMap<&str, HashMap<&str, Value<'_>>> = HashMap::from([(
        "802-11-wireless",
        HashMap::from([("ssid", Value::from(b"Home".to_vec()))]),
    )]);
    caller
        .call_method(
            person.unique_name().map(|name| name.as_str()),
            "/org/freedesktop/NetworkManager/SecretAgent",
            Some("org.freedesktop.NetworkManager.SecretAgent"),
            "GetSecrets",
            &(
                connection,
                object("/org/freedesktop/NetworkManager/Settings/1"),
                setting,
                Vec::<String>::new(),
                flags,
            ),
        )?
        .body()
        .deserialize::<Settings>()
}

/// **The password a person types goes to the network manager, and to nobody
/// else**: the network manager's own connection is answered with it; a
/// stranger on the same bus — anybody who could send to the person's
/// connection, an agent's own login among them — is refused and the person is
/// never asked; the person is not asked when the network manager says they may
/// not be, nor for an organisation's sign-in; and a person who declines is
/// said to have declined.
#[test]
fn the_password_is_given_to_the_network_manager_and_nobody_else() {
    let bus = ABus::started("secrets");
    let asked = Asked::default();
    let network_manager = a_network_manager(&bus, &asked, 2);
    let surface = Arc::new(Mutex::new(Vec::new()));
    let person = registered(
        &bus.address(),
        SecretAgent::answering(
            Box::new(Asking {
                asked: surface.clone(),
                typing: Some("correct horse"),
            }),
            bus.our_user(),
        ),
    )
    .unwrap();
    assert_eq!(*asked.lock().unwrap(), ["register world.alo.networks"]);

    let stranger = bus.connection();
    assert!(get_secrets(&stranger, &person, "802-11-wireless-security", 1).is_err());
    assert!(
        surface.lock().unwrap().is_empty(),
        "a stranger made the person be asked"
    );

    assert!(get_secrets(&network_manager, &person, "802-11-wireless-security", 0).is_err());
    assert!(get_secrets(&network_manager, &person, "802-1x", 1).is_err());
    assert!(surface.lock().unwrap().is_empty());

    let given = get_secrets(&network_manager, &person, "802-11-wireless-security", 1).unwrap();
    let psk = given
        .get("802-11-wireless-security")
        .and_then(|security| security.get("psk"))
        .map(|psk| String::try_from(psk.try_clone().unwrap()).unwrap());
    assert_eq!(psk.as_deref(), Some("correct horse"));
    assert_eq!(*surface.lock().unwrap(), ["Home"]);

    // Served as the connection is built, so the object is there before anybody
    // can call it — which is what `registered` gives the network manager by
    // registering only after the object is served.
    let declined = Arc::new(Mutex::new(Vec::new()));
    let declining = zbus::blocking::connection::Builder::address(bus.address().as_str())
        .unwrap()
        .serve_at(
            "/org/freedesktop/NetworkManager/SecretAgent",
            SecretAgent::answering(
                Box::new(Asking {
                    asked: declined.clone(),
                    typing: None,
                }),
                bus.our_user(),
            ),
        )
        .unwrap()
        .build()
        .unwrap();
    match get_secrets(&network_manager, &declining, "802-11-wireless-security", 1) {
        Err(zbus::Error::MethodError(name, _, _)) => assert_eq!(
            name.as_str(),
            "org.freedesktop.NetworkManager.SecretAgent.UserCanceled"
        ),
        other => panic!("a person who declined was answered {other:?}"),
    }
    assert_eq!(*declined.lock().unwrap(), ["Home"]);
}
