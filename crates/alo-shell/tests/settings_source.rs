//! The shell keeps no settings of its own, read from the source this crate
//! ships.
//!
//! The shell plan's *one place for settings* says that **this surface decides
//! nothing**: every value it writes goes through the crate that owns it, so
//! that what a person sets and what the machine enforces cannot disagree. Three
//! of the settings it is to hold — appearance, the dock and shortcuts — have no
//! file yet, and their own crates once said that file would be the shell's.
//! ADR 0038 proposes instead that each crate keeps its own, in the person's
//! folder, and task 6 waits on that decision.
//!
//! What must not happen while it waits is the option that decision rejects
//! arriving by accident: a compositor that serialises a `Changes`, names a
//! settings file or writes one. These tests read every shipped file in `src/`
//! and the manifest's shipped dependencies, so a `toml::to_string` in a drawing
//! file, a `fs::write` beside it or `XDG_CONFIG_HOME` in a string is a failing
//! build rather than a review comment.
//!
//! Comments are taken out first, and unit tests are not read: every
//! `#[cfg(test)]` module in these files is the last thing in its file, and
//! `*_tests.rs` and `*_testing.rs` files are tests. String literals are read
//! for the names of a settings location and blanked for everything else.
#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This crate's directory.
fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every shipped file in this crate's `src/`: its name and its text.
fn the_shipped_files() -> Vec<(String, String)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(manifest_dir().join("src")).unwrap() {
        let at = entry.unwrap().path();
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        let is_rust = Path::new(&named)
            .extension()
            .is_some_and(|extension| extension == "rs");
        let is_a_test = named.ends_with("_tests.rs") || named.ends_with("_testing.rs");
        if is_rust && !is_a_test {
            read.push((named, std::fs::read_to_string(&at).unwrap()));
        }
    }
    read.sort();
    assert!(
        read.iter().any(|(named, _)| named == "lib.rs"),
        "the source directory was not the one this test meant to read"
    );
    read
}

/// What in one file would make the shell a keeper of settings: the line, and
/// what on it would.
///
/// Three kinds of thing, each the shape option C of ADR 0038 would take:
/// serialising a value (`serde`, `toml`), writing a file's contents, and naming
/// where a person's settings are kept.
fn what_would_keep_a_setting(written: &str) -> Vec<(usize, &'static str)> {
    /// Identifiers that serialise a value into a file's shape.
    const SERIALISING: [&str; 6] = [
        "serde",
        "serde_json",
        "toml",
        "Serialize",
        "Deserialize",
        "to_string_pretty",
    ];
    /// Paths that write a file's contents, as they are spelt in code.
    const WRITING: [&str; 7] = [
        "fs::write",
        "File::create",
        "File::create_new",
        "File::options",
        "OpenOptions",
        "fs::rename",
        "fs::copy",
    ];
    /// Where a person's settings are, spelt the way a path or a variable is.
    const LOCATING: [&str; 5] = [
        "XDG_CONFIG_HOME",
        "CONFIG_HOME",
        "THE_SETTINGS",
        ".toml",
        ".config/",
    ];

    let mut found = Vec::new();
    let mut inside = false;
    let mut escaped = false;
    for (which, line) in written.lines().enumerate() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        if !inside && line.trim_start().starts_with("//") {
            continue;
        }
        let (code, whole) = without_comment(line, &mut inside, &mut escaped);
        for word in SERIALISING {
            if names(&code, word) {
                found.push((which + 1, word));
            }
        }
        for path in WRITING {
            if names(&code, path) {
                found.push((which + 1, path));
            }
        }
        for place in LOCATING {
            if whole.contains(place) {
                found.push((which + 1, place));
            }
        }
    }
    found
}

/// One line without its comment: once with string literals' contents taken
/// out, for code, and once with them kept, for the names of places.
fn without_comment(line: &str, inside: &mut bool, escaped: &mut bool) -> (String, String) {
    let mut code = String::new();
    let mut whole = String::new();
    let mut previous = None;
    let mut letters = line.chars().peekable();
    while let Some(letter) = letters.next() {
        if *inside {
            if *escaped {
                *escaped = false;
            } else if letter == '\\' {
                *escaped = true;
            } else if letter == '"' {
                *inside = false;
                code.push('"');
            }
            whole.push(letter);
            previous = Some(letter);
            continue;
        }
        if letter == '/' && letters.peek() == Some(&'/') {
            break;
        }
        if letter == '"' && previous != Some('\'') {
            *inside = true;
        }
        code.push(letter);
        whole.push(letter);
        previous = Some(letter);
    }
    *escaped = false;
    (code, whole)
}

/// Whether `line` uses `word` as a whole identifier or path.
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

/// The names of the dependencies the shipped shell is built with, which are
/// the manifest's `[target.….dependencies]` table and not its dev-dependencies.
fn the_shipped_dependencies(manifest: &str) -> Vec<String> {
    let mut named = Vec::new();
    let mut in_them = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_them = line.ends_with(".dependencies]") || line == "[dependencies]";
            continue;
        }
        if !in_them || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = line.split_once('=') {
            named.push(key.trim().to_owned());
        }
    }
    named
}

/// **The shell keeps no settings file of its own.** No shipped file in `src/`
/// serialises a value, writes a file's contents, or names where a person's
/// settings are kept — so a background, a dock edge or a shortcut can only ever
/// reach a disk through the crate that owns it, which is ADR 0038's
/// recommendation and the plan's *this surface decides nothing*.
#[test]
fn the_shell_keeps_no_settings_file_of_its_own() {
    let mut kept = Vec::new();
    for (named, written) in the_shipped_files() {
        for (line, what) in what_would_keep_a_setting(&written) {
            kept.push(format!("src/{named}:{line} names `{what}`"));
        }
    }
    assert!(
        kept.is_empty(),
        "the shell would keep a setting itself rather than through the crate that owns it:\n{}",
        kept.join("\n")
    );
}

/// **Nothing the shipped shell is built with would write a settings file.**
/// `serde` and `toml` are how every settings file in this workspace is written;
/// the crates that own a setting depend on them, and the surface that shows the
/// setting does not.
#[test]
fn the_shell_is_built_with_nothing_that_writes_a_settings_file() {
    let manifest = std::fs::read_to_string(manifest_dir().join("Cargo.toml")).unwrap();
    let shipped = the_shipped_dependencies(&manifest);
    assert!(
        shipped.iter().any(|named| named == "alo-appearance"),
        "the manifest was not read the way this test meant: {shipped:?}"
    );
    for refused in ["serde", "serde_json", "toml", "toml_edit"] {
        assert!(
            !shipped.iter().any(|named| named == refused),
            "the shipped shell depends on `{refused}`, which is how a settings file is written"
        );
    }
}

/// **A shell file that kept a setting is refused, one road at a time** — and
/// what only looks like one is not: a comment, a string that names no place, a
/// longer identifier, and anything inside the unit tests.
#[test]
fn a_shell_file_that_kept_a_setting_is_refused() {
    let caught = [
        ("let text = toml::to_string(&changes)?;", "toml"),
        ("use serde::Serialize;", "serde"),
        ("#[derive(Serialize)]", "Serialize"),
        ("std::fs::write(&at, text)?;", "fs::write"),
        ("let file = File::create(&at)?;", "File::create"),
        (
            "let file = OpenOptions::new().write(true).open(&at)?;",
            "OpenOptions",
        ),
        ("fs::rename(&fresh, &at)?;", "fs::rename"),
        (
            "let home = std::env::var_os(\"XDG_CONFIG_HOME\");",
            "XDG_CONFIG_HOME",
        ),
        ("let at = folder.join(\"appearance.toml\");", ".toml"),
        ("let at = home.join(\".config/alo\");", ".config/"),
        ("let at = alo_choosing::THE_SETTINGS;", "THE_SETTINGS"),
    ];
    for (line, what) in caught {
        let found = what_would_keep_a_setting(line);
        assert!(
            found
                .iter()
                .any(|(at, named)| *at == 1 && named.contains(what)),
            "`{line}` kept a setting and was not caught: {found:?}"
        );
    }

    let passed = [
        "// a comment may say toml::to_string and fs::write and XDG_CONFIG_HOME",
        "let said = strings.say(&key, &filling); // serde would be wrong here",
        "let tomlish = settled;",
        "let sentence = \"write the settings\";",
        "let runtime = std::env::var_os(\"XDG_RUNTIME_DIR\");",
        "let _ = fs::remove_file(self.path.with_extension(\"lock\"));",
    ];
    for line in passed {
        let found = what_would_keep_a_setting(line);
        assert!(
            found.is_empty(),
            "`{line}` keeps no setting and was refused: {found:?}"
        );
    }

    let only_in_a_test = "fn drawn() {}\n#[cfg(test)]\nmod tests {\n    fn f() { std::fs::write(\"a.toml\", \"\").unwrap(); }\n}\n";
    assert!(what_would_keep_a_setting(only_in_a_test).is_empty());

    let spanning = "let x = \"a string\n that ends here\"; std::fs::write(&at, text)?;";
    assert_eq!(what_would_keep_a_setting(spanning), vec![(2, "fs::write")]);
}

/// **The manifest's dev-dependencies are not the shipped shell.** A test may
/// read a file with `toml`; the refusal is about what the compositor carries.
#[test]
fn a_settings_writer_among_the_shipped_dependencies_is_refused() {
    let manifest = "[package]\nname = \"alo-shell\"\n\n\
                    [target.'cfg(target_os = \"linux\")'.dependencies]\n\
                    # a comment = not a dependency\n\
                    alo-appearance = { path = \"../alo-appearance\" }\n\
                    toml = { workspace = true }\n\n\
                    [target.'cfg(target_os = \"linux\")'.dev-dependencies]\n\
                    serde_json = { workspace = true }\n";
    let shipped = the_shipped_dependencies(manifest);
    assert_eq!(shipped, ["alo-appearance", "toml"]);
    assert!(!shipped.iter().any(|named| named == "serde_json"));
}
