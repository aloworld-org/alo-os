//! The check itself, run against the verbs this machine really ships.
//!
//! Everything in the crate is the method. This is the measurement: the ten verbs
//! `alo-files` and `alo-applications` declare, the real `docs/by-hand.md`, the
//! real `docs/features.md` and this workspace's own member list — read off the
//! disk this test is running on, so a verb added tomorrow with nothing said about
//! it fails here rather than in somebody's reading a release from now.
//!
//! # And the refusals, beside it
//!
//! A green check says the verbs and the document agree. It cannot say the check
//! would have noticed if they did not, and that is the half a reader has to
//! believe rather than see — so every finding this crate can produce is put in
//! front of it here against a fixture, including the two that matter most: **a
//! verb added with nothing said about it**, and **a crate declaring verbs that
//! nothing handed in.**

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use alo_by_hand::{Finding, Held, THE_ANSWERS, THE_WORKSPACE, held, whoever_declares_verbs};
use alo_capability::Verbs;

/// The only list of what gets built, which every by-hand answer quotes.
const THE_DEFINITION: &str = "docs/features.md";

/// The other half of it: how a person does each verb without the agent.
const THE_DOCUMENT: &str = "docs/by-hand.md";

/// The crates that declare the verbs alo OS ships.
///
/// Written here rather than in the crate, because this is the measurement: the
/// crate checks whatever list it is handed, and a crate declaring verbs that is
/// missing from this array is what [`Finding::AVerbListNobodyHandedIn`] is for.
const WHO_DECLARES_THEM: [&str; 2] = ["alo-files", "alo-applications"];

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// A file of this repository, read.
fn reading(named: &str) -> String {
    let at = the_repository().join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Every verb alo OS ships, on one list, as a daemon would be handed them.
fn what_this_machine_ships() -> Verbs {
    let mut verbs = Verbs::default();
    alo_files::declare_into(&mut verbs).expect("the six file verbs declare");
    alo_applications::declare_into(&mut verbs).expect("the four application verbs declare");
    verbs
}

/// **Every verb alo OS ships names how a person does the same thing without the
/// agent, or is named as owed with the release that owns the answer** — against
/// this repository, on the disk it is checked out on.
#[test]
fn every_verb_this_machine_ships_can_be_done_by_hand() {
    let here = the_repository();
    let off_the_disk = move |named: &str| fs::read_to_string(here.join(named)).ok();
    let verbs = what_this_machine_ships();

    match held(
        &verbs,
        &WHO_DECLARES_THEM,
        &reading(THE_DOCUMENT),
        &reading(THE_DEFINITION),
        &off_the_disk,
    ) {
        Ok(what) => {
            assert_eq!(
                what.verbs(),
                verbs.len(),
                "the check counted a different number of verbs than this machine \
                 declares"
            );
            assert!(
                what.verbs() > 0,
                "this machine declares no verbs at all, which means the check \
                 passed over a list it never found"
            );
            assert_eq!(
                what.by_hand() + what.partly() + what.owed(),
                what.verbs(),
                "a verb was answered without being counted as plain, partly or \
                 owed, so the shape of the promise does not add up to the verbs"
            );
        }
        Err(findings) => {
            let listed: Vec<String> = findings.iter().map(ToString::to_string).collect();
            panic!(
                "{} thing(s) do not add up between the verbs this machine ships and \
                 {THE_DOCUMENT}:\n\n- {}",
                listed.len(),
                listed.join("\n\n- ")
            );
        }
    }
}

/// **Every crate of this workspace that declares verbs was handed to the check.**
///
/// The failure a list of verb names could never catch, measured against the real
/// workspace: a crate declaring verbs that nothing here knows about would make
/// every verb in it invisible to the check above, which would go on passing.
#[test]
fn every_crate_that_declares_verbs_is_one_this_check_was_handed() {
    let here = the_repository();
    let off_the_disk = move |named: &str| fs::read_to_string(here.join(named)).ok();
    let mut declaring = whoever_declares_verbs(&reading(THE_WORKSPACE), &off_the_disk);
    declaring.sort();
    let mut handed = WHO_DECLARES_THEM.map(str::to_owned);
    handed.sort();
    assert_eq!(
        declaring,
        handed.to_vec(),
        "the crates of this workspace that declare verbs are not the ones this \
         test hands to the check"
    );
}

/// The definition those answers quote, written the way `docs/features.md` writes
/// it: three promises at two tiers, and a list item that is not a promise.
const A_DEFINITION: &str = "\
## The desktop

- [v0.01] Launcher and window management: open, focus, close, tile
- [v0.5] A file manager, with trash, and archives that open
- [ ] Something somebody left a checkbox on
- [v0.5] **Search your own files, without asking anything** — by name and kind
";

/// The verbs that fixture is about: one of each effect, declared the way the real
/// ones are.
fn two_verbs() -> Verbs {
    let mut verbs = Verbs::default();
    alo_files::declare_into(&mut verbs).expect("the six file verbs declare");
    let mut two = Verbs::default();
    for named in ["list_folder", "move_file"] {
        two.declare(
            verbs
                .of(named)
                .expect("the file verbs include this one")
                .clone(),
        )
        .expect("two names, declared once each");
    }
    two
}

/// A by-hand document about those two verbs, sound in every way this crate can
/// check.
fn a_sound_document() -> String {
    format!(
        "\
# Every verb, by hand

Prose that names `list_folder` as an example of what an entry looks like.

{THE_ANSWERS}

### list_folder

**By hand:** a person opens the folder and reads what is in it, in the file
manager — `A file manager, with trash, and archives that open`.

### move_file

**Owed at:** [v0.5] — the file manager is v0.5 work and nothing on this machine
shows a folder to drag anything into yet.

## What this document does not do

It does not decide whether `A file manager, with trash, and archives that open`
is a good answer.
"
    )
}

/// The workspace those verbs come from, and the crate that declares them.
fn a_workspace(named: &str) -> Option<String> {
    match named {
        "Cargo.toml" => Some(
            "[workspace]\nmembers = [\n  \"crates/alo-files\",\n  \"crates/alo-capability\",\n]\n"
                .to_owned(),
        ),
        "crates/alo-files/src/verbs.rs" => {
            Some("pub fn declare_into(verbs: &mut Verbs) {}".to_owned())
        }
        "crates/alo-capability/src/verbs.rs" => {
            Some("pub fn declare(&mut self, verb: Verb) {}".to_owned())
        }
        _ => None,
    }
}

/// The check over a fixture, as what it held or the findings it produced.
fn checking(document: &str) -> Result<Held, Vec<Finding>> {
    held(
        &two_verbs(),
        &["alo-files"],
        document,
        A_DEFINITION,
        &a_workspace,
    )
}

/// A sound document is held, and counted the way the promise actually sits: one
/// verb with a plain way, one owed.
///
/// It runs first for the reason every refusal below depends on: a check that
/// refused everything would pass all of them and mean nothing.
#[test]
fn a_document_that_adds_up_is_held_and_counted() {
    let what = checking(&a_sound_document())
        .unwrap_or_else(|findings| panic!("a sound document was refused: {findings:?}"));
    assert_eq!(what.verbs(), 2);
    assert_eq!(what.by_hand(), 1);
    assert_eq!(what.partly(), 0);
    assert_eq!(what.owed(), 1);
}

/// **A verb added with nothing said about it** — the finding this whole crate
/// exists for, and the one that would otherwise arrive silently.
#[test]
fn a_verb_the_document_says_nothing_about_is_the_finding() {
    let three = {
        let mut verbs = two_verbs();
        let mut all = Verbs::default();
        alo_files::declare_into(&mut all).expect("the six file verbs declare");
        verbs
            .declare(
                all.of("archive_folder")
                    .expect("archive_folder is one of the six")
                    .clone(),
            )
            .expect("a third name, declared once");
        verbs
    };
    let findings = held(
        &three,
        &["alo-files"],
        &a_sound_document(),
        A_DEFINITION,
        &a_workspace,
    )
    .expect_err("a verb nobody answered was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AVerbNobodyAnswered { verb } if verb == "archive_folder"
        )),
        "the check did not name the verb the document says nothing about: {findings:?}"
    );
}

/// **A crate declaring verbs that nothing handed in** — every verb in it would be
/// invisible, and the check would go on passing in exactly the same colour.
#[test]
fn a_crate_that_declares_verbs_behind_the_checks_back_is_refused() {
    let with_another = |named: &str| match named {
        "Cargo.toml" => Some(
            "[workspace]\nmembers = [\n  \"crates/alo-files\",\n  \"crates/alo-printing\",\n]\n"
                .to_owned(),
        ),
        "crates/alo-printing/src/verbs.rs" => {
            Some("pub fn declare_into(verbs: &mut Verbs) {}".to_owned())
        }
        other => a_workspace(other),
    };
    let findings = held(
        &two_verbs(),
        &["alo-files"],
        &a_sound_document(),
        A_DEFINITION,
        &with_another,
    )
    .expect_err("a crate declaring verbs that nothing handed in was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AVerbListNobodyHandedIn { crate_name } if crate_name == "alo-printing"
        )),
        "a new crate's whole verb list was invisible to this check: {findings:?}"
    );
}

/// A verb with neither a plain way nor a debt: an entry that talks about the verb
/// and answers nothing.
#[test]
fn a_verb_with_no_plain_way_and_nothing_owed_is_refused() {
    let empty = a_sound_document().replace(
        "**Owed at:** [v0.5] — the file manager is v0.5 work and nothing on this machine\nshows a folder to drag anything into yet.",
        "It is being thought about.",
    );
    let findings = checking(&empty).expect_err("a verb with neither was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AVerbWithNeither { verb } if verb == "move_file"
        )),
        "{findings:?}"
    );
}

/// A plain way that quotes nothing from the definition. The sentence reads
/// perfectly well, which is exactly why it is refused: nothing anywhere is on the
/// hook for building it.
#[test]
fn an_answer_that_quotes_no_promise_is_refused() {
    let unquoted = a_sound_document().replace(
        "in the file\nmanager — `A file manager, with trash, and archives that open`.",
        "in the file manager, which is a thing every desktop has always had.",
    );
    assert!(
        checking(&unquoted)
            .expect_err("an answer promising nothing was accepted")
            .iter()
            .any(|finding| matches!(finding, Finding::AnAnswerNamingNoSurface { .. })),
        "a plain way nothing in the definition promises was accepted, so ADR \
         0009's rule was answered with a sentence"
    );
}

/// A quotation the definition does not make — a promise reworded, or one that was
/// withdrawn and took a verb's plain way with it.
#[test]
fn an_answer_quoting_a_promise_that_is_not_there_is_refused() {
    let reworded = a_sound_document().replace(
        "`A file manager, with trash, and archives that open`.",
        "`A file manager, with a wastebasket, and archives that open`.",
    );
    let findings =
        checking(&reworded).expect_err("an answer quoting a promise nobody makes was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AWayNothingPromises { verb, quoted }
                if verb == "list_folder" && quoted.contains("wastebasket")
        )),
        "a quotation that had drifted from the definition was accepted: {findings:?}"
    );
}

/// A quotation that fits two promises, so the release that owes a person their
/// plain way would depend on which line the check reached first.
#[test]
fn an_answer_that_fits_two_promises_is_refused() {
    let twice_over = format!(
        "{A_DEFINITION}- [v1] A file manager, with trash, and archives that open, on a server\n"
    );
    let findings = held(
        &two_verbs(),
        &["alo-files"],
        &a_sound_document(),
        &twice_over,
        &a_workspace,
    )
    .expect_err("an answer matching two promises was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AWayPromisedMoreThanOnce { promises, .. } if *promises == 2
        )),
        "a quotation whose words appear in two promises answered a verb at \
         whichever release it reached first: {findings:?}"
    );
}

/// A debt with nobody on the hook for it, and a debt owed at a release nobody
/// ships.
#[test]
fn a_debt_with_no_release_or_a_release_nobody_ships_is_refused() {
    let nameless = a_sound_document().replace("**Owed at:** [v0.5] —", "**Owed at:**");
    assert!(
        checking(&nameless)
            .expect_err("a debt naming no release was accepted")
            .iter()
            .any(|finding| matches!(finding, Finding::AnOwedAnswerWithNoRelease { .. })),
        "the plain way was owed by nobody in particular"
    );

    let invented = a_sound_document().replace("[v0.5] —", "[v2] —");
    let findings = checking(&invented).expect_err("a release nobody ships was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AReleaseNobodyShips { named, .. } if named == "v2"
        )),
        "a verb was owed at a release the definition makes no promises for: \
         {findings:?}"
    );
}

/// An answer said as a shrug, in both of the forms an answer can take.
///
/// The plain half needs a definition of its own: a shrug that still quotes a
/// promise only fits where the promise itself is short, and *a terminal* is the
/// shortest thing this definition offers a person.
#[test]
fn a_shrug_is_not_an_answer_in_either_form() {
    let terse = format!("{A_DEFINITION}- [v0.5] A terminal\n");
    let plain = a_sound_document().replace(
        "a person opens the folder and reads what is in it, in the file\nmanager — \
         `A file manager, with trash, and archives that open`.",
        "`A terminal`.",
    );
    let findings = held(&two_verbs(), &["alo-files"], &plain, &terse, &a_workspace)
        .expect_err("a shrug was accepted as how a person lists a folder");
    assert!(
        findings
            .iter()
            .any(|finding| matches!(finding, Finding::AShrugRatherThanAnAnswer { .. })),
        "a verb's plain way was answered by a shrug: {findings:?}"
    );

    let owed = a_sound_document().replace(
        "the file manager is v0.5 work and nothing on this machine\nshows a folder to drag anything into yet.",
        "not yet.",
    );
    assert!(
        checking(&owed)
            .expect_err("`not yet` was accepted as why a person cannot do something")
            .iter()
            .any(|finding| matches!(finding, Finding::AShrugRatherThanAnAnswer { .. })),
        "a verb's plain way was owed with a shrug"
    );
}

/// An entry about a verb this machine does not declare, and a verb answered
/// twice.
#[test]
fn an_entry_about_no_verb_or_a_verb_answered_twice_is_refused() {
    let renamed = a_sound_document().replace("### list_folder", "### list_the_folder");
    let findings = checking(&renamed).expect_err("an entry about no verb was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AnEntryAboutNoVerb { verb } if verb == "list_the_folder"
        )),
        "an answer about a verb nothing declares was accepted: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| matches!(finding, Finding::AVerbNobodyAnswered { .. })),
        "the verb left with nothing said about it was not named: {findings:?}"
    );

    let twice = a_sound_document().replace(
        "## What this document does not do",
        "### list_folder\n\n**By hand:** a person opens the folder in \
         `A file manager, with trash, and archives that open` and reads it.\n\n\
         ## What this document does not do",
    );
    assert!(
        checking(&twice)
            .expect_err("a verb answered by two entries was accepted")
            .iter()
            .any(|finding| matches!(finding, Finding::AVerbAnsweredTwice { .. })),
        "two answers about one verb were both accepted, and the shorter one is \
         the one that gets read"
    );
}

/// And the two that would otherwise be silent: a document this crate cannot find,
/// and a workspace it cannot walk.
#[test]
fn a_document_or_a_workspace_that_cannot_be_read_is_checking_nothing() {
    let renamed = a_sound_document().replace(THE_ANSWERS, "## The verbs");
    let findings = checking(&renamed).expect_err("a document with no entries was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::NothingToCheck { verbs, .. } if *verbs == 2
        )),
        "a document with no entries in it passed as a check on two verbs: \
         {findings:?}"
    );

    let nothing = |_: &str| None;
    assert!(
        held(
            &two_verbs(),
            &["alo-files"],
            &a_sound_document(),
            A_DEFINITION,
            &nothing,
        )
        .expect_err("a workspace that could not be read was accepted")
        .iter()
        .any(|finding| matches!(finding, Finding::NoWorkspaceToWalk { .. })),
        "a workspace with nothing in it was taken for a workspace where nothing \
         declares a verb"
    );
}
