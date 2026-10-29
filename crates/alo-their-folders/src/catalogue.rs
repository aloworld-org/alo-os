//! **What every Linux desktop calls these folders in a language**: the
//! translations `xdg-user-dirs` ships, read from the machine rather than
//! written here.
//!
//! [ADR 0137](../../../docs/decisions/0137-home-folder-names-come-from-xdg-user-dirs.md):
//! a German person gets `Dokumente` because that is what `xdg-user-dirs` — the
//! package every Linux desktop uses to name and record these folders — calls
//! *Documents* in German, in a catalogue kept by that project's translators
//! for some seventy languages. A list of our own would be a second answer that
//! disagrees with Dolphin, GNOME Files and every other program on the first
//! machine where the two differ.
//!
//! # The file
//!
//! A GNU gettext catalogue, `<root>/<language>/LC_MESSAGES/xdg-user-dirs.mo`,
//! whose keys are the English names in `/etc/xdg/user-dirs.defaults` —
//! `Documents`, `Downloads`, `Pictures` and five more — and see
//! [`crate::Folder::xdg_name`]. The format is GNU's and is read here in
//! full: a magic number, a count, and two tables of lengths and offsets — one
//! for the English, one for the translation at the same position.
//!
//! # A language it does not have
//!
//! Is a language whose folders `xdg-user-dirs` itself leaves in English, and
//! so does this: the key is the name. That is the upstream answer on every
//! Linux desktop, not a fallback of ours. Of the 24 official EU languages the
//! catalogue the image installs has 22 (English is its source); **Maltese is
//! the one without**, measured on 2026-10-28 from Fedora 42's `xdg-user-dirs`
//! 0.18, and ADR 0137 records that.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use alo_strings::Language;

/// Where a machine keeps its gettext catalogues.
pub const THE_LOCALES: &str = "/usr/share/locale";

/// The catalogue's name under each language.
pub const THE_DOMAIN: &str = "xdg-user-dirs";

/// **One language's names for the folders.**
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Catalogue {
    /// Each English key, and its translation.
    names: BTreeMap<String, String>,
}

/// Why a file was not a catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotACatalogue {
    /// Shorter than its own header says.
    TooShort,
    /// Not a GNU catalogue at all.
    NotTheMagicNumber,
    /// A string that is not UTF-8.
    NotText,
}

/// The two byte orders a catalogue may be written in.
#[derive(Clone, Copy)]
enum Order {
    /// Least significant byte first, as every machine this runs on writes.
    Little,
    /// Most significant byte first.
    Big,
}

/// The `n`th 32-bit number at `at`.
fn number(bytes: &[u8], at: usize, order: Order) -> Result<usize, NotACatalogue> {
    let four: [u8; 4] = bytes
        .get(at..at.checked_add(4).ok_or(NotACatalogue::TooShort)?)
        .and_then(|it| it.try_into().ok())
        .ok_or(NotACatalogue::TooShort)?;
    let value = match order {
        Order::Little => u32::from_le_bytes(four),
        Order::Big => u32::from_be_bytes(four),
    };
    usize::try_from(value).map_err(|_| NotACatalogue::TooShort)
}

/// The string the table at `table` names as its `index`th.
fn string(bytes: &[u8], table: usize, index: usize, order: Order) -> Result<&str, NotACatalogue> {
    let entry = index
        .checked_mul(8)
        .and_then(|it| it.checked_add(table))
        .ok_or(NotACatalogue::TooShort)?;
    let length = number(bytes, entry, order)?;
    let offset = number(
        bytes,
        entry.checked_add(4).ok_or(NotACatalogue::TooShort)?,
        order,
    )?;
    let end = offset.checked_add(length).ok_or(NotACatalogue::TooShort)?;
    let raw = bytes.get(offset..end).ok_or(NotACatalogue::TooShort)?;
    std::str::from_utf8(raw).map_err(|_| NotACatalogue::NotText)
}

impl Catalogue {
    /// **A catalogue, read from its bytes.**
    ///
    /// # Errors
    /// [`NotACatalogue`] for anything that is not a whole GNU catalogue.
    pub fn read(bytes: &[u8]) -> Result<Self, NotACatalogue> {
        let order = match bytes.get(..4) {
            Some([0xde, 0x12, 0x04, 0x95]) => Order::Little,
            Some([0x95, 0x04, 0x12, 0xde]) => Order::Big,
            Some(_) => return Err(NotACatalogue::NotTheMagicNumber),
            None => return Err(NotACatalogue::TooShort),
        };
        let how_many = number(bytes, 8, order)?;
        let originals = number(bytes, 12, order)?;
        let translations = number(bytes, 16, order)?;
        let mut names = BTreeMap::new();
        for index in 0..how_many {
            let key = string(bytes, originals, index, order)?;
            let name = string(bytes, translations, index, order)?;
            // The header (an empty key), a key with a context and a plural
            // form are skipped: none of them names a folder.
            if key.is_empty() || key.contains(['\u{4}', '\0']) || name.contains('\0') {
                continue;
            }
            names.insert(key.to_owned(), name.to_owned());
        }
        Ok(Self { names })
    }

    /// **The catalogue for `language` under `root`**, or [`None`] where the
    /// machine has none for it — tried as the language is written (`pt_BR`),
    /// then as its broader form (`pt`).
    #[must_use]
    pub fn for_language(root: &Path, language: &Language) -> Option<Self> {
        let written = language.tag().replace('-', "_");
        let broader = written.split('_').next().unwrap_or_default().to_owned();
        [written, broader]
            .iter()
            .map(|it| the_file(root, it))
            .find_map(|at| std::fs::read(at).ok())
            .and_then(|bytes| Self::read(&bytes).ok())
    }

    /// What this language calls the folder whose English key is `key`.
    #[must_use]
    pub fn name_of(&self, key: &str) -> Option<&str> {
        self.names
            .get(key)
            .map(String::as_str)
            .filter(|it| !it.is_empty())
    }
}

/// Where one language's catalogue is.
#[must_use]
pub fn the_file(root: &Path, locale: &str) -> PathBuf {
    root.join(locale)
        .join("LC_MESSAGES")
        .join(format!("{THE_DOMAIN}.mo"))
}

/// A catalogue's bytes, as GNU `msgfmt` writes one, for a test.
#[cfg(test)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "a test's catalogue is a few hundred bytes"
)]
pub(crate) fn written(pairs: &[(&str, &str)]) -> Vec<u8> {
    let count = pairs.len() as u32;
    let originals = 28_u32;
    let translations = originals + 8 * count;
    let mut strings_at = translations + 8 * count;
    let mut head = Vec::new();
    for value in [0x9504_12de_u32, 0, count, originals, translations, 0, 0] {
        head.extend(value.to_le_bytes());
    }
    let mut tables = (Vec::new(), Vec::new());
    let mut text = Vec::new();
    for column in 0..2 {
        for (key, name) in pairs {
            let it = if column == 0 { key } else { name };
            let table = if column == 0 {
                &mut tables.0
            } else {
                &mut tables.1
            };
            table.extend((it.len() as u32).to_le_bytes());
            table.extend(strings_at.to_le_bytes());
            text.extend(it.as_bytes());
            text.push(0);
            strings_at += it.len() as u32 + 1;
        }
    }
    [head, tables.0, tables.1, text].concat()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A catalogue reads back what was written**, and the header is not a
    /// folder.
    #[test]
    fn a_catalogue_reads_back_what_was_written() {
        let bytes = written(&[
            ("", "Content-Type: text/plain\n"),
            ("Documents", "Dokumente"),
        ]);
        let catalogue = Catalogue::read(&bytes).unwrap();
        assert_eq!(catalogue.name_of("Documents"), Some("Dokumente"));
        assert_eq!(catalogue.name_of(""), None);
        assert_eq!(catalogue.name_of("Pictures"), None);
    }

    /// **Something that is not a catalogue is refused**, never half-read.
    #[test]
    fn something_that_is_not_a_catalogue_is_refused() {
        assert_eq!(Catalogue::read(b""), Err(NotACatalogue::TooShort));
        assert_eq!(
            Catalogue::read(b"not a catalogue at all, but long"),
            Err(NotACatalogue::NotTheMagicNumber)
        );
        let mut cut = written(&[("Documents", "Dokumente")]);
        cut.truncate(40);
        assert_eq!(Catalogue::read(&cut), Err(NotACatalogue::TooShort));
    }

    /// **A language is found as written, then as its broader form**, and a
    /// language the machine has no catalogue for is none.
    #[test]
    fn a_language_is_found_as_written_then_broader() {
        let root = std::env::temp_dir().join(format!("alo-catalogue-{}", std::process::id()));
        let at = the_file(&root, "pt");
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(&at, written(&[("Documents", "Documentos")])).unwrap();
        let brazilian = Language::written("pt-BR").unwrap();
        assert_eq!(
            Catalogue::for_language(&root, &brazilian)
                .unwrap()
                .name_of("Documents"),
            Some("Documentos")
        );
        let maltese = Language::written("mt").unwrap();
        assert_eq!(Catalogue::for_language(&root, &maltese), None);
        let _ = std::fs::remove_dir_all(&root);
    }
}
