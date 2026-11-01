//! **An application's scalable icon, drawn into pixels** for the Dock.
//!
//! Many applications ship their icon only as an SVG. Dolphin's Flatpak exports
//! `hicolor/scalable/apps/org.kde.dolphin.svg` and nothing else, so on a booted
//! machine on 2026-11-01 the Dock drew it as a letter. An application's SVG is
//! whatever its authors drew — gradients, transforms, masks — which is why this
//! uses `resvg`, a complete static-SVG renderer, rather than the strict subset
//! `alo-vector-shapes` reads for alo's own artwork.
//! [ADR 0144](../../../docs/decisions/0144-an-applications-scalable-icon-is-drawn-by-resvg.md)
//! has the choice.
//!
//! # What it will not read
//!
//! **Nothing outside the file.** `resvg` is built without its text and raster
//! image features, and given no resources directory, so an icon cannot make
//! this desktop open a font, a picture or any other path. A file larger than
//! [`AT_MOST`] is not read at all: an icon is a few kilobytes, and this runs in
//! the person's desktop for every installed application.

use alo_dock::artwork::APicture;

/// The largest icon file read, in bytes.
///
/// Measured once, by hand, on 2026-11-01 in the development PC's Ubuntu 24.04:
/// the largest of the 11 614 SVGs under `/usr/share/icons` was 98 162 bytes.
/// Four megabytes is forty times that, room for an application's own artwork
/// and none for a file that is not an icon.
pub(crate) const AT_MOST: u64 = 4 * 1024 * 1024;

/// The SVG at `at`, drawn `across` pixels square and centred, or [`None`] where
/// it would not become a picture.
///
/// **Every failure is [`None`]**, which is `decoded`'s rule beside it: one
/// broken icon must not cost any other application its own.
pub(crate) fn drawn(at: &std::path::Path, across: u32) -> Option<APicture> {
    let length = std::fs::metadata(at).ok()?.len();
    if length > AT_MOST {
        return None;
    }
    let bytes = std::fs::read(at).ok()?;
    drawn_from(&bytes, across)
}

/// The SVG in `bytes`, drawn `across` pixels square and centred.
pub(crate) fn drawn_from(bytes: &[u8], across: u32) -> Option<APicture> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(bytes, &options).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(across, across)?;
    let size = tree.size();
    // An icon is a few hundred pixels across at most, far inside the range f32
    // holds exactly.
    let side = across as f32;
    let scale = (side / size.width()).min(side / size.height());
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    // Centred, so an icon that is not square keeps its proportions and sits in
    // the middle of its slot.
    let left = (side - size.width() * scale) / 2.0;
    let top = (side - size.height() * scale) / 2.0;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_row(scale, 0.0, 0.0, scale, left, top),
        &mut pixmap.as_mut(),
    );
    // `tiny-skia` holds premultiplied colour; the Dock's pictures are straight
    // (`alo_dock::artwork::APicture`), so each pixel is divided back out.
    let pixels = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let straight = pixel.demultiply();
            [
                straight.red(),
                straight.green(),
                straight.blue(),
                straight.alpha(),
            ]
        })
        .collect();
    APicture::of(across, across, pixels).ok()
}

#[cfg(test)]
#[path = "a_scalable_icon_drawn_tests.rs"]
mod tests;
