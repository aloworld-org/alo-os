//! What the window panel draws.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, edge_of};
use crate::window_menu::{HowTheWindowIs, the_menu_of};
use smithay::utils::{Point, Size};

/// The Notes window's open panel.
fn a_panel() -> WindowMenu {
    let window = Rectangle::new(Point::from((112, 314)), Size::from((1056, 466)));
    let roomy = Size::from((4000, 3000));
    let edge = edge_of(window, Decorations::TheShellDraws, true, roomy);
    the_menu_of(&edge, HowTheWindowIs::default(), roomy).expect("a panel")
}

/// The fonts every drawing shapes with.
fn fonts() -> FontSystem {
    let mut fonts = FontSystem::new();
    fonts
        .db_mut()
        .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
    fonts
}

/// **The body is drawn first and reaches round the whole panel**, so every
/// choice is drawn on it and its shadow falls outside it.
#[test]
fn the_body_is_drawn_first_and_holds_the_whole_panel() {
    let menu = a_panel();
    let (rasters, _) = the_panel_drawn(
        &menu,
        PointingAtTheMenu::default(),
        &mut fonts(),
        100,
        (4000, 3000),
    );
    let body = rasters.first().expect("a body").area;
    let panel = in_pixels(menu.panel, 100);
    assert!(
        body.contains_rect(panel),
        "{body:?} does not hold {panel:?}"
    );
    assert!(
        body.size.h > panel.size.h,
        "the shadow reaches below the panel"
    );
}

/// **The heading and one word per choice are written**, and nothing else.
#[test]
fn the_heading_and_each_choice_are_written() {
    let menu = a_panel();
    let (_, words) = the_panel_drawn(
        &menu,
        PointingAtTheMenu::default(),
        &mut fonts(),
        100,
        (4000, 3000),
    );
    assert_eq!(words.len(), 1 + menu.rows.len());
}

/// **The choice in hand is the keyboard's before the pointer's**, so a person
/// moving the pointer while using the keys does not lose their place.
#[test]
fn the_keyboard_s_choice_is_the_one_in_hand() {
    let both = PointingAtTheMenu {
        hovered: Some(WhatARowDoes::FullScreen),
        focused: Some(WhatARowDoes::Maximise),
        preview: None,
    };
    assert_eq!(both.in_hand(), Some(WhatARowDoes::Maximise));
    let pointer_only = PointingAtTheMenu {
        hovered: Some(WhatARowDoes::FullScreen),
        focused: None,
        preview: None,
    };
    assert_eq!(pointer_only.in_hand(), Some(WhatARowDoes::FullScreen));
}

/// **The keyboard's choice gets its navy ring, as well as its colour** — one
/// more raster than the same panel with nothing focused.
#[test]
fn the_keyboard_s_choice_is_ringed_and_not_told_by_colour_alone() {
    let menu = a_panel();
    let (plain, _) = the_panel_drawn(
        &menu,
        PointingAtTheMenu::default(),
        &mut fonts(),
        100,
        (4000, 3000),
    );
    let focused = PointingAtTheMenu {
        hovered: None,
        focused: Some(WhatARowDoes::FullScreen),
        preview: None,
    };
    let (ringed, _) = the_panel_drawn(&menu, focused, &mut fonts(), 100, (4000, 3000));
    assert_eq!(ringed.len(), plain.len() + 1);
}

/// **The expand mark reads**, so Full screen is never drawn without it.
#[test]
fn the_full_screen_mark_reads() {
    assert!(the_full_screen_mark().is_some());
}

/// **A placement preview is an outline and its words, drawn under the panel**
/// — `11:84` — and nothing of it is drawn when no Left or Right is in hand.
#[test]
fn a_preview_is_drawn_under_the_panel_with_its_words() {
    let menu = a_panel();
    let outline = Rectangle::new(Point::from((0, 84)), Size::from((640, 700)));
    let (rasters, words) = the_panel_drawn(
        &menu,
        PointingAtTheMenu {
            hovered: Some(WhatARowDoes::Left),
            focused: None,
            preview: Some(ThePreview {
                outline,
                says: alo_access::words::LEFT_HALF_PREVIEW,
            }),
        },
        &mut fonts(),
        100,
        (4000, 3000),
    );
    assert_eq!(
        rasters.first().expect("the outline").area,
        in_pixels(outline, 100),
        "the outline is drawn first, so the panel stays readable over it"
    );
    assert_eq!(words.len(), 2 + menu.rows.len(), "the preview says where");
    let (plain, _) = the_panel_drawn(
        &menu,
        PointingAtTheMenu::default(),
        &mut fonts(),
        100,
        (4000, 3000),
    );
    assert_eq!(
        plain.len() + 2,
        rasters.len(),
        "an outline and a ground for its words, and nothing else"
    );
}

/// Which family and weight drew the first letter of `words`, read back off the
/// glyph rather than off the ask.
fn what_drew(fonts: &mut FontSystem, words: &str, attrs: &Attrs<'_>) -> (String, u16) {
    let mut buffer = cosmic_text::Buffer::new(fonts, Metrics::new(12.0, 16.0));
    buffer.set_size(fonts, None, None);
    buffer.set_text(fonts, words, attrs, cosmic_text::Shaping::Advanced);
    buffer.shape_until_scroll(fonts, false);
    let id = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .map(|glyph| glyph.font_id)
        .next()
        .expect("a glyph");
    let face = fonts.db().face(id).expect("a face");
    (
        face.families.first().expect("a family").0.clone(),
        face.weight.0,
    )
}

/// **Every choice's label is drawn in Manrope, at the weight the design
/// gives it** — read back off the glyph, through the font system the machine
/// draws with rather than this file's own one-face [`fonts`].
///
/// A booted machine on 2026-11-09 drew *Left*, *Maximise* and *Right* in Geist
/// Mono: Medium had no Manrope face, so cosmic-text left the family and took
/// the only face at 500. [`fonts`] holds Manrope alone, which is why every test
/// above passed while the machine drew the wrong face.
#[test]
fn a_choices_label_is_drawn_in_manrope() {
    let mut machines = crate::window_control_label::WindowControlLabels::new()
        .expect("the shell's own fonts")
        .fonts;
    let both = the_menu_of(
        &edge_of(
            Rectangle::new(Point::from((112, 314)), Size::from((1056, 466))),
            Decorations::TheShellDraws,
            true,
            Size::from((4000, 3000)),
        ),
        HowTheWindowIs {
            maximised: true,
            full_screen: true,
        },
        Size::from((4000, 3000)),
    )
    .expect("a panel");
    for row in a_panel().rows.iter().chain(both.rows.iter()) {
        let weight = the_labels_weight(row.does);
        let attrs = crate::painted_text::alos_own_text().weight(weight);
        assert_eq!(
            what_drew(&mut machines, row.does.its_own_word().says(), &attrs),
            ("Manrope".to_owned(), weight.0),
            "{:?}",
            row.does
        );
    }
    assert_eq!(the_labels_weight(WhatARowDoes::Left), Weight::MEDIUM);
    assert_eq!(the_labels_weight(WhatARowDoes::FullScreen), Weight::NORMAL);
}
