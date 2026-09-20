//! The original, inventoried before it is converted.
//!
//! One function, [`Original::of`], which reads the zip, hands it to the file
//! for its format, and adds what all three share: the linked content its
//! relationships name, and the macros `alo-opening` already found. **Every part
//! an inventory needs is read in full or the inventory does not complete**;
//! there is no partial [`Original`].

use std::collections::{BTreeMap, BTreeSet};

use alo_opening::Macros;

use crate::carried::{Field, FontName, Linked};
use crate::conversion::Conversion;
use crate::inventory::linked::{NotLinked, linked};
use crate::inventory::{excel, pages, powerpoint, word};
use crate::xml::NotXml;
use crate::zip::{NotRead, Zipped};

/// The most distinct font families an inventory keeps. A document setting text
/// in more is not one whose losses could be said as a list anybody reads.
pub const MOST_FONTS: usize = 256;

/// Why an original could not be inventoried.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotInventoried {
    /// A part could not be read out of the zip.
    #[error(transparent)]
    Zip(#[from] NotRead),
    /// A part was not XML.
    #[error(transparent)]
    Xml(#[from] NotXml),
    /// A part the format cannot be without is not there.
    #[error("the document has no {0}")]
    Missing(&'static str),
    /// More fonts than are kept.
    #[error("the document sets text in more fonts than are listed")]
    TooManyFonts,
    /// The format's parts are known and nothing in one has ever been read on a
    /// machine this repository gates on, so there is nothing to inventory by.
    #[error("no {0} has been inventoried on this machine")]
    NotMeasured(&'static str),
}

impl From<NotLinked> for NotInventoried {
    fn from(not: NotLinked) -> Self {
        match not {
            NotLinked::Zip(zip) => Self::Zip(zip),
            NotLinked::Xml(xml) => Self::Xml(xml),
        }
    }
}

/// What an original holds that a copy might not carry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Original {
    /// Every family text is set in, by its compared name.
    fonts: BTreeMap<String, FontName>,
    /// Every field whose value depends on when or where it is open.
    fields: BTreeSet<Field>,
    /// Every kind of content taken from elsewhere.
    linked: BTreeSet<Linked>,
    /// Whether it has comments.
    comments: bool,
    /// Whether it has tracked changes.
    tracked_changes: bool,
    /// Whether it carries macros.
    macros: bool,
}

impl Original {
    /// Inventory an original of this conversion's kind.
    ///
    /// # Errors
    /// [`NotInventoried`], and then nothing about the document is claimed.
    pub fn of(
        bytes: &[u8],
        conversion: Conversion,
        macros: Macros,
    ) -> Result<Self, NotInventoried> {
        let mut zipped = Zipped::of(bytes)?;
        let mut original = Self {
            macros: macros == Macros::Inside,
            ..Self::default()
        };
        match conversion {
            Conversion::WordDocument => word::inventory(&mut zipped, &mut original)?,
            Conversion::ExcelWorkbook => excel::inventory(&mut zipped, &mut original)?,
            Conversion::PowerPointPresentation => {
                powerpoint::inventory(&mut zipped, &mut original)?;
            }
            Conversion::PagesDocument => pages::inventory(&mut zipped, &mut original)?,
        }
        original.linked = linked(&mut zipped)?;
        Ok(original)
    }

    /// Text is set in this family.
    ///
    /// # Errors
    /// [`NotInventoried::TooManyFonts`].
    pub(crate) fn sets_text_in(&mut self, family: &str) -> Result<(), NotInventoried> {
        let Some(font) = FontName::from_document(family) else {
            return Ok(());
        };
        let compared = font.compared();
        if compared.is_empty() || self.fonts.contains_key(&compared) {
            return Ok(());
        }
        if self.fonts.len() == MOST_FONTS {
            return Err(NotInventoried::TooManyFonts);
        }
        self.fonts.insert(compared, font);
        Ok(())
    }

    /// It has a field of this kind.
    pub(crate) fn has_field(&mut self, field: Field) {
        self.fields.insert(field);
    }

    /// It has comments.
    pub(crate) fn has_comments(&mut self) {
        self.comments = true;
    }

    /// It has tracked changes.
    pub(crate) fn has_tracked_changes(&mut self) {
        self.tracked_changes = true;
    }

    /// Every family text is set in.
    pub fn fonts(&self) -> impl Iterator<Item = &FontName> {
        self.fonts.values()
    }

    /// Every field kind whose value depends on when or where it is open.
    #[must_use]
    pub fn fields(&self) -> &BTreeSet<Field> {
        &self.fields
    }

    /// Every kind of content taken from elsewhere.
    #[must_use]
    pub fn linked(&self) -> &BTreeSet<Linked> {
        &self.linked
    }

    /// Whether it has comments.
    #[must_use]
    pub fn comments(&self) -> bool {
        self.comments
    }

    /// Whether it has tracked changes.
    #[must_use]
    pub fn tracked_changes(&self) -> bool {
        self.tracked_changes
    }

    /// Whether it carries macros.
    #[must_use]
    pub fn macros(&self) -> bool {
        self.macros
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_zip, the_document};

    /// The families of some inventory, as names.
    fn families(original: &Original) -> Vec<&str> {
        original.fonts().map(FontName::as_str).collect()
    }

    /// **Each of the owner's three documents is inventoried as what it holds**
    /// — `crates/alo-converting/tests/documents/README.md` says what that is,
    /// and this is the same list read off the files.
    #[test]
    fn each_real_document_is_inventoried_as_what_it_holds() {
        let word = Original::of(
            &the_document("sample.docx"),
            Conversion::WordDocument,
            Macros::NoneSeen,
        )
        .unwrap();
        assert_eq!(families(&word), ["Garamond"]);
        assert_eq!(word.fields(), &BTreeSet::from([Field::Date]));
        assert_eq!(word.linked(), &BTreeSet::from([Linked::Picture]));
        assert!(word.comments());
        assert!(!word.tracked_changes());
        assert!(!word.macros());

        let excel = Original::of(
            &the_document("sample.xlsx"),
            Conversion::ExcelWorkbook,
            Macros::NoneSeen,
        )
        .unwrap();
        assert_eq!(families(&excel), ["Garamond"]);
        assert_eq!(excel.fields(), &BTreeSet::from([Field::TheCurrentMoment]));
        assert!(excel.linked().is_empty());
        assert!(excel.comments());

        let powerpoint = Original::of(
            &the_document("sample.pptx"),
            Conversion::PowerPointPresentation,
            Macros::NoneSeen,
        )
        .unwrap();
        assert_eq!(families(&powerpoint), ["Garamond"]);
        assert!(powerpoint.fields().is_empty());
        assert!(powerpoint.linked().is_empty());
        assert!(powerpoint.comments());
    }

    /// **A document without the part its format cannot be without is not
    /// inventoried**, rather than inventoried as holding nothing.
    #[test]
    fn a_document_missing_its_main_part_is_not_inventoried() {
        let bytes = a_zip(&[("[Content_Types].xml", b"<Types/>")]);
        for conversion in Conversion::EVERY {
            assert!(
                matches!(
                    Original::of(&bytes, conversion, Macros::NoneSeen),
                    Err(NotInventoried::Missing(_))
                ),
                "{conversion:?}"
            );
        }
    }

    /// Macros found by deciding are carried into the inventory.
    #[test]
    fn macros_found_by_deciding_are_inventoried() {
        let original = Original::of(
            &the_document("sample.docx"),
            Conversion::WordDocument,
            Macros::Inside,
        )
        .unwrap();
        assert!(original.macros());
    }

    /// **More families than are kept end the inventory**; a family said twice
    /// is kept once.
    #[test]
    fn more_families_than_are_kept_end_the_inventory() {
        let mut original = Original::default();
        original.sets_text_in("Garamond").unwrap();
        original.sets_text_in("garamond").unwrap();
        assert_eq!(original.fonts().count(), 1);
        for number in 1..MOST_FONTS {
            original.sets_text_in(&format!("Family {number}")).unwrap();
        }
        assert_eq!(
            original.sets_text_in("One Too Many"),
            Err(NotInventoried::TooManyFonts)
        );
    }
}
