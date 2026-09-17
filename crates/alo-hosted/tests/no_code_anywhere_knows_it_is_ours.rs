//! **No identifier, constant or branch outside this crate names alo's service.**
//!
//! [ADR 0014](../../../docs/decisions/0014-alos-own-model-is-a-provider-like-any-other.md):
//! *no default, no pre-selection and no special case anywhere in the code.* A
//! rule like that is kept by a test or it is kept by whoever remembers it, and
//! the thing about commercial pressure is that it arrives years later, with
//! somebody who was not in the room.
//!
//! So this reads the shipped source of **every crate** and fails when one of
//! them knows our service exists.
//!
//! # If this test has stopped you
//!
//! You have written code that treats alo's service as anything other than a
//! provider a person added. The fix is never to rename the constant: it is that
//! **the behaviour must be the same for every provider**. If ours needs a code
//! path, so does Mistral's; if it does not need one, neither do we.
//!
//! What it reads: `crates/*/src`, excluding this crate, which is the data.
//! Documentation is read too — a comment saying *ours is faster* is a default
//! waiting to be written.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// Ways code could come to know that one provider is ours.
const NOTHING_ELSE_MAY_SAY: [&str; 6] = [
    "api.alo.computer",
    "alo_hosted",
    "alo-hosted",
    "our own service",
    "our service",
    "alo's service",
];

#[test]
fn no_crate_but_this_one_knows_alo_sells_inference() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crates folder")
        .to_owned();
    let mut knows: Vec<String> = Vec::new();
    for source in every_shipped_source(&crates) {
        let text = std::fs::read_to_string(&source).unwrap_or_default();
        for name in NOTHING_ELSE_MAY_SAY {
            if text.to_lowercase().contains(name) {
                knows.push(format!("{} says `{name}`", source.display()));
            }
        }
    }
    assert!(
        knows.is_empty(),
        "code outside alo-hosted knows that one provider is ours, which is how a default, a \
         pre-selection or a quieter indicator begins:\n{knows:#?}\n\nThe fix is not a better \
         name. If alo's service needs this code path, so does every other provider; if it does \
         not, neither do we (ADR 0014)."
    );
}

/// Every `src` file of every crate but this one.
fn every_shipped_source(crates: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(crates) else {
        return found;
    };
    for entry in entries.flatten() {
        if entry.file_name() == "alo-hosted" {
            continue;
        }
        walk(&entry.path().join("src"), &mut found);
    }
    found
}

/// Every `.rs` under a folder.
fn walk(folder: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, found);
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            found.push(path);
        }
    }
}

/// **And this crate ships no client, no branch and no policy** — it is the
/// address and the disclosure, and the code that speaks to it is the code that
/// speaks to everybody.
#[test]
fn this_crate_is_data_and_not_code() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for file in {
        let mut found = Vec::new();
        walk(&src, &mut found);
        found
    } {
        let text = std::fs::read_to_string(&file).expect("this crate's own source");
        for machinery in [
            "ureq",
            "reqwest",
            "TcpStream",
            "std::net",
            "impl Asking",
            "fn ask",
        ] {
            assert!(
                !text.contains(machinery),
                "{} carries `{machinery}`; alo's service is spoken to by the code that speaks to \
                 every provider, or it is a special case",
                file.display()
            );
        }
    }
}
