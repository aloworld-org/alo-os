//! **What alo's own copy of a code editor is set to, so that what alo types is
//! what lands** —
//! [ADR 0153](../../../docs/decisions/0153-alo-types-by-keys-in-its-own-copy-and-reads-the-text-back.md),
//! the owner's *"Keys, in alo's own copy"* of 2026-11-13.
//!
//! VSCodium rewrites what is typed into it: it indents after a new line, closes
//! a bracket on its own, and turns a tab into spaces. Measured once by hand on
//! 2026-11-13 (`docs/autonomy/updates/how-alo-can-type-into-an-application.md`),
//! the file it saved was not the text typed — until these were off, when it
//! matched to the byte, twice.
//!
//! **Only in alo's own copy.** The file named here is inside the profile
//! [`crate::alos_copy()`] gives alo's copy, and nothing in this crate names the
//! person's own profile. This crate decides the text; `alo-desktop` writes it,
//! changing only these keys and keeping anything else in the file.

use std::path::PathBuf;

use crate::alos_copy::AlosCopy;

/// The applications whose copy is set this way: the two editors built on the
/// same editor component, whose settings are the same words.
pub const WITH_AN_EDITOR_THAT_REWRITES: [&str; 2] =
    ["com.vscodium.codium", "com.visualstudio.code"];

/// Each setting, by VSCodium's own name, and the value it is given, written as
/// JSON. ADR 0153's table says what each would otherwise do to typed text.
pub const TYPED_AS_IT_IS: [(&str, &str); 19] = [
    ("editor.autoIndent", r#""none""#),
    ("editor.autoClosingBrackets", r#""never""#),
    ("editor.autoClosingQuotes", r#""never""#),
    ("editor.autoClosingOvertype", r#""never""#),
    ("editor.autoClosingDelete", r#""never""#),
    ("editor.autoSurround", r#""never""#),
    (
        "editor.quickSuggestions",
        r#"{"other": "off", "comments": "off", "strings": "off"}"#,
    ),
    ("editor.suggestOnTriggerCharacters", "false"),
    ("editor.acceptSuggestionOnEnter", r#""off""#),
    ("editor.acceptSuggestionOnCommitCharacter", "false"),
    ("editor.wordBasedSuggestions", r#""off""#),
    ("editor.tabCompletion", r#""off""#),
    ("editor.snippetSuggestions", r#""none""#),
    ("editor.detectIndentation", "false"),
    ("editor.insertSpaces", "false"),
    ("editor.formatOnType", "false"),
    ("editor.linkedEditing", "false"),
    ("html.autoClosingTags", "false"),
    ("files.autoSave", r#""off""#),
];

/// The settings file of alo's copy of this application, when it is one whose
/// editor rewrites what is typed — or [`None`].
///
/// It is inside the copy's own data folder, `--user-data-dir`, which is
/// `<profile>/data` ([`crate::alos_copy()`]); the editor reads its person-level
/// settings from `User/settings.json` beneath it.
#[must_use]
pub fn its_settings_file(identifier: &str, copy: &AlosCopy) -> Option<PathBuf> {
    WITH_AN_EDITOR_THAT_REWRITES.contains(&identifier).then(|| {
        copy.profile()
            .join("data")
            .join("User")
            .join("settings.json")
    })
}

#[cfg(test)]
#[path = "typed_as_it_is_tests.rs"]
mod tests;
