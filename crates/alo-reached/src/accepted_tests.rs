//! The list, read and written.

use super::Accepted;

/// **Comments and blank lines are not entries.**
#[test]
fn the_header_is_not_a_list_entry() {
    let read = Accepted::read("# a comment\n\n\ncrates/a/src/b.rs\tnamed\n");
    assert_eq!(read.lines(), ["crates/a/src/b.rs\tnamed"]);
}

/// **A missing list reports everything**, which is the safe direction.
///
/// The unsafe direction would be a missing list read as *everything is
/// accepted*, and a gate that passes when its own list is gone is not a gate.
#[test]
fn an_absent_list_accepts_nothing() {
    assert_eq!(Accepted::read("").lines(), [] as [String; 0]);
}

/// **The file is sorted and deduplicated**, so two runs of one tree write the
/// same bytes.
#[test]
fn what_is_written_is_sorted_and_says_how_to_regenerate_itself() {
    let written = Accepted::as_written(&[
        "crates/z/src/a.rs\tlast".to_owned(),
        "crates/a/src/a.rs\tfirst".to_owned(),
        "crates/a/src/a.rs\tfirst".to_owned(),
    ]);
    let entries: Vec<&str> = written
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .collect();
    assert_eq!(
        entries,
        ["crates/a/src/a.rs\tfirst", "crates/z/src/a.rs\tlast"],
        "sorted, and the repeat collapsed"
    );
    assert!(
        written.contains("UPDATE_REACHED_SNAPSHOT=1"),
        "the file says how to regenerate itself, or somebody edits it by hand"
    );
    assert!(
        written.contains("MAY NOT GROW SILENTLY"),
        "and it says the asymmetry, which is the whole design"
    );
}

/// **Reading what was written gives back what went in.**
#[test]
fn a_round_trip_keeps_every_entry() {
    let lines = vec![
        "crates/a/src/a.rs\tone".to_owned(),
        "crates/b/src/b.rs\ttwo".to_owned(),
    ];
    let read = Accepted::read(&Accepted::as_written(&lines));
    assert_eq!(read.lines(), lines.as_slice());
}
