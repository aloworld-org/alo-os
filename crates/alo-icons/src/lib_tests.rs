//! Icon directories written to disk, searched the way a real machine is.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use super::{WhatWasFound, the_icon_called, where_icons_live};

/// Build an icon tree holding these files, and return the directory to search.
///
/// Each entry is a path relative to the icon directory — `hicolor/48x48/apps/a.png`
/// — and is created with a byte in it, because every answer here turns on a file
/// **being** there and an empty file is still a file.
///
/// **A counter and not a fingerprint of the contents**, which is this fixture's
/// second version. `alo-installed`'s keys its directory on what it holds, and
/// copying that here was wrong for a reason that crate never meets: two tests
/// here legitimately want *the same tree* — one asks it for a name that is in
/// it and the other for a name that is not — so a content key handed both the
/// same directory, and `remove_dir_all` in one raced `create_dir_all` in the
/// other. It failed as a missing path, in a different test each run, and the
/// failing count changed between two runs of an unchanged tree. A test that
/// assumes it is alone on the machine is the fault `CLAUDE.md` names; here two
/// tests assumed they were alone in one directory.
fn an_icon_tree_holding(files: &[&str]) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let mine = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let at = std::env::temp_dir().join(format!("alo-icons-{}-{mine}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    for file in files {
        let path = at.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, [0u8]).unwrap();
    }
    at
}

#[test]
fn an_application_with_one_icon_is_found_by_the_name_its_entry_gave() {
    let place = an_icon_tree_holding(&["hicolor/48x48/apps/org.gnome.Nautilus.png"]);
    let found = the_icon_called("org.gnome.Nautilus", std::slice::from_ref(&place), 48);
    assert_eq!(
        found.path(),
        Some(
            place
                .join("hicolor/48x48/apps/org.gnome.Nautilus.png")
                .as_path()
        )
    );
}

#[test]
fn an_application_nothing_is_called_is_found_as_nothing() {
    // The control at the other end: the same tree, searched for a name that is
    // not in it. Without this, the test above could pass against a lookup that
    // answers with the first file it meets whatever was asked for.
    let place = an_icon_tree_holding(&["hicolor/48x48/apps/org.gnome.Nautilus.png"]);
    assert_eq!(
        the_icon_called("firefox", &[place], 48),
        WhatWasFound::Nothing
    );
}

#[test]
fn a_name_is_matched_exactly_and_not_nearly() {
    let place = an_icon_tree_holding(&["hicolor/48x48/apps/firefox.png"]);
    assert!(matches!(
        the_icon_called("firefox", std::slice::from_ref(&place), 48),
        WhatWasFound::APicture(_)
    ));
    // A near miss points the Dock at another application's artwork, which is
    // the same reason `alo_applications::Installed` matches exactly.
    assert_eq!(
        the_icon_called("firefo", std::slice::from_ref(&place), 48),
        WhatWasFound::Nothing
    );
    assert_eq!(
        the_icon_called("firefoxx", &[place], 48),
        WhatWasFound::Nothing
    );
}

/// Case is part of the name, on the machine this runs on.
///
/// **Linux only, and the reason is the filesystem rather than the code.** This
/// crate asks the filesystem whether a path is a file, so what counts as *the
/// same name* is the filesystem's answer and not this crate's. ext4 and btrfs
/// are case-sensitive and `Firefox` is not `firefox`; NTFS is not, and the same
/// code there answers with `firefox.png` for both — which is correct behaviour
/// for that machine and would be a false failure here.
///
/// It is kept rather than deleted because alo OS is the Linux machine, and on
/// it this is a real guarantee: an icon name that differs only in case is a
/// different application's artwork.
#[test]
#[cfg(target_os = "linux")]
fn case_is_part_of_the_name_where_the_filesystem_says_it_is() {
    let place = an_icon_tree_holding(&["hicolor/48x48/apps/firefox.png"]);
    assert_eq!(
        the_icon_called("Firefox", &[place], 48),
        WhatWasFound::Nothing
    );
}

#[test]
fn the_nearest_size_at_least_as_large_is_taken() {
    let place = an_icon_tree_holding(&[
        "hicolor/16x16/apps/thing.png",
        "hicolor/32x32/apps/thing.png",
        "hicolor/64x64/apps/thing.png",
        "hicolor/256x256/apps/thing.png",
    ]);
    let found = the_icon_called("thing", std::slice::from_ref(&place), 48);
    assert_eq!(
        found.path(),
        Some(place.join("hicolor/64x64/apps/thing.png").as_path()),
        "64 is the smallest that is at least 48: scaling down loses nothing a person sees, \
         and scaling up is visibly soft"
    );
}

#[test]
fn an_exact_size_is_taken_over_a_larger_one() {
    let place = an_icon_tree_holding(&[
        "hicolor/48x48/apps/thing.png",
        "hicolor/256x256/apps/thing.png",
    ]);
    assert_eq!(
        the_icon_called("thing", std::slice::from_ref(&place), 48).path(),
        Some(place.join("hicolor/48x48/apps/thing.png").as_path())
    );
}

#[test]
fn the_largest_below_is_taken_when_nothing_reaches_the_size_asked_for() {
    let place = an_icon_tree_holding(&[
        "hicolor/16x16/apps/thing.png",
        "hicolor/32x32/apps/thing.png",
    ]);
    assert_eq!(
        the_icon_called("thing", std::slice::from_ref(&place), 48).path(),
        Some(place.join("hicolor/32x32/apps/thing.png").as_path()),
        "nothing reaches 48, so the least bad is the largest there is"
    );
}

#[test]
fn a_bare_size_directory_is_read_as_its_number() {
    // Some themes write `48` rather than `48x48`. Reading it as 0 would sort a
    // perfectly good icon below every other and silently prefer a smaller one.
    let place =
        an_icon_tree_holding(&["hicolor/48/apps/thing.png", "hicolor/16x16/apps/thing.png"]);
    assert_eq!(
        the_icon_called("thing", std::slice::from_ref(&place), 48).path(),
        Some(place.join("hicolor/48/apps/thing.png").as_path())
    );
}

#[test]
fn a_context_this_code_has_never_heard_of_is_still_searched() {
    // Themes use `apps`, and also `applications` and others. Guessing the
    // directory name would lose every theme that chose differently.
    let place = an_icon_tree_holding(&["hicolor/48x48/applications/thing.png"]);
    assert!(matches!(
        the_icon_called("thing", &[place], 48),
        WhatWasFound::APicture(_)
    ));
}

#[test]
fn an_icon_that_is_only_scalable_is_found_as_a_scalable_picture() {
    let place = an_icon_tree_holding(&["hicolor/scalable/apps/thing.svg"]);
    assert_eq!(
        the_icon_called("thing", std::slice::from_ref(&place), 48),
        WhatWasFound::AScalablePicture(place.join("hicolor/scalable/apps/thing.svg")),
        "an application whose only artwork is scalable was given no artwork at all"
    );
}

/// **Laid out as a Flatpak export is**: Dolphin's Flatpak ships its icon as
/// `share/icons/hicolor/scalable/apps/org.kde.dolphin.svg` and nothing else,
/// which is what drew it as a letter on a booted machine on 2026-11-01.
#[test]
fn a_flatpaks_scalable_only_icon_is_found_under_its_export() {
    let root = an_icon_tree_holding(&[
        "var/lib/flatpak/exports/share/icons/hicolor/scalable/apps/org.kde.dolphin.svg",
    ]);
    let exports = root.join("var/lib/flatpak/exports/share");
    let data_dirs = std::ffi::OsString::from(format!("{}:/nowhere/share", exports.display()));
    let places = where_icons_live(
        None,
        Some(std::ffi::OsStr::new("/nowhere")),
        Some(&data_dirs),
    );
    assert_eq!(
        the_icon_called("org.kde.dolphin", &places, 48),
        WhatWasFound::AScalablePicture(
            exports.join("icons/hicolor/scalable/apps/org.kde.dolphin.svg")
        )
    );
}

#[test]
fn a_scalable_copy_is_preferred_over_a_picture_too_small() {
    let place = an_icon_tree_holding(&[
        "hicolor/scalable/apps/thing.svg",
        "hicolor/32x32/apps/thing.png",
    ]);
    assert_eq!(
        the_icon_called("thing", std::slice::from_ref(&place), 48),
        WhatWasFound::AScalablePicture(place.join("hicolor/scalable/apps/thing.svg")),
        "a 32 scaled up to 48 is soft, and a scalable copy is drawn at 48"
    );
}

#[test]
fn a_picture_large_enough_is_preferred_over_a_scalable_copy() {
    let place = an_icon_tree_holding(&[
        "hicolor/scalable/apps/thing.svg",
        "hicolor/64x64/apps/thing.png",
    ]);
    assert_eq!(
        the_icon_called("thing", std::slice::from_ref(&place), 48),
        WhatWasFound::APicture(place.join("hicolor/64x64/apps/thing.png")),
        "the application's own drawn pixels at a size that reaches were passed over"
    );
}

#[test]
fn a_picture_too_small_is_still_taken_where_nothing_else_is() {
    let place = an_icon_tree_holding(&["hicolor/32x32/apps/thing.png"]);
    assert_eq!(
        the_icon_called("thing", std::slice::from_ref(&place), 48),
        WhatWasFound::APicture(place.join("hicolor/32x32/apps/thing.png"))
    );
}

#[test]
fn an_earlier_directory_wins_over_a_later_one() {
    let theirs = an_icon_tree_holding(&["hicolor/48x48/apps/browser.png"]);
    let shipped = an_icon_tree_holding(&["hicolor/256x256/apps/browser.png"]);
    let found = the_icon_called("browser", &[theirs.clone(), shipped.clone()], 48);
    assert_eq!(
        found.path(),
        Some(theirs.join("hicolor/48x48/apps/browser.png").as_path()),
        "the person's own icon directory comes first, and a bigger one further down the \
         list does not override what they chose"
    );
}

#[test]
fn an_absolute_name_is_a_path_and_is_used_as_given() {
    let place = an_icon_tree_holding(&["pixmaps/weather.png"]);
    let at = place.join("pixmaps/weather.png");
    assert_eq!(
        the_icon_called(at.to_str().unwrap(), &[], 48).path(),
        Some(at.as_path()),
        "the specification allows an absolute path, and an entry that gives one has said \
         exactly where it put its artwork"
    );
}

#[test]
fn an_absolute_name_pointing_at_nothing_is_nothing() {
    let place = an_icon_tree_holding(&["pixmaps/weather.png"]);
    let missing = place.join("pixmaps/gone.png");
    assert_eq!(
        the_icon_called(missing.to_str().unwrap(), &[], 48),
        WhatWasFound::Nothing
    );
}

#[test]
fn a_flat_directory_is_searched_because_pixmaps_is_one() {
    let place = an_icon_tree_holding(&["legacy.png"]);
    assert_eq!(
        the_icon_called("legacy", std::slice::from_ref(&place), 48).path(),
        Some(place.join("legacy.png").as_path())
    );
}

#[test]
fn a_themed_icon_beats_a_flat_one_of_the_same_name() {
    // `pixmaps` has no size in its path, so a flat file could only be compared
    // as 0 — below everything. A flat icon taking precedence would hand back
    // somebody's 32-pixel legacy artwork when a themed 256 was there.
    let place = an_icon_tree_holding(&["thing.png", "hicolor/64x64/apps/thing.png"]);
    assert_eq!(
        the_icon_called("thing", std::slice::from_ref(&place), 48).path(),
        Some(place.join("hicolor/64x64/apps/thing.png").as_path())
    );
}

#[test]
fn an_empty_name_finds_nothing_rather_than_searching_for_one() {
    let place = an_icon_tree_holding(&["hicolor/48x48/apps/thing.png"]);
    assert_eq!(the_icon_called("", &[place], 48), WhatWasFound::Nothing);
}

#[test]
fn a_directory_that_is_not_there_is_not_a_failure() {
    let nowhere = std::env::temp_dir().join("alo-icons-this-does-not-exist");
    let _ = std::fs::remove_dir_all(&nowhere);
    assert_eq!(
        the_icon_called("thing", &[nowhere], 48),
        WhatWasFound::Nothing,
        "a machine with no icon directory has no icons, which is an answer and not an error"
    );
}

#[test]
fn where_icons_live_follows_the_data_directories_and_ends_at_pixmaps() {
    let places = where_icons_live(
        Some(std::path::Path::new("/home/someone")),
        None,
        Some(std::ffi::OsStr::new("/opt/share:/usr/share")),
    );
    assert_eq!(
        places,
        vec![
            PathBuf::from("/home/someone/.local/share/icons"),
            PathBuf::from("/opt/share/icons"),
            PathBuf::from("/usr/share/icons"),
            PathBuf::from("/usr/share/pixmaps"),
        ]
    );
}

#[test]
fn an_empty_data_variable_is_not_a_directory() {
    // The same fault `alo_installed::where_entries_live` names: treating `""`
    // as a directory reads out of the process's working directory.
    let places = where_icons_live(
        Some(std::path::Path::new("/home/someone")),
        Some(std::ffi::OsStr::new("")),
        Some(std::ffi::OsStr::new("")),
    );
    assert_eq!(
        places,
        vec![
            PathBuf::from("/home/someone/.local/share/icons"),
            PathBuf::from("/usr/local/share/icons"),
            PathBuf::from("/usr/share/icons"),
            PathBuf::from("/usr/share/pixmaps"),
        ]
    );
}
