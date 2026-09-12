//! Nothing in the door that tests a provider holds, logs or renders the key.
//!
//! The acceptance for *test a provider before saving it* says the key is never
//! logged, never in the sentence and never in an error, and that this is held
//! by a test that reads the crate the way `alo-greeting`'s
//! `nothing_here_keeps_the_password` reads that one. This is that test, for the
//! three files the door is made of.
//!
//! The promise is about a shape rather than about a road: a test that typed a
//! key and looked for it afterwards would only ever show that the roads it
//! happened to walk are clean. This reads the code instead, so a `String`
//! added tomorrow is a failing build rather than a key in somebody's log.
//!
//! # What it reads, and what it deliberately does not
//!
//! Comments and string literals are taken out first, so a file may talk about
//! keys as much as it likes — the documentation in these files does, at length.
//! What is left is code, and in code the identifier may appear in exactly the
//! places a borrowed `alo_models::Secret` has to pass through on its way to the
//! one crate that can put it on a request: the field that holds the borrow, the
//! parameter that takes it, the constructor that stores it, and the two lines
//! that hand it to `alo_models::Trying` — one per path of the one function.
//!
//! Unit tests at the foot of a file are not read, which is the convention this
//! workspace keeps — every `#[cfg(test)]` module is the last thing in its file.
//! A test has to name a key to type one, and holding a test to a rule about
//! shipped code would only teach somebody to spell it differently.
//!
//! # And the two files downstream of the wire never name it at all
//!
//! `found.rs` and `vetted.rs` hold what the test found. Neither has a field,
//! a parameter or a line naming the key, because nothing that comes back from
//! the wire is made from one — which is what makes *never in the sentence*
//! true by construction rather than by care.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a file that could not be read is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The identifier this test is about.
const IT: &str = "key";

/// This crate's own source directory.
fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// One of the door's files, read.
fn read(named: &str) -> String {
    let at = src().join(named);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// The code of a file: no unit tests, no comments, no string literals.
///
/// Line by line, because a finding has to name one — but **not line by line
/// underneath**: a string literal here spans lines (every sentence in
/// `src/words.rs` does), so whether a line is inside one is state carried from
/// the line before it.
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
        let without_strings = outside_strings(line, &mut inside, &mut escaped);
        let Some((before_comment, _)) = without_strings.split_once("//") else {
            code.push((which + 1, without_strings));
            continue;
        };
        code.push((which + 1, before_comment.to_owned()));
    }
    code
}

/// One line with the contents of its string literals taken out, and whether it
/// ended inside one.
fn outside_strings(line: &str, inside: &mut bool, escaped: &mut bool) -> String {
    let mut outside = String::new();
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
            continue;
        }
        if letter == '"' {
            *inside = true;
            outside.push('"');
            continue;
        }
        outside.push(letter);
    }
    *escaped = false;
    outside
}

/// Whether this line uses the identifier, rather than a longer word ending in
/// or beginning with it — `RefusedTheKey` is a variant, `keyring` a word.
///
/// **One method call is not the credential.** `alo_strings::Word::key` is a
/// string table's key — `models.not-tried.key-not-accepted` — and every
/// sentence in this crate is looked up by one, so `.key()` is excluded and
/// nothing else is. A field, a binding, a parameter or a declaration named
/// `key` is still found: `fn key(` has no dot before it, and `self.key` has no
/// parenthesis after it.
fn names_it(line: &str) -> bool {
    let mut rest = line;
    while let Some(at) = rest.find(IT) {
        let before = rest.get(..at).and_then(|up_to| up_to.chars().next_back());
        let after = rest.get(at + IT.len()..).and_then(|on| on.chars().next());
        let a_word = |letter: Option<char>| {
            letter.is_some_and(|letter| letter.is_alphanumeric() || letter == '_')
        };
        let a_words_key = before == Some('.') && after == Some('(');
        if !a_word(before) && !a_word(after) && !a_words_key {
            return true;
        }
        let Some(on) = rest.get(at + IT.len()..) else {
            break;
        };
        rest = on;
    }
    false
}

/// Every line of the door's code that names the key, with where it is.
fn every_mention() -> Vec<(String, usize, String)> {
    let mut found = Vec::new();
    for file in ["vetting.rs", "found.rs", "vetted.rs"] {
        for (which, line) in the_code_of(&read(file)) {
            if names_it(&line) {
                found.push((file.to_owned(), which, line.trim().to_owned()));
            }
        }
    }
    found
}

/// **The key appears in the door's code exactly five times**, all in
/// `vetting.rs`: the field that borrows it, the parameter that takes it, the
/// constructor that stores it, and the two lines — one per path — that hand it
/// to the one crate that can put it on a request. There is nowhere else for it
/// to be, which is what makes *nothing here holds it* a property rather than a
/// promise.
#[test]
fn the_key_is_a_borrow_a_parameter_a_field_and_two_hand_overs_and_nothing_else() {
    let found = every_mention();
    assert_eq!(
        found.len(),
        5,
        "the key is named in the door's code somewhere new: {found:#?}"
    );
    for (file, _, _) in &found {
        assert_eq!(file, "vetting.rs", "the key reached {file}: {found:#?}");
    }

    let lines: Vec<&str> = found.iter().map(|(_, _, line)| line.as_str()).collect();
    assert!(
        lines.contains(&"key: Option<&'a Secret>,"),
        "the field that borrows it is missing or changed: {lines:#?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("pub fn provider(")
                && line.contains("key: Option<&'a Secret>")),
        "the parameter of the one constructor is missing or changed: {lines:#?}"
    );
    assert!(
        lines.contains(&"Self { provider, key }"),
        "the constructor that stores the borrow is missing or changed: {lines:#?}"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.contains("Trying::provider(self.provider, self.key)"))
            .count(),
        2,
        "the two hand-overs to alo_models::Trying are missing or changed: {lines:#?}"
    );
}

/// **Nothing on those lines copies it, keeps it or writes it down.** The
/// shapes that would: an owned copy, a format string, a print, a log, a
/// rendering. None of them appears on a line that names the key.
#[test]
fn nothing_on_a_line_naming_the_key_copies_prints_or_keeps_it() {
    for (file, which, line) in every_mention() {
        for forbidden in [
            "String",
            "to_owned",
            "to_string",
            "clone",
            "format!",
            "println",
            "eprintln",
            "print!",
            "log",
            "write!",
            "Debug",
            "Display",
            ".0",
            "as_str",
            "bearer",
        ] {
            assert!(
                !line.contains(forbidden),
                "{file}:{which} {forbidden}s the key: {line}"
            );
        }
    }
}

/// **The two files downstream of the wire never name the key at all**, and
/// the door's own `Debug` is derived from a `Secret` that renders as nothing —
/// so there is no rendering of any type in this door that could carry one.
#[test]
fn what_the_test_found_is_made_from_nothing_that_holds_the_key() {
    for file in ["found.rs", "vetted.rs"] {
        let code = the_code_of(&read(file));
        assert!(
            !code.iter().any(|(_, line)| names_it(line)),
            "{file} names the key, and what a test found must be made from nothing that holds one"
        );
        assert!(
            !code.iter().any(|(_, line)| line.contains("Secret")),
            "{file} holds a Secret, and nothing downstream of the wire may"
        );
    }
    let door = the_code_of(&read("vetting.rs"));
    assert!(
        !door
            .iter()
            .any(|(_, line)| line.contains("impl") && line.contains("Debug")),
        "the door writes its own Debug, which is where a key would be rendered by hand"
    );
}
