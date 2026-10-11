//! Whether the mark is there, where it is, and the one way a test of it lies.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a temporary directory that would not be made is the failure \
              being reported"
)]

use super::TheApplicationsArrived;
use std::path::Path;

/// **A fresh machine has not been walked**, which is the whole point.
#[test]
fn a_fresh_root_has_no_mark() {
    let root = tempfile::tempdir().expect("a temporary directory");
    let mark = TheApplicationsArrived::beneath(root.path());

    assert!(!mark.already(), "an untouched root was reported as walked");
}

/// **Recording it makes it true**, and reading is what the unit does.
#[test]
fn recording_it_is_what_the_next_boot_reads() {
    let root = tempfile::tempdir().expect("a temporary directory");
    let mark = TheApplicationsArrived::beneath(root.path());

    assert!(!mark.already(), "the control: not there to begin with");
    mark.record().expect("a mark can be recorded");
    assert!(mark.already());

    // And a second reader, built the same way, sees the same thing - because
    // the answer is the file and not anything held in memory.
    assert!(TheApplicationsArrived::beneath(root.path()).already());
}

/// **Recording twice is not an error**, because a machine whose mark was
/// written and whose disk then filled would otherwise fail on a path that has
/// nothing left to do.
#[test]
fn recording_it_again_is_harmless() {
    let root = tempfile::tempdir().expect("a temporary directory");
    let mark = TheApplicationsArrived::beneath(root.path());

    mark.record().expect("once");
    mark.record().expect("twice");
    assert!(mark.already());
}

/// **The directory is made rather than required.**
///
/// Nothing in `tmpfiles.d` creates it, deliberately — adding this needed no
/// second file to agree with — so the program making it is load-bearing.
#[test]
fn the_holding_directory_is_made_and_not_required() {
    let root = tempfile::tempdir().expect("a temporary directory");
    let mark = TheApplicationsArrived::beneath(root.path());
    let holding = mark.at().parent().expect("the mark is inside something");

    assert!(!holding.exists(), "the control: no directory to begin with");
    mark.record().expect("a mark can be recorded");
    assert!(holding.exists());
}

/// **`beneath` goes beneath**, which is the one way a test of this could pass
/// while measuring the real machine.
///
/// `WHERE_IT_LIVES` is absolute, and `Path::join` with an absolute path throws
/// the root away — so a `beneath` written the obvious way would hand every
/// test `/var/lib/alo-shipping/...` and report *already walked* on any
/// developer machine that had ever run it.
#[test]
fn a_root_is_not_thrown_away_by_an_absolute_path() {
    let root = Path::new("/somewhere/else");
    let mark = TheApplicationsArrived::beneath(root);

    assert!(
        mark.at().starts_with(root),
        "{:?} is not beneath {root:?}",
        mark.at()
    );
    assert_ne!(mark.at(), Path::new(super::WHERE_IT_LIVES));
}

/// **The machine's own path is the one the unit names.** If these two ever
/// disagree the unit's condition reads a file nothing writes, and a machine
/// walks its applications on every boot.
#[test]
fn the_path_is_the_one_the_unit_is_told_about() {
    assert_eq!(
        TheApplicationsArrived::on_this_machine().at(),
        Path::new("/var/lib/alo-shipping/the-applications-arrived")
    );
    assert_eq!(
        TheApplicationsArrived::on_this_machine().at(),
        Path::new(super::WHERE_IT_LIVES)
    );
}

/// **It is under /var and not /etc**, said as a test because the reason is a
/// decision: `/etc` survives a reinstall in a way that would hide a fresh
/// machine's applications from it.
#[test]
fn it_lives_under_var() {
    let at = TheApplicationsArrived::on_this_machine();

    assert!(at.at().starts_with("/var/"), "{:?}", at.at());
    assert!(!at.at().starts_with("/etc/"), "{:?}", at.at());
}

/// **The unit names the path this file defines**, asked of the real unit
/// rather than of a literal written twice.
///
/// This is the one place the two must agree, and it is here rather than in
/// `crates/alo-image` because the constant is here: a condition reading a file
/// nothing writes is a machine that walks its applications on every boot, and
/// the failure would be silent — the unit would run, succeed, and run again.
#[test]
fn the_shipped_unit_reads_the_path_this_file_writes() {
    let at = "../../image/usr/lib/systemd/system/alo-shipping.service";
    let Ok(unit) = std::fs::read_to_string(at) else {
        unreachable!("the shipped unit is in this repository, at {at}")
    };
    let expected = format!(
        "ConditionPathExists=!{}",
        TheApplicationsArrived::on_this_machine().at().display()
    );

    assert!(
        unit.lines().any(|line| line.trim() == expected),
        "the unit does not carry `{expected}`, so its condition reads a file nothing writes"
    );
}

/// **And it no longer asks `ConditionFirstBoot`**, which can never be true on
/// a machine written with `bootc install to-disk`.
///
/// Asserted on the directive and not on the file, because the unit's comment
/// explains the old condition at length and should go on doing so — a check
/// that forbade the words would forbid the explanation.
#[test]
fn the_shipped_unit_no_longer_asks_a_question_that_cannot_be_true() {
    let at = "../../image/usr/lib/systemd/system/alo-shipping.service";
    let Ok(unit) = std::fs::read_to_string(at) else {
        unreachable!("the shipped unit is in this repository, at {at}")
    };

    assert!(
        unit.contains("ConditionFirstBoot"),
        "the control: the unit still explains the old condition, so a directive hiding among \
         those words is what this test has to tell apart"
    );
    assert!(
        !unit
            .lines()
            .any(|line| line.trim().starts_with("ConditionFirstBoot")),
        "the unit asks ConditionFirstBoot as a directive, which is never true on a machine \
         written with `bootc install to-disk`"
    );
}
