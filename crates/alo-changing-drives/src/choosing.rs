//! Which drive a person means, out of what the disk service reports now.
//!
//! Reading what is plugged in needs no approval and no privilege — the disk
//! service answers it to anybody, which is why checking a disk's health is not
//! a broker verb at all (`alo-broker`'s `verbs`). So a surface picks here,
//! before it proposes anything, and the refusals a person is most likely to
//! meet — *it was taken out*, *there are two of them*, *there is nothing on it*
//! — are said without the broker ever being asked.
//!
//! What crosses the door afterwards is the identity, never the name: the drive
//! digests it, and the same name on a different stick is a different identity.

use alo_broker::Identity;
use alo_drives::{Drive, Filesystem, Plugged, TheDrives};

use crate::refusing::NotChanged;

/// The one drive plugged into this machine with this name.
///
/// # Errors
/// [`NotChanged::NonePluggedInCalled`] when nothing reports that name now, and
/// [`NotChanged::MoreThanOneCalled`] when two do — never a guess between them.
pub fn the_drive_called<'a>(drives: &'a TheDrives, called: &str) -> Result<&'a Drive, NotChanged> {
    let mut matching = drives
        .drives
        .iter()
        .filter(|drive| drive.identifier() == called);
    let Some(drive) = matching.next() else {
        return Err(NotChanged::NonePluggedInCalled(called.to_owned()));
    };
    if matching.next().is_some() {
        return Err(NotChanged::MoreThanOneCalled(called.to_owned()));
    }
    Ok(drive)
}

/// What is on this drive that this machine can open.
///
/// # Errors
/// [`NotChanged::PartOfThisMachine`] for one of the machine's own disks, and
/// [`NotChanged::NothingToOpen`] when the drive carries nothing this machine
/// recognises.
pub fn what_can_be_opened(drive: &Drive) -> Result<&[Filesystem], NotChanged> {
    a_drive_a_person_plugged_in(drive)?;
    if drive.filesystems().is_empty() {
        return Err(NotChanged::NothingToOpen(drive.identifier().to_owned()));
    }
    Ok(drive.filesystems())
}

/// The identity to open this one thing on this drive under.
///
/// # Errors
/// [`NotChanged::PartOfThisMachine`], and [`NotChanged::AlreadyOpen`] when it
/// is open already — which is not a failure and says so.
pub fn to_open(drive: &Drive, filesystem: &Filesystem) -> Result<Identity, NotChanged> {
    a_drive_a_person_plugged_in(drive)?;
    if filesystem.is_mounted() {
        return Err(NotChanged::AlreadyOpen(drive.identifier().to_owned()));
    }
    Ok(Identity::of_what_was_reported(
        &drive.filesystem_as_reported(filesystem),
    ))
}

/// The identity to finish with this drive under.
///
/// # Errors
/// [`NotChanged::PartOfThisMachine`]. A drive with nothing open on it is still
/// finished with: switching it off is the part a person is waiting for.
pub fn to_finish_with(drive: &Drive) -> Result<Identity, NotChanged> {
    a_drive_a_person_plugged_in(drive)?;
    Ok(Identity::of_what_was_reported(&drive.as_reported()))
}

/// That this is a drive a person plugged in, rather than part of the machine.
fn a_drive_a_person_plugged_in(drive: &Drive) -> Result<(), NotChanged> {
    match drive.plugged() {
        Plugged::Removable => Ok(()),
        Plugged::BuiltIn => Err(NotChanged::PartOfThisMachine(drive.identifier().to_owned())),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_drives::Health;

    /// A stick with one thing on it, open or not.
    fn a_stick(identifier: &str, uuid: &str, open: bool) -> Drive {
        Drive::reported(identifier, Plugged::Removable, Health::NotKnown)
            .unwrap()
            .with(Filesystem::reported(uuid, open).unwrap())
    }

    /// Everything reported at one moment.
    fn reported(drives: Vec<Drive>) -> TheDrives {
        TheDrives { drives }
    }

    /// **The one drive with that name is found.**
    #[test]
    fn the_one_drive_with_that_name_is_found() {
        let drives = reported(vec![
            a_stick("Kingston-DataTraveler-1C1B", "1234-ABCD", false),
            a_stick("SanDisk-Ultra-77", "5678-EF01", false),
        ]);
        let drive = the_drive_called(&drives, "SanDisk-Ultra-77").unwrap();
        assert_eq!(drive.identifier(), "SanDisk-Ultra-77");
    }

    /// **A drive that has been taken out is not guessed at**, and neither is
    /// one of two with the same name.
    #[test]
    fn a_drive_that_is_gone_or_doubled_is_never_guessed_at() {
        let empty = reported(Vec::new());
        assert_eq!(
            the_drive_called(&empty, "Kingston-DataTraveler-1C1B"),
            Err(NotChanged::NonePluggedInCalled(
                "Kingston-DataTraveler-1C1B".to_owned()
            ))
        );

        let twice = reported(vec![
            a_stick("SanDisk-Ultra-77", "1234-ABCD", false),
            a_stick("SanDisk-Ultra-77", "5678-EF01", false),
        ]);
        assert_eq!(
            the_drive_called(&twice, "SanDisk-Ultra-77"),
            Err(NotChanged::MoreThanOneCalled("SanDisk-Ultra-77".to_owned()))
        );
    }

    /// **One of the machine's own disks is refused, both ways.** It is the
    /// refusal that keeps *eject the system disk* from being a thing anybody
    /// can propose.
    #[test]
    fn one_of_the_machines_own_disks_is_refused_both_ways() {
        let built_in = Drive::reported("Samsung-SSD-990", Plugged::BuiltIn, Health::Good)
            .unwrap()
            .with(Filesystem::reported("9999-0000", true).unwrap());
        let on_it = built_in.filesystems().first().unwrap();
        let why = NotChanged::PartOfThisMachine("Samsung-SSD-990".to_owned());
        assert_eq!(to_open(&built_in, on_it), Err(why.clone()));
        assert_eq!(to_finish_with(&built_in), Err(why.clone()));
        assert_eq!(what_can_be_opened(&built_in), Err(why));
    }

    /// **A drive with nothing this machine recognises on it says so**, rather
    /// than failing later with a refusal about an approval.
    #[test]
    fn a_drive_with_nothing_on_it_says_so() {
        let bare =
            Drive::reported("Generic-Reader-99", Plugged::Removable, Health::NotKnown).unwrap();
        assert_eq!(
            what_can_be_opened(&bare),
            Err(NotChanged::NothingToOpen("Generic-Reader-99".to_owned()))
        );
    }

    /// **One already open is not opened again**, and nothing has gone wrong.
    #[test]
    fn one_already_open_is_not_opened_again() {
        let open = a_stick("Kingston-DataTraveler-1C1B", "1234-ABCD", true);
        let on_it = open.filesystems().first().unwrap();
        assert_eq!(
            to_open(&open, on_it),
            Err(NotChanged::AlreadyOpen(
                "Kingston-DataTraveler-1C1B".to_owned()
            ))
        );
    }

    /// **What is opened and what is finished with are different identities**,
    /// and neither is the same card read in another drive.
    #[test]
    fn what_is_opened_and_what_is_finished_with_are_different_identities() {
        let stick = a_stick("Kingston-DataTraveler-1C1B", "1234-ABCD", false);
        let on_it = stick.filesystems().first().unwrap();
        let opening = to_open(&stick, on_it).unwrap();
        let finishing = to_finish_with(&stick).unwrap();
        assert_ne!(opening, finishing);

        let elsewhere = a_stick("Generic-Reader-99", "1234-ABCD", false);
        let same_card = elsewhere.filesystems().first().unwrap();
        assert_ne!(opening, to_open(&elsewhere, same_card).unwrap());
    }
}
