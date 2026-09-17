//! The client, speaking the disk service's own interface on a real bus.
//!
//! A private `dbus-daemon` this test starts, with no service activation, and on
//! it a stand-in disk service served with `zbus` under udisks2's own name,
//! objects and interfaces. It is **not** udisks2, and nothing here claims what
//! udisks2 itself does on a machine — the report for this task says what was not
//! measured. What it proves is that the client reads what it says it reads, and
//! asks for exactly the change it was told to and nothing else, over a real bus.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use alo_drives::udisks::UDisks;
use alo_drives::{Drive, DriveService, Drives, Filesystem, Health, LoginName, Plugged};
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
        let place = std::env::temp_dir().join(format!("alo-drives-{}-{what}", std::process::id()));
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

/// One object's properties, interface by interface.
type Interfaces = HashMap<String, HashMap<String, OwnedValue>>;

/// Options, as a method receives them.
type Options = HashMap<String, OwnedValue>;

/// Where the stand-in's objects are.
fn at(name: &str) -> String {
    format!("/org/freedesktop/UDisks2/{name}")
}

/// An object path.
fn object(name: &str) -> OwnedObjectPath {
    OwnedObjectPath::try_from(at(name)).unwrap()
}

/// A value.
fn value<'a>(of: impl Into<Value<'a>>) -> OwnedValue {
    OwnedValue::try_from(of.into()).unwrap()
}

/// Properties from pairs.
fn properties(pairs: Vec<(&str, OwnedValue)>) -> HashMap<String, OwnedValue> {
    pairs
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

/// A drive's interfaces.
fn a_drive(identifier: &str, removable: bool, bus: &str, power_off: bool) -> Interfaces {
    HashMap::from([(
        "org.freedesktop.UDisks2.Drive".to_owned(),
        properties(vec![
            ("Id", value(identifier)),
            ("Removable", value(removable)),
            ("MediaRemovable", value(false)),
            ("ConnectionBus", value(bus)),
            ("CanPowerOff", value(power_off)),
            ("Ejectable", value(false)),
        ]),
    )])
}

/// A block device's interfaces, with a filesystem on it when `mounted` says
/// whether it is mounted.
fn a_block(drive: &str, uuid: &str, system: bool, mounted: Option<bool>) -> Interfaces {
    let mut interfaces = HashMap::from([(
        "org.freedesktop.UDisks2.Block".to_owned(),
        properties(vec![
            ("Drive", value(object(drive))),
            ("IdUUID", value(uuid)),
            ("HintSystem", value(system)),
        ]),
    )]);
    if let Some(mounted) = mounted {
        let points: Vec<Vec<u8>> = if mounted {
            vec![b"/run/media/alo/STICK\0".to_vec()]
        } else {
            Vec::new()
        };
        interfaces.insert(
            "org.freedesktop.UDisks2.Filesystem".to_owned(),
            properties(vec![("MountPoints", value(points))]),
        );
    }
    interfaces
}

/// Everything the stand-in reports.
fn the_machine() -> HashMap<OwnedObjectPath, Interfaces> {
    let mut stick = a_drive("Kingston-DataTraveler-1C1B", true, "usb", true);
    stick.insert(
        "org.freedesktop.UDisks2.Drive.Ata".to_owned(),
        properties(vec![
            ("SmartSupported", value(true)),
            ("SmartEnabled", value(false)),
            ("SmartUpdated", value(0_u64)),
            ("SmartFailing", value(false)),
        ]),
    );
    let mut ssd = a_drive("Samsung-SSD-970-S4EW", false, "", false);
    ssd.insert(
        "org.freedesktop.UDisks2.NVMe.Controller".to_owned(),
        properties(vec![
            ("SmartUpdated", value(1_760_000_000_u64)),
            ("SmartCritical", value(0_u32)),
        ]),
    );
    let mut old = a_drive("WDC-WD10-OLD", false, "sata", false);
    old.insert(
        "org.freedesktop.UDisks2.Drive.Ata".to_owned(),
        properties(vec![
            ("SmartSupported", value(true)),
            ("SmartEnabled", value(true)),
            ("SmartUpdated", value(1_760_000_000_u64)),
            ("SmartFailing", value(true)),
        ]),
    );
    HashMap::from([
        (object("drives/stick"), stick),
        (
            object("block_devices/sdb"),
            a_block("drives/stick", "", false, None),
        ),
        (
            object("block_devices/sdb1"),
            a_block("drives/stick", "1234-ABCD", false, Some(false)),
        ),
        (
            object("block_devices/sdb2"),
            a_block("drives/stick", "5678-EF01", false, Some(true)),
        ),
        (object("drives/ssd"), ssd),
        (
            object("block_devices/nvme0n1p2"),
            a_block("drives/ssd", "root-uuid", true, Some(true)),
        ),
        (object("drives/old"), old),
        (
            object("drives/booted_from_usb"),
            a_drive("Seagate-Expansion-NA8", false, "usb", true),
        ),
        (
            object("block_devices/sdc1"),
            a_block("drives/booted_from_usb", "sys-uuid", true, Some(true)),
        ),
        (object("drives/nameless"), a_drive("", true, "usb", true)),
    ])
}

/// The object manager.
struct Manager;

#[zbus::interface(name = "org.freedesktop.DBus.ObjectManager")]
impl Manager {
    /// Everything, at once.
    fn get_managed_objects(&self) -> HashMap<OwnedObjectPath, Interfaces> {
        the_machine()
    }
}

/// A filesystem, which writes down what it is asked.
struct AFilesystem {
    /// Its name, for what is written down.
    name: &'static str,
    /// What was asked.
    asked: Asked,
    /// Whether it refuses to unmount, as a busy one does.
    busy: bool,
}

/// Options, written down in a stable order.
fn written(options: &Options) -> String {
    let mut pairs: Vec<String> = options
        .iter()
        .map(|(name, value)| match &**value {
            Value::Str(text) => format!("{name}={}", text.as_str()),
            other => format!("{name}={other:?}"),
        })
        .collect();
    pairs.sort();
    format!("[{}]", pairs.join(","))
}

#[zbus::interface(name = "org.freedesktop.UDisks2.Filesystem")]
impl AFilesystem {
    /// Mount it.
    fn mount(&self, options: Options) -> String {
        self.asked
            .lock()
            .unwrap()
            .push(format!("mount {} {}", self.name, written(&options)));
        "/run/media/alo/STICK".to_owned()
    }

    /// Unmount it.
    fn unmount(&self, options: Options) -> zbus::fdo::Result<()> {
        self.asked
            .lock()
            .unwrap()
            .push(format!("unmount {} {}", self.name, written(&options)));
        if self.busy {
            return Err(zbus::fdo::Error::Failed("target is busy".to_owned()));
        }
        Ok(())
    }
}

/// A drive, which writes down what it is asked.
struct ADrive {
    /// Its name.
    name: &'static str,
    /// What was asked.
    asked: Asked,
}

#[zbus::interface(name = "org.freedesktop.UDisks2.Drive")]
impl ADrive {
    /// Switch it off.
    fn power_off(&self, options: Options) {
        self.asked
            .lock()
            .unwrap()
            .push(format!("power-off {} {}", self.name, written(&options)));
    }

    /// Eject its medium.
    fn eject(&self, options: Options) {
        self.asked
            .lock()
            .unwrap()
            .push(format!("eject {} {}", self.name, written(&options)));
    }
}

/// The stand-in disk service on `bus`, whose mounted filesystem is `busy` or
/// not.
fn a_disk_service(bus: &ABus, asked: &Asked, busy: bool) -> Connection {
    let filesystem = |name, busy| AFilesystem {
        name,
        asked: asked.clone(),
        busy,
    };
    let drive = |name| ADrive {
        name,
        asked: asked.clone(),
    };
    zbus::blocking::connection::Builder::address(bus.address().as_str())
        .unwrap()
        .name("org.freedesktop.UDisks2")
        .unwrap()
        .serve_at("/org/freedesktop/UDisks2", Manager)
        .unwrap()
        .serve_at(at("block_devices/sdb1"), filesystem("sdb1", false))
        .unwrap()
        .serve_at(at("block_devices/sdb2"), filesystem("sdb2", busy))
        .unwrap()
        .serve_at(
            at("block_devices/nvme0n1p2"),
            filesystem("nvme0n1p2", false),
        )
        .unwrap()
        .serve_at(at("drives/stick"), drive("stick"))
        .unwrap()
        .serve_at(at("drives/ssd"), drive("ssd"))
        .unwrap()
        .build()
        .unwrap()
}

/// The drive with this identifier, as the client reported it.
fn called<'a>(drives: &'a [Drive], identifier: &str) -> &'a Drive {
    drives
        .iter()
        .find(|drive| drive.identifier() == identifier)
        .unwrap_or_else(|| panic!("{identifier} was not reported"))
}

/// **What the disk service reports is what the client says**: every drive with
/// an identifier, whether a person plugged it in, the filesystems with a UUID on
/// it and whether each is mounted, and each drive's own assessment of its health
/// — never good unless the drive said so.
#[test]
fn the_disk_service_reports_each_drive_its_filesystems_and_its_health() {
    let bus = ABus::started("reports");
    let _service = a_disk_service(&bus, &Asked::default(), false);
    let drives = UDisks::at(&bus.address()).unwrap().now().unwrap().drives;

    let identifiers: Vec<&str> = drives.iter().map(Drive::identifier).collect();
    assert_eq!(
        identifiers,
        [
            "Seagate-Expansion-NA8",
            "WDC-WD10-OLD",
            "Samsung-SSD-970-S4EW",
            "Kingston-DataTraveler-1C1B",
        ]
    );

    let stick = called(&drives, "Kingston-DataTraveler-1C1B");
    assert_eq!(stick.plugged(), Plugged::Removable);
    assert_eq!(stick.health(), Health::NotKnown);
    let filesystems: Vec<(&str, bool)> = stick
        .filesystems()
        .iter()
        .map(|filesystem| (filesystem.uuid(), filesystem.is_mounted()))
        .collect();
    assert_eq!(filesystems, [("1234-ABCD", false), ("5678-EF01", true)]);

    let ssd = called(&drives, "Samsung-SSD-970-S4EW");
    assert_eq!(ssd.plugged(), Plugged::BuiltIn);
    assert_eq!(ssd.health(), Health::Good);
    assert_eq!(called(&drives, "WDC-WD10-OLD").health(), Health::Failing);

    // Attached over USB, and holding the system: part of the machine.
    let booted = called(&drives, "Seagate-Expansion-NA8");
    assert_eq!(booted.plugged(), Plugged::BuiltIn);
    assert_eq!(booted.health(), Health::NotKnown);
}

/// **A filesystem is mounted for the person and with nothing else**, and
/// ejecting a drive unmounts what is mounted on it, without force, before it is
/// switched off.
#[test]
fn each_change_is_made_against_exactly_what_the_disk_service_reported() {
    let bus = ABus::started("changes");
    let asked = Asked::default();
    let _service = a_disk_service(&bus, &asked, false);
    let client = UDisks::at(&bus.address()).unwrap();
    let drives = client.now().unwrap().drives;
    let stick = called(&drives, "Kingston-DataTraveler-1C1B");
    let unmounted = stick.filesystems().first().unwrap();

    client
        .mount(stick, unmounted, &LoginName::named("alo").unwrap())
        .unwrap();
    client.eject(stick).unwrap();

    assert_eq!(
        *asked.lock().unwrap(),
        [
            "mount sdb1 [as-user=alo]",
            "unmount sdb2 []",
            "power-off stick []",
        ]
    );
}

/// **Nothing is changed that the disk service did not report, that is part of
/// the machine, or that is already mounted** — and the service is not asked.
#[test]
fn nothing_is_changed_that_was_not_reported_or_is_part_of_the_machine() {
    let bus = ABus::started("refused");
    let asked = Asked::default();
    let _service = a_disk_service(&bus, &asked, false);
    let client = UDisks::at(&bus.address()).unwrap();
    let drives = client.now().unwrap().drives;
    let person = LoginName::named("alo").unwrap();

    let invented = Drive::reported(
        "Kingston-DataTraveler-1C1B",
        Plugged::Removable,
        Health::Good,
    )
    .unwrap()
    .with(Filesystem::reported("1234-ABCD", false).unwrap());
    assert!(
        client
            .mount(&invented, invented.filesystems().first().unwrap(), &person)
            .is_err()
    );
    assert!(client.eject(&invented).is_err());

    let ssd = called(&drives, "Samsung-SSD-970-S4EW");
    assert!(
        client
            .mount(ssd, ssd.filesystems().first().unwrap(), &person)
            .is_err()
    );
    assert!(client.eject(ssd).is_err());

    let stick = called(&drives, "Kingston-DataTraveler-1C1B");
    let mounted = stick.filesystems().get(1).unwrap();
    assert!(mounted.is_mounted());
    assert!(client.mount(stick, mounted, &person).is_err());

    assert!(
        asked.lock().unwrap().is_empty(),
        "{:?}",
        asked.lock().unwrap()
    );
}

/// **A filesystem that will not unmount leaves the drive switched on**: it is
/// never unmounted by force, and never switched off under what is still using
/// it.
#[test]
fn a_filesystem_that_will_not_unmount_leaves_the_drive_on() {
    let bus = ABus::started("busy");
    let asked = Asked::default();
    let _service = a_disk_service(&bus, &asked, true);
    let client = UDisks::at(&bus.address()).unwrap();
    let drives = client.now().unwrap().drives;
    let stick = called(&drives, "Kingston-DataTraveler-1C1B");

    assert!(client.eject(stick).is_err());
    assert_eq!(*asked.lock().unwrap(), ["unmount sdb2 []"]);
}
