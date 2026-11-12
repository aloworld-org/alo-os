//! **A press on Settings, down the pointer road a person's press takes**: a
//! category shows its page, a row is chosen as Enter chooses it — written to
//! the machine — Close shuts it, and a press on an application in front of
//! Settings never reaches it. Each at a pan and a zoom, so what is pressed is
//! what the camera drew.

use std::os::unix::fs::PermissionsExt as _;

use alo_appearance::TextScale;
use alo_canvas::Zoom;
use alo_strings::Direction;
use smithay::backend::input::ButtonState;
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Point, Rectangle, Size};

use crate::display_scale::DisplayScale;
use crate::settings_categories::SettingsCategory;
use crate::settings_testing::{Machine, a_persons_machine, light, settings_words};
use crate::which_window_is_in_front::AKindOfWindow;
use crate::{Server, SettingsRow};

/// The display these tests draw on: the design's own.
const A_DISPLAY: (i32, i32) = (1440, 960);

/// A session with a keyboard and a pointer, and Settings open on it.
fn with_settings_open(named: &str) -> (tempfile::TempDir, Server, Machine) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        named,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.enable_pointer().expect("a pointer on this seat");
    server.surfaces.popups.output_size = Some(Size::from(A_DISPLAY));
    let machine = a_persons_machine(named);
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    server.the_person_went_to_settings();
    (runtime, server, machine)
}

/// What a frame does with Settings: place it, lay it out in its room, and
/// keep where it drew everything — the steps `crate::direct_desktop` takes.
fn a_frame(server: &mut Server) {
    let sizes = crate::settings_on_the_plane::how_big_settings_is(
        A_DISPLAY,
        TextScale::ordinary(),
        DisplayScale::ONE,
    )
    .expect("room");
    let _ = server.settings_is_placed(sizes.window, A_DISPLAY, Direction::LeftToRight);
    let room = server
        .where_the_settings_room_is_drawn("", sizes)
        .expect("placed");
    let mut labels = crate::WindowControlLabels::new().expect("the bundled faces");
    let picture = crate::settings_raster::picture(
        server.the_settings_window(),
        &settings_words(),
        &mut labels,
        sizes.room,
        light(),
        std::time::SystemTime::now(),
        None,
        DisplayScale::ONE,
    )
    .expect("Settings is drawn");
    let drawn = crate::settings_as_drawn::SettingsAsDrawn::of(&picture, room);
    server.settings_was_drawn(drawn);
}

/// Press and let go at a point of the display, then carry out what the press
/// left waiting, as the lane does after every event.
fn press_at(server: &mut Server, at: Point<i32, Physical>) {
    server
        .pointer_motion(f64::from(at.x), f64::from(at.y), 1)
        .expect("the pointer moved");
    let _ = server
        .pointer_button(0x110, ButtonState::Pressed, 2)
        .expect("the press was handled");
    let _ = server
        .pointer_button(0x110, ButtonState::Released, 3)
        .expect("the release was handled");
    crate::a_key_reaches_settings::a_click_is_carried_out(server, &settings_words());
}

/// The middle of a rectangle.
fn middle(of: Rectangle<i32, Physical>) -> Point<i32, Physical> {
    Point::from((of.loc.x + of.size.w / 2, of.loc.y + of.size.h / 2))
}

/// **A press on a category shows its page**, zoomed out to half — where the
/// category is drawn, not where it would be at life size.
#[test]
fn a_press_on_a_category_shows_its_page_at_a_zoom() {
    let (_runtime, mut server, _machine) = with_settings_open("click-category");
    a_frame(&mut server);
    assert!(
        server
            .zoom_the_canvas(Zoom::of(500).expect("a zoom"), (0, 0))
            .is_some()
    );
    a_frame(&mut server);
    let seen = server
        .settings_as_drawn
        .as_ref()
        .and_then(|drawn| drawn.where_the_category_is(SettingsCategory::Terminal))
        .expect("drawn");
    press_at(&mut server, middle(seen));
    assert_eq!(
        server.the_settings_window().category(),
        Some(SettingsCategory::Terminal)
    );
    assert!(server.settings_is_taking_the_keys());
}

/// **A press on a row is Enter on it**, carried out by the steps a key takes:
/// choosing how code runs writes the machine's file.
#[test]
fn a_press_on_a_row_chooses_it_as_enter_does() {
    let (_runtime, mut server, machine) = with_settings_open("click-row");
    a_frame(&mut server);
    let category = server
        .settings_as_drawn
        .as_ref()
        .and_then(|drawn| drawn.where_the_category_is(SettingsCategory::RunningCode))
        .expect("drawn");
    press_at(&mut server, middle(category));
    a_frame(&mut server);
    let row = SettingsRow::HowCodeRuns(
        *alo_capability::HowCodeRuns::ALL
            .first()
            .expect("there is a way code runs"),
    );
    let seen = server
        .settings_as_drawn
        .as_ref()
        .and_then(|drawn| drawn.where_the_row_is(&row))
        .expect("the row is in view");
    assert!(
        !machine.places.how_code_runs().exists(),
        "the control: nothing was chosen before the press"
    );
    press_at(&mut server, middle(seen));
    assert!(
        machine.places.how_code_runs().exists(),
        "a press on the row did not choose it"
    );
    assert_eq!(
        server
            .the_settings_window()
            .focused(std::time::SystemTime::now()),
        Some(row)
    );
}

/// **The refusal path**: with the applications in front, Settings takes no
/// key; a press beside its window is not its own and changes nothing in it;
/// and a press on it brings it back in front, as a press on any window does.
#[test]
fn settings_behind_takes_no_key_and_a_press_beside_it_is_not_its() {
    let (_runtime, mut server, _machine) = with_settings_open("click-behind");
    a_frame(&mut server);
    let window = server
        .settings_as_drawn
        .as_ref()
        .map(crate::settings_as_drawn::SettingsAsDrawn::the_window_on_the_screen)
        .expect("drawn");
    let category = server
        .settings_as_drawn
        .as_ref()
        .and_then(|drawn| drawn.where_the_category_is(SettingsCategory::Terminal))
        .expect("drawn");
    server.the_person_went_to(AKindOfWindow::Applications);
    assert!(
        !server.settings_is_taking_the_keys(),
        "keys went to a window behind"
    );
    let before = server.the_settings_window().category();
    press_at(
        &mut server,
        Point::from((
            window.loc.x + window.size.w + 8,
            window.loc.y + window.size.h / 2,
        )),
    );
    assert_eq!(server.the_settings_window().category(), before);
    assert!(
        !server.settings_is_taking_the_keys(),
        "a press beside it brought it in front"
    );
    press_at(&mut server, middle(category));
    assert!(server.settings_is_taking_the_keys());
    assert_eq!(
        server.the_settings_window().category(),
        Some(SettingsCategory::Terminal)
    );
}

/// **Close on its edge shuts Settings**, and nothing is written by it.
#[test]
fn close_on_the_edge_shuts_settings() {
    let (_runtime, mut server, _machine) = with_settings_open("click-close");
    a_frame(&mut server);
    let window = server
        .settings_as_drawn
        .as_ref()
        .map(crate::settings_as_drawn::SettingsAsDrawn::the_window_on_the_screen)
        .expect("drawn");
    let edge = crate::an_alo_windows_edge::the_edge_over(window, Size::from((10_000, 10_000)), 100);
    let close = edge
        .controls
        .iter()
        .find(|control| control.does == crate::window_edge::OnTheEdge::Close)
        .expect("Close on the edge")
        .target;
    press_at(
        &mut server,
        Point::from((
            close.loc.x + close.size.w / 2,
            close.loc.y + close.size.h / 2,
        )),
    );
    assert!(
        !server.the_settings_window().is_open(),
        "Close left it open"
    );
    assert!(server.where_settings_is_on_the_plane().is_none());
}

/// Every choice in the Pointer section, each row's in reverse so that every
/// press changes something: the first of each row is what a machine comes
/// with.
fn every_pointer_choice() -> Vec<SettingsRow> {
    let mut parts: Vec<SettingsRow> = crate::PointerSize::ALL
        .into_iter()
        .rev()
        .map(SettingsRow::PointerSize)
        .collect();
    parts.extend(
        alo_appearance::PointerShape::ALL
            .into_iter()
            .rev()
            .map(SettingsRow::PointerShape),
    );
    parts.extend(
        alo_appearance::PointerColour::ALL
            .into_iter()
            .rev()
            .map(SettingsRow::PointerColour),
    );
    parts
}

/// Whether what Settings holds, and what sign-in would read from the
/// person's `appearance.toml`, are both `part`.
fn holds(server: &Server, machine: &Machine, part: &SettingsRow) -> (bool, bool) {
    let window = server.the_settings_window();
    let (kept, _) =
        alo_appearance::keeping::at_sign_in(&machine.kept(alo_appearance::keeping::THE_FILE));
    match part {
        SettingsRow::PointerSize(size) => (
            window.pointer_size() == Some(*size),
            kept.pointer_size() == *size,
        ),
        SettingsRow::PointerShape(shape) => (
            window.pointer_shape() == Some(*shape),
            kept.pointer_shape() == *shape,
        ),
        SettingsRow::PointerColour(colour) => (
            window.pointer_colour() == Some(*colour),
            kept.pointer_colour() == *colour,
        ),
        _ => (false, false),
    }
}

/// **A press on a pointer choice chooses that choice, and never the row the
/// keyboard is on** — at life size and zoomed out to half.
///
/// The walk of 2026-11-08 reported a press on *Larger* that turned *Reduced
/// motion* over instead, Reduced motion being where the keyboard was. The
/// machine's own files said otherwise — `appearance.toml` was written with
/// `pointer_size = "Larger"` at the press, and `access.toml` not at all — and
/// this test holds that answer: the keyboard is put on *Reduced motion*,
/// turned on, before every press, and every one of the eight choices is
/// pressed where the frame drew it. A press read as Enter on the focused row
/// would turn the switch off and write `access.toml`, and both are asserted.
#[test]
fn a_press_on_a_pointer_choice_chooses_it_and_not_the_row_with_the_keyboard() {
    let mut widths = Vec::new();
    for zoom in [None, Some(500)] {
        let (_runtime, mut server, machine) =
            with_settings_open(&format!("click-pointer-{}", zoom.unwrap_or(1000)));
        a_frame(&mut server);
        let page = server
            .settings_as_drawn
            .as_ref()
            .and_then(|drawn| drawn.where_the_category_is(SettingsCategory::Accessibility))
            .expect("drawn");
        press_at(&mut server, middle(page));
        a_frame(&mut server);
        // **Reduced motion on, by a press**, and the keyboard left on it. The
        // keyboard goes there first, which scrolls it into view: at the
        // bottom of the page it is drawn cut by the window's edge, and a press
        // on the half that is not seen is not a press on Settings.
        let _ = server.settings.pointed_at(
            &crate::settings_pointed_at::PointedAt::ARow(SettingsRow::ReducedMotion),
            std::time::SystemTime::now(),
        );
        a_frame(&mut server);
        let switch = server
            .settings_as_drawn
            .as_ref()
            .and_then(|drawn| drawn.where_the_row_is(&SettingsRow::ReducedMotion))
            .expect("the switch is in view");
        press_at(&mut server, middle(switch));
        assert_eq!(server.the_settings_window().motion_is_reduced(), Some(true));
        let access = machine.kept(alo_access::keeping::THE_FILE);
        let written = crate::settings_testing::bytes(&access);
        assert!(written.is_some(), "the control: the switch wrote its file");
        if let Some(zoom) = zoom {
            assert!(
                server
                    .zoom_the_canvas(Zoom::of(zoom).expect("a zoom"), (0, 0))
                    .is_some()
            );
        }
        for part in every_pointer_choice() {
            // **The keyboard back on the switch**, the way a press on it
            // would put it there, without turning it over.
            let _ = server.settings.pointed_at(
                &crate::settings_pointed_at::PointedAt::ARow(SettingsRow::ReducedMotion),
                std::time::SystemTime::now(),
            );
            a_frame(&mut server);
            assert_eq!(
                server
                    .the_settings_window()
                    .focused(std::time::SystemTime::now()),
                Some(SettingsRow::ReducedMotion),
                "the control: the keyboard is on the switch before the press"
            );
            assert_eq!(holds(&server, &machine, &part), (false, false), "{part:?}");
            let seen = server
                .settings_as_drawn
                .as_ref()
                .and_then(|drawn| drawn.where_the_row_is(&part))
                .expect("every pointer choice is in view");
            if part == SettingsRow::PointerSize(crate::PointerSize::Larger) {
                widths.push(seen.size.w);
            }
            press_at(&mut server, middle(seen));
            assert_eq!(
                holds(&server, &machine, &part),
                (true, true),
                "a press on {part:?} at {zoom:?} did not choose it"
            );
            assert_eq!(
                server.the_settings_window().motion_is_reduced(),
                Some(true),
                "a press on {part:?} at {zoom:?} turned the focused switch over"
            );
            assert_eq!(
                crate::settings_testing::bytes(&access),
                written,
                "a press on {part:?} at {zoom:?} wrote access.toml"
            );
        }
    }
    // **The control on the zoom**: *Larger* was drawn at half the width, so
    // the second pass really pressed a zoomed-out window.
    assert_eq!(widths.len(), 2, "Larger was pressed once at each size");
    let (life, half) = (widths.first().copied(), widths.last().copied());
    let (Some(life), Some(half)) = (life, half) else {
        return;
    };
    assert!(
        (half - life / 2).abs() <= 2,
        "zoomed out to half, Larger was {half} wide against {life}"
    );
}

/// **A press on the *Screen reader* switch turns it over, keeps it, and hands
/// it to the desktop** (ADR 0151) — on, then off, each written to the person's
/// `access.toml` and each held for the desktop's next frame, which is what
/// starts and stops the reader. Nothing is held before the switch is pressed.
#[test]
fn a_press_on_the_screen_reader_switch_reaches_the_desktop() {
    let (_runtime, mut server, machine) = with_settings_open("click-screen-reader");
    a_frame(&mut server);
    let page = server
        .settings_as_drawn
        .as_ref()
        .and_then(|drawn| drawn.where_the_category_is(SettingsCategory::Accessibility))
        .expect("drawn");
    press_at(&mut server, middle(page));
    a_frame(&mut server);
    assert_eq!(
        server.the_screen_reader_to_tell(),
        None,
        "the control: opening the page asks the desktop nothing"
    );
    assert_eq!(
        server.the_settings_window().screen_reader_is_on(),
        Some(false),
        "the screen reader is not off as this computer comes"
    );
    for expected in [true, false] {
        let switch = server
            .settings_as_drawn
            .as_ref()
            .and_then(|drawn| drawn.where_the_row_is(&SettingsRow::ScreenReader))
            .expect("the screen reader's switch is first on the page, in view");
        press_at(&mut server, middle(switch));
        assert_eq!(
            server.the_settings_window().screen_reader_is_on(),
            Some(expected)
        );
        assert_eq!(
            server.the_screen_reader_to_tell(),
            Some(expected),
            "the desktop is not told the switch"
        );
        let access = machine.kept(alo_access::keeping::THE_FILE);
        let kept =
            alo_access::keeping::read(access.parent().expect("a folder")).expect("the file reads");
        assert_eq!(kept.has(alo_access::Setting::ScreenReader), expected);
        a_frame(&mut server);
    }
}
