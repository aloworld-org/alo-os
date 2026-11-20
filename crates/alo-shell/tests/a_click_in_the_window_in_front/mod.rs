//! **One click at one point of the window in front, for alo** (ADR 0147,
//! amended 2026-11-17), against real clients on a real display.
//!
//! What is asserted is what the client was sent on the wire: one press and one
//! release of the primary button, at the point of its surface the proportions
//! name — and, for every refusal, **no button at all**.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or answer is the failure being reported"
)]

use super::support::{Application, Fixture};
use alo_acting::{Clicked, ToClick};
use alo_applications::Application as Named;
use alo_shell::typing_into_the_window_in_front::MayTypeInto;
use smithay::backend::input::ButtonState::Pressed;
use wayland_client::protocol::wl_pointer::ButtonState;

/// The application the window in front belongs to.
const A_GAME: &str = "net.supertuxkart.SuperTuxKart";

/// `BTN_LEFT`.
const PRIMARY: u32 = 0x110;

/// Answers the same for every process.
struct Answers(bool);

impl MayTypeInto for Answers {
    fn may_type_into(&mut self, _: &str, _: u32) -> bool {
        self.0
    }
}

fn a_display_with_a_pointer() -> Fixture {
    let f = Fixture::keyboard();
    f.backend(|s| s.enable_pointer())
        .expect("the display takes a pointer");
    f
}

fn mapped(f: &Fixture, app_id: &str) -> Application {
    let mut app = Application::new(f);
    app.toplevel.set_app_id(app_id.to_owned());
    app.configure();
    app.attach();
    app.sync();
    app
}

fn at(application: &str, across: u16, down: u16) -> ToClick {
    ToClick::at(
        Named::identified(application).expect("an identifier"),
        across,
        down,
    )
    .expect("a point inside the window")
}

fn clicked(f: &Fixture, asked: ToClick, may: bool) -> Clicked {
    f.backend(move |s| s.clicked_in_the_window_in_front(&asked, &mut Answers(may)))
}

/// **One press and one release, at the point named**: halfway across and a
/// quarter down a 16 × 16 window is (8, 4) on its surface.
#[test]
fn the_window_in_front_is_clicked_once_where_the_proportions_say() {
    let f = a_display_with_a_pointer();
    let mut game = mapped(&f, A_GAME);
    f.focus(Some(0)).expect("the window takes the keyboard");
    assert_eq!(clicked(&f, at(A_GAME, 500, 250), true), Clicked::Clicked);
    game.sync();
    assert_eq!(
        game.events.pointer.buttons,
        vec![
            (PRIMARY, ButtonState::Pressed),
            (PRIMARY, ButtonState::Released)
        ]
    );
    let entered = game
        .events
        .pointer
        .enters
        .first()
        .map(|(_, x, y)| (*x, *y))
        .expect("the pointer entered the window");
    assert_eq!(entered, (8.0, 4.0), "not the point the proportions name");
}

/// **The far corner is the window's last pixel**, never past it.
#[test]
fn the_far_corner_is_inside_the_window() {
    let f = a_display_with_a_pointer();
    let mut game = mapped(&f, A_GAME);
    f.focus(Some(0)).expect("the window takes the keyboard");
    assert_eq!(clicked(&f, at(A_GAME, 1000, 1000), true), Clicked::Clicked);
    game.sync();
    let entered = game
        .events
        .pointer
        .enters
        .first()
        .map(|(_, x, y)| (*x, *y))
        .expect("the pointer entered the window");
    assert_eq!(entered, (15.0, 15.0));
}

/// **No window has the keyboard: nothing is clicked.**
#[test]
fn nothing_is_clicked_when_no_window_has_the_keyboard() {
    let f = a_display_with_a_pointer();
    let mut game = mapped(&f, A_GAME);
    assert_eq!(clicked(&f, at(A_GAME, 500, 500), true), Clicked::NotInFront);
    game.sync();
    assert!(game.events.pointer.buttons.is_empty());
}

/// **Another application's window in front: it is not clicked.**
#[test]
fn another_applications_window_is_not_clicked() {
    let f = a_display_with_a_pointer();
    let mut browser = mapped(&f, "org.mozilla.firefox");
    f.focus(Some(0)).expect("the window takes the keyboard");
    assert_eq!(clicked(&f, at(A_GAME, 500, 500), true), Clicked::NotInFront);
    browser.sync();
    assert!(browser.events.pointer.buttons.is_empty());
}

/// **A process alo may not reach at the person's setting: nothing is clicked.**
#[test]
fn a_window_alo_may_not_reach_is_not_clicked() {
    let f = a_display_with_a_pointer();
    let mut game = mapped(&f, A_GAME);
    f.focus(Some(0)).expect("the window takes the keyboard");
    assert_eq!(
        clicked(&f, at(A_GAME, 500, 500), false),
        Clicked::NotAlosCopy
    );
    game.sync();
    assert!(game.events.pointer.buttons.is_empty());
}

/// **While the person holds a button, alo clicks nothing**: their press is
/// theirs, and the only button the client hears is the person's own.
#[test]
fn nothing_is_clicked_while_the_person_is_pressing() {
    let f = a_display_with_a_pointer();
    let mut game = mapped(&f, A_GAME);
    f.focus(Some(0)).expect("the window takes the keyboard");
    f.backend(|s| s.pointer_motion(4.0, 4.0, 10))
        .expect("the pointer moves");
    assert!(
        f.backend(|s| s.pointer_button(PRIMARY, Pressed, 11))
            .expect("the person presses"),
        "the control: the person's press reached the window"
    );
    assert_eq!(
        clicked(&f, at(A_GAME, 500, 500), true),
        Clicked::ThePersonIsPressing
    );
    game.sync();
    assert!(
        game.events.pointer.buttons.len() <= 1,
        "alo pressed into the person's press: {:?}",
        game.events.pointer.buttons
    );
}
