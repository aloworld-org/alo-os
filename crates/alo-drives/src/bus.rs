//! The disk service's names on the system bus, and the one way it is asked
//! anything.
//!
//! Every name here is udisks2's own published D-Bus interface
//! (`org.freedesktop.UDisks2`, stable since 2.0). They are written once so the
//! client and a test standing in for the service cannot spell one differently.
//!
//! **This file names every method the client may call**, in [`METHODS`], and
//! `tests/a_drive_is_only_ever_mounted_or_ejected.rs` holds the client to that
//! list and the list to five entries. A method that formats, repartitions or
//! erases is not spelt anywhere in this crate's source.

use zbus::blocking::Connection;
use zbus::zvariant::DynamicType;

/// The name the disk service owns on the system bus.
pub const THE_DISK_SERVICE: &str = "org.freedesktop.UDisks2";

/// The object that reports every other object.
pub const MANAGER_AT: &str = "/org/freedesktop/UDisks2";

/// The standard interface every object is reported through.
pub const OBJECT_MANAGER: &str = "org.freedesktop.DBus.ObjectManager";
/// A drive.
pub const DRIVE: &str = "org.freedesktop.UDisks2.Drive";
/// A drive that keeps an ATA self-assessment.
pub const ATA: &str = "org.freedesktop.UDisks2.Drive.Ata";
/// A drive that is an NVMe controller.
pub const NVME: &str = "org.freedesktop.UDisks2.NVMe.Controller";
/// A block device: a whole drive, or a partition of one.
pub const BLOCK: &str = "org.freedesktop.UDisks2.Block";
/// A block device holding a filesystem.
pub const FILESYSTEM: &str = "org.freedesktop.UDisks2.Filesystem";

/// Asking for every object and its properties, at once.
pub const GET_MANAGED_OBJECTS: &str = "GetManagedObjects";
/// Mounting a filesystem.
pub const MOUNT: &str = "Mount";
/// Unmounting a filesystem.
pub const UNMOUNT: &str = "Unmount";
/// Switching a drive off.
pub const POWER_OFF: &str = "PowerOff";
/// Ejecting a drive's medium.
pub const EJECT: &str = "Eject";

/// Every method the client calls, and there is no other.
pub const METHODS: [&str; 5] = [GET_MANAGED_OBJECTS, MOUNT, UNMOUNT, POWER_OFF, EJECT];

/// What went wrong asking, in English for a log.
pub fn failed(asking: &str, why: &dyn std::fmt::Display) -> String {
    format!("{asking}: {why}")
}

/// Call one of [`METHODS`] on the disk service, and read its answer as `T`.
pub fn call<T, B>(
    bus: &Connection,
    at: &str,
    interface: &str,
    method: &str,
    body: &B,
) -> Result<T, String>
where
    T: for<'d> zbus::export::serde::Deserialize<'d> + zbus::zvariant::Type,
    B: zbus::export::serde::Serialize + DynamicType,
{
    if !METHODS.contains(&method) {
        return Err(failed(method, &"not a method this client calls"));
    }
    bus.call_method(Some(THE_DISK_SERVICE), at, Some(interface), method, body)
        .map_err(|why| failed(method, &why))?
        .body()
        .deserialize::<T>()
        .map_err(|why| failed(method, &why))
}
