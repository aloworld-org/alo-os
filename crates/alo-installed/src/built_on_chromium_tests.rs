//! Packages laid out as flatpak lays them out, some carrying Chromium's own
//! resource file, read back for whether they are drawn by Chromium.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::*;

/// A directory of the test's own, emptied.
fn a_place(named: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "alo-installed-chromium-{}-{named}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// The package's files, where flatpak keeps them.
fn files_of(installation: &Path, identifier: &str) -> PathBuf {
    let at = installation
        .join("app")
        .join(identifier)
        .join("current")
        .join("active")
        .join("files");
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// Put a file at `inside`, below a package's files.
fn a_file(files: &Path, inside: &str) {
    let at = files.join(inside);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, b"").unwrap();
}

/// **A package carrying Chromium's resource file is drawn by Chromium**,
/// wherever its build put it: a browser that downloads itself at install
/// (`files/extra`, as Chrome and Edge do), one built into its own folder (as
/// Brave is), and an Electron application on no base of Electron's — in the
/// machine's installation or the person's own.
#[test]
fn a_package_carrying_chromiums_resources_is_drawn_by_chromium() {
    let machines = a_place("machines");
    let persons = a_place("persons");
    a_file(
        &files_of(&machines, "com.google.Chrome"),
        "extra/opt/google/chrome/chrome_100_percent.pak",
    );
    a_file(
        &files_of(&machines, "com.brave.Browser"),
        "brave/chrome_100_percent.pak",
    );
    a_file(
        &files_of(&persons, "md.obsidian.Obsidian"),
        "obsidian/chrome_100_percent.pak",
    );
    for identifier in [
        "com.google.Chrome",
        "com.brave.Browser",
        "md.obsidian.Obsidian",
    ] {
        assert!(
            is_built_on_chromium(&machines, Some(&persons), identifier),
            "{identifier} was not known as Chromium"
        );
    }
}

/// **Nothing that is not Chromium is handed Chromium's switch**: a GTK
/// application, a Qt one carrying its web engine's own resources (which are
/// Chromium inside but read no switch of Chromium's), a file whose name only
/// begins like Chromium's, a directory of that name, one deeper than is looked,
/// a package with no files, no package, and an identifier that could name
/// another place.
#[test]
fn nothing_that_is_not_chromium_is_read_as_chromium() {
    let machines = a_place("not-chromium");
    a_file(
        &files_of(&machines, "org.gnome.TextEditor"),
        "bin/gnome-text-editor",
    );
    a_file(
        &files_of(&machines, "org.kde.falkon"),
        "share/qt6/resources/qtwebengine_resources_100p.pak",
    );
    a_file(
        &files_of(&machines, "org.example.Lookalike"),
        "share/chrome_100_percent.pak.txt",
    );
    std::fs::create_dir_all(
        files_of(&machines, "org.example.Folder").join("chrome_100_percent.pak"),
    )
    .unwrap();
    a_file(
        &files_of(&machines, "org.example.Deep"),
        "a/b/c/d/e/f/g/chrome_100_percent.pak",
    );
    files_of(&machines, "org.kde.dolphin");
    a_file(&a_place("elsewhere"), "chrome_100_percent.pak");
    for identifier in [
        "org.gnome.TextEditor",
        "org.kde.falkon",
        "org.example.Lookalike",
        "org.example.Folder",
        "org.example.Deep",
        "org.kde.dolphin",
        "org.example.NotInstalled",
        "",
        "..",
        "../../elsewhere",
        "org.example/../..",
    ] {
        assert!(
            !is_built_on_chromium(&machines, None, identifier),
            "{identifier:?} was read as Chromium"
        );
    }
}

/// **A link is never followed**: a package pointing at Chromium's file
/// somewhere else is not, by that, drawn by Chromium.
#[cfg(unix)]
#[test]
fn a_link_to_chromiums_file_elsewhere_is_not_followed() {
    let machines = a_place("linked");
    let elsewhere = a_place("linked-elsewhere");
    a_file(&elsewhere, "chrome/chrome_100_percent.pak");
    let files = files_of(&machines, "org.example.Linked");
    std::os::unix::fs::symlink(elsewhere.join("chrome"), files.join("chrome")).unwrap();
    std::os::unix::fs::symlink(
        elsewhere.join("chrome/chrome_100_percent.pak"),
        files.join("chrome_100_percent.pak"),
    )
    .unwrap();
    assert!(!is_built_on_chromium(&machines, None, "org.example.Linked"));
}

/// **And the one question every caller asks says so**: a Chromium browser is
/// drawn by Chromium, so the desktop hands it the switch that describes its
/// page; a GTK application beside it is not, and is handed nothing.
#[test]
fn the_question_every_caller_asks_knows_a_browser_by_what_it_is_built_on() {
    let machines = a_place("asked");
    a_file(
        &files_of(&machines, "com.microsoft.Edge"),
        "extra/microsoft-edge/chrome_100_percent.pak",
    );
    a_file(&files_of(&machines, "org.gnome.Loupe"), "bin/loupe");
    assert!(crate::is_drawn_by_chromium_under(
        &machines,
        None,
        "com.microsoft.Edge"
    ));
    assert!(!crate::is_drawn_by_chromium_under(
        &machines,
        None,
        "org.gnome.Loupe"
    ));
}

/// **The look stops where its bounds say, to the entry and to the folder.**
/// One entry in each folder, so the order entries are read in is the only
/// one there is: the package's files hold `a`, `a` holds `b`, and `b` holds
/// Chromium's file — the third entry read, two folders down.
#[test]
fn the_look_stops_at_its_bounds_exactly() {
    let files = files_of(&a_place("bounded"), "org.example.Bounded");
    a_file(&files, "a/b/chrome_100_percent.pak");
    assert!(
        carries_chromiums_file_within(&files, 3, 3),
        "found within both bounds"
    );
    assert!(
        !carries_chromiums_file_within(&files, 2, 3),
        "read past the most entries"
    );
    assert!(
        !carries_chromiums_file_within(&files, 3, 2),
        "looked deeper than the deepest folder"
    );
}
