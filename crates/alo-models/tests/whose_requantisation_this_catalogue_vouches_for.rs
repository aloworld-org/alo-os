//! Whose requantisation this catalogue may vouch for.
//!
//! Rule 4 made a quantisation point at a file and rule 5 made the size belong to
//! it. Both refusals landed on the same two entries — `eurollm-9b-instruct` and
//! `teuken-7b-instruct`, whose publishers ship safetensors and no GGUF — and
//! left the question underneath them open: **may an entry name a file the
//! publisher did not publish, and what is this catalogue claiming when it does?**
//!
//! ADR 0026 answers it. A third party's artefact is nameable, and naming one
//! costs three statements — whose it is, which file exactly, and what a reader
//! needs to know — so that what this catalogue borrows is a *file* and never a
//! measurement. This file is that decision as a test rather than as prose.
//!
//! One test per acceptance criterion in `docs/autonomy/v0-01-lane-b-plan.md`'s
//! task 14:
//!
//! - **The decision is written down** where a citation of it lands, says what
//!   its status is, and answers each of the four questions the task names.
//! - **The rule is in the catalogue's own rules**, beside rules 4 and 5, because
//!   the next curator reads those and not this lane's plan.
//! - **A borrowed file that is not stated is refused** — every way one can be
//!   named without being vouched for, in front of the loader.
//! - **A grade belongs to the artefact it was earned against**, and an entry
//!   that names no file cannot carry one.
//! - **Nothing moved that this decision may not move**: no grade came with a
//!   borrowed file, and the two entries it is about carry none. Task 15 then
//!   chose an upload for each of them, which is why the last two tests here ask
//!   what survives any particular choice rather than asserting that none was
//!   made — that part is the history's to hold, not a test's.
//!
//! It needs no model, no runtime and no socket: it reads this repository's own
//! files and the catalogue compiled into this crate.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{fs, path::Path};

use alo_models::{Catalogue, Driving, Model};

/// The decision this change is.
const THE_DECISION: &str = "docs/decisions/0026-whose-requantisation-this-catalogue-vouches-for.md";

/// The catalogue's own rules, which are its header comment.
const THE_RULES: &str = "crates/alo-models/data/catalogue.toml";

/// The two entries whose publishers ship no quantised artefact, and which this
/// decision is about without changing.
const THE_TWO: [&str; 2] = ["eurollm-9b-instruct", "teuken-7b-instruct"];

/// A `sha256` as a repository publishes one, for the fixtures below.
const A_PIN: &str = "3f786850e387550fdab836ed7e6dc881de23001b0e5b4b0e2e7c8d1a9f0c4b6d";

/// A file of this repository, read.
fn reading(named: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// One entry of the catalogue we ship.
fn the_entry(id: &str) -> Model {
    Catalogue::built_in()
        .expect("the built-in catalogue loads; its own test says so first")
        .get(id)
        .unwrap_or_else(|| panic!("the catalogue no longer offers `{id}`"))
        .clone()
}

/// An entry written out, sound in everything but what a test then breaks.
///
/// A four-bit artefact of 1.7 billion parameters, so rule 5's arithmetic holds
/// and every refusal below is about the fault it names.
fn an_entry(artefact: bool, grade: &str, borrowed: Option<&str>) -> String {
    let pair = if artefact {
        "quantisation = \"Q4_K_M\"\nartefact = \"runtime:borrowed-q4_K_M\""
    } else {
        ""
    };
    let bytes = if artefact {
        "1_060_000_000"
    } else {
        "3_400_000_000"
    };
    let (vram, ram) = if artefact { (2.0, 3.0) } else { (4.0, 6.0) };
    let whose = borrowed.unwrap_or_default();
    format!(
        "[[model]]\nid = \"borrowed\"\nname = \"Borrowed\"\npublisher = \"A Publisher\"\n\
         parameters_b = 1.7\n{pair}\n\
         download_bytes = {bytes}\nmin_vram_gb = {vram}\nmin_ram_gb = {ram}\n\
         on_cpu = \"comfortable\"\ndrives_verbs = \"{grade}\"\n\
         upstream = \"https://example.test/borrowed\"\n\
         licence = {{ name = \"Apache-2.0\", spdx = \"Apache-2.0\", \
         commercial_use = \"permitted\" }}\n{whose}"
    )
}

/// A `[model.requantised]` block, written out.
fn whose(by: &str, sha256: &str, note: &str) -> String {
    format!("\n[model.requantised]\nby = \"{by}\"\nsha256 = \"{sha256}\"\nnote = \"{note}\"\n")
}

/// A statement that says all three things.
fn stated() -> String {
    whose(
        "somebody-else",
        A_PIN,
        "built from the publisher's own release; the upload carries no terms of its own",
    )
}

/// One refusal, shown happening: the entry fails to load and says why in words a
/// curator can act on.
fn refused(text: &str, saying: &str, complaint: &str) {
    match Catalogue::parse(text) {
        Ok(_) => panic!("{complaint}"),
        Err(why) => assert!(
            why.to_string().contains(saying),
            "{complaint}: it was refused, and for `{why}` rather than for `{saying}`"
        ),
    }
}

/// **The decision is written down, and it answers the questions it was asked.**
///
/// The task names four: whether a third party's artefact may be named at all,
/// what makes one nameable, how a grade earned against it is reported, and what
/// happens to an entry when the answer is no. A decision that recommended
/// without answering the fourth would leave the two entries it is about in
/// exactly the silence it was written to end.
#[test]
fn the_decision_is_recorded_and_answers_what_it_was_asked() {
    let decision = reading(THE_DECISION);

    assert!(
        decision
            .lines()
            .any(|line| line.starts_with("**Status:**") && line.to_lowercase().contains("accepted")),
        "{THE_DECISION} records no status, so a reader who followed a citation cannot tell a \
         recommendation from a rule"
    );
    // Whose judgement it was, which every decision taken under the standing
    // delegation says out loud rather than implying an owner reviewed it.
    assert!(
        decision.contains("standing delegation"),
        "{THE_DECISION} does not say whose judgement took it"
    );

    for (question, answered) in [
        ("may one be named at all", "may be named"),
        ("what makes one nameable", "sha256"),
        ("who made it", "requantiser"),
        ("how a grade is reported", "belongs to the artefact"),
        (
            "what happens when the answer is no",
            "carried and not omitted",
        ),
        ("what it may not weaken", "ADR 0007"),
        ("what quantising it ourselves would cost", "redistributed"),
    ] {
        assert!(
            decision.contains(answered),
            "{THE_DECISION} does not answer {question}: nothing in it says `{answered}`"
        );
    }

    // Options with what each costs, and a recommendation — the shape ADR 0024
    // and ADR 0025 used, which is what makes a decision checkable against its
    // own reasoning later.
    for heading in [
        "### Option A",
        "### Option B",
        "### Option C",
        "### Option D",
        "## The recommendation",
        "## Consequences",
    ] {
        assert!(
            decision.contains(heading),
            "{THE_DECISION} has no `{heading}` section"
        );
    }
}

/// **The rule is in the catalogue's own rules**, beside rules 4 and 5, because
/// the next curator reads those and not this plan.
///
/// Held to what a curator has to be able to act on: that there is a sixth rule
/// at all, that it says what a borrowed file costs, and that it says what
/// happens when that cost cannot be paid — a rule stating only the permission
/// would read as an invitation.
#[test]
fn the_rule_is_written_into_the_catalogues_own_rules() {
    let rules = reading(THE_RULES);
    let header: String = rules
        .lines()
        .take_while(|line| line.starts_with('#') || line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        header.contains("Six rules") && header.contains("# 6. "),
        "the catalogue's header does not carry a sixth rule, so this decision lives only in a \
         document the next curator will not open"
    );
    let (_, after) = header
        .split_once("# 6. ")
        .expect("the header has a rule 6; the assertion above said so");
    let rule_six = after
        .lines()
        .take_while(|line| !line.starts_with("# ") || line.starts_with("#    "))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !rule_six.contains("entries have been measured"),
        "the extraction ran past rule 6 into the header's narrative, so every assertion below \
         could be satisfied by a sentence the rule does not make"
    );

    for (what, missing) in [
        ("by", "who made the file"),
        ("sha256", "which file exactly"),
        ("note", "what a reader needs to know about it"),
        ("ADR 0026", "where the decision is"),
        (
            "carried rather than omitted",
            "what becomes of an entry that can pay none of it",
        ),
    ] {
        assert!(
            rule_six.contains(what),
            "rule 6 does not say {missing}: nothing in it mentions `{what}`"
        );
    }
    // And the sentence that keeps ADR 0007 whole, in the rule rather than only
    // in the decision: what is borrowed is a file, never a measurement.
    assert!(
        rule_six.contains("never a measurement"),
        "rule 6 does not say that a borrowed file is not a borrowed grade, which is the one way \
         this rule could be read as weakening ADR 0007"
    );
}

/// **A statement that says all three things loads**, so the refusals below are
/// about the faults they name rather than about a shape the loader never
/// accepted.
#[test]
fn an_artefact_somebody_else_made_may_be_named_when_the_entry_says_whose_it_is() {
    let text = an_entry(true, "not-measured", Some(&stated()));
    let catalogue = Catalogue::parse(&text).expect("a fully stated third-party artefact loads");
    let model = catalogue
        .models
        .first()
        .expect("the fixture has one entry")
        .clone();
    let borrowed = model
        .requantised
        .as_ref()
        .expect("the entry states whose artefact it is");
    assert_eq!(borrowed.by, "somebody-else");
    assert_eq!(borrowed.sha256, A_PIN);
    assert!(!borrowed.note.trim().is_empty());
}

/// **Every way a borrowed file can be named without being vouched for**, in
/// front of the loader.
///
/// This is the half a green catalogue cannot show anybody: the rule is only a
/// rule if the entries it exists to refuse are refused.
#[test]
fn a_borrowed_artefact_that_is_not_stated_is_refused() {
    // No name for who made it.
    refused(
        &an_entry(true, "not-measured", Some(&whose("  ", A_PIN, "a note"))),
        "no name for who",
        "an artefact somebody else made, attributed to nobody, was accepted",
    );

    // The publisher's own artefact described as a stranger's, which is a claim
    // about provenance that is not true.
    refused(
        &an_entry(
            true,
            "not-measured",
            Some(&whose("a publisher", A_PIN, "a note")),
        ),
        "own publisher",
        "a first-party artefact described as a requantisation was accepted",
    );

    // A pin nothing can check, which is the tag-re-pointed-under-us case.
    for pin in ["", "sha256-of-something", &"a".repeat(63), &"F".repeat(64)] {
        refused(
            &an_entry(
                true,
                "not-measured",
                Some(&whose("somebody-else", pin, "a note")),
            ),
            "pin nothing can check",
            "a third party's artefact with no digest behind it was accepted",
        );
    }

    // Nothing said about the file, which is rule 1's harm one field over — and
    // the sentence a curator who has not looked at it cannot write.
    refused(
        &an_entry(
            true,
            "not-measured",
            Some(&whose("somebody-else", A_PIN, " ")),
        ),
        "nothing said about it",
        "a third party's artefact with an empty note was accepted",
    );

    // A requantiser beside no artefact at all: whose file, when the entry names
    // none?
    refused(
        &an_entry(false, "not-measured", Some(&stated())),
        "names nobody's",
        "a requantiser was accepted on an entry that points at no file",
    );

    // And each of the three missing outright, which is the shape a curator
    // leaves when they stop halfway.
    for half in [
        format!("\n[model.requantised]\nsha256 = \"{A_PIN}\"\nnote = \"a note\"\n"),
        "\n[model.requantised]\nby = \"somebody-else\"\nnote = \"a note\"\n".to_owned(),
        format!("\n[model.requantised]\nby = \"somebody-else\"\nsha256 = \"{A_PIN}\"\n"),
    ] {
        assert!(
            Catalogue::parse(&an_entry(true, "not-measured", Some(&half))).is_err(),
            "a requantisation block missing one of its three fields was accepted: {half}"
        );
    }

    // A block nothing reads is worse than no block: spelled another way, it
    // would vanish and leave the entry looking like the publisher's own.
    refused(
        &an_entry(
            true,
            "not-measured",
            Some(&format!(
                "\n[model.requantized]\nby = \"somebody-else\"\nsha256 = \"{A_PIN}\"\nnote = \"n\"\n"
            )),
        ),
        "unknown field",
        "a provenance block spelled another way was ignored rather than refused",
    );
}

/// **A grade belongs to the artefact it was earned against.**
///
/// The reporting half of the decision, and the one that keeps ADR 0007 whole
/// from the other end: a measurement is ours, and the thing measured is named.
/// An entry that claims a grade with no file behind it is a number about no
/// weights in particular.
#[test]
fn a_grade_names_the_file_it_was_earned_against() {
    for grade in ["reliably", "sometimes", "rarely"] {
        refused(
            &an_entry(false, grade, None),
            "names no artefact",
            "a grade on an entry that points at no file was accepted",
        );
    }
    // Not measured is the one an entry with no artefact may state, and it is
    // what both European entries say.
    assert!(
        Catalogue::parse(&an_entry(false, "not-measured", None)).is_ok(),
        "an unmeasured entry that names no artefact was refused, which would leave a model \
         nobody has quantised for us with nowhere to be carried"
    );

    // And the catalogue we ship answers which file every grade in it is about.
    for model in Catalogue::built_in()
        .expect("the built-in catalogue loads")
        .models
    {
        assert_eq!(
            model.graded_against().is_some(),
            model.drives_verbs.has_been_measured(),
            "`{}` is graded `{:?}` and cannot say which artefact that grade is about",
            model.id,
            model.drives_verbs
        );
        if let Some(artefact) = model.graded_against() {
            assert_eq!(
                Some(artefact),
                model.artefact.as_deref(),
                "`{}` reports a grade against a file it does not name",
                model.id
            );
        }
    }
}

/// **Every borrowed file in the catalogue we ship is one the price was paid
/// for**, and no grade came with it.
///
/// This decision moved nothing when it was taken: it named no artefact for
/// anybody, because choosing whose file an entry means is a curation act with a
/// name on it and it was left to task 15. That task then paid the price for
/// both European entries, so the assertion worth keeping is no longer *nothing
/// was chosen* — the history holds that — but the one that outlives any
/// particular choice: **whatever is chosen, it is stated in full, and it brings
/// no measurement with it.**
///
/// The loader refuses a half-stated block, which the test above shows happening
/// on fixtures. This asks the same question of the data actually shipped, where
/// a curator in a hurry is the one who would answer it wrongly, and adds the
/// rule no loader can check for them: a borrowed file may never arrive carrying
/// a grade nobody here ran (ADR 0007, rule 3).
#[test]
fn every_borrowed_file_the_catalogue_ships_is_stated_and_brings_no_grade_with_it() {
    let catalogue = Catalogue::built_in().expect("the built-in catalogue loads");
    let measured = measured_entries();
    for model in &catalogue.models {
        let Some(borrowed) = &model.requantised else {
            continue;
        };
        assert!(
            model.quantised_at().is_some(),
            "`{}` says whose artefact it names and names none",
            model.id
        );
        assert!(
            !borrowed.by.trim().is_empty()
                && !borrowed.note.trim().is_empty()
                && borrowed.sha256.len() == 64,
            "`{}` states a requantisation block that does not say all three things",
            model.id
        );
        assert!(
            !borrowed
                .by
                .trim()
                .eq_ignore_ascii_case(model.publisher.trim()),
            "`{}` describes its own publisher's artefact as a stranger's",
            model.id
        );
        assert!(
            !model.drives_verbs.has_been_measured() || measured.contains(&model.id.as_str()),
            "`{}` carries a grade against somebody else's file and is not on the list of entries \
             this repository has actually run `alo-driving` against. A borrowed file is a file, \
             never a measurement",
            model.id
        );
    }
}

/// **And the two entries the question was about still claim no grade.**
///
/// Task 15 gave each of them an artefact; what it could not give either is a
/// measurement, because this lane's box cannot hold a 7B model at four bits. So
/// the honest state is an entry that names a file and says nobody has run it,
/// and that is the state this asserts — separately from the choosing, so that a
/// future change which measures one of them fails here and is read rather than
/// waved through.
#[test]
fn neither_european_entry_gained_a_grade_when_it_gained_a_file() {
    for id in THE_TWO {
        let model = the_entry(id);
        assert_eq!(
            model.drives_verbs,
            Driving::NotMeasured,
            "`{id}` gained a grade; was `alo-driving` run against the artefact it names?"
        );
        assert_eq!(
            model.graded_against(),
            None,
            "`{id}` reports a file a grade was earned against, and it has no grade"
        );
    }
}

/// Every entry this repository has run `alo-driving` against, as
/// `crates/alo-models/src/catalogue.rs`'s own list keeps it.
///
/// Duplicated here rather than shared on purpose: that list is a unit test's
/// constant, and a borrowed file arriving with a grade is exactly the case
/// where a shared constant would be updated to match the data instead of the
/// other way round.
fn measured_entries() -> [&'static str; 7] {
    [
        "phi-3-mini-instruct",
        "llama-3.2-3b-instruct",
        "qwen2.5-3b-instruct",
        "gemma-2-2b-instruct",
        "smollm2-1.7b-instruct",
        "qwen3-1.7b",
        "granite-3.2-2b-instruct",
    ]
}
