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

/// The decisions in `docs/decisions/`, each as its filename and its text, in
/// their numbered order.
///
/// # Filtered first and sorted second, and the order of those two is the whole
/// repair
///
/// This read the directory raw: no filter, no sort. `fs::read_dir` answers in
/// the order the filesystem chooses, and
/// [`a_real_decision_taken_off_the_real_list_is_refused`] takes the **first**
/// entry — so *which decision is removed* was the kernel's choice, on a test
/// whose name says *a real decision*.
///
/// **And the subject has to satisfy two conditions, which not every decision
/// does.** That test asserts the removal is noticed twice over: once as a
/// citation of a number nobody wrote, and once as a **link to a filename**
/// nobody wrote. Measured on 2026-10-01: of 81 decisions, **three are never
/// linked by filename anywhere in the repository** — `0050`, `0071` and `0074`.
/// Pick one of those three and the second assertion cannot pass, however sound
/// the check is.
///
/// So the test failed on a GitHub-hosted runner for six hours and passed on
/// every lane's machine: the runner's filesystem handed back one of the three
/// first, and no lane's did. A latent fault rather than a new one, and the
/// failure was `is linked to by name in this repository and the check did not
/// notice the file was gone` — the second assertion, not the first.
///
/// *Recorded because the first diagnosis of this was wrong and was published
/// before it was reproduced: that `README.md` came back first and the number
/// read off it was `READ`. That would have failed the **first** assertion, and
/// an attempt to reproduce it by planting a dotfile did not fail at all —
/// `read_dir` on ext4 answers in hash order, so planting a name that sorts first
/// does not make it arrive first. The mechanism above is the one in the runner's
/// own panic message.*
///
/// After filtering and sorting, the subject is always the lowest-numbered
/// decision, `0001-the-capability-model.md`, which is linked from 61 other
/// files. Deterministic, and deterministically able to pass.
///
/// **The filter is what makes this correct and the sort only makes it
/// deterministic**, and that distinction is load-bearing rather than pedantic.
/// Sorting alone appears to fix it, because `R` sorts after `0`. It would fix it
/// *by accident*: `.` is `0x2E` and `0` is `0x30`, so **a dotfile sorts before
/// every decision**, and the first `.DS_Store` or editor swapfile in this
/// directory would put the fault straight back with the number read as `.DS_`.
/// One of this fleet's lanes works on a Mac. *Found by that lane, reading the
/// proposed fix rather than the code it replaced.*
///
/// So a later reader must not take the sort for the repair and remove the filter
/// as redundant. The filter is the repair.
///
/// **The stronger form, considered and not taken.** The subject could be *named*
/// in the source rather than chosen by position at all, which is where this
/// family of fault ends: a test whose subject the filesystem picks is asking a
/// question less specific than the one it reports on. It is not taken because a
/// named decision that later stops being cited would fail this test for a reason
/// that has nothing to do with what it checks. Position after filtering and
/// sorting is deterministic, and [`is_a_decisions_name`] is asserted at the use
/// site so that a loosened filter says so rather than computing a number out of
/// whatever it was handed.
fn the_decisions() -> Vec<(String, String)> {
    let mut written = Vec::new();
    let entries = fs::read_dir(the_repository().join(THE_DECISIONS))
        .expect("this repository keeps its decisions where it says it does");
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_a_decisions_name(&name) {
            continue;
        }
        if let Ok(text) = fs::read_to_string(entry.path()) {
            written.push((name, text));
        }
    }
    written.sort();
    assert!(
        !written.is_empty(),
        "no file in {THE_DECISIONS} is named like a decision, so either the \
         convention changed or this filter is wrong — and an empty list would \
         make every check below pass for the wrong reason"
    );
    written
}

/// Whether this filename is a decision's, as `docs/decisions/` names them:
/// `NNNN-subject.md`.
///
/// Four digits and a hyphen, which `README.md`, a dotfile and an editor
/// swapfile all fail. Written as a predicate rather than inline so that the one
/// place that decides what a decision's name is can be cited from the two tests
/// that depend on it.
fn is_a_decisions_name(name: &str) -> bool {
    name.ends_with(".md")
        && name.as_bytes().get(4) == Some(&b'-')
        && name
            .get(..4)
            .is_some_and(|number| number.bytes().all(|byte| byte.is_ascii_digit()))
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
/// # The subject is derived from the property, not from a position
///
/// This test needs a decision that is **both cited by its number and linked by
/// its filename**, because it asserts the removal is noticed in both of those
/// ways. Not every decision is: measured 2026-10-01, of 81 decisions **four are
/// never linked by filename** in any `.rs` or `.md` — `0050`, `0070`, `0071`,
/// `0074` — and `0074`'s number is never cited either.
///
/// So the subject may not be chosen by position. It was `split_first()` on an
/// unsorted `read_dir`, which is how this failed on a hosted runner for six
/// hours while passing on every lane's machine: that filesystem offered one of
/// the four first and no lane's did.
///
/// **Filtering and sorting fixed the symptom and left the cause.** With them,
/// the subject is always the lowest-numbered decision — deterministic, and it
/// passes only because that decision happens to be linked from sixty files. A
/// test that passes because of a fact about `0001` is one that breaks the day
/// `0001` stops being cited, for a reason that has nothing to do with what it
/// checks. *Both lanes that read the fix said so independently, one about naming
/// the subject and one about the enumeration being short by one.*
///
/// So the subject is **searched for**: the lowest-numbered decision whose
/// removal is actually noticed both ways, and a loud refusal if there is none.
/// That has no filesystem dependence, no dependence on any particular decision,
/// and it cannot pass by lucky ordering — which is the difference between a fix
/// verified against the instances somebody enumerated and a fix verified against
/// the property.
#[test]
fn a_real_decision_taken_off_the_real_list_is_refused() {
    let decisions = the_decisions();
    let files = the_repositorys_own_words();
    let all_of_them = borrowed(&decisions);
    let words = borrowed(&files);

    // Every decision in turn, lowest-numbered first, until one is found whose
    // removal the check notices in both of the ways asserted below. The search
    // is the test's subject selection; the assertions are what it is about.
    let mut tried = Vec::new();
    let subject = all_of_them.iter().enumerate().find_map(|(at, gone)| {
        let number = gone.0.get(..4)?;
        let rest: Vec<_> = all_of_them
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != at)
            .map(|(_, kept)| *kept)
            .collect();
        let findings = held(&rest, &words, &NEIGHBOURS).err()?;
        let cited = findings.iter().any(|finding| {
            matches!(finding, Finding::ADecisionNobodyWrote { number: at, .. } if at == number)
        });
        let linked = findings.iter().any(
            |finding| matches!(finding, Finding::AFileNobodyWrote { named, .. } if named == gone.0),
        );
        tried.push((gone.0, cited, linked));
        (cited && linked).then_some(gone.0)
    });

    let subject = subject.unwrap_or_else(|| {
        let account: Vec<String> = tried
            .iter()
            .map(|(name, cited, linked)| {
                format!("    {name}  cited-by-number={cited}  linked-by-filename={linked}")
            })
            .collect();
        panic!(
            "no decision in this repository is both cited by its number and linked \
             by its filename, so removing one cannot be noticed both ways and this \
             check has nothing real to be about. Every decision was tried:\n{}",
            account.join("\n")
        )
    });

    // **Said out loud, because a search that silently settles on anything is the
    // fault this replaced.** The subject is in the output, so a reader never has
    // to work out which decision the test was about — and if it ever moves, the
    // run says so rather than quietly testing something else.
    println!("the decision taken off the list: {subject}");
    assert!(
        is_a_decisions_name(subject),
        "`{subject}` is not a decision's name, so `the_decisions` let something through"
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
