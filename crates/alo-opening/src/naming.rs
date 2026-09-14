//! What a file's name claims it is, and whether its bytes agree.
//!
//! An extension is a claim made by whoever named the file, and this crate
//! treats it as exactly that: **it is never used to decide what a file is**.
//! It is read afterwards, once the bytes have decided, for one purpose — to
//! notice when the two disagree, because *a file called `invoice.pdf` that is a
//! program* is the finding a person most needs and a machine that quietly
//! opened it as whatever it really is would have hidden.
//!
//! # Agreeing is not the same as being identical
//!
//! A claim accepts what that name honestly covers: `.txt` covers text in any
//! character set, `.zip` covers every document stored as a zip, `.docx` covers
//! the same document locked with a password (which Office saves under the same
//! name), and a name covers a damaged file of the same storage — a Word
//! document cut short cannot be seen to be a Word document, and calling it
//! *not what its name says* would be a second, false finding on top of the true
//! one. A file with nothing in it agrees with every name: there is nothing in
//! it to disagree.
//!
//! A name this crate has no claim for — no extension, or one it does not know —
//! claims nothing, and nothing can disagree with it.

use std::ffi::OsStr;
use std::path::Path;

use alo_strings::{Filling, Said, Strings};

use crate::appears::{Appears, Container};
use crate::kind::Kind;
use crate::words;

/// What a name says a file is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Named {
    /// One of the kinds this crate recognises.
    A(Kind),
    /// A program.
    AProgram,
}

/// Every extension this crate reads a claim from, and the claim.
const CLAIMS: [(&str, Named); 29] = [
    ("pdf", Named::A(Kind::Pdf)),
    ("docx", Named::A(Kind::WordDocument)),
    ("docm", Named::A(Kind::WordDocument)),
    ("xlsx", Named::A(Kind::ExcelWorkbook)),
    ("xlsm", Named::A(Kind::ExcelWorkbook)),
    ("pptx", Named::A(Kind::PowerPointPresentation)),
    ("pptm", Named::A(Kind::PowerPointPresentation)),
    ("odt", Named::A(Kind::OpenDocumentText)),
    ("ods", Named::A(Kind::OpenDocumentSpreadsheet)),
    ("odp", Named::A(Kind::OpenDocumentPresentation)),
    ("doc", Named::A(Kind::OlderWordDocument)),
    ("xls", Named::A(Kind::OlderExcelWorkbook)),
    ("ppt", Named::A(Kind::OlderPowerPointPresentation)),
    ("rtf", Named::A(Kind::RichText)),
    ("txt", Named::A(Kind::Text)),
    ("csv", Named::A(Kind::Text)),
    ("tsv", Named::A(Kind::Text)),
    ("md", Named::A(Kind::Text)),
    ("log", Named::A(Kind::Text)),
    ("png", Named::A(Kind::PngImage)),
    ("jpg", Named::A(Kind::JpegImage)),
    ("jpeg", Named::A(Kind::JpegImage)),
    ("gif", Named::A(Kind::GifImage)),
    ("webp", Named::A(Kind::WebpImage)),
    ("zip", Named::A(Kind::ZipArchive)),
    ("exe", Named::AProgram),
    ("dll", Named::AProgram),
    ("scr", Named::AProgram),
    ("appimage", Named::AProgram),
];

impl Named {
    /// What this name claims, or [`None`] when it claims nothing this crate
    /// reads.
    ///
    /// The last extension is the claim, and only it: `invoice.pdf.exe` claims
    /// to be a program, which is the whole of what that name is designed to
    /// hide.
    #[must_use]
    pub fn from_the_name(name: &OsStr) -> Option<Self> {
        let extension = Path::new(name).extension()?.to_str()?.to_ascii_lowercase();
        CLAIMS
            .iter()
            .find(|(claimed, _)| *claimed == extension)
            .map(|(_, named)| *named)
    }

    /// Whether a file whose bytes say this agrees with this name.
    #[must_use]
    pub fn agrees_with(self, appears: Appears) -> bool {
        match (self, appears) {
            (_, Appears::Nothing) | (Self::AProgram, Appears::AProgram) => true,
            (Self::A(Kind::Text), Appears::A(Kind::Text | Kind::TextInAnOlderCharacterSet)) => true,
            (Self::A(Kind::ZipArchive), Appears::A(kind)) => kind.is_stored_as_a_zip(),
            (Self::A(claimed), Appears::A(kind)) => claimed == kind,
            (Self::A(claimed), Appears::APasswordProtectedDocument) => claimed.is_current_office(),
            (Self::A(claimed), Appears::Damaged(Container::Pdf)) => claimed == Kind::Pdf,
            (Self::A(claimed), Appears::Damaged(Container::Compressed)) => {
                claimed.is_stored_as_a_zip()
            }
            (Self::A(claimed), Appears::Damaged(Container::OlderOffice)) => {
                claimed.is_stored_as_older_office() || claimed.is_current_office()
            }
            (Self::A(_), Appears::AProgram | Appears::Unrecognised) | (Self::AProgram, _) => false,
        }
    }

    /// What this name claims, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let word = match self {
            Self::A(kind) => kind.word(),
            Self::AProgram => words::A_PROGRAM,
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The claim a name makes.
    fn claim(name: &str) -> Option<Named> {
        Named::from_the_name(OsStr::new(name))
    }

    /// **The last extension is the claim, whatever its case**, and a name
    /// with none, or one nothing reads, claims nothing.
    #[test]
    fn the_last_extension_is_the_claim() {
        assert_eq!(claim("Invoice.PDF"), Some(Named::A(Kind::Pdf)));
        assert_eq!(claim("invoice.pdf.exe"), Some(Named::AProgram));
        assert_eq!(claim("report.docx"), Some(Named::A(Kind::WordDocument)));
        assert_eq!(claim("README"), None);
        assert_eq!(claim(".bashrc"), None);
        assert_eq!(claim("holiday.heic"), None);
    }

    /// Every claim is written once.
    #[test]
    fn every_extension_is_claimed_once() {
        let mut named: Vec<&str> = CLAIMS.iter().map(|(extension, _)| *extension).collect();
        named.sort_unstable();
        named.dedup();
        assert_eq!(named.len(), CLAIMS.len());
        for (extension, _) in CLAIMS {
            assert_eq!(extension, extension.to_ascii_lowercase());
        }
    }

    /// **A name agrees with what it honestly covers**, and with nothing else.
    #[test]
    fn a_name_agrees_with_what_it_honestly_covers() {
        let text = Named::A(Kind::Text);
        assert!(text.agrees_with(Appears::A(Kind::TextInAnOlderCharacterSet)));
        assert!(!text.agrees_with(Appears::A(Kind::Pdf)));

        let zip = Named::A(Kind::ZipArchive);
        assert!(zip.agrees_with(Appears::A(Kind::WordDocument)));
        assert!(zip.agrees_with(Appears::Damaged(Container::Compressed)));
        assert!(!zip.agrees_with(Appears::A(Kind::OlderWordDocument)));

        let docx = Named::A(Kind::WordDocument);
        assert!(docx.agrees_with(Appears::APasswordProtectedDocument));
        assert!(docx.agrees_with(Appears::Damaged(Container::Compressed)));
        assert!(docx.agrees_with(Appears::Damaged(Container::OlderOffice)));
        assert!(!docx.agrees_with(Appears::A(Kind::ZipArchive)));
        assert!(!docx.agrees_with(Appears::A(Kind::ExcelWorkbook)));
        assert!(!docx.agrees_with(Appears::Damaged(Container::Pdf)));

        let pdf = Named::A(Kind::Pdf);
        assert!(pdf.agrees_with(Appears::Damaged(Container::Pdf)));
        assert!(!pdf.agrees_with(Appears::AProgram));
        assert!(!pdf.agrees_with(Appears::Unrecognised));
        assert!(!pdf.agrees_with(Appears::APasswordProtectedDocument));

        let doc = Named::A(Kind::OlderWordDocument);
        assert!(doc.agrees_with(Appears::Damaged(Container::OlderOffice)));
        assert!(!doc.agrees_with(Appears::APasswordProtectedDocument));
    }

    /// **A program's name agrees only with a program**, and a program agrees
    /// with no other name.
    #[test]
    fn a_program_and_a_document_never_agree() {
        assert!(Named::AProgram.agrees_with(Appears::AProgram));
        for kind in Kind::EVERY {
            assert!(!Named::AProgram.agrees_with(Appears::A(kind)), "{kind:?}");
            assert!(!Named::A(kind).agrees_with(Appears::AProgram), "{kind:?}");
        }
    }

    /// **An empty file agrees with every name**, because there is nothing in it
    /// to disagree.
    #[test]
    fn an_empty_file_agrees_with_every_name() {
        assert!(Named::AProgram.agrees_with(Appears::Nothing));
        for kind in Kind::EVERY {
            assert!(Named::A(kind).agrees_with(Appears::Nothing), "{kind:?}");
        }
    }
}
