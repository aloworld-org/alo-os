//! The whole desktop on one display: the dock, the windows over the room it
//! leaves, the colours `alo-appearance` decides, and never deep teal.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::WindowControlLabels;
use crate::desktop_look::rgb;
use crate::desktop_testing::{a_second, an_afternoon, an_appearance, documents, noon_look, words};
use crate::painted::{Inked, Solid};
use alo_access::TurnedOn;
use alo_appearance::{Accent, Appearance, Following, Scheme, Shipped, TimeOfDay, Token};

/// Both windows open: what is running on the written afternoon, and what is
/// filling a folder of documents.
fn both_open() -> (RunningWindow, FillingWindow, crate::desktop_testing::Folder) {
    let (earlier, later) = an_afternoon();
    let mut running = RunningWindow::closed();
    running.opened(Ok(earlier), Ok(later), a_second());
    let folder = documents();
    let mut filling = FillingWindow::closed();
    filling.opened(&folder.at);
    (running, filling, folder)
}

/// The desktop drawn with both windows open on a display of `size`.
fn drawn(
    dock: &Dock,
    look: DesktopLook,
    running: &RunningWindow,
    filling: &FillingWindow,
    size: (i32, i32),
) -> Result<DesktopPicture, RenderError> {
    drawn_with(dock, look, running, filling, size, &[])
}

/// The same, with windows on the display for the dock to be asked about.
fn drawn_with(
    dock: &Dock,
    look: DesktopLook,
    running: &RunningWindow,
    filling: &FillingWindow,
    size: (i32, i32),
    windows: &[smithay::utils::Rectangle<i32, smithay::utils::Physical>],
) -> Result<DesktopPicture, RenderError> {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        dock,
        look,
        crate::desktop_raster::Shown {
            running: &running_shows(running, &strings),
            filling: &filling_shows(filling, &strings),
            division: crate::desktop_testing::an_undivided_display(),
            offer: crate::desktop_testing::nothing_offered(),
            windows,
            put_aside: crate::desktop_testing::nothing_put_aside(),
            filling_the_screen: false,
            display_scale: 100,
        },
        &mut labels.fonts,
        size,
    )
}

/// Every colour a picture paints: its shapes and every inked pixel.
fn every_colour(picture: &DesktopPicture) -> Vec<[u8; 3]> {
    let solids = |solids: &[Solid]| solids.iter().map(|solid| solid.colour).collect::<Vec<_>>();
    let inked = |inked: &[Inked]| {
        inked
            .iter()
            .flat_map(|inked| inked.pixels.iter().copied())
            .collect::<Vec<_>>()
    };
    let mut colours = solids(&picture.dock.as_ref().unwrap().solids);
    // The clock, the battery, the network and the volume were painted with the
    // dock and were covered by this walk's promise — not one pixel of the
    // desktop is the agent's colour. ADR 0076 took the status area off the Dock
    // and nothing draws them now, so there is nothing of theirs to check here
    // until the shell plan says where they go.
    // And the division — its rules and the outline a drop would take are on the
    // desktop, so the promise that none of it is the agent's colour covers them.
    colours.extend(solids(&picture.division.solids));
    for window in [&picture.running, &picture.filling] {
        colours.extend(solids(&window.solids));
        colours.extend(inked(&window.inked));
    }
    colours
}

/// **Not one pixel of the desktop is deep teal**, with both windows open, for
/// every accent a person can choose, light and dark, read either way — because
/// deep teal means the agent and the desktop is not the agent.
#[test]
fn no_pixel_on_the_desktop_is_deep_teal() {
    let (running, filling, _folder) = both_open();
    let deep_teal = rgb(Token::DeepTeal.colour());
    let evening = TimeOfDay::checked(20, 0).unwrap();
    for accent in Accent::ALL {
        for now in [TimeOfDay::checked(9, 0).unwrap(), evening] {
            for reading in [Direction::LeftToRight, Direction::RightToLeft] {
                let mut appearance = an_appearance();
                appearance.follow(Following::from(Shipped::the_evening_schedule()));
                appearance.set_accent(accent);
                let look = DesktopLook::of(&appearance, &TurnedOn::nothing(), now, reading);
                let picture =
                    drawn(&Dock::shipped(), look, &running, &filling, (1920, 1080)).unwrap();
                assert!(!picture.running.is_empty() && !picture.filling.is_empty());
                let colours = every_colour(&picture);
                assert!(
                    !colours.contains(&deep_teal),
                    "{accent:?} at {now:?} {reading:?} painted deep teal"
                );
                assert!(colours.contains(&rgb(accent.on(look.scheme()))));
            }
        }
    }
}

/// **The whole desktop turns dark when `alo-appearance` says so**, and only
/// then: on a machine following the evening schedule the dock, both windows and
/// their words are the light palette at nine and the dark one at eight in the
/// evening, and a machine set to stay light stays light at eight.
#[test]
fn the_whole_desktop_turns_dark_when_alo_appearance_says_so() {
    let (running, filling, _folder) = both_open();
    let dock = Dock::shipped();
    let size = (1920, 1080);
    let morning = TimeOfDay::checked(9, 0).unwrap();
    let evening = TimeOfDay::checked(20, 0).unwrap();

    let mut scheduled = Appearance::shipped();
    scheduled.follow(Following::from(Shipped::the_evening_schedule()));
    let mut always_light = Appearance::shipped();
    always_light.follow(Following::from(Scheme::Light));

    for (appearance, now, scheme) in [
        (&scheduled, morning, Scheme::Light),
        (&scheduled, evening, Scheme::Dark),
        (&always_light, evening, Scheme::Light),
    ] {
        assert_eq!(appearance.scheme_at(now), scheme);
        let look = DesktopLook::of(
            appearance,
            &TurnedOn::nothing(),
            now,
            Direction::LeftToRight,
        );
        let picture = drawn(&dock, look, &running, &filling, size).unwrap();
        let (ground, dock_ground, ink) = match scheme {
            Scheme::Light => (Token::Cream, Token::Porcelain, Token::Navy),
            Scheme::Dark => (Token::Charcoal, Token::Charcoal, Token::Cream),
        };
        assert_eq!(
            picture.dock.as_ref().unwrap().solids[0].colour,
            rgb(dock_ground.colour())
        );
        for window in [&picture.running, &picture.filling] {
            assert_eq!(window.solids[0].colour, rgb(ink.colour()), "{scheme:?}");
            assert_eq!(window.solids[1].colour, rgb(ground.colour()), "{scheme:?}");
        }
        let drawn = picture.dock.as_ref().unwrap();
        let accent = drawn
            .solids
            .iter()
            .find(|solid| solid.area == drawn.accent)
            .unwrap();
        assert_eq!(accent.colour, rgb(appearance.accent_at(now)));
    }
}

/// **No window covers the dock**, and two windows open at once share the room
/// the dock leaves without covering each other, the window of what is running
/// on the side a person starts reading from.
#[test]
fn no_window_covers_the_dock_and_two_share_the_room() {
    let (running, filling, _folder) = both_open();
    for reading in [Direction::LeftToRight, Direction::RightToLeft] {
        {
            let dock = Dock::shipped();
            let look = noon_look(&an_appearance(), reading);
            let picture = drawn(&dock, look, &running, &filling, (1920, 1080)).unwrap();
            let band = picture.dock.as_ref().unwrap().band;
            let one = picture.running.panel.unwrap();
            let other = picture.filling.panel.unwrap();
            assert!(band.intersection(one).is_none());
            assert!(band.intersection(other).is_none());
            assert!(one.intersection(other).is_none());
            match reading {
                Direction::LeftToRight => assert!(one.loc.x < other.loc.x),
                Direction::RightToLeft => assert!(one.loc.x > other.loc.x),
            }
        }
    }

    // One window alone has the whole room.
    let closed = FillingWindow::closed();
    let look = noon_look(&an_appearance(), Direction::LeftToRight);
    let alone = drawn(&Dock::shipped(), look, &running, &closed, (1920, 1080)).unwrap();
    assert!(alone.filling.is_empty());
    assert!(alone.running.panel.unwrap().size.w > 1800);
}

/// **A desktop that cannot be drawn whole is refused**: a display too small
/// for the dock, and open windows that cannot hold a row in the room left.
#[test]
fn a_desktop_that_cannot_be_drawn_whole_is_refused() {
    let (running, filling, _folder) = both_open();
    let look = noon_look(&an_appearance(), Direction::LeftToRight);
    assert!(matches!(
        drawn(&Dock::shipped(), look, &running, &filling, (100, 100)),
        Err(RenderError::DesktopScene)
    ));
    assert!(matches!(
        drawn(&Dock::shipped(), look, &running, &filling, (520, 400)),
        Err(RenderError::DesktopScene)
    ));
    let closed_running = RunningWindow::closed();
    let closed_filling = FillingWindow::closed();
    let bare = drawn(
        &Dock::shipped(),
        look,
        &closed_running,
        &closed_filling,
        (520, 400),
    )
    .unwrap();
    assert!(bare.running.is_empty() && bare.filling.is_empty());
    assert!(
        !bare.dock.as_ref().unwrap().solids.is_empty(),
        "the dock is still drawn"
    );
}

/// **The dock gives way to a window over it, and only when the person asked
/// for that.**
///
/// The end of the road `alo-dock` has held since #247 and nothing walked:
/// `Hiding` was the person's choice, `TheRoom` the input and `Dock::showing`
/// the decision, and until now nothing in this repository ever told the dock
/// that a window needed the room. This is the telling, from the drawn band.
#[test]
fn the_dock_gives_way_to_a_window_over_it_and_only_if_asked() {
    use smithay::utils::{Point, Rectangle, Size};

    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let look = noon_look(&an_appearance(), Direction::LeftToRight);
    let size = (1000, 800);

    // Where the dock actually is on this display, read off the picture rather
    // than guessed, so this test cannot drift from the layout.
    let band = drawn(&Dock::shipped(), look, &running, &filling, size)
        .unwrap()
        .dock
        .as_ref()
        .unwrap()
        .band;
    let over = [Rectangle::new(
        Point::from((band.loc.x, band.loc.y + 1)),
        Size::from((band.size.w, band.size.h)),
    )];
    let clear = [Rectangle::new(
        Point::from((0, 0)),
        Size::from((200, band.loc.y - 10)),
    )];

    // The shipped dock never gives way, whatever is over it.
    let shipped = Dock::shipped();
    assert_eq!(shipped.hiding(), alo_dock::Hiding::Never);
    for windows in [&over[..], &clear[..]] {
        let picture = drawn_with(&shipped, look, &running, &filling, size, windows).unwrap();
        assert!(
            picture.dock.is_some(),
            "a dock set to never hide stays whatever is over it"
        );
    }

    // The person's other choice, and the only arm that hides.
    let mut gives_way = Dock::shipped();
    gives_way.set_hiding(alo_dock::Hiding::WhenAWindowNeedsTheRoom);

    let nothing_open = drawn_with(&gives_way, look, &running, &filling, size, &[]).unwrap();
    assert!(
        nothing_open.dock.is_some(),
        "an empty desktop leaves the room free, so the dock stays"
    );

    let beside = drawn_with(&gives_way, look, &running, &filling, size, &clear).unwrap();
    assert!(
        beside.dock.is_some(),
        "a window nowhere near the band does not take the room"
    );

    let covered = drawn_with(&gives_way, look, &running, &filling, size, &over).unwrap();
    assert!(
        covered.dock.is_none(),
        "a window over the band takes the room, so the dock gives way"
    );

    // **And the room the panels get does not move.** This is what stops the
    // whole thing oscillating, so it is asserted rather than assumed: the
    // work area is the same whether the dock is showing or hidden.
    assert_eq!(
        covered.running.size, beside.running.size,
        "the work area must not depend on whether the dock is showing"
    );
    assert_eq!(covered.size, beside.size);
}
