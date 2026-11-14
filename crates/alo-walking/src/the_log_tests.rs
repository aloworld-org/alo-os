//! Finding a phrase in the serial log, before and after a mark.

use super::{all_with, first_with, lines_after};

/// A log with a line before the mark and two after.
const LOG: &[u8] = b"[ 1.0] booted\r\n[ 2.0] Started alo-screen-reader.service - Orca.\r\n[ 3.0] orca: Traceback (most recent call last)\n";

/// Where the mark is: after the first line.
const MARK: usize = 15;

/// A phrase said after the mark is found, as its whole line.
#[test]
fn a_phrase_after_the_mark_is_found_as_its_line() {
    assert_eq!(
        first_with(LOG, MARK, "Started alo-screen-reader.service"),
        Some("[ 2.0] Started alo-screen-reader.service - Orca.".to_owned())
    );
}

/// **A phrase said only before the mark is not found after it** — so an old
/// line can never answer a new question.
#[test]
fn a_phrase_before_the_mark_is_not_found_after_it() {
    assert_eq!(first_with(LOG, MARK, "booted"), None);
    assert!(first_with(LOG, 0, "booted").is_some());
}

/// Every line carrying any of several phrases.
#[test]
fn every_line_with_any_phrase() {
    let found = all_with(
        LOG,
        0,
        &["Traceback".to_owned(), "strange signature".to_owned()],
    );
    assert_eq!(found.len(), 1);
}

/// **A journal line a kernel line landed inside is read whole** — the tear
/// measured on the walk of 2026-11-14, byte for byte but for the audit
/// record's length.
#[test]
fn a_line_torn_by_a_kernel_line_is_mended() {
    let log = b"[  127.851670] alo-shipping[2168]: 8 of this machine's applications arrived, 0 s[  128.234364] audit: type=1130 msg='unit=alo-shipping res=success'\ntill owed\n[  129.000001] next\n";
    let records = lines_after(log, 0);
    assert_eq!(
        records,
        vec![
            "[  127.851670] alo-shipping[2168]: 8 of this machine's applications arrived, 0 still owed"
                .to_owned(),
            "[  128.234364] audit: type=1130 msg='unit=alo-shipping res=success'".to_owned(),
            "[  129.000001] next".to_owned(),
        ]
    );
}

/// A bracket that is not a timestamp does not cut a line, and a line with no
/// timestamp after a whole one is its own.
#[test]
fn only_a_timestamp_cuts_a_line() {
    let log = b"[    1.000000] a [list] of [1.5] things\nplain words\n";
    assert_eq!(
        lines_after(log, 0),
        vec![
            "[    1.000000] a [list] of [1.5] things".to_owned(),
            "plain words".to_owned()
        ]
    );
}

/// **Half a line is not read**: the end of a log still being written is
/// judged only once its line is whole.
#[test]
fn half_a_line_is_not_read() {
    let log = b"[    1.000000] whole\n[    2.000000] 8 arrived, 0 s";
    assert_eq!(first_with(log, 0, "arrived"), None);
    assert!(first_with(log, 0, "whole").is_some());
}

/// A mark past the end reads as nothing said, and a byte that is not text
/// does not hide its line.
#[test]
fn a_mark_past_the_end_and_a_byte_that_is_not_text() {
    assert!(lines_after(LOG, 10_000).is_empty());
    let lines = lines_after(b"a \xff line\n", 0);
    assert_eq!(lines, vec!["a \u{fffd} line".to_owned()]);
}
