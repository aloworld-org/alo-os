//! Nothing in this crate holds, logs or returns the password.
//!
//! The promise is about a shape rather than about a road: there is no field
//! for a password anywhere here, nothing copies one, and nothing prints one.
//! A test that typed a password and looked for it afterwards would only ever
//! show that the roads it happened to walk are clean; this reads the crate the
//! way `alo-saying`'s rented check reads for a rented name, so a `String` added
//! tomorrow is a failing build rather than a password in somebody's log.
//!
//! # What it reads, and what it deliberately does not
//!
//! Comments and string literals are taken out first, so a file may talk about
//! passwords as much as it likes — `src/words.rs` says the word in a sentence a
//! translator reads, and this crate's own documentation argues about it at
//! length. What is left is code, and in code the identifier may appear in
//! exactly two places: the parameter of the one method that takes one, and the
//! line that hands it straight to `alo-accounts`.
//!
//! Unit tests at the foot of a file are not read, and that is the convention
//! this workspace already keeps — every `#[cfg(test)]` module here is the last
//! thing in its file. A test that makes an account has to name a password, and
//! holding a test to a rule about shipped code would only teach somebody to
//! spell it differently.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_accounts::Accounts;
use alo_greeting::{Greeted, Greeting, Knocking, NotAnswered};
use alo_sessiond::{Answered, Knock};

/// The identifier this test is about.
const IT: &str = "password";

/// A door that opens whatever it is asked for and remembers nothing.
#[derive(Debug)]
struct ADoor;

impl Knocking for ADoor {
    fn knock(&self, _: Knock) -> Result<Answered, NotAnswered> {
        Ok(Answered::Opened)
    }
}

/// This crate's own source directory.
fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every `.rs` file this crate ships, as a path and its text.
fn every_file() -> Vec<(PathBuf, String)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        if at.extension().is_some_and(|kind| kind == "rs") {
            read.push((at.clone(), std::fs::read_to_string(&at).unwrap()));
        }
    }
    assert!(read.len() > 4, "this crate has more files than that");
    read
}

/// The code of a file: no unit tests, no comments, no string literals.
///
/// Line by line, because a finding has to name one — but **not line by line
/// underneath**: a string literal here spans lines (every sentence in
/// `src/words.rs` does), so whether a line is inside one is state carried from
/// the line before it. A scanner that forgot that would read the middle of a
/// translator's note as code, which is how this check first failed.
///
/// Whole-line comments are dropped before any of that, because a `"` in a
/// sentence a person reads is not the beginning of anything.
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
///
/// Quotes are left in place so what is removed is visible in a failure, and an
/// escaped quote inside a literal stays inside it.
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
    // A literal that reaches the end of a line carries on: `\` at the end of
    // one is the continuation this crate's own sentences are written with.
    *escaped = false;
    outside
}

/// Whether this line uses the identifier, rather than a longer word ending in
/// it — `a_wrong_password` is a test's name, not a binding.
fn names_it(line: &str) -> bool {
    let mut rest = line;
    while let Some(at) = rest.find(IT) {
        let before = rest.get(..at).and_then(|up_to| up_to.chars().next_back());
        let after = rest.get(at + IT.len()..).and_then(|on| on.chars().next());
        let a_word = |letter: Option<char>| {
            letter.is_some_and(|letter| letter.is_alphanumeric() || letter == '_')
        };
        if !a_word(before) && !a_word(after) {
            return true;
        }
        let Some(on) = rest.get(at + IT.len()..) else {
            break;
        };
        rest = on;
    }
    false
}

/// **The password appears in this crate's code exactly twice**: once as the
/// parameter of the one method that takes one, and once on the line that hands
/// it to the one thing on this machine that verifies passwords. There is
/// nowhere else for it to be, which is what makes *nothing here holds it* a
/// property rather than a promise.
#[test]
fn the_password_is_a_parameter_and_an_argument_and_nothing_else() {
    let mut found = Vec::new();
    for (at, written) in every_file() {
        for (which, line) in the_code_of(&written) {
            if names_it(&line) {
                found.push((at.clone(), which, line.trim().to_owned()));
            }
        }
    }

    assert_eq!(
        found.len(),
        2,
        "the password is named in this crate's code somewhere new: {found:#?}"
    );
    for (at, _, line) in &found {
        assert!(
            at.ends_with("greeting.rs"),
            "the password reached {}: {line}",
            at.display()
        );
    }
    let (_, _, parameter) = found.first().unwrap();
    assert!(
        parameter.contains("fn signs_in(") && parameter.contains("password: &str"),
        "the first mention is not the parameter of the one method that takes one: {parameter}"
    );
    let (_, _, handed_on) = found.get(1).unwrap();
    assert!(
        handed_on.contains("self.accounts.signs_in(name, password)"),
        "the second mention is not the line that hands it to alo-accounts: {handed_on}"
    );
}

/// **Nothing here copies it, keeps it or writes it down.** The shapes that
/// would: a field, an owned copy, a format string, a print. None of them can
/// appear on the two lines above, and this is what says so if one ever does.
#[test]
fn nothing_on_those_two_lines_keeps_or_prints_anything() {
    for (at, written) in every_file() {
        for (which, line) in the_code_of(&written) {
            if !names_it(&line) {
                continue;
            }
            for shape in [
                "to_owned",
                "to_string",
                "String",
                "clone",
                "format!",
                "println!",
                "eprintln!",
                "write!",
                "dbg!",
                "self.",
            ] {
                // `self.accounts.signs_in(name, password)` is the one `self.`
                // there is, and it is the handing on rather than the keeping.
                if shape == "self." && line.contains("self.accounts.signs_in(") {
                    continue;
                }
                assert!(
                    !line.contains(shape),
                    "{}:{which} does something with the password: {line}",
                    at.display()
                );
            }
        }
    }
}

/// **No type in this crate has a field that could hold one.** The check above
/// is about the identifier; this is about the shape, because a field called
/// `secret` would pass the first one and be the same bug.
///
/// Every field this crate declares is read out of its own source and held to a
/// list of what the greeter is made of. A field added to any type here fails
/// this until somebody says what it is, which is the moment to notice that it
/// is a password.
#[test]
fn every_field_this_crate_has_is_one_somebody_named() {
    /// What the greeter is made of, and the whole of it.
    const WHAT_IT_HOLDS: [&str; 5] = [
        // `Greeting`: the accounts, the described number, the door.
        "accounts", "person", "door", // `TheOpenersDoor`: where the door is.
        "at",   // Every refusal: what the machine said about it.
        "why",
    ];

    let mut fields = Vec::new();
    for (at, written) in every_file() {
        for (which, line) in the_code_of(&written) {
            let trimmed = line.trim();
            // A field declaration: `name: Type,` at a struct's indentation,
            // never a `let`, a `fn` parameter or a match arm.
            let Some((named, _)) = trimmed.split_once(": ") else {
                continue;
            };
            let named = named.trim_start_matches("pub ").trim();
            if !trimmed.ends_with(',')
                || named.is_empty()
                || !named
                    .chars()
                    .all(|letter| letter.is_ascii_lowercase() || letter == '_')
                || trimmed.starts_with("let ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || line.starts_with("    pub fn ")
                || line.contains("&self")
                || line.contains("&str")
                || line.contains("&mut")
                || line.contains("&Path")
                || line.contains("&Strings")
            {
                continue;
            }
            fields.push((at.clone(), which, named.to_owned()));
        }
    }

    assert!(!fields.is_empty(), "no fields were read at all");
    for (at, which, named) in fields {
        assert!(
            WHAT_IT_HOLDS.contains(&named.as_str()),
            "{}:{which} declares `{named}`, which nobody has said what it is — if it is \
             anything a person typed, it does not belong here",
            at.display()
        );
    }
}

/// **And what is printed carries none of it.** The shape checks above are what
/// make this true; this is the same fact from the ordinary end, on a real
/// sign-in with a password nothing else in the process has ever seen.
#[test]
fn nothing_a_greeting_prints_carries_what_was_typed() {
    const NOBODY_ELSES: &str = "hunter2-tourniquet-bramble";
    let mut store = Accounts::none().unwrap();
    store.created("ada", 1000, NOBODY_ELSES).unwrap();
    let greeting = Greeting::of(store, 1000, ADoor);

    let greeted = greeting.signs_in("ada", NOBODY_ELSES);
    assert!(matches!(greeted, Greeted::SignedIn(_)));

    for printed in [format!("{greeting:?}"), format!("{greeted:?}")] {
        assert!(
            !printed.contains(NOBODY_ELSES),
            "what was typed reached a debug line: {printed}"
        );
    }
    // And the greeter does not print the stored hashes either — a hash is not
    // a password, but it is what an offline guess runs against.
    let printed = format!("{greeting:?}");
    assert!(printed.contains("<not printed>"), "{printed}");
    assert!(!printed.contains("argon2"), "{printed}");
    assert!(!printed.contains('$'), "{printed}");
}
