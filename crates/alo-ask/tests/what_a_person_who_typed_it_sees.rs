//! The command as a person meets it: run the real binary, read what it printed.
//!
//! `which_model.rs` and `saying.rs` test the decisions. This tests the **order
//! they happen in**, which is the part only a process can be wrong about — and
//! the one refusal a person is most likely to see first, because it is what an
//! empty command line gets.
//!
//! # It asserts against English on purpose
//!
//! The sentences are keyed strings and a translated machine would print them in
//! the person's language. This run has no translations loaded, so what comes out
//! is each word's source text, which is what `alo_strings::Strings` answers with
//! when the languages run out. Matching on it pins **which word was chosen**,
//! which is the thing that can be wrong here; a test that matched on a key would
//! pass for a command that printed a marked key to a person.

#![expect(
    clippy::expect_used,
    reason = "in a test, a command this crate builds that will not run is the failure being reported"
)]

use std::path::Path;
use std::process::Command;

/// A login with nowhere of its own to keep settings in, named rather than made.
///
/// Nothing is written and nothing is read: `alo_choosing::Settings::at` answers
/// *untouched* for a file that is not there, which is the state of a person who
/// has never opened the settings panel. So the test needs a path and not a
/// directory, and it uses one under the system's own temporary folder rather
/// than creating anything to delete afterwards.
fn a_login_with_no_settings() -> std::path::PathBuf {
    std::env::temp_dir().join("alo-ask-a-login-that-has-chosen-nothing")
}

/// What the command printed on the error stream, and whether it succeeded.
fn run(arguments: &[&str], home: &Path) -> (bool, String) {
    let ran = Command::new(env!("CARGO_BIN_EXE_alo-ask"))
        .args(arguments)
        .env("HOME", home)
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("the command this crate builds is runnable");
    (
        ran.status.success(),
        String::from_utf8_lossy(&ran.stderr).into_owned(),
    )
}

/// **Nothing typed is answered before the machine is touched.**
///
/// The order this pins is the one that was wrong first: the model used to be
/// worked out before the command line was read, so a person who pressed return
/// by mistake on a machine whose runtime was not running was told *the model
/// runtime is not reachable* — true, and about the wrong thing entirely. The
/// second assertion is the whole point of the test, and it holds whether or not
/// anything is listening on this machine.
#[test]
fn a_person_who_typed_nothing_is_asked_for_the_question_and_nothing_else() {
    let (ok, said) = run(&[], &a_login_with_no_settings());

    assert!(!ok, "a command with no question in it did not fail: {said}");
    assert!(
        said.contains("there is nothing to ask yet"),
        "the person was not asked for their question: {said}"
    );
    assert!(
        !said.contains("runtime"),
        "the runtime was reached for before the command line was read: {said}"
    );
}

/// **A question made only of spaces is the same as no question.**
///
/// `alo_asking::Question::asked` trims before it decides, and this is the
/// evidence that the command's own early check trims too rather than testing a
/// string for emptiness and letting a space through to open a socket.
#[test]
fn spaces_are_not_a_question_either() {
    let (ok, said) = run(&["   "], &a_login_with_no_settings());

    assert!(!ok, "a question made of spaces did not fail: {said}");
    assert!(
        said.contains("there is nothing to ask yet"),
        "spaces were taken for a question: {said}"
    );
}
