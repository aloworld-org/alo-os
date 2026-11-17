use std::io::{BufRead as _, BufReader};
use std::path::{Path, PathBuf};

use alo_file_chooser::{Kind, Listing, NotListed};

use super::*;

/// Anna's home: one folder, one file.
struct AHome;

impl Folders for AHome {
    fn listed(&self, at: &Path) -> Result<Listing, NotListed> {
        match at.to_str() {
            Some("/home/anna") => Ok(Listing::of(
                vec!["Letters".to_owned()],
                vec!["march.txt".to_owned()],
            )),
            Some("/home") => Ok(Listing::of(vec!["anna".to_owned()], vec![])),
            Some("/") => Ok(Listing::of(vec!["home".to_owned()], vec![])),
            Some("/home/anna/Letters") => Ok(Listing::of(vec![], vec![])),
            _ => Err(NotListed::NotThere),
        }
    }

    fn holds(&self, at: &Path, name: &str) -> Option<Kind> {
        (at == Path::new("/home/anna") && name == "march.txt").then_some(Kind::File)
    }
}

/// The portal's end of one connection, and the desktop's.
fn asking(to: ToChoose) -> (AskedForAFile, UnixStream) {
    let (desktop, portal) = UnixStream::pair().unwrap();
    desktop.set_nonblocking(true).unwrap();
    let asked = ToChooseAFile::of(
        Application::identified("org.gnome.TextEditor").unwrap(),
        to,
        PathBuf::from("/home/anna"),
    )
    .unwrap();
    (AskedForAFile::of(asked, desktop), portal)
}

/// What the portal heard.
fn heard(portal: UnixStream) -> String {
    let mut line = String::new();
    BufReader::new(portal).read_line(&mut line).unwrap();
    line
}

/// **A file chosen is answered as its whole path**, and the sheet comes down.
#[test]
fn a_file_chosen_is_answered() {
    let now = Instant::now();
    let mut sheets = TheFileChooser::none();
    let (asked, portal) = asking(ToChoose::Open);
    sheets.asked(vec![asked], true, now, &AHome);
    assert!(sheets.is_up());
    let rows = sheets.shows().unwrap().chooser.rows().len();
    assert_eq!(rows, 3, "the folder above, Letters and march.txt");
    sheets.clicked(OnTheFileChooser::TheRow(2), now, &AHome);
    assert!(sheets.is_up(), "a first click only moves the keyboard");
    sheets.pressed(Key::Enter, now, &AHome);
    assert!(!sheets.is_up());
    assert_eq!(heard(portal), "chosen /home/anna/march.txt\n");
}

/// **Escape and Cancel are `cancelled`**, and nothing else is said.
#[test]
fn a_cancel_is_cancelled() {
    let now = Instant::now();
    let mut sheets = TheFileChooser::none();
    let (first, first_portal) = asking(ToChoose::Open);
    let (second, second_portal) = asking(ToChoose::Save {
        name: "new.txt".to_owned(),
    });
    sheets.asked(vec![first, second], true, now, &AHome);
    sheets.pressed(Key::Escape, now, &AHome);
    assert_eq!(heard(first_portal), "cancelled\n");
    assert!(sheets.is_up(), "the second goes up");
    sheets.clicked(OnTheFileChooser::Cancel, now, &AHome);
    assert_eq!(heard(second_portal), "cancelled\n");
    assert!(!sheets.is_up());
}

/// **Saving over a file asks first**, and the second press answers it.
#[test]
fn saving_over_a_file_asks_first() {
    let now = Instant::now();
    let mut sheets = TheFileChooser::none();
    let (asked, portal) = asking(ToChoose::Save {
        name: "march.txt".to_owned(),
    });
    sheets.asked(vec![asked], true, now, &AHome);
    sheets.clicked(OnTheFileChooser::TheButton, now, &AHome);
    assert!(sheets.is_up(), "replaced on the first press");
    sheets.clicked(OnTheFileChooser::TheButton, now, &AHome);
    assert_eq!(heard(portal), "chosen /home/anna/march.txt\n");
}

/// **One up and four waiting; the next is `cannot-show`, and so is every one
/// a desktop that cannot ask is sent.**
#[test]
fn too_many_or_no_person_cannot_show() {
    let now = Instant::now();
    let mut sheets = TheFileChooser::none();
    let mut portals = Vec::new();
    let mut asks = Vec::new();
    for _ in 0..=AS_MANY_AS_WAIT + 1 {
        let (asked, portal) = asking(ToChoose::Open);
        asks.push(asked);
        portals.push(portal);
    }
    sheets.asked(asks, true, now, &AHome);
    let last = portals.pop().unwrap();
    assert_eq!(heard(last), "cannot-show\n");
    let (asked, portal) = asking(ToChoose::Open);
    let mut nobody = TheFileChooser::none();
    nobody.asked(vec![asked], false, now, &AHome);
    assert!(!nobody.is_up());
    assert_eq!(heard(portal), "cannot-show\n");
}

/// **Five minutes unanswered is `unanswered`, not `cancelled`**; a portal
/// that hangs up takes its sheet down.
#[test]
fn unanswered_and_hung_up() {
    let now = Instant::now();
    let mut sheets = TheFileChooser::none();
    let (asked, portal) = asking(ToChoose::Open);
    sheets.asked(vec![asked], true, now, &AHome);
    sheets.waited(now + THE_LONGEST, &AHome);
    assert!(!sheets.is_up());
    assert_eq!(heard(portal), "unanswered\n");

    let (asked, portal) = asking(ToChoose::Open);
    sheets.asked(vec![asked], true, now, &AHome);
    drop(portal);
    sheets.waited(now, &AHome);
    assert!(!sheets.is_up());
}

/// **A reader's press is held to the sheet it was heard on**, and a row the
/// sheet does not have is refused.
#[test]
fn a_readers_press_is_held_to_its_sheet() {
    let now = Instant::now();
    let mut sheets = TheFileChooser::none();
    let (asked, portal) = asking(ToChoose::Open);
    sheets.asked(vec![asked], true, now, &AHome);
    let number = sheets.shows().unwrap().number;
    assert!(!sheets.a_reader_pressed(number + 1, OnTheFileChooser::Cancel, now, &AHome));
    assert!(!sheets.a_reader_pressed(number, OnTheFileChooser::TheRow(9), now, &AHome));
    assert!(sheets.is_up());
    assert!(sheets.a_reader_pressed(number, OnTheFileChooser::TheRow(2), now, &AHome));
    assert_eq!(heard(portal), "chosen /home/anna/march.txt\n");
}

/// **A sheet the display cannot hold is `cannot-show`**, never held with
/// nothing on the screen.
#[test]
fn a_sheet_not_drawn_cannot_show() {
    let now = Instant::now();
    let mut sheets = TheFileChooser::none();
    let (asked, portal) = asking(ToChoose::Open);
    sheets.asked(vec![asked], true, now, &AHome);
    sheets.could_not_be_drawn(now, &AHome);
    assert!(!sheets.is_up());
    assert_eq!(heard(portal), "cannot-show\n");
}

/// **The keys a person presses are the chooser's**: Tab, Backspace, Enter,
/// Escape, the arrows and letters, and nothing else.
#[test]
fn the_keys_are_read_as_the_chooser_reads_them() {
    use crate::SignInKey as K;
    assert_eq!(the_key(K::Letter('a')), Some(Key::Letter('a')));
    assert_eq!(the_key(K::Erase), Some(Key::Erase));
    assert_eq!(the_key(K::OtherField), Some(Key::Tab));
    assert_eq!(the_key(K::FieldBefore), Some(Key::BackTab));
    assert_eq!(the_key(K::Enter), Some(Key::Enter));
    assert_eq!(the_key(K::Clear), Some(Key::Escape));
    assert_eq!(the_key(K::Earlier), Some(Key::Up));
    assert_eq!(the_key(K::Later), Some(Key::Down));
    assert_eq!(the_key(K::Nothing), None);
    assert_eq!(the_key(K::TheScreenReader), None);
}
