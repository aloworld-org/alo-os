//! **A file the person chooses reaches the application that asked, and
//! upstream's frontend stays masked** — ADR 0159.
//!
//! The image carries Fedora's `xdg-desktop-portal` for one of its two
//! programs, the documents portal. The other, the frontend, would take
//! `org.freedesktop.portal.Desktop`, which `alo-portald` owns; if it could
//! start, two programs would answer one person's applications and every
//! decision `alo-portals` makes could be bypassed by the one it does not make.
//! So this holds the recipe to three things: the package is installed from the
//! pinned base, the frontend's unit is masked, and the documents portal is
//! **started** in the build.
//!
//! It reads `image/Containerfile` and nothing else, so it runs on every build.
//!
//! # What this cannot prove
//!
//! That a sandboxed application on a booted machine opens a file exported to
//! it. That needs a session, a FUSE mount and a Flatpak, and is the file
//! chooser's walk.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "an unreadable recipe is the failure this test reports, and a formatted panic names \
              the path it looked at"
)]

/// The recipe, as one string.
fn the_recipe() -> String {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("image/Containerfile");
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("the recipe is not readable at {}: {why}", at.display()))
}

/// What the build echoes once the block has done everything, which names it.
const THE_CLAIM: &str =
    "a chosen file reaches the application that asked, and upstream's frontend is masked";

/// The `RUN` block that installs the documents portal, comments removed, up to
/// its own claim and no further.
fn the_block() -> String {
    let recipe = the_recipe();
    let block = recipe
        .split("RUN dnf install")
        .find(|block| block.contains(THE_CLAIM))
        .expect("the recipe still has the block that installs the documents portal");
    let ends = block
        .find(THE_CLAIM)
        .map_or(block.len(), |at| at + THE_CLAIM.len());
    block
        .get(..ends)
        .unwrap_or(block)
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// **Installed from the base's own repository**, and nothing fetched.
#[test]
fn the_package_comes_from_the_pinned_base() {
    let block = the_block();
    assert!(
        block
            .lines()
            .next()
            .is_some_and(|first| first.split_whitespace().any(|word| word == "xdg-desktop-portal")),
        "the block no longer asks dnf for xdg-desktop-portal: {block}"
    );
    assert!(
        !block.contains("curl") && !block.contains("http"),
        "the block fetches something with no hash the recipe holds: {block}"
    );
}

/// **The frontend is masked, and the build checks that it is**, so nothing —
/// not even D-Bus activation before `alo-portald` is up — can start a second
/// answer to the person's applications.
#[test]
fn upstreams_frontend_stays_masked() {
    let block = the_block();
    assert!(
        block.contains("systemctl --global mask xdg-desktop-portal.service"),
        "the frontend's user unit is no longer masked: {block}"
    );
    assert!(
        block.contains("test -L /etc/systemd/user/xdg-desktop-portal.service"),
        "the build no longer checks the mask took: {block}"
    );
    let recipe = the_recipe();
    for undone in [
        "unmask xdg-desktop-portal.service",
        "enable xdg-desktop-portal.service",
    ] {
        assert!(
            !recipe.contains(undone),
            "the recipe undoes the mask with `{undone}`"
        );
    }
}

/// **The documents portal is started in the build**, not only found: a
/// program that dies at launch on a missing library fails here.
#[test]
fn the_documents_portal_is_started_in_the_build() {
    let block = the_block();
    assert!(
        block.contains("/usr/libexec/xdg-document-portal --help"),
        "the build no longer starts the documents portal: {block}"
    );
    assert!(
        block.contains("org.freedesktop.portal.Documents"),
        "the build no longer checks the documents portal can be reached by its name: {block}"
    );
}
