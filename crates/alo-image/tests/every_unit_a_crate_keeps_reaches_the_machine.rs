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
//! # The kind nothing checked either
//!
//! Until 2026-10-17 this file knew four extensions — `.service`, `.timer`,
//! `.path` and `.socket` — and `alo-letting-go` keeps a **fifth**:
//! `alo-forgetting.conf`, a `tmpfiles.d` file. It was therefore checked by
//! nothing at all, in the same way and for the same reason the units were.
//!
//! It is not a tidy-up. `alo-forgetting.path` watches
//! `/run/alo/asked-to-forget` with `DirectoryNotEmpty=`, and **that directory is
//! created by nothing else in this repository**. Ship the four units without the
//! `.conf` and systemd watches a path that never appears: a unit that loads,
//! that activates, and that can never fire. Nothing fails, nothing is logged,
//! and the person's *forget everything you are keeping for me* does nothing for
//! ever.
//!
//! So a file's **destination is now a function of its extension**
//! ([`where_it_goes`]), there are two directories rather than one, and
//! `every_kind_of_installable_file_a_crate_keeps_is_known` fails on a kind this
//! file has not been taught — a `.rules` for udev, a `.network`, a `.preset`.
//! Each would have been this fault again, and each would have been silent,
//! because **a file nobody looks for cannot be reported missing**.
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

/// Where the image keeps the `tmpfiles.d` files it ships.
///
/// **A second destination, because a crate can keep a second kind of thing.**
/// `alo-letting-go/alo-forgetting.conf` is a `tmpfiles.d` file and is every bit
/// as load-bearing as the units beside it: `alo-forgetting.path` watches
/// `/run/alo/asked-to-forget` with `DirectoryNotEmpty=`, and that directory is
/// created by **nothing else in this repository**. Ship the units without it and
/// systemd watches a path that never appears — a unit that loads, activates, and
/// can never fire. Silent, and invisible to every check there was.
const SHIPPED_TMPFILES: &str = "image/usr/lib/tmpfiles.d";

/// What a systemd unit file is called, as far as this test is concerned.
const A_UNIT_IS: [&str; 4] = ["service", "timer", "path", "socket"];

/// What a `tmpfiles.d` file is called.
///
/// One extension, and a crate keeps exactly one such file today. It is separate
/// from [`A_UNIT_IS`] rather than folded into it because the two go to
/// **different directories**, and a list that mixed them would need the
/// destination worked out at the comparison instead of here.
const A_TMPFILES_FILE_IS: [&str; 1] = ["conf"];

/// Where a file a crate keeps has to be copied to, by its extension.
///
/// [`None`] for anything else, which is how a crate's `Cargo.toml`, its
/// `README.md` and its source are passed over. **The one thing this must never
/// do is answer a directory for a file whose kind nobody has thought about** —
/// so a new extension is not shipped by default, it is simply not a thing this
/// test knows, and `every_kind_of_installable_file_a_crate_keeps_is_known`
/// is what notices.
fn where_it_goes(extension: &str) -> Option<&'static str> {
    if A_UNIT_IS.contains(&extension) {
        Some(SHIPPED)
    } else if A_TMPFILES_FILE_IS.contains(&extension) {
        Some(SHIPPED_TMPFILES)
    } else {
        None
    }
}

/// Units a crate keeps that the image deliberately does not ship yet, and what
/// each is waiting on.
///
/// **Every line here is a decision somebody has to make, not a fault to fix.**
/// Shipping a privileged service changes what runs on a machine, so none of
/// these is added by a lane that merely noticed it.
const NOT_YET_SHIPPED: [(&str, &str); 5] = [
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
        "alo-forgetting.conf",
        "The `tmpfiles.d` line that makes /run/alo/asked-to-forget, which \
         alo-forgetting.path watches with DirectoryNotEmpty= and which nothing \
         else in this repository creates. **It must not ship later than the path \
         unit, and it must not be forgotten when that one ships**: a path unit \
         watching a directory that never appears loads, activates and can never \
         fire. Ships with the four above.",
    ),
];

/// The repository's own root, from this crate's manifest.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("this crate sits in the repository that holds the image")
}

/// Every installable file kept beside a crate, as `(crate, file name)`.
///
/// Units and `tmpfiles.d` files both, because both have to be copied onto the
/// image and neither is copied by anything automatic.
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
            let is_installable = file
                .extension()
                .and_then(|it| it.to_str())
                .and_then(where_it_goes)
                .is_some();
            if is_installable {
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

    // **The control: that the walk read the repository**, not that it found a
    // particular number of units.
    //
    // *It counted units until 2026-10-17, and four of them shipped that day —
    // so the control failed on the very change it should have been indifferent
    // to.* A number that has to be edited whenever the thing it measures
    // legitimately moves is not a control; it is a second copy of the answer.
    //
    // What makes finding nothing impossible is that the walk looked at the
    // crates, and there are a hundred of those whatever happens to any unit.
    // Each unit named below is separately held to existing, by
    // `every_unit_named_as_unshipped_is_a_unit_that_exists`, so the lower bound
    // this used to assert is covered where it belongs.
    let crates = std::fs::read_dir(root.join("crates"))
        .expect("this repository has a crates directory")
        .count();
    assert!(
        crates >= 50,
        "only {crates} crates were walked, so this test is reading less of the repository \
         than it thinks"
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
        // Where this one goes, which is not the same directory for every kind.
        let goes = Path::new(named)
            .extension()
            .and_then(|it| it.to_str())
            .and_then(where_it_goes)
            .unwrap_or_else(|| {
                panic!(
                    "{whose} keeps {named}, and the walk should not have \
                                       returned a file it has no destination for"
                )
            });
        if root.join(goes).join(named).exists() {
            continue;
        }
        if NOT_YET_SHIPPED.iter().any(|(listed, _)| listed == named) {
            continue;
        }
        unlisted.push(format!("{whose} keeps {named}, which belongs in {goes}"));
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
    let root = the_repository();
    let mut shipping = Vec::new();
    for (named, _) in NOT_YET_SHIPPED {
        let goes = Path::new(named)
            .extension()
            .and_then(|it| it.to_str())
            .and_then(where_it_goes)
            .unwrap_or_else(|| {
                panic!(
                    "{named} is named as unshipped and is not a kind of file \
                                       this test knows where to put"
                )
            });
        if root.join(goes).join(named).exists() {
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

/// **Every kind of installable file a crate keeps is a kind this test knows
/// about.**
///
/// The gap this closes, and the reason it is a test rather than a wider
/// `A_UNIT_IS`. Until 2026-10-17 this file knew four extensions, and
/// `alo-letting-go` kept a fifth — a `tmpfiles.d` `.conf` — which was therefore
/// not checked by anything at all. A `.rules` file for udev, a `.network` for
/// systemd-networkd, a `.preset`: each would be the same fault again, and each
/// would be **silent**, because a file nobody looks for cannot be reported
/// missing.
///
/// So this walks a crate's own directory for every extension that *systemd and
/// its neighbours install from*, and fails on one [`where_it_goes`] has no
/// answer for. The failure says *add the extension and its directory*, which is
/// a two-line change, rather than leaving the next person to discover the file
/// was never shipped.
///
/// **Why a list of extensions rather than every file.** A crate directory also
/// holds `Cargo.toml`, `README.md` and sometimes a fixture, and none of those
/// goes on the image. The list below is *things a machine installs from a
/// directory*, which is a closed set that moves slowly, and a file outside it is
/// deliberately not this test's business.
#[test]
fn every_kind_of_installable_file_a_crate_keeps_is_known() {
    /// Extensions that some part of a Linux machine installs from a directory
    /// and reads at boot. Anything here that `where_it_goes` cannot place is a
    /// file that would ship nowhere and be missed by everything.
    const INSTALLED_FROM: [&str; 11] = [
        "service", "timer", "path", "socket", "target", "mount", "slice", "conf", "rules",
        "network", "link",
    ];

    let root = the_repository();
    let crates = root.join("crates");
    let mut unknown = Vec::new();
    let mut looked_at = 0_usize;

    for entry in std::fs::read_dir(&crates).expect("this repository has a crates directory") {
        let at = entry.expect("a directory entry").path();
        if !at.is_dir() {
            continue;
        }
        looked_at += 1;
        for file in std::fs::read_dir(&at).expect("a crate is a directory") {
            let file = file.expect("a directory entry").path();
            let Some(extension) = file.extension().and_then(|it| it.to_str()) else {
                continue;
            };
            if INSTALLED_FROM.contains(&extension) && where_it_goes(extension).is_none() {
                unknown.push(format!(
                    "{} keeps {}",
                    at.file_name().unwrap_or_default().to_string_lossy(),
                    file.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }
    }

    assert!(
        looked_at >= 50,
        "only {looked_at} crates were walked, so finding nothing below meant nothing"
    );
    assert!(
        unknown.is_empty(),
        "a crate keeps a kind of installable file this test does not know where to put, so \
         nothing checks whether it ships:\n  {}\n\n\
         Add the extension to `A_UNIT_IS` or `A_TMPFILES_FILE_IS` — or to a new list with its \
         own destination — and teach `where_it_goes` about it. Until then the file is invisible \
         to every check in this repository.",
        unknown.join("\n  ")
    );
}

/// **The control**: `where_it_goes` places what it should and refuses what it
/// should not.
///
/// Without this, a `where_it_goes` that answered `Some(SHIPPED)` for everything
/// would make the test above pass with nothing to report, and would also make
/// the walk return a crate's `Cargo.toml` as a unit. Both directions are
/// asserted, and the two destinations are asserted to be **different** — one
/// that answered the same directory for both kinds would put a `tmpfiles.d`
/// file among the units and look fine.
#[test]
fn where_each_kind_goes_is_known_and_the_two_places_differ() {
    assert_eq!(where_it_goes("service"), Some(SHIPPED));
    assert_eq!(where_it_goes("timer"), Some(SHIPPED));
    assert_eq!(where_it_goes("path"), Some(SHIPPED));
    assert_eq!(where_it_goes("socket"), Some(SHIPPED));
    assert_eq!(where_it_goes("conf"), Some(SHIPPED_TMPFILES));

    for not_installed in ["toml", "md", "rs", "json", "lock", ""] {
        assert_eq!(
            where_it_goes(not_installed),
            None,
            "{not_installed:?} is not something the image installs from a directory"
        );
    }

    assert_ne!(
        SHIPPED, SHIPPED_TMPFILES,
        "the two destinations are the same directory, so one kind is being checked against the \
         wrong place"
    );
}

/// **And both destinations exist, so a missing file is a missing file.**
///
/// The other control, and the one the units half already has for its own
/// directory: if `image/usr/lib/tmpfiles.d` could not be read, every
/// `tmpfiles.d` file a crate keeps would read as unshipped — which is the
/// answer this test gives when something is genuinely wrong, so the two must
/// not be confusable.
#[test]
fn both_places_the_image_installs_to_exist() {
    let root = the_repository();
    for (what, where_to, known) in [
        (SHIPPED, "units", "alo-sessiond.service"),
        (SHIPPED_TMPFILES, "tmpfiles.d files", "alo.conf"),
    ] {
        let at = root.join(what);
        assert!(
            at.is_dir(),
            "the image's {where_to} directory is not there: {}",
            at.display()
        );
        assert!(
            at.join(known).is_file(),
            "{} does not hold {known}, so it is not the directory this test thinks it is",
            at.display()
        );
    }
}
