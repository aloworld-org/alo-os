//! **A `pub use` of a module that only exists on Linux must be gated too**, and
//! this is the check that each one is.
//!
//! # The fault, and why no gate could see it
//!
//! `crates/alo-portals/src/lib.rs` declared `mod opening;` under
//! `#[cfg(target_os = "linux")]` and re-exported `opening::activated`
//! **unconditionally**. On Linux both lines exist and the crate compiles. On
//! Windows the module is not there and the `pub use` names nothing, so
//! `alo-portals` did not compile at all.
//!
//! It arrived in #635 and was found on **2026-10-19**, by the installer
//! candidate failing — the only build in this repository that targets Windows,
//! and one that runs on a **manual dispatch**. Every Linux gate in between was
//! green, correctly: there is no `cargo` invocation on the merge path that
//! compiles for Windows, so *main is green* has never meant *the installer
//! builds*.
//!
//! **Sweeping for the shape found a second one** that nothing had met:
//! `alo-terminal` re-exported `NotStopped` out of a Linux-only module, and
//! because nothing outside that crate names it, it broke no caller and would
//! have waited for whoever next dispatched a Windows build.
//!
//! # Why a source check rather than a Windows build on the merge path
//!
//! A Windows job would be the real answer and is not this test's to add: it is
//! minutes of a hosted runner per merge for a target nothing on the merge path
//! produces. This is the cheap half — it catches the one shape that caused both
//! instances, in milliseconds, on the machine that is already running.
//!
//! **So it is honest about what it is not.** It cannot tell you the installer
//! builds. It tells you that no re-export in this repository names a module its
//! own target does not have, which is a different and smaller claim.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unreadable repository is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// Fewer crates than this means the walk is broken, not that the repository
/// stopped having crates. Measured 2026-10-19: 120 `lib.rs` files.
const TOO_FEW_TO_BELIEVE: usize = 100;

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// The modules a file declares behind **any** `cfg`, and the `pub use` lines
/// that name a module without one.
///
/// Line-based on purpose: this is about two lines agreeing with each other, and
/// a parser would be a dependency and a second thing to be wrong.
fn re_exports_without_their_modules_gate(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut gated: Vec<&str> = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if !line.trim_start().starts_with("#[cfg(") {
            continue;
        }
        let Some(next) = lines.get(at + 1) else {
            continue;
        };
        let declared = next
            .trim_start()
            .strip_prefix("pub ")
            .unwrap_or(next.trim_start());
        if let Some(named) = declared
            .strip_prefix("mod ")
            .and_then(|rest| rest.strip_suffix(';'))
        {
            gated.push(named);
        }
    }

    let mut wrong = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        let Some(rest) = line.trim_start().strip_prefix("pub use ") else {
            continue;
        };
        let Some((named, _)) = rest.split_once("::") else {
            continue;
        };
        if !gated.contains(&named) {
            continue;
        }
        // Gated if the line above it is a `cfg`, which is how every correct one
        // in this repository is written.
        let above = at.checked_sub(1).and_then(|before| lines.get(before));
        if !above.is_some_and(|above| above.trim_start().starts_with("#[cfg(")) {
            wrong.push(format!("mod {named}, re-exported at line {}", at + 1));
        }
    }
    wrong
}

/// **No re-export in this repository names a module its target does not have.**
#[test]
fn every_re_export_of_a_gated_module_is_gated_too() {
    let repository = the_repository();
    let crates = repository.join("crates");
    let entries = std::fs::read_dir(&crates)
        .unwrap_or_else(|why| panic!("{} did not list: {why}", crates.display()));

    let mut read = 0_usize;
    let mut wrong: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let at = entry.path().join("src/lib.rs");
        let Ok(source) = std::fs::read_to_string(&at) else {
            continue;
        };
        read += 1;
        for one in re_exports_without_their_modules_gate(&source) {
            wrong.push(format!(
                "{}: {one}",
                at.strip_prefix(&repository).unwrap_or(&at).display()
            ));
        }
    }

    // The count before the findings, because a walk that found nothing reads
    // exactly like one that looked and was satisfied.
    assert!(
        read >= TOO_FEW_TO_BELIEVE,
        "only {read} crates were read, and this repository has more than that — the walk is broken"
    );
    assert!(
        wrong.is_empty(),
        "a module exists only on one target and its re-export does not, so the crate does not \
         compile anywhere else: {wrong:#?}"
    );
}

/// **And the walk can fail**, which is the only thing that makes the test above
/// worth reading.
///
/// Both instances are reconstructed as fixtures, because the real ones are
/// fixed and a check with nothing left to find is indistinguishable from one
/// that stopped looking.
#[test]
fn the_walk_catches_both_shapes_it_was_written_for() {
    let portals = "#[cfg(target_os = \"linux\")]\nmod opening;\npub use opening::activated;\n";
    assert_eq!(
        re_exports_without_their_modules_gate(portals).len(),
        1,
        "the shape that stopped alo-portals compiling for Windows was not caught"
    );

    let terminal = "#[cfg(target_os = \"linux\")]\nmod stopping;\npub use stopping::NotStopped;\n";
    assert_eq!(
        re_exports_without_their_modules_gate(terminal).len(),
        1,
        "the shape found by sweeping was not caught"
    );

    // And the corrected form passes, so the check is not simply refusing
    // everything that mentions a gated module.
    let fixed = "#[cfg(target_os = \"linux\")]\nmod opening;\n#[cfg(target_os = \"linux\")]\npub use opening::activated;\n";
    assert!(
        re_exports_without_their_modules_gate(fixed).is_empty(),
        "the corrected form is refused, so this would wall off every gated module"
    );

    // An ungated module re-exported ungated is the ordinary case and must pass.
    let ordinary = "mod measures;\npub use measures::ICON;\n";
    assert!(
        re_exports_without_their_modules_gate(ordinary).is_empty(),
        "an ordinary re-export is refused"
    );
}
