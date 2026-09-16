//! The two inventories, compared into what the copy carried.
//!
//! ADR 0039 §4's closed set, in one function. A PDF's page shows no comments,
//! no tracked changes, no fields that update, no macros and nothing fetched, so
//! each of those the original holds is not carried; a font is not carried when
//! the copy contains nothing of its family. **Reached only with both
//! inventories in hand**, which is the only way [`Carried::Everything`] is ever
//! said.

use std::collections::BTreeSet;

use crate::carried::{Carried, NotCarried};
use crate::inventory::copy::Copy;
use crate::inventory::original::Original;

/// What a copy carried from its original.
#[must_use]
pub fn carried(original: &Original, copy: &Copy) -> Carried {
    let mut not = BTreeSet::new();
    for font in original.fonts() {
        if !copy.contains_family(&font.compared()) {
            not.insert(NotCarried::FontSubstituted(font.clone()));
        }
    }
    not.extend(
        original
            .fields()
            .iter()
            .copied()
            .map(NotCarried::FieldFixed),
    );
    not.extend(
        original
            .linked()
            .iter()
            .copied()
            .map(NotCarried::LinkedNotFetched),
    );
    if original.macros() {
        not.insert(NotCarried::Macros);
    }
    if original.comments() {
        not.insert(NotCarried::Comments);
    }
    if original.tracked_changes() {
        not.insert(NotCarried::TrackedChanges);
    }
    Carried::of(not)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::carried::{Field, FontName, Linked};

    /// **A copy that contains the original's family carried it; one that does
    /// not, did not** — and an original with nothing to lose lost nothing.
    #[test]
    fn a_font_is_carried_when_the_copy_contains_its_family() {
        let mut original = Original::default();
        original.sets_text_in("Garamond").unwrap();
        original.sets_text_in("Liberation Serif").unwrap();
        let copy = Copy::of(
            b"%PDF-1.7 /BaseFont/AAAAAA+LiberationSerif /BaseFont/BAAAAA+NotoSerif-Regular",
        )
        .unwrap();
        assert_eq!(
            carried(&original, &copy),
            Carried::of(BTreeSet::from([NotCarried::FontSubstituted(
                FontName::from_document("Garamond").unwrap()
            )]))
        );

        let mut plain = Original::default();
        plain.sets_text_in("Liberation Serif").unwrap();
        assert_eq!(carried(&plain, &copy), Carried::Everything);
    }

    /// Everything a page cannot show is not carried, each by name.
    #[test]
    fn what_a_page_cannot_show_is_not_carried() {
        let mut original = Original::default();
        original.has_field(Field::Time);
        original.has_comments();
        original.has_tracked_changes();
        let copy = Copy::of(b"%PDF-1.7").unwrap();
        let carried = carried(&original, &copy);
        assert_eq!(
            carried.not_carried().cloned().collect::<Vec<_>>(),
            [
                NotCarried::FieldFixed(Field::Time),
                NotCarried::Comments,
                NotCarried::TrackedChanges
            ]
        );
        assert!(
            !carried
                .not_carried()
                .any(|not| *not == NotCarried::LinkedNotFetched(Linked::Picture))
        );
    }
}
