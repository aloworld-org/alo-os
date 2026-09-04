//! One thing on a disk, named the way the kernel names it.
//!
//! A grant in `alo-capability` is a path, decided lexically and without
//! touching a disk, and that is right for the crate that answers a person's
//! question about what an agent may reach. It is not what the kernel has to
//! hand. Inside `file_open` there is no path — there is a `struct file`, and
//! underneath it a chain of directory entries, each with an inode number and
//! the filesystem it lives on.
//!
//! So a bound crosses into the kernel as those two numbers, and the daemon
//! resolves the person's folder into them once, at the moment the turn begins.
//! Two consequences, both of which are improvements on comparing text:
//!
//! - **A symbolic link out of a granted folder cannot widen the bound.** The
//!   kernel walks the directory entries the open actually went through, not a
//!   name somebody assembled, so `path.rs`'s note about resolving links first
//!   is answered by the mechanism instead of by whoever wrote the verb.
//! - **A name is not a place.** Renaming a granted folder does not move the
//!   bound off it, and creating a folder with the granted folder's old name
//!   does not move the bound onto it.

/// One thing on a disk: which filesystem, and which inode on it.
///
/// Both numbers are the kernel's own, and the second alone is not enough — two
/// filesystems mounted at once will each have an inode 2, and a bound that
/// compared inode numbers only would let a turn granted one machine's folder
/// reach a different folder on a USB stick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Place {
    /// The filesystem, as `super_block->s_dev` holds it.
    ///
    /// This is *not* what `stat` reports in `st_dev`: the two are different
    /// packings of the same major and minor numbers, and comparing one against
    /// the other never matches. `alo-bounding` converts, and `docs/quirks.md`
    /// records it.
    device: u64,

    /// The inode, as `inode->i_ino` holds it.
    inode: u64,
}

impl Place {
    /// A place named by the two numbers the kernel knows it by.
    #[must_use]
    pub const fn of(device: u64, inode: u64) -> Self {
        Self { device, inode }
    }

    /// The filesystem this place is on.
    #[must_use]
    pub const fn device(&self) -> u64 {
        self.device
    }

    /// The inode this place is.
    #[must_use]
    pub const fn inode(&self) -> u64 {
        self.inode
    }

    /// The value as the map holds it.
    ///
    /// The map is shared memory between two programs compiled separately, so
    /// the order of these two numbers is the only thing keeping them talking
    /// about the same folder. It is decided here, once, and both halves call
    /// this rather than laying out sixteen bytes of their own.
    #[must_use]
    pub const fn words(&self) -> [u64; 2] {
        [self.device, self.inode]
    }

    /// A place read back out of the map.
    #[must_use]
    pub const fn of_words(words: [u64; 2]) -> Self {
        let [device, inode] = words;
        Self { device, inode }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The map is two programs sharing memory, so a round trip through it has
    /// to be exactly the identity — and in the order written here, because the
    /// kernel half reads the same two words by position.
    #[test]
    fn a_place_survives_the_map_unchanged() {
        let place = Place::of(0x0080_0002, 4_198_531);
        assert_eq!(Place::of_words(place.words()), place);
        assert_eq!(place.words(), [0x0080_0002, 4_198_531]);
        assert_eq!(Place::of_words([7, 9]).device(), 7);
        assert_eq!(Place::of_words([7, 9]).inode(), 9);
    }

    /// The same inode number on two filesystems is two different places. A
    /// bound that forgot the device would be a grant to every mounted disk at
    /// once.
    #[test]
    fn the_same_inode_on_another_filesystem_is_somewhere_else() {
        assert_ne!(Place::of(1, 42), Place::of(2, 42));
        assert_ne!(Place::of(1, 42), Place::of(1, 43));
        assert_eq!(Place::of(1, 42), Place::of(1, 42));
    }
}
