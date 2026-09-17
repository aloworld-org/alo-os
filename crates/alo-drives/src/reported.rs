//! What the disk service reported: the drives on this machine, how each is
//! attached, how healthy it says each is, and the filesystems on each.
//!
//! Every value here is made from a report and from nothing else, and each is
//! compared by the bytes it was reported under ([`Drive::as_reported`],
//! [`Drive::filesystem_as_reported`]). An identifier is matched against the one
//! a person approved; it is never handed back to the disk service as an
//! instruction, and neither is anything else here — the client acts on the
//! object the service itself reported, which a test's drive does not have.

/// The most bytes a drive's identifier may be. The disk service writes vendor,
/// model and serial number joined by dashes, which is well under this.
const LONGEST_IDENTIFIER: usize = 256;

/// The most bytes a filesystem's UUID may be. The longest any filesystem writes
/// is thirty-six characters.
const LONGEST_UUID: usize = 128;

/// What every drive's identity begins with, so it can never be mistaken for a
/// filesystem's or for anything else digested on this machine.
const A_DRIVE: &[u8] = b"alo-drives drive 1\0";

/// What every filesystem's identity begins with.
const A_FILESYSTEM: &[u8] = b"alo-drives filesystem 1\0";

/// How a drive is attached, as far as a person plugging it in is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plugged {
    /// A drive a person plugs in and takes away: a USB stick, a card, a disk in
    /// a USB enclosure. The only kind the broker mounts or ejects.
    Removable,
    /// Part of the machine, or holding the system itself.
    BuiltIn,
}

/// How healthy the disk service says a drive is.
///
/// Three answers and no score. A drive's own self-assessment is either passing
/// or failing, and a number between the two would be a judgement this machine
/// made rather than one the drive reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Health {
    /// The drive has assessed itself and reports nothing wrong.
    Good,
    /// The drive reports that it is failing, or has raised a critical warning.
    Failing,
    /// The drive keeps no self-assessment, keeps it switched off, or has not
    /// been read yet — a USB stick, most often. Never read as good.
    NotKnown,
}

/// A filesystem on a drive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filesystem {
    /// The filesystem's own UUID, as it was written when it was made.
    uuid: String,
    /// Whether it is mounted anywhere now.
    mounted: bool,
    /// Where the disk service reported it, when it did.
    pub(crate) at: Option<String>,
}

impl Filesystem {
    /// A filesystem as the disk service reported it, or nothing when it has no
    /// UUID it could be named by again.
    #[must_use]
    pub fn reported(uuid: &str, mounted: bool) -> Option<Self> {
        a_name(uuid, LONGEST_UUID).then(|| Self {
            uuid: uuid.to_owned(),
            mounted,
            at: None,
        })
    }

    /// Its UUID.
    #[must_use]
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// Whether it is mounted anywhere now.
    #[must_use]
    pub const fn is_mounted(&self) -> bool {
        self.mounted
    }
}

/// A drive on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drive {
    /// The identifier the disk service keeps for it across plugging in and out.
    identifier: String,
    /// How it is attached.
    plugged: Plugged,
    /// How healthy it says it is.
    health: Health,
    /// The filesystems on it that have a UUID.
    filesystems: Vec<Filesystem>,
    /// Whether the disk service can switch it off once it is unmounted.
    pub(crate) can_power_off: bool,
    /// Whether its medium can be ejected.
    pub(crate) ejectable: bool,
    /// Where the disk service reported it, when it did.
    pub(crate) at: Option<String>,
}

impl Drive {
    /// A drive as the disk service reported it, or nothing when it reported no
    /// identifier the drive could be named by again.
    #[must_use]
    pub fn reported(identifier: &str, plugged: Plugged, health: Health) -> Option<Self> {
        a_name(identifier, LONGEST_IDENTIFIER).then(|| Self {
            identifier: identifier.to_owned(),
            plugged,
            health,
            filesystems: Vec::new(),
            can_power_off: false,
            ejectable: false,
            at: None,
        })
    }

    /// The same drive, with one more filesystem on it.
    #[must_use]
    pub fn with(mut self, filesystem: Filesystem) -> Self {
        self.filesystems.push(filesystem);
        self
    }

    /// Its identifier.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// How it is attached.
    #[must_use]
    pub const fn plugged(&self) -> Plugged {
        self.plugged
    }

    /// How healthy it says it is.
    #[must_use]
    pub const fn health(&self) -> Health {
        self.health
    }

    /// The filesystems on it.
    #[must_use]
    pub fn filesystems(&self) -> &[Filesystem] {
        &self.filesystems
    }

    /// What this drive's identity is digested from: its identifier, after a
    /// prefix no filesystem's identity has.
    #[must_use]
    pub fn as_reported(&self) -> Vec<u8> {
        [A_DRIVE, self.identifier.as_bytes()].concat()
    }

    /// What the identity of a filesystem on this drive is digested from: the
    /// drive's identifier and the filesystem's UUID, so the same card read in
    /// another reader, or a copy of the filesystem on another drive, is not the
    /// filesystem a person approved.
    #[must_use]
    pub fn filesystem_as_reported(&self, filesystem: &Filesystem) -> Vec<u8> {
        [
            A_FILESYSTEM,
            self.identifier.as_bytes(),
            b"\0",
            filesystem.uuid.as_bytes(),
        ]
        .concat()
    }
}

/// Everything the disk service reported at one moment.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheDrives {
    /// Every drive it reported with an identifier.
    pub drives: Vec<Drive>,
}

/// Whether some reported text can name a thing again: not empty, not longer
/// than `longest`, and holding no control character — which would let one name
/// end where another begins inside an identity.
fn a_name(text: &str, longest: usize) -> bool {
    !text.is_empty() && text.len() <= longest && !text.chars().any(char::is_control)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A stick with one filesystem on it.
    fn a_stick(identifier: &str, uuid: &str) -> Drive {
        Drive::reported(identifier, Plugged::Removable, Health::NotKnown)
            .unwrap()
            .with(Filesystem::reported(uuid, false).unwrap())
    }

    /// **A drive and a filesystem never share an identity**, and a filesystem's
    /// identity is its drive's and its own together.
    #[test]
    fn a_drive_and_its_filesystem_are_different_identities() {
        let stick = a_stick("Kingston-DataTraveler-1C1B", "1234-ABCD");
        let filesystem = stick.filesystems().first().unwrap();
        assert_ne!(
            stick.as_reported(),
            stick.filesystem_as_reported(filesystem)
        );

        let same_card_elsewhere = a_stick("Generic-Reader-99", "1234-ABCD");
        let elsewhere = same_card_elsewhere.filesystems().first().unwrap();
        assert_ne!(
            stick.filesystem_as_reported(filesystem),
            same_card_elsewhere.filesystem_as_reported(elsewhere)
        );
    }

    /// **The boundary between an identifier and a UUID cannot be moved**: a
    /// drive called `a` with a filesystem `b-c` is not a drive called `a-b` with
    /// a filesystem `c`, and neither may hold the separator.
    #[test]
    fn where_one_name_ends_cannot_be_moved() {
        let one = a_stick("a", "b-c");
        let two = a_stick("a-b", "c");
        assert_ne!(
            one.filesystem_as_reported(one.filesystems().first().unwrap()),
            two.filesystem_as_reported(two.filesystems().first().unwrap())
        );
        assert_eq!(
            Drive::reported("a\0b", Plugged::Removable, Health::Good),
            None
        );
        assert_eq!(Filesystem::reported("b\0c", false), None);
    }

    /// **Something with no name to be found by again is not reported**: no
    /// identifier, or one longer than a disk service writes.
    #[test]
    fn a_drive_or_filesystem_with_no_lasting_name_is_not_reported() {
        assert_eq!(Drive::reported("", Plugged::Removable, Health::Good), None);
        assert_eq!(
            Drive::reported(&"x".repeat(257), Plugged::Removable, Health::Good),
            None
        );
        assert_eq!(Filesystem::reported("", true), None);
        assert!(Drive::reported(&"x".repeat(256), Plugged::BuiltIn, Health::Good).is_some());
    }
}
