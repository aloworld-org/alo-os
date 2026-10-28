//! Which folder a shell starts in, for every kind of home a session can name.
//!
//! The ordinary case first, then each refusal: a refusal here is the case where
//! a person would otherwise be dropped somewhere without a word, so each one is
//! held to its own reason rather than to *not their home* in general.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::path::Path;

use super::{WhereItStarts, WhyNotHome};

/// The ordinary case: a home that is a folder is where the shell starts.
#[test]
fn a_home_that_is_a_folder_is_where_the_shell_starts() {
    let home = tempfile::tempdir().expect("a folder to be a home");
    let starts = WhereItStarts::for_a_home(Some(home.path().as_os_str()));
    assert_eq!(starts, WhereItStarts::TheirHome(home.path().to_path_buf()));
    assert_eq!(starts.the_folder(), home.path());
}

/// No home named at all: the top of the disk, with the reason kept.
#[test]
fn a_session_that_names_no_home_starts_at_the_top_of_the_disk_and_says_so() {
    let starts = WhereItStarts::for_a_home(None);
    assert_eq!(
        starts,
        WhereItStarts::NotTheirHome(WhyNotHome::NoneWasNamed)
    );
    assert_eq!(starts.the_folder(), Path::new("/"));
}

/// An empty `$HOME` is treated as none, not as the current folder.
#[test]
fn an_empty_home_is_no_home() {
    assert_eq!(
        WhereItStarts::for_a_home(Some(OsStr::new(""))),
        WhereItStarts::NotTheirHome(WhyNotHome::NoneWasNamed)
    );
}

/// A relative home would depend on where the compositor stands, so it is refused.
#[test]
fn a_home_that_is_not_a_full_path_is_refused_by_name() {
    let starts = WhereItStarts::for_a_home(Some(OsStr::new("alo")));
    assert_eq!(
        starts,
        WhereItStarts::NotTheirHome(WhyNotHome::NotAFullPath("alo".into()))
    );
    assert_eq!(starts.the_folder(), Path::new("/"));
}

/// The refusal the boot walk is most likely to meet: a home nobody made.
#[test]
fn a_home_that_does_not_exist_is_refused_by_name() {
    let somewhere = tempfile::tempdir().expect("a folder to be missing from");
    let missing = somewhere.path().join("nobody-made-this");
    let starts = WhereItStarts::for_a_home(Some(missing.as_os_str()));
    assert_eq!(
        starts,
        WhereItStarts::NotTheirHome(WhyNotHome::NotAFolder(missing))
    );
    assert_eq!(starts.the_folder(), Path::new("/"));
}

/// A file where the home should be is not a folder to start in.
#[test]
fn a_home_that_is_a_file_is_refused_by_name() {
    let file = tempfile::NamedTempFile::new().expect("a file where a home should be");
    assert_eq!(
        WhereItStarts::for_a_home(Some(file.path().as_os_str())),
        WhereItStarts::NotTheirHome(WhyNotHome::NotAFolder(file.path().to_path_buf()))
    );
}

/// **The reason names the home it refused**, so the line on the error stream
/// is one somebody mending the machine can act on.
#[test]
fn the_reason_names_the_folder_it_could_not_use() {
    let said = WhyNotHome::NotAFolder("/var/home/alo".into()).to_string();
    assert!(said.contains("/var/home/alo"), "{said}");
}
