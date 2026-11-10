//! The band turned into pixels: the title and the application's name, the
//! controls' layer, the panel above every window, and refusing what will not
//! fit.

use super::*;
use crate::window_edge::{Decorations, OnTheEdge, edge_of};
use crate::window_edge_paint::Pointing;
use smithay::utils::{Point, Size};

/// Both bundled faces, as the shell loads them.
fn fonts() -> cosmic_text::FontSystem {
    let mut fonts = cosmic_text::FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
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
            showing: Some(crate::window_edge_tooltip::Because::APointerIsOnIt),
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
                showing: None,
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
