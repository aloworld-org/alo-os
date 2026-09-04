//! The seven offsets, found in this kernel and checked before they are used.
//!
//! [`Field`] says which seven and how wide each should be; `btf.rs` says where
//! this kernel keeps them. This file is the meeting of the two, and the two
//! refusals that come out of it.
//!
//! # The width check is the point of this file
//!
//! Finding an offset is a lookup. Checking that the member is the width the
//! program reads is the part that matters, because it is the failure that does
//! not announce itself: a kernel where `i_ino` became four bytes would still
//! have an `i_ino`, at an offset, and the program would read it plus four bytes
//! of whatever follows it, compare that against a granted inode, and refuse
//! every file — or, on a different arrangement of the same accident, allow one.
//!
//! So a width that does not match is a refusal at start-up with both numbers in
//! it, rather than a boundary that is imposed and wrong.

use alo_bounding_map::Field;

use crate::{btf::Types, failing::NotBounded};

/// Where this kernel keeps the fields the program reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offsets {
    /// One offset per [`Field`], in [`Field::index`] order.
    found: [u32; Field::ALL.len()],
}

impl Offsets {
    /// Every field, looked up in this kernel and held to its width.
    ///
    /// The first field that is missing or the wrong width ends this, because
    /// there is nothing useful to do with six of seven: a walk missing one step
    /// is a walk that refuses everything.
    pub fn found(types: &Types) -> Result<Self, NotBounded> {
        let mut found = [0; Field::ALL.len()];
        for field in Field::ALL {
            let member = types
                .member(field.structure(), field.member())
                .ok_or_else(|| NotBounded::missing(field))?;
            if member.width != field.width() {
                return Err(NotBounded::wrong_width(field, member.width));
            }
            if let Some(slot) = found.get_mut(field.index() as usize) {
                *slot = member.offset;
            }
        }
        Ok(Self { found })
    }

    /// Where one field sits.
    #[must_use]
    pub fn at(&self, field: Field) -> u32 {
        self.found
            .get(field.index() as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Each field's slot in the map, and what goes in it.
    pub fn each(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        Field::ALL
            .into_iter()
            .map(|field| (field.index(), self.at(field)))
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing;

    /// The ordinary case, against the fixture: seven fields, each at the offset
    /// the type information gives.
    #[test]
    fn every_field_is_found_where_this_kernel_keeps_it() {
        let types = Types::read(testing::some_type_information()).expect("the fixture reads");
        let offsets = Offsets::found(&types).expect("the fixture has all seven");
        assert_eq!(offsets.at(Field::FilePath), 16);
        assert_eq!(offsets.at(Field::PathDentry), 8);
        assert_eq!(offsets.at(Field::DentryParent), 24);
        assert_eq!(offsets.at(Field::DentryInode), 32);
        assert_eq!(offsets.at(Field::DentrySuper), 40);
        assert_eq!(offsets.at(Field::InodeNumber), 32);
        assert_eq!(offsets.at(Field::SuperDevice), 8);
        assert_eq!(offsets.each().count(), Field::ALL.len());
    }

    /// A kernel with no `struct file` at all — far enough from what alo OS
    /// certifies that there is nothing to walk, and a refusal that names what
    /// was looked for.
    #[test]
    fn a_kernel_missing_a_field_is_refused_by_name() {
        let types =
            Types::read(testing::type_information_without_a_file()).expect("the fixture reads");
        assert!(matches!(
            Offsets::found(&types),
            Err(NotBounded::FieldIsMissing {
                structure: "file",
                member: "f_path"
            })
        ));
    }

    /// The refusal this file exists for: the field is there, at a width that
    /// would have the program read a number and part of its neighbour.
    #[test]
    fn a_field_of_the_wrong_width_is_refused_with_both_numbers() {
        let types = Types::read(testing::type_information_with_a_narrow_inode_number())
            .expect("the fixture reads");
        assert!(matches!(
            Offsets::found(&types),
            Err(NotBounded::FieldIsNotTheWidth {
                structure: "inode",
                member: "i_ino",
                found: 4,
                wanted: 8
            })
        ));
    }
}
