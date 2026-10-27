//! Where in the kernel's own structures the program has to look.
//!
//! A BPF program on `file_open` or `file_permission` is handed a
//! `struct file *` and nothing else. Everything the decision needs — which
//! inode, on which filesystem, inside which directory, and whether the file is
//! a socket at all — is reached by stepping through kernel structures whose
//! layout is decided when that kernel is compiled and is not the same on the
//! next one. The program on `socket_sendmsg` is in the same position with a
//! `struct socket *` and a `struct msghdr *`: who a socket is joined to sits
//! several structures in.
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
/// and the filesystem. After those come the six the message hook reads, from
/// the socket it was handed to the address of whoever is on the other end —
/// then the one field the read-and-write hook needs that the walk does not,
/// which is what kind of file it was handed — then the two a link-local
/// destination needs to be decided on its interface (ADR 0041), and last the
/// one that says whether a message carries control messages, which can move an
/// IPv4 datagram off the interface its socket is held to (ADR 0044). The last
/// four tell one btrfs subvolume from another, which the filesystem's own
/// device number cannot
/// (`docs/quirks/btrfs-gives-every-subvolume-its-own-device-number.md`).
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

    /// `struct socket`'s `sk` — the protocol's half of a socket, which is where
    /// a joined socket keeps who it is joined to.
    ///
    /// A `struct socket` is the filesystem's view of a socket and knows nothing
    /// about addresses; the `struct sock` it points at is the network's view
    /// and knows the peer.
    SocketSock,

    /// `struct sock`'s `__sk_common.skc_family` — which family of address this
    /// socket speaks, as the kernel numbers them.
    SockFamily,

    /// `struct sock`'s `__sk_common.skc_dport` — the peer's port, in network
    /// order, and zero for a socket joined to nobody.
    SockPort,

    /// `struct sock`'s `__sk_common.skc_daddr` — the peer's IPv4 address, in
    /// network order.
    SockAddress,

    /// `struct sock`'s `__sk_common.skc_v6_daddr` — the peer's IPv6 address,
    /// sixteen bytes in network order.
    SockAddress6,

    /// `struct msghdr`'s `msg_name` — the address a message names, or null for
    /// a message that goes wherever the socket is already joined.
    MessageName,

    /// `struct inode`'s `i_mode` — what kind of file this is, in the top four
    /// bits, and its permission bits below them.
    ///
    /// Read by the hook on every read and write, and for one reason: a socket
    /// is a file too, and a write on one is decided by `socket_sendmsg`, which
    /// reads where the bytes are going. The read-and-write hook has to know
    /// which kind it was handed so as to leave a socket to the hook that can
    /// decide about it, and the kind is in the mode.
    InodeMode,

    /// `struct sock`'s `__sk_common.skc_bound_dev_if` — the interface a socket
    /// is held to, as the kernel numbers interfaces, and zero for one held to
    /// none.
    ///
    /// Where a link-local destination's interface is when the address itself
    /// does not say it: a `connect` with a scope sets it, and a socket bound to
    /// an interface or to a scoped link-local address has it already. ADR 0041
    /// is why a link-local departure is decided with it.
    SockBoundInterface,

    /// `struct msghdr`'s `msg_namelen` — how many bytes of the address a
    /// message names are really there.
    ///
    /// Read before an IPv6 address's scope is, because the kernel copies only
    /// that many bytes of the caller's address and reads a scope only from one
    /// long enough to hold it: past the length is whatever was on the stack.
    MessageNameLength,

    /// `struct msghdr`'s `msg_controllen` — how many bytes of control messages
    /// a message carries, and zero for one that carries none.
    ///
    /// Read for one reason (ADR 0044): an IPv4 datagram's `IP_PKTINFO` names
    /// the interface it leaves by **ahead of** the one its socket is held to,
    /// and the kernel does not check the two agree — so a message carrying
    /// control messages is decided as a message held to no interface. The bytes
    /// of the control messages are not read.
    MessageControlLength,

    /// `struct super_block`'s `s_magic` — which kind of filesystem this is.
    ///
    /// Read to ask one question: is this btrfs, whose device number is the
    /// subvolume's rather than the filesystem's?
    SuperMagic,

    /// `struct btrfs_inode`'s `vfs_inode.i_ino` — the inode number again, as
    /// btrfs's own inode holds it.
    ///
    /// Read for **where** it is, never for what it is. btrfs keeps the inode the
    /// program holds inside a larger one of its own, and the distance between
    /// this offset and [`Field::InodeNumber`]'s is where the larger one begins.
    /// Asked this way, rather than for `vfs_inode` itself, because a `struct
    /// inode` is a different size on every kernel and an inode number is not, so
    /// the width check still means something.
    BtrfsInodeNumber,

    /// `struct btrfs_inode`'s `root` — the subvolume this inode is in.
    BtrfsInodeRoot,

    /// `struct btrfs_root`'s `anon_dev` — the device number btrfs gives that
    /// subvolume, which is the one `stat` reports as `st_dev`.
    BtrfsRootDevice,
}

/// What `s_magic` holds on a btrfs filesystem, as `include/uapi/linux/magic.h`
/// names it (`BTRFS_SUPER_MAGIC`).
pub const BTRFS_MAGIC: u64 = 0x9123_683E;

/// What the loader writes into a slot for a field this kernel does not
/// describe, which only a field [`Field::only_btrfs_has`] may be.
///
/// Not zero, because zero is a real offset. A program that finds this in a
/// slot it needs refuses: it cannot tell one subvolume from another, so it
/// does not guess.
pub const NOT_IN_THIS_KERNEL: u32 = u32::MAX;

impl Field {
    /// Every field the program needs, in the order it needs them.
    ///
    /// The loader walks this, so a field added here is a field looked up rather
    /// than a field silently left at zero.
    pub const ALL: [Self; 21] = [
        Self::FilePath,
        Self::PathDentry,
        Self::DentryParent,
        Self::DentryInode,
        Self::DentrySuper,
        Self::InodeNumber,
        Self::SuperDevice,
        Self::SocketSock,
        Self::SockFamily,
        Self::SockPort,
        Self::SockAddress,
        Self::SockAddress6,
        Self::MessageName,
        Self::InodeMode,
        Self::SockBoundInterface,
        Self::MessageNameLength,
        Self::MessageControlLength,
        Self::SuperMagic,
        Self::BtrfsInodeNumber,
        Self::BtrfsInodeRoot,
        Self::BtrfsRootDevice,
    ];

    /// Whether this field belongs to btrfs rather than to every kernel.
    ///
    /// A kernel that builds btrfs as a module describes its structures
    /// somewhere other than `/sys/kernel/btf/vmlinux`, and a kernel without
    /// btrfs has none. Neither is a reason to impose no boundary on any other
    /// filesystem, so these three are written as [`NOT_IN_THIS_KERNEL`] there
    /// and every file on btrfs is refused instead. The image's kernel builds
    /// btrfs in and describes all three.
    #[must_use]
    pub const fn only_btrfs_has(self) -> bool {
        matches!(
            self,
            Self::BtrfsInodeNumber | Self::BtrfsInodeRoot | Self::BtrfsRootDevice
        )
    }

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
            Self::SocketSock => 7,
            Self::SockFamily => 8,
            Self::SockPort => 9,
            Self::SockAddress => 10,
            Self::SockAddress6 => 11,
            Self::MessageName => 12,
            Self::InodeMode => 13,
            Self::SockBoundInterface => 14,
            Self::MessageNameLength => 15,
            Self::MessageControlLength => 16,
            Self::SuperMagic => 17,
            Self::BtrfsInodeNumber => 18,
            Self::BtrfsInodeRoot => 19,
            Self::BtrfsRootDevice => 20,
        }
    }

    /// The kernel structure this field is a member of.
    #[must_use]
    pub const fn structure(self) -> &'static str {
        match self {
            Self::FilePath => "file",
            Self::PathDentry => "path",
            Self::DentryParent | Self::DentryInode | Self::DentrySuper => "dentry",
            Self::InodeNumber | Self::InodeMode => "inode",
            Self::SuperDevice | Self::SuperMagic => "super_block",
            Self::SocketSock => "socket",
            Self::SockFamily
            | Self::SockPort
            | Self::SockAddress
            | Self::SockAddress6
            | Self::SockBoundInterface => "sock",
            Self::MessageName | Self::MessageNameLength | Self::MessageControlLength => "msghdr",
            Self::BtrfsInodeNumber | Self::BtrfsInodeRoot => "btrfs_inode",
            Self::BtrfsRootDevice => "btrfs_root",
        }
    }

    /// The member's name inside that structure.
    ///
    /// A name with a dot in it is a path through **named** members: `struct
    /// sock` keeps everything about its peer inside a member called
    /// `__sk_common`, and a search that only looked at the structure's own
    /// members would answer that no kernel has a `skc_daddr`. The loader
    /// follows each segment into the member it names and adds the offsets up,
    /// so the number the program reads is measured from the start of the
    /// structure named by [`Field::structure`] — which is the pointer the
    /// program holds.
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
            Self::SocketSock => "sk",
            Self::SockFamily => "__sk_common.skc_family",
            Self::SockPort => "__sk_common.skc_dport",
            Self::SockAddress => "__sk_common.skc_daddr",
            Self::SockAddress6 => "__sk_common.skc_v6_daddr",
            Self::MessageName => "msg_name",
            Self::InodeMode => "i_mode",
            Self::SockBoundInterface => "__sk_common.skc_bound_dev_if",
            Self::MessageNameLength => "msg_namelen",
            Self::MessageControlLength => "msg_controllen",
            Self::SuperMagic => "s_magic",
            Self::BtrfsInodeNumber => "vfs_inode.i_ino",
            Self::BtrfsInodeRoot => "root",
            Self::BtrfsRootDevice => "anon_dev",
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
            Self::PathDentry
            | Self::DentryParent
            | Self::DentryInode
            | Self::DentrySuper
            | Self::SocketSock
            | Self::MessageName
            | Self::BtrfsInodeRoot => 8,
            // `__kernel_size_t`, which is an `unsigned long`.
            Self::MessageControlLength => 8,
            // `unsigned long` on the machines alo OS certifies, and a magic
            // number is one too.
            Self::InodeNumber | Self::BtrfsInodeNumber | Self::SuperMagic => 8,
            // `dev_t`, which is thirty-two bits and has been since 2.6.
            Self::SuperDevice | Self::BtrfsRootDevice => 4,
            // A family and a port are both sixteen bits, in every `sockaddr`
            // there is and in the socket that remembers them — and so is a
            // mode, which is `umode_t`, an `unsigned short`.
            Self::SockFamily | Self::SockPort | Self::InodeMode => 2,
            // An `int` each: an interface index, and a length.
            Self::SockBoundInterface | Self::MessageNameLength => 4,
            // A `__be32`, and a `struct in6_addr`.
            Self::SockAddress => 4,
            Self::SockAddress6 => 16,
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
        assert_eq!(Field::ALL.len(), 21);
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
            assert!(matches!(field.width(), 2 | 4 | 8 | 16));
        }
    }

    /// A path through named members begins and ends with a name, and never
    /// holds an empty segment: `__sk_common..skc_daddr` would be a search for
    /// a member called nothing, which every anonymous union answers to.
    #[test]
    fn a_path_through_named_members_has_no_empty_segment() {
        for field in Field::ALL {
            assert!(
                field.member().split('.').all(|segment| !segment.is_empty()),
                "{:?} names `{}`",
                field,
                field.member()
            );
        }
    }

    /// Only the three fields btrfs itself declares may be absent. `s_magic` is
    /// every filesystem's, and a kernel without it is one where the program
    /// could not even ask whether a file is on btrfs.
    #[test]
    fn only_the_three_btrfs_structures_may_be_absent() {
        for field in Field::ALL {
            assert_eq!(
                field.only_btrfs_has(),
                field.structure().starts_with("btrfs_"),
                "{field:?}"
            );
        }
    }

    /// The marker for an absent field is no offset any of these structures
    /// has, so it cannot be read as one.
    #[test]
    fn the_marker_for_an_absent_field_is_not_an_offset() {
        assert_ne!(NOT_IN_THIS_KERNEL, 0);
        assert!(NOT_IN_THIS_KERNEL > 1 << 20);
    }
}
