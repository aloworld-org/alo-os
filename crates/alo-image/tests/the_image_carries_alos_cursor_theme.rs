//! **The image carries alo's cursor theme and names it as the default**, so an
//! application that draws its own pointer draws alo's.
//!
//! It reads `image/Containerfile` and the files the recipe copies, and nothing
//! else, so it runs anywhere. What it cannot do is boot: whether a given
//! application on a booted machine reads `XCURSOR_THEME` is a walk, not this.
//!
//! # What has to agree
//!
//! - the recipe **runs** `alo-cursor-theme` — not a `test -x` on it — into the
//!   directory it then copies to `/usr/share/icons/alo/`;
//! - the default names a theme of the name `alo_cursors::theme::THE_NAME` writes,
//!   at the size `alo_cursors::measures::LOGICAL_SIZE` draws at;
//! - the `default` theme inherits it;
//! - and the default is in `/usr/lib/environment.d`, where a person's own
//!   `~/.config/environment.d` overrides it — a default they can change, never a
//!   value pushed over theirs (`CLAUDE.md`, law 5).

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a recipe that cannot be read is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The repository's root.
fn the_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A file of the repository, as text.
fn read(path: &str) -> String {
    std::fs::read_to_string(the_root().join(path))
        .unwrap_or_else(|why| panic!("{path} could not be read: {why}"))
}

/// The recipe with each `\`-continued line joined to the next, so one `RUN`
/// is one line.
fn the_recipe() -> String {
    read("image/Containerfile").replace("\\\n", " ")
}

#[test]
fn the_recipe_runs_the_generator_into_what_it_copies() {
    let recipe = the_recipe();
    assert!(
        recipe
            .lines()
            .any(|line| line.starts_with("RUN cargo build")
                && line.contains("--package alo-cursors")),
        "the shell stage does not build alo-cursors"
    );
    let ran = recipe
        .lines()
        .find(|line| line.starts_with("RUN target/release/alo-cursor-theme "))
        .unwrap_or_else(|| panic!("the recipe never runs alo-cursor-theme"));
    let written = ran
        .split_whitespace()
        .nth(2)
        .unwrap_or_else(|| panic!("alo-cursor-theme is run with nowhere to write"));
    assert!(
        recipe
            .lines()
            .any(|line| line.starts_with("COPY --from=shell ")
                && line.contains(&format!(
                    "{written}/ /usr/share/icons/{}/",
                    alo_cursors::theme::THE_NAME
                ))),
        "the theme written to {written} is not what is copied to /usr/share/icons"
    );
}

#[test]
fn the_default_names_the_theme_at_the_size_the_pointer_is_drawn() {
    let default = read("image/usr/lib/environment.d/60-alo-cursor.conf");
    let set: Vec<&str> = default
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .collect();
    assert_eq!(
        set,
        [
            format!("XCURSOR_THEME={}", alo_cursors::theme::THE_NAME),
            format!("XCURSOR_SIZE={}", alo_cursors::measures::LOGICAL_SIZE),
        ],
        "the session's default is not alo's theme at alo's size"
    );
    let recipe = the_recipe();
    assert!(
        recipe
            .lines()
            .any(|line| line == "COPY image/usr/lib/environment.d/ /usr/lib/environment.d/"),
        "the default is in the tree and not in the image"
    );
}

#[test]
fn the_default_theme_inherits_alos() {
    let index = read("image/usr/share/icons/default/index.theme");
    assert!(index.contains("[Icon Theme]\n"));
    assert!(index.contains(&format!("\nInherits={}\n", alo_cursors::theme::THE_NAME)));
    assert!(
        the_recipe().lines().any(|line| line
            == "COPY image/usr/share/icons/default/index.theme /usr/share/icons/default/index.theme"),
        "the default theme's index is in the tree and not in the image"
    );
}
