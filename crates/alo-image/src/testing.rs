//! An image of this crate's own, made by copying the real one and breaking one
//! line of it.
//!
//! Every check in `crate::checking` needs two things said about it: that the
//! image this repository ships passes it, and that an image which does not is
//! caught. The second half is what a fixture is for — and it is a **copy**
//! rather than a hand-written image, because a fixture assembled by hand is a
//! second image that drifts from the shipped one and quietly stops testing it.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use crate::image::Image;

/// The loader's unit, beneath the image's root.
pub(crate) const THE_LOADERS_UNIT: &str = "usr/lib/systemd/system/alo-boundaryd.service";

/// The agent service's unit, beneath the image's root.
pub(crate) const THE_AGENTS_UNIT: &str = "usr/lib/systemd/system/alo-agentd.service";

/// The opener's unit, beneath the image's root.
pub(crate) const THE_OPENERS_UNIT: &str = "usr/lib/systemd/system/alo-sessiond.service";

/// The model service's unit, beneath the image's root.
pub(crate) const THE_SERVERS_UNIT: &str = "usr/lib/systemd/system/alo-modeld.service";

/// What the image makes at boot, beneath the image's root.
pub(crate) const THE_TMPFILES: &str = "usr/lib/tmpfiles.d/alo.conf";

/// The logins the image makes, beneath the image's root.
pub(crate) const THE_SYSUSERS: &str = "usr/lib/sysusers.d/alo.conf";

/// What the machine says about itself, beneath the image's root.
pub(crate) const THE_DESCRIPTION_FILE: &str = "etc/alo/agentd.toml";

/// The recipe the image is built from, beneath the image's directory.
pub(crate) const THE_CONTAINERFILE: &str = "Containerfile";

/// The document that turns the image into a disk, from the image's directory.
///
/// Beside the image rather than inside it, which is why a copy is a copy of
/// both: the disk is declared in the recipe and described in `docs/`, and a
/// fixture that carried only one of them could not break either against the
/// other.
pub(crate) const THE_BOOTING_DOCUMENT: &str = "../docs/booting.md";

/// Where the accounts a person signs in with would be, beneath the image's
/// root — a file no correct image has, which is why it is only ever written by
/// a fixture.
pub(crate) fn the_store_file() -> &'static str {
    alo_accounts::THE_ACCOUNTS.trim_start_matches('/')
}

/// The image at this root, read.
pub(crate) fn image_at(root: &Path) -> Image {
    Image::at(root).unwrap()
}

/// A copy of the image this repository ships, somewhere a test may write.
///
/// The copy is laid out the way the repository is — the image in `image/`, the
/// document beside it in `docs/` — because `Image::at` reads
/// `docs/booting.md` through the image's own directory, and a fixture whose
/// document was the real one would be a test that edited this repository.
pub(crate) fn a_copy_of_the_image(what: &str) -> PathBuf {
    let of = std::env::temp_dir().join(format!("alo-image-{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&of));

    let at = of.join("image");
    copied(Path::new(crate::THE_IMAGE), &at);

    let document = at.join(THE_BOOTING_DOCUMENT);
    std::fs::create_dir_all(document.parent().unwrap()).unwrap();
    std::fs::copy(
        Path::new(crate::THE_IMAGE).join(THE_BOOTING_DOCUMENT),
        &document,
    )
    .unwrap();

    at
}

/// One directory into another, everything in it, all the way down.
fn copied(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let landing = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copied(&entry.path(), &landing);
        } else {
            std::fs::copy(entry.path(), &landing).unwrap();
        }
    }
}

/// The recipe's release line, read from the recipe rather than repeated.
///
/// A release number changes every time one is published, and a test that spells
/// it out has to be edited by whoever publishes — a chore at the exact moment
/// care is wanted, and a way for a check to end up asserting against a release
/// nobody ships any more.
pub(crate) fn the_release_line() -> String {
    let recipe =
        std::fs::read_to_string(Path::new(crate::THE_IMAGE).join(THE_CONTAINERFILE)).unwrap();
    let line = recipe
        .lines()
        .find(|line| line.starts_with("LABEL org.opencontainers.image.version="));
    assert!(
        line.is_some(),
        "the recipe names no release, which is what the caller is about"
    );
    line.unwrap_or_default().to_owned()
}

/// The release the recipe names, as three numbers alone.
///
/// Every fixture below edits a file that spells the release — the pin, the
/// recipe, the notes, the document a person follows — and each one used to
/// spell it too. That made publishing a release a hunt through twenty-one
/// literals in six files, at the moment somebody is least able to afford a
/// mistake, and it left checks asserting against releases nobody ships. They
/// read it from the recipe now.
pub(crate) fn the_release() -> String {
    the_release_line()
        .split('"')
        .nth(1)
        .unwrap_or_default()
        .to_owned()
}

/// The release the **pin** holds — what an installer pulls today.
///
/// Deliberately not [`the_release`], which is what the *recipe builds*. Between
/// releases they are the same sentence; while one is in flight they are not,
/// and that difference is the whole point of `next`. A fixture that edits the
/// pin must spell the pin's release, and one that edits the recipe must spell
/// the recipe's, or it changes nothing and the test passes by checking the
/// shipped image again.
pub(crate) fn the_pinned_release() -> String {
    let pin = std::fs::read_to_string(Path::new(crate::THE_IMAGE).join("pinned.toml")).unwrap();
    let line = pin
        .lines()
        .find(|line| line.starts_with("version = "))
        .unwrap_or_default();
    line.split('"').nth(1).unwrap_or_default().to_owned()
}

/// The pin's version line, as the shipped pin spells it.
pub(crate) fn the_version_line() -> String {
    format!("version = \"{}\"", the_pinned_release())
}

/// One `name = "value"` line's value, from the shipped pin.
fn the_pins(field: &str) -> String {
    let pin = std::fs::read_to_string(Path::new(crate::THE_IMAGE).join(THE_PIN_FILE)).unwrap();
    let wanted = format!("{field} = ");
    let line = pin
        .lines()
        .find(|line| line.starts_with(&wanted))
        .unwrap_or_default();
    line.split('"').nth(1).unwrap_or_default().to_owned()
}

/// **The digest the owner signed**, read from the pin rather than spelled.
///
/// Spelling it was how pinning a release came to mean editing seven literals
/// across five files. Each one missed stopped the gate; each one missed the
/// other way would have left a test asserting against an image nobody ships.
/// The pin is the one place a digest belongs, because the pin is what an
/// installer reads.
pub(crate) fn the_pinned_digest() -> String {
    the_pins("digest")
}

/// **The commit the pinned image was built from**, read from the pin.
pub(crate) fn the_pinned_revision() -> String {
    the_pins("revision")
}

/// The tag line `docs/booting.md` and the release notes spell, which names what
/// is pinned rather than what is being built.
pub(crate) fn the_tag_line() -> String {
    format!("tag: {}", the_pinned_release())
}

/// A copy of the image with **no release in flight**.
///
/// Fixtures below arrange a disagreement between the recipe and the pin and
/// assert that it is caught. While a release is in flight the shipped files
/// already disagree, legitimately — the recipe names the candidate, the pin
/// still holds what an installer pulls — so a fixture built straight from them
/// would arrange two disagreements and assert about one, and would keep passing
/// if the thing it meant to catch stopped being caught.
///
/// So each starts from the state this repository is in *between* releases: the
/// recipe naming exactly what the pin holds, and nothing declared. Written to
/// work in either state, so that publishing a release does not mean editing
/// fixtures.
pub(crate) fn a_copy_between_releases(what: &str) -> PathBuf {
    let root = a_copy_of_the_image(what);

    let pin = root.join(THE_PIN_FILE);
    let held = std::fs::read_to_string(&pin).unwrap();
    let without: String = held
        .lines()
        .filter(|line| !line.starts_with("next = "))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    std::fs::write(
        &pin,
        format!(
            "{without}
"
        ),
    )
    .unwrap();

    let recipe = root.join(THE_CONTAINERFILE);
    let built = std::fs::read_to_string(&recipe).unwrap();
    std::fs::write(
        &recipe,
        built.replace(
            &the_release_line(),
            &format!(
                "LABEL org.opencontainers.image.version=\"{}\"",
                the_pinned_release()
            ),
        ),
    )
    .unwrap();

    root
}

/// The pin, beneath the image's directory.
pub(crate) const THE_PIN_FILE: &str = "pinned.toml";

/// Change one thing in one of this image's files.
///
/// The `from` has to be there: a fixture whose edit silently did nothing is a
/// test that passes because it checked the shipped image again.
pub(crate) fn edited(root: &Path, file: &str, from: &str, to: &str) {
    let at = root.join(file);
    let before = std::fs::read_to_string(&at).unwrap();
    assert!(
        before.contains(from),
        "{file} does not contain `{from}`, so this fixture changed nothing"
    );
    std::fs::write(&at, before.replace(from, to)).unwrap();
}
