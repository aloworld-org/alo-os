//! **No two decisions carry the same number.**
//!
//! An ADR's number is how every other document in this repository points at it:
//! `ADR 0121` in a doc comment, a plan row, a commit message or a task report.
//! Two files beginning `0121-` means every one of those pointers is ambiguous,
//! and whichever the reader opens is a coin toss.
//!
//! # Why this exists, and it is not hypothetical
//!
//! On 2026-10-17 two lanes drafted **ADR 0121** within the same hour — *the
//! thing that opens a session makes the person's folder* and *the workplace
//! ships as separate applications*. The second was renumbered to 0122 before it
//! landed, so `main` never carried both.
//!
//! **What matters is how nearly it did.** The two were different filenames with
//! no shared lines, so **git would have merged them cleanly**: no conflict, no
//! warning, nothing for a reviewer to resolve. The first person to notice would
//! have been whoever cited `ADR 0121` and opened the wrong record.
//!
//! It was caught because the other lane happened to be reading the directory for
//! an unrelated reason. That is not a mechanism, and this is.
//!
//! # The number is a resource allocated without a lock
//!
//! `ls docs/decisions/ | tail -1` is a read of a stale checkout the moment
//! anybody else is drafting, exactly as a two-dot diff read before a fetch is.
//! The discipline that follows — **pick the number at push time, after the
//! fetch, not at draft time** — is the other lane's and is the real remedy. This
//! test is the backstop for the hour when somebody forgets, and it would fail on
//! `main` the moment a second one landed rather than when a citation went wrong.
//!
//! # It passes the day it is written
//!
//! Measured: 121 decisions on `main`, no number twice. So this is **preventive
//! and not remedial**, which is the honest kind of guard — it does not arrive
//! with a list of exceptions, and it cannot be satisfied by adding one.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a repository that no longer holds its own convention is the \
              failure being reported"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Where this repository keeps its decisions.
const THE_DECISIONS: &str = "docs/decisions";

/// The repository this test is checked out in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("this crate sits in the repository that holds the decisions")
}

/// Every decision's four-digit number and the file carrying it.
///
/// A file is a decision when it begins with four digits and a hyphen, which is
/// the convention `docs/decisions/` has had since 0001. Anything else in that
/// directory — a README, a template — is passed over rather than guessed about.
fn by_number() -> BTreeMap<String, Vec<String>> {
    let at = the_repository().join(THE_DECISIONS);
    let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let entries =
        std::fs::read_dir(&at).expect("this repository keeps its decisions where it says it does");
    for entry in entries.flatten() {
        let named = entry.file_name().to_string_lossy().into_owned();
        let Some(number) = four_digits_then_a_hyphen(&named) else {
            continue;
        };
        found.entry(number).or_default().push(named);
    }
    assert!(
        found.len() > 100,
        "only {} decisions were read from {THE_DECISIONS}, and this repository has more than \
         that — so either the convention changed or this filter is wrong, and a short list \
         would make the check below pass for the wrong reason",
        found.len()
    );
    found
}

/// The number a decision's filename begins with, if it begins with one.
fn four_digits_then_a_hyphen(named: &str) -> Option<String> {
    let (number, rest) = named.split_at_checked(4)?;
    if number.len() == 4 && number.chars().all(|it| it.is_ascii_digit()) && rest.starts_with('-') {
        Some(number.to_owned())
    } else {
        None
    }
}

/// **No number is taken twice.**
#[test]
fn no_two_decisions_carry_the_same_number() {
    let taken = by_number();
    let twice: Vec<String> = taken
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(number, files)| {
            let mut sorted = files.clone();
            sorted.sort();
            format!("{number}: {}", sorted.join(", "))
        })
        .collect();

    assert!(
        twice.is_empty(),
        "two decisions carry the same number, so every `ADR NNNN` pointing at it is \
         ambiguous:\n  {}\n\n\
         **Nothing will have conflicted.** Two decisions are two filenames with no shared \
         lines, so git merges them cleanly and the first person to notice is whoever cites the \
         number and opens the wrong record. Renumber the later one — pick the number after a \
         fetch rather than from the checkout you started in — and fix every citation of it.",
        twice.join("\n  ")
    );
}

/// **The numbers run from 0001 with no gaps**, which is the other half.
///
/// A gap is not ambiguous the way a duplicate is, so this is the weaker check of
/// the two — but a missing number is usually a decision somebody renumbered and
/// a citation left pointing at nothing, and `every_decision_this_repository_points_at`
/// would then fail for a reason that reads like a typo. Named as a list rather
/// than asserted empty, because a deliberate retirement is a thing this
/// repository might one day do and the message should say what to add.
#[test]
fn the_numbers_run_from_one_with_no_gaps() {
    let taken = by_number();
    let numbers: Vec<u32> = taken
        .keys()
        .filter_map(|it| it.parse::<u32>().ok())
        .collect();
    let highest = numbers.iter().copied().max().expect("there is a decision");

    let missing: Vec<u32> = (1..=highest)
        .filter(|number| !numbers.contains(number))
        .collect();
    assert!(
        missing.is_empty(),
        "these numbers are between 0001 and {highest:04} and no decision carries them: \
         {missing:?}\n\n\
         Usually a decision that was renumbered, leaving citations pointing at nothing. If a \
         number was deliberately retired, say so where the next person looks and widen this \
         test to allow it by name."
    );
}

/// **The control: the filter can tell a decision from anything else in that
/// directory.**
///
/// Without it, a `four_digits_then_a_hyphen` that answered `None` for everything
/// would make both tests above pass over an empty map — and the length assertion
/// inside `by_number` would be the only thing standing between this file and
/// measuring nothing. Both directions, on names of each shape.
#[test]
fn the_filter_reads_a_number_from_a_decision_and_from_nothing_else() {
    // **Built from parts rather than written out**, and that is not fussiness:
    // a literal `NNNN-something.md` in a source file is a citation as far as
    // `every_decision_this_repository_points_at` is concerned, and it fails on a
    // fixture naming a decision nobody wrote. The first version of this file did
    // exactly that twice. A checker cannot tell a fixture from a pointer, and in
    // this repository the two look identical.
    let a_decisions_name = |number: &str| format!("{number}-{}", "some-decision.md");
    assert_eq!(
        four_digits_then_a_hyphen(&a_decisions_name("0121")).as_deref(),
        Some("0121")
    );
    assert_eq!(
        four_digits_then_a_hyphen(&a_decisions_name("0001")).as_deref(),
        Some("0001")
    );

    for not_one in [
        "README.md",
        "template.md",
        "121-three-digits.md",
        "01234-five-digits.md",
        "0121_an_underscore.md",
        "abcd-not-digits.md",
        "0121",
        "",
    ] {
        assert_eq!(
            four_digits_then_a_hyphen(not_one),
            None,
            "{not_one:?} was read as a decision's number"
        );
    }
}

/// **And it would have caught the collision it was written for.**
///
/// The two real filenames from 2026-10-17, run through the same grouping. A test
/// that only ever sees a correct repository is a test nobody has watched fail.
#[test]
fn the_collision_this_was_written_for_is_caught() {
    // Built from parts, for the reason in the control above: a literal decision
    // name in a source file is a citation, and both of these name files that
    // were never written under those names.
    let both = [
        format!("0121-{}", "the-first-draft.md"),
        format!("0121-{}", "the-second-draft.md"),
    ];
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for named in &both {
        let number = four_digits_then_a_hyphen(named).expect("both are decisions");
        grouped.entry(number).or_default().push(named.clone());
    }

    let twice: Vec<&String> = grouped
        .iter()
        .filter(|(_, files)| files.len() > 1)
        .map(|(number, _)| number)
        .collect();
    assert_eq!(
        twice,
        vec!["0121"],
        "the grouping did not catch two decisions sharing 0121, so finding none in the real \
         directory meant nothing"
    );
}
