//! The keys an entry is started by, read from entries shaped like real ones.

use super::*;

/// Dolphin's own entry, abridged: no bus activation, an `Exec` with `%u`.
const DOLPHIN: &str = "[Desktop Entry]
Type=Application
Exec=dolphin %u
Icon=system-file-manager
Name=Dolphin
GenericName=File Manager

[Desktop Action new-window]
Name=Open a New Window
Exec=dolphin --new-window
";

#[test]
fn the_entry_group_is_read_and_an_action_is_not() {
    let read = TheEntry::read(DOLPHIN);
    assert_eq!(
        read,
        Some(TheEntry {
            kind: Some("Application".to_owned()),
            name: Some("Dolphin".to_owned()),
            exec: Some("dolphin %u".to_owned()),
            icon: Some("system-file-manager".to_owned()),
            ..TheEntry::default()
        }),
        "the action's Exec and Name belong to a right-click entry, not to the application"
    );
}

#[test]
fn the_flags_are_true_only_when_they_say_true() {
    let read = TheEntry::read(
        "[Desktop Entry]\nDBusActivatable=true\nTerminal=false\nHidden=true\nPath=/srv\n",
    );
    assert_eq!(
        read.map(|it| (it.bus_activatable, it.terminal, it.hidden, it.folder)),
        Some((true, false, true, Some("/srv".to_owned())))
    );
}

#[test]
fn a_localised_exec_is_not_the_exec() {
    let read = TheEntry::read("[Desktop Entry]\nExec[de]=nicht\n");
    assert_eq!(read.and_then(|it| it.exec), None);
}

#[test]
fn a_file_with_no_entry_group_is_not_an_entry() {
    assert_eq!(TheEntry::read("[Something Else]\nExec=x\n"), None);
    assert_eq!(TheEntry::read(""), None);
}
