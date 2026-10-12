//! What pinning and unpinning change, and what each owes the disk.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_dock::pinning::{ARole, Discovery};
use alo_dock::{ADockEntry, ANativeAction, AppId};

use crate::Server;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let bound = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, bound)
}

fn app(named: &str) -> AppId {
    AppId::named(named).expect("a plain identifier is an application")
}

/// A Dock already holding Files and the Settings entry.
fn a_dock_with_files_and_settings(named: &str) -> (tempfile::TempDir, Server) {
    let (directory, mut server) = server(named);
    server.its_dock_is_filled_from(&Discovery::Completed(vec![(
        ARole::Files,
        app("org.kde.dolphin"),
    )]));
    (directory, server)
}

/// What is pinned now, by name.
fn pinned(server: &Server) -> Vec<String> {
    server
        .what_the_dock_is_pinned_to()
        .each_pinned()
        .map(|entry| match entry {
            ADockEntry::Application(app) => app.name().to_owned(),
            ADockEntry::Native(ANativeAction::Settings) => "Settings".to_owned(),
        })
        .collect()
}

/// **Pinning adds it at the end and owes the disk the whole list.**
///
/// The whole list rather than the change, because a write replaces the file:
/// what is owed is what the file should say, which is the shape
/// `keep_these_pins` already takes.
#[test]
fn pinning_an_application_adds_it_and_owes_a_write() {
    let (_directory, mut server) = a_dock_with_files_and_settings("pinning-adds");

    let owed = server
        .this_application_is_pinned(&app("org.mozilla.firefox"))
        .expect("pinning something new owes a write");

    assert_eq!(
        pinned(&server),
        vec!["org.kde.dolphin", "Settings", "org.mozilla.firefox"],
        "pinning did not put the application at the end of what the person had"
    );
    assert_eq!(
        owed,
        server
            .what_the_dock_is_pinned_to()
            .each_pinned()
            .cloned()
            .collect::<Vec<_>>(),
        "what is owed to the disk is not what the Dock now holds"
    );
}

/// **Pinning something already pinned changes nothing and owes nothing.**
///
/// `Holding::pin_entry`'s rule, and the reason it has one: an icon that moved
/// because somebody pinned it twice is the instability that file exists to
/// prevent. Owing a write here would also rewrite `dock.toml` for a press that
/// did nothing.
#[test]
fn pinning_what_is_already_pinned_owes_nothing() {
    let (_directory, mut server) = a_dock_with_files_and_settings("pinning-twice");
    let before = pinned(&server);

    assert_eq!(
        server.this_application_is_pinned(&app("org.kde.dolphin")),
        None,
        "pinning something already pinned owed a write"
    );
    assert_eq!(pinned(&server), before, "it moved");
}

/// **Unpinning takes it off and owes a write.**
#[test]
fn unpinning_an_application_takes_it_off_and_owes_a_write() {
    let (_directory, mut server) = a_dock_with_files_and_settings("unpinning");

    let owed = server
        .this_entry_is_unpinned(&ADockEntry::Application(app("org.kde.dolphin")))
        .expect("unpinning something pinned owes a write");

    assert_eq!(pinned(&server), vec!["Settings"]);
    assert_eq!(owed.len(), 1);
}

/// **The Settings entry comes off exactly as an application does.**
///
/// The owner's ruling of 2026-10-12: *native describes the implementation; it
/// does not mean permanent.* Law 5 is what gives it teeth — a change that takes
/// a choice away is a bug **whatever the reason given for it**, and the reason
/// here would be a good one: Settings always works if Settings is always there.
///
/// Asserted beside an application's removal in the same file rather than in a
/// test of its own, so the native half cannot be deleted as redundant by
/// somebody who read only the other.
#[test]
fn the_settings_entry_can_be_unpinned_like_anything_else() {
    let (_directory, mut server) = a_dock_with_files_and_settings("unpinning-settings");

    let owed = server
        .this_entry_is_unpinned(&ADockEntry::Native(ANativeAction::Settings))
        .expect("a native entry refused to come off, so it is permanent");

    assert_eq!(pinned(&server), vec!["org.kde.dolphin"]);
    assert_eq!(owed.len(), 1);
}

/// **Unpinning what is not pinned changes nothing and owes nothing.**
#[test]
fn unpinning_what_is_not_pinned_owes_nothing() {
    let (_directory, mut server) = a_dock_with_files_and_settings("unpinning-absent");
    let before = pinned(&server);

    assert_eq!(
        server.this_entry_is_unpinned(&ADockEntry::Application(app("org.mozilla.firefox"))),
        None,
        "unpinning something that was not pinned owed a write"
    );
    assert_eq!(pinned(&server), before);
}

/// **A person who empties their Dock is owed the write that keeps it empty.**
///
/// The clause the whole saved shape exists for. An empty list a person made is
/// a choice and stays one — and refusing to write it here, on the grounds that
/// there is nothing to write, is exactly how the defaults would come back at
/// their next sign-in.
#[test]
fn emptying_the_dock_owes_a_write_of_the_empty_list() {
    let (_directory, mut server) = a_dock_with_files_and_settings("emptying");

    server
        .this_entry_is_unpinned(&ADockEntry::Application(app("org.kde.dolphin")))
        .expect("the first comes off");
    let owed = server
        .this_entry_is_unpinned(&ADockEntry::Native(ANativeAction::Settings))
        .expect("the last one off owes a write like any other");

    assert!(
        owed.is_empty(),
        "what is owed is not the empty list the person made: {owed:?}"
    );
    assert!(pinned(&server).is_empty());
}

/// **A change is owed once and taken once.**
///
/// `one approval causes exactly one execution` is the capability model's rule
/// and it is this one too: a list read rather than taken would be written on
/// every frame for ever after a single press, which is a person's settings file
/// rewritten sixty times a second.
#[test]
fn a_change_is_owed_once_and_taken_once() {
    let (_directory, mut server) = a_dock_with_files_and_settings("owed-once");
    server
        .this_application_is_pinned(&app("org.mozilla.firefox"))
        .expect("the premise");

    let first = server.take_the_pins_owed_to_the_disk();

    assert!(first.is_some(), "the change was not waiting to be written");
    assert_eq!(
        server.take_the_pins_owed_to_the_disk(),
        None,
        "the same change is owed twice, so the file would be rewritten every frame"
    );
}

/// **Two changes before a frame leave one write, of what the Dock holds after
/// both.**
///
/// Each answer is the whole list rather than a change to it, so the last one
/// wins and is correct. A queue of changes would have to be replayed in order
/// against a file somebody may have edited meanwhile; one list does not.
#[test]
fn two_changes_before_a_frame_leave_one_write_of_the_end_state() {
    let (_directory, mut server) = a_dock_with_files_and_settings("two-changes");

    server
        .this_application_is_pinned(&app("org.mozilla.firefox"))
        .expect("the first");
    server
        .this_entry_is_unpinned(&ADockEntry::Application(app("org.kde.dolphin")))
        .expect("the second");

    let owed = server
        .take_the_pins_owed_to_the_disk()
        .expect("something is owed");
    assert_eq!(
        owed,
        vec![
            ADockEntry::Native(ANativeAction::Settings),
            ADockEntry::Application(app("org.mozilla.firefox")),
        ],
        "what is written is not the Dock as it stands after both changes"
    );
    assert_eq!(
        owed,
        server
            .what_the_dock_is_pinned_to()
            .each_pinned()
            .cloned()
            .collect::<Vec<_>>(),
        "what is written and what is drawn have come apart"
    );
}

/// **A change that does nothing owes nothing**, so a press that changed nothing
/// does not rewrite a person's file.
#[test]
fn a_change_that_does_nothing_leaves_nothing_owed() {
    let (_directory, mut server) = a_dock_with_files_and_settings("nothing-owed");

    assert_eq!(
        server.this_application_is_pinned(&app("org.kde.dolphin")),
        None
    );
    assert_eq!(
        server.take_the_pins_owed_to_the_disk(),
        None,
        "a press that changed nothing asked for a write"
    );
}
