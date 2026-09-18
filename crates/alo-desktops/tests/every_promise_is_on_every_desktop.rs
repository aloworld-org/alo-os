//! The egress indicator, the approval surface and the agent overlay are on
//! **every** desktop at once, and nothing can put one on a single desktop.
//!
//! This is the acceptance criterion the task states a reason for: *a promise
//! that lived on desktop 1 would be a promise nobody on desktop 3 could see*.
//! Each of the three is a promise alo OS is sold on — law 1's indicator, the one
//! approval a change waits for, and the single road to the agent — so this file
//! holds them not only where they start but after every change a person can make
//! to a row of desktops, and holds every road that could move one to a refusal.
//!
//! Nothing here has been looked at on a screen. What a person sees is the
//! compositor's, and this is what it will be told.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_desktops::{
    Always, Desktops, DisplayId, MOST_DESKTOPS, Name, OnADisplay, Position, Promises, Refused,
    Switch, desktop_words,
};
use alo_dividing::{Area, Point, Size, WindowId};
use alo_strings::Strings;

/// A 1920 by 1080 display at the origin, in logical units.
fn area() -> Area {
    Area::of(Point::at(0, 0), Size::of(1920, 1080)).unwrap()
}

/// The window the compositor calls `id`.
fn window(id: u64) -> WindowId {
    WindowId::from_compositor(id)
}

/// The three promise surfaces of a display.
fn promises() -> Promises {
    Promises::of(window(9001), window(9002), window(9003)).unwrap()
}

/// A display with `how_many` desktops on it, and the person's desktops around
/// it.
fn a_display_with(how_many: usize) -> (Desktops, DisplayId) {
    let laptop = DisplayId::from_compositor(1);
    let mut desktops = Desktops::default();
    let display = desktops.plug_in(laptop, area(), promises()).unwrap();
    while display.how_many() < how_many {
        display.add().unwrap();
    }
    (desktops, laptop)
}

/// That every desktop of this display shows all three promises, in the same
/// windows.
fn all_three_everywhere(display: &OnADisplay) {
    assert!(display.how_many() >= 1);
    for (at, desktop) in display.in_order() {
        let windows = display.windows_on(desktop.id()).unwrap();
        for always in Always::ALL {
            let surface = display.promises().the_surface(always);
            assert_eq!(
                display.promise_on(desktop.id(), always),
                Some(surface),
                "{always:?} is not on desktop {}",
                at.number()
            );
            assert!(
                windows.contains(&surface),
                "{always:?} is not among the windows of desktop {}",
                at.number()
            );
            assert!(display.is_everywhere(surface), "{always:?}");
            assert!(
                !desktop.holds(surface),
                "{always:?} is on desktop {} alone",
                at.number()
            );
        }
    }
}

/// **The egress indicator, the approval surface and the agent overlay are on
/// every desktop at once** — from the first desktop a display has to the
/// sixteenth, and after every change a person can make to the row.
#[test]
fn the_indicator_the_approval_and_the_agent_are_on_every_desktop() {
    let (mut desktops, laptop) = a_display_with(1);
    let display = desktops.on_mut(laptop).unwrap();

    // One desktop: all three.
    all_three_everywhere(display);

    // A desktop added: all three on the new one too, without anybody putting
    // them there.
    let first = display.current();
    let second = display.add().unwrap();
    let third = display.add().unwrap();
    all_three_everywhere(display);

    // A person's windows on the desktops beside them.
    display.put_on(first, window(1)).unwrap();
    display.put_on(second, window(2)).unwrap();
    display.on_every_desktop(window(3)).unwrap();
    all_three_everywhere(display);

    // Named, reordered, switched to, and one removed.
    display
        .call_it(second, Some(Name::given("Post").unwrap()))
        .unwrap();
    display.reorder(third, Position::first()).unwrap();
    display.switch(Switch::Next).unwrap();
    display.remove(second).unwrap();
    all_three_everywhere(display);

    // A person's window closed — the three are not touched by it.
    display.close(window(1)).unwrap();
    display.close(window(3)).unwrap();
    all_three_everywhere(display);

    // And every desktop up to the cap.
    let (mut full, laptop) = a_display_with(MOST_DESKTOPS);
    let display = full.on_mut(laptop).unwrap();
    assert_eq!(display.how_many(), MOST_DESKTOPS);
    all_three_everywhere(display);
}

/// **No promise can be put on one desktop, pinned, or closed.** Each of the
/// three roads that could move a window refuses one, naming which promise it
/// was, and says why in the language the person reads.
#[test]
fn a_promise_cannot_be_put_on_one_desktop_or_closed() {
    let strings = Strings::of(desktop_words().unwrap());
    let (mut desktops, laptop) = a_display_with(3);
    let display = desktops.on_mut(laptop).unwrap();
    let first = display.current();

    for always in Always::ALL {
        let surface = display.promises().the_surface(always);
        for refused in [
            display.put_on(first, surface).unwrap_err(),
            display.on_every_desktop(surface).unwrap_err(),
            display.close(surface).unwrap_err(),
        ] {
            assert_eq!(refused, Refused::APromise(always));
            let said = refused.said(&strings);
            assert!(!said.is_a_bug(), "{always:?}");
            assert!(said.unfilled().is_empty(), "{always:?}: {said}");
            // The sentence does not name the promise; the shell marks it, and
            // what it marks it with is the promise's own name.
            assert!(!always.said(&strings).is_a_bug(), "{always:?}");
            assert_eq!(refused.window(), None);
        }
        // Nothing has moved: it is still on every desktop, in the same window.
        assert!(display.is_everywhere(surface));
    }
    all_three_everywhere(display);
}

/// One surface cannot stand for two promises: dismissing one would dismiss the
/// other, so it is refused where the shell states it.
#[test]
fn one_surface_cannot_be_two_promises() {
    let one = window(9001);
    for (first, second, third) in [
        (one, one, window(9003)),
        (one, window(9002), one),
        (window(9005), one, one),
    ] {
        let refused = Promises::of(first, second, third).unwrap_err();
        assert_eq!(refused.window(), one);
        let (a, b) = refused.both();
        assert_ne!(a, b);
        assert!(refused.to_string().contains("9001"), "{refused}");
    }
    assert!(Promises::of(window(1), window(2), window(3)).is_ok());
}
