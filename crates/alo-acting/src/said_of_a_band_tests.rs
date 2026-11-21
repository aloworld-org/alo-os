//! Each line, said one way and read back.

use super::*;

/// The terminal's band, where the window lane's shell would draw it.
fn the_terminals_band() -> OnTheScreen {
    OnTheScreen {
        x: 100,
        y: 46,
        w: 640,
        h: 54,
    }
}

/// **A band drawn is said with its kind, its id and where**, and read back.
#[test]
fn a_band_drawn_is_said_and_read_back() {
    let line = a_band_is_drawn(TheKind::Terminal, "terminal", the_terminals_band());
    assert_eq!(
        line,
        "alo-shell: a band is drawn: terminal \"terminal\" at 100,46 640x54"
    );
    // On the journal, behind the process's own name.
    let on_the_journal = format!("[  12.5] alo-desktop[2354]: {line}");
    assert_eq!(
        read_a_band_is_drawn(&on_the_journal),
        Some((
            TheKind::Terminal,
            "terminal".to_owned(),
            the_terminals_band()
        ))
    );
}

/// **A band no longer drawn is said, and read back** — so a band that falls
/// off the screen is a line to fail on.
#[test]
fn a_band_no_longer_drawn_is_said_and_read_back() {
    let line = a_band_is_no_longer_drawn(TheKind::Application, "org.gnome.TextEditor");
    assert_eq!(
        line,
        "alo-shell: a band is no longer drawn: application \"org.gnome.TextEditor\""
    );
    assert_eq!(
        read_a_band_is_no_longer_drawn(&line),
        Some((TheKind::Application, "org.gnome.TextEditor".to_owned()))
    );
}

/// **A band's controls are said by their English source words and where each
/// is**, only the ones that kind has, and read back.
#[test]
fn a_bands_controls_are_said_and_read_back() {
    let at = |x: i32| OnTheScreen {
        x,
        y: 51,
        w: 44,
        h: 44,
    };
    let controls = [
        (WINDOW_ACTIONS, at(580)),
        (MINIMISE, at(628)),
        (CLOSE, at(676)),
    ];
    let line = a_bands_controls_are_drawn(TheKind::Settings, "settings", &controls);
    assert_eq!(
        line,
        "alo-shell: a band's controls are drawn: settings \"settings\": \
         \"Window actions\" at 580,51 44x44, \"Minimise\" at 628,51 44x44, \
         \"Close\" at 676,51 44x44"
    );
    let Some((kind, id, read)) = read_a_bands_controls_are_drawn(&line) else {
        unreachable!("the line reads back: {line}");
    };
    assert_eq!((kind, id.as_str()), (TheKind::Settings, "settings"));
    assert_eq!(
        read,
        controls
            .iter()
            .map(|(name, at)| ((*name).to_owned(), *at))
            .collect::<Vec<_>>()
    );
}

/// **The four results are said one way each**, the press with who pressed.
#[test]
fn the_results_are_said_one_way_each() {
    assert_eq!(
        a_bands_control_was_pressed(TheKind::Task, "task 1", CLOSE, PressedBy::Pointer),
        "alo-shell: a band's control was pressed: task \"task 1\": \"Close\" by pointer"
    );
    assert_eq!(
        the_window_panel_opened_for(TheKind::Record, "record"),
        "alo-shell: the window panel opened for record \"record\""
    );
    assert_eq!(
        a_window_was_put_aside(TheKind::Terminal, "terminal"),
        "alo-shell: a window was put aside: terminal \"terminal\""
    );
    assert_eq!(
        a_window_was_closed(TheKind::Application, "org.kde.dolphin"),
        "alo-shell: a window was closed: application \"org.kde.dolphin\""
    );
}

/// **What is not the contract is not read as it**: a line of another kind, a
/// kind nobody declared, or a size that is not a size.
#[test]
fn what_is_not_the_contract_is_not_read() {
    assert_eq!(
        read_a_band_is_drawn("alo-shell: a band is drawn: window \"x\" at 1,2 3x4"),
        None
    );
    assert_eq!(
        read_a_band_is_drawn("alo-shell: a band is drawn: terminal \"terminal\" at 1,2 3"),
        None
    );
    assert_eq!(read_a_band_is_drawn("something else entirely"), None);
    assert_eq!(
        read_a_bands_controls_are_drawn(
            "alo-shell: a band's controls are drawn: terminal \"terminal\": nonsense"
        ),
        None
    );
}
