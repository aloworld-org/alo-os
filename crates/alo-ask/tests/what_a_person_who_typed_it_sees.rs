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

/// A login whose settings name a provider, with this text in them.
///
/// Written to a real file under a real `HOME`, because what is being tested is
/// the command reading a person's settings — a fixture that handed it a
/// `Settings` would test the decision, which `alo-choosing` already tests.
fn a_login_whose_settings_say(what: &str, text: &str) -> std::path::PathBuf {
    let home = std::env::temp_dir().join(format!("alo-ask-{}-{what}", std::process::id()));
    let folder = home.join(".config").join("alo");
    std::fs::create_dir_all(&folder).expect("a temporary folder can be made");
    std::fs::write(folder.join("settings.toml"), text).expect("settings can be written");
    home
}

/// **A person who chose a provider and said nothing else may answer is not
/// answered, and is told which setting did it.**
///
/// The whole road, through the real binary: their file is read, the setting in
/// it is honoured, and the sentence names their own provider and model so they
/// can change their mind. `docs/features.md`: *Unless the person says
/// otherwise, in Settings* — the owner's instruction of 2026-10-18.
#[test]
fn a_person_who_forbade_anything_else_is_not_answered_from_this_machine() {
    let home = a_login_whose_settings_say(
        "forbade",
        "format = 3\n\n\
         [answers]\n\
         provider = { name = \"Mistral\", model = \"mistral-small-latest\" }\n\n\
         [[provider]]\n\
         name = \"Mistral\"\n\
         endpoint = \"https://api.mistral.ai\"\n\
         region = \"the EU\"\n\n\
         [when-mine-is-unavailable]\n\
         may-use-another = false\n",
    );

    let (ok, said) = run(&["may the tenant sublet"], &home);

    assert!(!ok, "a question nothing may answer succeeded: {said}");
    assert!(
        said.contains("told alo not to use anything else"),
        "the person was not told which of their settings decided it: {said}"
    );
    assert!(
        said.contains("mistral-small-latest") && said.contains("Mistral"),
        "the sentence did not name what they chose, so they cannot find it: {said}"
    );
    assert!(
        !said.contains("on this machine"),
        "it answered from this machine anyway: {said}"
    );
}

/// **And a person who has not been asked is still answered.**
///
/// The sharp one. Reading *nobody has decided* as *they refused* would stop
/// somebody getting answers they get today, which law 5 calls taking a choice
/// away — so the same settings **without** the section must not refuse.
///
/// It asserts on what it does *not* say, because whether an answer arrives
/// depends on a model runtime this test has no business needing.
#[test]
fn a_person_who_has_not_been_asked_is_not_refused() {
    let home = a_login_whose_settings_say(
        "not-asked",
        "format = 3\n\n\
         [answers]\n\
         provider = { name = \"Mistral\", model = \"mistral-small-latest\" }\n\n\
         [[provider]]\n\
         name = \"Mistral\"\n\
         endpoint = \"https://api.mistral.ai\"\n\
         region = \"the EU\"\n",
    );

    let (_ok, said) = run(&["may the tenant sublet"], &home);

    assert!(
        !said.contains("told alo not to use anything else"),
        "a person who has never seen the setting was refused as though they had said no: {said}"
    );
}

/// **A person who chose a provider is told about their provider, and never
/// about this machine's runtime.**
///
/// What this pins is the **routing**, which is the thing that was measured
/// wrong: *with a provider in their settings and one local model installed, a
/// person is answered locally and told only `on this machine`* (ADR 0125).
/// `main.rs` sends a chosen provider to the daemon before it looks at the
/// catalogue or the runtime, and says in its own words why: *a person whose
/// question is going to OpenAI must not first be told a local runtime is
/// unreachable.*
///
/// # Why it asserts this and not *never answered on this machine*
///
/// That was the first version of this test and it was **vacuous**. On a machine
/// with no model runtime — which is every machine in CI — nothing can answer
/// locally whatever the routing does, so an assertion that no answer came from
/// here passes for a command that had fallen through to the catalogue. It also
/// matched the phrase *on this machine* inside an unrelated and correct
/// sentence: *the alo service is not running on this machine.*
///
/// This version can fail. Take the provider road out of `main.rs` and the
/// person is told about the runtime instead, which is what both assertions
/// below catch.
///
/// # And the setting is deliberately absent from these settings
///
/// The test above holds this when the person turned fallback **off**, which is
/// a thing they asked for. This is the default — every person who has not
/// opened that row — and nothing held the routing to it.
#[test]
fn a_person_who_chose_a_provider_is_told_about_their_provider() {
    let home = a_login_whose_settings_say(
        "provider-no-fallback-setting",
        "format = 3\n\n\
         [answers]\n\
         provider = { name = \"Mistral\", model = \"mistral-small-latest\" }\n\n\
         [[provider]]\n\
         name = \"Mistral\"\n\
         endpoint = \"https://api.mistral.ai\"\n\
         region = \"the EU\"\n",
    );

    let (ok, said) = run(&["what is the capital of Spain"], &home);

    assert!(!ok, "a question nothing could answer succeeded: {said}");
    assert!(
        said.contains("provider"),
        "a person who chose a provider was not told anything about it: {said}"
    );
    assert!(
        !said.contains("runtime"),
        "a person whose question was going to a provider was told about this machine's \
         runtime: {said}"
    );
    // Not the *forbidden* sentence either, because they forbade nothing — that
    // would be the opposite fault, refusing somebody who never said no.
    assert!(
        !said.contains("told alo not to use anything else"),
        "a person who never saw the setting was refused as though they had said no: {said}"
    );
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
