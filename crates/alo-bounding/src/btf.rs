//! The kernel's own description of itself, and the one question asked of it.
//!
//! A kernel built with `CONFIG_DEBUG_INFO_BTF=y` publishes the layout of every
//! structure it contains at `/sys/kernel/btf/vmlinux`. This file reads that and
//! answers exactly one question: **at what byte offset, in this kernel, does
//! this member of this structure sit, and how wide is it.**
//!
//! It is the alternative to the usual answer, which is to generate a header
//! from one kernel and compile the program against it. ADR 0015 rules that out
//! in its own words — *no module compiled against a kernel version* — and the
//! reason is not tidiness: a baked-in offset does not fail when the machine
//! takes a kernel update, it reads the wrong eight bytes and refuses the wrong
//! files, silently, on somebody's machine.
//!
//! # Why this is written here rather than rented
//!
//! `aya` parses this format and keeps what it parsed private, because what it
//! needs from it is the type of a function to attach to rather than the layout
//! of a structure. Nothing else in the workspace reads a kernel format, so this
//! is the file that does — the same shape as `alo-files`' `zip.rs`, which is
//! there because a format nobody else in this repository speaks is still a
//! format, and it belongs in one file with the specification quoted beside it.
//!
//! # The format, as much of it as is used
//!
//! A header, then a run of type records, then a run of null-terminated strings.
//! Every offset in the header is measured from the end of the header, which is
//! the first thing that catches somebody reading it quickly.
//!
//! ```text
//! header   magic u16 = 0xeb9f, version u8, flags u8, header length u32,
//!          type offset u32, type length u32, string offset u32, string length u32
//! type     name offset u32, info u32, size-or-type u32, then extra by kind
//! info     bits  0..16  how many members, parameters or values follow
//!          bits 24..29  which kind of type this is
//!          bit  31      whether a member's offset carries a bitfield width
//! ```
//!
//! Type ids start at 1; id 0 is `void` and is not written down.

use std::fs;

use crate::failing::NotBounded;

/// Where a kernel that was built to say so keeps its own type information.
const WHERE_THE_KERNEL_SAYS: &str = "/sys/kernel/btf/vmlinux";

/// What the first two bytes of the file are, on a machine that wrote them in
/// its own byte order.
const MAGIC: u16 = 0xeb9f;

/// How many bytes a header is, and where offsets are measured from if the
/// header does not say otherwise.
const SMALLEST_HEADER: usize = 24;

/// How many bytes one type record is before its kind-specific tail.
const RECORD: usize = 12;

/// How many bytes a pointer is.
///
/// Not read from the type information, because BTF describes a pointer's target
/// and not its width — the width is the machine's, and alo OS certifies
/// sixty-four-bit machines.
const POINTER: u32 = 8;

/// How far a chain of typedefs and qualifiers is followed before it is treated
/// as a kernel that cannot be read.
///
/// A cycle here would be a file that hangs the daemon at start-up, and this
/// file is read from `/sys` rather than written by us.
const PATIENCE: usize = 16;

/// The kinds of type this file knows how to step over or resolve.
mod kind {
    /// An integer, with four bytes of encoding after it.
    pub const INT: u32 = 1;
    /// A pointer.
    pub const PTR: u32 = 2;
    /// An array, with an element type, an index type and a count after it.
    pub const ARRAY: u32 = 3;
    /// A structure, with one member record per member after it.
    pub const STRUCT: u32 = 4;
    /// A union, laid out like a structure.
    pub const UNION: u32 = 5;
    /// An enumeration, with one value record per value after it.
    pub const ENUM: u32 = 6;
    /// A forward declaration.
    pub const FWD: u32 = 7;
    /// A name for another type.
    pub const TYPEDEF: u32 = 8;
    /// `volatile`.
    pub const VOLATILE: u32 = 9;
    /// `const`.
    pub const CONST: u32 = 10;
    /// `restrict`.
    pub const RESTRICT: u32 = 11;
    /// A function.
    pub const FUNC: u32 = 12;
    /// A function's signature, with one parameter record each after it.
    pub const FUNC_PROTO: u32 = 13;
    /// A variable.
    pub const VAR: u32 = 14;
    /// A section, with one record per variable in it.
    pub const DATASEC: u32 = 15;
    /// A floating point number.
    pub const FLOAT: u32 = 16;
    /// A tag on a declaration.
    pub const DECL_TAG: u32 = 17;
    /// A tag on a type.
    pub const TYPE_TAG: u32 = 18;
    /// An enumeration whose values need sixty-four bits.
    pub const ENUM64: u32 = 19;
}

/// Where a member sits in the structure it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Member {
    /// How many bytes from the start of the structure.
    pub offset: u32,

    /// How many bytes the member occupies.
    pub width: u32,
}

/// This kernel's account of its own structures.
#[derive(Debug)]
pub struct Types {
    /// The whole file, as the kernel published it.
    file: Vec<u8>,

    /// Where each type record starts, by type id. The first entry is id 1.
    at: Vec<usize>,

    /// Where the strings start.
    strings: usize,

    /// How long the strings are.
    strings_length: usize,
}

impl Types {
    /// What the kernel this daemon is running on says about itself.
    ///
    /// A kernel that says nothing is a kernel this boundary will not be imposed
    /// on, and the refusal names the file rather than the feature — because
    /// what is missing is `CONFIG_DEBUG_INFO_BTF`, and somebody has to be able
    /// to work that out from a machine that will not start.
    pub fn of_this_kernel() -> Result<Self, NotBounded> {
        let file =
            fs::read(WHERE_THE_KERNEL_SAYS).map_err(|why| NotBounded::NoTypeInformation {
                path: WHERE_THE_KERNEL_SAYS.to_owned(),
                why,
            })?;
        Self::read(file)
    }

    /// The same, from bytes somebody already has.
    pub fn read(file: Vec<u8>) -> Result<Self, NotBounded> {
        let magic = u16(&file, 0).ok_or(NotBounded::TypesAreNotReadable {
            what: "the file is shorter than a header",
        })?;
        if magic != MAGIC {
            return Err(NotBounded::TypesAreNotReadable {
                what: "the file does not begin as type information, or was written by a machine \
                       of the other byte order",
            });
        }
        let header = u32(&file, 4).ok_or(NotBounded::TypesAreNotReadable {
            what: "the file is shorter than a header",
        })? as usize;
        let header = header.max(SMALLEST_HEADER);
        let field = |at: usize| {
            u32(&file, at).ok_or(NotBounded::TypesAreNotReadable {
                what: "the header stops before it has said where anything is",
            })
        };
        let types = header + field(8)? as usize;
        let types_length = field(12)? as usize;
        let strings = header + field(16)? as usize;
        let strings_length = field(20)? as usize;

        if file.len() < types + types_length || file.len() < strings + strings_length {
            return Err(NotBounded::TypesAreNotReadable {
                what: "the header describes more type information than the file holds",
            });
        }

        let mut at = Vec::new();
        let mut walked = types;
        while walked + RECORD <= types + types_length {
            at.push(walked);
            let info = u32(&file, walked + 4).ok_or(NotBounded::TypesAreNotReadable {
                what: "a type record stops in the middle of itself",
            })?;
            walked += RECORD
                + tail(info).ok_or(NotBounded::TypesAreNotReadable {
                    what: "a type record is of a kind this kernel's writer invented",
                })?;
        }

        Ok(Self {
            file,
            at,
            strings,
            strings_length,
        })
    }

    /// Where a member of a structure sits, and how wide it is.
    ///
    /// [`None`] when this kernel has no such structure, or the structure has no
    /// such member — both of which mean the same thing to the caller, which is
    /// that the program cannot be told where to look.
    pub fn member(&self, structure: &str, member: &str) -> Option<Member> {
        let found = self.structure_called(structure)?;
        let info = u32(&self.file, found + 4)?;
        let bitfields = info >> 31 == 1;
        let members = (info & 0xffff) as usize;
        for which in 0..members {
            let record = found + RECORD + which * 12;
            let name = u32(&self.file, record)?;
            if self.string_at(name)? != member {
                continue;
            }
            let of_type = u32(&self.file, record + 4)?;
            let offset = u32(&self.file, record + 8)?;
            // With bitfields in the structure the high byte of the offset is
            // the member's width in bits, and only the low twenty-four are the
            // position. Without them the whole word is the position. Both are
            // in bits, and nothing this program reads is a bitfield.
            let bits = if bitfields {
                offset & 0x00ff_ffff
            } else {
                offset
            };
            return Some(Member {
                offset: bits / 8,
                width: self.width_of(of_type, PATIENCE)?,
            });
        }
        None
    }

    /// Where the record for a structure of that name starts.
    ///
    /// A name can belong to several types — a forward declaration and the
    /// structure it stands for share one — so the search is for a structure
    /// with members, which is the definition rather than the promise of one.
    fn structure_called(&self, name: &str) -> Option<usize> {
        self.at.iter().copied().find(|record| {
            let Some(info) = u32(&self.file, record + 4) else {
                return false;
            };
            if (info >> 24) & 0x1f != kind::STRUCT || info & 0xffff == 0 {
                return false;
            }
            u32(&self.file, *record)
                .and_then(|at| self.string_at(at))
                .is_some_and(|found| found == name)
        })
    }

    /// How many bytes a type occupies, following the names and qualifiers put
    /// in front of it.
    fn width_of(&self, of_type: u32, patience: usize) -> Option<u32> {
        if patience == 0 || of_type == 0 {
            return None;
        }
        let record = *self.at.get((of_type as usize).checked_sub(1)?)?;
        let info = u32(&self.file, record + 4)?;
        let size_or_type = u32(&self.file, record + 8)?;
        match (info >> 24) & 0x1f {
            kind::INT | kind::STRUCT | kind::UNION | kind::ENUM | kind::ENUM64 | kind::FLOAT => {
                Some(size_or_type)
            }
            kind::PTR => Some(POINTER),
            kind::TYPEDEF
            | kind::VOLATILE
            | kind::CONST
            | kind::RESTRICT
            | kind::TYPE_TAG
            | kind::DECL_TAG => self.width_of(size_or_type, patience - 1),
            kind::ARRAY => {
                let element = u32(&self.file, record + RECORD)?;
                let many = u32(&self.file, record + RECORD + 8)?;
                self.width_of(element, patience - 1)?.checked_mul(many)
            }
            _ => None,
        }
    }

    /// One of the null-terminated names at the end of the file.
    fn string_at(&self, offset: u32) -> Option<&str> {
        let from = self.strings.checked_add(offset as usize)?;
        if offset as usize >= self.strings_length {
            return None;
        }
        let rest = self.file.get(from..self.strings + self.strings_length)?;
        let end = rest.iter().position(|byte| *byte == 0)?;
        std::str::from_utf8(rest.get(..end)?).ok()
    }
}

/// How many bytes follow a type record, given its info word.
///
/// [`None`] for a kind this file does not know, because the next record's
/// position depends on this answer — a guess would not misread one type, it
/// would misread every type after it.
fn tail(info: u32) -> Option<usize> {
    let many = (info & 0xffff) as usize;
    match (info >> 24) & 0x1f {
        kind::INT | kind::VAR | kind::DECL_TAG => Some(4),
        kind::ARRAY => Some(12),
        kind::STRUCT | kind::UNION | kind::DATASEC => Some(many * 12),
        kind::ENUM | kind::FUNC_PROTO => Some(many * 8),
        kind::ENUM64 => Some(many * 12),
        kind::PTR
        | kind::FWD
        | kind::TYPEDEF
        | kind::VOLATILE
        | kind::CONST
        | kind::RESTRICT
        | kind::FUNC
        | kind::FLOAT
        | kind::TYPE_TAG => Some(0),
        _ => None,
    }
}

/// Two bytes, in the byte order the machine that wrote them used.
fn u16(file: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(file.get(at..at + 2)?.try_into().ok()?))
}

/// Four bytes, in the byte order the machine that wrote them used.
fn u32(file: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(file.get(at..at + 4)?.try_into().ok()?))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing;

    /// The seven fields the program needs, found in a small file written by
    /// this repository's own fixture rather than by a kernel — so the parsing
    /// is held to a case on a machine with no kernel worth reading.
    #[test]
    fn a_member_is_found_where_the_type_information_says_it_is() {
        let types = Types::read(testing::some_type_information()).expect("the fixture reads");
        assert_eq!(
            types.member("file", "f_path"),
            Some(Member {
                offset: 16,
                width: 16
            })
        );
        assert_eq!(
            types.member("dentry", "d_parent"),
            Some(Member {
                offset: 24,
                width: 8
            })
        );
        assert_eq!(
            types.member("super_block", "s_dev"),
            Some(Member {
                offset: 8,
                width: 4
            })
        );
    }

    /// A name is followed through to the type it names — `dev_t` is a name for
    /// a name for a thirty-two bit integer, and a boundary that answered eight
    /// there would read a device number and half of whatever is beside it.
    #[test]
    fn a_width_is_the_width_of_what_a_name_finally_names() {
        let types = Types::read(testing::some_type_information()).expect("the fixture reads");
        assert_eq!(
            types.member("super_block", "s_dev").map(|m| m.width),
            Some(4)
        );
        assert_eq!(types.member("inode", "i_ino").map(|m| m.width), Some(8));
    }

    /// A structure this kernel does not have, and a member the structure does
    /// not have, are the same answer: the program cannot be told where to look.
    #[test]
    fn a_structure_or_a_member_that_is_not_there_is_not_invented() {
        let types = Types::read(testing::some_type_information()).expect("the fixture reads");
        assert_eq!(types.member("file", "f_something_else"), None);
        assert_eq!(types.member("no_such_structure", "f_path"), None);
    }

    /// The file is read from `/sys` rather than written by us, so every way it
    /// can be wrong is a refusal with a sentence rather than a panic.
    #[test]
    fn a_file_that_is_not_type_information_is_refused() {
        assert!(matches!(
            Types::read(Vec::new()),
            Err(NotBounded::TypesAreNotReadable { .. })
        ));
        assert!(matches!(
            Types::read(b"not a kernel at all, but long enough to have a header".to_vec()),
            Err(NotBounded::TypesAreNotReadable { .. })
        ));
        let mut truncated = testing::some_type_information();
        truncated.truncate(30);
        assert!(matches!(
            Types::read(truncated),
            Err(NotBounded::TypesAreNotReadable { .. })
        ));
    }
}
