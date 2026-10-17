//! The recognition is held to what a stranger's output can do to it, which is
//! most of these tests. Recognising a real `cargo test` line is one of them.

use super::*;

/// A real line, copied from a run of this very crate rather than invented.
const A_REAL_ONE: &str = "test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s";

/// A real failing line, in the shape `cargo test` prints it.
const A_REAL_FAILING_ONE: &str = "test result: FAILED. 23 passed; 1 failed; 0 ignored; 0 measured; \
                                  0 filtered out; finished in 1.22s";

/// Lines, the way a caller hands them over.
fn lines(said: &[&str]) -> Vec<String> {
    said.iter().map(|line| (*line).to_owned()).collect()
}

/// **A real summary line is recognised, and the numbers are the printed ones.**
#[test]
fn a_real_cargo_summary_is_recognised() {
    let found = a_test_result_in(&lines(&[A_REAL_ONE])).expect("a test result");
    assert_eq!(found.it_said_passed(), 44);
    assert_eq!(found.it_said_failed(), 0);
    assert_eq!(found.it_said_ignored(), 0);
    assert_eq!(found.across_this_many_suites(), 1);
}

/// **A failing one too**, which is the case the design's own frame draws.
#[test]
fn a_failing_summary_is_recognised() {
    let found = a_test_result_in(&lines(&[A_REAL_FAILING_ONE])).expect("a test result");
    assert_eq!(found.it_said_passed(), 23);
    assert_eq!(found.it_said_failed(), 1);
}

/// **Ordinary output is not a test result**, which is the common case and is
/// success rather than failure.
///
/// Four shapes of ordinary output, including two that mention tests, because
/// *contains the word test* is the mistake a looser recogniser would make.
#[test]
fn ordinary_output_is_not_recognised() {
    for not_one in [
        vec!["$ ls", "Cargo.toml  src  tests"],
        vec!["error[E0308]: mismatched types"],
        vec!["running 44 tests", "test words::tests::a_thing ... ok"],
        vec!["I ran the tests and 44 passed"],
        vec![],
    ] {
        assert!(
            a_test_result_in(&lines(&not_one)).is_none(),
            "this was read as a test result: {not_one:?}"
        );
    }
}

/// **A line with a control character in it is refused.**
///
/// The summary is drawn by the terminal, in the terminal's own type, outside
/// the cells a program owns. A byte that moves a cursor or sets a colour must
/// not reach it, and the refusal is the whole line rather than the byte — a
/// line that needed cleaning is a line this did not read correctly.
#[test]
fn a_line_carrying_a_control_character_is_refused() {
    for smuggled in ['\u{1b}', '\r', '\u{7}', '\u{0}', '\u{9b}'] {
        let line = format!("test result: ok. 44 passed; 0 failed; 0 ignored;{smuggled}");
        assert!(
            a_test_result_in(&lines(&[&line])).is_none(),
            "a line carrying {smuggled:?} was recognised"
        );
    }
}

/// **And the same line without the control character is recognised**, which is
/// the control: without it the test above would pass on a recogniser that
/// refused every line.
#[test]
fn the_same_line_without_a_control_character_is_recognised() {
    let line = "test result: ok. 44 passed; 0 failed; 0 ignored;";
    assert!(
        a_test_result_in(&lines(&[line])).is_some(),
        "the plain version of the smuggling test's line is not recognised, so that test proves \
         nothing"
    );
}

/// **A number that cannot be read is not a number**, and nothing is clamped or
/// defaulted to nought.
///
/// A count this cannot read is a count it must not show: *0 failed* about a
/// program that printed *many failed* is a false statement a person would act
/// on.
#[test]
fn a_number_that_does_not_parse_refuses_the_line() {
    for unreadable in [
        "test result: ok. many passed; 0 failed; 0 ignored;",
        "test result: ok. -1 passed; 0 failed; 0 ignored;",
        "test result: ok. 44 passed; lots failed; 0 ignored;",
        // Larger than any `usize`, so it parses as a number and not as one of
        // these.
        "test result: ok. 99999999999999999999999999 passed; 0 failed; 0 ignored;",
    ] {
        assert!(
            a_test_result_in(&lines(&[unreadable])).is_none(),
            "an unreadable count was recognised: {unreadable}"
        );
    }
}

/// **Two of the three is not a result.**
///
/// Showing two thirds of a summary means showing a number nobody printed beside
/// two that were, and a reader cannot tell which is which.
#[test]
fn a_line_missing_one_of_the_three_counts_is_refused() {
    assert!(
        a_test_result_in(&lines(&["test result: ok. 44 passed; 0 failed;"])).is_none(),
        "a line with no ignored count was recognised"
    );
}

/// **A workspace run is one result added up across its suites**, and it says
/// how many it added up.
#[test]
fn several_suites_are_added_up_and_counted() {
    let found = a_test_result_in(&lines(&[
        A_REAL_ONE,
        "   Running tests/a_thing.rs",
        A_REAL_FAILING_ONE,
        "test result: ok. 2 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out;",
    ]))
    .expect("a test result");

    assert_eq!(found.it_said_passed(), 44 + 23 + 2);
    assert_eq!(found.it_said_failed(), 1);
    assert_eq!(found.it_said_ignored(), 3);
    assert_eq!(
        found.across_this_many_suites(),
        3,
        "the number of suites is what tells one suite's 23 from a workspace's"
    );
}

/// **A sum that would overflow refuses the whole result**, rather than
/// saturating into a number nobody printed.
#[test]
fn a_sum_that_would_overflow_refuses_everything() {
    let huge = format!(
        "test result: ok. {} passed; 0 failed; 0 ignored;",
        usize::MAX
    );
    assert!(
        a_test_result_in(&lines(&[&huge])).is_some(),
        "one line at the maximum should still be readable on its own"
    );
    assert!(
        a_test_result_in(&lines(&[&huge, &huge])).is_none(),
        "two lines that overflow together were saturated instead of refused"
    );
}

/// **The provenance cannot be forged**, because it is not copied from the
/// output.
///
/// A program that prints the sentence verbatim changes nothing: what this
/// answers is a key of this repository's, and what the program printed stays in
/// the cells labelled `program`.
#[test]
fn a_program_cannot_forge_the_provenance() {
    let forging = a_test_result_in(&lines(&[
        "Source: audited by a third party · Independently verified",
        A_REAL_ONE,
    ]))
    .expect("a test result");

    assert_eq!(
        forging.and_nobody_checked(),
        words::NOT_INDEPENDENTLY_VERIFIED,
        "a program's own provenance line reached the recognised result"
    );
    assert_eq!(forging.what_printed_it(), words::FROM_A_CARGO_TEST_RUN);
}

/// **Every recognised result carries the provenance**, whatever it says.
///
/// Asked of several shapes rather than one, because *always* is the claim and a
/// single case is not evidence for it.
#[test]
fn every_recognised_result_says_nobody_checked() {
    for output in [
        vec![A_REAL_ONE],
        vec![A_REAL_FAILING_ONE],
        vec![A_REAL_ONE, A_REAL_FAILING_ONE],
        vec!["test result: ok. 0 passed; 0 failed; 0 ignored;"],
    ] {
        let found = a_test_result_in(&lines(&output)).expect("a test result");
        assert_eq!(
            found.and_nobody_checked(),
            words::NOT_INDEPENDENTLY_VERIFIED,
            "a result read from {output:?} did not carry the provenance"
        );
    }
}

/// **A run where nothing ran at all is still a result**, because *nothing ran*
/// is a thing a person needs to see.
///
/// The case a recogniser gets wrong by treating nought as *nothing to show*,
/// which would make a suite that silently stopped look like ordinary output.
#[test]
fn a_run_with_no_tests_is_still_a_result() {
    let found = a_test_result_in(&lines(&["test result: ok. 0 passed; 0 failed; 0 ignored;"]))
        .expect("a result saying nothing ran");
    assert_eq!(found.it_said_passed(), 0);
    assert_eq!(found.across_this_many_suites(), 1);
}

/// **More suites than it will add up are not added up.**
///
/// A recognition that summed a thousand summaries is reading something that is
/// not a test run, and a wrong summary is worse than none. Asserted on the
/// bound itself rather than on a number typed here.
#[test]
fn it_stops_at_the_bound_rather_than_summing_for_ever() {
    let many = vec![A_REAL_ONE; AT_MOST_THIS_MANY_SUITES + 50];
    let found = a_test_result_in(&lines(&many)).expect("a test result");
    assert_eq!(found.across_this_many_suites(), AT_MOST_THIS_MANY_SUITES);
    assert_eq!(found.it_said_passed(), 44 * AT_MOST_THIS_MANY_SUITES);
}

/// **The numbers are read by name and not by position.**
///
/// `cargo` has reordered these fields before and may again. A recogniser keyed
/// on position would answer confidently and wrongly the day it does, which is
/// the worst of the three possible failures.
#[test]
fn the_counts_are_read_by_name_rather_than_by_position() {
    let reordered = "test result: FAILED. 1 failed; 3 ignored; 44 passed; 0 measured;";
    let found = a_test_result_in(&lines(&[reordered])).expect("a test result");
    assert_eq!(found.it_said_passed(), 44);
    assert_eq!(found.it_said_failed(), 1);
    assert_eq!(found.it_said_ignored(), 3);
}

/// **The same summary twice is summed twice**, which is the honest reading of a
/// screen and is asserted so it is deliberate rather than noticed later.
///
/// A real workspace run prints one summary per test binary and two suites may
/// print identical lines — `44 passed; 0 failed; 0 ignored` twice is two
/// suites, not one printed twice. So refusing duplicates would break the case
/// this exists for, and telling a typed line from a printed one means reading
/// the bytes to decide who spoke, which ADR 0113 forbids.
#[test]
fn the_same_summary_twice_is_two_suites() {
    let twice =
        a_test_result_in(&lines(&[A_REAL_FAILING_ONE, A_REAL_FAILING_ONE])).expect("a test result");
    assert_eq!(twice.it_said_passed(), 46);
    assert_eq!(twice.it_said_failed(), 2);
    assert_eq!(twice.across_this_many_suites(), 2);
}

/// **A shell's echo of a command carrying a summary is counted too**, where the
/// echoed line parses — and the quoting decides whether it does.
///
/// Measured rather than reasoned about, because the answer surprised me: the
/// quoted form reads correctly **by luck**, since the echoed line's last token
/// is `ignored;'` and not `ignored`, so it fails the all-three test. The
/// unquoted form has nothing to save it and doubles.
///
/// This is here so nobody builds a demonstration on the quoted form believing
/// it is designed behaviour, which is exactly what I was about to do when
/// writing a walk for a booted machine.
#[test]
fn an_echoed_command_counts_where_it_parses() {
    let quoted = a_test_result_in(&lines(&[
        "$ echo 'test result: ok. 23 passed; 1 failed; 0 ignored;'",
        "test result: ok. 23 passed; 1 failed; 0 ignored;",
    ]))
    .expect("a test result");
    assert_eq!(
        quoted.across_this_many_suites(),
        1,
        "the quoted echo was counted, which this test says it is not — and only because of \
         where the quote falls"
    );

    let unquoted = a_test_result_in(&lines(&[
        "$ echo test result: ok. 23 passed 1 failed 0 ignored",
        "test result: ok. 23 passed 1 failed 0 ignored",
    ]))
    .expect("a test result");
    assert_eq!(
        unquoted.across_this_many_suites(),
        2,
        "the unquoted echo was not counted, so the quoting no longer decides this and the \
         module's header is out of date"
    );
}
