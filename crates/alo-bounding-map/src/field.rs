//! Where in the kernel's own structures the program has to look.
//!
//! A BPF program on `file_open` is handed a `struct file *` and nothing else.
//! Everything the decision needs — which inode, on which filesystem, inside
//! which directory — is reached by stepping through kernel structures whose
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
/// the socket it was handed to the address of whoever is on the other end.
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
}

impl Field {
    /// Every field the program needs, in the order it needs them.
    ///
    /// The loader walks this, so a field added here is a field looked up rather
    /// than a field silently left at zero.
    pub const ALL: [Self; 13] = [
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
            Self::SocketSock => 7,
            Self::SockFamily => 8,
            Self::SockPort => 9,
            Self::SockAddress => 10,
            Self::SockAddress6 => 11,
            Self::MessageName => 12,
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
            Self::SocketSock => "socket",
            Self::SockFamily | Self::SockPort | Self::SockAddress | Self::SockAddress6 => "sock",
            Self::MessageName => "msghdr",
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
            | Self::MessageName => 8,
            // `unsigned long` on the machines alo OS certifies.
            Self::InodeNumber => 8,
            // `dev_t`, which is thirty-two bits and has been since 2.6.
            Self::SuperDevice => 4,
            // A family and a port are both sixteen bits, in every `sockaddr`
            // there is and in the socket that remembers them.
            Self::SockFamily | Self::SockPort => 2,
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
        assert_eq!(Field::ALL.len(), 13);
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
}
