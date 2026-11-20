use super::*;

fn a_game() -> Application {
    Application::identified("net.supertuxkart.SuperTuxKart").unwrap()
}

/// **What is written is what is read**, at both corners and the middle.
#[test]
fn a_click_reads_back_as_it_was_written() {
    for (across, down) in [(0, 0), (500, 400), (1000, 1000)] {
        let asked = ToClick::at(a_game(), across, down).unwrap();
        assert_eq!(
            asked.written(),
            format!("click-at net.supertuxkart.SuperTuxKart {across} {down}")
        );
        assert_eq!(ToClick::read(&asked.written()), Some(Ok(asked.clone())));
        assert_eq!(
            ToClick::read(&format!("{}\n", asked.written())),
            Some(Ok(asked))
        );
    }
}

/// **Never a point outside the window**: past the whole width or height is not
/// a click, made or read.
#[test]
fn a_point_outside_the_window_is_no_click() {
    assert_eq!(ToClick::at(a_game(), 1001, 0), None);
    assert_eq!(ToClick::at(a_game(), 0, 1001), None);
    for line in [
        "click-at net.supertuxkart.SuperTuxKart 1001 5",
        "click-at net.supertuxkart.SuperTuxKart 5 1001",
        "click-at net.supertuxkart.SuperTuxKart 65536 5",
    ] {
        assert_eq!(
            ToClick::read(line),
            Some(Err(NotToClick::NotOnePoint)),
            "{line}"
        );
    }
}

/// **One application, one point, nothing else**: no second window, no drag,
/// no pixels by another spelling.
#[test]
fn anything_but_one_point_is_refused() {
    for line in [
        "click-at",
        "click-at net.supertuxkart.SuperTuxKart",
        "click-at net.supertuxkart.SuperTuxKart 5",
        "click-at net.supertuxkart.SuperTuxKart 5 5 5",
        "click-at net.supertuxkart.SuperTuxKart 5 5 org.mozilla.firefox",
        "click-at net.supertuxkart.SuperTuxKart -5 5",
        "click-at net.supertuxkart.SuperTuxKart +5 5",
        "click-at net.supertuxkart.SuperTuxKart 05 5",
        "click-at net.supertuxkart.SuperTuxKart 0.5 0.5",
        "click-at net.supertuxkart.SuperTuxKart  5 5",
        "click-at not/an/identifier 5 5",
    ] {
        assert_eq!(
            ToClick::read(line),
            Some(Err(NotToClick::NotOnePoint)),
            "{line}"
        );
    }
    let long = format!("click-at {} 5 5", "a".repeat(Request::THE_MOST));
    assert_eq!(ToClick::read(&long), Some(Err(NotToClick::TooLong)));
}

/// **A line about anything else is not this door's to read.**
#[test]
fn another_act_is_not_a_click() {
    for line in [
        "picture-of net.supertuxkart.SuperTuxKart",
        "clicked",
        "click-at-all 1 1",
    ] {
        assert_eq!(ToClick::read(line), None, "{line}");
    }
}

/// **Every answer reads back as itself, and nothing else is an answer.**
#[test]
fn every_answer_reads_back_and_nothing_else_does() {
    for answer in [
        Clicked::Clicked,
        Clicked::NotInFront,
        Clicked::NotAlosCopy,
        Clicked::ThePersonIsPressing,
        Clicked::CouldNotClick,
    ] {
        assert_eq!(Clicked::read(answer.written()), Some(answer));
        assert_eq!(
            Clicked::read(&format!("{}\n", answer.written())),
            Some(answer)
        );
    }
    for not in ["", "Clicked", "clicked twice", "typed", "acted"] {
        assert_eq!(Clicked::read(not), None, "{not}");
    }
}

/// **A line exactly as long as the door allows is read, not refused as too
/// long**: the bound is the longest, not one short of it.
#[test]
fn a_line_at_the_longest_is_not_too_long() {
    let filler = "a".repeat(Request::THE_MOST - "click-at  5 5".len());
    let line = format!("click-at {filler} 5 5");
    assert_eq!(line.len(), Request::THE_MOST);
    assert_ne!(ToClick::read(&line), Some(Err(NotToClick::TooLong)));
    assert_eq!(
        ToClick::read(&format!("click-at a{filler} 5 5")),
        Some(Err(NotToClick::TooLong))
    );
}

/// **The log says which application, where, and what the desktop answered.**
#[test]
fn the_log_names_the_application_the_point_and_the_answer() {
    let said = crate::said_on_the_log::alo_clicked(
        &ToClick::at(a_game(), 500, 400).unwrap(),
        Clicked::ThePersonIsPressing,
    );
    assert_eq!(
        said,
        "alo asked for a click in the window in front of net.supertuxkart.SuperTuxKart at 500 \
         400, and it was answered the-person-is-pressing"
    );
}
