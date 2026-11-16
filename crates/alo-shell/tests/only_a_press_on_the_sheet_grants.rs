//! **Only the person's press on the sheet grants alo an application it asked
//! for** — the owner's fourth test of 2026-11-16, *an agent never able to make
//! the grant itself* (`docs/design/when-alo-asks-for-an-application.md`).
//!
//! The behaviour is tested beside the code: a key, a click and a reader's
//! press answer *granted*, and silence, a sheet nobody can see and a task that
//! ended do not (`src/the_application_sheet_tests.rs`). What a behaviour test
//! cannot show is that **no other road exists**, so this reads the shipped
//! source and holds three things to it:
//!
//! 1. the proof that a person pressed (`APress`), without which the answer
//!    *granted* cannot be made, is made in one place, the sheet's own `press`,
//!    which only a person's key, click or reader's press reaches;
//! 2. the grant is made in one function, and only the carrying-out of that
//!    answer calls it;
//! 3. no file outside Settings' and this sheet's makes a `Changing` at all.
//!
//! And the agent's side: the daemon's door for an agent has no message that
//! grants, and its *wanted* list is refused there (`alo-protocol`'s and
//! `alo-agentd`'s own tests, from the change that added it).

#![expect(
    clippy::panic,
    reason = "in a test, a panic on source that will not read is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The shell's source folder.
fn the_source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every shipped source file under `src`, at any depth: its path from `src`
/// and its code, with comment lines taken out and everything from a test
/// module at the file's top level onwards left out. **Only a top-level
/// `#[cfg(test)]`** ends the shipped code: one indented inside an item marks a
/// single test helper, and the code after it ships.
fn the_shipped_code() -> Vec<(String, String)> {
    let mut read = Vec::new();
    let mut folders = vec![the_source()];
    while let Some(folder) = folders.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            panic!("{} would not read", folder.display());
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                folders.push(path);
                continue;
            }
            let named = path
                .strip_prefix(the_source())
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            // **A test file or a test-only module is not shipped**: `*_tests.rs`
            // are tests, and `*_testing.rs` are declared `#[cfg(test)]` in `lib.rs`.
            if !named.ends_with(".rs")
                || named.ends_with("_tests.rs")
                || named.ends_with("_testing.rs")
            {
                continue;
            }
            let Ok(written) = std::fs::read_to_string(&path) else {
                panic!("{named} would not read");
            };
            let shipped = without_test_items(&written);
            read.push((named, shipped.join("\n")));
        }
    }
    read.sort();
    assert!(read.len() > 100, "the source was not read: {}", read.len());
    assert!(
        read.iter().any(|(named, _)| named.starts_with("bin/")),
        "a folder under src was not read"
    );
    read
}

/// A file's lines with comments taken out and each top-level `#[cfg(test)]`
/// item left out — a `mod name;` line, or a block to its closing `}` at the
/// start of a line — and everything else kept, after it as well as before.
fn without_test_items(written: &str) -> Vec<&str> {
    let mut kept = Vec::new();
    let mut skipping = false;
    let mut in_a_block = false;
    for line in written.lines() {
        if in_a_block {
            in_a_block = line != "}";
            continue;
        }
        if skipping {
            if line.starts_with("mod ")
                || line.starts_with("pub mod ")
                || line.starts_with("pub(crate) mod ")
            {
                skipping = false;
                in_a_block = line.ends_with('{');
            }
            continue;
        }
        if line == "#[cfg(test)]" {
            skipping = true;
            continue;
        }
        if !line.trim_start().starts_with("//") {
            kept.push(line);
        }
    }
    kept
}

/// The test of the test: a test item in the middle of a file hides nothing
/// after it, and a test block is left out whole.
#[test]
fn a_test_item_hides_only_itself() {
    let written = "fn a() {}
#[cfg(test)]
mod helpers;
fn b() {}
#[cfg(test)]
#[path = \"x.rs\"]
mod tests {
    fn c() {}
}
fn d() {}
";
    assert_eq!(
        without_test_items(written),
        ["fn a() {}", "fn b() {}", "fn d() {}"]
    );
}

/// The files whose code holds `word`, in order.
fn holding(code: &[(String, String)], word: &str) -> Vec<String> {
    code.iter()
        .filter(|(_, shipped)| shipped.contains(word))
        .map(|(named, _)| named.clone())
        .collect()
}

/// **The proof of a press is made in the sheet's own file, once**, inside
/// its `press`: its field is private there, so the compiler already refuses it
/// anywhere else, and this says where *there* is.
#[test]
fn a_press_is_proved_in_one_place() {
    let code = the_shipped_code();
    assert_eq!(holding(&code, "APress(())"), ["the_application_sheet.rs"]);
    let (_, sheet) = code
        .iter()
        .find(|(named, _)| named == "the_application_sheet.rs")
        .unwrap_or_else(|| panic!("the sheet's file is gone"));
    // The type's own declaration, and the one press.
    assert_eq!(sheet.matches("APress(())").count(), 2, "{sheet}");
}

/// **The grant is made by one function, and one caller carries the answer
/// *granted* to it.**
#[test]
fn one_function_makes_the_grant_and_only_the_answer_reaches_it() {
    let code = the_shipped_code();
    assert_eq!(
        holding(&code, "fn alo_is_given("),
        ["giving_alo_an_application_it_asked_for.rs"]
    );
    let callers: Vec<String> = holding(&code, "alo_is_given(")
        .into_iter()
        .filter(|named| named != "giving_alo_an_application_it_asked_for.rs")
        .collect();
    assert_eq!(callers, ["the_application_sheet_on_the_server.rs"]);
}

/// **No shipped file writes the grants file by itself**: only through a
/// `Changing`, which writes it whole and then tells the daemon.
#[test]
fn no_file_writes_the_grants_around_a_changing() {
    let code = the_shipped_code();
    assert_eq!(
        holding(&code, "alo_remembering::kept("),
        Vec::<String>::new()
    );
    for maker in ["Grant::checked", "Grants::default().grant", ".grant(Grant"] {
        assert_eq!(holding(&code, maker), Vec::<String>::new(), "{maker}");
    }
}

/// **No other file makes a `Changing`**: Settings' grants section, and the
/// file that carries the sheet's answer to the grants.
#[test]
fn only_settings_and_the_sheet_change_the_grants() {
    let code = the_shipped_code();
    assert_eq!(
        holding(&code, "Changing::of("),
        [
            "giving_alo_an_application_it_asked_for.rs",
            "settings_granted.rs"
        ]
    );
}
