//! **The `alo-chooses-file` words, tested from the contract and not from the
//! code.** Written by a second reader who read `docs/contracts/application-door.md`'s
//! sections *alo answers the file chooser* and *The person chooses a file* (for
//! how a folder and a name are percent-encoded) and `agent-verbs.md`'s
//! `choose_on_the_file_chooser`, and nothing of the implementation. Every
//! assertion is a sentence of those: the shape of the line, what is encoded and
//! how, every refusal a folder and a name can meet, that an answer which is not
//! exactly `chosen <application>`, `no-sheet` or `not-believed` is never read as
//! one, and that a line of another act is not this act's.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_acting::{ALO_CHOOSES_FILE, AnsweredTheChooser, NotAnAnswer, ToAnswerAFileChooser};
use alo_applications::Application;

/// The application whose sheet is answered in these tests.
const ASKING: &str = "org.gnome.TextEditor";

/// That application, as alo knows it.
fn asking() -> Application {
    Application::identified(ASKING).unwrap()
}

/// An answer of `name` in `folder`, as `alo-agentd` would build it.
fn of(folder: &str, name: &str) -> Result<ToAnswerAFileChooser, NotAnAnswer> {
    ToAnswerAFileChooser::of(PathBuf::from(folder), name)
}

/// `alo-chooses-file <folder> <name>`: the act's own word is the one the
/// contract names.
#[test]
fn the_act_is_named_alo_chooses_file() {
    assert_eq!(ALO_CHOOSES_FILE, "alo-chooses-file");
}

/// The words: the act, the folder, one name, separated by single spaces, with
/// no newline in what is written.
#[test]
fn a_plain_answer_is_written_as_the_act_the_folder_and_the_name() {
    let answer = of("/var/home/alo/Documents", "notes.txt").unwrap();
    assert_eq!(
        answer.written(),
        "alo-chooses-file /var/home/alo/Documents notes.txt"
    );
    assert!(!answer.written().contains('\n'));
    assert_eq!(answer.folder(), Path::new("/var/home/alo/Documents"));
    assert_eq!(answer.name(), "notes.txt");
}

/// The file the answer names is the name inside the folder.
#[test]
fn the_file_is_the_name_inside_the_folder() {
    let answer = of("/var/home/alo/Documents", "notes.txt").unwrap();
    assert_eq!(
        answer.file(),
        PathBuf::from("/var/home/alo/Documents").join("notes.txt")
    );
}

/// A folder and a name are percent-encoded: a space is `%20`, and in a folder
/// `/` is left as it is.
#[test]
fn spaces_are_percent_encoded_in_the_folder_and_the_name() {
    let answer = of("/var/home/alo/My Documents", "to Rui.txt").unwrap();
    assert_eq!(
        answer.written(),
        "alo-chooses-file /var/home/alo/My%20Documents to%20Rui.txt"
    );
}

/// Every byte outside the unreserved set is `%XX`, so a non-ASCII letter is
/// written as its UTF-8 bytes, each encoded.
#[test]
fn a_letter_outside_ascii_is_written_as_its_utf8_bytes() {
    let answer = of("/var/home/alo/Bücher", "für Rui.txt").unwrap();
    assert_eq!(
        answer.written(),
        "alo-chooses-file /var/home/alo/B%C3%BCcher f%C3%BCr%20Rui.txt"
    );
}

/// `A`–`Z`, `a`–`z`, `0`–`9`, `-`, `.`, `_` and `~` are written as they are.
#[test]
fn the_unreserved_bytes_are_written_as_they_are() {
    let answer = of("/srv/A-z_0.9~", "B-y_1.8~").unwrap();
    assert_eq!(answer.written(), "alo-chooses-file /srv/A-z_0.9~ B-y_1.8~");
}

/// A `/` is only left alone in a folder; in a name it cannot occur at all, and
/// any other reserved byte — `%` itself, `+`, `#`, `?` — is encoded.
#[test]
fn reserved_bytes_are_encoded() {
    let answer = of("/srv/100% sure", "a+b#c?d.txt").unwrap();
    assert_eq!(
        answer.written(),
        "alo-chooses-file /srv/100%25%20sure a%2Bb%23c%3Fd.txt"
    );
}

/// What is written is read back as the same answer: spaces and non-ASCII
/// survive the round trip in both the folder and the name.
#[test]
fn an_answer_reads_back_as_itself() {
    for (folder, name) in [
        ("/var/home/alo/Documents", "notes.txt"),
        ("/var/home/alo/My Documents", "to Rui.txt"),
        ("/var/home/alo/Bücher/Ελληνικά", "日本語 ファイル.md"),
        ("/srv/100% sure", "a+b#c?d.txt"),
        ("/var/home/alo", "~tilde-and_dash.~"),
    ] {
        let answer = of(folder, name).unwrap();
        let heard = ToAnswerAFileChooser::read(&answer.written());
        assert_eq!(heard, Some(Ok(answer)), "{folder} / {name}");
        let heard = heard.unwrap().unwrap();
        assert_eq!(heard.folder(), Path::new(folder));
        assert_eq!(heard.name(), name);
    }
}

/// Where the sheet starts is only where it starts: the person may go up to the
/// root, so the root itself is a folder alo can answer in.
#[test]
fn the_root_is_a_folder() {
    let answer = of("/", "notes.txt").unwrap();
    assert_eq!(
        ToAnswerAFileChooser::read(&answer.written()),
        Some(Ok(answer))
    );
}

/// A folder is a whole path from the root: one that does not start there is
/// not a folder.
#[test]
fn a_folder_that_is_not_a_whole_path_is_refused() {
    assert_eq!(
        of("var/home/alo", "notes.txt"),
        Err(NotAnAnswer::NotAFolder)
    );
    assert_eq!(of("", "notes.txt"), Err(NotAnAnswer::NotAFolder));
}

/// Only ordinary names: no `.`, no `..`, no empty name, no trailing `/`.
#[test]
fn a_folder_with_anything_but_ordinary_names_is_refused() {
    for folder in [
        "/var/home/alo/.",
        "/var/home/./alo",
        "/var/home/alo/..",
        "/var/home/alo/../alo",
        "/var//home/alo",
        "/var/home/alo/",
    ] {
        assert_eq!(
            of(folder, "notes.txt"),
            Err(NotAnAnswer::NotAFolder),
            "{folder:?}"
        );
    }
}

/// Neither may decode to a control character.
#[test]
fn a_folder_with_a_control_character_is_refused() {
    for folder in [
        "/var/home/al\no",
        "/var/home/al\to",
        "/var/home/al\u{7f}o",
        "/var/\0",
    ] {
        assert_eq!(
            of(folder, "notes.txt"),
            Err(NotAnAnswer::NotAFolder),
            "{folder:?}"
        );
    }
}

/// A name is one name: not empty, not `.` or `..`, no `/`.
#[test]
fn a_name_that_is_not_one_name_is_refused() {
    for name in ["", ".", "..", "a/b", "/notes.txt", "notes.txt/"] {
        assert_eq!(
            of("/var/home/alo", name),
            Err(NotAnAnswer::NotAName),
            "{name:?}"
        );
    }
}

/// A name is at most 255 bytes — counted in bytes, so 255 is taken and 256 is
/// not, and a non-ASCII letter counts as its UTF-8 length.
#[test]
fn a_name_is_at_most_255_bytes() {
    let longest = "a".repeat(255);
    let answer = of("/var/home/alo", &longest).unwrap();
    assert_eq!(answer.name(), longest);

    assert_eq!(
        of("/var/home/alo", &"a".repeat(256)),
        Err(NotAnAnswer::NotAName)
    );
    // 128 two-byte letters: 128 characters, 256 bytes.
    assert_eq!(
        of("/var/home/alo", &"ü".repeat(128)),
        Err(NotAnAnswer::NotAName)
    );
}

/// A name may not decode to a control character.
#[test]
fn a_name_with_a_control_character_is_refused() {
    for name in ["no\ntes.txt", "no\ttes.txt", "notes\u{7f}", "\0"] {
        assert_eq!(
            of("/var/home/alo", name),
            Err(NotAnAnswer::NotAName),
            "{name:?}"
        );
    }
}

/// A line with fewer words than the act, a folder and a name is not an answer.
#[test]
fn a_line_with_too_few_words_is_refused() {
    for line in ["alo-chooses-file", "alo-chooses-file /var/home/alo"] {
        assert_eq!(
            ToAnswerAFileChooser::read(line),
            Some(Err(NotAnAnswer::NotEnoughWords)),
            "{line:?}"
        );
    }
}

/// A line that is not exactly the act, a folder and a name is closed with
/// nothing written: an extra word, or an extra space, is never read as one.
#[test]
fn a_line_that_is_not_exactly_three_words_is_refused() {
    for line in [
        "alo-chooses-file /var/home/alo notes.txt more",
        "alo-chooses-file  /var/home/alo notes.txt",
        "alo-chooses-file /var/home/alo  notes.txt",
        "alo-chooses-file /var/home/alo notes.txt ",
    ] {
        let heard = ToAnswerAFileChooser::read(line);
        assert!(
            matches!(heard, Some(Err(_))),
            "{line:?} was read as {heard:?}"
        );
    }
}

/// A folder written on the line is held to the same rules as one built.
#[test]
fn a_folder_on_the_line_that_is_not_one_is_refused() {
    for line in [
        "alo-chooses-file var/home/alo notes.txt",
        "alo-chooses-file /var/home/alo/.. notes.txt",
        "alo-chooses-file /var/home/./alo notes.txt",
        "alo-chooses-file /var//home notes.txt",
        "alo-chooses-file /var/home/alo/ notes.txt",
        // `..` spelled in percent-encoding is still `..`.
        "alo-chooses-file /var/home/%2E%2E notes.txt",
        // A newline spelled in percent-encoding is still a control character.
        "alo-chooses-file /var/home/al%0Ao notes.txt",
    ] {
        assert_eq!(
            ToAnswerAFileChooser::read(line),
            Some(Err(NotAnAnswer::NotAFolder)),
            "{line:?}"
        );
    }
}

/// A name written on the line is held to the same rules as one built, after
/// decoding: `%2F` is a `/`, `%2E%2E` is `..`.
#[test]
fn a_name_on_the_line_that_is_not_one_is_refused() {
    for line in [
        "alo-chooses-file /var/home/alo a%2Fb",
        "alo-chooses-file /var/home/alo %2E%2E",
        "alo-chooses-file /var/home/alo %2E",
        "alo-chooses-file /var/home/alo ..",
        "alo-chooses-file /var/home/alo no%0Ates",
        "alo-chooses-file /var/home/alo %00",
    ] {
        assert_eq!(
            ToAnswerAFileChooser::read(line),
            Some(Err(NotAnAnswer::NotAName)),
            "{line:?}"
        );
    }
    let too_long = format!("alo-chooses-file /var/home/alo {}", "a".repeat(256));
    assert_eq!(
        ToAnswerAFileChooser::read(&too_long),
        Some(Err(NotAnAnswer::NotAName))
    );
}

/// Neither may decode to anything that is not text, and a `%` that does not
/// begin two hexadecimal digits is not an encoding.
#[test]
fn an_encoding_that_does_not_decode_to_text_is_refused() {
    for line in [
        "alo-chooses-file /var/home/alo notes%FF.txt",
        "alo-chooses-file /var/home/alo notes%C3.txt",
        "alo-chooses-file /var/home/alo notes%G1.txt",
        "alo-chooses-file /var/home/alo notes%4",
        "alo-chooses-file /var/home/alo notes%",
        "alo-chooses-file /var/home/%FF notes.txt",
        "alo-chooses-file /var/home/al%ZZo notes.txt",
    ] {
        let heard = ToAnswerAFileChooser::read(line);
        assert!(
            matches!(heard, Some(Err(_))),
            "{line:?} was read as {heard:?}"
        );
    }
}

/// A line of another act is not this act's: it is not answered, and not
/// refused as this act either.
#[test]
fn a_line_of_another_act_is_not_this_act() {
    for line in [
        "",
        "choose-file org.gnome.TextEditor open /var/home/alo",
        "open-file org.gnome.TextEditor /var/home/alo/notes.txt",
        "front",
        "chosen org.gnome.TextEditor",
        " alo-chooses-file /var/home/alo notes.txt",
    ] {
        assert_eq!(ToAnswerAFileChooser::read(line), None, "{line:?}");
    }
}

/// A word that only begins with the act's letters is never read as the act
/// answered.
#[test]
fn a_longer_word_is_never_read_as_an_answer() {
    let heard = ToAnswerAFileChooser::read("alo-chooses-files /var/home/alo notes.txt");
    assert!(!matches!(heard, Some(Ok(_))), "read as {heard:?}");
}

/// `chosen <application>`: the identifier of the application whose sheet was
/// answered, and it reads back as itself.
#[test]
fn chosen_is_written_with_the_application_and_reads_back() {
    let answer = AnsweredTheChooser::Chosen(asking());
    assert_eq!(answer.written(), "chosen org.gnome.TextEditor");
    assert_eq!(AnsweredTheChooser::read(&answer.written()), Some(answer));
}

/// `no-sheet` and `not-believed` are written as the contract spells them, and
/// read back.
#[test]
fn no_sheet_and_not_believed_are_written_and_read_back() {
    assert_eq!(AnsweredTheChooser::NoSheet.written(), "no-sheet");
    assert_eq!(AnsweredTheChooser::NotBelieved.written(), "not-believed");
    assert_eq!(
        AnsweredTheChooser::read("no-sheet"),
        Some(AnsweredTheChooser::NoSheet)
    );
    assert_eq!(
        AnsweredTheChooser::read("not-believed"),
        Some(AnsweredTheChooser::NotBelieved)
    );
}

/// `chosen` with no application, or with something that is not an
/// application's identifier, is never read as a choice.
#[test]
fn chosen_without_an_application_is_not_a_choice() {
    for heard in [
        "chosen",
        "chosen ",
        "chosen  org.gnome.TextEditor",
        "chosen org.gnome.TextEditor extra",
        "chosen not an identifier",
        "chosen /var/home/alo/notes.txt",
        "chosen ../../etc",
    ] {
        let read = AnsweredTheChooser::read(heard);
        assert!(
            !matches!(read, Some(AnsweredTheChooser::Chosen(_))),
            "{heard:?} was read as {read:?}"
        );
    }
}

/// An answer the contract does not name — the portal's own `cancelled` or
/// `unanswered`, a misspelling, a different case, extra words — is no answer.
#[test]
fn an_unknown_answer_is_no_answer() {
    for heard in [
        "",
        "cancelled",
        "unanswered",
        "cannot-show",
        "No-Sheet",
        "NOT-BELIEVED",
        "no sheet",
        "no-sheet extra",
        "not-believed because",
        "chose org.gnome.TextEditor",
        "yes",
    ] {
        assert_eq!(AnsweredTheChooser::read(heard), None, "{heard:?}");
    }
}
