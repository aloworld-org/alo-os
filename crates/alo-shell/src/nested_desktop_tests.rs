//! A desktop frame: the dock, its status area holding the egress indicator, and
//! the surfaces above them — or the refusal that stops the whole frame.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::desktop_testing::{a_second, an_afternoon, an_appearance, noon_look};
use crate::egress_status_testing::{asking_a_provider, fetching, noon, words};
use crate::record_testing::{an_afternoon_kept, light};
use crate::{RecordOpened, RecordWindow};
use alo_dock::Dock;
use alo_egress::{EgressPolicy, Indicator};
use alo_indicator::{Drew, Indicating};
use alo_strings::Direction;

/// **The egress indicator sits in the top-left corner, clear of the Dock and
/// of the top controls** — the owner's decisions of 2026-10-31. Read either
/// way, on a laptop and on a large screen. It sat at the far end of the dock
/// until then, and this test asserted that.
#[test]
fn the_egress_indicator_sits_in_the_top_left_corner_clear_of_the_dock() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();

    let mut indicator = Indicator::default();
    let mut quiet = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut quiet), &indicator),
        Drew::Shown
    );
    let asking = indicator
        .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
        .unwrap();
    let errand = indicator
        .beginning(&EgressPolicy::Anywhere, fetching(), noon())
        .unwrap();
    let mut lit = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut lit), &indicator),
        Drew::Shown
    );

    for size in [(1366, 768), (3840, 2160)] {
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            {
                let dock = Dock::shipped();
                let frame = |egress| DesktopFrame {
                    tasks: &[],
                    market: None,
                    what_each_application_draws: &alo_dock::artwork::NOTHING,
                    the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
                    the_docks_words: crate::desktop_testing::no_words(),
                    in_use: &[],
                    notifications: &[],
                    capturing: None,
                    division: crate::desktop_testing::an_undivided_display(),
                    offer: crate::desktop_testing::nothing_offered(),
                    windows: &[],
                    put_aside: crate::desktop_testing::nothing_put_aside(),
                    // A fixture that is not about revealing draws the panel, so what it lays out is
                    // the rail rather than an empty column.
                    panel_is_revealed: true,
                    filling_the_screen: false,
                    display_scale: 100,
                    on_the_dock: &[],
                    in_more_apps: &[],
                    the_keyboard_is_on_the_bar: None,
                    the_docks_name_is_for: None,
                    the_bar: None,
                    dock: &dock,
                    look: noon_look(&an_appearance(), reading),
                    strings: &strings,
                    egress,
                    running: &running,
                    filling: &filling,
                };

                // **Quiet, the circle is there from the first frame**, in the
                // top-left corner.
                let pictures =
                    frame_pictures(frame(&quiet), None, None, &mut labels, size, None).unwrap();
                assert!(pictures.status.rows.is_empty());
                assert!(!pictures.status.is_empty(), "no quiet circle on a desktop frame");
                assert!(pictures.desktop.dock.as_ref().unwrap().surface.is_some());

                let pictures =
                    frame_pictures(frame(&lit), None, None, &mut labels, size, None).unwrap();
                let dock_drawn = pictures.desktop.dock.as_ref().unwrap();
                let rows = &pictures.status.rows;
                assert_eq!(rows.len(), 2, "{reading:?} {size:?}");
                for row in rows {
                    assert!(
                        dock_drawn.band.intersection(row.area).is_none(),
                        "{reading:?}: a line covers the dock"
                    );
                }
                // **The top-left corner, whichever way a person reads** — the
                // owner's decisions of 2026-10-31. It sat at the far end of the
                // Dock until then; this test said so, and was rewritten with
                // the move. 20 from the left and 10 from the top at 100.
                let first = rows.first().unwrap().area;
                assert_eq!((first.loc.x, first.loc.y), (20, 10), "{reading:?} {size:?}");
                // **And the top controls start just to its right.**
                let band = pictures.status.band.unwrap();
                if let Some(top_controls) = pictures.desktop.top_controls {
                    assert!(
                        !top_controls.overlaps(band),
                        "{reading:?} {size:?}: the top controls claim the indicator's corner"
                    );
                    assert!(top_controls.loc.x >= band.loc.x + band.size.w);
                }
            }
        }
    }
    assert!(indicator.ended(asking));
    assert!(indicator.ended(errand));
}

/// **A desktop frame whose indicator was never told is refused whole** — dock,
/// windows and all — because a frame that cannot say whether anything is
/// leaving is not drawn, whatever else it holds.
#[test]
fn a_desktop_frame_whose_indicator_was_never_told_is_refused_whole() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let dock = Dock::shipped();
    let (earlier, later) = an_afternoon();
    let mut running = RunningWindow::closed();
    running.opened(Ok(earlier), Ok(later), a_second());
    let filling = FillingWindow::closed();
    let never_told = EgressStatus::on_an_output();
    let frame = DesktopFrame {
        tasks: &[],
        market: None,
        what_each_application_draws: &alo_dock::artwork::NOTHING,
        the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
        the_docks_words: crate::desktop_testing::no_words(),
        in_use: &[],
        notifications: &[],
        capturing: None,
        division: crate::desktop_testing::an_undivided_display(),
        offer: crate::desktop_testing::nothing_offered(),
        windows: &[],
        put_aside: crate::desktop_testing::nothing_put_aside(),
        // A fixture that is not about revealing draws the panel, so what it lays out is
        // the rail rather than an empty column.
        panel_is_revealed: true,
        filling_the_screen: false,
        display_scale: 100,
        on_the_dock: &[],
        in_more_apps: &[],
        the_keyboard_is_on_the_bar: None,
        the_docks_name_is_for: None,
        the_bar: None,
        dock: &dock,
        look: noon_look(&an_appearance(), Direction::LeftToRight),
        strings: &strings,
        egress: &never_told,
        running: &running,
        filling: &filling,
    };
    assert!(matches!(
        frame_pictures(frame, None, None, &mut labels, (1920, 1080), None),
        Err(RenderError::EgressStatusUnknown)
    ));
}

/// **The desktop is the frame the other surfaces sit in**: the record window
/// travels in the same frame, drawn above the dock.
#[test]
fn the_record_window_sits_in_the_desktop_frame() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let dock = Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut told = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut told), &Indicator::default()),
        Drew::Shown
    );
    let kept = an_afternoon_kept();
    let mut window = RecordWindow::on_an_output();
    assert_eq!(
        window.opened_by_hand(&kept.recounting()),
        RecordOpened::Shown
    );
    let desktop = DesktopFrame {
        tasks: &[],
        market: None,
        what_each_application_draws: &alo_dock::artwork::NOTHING,
        the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
        the_docks_words: crate::desktop_testing::no_words(),
        in_use: &[],
        notifications: &[],
        capturing: None,
        division: crate::desktop_testing::an_undivided_display(),
        offer: crate::desktop_testing::nothing_offered(),
        windows: &[],
        put_aside: crate::desktop_testing::nothing_put_aside(),
        // A fixture that is not about revealing draws the panel, so what it lays out is
        // the rail rather than an empty column.
        panel_is_revealed: true,
        filling_the_screen: false,
        display_scale: 100,
        on_the_dock: &[],
        in_more_apps: &[],
        the_keyboard_is_on_the_bar: None,
        the_docks_name_is_for: None,
        the_bar: None,
        dock: &dock,
        look: noon_look(&an_appearance(), Direction::LeftToRight),
        strings: &strings,
        egress: &told,
        running: &running,
        filling: &filling,
    };
    let record = RecordFrame {
        window: &window,
        strings: &strings,
        look: light(),
        clock: crate::record_testing::a_clock(),
    };
    let pictures =
        frame_pictures(desktop, Some(record), None, &mut labels, (1920, 1080), None).unwrap();
    assert!(!pictures.record.as_ref().unwrap().entries.is_empty());
    assert!(pictures.approval.is_none());
    assert!(pictures.desktop.dock.as_ref().unwrap().surface.is_some());
}

/// **A frame carries the panel's reveal state through to the picture**, rather
/// than this file deciding it.
///
/// Whether the panel is revealed is the compositor's state, held in
/// `alo_dock::revealing` and supplied by whoever builds the frame. A frame that
/// dropped it would be a desktop whose panel is always in the way, and every
/// other fixture in this file passes `panel_is_revealed: true`, so without this
/// test substituting `true` here passes the whole suite — it was measured doing
/// exactly that.
///
/// The panel holds windows, because a concealed panel and an empty one draw the
/// same picture.
#[test]
fn the_frame_carries_whether_the_panel_is_revealed() {
    let strings = words();
    let dock = Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut told = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut told), &Indicator::default()),
        Drew::Shown
    );

    let drawn = |revealed: bool| {
        let mut labels = WindowControlLabels::new().unwrap();
        frame_pictures(
            DesktopFrame {
                tasks: &[],
                market: None,
                what_each_application_draws: &alo_dock::artwork::NOTHING,
                the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
                the_docks_words: crate::desktop_testing::no_words(),
                in_use: &[],
                notifications: &[],
                capturing: None,
                division: crate::desktop_testing::an_undivided_display(),
                offer: crate::desktop_testing::nothing_offered(),
                windows: &[],
                put_aside: crate::desktop_testing::three_windows_put_aside(),
                panel_is_revealed: revealed,
                filling_the_screen: false,
                display_scale: 100,
                on_the_dock: &[],
                in_more_apps: &[],
                the_keyboard_is_on_the_bar: None,
                the_docks_name_is_for: None,
                the_bar: None,
                dock: &dock,
                look: noon_look(&an_appearance(), Direction::LeftToRight),
                strings: &strings,
                egress: &told,
                running: &running,
                filling: &filling,
            },
            None,
            None,
            &mut labels,
            (1920, 1080),
            None,
        )
    };

    let shown = drawn(true).unwrap();
    let hidden = drawn(false).unwrap();

    assert!(
        shown.desktop.panel.rail.size.h > 0,
        "a revealed panel holding windows draws a rail, so the fixture can tell the two apart"
    );
    assert_eq!(
        hidden.desktop.panel.rail.size.h, 0,
        "the frame drew the rail for a panel nobody has reached for, so what the frame carries \
         never reaches the draw path"
    );
}

/// **A frame carrying an open market gets a market in its picture.**
///
/// This function reads nineteen fields off the frame and, until 2026-10-19,
/// wrote the twentieth as a literal `None` — so `Server::market` was carried
/// to the frame by `crate::what_the_server_tells_a_frame` and thrown away by
/// the one function that builds the picture for **both** roads.
///
/// Every fixture above passes `market: None`, which is why the whole suite
/// stayed green: there was no frame anywhere that carried one. That is the
/// same shape as `the_frame_carries_whether_the_panel_is_revealed` above, and
/// its note — *without this test substituting a constant here passes the whole
/// suite* — was measured true a second time.
///
/// What a person met, measured on a booted machine that day with a control
/// each way: a press on the Applications entry took the button count from 3 to
/// 4, the screen stayed at 9323 bytes, and a further key left the key count at
/// 6 — the window had opened, drawn nothing, and taken the keyboard.
#[test]
fn the_frame_carries_the_market_a_person_opened() {
    use smithay::utils::{Point, Rectangle, Size};

    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let dock = Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut told = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut told), &Indicator::default()),
        Drew::Shown
    );

    let mut window = crate::MarketWindow::closed();
    window.opened(alo_software::Catalogue::decided().unwrap(), Vec::new());
    assert!(window.is_open(), "the premise: the window is open");
    let showing = window.shows();
    let the_markets_words = crate::TheMarketsWords::said(&strings);
    let look = noon_look(&an_appearance(), Direction::LeftToRight);
    let placed = crate::where_the_market_opens::where_the_market_opens(
        crate::where_a_task_window_opens::as_a_frame_draws_it(Rectangle::new(
            Point::from((0, 0)),
            Size::from((1920, 1080)),
        )),
        &showing,
        look.scale(),
        crate::display_scale::DisplayScale::ONE,
    );
    assert!(
        placed.is_some(),
        "the premise: the market has somewhere to be drawn"
    );
    let at = placed.unwrap();

    let desktop = DesktopFrame {
        tasks: &[],
        market: Some(crate::market_raster::AMarketOnTheDisplay {
            at,
            showing,
            words: &the_markets_words,
        }),
        what_each_application_draws: &alo_dock::artwork::NOTHING,
        the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
        the_docks_words: crate::desktop_testing::no_words(),
        in_use: &[],
        notifications: &[],
        capturing: None,
        division: crate::desktop_testing::an_undivided_display(),
        offer: crate::desktop_testing::nothing_offered(),
        windows: &[],
        put_aside: crate::desktop_testing::nothing_put_aside(),
        panel_is_revealed: true,
        filling_the_screen: false,
        display_scale: 100,
        on_the_dock: &[],
        in_more_apps: &[],
        the_keyboard_is_on_the_bar: None,
        the_docks_name_is_for: None,
        the_bar: None,
        dock: &dock,
        look,
        strings: &strings,
        egress: &told,
        running: &running,
        filling: &filling,
    };

    let pictures = frame_pictures(desktop, None, None, &mut labels, (1920, 1080), None).unwrap();
    assert!(
        pictures.desktop.market.is_some(),
        "a person had the market open and the frame's picture holds no market, \
         so nothing will be painted and -- because an open window takes the keys -- \
         they lose the keyboard with nothing on screen to say why"
    );
}

/// **The frame carries where the keyboard is on the bar**, through the one
/// function that builds a picture for both roads.
///
/// # Why this seam and not the raster's
///
/// `crate::dock_raster`'s own tests hold that the ring is drawn when it is
/// asked for. They cannot see the question being asked: `frame_pictures` reads
/// the frame's fields into `desktop_raster::Shown`, and **a literal written
/// there instead of the frame's value is invisible to every test below it.**
/// That is not a hypothetical — `market: None` was written in this block, the
/// market's own raster tests stayed green, and a person pressing the
/// Applications entry lost their keyboard to a window that was never drawn.
///
/// So this asserts the only thing that distinguishes the two: a Dock drawn
/// with the keyboard on its first slot is not pixel-for-pixel a Dock drawn with
/// the keyboard elsewhere.
#[test]
fn the_frame_carries_the_slot_the_keyboard_is_on() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let dock = Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut told = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut told), &Indicator::default()),
        Drew::Shown
    );

    // **`mut`, because it lends the labels to each draw.** The labels are the
    // window controls' words, said once and reused — which is the point of
    // them, and the reason this is a closure rather than three copies.
    let mut painted = |the_keyboard_is_on_the_bar| {
        let frame = DesktopFrame {
            tasks: &[],
            market: None,
            what_each_application_draws: &alo_dock::artwork::NOTHING,
            the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
            the_docks_words: crate::desktop_testing::no_words(),
            in_use: &[],
            notifications: &[],
            capturing: None,
            division: crate::desktop_testing::an_undivided_display(),
            offer: crate::desktop_testing::nothing_offered(),
            windows: &[],
            put_aside: crate::desktop_testing::nothing_put_aside(),
            panel_is_revealed: true,
            filling_the_screen: false,
            display_scale: 100,
            on_the_dock: &[],
            in_more_apps: &[],
            the_bar: None,
            the_keyboard_is_on_the_bar,
            the_docks_name_is_for: None,
            dock: &dock,
            look: noon_look(&an_appearance(), Direction::LeftToRight),
            strings: &strings,
            egress: &told,
            running: &running,
            filling: &filling,
        };
        // **`unwrap` and not `expect`**, which is this file's own convention:
        // its header allows the first and refuses the second, because in a test
        // a panic on an unexpected `None` is the failure being reported. A told
        // indicator and an undivided display draw a desktop, and that desktop's
        // bar is drawn on every frame.
        let pictures = frame_pictures(frame, None, None, &mut labels, (1920, 1080), None).unwrap();
        pictures
            .desktop
            .dock
            .as_ref()
            .unwrap()
            .surface
            .as_ref()
            .unwrap()
            .pixels
            .clone()
    };

    let quiet = painted(None);
    let reached = painted(Some(0));
    assert_ne!(
        reached, quiet,
        "the frame said the keyboard was on the bar's first slot and the picture is identical \
         to one where it is not, so the frame's answer never reached the Dock's drawing — \
         which is a person pressing F6 and seeing nothing happen"
    );
    // **The control.** Two draws of the same frame must agree, or the
    // comparison above would pass on a picture that merely differs twice.
    assert_eq!(
        painted(None),
        quiet,
        "two pictures of the same frame differ, so this test cannot say anything"
    );
}

/// **Every picture a frame holds reaches the layers a display paints.**
///
/// Until 2026-10-27 the direct backend built its layers by hand and named three
/// of them: alo's own terminal, the camera-and-microphone indicator and the
/// notifications were laid out every frame and dropped. On a booted machine
/// `Ctrl+Alt+T` opened a terminal nobody could see, and nothing failed, so
/// nothing said so. Both backends now take their layers from
/// [`DesktopPictures::layers`], and this holds that one place to the whole
/// list.
#[test]
fn every_picture_a_frame_holds_reaches_the_layers_a_display_paints() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let dock = Dock::shipped();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut told = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut told), &Indicator::default()),
        Drew::Shown
    );
    let frame = DesktopFrame {
        tasks: &[],
        market: None,
        what_each_application_draws: &alo_dock::artwork::NOTHING,
        the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
        the_docks_words: crate::desktop_testing::no_words(),
        in_use: &[],
        notifications: &[],
        capturing: None,
        division: crate::desktop_testing::an_undivided_display(),
        offer: crate::desktop_testing::nothing_offered(),
        windows: &[],
        put_aside: crate::desktop_testing::nothing_put_aside(),
        panel_is_revealed: true,
        filling_the_screen: false,
        display_scale: 100,
        on_the_dock: &[],
        in_more_apps: &[],
        the_keyboard_is_on_the_bar: None,
        the_docks_name_is_for: None,
        the_bar: None,
        dock: &dock,
        look: noon_look(&an_appearance(), Direction::LeftToRight),
        strings: &strings,
        egress: &told,
        running: &running,
        filling: &filling,
    };
    let size = (1920, 1080);

    // A terminal drawn by the production drawing, from the production fonts.
    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_terminal::words::declare_into(&mut vocabulary).unwrap();
    let the_terminals_words = alo_strings::Strings::of(vocabulary);
    let screen = alo_terminal::TerminalOutput::of(alo_terminal::HowBig::of(80, 24, 100).unwrap());
    let terminal = crate::the_terminals_cells_are_drawn::picture(
        &mut labels.fonts,
        crate::the_terminals_cells_are_drawn::Shows {
            screen: &screen,
            what_alo_said: &alo_terminal::WhatAloSaid::nothing(),
            a_test_result: None,
            a_benchmark: None,
            a_diff: None,
            the_strip: crate::the_terminals_strip::WhatTheStripSays::ONLY_WHERE_IT_RUNS,
        },
        size,
        1.0,
        alo_appearance::Scheme::Light,
        &the_terminals_words,
        None,
    )
    .unwrap();

    let with_one = frame_pictures(frame, None, None, &mut labels, size, Some(terminal)).unwrap();
    let layers = with_one.layers(&[]);
    assert!(
        layers.terminal.is_some(),
        "a frame holding alo's own terminal handed the painter no terminal, which is a \
         person pressing Ctrl+Alt+T and seeing nothing"
    );
    assert!(layers.desktop.is_some(), "the desktop was not handed on");
    assert!(layers.status.is_some(), "what is leaving was not handed on");
    assert!(
        layers.in_use.is_some(),
        "what is watching or listening was not handed on"
    );
    assert!(
        layers.notifications.is_some(),
        "the notifications were not handed on"
    );

    // **The control**: with no terminal open, none is handed on, so the
    // assertion above is about the picture and not about the field always
    // being filled.
    let without = frame_pictures(frame, None, None, &mut labels, size, None).unwrap();
    assert!(without.layers(&[]).terminal.is_none());
}

/// **The display's scale reaches the Dock and the indicator on it, through the
/// frame**, which is the road `crate::direct_desktop` takes for each display
/// with that display's own scale.
///
/// Two displays: a one-to-one 1920 × 1080 and a two-times 3840 × 2160. The
/// second's Dock is exactly twice the first's, and its indicator rows are
/// twice as tall and still clear of it — so the frame carried the scale to
/// both, rather than to the Dock alone and leaving the rows sitting on it.
#[test]
fn a_two_times_display_draws_its_dock_and_its_indicator_twice_the_size() {
    let strings = words();
    let mut labels = WindowControlLabels::new().unwrap();
    let running = RunningWindow::closed();
    let filling = FillingWindow::closed();
    let mut indicator = Indicator::default();
    drop(indicator.beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon()));
    let mut lit = EgressStatus::on_an_output();
    assert_eq!(
        Indicating::nowhere().show(Some(&mut lit), &indicator),
        Drew::Shown
    );
    let dock = Dock::shipped();
    let mut at = |size: (i32, i32), display_scale: u16| {
        let frame = DesktopFrame {
            tasks: &[],
            market: None,
            what_each_application_draws: &alo_dock::artwork::NOTHING,
            the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
            the_docks_words: crate::desktop_testing::no_words(),
            in_use: &[],
            notifications: &[],
            capturing: None,
            division: crate::desktop_testing::an_undivided_display(),
            offer: crate::desktop_testing::nothing_offered(),
            windows: &[],
            put_aside: crate::desktop_testing::nothing_put_aside(),
            panel_is_revealed: false,
            filling_the_screen: false,
            display_scale,
            on_the_dock: &[],
            in_more_apps: &[],
            the_keyboard_is_on_the_bar: None,
            the_docks_name_is_for: None,
            the_bar: None,
            dock: &dock,
            look: noon_look(&an_appearance(), Direction::LeftToRight),
            strings: &strings,
            egress: &lit,
            running: &running,
            filling: &filling,
        };
        frame_pictures(frame, None, None, &mut labels, size, None).unwrap()
    };
    let one = at((1920, 1080), 100);
    let two = at((3840, 2160), 200);
    let (band_one, band_two) = (
        one.desktop.dock.as_ref().unwrap().band,
        two.desktop.dock.as_ref().unwrap().band,
    );
    assert_eq!(band_two.size.h, 2 * band_one.size.h, "the Dock's thickness");
    assert_eq!(band_two.size.w, 2 * band_one.size.w, "the Dock's length");
    assert_eq!(band_two.loc.y, 2 * band_one.loc.y, "where the Dock sits");
    let (row_one, row_two) = (
        one.status.rows.first().unwrap().area,
        two.status.rows.first().unwrap().area,
    );
    assert_eq!(row_two.size.h, 2 * row_one.size.h, "the indicator's row");
    assert_eq!(row_two.loc.y, 2 * row_one.loc.y, "where the row sits");
    assert!(
        row_two.loc.y + row_two.size.h <= band_two.loc.y,
        "the row {row_two:?} runs into the Dock {band_two:?}"
    );
}
