//! The storage verbs, carried out: a removable drive mounted for the signed-in
//! person, or ejected — and nothing else, ever.
//!
//! By the time a verb reaches here the door has decided it is exactly one a
//! person approved, once, and written that down. What is left is to do it to the
//! right drive and to no other — and the broker was never told which drive in
//! any form it could act on. It was told thirty-two bytes. So each verb asks the
//! disk service what it has **now**, digests what it reported for each drive or
//! filesystem the way the side that asked digested it, and acts on the one that
//! matches:
//!
//! | verb | matched against | acts with |
//! |---|---|---|
//! | `storage.mount` | every filesystem on every drive, by the drive's identifier and the filesystem's UUID | `DriveService::mount`, for the person |
//! | `storage.eject` | every drive, by its identifier | `DriveService::eject` |
//!
//! **No match is no change**, and neither are two. **A drive that is part of the
//! machine is never mounted or ejected here**, whatever identity names it: the
//! system drive, and anything the disk service does not say a person plugs in.
//! **A filesystem already mounted is not mounted again**, because one approval is
//! one execution.
//!
//! # For the signed-in person, and nobody else
//!
//! A filesystem is mounted *as* the person the door is for — the user
//! `alo-agentd` runs as, from the machine description — by the login name this
//! machine's own account file gives that user. Not the agent's login, not root's,
//! and never a name a request carried, because a request carries none. The disk
//! service puts it where that person's session finds their drives.
//!
//! # And grants nobody anything
//!
//! Mounting makes no grant, and nothing here could make one: this crate depends
//! on no crate that holds grants. An agent reads what is on a drive the way it
//! reads any folder — when the person grants it, in a picker (`alo-picking`).
//!
//! # Health is not here
//!
//! Checking a drive's health is a read. The disk service answers it to anybody on
//! the system bus (`alo_drives::Drives::now`), so it needs neither the broker nor
//! an approval, and a read that waited for one would be a read made into a change
//! (ADR 0001 §5).

use std::path::{Path, PathBuf};

use alo_broker::{Identity, NotCarried};
use alo_drives::{Drive, DriveService, Filesystem, LoginName, Plugged};

/// Where this machine's accounts are written.
pub const THE_ACCOUNTS: &str = "/etc/passwd";

/// What carries the storage verbs out on this machine.
#[derive(Debug)]
pub struct Storage<D> {
    /// The disk service.
    service: D,
    /// This machine's account file.
    accounts: PathBuf,
    /// The person the door is for, whom a drive is mounted for.
    person: u32,
}

impl<D: DriveService> Storage<D> {
    /// Carry the storage verbs out against this disk service, mounting for
    /// `person` by the name `accounts` gives them.
    #[must_use]
    pub fn against(service: D, accounts: &Path, person: u32) -> Self {
        Self {
            service,
            accounts: accounts.to_owned(),
            person,
        }
    }

    /// The disk service, for a test to look at what it was asked.
    #[must_use]
    pub const fn service(&self) -> &D {
        &self.service
    }

    /// `storage.mount`: mount the one filesystem reported under this identity,
    /// on a removable drive, for the person.
    ///
    /// # Errors
    /// [`NotCarried`], and nothing was mounted.
    pub fn mount(&self, identity: Identity) -> Result<(), NotCarried> {
        let now = self
            .service
            .now()
            .map_err(|why| NotCarried(why.to_string()))?;
        let candidates: Vec<(&Drive, &Filesystem)> = now
            .drives
            .iter()
            .flat_map(|drive| drive.filesystems().iter().map(move |fs| (drive, fs)))
            .filter(|(drive, fs)| {
                Identity::of_what_was_reported(&drive.filesystem_as_reported(fs)) == identity
            })
            .collect();
        let (drive, filesystem) = exactly_one(&candidates, "filesystem")?;
        plugged_in(drive)?;
        if filesystem.is_mounted() {
            return Err(NotCarried(
                "the filesystem approved is already mounted, so nothing was changed".to_owned(),
            ));
        }
        let person = self.persons_login()?;
        self.service
            .mount(drive, filesystem, &person)
            .map_err(|why| NotCarried(why.to_string()))
    }

    /// `storage.eject`: unmount and eject the one removable drive reported under
    /// this identity.
    ///
    /// # Errors
    /// [`NotCarried`], and the drive is as the disk service left it — if a
    /// filesystem would not unmount, still switched on.
    pub fn eject(&self, identity: Identity) -> Result<(), NotCarried> {
        let now = self
            .service
            .now()
            .map_err(|why| NotCarried(why.to_string()))?;
        let candidates: Vec<&Drive> = now
            .drives
            .iter()
            .filter(|drive| Identity::of_what_was_reported(&drive.as_reported()) == identity)
            .collect();
        let drive = exactly_one(&candidates, "drive")?;
        plugged_in(drive)?;
        self.service
            .eject(drive)
            .map_err(|why| NotCarried(why.to_string()))
    }

    /// The login name this machine's accounts give the person.
    fn persons_login(&self) -> Result<LoginName, NotCarried> {
        let accounts = std::fs::read_to_string(&self.accounts).map_err(|why| {
            NotCarried(format!(
                "{} could not be read, so whom to mount the drive for is not known: {why}",
                self.accounts.display()
            ))
        })?;
        LoginName::of_user(&accounts, self.person).ok_or_else(|| {
            NotCarried(format!(
                "{} does not name exactly one login for user {}, so nothing was mounted",
                self.accounts.display(),
                self.person
            ))
        })
    }
}

/// The one thing reported under the identity approved, or no change.
fn exactly_one<T: Copy>(candidates: &[T], what: &str) -> Result<T, NotCarried> {
    match candidates {
        [one] => Ok(*one),
        [] => Err(NotCarried(format!(
            "the disk service reports no {what} with the identity that was approved, so nothing \
             was changed"
        ))),
        _ => Err(NotCarried(format!(
            "the disk service reports more than one {what} with the identity that was approved, \
             so nothing was changed"
        ))),
    }
}

/// A drive a person plugged in, or no change.
fn plugged_in(drive: &Drive) -> Result<(), NotCarried> {
    match drive.plugged() {
        Plugged::Removable => Ok(()),
        Plugged::BuiltIn => Err(NotCarried(
            "the drive approved is part of the machine, and only a drive a person plugged in is \
             mounted or ejected through the broker, so nothing was changed"
                .to_owned(),
        )),
    }
}
