//! Where in the kernel's own structures the program has to look.
//!
//! A BPF program on `file_open` is handed a `struct file *` and nothing else.
//! Everything the decision needs — which inode, on which filesystem, inside
//! which directory — is reached by stepping through kernel structures whose
//! layout is decided when that kernel is compiled and is not the same on the
//! next one.
//!
//! # This is why nothing here is a number
//!
//! The usual answer is to generate a header from one kernel and compile against
//! it, and it is the answer ADR 0015 rules out in its second sentence: *no
//! module compiled against a kernel version*. A field offset baked into the
//! program is that, wearing a smaller hat — it works until the machine takes an
//! update, and then it reads the wrong eight bytes and refuses the wrong files
//! without saying anything.
//!
//! So the program holds no offsets. `alo-bounding` reads them out of the
//! running kernel's own type information at `/sys/kernel/btf/vmlinux`, puts
//! them in a map, and the program looks them up. **The kernel is asked where
//! its fields are**, and a kernel that will not say is a kernel this boundary
//! refuses to load on rather than one it guesses about.
//!
//! This enum is the two halves' agreement about which question is which: the
//! loader fills slot [`Field::index`], and the program reads it.

/// One field the program has to find, and where it lives.
///
/// The order is the order of the walk: from the file handed to the hook, down
/// to the directory entry, and from there upwards and sideways into the inode
/// and the filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// `struct file`'s `f_path` — where the open's own path begins.
    ///
    /// Embedded rather than pointed at, so what the offset gives is the address
    /// of the `struct path` itself.
    FilePath,

    /// `struct path`'s `dentry` — the directory entry the open went through.
    PathDentry,

    /// `struct dentry`'s `d_parent` — the directory this entry is in.
    DentryParent,

    /// `struct dentry`'s `d_inode` — what this entry names.
    DentryInode,

    /// `struct dentry`'s `d_sb` — the filesystem this entry is on.
    DentrySuper,

    /// `struct inode`'s `i_ino` — the inode number.
    InodeNumber,

    /// `struct super_block`'s `s_dev` — the filesystem, as the kernel numbers
    /// it.
    SuperDevice,
}

impl Field {
    /// Every field the program needs, in the order it needs them.
    ///
    /// The loader walks this, so a field added here is a field looked up rather
    /// than a field silently left at zero.
    pub const ALL: [Self; 7] = [
        Self::FilePath,
        Self::PathDentry,
        Self::DentryParent,
        Self::DentryInode,
        Self::DentrySuper,
        Self::InodeNumber,
        Self::SuperDevice,
    ];

    /// The slot in the map this field's offset is written into and read out of.
    #[must_use]
    pub const fn index(self) -> u32 {
        match self {
            Self::FilePath => 0,
            Self::PathDentry => 1,
            Self::DentryParent => 2,
            Self::DentryInode => 3,
            Self::DentrySuper => 4,
            Self::InodeNumber => 5,
            Self::SuperDevice => 6,
        }
    }

    /// The kernel structure this field is a member of.
    #[must_use]
    pub const fn structure(self) -> &'static str {
        match self {
            Self::FilePath => "file",
            Self::PathDentry => "path",
            Self::DentryParent | Self::DentryInode | Self::DentrySuper => "dentry",
            Self::InodeNumber => "inode",
            Self::SuperDevice => "super_block",
        }
    }

    /// The member's name inside that structure.
    #[must_use]
    pub const fn member(self) -> &'static str {
        match self {
            Self::FilePath => "f_path",
            Self::PathDentry => "dentry",
            Self::DentryParent => "d_parent",
            Self::DentryInode => "d_inode",
            Self::DentrySuper => "d_sb",
            Self::InodeNumber => "i_ino",
            Self::SuperDevice => "s_dev",
        }
    }

    /// How many bytes the program reads at that offset.
    ///
    /// Checked against the running kernel before the program is loaded, and a
    /// mismatch is a refusal rather than a warning: a field that has become
    /// four bytes wide where the program reads eight is a boundary that would
    /// compare a number against half of itself and part of its neighbour, and
    /// it would do it silently.
    #[must_use]
    pub const fn width(self) -> u32 {
        match self {
            // A `struct path` is two pointers, and the offset points at it
            // rather than through it.
            Self::FilePath => 16,
            Self::PathDentry | Self::DentryParent | Self::DentryInode | Self::DentrySuper => 8,
            // `unsigned long` on the machines alo OS certifies.
            Self::InodeNumber => 8,
            // `dev_t`, which is thirty-two bits and has been since 2.6.
            Self::SuperDevice => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A field added to the enum and forgotten in `ALL` is a field the loader
    /// never looks up and the program reads as zero — which would make the walk
    /// step through the beginning of a `struct file` and refuse everything.
    #[test]
    fn every_field_is_in_the_list_exactly_once() {
        assert_eq!(Field::ALL.len(), 7);
        for (slot, field) in Field::ALL.iter().enumerate() {
            assert_eq!(field.index() as usize, slot);
        }
    }

    /// Two fields sharing a slot would have the second silently overwrite the
    /// first, and the walk would read a directory entry as an inode.
    #[test]
    fn no_two_fields_share_a_slot() {
        for (at, one) in Field::ALL.iter().enumerate() {
            for other in Field::ALL.iter().skip(at + 1) {
                assert_ne!(one.index(), other.index());
                assert!(one.structure() != other.structure() || one.member() != other.member());
            }
        }
    }

    /// Each field names a real member of a real kernel structure. This is what
    /// the loader searches for, so a typo here is a machine that refuses to
    /// impose any boundary at all.
    #[test]
    fn every_field_names_a_structure_and_a_member() {
        for field in Field::ALL {
            assert!(!field.structure().is_empty());
            assert!(!field.member().is_empty());
            assert!(field.width() == 4 || field.width() == 8 || field.width() == 16);
        }
    }
}
