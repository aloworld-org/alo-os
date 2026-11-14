//! **alo's own copy of a code editor, set so that what alo types is what
//! lands** —
//! [ADR 0153](../../../docs/decisions/0153-alo-types-by-keys-in-its-own-copy-and-reads-the-text-back.md).
//!
//! `alo_launching::its_settings_file` names the file, in alo's copy's own
//! profile, and `alo_launching::TYPED_AS_IT_IS` what goes in it. This writes
//! it, each time alo's copy starts, **changing only those keys**: the person can
//! change other settings in alo's copy, and those are kept. A file that is not a
//! JSON object — VSCodium allows comments in it, which JSON does not — is
//! **left as it is** and the reason is said; what alo types is read back before
//! it is saved, so a setting that did not take is caught there.

use std::fs::{DirBuilder, OpenOptions};
use std::io::Write as _;
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _};
use std::path::Path;

use serde_json::{Map, Value};

/// Why a settings file was left as it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LeftAsItIs {
    /// It is not one JSON object, so changing keys in it would mean replacing
    /// what somebody wrote.
    NotAnObject,
}

/// The settings `existing` holds — or none, for a file that is not there —
/// with each of alo's set, and everything else kept.
pub(crate) fn with_typing_left_alone(existing: Option<&str>) -> Result<String, LeftAsItIs> {
    let mut settings = match existing.map(str::trim) {
        None | Some("") => Map::new(),
        Some(text) => match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(settings)) => settings,
            _ => return Err(LeftAsItIs::NotAnObject),
        },
    };
    for (name, value) in alo_launching::TYPED_AS_IT_IS {
        // Each value is a constant reviewed in `alo_launching`, and its test
        // reads every one as JSON; one that would not read is left out rather
        // than written as a string the editor would misread.
        if let Ok(value) = serde_json::from_str::<Value>(value) {
            settings.insert(name.to_owned(), value);
        }
    }
    let mut written = serde_json::to_string_pretty(&Value::Object(settings))
        .map_err(|_| LeftAsItIs::NotAnObject)?;
    written.push('\n');
    Ok(written)
}

/// Write alo's settings into the file at `file`, making its folders private to
/// the person, and answer what was not done.
pub(crate) fn leave_typing_alone_in(file: &Path) -> Result<(), String> {
    let existing = match std::fs::read_to_string(file) {
        Ok(text) => Some(text),
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => None,
        Err(why) => return Err(format!("{} could not be read: {why}", file.display())),
    };
    let written =
        with_typing_left_alone(existing.as_deref()).map_err(|LeftAsItIs::NotAnObject| {
            format!(
                "{} is not a JSON object, so it was left as it is",
                file.display()
            )
        })?;
    let Some(folder) = file.parent() else {
        return Err(format!("{} has no folder", file.display()));
    };
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(folder)
        .map_err(|why| format!("{} could not be made: {why}", folder.display()))?;
    // Written beside it and moved over it, so an editor reading it never
    // reads half a file.
    let beside = folder.join(".settings.json.alo");
    let wrote = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&beside)
        .and_then(|mut opened| opened.write_all(written.as_bytes()))
        .and_then(|()| std::fs::rename(&beside, file));
    wrote.map_err(|why| format!("{} could not be written: {why}", file.display()))
}

#[cfg(test)]
#[path = "typing_left_as_it_is_tests.rs"]
mod tests;
