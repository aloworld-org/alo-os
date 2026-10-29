//! That a Dock control's name stays while the pointer is on the control or on
//! the name, that Escape dismisses it without moving anything, and that Escape
//! is taken for nothing else.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_appearance::TextScale;
use alo_dock::places::WhatIsHere;
use alo_dock::{AppId, Dock, Edge};
use alo_strings::Direction;
use smithay::backend::input::KeyState;

use crate::display_scale::DisplayScale;
use crate::dock_raster::DockPicture;
use crate::the_docks_words::TheDocksWords;

/// Escape and Enter, in evdev's numbering.
const ESCAPE: u32 = 1;
const ENTER: u32 = 28;

/// The Dock's words, in English.
fn words() -> TheDocksWords {
    let strings = alo_strings::Strings::of(
        alo_saying::everything_this_machine_can_say().expect("the vocabulary"),
    );
    TheDocksWords::said(&strings, &crate::WhatTheDockHasOpen::Nothing)
}

/// A bottom Dock on a 1920 × 1080 display holding Files and Mail.
fn a_dock() -> DockPicture {
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("a patch");
    let mut windows = alo_dock::Windows::none();
    for (number, name) in ["Files", "Mail"].iter().enumerate() {
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number as u64),
            Some(AppId::named(name).expect("a named application")),
            "",
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    let apps = alo_dock::Holding::nothing().showing(&windows);
    let mut dock = Dock::shipped();
    dock.set_edge(Edge::Bottom);
    crate::dock_raster::picture(
        &dock,
        crate::desktop_testing::noon_look(
            &crate::desktop_testing::an_appearance(),
            Direction::LeftToRight,
        ),
        (1920, 1080),
        &apps,
        &apps,
        (&crate::WhatTheDockHasOpen::Nothing, &words()),
        None,
        &alo_dock::artwork::NOTHING,
        &mut crate::WindowControlLabels::new()
            .expect("the bundled faces")
            .fonts,
        DisplayScale::ONE,
    )
    .unwrap_or_else(|why| panic!("the Dock would not draw: {why:?}"))
}

/// Which slot `name`'s icon is.
fn slot_of(picture: &DockPicture, name: &str) -> usize {
    let what = WhatIsHere::Application(AppId::named(name).expect("named"));
    let place = picture
        .standing
        .places
        .each()
        .iter()
        .find(|place| place.what() == &what)
        .unwrap_or_else(|| panic!("{name} is not on this Dock"));
    crate::the_bars_pressable_slots::which_one_is(&picture.standing.places, place)
        .expect("a control a press reaches")
}

/// The middle of slot `which` on `picture`, in output pixels.
fn the_middle_of(picture: &DockPicture, which: usize) -> (f64, f64) {
    let place = crate::the_bars_pressable_slots::the_one_at(&picture.standing.places, which)
        .expect("the slot");
    let x = picture.band.loc.x
        + i32::try_from(place.from_the_start() + place.across() / 2).expect("small");
    let y = picture.band.loc.y + picture.band.size.h / 2;
    (f64::from(x), f64::from(y))
}

/// A seat of this test's own, with a keyboard and a pointer.
struct ASeat {
    /// Held so the runtime directory outlives the server.
    _runtime: tempfile::TempDir,
    server: crate::Server,
}

fn a_seat(named: &str) -> ASeat {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = crate::Server::bind_keyboard(
        runtime.path(),
        named,
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..smithay::input::keyboard::XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.enable_pointer().expect("a pointer on this seat");
    ASeat {
        _runtime: runtime,
        server,
    }
}

/// One frame: the Dock drawn with whatever name the server asks for, and both
/// recorded, as `crate::direct_desktop` records them.
fn a_frame(server: &mut crate::Server, dock: &DockPicture) -> Option<usize> {
    let mut picture = dock.clone();
    let asked = server.the_docks_name_is_for("a-display");
    crate::dock_tooltip::show_the_name(
        &mut picture,
        asked,
        &words(),
        TextScale::default(),
        Direction::LeftToRight,
        &mut crate::WindowControlLabels::new()
            .expect("the bundled faces")
            .fonts,
    );
    server.the_docks_slots_were_drawn("a-display", picture.standing.clone());
    server.the_docks_name_was_drawn("a-display", picture.the_name);
    asked
}

/// Move the pointer to `at`, then draw a frame.
fn point_at(seat: &mut ASeat, dock: &DockPicture, at: (f64, f64), time: u32) -> Option<usize> {
    seat.server
        .pointer_motion(at.0, at.1, time)
        .expect("the pointer moved");
    a_frame(&mut seat.server, dock)
}

/// **The pointer can move from the control onto its name without it going**:
/// across the gap between the bar and the name, and over the name itself. Off
/// both, it goes; onto a neighbour, the neighbour's name is shown.
#[test]
fn the_name_stays_while_the_pointer_is_on_the_control_or_the_name() {
    let mut seat = a_seat("the-name-stays-while-pointed-at");
    let dock = a_dock();
    seat.server
        .the_docks_slots_were_drawn("a-display", dock.standing.clone());
    let mail = slot_of(&dock, "Mail");
    let (x, y) = the_middle_of(&dock, mail);

    assert_eq!(point_at(&mut seat, &dock, (x, y), 1), Some(mail));
    let drawn = seat
        .server
        .the_docks_name_as_drawn
        .get("a-display")
        .copied()
        .expect("the name was drawn and recorded");
    let band_top = f64::from(dock.band.loc.y);

    // Straight up from the icon: out of the bar, across the gap, onto the name.
    let mut time = 2;
    let top_of_the_name = f64::from(drawn.reach.loc.y);
    let mut up = band_top - 1.0;
    while up > top_of_the_name {
        assert_eq!(
            point_at(&mut seat, &dock, (x, up), time),
            Some(mail),
            "the name went at y = {up}, between the control and the name"
        );
        up -= 3.0;
        time += 1;
    }

    // Above the name, it goes.
    assert_eq!(
        point_at(&mut seat, &dock, (x, top_of_the_name - 4.0), time),
        None,
        "the name stayed for a pointer over the canvas"
    );
    // And it does not come back for a pointer where it was, now that it is gone.
    assert_eq!(
        point_at(&mut seat, &dock, (x, top_of_the_name + 2.0), time + 1),
        None,
        "a name came back for a pointer over where it used to be"
    );

    // Onto Files: Files's name, even where Mail's reach might have covered it.
    let files = slot_of(&dock, "Files");
    assert_eq!(
        point_at(&mut seat, &dock, the_middle_of(&dock, files), time + 2),
        Some(files)
    );
}

/// **Escape dismisses the name without moving anything**: the pointer is
/// still on the control, the keyboard's place on the bar and the area that has
/// the keyboard are what they were, and nothing on the Dock opened or closed.
/// And it stays dismissed, frame after frame, while the same control is asked
/// about.
#[test]
fn escape_dismisses_the_name_and_moves_nothing() {
    let mut seat = a_seat("escape-dismisses-the-name");
    let dock = a_dock();
    seat.server
        .the_docks_slots_were_drawn("a-display", dock.standing.clone());
    let mail = slot_of(&dock, "Mail");
    let at = the_middle_of(&dock, mail);
    assert_eq!(point_at(&mut seat, &dock, at, 1), Some(mail));

    let keyboard_on = seat.server.which_slot_on_the_bar;
    let area = seat.server.which_area_has_the_keyboard;
    let open = seat.server.the_dock_was_asked_for.clone();

    assert!(
        seat.server
            .a_key_dismisses_the_docks_name(ESCAPE, KeyState::Pressed),
        "Escape was not taken with a name showing"
    );
    for frame in 0..3 {
        assert_eq!(
            a_frame(&mut seat.server, &dock),
            None,
            "the name came back on frame {frame} with nothing moved"
        );
    }
    assert_eq!(
        seat.server.the_slot_the_pointer_is_on("a-display"),
        Some(mail),
        "the pointer moved"
    );
    assert_eq!(seat.server.which_slot_on_the_bar, keyboard_on);
    assert_eq!(seat.server.which_area_has_the_keyboard, area);
    assert_eq!(seat.server.the_dock_was_asked_for, open);
}

/// **Once dismissed, the name comes back when the pointer has left and come
/// back** — dismissing one is not switching names off.
#[test]
fn a_dismissed_name_comes_back_after_the_pointer_leaves_and_returns() {
    let mut seat = a_seat("a-dismissed-name-comes-back");
    let dock = a_dock();
    seat.server
        .the_docks_slots_were_drawn("a-display", dock.standing.clone());
    let mail = slot_of(&dock, "Mail");
    let at = the_middle_of(&dock, mail);
    assert_eq!(point_at(&mut seat, &dock, at, 1), Some(mail));
    assert!(
        seat.server
            .a_key_dismisses_the_docks_name(ESCAPE, KeyState::Pressed)
    );
    assert_eq!(a_frame(&mut seat.server, &dock), None);

    assert_eq!(point_at(&mut seat, &dock, (10.0, 10.0), 2), None);
    assert_eq!(
        point_at(&mut seat, &dock, at, 3),
        Some(mail),
        "the name stayed dismissed after the pointer left and came back"
    );
}

/// **The keyboard's name is dismissed the same way**, and comes back when the
/// keyboard moves to another control.
#[test]
fn the_keyboards_name_is_dismissed_until_the_keyboard_moves() {
    let mut seat = a_seat("the-keyboards-name-is-dismissed");
    let dock = a_dock();
    let (files, mail) = (slot_of(&dock, "Files"), slot_of(&dock, "Mail"));
    seat.server.which_slot_on_the_bar = Some(files);
    assert_eq!(a_frame(&mut seat.server, &dock), Some(files));
    assert!(
        seat.server
            .a_key_dismisses_the_docks_name(ESCAPE, KeyState::Pressed)
    );
    assert_eq!(a_frame(&mut seat.server, &dock), None);
    assert_eq!(
        seat.server.which_slot_on_the_bar,
        Some(files),
        "the keyboard moved"
    );

    seat.server.which_slot_on_the_bar = Some(mail);
    assert_eq!(
        a_frame(&mut seat.server, &dock),
        Some(mail),
        "the next control's name was not shown"
    );
}

/// **Escape is taken for nothing else.** With no name showing it is not
/// taken, so it reaches whatever it reached before; nor is any other key, nor
/// Escape's release; and a second Escape, with the name already gone, is not
/// taken either.
#[test]
fn escape_is_taken_only_while_a_name_is_showing() {
    let mut seat = a_seat("escape-is-taken-only-for-a-name");
    let dock = a_dock();
    assert_eq!(a_frame(&mut seat.server, &dock), None, "the premise");
    assert!(
        !seat
            .server
            .a_key_dismisses_the_docks_name(ESCAPE, KeyState::Pressed),
        "Escape was taken with no name showing"
    );

    seat.server.which_slot_on_the_bar = Some(slot_of(&dock, "Files"));
    assert!(a_frame(&mut seat.server, &dock).is_some(), "the premise");
    assert!(
        !seat
            .server
            .a_key_dismisses_the_docks_name(ENTER, KeyState::Pressed),
        "Enter was taken by the name"
    );
    assert!(
        !seat
            .server
            .a_key_dismisses_the_docks_name(ESCAPE, KeyState::Released),
        "Escape's release was taken"
    );
    assert!(
        seat.server
            .a_key_dismisses_the_docks_name(ESCAPE, KeyState::Pressed)
    );
    assert!(
        !seat
            .server
            .a_key_dismisses_the_docks_name(ESCAPE, KeyState::Pressed),
        "a second Escape was taken with the name already gone"
    );
}
