//! **This crate holds no key and signs nothing**, checked against its own source
//! rather than promised in a comment.
//!
//! `alo-image` already makes this distinction for the release workflow, and its
//! sharpest line is the name of one of its tests: *the comment saying the owner
//! signs is not signing.* The same applies here, pointing the other way — a
//! comment saying this crate does not sign is not a guarantee that it does not.
//!
//! # Why it matters more here than it looks
//!
//! The standing rule is that **a private signing key never lands on a machine an
//! agent runs on — the owner signs.** An egress attestation is the artifact most
//! likely to tempt somebody past that rule, because a statement about one machine
//! over one period is only convenient if the machine can produce it whole. The
//! moment this crate grows a `sign()`, that rule is broken by a change that will
//! look like completing the feature.
//!
//! So the refusal is mechanical: if somebody adds a signing dependency or a call
//! that reaches for a key, this test fails and they have to argue for it in the
//! open — which is where **who signs an attestation at all** belongs, since it is
//! still undecided. See this crate's own documentation for the two branches.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// What a crate that signs reaches for.
///
/// Names of things that hold or use a private key, not the word "sign" on its
/// own — this crate's documentation discusses signing at length and must be able
/// to go on doing so. The distinction is exactly `alo-image`'s: a sentence about
/// signing is not signing.
const REACHES_FOR_A_KEY: [&str; 10] = [
    "ed25519",
    "minisign",
    "openssl",
    "rustls-pemfile",
    "PrivateKey",
    "private_key",
    "SigningKey",
    "signing_key",
    "Keypair",
    "secret_key",
];

/// This crate's own directory.
fn this_crate() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Every line of Rust this crate ships, with where it came from.
fn every_line() -> Vec<(String, String)> {
    let mut found = Vec::new();
    let src = this_crate().join("src");
    let entries = fs::read_dir(&src).expect("this crate keeps its source in src/");
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".rs") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).expect("a source file this crate ships");
        for line in text.lines() {
            found.push((format!("src/{name}"), line.to_owned()));
        }
    }
    assert!(
        found.len() > 100,
        "{} line(s) were read, which is not this crate — the walk is broken rather than the \
         crate being empty",
        found.len()
    );
    found
}

/// **Nothing in this crate reaches for a key.**
#[test]
fn nothing_here_reaches_for_a_signing_key() {
    let offending: Vec<String> = every_line()
        .into_iter()
        .filter(|(_, line)| {
            REACHES_FOR_A_KEY
                .iter()
                .any(|reaching| line.contains(reaching))
        })
        .map(|(file, line)| format!("{file}: {}", line.trim()))
        .collect();

    assert!(
        offending.is_empty(),
        "this crate reaches for a signing key in {} place(s):\n\n{}\n\nThe standing rule is that \
         a private signing key never lands on a machine an agent runs on — the owner signs. And \
         **who signs an attestation at all is undecided**: either the owner signs every machine's \
         every period, which makes it a favour rather than an artifact, or the machine signs its \
         own, which puts a key on every machine. That decision belongs in an ADR, not in a crate \
         that grew a `sign()`.",
        offending.len(),
        offending.join("\n")
    );
}

/// **And it declares no dependency that could sign**, which is the other way in.
#[test]
fn it_depends_on_nothing_that_signs() {
    let manifest =
        fs::read_to_string(this_crate().join("Cargo.toml")).expect("this crate has a manifest");
    let named: Vec<&str> = REACHES_FOR_A_KEY
        .iter()
        .filter(|reaching| manifest.contains(*reaching))
        .copied()
        .collect();
    assert!(
        named.is_empty(),
        "the manifest names {named:?}. A dependency that can sign is a key this crate is one \
         line away from holding."
    );
}

/// **The check is watched catching something**, because one that has never
/// refused is one that passes on the day it stops looking.
///
/// It is given the line a crate would contain if somebody completed the feature
/// the tempting way, and must find it.
#[test]
fn the_check_finds_a_crate_that_did_sign() {
    let tempting = "    let signing_key = SigningKey::from_bytes(&read(\"/etc/alo/attest.key\")?);";
    assert!(
        REACHES_FOR_A_KEY
            .iter()
            .any(|reaching| tempting.contains(reaching)),
        "the list does not catch the most obvious way to break the rule, so it guards nothing"
    );
}

/// And a sentence about signing is not signing — this crate's documentation
/// discusses it at length and must go on being able to.
#[test]
fn writing_about_signing_is_not_signing() {
    let prose = "//! the owner signs by digest, and this crate holds no key at all";
    assert!(
        !REACHES_FOR_A_KEY
            .iter()
            .any(|reaching| prose.contains(reaching)),
        "the list catches prose, which would make the crate unable to explain itself"
    );
}
