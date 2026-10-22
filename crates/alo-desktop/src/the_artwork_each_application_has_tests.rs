//! Desktop entries and icon files written to real directories, read back as
//! artwork — with a real PNG on disk, because *it decoded* is the claim.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;

use alo_dock::window::AppId;

use super::the_artwork_each_application_has;

/// A directory nothing else in this process is using.
///
/// **A counter, not a fingerprint of the contents.** `alo-icons` keys its
/// fixture this way after the version that keyed on contents gave two tests one
/// directory and let them race — failing in a different test each run.
fn somewhere_of_its_own(what: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let mine = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let at = std::env::temp_dir().join(format!("alo-artwork-{}-{mine}-{what}", std::process::id()));
    let _ = std::fs::remove_dir_all(&at);
    std::fs::create_dir_all(&at).unwrap();
    at
}

/// Write a desktop entry naming this icon, and return the directory holding it.
fn an_entry(identifier: &str, icon: Option<&str>) -> PathBuf {
    let at = somewhere_of_its_own("entries");
    let line = icon.map_or_else(String::new, |icon| format!("Icon={icon}\n"));
    std::fs::write(
        at.join(format!("{identifier}.desktop")),
        format!("[Desktop Entry]\nType=Application\nName={identifier}\n{line}"),
    )
    .unwrap();
    at
}

/// Write a **real** PNG of this size into a hicolor theme, and return the icon
/// directory.
///
/// Encoded rather than hand-written, because what is under test is that a file
/// on this machine decodes. A fixture holding bytes somebody typed would prove
/// the decoder accepts those bytes and nothing about a PNG.
fn an_icon_tree_with_a_real_png(named: &str, across: u32, down: u32) -> PathBuf {
    let at = somewhere_of_its_own("icons");
    let apps = at.join(format!("hicolor/{across}x{down}/apps"));
    std::fs::create_dir_all(&apps).unwrap();
    let pixels = image::RgbaImage::from_pixel(across, down, image::Rgba([10, 20, 30, 255]));
    pixels.save(apps.join(format!("{named}.png"))).unwrap();
    at
}

#[test]
fn an_application_that_names_an_icon_on_this_machine_draws_it() {
    let entries = an_entry("thing", Some("thing"));
    let icons = an_icon_tree_with_a_real_png("thing", 48, 48);
    let (artwork, went) = the_artwork_each_application_has(&[entries], &[icons], 48);

    let drawn = artwork.of(&AppId::named("thing").unwrap()).unwrap();
    assert_eq!((drawn.across(), drawn.down()), (48, 48));
    assert_eq!(
        drawn.pixels().len(),
        48 * 48 * 4,
        "four bytes a pixel, which is what the shell's raster works in"
    );
    assert_eq!(
        drawn.pixels().get(..4),
        Some(&[10u8, 20, 30, 255][..]),
        "red green blue alpha, in the order the picture was written"
    );
    assert_eq!(went.drew_a_picture, 1);
    assert_eq!(went.named_none, 0);
}

/// Artwork is keyed by the identifier, and by neither of the other two names.
///
/// **The identifier, the shown name and the icon name are three different
/// strings here on purpose**, which is the second version of this test. The
/// first gave an application the same string for all three — as the other
/// fixtures in this file still do, harmlessly — and so it could not tell the
/// three keys apart. It passed against a deliberate mis-key, and the only
/// reason that was found is that the mis-key was tried.
///
/// A real machine looks like this rather than like the first version:
/// `firefox.desktop` is called *Firefox* and names `Icon=web-browser`.
#[test]
fn it_is_keyed_by_the_identifier_the_dock_builds_an_app_id_from() {
    let at = somewhere_of_its_own("entries");
    std::fs::write(
        at.join("org.gnome.Nautilus.desktop"),
        "[Desktop Entry]\nType=Application\nName=Files\nIcon=system-file-manager\n",
    )
    .unwrap();
    let icons = an_icon_tree_with_a_real_png("system-file-manager", 48, 48);
    let (artwork, went) = the_artwork_each_application_has(&[at], &[icons], 48);

    assert_eq!(went.drew_a_picture, 1, "the control: a picture was decoded");
    assert!(
        artwork
            .of(&AppId::named("org.gnome.Nautilus").unwrap())
            .is_some(),
        "found under the identifier, which is what both callers that build an AppId use"
    );
    assert!(
        artwork.of(&AppId::named("Files").unwrap()).is_none(),
        "and not under the shown name: Application::shown is the name and the identifier \
         together, a fragment for a row, and no AppId anywhere is built from it"
    );
    assert!(
        artwork
            .of(&AppId::named("system-file-manager").unwrap())
            .is_none(),
        "and not under the icon name either, which is a name for a picture and not for an \
         application -- two applications may share one"
    );
}

#[test]
fn an_application_naming_no_icon_is_counted_as_naming_none() {
    let entries = an_entry("plain", None);
    let icons = an_icon_tree_with_a_real_png("something-else", 48, 48);
    let (artwork, went) = the_artwork_each_application_has(&[entries], &[icons], 48);
    assert!(artwork.is_empty());
    assert_eq!(went.named_none, 1);
    assert_eq!(
        went.named_one_that_is_not_here, 0,
        "it named nothing to miss"
    );
    assert_eq!(went.drew_a_picture, 0);
}

#[test]
fn an_application_naming_an_icon_that_is_not_here_is_counted_apart() {
    // Different machine, different count: this one has a broken entry, the one
    // above has an application that simply ships no artwork.
    let entries = an_entry("ghost", Some("not-installed"));
    let icons = an_icon_tree_with_a_real_png("something-else", 48, 48);
    let (artwork, went) = the_artwork_each_application_has(&[entries], &[icons], 48);
    assert!(artwork.is_empty());
    assert_eq!(went.named_one_that_is_not_here, 1);
    assert_eq!(went.named_none, 0);
}

#[test]
fn an_application_whose_only_artwork_is_scalable_is_counted_apart_again() {
    let entries = an_entry("vector", Some("vector"));
    let icons = somewhere_of_its_own("icons");
    let apps = icons.join("hicolor/scalable/apps");
    std::fs::create_dir_all(&apps).unwrap();
    std::fs::write(apps.join("vector.svg"), b"<svg/>").unwrap();

    let (artwork, went) = the_artwork_each_application_has(&[entries], &[icons], 48);
    assert!(artwork.is_empty());
    assert_eq!(
        went.only_scalable, 1,
        "nothing here rasterises an SVG, and saying so is how a report can tell a machine \
         with no artwork from one whose artwork we cannot read"
    );
    assert_eq!(went.named_one_that_is_not_here, 0);
}

#[test]
fn artwork_that_will_not_decode_is_counted_and_costs_no_other_application_its_icon() {
    // One truncated file must not take the whole Dock down with it, which is
    // `alo_installed::read`'s rule: a file that does not parse is skipped
    // rather than failing the walk.
    let entries = somewhere_of_its_own("entries");
    for named in ["broken", "fine"] {
        std::fs::write(
            entries.join(format!("{named}.desktop")),
            format!("[Desktop Entry]\nType=Application\nName={named}\nIcon={named}\n"),
        )
        .unwrap();
    }
    let icons = an_icon_tree_with_a_real_png("fine", 48, 48);
    // Put the rubbish beside the good one, in the same theme directory.
    std::fs::write(icons.join("hicolor/48x48/apps/broken.png"), b"not a png").unwrap();

    let (artwork, went) = the_artwork_each_application_has(&[entries], &[icons], 48);
    assert_eq!(went.would_not_decode, 1);
    assert_eq!(
        went.drew_a_picture, 1,
        "the application beside it still got its icon"
    );
    assert!(artwork.of(&AppId::named("fine").unwrap()).is_some());
    assert!(artwork.of(&AppId::named("broken").unwrap()).is_none());
}

#[test]
fn the_size_asked_for_decides_which_of_several_is_decoded() {
    let entries = an_entry("thing", Some("thing"));
    let icons = somewhere_of_its_own("icons");
    for size in [16u32, 64, 256] {
        let apps = icons.join(format!("hicolor/{size}x{size}/apps"));
        std::fs::create_dir_all(&apps).unwrap();
        image::RgbaImage::from_pixel(size, size, image::Rgba([1, 2, 3, 255]))
            .save(apps.join("thing.png"))
            .unwrap();
    }
    let (artwork, _) = the_artwork_each_application_has(
        std::slice::from_ref(&entries),
        std::slice::from_ref(&icons),
        48,
    );
    assert_eq!(
        artwork
            .of(&AppId::named("thing").unwrap())
            .unwrap()
            .across(),
        64,
        "the nearest at least as large as the box it will be drawn into"
    );
    // The control: asking for a different box picks a different file, so the
    // size above is the one asked for and not the only one that decodes.
    let (smaller, _) = the_artwork_each_application_has(&[entries], &[icons], 16);
    assert_eq!(
        smaller
            .of(&AppId::named("thing").unwrap())
            .unwrap()
            .across(),
        16
    );
}

#[test]
fn a_machine_with_nothing_installed_has_no_artwork_and_no_complaints() {
    let entries = somewhere_of_its_own("entries");
    let icons = somewhere_of_its_own("icons");
    let (artwork, went) = the_artwork_each_application_has(&[entries], &[icons], 48);
    assert!(artwork.is_empty());
    assert_eq!(went, super::HowItWent::default());
}
