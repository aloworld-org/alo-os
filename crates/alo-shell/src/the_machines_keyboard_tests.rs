//! What a machine decides its keyboard is, read from the layout it would type
//! through rather than from a call returning `Ok`.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected variant is the failure being reported"
)]

use super::{WhoSaidSo, the_keyboard_this_machine_has};

/// A folder this test alone writes in, emptied of whatever a previous run left.
fn a_folder_of_our_own(what: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-shell-machine-keyboard-{what}"));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// This machine's file, with these lines in it.
fn a_machine_whose_file_says(what: &str, lines: &str) -> std::path::PathBuf {
    let at = a_folder_of_our_own(what).join(alo_keyboards::keeping::THE_FILE);
    std::fs::write(&at, lines).unwrap();
    at
}

#[test]
fn a_machine_nobody_has_told_types_through_the_layout_its_unit_file_names() {
    let nothing = a_folder_of_our_own("untold").join(alo_keyboards::keeping::THE_FILE);
    let has = the_keyboard_this_machine_has(&nothing, Some("gb")).unwrap();
    assert_eq!(has.layout(), "gb");
    assert_eq!(has.variant(), "");
    assert_eq!(*has.from(), WhoSaidSo::TheUnitFile);
}

#[test]
fn a_machine_that_was_told_types_through_what_it_was_told_and_not_the_unit_file() {
    // The unit file still says `gb`, as every unit file this repository has
    // ever written does. The file has to win, or none of this changes anything.
    let at = a_machine_whose_file_says("told", "format = 1\nlayouts = [\"fr\"]\n");
    let has = the_keyboard_this_machine_has(&at, Some("gb")).unwrap();
    assert_eq!(has.layout(), "fr");
    assert_eq!(*has.from(), WhoSaidSo::ThisMachinesFile);
}

#[test]
fn a_machine_told_an_american_layout_is_not_mistaken_for_one_nobody_has_told() {
    // **The positive control for asking whether the file is there.**
    // `keeping::read` answers a missing file with the keyboards the release
    // ships, and the release ships `us` — so if existence were not checked
    // first, this case and the untold one would come back identical and the
    // unit file's word would be unreachable. The two differ only in `from`.
    let at = a_machine_whose_file_says("american", "format = 1\nlayouts = [\"us\"]\n");
    let told = the_keyboard_this_machine_has(&at, Some("gb")).unwrap();
    assert_eq!(told.layout(), "us");
    assert_eq!(*told.from(), WhoSaidSo::ThisMachinesFile);

    let untold = the_keyboard_this_machine_has(
        &a_folder_of_our_own("american-untold").join(alo_keyboards::keeping::THE_FILE),
        Some("gb"),
    )
    .unwrap();
    assert_eq!(untold.layout(), "gb");
    assert_eq!(*untold.from(), WhoSaidSo::TheUnitFile);
}

#[test]
fn a_variant_reaches_the_keymap_rather_than_being_dropped() {
    // `fr(oss)` and `fr` are different keyboards. A reader that kept only the
    // name would be the same bug as the hardcoded layout, one layer in.
    let at = a_machine_whose_file_says("variant", "format = 1\nlayouts = [\"fr(oss)\"]\n");
    let has = the_keyboard_this_machine_has(&at, Some("gb")).unwrap();
    assert_eq!(has.layout(), "fr");
    assert_eq!(has.variant(), "oss");
}

#[test]
fn the_first_of_several_kept_layouts_is_the_one_a_password_is_typed_through() {
    let at = a_machine_whose_file_says("several", "format = 1\nlayouts = [\"de\", \"gr\"]\n");
    let has = the_keyboard_this_machine_has(&at, Some("gb")).unwrap();
    assert_eq!(has.layout(), "de");
}

#[test]
fn a_file_that_did_not_read_leaves_a_sign_in_screen_standing_and_says_why() {
    // A hand edit with one mistake in it must not be a machine with no sign-in
    // screen on it. The refusal's own sentence is carried so the log says which
    // file to mend.
    let at = a_machine_whose_file_says("unreadable", "format = 1\nlayouts = \"fr\"\n");
    let has = the_keyboard_this_machine_has(&at, Some("gb")).unwrap();
    assert_eq!(has.layout(), "gb");
    match has.from() {
        WhoSaidSo::TheUnitFileBecauseTheMachinesFileDidNotRead(why) => {
            assert!(
                why.contains("keyboards.toml"),
                "the reason must name the file to mend, and said: {why}"
            );
        }
        other => panic!("an unreadable file was not reported: {other:?}"),
    }
}

#[test]
fn a_machine_with_neither_refuses_rather_than_guessing_a_layout() {
    let nothing = a_folder_of_our_own("neither").join(alo_keyboards::keeping::THE_FILE);
    assert!(the_keyboard_this_machine_has(&nothing, None).is_err());
}

#[test]
fn a_unit_file_that_names_an_empty_layout_has_named_nothing() {
    // `Environment=ALO_KEYBOARD=` reads as an empty string, and XKB given an
    // empty layout quietly takes its own default — so this has to be refused
    // here or it is a machine typing through a layout nobody chose.
    let nothing = a_folder_of_our_own("empty").join(alo_keyboards::keeping::THE_FILE);
    assert!(the_keyboard_this_machine_has(&nothing, Some("")).is_err());
    assert!(the_keyboard_this_machine_has(&nothing, Some("   ")).is_err());
}
