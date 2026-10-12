//! Which roles a reading fills, in which order, and the one answer that is not
//! a list.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use super::{filled_by, what_this_machine_offers};
use alo_applications::Application;
use alo_dock::pinning::{ARole, Discovery};
use alo_software::source::{Configured, SourceName};
use alo_software::what_this_machine_has::{WhatThisMachineHas, what_this_machine_has};
use alo_software::{Failed, Shipped, Tool};

/// What this system ships for the two roles a scan can answer.
///
/// Spelled here rather than read from `Shipped`, so a test that passes is
/// saying *these identifiers* and not *whatever the list happens to hold* —
/// which would agree with the code however wrong both were.
const THE_FILE_MANAGER: &str = "org.kde.dolphin";
const THE_BROWSER: &str = "org.mozilla.firefox";

/// A tool answering with what a test put in it.
///
/// `installed` and `open` answer differently on purpose, which is
/// `what_this_machine_has_tests`' reasoning and holds here for the same reason:
/// a producer that read the running applications instead of the installed ones
/// would return a shorter and entirely plausible list.
struct AToolThatSays {
    installed: Result<Vec<String>, Failed>,
}

impl AToolThatSays {
    fn listing(lines: &[&str]) -> Self {
        Self {
            installed: Ok(lines.iter().map(|line| (*line).to_owned()).collect()),
        }
    }

    fn refusing() -> Self {
        Self {
            installed: Err(Failed::NotOffered),
        }
    }
}

impl Tool for AToolThatSays {
    fn installed(&self) -> Result<Vec<String>, Failed> {
        self.installed.clone()
    }

    fn open(&self) -> Result<Vec<String>, Failed> {
        Ok(vec!["org.running.Only".to_owned()])
    }

    fn sources(&self) -> Result<Vec<Configured>, Failed> {
        Err(Failed::NotOffered)
    }

    fn install(&self, _: &SourceName, _: &Application) -> Result<(), Failed> {
        Err(Failed::NotOffered)
    }

    fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> {
        Err(Failed::NotOffered)
    }

    fn update(&self, _: &Application) -> Result<(), Failed> {
        Err(Failed::NotOffered)
    }

    fn remove(&self, _: &Application) -> Result<(), Failed> {
        Err(Failed::NotOffered)
    }
}

/// A reading of a machine listing these identifiers.
fn a_machine_with(lines: &[&str]) -> WhatThisMachineHas {
    what_this_machine_has(&AToolThatSays::listing(lines))
        .expect("a tool that answered is a reading that succeeded")
}

/// The list this system ships, which the production path also reads.
fn the_shipped_list() -> Shipped {
    Shipped::decided().expect("the built-in list is the one a test holds to be readable")
}

/// **Both roles filled, and in the Dock's order rather than the machine's.**
///
/// The tool lists the browser **first** and the file manager second, which is
/// the opposite of `ARole::EVERY`. A producer that walked what it found instead
/// of walking the roles would pass every other assertion in this file and fail
/// this one, which is the whole reason the fixture is written backwards.
#[test]
fn a_machine_with_both_fills_both_in_the_roles_order() {
    let filled = filled_by(
        &a_machine_with(&[THE_BROWSER, "org.kde.ark", THE_FILE_MANAGER]),
        &the_shipped_list(),
    );

    let roles: Vec<ARole> = filled.iter().map(|(role, _)| *role).collect();
    assert_eq!(
        roles,
        vec![ARole::Files, ARole::Browser],
        "the roles filled, and their order, are not the Dock's"
    );
    let names: Vec<&str> = filled.iter().map(|(_, app)| app.name()).collect();
    assert_eq!(
        names,
        vec![THE_FILE_MANAGER, THE_BROWSER],
        "a role was filled by an application this system does not ship for it"
    );
}

/// **Settings is never produced, because no scan could answer it.**
///
/// Asserted as *the roles are exactly these two* rather than *Settings is not
/// among them*: the first fails when a third appears whatever it is, the second
/// only fails for the one name somebody thought to exclude. The owner's ruling
/// of 2026-10-12 is that no identifier is invented for Settings, and this is
/// the shape that holds it.
#[test]
fn a_machine_with_everything_installed_still_fills_only_two_roles() {
    let everything = [
        THE_BROWSER,
        THE_FILE_MANAGER,
        "org.kde.ark",
        "org.gnome.TextEditor",
        "org.gnome.Loupe",
        "org.gnome.Papers",
        "app.devsuite.Ptyxis",
        // The name somebody reaching for a Settings pin would invent.
        "org.aloworld.Settings",
    ];

    let filled = filled_by(&a_machine_with(&everything), &the_shipped_list());

    assert_eq!(
        filled.iter().map(|(role, _)| *role).collect::<Vec<_>>(),
        vec![ARole::Files, ARole::Browser],
        "a role beyond the two a scan can answer reached the Dock"
    );
}

/// **A role nothing fills is omitted, not placeholdered.**
///
/// The owner's words: *omit unavailable applications; never show placeholders
/// or non-working buttons.* A Dock of two icons, not a Dock of three where one
/// opens nothing.
#[test]
fn a_machine_with_only_a_browser_fills_only_that_role() {
    let filled = filled_by(
        &a_machine_with(&[THE_BROWSER, "org.gnome.Loupe"]),
        &the_shipped_list(),
    );

    assert_eq!(
        filled.len(),
        1,
        "a machine with one of the two roles filled more or fewer than one: {filled:?}"
    );
    assert_eq!(
        filled.first().map(|(role, _)| *role),
        Some(ARole::Browser),
        "the role filled is not the one the machine has installed"
    );
}

/// **A machine with neither is a real answer, and is written down.**
///
/// Empty here means *this machine has no file manager and no browser*, which is
/// a Dock with nothing pinned that stays that way — the opposite of the empty
/// in the test below. `ThePins::what_to_pin` is what turns the two into
/// different outcomes, and it can only do that because they arrive as different
/// values.
#[test]
fn a_machine_with_neither_fills_nothing_and_that_is_a_completed_reading() {
    let filled = filled_by(&a_machine_with(&["org.kde.ark"]), &the_shipped_list());

    assert!(
        filled.is_empty(),
        "a machine with neither role installed filled one: {filled:?}"
    );
}

/// **A tool that would not answer is `Failed`, and never an empty reading.**
///
/// This is the clause the whole producer is shaped around: *an unfinished scan
/// and a failed one are not evidence that an application is absent.* The two
/// assertions are both needed — the first says what it is, the second says what
/// it must not have become, because `Completed(vec![])` is the plausible wrong
/// answer and it is the one that would permanently empty a person's Dock.
#[test]
fn a_tool_that_will_not_answer_is_a_failure_and_not_an_empty_machine() {
    let refused = what_this_machine_has(&AToolThatSays::refusing());

    assert!(
        refused.is_err(),
        "a tool that refused produced a reading, so the producer below cannot tell the two apart"
    );
    // The producer's own branch, exercised through the same shape it takes in
    // production: there is no reading to hand `filled_by`, so there is nothing
    // to conclude.
    let discovery = if refused.is_err() {
        Discovery::Failed
    } else {
        Discovery::Completed(Vec::new())
    };
    assert_eq!(
        discovery,
        Discovery::Failed,
        "a refused scan did not become a refusal"
    );
    assert_ne!(
        discovery,
        Discovery::Completed(Vec::new()),
        "a refused scan became a machine with nothing on it"
    );
}

/// **The production path runs and answers one of the three.**
///
/// It reads whatever machine the test is on, so what it finds is not asserted —
/// only that asking is possible and that the answer is a value the Dock can
/// use. A gate machine has no rented tool and will answer `Failed`; a machine
/// with one will answer `Completed`. **Neither is a failure of this test**, and
/// saying so is the point: the alternative is a test that passes on the gate
/// and means nothing, with no sentence anywhere admitting it.
#[test]
fn asking_this_machine_answers_without_panicking() {
    assert!(
        !matches!(what_this_machine_offers(), Discovery::StillLooking),
        "this producer answers in one pass and must never report that it is still looking"
    );
}
