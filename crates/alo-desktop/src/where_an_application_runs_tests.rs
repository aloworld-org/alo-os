//! Which directories under a slice are an application's, on a slice made here.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::*;

/// A slice of this test's own, empty.
fn a_slice(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-app-slice-{}-{what}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// A directory's inode, which is what a control group's number is.
fn number_of(at: &Path) -> u64 {
    std::fs::metadata(at).unwrap().ino()
}

/// **Both shapes are found, with what is beneath them, and nothing else is** —
/// not another application whose name begins the same, and not a unit of the
/// person's that is not an application.
#[test]
fn an_applications_units_and_their_insides_are_found_and_nothing_else() {
    let slice = a_slice("found");
    let started = slice.join("app-alo-org.mozilla.Thunderbird@1f.service");
    let flatpak = slice.join("app-flatpak-org.mozilla.Thunderbird-4242.scope");
    let inside = flatpak.join("helper");
    for made in [&started, &inside] {
        std::fs::create_dir_all(made).unwrap();
    }
    for other in [
        "app-flatpak-org.mozilla.ThunderbirdBeta-1.scope",
        "app-alo-org.mozilla.Thunder@1.service",
        "app-flatpak-org.mozilla.Thunderbird-x.scope",
        "alo-agentd.service",
    ] {
        std::fs::create_dir_all(slice.join(other)).unwrap();
    }

    let mut found = UnderTheAppSlice::beneath(&slice).groups_of("org.mozilla.Thunderbird");
    found.sort_unstable();
    let mut expected = vec![number_of(&started), number_of(&flatpak), number_of(&inside)];
    expected.sort_unstable();
    assert_eq!(found, expected);

    let _ = std::fs::remove_dir_all(slice);
}

/// **An application that is not running has no control groups**, and a slice
/// that is not there — a session with no manager — answers the same, never a
/// guess.
#[test]
fn nothing_running_and_no_slice_both_answer_nothing() {
    let slice = a_slice("empty");
    assert!(
        UnderTheAppSlice::beneath(&slice)
            .groups_of("org.mozilla.Thunderbird")
            .is_empty()
    );
    assert!(
        UnderTheAppSlice::beneath(&slice.join("not-there"))
            .groups_of("org.mozilla.Thunderbird")
            .is_empty()
    );
    let _ = std::fs::remove_dir_all(slice);
}

/// **A file named like a unit is not a control group.**
#[test]
fn a_file_named_like_a_unit_is_not_counted() {
    let slice = a_slice("file");
    std::fs::write(
        slice.join("app-flatpak-org.mozilla.Thunderbird-1.scope"),
        b"",
    )
    .unwrap();
    assert!(
        UnderTheAppSlice::beneath(&slice)
            .groups_of("org.mozilla.Thunderbird")
            .is_empty()
    );
    let _ = std::fs::remove_dir_all(slice);
}

/// **More than can be counted is answered as more**, so the counting can say
/// it missed part of the application rather than this file choosing silently.
#[test]
fn more_than_can_be_counted_is_answered_as_more() {
    let slice = a_slice("many");
    let unit = slice.join("app-flatpak-org.mozilla.Thunderbird-1.scope");
    for each in 0..(THE_MOST_GROUPS + 5) {
        std::fs::create_dir_all(unit.join(format!("part-{each}"))).unwrap();
    }
    let found = UnderTheAppSlice::beneath(&slice).groups_of("org.mozilla.Thunderbird");
    assert_eq!(found.len(), THE_MOST_GROUPS + 1);
    let _ = std::fs::remove_dir_all(slice);
}

/// **An application whose ID holds a `-` is found in the scope Flatpak names
/// for it**, where Flatpak wrote the `-` as `\x2d` — and its unescaped spelling,
/// which Flatpak never writes, is not taken for it.
#[test]
fn a_flatpak_scope_with_an_escaped_dash_is_found() {
    let slice = a_slice("dash");
    let flatpak = slice.join(r"app-flatpak-io.gitlab.librewolf\x2dcommunity-2716289303.scope");
    std::fs::create_dir_all(&flatpak).unwrap();
    std::fs::create_dir_all(slice.join("app-flatpak-io.gitlab.librewolf-community-1.scope"))
        .unwrap();

    let found = UnderTheAppSlice::beneath(&slice).groups_of("io.gitlab.librewolf-community");
    assert_eq!(found, vec![number_of(&flatpak)]);

    let _ = std::fs::remove_dir_all(slice);
}
