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

/// What the image makes at boot, beneath the image's root.
pub(crate) const THE_TMPFILES: &str = "usr/lib/tmpfiles.d/alo.conf";

/// The logins the image makes, beneath the image's root.
pub(crate) const THE_SYSUSERS: &str = "usr/lib/sysusers.d/alo.conf";

/// What the machine says about itself, beneath the image's root.
pub(crate) const THE_DESCRIPTION_FILE: &str = "etc/alo/agentd.toml";

/// The image at this root, read.
pub(crate) fn image_at(root: &Path) -> Image {
    Image::at(root).unwrap()
}

/// A copy of the image this repository ships, somewhere a test may write.
pub(crate) fn a_copy_of_the_image(what: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!("alo-image-{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&at));
    copied(Path::new(crate::THE_IMAGE), &at);
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
