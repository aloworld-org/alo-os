//! The sign-in screen's promises that are about shape, read from the source
//! this crate ships.
//!
//! A test that typed a password and looked for it afterwards would only show
//! that the roads it walked are clean. These read the files instead, the way
//! `alo-greeting`'s `nothing_here_keeps_the_password.rs` reads that crate, so
//! a `#[derive(Debug)]` added tomorrow to a type holding a password — or a
//! call around the greeting straight to the accounts — is a failing build
//! rather than a password in somebody's log or a screen that knocks for a name
//! nobody authenticated.
//!
//! Comments and string literals are taken out first, so a file may talk about
//! passwords and `alo-sessiond` as much as it likes. Unit tests are not read:
//! every `#[cfg(test)]` module in this crate is the last thing in its file,
//! and `*_tests.rs` files are tests.
#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

#[path = "support/password_holders.rs"]
mod password_holders;

/// This crate's source directory.
fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every shipped `.rs` file: its name and its code.
fn shipped() -> Vec<(String, Vec<(usize, String)>)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        if at.extension().is_some_and(|kind| kind == "rs") && !named.ends_with("_tests.rs") {
            read.push((named, the_code_of(&std::fs::read_to_string(&at).unwrap())));
        }
    }
    assert!(read.len() > 50, "this crate has more files than that");
    read
}

/// The files that make up the sign-in screen.
fn the_sign_in_files() -> Vec<(String, Vec<(usize, String)>)> {
    let files: Vec<_> = shipped()
        .into_iter()
        .filter(|(named, _)| named.starts_with("sign_in_") || named == "nested_sign_in.rs")
        .collect();
    assert_eq!(
        files.len(),
        8,
        "a sign-in file was added or lost: {:?}",
        file_names(&files)
    );
    files
}

/// The names, for a failure.
fn file_names(files: &[(String, Vec<(usize, String)>)]) -> Vec<&str> {
    files.iter().map(|(named, _)| named.as_str()).collect()
}

/// The code of a file: no unit tests, no comments, no string literals.
fn the_code_of(written: &str) -> Vec<(usize, String)> {
    let mut code = Vec::new();
    let mut inside = false;
    let mut escaped = false;
    for (which, line) in written.lines().enumerate() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        if !inside && line.trim_start().starts_with("//") {
            continue;
        }
        let outside = outside_strings(line, &mut inside, &mut escaped);
        let code_part = match outside.split_once("//") {
            Some((before, _)) => before.to_owned(),
            None => outside,
        };
        code.push((which + 1, code_part));
    }
    code
}

/// One line with the contents of its string literals taken out.
fn outside_strings(line: &str, inside: &mut bool, escaped: &mut bool) -> String {
    let mut outside = String::new();
    let mut previous = None;
    for letter in line.chars() {
        if *inside {
            if *escaped {
                *escaped = false;
            } else if letter == '\\' {
                *escaped = true;
            } else if letter == '"' {
                *inside = false;
                outside.push('"');
            }
            previous = Some(letter);
            continue;
        }
        // A `'"'` is a character, not the start of a string.
        if letter == '"' && previous != Some('\'') {
            *inside = true;
        }
        outside.push(letter);
        previous = Some(letter);
    }
    *escaped = false;
    outside
}

/// Whether `line` uses `word` as a whole identifier.
fn names(line: &str, word: &str) -> bool {
    line.match_indices(word).any(|(at, _)| {
        let before = line.get(..at).and_then(|up_to| up_to.chars().next_back());
        let after = line.get(at + word.len()..).and_then(|on| on.chars().next());
        let part_of_a_word = |letter: Option<char>| {
            letter.is_some_and(|letter| letter.is_alphanumeric() || letter == '_')
        };
        !part_of_a_word(before) && !part_of_a_word(after)
    })
}

/// **The type that holds the password has no road out of it**: nothing
/// derived, no `Debug`, no `Display`, no copy, no comparison, and nothing in
/// its file that prints, formats or panics.
#[test]
fn the_password_holder_has_no_road_out() {
    let (_, code) = shipped()
        .into_iter()
        .find(|(named, _)| named == "sign_in_password.rs")
        .unwrap();
    assert!(
        code.iter()
            .any(|(_, line)| line.contains("struct TypedPassword"))
    );
    for (which, line) in code {
        for road in [
            "derive",
            "Debug",
            "Display",
            "Clone",
            "Copy",
            "PartialEq",
            "Hash",
            "Serialize",
            "print",
            "format!",
            "write!",
            "dbg!",
            "panic!",
            "expect(",
            "unwrap(",
            "log::",
            "tracing",
        ] {
            assert!(
                !line.contains(road),
                "sign_in_password.rs:{which} has `{road}` in it: {line}"
            );
        }
    }
}

/// **No type that carries a password can print one.** Every type holding a
/// password — directly, or by holding a type that does — is read for a derived
/// or written `Debug` or `Display`; and every type in the crate with a field
/// naming one of them must itself be on the list, so that a new holder cannot
/// be added without this test being told.
#[test]
fn no_type_that_carries_a_password_has_a_debug_that_could_print_it() {
    let carries = password_holders::derived(&shipped());
    let mut declared = Vec::new();
    for (named, code) in shipped() {
        let mut attributes: Vec<String> = Vec::new();
        let mut within: Option<String> = None;
        for (which, line) in &code {
            let trimmed = line.trim();
            for holder in &carries {
                for printing in ["Debug for", "Display for"] {
                    assert!(
                        !(trimmed.contains(printing)
                            && names(trimmed, holder)
                            && trimmed
                                .split(printing)
                                .nth(1)
                                .is_some_and(|after| names(after, holder))),
                        "{named}:{which} can print a {holder}: {trimmed}"
                    );
                }
            }
            if trimmed.starts_with("#[") {
                attributes.push(trimmed.to_owned());
                continue;
            }
            let declaration = [
                "pub struct ",
                "pub(crate) struct ",
                "struct ",
                "pub enum ",
                "pub(crate) enum ",
                "enum ",
            ]
            .iter()
            .find_map(|opening| trimmed.strip_prefix(opening));
            if let Some(rest) = declaration {
                let type_named: String = rest
                    .chars()
                    .take_while(|letter| letter.is_alphanumeric() || *letter == '_')
                    .collect();
                if carries.contains(&type_named) {
                    declared.push(type_named.clone());
                    for attribute in &attributes {
                        assert!(
                            !attribute.contains("Debug") && !attribute.contains("Clone"),
                            "{named}:{which} derives a road out for {type_named}: {attribute}"
                        );
                    }
                }
                if !trimmed.ends_with(';') && !trimmed.ends_with('}') {
                    within = Some(type_named);
                }
            } else if line.starts_with('}') {
                within = None;
            } else if let Some(holding) = &within {
                for holder in &carries {
                    if names(trimmed, holder) {
                        assert!(
                            carries.contains(holding),
                            "{named}:{which}: `{holding}` holds a {holder}, so it carries a password \
                             and has to be on this test's list: {trimmed}"
                        );
                    }
                }
            }
            attributes.clear();
        }
    }
    declared.sort();
    let mut expected: Vec<String> = carries.iter().map(|holder| holder.to_owned()).collect();
    expected.sort();
    assert_eq!(
        declared, expected,
        "a type on the list is not declared where it was read"
    );
}

/// **The sign-in screen writes no words of its own.** Every sentence it shows
/// is handed to it by `alo-greeting`, so nothing in its files declares a word,
/// fills a gap or looks a key up.
#[test]
fn the_sign_in_screen_writes_no_words_of_its_own() {
    for (named, code) in the_sign_in_files() {
        for (which, line) in code {
            for writing in [
                "Word::saying",
                "Word::",
                "Vocabulary",
                "Filling",
                ".say(",
                "Key::named",
            ] {
                assert!(
                    !line.contains(writing),
                    "{named}:{which} writes its own words: {line}"
                );
            }
        }
    }
}

/// **The screen calls the greeting and never what the greeting composes.** No
/// sign-in file names `alo_sessiond`, knocks, or reaches the accounts; the one
/// thing it takes from `alo_accounts` is the `Session` it hands over to; and
/// the one sign-in it asks for is `Greeting::signs_in`, once.
#[test]
fn the_sign_in_screen_calls_the_greeting_and_never_what_it_composes() {
    let mut asked = Vec::new();
    for (named, code) in the_sign_in_files() {
        for (which, line) in code {
            assert!(
                !line.contains("alo_sessiond"),
                "{named}:{which} reaches the opener: {line}"
            );
            assert!(
                !names(&line, "Knock"),
                "{named}:{which} knocks itself: {line}"
            );
            assert!(
                !line.contains(".knock("),
                "{named}:{which} knocks itself: {line}"
            );
            assert!(
                !names(&line, "Accounts"),
                "{named}:{which} reaches the accounts: {line}"
            );
            for (at, _) in line.match_indices("alo_accounts::") {
                let after = line.get(at + "alo_accounts::".len()..).unwrap_or_default();
                assert!(
                    after.starts_with("Session") && !after.starts_with("Session::"),
                    "{named}:{which} takes more than the session type from alo_accounts: {line}"
                );
            }
            if line.contains(".signs_in(") {
                asked.push((named.clone(), line.trim().to_owned()));
            }
        }
    }
    assert_eq!(asked.len(), 1, "{asked:#?}");
    let (named, line) = asked.first().unwrap();
    assert_eq!(named, "sign_in_screen.rs");
    assert!(
        line.starts_with("let greeted = greeting.signs_in("),
        "{line}"
    );

    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let shipped_dependencies = manifest
        .split("[target.'cfg(target_os = \"linux\")'.dependencies]")
        .nth(1)
        .and_then(|rest| rest.split("\n[").next())
        .unwrap();
    assert!(shipped_dependencies.contains("alo-greeting"));
    assert!(
        !shipped_dependencies.contains("alo-sessiond"),
        "the shipped shell depends on the opener directly"
    );
}
