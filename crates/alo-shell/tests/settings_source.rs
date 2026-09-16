//! The shell keeps no settings of its own, and Settings changes nothing except
//! through the crate that owns each setting — read from the source this crate
//! ships.
//!
//! The shell plan's *one place for settings* says that **this surface decides
//! nothing**: every value it writes goes through the crate that owns it, so
//! that what a person sets and what the machine enforces cannot disagree.
//! ADR 0038 decided that each crate keeps its own file in the person's folder,
//! and rejected a compositor that serialises a `Changes`, names a settings file
//! or writes one. The first four tests hold the whole shell to that; the last
//! two hold the Settings files to the rest of the plan's constraint — no words
//! of their own, no grant, one way to revoke, every write through its keeper,
//! and no agent's road in.
//!
//! These tests read every shipped file in `src/`
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

/// The files that make up Settings, and each one's code: comments gone, string
/// literals' contents blanked, unit tests not read.
fn the_settings_files() -> Vec<(String, Vec<(usize, String)>)> {
    let mut read = Vec::new();
    for (named, written) in the_shipped_files() {
        if named.starts_with("settings_") || named == "nested_settings.rs" {
            read.push((named, the_code_of(&written)));
        }
    }
    let names: Vec<&str> = read.iter().map(|(named, _)| named.as_str()).collect();
    assert_eq!(
        names,
        [
            "nested_settings.rs",
            "settings_answering.rs",
            "settings_chord.rs",
            "settings_granted.rs",
            "settings_keepers.rs",
            "settings_kept.rs",
            "settings_keys.rs",
            "settings_lines.rs",
            "settings_paint.rs",
            "settings_paired.rs",
            "settings_places.rs",
            "settings_raster.rs",
            "settings_seat.rs",
            "settings_window.rs",
        ],
        "a Settings file was added or lost, and this test has to be told"
    );
    read
}

/// The code of a file, line by line: no unit tests, no comments, and string
/// literals' contents taken out.
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
        let (without, _) = without_comment(line, &mut inside, &mut escaped);
        code.push((which + 1, without));
    }
    code
}

/// Writing words of its own, which only a crate's vocabulary may do.
const WORDING: [&str; 6] = [
    "Word::",
    "Vocabulary",
    "Key::named",
    "format!",
    "push_str",
    "concat!",
];

/// Granting, or revoking any way but `alo_changing::Changing::revoked`.
const GRANTING: [&str; 12] = [
    "Granting",
    "alo_picking",
    ".granted(",
    ".grant(",
    ".revoke(",
    "revoke_on",
    "revoke_pairing",
    "revoke_allowed",
    "pairings_kept",
    "alo_remembering::kept",
    "reset_everything",
    "put_everything_back",
];

/// Answering setup again, which is asked once.
const RE_ASKING: [&str; 3] = ["SettingUp", ".answer(", ".setting_up("];

/// An agent's road into Settings.
const AN_AGENTS_ROAD: [&str; 5] = [
    "alo_turn",
    "alo_approving",
    "alo_context",
    "alo_overlay",
    "Turning",
];

/// The keepers' own doors, which only `settings_keepers.rs` calls.
const KEEPING: [&str; 3] = [
    "keeping::keep",
    "keeping::put_back_as_shipped",
    "keeping::at_sign_in",
];

/// What in Settings' code would word something, grant something, revoke by a
/// road of its own, write a setting around its keeper, or reach an agent: the
/// line, and what on it would.
fn what_settings_must_not_do(named: &str, code: &[(usize, String)]) -> Vec<(usize, &'static str)> {
    let mut found = Vec::new();
    for (line, text) in code {
        for word in WORDING
            .iter()
            .chain(&GRANTING)
            .chain(&RE_ASKING)
            .chain(&AN_AGENTS_ROAD)
        {
            if text.contains(word) {
                found.push((*line, *word));
            }
        }
        if text.contains('"') && !text.trim_start().starts_with("reason = ") {
            found.push((*line, "a string a person could read"));
        }
        if named != "settings_keepers.rs" {
            for door in KEEPING {
                if text.contains(door) {
                    found.push((*line, door));
                }
            }
        }
        if named != "settings_granted.rs" && text.contains("Changing::of") {
            found.push((*line, "Changing::of"));
        }
        if named != "settings_answering.rs"
            && (text.contains("answered_by") || text.contains("Choosing::at"))
        {
            found.push((*line, "answered_by"));
        }
    }
    found
}

/// **Settings words nothing, grants nothing, revokes one way, writes only
/// through each setting's keeper, and has no agent's road into it.** Every
/// sentence is a crate's; a grant is `alo-picking`'s to make; a grant and a
/// pairing are revoked with `alo_changing::Changing::revoked` alone; a setting
/// reaches a disk through the crate that owns it; and no turn, approval or
/// overlay is named in a Settings file.
#[test]
fn settings_words_nothing_grants_nothing_and_changes_only_through_the_owning_crates() {
    let mut refused = Vec::new();
    let mut revoking = Vec::new();
    for (named, code) in the_settings_files() {
        for (line, what) in what_settings_must_not_do(&named, &code) {
            refused.push(format!("src/{named}:{line} `{what}`"));
        }
        for (line, text) in &code {
            if text.contains("Changing::of") {
                revoking.push(format!("src/{named}:{line}"));
                assert!(
                    text.contains(".revoked(row, now)"),
                    "src/{named}:{line} makes a Changing for something other than revoking a row: {text}"
                );
            }
        }
    }
    assert!(
        refused.is_empty(),
        "Settings does what it must not:\n{}",
        refused.join("\n")
    );
    assert_eq!(
        revoking.len(),
        1,
        "a grant and a pairing are revoked by one call: {revoking:?}"
    );
}

/// **Each road is caught in a Settings file, and what only looks like one is
/// not** — so the test above is a test.
#[test]
fn a_settings_file_that_words_grants_or_writes_around_its_keeper_is_refused() {
    let caught = [
        (
            "settings_lines.rs",
            "let said = format!(\"{name} chosen\");",
            "format!",
        ),
        (
            "settings_lines.rs",
            "const CHOSEN: Word = Word::saying(\"x\", \"y\");",
            "Word::",
        ),
        (
            "settings_window.rs",
            "let chosen = \"chosen\";",
            "a string a person could read",
        ),
        (
            "settings_granted.rs",
            "let granting: Granting = Granting::to(who, lasting);",
            "Granting",
        ),
        (
            "settings_granted.rs",
            "changing.granted(&granting, &chosen, now)?;",
            ".granted(",
        ),
        (
            "settings_granted.rs",
            "grants.revoke(seen.id());",
            ".revoke(",
        ),
        (
            "settings_granted.rs",
            "seen.revoke_on(&mut machine);",
            "revoke_on",
        ),
        (
            "settings_granted.rs",
            "daemon.revoke_pairing(seen.machine());",
            "revoke_pairing",
        ),
        (
            "settings_granted.rs",
            "alo_remembering::kept(&at, &grants, now)?;",
            "alo_remembering::kept",
        ),
        (
            "settings_window.rs",
            "self.appearance.put_everything_back();",
            "put_everything_back",
        ),
        (
            "settings_answering.rs",
            "setting_up.answer(&answer)?;",
            ".answer(",
        ),
        (
            "settings_window.rs",
            "let turn: alo_turn::Turn;",
            "alo_turn",
        ),
        (
            "settings_window.rs",
            "alo_dock::keeping::keep(&at, dock.changes())?;",
            "keeping::keep",
        ),
        (
            "settings_window.rs",
            "Changing::of(&mut grants, &at, daemon).revoked(row, now);",
            "Changing::of",
        ),
        (
            "settings_window.rs",
            "choosing.answered_by(None)?;",
            "answered_by",
        ),
    ];
    for (named, line, what) in caught {
        let found = what_settings_must_not_do(named, &the_code_of(line));
        assert!(
            found.iter().any(|(at, road)| *at == 1 && *road == what),
            "`{line}` in {named} was not caught as `{what}`: {found:?}"
        );
    }

    let passed = [
        (
            "settings_window.rs",
            "// format!(\"a comment\") and .revoke( in a comment",
        ),
        (
            "settings_window.rs",
            "SettingsRow::Granted(row) => self.granted.revoked(row, daemon, now, strings),",
        ),
        (
            "settings_keepers.rs",
            "alo_dock::keeping::keep(at, drawn.changes())",
        ),
        (
            "settings_granted.rs",
            "let answered = Changing::of(grants, &self.grants_at, daemon).revoked(row, now);",
        ),
        (
            "settings_answering.rs",
            "SettingsChoice::NotAtAll => choosing.answered_by(None),",
        ),
        ("nested_settings.rs", "    reason = \"one session frame\""),
        (
            "settings_lines.rs",
            "strings.say(&may.word().key(), &Filling::nothing())",
        ),
    ];
    for (named, line) in passed {
        let found = what_settings_must_not_do(named, &the_code_of(line));
        assert!(
            found.is_empty(),
            "`{line}` in {named} was refused: {found:?}"
        );
    }
    let only_in_a_test =
        "fn drawn() {}\n#[cfg(test)]\nmod tests {\n    fn f() { let _ = format!(\"x\"); }\n}\n";
    assert!(
        what_settings_must_not_do("settings_lines.rs", &the_code_of(only_in_a_test)).is_empty()
    );
}
