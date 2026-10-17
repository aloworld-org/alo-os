//! A run of tests, recognised in what a program printed — or nothing, which is
//! the ordinary case and not the failure case.
//!
//! `docs/features.md`, starred: *Output that is not stuck in a character grid.
//! A benchmark draws a chart you can zoom; a long error becomes something you
//! keep beside your editor and come back to tomorrow.*
//!
//! # The design specifies this one, and specifies it completely
//!
//! `docs/design/figma-snapshot/485-28999.xml` holds a frame named **`Rich test
//! result / untrusted program data`**, and it is in the node that *is*
//! authoritative for the gutter and for provenance. Four things in it, in this
//! order:
//!
//! | what the design draws | what it is |
//! |---|---|
//! | `Tests · 23 passed · 1 failed` | the summary, from the numbers |
//! | `Source: cargo test output · Not independently verified` | **the provenance, and it is not optional** |
//! | the failure, at length | the program's own text, unchanged |
//! | three `Ask alo / Action` | not built here — they are the agent's road |
//!
//! Two of that frame's own annotations are the rules this file obeys:
//! ***Formatted output is not system evidence*** and ***Text reveals original
//! bytes*.**
//!
//! The two other rich views the design has — `Terminal / View / Benchmark
//! comparison` and `Terminal / View / File diff` — are **instances of
//! components the snapshot does not contain**, so their insides are not
//! specified anywhere in this repository. This is the one that is, which is why
//! it is the one built. *One that works, rather than half of all of them.*
//!
//! # Everything here is a claim the program made, and the names say so
//!
//! There is no `passed()`. There is [`ATestResult::it_said_passed`], because a
//! count scraped out of a stranger's output is evidence of what was printed and
//! of nothing else — the program may have lied, or been wrong, or be a program
//! that prints test results without running tests.
//!
//! This is the design's *formatted output is not system evidence* turned into
//! method names, so a caller cannot write the sentence that would be wrong
//! without reading the word `said`. [`ATestResult::and_nobody_checked`] carries
//! the provenance and takes no argument, so no shape of this type lacks it —
//! the same shape as the disclaimer on
//! [`crate::what_a_command_will_do::WhatACommandWillDo`].
//!
//! # Recognising nothing is the ordinary case
//!
//! The design draws the **text fallback first** of its three frames, which it
//! explains in its own words: *the fallback is drawn first, which is the design
//! saying that recognising nothing is the ordinary case rather than the failure
//! case.*
//!
//! So [`a_test_result_in`] answers [`None`] for almost everything a terminal
//! ever shows, and that is success. Nothing here reports a refusal, warns, or
//! marks output as unrecognised: the program's text is already on the screen and
//! is what a person reads.
//!
//! # What it refuses, which is most of the file
//!
//! *Untrusted* is the load-bearing word in the plan, and these are reachable by
//! anything that can print:
//!
//! - **a control character anywhere in the line is a refusal.** A summary is
//!   drawn by the terminal in the terminal's own type, outside the cells a
//!   program owns, so a byte that moves a cursor or sets a colour must never
//!   reach it. The grid has already consumed real escape sequences before this
//!   sees a line, so this is defence behind a wall that already stands — and it
//!   is here because the wall is somebody else's and this is the one place that
//!   would be exploited if it fell.
//! - **a number that does not parse is not a number.** No clamping and no
//!   defaulting to nought: the line contributes nothing, because a program that
//!   printed `many passed` has said something this cannot read.
//! - **a sum that would overflow refuses the whole result.** Not a saturating
//!   add, which would quietly show a number nobody printed.
//! - **the provenance cannot be forged**, because it is not copied from the
//!   output. A program that prints *Source: cargo test output · Not
//!   independently verified* changes nothing: that sentence is this
//!   repository's, by key, and what the program printed stays in the cells
//!   labelled `program`.
//! - **the summary cannot be forged either**, for the same reason: it is built
//!   from numbers that were parsed, never from a line that was matched.

use crate::words;

/// How many suite summaries one recognition will add up.
///
/// A workspace run prints one `test result:` line per test binary, and this
/// repository has dozens. The bound is not about memory — it is that a
/// recognition which added up a thousand lines is almost certainly reading
/// something that is not a test run at all, and a wrong summary is worse than
/// no summary.
const AT_MOST_THIS_MANY_SUITES: usize = 256;

/// What a program's output said about a run of tests.
///
/// **Every number here is what was printed.** See this module's header for why
/// each accessor is named `it_said_`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ATestResult {
    /// What the output said passed.
    passed: usize,
    /// What the output said failed.
    failed: usize,
    /// What the output said was ignored.
    ignored: usize,
    /// How many suite summaries were added up to get there.
    suites: usize,
}

impl ATestResult {
    /// How many tests the output said passed.
    #[must_use]
    pub const fn it_said_passed(&self) -> usize {
        self.passed
    }

    /// How many tests the output said failed.
    #[must_use]
    pub const fn it_said_failed(&self) -> usize {
        self.failed
    }

    /// How many tests the output said were ignored.
    #[must_use]
    pub const fn it_said_ignored(&self) -> usize {
        self.ignored
    }

    /// How many suite summaries this was added up from.
    ///
    /// One for a single test binary, and one per binary for a workspace run.
    /// Kept because *23 passed* read off one suite and *23 passed* added up
    /// across forty are different facts, and a person reading a summary of a
    /// workspace run should be able to tell.
    #[must_use]
    pub const fn across_this_many_suites(&self) -> usize {
        self.suites
    }

    /// What printed it, which is the only format this recognises.
    #[must_use]
    pub const fn what_printed_it(&self) -> alo_strings::Word {
        words::FROM_A_CARGO_TEST_RUN
    }

    /// *Not independently verified* — always, for every recognised result.
    ///
    /// Takes `&self` and answers unconditionally, so there is no shape of this
    /// type without it and no argument that turns it off. The design draws this
    /// sentence directly under the summary and its frame's own annotation is
    /// *formatted output is not system evidence*; a summary shown without it
    /// would be this repository asserting a stranger's arithmetic.
    #[must_use]
    pub const fn and_nobody_checked(&self) -> alo_strings::Word {
        words::NOT_INDEPENDENTLY_VERIFIED
    }
}

/// Whether this line is safe to read numbers out of and draw a summary from.
///
/// A control character is a refusal — see this module's header. `is_control`
/// covers C0, C1 and the Unicode control range, which is wider than the escape
/// sequences a terminal would act on and deliberately so: the question here is
/// not *would this move a cursor* but *is this plain text*.
fn it_is_plain_text(line: &str) -> bool {
    !line.chars().any(char::is_control)
}

/// The number before this word on this line, where there is one.
///
/// `44 passed` answers `Some(44)`. `many passed` answers [`None`], and so does
/// a number too large for a `usize` — neither is clamped, because a count this
/// cannot read is a count it must not show.
fn the_number_before(line: &str, word: &str) -> Option<usize> {
    let mut said: Option<usize> = None;
    let mut previous: Option<&str> = None;
    for token in line.split_whitespace() {
        // Trailing punctuation: cargo writes `44 passed;`.
        let bare = token.trim_end_matches([';', ',', '.']);
        if bare == word {
            said = previous?.parse().ok();
            break;
        }
        previous = Some(token.trim_end_matches([';', ',', '.']));
    }
    said
}

/// A run of tests recognised in these lines, or [`None`].
///
/// [`None`] for almost all output a terminal ever shows, and that is the
/// ordinary case rather than a failure — see this module's header.
///
/// # What it reads
///
/// Lines holding `test result:`, which is what `cargo test` prints once per
/// test binary. Every such line contributes its three counts, and they are
/// added up; a workspace run is therefore one result across many suites.
///
/// # What makes it answer [`None`]
///
/// No such line at all; every such line refused; or a sum that would overflow.
/// The refusals are in this module's header, with the reason for each.
#[must_use]
pub fn a_test_result_in(lines: &[String]) -> Option<ATestResult> {
    let mut found = ATestResult {
        passed: 0,
        failed: 0,
        ignored: 0,
        suites: 0,
    };

    for line in lines {
        if found.suites >= AT_MOST_THIS_MANY_SUITES {
            break;
        }
        if !line.contains("test result:") || !it_is_plain_text(line) {
            continue;
        }
        // **All three or none.** A line that gave two of the three has not been
        // read correctly, and showing two thirds of a summary is showing a
        // number nobody printed beside two that were.
        let (Some(passed), Some(failed), Some(ignored)) = (
            the_number_before(line, "passed"),
            the_number_before(line, "failed"),
            the_number_before(line, "ignored"),
        ) else {
            continue;
        };

        // **Checked, not saturating.** A saturating add would show a total the
        // program never printed, which is the one thing this file is for.
        found = ATestResult {
            passed: found.passed.checked_add(passed)?,
            failed: found.failed.checked_add(failed)?,
            ignored: found.ignored.checked_add(ignored)?,
            suites: found.suites.checked_add(1)?,
        };
    }

    (found.suites > 0).then_some(found)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "a_test_result_a_program_printed_tests.rs"]
mod tests;
