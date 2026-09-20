//! The whole desktop on one display: the dock, the windows over the room it
//! leaves, the colours `alo-appearance` decides, and never terracotta.
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
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        dock,
        look,
        &running_shows(running, &strings),
        &filling_shows(filling, &strings),
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
    let mut colours = solids(&picture.dock.solids);
    for window in [&picture.running, &picture.filling] {
        colours.extend(solids(&window.solids));
        colours.extend(inked(&window.inked));
    }
    colours
}

/// **Not one pixel of the desktop is terracotta**, with both windows open, for
/// every accent a person can choose, light and dark, read either way — because
/// terracotta means the agent and the desktop is not the agent.
#[test]
fn no_pixel_on_the_desktop_is_terracotta() {
    let (running, filling, _folder) = both_open();
    let terracotta = rgb(Token::Terracotta.colour());
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
                    !colours.contains(&terracotta),
                    "{accent:?} at {now:?} {reading:?} painted terracotta"
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
        assert_eq!(picture.dock.solids[0].colour, rgb(dock_ground.colour()));
        for window in [&picture.running, &picture.filling] {
            assert_eq!(window.solids[0].colour, rgb(ink.colour()), "{scheme:?}");
            assert_eq!(window.solids[1].colour, rgb(ground.colour()), "{scheme:?}");
        }
        let accent = picture
            .dock
            .solids
            .iter()
            .find(|solid| solid.area == picture.dock.accent)
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
        for edge in Edge::ALL {
            let mut dock = Dock::shipped();
            dock.set_edge(edge);
            let look = noon_look(&an_appearance(), reading);
            let picture = drawn(&dock, look, &running, &filling, (1920, 1080)).unwrap();
            let band = picture.dock.band;
            let one = picture.running.panel.unwrap();
            let other = picture.filling.panel.unwrap();
            assert!(band.intersection(one).is_none(), "{edge:?}");
            assert!(band.intersection(other).is_none(), "{edge:?}");
            assert!(one.intersection(other).is_none(), "{edge:?}");
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
    assert!(!bare.dock.solids.is_empty(), "the dock is still drawn");
}
