//! The band turned into pixels: the title and the application's name, the
//! controls' layer, the panel above every window, and refusing what will not
//! fit.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, OnTheEdge, edge_of};
use crate::window_edge_paint::Pointing;
use smithay::utils::{Point, Size};

/// Both bundled faces, as the shell loads them.
fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(alo_drawing::fonts::MANROPE.to_vec());
    fonts
        .db_mut()
        .load_font_data(alo_drawing::fonts::INTER.to_vec());
    fonts
}

/// A window with room above it for its band.
fn window() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((40, 100)), Size::from((1056, 466)))
}

/// How the controls look with nothing pointed at, at `how_shown`.
fn controls(how_shown: u16) -> HowTheControlsLook {
    HowTheControlsLook {
        pointing: Pointing::default(),
        open: false,
        how_shown,
        look: WhichLook::of(true),
    }
}

/// The Notes window, named *The next chapter*, of *Notes*.
fn notes(how_shown: u16) -> TheWindow<'static> {
    TheWindow {
        title: Some("The next chapter"),
        application: Some("Notes"),
        look: WhichLook::of(true),
        body: Some(window()),
        laid_over: false,
        controls: controls(how_shown),
    }
}

/// The band of [`window`], revealed.
fn band() -> WindowEdge {
    edge_of(
        window(),
        Decorations::TheShellDraws,
        true,
        Size::from((4000, 3000)),
    )
}

/// **The title, then the application's name** — never the same word twice,
/// and the name alone where the window has no title of its own.
#[test]
fn the_band_says_the_title_then_the_name_and_never_one_word_twice() {
    assert_eq!(
        what_the_band_says(Some("The next chapter"), Some("Notes")),
        (Some("The next chapter"), Some("Notes"))
    );
    assert_eq!(
        what_the_band_says(Some("Notes"), Some("Notes")),
        (None, Some("Notes"))
    );
    assert_eq!(
        what_the_band_says(None, Some("Notes")),
        (None, Some("Notes"))
    );
    assert_eq!(
        what_the_band_says(Some("   "), Some("Notes")),
        (None, Some("Notes"))
    );
    assert_eq!(
        what_the_band_says(Some("Draft"), None),
        (Some("Draft"), None)
    );
}

/// **A title that ends in its application's name says it once**: Dolphin's
/// *Home — Dolphin* reads *Home*, then *Dolphin* quietly — the boot walk of
/// 2026-11-09 showed *Home — Dolphin · Dolphin*.
#[test]
fn a_title_ending_in_its_applications_name_does_not_say_it_twice() {
    assert_eq!(
        what_the_band_says(Some("Home — Dolphin"), Some("Dolphin")),
        (Some("Home"), Some("Dolphin"))
    );
    for title in [
        "Draft – Text Editor",
        "Draft - Text Editor",
        "Draft | Text Editor",
        "Draft · Text Editor",
    ] {
        assert_eq!(
            what_the_band_says(Some(title), Some("Text Editor")),
            (Some("Draft"), Some("Text Editor")),
            "{title}"
        );
    }
    assert_eq!(
        what_the_band_says(Some(" — Dolphin"), Some("Dolphin")),
        (None, Some("Dolphin")),
        "nothing left of the title is no title"
    );
    // **A title that only mentions the application keeps every word**, and
    // so does one whose ending is a different application.
    assert_eq!(
        what_the_band_says(Some("Dolphin tips"), Some("Dolphin")),
        (Some("Dolphin tips"), Some("Dolphin"))
    );
    assert_eq!(
        what_the_band_says(Some("Home — Mozilla Firefox"), Some("Firefox")),
        (Some("Home — Mozilla Firefox"), Some("Firefox"))
    );
    assert_eq!(
        what_the_band_says(Some("Home—Dolphin"), Some("Dolphin")),
        (Some("Home—Dolphin"), Some("Dolphin")),
        "no separator, no ending taken"
    );
}

/// **Both are written on the band**, at rest as much as revealed.
#[test]
fn the_title_and_the_name_are_written_whether_or_not_the_controls_are_shown() {
    for how_shown in [0, crate::the_controls_fade::ALL_THE_WAY] {
        let concealed = edge_of(
            window(),
            Decorations::TheShellDraws,
            how_shown > 0,
            Size::from((4000, 3000)),
        );
        let drawn = EdgePicture::of(
            &concealed,
            OverTheEdge::default(),
            notes(how_shown),
            &mut fonts(),
            (4000, 3000),
            100,
        );
        let words = drawn.as_ref().map_or(0, |it| it.flat().1.len());
        assert_eq!(words, 2, "shown {how_shown}: the title and the name");
        assert_eq!(
            drawn.as_ref().map(EdgePicture::the_whole_title),
            Some("The next chapter"),
            "a reader is given the whole title"
        );
    }
}

/// **The controls' layer follows the fade**: three rasters shown, none gone.
#[test]
fn the_controls_layer_follows_the_fade() {
    let shown = EdgePicture::of(
        &band(),
        OverTheEdge::default(),
        notes(crate::the_controls_fade::ALL_THE_WAY),
        &mut fonts(),
        (4000, 3000),
        100,
    );
    assert_eq!(shown.as_ref().map_or(0, |it| it.over().len()), 3);
    let gone = EdgePicture::of(
        &band(),
        OverTheEdge::default(),
        notes(0),
        &mut fonts(),
        (4000, 3000),
        100,
    );
    assert_eq!(gone.as_ref().map_or(1, |it| it.over().len()), 0);
}

/// **A band that would fall off the display is not drawn at all**: half a band
/// is a control a person reaches for where it is not.
#[test]
fn a_band_that_would_fall_off_the_display_is_not_drawn() {
    let at_the_very_top = Rectangle::new(Point::from((40, 10)), Size::from((600, 400)));
    let edge = edge_of(
        at_the_very_top,
        Decorations::TheShellDraws,
        true,
        Size::from((4000, 3000)),
    );
    assert!(
        EdgePicture::of(
            &edge,
            OverTheEdge::default(),
            notes(1000),
            &mut fonts(),
            (4000, 3000),
            100
        )
        .is_none()
    );
}

/// **A pointed-at control's name is shown**, so an icon-only control always
/// has its word.
#[test]
fn a_pointed_at_control_shows_its_name() {
    let mut window = notes(crate::the_controls_fade::ALL_THE_WAY);
    window.controls.pointing.hovered = Some(OnTheEdge::WindowActions);
    let drawn = EdgePicture::of(
        &band(),
        OverTheEdge {
            menu: None,
            field: None,
            showing: Some(crate::window_edge_tooltip::Because::APointerIsOnIt),
            alo: None,
        },
        window,
        &mut fonts(),
        (4000, 3000),
        100,
    );
    let (solids, words) = drawn
        .as_ref()
        .map_or((0, 0), |it| (it.flat().0.len(), it.flat().1.len()));
    assert_eq!(solids, 2, "the tooltip's line and ground");
    assert_eq!(words, 3, "the title, the name, and the control's name");
}

/// **An open panel goes on the layer above every window**, with its words.
#[test]
fn an_open_panel_is_drawn_above_every_window() {
    let edge = band();
    let menu = crate::window_menu::the_menu_of(
        &edge,
        crate::window_menu::HowTheWindowIs::default(),
        Size::from((4000, 3000)),
    );
    let drawn = menu.as_ref().and_then(|menu| {
        EdgePicture::of(
            &edge,
            OverTheEdge {
                menu: Some((menu, crate::window_menu_paint::PointingAtTheMenu::default())),
                field: None,
                showing: None,
                alo: None,
            },
            notes(crate::the_controls_fade::ALL_THE_WAY),
            &mut fonts(),
            (4000, 3000),
            100,
        )
    });
    assert!(drawn.as_ref().is_some_and(|it| !it.on_top().is_empty()));
    assert!(
        drawn
            .as_ref()
            .is_some_and(|it| it.on_top_words().len() == 5),
        "the heading and one word for each of the four rows: Left, Maximise, Right and Full screen"
    );
}

/// **Out over full screen, the band's ground is drawn over the window**, first
/// and under its controls, opaque in the look's ground — and nothing is drawn
/// under the window, where the application would cover it.
///
/// A booted machine on 2026-11-09 drew the band out over a full-screen Dolphin
/// with its ground under the window, so the title and the three controls lay
/// straight over Dolphin's toolbar.
#[test]
fn a_band_out_over_full_screen_has_its_own_ground_over_the_window() {
    let edge = band();
    let mut over_full_screen = notes(crate::the_controls_fade::ALL_THE_WAY);
    over_full_screen.body = None;
    over_full_screen.laid_over = true;
    let drawn = EdgePicture::of(
        &edge,
        OverTheEdge::default(),
        over_full_screen,
        &mut fonts(),
        (4000, 3000),
        100,
    )
    .expect("a band on the display");
    assert!(drawn.behind().is_empty(), "nothing under the window");
    assert_eq!(drawn.over().len(), 1 + 3, "the ground, then three controls");
    let ground = drawn.over().first().expect("the ground");
    assert!(
        crate::painted::on_the_frame(ground.area)
            .contains(Point::from((edge.strip.loc.x + 1, edge.strip.loc.y + 1)))
            && crate::painted::on_the_frame(ground.area).contains(Point::from((
                edge.strip.loc.x + edge.strip.size.w - 2,
                edge.strip.loc.y + edge.strip.size.h - 2,
            ))),
        "the ground covers the whole band, corners included: {:?}",
        ground.area
    );
    let (w, h) = (ground.area.size.w, ground.area.size.h);
    let middle = usize::try_from(((h / 2) * w + w / 2) * 4).expect("an index");
    let colours = WhichLook::of(true).colours();
    assert_eq!(
        ground.pixels.get(middle..middle + 4),
        Some(&[colours.strip[0], colours.strip[1], colours.strip[2], 255][..]),
        "opaque, in the look's ground"
    );

    let at_rest = EdgePicture::of(
        &edge,
        OverTheEdge::default(),
        notes(crate::the_controls_fade::ALL_THE_WAY),
        &mut fonts(),
        (4000, 3000),
        100,
    )
    .expect("a band on the display");
    assert_eq!(
        at_rest.over().len(),
        3,
        "at rest the ground is under the window"
    );
    assert!(!at_rest.behind().is_empty());
}

/// alo, operating a window it chose: the border alone, working.
fn alo_operating() -> crate::alo_in_a_window::AloHere {
    crate::alo_in_a_window::AloHere {
        which: crate::the_tasks_a_person_opened::WhichTask::numbered(0),
        how: crate::alo_in_a_window::HowAloIsHere::Working,
        status: alo_tasks::words::ALO_IS_WORKING,
        what: "Open hello.txt in VSCodium and type a line into it".to_owned(),
        wears: crate::alo_in_a_window::Wears::TheBorder,
        sending: None,
    }
}

/// How many fully deep-teal pixels a picture's layers over the window hold.
fn teal_over(picture: &EdgePicture) -> usize {
    let teal = crate::window_menu_paint::rgb(alo_appearance::Role::AccentDefault);
    picture
        .over()
        .iter()
        .flat_map(|it| it.pixels.as_chunks::<4>().0)
        .filter(|pixel| pixel[3] == 255 && pixel[..3] == teal)
        .count()
}

/// **A window alo operates near the top of the screen still wears alo's
/// border.** VSCodium on the walk of 2026-11-16 (run 43805568910) sat 30 below
/// the top of a 1280 × 800 display, so its band — 54 tall, above it — would
/// fall off the screen, and the whole picture was refused: the shell said alo
/// was at work in the window, and nothing reached the screen. Asked here
/// through the road the compositor takes for an application's window
/// (`crate::every_windows_edge`: `edge_of`, then this), not of the border's
/// painter alone, which is what let the earlier tests pass.
#[test]
fn a_window_alo_operates_near_the_top_still_wears_the_border() {
    let output = (1280, 800);
    let window = Rectangle::new(Point::from((16, 30)), Size::from((1136, 630)));
    let edge = edge_of(
        window,
        Decorations::TheApplicationDraws(crate::window_edge::ItsOwnButtons::NOT_KNOWN),
        false,
        Size::from(output),
    );
    let here = alo_operating();
    let picture = EdgePicture::of(
        &edge,
        OverTheEdge {
            menu: None,
            field: None,
            showing: None,
            alo: Some(crate::alos_band_in_a_window::AlosBand {
                here: &here,
                card: edge.region.merge(window),
                window,
                stop_pointed_at: false,
            }),
        },
        TheWindow {
            title: Some("hello.txt - VSCodium"),
            application: Some("VSCodium"),
            look: WhichLook::of(true),
            body: Some(window),
            laid_over: false,
            controls: controls(0),
        },
        &mut fonts(),
        output,
        100,
    )
    .expect("the window alo operates was given no picture at all");
    // The border round the window: its sides alone are thousands of pixels.
    assert!(
        teal_over(&picture) > 2 * (1136 + 630),
        "no border was drawn: {} teal pixels",
        teal_over(&picture)
    );
    // And every raster stays on the display.
    let screen = Rectangle::<i32, smithay::utils::Physical>::from_size(output.into());
    for raster in picture.over() {
        let area = crate::painted::on_the_frame(raster.area);
        assert_eq!(
            screen.intersection(area),
            Some(area),
            "{area:?} is drawn off the display"
        );
    }
}

/// **A window alo is not in, whose band would fall off the display, is still
/// not drawn** — the refusal stage 3 relies on is unchanged.
#[test]
fn a_band_off_the_display_with_no_alo_is_still_refused() {
    let window = Rectangle::new(Point::from((16, 30)), Size::from((1136, 630)));
    let edge = edge_of(
        window,
        Decorations::TheApplicationDraws(crate::window_edge::ItsOwnButtons::NOT_KNOWN),
        false,
        Size::from((1280, 800)),
    );
    assert!(
        EdgePicture::of(
            &edge,
            OverTheEdge::default(),
            notes(0),
            &mut fonts(),
            (1280, 800),
            100
        )
        .is_none()
    );
}
