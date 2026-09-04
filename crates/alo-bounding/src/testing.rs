//! The fixture the other files' tests are written against.
//!
//! One thing is hard to test about this crate and it is the reading of
//! `/sys/kernel/btf/vmlinux`: the file is enormous, it is different on every
//! machine, and a test that read the real one would pass or fail for reasons
//! belonging to whoever built the kernel rather than to this repository.
//!
//! So this builds a small one — seven structures with the seven members the
//! program looks for, in a layout chosen to be *wrong* in the ways a real
//! kernel is inconvenient: a device number reached through two names before it
//! is an integer, a member that is a structure rather than a pointer to one, a
//! structure that points at itself, and `f_path` inside an anonymous union
//! rather than beside its neighbours.
//!
//! That last one is measured rather than invented. It is where Linux 6.18 keeps
//! it, and it is the shape that stopped this crate loading on a kernel whose
//! `struct file` was perfectly ordinary — so the ordinary fixture has it, and
//! every test that reads this file is a test of the walk into it.

/// A small piece of type information, in the format the kernel publishes.
///
/// The offsets and widths here are the ones `btf.rs`'s tests assert, and they
/// are made up rather than measured — what is being tested is the reading, and
/// a fixture that copied one kernel's numbers would be a test of that kernel.
pub fn some_type_information() -> Vec<u8> {
    written(Kernel::Ordinary)
}

/// A kernel with no `struct file` in it at all: far enough from what alo OS
/// certifies that the walk has nothing to walk.
pub fn type_information_without_a_file() -> Vec<u8> {
    written(Kernel::WithoutAFile)
}

/// A kernel whose inode number is four bytes rather than eight — the failure
/// that does not announce itself, and the one `fields.rs` checks for.
pub fn type_information_with_a_narrow_inode_number() -> Vec<u8> {
    written(Kernel::WithANarrowInodeNumber)
}

/// A kernel whose `struct file` holds an anonymous member of its own type.
///
/// No compiler would emit it, which is exactly why it is here: this file is
/// read from `/sys` rather than written by us, and a search that followed
/// anonymous members without counting them would not misread this kernel, it
/// would never come back from it.
pub fn type_information_that_points_into_itself() -> Vec<u8> {
    written(Kernel::ThatPointsIntoItself)
}

/// Which of the four fixtures is being written.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kernel {
    /// One this boundary can be imposed on.
    Ordinary,

    /// One with nothing called `file` in it.
    WithoutAFile,

    /// One where `i_ino` is not the width the program reads.
    WithANarrowInodeNumber,

    /// One whose anonymous member leads back to where it started.
    ThatPointsIntoItself,
}

/// The fixture, in whichever of its three shapes.
fn written(kernel: Kernel) -> Vec<u8> {
    let mut writing = Writing::new();

    // Two integers, and the two names a device number hides behind.
    let unsigned_int = writing.integer("unsigned int", 4);
    let unsigned_long = writing.integer("unsigned long", 8);
    let kernel_dev_t = writing.name_for("__kernel_dev_t", unsigned_int);
    let dev_t = writing.name_for("dev_t", kernel_dev_t);

    // A structure that points at itself, which is what walking upwards means.
    let dentry = writing.reserve();
    let dentry_pointer = writing.pointer_to(dentry);
    let inode = writing.reserve();
    let inode_pointer = writing.pointer_to(inode);
    let super_block = writing.reserve();
    let super_pointer = writing.pointer_to(super_block);
    let path = writing.reserve();
    let file = writing.reserve();
    let where_the_path_is = writing.reserve();

    writing.structure(
        dentry,
        "dentry",
        200,
        &[
            ("d_flags", unsigned_int, 0),
            ("d_parent", dentry_pointer, 24),
            ("d_inode", inode_pointer, 32),
            ("d_sb", super_pointer, 40),
        ],
    );
    let inode_number = if kernel == Kernel::WithANarrowInodeNumber {
        unsigned_int
    } else {
        unsigned_long
    };
    writing.structure(
        inode,
        "inode",
        600,
        &[("i_mode", unsigned_int, 0), ("i_ino", inode_number, 32)],
    );
    writing.structure(
        super_block,
        "super_block",
        900,
        &[("s_blocksize", unsigned_long, 0), ("s_dev", dev_t, 8)],
    );
    writing.structure(
        path,
        "path",
        16,
        &[("mnt", super_pointer, 0), ("dentry", dentry_pointer, 8)],
    );
    // A kernel without a `struct file` is not a file short of a record — it is
    // a kernel where that structure is called something else, or is not there
    // at all. Written under another name, so the file still parses and the
    // refusal is about the search rather than about the format.
    let called = if kernel == Kernel::WithoutAFile {
        "not_a_file"
    } else {
        "file"
    };
    // Where Linux 6.18 keeps `f_path`: an unnamed union, with a second member
    // over the same bytes, sixteen into the file.
    writing.union(
        where_the_path_is,
        "",
        16,
        &[("f_path", path, 0), ("__f_path", unsigned_long, 0)],
    );
    let unnamed = if kernel == Kernel::ThatPointsIntoItself {
        file
    } else {
        where_the_path_is
    };
    writing.structure(
        file,
        called,
        400,
        &[("f_mode", unsigned_int, 0), ("", unnamed, 16)],
    );

    writing.finished()
}

/// How many bytes a type record is before its kind-specific tail.
const RECORD: usize = 12;

/// A piece of type information being written, in the kernel's own format.
struct Writing {
    /// The type records, in order, each already the right length.
    types: Vec<Vec<u8>>,

    /// The names, null-terminated, starting with the empty one.
    strings: Vec<u8>,
}

impl Writing {
    /// Nothing written yet, and the empty name already at offset zero — which
    /// is the format's own convention for *this thing has no name*.
    fn new() -> Self {
        Self {
            types: Vec::new(),
            strings: vec![0],
        }
    }

    /// A name, at the offset it ends up at.
    ///
    /// The empty one is already at zero and is not written again, which is the
    /// format's own convention for *this thing has no name* and is how a real
    /// kernel writes an anonymous member.
    fn named(&mut self, name: &str) -> u32 {
        if name.is_empty() {
            return 0;
        }
        let at = self.strings.len() as u32;
        self.strings.extend_from_slice(name.as_bytes());
        self.strings.push(0);
        at
    }

    /// Somewhere for a type that has to exist before it can be written, because
    /// something it contains points back at it.
    fn reserve(&mut self) -> u32 {
        self.types.push(vec![0; RECORD]);
        self.types.len() as u32
    }

    /// An integer of a given width.
    fn integer(&mut self, name: &str, width: u32) -> u32 {
        let at = self.named(name);
        let mut record = record(at, 1, 0, width);
        record.extend_from_slice(&0u32.to_le_bytes());
        self.types.push(record);
        self.types.len() as u32
    }

    /// A pointer to something.
    fn pointer_to(&mut self, target: u32) -> u32 {
        self.types.push(record(0, 2, 0, target));
        self.types.len() as u32
    }

    /// Another name for a type that already exists.
    fn name_for(&mut self, name: &str, target: u32) -> u32 {
        let at = self.named(name);
        self.types.push(record(at, 8, 0, target));
        self.types.len() as u32
    }

    /// A structure, written into a place already reserved for it. Member
    /// offsets are given in bytes here and written in bits, as the format
    /// keeps them.
    fn structure(&mut self, id: u32, name: &str, size: u32, members: &[(&str, u32, u32)]) {
        self.composite(id, 4, name, size, members);
    }

    /// A union, which is a structure whose members all begin at the same place.
    fn union(&mut self, id: u32, name: &str, size: u32, members: &[(&str, u32, u32)]) {
        self.composite(id, 5, name, size, members);
    }

    /// Either of the two, which differ in the format by one number.
    fn composite(
        &mut self,
        id: u32,
        kind: u32,
        name: &str,
        size: u32,
        members: &[(&str, u32, u32)],
    ) {
        let at = self.named(name);
        let mut record = record(at, kind, members.len() as u32, size);
        let named: Vec<u32> = members
            .iter()
            .map(|(name, _, _)| self.named(name))
            .collect();
        for ((_, of_type, offset), name) in members.iter().zip(named) {
            record.extend_from_slice(&name.to_le_bytes());
            record.extend_from_slice(&of_type.to_le_bytes());
            record.extend_from_slice(&(offset * 8).to_le_bytes());
        }
        if let Some(slot) = self.types.get_mut(id as usize - 1) {
            *slot = record;
        }
    }

    /// The whole file: a header, the types, then the names.
    fn finished(self) -> Vec<u8> {
        let types: Vec<u8> = self.types.into_iter().flatten().collect();
        let mut file = Vec::new();
        file.extend_from_slice(&0xeb9f_u16.to_le_bytes());
        file.push(1);
        file.push(0);
        file.extend_from_slice(&24u32.to_le_bytes());
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&(types.len() as u32).to_le_bytes());
        file.extend_from_slice(&(types.len() as u32).to_le_bytes());
        file.extend_from_slice(&(self.strings.len() as u32).to_le_bytes());
        file.extend_from_slice(&types);
        file.extend_from_slice(&self.strings);
        file
    }
}

/// One type record's first twelve bytes.
fn record(name: u32, kind: u32, many: u32, size_or_type: u32) -> Vec<u8> {
    let mut written = Vec::with_capacity(RECORD);
    written.extend_from_slice(&name.to_le_bytes());
    written.extend_from_slice(&((kind << 24) | many).to_le_bytes());
    written.extend_from_slice(&size_or_type.to_le_bytes());
    written
}
