//! The road, read off entries written to a folder of this test's own.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use super::*;

/// A folder holding these entries.
fn a_place(called: &str, entries: &[(&str, &str)]) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-agentd-starting-{}-{called}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for (named, text) in entries {
        std::fs::write(at.join(named), text).unwrap();
    }
    at
}

#[test]
fn each_entry_names_its_own_road_or_none() {
    let place = a_place(
        "roads",
        &[
            (
                "org.kde.dolphin.desktop",
                "[Desktop Entry]\nType=Application\nName=Dolphin\nExec=dolphin %u\n",
            ),
            (
                "org.gnome.TextEditor.desktop",
                "[Desktop Entry]\nType=Application\nName=Text Editor\nDBusActivatable=true\n\
                 Exec=gnome-text-editor %U\n",
            ),
            (
                "org.example.Nothing.desktop",
                "[Desktop Entry]\nType=Application\nName=Nothing\n",
            ),
            (
                "org.example.Broken.desktop",
                "[Desktop Entry]\nType=Application\nName=Broken\nExec=app \"never closed\n",
            ),
        ],
    );
    let here = HowApplicationsStartHere::in_these(vec![place]);
    assert_eq!(
        here.how_it_starts("org.kde.dolphin"),
        Ok(TheRoad::FromItsCommand)
    );
    assert_eq!(
        here.how_it_starts("org.gnome.TextEditor"),
        Ok(TheRoad::OverTheBus)
    );
    assert_eq!(
        here.how_it_starts("org.example.Nothing"),
        Err(NoRoad::NothingStartsIt),
        "no Exec and no bus is refused, not run"
    );
    assert_eq!(
        here.how_it_starts("org.example.Broken"),
        Err(NoRoad::NothingStartsIt),
        "a malformed Exec is refused, not run"
    );
    assert_eq!(here.how_it_starts("org.example.Gone"), Err(NoRoad::NoEntry));
}
