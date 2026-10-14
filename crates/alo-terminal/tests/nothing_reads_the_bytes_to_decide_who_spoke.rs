//! **The guarantee of ADR 0113 is an absence, and an absence needs a guard.**
//!
//! `who_is_speaking.rs` promises that an origin can never be obtained from
//! text: no parser, no `FromStr`, no constructor taking a string, no heuristic
//! over output. A program can print `system`, and the gutter still says
//! `program`.
//!
//! Nothing in the type system states that. The promise is kept by a function
//! *not existing*, and the failure mode is somebody adding it in good faith a
//! year from now — a `WhoIsSpeaking::of(line: &str)` looks helpful and would
//! quietly hand every program the ability to label itself as alo.
//!
//! So this reads the module's own source and refuses a signature that could
//! see text.
//!
//! # Why this is a source test and not a behaviour test
//!
//! Because there is no behaviour to test. A function that does not exist
//! cannot be called wrongly; it can only be added. `CLAUDE.md` asks what would
//! have to be true for a green check to pass while the product is broken, and
//! for every behavioural test of this module the answer is *somebody added a
//! second road and tested the first one* — which is exactly what this catches
//! and nothing else does.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unreadable module is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;

/// The module whose surface is held to the absence.
fn the_module() -> String {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "src", "who_is_speaking.rs"]
        .iter()
        .collect();
    fs::read_to_string(&path)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", path.display()))
}

/// Every function signature in the module, as one line each.
///
/// Signatures only: a doc comment may say the word `&str` while discussing the
/// rule, and refusing that would make the module undocumentable.
fn the_signatures(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim_start)
        .filter(|line| {
            (line.starts_with("pub fn ")
                || line.starts_with("fn ")
                || line.starts_with("pub const fn ")
                || line.starts_with("const fn "))
                && !line.starts_with("//")
        })
        .map(ToOwned::to_owned)
        .collect()
}

/// **The search works**, so an empty answer below means absence and not a
/// query that could never have matched.
///
/// Without this, deleting the module, renaming it, or breaking the filter
/// would all read as the guarantee being kept.
#[test]
fn the_search_finds_what_is_there() {
    let source = the_module();
    assert!(
        source.len() > 2000,
        "the module is {} bytes, which is not the file this test is about",
        source.len()
    );
    let signatures = the_signatures(&source);
    assert!(
        signatures.len() >= 4,
        "found {} signatures, so the filter is not reading this file: {signatures:#?}",
        signatures.len()
    );
    // A positive control: `regions_of` takes a count, and the search can see
    // its parameter. A search that cannot see this one could not see a `&str`
    // either.
    assert!(
        signatures
            .iter()
            .any(|line| line.contains("regions_of") && line.contains("usize")),
        "the search cannot see the one parameter this crate does take: {signatures:#?}"
    );
}

/// **No function here can be handed text.**
///
/// The guard itself. Any parameter that is a string is a road from what a
/// program printed to what the gutter says, however it is named.
#[test]
fn no_signature_in_the_module_takes_text() {
    let source = the_module();
    let offending: Vec<String> = the_signatures(&source)
        .into_iter()
        .filter(|line| {
            line.contains("&str")
                || line.contains("String")
                || line.contains("&[u8]")
                || line.contains("impl AsRef<str>")
                || line.contains("Cow<")
        })
        .collect();

    assert!(
        offending.is_empty(),
        "ADR 0113 says origin is where bytes came from and never what they say, so nothing \
         here may take text. These signatures can:\n{offending:#?}\n\nIf a region really must \
         be decided from a source rather than a count, give that road its own named type \
         carrying where the bytes entered — never the bytes."
    );
}

/// **And no trait that turns text into an origin is implemented.**
///
/// `FromStr` is the one that would slip past the signature filter most easily,
/// because its parameter sits on the trait rather than on the `impl` line.
///
/// **Matched on the trait's name anywhere in an `impl` line, not on a literal
/// `impl FromStr`.** The first version of this test looked for that literal and
/// a planted `impl core::str::FromStr` walked straight through it — the
/// signature test caught that one, so the hole was invisible until the guard
/// was deliberately attacked. A path-qualified `impl` is the obvious way this
/// would really arrive.
#[test]
fn no_trait_turns_text_into_an_origin() {
    let source = the_module();
    let implementations: Vec<&str> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("impl ") || line.starts_with("unsafe impl "))
        .collect();

    for line in &implementations {
        for forbidden in [
            "FromStr",
            "From<&str",
            "From<String",
            "TryFrom<&str",
            "TryFrom<String",
        ] {
            assert!(
                !line.contains(forbidden),
                "`{line}` implements {forbidden}, which makes text into an origin and ADR 0113 \
                 forbids it"
            );
        }
    }
}

/// **The trait guard refuses a path-qualified implementation**, which is the
/// hole the first version of it had.
///
/// A guard that has never refused anything is a guard nobody has tested, so
/// the refusal is exercised here against text rather than against the file —
/// the same filter, given a line it must reject.
#[test]
fn the_trait_guard_refuses_a_qualified_path() {
    for planted in [
        "impl FromStr for WhoIsSpeaking {",
        "impl core::str::FromStr for WhoIsSpeaking {",
        "impl std::str::FromStr for WhoIsSpeaking {",
        "impl From<&str> for WhoIsSpeaking {",
        "impl TryFrom<String> for WhoIsSpeaking {",
    ] {
        let caught = [
            "FromStr",
            "From<&str",
            "From<String",
            "TryFrom<&str",
            "TryFrom<String",
        ]
        .iter()
        .any(|forbidden| planted.contains(forbidden));
        assert!(caught, "the guard would let `{planted}` through");
    }

    // And a control: an implementation that is not a road from text must not
    // be refused, or the guard would forbid ordinary code.
    for allowed in [
        "impl WhoIsSpeaking {",
        "impl ARegion {",
        "impl From<ARegion> for usize {",
    ] {
        let caught = [
            "FromStr",
            "From<&str",
            "From<String",
            "TryFrom<&str",
            "TryFrom<String",
        ]
        .iter()
        .any(|forbidden| allowed.contains(forbidden));
        assert!(!caught, "the guard wrongly refuses `{allowed}`");
    }
}
