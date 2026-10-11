//! Where a forwarded journal goes, and the two ways a reader of it is wrong.

use super::TheJournalSettings;

/// **The drop-in this repository ships names the serial line**, asked of the
/// real file rather than of a fixture.
///
/// A check that only ever meets a fixture is a check about a fixture, and the
/// karg this replaced passed every fixture it had.
#[test]
fn the_drop_in_this_repository_ships_names_the_port() {
    let at = "../../image/usr/lib/systemd/journald.conf.d/20-the-journal-goes-to-the-reader.conf";
    let Ok(text) = std::fs::read_to_string(at) else {
        unreachable!("the shipped journald drop-in is in this repository, at {at}")
    };
    let read = TheJournalSettings::read(&text);

    assert_eq!(read.where_a_forwarded_journal_goes(), Some("/dev/ttyS0"));
    assert!(read.goes_to_one_of(&["ttyS0"]));
}

/// **The stock file's commented default is not a setting.**
///
/// `/usr/lib/systemd/journald.conf` ships `#TTYPath=/dev/console`. A reader
/// that took that for a setting would report every image carrying the stock
/// file as having configured the screen — which is the right answer by
/// accident and for the wrong reason, and would stop being right the day
/// upstream changed its default.
#[test]
fn a_commented_default_is_not_a_setting() {
    for commented in [
        "#TTYPath=/dev/console",
        "# TTYPath=/dev/console",
        "   #TTYPath=/dev/ttyS0",
        ";TTYPath=/dev/ttyS0",
    ] {
        let read = TheJournalSettings::read(&format!("[Journal]\n{commented}\n"));

        assert_eq!(
            read.where_a_forwarded_journal_goes(),
            None,
            "{commented} was read as a setting"
        );
    }
}

/// **Nothing said is the screen, not unknown**, which is the whole reason
/// this file exists: journald's default is `/dev/console`, and `console.toml`
/// makes that the screen on purpose.
#[test]
fn nothing_said_does_not_go_to_a_serial_line() {
    for text in ["", "[Journal]\n", "[Journal]\nStorage=persistent\n"] {
        let read = TheJournalSettings::read(text);

        assert_eq!(read.where_a_forwarded_journal_goes(), None);
        assert!(
            !read.goes_to_one_of(&["ttyS0", "ttyUSB0"]),
            "an unset path was accepted as going to a serial line"
        );
    }
}

/// **A path that is not one of this image's lines is refused**, including the
/// default that caused the fault.
#[test]
fn a_path_that_is_not_one_of_these_lines_is_refused() {
    for path in ["/dev/console", "/dev/tty0", "/dev/tty1", "/dev/ttyS1"] {
        let read = TheJournalSettings::read(&format!("[Journal]\nTTYPath={path}\n"));

        assert_eq!(read.where_a_forwarded_journal_goes(), Some(path));
        assert!(
            !read.goes_to_one_of(&["ttyS0"]),
            "{path} is not a line this image names and was accepted"
        );
    }
}

/// **A port may be written with or without `/dev/`** on either side, because
/// journald wants a path and a `console=` names a port.
#[test]
fn a_port_matches_with_or_without_the_dev_prefix() {
    for path in ["/dev/ttyS0", "ttyS0"] {
        let read = TheJournalSettings::read(&format!("[Journal]\nTTYPath={path}\n"));

        assert!(read.goes_to_one_of(&["ttyS0"]), "{path} was not matched");
    }
}

/// **The last assignment wins**, which is systemd's rule for a key repeated
/// in one file. A reader stopping at the first would report the overridden
/// value, which is the opposite of what the machine will do.
#[test]
fn the_last_assignment_is_the_one_that_counts() {
    let read = TheJournalSettings::read(
        "[Journal]\nTTYPath=/dev/tty0\nStorage=persistent\nTTYPath=/dev/ttyS0\n",
    );

    assert_eq!(read.where_a_forwarded_journal_goes(), Some("/dev/ttyS0"));
    assert!(read.goes_to_one_of(&["ttyS0"]));
}

/// **Whitespace and case are systemd's, not ours.** `TTYPATH = /dev/ttyS0 `
/// is the same setting, and a reader that insisted on one spelling would
/// report a configured machine as unconfigured.
#[test]
fn spacing_and_case_are_read_the_way_systemd_reads_them() {
    for written in [
        "TTYPath=/dev/ttyS0",
        "  TTYPath = /dev/ttyS0  ",
        "ttypath=/dev/ttyS0",
        "TTYPATH=/dev/ttyS0",
    ] {
        let read = TheJournalSettings::read(&format!("[Journal]\n{written}\n"));

        assert!(
            read.goes_to_one_of(&["ttyS0"]),
            "{written} was not read as setting the path"
        );
    }
}

/// **An empty value is nothing said**, not a path called the empty string.
#[test]
fn an_empty_value_says_nothing() {
    let read = TheJournalSettings::read("[Journal]\nTTYPath=\n");

    assert_eq!(read.where_a_forwarded_journal_goes(), None);
}
