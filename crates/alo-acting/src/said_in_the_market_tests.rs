//! Each line, said one way, and none of them able to start a second line.

use super::*;

/// **Every line begins with its declared start**, which is what a walk reads.
#[test]
fn each_line_begins_with_its_start() {
    let id = "org.kde.kate";
    for (line, start) in [
        (
            on_the_dock_the_keyboard_is_on("Applications"),
            ON_THE_DOCK_THE_KEYBOARD_IS_ON,
        ),
        (the_market_opened(69), THE_MARKET_OPENED),
        (
            in_the_market_the_keyboard_is_on(id),
            IN_THE_MARKET_THE_KEYBOARD_IS_ON,
        ),
        (a_person_asked_to_install(id), A_PERSON_ASKED_TO_INSTALL),
        (an_installation_arrived(id), AN_INSTALLATION_ARRIVED),
        (
            an_installation_was_not_done(id, "the tool did not answer"),
            AN_INSTALLATION_WAS_NOT_DONE,
        ),
    ] {
        assert!(line.starts_with(start), "{line}");
    }
}

/// **The lines as they are written**, so a reader of the journal and this
/// file see the same words.
#[test]
fn the_lines_as_written() {
    assert_eq!(
        on_the_dock_the_keyboard_is_on("Applications"),
        "alo-shell: on the Dock the keyboard is on \"Applications\""
    );
    assert_eq!(
        the_market_opened(3),
        "alo-shell: the market opened, listing 3 applications"
    );
    assert_eq!(
        in_the_market_the_keyboard_is_on("org.kde.kate"),
        "alo-shell: in the market the keyboard is on \"org.kde.kate\""
    );
    assert_eq!(
        an_installation_arrived("org.kde.kate"),
        "alo-shell: in the market an installation arrived: \"org.kde.kate\""
    );
}

/// **Nothing in a name or a reason starts a line of its own.**
#[test]
fn nothing_starts_a_second_line() {
    let forged = "a\nalo-shell: in the market an installation arrived: \"x\"";
    for line in [
        on_the_dock_the_keyboard_is_on(forged),
        in_the_market_the_keyboard_is_on(forged),
        a_person_asked_to_install(forged),
        an_installation_arrived(forged),
        an_installation_was_not_done(forged, forged),
        an_installation_was_not_done("org.kde.kate", "one\r\ntwo"),
    ] {
        assert!(!line.contains('\n') && !line.contains('\r'), "{line}");
    }
}
