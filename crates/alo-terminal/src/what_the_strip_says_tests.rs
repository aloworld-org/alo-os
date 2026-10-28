//! How a folder is written in the terminal's strip, held to its cases.
//!
//! Every case that abbreviates has a control beside it that must not, because a
//! function that wrote `~` in front of everything would pass the first half of
//! this file alone.

use std::path::Path;

use super::{WhereTheShellRuns, the_folder_as_a_person_reads_it};

/// The home this file's cases are about.
const HOME: &str = "/home/disan";

/// The folder as the strip writes it, with [`HOME`] as the home.
fn read(folder: &str) -> String {
    the_folder_as_a_person_reads_it(Path::new(folder), Some(Path::new(HOME)))
}

#[test]
fn the_home_itself_is_a_tilde() {
    assert_eq!(read("/home/disan"), "~");
}

#[test]
fn a_folder_inside_the_home_is_written_from_the_tilde() {
    assert_eq!(read("/home/disan/Projects/launch"), "~/Projects/launch");
}

#[test]
fn a_folder_outside_the_home_is_written_in_full() {
    assert_eq!(read("/etc/alo"), "/etc/alo");
    assert_eq!(read("/"), "/");
}

/// **A neighbour whose name begins with the home's is not inside it.** The
/// fault a comparison of text rather than of folder names would make.
#[test]
fn a_neighbour_sharing_the_homes_first_letters_is_not_inside_it() {
    assert_eq!(read("/home/disandra"), "/home/disandra");
    assert_eq!(read("/home/disandra/notes"), "/home/disandra/notes");
}

/// **A home of `/` abbreviates nothing**, because every folder is inside it and
/// `~/etc` would be a lie about where the person is.
#[test]
fn a_home_at_the_top_of_the_disk_abbreviates_nothing() {
    let at_the_top =
        |folder: &str| the_folder_as_a_person_reads_it(Path::new(folder), Some(Path::new("/")));
    assert_eq!(at_the_top("/etc"), "/etc");
    assert_eq!(at_the_top("/"), "/");
}

#[test]
fn with_no_home_known_every_folder_is_written_in_full() {
    assert_eq!(
        the_folder_as_a_person_reads_it(Path::new("/home/disan/Projects"), None),
        "/home/disan/Projects"
    );
}

/// The one place the shell runs today says so in a word from the vocabulary,
/// not in English written here.
#[test]
fn a_shell_on_this_machine_is_said_with_the_vocabularys_word() {
    assert_eq!(
        WhereTheShellRuns::OnThisMachine.the_word(),
        crate::words::ON_THIS_MACHINE
    );
}
