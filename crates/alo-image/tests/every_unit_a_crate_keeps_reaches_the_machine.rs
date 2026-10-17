//! **A unit a crate keeps is a unit the image ships — or is named here as one it
//! deliberately does not.**
//!
//! # The direction nothing checked
//!
//! `the_image_starts_what_it_ships.rs` asks, of every unit **in `image/`**,
//! whether anything starts it and whether its `ExecStart` has a file. Its own
//! header records two units reaching `main` that nothing started, and the
//! convention it fixed.
//!
//! It cannot see this fault, because it only looks at units the image already
//! has. **A unit a crate keeps and the image never copies is invisible to it**,
//! and so is the binary that unit would have run.
//!
//! Measured 2026-10-17: **eight unit files live in three crates and the image
//! ships none of them.** `alo-letting-go` is four of the eight, and it is the
//! clearest case of what that costs — ADR 0045 promises that what an undo keeps
//! is let go of on a daily timer, and *the window* and *the person's screen for
//! it* both ship, while the half that does the letting go does not. A machine
//! built today would keep every snapshot for ever, and the symptom is a disk
//! filling quietly rather than anything failing.
//!
//! # Why a list and not a failure
//!
//! Because whether those three components should ship is a decision and not a
//! bug to be fixed by a test, and shipping three privileged services is a change
//! to what runs on somebody's machine.
//!
//! So each one is named below with what it is waiting on. **What this test buys
//! is that the set cannot grow in silence**: a ninth unit written in a crate and
//! not shipped fails here until a person says which it is. An absence nobody
//! listed is the thing that cost this repository two releases in the other
//! direction.
//!
//! # It needs no image build
//!
//! Both halves are files on disk, read as text, so this runs in the ordinary
//! gates — which is the point. The image build is the thing nobody runs until a
//! release, and that is how a gap like this survives.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// Where the image keeps the units it ships.
const SHIPPED: &str = "image/usr/lib/systemd/system";

/// What a systemd unit file is called, as far as this test is concerned.
const A_UNIT_IS: [&str; 4] = ["service", "timer", "path", "socket"];

/// Units a crate keeps that the image deliberately does not ship yet, and what
/// each is waiting on.
///
/// **Every line here is a decision somebody has to make, not a fault to fix.**
/// Shipping a privileged service changes what runs on a machine, so none of
/// these is added by a lane that merely noticed it.
const NOT_YET_SHIPPED: [(&str, &str); 8] = [
    (
        "alo-letting-go.service",
        "ADR 0045 term 7's privileged remover. Its window and the person's screen \
         for it both ship; this does not, so nothing lets go of a snapshot. \
         Waiting on: whether to ship it now, when nothing yet takes a bracket, or \
         with the bracket-taking it exists for.",
    ),
    (
        "alo-letting-go.timer",
        "What makes the remover daily. Ships or not with the service above.",
    ),
    (
        "alo-forgetting.service",
        "The one act that forgets everything a machine was keeping for a person \
         (ADR 0045 point 5). Ships with the remover.",
    ),
    (
        "alo-forgetting.path",
        "What notices the person asked. Ships with the act above.",
    ),
    (
        "alo-brokerd.service",
        "The broker that carries out a change to the whole machine (ADR 0001 §2). \
         Waiting on: whether the broker is meant to ship in this release.",
    ),
    (
        "alo-applying-an-update.service",
        "The broker's half that applies an update. Ships with the broker.",
    ),
    (
        "alo-going-back.service",
        "The broker's half that goes back to the build before. Ships with the \
         broker.",
    ),
    (
        "alo-looking-once.service",
        "Waiting on: whether this component is meant to ship in this release.",
    ),
];

/// The repository's own root, from this crate's manifest.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("this crate sits in the repository that holds the image")
}

/// Every unit file kept beside a crate, as `(crate, file name)`.
fn units_kept_in_crates(root: &Path) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let crates = root.join("crates");
    for entry in std::fs::read_dir(&crates).expect("this repository has a crates directory") {
        let at = entry.expect("a directory entry").path();
        if !at.is_dir() {
            continue;
        }
        let Some(whose) = at.file_name().map(|it| it.to_string_lossy().into_owned()) else {
            continue;
        };
        for file in std::fs::read_dir(&at).expect("a crate is a directory") {
            let file = file.expect("a directory entry").path();
            let is_a_unit = file
                .extension()
                .is_some_and(|it| A_UNIT_IS.iter().any(|kind| it == *kind));
            if is_a_unit {
                let named = file
                    .file_name()
                    .expect("a file that was read has a name")
                    .to_string_lossy()
                    .into_owned();
                found.push((whose.clone(), named));
            }
        }
    }
    found.sort();
    found
}

/// **Every unit a crate keeps is shipped, or is named as one that is not.**
#[test]
fn every_unit_a_crate_keeps_is_shipped_or_named_as_not() {
    let root = the_repository();
    let kept = units_kept_in_crates(&root);

    // **The control.** A walk that found nothing would satisfy every assertion
    // below, and this test would be green about a repository it never read.
    assert!(
        kept.len() >= 8,
        "only {} unit files were found beside crates, and there were eight on \
         2026-10-17 — so this test is reading less than it thinks",
        kept.len()
    );

    // **The other control**: the comparison has to be able to say yes. If the
    // shipped directory could not be read, everything would read as missing.
    let units = root.join(SHIPPED);
    let shipping = std::fs::read_dir(&units)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", units.display()))
        .count();
    assert!(
        shipping >= 10,
        "the image ships {shipping} units, and it shipped twelve on 2026-10-17 — so a \
         missing unit below might be a directory this test could not read"
    );

    let mut unlisted = Vec::new();
    for (whose, named) in &kept {
        if units.join(named).exists() {
            continue;
        }
        if NOT_YET_SHIPPED.iter().any(|(listed, _)| listed == named) {
            continue;
        }
        unlisted.push(format!("{whose} keeps {named}"));
    }

    assert!(
        unlisted.is_empty(),
        "a unit lives beside a crate and the image neither ships it nor names it as one \
         it deliberately does not:\n  {}\n\n\
         **A unit the image does not copy is a unit nothing on a machine can start**, and \
         `the_image_starts_what_it_ships.rs` cannot see it — that test reads the units in \
         `image/` and this one is not there. Either add it to `image/usr/lib/systemd/system/` \
         and to the recipe's `systemctl enable` list, or add it to `NOT_YET_SHIPPED` in this \
         file with what it is waiting on.",
        unlisted.join("\n  ")
    );
}

/// **Nothing in the not-yet-shipped list has quietly started shipping.**
///
/// The other half, and the reason the list is not a comment: an excuse that has
/// stopped being true is an excuse somebody will read as current. The same rule
/// the gate's own declared-failures file is held to.
#[test]
fn nothing_named_as_unshipped_is_shipping_after_all() {
    let units = the_repository().join(SHIPPED);
    let mut shipping = Vec::new();
    for (named, _) in NOT_YET_SHIPPED {
        if units.join(named).exists() {
            shipping.push(named);
        }
    }
    assert!(
        shipping.is_empty(),
        "these are named in `NOT_YET_SHIPPED` and the image ships them after all, so the \
         reason beside each is no longer true and should go: {shipping:?}"
    );
}

/// **Every unit named as unshipped still exists to be shipped.**
///
/// A name that no longer matches a file is a line nobody will delete, and a list
/// of eight where seven are real reads exactly like a list of eight.
#[test]
fn every_unit_named_as_unshipped_is_a_unit_that_exists() {
    let root = the_repository();
    let kept = units_kept_in_crates(&root);
    let mut gone = Vec::new();
    for (named, _) in NOT_YET_SHIPPED {
        if !kept.iter().any(|(_, it)| it == named) {
            gone.push(named);
        }
    }
    assert!(
        gone.is_empty(),
        "these are named in `NOT_YET_SHIPPED` and no crate keeps them any more, so the \
         lines are about nothing: {gone:?}"
    );
}

/// **Each one says what it is waiting on**, because a list of names is a list
/// nobody can act on.
#[test]
fn each_unshipped_unit_says_what_it_waits_on() {
    for (named, waiting) in NOT_YET_SHIPPED {
        assert!(
            waiting.len() > 40,
            "{named} is named as unshipped and says too little about why: {waiting:?}"
        );
    }
}
