//! That the admission happens, that it happens **first**, and that a box
//! outside this service is refused before anything is written.
//!
//! # How the order is tested without a kernel
//!
//! [`super::enter_the_box_and_become`] writes and then `exec`s, and a
//! successful `exec` would replace this test process — so every test here gives
//! it a command that **cannot** start. The function then returns, and what it
//! wrote is still on disk to be read.
//!
//! That is the whole trick: a failing `exec` is what makes the write
//! observable. A test that passed a real program would prove nothing, because
//! there would be no test left to assert in.
//!
//! The `cgroup.procs` here is an ordinary file in a temporary folder rather
//! than a real control group, which is what makes these runnable as the person
//! and on any machine. What a real one does with the byte is the kernel's and
//! is measured in `docs/quirks.md`; what is asserted here is that the byte is
//! written, to the right file, before the command is reached.

use std::{fs, path::PathBuf};

use super::{MEANING_ME, THE_PROCESSES, enter_the_box_and_become};

/// A folder of this test's own, with a box in it that has a `cgroup.procs`.
///
/// Named after the process so two tests running at once cannot collide, which
/// is a fault this repository has had twice.
fn a_box_to_enter(what: &str) -> (PathBuf, PathBuf) {
    let inside = std::env::temp_dir().join(format!(
        "alo-running-{}-{what}-{}",
        std::process::id(),
        // Two tests in one process share a pid, so the name carries what it is
        // for as well. A collision here would be a test failing for a reason
        // that has nothing to do with what it tests.
        what.len()
    ));
    let the_box = inside.join("a-run");
    fs::create_dir_all(&the_box).expect("a temporary folder can be made");
    (inside, the_box)
}

/// A command that cannot start, so that `exec` fails and the test survives it.
fn a_command_that_cannot_start() -> PathBuf {
    PathBuf::from("/nowhere/at/all/alo-no-such-command")
}

/// **The admission is written, and it is written before the command.**
///
/// The command cannot start, so the function returns — and `cgroup.procs`
/// already holds the byte. That ordering is the guarantee ADR 0117 is about.
#[test]
fn the_box_is_entered_before_the_command_is_reached() {
    let (inside, the_box) = a_box_to_enter("entered-first");

    let why = enter_the_box_and_become(
        &inside,
        &the_box,
        &a_command_that_cannot_start(),
        &[] as &[&str],
    );
    assert_eq!(
        why.kind(),
        std::io::ErrorKind::NotFound,
        "the command should have failed to start, and failed differently: {why}"
    );

    let written = fs::read_to_string(the_box.join(THE_PROCESSES))
        .expect("the admission should have been written before the command was reached");
    assert_eq!(
        written, MEANING_ME,
        "what was written is not the byte cgroup v2 reads as this process"
    );

    // **And it says the box was joined**, so a refusal here is read as the
    // command's and not the box's (2026-10-31: one sentence served both).
    assert!(
        why.to_string().contains("the box was joined"),
        "a command that would not start after the box was joined did not say so: {why}"
    );

    drop(fs::remove_dir_all(&inside));
}

/// **A box that will not take this process says so, and says nothing ran.**
///
/// The other step of the two. The box here is a folder with no `cgroup.procs`
/// that can be written — a directory in its place — so the admission fails,
/// and what comes back must name the box rather than the command.
#[test]
fn a_box_that_will_not_take_this_process_says_so() {
    let (inside, the_box) = a_box_to_enter("will-not-take");
    fs::create_dir_all(the_box.join(THE_PROCESSES)).expect("a folder where the file would be");

    let why = enter_the_box_and_become(
        &inside,
        &the_box,
        &a_command_that_cannot_start(),
        &[] as &[&str],
    );

    let said = why.to_string();
    assert!(
        said.contains("would not take this process") && said.contains("nothing was run"),
        "a box that refused the admission was not named as the step that failed: {said}"
    );
    assert!(
        !said.contains("the box was joined"),
        "a refused admission was described as a joined box: {said}"
    );

    drop(fs::remove_dir_all(&inside));
}

/// **A box that is not this service's is refused, and nothing is written.**
///
/// The one argument that decides where a process puts itself is the one checked,
/// and the check comes before the write — so the refusal leaves no `cgroup.procs`
/// behind at all.
#[test]
fn a_box_outside_this_service_is_refused_and_nothing_is_written() {
    let (inside, _) = a_box_to_enter("refused");
    let somewhere_else =
        std::env::temp_dir().join(format!("alo-running-elsewhere-{}", std::process::id()));
    fs::create_dir_all(&somewhere_else).expect("a temporary folder can be made");

    let why = enter_the_box_and_become(
        &inside,
        &somewhere_else,
        &a_command_that_cannot_start(),
        &[] as &[&str],
    );
    assert_eq!(
        why.kind(),
        std::io::ErrorKind::PermissionDenied,
        "a box outside this service should be a refusal: {why}"
    );
    assert!(
        !somewhere_else.join(THE_PROCESSES).exists(),
        "a refused box was written to anyway"
    );

    drop(fs::remove_dir_all(&inside));
    drop(fs::remove_dir_all(&somewhere_else));
}

/// **A box that climbs out with `..` is refused.**
///
/// The target is made, so that what is being measured is the climb and not the
/// absence. **This test caught the first version of the check**, which compared
/// the paths as they were written: `inside/../somewhere` starts with `inside`
/// by components, so the refusal never happened and the write was attempted
/// outside the service.
#[test]
fn a_box_that_climbs_out_is_refused() {
    let (inside, _) = a_box_to_enter("climbing");
    let climbed_to = inside
        .parent()
        .expect("a temporary folder has a parent")
        .join(format!("alo-running-not-ours-{}", std::process::id()));
    fs::create_dir_all(&climbed_to).expect("a temporary folder can be made");
    let climbing = inside
        .join("..")
        .join(climbed_to.file_name().expect("it was just named"));

    let why = enter_the_box_and_become(
        &inside,
        &climbing,
        &a_command_that_cannot_start(),
        &[] as &[&str],
    );
    assert_eq!(
        why.kind(),
        std::io::ErrorKind::PermissionDenied,
        "a box reached by climbing out should be a refusal: {why}"
    );
    assert!(
        !climbed_to.join(THE_PROCESSES).exists(),
        "a box outside the service was written to by climbing out of it"
    );

    drop(fs::remove_dir_all(&inside));
    drop(fs::remove_dir_all(&climbed_to));
}

/// **And a symbolic link out of the service is refused**, which is the other
/// half of the same fault.
///
/// A link under `inside` pointing outside it passes any check made on the path
/// as written — the path really is under the service, and what it names is not.
/// Resolving both sides answers this and the climb above together, and this
/// test is what says the resolving is not just about `..`.
#[test]
fn a_box_reached_through_a_link_out_of_the_service_is_refused() {
    let (inside, _) = a_box_to_enter("linked");
    let outside = inside
        .parent()
        .expect("a temporary folder has a parent")
        .join(format!("alo-running-outside-{}", std::process::id()));
    fs::create_dir_all(&outside).expect("a temporary folder can be made");

    let link = inside.join("looks-like-ours");
    std::os::unix::fs::symlink(&outside, &link).expect("a link can be made");

    let why = enter_the_box_and_become(
        &inside,
        &link,
        &a_command_that_cannot_start(),
        &[] as &[&str],
    );
    assert_eq!(
        why.kind(),
        std::io::ErrorKind::PermissionDenied,
        "a box reached through a link out of the service should be a refusal: {why}"
    );
    assert!(
        !outside.join(THE_PROCESSES).exists(),
        "a box outside the service was written to through a link"
    );

    drop(fs::remove_dir_all(&inside));
    drop(fs::remove_dir_all(&outside));
}

/// **A box in the right place that is not there is refused as an absence.**
///
/// The control for the two refusals above: a function that refused every path
/// it was given would pass both of them. This one is inside the service and
/// still refused, and the **kind** of refusal is what tells the two cases
/// apart — a reader of the record needs *there is no such box* and *that box is
/// not yours* to be different news.
#[test]
fn a_box_in_the_right_place_that_is_not_there_is_an_absence() {
    let (inside, _) = a_box_to_enter("missing");
    let missing = inside.join("never-made");

    let why = enter_the_box_and_become(
        &inside,
        &missing,
        &a_command_that_cannot_start(),
        &[] as &[&str],
    );
    assert_eq!(
        why.kind(),
        std::io::ErrorKind::NotFound,
        "a box in the right place that is absent should be an absence: {why}"
    );

    drop(fs::remove_dir_all(&inside));
}
