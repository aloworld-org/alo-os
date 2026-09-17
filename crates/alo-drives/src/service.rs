//! Asking the disk service what there is, and telling it the two changes the
//! broker makes.
//!
//! Two traits, because the two sides need different things. The person's side —
//! listing drives in Settings, reading a drive's health — only asks
//! ([`Drives`]). The broker also changes ([`DriveService`]), and only after its
//! door has decided a change is exactly one a person approved. A trait rather
//! than the client itself so that which drive is acted on is tested against
//! every shape of answer; `crate::udisks` is the one a machine runs.
//!
//! **There are two changes and no third.** Nothing on either trait formats,
//! repartitions, erases, relabels, repairs or unlocks a drive, so no carrier the
//! broker holds can be written to do one.
//!
//! The failures carry English for a service log and a record's reason, never
//! for a person: what a person reads is said by the surface that asked, from
//! the broker's one-word answer.

use crate::login::LoginName;
use crate::reported::{Drive, Filesystem, TheDrives};

/// The disk service could not be asked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAnswering(pub String);

impl std::fmt::Display for NotAnswering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the disk service could not be asked: {}", self.0)
    }
}

impl std::error::Error for NotAnswering {}

/// The disk service did not make a change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotDone(pub String);

impl std::fmt::Display for NotDone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the disk service did not make the change: {}", self.0)
    }
}

impl std::error::Error for NotDone {}

/// What there is now.
pub trait Drives {
    /// Everything the disk service reports at this moment, health included.
    ///
    /// # Errors
    /// [`NotAnswering`].
    fn now(&self) -> Result<TheDrives, NotAnswering>;
}

/// What there is now, and the two changes the broker makes.
pub trait DriveService: Drives {
    /// Mount this filesystem on this drive for the person whose login this is,
    /// where the disk service puts that person's drives, with no options of
    /// ours but whom it is for.
    ///
    /// # Errors
    /// [`NotDone`].
    fn mount(
        &self,
        drive: &Drive,
        filesystem: &Filesystem,
        for_the_person: &LoginName,
    ) -> Result<(), NotDone>;

    /// Unmount every filesystem on this drive, never by force, and then switch
    /// it off or eject its medium — or, when any filesystem will not unmount,
    /// stop there and say so.
    ///
    /// # Errors
    /// [`NotDone`].
    fn eject(&self, drive: &Drive) -> Result<(), NotDone>;
}
