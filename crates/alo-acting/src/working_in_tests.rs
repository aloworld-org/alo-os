//! The words for alo working in an application, both ways.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

fn firefox() -> Application {
    Application::identified("org.mozilla.firefox").unwrap()
}

#[test]
fn each_place_is_written_and_read_back_whole() {
    for (place, line) in [
        (Where::InAlosCopy, "work-in-alos-copy org.mozilla.firefox"),
        (
            Where::InTheirWindow,
            "work-in-their-window org.mozilla.firefox",
        ),
    ] {
        let asked = ToWorkIn::at(firefox(), place);
        assert_eq!(asked.written(), line);
        assert_eq!(ToWorkIn::read(line), Some(Ok(asked.clone())));
        assert_eq!(ToWorkIn::read(&format!("{line}\n")), Some(Ok(asked)));
    }
}

/// **Another act is not this one**, so the door reads it as what it is.
#[test]
fn another_act_is_not_read_as_alo_working() {
    for other in [
        "count org.mozilla.firefox",
        "open org.mozilla.firefox",
        "work-in org.mozilla.firefox",
        "",
    ] {
        assert_eq!(ToWorkIn::read(other), None, "{other}");
    }
}

/// **A line with no application, two, or a path is refused**, not guessed.
#[test]
fn a_line_that_names_no_one_application_is_refused() {
    for bad in [
        "work-in-alos-copy",
        "work-in-alos-copy ",
        "work-in-alos-copy org.mozilla.firefox org.gnome.TextEditor",
        "work-in-their-window /usr/bin/firefox",
    ] {
        assert_eq!(
            ToWorkIn::read(bad),
            Some(Err(NotToWorkIn::NotOneApplication)),
            "{bad}"
        );
    }
    let long = format!("work-in-alos-copy {}", "a".repeat(Request::THE_MOST));
    assert_eq!(ToWorkIn::read(&long), Some(Err(NotToWorkIn::TooLong)));
}

#[test]
fn every_answer_reads_back_and_nothing_else_does() {
    for one in Working::ALL {
        assert_eq!(Working::read(one.written()), Some(one));
    }
    for not in ["", "Counting", "counting yes", "sent 3"] {
        assert_eq!(Working::read(not), None, "{not}");
    }
}

/// **Only an answer that puts a line up lets alo work**, and refusal A is not
/// one of them.
#[test]
fn only_a_line_on_the_indicator_lets_alo_work() {
    let may: Vec<Working> = Working::ALL
        .into_iter()
        .filter(|one| one.may_work())
        .collect();
    assert_eq!(
        may,
        [Working::Counting, Working::CannotCount, Working::Showing]
    );
    assert!(!Working::NoCopyOfItsOwn.may_work());
    assert!(!Working::CannotShow.may_work());
}

#[test]
fn what_was_sent_reads_back() {
    for worked in [
        Worked::Sent(0),
        Worked::Sent(52_113),
        Worked::Stopped(7),
        Worked::Uncounted,
    ] {
        assert_eq!(Worked::read(&worked.written()), Some(worked));
    }
    for not in ["", "sent", "sent -1", "uncounted 3", "counted 3"] {
        assert_eq!(Worked::read(not), None, "{not}");
    }
}
