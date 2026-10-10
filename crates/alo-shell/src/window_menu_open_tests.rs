//! That a person can open the menu, move through it, close it and copy from it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a menu the state said was open is the failure being reported"
)]

use super::*;

/// A title long enough that reading it is the reason to open the menu.
const A_LONG_TITLE: &str = "The quarterly accounts for the Lagos office";

/// A menu open for a window, by one of the three roads.
///
/// Takes no real `WlSurface` — a surface needs a client on a socket — so the
/// window's identity is exercised by `the_window_it_is_open_for_is_the_one_asked_for`
/// in the shell's own integration tests and the state is exercised here.
fn opened(road: OpenedBy, rows: usize) -> TheWindowMenu {
    let mut menu = TheWindowMenu::closed();
    menu.opened_without_a_surface(A_LONG_TITLE, Point::from((100, 100)), road, rows);
    menu
}

/// **The keyboard road arrives with a row focused and the pointer road does
/// not.**
///
/// The one difference between them, and it is the difference between a menu a
/// keyboard can use immediately and one that needs a Down press first. The
/// other way round, a pointer road that focused row zero would light a row
/// under a pointer that is not on it.
#[test]
fn the_keyboard_opens_with_a_row_and_the_pointer_does_not() {
    assert_eq!(
        opened(OpenedBy::TheKeyboard, 5).focused(),
        Some(0),
        "a person who pressed a key has nothing to hover with"
    );
    assert_eq!(opened(OpenedBy::APointer, 5).focused(), None);
    assert_eq!(opened(OpenedBy::ATouch, 5).focused(), None);
}

/// **A menu with no rows focuses nothing, by any road.**
///
/// `alo_menus` decides what a window offers and may offer nothing — on a
/// machine with no agent one entry is absent entirely — so a focus of row zero
/// would be a keyboard on a row that is not there.
#[test]
fn a_menu_with_no_rows_focuses_nothing() {
    for road in [OpenedBy::TheKeyboard, OpenedBy::APointer, OpenedBy::ATouch] {
        assert_eq!(opened(road, 0).focused(), None, "{road:?}");
    }
}

/// **Down and Up wrap**, which is what every menu a person has used does.
///
/// Without wrapping the last row is `rows - 1` presses from the first instead
/// of one, which a person discovers by holding Down and watching it stop.
#[test]
fn the_keyboard_wraps_at_both_ends() {
    let mut menu = opened(OpenedBy::TheKeyboard, 3);
    assert_eq!(menu.focused(), Some(0));
    menu.the_keyboard_went_down(3);
    assert_eq!(menu.focused(), Some(1));
    menu.the_keyboard_went_down(3);
    assert_eq!(menu.focused(), Some(2));
    menu.the_keyboard_went_down(3);
    assert_eq!(menu.focused(), Some(0), "Down at the bottom did not wrap");

    menu.the_keyboard_went_up(3);
    assert_eq!(menu.focused(), Some(2), "Up at the top did not wrap");
}

/// **A pointer road reached by keyboard starts where the pointer was.**
///
/// What makes the two roads one menu rather than two: a person who opened it
/// with a click, moved the pointer down two rows and then reached for the
/// keyboard carries on from there.
#[test]
fn the_keyboard_carries_on_from_where_the_pointer_was() {
    let mut menu = opened(OpenedBy::APointer, 4);
    assert_eq!(menu.focused(), None);
    menu.the_pointer_is_on(2);
    assert_eq!(menu.focused(), Some(2));
    menu.the_keyboard_went_down(4);
    assert_eq!(
        menu.focused(),
        Some(3),
        "the keyboard restarted rather than carrying on"
    );
}

/// **Nothing moves in a menu that is not open.**
///
/// A key arriving after a dismissal is an ordinary case — a person presses
/// Escape and Down in quick succession — and a focus set on a closed menu would
/// be a row lit with no menu around it.
#[test]
fn a_closed_menu_takes_no_keys_and_no_hover() {
    let mut menu = TheWindowMenu::closed();
    menu.the_keyboard_went_down(5);
    menu.the_keyboard_went_up(5);
    menu.the_pointer_is_on(2);
    assert_eq!(menu.focused(), None);
    assert!(menu.open_for().is_none());
}

/// **Dismissing forgets the window, the focus and the selection.**
///
/// All three go, because a menu that reopened with the previous window's
/// selection would offer to copy a title nobody is looking at.
#[test]
fn dismissing_forgets_everything_it_was_holding() {
    let mut menu = opened(OpenedBy::TheKeyboard, 3);
    menu.select_the_whole_title();
    assert!(!menu.selection().is_empty());

    menu.dismissed();

    assert_eq!(menu.focused(), None);
    assert!(menu.selection().is_empty());
    assert_eq!(menu.selected(), "");
    assert!(
        menu.copy_the_title().is_none(),
        "a closed menu offered a copy"
    );
}

/// **A selection is one substring, both ways round, and clamped.**
#[test]
fn a_selection_reads_the_title_it_was_taken_from() {
    let mut menu = opened(OpenedBy::APointer, 3);
    menu.select_the_whole_title();
    assert_eq!(menu.selected(), A_LONG_TITLE);

    menu.selecting_from(4);
    menu.selecting_to(13);
    assert_eq!(menu.selected(), "quarterly");

    // Dragged backwards, which a person does.
    menu.selecting_from(13);
    menu.selecting_to(4);
    assert_eq!(menu.selected(), "quarterly");

    // Dragged off the end, which a person also does.
    menu.selecting_from(0);
    menu.selecting_to(10_000);
    assert_eq!(menu.selected(), A_LONG_TITLE);
}

/// **A selection across a conjunct never panics.**
///
/// Every byte offset, including the ones inside a character. A pointer gives
/// byte offsets and a naive slice would panic on the scripts with the least
/// software already.
#[test]
fn every_offset_into_a_devanagari_title_is_safe() {
    let devanagari = "नमस्ते दुनिया";
    let mut menu = TheWindowMenu::closed();
    menu.opened_without_a_surface(devanagari, Point::from((0, 0)), OpenedBy::APointer, 3);

    for from in 0..=devanagari.len() {
        for to in 0..=devanagari.len() {
            menu.selecting_from(from);
            menu.selecting_to(to);
            let got = menu.selected();
            assert!(
                devanagari.contains(got),
                "{got:?} is not part of {devanagari:?}"
            );
        }
    }
}

/// **Copying with nothing selected copies the whole title.**
///
/// A person who opened the menu and chose *copy* without dragging meant the
/// title. Copying nothing would be a command that silently did nothing, which
/// is worse than one that refused.
#[test]
fn copying_without_a_selection_copies_the_whole_title() {
    let menu = opened(OpenedBy::TheKeyboard, 3);
    let (offer, mut source) = menu.copy_the_title().expect("an open menu offers a copy");

    assert!(offer.offers(&Kind::text()), "the offer is not text");
    let given = source
        .give(&Kind::text())
        .expect("the title hands itself over");
    assert_eq!(
        String::from_utf8(given).expect("a title is text"),
        A_LONG_TITLE
    );
}

/// **And copying with a selection copies that.**
#[test]
fn copying_with_a_selection_copies_the_selection() {
    let mut menu = opened(OpenedBy::APointer, 3);
    menu.selecting_from(4);
    menu.selecting_to(13);

    let (_offer, mut source) = menu.copy_the_title().expect("an open menu offers a copy");
    let given = source
        .give(&Kind::text())
        .expect("the selection hands itself over");
    assert_eq!(
        String::from_utf8(given).expect("a selection is text"),
        "quarterly"
    );
}

/// **A form that was never offered is refused rather than answered wrongly.**
///
/// `Clipboard` checks this first, so reaching it means a later caller asked
/// directly. Answering a title's bytes for an image request would be worse
/// than refusing: the asker would paste text into a picture.
#[test]
fn a_form_that_was_not_offered_is_refused() {
    let menu = opened(OpenedBy::TheKeyboard, 3);
    let (_offer, mut source) = menu.copy_the_title().expect("an open menu offers a copy");
    assert!(source.give(&Kind::image_png()).is_err());
}

/// **A window that named itself nothing offers no copy and keeps its actions.**
///
/// `FrameName` carries that case, and a menu that refused to open over it would
/// take away every action on a window because its title was absent.
#[test]
fn a_window_with_no_title_offers_no_copy() {
    let mut menu = TheWindowMenu::closed();
    menu.opened_without_a_surface("", Point::from((0, 0)), OpenedBy::TheKeyboard, 4);
    assert_eq!(menu.focused(), Some(0), "the rows went with the title");
    assert!(
        menu.copy_the_title().is_none(),
        "an empty title was offered to the clipboard, which a paste cannot tell from a failure"
    );
}

/// **Choosing answers the focused row's action, and nothing when none is.**
#[test]
fn choosing_answers_the_focused_rows_action() {
    let rows = [Action::CloseTheWindow, Action::TheLeftHalf];
    let mut menu = opened(OpenedBy::APointer, rows.len());
    assert_eq!(
        chosen_from(&menu, &rows),
        None,
        "a pointer road with nothing focused chose something"
    );

    menu.the_pointer_is_on(1);
    assert_eq!(chosen_from(&menu, &rows), Some(Action::TheLeftHalf));

    // A focus past the end answers nothing rather than panicking: the rows a
    // window offers can change while the menu is open.
    menu.the_pointer_is_on(99);
    assert_eq!(chosen_from(&menu, &rows), None);
}
