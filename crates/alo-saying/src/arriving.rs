//! One translation, as whoever wrote it wrote it.
//!
//! The file is `docs/contracts/translations.md`, which is where the shape below
//! is described for the people who write one. It is TOML for the reason
//! `docs/contracts/machine-description.md` is: the record and the protocol are
//! JSON because a program writes them and a program reads them, and this one is
//! typed by a person who needs somewhere to leave a note about why a sentence
//! is worded the way it is.
//!
//! ```toml
//! format = 1
//! language = "de"
//!
//! [says]
//! "files.gone" = "Es ist nicht mehr da"
//! ```
//!
//! # The shape is this crate's, not `alo-strings`'
//!
//! `alo_strings::Translation` already deserialises, and using its serde shape
//! as the file format would have tied a contract other people write files
//! against to the field names of a type inside another crate. So the file is
//! read into the struct below and turned into a `Translation`, and renaming a
//! field in `alo-strings` stays a rename.
//!
//! # `format` is answered before anything else in the file
//!
//! A file written for a later alo OS is refused **as one**, rather than as
//! whichever of its keys this version happened not to know — which is the rule
//! `docs/contracts/machine-description.md` and `docs/contracts/record-file.md`
//! both state, arriving at the third file a person types. That costs a second
//! parse of the same text and is worth it: without it, `deny_unknown_fields`
//! would report a future addition as a typo.

use std::collections::BTreeMap;

use alo_strings::{Key, Language, Translation};
use serde::Deserialize;

use crate::failing::NotSpoken;

/// The shape of a translation file this alo OS reads.
pub const THE_FORMAT: u64 = 1;

/// What is read out of the file first, and on its own.
#[derive(Debug, Deserialize)]
struct TheFormat {
    /// Which shape the file is in.
    format: u64,
}

/// One translation as it is written down.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AsWritten {
    /// Which shape the file is in. Read again here so that the whole file has
    /// one shape rather than a checked half and an unchecked one.
    #[expect(
        dead_code,
        reason = "read by TheFormat; named here so deny_unknown_fields does not call it a typo"
    )]
    format: u64,
    /// Which language it is written for.
    language: String,
    /// What it says, key by key. A file with nothing in it yet is a language
    /// somebody has started rather than a mistake — a translation arrives a few
    /// hundred strings at a time, which `alo-strings` settled.
    #[serde(default)]
    says: BTreeMap<String, String>,
}

/// One translation, read out of what a file held.
///
/// Split from the disk on purpose, as `alo-keeping`'s reading is: every shape a
/// translation file can be wrong in is testable without one, and the disk tests
/// are then about the disk.
///
/// # Errors
///
/// [`NotSpoken::NotWritten`] for something that is not TOML or is missing a
/// field, [`NotSpoken::FromANewerAlo`] for a shape this version does not read,
/// [`NotSpoken::NotALanguage`] and [`NotSpoken::NotAKey`] for the two things in
/// it that have to be more than text.
pub(crate) fn as_written(file: &str, text: &str) -> Result<Translation, NotSpoken> {
    let shape: TheFormat = toml::from_str(text).map_err(|why| NotSpoken::NotWritten {
        file: file.to_owned(),
        why: one_line(&why.to_string()),
    })?;
    if shape.format != THE_FORMAT {
        return Err(NotSpoken::FromANewerAlo {
            file: file.to_owned(),
            format: shape.format,
            reads: THE_FORMAT,
        });
    }

    let written: AsWritten = toml::from_str(text).map_err(|why| NotSpoken::NotWritten {
        file: file.to_owned(),
        why: one_line(&why.to_string()),
    })?;

    let language = Language::written(&written.language).map_err(|why| NotSpoken::NotALanguage {
        file: file.to_owned(),
        tag: written.language.clone(),
        why: why.to_string(),
    })?;

    let mut translation = Translation::into_language(language);
    for (named, text) in written.says {
        let key = Key::named(&named).map_err(|why| NotSpoken::NotAKey {
            file: file.to_owned(),
            named: named.clone(),
            why: why.to_string(),
        })?;
        translation = translation.says(key, text);
    }
    Ok(translation)
}

/// What a parser said, on one line.
///
/// `toml` reports an error as several lines with the offending line drawn
/// underneath, which is the right thing in a terminal and the wrong thing in a
/// service log where one entry is one line. The text is kept; only the shape of
/// it changes.
fn one_line(why: &str) -> String {
    why.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A file as `docs/contracts/translations.md` describes one.
    const GERMAN: &str = r#"
format = 1
language = "de"

[says]
"files.gone" = "Es ist nicht mehr da"
"files.too-big" = "{path} ist {bytes} Bytes groß"
"#;

    #[test]
    fn a_translation_is_read_out_of_what_a_file_holds() {
        let translation = as_written("de.toml", GERMAN).unwrap();
        assert_eq!(translation.language().tag(), "de");
        assert_eq!(translation.how_many(), 2);
        assert_eq!(
            translation.texts().next(),
            Some((&Key::named("files.gone").unwrap(), "Es ist nicht mehr da"))
        );
    }

    /// **A language somebody started is not a mistake.** A file with no
    /// sentences in it yet loads, because a translation arrives a few hundred
    /// strings at a time and refusing the first commit is how a translator
    /// stops.
    #[test]
    fn a_translation_with_nothing_in_it_yet_is_read() {
        let translation = as_written("mt.toml", "format = 1\nlanguage = \"mt\"\n").unwrap();
        assert_eq!(translation.language().tag(), "mt");
        assert!(translation.is_empty());
    }

    /// A tag is normalised when it is made, so one language cannot arrive as
    /// two files' worth of settings.
    #[test]
    fn a_language_is_read_the_way_the_rest_of_the_world_writes_it() {
        let translation = as_written("pt.toml", "format = 1\nlanguage = \"PT-br\"\n").unwrap();
        assert_eq!(translation.language().tag(), "pt-BR");
    }

    /// **A shape this alo OS does not read is refused as a shape**, not as
    /// whichever key it happened not to know — so the sentence says what is
    /// really wrong.
    #[test]
    fn a_file_from_a_newer_alo_is_refused_as_one() {
        let refused = as_written(
            "de.toml",
            "format = 2\nlanguage = \"de\"\nplurals = \"whatever they add\"\n",
        )
        .unwrap_err();
        assert!(matches!(
            refused,
            NotSpoken::FromANewerAlo {
                format: 2,
                reads: 1,
                ..
            }
        ));
        assert!(refused.to_string().contains("later one"), "{refused}");
    }

    /// A key this alo OS does not have is **not** refused here: it is an
    /// ordinary line that the load leaves out, which [`crate::loading`] does.
    /// What is refused is a name that could never be a key at all.
    #[test]
    fn a_line_that_could_never_be_a_string_is_refused() {
        let refused = as_written(
            "de.toml",
            "format = 1\nlanguage = \"de\"\n[says]\ngone = \"weg\"\n",
        )
        .unwrap_err();
        assert!(matches!(refused, NotSpoken::NotAKey { .. }));
        assert!(refused.to_string().contains("gone"), "{refused}");
    }

    /// A language that is not one names the tag, because that is the character
    /// somebody has to go and change.
    #[test]
    fn a_tag_that_is_not_a_language_is_refused_and_quoted() {
        let refused = as_written("de.toml", "format = 1\nlanguage = \"deutsch\"\n").unwrap_err();
        assert!(matches!(refused, NotSpoken::NotALanguage { .. }));
        assert!(refused.to_string().contains("deutsch"), "{refused}");
    }

    /// **A field nobody declared is a typo, and a typo is refused.** A file
    /// whose strings are under `strings` rather than `says` would otherwise
    /// load as a language with nothing in it, and look complete.
    #[test]
    fn a_field_this_alo_does_not_know_is_refused() {
        let refused = as_written(
            "de.toml",
            "format = 1\nlanguage = \"de\"\n[strings]\n\"files.gone\" = \"weg\"\n",
        )
        .unwrap_err();
        assert!(matches!(refused, NotSpoken::NotWritten { .. }));
    }

    /// A file that is not TOML says so, in the words of whatever tried to read
    /// it, on one line.
    #[test]
    fn what_is_not_a_translation_file_says_so_on_one_line() {
        let refused = as_written("de.toml", "this is not a translation").unwrap_err();
        assert!(matches!(refused, NotSpoken::NotWritten { .. }));
        let said = refused.to_string();
        assert!(!said.contains('\n'), "{said}");
        assert!(said.contains("de.toml"), "{said}");
    }

    /// A file with no `format` in it is missing the one thing that says what it
    /// is, and there is no default: item 23's rule about a catalogue entry, at
    /// the top of a translation.
    #[test]
    fn a_file_that_does_not_say_what_shape_it_is_in_is_refused() {
        let refused = as_written("de.toml", "language = \"de\"\n").unwrap_err();
        assert!(matches!(refused, NotSpoken::NotWritten { .. }));
    }

    /// Several lines of a parser's complaint become one, and nothing in them is
    /// thrown away.
    #[test]
    fn a_complaint_over_several_lines_becomes_one_line() {
        assert_eq!(
            one_line("TOML parse error at line 4\n  |\n4 | gone\n  | ^\n"),
            "TOML parse error at line 4 | 4 | gone | ^"
        );
    }
}
