//! The *Open with* sheet as drawn: frame `25:52`'s words, one row of 72 per
//! application at every scale, *Open* to the right of *Cancel*, and a display
//! too small refused rather than drawn in part.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::net::UnixStream;
use std::time::Instant;

use alo_acting::ToChooseALink;
use alo_applications::{Application, Link};
use alo_strings::Vocabulary;

use super::*;
use crate::the_open_with_sheet::{AskedToChoose, TheOpenWithSheet};

fn words() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::the_open_with_sheets_words::declare_the_open_with_sheets_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A sheet up for Papers asking to open a web link, Firefox the default and
/// LibreWolf the other; the portal's end is handed back to be kept open.
fn a_sheet() -> (UnixStream, TheOpenWithSheet) {
    let (portal, desktop) = UnixStream::pair().unwrap();
    desktop.set_nonblocking(true).unwrap();
    let asked = ToChooseALink::of(
        Application::identified("org.gnome.Papers").unwrap(),
        Link::checked("https://example.com/invoice").unwrap(),
        vec![
            Application::identified("org.mozilla.firefox").unwrap(),
            Application::identified("io.gitlab.librewolf-community").unwrap(),
        ],
    )
    .unwrap();
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(
        vec![AskedToChoose::of(asked, desktop)],
        true,
        Instant::now(),
    );
    (portal, sheet)
}

fn named(identifier: &str) -> (String, Option<alo_dock::artwork::APicture>) {
    let name = match identifier {
        "org.gnome.Papers" => "Papers",
        "org.mozilla.firefox" => "Firefox",
        "io.gitlab.librewolf-community" => "LibreWolf",
        other => other,
    };
    (name.to_owned(), None)
}

fn drawn_at(
    sheet: &TheOpenWithSheet,
    size: (i32, i32),
    per_cent: u16,
) -> Result<OpenWithPicture, RenderError> {
    let mut labels = WindowControlLabels::new().unwrap();
    drawn(
        sheet.shows().unwrap(),
        &words(),
        &mut labels,
        size,
        crate::settings_testing::light(),
        DisplayScale::of_per_cent(per_cent),
        &named,
    )
}

/// **The frame's words, the default said as the default, and one row of 72
/// for each application at one scale and 144 at two.**
#[test]
fn the_sheet_says_what_the_frame_says_at_every_scale() {
    let (_portal, sheet) = a_sheet();
    for (size, per_cent, factor) in [((1440, 960), 100, 1), ((2880, 1920), 200, 2)] {
        let drawn = drawn_at(&sheet, size, per_cent).unwrap();
        for said in [
            "Open with",
            "Papers asked to open https://example.com/invoice",
            "Firefox",
            "Your default for web links",
            "LibreWolf",
            "Installed",
            "Cancel",
            "Open",
        ] {
            assert!(
                drawn.texts.iter().any(|it| it == said),
                "{said}: {:?}",
                drawn.texts
            );
        }
        assert_eq!(drawn.rows.len(), 2);
        for row in &drawn.rows {
            assert_eq!(row.size.h, 72 * factor, "{row:?}");
        }
        assert!(drawn.cancel.loc.x + drawn.cancel.size.w <= drawn.open.loc.x);
        assert_eq!(drawn.open.size.h, 48 * factor);
        assert!(drawn.washed.area.size.w >= 480 * factor);
    }
}

/// **A display too small to hold the sheet is refused**, never drawn in part.
#[test]
fn a_display_too_small_is_refused() {
    let (_portal, sheet) = a_sheet();
    assert!(drawn_at(&sheet, (200, 960), 100).is_err());
    assert!(drawn_at(&sheet, (1440, 200), 100).is_err());
}
