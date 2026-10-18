//! **The refusals outnumber the happy paths**, which `CLAUDE.md` asks for and
//! which matters here because everything this module reads was printed by a
//! program nobody vetted.

#![expect(
    clippy::indexing_slicing,
    reason = "in a test, indexing past a length the test just asserted is the failure being reported"
)]

use super::*;

/// `hyperfine`'s output for two commands, as it prints it.
fn two_runs() -> Vec<String> {
    [
        "Benchmark 1: ./previous",
        "  Time (mean ± σ):     240.0 ms ±   5.0 ms    [User: 120.0 ms, System: 30.0 ms]",
        "  Range (min … max):   230.0 ms … 250.0 ms    10 runs",
        "",
        "Benchmark 2: ./current",
        "  Time (mean ± σ):     180.0 ms ±   3.0 ms    [User: 90.0 ms, System: 20.0 ms]",
        "  Range (min … max):   175.0 ms … 190.0 ms    10 runs",
        "",
        "Summary",
        "  './current' ran",
        "    1.33 ± 0.03 times faster than './previous'",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect()
}

/// **The two runs the design draws are read, with their names and their
/// times.**
#[test]
fn two_named_runs_are_recognised() {
    let found = a_benchmark_in(&two_runs()).expect("hyperfine's output is a benchmark");
    let runs = found.it_said_runs();

    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].it_said_called(), "./previous");
    assert_eq!(runs[0].it_said_microseconds(), 240_000);
    assert_eq!(runs[1].it_said_called(), "./current");
    assert_eq!(runs[1].it_said_microseconds(), 180_000);
}

/// **The order is the program's and not sorted by duration.**
///
/// The design draws `Previous` above `Current`, which is the order they were
/// printed. Sorting would put the faster one first and silently reorder
/// somebody's comparison.
#[test]
fn the_runs_keep_the_order_the_program_printed_them_in() {
    let found = a_benchmark_in(&two_runs()).expect("a benchmark");
    let names: Vec<_> = found
        .it_said_runs()
        .iter()
        .map(ARun::it_said_called)
        .collect();
    assert_eq!(names, vec!["./previous", "./current"]);
}

/// **The longest is what a shared scale has to fit.**
#[test]
fn the_longest_run_is_the_slowest_one() {
    let found = a_benchmark_in(&two_runs()).expect("a benchmark");
    assert_eq!(
        found
            .the_longest()
            .expect("two runs have a longest")
            .it_said_called(),
        "./previous"
    );
}

/// **Seconds are read as well as milliseconds**, because `hyperfine` prints
/// whichever suits the magnitude.
#[test]
fn a_run_measured_in_seconds_is_read_as_seconds() {
    let lines: Vec<String> = [
        "Benchmark 1: slow",
        "  Time (mean ± σ):      1.5 s ±  0.1 s",
        "Benchmark 2: fast",
        "  Time (mean ± σ):    500.0 ms ±  10.0 ms",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();

    let found = a_benchmark_in(&lines).expect("a benchmark");
    assert_eq!(found.it_said_runs()[0].it_said_microseconds(), 1_500_000);
    assert_eq!(found.it_said_runs()[1].it_said_microseconds(), 500_000);
}

/// **One decimal place is a tenth, not a unit of its own.**
///
/// Reading the `5` of `240.5` as five microseconds would be wrong by a factor
/// of a hundred, and `240.50` must mean the same duration as `240.5`.
#[test]
fn a_fraction_is_scaled_by_how_many_digits_it_has() {
    let one_place: Vec<String> = [
        "Benchmark 1: a",
        "  Time (mean): 240.5 ms",
        "Benchmark 2: b",
        "  Time (mean): 1.0 ms",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();
    let two_places: Vec<String> = [
        "Benchmark 1: a",
        "  Time (mean): 240.50 ms",
        "Benchmark 2: b",
        "  Time (mean): 1.00 ms",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();

    let a = a_benchmark_in(&one_place).expect("a benchmark");
    let b = a_benchmark_in(&two_places).expect("a benchmark");
    assert_eq!(a.it_said_runs()[0].it_said_microseconds(), 240_500);
    assert_eq!(
        a.it_said_runs()[0].it_said_microseconds(),
        b.it_said_runs()[0].it_said_microseconds(),
        "240.5 and 240.50 are the same duration"
    );
}

/// **One run is not a comparison**, so it is not drawn.
///
/// A single bar on a shared scale says nothing the number did not already say,
/// and the design's frame is a comparison of two.
#[test]
fn a_single_run_is_not_a_comparison() {
    let lines: Vec<String> = [
        "Benchmark 1: alone",
        "  Time (mean ± σ):  240.0 ms ± 5.0 ms",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();
    assert_eq!(a_benchmark_in(&lines), None);
}

/// **A unit nobody knows is refused, never assumed.**
///
/// Guessing milliseconds for `240.0 ns` would be wrong by a factor of a
/// million while looking entirely reasonable on a chart.
#[test]
fn a_unit_this_does_not_know_is_refused() {
    let lines: Vec<String> = [
        "Benchmark 1: a",
        "  Time (mean ± σ):  240.0 ns ± 5.0 ns",
        "Benchmark 2: b",
        "  Time (mean ± σ):  180.0 ns ± 3.0 ns",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();
    assert_eq!(
        a_benchmark_in(&lines),
        None,
        "nanoseconds are not read, so there is no comparison"
    );
}

/// **A name with no time after it contributes nothing.**
#[test]
fn a_name_with_no_time_is_dropped() {
    let lines: Vec<String> = [
        "Benchmark 1: announced but never timed",
        "Benchmark 2: a",
        "  Time (mean ± σ):  240.0 ms ± 5.0 ms",
        "Benchmark 3: b",
        "  Time (mean ± σ):  180.0 ms ± 3.0 ms",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();

    let found = a_benchmark_in(&lines).expect("two of the three were timed");
    assert_eq!(found.it_said_runs().len(), 2);
    assert_eq!(found.it_said_runs()[0].it_said_called(), "a");
}

/// **A control character abandons the pending name.**
///
/// Otherwise a name printed before an escape sequence would pair with a time
/// printed after it, which is two unrelated things drawn as one row.
#[test]
fn an_escape_sequence_between_a_name_and_its_time_breaks_the_pair() {
    let lines: Vec<String> = [
        "Benchmark 1: a",
        "\u{1b}[2J",
        "  Time (mean ± σ):  240.0 ms ± 5.0 ms",
        "Benchmark 2: b",
        "  Time (mean ± σ):  180.0 ms ± 3.0 ms",
        "Benchmark 3: c",
        "  Time (mean ± σ):  120.0 ms ± 3.0 ms",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();

    let found = a_benchmark_in(&lines).expect("b and c are a comparison");
    let names: Vec<_> = found
        .it_said_runs()
        .iter()
        .map(ARun::it_said_called)
        .collect();
    assert_eq!(
        names,
        vec!["b", "c"],
        "a's time arrived after a control character"
    );
}

/// **The deviation is not a duration.**
///
/// `240.0 ms ± 5.0 ms` holds two numbers and only the first is the mean.
/// Drawing a bar for the deviation would draw the wrong quantity entirely.
#[test]
fn the_deviation_after_the_mean_is_not_read_as_the_mean() {
    let found = a_benchmark_in(&two_runs()).expect("a benchmark");
    assert_eq!(
        found.it_said_runs()[0].it_said_microseconds(),
        240_000,
        "the mean, not the 5.0 ms deviation beside it"
    );
}

/// **Ordinary output is not a benchmark**, which is nearly everything.
#[test]
fn ordinary_output_is_not_a_benchmark() {
    let lines: Vec<String> = [
        "total 48",
        "drwxr-xr-x  7 someone staff  224 Oct 18 10:11 .",
        "Benchmarking is hard and this line mentions it",
    ]
    .iter()
    .map(|it| (*it).to_owned())
    .collect();
    assert_eq!(a_benchmark_in(&lines), None);
}

/// **A screen of names cannot make this allocate without limit.**
#[test]
fn a_screen_of_benchmarks_stops_at_the_bound() {
    let mut lines = Vec::new();
    for which in 0..100 {
        lines.push(format!("Benchmark {which}: run {which}"));
        lines.push("  Time (mean ± σ):  240.0 ms ± 5.0 ms".to_owned());
    }
    let found = a_benchmark_in(&lines).expect("many runs are still a comparison");
    assert!(
        found.it_said_runs().len() <= AT_MOST_THIS_MANY_RUNS,
        "read {} runs, which is past the bound",
        found.it_said_runs().len()
    );
}

/// **Every benchmark carries its provenance**, and no shape of the type omits
/// it.
#[test]
fn a_benchmark_always_says_where_it_came_from_and_that_nobody_checked() {
    let found = a_benchmark_in(&two_runs()).expect("a benchmark");
    assert_eq!(
        found.what_printed_it().named(),
        "terminal.benchmark.from-hyperfine"
    );
    assert_eq!(
        found.and_nobody_checked().named(),
        "terminal.result.not-verified"
    );
}
