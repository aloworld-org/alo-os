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
            the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
            the_docks_words: crate::desktop_testing::no_words(),
            running: &running_shows(running, &strings),
            filling: &filling_shows(filling, &strings),
            division: crate::desktop_testing::an_undivided_display(),
            offer: crate::desktop_testing::nothing_offered(),
            windows,
            put_aside: crate::desktop_testing::nothing_put_aside(),
            // A fixture that is not about revealing draws the panel, so what it lays out is
            // the rail rather than an empty column.
            panel_is_revealed: true,
            filling_the_screen: false,
            display_scale: 100,
            on_the_dock: &[],
            in_more_apps: &[],
        },
        &mut labels.fonts,
        size,
    )
}

/// A desktop whose Dock holds more than its edge does, with its list open.
///
/// A separate helper for the same reason `drawn_at` is one: every other caller
/// means *a Dock that fits with its list closed*, and giving them all two more
/// arguments would be noise in a hundred call sites to serve one test.
fn drawn_with_an_open_list(
    size: (i32, i32),
    edge: alo_dock::Edge,
    reading: Direction,
    how_many: u64,
) -> Result<DesktopPicture, RenderError> {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).unwrap();
    let mut windows = alo_dock::Windows::none();
    for number in 0..how_many {
        let named = format!("app-{number}");
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number),
            Some(alo_dock::AppId::named(&named).unwrap()),
            &named,
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    let crowded = alo_dock::Holding::nothing().showing(&windows);
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    picture(
        &dock,
        noon_look(&an_appearance(), reading),
        crate::desktop_raster::Shown {
            the_dock_has_open: &crate::what_the_dock_has_open::MORE_APPS,
            the_docks_words: crate::desktop_testing::no_words(),
            running: &running_shows(&RunningWindow::closed(), &strings),
            filling: &filling_shows(&FillingWindow::closed(), &strings),
            division: crate::desktop_testing::an_undivided_display(),
            offer: crate::desktop_testing::nothing_offered(),
            windows: &[],
            // **Windows a person minimised, so the panel has a rail.** Three
            // of one application, which is the shape the owner's ruling of
            // 2026-10-11 separates the two surfaces on: three items here and
            // one icon on the Dock.
            put_aside: crate::desktop_testing::three_windows_put_aside(),
            panel_is_revealed: true,
            filling_the_screen: false,
            display_scale: 100,
            on_the_dock: &crowded,
            in_more_apps: &crowded,
        },
        &mut labels.fonts,
        size,
    )
}

/// The same, on a display of `display_scale` hundredths.
///
/// A separate helper rather than a sixth argument on [`drawn_with`], because
/// every existing caller means *one to one* and giving them all a `100` to pass
/// would be noise in a hundred call sites to serve two tests.
fn drawn_at(
    dock: &Dock,
    look: DesktopLook,
    running: &RunningWindow,
    filling: &FillingWindow,
    size: (i32, i32),
    display_scale: u16,
) -> Result<DesktopPicture, RenderError> {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        dock,
        look,
        crate::desktop_raster::Shown {
            the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
            the_docks_words: crate::desktop_testing::no_words(),
            running: &running_shows(running, &strings),
            filling: &filling_shows(filling, &strings),
            division: crate::desktop_testing::a_display_two_windows_share(),
            offer: crate::desktop_testing::nothing_offered(),
            windows: &[],
            put_aside: crate::desktop_testing::nothing_put_aside(),
            // A fixture that is not about revealing draws the panel, so what it lays out is
            // the rail rather than an empty column.
            panel_is_revealed: true,
            filling_the_screen: false,
            display_scale,
            on_the_dock: &[],
            in_more_apps: &[],
        },
        &mut labels.fonts,
        size,
    )
}

/// **Which space the Dock's band and the panel's column are in, asked of the
/// raster instead of argued from the types.**
///
/// # Why this cannot be answered by reading
///
/// Both are typed `Rectangle<i32, Physical>` and **that label proves nothing**:
/// every one of them is built from `size` and from measures scaled by the
/// person's *text* size, and the marker is satisfied by construction. The
/// module header of `canvas_fixed_controls` is a monument to the cost of
/// guessing — two lanes spent an hour on 2026-09-30 diagnosing a unit mismatch
/// that did not exist, from exactly these types.
///
/// So this asserts the only thing that distinguishes the two possibilities:
/// **a physical rectangle changes with the display's scale and a logical one
/// does not.**
///
/// # What the answer means, either way
///
/// If they are unchanged, they are in the room they were handed, which arrives
/// laid out — and anything compared against them must stay logical too. If they
/// double, they are true framebuffer pixels and a logical floor has to be
/// converted before it is compared.
///
/// The division is drawn beside them and is the control: it *does* take the
/// scale and convert, and `the_displays_scale_reaches_what_is_drawn` holds that
/// it changes. One raster, two behaviours, told apart in one place.
#[test]
fn the_dock_band_and_the_panel_column_do_not_move_with_the_displays_scale() {
    let appearance = an_appearance();
    let look = noon_look(&appearance, Direction::LeftToRight);
    let (running, filling, _folder) = both_open();
    let size = (1920, 1080);

    let at_one = drawn_at(&Dock::shipped(), look, &running, &filling, size, 100).unwrap();
    let one_band = at_one.dock.as_ref().map(|dock| dock.band);
    let one_column = at_one.panel.reserved;

    for scale in [125u16, 150, 200] {
        let dense = drawn_at(&Dock::shipped(), look, &running, &filling, size, scale).unwrap();
        assert_eq!(
            dense.dock.as_ref().map(|dock| dock.band),
            one_band,
            "at {scale} per cent the Dock's band is {:?} and at 100 it is {one_band:?}. \
             A band that moves with the scale is in framebuffer pixels; one that does not \
             is in the laid-out room it was handed. Whichever this is, everything compared \
             against it has to be in the same space.",
            dense.dock.as_ref().map(|dock| dock.band)
        );
        assert_eq!(
            dense.panel.reserved, one_column,
            "at {scale} per cent the panel's reserved column is {:?} and at 100 it is \
             {one_column:?} — and a pointer is tested against this rectangle.",
            dense.panel.reserved
        );
    }
}

/// **A display's scale reaches the surfaces, and this is the test that was
/// missing while it did not.**
///
/// `division_raster::picture` has always taken a scale and converted with it,
/// and its only caller passed the literal `1` — so every surface was drawn as
/// though every display were one to one, and **nothing anywhere could tell**.
/// This asserts the seam rather than the arithmetic: `division_raster`'s own
/// tests hold the multiplication, and this holds that the number arrives.
///
/// At 125, 150 and 200 by the owner's direction of 2026-10-01. The fractional
/// two matter most: the scale was a whole-number factor until 2026-10-02 and
/// both of them floored to 1, so a test at 1 against 2 could not have seen it.
#[test]
fn the_displays_scale_reaches_what_is_drawn() {
    let appearance = an_appearance();
    let look = noon_look(&appearance, Direction::LeftToRight);
    let (running, filling, _folder) = both_open();
    let size = (1920, 1080);

    let at_one = drawn_at(&Dock::shipped(), look, &running, &filling, size, 100).unwrap();
    // **Unwrapped rather than matched.** A division two windows share has
    // shares in it; if it has none the fixture is wrong and that is the failure
    // to report. `an_undivided_display` has none at all — which is why this test
    // does not use it, and why comparing `Option`s here would have let an
    // `assert_eq` pass by holding `None` against `None`.
    let one_to_one = *at_one.division.shares.first().unwrap();

    for scale in [125u16, 150, 200] {
        let scaled = drawn_at(&Dock::shipped(), look, &running, &filling, size, scale).unwrap();
        let share = *scaled.division.shares.first().unwrap();
        assert_ne!(
            share, one_to_one,
            "a display at {scale} per cent drew the same division as one at 100, \
             so the scale did not reach the raster at all"
        );
        assert_eq!(
            share.size.w,
            one_to_one.size.w * i32::from(scale) / 100,
            "width at {scale}"
        );
        assert_eq!(
            share.size.h,
            one_to_one.size.h * i32::from(scale) / 100,
            "height at {scale}"
        );
    }
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
                let look = DesktopLook::of(
                    &appearance,
                    &TurnedOn::nothing(),
                    now,
                    reading,
                    &crate::desktop_testing::a_display(),
                );
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
            &crate::desktop_testing::a_display(),
        );
        let picture = drawn(&dock, look, &running, &filling, size).unwrap();
        let (ground, dock_ground, ink) = match scheme {
            Scheme::Light => (Token::Cream, Token::Porcelain, Token::Navy),
            Scheme::Dark => (Token::Charcoal, Token::Charcoal, Token::Cream),
        };
        // **The Dock's ground is in its surface raster now**, not in a flat
        // rectangle: since 2026-10-11 the band is drawn with a radius and a
        // shadow, which no rectangle can be. Read at the middle of the band,
        // where neither the corner nor the hairline reaches.
        let dock = picture.dock.as_ref().unwrap();
        let surface = dock.surface.as_ref().unwrap();
        // **Just inside the bar's own margin, not its middle.** The middle of a
        // Dock is a badge since 2026-10-11, and a badge is a different colour
        // on purpose — so sampling there read `bg/cool` and called it the
        // Dock's ground.
        let middle = (
            dock.band.loc.x + 3 - surface.area.loc.x,
            dock.band.loc.y + dock.band.size.h / 2 - surface.area.loc.y,
        );
        let at = ((middle.1 as usize) * (surface.area.size.w as usize) + (middle.0 as usize)) * 4;
        let bytes = surface.pixels.get(at..at + 3).unwrap();
        assert_eq!(bytes, rgb(dock_ground.colour()).as_slice(), "{scheme:?}");
        for window in [&picture.running, &picture.filling] {
            assert_eq!(window.solids[0].colour, rgb(ink.colour()), "{scheme:?}");
            assert_eq!(window.solids[1].colour, rgb(ground.colour()), "{scheme:?}");
        }
        // **No accent on the Dock any more.** The owner removed the rule across
        // the bar on 2026-10-11 — *running and focused indicators belong to
        // individual application slots* — so there is nothing here to carry the
        // person's colour, and the assertion that there was has gone with it.
        // What each scheme's grounds are is still checked above.
        let _ = &appearance;
        let _ = now;
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
    // **520 × 480, not 520 × 400.** The shortest side a screen may have moved
    // from 384 to 456 on 2026-10-10, when the dock's measured 76 replaced the
    // proposed 64: the floor is the dock's share times its thickness, so a
    // thicker bar asks for a taller screen. At 400 this pair would now be
    // refused for being too small rather than for the reason this test is
    // about, and the case below would be refused with it — which is the whole
    // distinction the test draws.
    assert!(matches!(
        drawn(&Dock::shipped(), look, &running, &filling, (520, 480)),
        Err(RenderError::DesktopScene)
    ));
    let closed_running = RunningWindow::closed();
    let closed_filling = FillingWindow::closed();
    let bare = drawn(
        &Dock::shipped(),
        look,
        &closed_running,
        &closed_filling,
        (520, 480),
    )
    .unwrap();
    assert!(bare.running.is_empty() && bare.filling.is_empty());
    assert!(
        bare.dock.as_ref().unwrap().surface.is_some(),
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

/// **The whole desktop asks the panel's reveal state, rather than the panel
/// deciding for itself.**
///
/// `crate::panel_raster` has its own test that a concealed panel draws no rail,
/// and that test would go on passing if this file handed it
/// `WhetherRevealed::Revealed` unconditionally. Every other fixture here passes
/// `panel_is_revealed: true`, so without this test nothing in the crate
/// distinguishes *the flag is read* from *the flag exists*.
///
/// The panel holds three windows, because a concealed panel and an empty one
/// draw the same picture.
#[test]
fn concealing_the_panel_reaches_the_draw_path_from_the_whole_desktop() {
    let appearance = an_appearance();
    let look = noon_look(&appearance, alo_strings::Direction::LeftToRight);
    let (running, filling, _folder) = both_open();
    let strings = words();

    let drawn = |revealed: bool| {
        let mut labels = WindowControlLabels::new().unwrap();
        picture(
            &Dock::shipped(),
            look,
            crate::desktop_raster::Shown {
                the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
                the_docks_words: crate::desktop_testing::no_words(),
                running: &running_shows(&running, &strings),
                filling: &filling_shows(&filling, &strings),
                division: crate::desktop_testing::an_undivided_display(),
                offer: crate::desktop_testing::nothing_offered(),
                windows: &[],
                put_aside: crate::desktop_testing::three_windows_put_aside(),
                panel_is_revealed: revealed,
                filling_the_screen: false,
                display_scale: 100,
                on_the_dock: &[],
                in_more_apps: &[],
            },
            &mut labels.fonts,
            (1920, 1080),
        )
    };

    let shown = drawn(true).unwrap();
    let hidden = drawn(false).unwrap();

    assert!(
        shown.panel.rail.size.h > 0,
        "a revealed panel holding three windows draws a rail, so the fixture is capable of \
         telling the two apart"
    );
    assert_eq!(
        hidden.panel.rail.size.h, 0,
        "the desktop drew the panel's rail for a panel nobody has reached for, so the reveal \
         state never reaches crate::panel_raster"
    );
    assert_eq!(
        hidden.panel.reserved, shown.panel.reserved,
        "the desktop gave up the panel's column when it concealed"
    );
}

/// **Every rounded panel the Dock drew reaches the painter.**
///
/// `DockPicture::panels` is where a rasterised panel is put and
/// `DesktopPicture::washed` is the only thing that carries one to a renderer —
/// so a panel in the first and not the second is a panel nobody sees.
///
/// **That is exactly what happened on 2026-10-11.** The overflow list gained a
/// rounded surface, a test asserted it was in `panels` and rounded, and it
/// passed — while `washed` still yielded the Dock's own surface alone and the
/// list drew as bare rows on the canvas. A test about the picture stood in for
/// a test about the painter.
#[test]
fn every_panel_the_dock_drew_reaches_the_painter() {
    use crate::desktop_raster::DesktopPicture;

    let drawn: DesktopPicture = drawn_with_an_open_list(
        (1366, 768),
        alo_dock::Edge::Bottom,
        Direction::LeftToRight,
        60,
    )
    .unwrap();

    assert!(drawn.dock.is_some(), "the premise: a Dock was drawn");
    let dock = drawn.dock.as_ref().unwrap();
    let wanted = usize::from(dock.surface.is_some()) + dock.panels.len();
    assert!(!dock.panels.is_empty(), "the premise: a panel was drawn");
    assert_eq!(
        drawn.washed().count(),
        wanted,
        "{} rasters were drawn and {} reach the painter",
        wanted,
        drawn.washed().count()
    );
}

/// **The Dock's overflow list and the put-aside panel are two surfaces, and
/// the list opens clear of the panel's column.**
///
/// The owner's ruling of 2026-10-11 separates them: the right-hand panel holds
/// *windows a person minimised*, one item per window with its own title and
/// preview, and it *never receives applications merely because the Dock is
/// full*; the Dock's overflow is *a compact, bounded popover anchored to that
/// control, opening inward and clear of the minimised-window panel*, and *must
/// not become a persistent right-hand column*.
///
/// **What was wrong was arithmetic, not either surface's contents.** The list
/// hangs off the bar's far end and is pulled back onto the display; a bottom
/// Dock wide enough to overflow has its far end near the display's right-hand
/// end, which is exactly where the panel's column is. So the popover opened
/// over the panel and the two read as one right-hand column — while the data
/// behind each was correct and separate the whole time.
///
/// Asked of the rectangles **this frame drew**, not of a recomputed column: the
/// panel's own `reserved` is what a pointer is classified against, so a popover
/// that misses a column derived twice would still be over the real one.
#[test]
fn the_overflow_list_opens_clear_of_the_put_aside_panels_column() {
    for edge in [
        alo_dock::Edge::Bottom,
        alo_dock::Edge::Top,
        alo_dock::Edge::Left,
        alo_dock::Edge::Right,
    ] {
        // Both directions, because the panel mirrors to the left when a person
        // reads right to left and the list has to follow it.
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let drawn = drawn_with_an_open_list((1366, 768), edge, reading, 60).unwrap();
            assert!(
                drawn.dock.is_some(),
                "{edge:?}, {reading:?}: the premise: a Dock was drawn"
            );
            let dock = drawn.dock.as_ref().unwrap();
            assert!(
                dock.overflow_list.is_some(),
                "{edge:?}, {reading:?}: the premise: the list was drawn"
            );
            let list = dock.overflow_list.unwrap();
            let column = drawn.panel.reserved;
            let apart = list.loc.x + list.size.w <= column.loc.x
                || column.loc.x + column.size.w <= list.loc.x
                || list.loc.y + list.size.h <= column.loc.y
                || column.loc.y + column.size.h <= list.loc.y;
            assert!(
                apart,
                "{edge:?}, {reading:?}: the overflow list at {list:?} runs into the panel's \
                 column at {column:?}"
            );
            // And not merely apart: the Dock's own gap between them, so the two
            // surfaces read as two. The shadow is allowed to fall across the
            // column — it is soft and belongs to the list — which is why this
            // asks about the surface rather than about `panels[0].area`.
            let gap = i32::try_from(alo_dock::measures::BESIDE_THE_DOCK).unwrap_or(0);
            let between = if column.loc.x <= 0 {
                list.loc.x - (column.loc.x + column.size.w)
            } else {
                column.loc.x - (list.loc.x + list.size.w)
            };
            assert!(
                between >= gap,
                "{edge:?}, {reading:?}: {between} between the list and the panel's column, and \
                 the Dock's gap is {gap}"
            );
        }
    }
}

/// **The list stays a bounded popover rather than a column**, which is the
/// other half of the same ruling: *must not become a persistent right-hand
/// column*.
///
/// Separate from the test above because they fail for different reasons and a
/// reader needs to know which. Moving the list clear of the panel could be done
/// by making it a second full-height column beside the first, and that would
/// satisfy *clear of the panel* while breaking the thing the clause is for.
///
/// So: bounded on both axes — narrower than the display and shorter than it —
/// and anchored to the bar, meaning it touches the band's line rather than
/// floating anywhere on the canvas.
#[test]
fn the_overflow_list_is_a_bounded_popover_anchored_to_the_bar() {
    let (width, height) = (1366, 768);
    for edge in [
        alo_dock::Edge::Bottom,
        alo_dock::Edge::Top,
        alo_dock::Edge::Left,
        alo_dock::Edge::Right,
    ] {
        let drawn =
            drawn_with_an_open_list((width, height), edge, Direction::LeftToRight, 60).unwrap();
        assert!(
            drawn.dock.is_some(),
            "{edge:?}: the premise: a Dock was drawn"
        );
        let dock = drawn.dock.as_ref().unwrap();
        assert!(
            dock.overflow_list.is_some(),
            "{edge:?}: the premise: the list was drawn"
        );
        let list = dock.overflow_list.unwrap();
        assert!(
            list.size.w < width && list.size.h < height,
            "{edge:?}: the list is {:?} on a {width}x{height} display, which is not bounded",
            list.size
        );
        // **Anchored to the control, not merely on the screen.** The gap
        // between the list and the band is a measured distance rather than
        // wherever clamping left them.
        //
        // It was `GAP`, which is 8, and is `BESIDE_THE_DOCK`, which is 12,
        // since 2026-10-11. The two were the same number by accident: `GAP` is
        // the room between things *inside* one surface, and this is the room
        // between two surfaces. The popover's own frame measures it — the
        // popover's lower edge at 848 against a Dock beginning at 860 — and the
        // owner states it in words as well.
        let gap = i32::try_from(alo_dock::measures::BESIDE_THE_DOCK).unwrap_or(0);
        let band = dock.band;
        let touching = match edge {
            alo_dock::Edge::Bottom => band.loc.y - (list.loc.y + list.size.h),
            alo_dock::Edge::Top => list.loc.y - (band.loc.y + band.size.h),
            alo_dock::Edge::Left => list.loc.x - (band.loc.x + band.size.w),
            alo_dock::Edge::Right => band.loc.x - (list.loc.x + list.size.w),
        };
        let column = drawn.panel.reserved;
        // **Where the Dock's own band is inside that column, clearing the panel
        // wins and the list is as near the control as that allows.**
        //
        // Measured 2026-10-11: a right-edge Dock on a left-to-right display
        // puts its band at x 1290..1366 and the panel reserves 1254..1366, so
        // the band is *inside* the column. The two are separate fixed surfaces
        // competing for one edge, which is a question nobody has settled — and
        // nothing this file does can settle it. What it can do is refuse to
        // pretend: on such an edge the list cannot be both flush to the bar and
        // clear of the panel, and *clear of the panel* is the ruling, so the
        // list stops at the column and this asserts it stopped **there** rather
        // than somewhere.
        let band_is_in_the_column =
            band.loc.x < column.loc.x + column.size.w && column.loc.x < band.loc.x + band.size.w;
        if band_is_in_the_column {
            assert_eq!(
                list.loc.x + list.size.w,
                column.loc.x - gap,
                "{edge:?}: the band is inside the panel's column, so the list should stop one \
                 gap short of the column and instead it is at {list:?}"
            );
        } else {
            assert_eq!(
                touching, gap,
                "{edge:?}: the list sits {touching} from the band and the Dock's gap is {gap}, \
                 so it is not anchored to the control"
            );
        }
    }
}

/// **Sixty applications on a 768-high output, measured through the draw.**
///
/// The owner asked for this one by name: *verify the production result with 60
/// applications on a 768-logical-pixel-high output. Include the measured outer
/// height and list-viewport height.* This is that measurement, taken from the
/// rectangle the production raster produced rather than from a layout type
/// asked the same question separately — the two could agree while the drawing
/// used neither.
///
/// It is a test and not a line printed by the probe, because a probe's number
/// is a measurement somebody took once and a test is one that runs again.
///
/// The numbers are asserted absolutely. A relative check — *not taller than the
/// cap* — would pass against a popover of one pixel, and the fault being
/// guarded is a popover that is the wrong size rather than one that is absent.
#[test]
fn sixty_applications_on_a_768_output_stop_at_the_caps() {
    let drawn = drawn_with_an_open_list(
        (1366, 768),
        alo_dock::Edge::Bottom,
        Direction::LeftToRight,
        60,
    )
    .unwrap();
    assert!(drawn.dock.is_some(), "the premise: a Dock was drawn");
    let dock = drawn.dock.as_ref().unwrap();
    assert!(
        dock.overflow_list.is_some(),
        "the premise: More apps was drawn"
    );
    let list = dock.overflow_list.unwrap();

    assert_eq!(
        list.size.h,
        i32::try_from(alo_dock::measures::MORE_APPS_IS_AT_MOST_TALL).unwrap(),
        "the outer height is not the owner's cap: {:?}",
        list.size
    );
    assert_eq!(list.size.h, 536, "and the cap is 536");
    assert_eq!(
        list.size.w,
        i32::try_from(alo_dock::measures::MORE_APPS_IS_WIDE).unwrap(),
        "the width is not the design's",
    );
    assert_eq!(list.size.w, 448, "and the width is 448");

    // **It really is sixty**, or the whole measurement is about something else.
    // The fixture is handed the number, so this says it rather than deriving it
    // from a list that could change underneath: `dock.over` is the overflow
    // tail and would be the wrong one to count, because the overview is not the
    // overflow and does not depend on how many fitted on the bar.
    //
    // A smaller Dock is drawn beside it as the control. Fewer applications must
    // give a shorter popover, or the cap is never being reached and this test
    // proves nothing about it.
    let four = drawn_with_an_open_list(
        (1366, 768),
        alo_dock::Edge::Bottom,
        Direction::LeftToRight,
        4,
    )
    .unwrap();
    let shorter = four
        .dock
        .and_then(|it| it.overflow_list)
        .map(|it| it.size.h);
    assert!(
        shorter.is_some_and(|tall| tall < list.size.h),
        "four applications gave {shorter:?} against sixty's {}, so the cap was never reached",
        list.size.h
    );

    // And it clears the Dock by the measured gap rather than touching it.
    let gap = dock.band.loc.y - (list.loc.y + list.size.h);
    assert_eq!(
        gap,
        i32::try_from(alo_dock::measures::BESIDE_THE_DOCK).unwrap(),
        "the gap to the Dock is not the measured one"
    );
    assert_eq!(gap, 12, "and that gap is 12");
}

/// **An open application in *More apps* carries a visible running mark.**
///
/// Not *a mark was pushed somewhere* — a mark that **reaches the pixels**. The
/// fault this guards is one the code read as correct: the marks were built,
/// the loop was right, and they went into `solids`, which
/// `crate::desktop_paint` paints **before** any panel. So every one of them was
/// drawn and then covered by the popover it belonged to, and a pixel sample
/// under every badge on 2026-10-11 found nothing at all.
///
/// It is asked of the popover's raster rather than of a list, because a list
/// cannot tell the difference between a shape that is drawn and a shape that is
/// drawn and hidden.
#[test]
fn an_open_application_in_more_apps_has_a_mark_in_the_raster() {
    let drawn = drawn_with_an_open_list(
        (1366, 768),
        alo_dock::Edge::Bottom,
        Direction::LeftToRight,
        60,
    )
    .unwrap();
    assert!(drawn.dock.is_some(), "the premise: a Dock was drawn");
    let dock = drawn.dock.as_ref().unwrap();
    assert_eq!(
        dock.panels.len(),
        1,
        "the premise: one panel was rasterised"
    );
    let panel = dock.panels.first().unwrap();
    assert!(
        dock.overflow_list.is_some(),
        "the premise: More apps was drawn"
    );
    let list = dock.overflow_list.unwrap();

    // The ink colour the marks are drawn in, taken from the look rather than
    // written down: a test naming a colour would pass against the wrong one.
    let look = noon_look(&an_appearance(), Direction::LeftToRight);
    let ink = look.palette().unwrap().ink;

    // Count the pixels of that colour inside the popover's raster. A mark is
    // 4 by 3, so one application gives about twelve; sixty give far more. The
    // text is a different colour — `badge_ink` on the badges and `ink` for the
    // names — so this is deliberately a floor rather than an exact count, and
    // the control below is what makes the floor mean something.
    let of_that_colour = |raster: &crate::painted::Washed| {
        raster
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| px[0] == ink[0] && px[1] == ink[1] && px[2] == ink[2] && px[3] > 200)
            .count()
    };
    let marks = of_that_colour(panel);
    assert!(
        marks > 0,
        "no pixel in the popover's raster is the mark's colour, so every running \
         mark was drawn and covered — {} by {} raster",
        panel.area.size.w,
        panel.area.size.h
    );

    // **The control.** A Dock whose applications have no windows draws no
    // marks, so the count above has to fall. Without this, a popover whose
    // whole surface happened to be that colour would pass.
    let closed = drawn_with_an_open_list(
        (1366, 768),
        alo_dock::Edge::Bottom,
        Direction::LeftToRight,
        0,
    );
    if let Ok(closed) = closed
        && let Some(none) = closed.dock.as_ref().and_then(|it| it.panels.first())
    {
        assert!(
            of_that_colour(none) < marks,
            "a Dock with nothing open has as many mark-coloured pixels as one \
             with sixty, so the count is measuring something else: {} against {marks}",
            of_that_colour(none)
        );
    }

    // And the raster really is the popover's, not some other panel's.
    //
    // **The shadow reaches `SPREADS + FALLS`, not `SPREADS`.** It is drawn
    // against a rectangle pushed down by `THE_SHADOW_FALLS` and faded across
    // `THE_SHADOW_SPREADS`, so the furthest it can touch is the two together —
    // and the raster is inflated by that on every side rather than by the fade
    // alone. Measured here rather than assumed: the first form of this
    // assertion used `SPREADS` and was 16 short in each direction.
    let reach = i32::try_from(
        alo_dock::measures::THE_SHADOW_SPREADS + alo_dock::measures::THE_SHADOW_FALLS,
    )
    .unwrap();
    assert_eq!(
        (panel.area.size.w, panel.area.size.h),
        (list.size.w + 2 * reach, list.size.h + 2 * reach),
        "the raster is not the popover's surface plus its shadow's reach"
    );
}
