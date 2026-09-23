//! **The menu a machine with two systems on it starts at**, as
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
//! decided it: the base's own loader, configured and never changed, offering
//! alo OS and Windows, a short countdown, and the last choice kept in the
//! loader's own environment block.
//!
//! The unit tests beside `src/menu.rs` hold each piece. This is the
//! acceptance: the whole file as a machine would write it, read back as the
//! decision reads.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic naming what could not be read is the failure being reported"
)]

use alo_starting::{
    LONGEST_TITLE, Menu, NotAMenu, SAVED_ENTRY, THE_BLOCK_ON_THE_ESP, THE_COUNTDOWN, THE_MENU,
    THE_WINDOWS_ENTRY, THE_WINDOWS_LOADER, words,
};

/// The menu a machine would write, with the title from the machine's own
/// vocabulary.
fn the_menu() -> String {
    Menu::offering(words::THE_WINDOWS_ENTRY_TITLE.says(), THE_COUNTDOWN)
        .expect("the menu's own title is one the menu can be written with")
        .written()
}

/// **It offers Windows, by handing Windows's own program to the firmware.**
///
/// ADR 0062's option A, in the one line it comes down to: alo OS's loader
/// stands in front, and Windows is started by chain-loading the signed program
/// Windows installed for itself. Nothing here signs, copies or replaces it.
#[test]
fn it_offers_windows_by_handing_windowss_own_program_over() {
    let written = the_menu();
    assert!(
        written.contains(&format!(
            "menuentry '{}'",
            words::THE_WINDOWS_ENTRY_TITLE.says()
        )),
        "{written}"
    );
    assert!(
        written.contains(&format!("chainloader {THE_WINDOWS_LOADER}")),
        "{written}"
    );
    assert!(
        written.contains(&format!("--id '{THE_WINDOWS_ENTRY}'")),
        "{written}"
    );
}

/// **It counts down, visibly.** A menu with no countdown is a machine that
/// waits for somebody; a menu that is never drawn is not a menu. ADR 0062 asks
/// for a short one, and a short one is what this is.
#[test]
fn it_counts_down_visibly_and_briefly() {
    let written = the_menu();
    assert!(written.contains("set timeout_style=menu"), "{written}");
    assert!(
        written.contains(&format!("set timeout={THE_COUNTDOWN}")),
        "{written}"
    );
    assert!((1..=30).contains(&THE_COUNTDOWN), "{THE_COUNTDOWN}");
}

/// **It starts at the last choice, and writes the choice where it read it.**
/// One place, read and written by the loader itself — ADR 0062's third term —
/// and since [ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)
/// term 1 that place is named in full, on the partition both systems share,
/// rather than left to the loader's own directory under `/boot`.
#[test]
fn it_starts_at_the_last_choice_and_saves_the_next_one() {
    let written = the_menu();
    let block = format!("(${{esp}}){THE_BLOCK_ON_THE_ESP}");
    assert!(
        written.contains(&format!("load_env -f {block} {SAVED_ENTRY}")),
        "{written}"
    );
    assert!(
        written.contains(&format!("set default=\"${{{SAVED_ENTRY}}}\"")),
        "{written}"
    );
    assert!(
        written.contains(&format!("set {SAVED_ENTRY}='{THE_WINDOWS_ENTRY}'")),
        "{written}"
    );
    assert!(
        written.contains(&format!("save_env -f {block} {SAVED_ENTRY}")),
        "{written}"
    );
}

/// **The partition holding the last choice is found by that file**, at every
/// start, and never by an identifier of somebody's disk kept in this one.
///
/// The same reasoning the Windows entry is written under: an identifier learned
/// once and never checked again is a second copy of a fact about a machine, and
/// this crate exists to have one copy of that fact.
#[test]
fn the_partition_is_found_by_the_file_rather_than_by_an_identifier() {
    let written = the_menu();
    assert!(
        written.contains(&format!(
            "search --no-floppy --set=esp --file {THE_BLOCK_ON_THE_ESP}"
        )),
        "{written}"
    );
    for never in ["--fs-uuid", "--label", "hd0", "(hd", "/dev/"] {
        assert!(!written.contains(never), "{never} is named in {written}");
    }
}

/// **It is a file of ours beside the base's, not an edit to the base's.**
///
/// [ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md):
/// the base is configured, never patched. The file is written whole, says so in
/// its own first lines, and writes no alo OS entry of its own — those are the
/// base's, made from the kernels on the machine, and a copy of one here would
/// be wrong at the next update.
#[test]
fn it_is_configuration_beside_the_base_rather_than_a_change_to_it() {
    let written = the_menu();
    let first = written
        .lines()
        .take(2)
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase();
    assert!(first.contains("generated"), "{first}");
    assert!(first.contains("never edited"), "{first}");

    assert!(THE_MENU.ends_with("/custom.cfg"), "{THE_MENU}");
    for never in [
        "linux ",
        "initrd ",
        "bls_import",
        "blscfg",
        "menuentry 'alo",
    ] {
        assert!(
            !written.contains(never),
            "the menu writes {never:?}, which is the base's to write: {written}"
        );
    }
    assert_eq!(
        written.matches("menuentry ").count(),
        1,
        "the menu writes more than the one entry that is ours: {written}"
    );
}

/// **A title that would break the file is refused rather than written.**
///
/// The title comes from a translation, which is a file somebody else wrote, and
/// the failure it would cause is the worst kind this crate can have: a machine
/// that starts at a loader's prompt, where there is no browser to look the
/// answer up in and no Windows to go back to without knowing a key.
#[test]
fn a_title_that_would_break_the_file_is_refused() {
    for title in [
        "",
        "Wind'ows",
        "Windows ${x}",
        &"W".repeat(LONGEST_TITLE + 1),
    ] {
        assert!(
            matches!(
                Menu::offering(title, THE_COUNTDOWN),
                Err(NotAMenu::NotATitle { .. })
            ),
            "{title:?} was written into the menu"
        );
    }
    assert!(matches!(
        Menu::offering("Windows", 0),
        Err(NotAMenu::NotACountdown { .. })
    ));
}
