//! What the counter and the fixed point answer, on text chosen here rather
//! than discovered in the repository.
//!
//! **The inputs are an argument, never a tree.** A test that read the real
//! workspace would be measuring whatever happens to be committed today, and
//! `docs/misreadings/a-test-that-asserted-its-own-premise.md` is about exactly
//! that: when a test cannot choose its input, the input is in the wrong place.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what was measured is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::{after_pub, pub_fns_in, what_nothing_reaches};

/// A workspace on disk holding exactly the files a test names.
///
/// **A real directory rather than a string**, because `what_nothing_reaches`
/// reads a tree — that is its subject. What keeps this honest is that the test
/// writes every byte of it: the tree is still an argument, which is what this
/// module's header asks for. The repository's own tree is read by
/// `tests/every_pub_fn_is_reached_or_accepted.rs` and nowhere else.
fn a_workspace_of(called: &str, files: &[(&str, &str, &str)]) -> PathBuf {
    // **Named by the test rather than derived from the files.** A name computed
    // from the content collides when two tests happen to agree on it, and then
    // one test's fixture is the next one's input — which `CLAUDE.md` records as
    // a fault three unrelated tests hit at once by assuming they were alone on
    // the machine.
    let at = std::env::temp_dir().join(format!("alo-reached-{}-{called}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    let mut members = Vec::new();
    for (crate_named, within, code) in files {
        let path = at.join("crates").join(crate_named).join(within);
        std::fs::create_dir_all(path.parent().expect("a parent directory"))
            .expect("making the crate directory");
        std::fs::write(&path, code).expect("writing the file");
        let member = format!("\"crates/{crate_named}\"");
        if !members.contains(&member) {
            members.push(member);
        }
    }
    std::fs::write(
        at.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{}]\n", members.join(", ")),
    )
    .expect("writing the manifest");
    at
}

/// The names a sweep of a workspace reports, in order.
fn what_nothing_reaches_in(at: &Path) -> Vec<String> {
    what_nothing_reaches(at)
        .into_iter()
        .map(|it| it.named)
        .collect()
}

/// **Every shape of `pub fn` this workspace writes is found.**
#[test]
fn every_shape_of_a_public_function_is_found() {
    let code = "\
pub fn plain() {}
pub const fn constant() {}
pub async fn waiting() {}
pub unsafe fn risky() {}
    pub fn indented() {}
";
    let found: Vec<String> = pub_fns_in(code).into_iter().map(|(_, it)| it).collect();
    assert_eq!(
        found,
        ["plain", "constant", "waiting", "risky", "indented"],
        "{found:?}"
    );
}

/// **`pub(crate)` is not public surface and is left out.**
///
/// A function visible only inside its own crate is reached or not by that
/// crate's ordinary rules, and `rustc`'s dead-code warning already catches it.
/// What this gate is for is the surface `rustc` cannot warn about.
#[test]
fn a_crate_private_function_is_not_this_gates_business() {
    assert_eq!(after_pub("pub fn open()"), Some(" fn open()"));
    assert_eq!(after_pub("pub(crate) fn inside()"), None);
    assert_eq!(after_pub("pub(super) fn above()"), None);
    assert_eq!(after_pub("fn private()"), None);

    let found = pub_fns_in("pub(crate) fn inside() {}\npub fn out() {}\n");
    let named: Vec<String> = found.into_iter().map(|(_, it)| it).collect();
    assert_eq!(named, ["out"], "{named:?}");
}

/// **The line is reported, so a reader can open it.**
#[test]
fn the_line_is_reported() {
    let found = pub_fns_in("\n\npub fn third() {}\n");
    assert_eq!(found, [(3, "third".to_owned())], "{found:?}");
}

/// **A name is a whole identifier, not a prefix.**
///
/// `event11` read as `event1` is the shape of a fault this repository has
/// already paid for once in a different file.
#[test]
fn a_name_stops_at_the_identifier() {
    let found = pub_fns_in("pub fn draw_it(width: u32) -> u32 {}\n");
    let named: Vec<String> = found.into_iter().map(|(_, it)| it).collect();
    assert_eq!(named, ["draw_it"], "{named:?}");
}

/// **A call made inside a function nothing reaches is not a call**, which is
/// hole one on `docs/autonomy/what-nothing-calls.md`: `alo-dock`'s
/// `where_it_sits` looked reached because a one-line wrapper named it and
/// nothing called the wrapper.
#[test]
fn a_wrapper_nobody_calls_does_not_reach_what_it_wraps() {
    let named = what_nothing_reaches_in(&a_workspace_of(
        "a-wrapper",
        &[(
            "one",
            "src/lib.rs",
            "pub fn where_it_sits() {}\npub fn the_wrapper() { where_it_sits() }\n",
        )],
    ));
    assert!(
        named.iter().any(|it| it == "where_it_sits") && named.iter().any(|it| it == "the_wrapper"),
        "the wrapper reaches nothing, so neither is reached; found {named:?}"
    );
}

/// **A private function's call still counts when a `pub fn` beside it is
/// dead.**
///
/// This is the regression test for the fault found on 2026-10-21 by sampling
/// five findings and triaging them by hand: the fixed point dropped a whole
/// **file** once every `pub fn` in it was unreached, and since `after_pub`
/// ignores `pub(crate)` and private functions, a file holding one dead `pub fn`
/// and live private ones was dropped entire. Three of the five findings sampled
/// were false, `Bounds::reaching_nothing_but` among them — called in product
/// code at `crates/alo-agentd/src/bounding.rs:214`.
///
/// What makes the function-sized unit sound is that `rustc`'s `dead_code` is
/// warn-by-default and the gate denies warnings, so an unreached private
/// function fails CI already and cannot be silently dead here.
#[test]
fn a_dead_public_function_does_not_silence_its_neighbours() {
    let named = what_nothing_reaches_in(&a_workspace_of(
        "a-neighbour",
        &[
            ("one", "src/lib.rs", "pub fn reached_from_elsewhere() {}\n"),
            (
                "two",
                "src/lib.rs",
                "pub fn nothing_calls_this() {}\nfn a_private_one() { reached_from_elsewhere() }\n",
            ),
        ],
    ));
    assert!(
        !named.iter().any(|it| it == "reached_from_elsewhere"),
        "a private function called it, and the dead `pub fn` in the same file must not \
         take that call away; found {named:?}"
    );
    assert!(
        named.iter().any(|it| it == "nothing_calls_this"),
        "the dead one is still dead; found {named:?}"
    );
}

/// **An example reaches a function**, because `clippy --all-targets` compiles
/// all 49 of this workspace's examples, so a call from one is code.
///
/// `Nested::submit_with_approval` and `Nested::pump_sign_in` are reached only
/// from `crates/alo-shell/examples`, and the first version of this crate
/// reported both as reached by nothing, which was untrue.
#[test]
fn a_call_from_an_example_is_not_a_function_nothing_reaches() {
    let named = what_nothing_reaches_in(&a_workspace_of(
        "an-example",
        &[
            (
                "one",
                "src/lib.rs",
                "pub fn only_an_example_calls_it() {}\n",
            ),
            (
                "one",
                "examples/a_check.rs",
                "fn main() { only_an_example_calls_it() }\n",
            ),
        ],
    ));
    assert!(
        named.is_empty(),
        "the example calls it, so nothing is unreached; found {named:?}"
    );
}
