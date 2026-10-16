//! What a run is allowed to be, and the four ways it is refused.
//!
//! Every refusal is tested, which `CLAUDE.md` asks for in those words: *for
//! anything an agent can reach, "and it was stopped" is the test that matters.*
//! A run reaches a program on this machine, so each of these is a road that
//! must stay shut.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use alo_capability::HowCodeRuns;

use super::ARun;
use crate::failing::NotRun;

/// A program every machine running these tests has.
fn a_real_program() -> PathBuf {
    PathBuf::from("/bin/sh")
}

/// **A well-formed run is accepted**, which is the control every refusal below
/// needs: a checker that refused everything would pass all five of them.
#[test]
fn a_run_naming_a_real_program_and_a_place_is_accepted() {
    let run = ARun::checked(
        HowCodeRuns::InASealedBox,
        a_real_program(),
        vec!["-c".into(), "true".into()],
        vec![PathBuf::from("/bin")],
    )
    .expect("a real program with a place to reach is a run");

    assert_eq!(run.program(), a_real_program());
    assert_eq!(run.arguments(), ["-c", "true"].map(OsString::from));
    assert_eq!(run.may_reach(), [PathBuf::from("/bin")]);
}

/// **A relative program is refused**, because what it means depends on where
/// this service happens to be and that is not what anybody approved.
#[test]
fn a_relative_program_is_refused() {
    let wrong = ARun::checked(
        HowCodeRuns::InASealedBox,
        PathBuf::from("sh"),
        vec![],
        vec![PathBuf::from("/bin")],
    )
    .expect_err("a relative program is not a run");
    assert!(
        matches!(wrong, NotRun::NotAnAbsoluteProgram(ref which) if which == Path::new("sh")),
        "a relative program was refused for the wrong reason: {wrong}"
    );
}

/// **A program that is not there is refused**, and says so as an absence rather
/// than as some other complaint.
#[test]
fn a_program_that_is_not_there_is_refused() {
    let wrong = ARun::checked(
        HowCodeRuns::InASealedBox,
        PathBuf::from("/nowhere/at/all/alo-no-such-program"),
        vec![],
        vec![PathBuf::from("/bin")],
    )
    .expect_err("a program that does not exist is not a run");
    assert!(
        matches!(wrong, NotRun::NoSuchProgram { .. }),
        "a missing program was refused for the wrong reason: {wrong}"
    );
}

/// **A run that names no place is refused**, rather than quietly becoming a box
/// around nothing — which could not open the program it was asked to run.
#[test]
fn a_run_with_nowhere_to_reach_is_refused() {
    let wrong = ARun::checked(HowCodeRuns::InASealedBox, a_real_program(), vec![], vec![])
        .expect_err("a run with no places is not a run");
    assert!(
        matches!(wrong, NotRun::NothingToReach),
        "a run with no places was refused for the wrong reason: {wrong}"
    );
}

/// **The other two levels are refused here**, and this is the one that matters
/// most: the dangerous failure is not a refusal, it is a crate that accepted
/// *full trust* and built a box anyway, or worse ran without one.
///
/// Both are asserted, because a check written as *is it the sealed box* and one
/// written as *is it not full trust* behave identically on full trust and
/// differently on asking-each-time.
#[test]
fn a_level_that_is_not_the_sealed_box_is_refused() {
    for (chose, expected) in [
        (HowCodeRuns::AskingEachTime, "asking each time"),
        (HowCodeRuns::AtFullTrust, "full trust"),
    ] {
        let wrong = ARun::checked(chose, a_real_program(), vec![], vec![PathBuf::from("/bin")])
            .expect_err("only the sealed box is run here");
        match wrong {
            NotRun::NotTheSealedBox { chose: said } => assert_eq!(
                said, expected,
                "{chose:?} was refused, and named the wrong level"
            ),
            other => panic!("{chose:?} was refused for the wrong reason: {other}"),
        }
    }
}

/// **The level is checked before the request is looked at.**
///
/// A run at full trust with a nonsense program is refused for the *level*, not
/// for the program. It matters because the level is the person's decision and
/// the rest is a well-formedness question: a caller that sent the wrong level
/// should hear about that first, and should not have to make its arguments
/// valid to be told.
#[test]
fn the_level_is_the_first_thing_checked() {
    let wrong = ARun::checked(
        HowCodeRuns::AtFullTrust,
        PathBuf::from("not/absolute/and/not/there"),
        vec![],
        vec![],
    )
    .expect_err("full trust is not run here");
    assert!(
        matches!(wrong, NotRun::NotTheSealedBox { .. }),
        "a bad request at the wrong level was refused for the request: {wrong}"
    );
}

/// **A refused run asks the machine about nothing it named**, which is
/// `alo-bounding`'s order one level up.
///
/// The three refusals that do not need the filesystem are given a program under
/// a directory that does not exist. A checker that looked the program up before
/// deciding would produce `NoSuchProgram` for all three; each must produce its
/// own reason instead.
#[test]
fn a_refused_run_decides_before_it_asks_the_machine() {
    let nowhere = PathBuf::from("/nowhere/at/all/alo-no-such-program");

    let level = ARun::checked(
        HowCodeRuns::AtFullTrust,
        nowhere.clone(),
        vec![],
        vec![PathBuf::from("/bin")],
    )
    .expect_err("the level is wrong");
    assert!(matches!(level, NotRun::NotTheSealedBox { .. }), "{level}");

    let relative = ARun::checked(
        HowCodeRuns::InASealedBox,
        PathBuf::from("alo-no-such-program"),
        vec![],
        vec![PathBuf::from("/bin")],
    )
    .expect_err("the program is relative");
    assert!(
        matches!(relative, NotRun::NotAnAbsoluteProgram(_)),
        "{relative}"
    );

    let nothing = ARun::checked(HowCodeRuns::InASealedBox, nowhere, vec![], vec![])
        .expect_err("there is nowhere to reach");
    assert!(matches!(nothing, NotRun::NothingToReach), "{nothing}");
}
