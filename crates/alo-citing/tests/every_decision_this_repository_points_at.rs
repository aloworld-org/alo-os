//! The check itself, run against the citations this repository really carries.
//!
//! Everything in the crate is the method. This is the measurement: every `.rs`
//! and `.md` file in this checkout, every `ADR` and four digits in them, every
//! decision filename they link to, and the contents of `docs/decisions/` — read
//! off the disk this test is running on, so a citation written tomorrow of a
//! decision nobody wrote fails here rather than in a reader's head a year later.
//!
//! # And the refusals, beside it
//!
//! A green check says the pointers land. It cannot say the check would have
//! noticed if they did not, and that is the half a reader has to believe rather
//! than see — so every finding this crate can produce is put in front of it here
//! against a fixture, and one of them is put in front of it against **this
//! repository**, with a real decision taken off the real list.
//!
//! # Why the fixtures spell their numbers sideways
//!
//! A citation of a decision nobody wrote, written out in full in this file,
//! would be one this file really carries — and the measurement above reads this
//! file. So the fixtures build their pointers from [`NOBODY_WROTE`] rather than
//! writing them, and the check's own tests are therefore the first thing it
//! holds to its rule. That is not a workaround; it is the clearest evidence
//! available that the real reading looks at the real disk.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use alo_citing::{Finding, Held, THE_DECISIONS, held};

/// The repositories other than this one whose decisions this one may cite.
///
/// `alo-workplace` is the workspace above, which has ADRs of its own and is
/// cited by number in four places. This is a standing permission for a number
/// not to resolve here, so an entry nothing cites is a finding: it is one more
/// place a citation of a decision nobody wrote could hide.
const NEIGHBOURS: [&str; 1] = ["alo-workplace"];

/// Where a decision this repository does not have would be numbered.
///
/// Never written beside the word that makes it a citation — see this file's
/// header.
const NOBODY_WROTE: &str = "0099";

/// A number belonging to `alo-workplace` and to no decision here, for the
/// fixture that cites it without saying so.
const ELSEWHERES: &str = "0047";

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the repository is where this crate says it is")
}

/// What is not this repository's own text: what the build wrote, and what git
/// keeps.
fn is_not_ours(name: &str) -> bool {
    matches!(name, ".git" | "target" | "node_modules") || name.starts_with("target-")
}

/// Every `.rs` and `.md` file under `directory`, as a repository-relative path
/// with forward slashes and the text in it.
fn everything_written_under(directory: &Path, below: &str, into: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = if below.is_empty() {
            name.clone()
        } else {
            format!("{below}/{name}")
        };
        if entry.path().is_dir() {
            if !is_not_ours(&name) {
                everything_written_under(&entry.path(), &path, into);
            }
        } else if (name.ends_with(".rs") || name.ends_with(".md"))
            && let Ok(text) = fs::read_to_string(entry.path())
        {
            into.push((path, text));
        }
    }
}

/// The repository's Rust and Markdown, read.
fn the_repositorys_own_words() -> Vec<(String, String)> {
    let mut written = Vec::new();
    everything_written_under(&the_repository(), "", &mut written);
    assert!(
        written.len() > 100,
        "{} file(s) were read, which is not this repository",
        written.len()
    );
    written
}

/// The contents of `docs/decisions/`, each as its filename and its text.
fn the_decisions() -> Vec<(String, String)> {
    let mut written = Vec::new();
    let entries = fs::read_dir(the_repository().join(THE_DECISIONS))
        .expect("this repository keeps its decisions where it says it does");
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Ok(text) = fs::read_to_string(entry.path()) {
            written.push((name, text));
        }
    }
    written
}

/// A list of owned pairs, borrowed for [`held`].
fn borrowed(written: &[(String, String)]) -> Vec<(&str, &str)> {
    written
        .iter()
        .map(|(name, text)| (name.as_str(), text.as_str()))
        .collect()
}

/// **Every decision this repository points at is one somebody wrote, and every
/// decision it has says what it is** — against this repository, on the disk it is
/// checked out on.
#[test]
fn every_decision_this_repository_points_at_exists() {
    let decisions = the_decisions();
    let files = the_repositorys_own_words();
    match held(&borrowed(&decisions), &borrowed(&files), &NEIGHBOURS) {
        Ok(what) => {
            assert!(
                what.decisions() > 20,
                "{} decision(s) were read, which means the check passed over a \
                 directory it never opened",
                what.decisions()
            );
            assert!(
                what.cited() > 100,
                "{} citation(s) were read in a repository that carries hundreds, \
                 so the check stopped looking somewhere",
                what.cited()
            );
            assert!(
                what.named() > 20,
                "{} decision file(s) were linked to, and this repository's own \
                 decisions link to each other far more than that",
                what.named()
            );
            assert!(
                what.elsewhere() > 0,
                "no citation of a neighbouring repository's decisions was seen, \
                 and `{}` is named as one this repository cites",
                NEIGHBOURS.join(", ")
            );
        }
        Err(findings) => {
            let listed: Vec<String> = findings.iter().map(ToString::to_string).collect();
            panic!(
                "{} pointer(s) into this repository's decisions do not land:\n\n- {}",
                listed.len(),
                listed.join("\n\n- ")
            );
        }
    }
}

/// **The check refuses a real decision taken off the real list** — the same
/// measurement as above, with one decision's file removed from what
/// `docs/decisions/` is said to hold.
///
/// Every refusal below this is shown against a fixture, which is what makes each
/// of them cheap to write and easy to disbelieve: a check could pass its fixtures
/// and still never look at this repository. This is the one that says the real
/// reading refuses, on the real disk, for the real reason.
#[test]
fn a_real_decision_taken_off_the_real_list_is_refused() {
    let decisions = the_decisions();
    let files = the_repositorys_own_words();
    let all_of_them = borrowed(&decisions);
    let (gone, rest) = all_of_them
        .split_first()
        .expect("this repository has at least one decision");
    let number = gone
        .0
        .get(..4)
        .expect("a decision's name begins with its number");
    let findings = held(rest, &borrowed(&files), &NEIGHBOURS).expect_err(
        "a decision of this repository was removed and every pointer at it still landed",
    );
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ADecisionNobodyWrote { number: cited, .. } if cited == number
        )),
        "`{}` is cited in this repository, it was taken off the list of what \
         exists, and the check did not name a single one of those citations",
        gone.0
    );
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AFileNobodyWrote { named, .. } if named == gone.0
        )),
        "`{}` is linked to by name in this repository and the check did not \
         notice the file was gone",
        gone.0
    );
}

/// A repository with two decisions, one of which the other cites.
fn a_repository() -> Vec<(String, String)> {
    vec![
        (
            "docs/contracts/agent-verbs.md".to_owned(),
            "No verb runs an arbitrary command (ADR 0001 §1), and the workspace's \
             own `alo-workplace` ADR 0047 said the same."
                .to_owned(),
        ),
        (
            "crates/alo-picking/src/lib.rs".to_owned(),
            "//! A grant is a folder a person picked (ADR 0001 §3), and where \
             inference happens is [ADR 0008](0008-where-inference-happens.md)."
                .to_owned(),
        ),
    ]
}

/// Two decisions, each saying what it is.
fn the_two_decisions() -> Vec<(String, String)> {
    vec![
        (
            "0001-the-capability-model.md".to_owned(),
            "# ADR 0001\n\n**Status:** accepted — the foundation\n".to_owned(),
        ),
        (
            "0008-where-inference-happens.md".to_owned(),
            "# ADR 0008\n\n**Status:** accepted\n".to_owned(),
        ),
    ]
}

/// The check over a fixture, as what it held or the findings it produced.
fn checking(
    decisions: &[(String, String)],
    files: &[(String, String)],
) -> Result<Held, Vec<Finding>> {
    held(&borrowed(decisions), &borrowed(files), &NEIGHBOURS)
}

/// A repository whose pointers land is held, and counted the way it actually
/// sits: three citations of its own decisions, one of a neighbour's, one link.
///
/// It runs first for the reason every refusal below depends on: a check that
/// refused everything would pass all of them and mean nothing.
#[test]
fn a_repository_whose_pointers_land_is_held_and_counted() {
    let what = checking(&the_two_decisions(), &a_repository())
        .unwrap_or_else(|findings| panic!("a sound repository was refused: {findings:?}"));
    assert_eq!(what.decisions(), 2);
    assert_eq!(what.cited(), 3);
    assert_eq!(what.elsewhere(), 1);
    assert_eq!(what.named(), 1);
}

/// **A citation of a decision nobody wrote** — the finding this whole crate
/// exists for, and the one an ADR number makes easiest to believe, because the
/// number looks like a fact.
#[test]
fn a_citation_of_a_decision_nobody_wrote_is_the_finding() {
    let mut files = a_repository();
    files.push((
        "docs/features.md".to_owned(),
        format!("A person can always do it by hand (ADR {NOBODY_WROTE})."),
    ));
    let findings = checking(&the_two_decisions(), &files)
        .expect_err("a citation of a decision nobody wrote was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ADecisionNobodyWrote { number, file, line, .. }
                if number == NOBODY_WROTE && file == "docs/features.md" && *line == 1
        )),
        "a sentence borrowed the authority of a decision nobody made and the \
         check passed: {findings:?}"
    );
}

/// And a link to a decision file that is not there — a rename nobody followed
/// through, which leaves every link to it reading exactly as it did before.
#[test]
fn a_link_to_a_decision_file_nobody_wrote_is_a_finding() {
    let gone = format!("{NOBODY_WROTE}-a-decision-under-another-name.md");
    let mut files = a_repository();
    files.push((
        "ROADMAP.md".to_owned(),
        format!("The exit gate, and [what it rests on]({gone})."),
    ));
    let findings = checking(&the_two_decisions(), &files)
        .expect_err("a link to a decision file that is not there was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::AFileNobodyWrote { named, file, .. }
                if *named == gone && file == "ROADMAP.md"
        )),
        "{findings:?}"
    );
}

/// Two files claiming one number. The pointer lands and is still wrong, because
/// which decision it resolves to depends on which file a reader opened.
#[test]
fn two_decisions_claiming_one_number_is_a_finding() {
    let mut decisions = the_two_decisions();
    decisions.push((
        format!("{}-the-capability-model-again.md", "0001"),
        "# A second one\n\n**Status:** accepted\n".to_owned(),
    ));
    let findings = checking(&decisions, &a_repository())
        .expect_err("two decisions claiming one number were accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::TwoDecisionsOneNumber { number, .. } if number == "0001"
        )),
        "every citation of that number resolves to whichever file a reader \
         opens first, and nothing said so: {findings:?}"
    );
}

/// **A decision whose own file does not say what its status is** — which every
/// reader who follows a citation to it will read as settled.
#[test]
fn a_decision_that_does_not_say_what_it_is_is_a_finding() {
    let mut decisions = the_two_decisions();
    if let Some(first) = decisions.first_mut() {
        first.1 = "# ADR 0001\n\n**Date:** 2026-09-02\n".to_owned();
    }
    let findings = checking(&decisions, &a_repository())
        .expect_err("a decision recording no status was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ADecisionWithNoStatus { file } if file == "0001-the-capability-model.md"
        )),
        "a recommendation waiting on the owner would have been read as a rule: \
         {findings:?}"
    );
}

/// **A neighbouring repository's decision is cited without naming the
/// repository** — and is then a citation of a decision nobody wrote, because
/// that is exactly what it looks like to a reader.
#[test]
fn a_neighbours_decision_cited_without_the_repository_is_refused() {
    let files = vec![(
        "docs/contracts/agent-verbs.md".to_owned(),
        format!("Unchanged from the workspace's ADR {ELSEWHERES}, because a person should see it."),
    )];
    let findings = checking(&the_two_decisions(), &files)
        .expect_err("a bare number belonging to another repository was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ADecisionNobodyWrote { number, .. } if number == "0047"
        )),
        "a reader would have looked for that decision in this repository: \
         {findings:?}"
    );
}

/// A neighbour nobody cites. The list is a standing permission for a number not
/// to resolve, and one that is not needed is one more place a typo can hide.
#[test]
fn a_neighbour_nobody_cites_is_a_finding() {
    let files = vec![(
        "crates/alo-picking/src/lib.rs".to_owned(),
        "//! A grant is a folder a person picked (ADR 0001 §3).".to_owned(),
    )];
    let findings = held(
        &borrowed(&the_two_decisions()),
        &borrowed(&files),
        &NEIGHBOURS,
    )
    .expect_err("a repository named as cited, and cited nowhere, was accepted");
    assert!(
        findings.iter().any(|finding| matches!(
            finding,
            Finding::ANeighbourNobodyCites { repository } if repository == "alo-workplace"
        )),
        "{findings:?}"
    );
}

/// And the two that would otherwise be silent: nothing to check against, and
/// nothing found to check. Either one passes in exactly the same colour as a
/// repository whose pointers all land.
#[test]
fn a_check_that_read_nothing_is_checking_nothing() {
    assert!(
        held(&[], &borrowed(&a_repository()), &NEIGHBOURS)
            .expect_err("a repository with no decisions at all was accepted")
            .iter()
            .any(|finding| matches!(finding, Finding::NoDecisionsToCheck { .. })),
        "every citation in the repository pointed at nothing and the check was \
         content"
    );

    let silent = vec![(
        "docs/features.md".to_owned(),
        "A person can always do it by hand.".to_owned(),
    )];
    assert!(
        checking(&the_two_decisions(), &silent)
            .expect_err("a reading that found no citation at all was accepted")
            .iter()
            .any(
                |finding| matches!(finding, Finding::NothingCitesAnything { files } if *files == 1)
            ),
        "a reading that saw no citation anywhere was taken for a repository that \
         points at nothing"
    );
}
