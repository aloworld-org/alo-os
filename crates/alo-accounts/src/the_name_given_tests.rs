//! The name file's place and shape, held to every case on any host.

use super::*;

/// A folder of this test's own, emptied of any earlier run.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-names-{what}-{}", std::process::id()));
    let _cleared = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// **The place is named from the top of the disk**, asked of the text rather
/// than `Path::is_absolute`, which answers about the host a test runs on.
#[test]
fn the_names_are_kept_under_var_lib() {
    assert!(THE_NAMES.starts_with("/var/lib/"), "{THE_NAMES}");
    assert_eq!(THE_NAMES_MODE, 0o711);
    assert_eq!(A_NAMES_MODE, 0o600);
}

/// **One file per account number**, named by the number and nothing else.
#[test]
fn a_persons_name_is_a_file_named_by_their_number() {
    assert_eq!(
        where_their_name_is(Path::new("/var/lib/alo-names"), 1000),
        Path::new("/var/lib/alo-names").join("1000")
    );
}

/// **What is written reads back as itself**, in any script.
#[test]
fn a_name_written_reads_back_as_itself() {
    for name in ["anna", "Ánna-María", "سارة", "שרה", "Αλέξανδρος", "李华"] {
        assert_eq!(as_read(&as_written(name)).as_deref(), Some(name), "{name}");
    }
}

/// **An empty file, or one line ending alone, is no name** — not an empty one.
#[test]
fn an_empty_file_is_no_name() {
    for empty in ["", "\n", "\r\n"] {
        assert_eq!(as_read(empty), None, "{empty:?}");
    }
}

/// **A control character is not part of a name.** A terminal escape drawn
/// into the strip, or a second line, would be something nobody typed.
#[test]
fn a_file_with_a_control_character_is_not_a_name() {
    for wrong in ["an\nna\n", "anna\u{1b}[31m", "an\tna", "anna\n\n", "\u{7f}"] {
        assert_eq!(as_read(wrong), None, "{wrong:?}");
    }
}

/// **A file longer than a name may be is not cut to one**, it is refused.
#[test]
fn a_name_longer_than_the_bound_is_refused_rather_than_cut() {
    let at_the_bound = "a".repeat(THE_LONGEST_NAME);
    assert_eq!(
        as_read(&as_written(&at_the_bound)).as_deref(),
        Some(at_the_bound.as_str())
    );
    let beyond = "a".repeat(THE_LONGEST_NAME + 1);
    assert_eq!(as_read(&as_written(&beyond)), None);
}

/// **No file is no name, and not a failure**: the state of every account
/// whose person has not signed in since this was built.
#[test]
fn no_file_is_no_name_and_not_a_failure() {
    let under = a_folder_of_our_own("none");
    assert!(their_name(&under, 1000).unwrap().is_none());
}

/// **A file is read back from the disk**, by the reader the desktop uses.
#[test]
fn a_file_on_the_disk_is_read_back() {
    let under = a_folder_of_our_own("one");
    std::fs::write(where_their_name_is(&under, 1000), as_written("anna")).unwrap();
    assert_eq!(their_name(&under, 1000).unwrap().as_deref(), Some("anna"));
    assert!(
        their_name(&under, 1001).unwrap().is_none(),
        "another number read this person's name"
    );
}

/// **A file holding something that is not a name is refused by name**, so the
/// desktop's journal says why the strip has no name in it.
#[test]
fn a_file_that_is_not_a_name_is_refused() {
    let under = a_folder_of_our_own("wrong");
    let at = where_their_name_is(&under, 1000);

    std::fs::write(&at, "a".repeat(THE_LONGEST_NAME * 4)).unwrap();
    assert!(matches!(
        their_name(&under, 1000),
        Err(NameNotRead::NotAName { .. })
    ));

    std::fs::write(&at, [0xff, 0xfe, b'\n']).unwrap();
    assert!(matches!(
        their_name(&under, 1000),
        Err(NameNotRead::NotAName { .. })
    ));
}
