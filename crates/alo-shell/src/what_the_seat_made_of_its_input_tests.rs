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

/// **A press that reached nothing is counted as arriving and not as landing.**
///
/// The two numbers answer different questions and the whole point of having
/// both is that a machine can tell them apart. Measured on 2026-10-18: three
/// presses arrived, none was refused, and the Dock did not move — at which
/// point *it arrived* was all the machine could say, and *it arrived and hit
/// nothing* and *it hit something that then did nothing* need different fixes.
///
/// A seat with no Dock recorded is the simplest machine where a press can
/// reach nothing, which is what this uses.
#[test]
fn a_press_that_reached_nothing_arrives_without_landing() -> Result<(), InputError> {
    let (_runtime, mut server) = a_seat("reached-nothing");

    button(&mut server, BTN_LEFT, ButtonState::Pressed, 1)?;

    let made = server.what_the_seat_made_of_its_input();
    assert_eq!(made.button_presses, 1, "the press did not arrive");
    assert_eq!(
        made.furniture_presses, 0,
        "a press that reached nothing was counted as landing on something, so the one \
         reading that tells the two faults apart cannot"
    );
    assert_eq!(made.refused, 0);
    Ok(())
}

/// **A press the Dock takes is counted as landing.**
///
/// The other side of the test above, because a number that is always nought
/// distinguishes nothing. The Dock is laid out by the same function the draw
/// uses, and the press is aimed at the middle of the slot that layout says is
/// its first — asked of the geometry rather than assumed from the design.
#[test]
fn a_press_the_dock_takes_is_counted_as_landing() -> Result<(), InputError> {
    use crate::where_the_dock_is::where_the_dock_is;

    let (_runtime, mut server) = a_seat("landed-on-the-dock");

    // **The two entries a fresh machine has before anything is installed**,
    // pinned rather than assumed: `Holding::nothing()` pins nothing, and a
    // Dock with no entries lays out only its fixed controls — which is a Dock
    // this test would press at and learn nothing from.
    let dock = alo_dock::Dock::shipped();
    let mut holding = alo_dock::Holding::nothing();
    holding.pin_entry(alo_dock::ADockEntry::Native(
        alo_dock::ANativeAction::Settings,
    ));
    holding.pin_entry(alo_dock::ADockEntry::Native(
        alo_dock::ANativeAction::Applications,
    ));
    let on_the_dock = holding.showing(&alo_dock::Windows::none());
    let places =
        where_the_dock_is(&dock, EXTENT, &on_the_dock).expect("a dock this display can hold");
    let first = places
        .places
        .each()
        .first()
        .expect("a dock with at least one slot");
    // **A slot says where it starts along the bar, not where it is on the
    // screen.** Measured the hard way: read as absolute, the first slot's
    // middle came out at x=180 on a bar that starts at 384, so the press
    // landed off the Dock entirely and the test failed for its own reason
    // rather than the code's.
    let middle = first.from_the_start() + first.across() / 2;
    let at_x = places.band.loc.x + i32::try_from(middle).expect("a place on a bar fits in an i32");
    let at_y = places.band.loc.y + places.band.size.h / 2;
    server.the_docks_slots_were_drawn("a-display", places);

    server.direct_seat(
        true,
        EXTENT,
        DirectSeatEvent::Pointer(DirectPointerEvent::Absolute {
            x: f64::from(at_x) / f64::from(EXTENT.0),
            y: f64::from(at_y) / f64::from(EXTENT.1),
            time: 14,
        }),
    )?;
    button(&mut server, BTN_LEFT, ButtonState::Pressed, 1)?;

    let made = server.what_the_seat_made_of_its_input();
    assert_eq!(made.button_presses, 1);
    assert_eq!(
        made.furniture_presses, 1,
        "the Dock took the press and the machine reports it reached nothing"
    );
    Ok(())
}

/// **A press on the market's own entry opens the market.**
///
/// The whole road, end to end, through the seat: `direct_seat` validates the
/// button, `direct_pointer` routes it, `pointer_button` offers it to the Dock,
/// the Dock reads its own geometry and answers `Do(Applications)`, and
/// `the_market_opens` opens the window.
///
/// **Written because reading that road proved nothing.** On 2026-10-18 a
/// booted machine took three presses on that entry, refused none of them, and
/// opened no window — and every link above, read twice, looked correct. A test
/// that walks the whole of it at the size that machine draws at is what says
/// whether the fault is in this code or in something around it.
///
/// It presses the slot the layout says is the market's, rather than a pixel
/// from the design: a test that tells the geometry where to press cannot find
/// a geometry that is wrong.
#[test]
fn a_press_on_the_markets_entry_opens_the_market() -> Result<(), InputError> {
    use crate::where_the_dock_is::where_the_dock_is;
    use alo_dock::places::WhatIsHere;

    let (_runtime, mut server) = a_seat("the-market-opens");

    let dock = alo_dock::Dock::shipped();
    let mut holding = alo_dock::Holding::nothing();
    holding.pin_entry(alo_dock::ADockEntry::Native(
        alo_dock::ANativeAction::Settings,
    ));
    holding.pin_entry(alo_dock::ADockEntry::Native(
        alo_dock::ANativeAction::Applications,
    ));
    let on_the_dock = holding.showing(&alo_dock::Windows::none());
    let places =
        where_the_dock_is(&dock, EXTENT, &on_the_dock).expect("a dock this display can hold");

    // The slot the layout itself calls the market's.
    let market = places
        .places
        .each()
        .iter()
        .find(|place| {
            matches!(
                place.what(),
                WhatIsHere::ANativeAction(alo_dock::ANativeAction::Applications)
            )
        })
        .expect("a dock holding the market's entry");
    let middle = market.from_the_start() + market.across() / 2;
    let at_x = places.band.loc.x + i32::try_from(middle).expect("a place on a bar fits in an i32");
    let at_y = places.band.loc.y + places.band.size.h / 2;
    server.the_docks_slots_were_drawn("a-display", places);

    assert!(!server.market.is_open(), "the premise: the market is shut");

    server.direct_seat(
        true,
        EXTENT,
        DirectSeatEvent::Pointer(DirectPointerEvent::Absolute {
            x: f64::from(at_x) / f64::from(EXTENT.0),
            y: f64::from(at_y) / f64::from(EXTENT.1),
            time: 15,
        }),
    )?;
    button(&mut server, BTN_LEFT, ButtonState::Pressed, 1)?;

    assert!(
        server.market.is_open(),
        "a press on the market's own entry, at the middle of the slot the layout gives it, \
         opened nothing"
    );
    Ok(())
}

/// **The machine's own construction, end to end: a fresh Dock, filled the way
/// a fresh machine fills it, pressed where its market entry is drawn.**
///
/// The test above pins the two native entries **by hand**, which is how every
/// press test in this crate builds a Dock. A booted machine does not: it calls
/// `Server::its_dock_is_filled_from_what_they_kept` once, with nobody's kept
/// pins and a discovery that completed with nothing — *the Dock's default
/// roles resolve to `Completed([])`*, in that machine's own words on
/// 2026-10-18.
///
/// **That difference is the only one left between a test that passes and a
/// machine that does nothing**, so it is the one a test has to close. A seam a
/// caller has to remember to connect is a seam that will be found
/// disconnected — this crate has measured that once already, when three call
/// sites each supplied `Holding::nothing()` and 854 of 854 tests passed with
/// the draw reading a constant.
#[test]
fn a_dock_filled_the_way_a_fresh_machine_fills_it_opens_the_market() -> Result<(), InputError> {
    use crate::where_the_dock_is::where_the_dock_is;
    use alo_dock::pinning::{Discovery, ThePins};
    use alo_dock::places::WhatIsHere;

    let (_runtime, mut server) = a_seat("a-fresh-machines-dock");

    // Nobody's kept pins, and a discovery that found nothing — the two the
    // booted machine had.
    let _owed = server
        .its_dock_is_filled_from_what_they_kept(&ThePins::NeverSet, &Discovery::Completed(vec![]));

    let on_the_dock = server.what_this_persons_dock_holds();
    assert!(
        !on_the_dock.is_empty(),
        "a fresh machine's Dock came out empty, so nothing it draws can be pressed"
    );

    let places = where_the_dock_is(&alo_dock::Dock::shipped(), EXTENT, &on_the_dock)
        .expect("a dock this display can hold");
    let market = places
        .places
        .each()
        .iter()
        .find(|place| {
            matches!(
                place.what(),
                WhatIsHere::ANativeAction(alo_dock::ANativeAction::Applications)
            )
        })
        .expect("a fresh machine's Dock holds the market's entry");
    let middle = market.from_the_start() + market.across() / 2;
    let at_x = places.band.loc.x + i32::try_from(middle).expect("a place on a bar fits in an i32");
    let at_y = places.band.loc.y + places.band.size.h / 2;
    server.the_docks_slots_were_drawn("a-display", places);

    server.direct_seat(
        true,
        EXTENT,
        DirectSeatEvent::Pointer(DirectPointerEvent::Absolute {
            x: f64::from(at_x) / f64::from(EXTENT.0),
            y: f64::from(at_y) / f64::from(EXTENT.1),
            time: 16,
        }),
    )?;
    button(&mut server, BTN_LEFT, ButtonState::Pressed, 1)?;

    assert!(
        server.market.is_open(),
        "a fresh machine's own Dock, pressed where its own layout puts the market, opened \
         nothing"
    );
    Ok(())
}

/// **An empty panel must not claim a press meant for the Dock.**
///
/// `crate::direct_seat` offers every button to the panel before routing it, and
/// a claimed press never reaches `pointer_button` — so it never reaches the
/// Dock either. Every other press test in this crate runs with **no panel
/// drawn**, where `what_the_pointer_is_to_the_panel` answers [`None`] and the
/// press passes straight on. A booted machine always has one, and on
/// 2026-10-18 it had an **empty** one: nothing had been put aside, because
/// nothing had been opened.
///
/// That combination — a panel drawn, holding nothing, and a press aimed at the
/// Dock — is the one no test covered, and it is the last difference between a
/// machine that does nothing and a suite that passes.
#[test]
fn an_empty_panel_does_not_claim_a_press_meant_for_the_dock() -> Result<(), InputError> {
    use crate::desktop_testing::{an_appearance, noon_look};
    use crate::panel_raster::{WhetherRevealed, picture};
    use crate::where_the_dock_is::where_the_dock_is;
    use crate::which_preview_the_pointer_is_on::ThePanelAsDrawn;
    use alo_dock::pinning::{Discovery, ThePins};
    use alo_dock::places::WhatIsHere;
    use alo_put_aside::Panel;
    use alo_put_aside::the_region_the_panel_claims::WhichEdge;
    use alo_strings::Direction;

    let (_runtime, mut server) = a_seat("an-empty-panel");

    // **The panel a fresh desktop has: drawn, and holding nothing.**
    let panel = Panel::new();
    let drawn = ThePanelAsDrawn::of(
        &panel,
        picture(
            &panel,
            noon_look(&an_appearance(), Direction::LeftToRight),
            EXTENT,
            WhichEdge::Right,
            WhetherRevealed::Concealed,
        )
        .expect("a panel this display can hold"),
    );
    server.the_panel_was_drawn(drawn);

    // The Dock a fresh machine fills for itself.
    let _owed = server
        .its_dock_is_filled_from_what_they_kept(&ThePins::NeverSet, &Discovery::Completed(vec![]));
    let on_the_dock = server.what_this_persons_dock_holds();
    let places = where_the_dock_is(&alo_dock::Dock::shipped(), EXTENT, &on_the_dock)
        .expect("a dock this display can hold");
    let market = places
        .places
        .each()
        .iter()
        .find(|place| {
            matches!(
                place.what(),
                WhatIsHere::ANativeAction(alo_dock::ANativeAction::Applications)
            )
        })
        .expect("a fresh machine's Dock holds the market's entry");
    let middle = market.from_the_start() + market.across() / 2;
    let at_x = places.band.loc.x + i32::try_from(middle).expect("a place on a bar fits in an i32");
    let at_y = places.band.loc.y + places.band.size.h / 2;
    server.the_docks_slots_were_drawn("a-display", places);

    server.direct_seat(
        true,
        EXTENT,
        DirectSeatEvent::Pointer(DirectPointerEvent::Absolute {
            x: f64::from(at_x) / f64::from(EXTENT.0),
            y: f64::from(at_y) / f64::from(EXTENT.1),
            time: 17,
        }),
    )?;
    button(&mut server, BTN_LEFT, ButtonState::Pressed, 1)?;

    assert!(
        server.market.is_open(),
        "an empty panel claimed a press aimed at the Dock, so it never reached the Dock at all"
    );
    Ok(())
}

/// **A market that opened must also have somewhere to be drawn.**
///
/// `MarketWindow::opened` cannot fail — it sets a flag — so every press test in
/// this crate ends at *the window is open* and stops. The frame is a second
/// question: `where_the_market_opens` answers [`None`] three different ways,
/// each silent, and a `None` there is a window that is open and draws nothing.
///
/// A person meets exactly what a press that was ignored looks like. So this
/// asks the placement, at the size the booted machine of 2026-10-18 draws at,
/// with the catalogue that machine ships.
#[test]
fn the_market_has_somewhere_to_be_drawn_on_the_size_a_machine_draws() {
    use crate::where_the_market_opens::where_the_market_opens;

    let mut window = crate::MarketWindow::closed();
    window.opened(
        alo_software::Catalogue::decided().expect("the catalogue this machine ships"),
        Vec::new(),
    );
    assert!(window.is_open(), "the premise: the window is open");

    let showing = window.shows();
    let view = smithay::utils::Rectangle::new(
        smithay::utils::Point::from((0, 0)),
        smithay::utils::Size::from((1280, 800)),
    );

    assert!(
        where_the_market_opens(view, &showing, alo_appearance::Appearance::shipped().text())
            .is_some(),
        "the market is open and has nowhere to be drawn, so a person who pressed its entry \
         sees exactly what an ignored press looks like"
    );
}

/// **A seat that has recorded no Dock reports none, and one that has reports
/// its slots.**
///
/// Nought and a Dock on the screen are the same picture and opposite faults: a
/// Dock drawn from one list and hit-tested against another looks exactly like
/// a Dock that ignores presses. So the number is reported, and both sides of
/// it are held here — a number that is always nought distinguishes nothing.
#[test]
fn a_seat_says_how_many_dock_slots_were_recorded() {
    use crate::where_the_dock_is::where_the_dock_is;
    use alo_dock::pinning::{Discovery, ThePins};

    let (_runtime, mut server) = a_seat("slots-recorded");
    assert_eq!(
        server.what_the_seat_made_of_its_input().dock_slots_recorded,
        0,
        "the premise: nothing has drawn a Dock yet"
    );

    let _owed = server
        .its_dock_is_filled_from_what_they_kept(&ThePins::NeverSet, &Discovery::Completed(vec![]));
    let on_the_dock = server.what_this_persons_dock_holds();
    let places = where_the_dock_is(&alo_dock::Dock::shipped(), EXTENT, &on_the_dock)
        .expect("a dock this display can hold");
    let how_many = places.places.each().len();
    server.the_docks_slots_were_drawn("a-display", places);

    assert_eq!(
        server.what_the_seat_made_of_its_input().dock_slots_recorded,
        how_many,
        "a Dock was recorded and the seat still reports none, so the one reading that tells \
         a Dock nobody recorded from a Dock that ignores presses cannot"
    );
    assert!(
        how_many > 0,
        "a fresh machine's Dock laid out no slots at all"
    );
}
