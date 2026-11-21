//! The Dock and the market say where their keyboard is, by name, on the road a
//! person walks to install an application — pressed as a person presses.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use smithay::backend::input::KeyState;
use smithay::input::keyboard::XkbConfig;
use std::os::unix::fs::PermissionsExt as _;

use crate::Server;
use crate::tab_moves_among_the_bars_controls::WhichWay;
use crate::which_area_has_the_keyboard::AnArea;

/// Escape, Down and Enter, as evdev numbers them.
const KEY_ESC: u32 = 1;
const KEY_DOWN: u32 = 108;
const KEY_ENTER: u32 = 28;

/// The display the Dock is laid out on.
const EXTENT: (i32, i32) = (1280, 800);

/// The application the walk installs from the market (`crates/alo-walking`).
const KATE: &str = "org.kde.kate";

/// A seat with a US keyboard and a fresh machine's Dock drawn on one display.
fn a_seat_with_its_dock(what: &str) -> (tempfile::TempDir, Server) {
    use alo_dock::pinning::{Discovery, ThePins};
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
    let _owed = server
        .its_dock_is_filled_from_what_they_kept(&ThePins::NeverSet, &Discovery::Completed(vec![]));
    let on_the_dock = server.what_this_persons_dock_holds();
    let places = crate::where_the_dock_is::where_the_dock_is(
        &alo_dock::Dock::shipped(),
        EXTENT,
        &on_the_dock,
        crate::display_scale::DisplayScale::ONE,
    )
    .expect("a dock this display can hold");
    server.the_docks_slots_were_drawn("a-display", places);
    (runtime, server)
}

/// One key pressed and let go at the market.
fn tapped(server: &mut Server, code: u32) {
    server
        .the_market_takes_this_key(code, KeyState::Pressed, 1)
        .expect("a seat with a keyboard");
    server
        .the_market_takes_this_key(code, KeyState::Released, 2)
        .expect("a seat with a keyboard");
}

/// **The whole road, by name**: F6 says the Dock's first slot, Tab says each
/// slot until *Applications* — the word the Dock draws for the market's entry
/// (`alo_dock::words::APPLICATIONS_ON_THE_DOCK`) — Enter opens the market and
/// it says its first card, and Down says each card until Kate. A walk finds
/// both by these names rather than by counting presses.
#[test]
fn the_dock_and_the_market_say_where_the_keyboard_is_by_name() {
    let (_runtime, mut server) = a_seat_with_its_dock("the-road-to-the-market-is-said");
    let applications = alo_dock::words::APPLICATIONS_ON_THE_DOCK.says();

    // F6.
    server.the_keyboard_is_now_in(Some(AnArea::TheDock));
    assert!(
        server.said_on_the_dock.is_some(),
        "the keyboard arrived on the Dock and nothing said which slot"
    );

    // Tab, until the Dock says Applications.
    for _ in 0..24 {
        if server.said_on_the_dock.as_deref() == Some(applications) {
            break;
        }
        assert!(server.tab_moves_among_the_bars_controls(WhichWay::Onward));
    }
    assert_eq!(
        server.said_on_the_dock.as_deref(),
        Some(applications),
        "Tab never brought the Dock to say the keyboard is on Applications"
    );

    // Enter: the market opens, on its first card.
    assert!(server.the_keyboard_takes_the_slot_it_is_on());
    assert!(
        server.market.is_open(),
        "the premise: Enter opened the market"
    );
    let first = alo_software::Catalogue::decided()
        .expect("the shipped catalogue")
        .every()
        .first()
        .map(|one| one.application().identifier().to_owned());
    assert_eq!(
        server.said_in_the_market, first,
        "the market opened and did not say the card it opened on"
    );

    // Down, until the market says Kate.
    for _ in 0..128 {
        if server.said_in_the_market.as_deref() == Some(KATE) {
            break;
        }
        tapped(&mut server, KEY_DOWN);
    }
    assert_eq!(
        server.said_in_the_market.as_deref(),
        Some(KATE),
        "Down never brought the market to say the keyboard is on Kate"
    );

    // Enter asks for Kate.
    tapped(&mut server, KEY_ENTER);
    assert_eq!(
        server
            .take_what_was_asked_to_install()
            .map(|listed| listed.application().identifier().to_owned())
            .as_deref(),
        Some(KATE)
    );
}

/// **Leaving forgets what was said**, so arriving again says it again: a walk
/// that comes back waits for a new line, not the old one.
#[test]
fn leaving_forgets_what_was_said() {
    let (_runtime, mut server) = a_seat_with_its_dock("the-road-forgets");
    server.the_keyboard_is_now_in(Some(AnArea::TheDock));
    server.the_keyboard_is_now_in(None);
    assert_eq!(server.said_on_the_dock, None, "the Dock left, still said");

    server.do_this_native_action(alo_dock::ANativeAction::Applications);
    assert!(server.said_in_the_market.is_some());
    tapped(&mut server, KEY_ESC);
    assert!(!server.market.is_open(), "the premise: Escape closed it");
    assert_eq!(
        server.said_in_the_market, None,
        "the market closed, still said"
    );
}
