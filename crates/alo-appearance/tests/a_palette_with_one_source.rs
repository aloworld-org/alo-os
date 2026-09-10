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
//! `alo_appearance::Token` carries the six as Rust constants. A designer reads
//! `docs/design/figma-brief.md`, which prints them as a table. Both are copies,
//! both are legitimate, and both are exactly how a palette silently drifts —
//! somebody corrects one hex and the other two keep the old one for a year.
//!
//! So the copies stay and the **drift** goes: change a value in the source and
//! this test names every place that has not caught up.
//!
//! # What it does not do
//!
//! It does not read `alo-workplace/web/src/ds/tokens.css`, which is in another
//! repository and not present beside this checkout. Ending CSS's authority over
//! the palette is what this file establishes *here*; the web client adopting the
//! generated custom properties is that repository's work, and until it does, its
//! stylesheet is a fourth copy nothing checks. That is stated in the report
//! rather than implied by silence.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#![expect(
    clippy::panic,
    reason = "a missing or malformed colour names itself in the failure, which an unwrap could not"
)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use alo_appearance::{Colour, Token};

/// The shape of palette this test reads.
const THE_FORMAT: i64 = 1;

/// Where the source lives, from this crate.
fn the_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/design/palette.toml")
}

/// The design brief, which prints the same table for a person to read.
fn the_brief() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/design/figma-brief.md")
}

/// Every colour the source names, by the key it names it under.
///
/// Parsed with the same crate the rest of this repository reads TOML with, and
/// the format number is answered first — a source written for a later alo OS is
/// refused rather than half-read, which is the rule `record-file.md` and
/// `machine-description.md` both state.
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

/// What the source calls this token.
///
/// Kebab-case, the way every other key a person types in this repository is
/// spelled — `warm-stone`, not `WarmStone`. The mapping is written out rather
/// than derived from the variant name, because a derivation would silently
/// rename a colour the day somebody renamed a variant.
fn named(token: Token) -> &'static str {
    match token {
        Token::Navy => "navy",
        Token::Terracotta => "terracotta",
        Token::Cream => "cream",
        Token::Porcelain => "porcelain",
        Token::Charcoal => "charcoal",
        Token::WarmStone => "warm-stone",
    }
}

/// **Every colour the shell is built out of is the colour the source states.**
///
/// The one that matters: `Token::colour` is what a frame is actually drawn
/// with, and this is what stops it being a hand-typed hex nobody rechecked.
#[test]
fn the_shells_colours_are_the_sources_colours() {
    let written = as_written();

    for token in Token::ALL {
        let key = named(token);
        let said = written
            .get(key)
            .unwrap_or_else(|| panic!("the palette source names no colour `{key}`"));
        let from_the_source = Colour::written(said)
            .unwrap_or_else(|_| panic!("`{said}` in the palette source is not a colour"));

        assert_eq!(
            token.colour(),
            from_the_source,
            "`{key}` is {} in the shell and {said} in the palette source",
            token.colour()
        );
    }
}

/// **And the source names exactly these six** — no colour the shell has never
/// heard of, and none of the shell's missing.
///
/// The half the test above cannot see. Without it, a colour could be added to
/// the source, be used by the web client, and never exist in the operating
/// system — which is the same drift running the other way.
#[test]
fn the_source_names_exactly_the_colours_the_shell_has() {
    let written = as_written();
    let ours: std::collections::BTreeSet<&str> = Token::ALL.iter().copied().map(named).collect();
    let theirs: std::collections::BTreeSet<&str> = written.keys().map(String::as_str).collect();

    assert_eq!(
        theirs, ours,
        "the palette source and the shell do not name the same colours"
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
/// but these six are six distinct colours by construction, and two of them
/// becoming equal is far more likely to be a copy-paste than a decision.
#[test]
fn no_two_of_the_six_are_the_same_colour() {
    let written = as_written();
    let distinct: std::collections::BTreeSet<&String> = written.values().collect();
    assert_eq!(
        distinct.len(),
        written.len(),
        "two colours in the palette source have the same value: {written:?}"
    );
}
