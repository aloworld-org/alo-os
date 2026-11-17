use super::*;
use crate::testing::WrittenDown;

fn a_home() -> WrittenDown {
    WrittenDown::default()
        .with("/", &["home"], &[])
        .with("/home", &["anna"], &[])
        .with("/home/anna", &["Letters", "Music"], &["march.txt"])
        .with("/home/anna/Letters", &[], &["to Rui.txt"])
        .unreadable("/home/anna/Music")
}

/// **The keyboard starts on the first thing inside the folder**, past the
/// folder above, and on the folder above only where nothing else is there.
#[test]
fn the_keyboard_starts_inside_the_folder() {
    let disk = a_home();
    let chooser = Chooser::to_open(Path::new("/home/anna"), &disk);
    assert_eq!(chooser.rows()[chooser.focus()].name, "Letters");
    let empty = WrittenDown::default().with("/home/anna/empty", &[], &[]);
    let chooser = Chooser::to_open(Path::new("/home/anna/empty"), &empty);
    assert_eq!(chooser.focus(), 0);
    assert_eq!(chooser.rows()[0].kind, Kind::Above);
}

/// **Going up puts the keyboard on the folder just left**, so going back in
/// is one Enter.
#[test]
fn going_up_lands_on_the_folder_just_left() {
    let disk = a_home();
    let mut chooser = Chooser::to_open(Path::new("/home/anna/Letters"), &disk);
    assert_eq!(chooser.key(Key::Erase, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home/anna"));
    assert_eq!(chooser.rows()[chooser.focus()].name, "Letters");
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home/anna/Letters"));
}

/// **A starting folder that cannot be read is stood in with nothing shown**,
/// said, and the person can still go up.
#[test]
fn a_starting_folder_that_cannot_be_read_can_be_left() {
    let disk = a_home();
    let mut chooser = Chooser::to_open(Path::new("/home/anna/Music"), &disk);
    assert_eq!(chooser.note(), Some(&Note::CouldNotOpen("Music".to_owned())));
    assert_eq!(chooser.rows().len(), 1, "only the folder above");
    assert_eq!(chooser.key(Key::Enter, &disk), None);
    assert_eq!(chooser.at(), Path::new("/home/anna"));
    assert_eq!(chooser.note(), None, "a sentence about a folder left behind");
}

/// **A letter jump goes round**, sets case aside, and never lands on the
/// folder above.
#[test]
fn a_letter_jump_goes_round() {
    let disk = a_home();
    let mut chooser = Chooser::to_open(Path::new("/home/anna"), &disk);
    assert_eq!(chooser.key(Key::Letter('M'), &disk), None);
    assert_eq!(chooser.rows()[chooser.focus()].name, "Music");
    assert_eq!(chooser.key(Key::Letter('m'), &disk), None);
    assert_eq!(chooser.rows()[chooser.focus()].name, "march.txt");
    assert_eq!(chooser.key(Key::Letter('m'), &disk), None);
    assert_eq!(chooser.rows()[chooser.focus()].name, "Music");
    assert_eq!(chooser.key(Key::Letter('h'), &disk), None);
    assert_ne!(chooser.rows()[chooser.focus()].kind, Kind::Above);
}

/// **When opening, Tab does nothing and letters never become a name.**
#[test]
fn opening_has_no_name() {
    let disk = a_home();
    let mut chooser = Chooser::to_open(Path::new("/home/anna"), &disk);
    assert_eq!(chooser.key(Key::Tab, &disk), None);
    assert_eq!(chooser.on(), Where::TheRows);
    chooser.pressed_the_name();
    assert_eq!(chooser.on(), Where::TheRows);
    assert_eq!(chooser.name(), "");
}

/// **A control character typed is not part of a name**, and the name stops
/// at the longest a name may be.
#[test]
fn a_name_holds_only_what_a_name_may() {
    let disk = a_home();
    let mut chooser = Chooser::to_save(Path::new("/home/anna"), "", &disk);
    for letter in ['a', '\n', '\t', '\u{7}', 'b'] {
        assert_eq!(chooser.key(Key::Letter(letter), &disk), None);
    }
    assert_eq!(chooser.name(), "ab");
    for _ in 0..300 {
        let _ = chooser.key(Key::Letter('é'), &disk);
    }
    assert!(chooser.name().len() <= alo_acting::THE_LONGEST_NAME);
    assert!(alo_acting::is_a_name(chooser.name()));
}

/// **The page shown is the one the keyboard is on**, a whole page at a time.
#[test]
fn a_page_at_a_time() {
    let files: Vec<String> = (0..10).map(|n| format!("{n}.txt")).collect();
    let names: Vec<&str> = files.iter().map(String::as_str).collect();
    let disk = WrittenDown::default().with("/", &[], &names);
    let mut chooser = Chooser::to_open(Path::new("/"), &disk);
    assert_eq!(chooser.shown(4), 0..4);
    for _ in 0..4 {
        let _ = chooser.key(Key::Down, &disk);
    }
    assert_eq!(chooser.shown(4), 4..8);
    let _ = chooser.key(Key::Last, &disk);
    assert_eq!(chooser.shown(4), 8..10);
    assert_eq!(chooser.shown(0), 9..10, "a page is at least one row");
}
