//! **alo's own windows go into the right-hand panel and come back** — and, as
//! carefully, what must not happen: a window that is not open going aside, a
//! window aside still taking keys, a square left behind for a window that
//! closed, a window hidden with no panel to bring it back from.
//!
//! The terminal is a real shell on a real pseudoterminal, because *it keeps
//! running while it is aside* is the kernel's answer and a mocked one would
//! test the mock.

use std::os::unix::fs::PermissionsExt as _;
use std::time::Duration;

use alo_put_aside::{AlosOwnWindow, Panel};
use alo_shortcuts::{Action, Shortcuts};
use alo_terminal::IsACommandRunning;
use smithay::input::keyboard::XkbConfig;

use crate::Server;
use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;
use crate::which_window_is_in_front::AKindOfWindow;

/// A session with a keyboard on a US layout and the shipped shortcuts.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        what,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.the_shortcuts_are(Shortcuts::shipped());
    (runtime, server)
}

/// A session with Settings open, in front and taking the keys.
fn with_settings(what: &str) -> (tempfile::TempDir, Server) {
    let machine = crate::settings_testing::a_persons_machine(what);
    let (runtime, mut server) = a_session(what);
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    assert!(server.settings_is_taking_the_keys(), "the control");
    (runtime, server)
}

/// Settings asked aside and the frame that lends the panel run.
fn settings_aside(server: &mut Server, panel: &mut Panel) {
    server.this_alo_window_is_asked_to_go_aside(AlosOwnWindow::Settings);
    assert_eq!(
        server.put_alos_windows_aside_that_were_asked(Some(panel)),
        1
    );
}

/// **Minimised, Settings leaves the canvas and the keys, and stays open.**
#[test]
fn settings_put_aside_leaves_the_canvas_and_the_keys_and_stays_open() {
    let (_runtime, mut server) = with_settings("aside-settings");
    let mut panel = Panel::new();

    settings_aside(&mut server, &mut panel);

    assert!(panel.holds_alos_window(AlosOwnWindow::Settings));
    assert!(server.this_alo_window_is_aside(AlosOwnWindow::Settings));
    assert!(!server.settings_is_open_here(), "it is still on the canvas");
    assert!(server.where_settings_is_on_the_plane().is_none());
    assert!(
        !server.settings_is_taking_the_keys(),
        "a window nobody can see still takes the keys"
    );
    assert!(
        server.the_settings_window().is_open(),
        "minimising closed it"
    );
    assert_ne!(server.the_kind_in_front(), AKindOfWindow::Settings);
}

/// **Pressing its square brings it back**, in front with the keys, and the
/// square leaves the panel.
#[test]
fn bringing_settings_back_puts_it_in_front_with_the_keys() {
    let (_runtime, mut server) = with_settings("back-settings");
    let mut panel = Panel::new();
    settings_aside(&mut server, &mut panel);

    assert!(server.bring_alos_window_back(&mut panel, AlosOwnWindow::Settings));

    assert!(!panel.holds_alos_window(AlosOwnWindow::Settings));
    assert!(!server.this_alo_window_is_aside(AlosOwnWindow::Settings));
    assert!(server.settings_is_open_here());
    assert!(server.settings_is_taking_the_keys());
}

/// **Any road to it brings it back**: the Dock or the chord that opens
/// Settings goes to it, and the square leaves the panel on the next frame.
#[test]
fn going_to_settings_by_another_road_brings_it_back() {
    let (_runtime, mut server) = with_settings("road-settings");
    let mut panel = Panel::new();
    settings_aside(&mut server, &mut panel);

    server.the_person_went_to_settings();
    assert!(server.settings_is_open_here());
    assert_eq!(
        server.the_panel_lets_go_of_alos_windows_back(Some(&mut panel)),
        1
    );
    assert_eq!(panel.holding_everything(), 0, "a square for nothing stayed");
}

/// **The refusal path: a window that is not open is not put aside**, and no
/// square appears for it.
#[test]
fn a_window_that_is_not_open_is_not_put_aside() {
    let (_runtime, mut server) = a_session("aside-nothing-open");
    let mut panel = Panel::new();
    server.this_alo_window_is_asked_to_go_aside(AlosOwnWindow::TheRecord);
    server.this_alo_window_is_asked_to_go_aside(AlosOwnWindow::TheTerminal);

    assert_eq!(
        server.put_alos_windows_aside_that_were_asked(Some(&mut panel)),
        0
    );
    assert_eq!(panel.holding_everything(), 0);
    assert!(!server.this_alo_window_is_aside(AlosOwnWindow::TheRecord));
}

/// **The refusal path: with no panel nothing is hidden**, so a window can
/// never be away with nowhere to bring it back from — and the ask is not
/// kept to be met later.
#[test]
fn with_no_panel_nothing_is_put_aside() {
    let (_runtime, mut server) = with_settings("aside-no-panel");
    server.this_alo_window_is_asked_to_go_aside(AlosOwnWindow::Settings);

    assert_eq!(server.put_alos_windows_aside_that_were_asked(None), 0);
    assert!(server.settings_is_open_here());
    assert!(server.settings_is_taking_the_keys());
    assert_eq!(
        server.put_alos_windows_aside_that_were_asked(Some(&mut Panel::new())),
        0,
        "the ask was kept and met a frame later"
    );
}

/// **Asked twice is one square.**
#[test]
fn asked_twice_is_one_square() {
    let (_runtime, mut server) = with_settings("aside-twice");
    let mut panel = Panel::new();
    server.this_alo_window_is_asked_to_go_aside(AlosOwnWindow::Settings);
    settings_aside(&mut server, &mut panel);
    server.this_alo_window_is_asked_to_go_aside(AlosOwnWindow::Settings);

    assert_eq!(
        server.put_alos_windows_aside_that_were_asked(Some(&mut panel)),
        0
    );
    assert_eq!(panel.holding_everything(), 1);
}

/// **A window closed while aside leaves the panel and the server both**: no
/// square is left standing for a window that is gone.
#[test]
fn the_record_closed_while_aside_leaves_the_panel() {
    let (_runtime, mut server) = a_session("aside-record-closed");
    let kept = crate::record_testing::an_afternoon_kept();
    server.the_record_is_kept(Ok(kept.recounting()));
    let _ = server.the_record_is_opened();
    assert!(server.the_record_is_taking_the_keys(), "the control");
    let mut panel = Panel::new();
    server.this_alo_window_is_asked_to_go_aside(AlosOwnWindow::TheRecord);
    assert_eq!(
        server.put_alos_windows_aside_that_were_asked(Some(&mut panel)),
        1
    );
    assert!(!server.the_record_is_open_here());
    assert!(!server.the_record_is_taking_the_keys());

    server.the_person_closes_the_record();
    assert_eq!(
        server.the_panel_lets_go_of_alos_windows_back(Some(&mut panel)),
        1
    );
    assert_eq!(panel.holding_everything(), 0);
    assert!(!server.this_alo_window_is_aside(AlosOwnWindow::TheRecord));
}

/// **A window the server holds aside that the panel does not is brought back
/// onto the canvas**, never left where nobody can reach it.
#[test]
fn a_window_the_panel_lost_comes_back_onto_the_canvas() {
    let (_runtime, mut server) = with_settings("aside-panel-lost");
    let mut panel = Panel::new();
    settings_aside(&mut server, &mut panel);

    let mut another = Panel::new();
    let _ = server.the_panel_lets_go_of_alos_windows_back(Some(&mut another));

    assert!(!server.this_alo_window_is_aside(AlosOwnWindow::Settings));
    assert!(server.settings_is_open_here());
}

/// **The Minimise chord at Settings puts Settings aside**, not the application
/// behind it.
#[test]
fn the_minimise_chord_puts_the_window_with_the_keys_aside() {
    let (_runtime, mut server) = with_settings("aside-chord");
    let shortcuts = Shortcuts::shipped();
    let minimise = shortcuts
        .chord_for(Action::MinimiseWindow)
        .expect("the release ships a Minimise chord");

    assert!(server.a_chord_puts_alos_own_window_aside(&shortcuts, minimise));
    let mut panel = Panel::new();
    assert_eq!(
        server.put_alos_windows_aside_that_were_asked(Some(&mut panel)),
        1
    );
    assert!(panel.holds_alos_window(AlosOwnWindow::Settings));
}

/// **The refusal path for the chord**: another chord, or Settings behind an
/// application, puts nothing aside and hands the chord on.
#[test]
fn the_minimise_chord_is_not_taken_from_an_application() {
    let (_runtime, mut server) = with_settings("aside-chord-refused");
    let shortcuts = Shortcuts::shipped();
    let settings = shortcuts
        .chord_for(Action::Settings)
        .expect("the release ships a chord for Settings");
    assert!(!server.a_chord_puts_alos_own_window_aside(&shortcuts, settings));

    server.the_person_went_to(AKindOfWindow::Applications);
    let minimise = shortcuts
        .chord_for(Action::MinimiseWindow)
        .expect("the release ships a Minimise chord");
    assert!(
        !server.a_chord_puts_alos_own_window_aside(&shortcuts, minimise),
        "a window without the keyboard took the chord"
    );
    assert_eq!(
        server.put_alos_windows_aside_that_were_asked(Some(&mut Panel::new())),
        0
    );
}

/// Pump the terminal until it reads `wanted`, or ten seconds pass.
fn until_it_reads(server: &mut Server, wanted: IsACommandRunning) -> bool {
    for _ in 0..400 {
        let _ = server.the_terminal_was_written_to();
        if server
            .alos_own_terminal()
            .is_some_and(|terminal| terminal.is_a_command_running() == wanted)
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    false
}

/// **The terminal keeps running while it is aside**: the same shell, not
/// drawn, not taking keys — and back in front with the keys when its square
/// is pressed.
#[test]
fn the_terminal_keeps_running_while_it_is_aside() {
    let (_runtime, mut server) = a_session("aside-terminal");
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    server.the_person_went_to(AKindOfWindow::TheTerminal);
    assert!(
        until_it_reads(&mut server, IsACommandRunning::No),
        "the shell never reached its prompt"
    );
    let shell = server
        .alos_own_terminal()
        .expect("a terminal")
        .the_shells_process();
    let mut panel = Panel::new();

    server.this_alo_window_is_asked_to_go_aside(AlosOwnWindow::TheTerminal);
    assert_eq!(
        server.put_alos_windows_aside_that_were_asked(Some(&mut panel)),
        1
    );
    assert!(!server.the_terminal_is_taking_the_keys());
    assert!(server.where_the_terminal_is_on_the_plane().is_none());
    assert_eq!(
        server
            .alos_own_terminal()
            .expect("aside is not closed")
            .the_shells_process(),
        shell
    );

    assert!(server.bring_alos_window_back(&mut panel, AlosOwnWindow::TheTerminal));
    assert!(server.the_terminal_is_taking_the_keys());
    assert_eq!(panel.holding_everything(), 0);
}
