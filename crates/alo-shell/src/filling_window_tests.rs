//! The window of what is filling the disk, opened on folders on a real disk:
//! sizes that open up, each the size the count gave, and what it refuses.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use super::*;
use crate::WindowControlLabels;
use crate::desktop_list::{ListLook, ListPicture, ListShows, picture};
use crate::desktop_raster::filling_shows;
use crate::desktop_testing::{an_appearance, documents, noon_look, words};
use alo_measuring::{Counted, Node};
use alo_strings::Direction;
use smithay::utils::Rectangle;

/// The window drawn in the whole of a tall display.
fn drawn(window: &FillingWindow) -> ListPicture {
    let look = noon_look(&an_appearance(), Direction::LeftToRight);
    let mut labels = WindowControlLabels::new().unwrap();
    let size = (1920, 2160);
    picture(
        &filling_shows(window, &words()),
        &mut labels.fonts,
        size,
        Rectangle::from_size(size.into()),
        ListLook {
            measure: look.measure(),
            palette: look.palette().unwrap(),
            right_to_left: false,
        },
    )
    .unwrap()
}

/// What the window counted.
fn holding_of(window: &FillingWindow) -> &Holding {
    match window.shows() {
        FillingShows::Holding { holding, .. } => holding,
        other => panic!("nothing counted is shown: {other:?}"),
    }
}

/// The child of `node` called `name`.
fn child<'a>(node: &'a Node, name: &str) -> &'a Node {
    node.children
        .iter()
        .find(|child| child.name == name)
        .unwrap()
}

/// Every row drawn, as its cells' text.
fn rows_of(picture: &ListPicture) -> Vec<Vec<String>> {
    picture
        .rows
        .iter()
        .map(|row| row.cells.iter().map(|cell| cell.text.clone()).collect())
        .collect()
}

/// Select the row whose first cell is `name`, moving down from the top.
fn select(window: &mut FillingWindow, name: &str) {
    window.pressed(FillingKey::First);
    for _ in 0..64 {
        let picture = drawn(window);
        let FillingShows::Holding { selected, .. } = window.shows() else {
            panic!("nothing counted");
        };
        let at = picture
            .rows
            .iter()
            .find(|row| row.index == selected)
            .unwrap();
        if at.cells[0].text == name {
            return;
        }
        assert_eq!(window.pressed(FillingKey::Next), FillingPressed::Moved);
    }
    panic!("no row called {name}");
}

/// **What is filling the disk is drawn as sizes that open up.** The folder
/// asked about opens with what is directly inside it in view, each row its
/// name and its size in bytes exactly as the count gave it — the same bytes
/// the filesystem reports for each file — in the order the count holds them.
/// Opening a folder shows what is inside it one level further in; closing it
/// hides that again; and neither changes a size or what was counted.
#[test]
fn what_is_filling_the_disk_is_drawn_as_sizes_that_open_up() {
    let folder = documents();
    let mut window = FillingWindow::closed();
    window.opened(&folder.at);
    let holding = holding_of(&window).clone();
    let tree = &holding.tree;
    assert_eq!(tree.size, 1000 + 2000 + 500 + 40_000 + 12);

    let first = drawn(&window);
    let mut expected = vec![vec![tree.name.clone(), format!("{}", tree.size)]];
    for node in &tree.children {
        expected.push(vec![node.name.clone(), format!("{}", node.size)]);
    }
    assert_eq!(rows_of(&first), expected);
    for (row, node) in first.rows.iter().skip(1).zip(&tree.children) {
        assert_eq!(row.cells[1].text, format!("{}", node.size));
        if node.kind == alo_measuring::Kind::File {
            let on_disk = std::fs::metadata(&node.at).unwrap().len();
            assert_eq!(row.cells[1].text, format!("{on_disk}"), "{}", node.name);
        }
    }
    let marks: Vec<Option<bool>> = first
        .rows
        .iter()
        .map(|row| row.mark.map(|(_, open)| open))
        .collect();
    let letters_at = first
        .rows
        .iter()
        .position(|row| row.cells[0].text == "letters")
        .unwrap();
    assert_eq!(marks[0], Some(true), "the folder asked about is open");
    assert_eq!(marks[letters_at], Some(false), "letters is closed");
    for (row, mark) in first.rows.iter().zip(&marks) {
        if row.cells[0].text.ends_with(".jpg") || row.cells[0].text.ends_with(".txt") {
            assert_eq!(*mark, None, "a file has nothing to open");
        }
    }

    // Opening letters: what is inside it, one level further in.
    select(&mut window, "letters");
    assert_eq!(window.pressed(FillingKey::Open), FillingPressed::Opened);
    assert_eq!(window.pressed(FillingKey::Open), FillingPressed::Nothing);
    let opened = drawn(&window);
    let letters = child(tree, "letters");
    let inside: Vec<Vec<String>> = letters
        .children
        .iter()
        .map(|node| vec![node.name.clone(), format!("{}", node.size)])
        .collect();
    let rows = rows_of(&opened);
    assert_eq!(
        rows[letters_at + 1..letters_at + 1 + inside.len()],
        inside[..]
    );
    assert_eq!(
        opened.rows[letters_at + 1].area.loc.x,
        opened.rows[letters_at].area.loc.x,
        "rows share their box; the cells are set in"
    );
    assert!(
        opened.rows[letters_at + 1].cells[0].area.loc.x
            > opened.rows[letters_at].cells[0].area.loc.x,
        "what is inside a folder is set in one level"
    );
    assert_eq!(child(letters, "old").size, 500);
    assert_eq!(letters.size, 3500);

    // Opening changed nothing that was counted.
    assert_eq!(holding_of(&window), &holding);

    // Closing hides it again, and Enter opens and closes too.
    assert_eq!(window.pressed(FillingKey::Shut), FillingPressed::Shut);
    assert_eq!(rows_of(&drawn(&window)), expected);
    assert_eq!(window.pressed(FillingKey::Toggle), FillingPressed::Opened);
    assert_eq!(window.pressed(FillingKey::Toggle), FillingPressed::Shut);
}

/// **A folder that could not be counted is `alo-measuring`'s refusal in the
/// window, never an empty tree** — one that is not there, and a file named as
/// though it were a folder.
#[test]
fn a_folder_that_cannot_be_counted_is_refused_in_the_window() {
    let strings = words();
    let folder = documents();
    for named in [folder.at.join("nowhere"), folder.at.join("notes.txt")] {
        let mut window = FillingWindow::closed();
        window.opened(&named);
        assert!(window.is_open());
        let FillingShows::Refusal(why) = window.shows() else {
            panic!("{} was not refused: {:?}", named.display(), window.shows());
        };
        assert!(matches!(why, NotMeasured::NotCounted { .. }));
        let said = why.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        match filling_shows(&window, &strings) {
            ListShows::Refusal(text) => assert_eq!(text, said.text()),
            other => panic!("not drawn as a refusal: {other:?}"),
        }
        let picture = drawn(&window);
        assert!(picture.rows.is_empty());
        assert_eq!(picture.refusal.unwrap().text, said.into_text());
        // Keys that move or open do nothing on a refusal; Escape still closes.
        assert_eq!(window.pressed(FillingKey::Next), FillingPressed::Nothing);
        assert_eq!(window.pressed(FillingKey::Open), FillingPressed::Nothing);
        assert_eq!(window.pressed(FillingKey::Close), FillingPressed::Closed);
        assert!(!window.is_open());
    }
}

/// **A size that is not the whole truth says why beside it**, in
/// `alo-measuring`'s words: a folder the machine would not read is a sentence
/// beside its size, never a silent zero.
#[test]
fn a_size_that_is_not_the_whole_truth_says_why_beside_it() {
    use std::os::unix::fs::PermissionsExt;
    let folder = documents();
    let locked = folder.at.join("locked");
    std::fs::create_dir_all(locked.join("inside")).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    let mut window = FillingWindow::closed();
    window.opened(&folder.at);
    let holding = holding_of(&window).clone();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o700)).unwrap();

    let node = child(&holding.tree, "locked");
    let picture = drawn(&window);
    let row = picture
        .rows
        .iter()
        .find(|row| row.cells[0].text == "locked")
        .unwrap();
    match &node.counted {
        Counted::NotRead { .. } => {
            let said = node.counted.said(&words()).unwrap();
            assert_eq!(row.cells.len(), 3);
            assert_eq!(row.cells[2].text, said.text());
        }
        // A test running as root reads the folder anyway, and then the size is
        // the whole truth and nothing is said beside it.
        Counted::Whole => assert_eq!(row.cells.len(), 2),
        other => panic!("{other:?}"),
    }
}

/// **Counting again keeps what was open**, and a selection past the end of a
/// smaller count comes back to the last row rather than pointing at nothing.
#[test]
fn counting_again_keeps_what_was_open() {
    let folder = documents();
    let mut window = FillingWindow::closed();
    window.opened(&folder.at);
    select(&mut window, "letters");
    window.pressed(FillingKey::Open);
    assert_eq!(window.pressed(FillingKey::Last), FillingPressed::Moved);

    std::fs::remove_file(folder.at.join("photo.jpg")).unwrap();
    std::fs::remove_file(folder.at.join("notes.txt")).unwrap();
    assert_eq!(
        window.pressed(FillingKey::CountAgain),
        FillingPressed::Counted
    );
    let holding = holding_of(&window);
    assert_eq!(holding.tree.size, 3500);
    let rows = rows_of(&drawn(&window));
    assert!(
        rows.iter().any(|cells| cells[0] == "to-ada.txt"),
        "letters is still open: {rows:?}"
    );
    let FillingShows::Holding { selected, .. } = window.shows() else {
        panic!("nothing counted");
    };
    assert_eq!(selected, rows.len() - 1);

    // The folder itself gone: the count is refused in the window.
    std::fs::remove_dir_all(&folder.at).unwrap();
    assert_eq!(
        window.pressed(FillingKey::CountAgain),
        FillingPressed::Counted
    );
    assert!(matches!(window.shows(), FillingShows::Refusal(_)));
}

/// **A closed window does nothing with any key**, and counts nothing.
#[test]
fn a_closed_window_does_nothing() {
    let mut window = FillingWindow::closed();
    for key in [
        FillingKey::Next,
        FillingKey::Open,
        FillingKey::Toggle,
        FillingKey::CountAgain,
        FillingKey::Close,
    ] {
        assert_eq!(window.pressed(key), FillingPressed::Nothing);
    }
    assert_eq!(window.shows(), FillingShows::Nothing);
    assert!(drawn(&window).is_empty());
}
