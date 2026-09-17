//! The rented disk service, asked and told over the system bus.
//!
//! Configured, never patched (ADR 0011): this speaks udisks2's published D-Bus
//! interface and nothing else. It asks for the whole picture at once, in one
//! call ([`Drives::now`]), so a drive and its filesystems are always read from
//! the same moment; and it makes the two changes the broker carries out, each
//! against an object the disk service itself reported a moment before.
//!
//! # The system bus is named, never looked up in an environment
//!
//! [`THE_SYSTEM_BUS`] is the address systemd gives the system bus. A process
//! that read `DBUS_SYSTEM_BUS_ADDRESS` instead would be a privileged process
//! whose environment chose which disk service it obeyed.
//!
//! # What is read, and how
//!
//! - **A drive** is an object with the drive interface and a non-empty `Id`.
//!   It is [`Plugged::Removable`] when the disk service says it or its medium is
//!   removable, or that it is attached over USB or an SD slot — **and** none of
//!   its block devices carries the service's hint that it belongs to the
//!   system. Anything else is [`Plugged::BuiltIn`].
//! - **Its health** is the drive's own self-assessment: an ATA drive's
//!   `SmartFailing`, when it keeps one, has it switched on and has been read;
//!   an NVMe drive's critical-warning bits, when they have been read. Anything
//!   else is [`Health::NotKnown`], never good.
//! - **A filesystem** is a block device with the filesystem interface, on a
//!   drive, with a UUID; it is mounted when the service lists any mount point.
//!
//! # What a change is made with
//!
//! - **mount** — `Mount` on the filesystem's object, with one option:
//!   `as-user`, the person's login name, so the disk service mounts it where
//!   that person's session finds their drives and owned the way their drives
//!   are. No filesystem type, no mount options, no place.
//! - **eject** — `Unmount` with no options (so never by force) on every mounted
//!   filesystem of the drive, then `PowerOff` when the drive can be switched
//!   off, or `Eject` when its medium can be ejected. A filesystem that will not
//!   unmount stops it there.

use std::collections::HashMap;
use std::time::Duration;

use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

use crate::bus::{
    ATA, BLOCK, DRIVE, EJECT, FILESYSTEM, GET_MANAGED_OBJECTS, MANAGER_AT, MOUNT, NVME,
    OBJECT_MANAGER, POWER_OFF, UNMOUNT, call,
};
use crate::login::LoginName;
use crate::reported::{Drive, Filesystem, Health, Plugged, TheDrives};
use crate::service::{DriveService, Drives, NotAnswering, NotDone};

/// The system bus, as systemd makes it.
pub const THE_SYSTEM_BUS: &str = "unix:path=/run/dbus/system_bus_socket";

/// How long one question or change may take. Unmounting a slow stick flushes
/// what was written to it first, which takes a while; a disk service that stops
/// answering is still a change not made, never a broker that waits for ever.
const ANSWERING: Duration = Duration::from_secs(90);

/// How the disk service names the buses a person plugs a drive in by.
const PLUGGED_IN_BY: [&str; 2] = ["usb", "sdio"];

/// One interface's properties.
type Properties = HashMap<String, OwnedValue>;

/// One object's properties, interface by interface.
type Interfaces = HashMap<String, Properties>;

/// Every object the disk service reports.
type Objects = HashMap<OwnedObjectPath, Interfaces>;

/// Options passed to a method: text keys, any values.
type Options<'a> = HashMap<&'a str, Value<'a>>;

/// The disk service on one bus.
#[derive(Debug)]
pub struct UDisks {
    /// The bus.
    bus: Connection,
}

impl UDisks {
    /// The disk service on this machine's system bus.
    ///
    /// # Errors
    /// [`NotAnswering`] when the system bus cannot be reached.
    pub fn on_this_machine() -> Result<Self, NotAnswering> {
        Self::at(THE_SYSTEM_BUS)
    }

    /// The disk service on the bus at this address — how a test reaches one of
    /// its own.
    ///
    /// # Errors
    /// [`NotAnswering`] when that bus cannot be reached.
    pub fn at(address: &str) -> Result<Self, NotAnswering> {
        let bus = zbus::blocking::connection::Builder::address(address)
            .map(|building| building.method_timeout(ANSWERING))
            .and_then(zbus::blocking::connection::Builder::build)
            .map_err(|why| NotAnswering(why.to_string()))?;
        Ok(Self { bus })
    }
}

impl Drives for UDisks {
    fn now(&self) -> Result<TheDrives, NotAnswering> {
        let objects: Objects = call(
            &self.bus,
            MANAGER_AT,
            OBJECT_MANAGER,
            GET_MANAGED_OBJECTS,
            &(),
        )
        .map_err(NotAnswering)?;
        Ok(read(&objects))
    }
}

impl DriveService for UDisks {
    fn mount(
        &self,
        drive: &Drive,
        filesystem: &Filesystem,
        for_the_person: &LoginName,
    ) -> Result<(), NotDone> {
        let (Some(_), Some(at)) = (&drive.at, &filesystem.at) else {
            return Err(NotDone(
                "that filesystem was not reported by this disk service".to_owned(),
            ));
        };
        if drive.plugged() != Plugged::Removable {
            return Err(NotDone(
                "a drive that is part of the machine is not mounted through the broker".to_owned(),
            ));
        }
        if filesystem.is_mounted() {
            return Err(NotDone("that filesystem is already mounted".to_owned()));
        }
        let options: Options<'_> =
            HashMap::from([("as-user", Value::from(for_the_person.as_str()))]);
        call::<String, _>(&self.bus, at, FILESYSTEM, MOUNT, &options)
            .map(drop)
            .map_err(NotDone)
    }

    fn eject(&self, drive: &Drive) -> Result<(), NotDone> {
        let Some(at) = &drive.at else {
            return Err(NotDone(
                "that drive was not reported by this disk service".to_owned(),
            ));
        };
        if drive.plugged() != Plugged::Removable {
            return Err(NotDone(
                "a drive that is part of the machine is not ejected through the broker".to_owned(),
            ));
        }
        let nothing: Options<'_> = HashMap::new();
        for filesystem in drive.filesystems().iter().filter(|f| f.is_mounted()) {
            let Some(mounted_at) = &filesystem.at else {
                return Err(NotDone(
                    "a filesystem on that drive was not reported by this disk service".to_owned(),
                ));
            };
            call::<(), _>(&self.bus, mounted_at, FILESYSTEM, UNMOUNT, &nothing).map_err(|why| {
                NotDone(format!(
                    "a filesystem on the drive would not unmount, so the drive was left as it \
                     is: {why}"
                ))
            })?;
        }
        if drive.can_power_off {
            call::<(), _>(&self.bus, at, DRIVE, POWER_OFF, &nothing).map_err(NotDone)
        } else if drive.ejectable {
            call::<(), _>(&self.bus, at, DRIVE, EJECT, &nothing).map_err(NotDone)
        } else {
            Ok(())
        }
    }
}

/// The disk service on this machine's system bus, reached afresh for every
/// question and every change.
///
/// What the broker holds, rather than a [`UDisks`] made once: the disk service
/// is started when it is first asked for, and a connection made at start-up to
/// a bus that was restarted since would answer nothing for the rest of the
/// broker's life.
#[derive(Debug, Clone, Copy, Default)]
pub struct OnThisMachine;

impl Drives for OnThisMachine {
    fn now(&self) -> Result<TheDrives, NotAnswering> {
        UDisks::on_this_machine()?.now()
    }
}

impl DriveService for OnThisMachine {
    fn mount(
        &self,
        drive: &Drive,
        filesystem: &Filesystem,
        for_the_person: &LoginName,
    ) -> Result<(), NotDone> {
        reached()?.mount(drive, filesystem, for_the_person)
    }

    fn eject(&self, drive: &Drive) -> Result<(), NotDone> {
        reached()?.eject(drive)
    }
}

/// The disk service on this machine, for a change.
fn reached() -> Result<UDisks, NotDone> {
    UDisks::on_this_machine().map_err(|why| NotDone(why.to_string()))
}

/// Every drive with an identifier, in the order of the objects' paths, with the
/// filesystems on it.
fn read(objects: &Objects) -> TheDrives {
    let mut paths: Vec<&OwnedObjectPath> = objects.keys().collect();
    paths.sort_by(|a, b| a.as_str().cmp(b.as_str()));

    let mut drives = Vec::new();
    for path in &paths {
        let Some(interfaces) = objects.get(*path) else {
            continue;
        };
        let Some(drive) = interfaces.get(DRIVE) else {
            continue;
        };
        let blocks: Vec<(&OwnedObjectPath, &Properties, &Interfaces)> = paths
            .iter()
            .filter_map(|other| Some((*other, objects.get(*other)?)))
            .filter_map(|(other, its)| Some((other, its.get(BLOCK)?, its)))
            .filter(|(_, block, _)| object(block, "Drive").is_some_and(|on| on == path.as_str()))
            .collect();
        let removable = flag(drive, "Removable")
            || flag(drive, "MediaRemovable")
            || text(drive, "ConnectionBus").is_some_and(|bus| PLUGGED_IN_BY.contains(&bus));
        let the_systems = blocks.iter().any(|(_, block, _)| flag(block, "HintSystem"));
        let plugged = if removable && !the_systems {
            Plugged::Removable
        } else {
            Plugged::BuiltIn
        };
        let Some(mut one) = text(drive, "Id")
            .and_then(|identifier| Drive::reported(identifier, plugged, health(interfaces)))
        else {
            continue;
        };
        one.can_power_off = flag(drive, "CanPowerOff");
        one.ejectable = flag(drive, "Ejectable");
        one.at = Some(path.as_str().to_owned());
        for (block_at, block, its) in &blocks {
            let Some(mounts) = its.get(FILESYSTEM) else {
                continue;
            };
            let Some(mut filesystem) = text(block, "IdUUID")
                .and_then(|uuid| Filesystem::reported(uuid, listed(mounts, "MountPoints")))
            else {
                continue;
            };
            filesystem.at = Some(block_at.as_str().to_owned());
            one = one.with(filesystem);
        }
        drives.push(one);
    }
    TheDrives { drives }
}

/// A drive's own self-assessment, when it keeps one that has been read.
fn health(interfaces: &Interfaces) -> Health {
    if let Some(ata) = interfaces.get(ATA)
        && flag(ata, "SmartSupported")
        && flag(ata, "SmartEnabled")
        && number(ata, "SmartUpdated") > 0
    {
        return if flag(ata, "SmartFailing") {
            Health::Failing
        } else {
            Health::Good
        };
    }
    if let Some(nvme) = interfaces.get(NVME)
        && number(nvme, "SmartUpdated") > 0
    {
        return if warnings(nvme, "SmartCritical") != 0 {
            Health::Failing
        } else {
            Health::Good
        };
    }
    Health::NotKnown
}

/// A boolean property, false when it is missing or not a boolean.
fn flag(properties: &Properties, name: &str) -> bool {
    matches!(
        properties.get(name).map(|value| &**value),
        Some(Value::Bool(true))
    )
}

/// A text property, when it is text and not empty.
fn text<'a>(properties: &'a Properties, name: &str) -> Option<&'a str> {
    match properties.get(name).map(|value| &**value) {
        Some(Value::Str(text)) if !text.is_empty() => Some(text.as_str()),
        _ => None,
    }
}

/// An object-path property, when it names an object.
fn object<'a>(properties: &'a Properties, name: &str) -> Option<&'a str> {
    match properties.get(name).map(|value| &**value) {
        Some(Value::ObjectPath(path)) if path.as_str() != "/" => Some(path.as_str()),
        _ => None,
    }
}

/// An unsigned 64-bit property, zero when it is missing.
fn number(properties: &Properties, name: &str) -> u64 {
    match properties.get(name).map(|value| &**value) {
        Some(Value::U64(number)) => *number,
        _ => 0,
    }
}

/// An unsigned 32-bit bit set, zero when it is missing.
fn warnings(properties: &Properties, name: &str) -> u32 {
    match properties.get(name).map(|value| &**value) {
        Some(Value::U32(bits)) => *bits,
        _ => 0,
    }
}

/// Whether an array property lists anything.
fn listed(properties: &Properties, name: &str) -> bool {
    matches!(properties.get(name).map(|value| &**value), Some(Value::Array(array)) if !array.is_empty())
}
