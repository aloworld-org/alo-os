//! What a drive is on this machine, as the rented disk service reports it.
//!
//! ADR 0011 rents the base's services, and the broker plan says so again:
//! *configured, never patched*. On the base alo OS is built on the disk service
//! is udisks2, and this crate is everything alo OS needs to know about what it
//! reports — read by the person's side, which lists drives and reads their
//! health, and by the privileged broker, which mounts and ejects a removable
//! drive against what the service reports at that moment.
//!
//! | | |
//! |---|---|
//! | [`Drive`], [`Filesystem`], [`Health`], [`TheDrives`] | What the disk service reported, and nothing it did not |
//! | [`Drives`], [`DriveService`] | Asking what there is now, and the two changes the broker makes |
//! | [`LoginName`] | Whom a drive is mounted for: the person's own login, never text a request carried |
//! | `udisks` | The client that speaks the disk service's own interface on the system bus (Linux only) |
//!
//! # A drive is compared by what was reported, never by what anybody typed
//!
//! A drive is named by the identifier the disk service keeps for it across
//! plugging in and out — its vendor, model and serial number, as the service
//! writes them — and a filesystem by that and the filesystem's own UUID. Those
//! are the bytes the broker's `alo_broker::Identity` is digested from on both
//! sides of its door ([`Drive::as_reported`], [`Drive::filesystem_as_reported`]).
//! A device node such as `/dev/sdb1` is never one: it is whichever drive was
//! plugged in second, and a request naming it would be a request naming a
//! device.
//!
//! # Two changes, and the rest is never asked for
//!
//! [`DriveService`] mounts a filesystem on a removable drive for the person and
//! ejects a removable drive. **Nothing here formats, repartitions, erases,
//! relabels, repairs or unlocks anything**, and a test reads the client for
//! every method it can call on the disk service and refuses any other. Health is
//! a read: it is in what [`Drives::now`] reports, which the disk service answers
//! to anybody on the system bus, so reading it needs no privilege and no
//! approval (ADR 0001 §5).
//!
//! # Plugging a drive in grants nobody anything
//!
//! This crate subscribes to nothing — it answers when it is asked, and nothing
//! it does is started by a drive appearing — and it depends on no crate that
//! could make a grant. A drive the broker mounts is mounted for the person, at
//! the place the disk service chooses for them, and what an agent may read on
//! it is granted the way every folder is, by the person, in a picker.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

#[cfg(target_os = "linux")]
mod bus;
mod login;
mod reported;
mod service;
#[cfg(target_os = "linux")]
pub mod udisks;

pub use login::LoginName;
pub use reported::{Drive, Filesystem, Health, Plugged, TheDrives};
pub use service::{DriveService, Drives, NotAnswering, NotDone};
