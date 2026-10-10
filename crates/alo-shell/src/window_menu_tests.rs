//! That the title in the menu can be read, selected and copied whole.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on a row the layout said was there is the failure being reported"
)]

use super::*;
use alo_menus::{Subject, TheAgent};

/// The output this lane's design file is drawn against.
const THE_OUTPUT: (i32, i32) = (1440, 900);

/// A font system with both bundled faces, as the shell builds one.
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

/// A window's menu, on a machine with an agent.
fn a_windows_menu() -> Menu {
    Menu::over(Subject::AWindow, TheAgent::OnThisMachine)
}

const A_LONG_TITLE: &str = "The quarterly accounts for the Lagos office, with every reconciliation and \
     the three notes the auditor asked for";

/// **The whole title is in the menu, not a cut one.**
///
/// This is the clause: *the window menu also exposes the full, selectable
/// title*. The edge cuts; the menu does not.
#[test]
fn the_menu_holds_the_whole_title() {
    let mut fonts = fonts();
    let menu = window_menu(
        &a_windows_menu(),
        A_LONG_TITLE,
        Point::from((100, 100)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window's menu has entries");

    assert_eq!(menu.whole, A_LONG_TITLE);
    assert!(!menu.title.is_empty(), "the title block has no lines");
    let rejoined: String = menu.title.iter().map(|line| line.text.as_str()).collect();
    assert_eq!(
        rejoined, A_LONG_TITLE,
        "the wrapped lines do not rejoin into the title"
    );
}

/// **Every entry `alo_menus` offers gets a row, and every row is pressable.**
///
/// The content is that crate's and the count is its answer, so this asserts the
/// layout lost none of it rather than asserting a number of its own.
#[test]
fn every_entry_the_closed_list_offers_has_a_row() {
    let mut fonts = fonts();
    let offered = a_windows_menu();
    let menu = window_menu(
        &offered,
        "Ledger",
        Point::from((100, 100)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window's menu has entries");

    assert!(
        !offered.entries().is_empty(),
        "the closed list offered nothing"
    );
    assert_eq!(menu.rows.len(), offered.entries().len());
    for (row, does) in menu.rows.iter().zip(offered.entries()) {
        assert_eq!(row.does, *does, "the rows are not in the list's order");
        assert_eq!(
            row.area.size.h, A_ROW_IS_TALL,
            "a row is not the height a finger needs"
        );
        assert_eq!(
            row.area.size.w, menu.panel.size.w,
            "a row is narrower than the menu, so a press beside the word misses"
        );
    }
}

/// **No two rows overlap**, which a person discovers by choosing the wrong one.
#[test]
fn no_two_rows_overlap() {
    let mut fonts = fonts();
    let menu = window_menu(
        &a_windows_menu(),
        "Ledger",
        Point::from((100, 100)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window's menu has entries");

    for (which, row) in menu.rows.iter().enumerate() {
        for other in &menu.rows[which + 1..] {
            assert_eq!(
                row.area.intersection(other.area),
                None,
                "{:?} and {:?} overlap",
                row.does,
                other.does
            );
        }
    }
}

/// **The title block and the entries never overlap**, so dragging a selection
/// cannot choose an action.
///
/// That is the failure a person meets by trying to copy their title and
/// closing their window instead.
#[test]
fn selecting_the_title_cannot_choose_an_action() {
    let mut fonts = fonts();
    let menu = window_menu(
        &a_windows_menu(),
        A_LONG_TITLE,
        Point::from((100, 100)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window's menu has entries");

    assert!(!menu.title.is_empty());
    let lowest = menu
        .title
        .iter()
        .map(|line| line.at.y + A_TITLE_LINE_IS_TALL)
        .max()
        .expect("the title has lines");
    for row in &menu.rows {
        assert!(
            row.area.loc.y >= lowest,
            "{:?} starts at {} which is inside the title block ending at {lowest}",
            row.does,
            row.area.loc.y
        );
    }
}

/// **A selection is one substring of the title**, however many lines it crossed.
///
/// What a person copies is their title, not the shape the panel wrapped it
/// into.
#[test]
fn a_selection_is_one_substring_of_the_title() {
    let mut fonts = fonts();
    let menu = window_menu(
        &a_windows_menu(),
        A_LONG_TITLE,
        Point::from((100, 100)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window's menu has entries");

    assert_eq!(menu.selected(0, A_LONG_TITLE.len()), A_LONG_TITLE);
    assert_eq!(menu.selected(0, 3), "The");
    // Reversed, because a person drags both ways.
    assert_eq!(menu.selected(3, 0), "The");
    // Past the end, because a drag leaves the panel.
    assert_eq!(menu.selected(0, 10_000), A_LONG_TITLE);
    assert_eq!(menu.selected(10_000, 10_001), "");
}

/// **A selection never panics on a multi-byte script**, which slicing on a raw
/// pointer index would.
///
/// The i18n reasoning one layer along from the edge's cut: a byte index from a
/// pointer is not a character boundary, and the scripts it breaks are the ones
/// with the least software already.
#[test]
fn a_selection_across_a_conjunct_does_not_panic() {
    let mut fonts = fonts();
    let devanagari = "नमस्ते दुनिया";
    let menu = window_menu(
        &a_windows_menu(),
        devanagari,
        Point::from((100, 100)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window's menu has entries");

    // Every byte index, including the ones inside a character.
    for from in 0..=devanagari.len() {
        for to in 0..=devanagari.len() {
            let got = menu.selected(from, to);
            assert!(
                devanagari.contains(got),
                "{got:?} is not part of {devanagari:?}"
            );
        }
    }
    assert_eq!(menu.selected(0, devanagari.len()), devanagari);
}

/// **Every grapheme boundary has an x, and they increase.**
///
/// What makes the title selectable with a pointer: a press between two letters
/// has to land between them. Boundaries that did not increase would select
/// backwards somewhere in the middle of a word.
#[test]
fn the_boundaries_are_one_per_grapheme_and_increase() {
    use unicode_segmentation::UnicodeSegmentation;
    let mut fonts = fonts();
    let menu = window_menu(
        &a_windows_menu(),
        A_LONG_TITLE,
        Point::from((100, 100)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window's menu has entries");

    for line in &menu.title {
        let graphemes = line.text.graphemes(true).count();
        assert_eq!(
            line.boundaries.len(),
            graphemes + 1,
            "{:?} has {graphemes} graphemes and {} boundaries",
            line.text,
            line.boundaries.len()
        );
        assert_eq!(line.boundaries[0], line.at.x);
        for pair in line.boundaries.windows(2) {
            assert!(
                pair[1] >= pair[0],
                "{:?} has boundaries going backwards: {pair:?}",
                line.text
            );
        }
    }
}

/// **A pointer in the title block answers with a boundary**, and the far left
/// of the first line is the start of the title.
#[test]
fn a_pointer_in_the_title_finds_where_it_is() {
    let mut fonts = fonts();
    let menu = window_menu(
        &a_windows_menu(),
        A_LONG_TITLE,
        Point::from((100, 100)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window's menu has entries");

    let first = &menu.title[0];
    assert_eq!(
        menu.boundary_at(Point::from((first.at.x, first.at.y + 2))),
        Some(0),
        "the left of the first line is not the start of the title"
    );
    // And a point below every line is in none of them.
    let below = menu.panel.loc.y + menu.panel.size.h + 10;
    assert_eq!(menu.boundary_at(Point::from((first.at.x, below))), None);
}

/// **A menu opened near an edge is moved inside the screen**, as the tooltip is.
#[test]
fn a_menu_near_an_edge_is_moved_inside_the_screen() {
    let mut fonts = fonts();
    let menu = window_menu(
        &a_windows_menu(),
        A_LONG_TITLE,
        Point::from((THE_OUTPUT.0 - 20, THE_OUTPUT.1 - 20)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window's menu has entries");

    assert!(menu.panel.loc.x >= 0);
    assert!(menu.panel.loc.y >= 0);
    assert!(menu.panel.loc.x + menu.panel.size.w <= THE_OUTPUT.0);
    assert!(
        menu.panel.loc.y + menu.panel.size.h <= THE_OUTPUT.1,
        "{:?} runs off the bottom of {}",
        menu.panel,
        THE_OUTPUT.1
    );
}

/// **A window with no title still has a menu.**
///
/// An application that named itself nothing is a real case — `FrameName`
/// carries it — and a menu that refused would take away every action over that
/// window because its title was absent.
#[test]
fn a_window_with_no_title_still_has_its_actions() {
    let mut fonts = fonts();
    let menu = window_menu(
        &a_windows_menu(),
        "",
        Point::from((100, 100)),
        &mut fonts,
        THE_OUTPUT,
    )
    .expect("a window with no title still offers its actions");

    assert!(menu.title.is_empty(), "an empty title produced a line");
    assert!(!menu.rows.is_empty(), "the actions went with the title");
    assert_eq!(menu.rule.size.h, 0, "a rule was drawn under nothing");
    assert_eq!(menu.selected(0, 10), "");
}
