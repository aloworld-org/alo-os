//! A benchmark comparison, recognised in what a program printed — or nothing,
//! which is the ordinary case and not the failure case.
//!
//! `docs/features.md`, starred: *Output that is not stuck in a character grid.
//! **A benchmark draws a chart you can zoom**; a long error becomes something
//! you keep beside your editor and come back to tomorrow.*
//!
//! # The design specifies this one too, since 2026-10-18
//!
//! `479:28967` — *Terminal / View / Benchmark comparison*, 990x320 — measured
//! off the live file by the desktop lane at the owner's instruction, and
//! recorded in `docs/design/the-terminal-window.md`. Five things in it:
//!
//! | what the design draws | what it is |
//! |---|---|
//! | `Build time · 25% lower in this comparison` | the summary, from the numbers |
//! | `bench-014 + bench-015 · Saved at 13:04 · Illustrative data` | **the provenance, and it is not optional** |
//! | two rows, each a label, a bar and a value | the comparison |
//! | `Same linear scale · Lower is faster · Inspect values when focused` | the scale note |
//! | three `Ask alo / Action` | not built — they are the agent's road |
//!
//! **The geometry is not here.** A bar's length is its value against a scale,
//! and a scale is a property of the drawing surface, so it lives in `alo-shell`
//! beside the rest of the terminal's layout. This file holds **what the program
//! said**, in microseconds, and nothing about pixels.
//!
//! # Everything here is a claim the program made, and the names say so
//!
//! There is no `milliseconds()`. There is [`ARun::it_said_microseconds`],
//! because a duration scraped out of a stranger's output is evidence of what
//! was printed and of nothing else — the program may have lied, or measured
//! something other than what its label says, or be a program that prints
//! timings without running anything.
//!
//! This is the design's *formatted output is not system evidence* turned into
//! method names, the same shape
//! [`crate::a_test_result_a_program_printed`] already has.
//!
//! # One format, and it is named
//!
//! **`hyperfine`**, because the design draws a *comparison of named runs in
//! milliseconds* and that is the shape of its output. `CLAUDE.md`: *cut scope —
//! one printer that works, not half of all printers.*
//!
//! **This is a choice and it is recorded as one.** `cargo bench`'s
//! `bench: 1,234 ns/iter` is the obvious second, and nothing here is shaped
//! against adding it: a second recogniser would answer the same [`ABenchmark`]
//! and the drawing would not change. What a reader must not conclude is that
//! `hyperfine` is the only benchmark alo OS will ever read.
//!
//! # Recognising nothing is the ordinary case
//!
//! Almost everything a terminal shows is not a benchmark, so [`None`] is the
//! answer nearly always and is not a failure. A refusal that drew attention to
//! itself would make every `ls` look like a fault.

use crate::words;

/// At most this many runs are read out of one screen.
///
/// A bound rather than a budget: the screen is a fixed number of lines, so this
/// cannot be reached by ordinary output, and a comparison of forty runs is not
/// a thing the design draws. It exists so a screen of adversarial text cannot
/// make this allocate without limit.
const AT_MOST_THIS_MANY_RUNS: usize = 16;

/// How many microseconds are in a millisecond, and in a second.
///
/// Named because a bare `1_000` beside a bare `1_000_000` is where a unit
/// mistake lives, and a benchmark that is wrong by a thousand reads perfectly.
const MICROSECONDS_IN_A_MILLISECOND: u64 = 1_000;
/// As above, for seconds.
const MICROSECONDS_IN_A_SECOND: u64 = 1_000_000;

/// One run of the comparison: what it was called and how long it took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ARun {
    /// What the program called it.
    name: String,
    /// How long the program said it took.
    microseconds: u64,
}

impl ARun {
    /// What the program called this run.
    ///
    /// **Not this repository's words.** A caller drawing this is drawing a
    /// stranger's label, which is why it is never a key.
    #[must_use]
    pub fn it_said_called(&self) -> &str {
        &self.name
    }

    /// How long the program said this run took, in microseconds.
    ///
    /// Microseconds rather than a float: `hyperfine` prints one decimal of
    /// milliseconds, so a thousandth of a millisecond is exact in an integer
    /// and the arithmetic that follows has no rounding in it at all.
    #[must_use]
    pub const fn it_said_microseconds(&self) -> u64 {
        self.microseconds
    }
}

/// A benchmark comparison, as a program printed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ABenchmark {
    /// The runs, in the order the program printed them.
    runs: Vec<ARun>,
}

impl ABenchmark {
    /// The runs, in the order the program printed them.
    ///
    /// **Order is the program's**, not sorted here: the design draws
    /// `Previous` above `Current`, which is the order they were printed and not
    /// an order by duration.
    #[must_use]
    pub fn it_said_runs(&self) -> &[ARun] {
        &self.runs
    }

    /// The longest run, which is what a shared scale has to fit.
    ///
    /// `None` only for a benchmark with no runs, which this module never
    /// builds — [`a_benchmark_in`] answers [`None`] rather than an empty one.
    #[must_use]
    pub fn the_longest(&self) -> Option<&ARun> {
        self.runs.iter().max_by_key(|run| run.microseconds)
    }

    /// What printed it, for the provenance line.
    #[must_use]
    pub const fn what_printed_it(&self) -> alo_strings::Word {
        words::FROM_A_HYPERFINE_RUN
    }

    /// That nobody checked it, which no shape of this type can omit.
    ///
    /// Takes no argument for the same reason
    /// [`crate::a_test_result_a_program_printed::ATestResult::and_nobody_checked`]
    /// does: a disclaimer a caller can forget is a disclaimer that will be
    /// forgotten.
    #[must_use]
    pub const fn and_nobody_checked(&self) -> alo_strings::Word {
        words::NOT_INDEPENDENTLY_VERIFIED
    }
}

/// Whether this line is safe to read a benchmark out of.
///
/// The same rule [`crate::a_test_result_a_program_printed`] uses, and for the
/// same reason: the question is not *would this move a cursor* but *is this
/// plain text*.
fn it_is_plain_text(line: &str) -> bool {
    !line.chars().any(char::is_control)
}

/// A duration in microseconds, from a number and the unit beside it.
///
/// `hyperfine` prints seconds for slow commands and milliseconds for fast ones,
/// so both are read. **A unit this does not know is a refusal** rather than an
/// assumption: guessing milliseconds for an unknown unit is how a benchmark
/// ends up wrong by a factor of a thousand while looking entirely reasonable.
fn microseconds_from(number: &str, unit: &str) -> Option<u64> {
    let scale = match unit {
        "ms" => MICROSECONDS_IN_A_MILLISECOND,
        "s" => MICROSECONDS_IN_A_SECOND,
        _ => return None,
    };
    let (whole, fraction) = match number.split_once('.') {
        Some((whole, fraction)) => (whole, fraction),
        None => (number, ""),
    };
    let whole: u64 = whole.parse().ok()?;
    // **The fraction is scaled by its own length**, so `.5` and `.50` are the
    // same duration. Reading `5` as five of the next unit down would make one
    // decimal place ten times too small.
    let fraction_value: u64 = if fraction.is_empty() {
        0
    } else {
        let digits: u64 = fraction.parse().ok()?;
        let places = u32::try_from(fraction.len()).ok()?;
        let divisor = 10_u64.checked_pow(places)?;
        digits.checked_mul(scale)? / divisor
    };
    whole.checked_mul(scale)?.checked_add(fraction_value)
}

/// The name on a `Benchmark N: <name>` line, where there is one.
fn the_name_on(line: &str) -> Option<&str> {
    let after = line.trim().strip_prefix("Benchmark ")?;
    let (_number, name) = after.split_once(": ")?;
    let name = name.trim();
    (!name.is_empty()).then_some(name)
}

/// The first duration on a `Time (mean …)` line, where there is one.
///
/// Takes the **first** number and unit after the colon, which is the mean.
/// What follows it is the deviation, and showing a deviation as a duration
/// would draw a bar for the wrong quantity.
fn the_mean_on(line: &str) -> Option<u64> {
    let trimmed = line.trim();
    if !trimmed.starts_with("Time (mean") {
        return None;
    }
    let after = trimmed.split_once(':')?.1;
    let mut tokens = after.split_whitespace();
    let number = tokens.next()?;
    let unit = tokens.next()?;
    microseconds_from(number, unit)
}

/// A benchmark comparison recognised in these lines, or [`None`].
///
/// [`None`] for almost all output a terminal ever shows, and that is the
/// ordinary case rather than a failure — see this module's header.
///
/// # What it reads
///
/// `hyperfine`'s output: a `Benchmark N: <name>` line, and the
/// `Time (mean ± σ):` line that follows it. A name with no time after it, or a
/// time with no name before it, contributes nothing.
///
/// # What makes it answer [`None`]
///
/// Fewer than two runs. **A comparison needs two things to compare**, and a
/// single bar on a shared scale is a chart that says nothing the number did not
/// already say.
#[must_use]
pub fn a_benchmark_in(lines: &[String]) -> Option<ABenchmark> {
    let mut runs: Vec<ARun> = Vec::new();
    let mut waiting_for_a_time: Option<String> = None;

    for line in lines {
        if runs.len() >= AT_MOST_THIS_MANY_RUNS {
            break;
        }
        if !it_is_plain_text(line) {
            // **A control character abandons the pending name**, rather than
            // letting a name from before an escape sequence pair with a time
            // from after it.
            waiting_for_a_time = None;
            continue;
        }
        if let Some(name) = the_name_on(line) {
            waiting_for_a_time = Some(name.to_owned());
            continue;
        }
        if let Some(microseconds) = the_mean_on(line)
            && let Some(name) = waiting_for_a_time.take()
        {
            runs.push(ARun { name, microseconds });
        }
    }

    (runs.len() >= 2).then_some(ABenchmark { runs })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
#[path = "a_benchmark_a_program_printed_tests.rs"]
mod tests;
