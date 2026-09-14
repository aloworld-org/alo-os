//! A panel of rows, laid out: what is in view, what waits, and what is refused.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use crate::WindowControlLabels;
use crate::desktop_testing::{an_appearance, noon_look};
use alo_strings::Direction;

/// How a panel looks at midday, read `reading`.
fn look(reading: Direction) -> ListLook {
    let look = noon_look(&an_appearance(), reading);
    ListLook {
        measure: look.measure(),
        palette: look.palette().unwrap(),
        right_to_left: reading == Direction::RightToLeft,
    }
}

/// `how_many` rows of three cells each.
fn rows(how_many: usize) -> Vec<ListRow> {
    (0..how_many)
        .map(|at| ListRow {
            depth: 0,
            opened: None,
            cells: vec![
                format!("row {at}"),
                format!("{}", at * 1000),
                "x".to_owned(),
            ],
        })
        .collect()
}

/// A region of a 1920×1080 display.
fn region(width: i32, height: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((100, 50).into(), (width, height).into())
}

/// Lay `shows` out in `region` of a 1920×1080 display.
fn laid(
    shows: &ListShows,
    region: Rectangle<i32, Physical>,
    reading: Direction,
) -> Result<ListPicture, RenderError> {
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        shows,
        &mut labels.fonts,
        (1920, 1080),
        region,
        look(reading),
    )
}

/// **Rows are drawn whole or wait for the view**: a list longer than the
/// panel draws the rows that fit, each whole and inside the panel, marks what
/// is out of view with a rail, and a view moved to the end draws the end.
#[test]
fn rows_are_drawn_whole_or_wait_for_the_view() {
    let all = rows(200);
    let shows = ListShows::Rows {
        remarks: vec!["above".to_owned()],
        rows: all.clone(),
        first: 0,
        selected: None,
    };
    let panel = region(900, 600);
    let drawn = laid(&shows, panel, Direction::LeftToRight).unwrap();
    assert!(!drawn.rows.is_empty() && drawn.rows.len() < 200);
    assert!(!drawn.earlier_out_of_view && drawn.later_out_of_view);
    assert!(drawn.rail.is_some());
    for (at, row) in drawn.rows.iter().enumerate() {
        assert_eq!(row.index, at);
        assert_eq!(panel.intersection(row.area), Some(row.area));
        let cells: Vec<&str> = row.cells.iter().map(|cell| cell.text.as_str()).collect();
        assert_eq!(
            cells,
            all[at].cells.iter().map(String::as_str).collect::<Vec<_>>()
        );
        for cell in &row.cells {
            assert_eq!(panel.intersection(cell.area), Some(cell.area));
        }
    }
    assert_eq!(drawn.remarks[0].text, "above");

    // Columns line up: every row's second cell starts at the same place.
    let second = drawn.rows[0].cells[1].area.loc.x;
    assert!(
        drawn
            .rows
            .iter()
            .all(|row| row.cells[1].area.loc.x == second)
    );

    let end = ListShows::Rows {
        remarks: Vec::new(),
        rows: all,
        first: 199,
        selected: None,
    };
    let drawn = laid(&end, panel, Direction::LeftToRight).unwrap();
    assert_eq!(drawn.rows.len(), 1);
    assert_eq!(drawn.rows[0].index, 199);
    assert!(drawn.earlier_out_of_view && !drawn.later_out_of_view);
}

/// **The selected row is always in view, marked in the accent**, however far
/// down it is — and it is the only thing in the panel in that colour.
#[test]
fn the_selected_row_is_always_in_view_and_marked_in_the_accent() {
    let panel = region(900, 400);
    for selected in [0, 5, 120, 199] {
        let shows = ListShows::Rows {
            remarks: Vec::new(),
            rows: rows(200),
            first: 0,
            selected: Some(selected),
        };
        let drawn = laid(&shows, panel, Direction::LeftToRight).unwrap();
        let row = drawn.rows.iter().find(|row| row.index == selected).unwrap();
        let bar = drawn.selection.unwrap();
        assert_eq!(bar.loc.y, row.area.loc.y);
        let accent = look(Direction::LeftToRight).palette.accent;
        let in_accent: Vec<&Solid> = drawn
            .solids
            .iter()
            .filter(|solid| solid.colour == accent)
            .collect();
        assert_eq!(in_accent.len(), 1);
        assert_eq!(in_accent[0].area, bar);
    }
}

/// **A row that opens says whether it is open in shapes**: a closed row has
/// one more bar than an open one, so which it is never rests on colour.
#[test]
fn open_and_closed_are_told_apart_by_shape() {
    let shows = |opened| ListShows::Rows {
        remarks: Vec::new(),
        rows: vec![ListRow {
            depth: 1,
            opened: Some(opened),
            cells: vec!["folder".to_owned()],
        }],
        first: 0,
        selected: None,
    };
    let open = laid(&shows(true), region(600, 300), Direction::LeftToRight).unwrap();
    let closed = laid(&shows(false), region(600, 300), Direction::LeftToRight).unwrap();
    assert!(open.rows[0].mark.unwrap().1);
    assert!(!closed.rows[0].mark.unwrap().1);
    assert_eq!(closed.solids.len(), open.solids.len() + 1);
}

/// **Read right to left, the cells run from the right**: the first cell of a
/// row is on its right, and the rail is on the left.
#[test]
fn read_right_to_left_the_cells_run_from_the_right() {
    let shows = ListShows::Rows {
        remarks: Vec::new(),
        rows: rows(3),
        first: 0,
        selected: None,
    };
    let panel = region(900, 600);
    let ltr = laid(&shows, panel, Direction::LeftToRight).unwrap();
    let rtl = laid(&shows, panel, Direction::RightToLeft).unwrap();
    let (first, second) = (&rtl.rows[0].cells[0], &rtl.rows[0].cells[1]);
    assert!(first.area.loc.x > second.area.loc.x);
    assert!(ltr.rows[0].cells[0].area.loc.x < ltr.rows[0].cells[1].area.loc.x);
}

/// **A refusal is a sentence in a panel, never an empty list**, and a closed
/// panel draws nothing at all.
#[test]
fn a_refusal_is_a_sentence_and_closed_is_nothing() {
    let refused = laid(
        &ListShows::Refusal("could not be read".to_owned()),
        region(900, 600),
        Direction::LeftToRight,
    )
    .unwrap();
    assert!(refused.rows.is_empty());
    assert_eq!(refused.refusal.unwrap().text, "could not be read");
    assert!(refused.panel.is_some());

    let closed = laid(&ListShows::Nothing, region(0, 0), Direction::LeftToRight).unwrap();
    assert!(closed.is_empty());
}

/// **A panel that cannot hold what it must is refused, not cut**: a region too
/// small, a region outside the display, and sentences above the rows that do
/// not fit.
#[test]
fn a_panel_that_cannot_hold_what_it_must_is_refused() {
    let shows = ListShows::Rows {
        remarks: Vec::new(),
        rows: rows(3),
        first: 0,
        selected: None,
    };
    for bad in [
        region(120, 600),
        region(900, 80),
        Rectangle::new((1500, 0).into(), (900, 600).into()),
        Rectangle::new((-10, 0).into(), (900, 600).into()),
    ] {
        assert!(matches!(
            laid(&shows, bad, Direction::LeftToRight),
            Err(RenderError::DesktopScene)
        ));
    }
    let crowded = ListShows::Rows {
        remarks: vec!["a sentence".to_owned(); 40],
        rows: rows(3),
        first: 0,
        selected: None,
    };
    assert!(matches!(
        laid(&crowded, region(900, 300), Direction::LeftToRight),
        Err(RenderError::DesktopScene)
    ));
}
