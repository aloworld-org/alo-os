//! **The `choose-file` words, tested from the contract and not from the code.**
//! Written by a second reader who read `docs/contracts/application-door.md`'s
//! section *The person chooses a file* and `docs/design/the-file-chooser.md`,
//! and nothing of the implementation: every assertion here is a sentence of
//! that contract — the shape of the line, what is percent-encoded and how,
//! every refusal it names, and that an answer which is not a whole path to a
//! file, or a word nobody defined, is never read as a choice.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_acting::{
    CHOOSE_FILE, ChoseAFile, NotAFileToChoose, THE_LONGEST_NAME, ToChoose, ToChooseAFile,
    is_a_name, is_a_whole_path,
};
use alo_applications::Application;

/// The application every request here comes from.
const ASKING: &str = "org.gnome.TextEditor";

/// The application every request here comes from, as alo knows it.
fn asking() -> Application {
    Application::identified(ASKING).unwrap()
}

/// A request to open, starting in `folder`.
fn to_open(folder: &str) -> Result<ToChooseAFile, NotAFileToChoose> {
    ToChooseAFile::of(asking(), ToChoose::Open, PathBuf::from(folder))
}

/// A request to save under `name`, starting in `folder`.
fn to_save(folder: &str, name: &str) -> Result<ToChooseAFile, NotAFileToChoose> {
    ToChooseAFile::of(
        asking(),
        ToChoose::Save {
            name: name.to_owned(),
        },
        PathBuf::from(folder),
    )
}

/// What reading `line` gives, when it is this act and malformed.
fn refused(line: &str) -> Option<NotAFileToChoose> {
    match ToChooseAFile::read(line) {
        Some(Err(why)) => Some(why),
        _ => None,
    }
}

/// A line to open in `folder`, written by hand as the contract shows it.
fn open_line(folder: &str) -> String {
    format!("{CHOOSE_FILE} {ASKING} open {folder}")
}

/// A line to save under `name` in `folder`, written by hand.
fn save_line(folder: &str, name: &str) -> String {
    format!("{CHOOSE_FILE} {ASKING} save {folder} {name}")
}

/// A line to open that is exactly `length` bytes long, its folder made of
/// ordinary names of at most a hundred bytes each.
fn an_open_line_of(length: usize) -> String {
    let prefix = format!("{CHOOSE_FILE} {ASKING} open ");
    let room = length - prefix.len();
    let mut folder = String::new();
    while room - folder.len() > 101 {
        folder.push('/');
        folder.push_str(&"a".repeat(99));
    }
    let rest = room - folder.len();
    folder.push('/');
    folder.push_str(&"a".repeat(rest - 1));
    format!("{prefix}{folder}")
}

/// The act is named as the contract names it, and a name is at most 255 bytes.
#[test]
fn the_act_is_called_choose_file_and_a_name_is_at_most_255_bytes() {
    assert_eq!(CHOOSE_FILE, "choose-file");
    assert_eq!(THE_LONGEST_NAME, 255);
}

/// `→ choose-file <asking-application> open <folder>`, exactly.
#[test]
fn an_open_is_written_as_the_contract_draws_it() {
    let request = to_open("/home/anna").unwrap();
    assert_eq!(
        request.written(),
        "choose-file org.gnome.TextEditor open /home/anna"
    );
    assert!(!request.written().ends_with('\n'));
}

/// `→ choose-file <asking-application> save <folder> <name>`, exactly.
#[test]
fn a_save_is_written_with_the_name_the_application_suggests() {
    let request = to_save("/home/anna", "notes.txt").unwrap();
    assert_eq!(
        request.written(),
        "choose-file org.gnome.TextEditor save /home/anna notes.txt"
    );
}

/// A space in a folder or a name is written `%20`, because the line carries
/// two of them and a space separates its words.
#[test]
fn a_space_is_written_percent_encoded() {
    let request = to_save("/home/anna/My Documents", "my letter.txt").unwrap();
    assert_eq!(
        request.written(),
        "choose-file org.gnome.TextEditor save /home/anna/My%20Documents my%20letter.txt"
    );
}

/// A letter outside ASCII is written as its UTF-8 bytes, each `%XX`, as a
/// `file:` address writes it.
#[test]
fn a_letter_outside_ascii_is_written_as_its_bytes() {
    let request = to_save("/home/zoë", "Brief für Rui.txt").unwrap();
    let written = request.written();
    assert!(written.is_ascii(), "{written}");
    assert_eq!(
        written,
        "choose-file org.gnome.TextEditor save /home/zo%C3%AB Brief%20f%C3%BCr%20Rui.txt"
    );
}

/// The unreserved bytes are written as they are, and every other byte is not —
/// a `%` itself included, or it could not be read back.
#[test]
fn only_the_unreserved_bytes_are_written_as_they_are() {
    let plain = to_save("/home/anna", "a-b.c_d~E9").unwrap();
    assert_eq!(
        plain.written(),
        "choose-file org.gnome.TextEditor save /home/anna a-b.c_d~E9"
    );
    let marked = to_save("/home/anna", "50%+#.txt").unwrap();
    assert_eq!(
        marked.written(),
        "choose-file org.gnome.TextEditor save /home/anna 50%25%2B%23.txt"
    );
}

/// What is written is read back as the same request, open and save alike.
#[test]
fn a_request_reads_back_as_itself() {
    let open = to_open("/home/anna/My Documents").unwrap();
    let read = ToChooseAFile::read(&open.written()).unwrap().unwrap();
    assert_eq!(read.asking(), &asking());
    assert!(matches!(read.to(), ToChoose::Open));
    assert_eq!(read.starting_in(), Path::new("/home/anna/My Documents"));
    assert_eq!(read.written(), open.written());

    let save = to_save("/home/zoë/Lettres", "Brief für Rui 50%.txt").unwrap();
    let read = ToChooseAFile::read(&save.written()).unwrap().unwrap();
    assert_eq!(read.asking(), &asking());
    assert!(
        matches!(read.to(), ToChoose::Save { name } if name == "Brief für Rui 50%.txt"),
        "the name came back changed"
    );
    assert_eq!(read.starting_in(), Path::new("/home/zoë/Lettres"));
    assert_eq!(read.written(), save.written());
}

/// A hand-written line in the contract's shape is read, with its escapes
/// decoded.
#[test]
fn a_line_written_by_hand_is_read() {
    let read = ToChooseAFile::read(&save_line("/home/anna/My%20Documents", "a%20b.txt"))
        .unwrap()
        .unwrap();
    assert_eq!(read.starting_in(), Path::new("/home/anna/My Documents"));
    assert!(matches!(read.to(), ToChoose::Save { name } if name == "a b.txt"));
}

/// A folder is a whole path from the root with only ordinary names in it:
/// none relative, no `.`, no `..`, no empty name, no trailing `/`, no control
/// character.
#[test]
fn a_folder_that_is_not_a_whole_ordinary_path_is_refused() {
    for folder in [
        "home/anna",
        "anna",
        "",
        "/home/./anna",
        "/home/../anna",
        "/home/anna/.",
        "/home/anna/..",
        "/home//anna",
        "//home/anna",
        "/home/anna/",
        "/home/an\nna",
        "/home/an\tna",
        "/home/an\u{7f}na",
    ] {
        assert_eq!(
            to_open(folder).err(),
            Some(NotAFileToChoose::NotAFolder),
            "{folder:?} was taken as a folder"
        );
    }
}

/// A name is one name: not empty, not `.` or `..`, no `/`, no control
/// character, at most 255 bytes.
#[test]
fn a_name_that_is_not_one_name_is_refused() {
    let too_long = "a".repeat(256);
    for name in [
        "",
        ".",
        "..",
        "a/b",
        "/",
        "letter\n.txt",
        "a\u{0}b",
        &too_long,
    ] {
        assert_eq!(
            to_save("/home/anna", name).err(),
            Some(NotAFileToChoose::NotAName),
            "{name:?} was taken as a name"
        );
    }
}

/// The 255 is bytes, not letters: 255 bytes is a name, 256 is not, however
/// few letters they make.
#[test]
fn the_longest_name_is_counted_in_bytes() {
    let at_the_limit = format!("{}a", "ü".repeat(127));
    assert_eq!(at_the_limit.len(), 255);
    assert!(to_save("/home/anna", &at_the_limit).is_ok());
    assert!(is_a_name(&at_the_limit));

    let over = "ü".repeat(128);
    assert_eq!(over.len(), 256);
    assert_eq!(
        to_save("/home/anna", &over).err(),
        Some(NotAFileToChoose::NotAName)
    );
    assert!(!is_a_name(&over));
}

/// A name may begin with a dot: a hidden name is still a name a person may
/// type and save under.
#[test]
fn a_hidden_name_is_a_name() {
    assert!(is_a_name(".bashrc"));
    assert!(to_save("/home/anna", ".bashrc").is_ok());
}

/// The two rules are public, and agree with what the request refuses.
#[test]
fn the_rules_for_a_name_and_a_whole_path_are_the_contracts() {
    for name in [
        "notes.txt",
        "my letter.txt",
        "Brief für Rui.txt",
        "..hidden",
        "a..",
    ] {
        assert!(is_a_name(name), "{name:?}");
    }
    for name in ["", ".", "..", "a/b", "a\nb"] {
        assert!(!is_a_name(name), "{name:?}");
    }
    for path in ["/home/anna", "/home/anna/My Documents", "/a"] {
        assert!(is_a_whole_path(Path::new(path)), "{path:?}");
    }
    for path in [
        "home/anna",
        "/home/./anna",
        "/home/../anna",
        "/home//anna",
        "/home/anna/",
        "/home/a\nb",
    ] {
        assert!(!is_a_whole_path(Path::new(path)), "{path:?}");
    }
}

/// The same refusals hold for a line read off the door: a folder that is not
/// a whole ordinary path is not a request.
#[test]
fn a_line_whose_folder_is_not_a_whole_path_is_refused() {
    for folder in [
        "home/anna",
        "/home/./anna",
        "/home/../anna",
        "/home//anna",
        "/home/anna/",
        "/home/%2E%2E/anna",
        "/home/%2e/anna",
        "/home/an%0Ana",
        "/home/an%09na",
        "/home/an%7Fna",
        "/home/an%00na",
    ] {
        assert_eq!(
            refused(&open_line(folder)),
            Some(NotAFileToChoose::NotAFolder),
            "{folder:?} was read as a folder"
        );
    }
}

/// A name read off the door is refused after decoding, as it is before: a `/`
/// hidden as `%2F` is still a `/`.
#[test]
fn a_line_whose_name_is_not_one_name_is_refused() {
    let too_long = "a".repeat(256);
    for name in [
        "a%2Fb", "a/b", "%2E", "%2E%2E", ".", "..", "a%0Ab", "a%00b", &too_long,
    ] {
        assert_eq!(
            refused(&save_line("/home/anna", name)),
            Some(NotAFileToChoose::NotAName),
            "{name:?} was read as a name"
        );
    }
}

/// A `%` that does not begin two hexadecimal digits is not an escape, and a
/// line holding one is not a request.
#[test]
fn a_bad_escape_is_refused() {
    for folder in ["/home/50%", "/home/a%2", "/home/a%G0", "/home/a%%20"] {
        assert_eq!(
            refused(&open_line(folder)),
            Some(NotAFileToChoose::NotAFolder),
            "{folder:?}"
        );
    }
    for name in ["50%", "a%2", "a%zz.txt"] {
        assert_eq!(
            refused(&save_line("/home/anna", name)),
            Some(NotAFileToChoose::NotAName),
            "{name:?}"
        );
    }
}

/// Escapes that decode to bytes that are not text are refused: a lone `%FF`,
/// and half of a two-byte letter.
#[test]
fn escapes_that_are_not_text_are_refused() {
    for folder in ["/home/%FF", "/home/zo%C3", "/home/%C3%28"] {
        assert_eq!(
            refused(&open_line(folder)),
            Some(NotAFileToChoose::NotAFolder),
            "{folder:?}"
        );
    }
    assert_eq!(
        refused(&save_line("/home/anna", "%FF.txt")),
        Some(NotAFileToChoose::NotAName)
    );
}

/// "Every byte outside the unreserved set is written `%XX`", and a line that
/// is not exactly this is not a request: a raw letter outside ASCII is not
/// the contract's line.
#[test]
fn a_byte_that_should_have_been_escaped_is_refused() {
    assert!(refused(&open_line("/home/zoë")).is_some());
    assert!(refused(&save_line("/home/anna", "für.txt")).is_some());
}

/// The line is at most 4096 bytes: exactly 4096 is read, one more is not.
#[test]
fn a_line_over_4096_bytes_is_refused() {
    let longest = an_open_line_of(4096);
    assert_eq!(longest.len(), 4096);
    assert!(matches!(ToChooseAFile::read(&longest), Some(Ok(_))));

    let over = format!("{longest}a");
    assert_eq!(over.len(), 4097);
    assert_eq!(refused(&over), Some(NotAFileToChoose::TooLong));
}

/// A line with a word missing is not a request.
#[test]
fn a_line_with_a_word_missing_is_refused() {
    for line in [
        "choose-file".to_owned(),
        format!("{CHOOSE_FILE} {ASKING}"),
        format!("{CHOOSE_FILE} {ASKING} open"),
        format!("{CHOOSE_FILE} {ASKING} save"),
        format!("{CHOOSE_FILE} {ASKING} save /home/anna"),
    ] {
        assert_eq!(
            refused(&line),
            Some(NotAFileToChoose::NotEnoughWords),
            "{line:?}"
        );
    }
}

/// A line with a word too many is not a request either, though it begins as
/// one: `open` carries no name, and a name carries no second word.
#[test]
fn a_line_with_a_word_too_many_is_refused() {
    for line in [
        format!("{} extra", open_line("/home/anna")),
        format!("{} extra", save_line("/home/anna", "a.txt")),
        save_line("/home/anna", "my letter.txt"),
    ] {
        assert!(refused(&line).is_some(), "{line:?} was read");
    }
}

/// Two spaces make an empty word, which is not the contract's line.
#[test]
fn a_line_with_an_empty_word_is_refused() {
    for line in [
        format!("{CHOOSE_FILE}  {ASKING} open /home/anna"),
        format!("{CHOOSE_FILE} {ASKING}  open /home/anna"),
        format!("{CHOOSE_FILE} {ASKING} open  /home/anna"),
        format!("{CHOOSE_FILE} {ASKING} open /home/anna "),
    ] {
        assert!(refused(&line).is_some(), "{line:?} was read");
    }
}

/// The third word is `open` or `save`, and nothing else.
#[test]
fn a_line_that_is_neither_open_nor_save_is_refused() {
    for how in ["read", "OPEN", "Save", "opens", "delete"] {
        let line = format!("{CHOOSE_FILE} {ASKING} {how} /home/anna a.txt");
        assert_eq!(
            refused(&line),
            Some(NotAFileToChoose::NeitherOpenNorSave),
            "{how:?}"
        );
    }
}

/// The second word is an application's identifier, and one no application
/// could have is refused.
#[test]
fn a_line_whose_asker_is_not_an_application_is_refused() {
    let line = format!("{CHOOSE_FILE} org/evil open /home/anna");
    assert_eq!(refused(&line), Some(NotAFileToChoose::NotAnApplication));
}

/// A line that is some other act is not this one, and is left for that act.
#[test]
fn a_line_of_another_act_is_not_this_act() {
    for line in [
        "",
        "choose-link org.gnome.TextEditor https://example.org",
        "open-file org.gnome.TextEditor /home/anna/a.txt",
        "chosen /home/anna/a.txt",
        "front",
    ] {
        assert!(ToChooseAFile::read(line).is_none(), "{line:?}");
    }
}

/// `chosen <path>`, `cancelled`, `unanswered` and `cannot-show`, exactly, with
/// the path written as a folder is.
#[test]
fn each_answer_is_written_as_the_contract_draws_it() {
    assert_eq!(
        ChoseAFile::Chosen(PathBuf::from("/home/anna/my letter.txt")).written(),
        "chosen /home/anna/my%20letter.txt"
    );
    assert_eq!(
        ChoseAFile::Chosen(PathBuf::from("/home/zoë/für.txt")).written(),
        "chosen /home/zo%C3%AB/f%C3%BCr.txt"
    );
    assert_eq!(ChoseAFile::Cancelled.written(), "cancelled");
    assert_eq!(ChoseAFile::Unanswered.written(), "unanswered");
    assert_eq!(ChoseAFile::CannotShow.written(), "cannot-show");
}

/// Every answer reads back as itself.
#[test]
fn each_answer_reads_back_as_itself() {
    let chosen = ChoseAFile::Chosen(PathBuf::from("/home/zoë/My Letters/für Rui 50%.txt"));
    assert!(
        matches!(
            ChoseAFile::read(&chosen.written()),
            Some(ChoseAFile::Chosen(path)) if path == Path::new("/home/zoë/My Letters/für Rui 50%.txt")
        ),
        "{}",
        chosen.written()
    );
    assert!(matches!(
        ChoseAFile::read("cancelled"),
        Some(ChoseAFile::Cancelled)
    ));
    assert!(matches!(
        ChoseAFile::read("unanswered"),
        Some(ChoseAFile::Unanswered)
    ));
    assert!(matches!(
        ChoseAFile::read("cannot-show"),
        Some(ChoseAFile::CannotShow)
    ));
}

/// "A caller reads `chosen` of anything but a whole path to a file as no
/// answer": nothing here may come back as a choice.
#[test]
fn a_chosen_that_is_not_a_whole_path_to_a_file_is_never_a_choice() {
    for heard in [
        "chosen",
        "chosen ",
        "chosen /",
        "chosen home/anna/a.txt",
        "chosen a.txt",
        "chosen /home/anna/",
        "chosen /home//anna/a.txt",
        "chosen /home/./a.txt",
        "chosen /home/../etc/passwd",
        "chosen /home/anna/..",
        "chosen /home/%2E%2E/etc/passwd",
        "chosen /home/anna/a%0A.txt",
        "chosen /home/anna/a%00.txt",
        "chosen /home/anna/a%ZZ.txt",
        "chosen /home/anna/a%2",
        "chosen /home/anna/%FF.txt",
        "chosen /home/anna/a.txt /home/anna/b.txt",
        "chosen /home/anna/a b.txt",
        "chosen  /home/anna/a.txt",
    ] {
        assert!(
            !matches!(ChoseAFile::read(heard), Some(ChoseAFile::Chosen(_))),
            "{heard:?} was read as a choice"
        );
    }
}

/// A word the contract does not define is not an answer, and none of the
/// answers carries a second word.
#[test]
fn an_unknown_word_is_not_an_answer() {
    for heard in [
        "",
        "maybe",
        "Cancelled",
        "CANNOT-SHOW",
        "cannot show",
        "cancelled now",
        "unanswered /home/anna/a.txt",
        "choose /home/anna/a.txt",
        "Chosen /home/anna/a.txt",
    ] {
        assert!(ChoseAFile::read(heard).is_none(), "{heard:?} was read");
    }
}
