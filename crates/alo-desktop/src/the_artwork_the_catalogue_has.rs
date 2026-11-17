//! **The picture each application in the market draws**, where this machine
//! already holds it, read once and handed to the shell as data.
//!
//! The owner, 2026-11-17, on seeing the market draw a grey square beside an
//! identifier: *real application names and icons, never identifiers.* An
//! application a person has not installed has no desktop entry and so no icon
//! the Dock's reading would find. Its **source** publishes one: flatpak keeps
//! each remote's catalogue, pictures included, under
//! `<installation>/appstream/<remote>/<arch>/active/icons/<size>/<id>.png`.
//! That is a file this machine already has, so reading it sends nothing
//! anywhere.
//!
//! # A letter where there is none, and that is the design's own tile
//!
//! A machine that has never refreshed a remote's catalogue has no such files,
//! and an application from its maker's own release (ADR 0158) is in no remote
//! at all. Those draw the first letter of their name, which is what the
//! design's tile shows and the Dock's rule for an application without a
//! picture. **Whether a fresh alo machine holds Flathub's pictures has not
//! been measured** on a booted one.
//!
//! # Into the same map as the Dock's, and never over one of its pictures
//!
//! `TheArtwork::draws` keeps the first picture it is given for an application,
//! and this runs after the installed applications' own, so an installed
//! application keeps the icon its entry names.

use std::path::{Path, PathBuf};

use alo_dock::artwork::{APicture, TheArtwork};
use alo_dock::window::AppId;

/// The sizes a source's catalogue keeps, largest first — the market's tile is
/// drawn smaller than either and scaled down where it is drawn.
const THE_SIZES: [&str; 2] = ["128x128", "64x64"];

/// Every directory under `appstream` that holds a remote's pictures for this
/// architecture: `<appstream>/<remote>/<arch>/active/icons/<size>`, largest
/// size first, remotes in name order.
#[must_use]
pub fn where_the_catalogues_pictures_are(appstream: &Path, arch: &str) -> Vec<PathBuf> {
    let Ok(remotes) = std::fs::read_dir(appstream) else {
        return Vec::new();
    };
    let mut remotes: Vec<PathBuf> = remotes
        .filter_map(Result::ok)
        .map(|one| one.path())
        .collect();
    remotes.sort();
    THE_SIZES
        .iter()
        .flat_map(|size| {
            remotes
                .iter()
                .map(move |remote| remote.join(arch).join("active/icons").join(size))
        })
        .filter(|directory| directory.is_dir())
        .collect()
}

/// Add each catalogued application's picture to `artwork`, from the first of
/// `directories` that holds `<identifier>.png`, and say how many were found.
pub fn the_artwork_the_catalogue_has(
    artwork: &mut TheArtwork,
    catalogue: &alo_software::Catalogue,
    directories: &[PathBuf],
) -> usize {
    let mut found = 0;
    for one in catalogue.every() {
        let identifier = one.application().identifier();
        let Ok(known_as) = AppId::named(identifier) else {
            continue;
        };
        let picture = directories
            .iter()
            .map(|directory| directory.join(format!("{identifier}.png")))
            .find_map(|file| decoded(&file));
        if let Some(picture) = picture {
            artwork.draws(known_as, picture);
            found += 1;
        }
    }
    found
}

/// One PNG's pixels, or [`None`] for a file that is not there or not a PNG.
fn decoded(at: &Path) -> Option<APicture> {
    let bytes = std::fs::read(at).ok()?;
    let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).ok()?;
    let rgba = decoded.to_rgba8();
    let (across, down) = rgba.dimensions();
    APicture::of(across, down, rgba.into_raw()).ok()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A two-by-two PNG of one colour.
    fn a_png(rgba: [u8; 4]) -> Vec<u8> {
        let image = image::RgbaImage::from_pixel(2, 2, image::Rgba(rgba));
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        bytes.into_inner()
    }

    /// **A catalogued application's picture is found where flatpak keeps its
    /// source's catalogue**, the largest size first; one with no picture there
    /// gets none, and an installed application's own is not replaced.
    #[test]
    fn the_catalogues_pictures_are_found_where_flatpak_keeps_them() {
        let root = tempfile::tempdir().unwrap();
        let appstream = root.path().join("appstream");
        let large = appstream.join("flathub/x86_64/active/icons/128x128");
        let small = appstream.join("flathub/x86_64/active/icons/64x64");
        std::fs::create_dir_all(&large).unwrap();
        std::fs::create_dir_all(&small).unwrap();
        std::fs::write(large.join("org.gimp.GIMP.png"), a_png([1, 2, 3, 255])).unwrap();
        std::fs::write(small.join("org.gimp.GIMP.png"), a_png([9, 9, 9, 255])).unwrap();
        std::fs::write(small.join("org.kde.krita.png"), a_png([4, 5, 6, 255])).unwrap();
        std::fs::write(small.join("org.inkscape.Inkscape.png"), b"not a png").unwrap();

        let directories = where_the_catalogues_pictures_are(&appstream, "x86_64");
        assert_eq!(directories, vec![large, small]);

        let mut artwork = TheArtwork::none();
        let installed = AppId::named("org.kde.krita").unwrap();
        artwork.draws(
            installed.clone(),
            APicture::of(1, 1, vec![7, 7, 7, 255]).unwrap(),
        );

        let catalogue = alo_software::Catalogue::decided().unwrap();
        let found = the_artwork_the_catalogue_has(&mut artwork, &catalogue, &directories);

        assert_eq!(
            found, 2,
            "GIMP and Krita were found, Inkscape's would not decode"
        );
        let gimp = artwork.of(&AppId::named("org.gimp.GIMP").unwrap()).unwrap();
        assert_eq!(
            gimp.pixels().get(..4),
            Some(&[1, 2, 3, 255][..]),
            "the larger one"
        );
        assert_eq!(
            artwork.of(&installed).unwrap().pixels(),
            &[7, 7, 7, 255],
            "an installed application's own picture was replaced"
        );
        assert!(
            artwork
                .of(&AppId::named("org.inkscape.Inkscape").unwrap())
                .is_none()
        );
    }

    /// No catalogue on the machine is no pictures, and nothing fails.
    #[test]
    fn a_machine_with_no_catalogue_has_no_pictures_and_says_nothing_went_wrong() {
        let root = tempfile::tempdir().unwrap();
        assert!(
            where_the_catalogues_pictures_are(&root.path().join("appstream"), "x86_64").is_empty()
        );
    }
}
