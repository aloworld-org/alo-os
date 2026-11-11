//! **Applications describe themselves as the machine comes** — the owner's
//! decision of 2026-11-11 — because the image names, for every person's
//! systemd manager, exactly what `alo_launching::describing_themselves` says
//! a session is told when the switch is on.
//!
//! It reads `image/Containerfile` and the file the recipe copies, and nothing
//! else, so it runs anywhere. **What it cannot do is boot**: whether each
//! application on a booted alo machine then describes itself is a walk, and
//! the toolkits' own answers were measured once, by hand, on 2026-11-11
//! (`docs/quirks/chromium-describes-only-its-window-frame-until-asked.md`).
//!
//! # What has to agree
//!
//! - the file is in `/usr/lib/environment.d`, where a person's own
//!   `~/.config/environment.d` overrides it — a default they can change;
//! - the recipe copies that folder into the image;
//! - and its variables are **exactly** the *on* answer, so the desktop turning
//!   the switch off and on again tells the session the same thing the machine
//!   came with, and nothing the image says is something the switch cannot
//!   take back.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a recipe that cannot be read is the failure being reported"
)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Where the default is, in the repository and so in the image.
const THE_FILE: &str = "image/usr/lib/environment.d/60-alo-apps-describe-themselves.conf";

/// The repository's root.
fn the_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A file of the repository, as text.
fn read(path: &str) -> String {
    std::fs::read_to_string(the_root().join(path))
        .unwrap_or_else(|why| panic!("{path} could not be read: {why}"))
}

/// The `NAME=value` lines of an environment.d file, comments and blanks left
/// out.
fn the_variables(text: &str) -> BTreeSet<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_image_says_exactly_what_the_switch_says_when_on() {
    let in_the_image = the_variables(&read(THE_FILE));
    let told = alo_launching::describing_themselves::what_the_session_is_told(true);
    let when_on: BTreeSet<String> = told
        .everywhere
        .iter()
        .chain(&told.the_manager_only)
        .cloned()
        .collect();
    assert_eq!(
        in_the_image, when_on,
        "the machine comes with something other than the switch turned on"
    );
}

#[test]
fn nothing_the_image_says_is_something_off_cannot_take_back() {
    let off = alo_launching::describing_themselves::what_the_session_is_told(false);
    let named = |line: &str| line.split_once('=').map(|(name, _)| name.to_owned());
    let taken_back: BTreeSet<String> = off
        .everywhere
        .iter()
        .filter_map(|line| named(line))
        .chain(off.forgotten.iter().map(|it| (*it).to_owned()))
        .collect();
    for line in the_variables(&read(THE_FILE)) {
        let name = named(&line).unwrap_or_else(|| panic!("{line} is not NAME=value"));
        assert!(
            taken_back.contains(&name),
            "{name} is set by the image and turning the switch off never changes it"
        );
    }
}

#[test]
fn the_recipe_copies_the_folder_the_default_is_in() {
    let recipe = read("image/Containerfile");
    assert!(
        recipe
            .lines()
            .any(|line| line.trim() == "COPY image/usr/lib/environment.d/ /usr/lib/environment.d/"),
        "the recipe does not copy /usr/lib/environment.d, so the machine comes with nothing"
    );
}
