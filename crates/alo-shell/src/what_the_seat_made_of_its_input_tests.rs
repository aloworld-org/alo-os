//! That a seat counts what it routed, where it classifies it — through
//! `crate::direct_seat` on a real display, never by calling the counters.
//!
//! `crate::a_key_reaches_the_market_tests` records what calling past the seat
//! costs: a test that drove a window directly stayed green after the line
//! carrying a press out had been deleted. The counters here are one line each,
//! so a test that called them would assert that `+= 1` adds one. Every press
//! below goes in as an evdev code and is read back through the one accessor a
//! lane uses.

use smithay::backend::input::{ButtonState, KeyState};
use smithay::input::keyboard::XkbConfig;
use std::os::unix::fs::PermissionsExt as _;

use crate::{DirectKeyEvent, DirectPointerEvent, DirectSeatEvent, InputError, Server};

/// `a`, as evdev numbers it.
const KEY_A: u32 = 30;
/// `BTN_LEFT`, the first button `crate::direct_seat` will take.
const BTN_LEFT: u32 = 0x110;
/// One above `BTN_TASK`, which is the first it will not.
const BTN_BEYOND: u32 = 0x118;
/// The display these tests pretend to have, for the absolute motion below.
const EXTENT: (i32, i32) = (1280, 800);

/// A session with a keyboard and a pointer, in a private directory of its own.
///
/// **The pointer is enabled here** rather than in each test, because a seat
/// without one answers `PointerUnavailable` to the first button it is handed —
/// which is the fault `enable_pointer` was added to `session_desktop` to fix,
/// and not the thing any test below is about.
fn a_seat(what: &str) -> (tempfile::TempDir, Server) {
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
    server
        .enable_pointer()
        .expect("a seat that takes a pointer");
    (runtime, server)
}

/// One key transition, as libinput would hand it over.
fn key(server: &mut Server, state: KeyState, count: u32) -> Result<(), InputError> {
    server.direct_seat(
        true,
        EXTENT,
        DirectSeatEvent::Key(
            DirectKeyEvent {
                code: KEY_A,
                state,
                time: 11,
            },
            count,
        ),
    )
}

/// One pointer button transition, as libinput would hand it over.
fn button(
    server: &mut Server,
    code: u32,
    state: ButtonState,
    count: u32,
) -> Result<(), InputError> {
    server.direct_seat(
        true,
        EXTENT,
        DirectSeatEvent::Button {
            code,
            state,
            time: 12,
            count,
        },
    )
}

/// **A press is counted and its release is not.**
///
/// A press is something a person did; a release is the end of the same act.
/// Counting both would report two for one click, and *button presses: 6* after
/// three clicks is a number a reader has to decode before it is of any use —
/// on a machine whose only way of telling anybody anything is to print a
/// sentence, that decoding is the whole cost.
#[test]
fn a_press_is_counted_and_the_release_that_ends_it_is_not() -> Result<(), InputError> {
    let (_runtime, mut server) = a_seat("counts-presses");

    assert_eq!(
        server.what_the_seat_made_of_its_input(),
        crate::WhatTheSeatMadeOfIt::default(),
        "the premise: a seat that has been handed nothing has made nothing of it"
    );

    key(&mut server, KeyState::Pressed, 1)?;
    button(&mut server, BTN_LEFT, ButtonState::Pressed, 1)?;

    let after_the_presses = server.what_the_seat_made_of_its_input();
    assert_eq!(after_the_presses.key_presses, 1);
    assert_eq!(after_the_presses.button_presses, 1);
    assert_eq!(
        after_the_presses.refused, 0,
        "an event this seat routed was also counted as one it could not read"
    );

    key(&mut server, KeyState::Released, 0)?;
    button(&mut server, BTN_LEFT, ButtonState::Released, 0)?;

    assert_eq!(
        server.what_the_seat_made_of_its_input(),
        after_the_presses,
        "a release moved the counts, so one click reads as two and the sentence a machine \
         with no shell prints has to be decoded before it can be used"
    );
    Ok(())
}

/// **The keys are the control, and that is why they are separate.**
///
/// *Button presses: 0* has two readings — a mouse whose buttons are not
/// arriving, or a seat whose counting is not running at all — and only the
/// keys beside it tell those apart. This is the reading that was wanted on
/// 2026-10-18 and could not be had: keys climbing while buttons stand still.
#[test]
fn keys_climb_while_buttons_stand_still_when_only_keys_arrive() -> Result<(), InputError> {
    let (_runtime, mut server) = a_seat("keys-are-the-control");

    for _ in 0..4 {
        key(&mut server, KeyState::Pressed, 1)?;
        key(&mut server, KeyState::Released, 0)?;
    }

    let made = server.what_the_seat_made_of_its_input();
    assert_eq!(made.key_presses, 4, "the control did not move");
    assert_eq!(
        made.button_presses, 0,
        "a seat that was handed no button reported one, so the control proves nothing"
    );
    assert_eq!(made.refused, 0);
    Ok(())
}

/// **An event the seat refused is counted by neither of the two routed
/// numbers.**
///
/// The refusal itself is the lane's to count — `crate::direct_desktop` calls
/// `Server::an_input_event_was_refused` once it has decided the refusal is
/// survivable — so what this holds shut is the other half: a button this seat
/// would not take must not also appear as a button it routed. One event in two
/// numbers would make the pair that answers *did my click arrive* answer it
/// wrongly in the one case it exists for.
#[test]
fn a_button_this_seat_refuses_is_not_also_a_button_it_routed() {
    let (_runtime, mut server) = a_seat("refused-is-not-routed");

    let why = button(&mut server, BTN_BEYOND, ButtonState::Pressed, 1)
        .expect_err("a code above BTN_TASK is refused");
    assert!(
        why.is_about_one_event(),
        "{why:?} would end the display loop, so one mouse with an extra button takes the \
         desktop away"
    );

    assert_eq!(
        server.what_the_seat_made_of_its_input(),
        crate::WhatTheSeatMadeOfIt::default(),
        "a refused button was counted as routed"
    );

    // And the lane's half, from the other side: the refusal lands in the third
    // number and leaves the first two where they were.
    server.an_input_event_was_refused();
    let made = server.what_the_seat_made_of_its_input();
    assert_eq!(made.refused, 1);
    assert_eq!((made.key_presses, made.button_presses), (0, 0));
}

/// **A press the panel took is still a press this seat made sense of.**
///
/// `crate::direct_seat` offers every button to the panel before routing it, and
/// a claimed one returns early — so until 2026-10-18 it returned before any
/// counting could happen. That is the shape of the fault this whole change is
/// about: a click that reached the machine, did something, and left no sign it
/// had arrived. A number that answered *did my click arrive* with **no** for
/// every click on this compositor's own furniture would be worse than no
/// number, because it would be believed.
#[test]
fn a_press_the_panel_took_is_counted_as_one_the_seat_routed() -> Result<(), InputError> {
    use crate::desktop_testing::{an_appearance, noon_look};
    use crate::panel_raster::{WhetherRevealed, picture};
    use crate::which_preview_the_pointer_is_on::ThePanelAsDrawn;
    use alo_canvas::{Place, Zoom};
    use alo_dock::window::{AppId, HowItSits, Window, WindowId};
    use alo_dock::{Patch, Spot};
    use alo_put_aside::Panel;
    use alo_put_aside::the_region_the_panel_claims::WhichEdge;
    use alo_put_aside::whether_it_is_private::Privacy;
    use alo_strings::Direction;

    let (_runtime, mut server) = a_seat("the-panel-took-it");

    let mut panel = Panel::new();
    panel
        .put_aside(
            &Window::of(
                WindowId::numbered(11),
                Some(AppId::named("Docs").expect("a fixture names its application")),
                "A window",
                Patch::of(Spot::at(0, 0), 800, 600).expect("a fixture gives its patch an extent"),
                HowItSits::OnTheCanvas,
            ),
            Zoom::LIFE_SIZE,
            Place::FIRST,
            Privacy::Ordinary,
        )
        .expect("a window this panel can hold");
    let drawn = ThePanelAsDrawn::of(
        &panel,
        picture(
            &panel,
            noon_look(&an_appearance(), Direction::LeftToRight),
            EXTENT,
            WhichEdge::Right,
            WhetherRevealed::Revealed,
        )
        .expect("a panel the design's own frame can hold"),
    );
    let rail = drawn.rail();
    server.the_panel_was_drawn(drawn);

    // **Onto the rail, as a fraction of the display**, because that is the
    // space `DirectPointerEvent::Absolute` is in. The middle of the rail is
    // `ThePointer::OnTheSurface`, which is the one classification the panel
    // takes a press on.
    server.direct_seat(
        true,
        EXTENT,
        DirectSeatEvent::Pointer(DirectPointerEvent::Absolute {
            x: f64::from(rail.loc.x + rail.size.w / 2) / f64::from(EXTENT.0),
            y: f64::from(rail.loc.y + rail.size.h / 2) / f64::from(EXTENT.1),
            time: 13,
        }),
    )?;

    button(&mut server, BTN_LEFT, ButtonState::Pressed, 1)?;

    // **That the panel took it, asserted before the count is read.** Without
    // this the test passes either way: a press the panel declined is routed on
    // by `direct_pointer` and counted there, so every assertion below would
    // hold while the road this test exists for went untravelled. `Server` puts
    // the point of a claimed press here and nowhere else.
    assert_eq!(
        server.asked_to_bring_back.len(),
        1,
        "the panel did not claim the press, so this test is aiming at the wrong pixel and \
         proves nothing about the road it names"
    );

    let made = server.what_the_seat_made_of_its_input();
    assert_eq!(
        made.button_presses, 1,
        "the panel claimed the press and the machine has no sign the click ever arrived"
    );
    assert_eq!(made.refused, 0);
    Ok(())
}
