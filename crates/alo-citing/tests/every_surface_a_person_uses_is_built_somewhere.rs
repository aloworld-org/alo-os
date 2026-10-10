//! **Every surface a person is promised is built somewhere a machine can reach
//! it** — or is listed here as one that is not, with its reason.
//!
//! # The fault this exists for, and why no other test can see it
//!
//! Twice on 2026-10-05 a promise turned out to be blocked by code that was
//! finished, correct and thoroughly tested, and that nothing on a machine ever
//! brought into being:
//!
//! - `alo_remembering::whose` — `docs/decisions/0088-a-machines-grants-belong-
//!   to-a-person.md` calls it *correct code with no production caller*.
//! - `alo-shell`'s `SettingsWindow` — `the-shell-plan.md` task 18: *every
//!   construction of it sits in a test file or a `#[cfg(test)]` block*.
//!
//! Both passed every test their crates had. **A test suite cannot see this,
//! because the tests are the callers.** 8867 tests passing says nothing about
//! whether a machine can take a screenshot.
//!
//! # The question it asks
//!
//! Outside a type's own file, and outside every test, does anything write
//! `Type::` at all? That is how a type is built or reached in Rust, so a type
//! whose `Type::` appears only in tests is a type only tests make.
//!
//! An earlier instrument asked whether a reachable file *named* the type, and
//! got `SettingsWindow` wrong: a reachable file names it in a signature while
//! nothing constructs one. **Naming is not using**, and the difference is the
//! whole bug.
//!
//! # Why the debt is listed rather than merely forbidden
//!
//! Four surfaces are unreachable today. Forbidding that outright would mean
//! this check could not land until somebody fixed four crates across three
//! lanes. So each is named with its reason, and the list is held in **both**
//! directions: a surface that is still unreachable keeps its entry, and one
//! that somebody wires in **fails this test** until its entry is removed. Debt
//! cannot be paid silently and cannot grow silently.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// Whether a machine can reach a surface today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reach {
    /// Something outside a test builds one. This is the ordinary state.
    AMachineCan,
    /// Only a test ever builds one, with the reason and whose it is.
    OnlyATestDoes {
        /// Why, in the words of whatever measured it.
        why: &'static str,
    },
}

/// The surfaces a person is promised, and whether a machine can reach each.
///
/// Deliberately a short, named list rather than a sweep of all 1,547 public
/// types. A sweep reports error types reached through `?`, which never name
/// themselves — noise that would make this check unreadable and therefore
/// unread. These are things `docs/features.md` promises a person can *do*.
const EVERY_SURFACE_A_PERSON_USES: [(&str, Reach); 7] = [
    // **Paid, 2026-10-10**, and the entry it replaces called itself *the one
    // entry here that must not be 'fixed'*. It was right on 2026-10-05 and its
    // reason had gone stale twice over by the time the draw reached it:
    //
    // - *two of them can be laid out today* — all four lay out now. The owner
    //   ruled on 2026-10-10 that a side dock names an icon in a tooltip beside
    //   it rather than under it, so `A_NAME_BESIDE_AN_ICON` is the measurement
    //   whose absence was the whole obstacle, and the difference the old entry
    //   described stopped existing.
    // - *crate::layout::NotLaidOut is that difference as a type* — that type is
    //   gone, deleted in the same change that gave the four edges a layout. An
    //   entry here citing a type nobody can build is a reason that reads as
    //   current and cannot be checked.
    //
    // **And what it feared is not what wiring this did.** *Wiring this would
    // offer a person two edges that do not lay out* guarded a promise, and the
    // edge still reaches no promise: it lives on `Shipped` and deliberately not
    // on `Changes`, so there is no setting and nobody is offered anything. The
    // production path that builds one is the draw — `dock_raster` placing the
    // bar on the edge it is given, and `egress_status_place` reserving a side
    // dock's lane and stacking away from it. That is *making an existing
    // promise true*, which is the half of the scope rule that needs no asking.
    //
    // The owner's authorisation of 2026-10-04 is unchanged and still binding;
    // it is the setting it withholds, not the type.
    //
    // **And this entry now answers for two different types, which is a
    // weakness of the instrument rather than of the entry.** `builds_one`
    // matches a bare name with identifier boundaries, across every crate, so
    // `alo_dock::Edge` and `alo_shell::window_edge::Edge` — added by the
    // external window edge, a wholly unrelated type — are indistinguishable
    // to it. Measured 2026-10-10: this same test failed on **both** pull
    // requests 603 and 607 in the same hour, for two different `Edge`s, and
    // one change to this line cleared both.
    //
    // The consequence to know about: if the Dock's edge were ever unwired,
    // this would go on reading `AMachineCan` because the window edge builds
    // one. This file's own note above records two earlier versions of exactly
    // this fault — `Edge::` matching `FrameEdge::Top`, then a lost leading
    // `::` — and both were about *one* type matched wrongly. This is two
    // types matched as one, which no boundary rule can fix: the name is
    // genuinely ambiguous and the entry would have to carry a crate to stop
    // being so. Left as it is, because adding a path to one entry while six
    // others are bare would make the list look more precise than it is.
    ("Edge", Reach::AMachineCan),
    // **Paid, 2026-10-05.** This branch was written while it read
    // `OnlyATestDoes`, with the reason *Super+I is shipped, declared, routed and
    // dispatched, and settings_command.rs takes the window as a parameter —
    // nothing on a running machine holds one to pass it*. It was paid before
    // this branch landed, so **the entry this rebase kept is the one from main,
    // not the one this commit was written with**: `Server` holds a window now.
    //
    // The list is what said so. The entry failed the moment production built
    // one, which is this check working in the direction that matters — debt
    // cannot be paid silently any more than it can grow silently.
    ("SettingsWindow", Reach::AMachineCan),
    (
        "Screenshot",
        Reach::OnlyATestDoes {
            why: "capture-and-the-room-plan.md is 7 of 7 and ROADMAP.md ticks \
                  Capture, and every Screenshot::of is inside taking.rs's own \
                  test module. Outside it, only doc comments. Measured 2026-10-05",
        },
    ),
    (
        "Recording",
        Reach::OnlyATestDoes {
            why: "the other half of the same tick: Recording::of appears in \
                  recording.rs's test module and two test files, and nowhere a \
                  machine runs. Measured 2026-10-05",
        },
    ),
    // **Paid on 2026-10-08 by task 11 of `more-than-one-display-plan.md`.**
    // `alo-desktop`'s `their_displays` answers with a `NightLight::as_shipped()`
    // resolved against that machine's own clock, so a running machine builds
    // one. The entry failed the moment it did — this check refuses in both
    // directions, and it is the only proof that production reaches this rather
    // than that it could.
    ("NightLight", Reach::AMachineCan),
    // The two below are the check's own control. If this instrument ever stops
    // seeing a production caller that is plainly there, it has broken, and
    // these fail rather than the list above quietly growing.
    ("Moved", Reach::AMachineCan),
    ("ThisMachine", Reach::AMachineCan),
];

/// This repository's crates.
fn the_crates() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .expect("the crates directory is beside this one")
}

/// Whether a path is a test rather than something a machine runs.
fn is_a_test(path: &Path) -> bool {
    let said = path.to_string_lossy().replace('\\', "/");
    [
        "/tests/",
        "_tests.rs",
        "/testing.rs",
        "_testing.rs",
        "/unit_fixtures/",
        "/examples/",
    ]
    .iter()
    .any(|mark| said.contains(mark))
}

/// A file's production text: cut at its first `#[cfg(test)]`, comments removed.
///
/// Cutting at the test module is right whenever that module is last in the
/// file, which is this repository's consistent habit. It can only **hide**
/// production text, never invent it, so the error runs toward calling a surface
/// unreachable — which is the safe direction for a list somebody acts on.
///
/// Comments go because a doc comment naming `SettingsWindow::opened_by_hand` is
/// prose about the road, not the road.
fn what_a_machine_runs(text: &str) -> String {
    let before_tests = text.split("#[cfg(test)]").next().unwrap_or("");
    before_tests
        .lines()
        .map(|line| line.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every `.rs` file under the crates, as (path, whole text).
fn every_source(at: &Path, into: &mut Vec<(PathBuf, String)>) {
    let Ok(entries) = fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            every_source(&path, into);
        } else if path.extension().is_some_and(|it| it == "rs")
            && let Ok(text) = fs::read_to_string(&path)
        {
            into.push((path, text));
        }
    }
}

/// Where a type is defined, and whether anything outside it and outside every
/// test writes `Type::`.
fn reached(name: &str, sources: &[(PathBuf, String)]) -> (Option<PathBuf>, Vec<String>) {
    let defined = format!("pub struct {name}");
    let other = format!("pub enum {name}");
    let home = sources
        .iter()
        .find(|(path, text)| {
            !is_a_test(path)
                && what_a_machine_runs(text).lines().any(|l| {
                    l.trim_start().starts_with(&defined) || l.trim_start().starts_with(&other)
                })
        })
        .map(|(path, _)| path.clone());

    let mut callers = Vec::new();
    for (path, text) in sources {
        if is_a_test(path) {
            continue;
        }
        if builds_one(name, &what_a_machine_runs(text)) {
            callers.push(path.to_string_lossy().replace('\\', "/"));
        }
    }
    (home, callers)
}

/// Whether some text builds one of these, either way Rust offers it.
///
/// **Both forms, because the first version of this knew only one.** `Type::`
/// catches a constructor that is an associated function; `Type {` catches a
/// struct literal, which names no `::` at all. The Mac found that gap in their
/// own crate — `popups.rs` builds a `Popup { … }` three screens from its
/// definition — and their framing is the one worth keeping: **a hit is a
/// question and a miss is not an answer.** A type built through a trait, a
/// factory or `Default` is still invisible here.
///
/// **The type's own file counts**, which the first version also had wrong by
/// excluding it. A type built by a production function in its own module is
/// built; that function needing a caller of its own is a different question,
/// and conflating the two reported the very line somebody was pointing at as
/// proof the type was dead.
///
/// Lines that define or implement are skipped, or `impl Popup {` in another
/// module would read as construction.
fn builds_one(name: &str, text: &str) -> bool {
    let associated = format!("{name}::");
    let spaced = format!("{name} {{");
    let tight = format!("{name}{{");
    text.lines()
        .filter(|line| !declares_rather_than_builds(line))
        .any(|line| {
            [&associated, &spaced, &tight]
                .iter()
                .any(|what| names_it_whole(line, what))
        })
}

/// Whether a line holds this text as a **whole name** rather than the tail of a
/// longer one.
///
/// **`contains` is not enough and it put a wrong entry in the list above.**
/// Looking for `Edge::` matched `FrameEdge::Top`, so `alo_dock::Edge` was
/// reported as built by six files in `alo-shell` — a crate that imports it zero
/// times. The Python draft of this check had a word boundary and the Rust one
/// lost it, which is the same fault the third PC warned about for crate names
/// and I checked for crates and not for types.
///
/// So the character before a match must not continue an identifier: Rust
/// identifiers are alphanumeric or `_`, and nothing else.
///
/// **A leading `::` must be allowed, and the first version of this forbade it.**
/// Excluding `:` as well looked tidier — it would stop `other::Edge::` being
/// read as a bare `Edge::` — but a qualified path is how one crate normally
/// names another's type, so `alo_dock::Edge::Bottom` stopped counting as a use.
/// Breaking it in both directions is what found that: injecting a real
/// `alo_dock::Edge::Bottom` into production left the test **green**, which is
/// the outcome that means a check cannot see the thing it is named for. A false
/// positive had been traded for a worse false negative, and only the break said
/// so.
fn names_it_whole(line: &str, what: &str) -> bool {
    let mut from = 0;
    while let Some(at) = line.get(from..).and_then(|rest| rest.find(what)) {
        let start = from + at;
        let before = line.get(..start).and_then(|head| head.chars().next_back());
        let whole = before.is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
        if whole {
            return true;
        }
        from = start + 1;
    }
    false
}

/// Whether a line declares something rather than building one.
///
/// **A function signature is the trap, and it cost a wrong answer.** Adding the
/// struct-literal form made `pub const fn taking(&self) -> &Screenshot {` match:
/// that `{` opens the function body, not a literal. Two surfaces flipped from
/// test-only to built on the strength of a getter that returns a reference to
/// one, which is the opposite of building one.
///
/// So a line that declares a function, an impl, a type, a trait or a `use` is
/// not a construction whatever it contains. A one-line `fn f() -> X { X { .. } }`
/// would be missed; rustfmt splits those here, and missing one is the safe
/// direction.
fn declares_rather_than_builds(line: &str) -> bool {
    let line = line.trim_start();
    let after_visibility = line
        .strip_prefix("pub(crate) ")
        .or_else(|| line.strip_prefix("pub "))
        .unwrap_or(line);
    let after_qualifiers = after_visibility
        .strip_prefix("const ")
        .or_else(|| after_visibility.strip_prefix("async "))
        .or_else(|| after_visibility.strip_prefix("unsafe "))
        .unwrap_or(after_visibility);
    after_qualifiers.starts_with("fn ")
        || line.starts_with("impl")
        || line.starts_with("use ")
        || ["struct ", "enum ", "trait ", "type ", "union "]
            .iter()
            .any(|kind| after_visibility.starts_with(kind))
}

/// **Every surface listed is in the state this file says it is in.**
#[test]
fn every_surface_a_person_uses_is_built_somewhere_a_machine_can_reach() {
    let crates = the_crates();
    let mut sources = Vec::new();
    every_source(&crates, &mut sources);
    assert!(
        sources.len() > 2_000,
        "this walk read {} source files, which is a walk that stopped rather \
         than a repository that shrank",
        sources.len()
    );

    let mut wrong = Vec::new();
    for (name, expected) in EVERY_SURFACE_A_PERSON_USES {
        let (home, callers) = reached(name, &sources);
        assert!(
            home.is_some(),
            "{name} is named here as a surface a person uses and no crate \
             defines it. Either it was renamed — in which case rename it here — \
             or this list is describing a repository that no longer exists"
        );
        match (expected, callers.is_empty()) {
            (Reach::AMachineCan, true) => wrong.push(format!(
                "{name} is listed as reachable and nothing outside a test builds one. \
                 Either a caller was removed — which is a person losing a feature \
                 while every test still passes — or this check has broken and is \
                 no longer seeing callers that are there."
            )),
            (Reach::OnlyATestDoes { .. }, false) => wrong.push(format!(
                "{name} is listed as unreachable and {} now builds one: {}. \
                 That is somebody paying the debt, so REMOVE ITS ENTRY from \
                 EVERY_SURFACE_A_PERSON_USES and set it to Reach::AMachineCan.",
                callers.len(),
                callers.join(", ")
            )),
            _ => {}
        }
    }

    assert!(
        wrong.is_empty(),
        "{} surface(s) are not in the state this file says:\n\n- {}",
        wrong.len(),
        wrong.join("\n\n- ")
    );
}

/// **Every unreachable surface carries a reason somebody can act on.**
///
/// A list of names with no reasons rots into a list nobody reviews, which is
/// how the thing it records becomes permanent.
#[test]
fn every_surface_a_machine_cannot_reach_says_why() {
    for (name, reach) in EVERY_SURFACE_A_PERSON_USES {
        if let Reach::OnlyATestDoes { why } = reach {
            assert!(
                why.len() > 60,
                "{name} is listed as unreachable with too little reason to act on: {why:?}"
            );
            assert!(
                why.contains("2026-"),
                "{name}'s reason carries no date, so nobody can tell whether it \
                 was measured today or last month: {why:?}"
            );
        }
    }
}
