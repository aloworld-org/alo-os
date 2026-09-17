//! No argument of any system verb is a string, a path, or anything that holds
//! one — walked over every type an argument is made of.
//!
//! The plan: *each verb's arguments as closed types with no string that becomes
//! a path, a command, a configuration line or a device name — held by a test
//! that walks every argument type and fails on a free `String` or `PathBuf`.*
//!
//! Rust has no reflection, so the walk is over the source the compiler builds:
//! start at `SystemVerb`, read the type of every variant's payload, find that
//! type's own definition and read its fields, and so on until every leaf is a
//! byte or an array of bytes. **Any type that is not defined in the files that
//! hold the list, and is not a byte, fails** — a `String` and a `PathBuf` by
//! name, and anything else nobody has looked at yet, because a walk that only
//! knew what to refuse would pass the first type nobody thought of.
//!
//! The compiler holds the other half: `SystemVerb` must be `Copy`
//! (`src/verbs.rs`), which nothing owning heap text or a path can be.

use std::collections::BTreeSet;

/// The files the list and its argument types are defined in.
const DEFINED_IN: [&str; 2] = [
    concat!(env!("CARGO_MANIFEST_DIR"), "/src/verbs.rs"),
    concat!(env!("CARGO_MANIFEST_DIR"), "/src/arguments.rs"),
];

/// Where the walk starts.
const THE_LIST: &str = "SystemVerb";

/// What a closed argument may bottom out in: a byte, and the constant that
/// says how many of them an identity is.
const LEAVES: [&str; 2] = ["u8", "IDENTITY_BYTES"];

/// Types that must never be reached, named so a failure says which it was.
const NEVER: [&str; 12] = [
    "String", "str", "PathBuf", "Path", "OsString", "OsStr", "CString", "CStr", "Vec", "Box",
    "char", "Cow",
];

/// The source of the files, with comments and each file's tests cut off.
fn the_source() -> String {
    DEFINED_IN
        .iter()
        .map(|path| {
            let written = std::fs::read_to_string(path).unwrap_or_default();
            assert!(!written.is_empty(), "{path} could not be read");
            let before_tests = written.split("#[cfg(test)]").next().unwrap_or_default();
            before_tests
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The body of `pub enum name { … }` or `pub struct name( … );`, if defined.
fn body_of<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    for (opening, closing) in [
        (format!("pub enum {name} {{"), "\n}"),
        (format!("pub struct {name}("), ");"),
        (format!("pub struct {name} {{"), "\n}"),
    ] {
        if let Some((_, after)) = source.split_once(&opening) {
            return after.split_once(closing).map(|(body, _)| body);
        }
    }
    None
}

/// The types named in a body: every identifier that is inside parentheses,
/// brackets or after a field's colon — which is to say not a variant's own
/// name.
fn types_named_in(body: &str) -> Vec<String> {
    let mut named = Vec::new();
    let mut depth = 0_u32;
    let mut after_colon = false;
    let mut word = String::new();
    for character in body.chars().chain(std::iter::once(' ')) {
        if character.is_alphanumeric() || character == '_' {
            word.push(character);
            continue;
        }
        if !word.is_empty() && (depth > 0 || after_colon) {
            named.push(std::mem::take(&mut word));
        }
        word.clear();
        match character {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' => depth = depth.saturating_sub(1),
            ':' => after_colon = true,
            ',' | '\n' => after_colon = false,
            _ => {}
        }
    }
    named
        .into_iter()
        .filter(|word| !word.chars().all(|c| c.is_ascii_digit()))
        .collect()
}

/// **Every type any verb's argument is made of is a byte, or a type defined
/// beside the list that is itself made of bytes** — and no string, path or
/// owned buffer is reachable from the list at all.
#[test]
fn no_argument_of_any_verb_can_hold_text_a_path_or_a_device_name() {
    let source = the_source();
    let mut to_walk = vec![THE_LIST.to_owned()];
    let mut walked = BTreeSet::new();
    let mut leaves_reached = BTreeSet::new();

    while let Some(name) = to_walk.pop() {
        if !walked.insert(name.clone()) {
            continue;
        }
        let body = body_of(&source, &name);
        assert!(
            body.is_some(),
            "`{name}` is reached from a system verb's arguments and is not a type defined beside \
             the list — a verb's argument may be made only of bytes and types defined in \
             src/verbs.rs or src/arguments.rs"
        );
        for reached in types_named_in(body.unwrap_or_default()) {
            assert!(
                !NEVER.contains(&reached.as_str()),
                "`{name}` holds a `{reached}`, and no argument of a system verb may"
            );
            if LEAVES.contains(&reached.as_str()) {
                leaves_reached.insert(reached);
            } else {
                to_walk.push(reached);
            }
        }
    }

    // The walk really went somewhere: through the two argument shapes and down
    // to bytes. A parser that found nothing would otherwise pass.
    for expected in ["SystemVerb", "Identity", "Switch"] {
        assert!(
            walked.contains(expected),
            "the walk never reached {expected}: {walked:?}"
        );
    }
    assert!(leaves_reached.contains("u8"), "{leaves_reached:?}");
}

/// **The walk refuses what it exists to refuse.** The same walk over a list
/// with a `String` in it, a `PathBuf` in it, or a type it has never heard of
/// fails — shown here against the parser directly, so a walk that quietly
/// stopped finding anything could not pass the test above.
#[test]
fn the_walk_finds_a_string_a_path_and_a_type_nobody_looked_at() {
    let written = "
pub enum SystemVerb {
    AddPrinter(Identity),
    Rename { to: String },
    Mount(std::path::PathBuf),
    Configure(Line),
}
pub struct Identity([u8; IDENTITY_BYTES]);
";
    let reached = types_named_in(body_of(written, "SystemVerb").unwrap_or_default());
    assert!(reached.contains(&"Identity".to_owned()), "{reached:?}");
    assert!(reached.contains(&"String".to_owned()), "{reached:?}");
    assert!(reached.contains(&"PathBuf".to_owned()), "{reached:?}");
    assert!(reached.contains(&"Line".to_owned()), "{reached:?}");
    assert_eq!(body_of(written, "Line"), None);
    assert!(!reached.contains(&"AddPrinter".to_owned()), "{reached:?}");
    assert_eq!(
        types_named_in(body_of(written, "Identity").unwrap_or_default()),
        ["u8", "IDENTITY_BYTES"]
    );
}
