//! One palette, one source, and nothing allowed to disagree with it.
//!
//! `docs/design/palette.toml` is the source (ROADMAP.md, v0.01: *the colours
//! come from a source this repository can read — and that is not CSS*). This is
//! what makes that sentence true rather than aspirational: it reads that file
//! and holds every other copy of the palette in this repository to it.
//!
//! # Why copies exist at all
//!
//! A compositor cannot parse a file at the moment it draws a frame, so
//! `alo_appearance::Role` carries the twelve as Rust constants. A designer reads
//! `docs/design/figma-brief.md`, which prints them as a table. Both are copies,
//! both are legitimate, and both are exactly how a palette silently drifts —
//! somebody corrects one hex and the other two keep the old one for a year.
//!
//! So the copies stay and the **drift** goes: change a value in the source and
//! this test names every place that has not caught up.
//!
//! # And where the source itself gets its values
//!
//! The design file, since 2026-10-04 (ADR 0093). That raises the question this
//! file is really for: **a check that needs live Figma access is a check almost
//! nobody can run.** Of the three machines working on this repository, one has
//! Figma and two do not.
//!
//! So the measurement is committed. `docs/design/figma-snapshot/variables.toml`
//! records what was measured, from which node, on which day, by which method,
//! and this test reads *that* — so the gate runs offline, on any machine, and a
//! value changed in the source without the measurement changing with it is
//! caught here rather than by whoever next opens Figma.
//!
//! # What it does not do
//!
//! It does not read `alo-workplace/web/src/ds/tokens.css`, which is in another
//! repository and not present beside this checkout. Ending CSS's authority over
//! the palette is what this file establishes *here*; the web client adopting the
//! generated custom properties is that repository's work, and until it does, its
//! stylesheet is a fourth copy nothing checks. That is stated in the report
//! rather than implied by silence.
//!
//! It does not check the dark theme, because there is no dark theme to check.
//! The design binds the same twelve names to different values in a dark mode and
//! **dark-theme parity needs its own measured evidence** before anything claims
//! it. `charcoal` is the dark ground this repository actually has, and it is
//! held to the source like the rest.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#![expect(
    clippy::panic,
    reason = "a missing or malformed colour names itself in the failure, which an unwrap could not"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use alo_appearance::{Colour, Role, Token};

/// The shape of palette this test reads.
///
/// **2 since 2026-10-04.** 1 was six entries named by colour; 2 is twelve
/// semantic roles under the design file's own variable names. A reader written
/// for one shape refuses the other rather than reading `text/primary` as though
/// it were `navy`, which is the rule `record-file.md` and
/// `machine-description.md` both state.
const THE_FORMAT: i64 = 2;

/// The one entry in the source that is not one of the design file's roles.
///
/// The dark ground. `crates/alo-appearance/src/accent.rs` measures every
/// choosable accent against it — a hex that reads on cream is illegible on
/// charcoal — so it is retained deliberately, and named here so that *retained*
/// and *forgotten* are different things to this test.
const NOT_A_DESIGN_ROLE: &str = "charcoal";

/// This repository, from this crate.
fn the_repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Where the source lives.
fn the_source() -> PathBuf {
    the_repository().join("docs/design/palette.toml")
}

/// The design brief, which prints the same table for a person to read.
fn the_brief() -> PathBuf {
    the_repository().join("docs/design/figma-brief.md")
}

/// The committed measurement of the design file's variables.
fn the_snapshot() -> PathBuf {
    the_repository().join("docs/design/figma-snapshot/variables.toml")
}

/// Every colour the source names, by the key it names it under.
///
/// Parsed with the same crate the rest of this repository reads TOML with, and
/// the format number is answered first — a source written for a later alo OS is
/// refused rather than half-read.
fn as_written() -> BTreeMap<String, String> {
    let said = std::fs::read_to_string(the_source()).expect("the palette source is where it says");
    let read: toml::Table = said.parse().expect("the palette source is TOML");

    assert_eq!(
        read.get("format").and_then(toml::Value::as_integer),
        Some(THE_FORMAT),
        "the palette source says a shape this test does not read"
    );

    read.iter()
        .filter_map(|(name, value)| {
            let hex = value.as_table()?.get("hex")?.as_str()?;
            Some((name.clone(), hex.to_owned()))
        })
        .collect()
}

/// Every light-theme value the committed measurement records, by role name.
fn as_measured() -> BTreeMap<String, String> {
    let said = std::fs::read_to_string(the_snapshot()).expect("the snapshot is where it says");
    let read: toml::Table = said.parse().expect("the snapshot is TOML");

    let light = read
        .get("light")
        .and_then(toml::Value::as_table)
        .expect("the snapshot records a light theme")
        .clone();

    light
        .iter()
        .filter_map(|(name, value)| {
            let hex = value.as_table()?.get("hex")?.as_str()?;
            Some((name.clone(), hex.to_owned()))
        })
        .collect()
}

/// **Every colour the shell is built out of is the colour the source states.**
///
/// The one that matters: `Role::colour` is what a frame is actually drawn with,
/// and this is what stops it being a hand-typed hex nobody rechecked.
#[test]
fn the_shells_colours_are_the_sources_colours() {
    let written = as_written();

    for role in Role::ALL {
        let key = role.key();
        let said = written
            .get(key)
            .unwrap_or_else(|| panic!("the palette source names no colour `{key}`"));
        let from_the_source = Colour::written(said)
            .unwrap_or_else(|_| panic!("`{said}` in the palette source is not a colour"));

        assert_eq!(
            role.colour(),
            from_the_source,
            "`{key}` is {} in the shell and {said} in the palette source",
            role.colour()
        );
    }
}

/// **And the retained dark ground is held to the source too**, because it is the
/// one value the design file does not define and therefore the one nobody else
/// is checking.
#[test]
fn the_dark_ground_is_the_colour_the_source_states() {
    let written = as_written();
    let said = written
        .get(NOT_A_DESIGN_ROLE)
        .unwrap_or_else(|| panic!("the palette source names no `{NOT_A_DESIGN_ROLE}`"));
    let from_the_source = Colour::written(said)
        .unwrap_or_else(|_| panic!("`{said}` in the palette source is not a colour"));

    assert_eq!(
        Token::Charcoal.colour(),
        from_the_source,
        "the dark ground is {} in the shell and {said} in the palette source",
        Token::Charcoal.colour()
    );
}

/// **And the source names exactly these thirteen** — no colour the shell has
/// never heard of, and none of the shell's missing.
///
/// The half the test above cannot see. Without it, a colour could be added to
/// the source, be used by the web client, and never exist in the operating
/// system — which is the same drift running the other way.
#[test]
fn the_source_names_exactly_the_colours_the_shell_has() {
    let written = as_written();
    let mut ours: BTreeSet<&str> = Role::ALL.iter().map(|role| role.key()).collect();
    ours.insert(NOT_A_DESIGN_ROLE);
    let theirs: BTreeSet<&str> = written.keys().map(String::as_str).collect();

    assert_eq!(
        theirs, ours,
        "the palette source and the shell do not name the same colours"
    );
}

/// **The source agrees with the design file**, read from the committed
/// measurement rather than from Figma.
///
/// This is the test the whole snapshot exists for. Without it, `palette.toml`
/// and `role.rs` could agree with each other perfectly while both having
/// drifted from the design — three copies in step, all of them wrong, and
/// nothing to say so on a machine without Figma access.
#[test]
fn the_source_is_the_design_files_measured_values() {
    let written = as_written();
    let measured = as_measured();

    assert_eq!(
        measured.len(),
        Role::ALL.len(),
        "the snapshot records {} light values and the shell has {} roles",
        measured.len(),
        Role::ALL.len()
    );

    for role in Role::ALL {
        let key = role.key();
        let from_the_design = measured
            .get(key)
            .unwrap_or_else(|| panic!("the committed measurement records no `{key}`"));
        let from_the_source = written
            .get(key)
            .unwrap_or_else(|| panic!("the palette source names no colour `{key}`"));

        assert!(
            from_the_design.eq_ignore_ascii_case(from_the_source),
            "`{key}` is {from_the_source} in the palette source and {from_the_design} in the \
             design file"
        );
    }
}

/// **And the only entry the design file does not account for is the one that
/// says so.**
///
/// Without this, a thirteenth colour could be added to the source with no
/// counterpart in the design and nothing would object — which is how
/// `porcelain` came to exist as a colour with four meanings.
#[test]
fn every_colour_in_the_source_is_measured_or_is_the_declared_exception() {
    let written = as_written();
    let measured = as_measured();

    let unaccounted: Vec<&str> = written
        .keys()
        .map(String::as_str)
        .filter(|name| !measured.contains_key(*name))
        .filter(|name| *name != NOT_A_DESIGN_ROLE)
        .collect();

    assert!(
        unaccounted.is_empty(),
        "the palette source names colours the design file does not, and which are not the \
         declared exception `{NOT_A_DESIGN_ROLE}`: {unaccounted:?}"
    );
}

/// **The agent's colour is the value ADR 0067 reserved.**
///
/// Asserted on its own because of how it could go wrong silently. The design
/// file's variables are not mode-coherent — the swatch *alo on dark* returns
/// `accent/default = #77C8C6` beside a light `bg/surface` — so a refresh of the
/// measurement taken from the wrong node would carry the dark teal in as the
/// agent's light colour, and every other test here would still pass.
#[test]
fn the_agents_colour_is_the_one_adr_0067_reserved() {
    let reserved = Colour::written("#0F6B72").expect("a hex this file wrote is a colour");
    let key = Role::AccentDefault.key();

    assert_eq!(Role::AccentDefault.colour(), reserved, "in the shell");
    assert_eq!(
        as_written().get(key).map(String::as_str),
        Some("#0F6B72"),
        "in the palette source"
    );
    assert_eq!(
        as_measured().get(key).map(String::as_str),
        Some("#0F6B72"),
        "in the committed measurement — #77C8C6 here is the dark mode, measured from the \
         wrong node"
    );
}

/// **The design brief is held to the source too.**
///
/// It is the copy a person reads before drawing anything, so a stale hex there
/// is a design drawn in a colour the machine does not have. The brief prints
/// each as `` `#102A43` `` in a table; this asserts the source's value appears
/// in it, rather than parsing the table's shape — the table can be reformatted
/// and this still means what it means.
#[test]
fn the_design_brief_prints_the_sources_colours() {
    let brief = std::fs::read_to_string(the_brief()).expect("the design brief is where it says");
    let brief = brief.to_ascii_uppercase();

    for (name, hex) in as_written() {
        assert!(
            brief.contains(&hex.to_ascii_uppercase()),
            "the design brief does not carry {hex} for `{name}`, so a designer is reading a \
             colour this machine does not have"
        );
    }
}

/// **A colour appears once in the source**, so there is no second place to
/// change it and forget.
///
/// Two keys with one value is not itself wrong — a palette may reuse a colour —
/// but these are distinct colours by construction, and two of them becoming
/// equal is far more likely to be a copy-paste than a decision.
#[test]
fn no_two_colours_in_the_source_are_the_same() {
    let written = as_written();
    let distinct: BTreeSet<&String> = written.values().collect();
    assert_eq!(
        distinct.len(),
        written.len(),
        "two colours in the palette source have the same value: {written:?}"
    );
}

/// **The snapshot's manifest describes the file sitting beside it.**
///
/// Not a palette question, and here because this is the only test in any crate
/// that reads the snapshot directory at all.
///
/// It is here because it already went wrong: `MANIFEST.md` recorded 2,246,045
/// bytes, 25,221 lines and 20,105 nodes; the `#489` refresh carried the owner's
/// Figma edits into `70-28.xml`; and the manifest went on describing the file as
/// it had been. **A description a file cannot contradict is this repository's
/// recurring fault**, and the cure is to let the file contradict it.
#[test]
fn the_snapshot_manifest_describes_the_file_beside_it() {
    let folder = the_repository().join("docs/design/figma-snapshot");
    let manifest =
        std::fs::read_to_string(folder.join("MANIFEST.md")).expect("the manifest is where it says");
    let exported =
        std::fs::read_to_string(folder.join("70-28.xml")).expect("the export is where it says");

    for (count, what) in [
        (exported.len(), "bytes"),
        (exported.lines().count(), "lines"),
        (exported.matches("id=\"").count(), "nodes"),
    ] {
        let claim = format!("{} {what}", with_separators(count));
        assert!(
            manifest.contains(&claim),
            "70-28.xml is {claim} and the manifest does not say so"
        );
    }
}

/// **The source's `[was]` table says where each old name went, and it is true.**
///
/// It exists for a reader holding a name from before 2026-10-04 — an older
/// document, the public contract, another repository. Until this test it was
/// prose, which is how the six-row table it replaced came to be wrong in two
/// values without anybody noticing.
///
/// `porcelain` is the one entry that does not name a role, and that is the
/// answer rather than a gap: four consumers meant four things by it, so the
/// table says `per consumer` and `Token::Porcelain` itself carries `bg/surface`.
#[test]
fn the_sources_migration_table_says_where_each_old_name_went() {
    let said = std::fs::read_to_string(the_source()).expect("the palette source is where it says");
    let read: toml::Table = said.parse().expect("the palette source is TOML");
    let was = read
        .get("was")
        .and_then(toml::Value::as_table)
        .expect("the palette source says what the old names became");

    // The spelling each name had, kebab-case, the way a person typed it.
    let named = |token| match token {
        Token::Navy => "navy",
        Token::DeepTeal => "deep-teal",
        Token::Cream => "cream",
        Token::Porcelain => "porcelain",
        Token::Charcoal => "charcoal",
        Token::WarmStone => "warm-stone",
    };

    for token in Token::ALL {
        let old = named(token);
        let became = was
            .get(old)
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| panic!("the migration table does not say what `{old}` became"));

        let expected = match token {
            // Not one role, and deliberately not mapped to one.
            Token::Porcelain => "per consumer",
            // Retained under its own name; the design file does not define it.
            Token::Charcoal => NOT_A_DESIGN_ROLE,
            other => other
                .role()
                .unwrap_or_else(|| panic!("`{old}` has no role and is not an exception"))
                .key(),
        };

        assert_eq!(
            became, expected,
            "the migration table says `{old}` became `{became}`, and it became `{expected}`"
        );
    }

    assert_eq!(
        was.len(),
        Token::ALL.len(),
        "the migration table and the names a person can pick are different lengths: {was:?}"
    );
}

/// A count the way the manifest writes it, so the comparison is against what a
/// person reads rather than against a reformatting of it.
fn with_separators(count: usize) -> String {
    let digits = count.to_string();
    let mut grouped = String::new();
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && digits.len().saturating_sub(at) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}
