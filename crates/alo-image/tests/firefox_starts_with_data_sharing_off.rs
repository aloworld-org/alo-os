//! **Firefox starts with sharing data with Mozilla switched off, and the person
//! can switch it back on.** The owner, 2026-11-13: *"Off by default"*.
//!
//! It reads the files the image ships and the recipe, and nothing else, so it
//! runs anywhere. What it cannot do is start Firefox: whether Firefox's welcome
//! on a booted machine shows the boxes unticked is a walk, not this.
//!
//! # What has to agree
//!
//! - the four defaults are there, each `false`;
//! - every setting is `pref`, never `lockPref` — a lock would take the choice
//!   away (`CLAUDE.md`, law 5) — and nothing else is in the file;
//! - the tmpfiles.d line links the place Flathub's Firefox reads the machine's
//!   settings from to the directory the recipe copies the file into, for the
//!   architecture the recipe builds;
//! - the recipe copies that directory and checks the file at build time;
//! - nothing hides Firefox's Terms of Use: only data sharing is decided here.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a file that cannot be read is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The settings file, as the repository keeps it.
const THE_PREFS: &str = "image/usr/share/alo/firefox/defaults/pref/alo-data-sharing-starts-off.js";

/// The tmpfiles.d file that puts it where Firefox looks.
const THE_LINK: &str = "image/usr/lib/tmpfiles.d/alo-firefox-shares-nothing-at-first.conf";

/// Where the image puts the directory Firefox is given.
const ON_THE_MACHINE: &str = "/usr/share/alo/firefox";

/// What is switched off, each with the words Firefox's Settings uses for it.
const SWITCHED_OFF: [(&str, &str); 4] = [
    (
        "datareporting.healthreport.uploadEnabled",
        "send technical and interaction data",
    ),
    ("datareporting.usage.uploadEnabled", "daily usage ping"),
    (
        "app.shield.optoutstudies.enabled",
        "install and run studies",
    ),
    (
        "browser.crashReports.unsubmittedCheck.autoSubmit2",
        "backlogged crash reports",
    ),
];

/// The repository's root.
fn the_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A file of the repository, as text.
fn read(path: &str) -> String {
    std::fs::read_to_string(the_root().join(path))
        .unwrap_or_else(|why| panic!("{path} could not be read: {why}"))
}

/// The recipe with each `\`-continued line joined to the next.
fn the_recipe() -> String {
    read("image/Containerfile").replace("\\\n", " ")
}

/// The lines of the settings file that are not comments or blank.
fn the_settings() -> Vec<String> {
    read(THE_PREFS)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .map(str::to_owned)
        .collect()
}

#[test]
fn each_kind_of_sharing_starts_off() {
    let settings = the_settings();
    for (name, what) in SWITCHED_OFF {
        let wanted = format!("pref(\"{name}\", false);");
        assert!(
            settings.contains(&wanted),
            "Firefox's \"{what}\" does not start off: no `{wanted}` in {THE_PREFS}"
        );
    }
}

#[test]
fn nothing_is_locked_and_nothing_else_is_set() {
    let settings = the_settings();
    // The control: that something was read at all.
    assert!(
        !settings.is_empty(),
        "{THE_PREFS} has no settings, so the checks below would pass on nothing"
    );
    for line in &settings {
        assert!(
            !line.contains("lockPref") && !line.contains("sticky_pref"),
            "`{line}` locks a setting, which takes the choice away from the person"
        );
        let is_one_of_ours = SWITCHED_OFF
            .iter()
            .any(|(name, _)| *line == format!("pref(\"{name}\", false);"));
        assert!(
            is_one_of_ours,
            "`{line}` is in {THE_PREFS} and is not one of the defaults the owner decided"
        );
    }
    assert_eq!(
        settings.len(),
        SWITCHED_OFF.len(),
        "a default is set twice in {THE_PREFS}"
    );
}

/// The architecture the recipe builds, as flatpak names it.
fn the_architecture() -> String {
    let recipe = the_recipe();
    let target = recipe
        .lines()
        .find_map(|line| line.strip_prefix("ARG THE_TARGET="))
        .unwrap_or_else(|| panic!("the recipe names no default THE_TARGET"));
    target
        .split('-')
        .next()
        .unwrap_or_else(|| panic!("THE_TARGET `{target}` has no architecture"))
        .to_owned()
}

#[test]
fn the_link_is_where_flathubs_firefox_looks() {
    let arch = the_architecture();
    let wanted = format!(
        "L /var/lib/flatpak/extension/org.mozilla.firefox.systemconfig/{arch}/stable \
         - - - - {ON_THE_MACHINE}"
    );
    let file = read(THE_LINK);
    let lines: Vec<&str> = file
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .collect();
    assert!(
        lines.contains(&wanted.as_str()),
        "{THE_LINK} does not link `{wanted}`"
    );
    // `L` and never `L+`: a directory somebody put there is theirs.
    assert!(
        !lines.iter().any(|line| line.starts_with("L+")),
        "{THE_LINK} would replace a directory the person or their organisation put there"
    );
    // Every parent the link sits in is made, so the link has somewhere to go.
    for parent in [
        "/var/lib/flatpak/extension".to_owned(),
        "/var/lib/flatpak/extension/org.mozilla.firefox.systemconfig".to_owned(),
        format!("/var/lib/flatpak/extension/org.mozilla.firefox.systemconfig/{arch}"),
    ] {
        assert!(
            lines
                .iter()
                .any(|line| *line == format!("d {parent} 0755 root root -")),
            "{THE_LINK} does not make {parent}"
        );
    }
}

#[test]
fn the_recipe_copies_the_settings_and_checks_them() {
    let recipe = the_recipe();
    assert!(
        recipe
            .lines()
            .any(|line| line == format!("COPY image/usr/share/alo/firefox/ {ON_THE_MACHINE}/")),
        "the recipe does not copy the Firefox settings to {ON_THE_MACHINE}"
    );
    assert!(
        recipe
            .lines()
            .any(|line| line.starts_with("COPY image/usr/lib/tmpfiles.d/ /usr/lib/tmpfiles.d/")),
        "the recipe does not ship the tmpfiles.d files, so the link is never made"
    );
    let check = recipe
        .lines()
        .find(|line| line.starts_with("RUN prefs=/usr/share/alo/firefox/defaults/pref/"))
        .unwrap_or_else(|| panic!("the build never checks the Firefox settings file"));
    assert!(
        check.contains("! grep -Ev") && check.contains("(true|false)"),
        "the build does not refuse a line that is not a plain default"
    );
    assert!(
        check.contains(&format!("= {}", SWITCHED_OFF.len())),
        "the build does not count the {} defaults",
        SWITCHED_OFF.len()
    );
}

#[test]
fn firefoxs_terms_of_use_are_not_hidden() {
    // Only data sharing is decided here. Hiding Mozilla's terms from the person
    // is not something anybody decided, so neither file may do it.
    for path in [THE_PREFS, THE_LINK] {
        let text = read(path);
        for hiding in ["SkipTermsOfUse", "termsofuse.", "TermsOfUse\""] {
            assert!(
                !text.contains(hiding),
                "{path} mentions `{hiding}`, which would hide Firefox's Terms of Use"
            );
        }
    }
    // And no enterprise policy file is shipped, which would lock settings and
    // tell the person an organisation manages their browser.
    assert!(
        !the_root()
            .join("image/usr/share/alo/firefox/policies")
            .exists(),
        "a Firefox policies directory is shipped; policies lock what they set"
    );
}
