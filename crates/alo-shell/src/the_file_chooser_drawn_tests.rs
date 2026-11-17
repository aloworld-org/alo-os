//! The file chooser as drawn: its words, its rows a page at a time with the
//! keyboard's row always among them, as many as the display holds, the name
//! when saving, and a display too small refused rather than drawn in part.

use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Instant;

use alo_acting::{ToChoose, ToChooseAFile};
use alo_applications::Application;
use alo_file_chooser::{Folders, Key, Listing, NotListed};
use alo_strings::Vocabulary;

use smithay::utils::Rectangle;

use super::*;
use crate::the_file_chooser::{AskedForAFile, TheFileChooser};

fn words() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::the_file_choosers_words::declare_the_file_choosers_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A home with two folders and forty files.
struct AHome;

impl Folders for AHome {
    fn listed(&self, at: &Path) -> Result<Listing, NotListed> {
        if at == Path::new("/home/anna") {
            let files = (0..40).map(|n| format!("letter {n:02}.txt")).collect();
            Ok(Listing::of(
                vec!["Letters".to_owned(), "Music".to_owned()],
                files,
            ))
        } else {
            Err(NotListed::NotThere)
        }
    }

    fn holds(&self, _: &Path, _: &str) -> Option<Kind> {
        None
    }
}

/// A sheet up for Text Editor; the portal's end is handed back to be kept.
fn a_sheet(to: ToChoose) -> (UnixStream, TheFileChooser) {
    let (portal, desktop) = UnixStream::pair().unwrap();
    desktop.set_nonblocking(true).unwrap();
    let asked = ToChooseAFile::of(
        Application::identified("org.gnome.TextEditor").unwrap(),
        to,
        PathBuf::from("/home/anna"),
    )
    .unwrap();
    let mut sheet = TheFileChooser::none();
    sheet.asked(
        vec![AskedForAFile::of(asked, desktop)],
        true,
        Instant::now(),
        &AHome,
    );
    (portal, sheet)
}

fn called(identifier: &str) -> String {
    if identifier == "org.gnome.TextEditor" {
        "Text Editor".to_owned()
    } else {
        identifier.to_owned()
    }
}

fn drawn_at(
    sheet: &TheFileChooser,
    size: (i32, i32),
    per_cent: u16,
) -> Result<FileChooserPicture, RenderError> {
    let mut labels = WindowControlLabels::new().unwrap();
    drawn(
        sheet.shows().unwrap(),
        &words(),
        &mut labels,
        crate::where_a_sheet_sits::OnTheDisplay::with_the_bar_at(size, None),
        crate::settings_testing::light(),
        DisplayScale::of_per_cent(per_cent),
        &called,
    )
}

/// **The words drawn are the sheet's**, opening, at one scale and two.
#[test]
fn opening_says_what_the_design_note_says() {
    let (_portal, sheet) = a_sheet(ToChoose::Open);
    for (size, per_cent) in [((1440, 960), 100), ((2880, 1920), 200)] {
        let drawn = drawn_at(&sheet, size, per_cent).unwrap();
        for said in [
            "Open a file",
            "Text Editor asked to open a file",
            "In /home/anna",
            "home",
            "The folder above",
            "Letters",
            "Folder",
            "Cancel",
            "Open",
        ] {
            assert!(
                drawn.texts.iter().any(|it| it == said),
                "{said}: {:?}",
                drawn.texts
            );
        }
        assert!(drawn.name.is_none(), "no name when opening");
        assert!(drawn.cancel.loc.x + drawn.cancel.size.w <= drawn.button.loc.x);
    }
}

/// **Saving draws the name**, holding what the application suggested, and
/// *Save*.
#[test]
fn saving_draws_the_name() {
    let (_portal, sheet) = a_sheet(ToChoose::Save {
        name: "notes.txt".to_owned(),
    });
    let drawn = drawn_at(&sheet, (1440, 960), 100).unwrap();
    for said in [
        "Save a file",
        "Text Editor asked to save a file",
        "Name",
        "notes.txt",
        "Save",
    ] {
        assert!(
            drawn.texts.iter().any(|it| it == said),
            "{said}: {:?}",
            drawn.texts
        );
    }
    assert!(drawn.name.is_some());
}

/// **As many rows as the display holds**: more on a tall display than a short
/// one, never fewer than the fewest or more than the most, and the row the
/// keyboard is on always among them.
#[test]
fn the_rows_follow_the_display() {
    let (_portal, mut sheet) = a_sheet(ToChoose::Open);
    let tall = drawn_at(&sheet, (1440, 1400), 100).unwrap();
    let short = drawn_at(&sheet, (1440, 700), 100).unwrap();
    assert_eq!(tall.rows.len(), THE_MOST_ROWS);
    assert!(short.rows.len() < tall.rows.len(), "{}", short.rows.len());
    assert!(short.rows.len() >= THE_FEWEST_ROWS);
    for _ in 0..20 {
        sheet.pressed(Key::Down, Instant::now(), &AHome);
    }
    let focus = sheet.shows().unwrap().chooser.focus();
    for size in [(1440, 1400), (1440, 700)] {
        let drawn = drawn_at(&sheet, size, 100).unwrap();
        assert!(drawn.rows.iter().any(|(at, _)| *at == focus), "{size:?}");
    }
}

/// **Clear of the Bar at every display size**: with the Bar's band where a
/// display draws it, the sheet's foot is above the band's top — on a small
/// display, the design's, and a tall dense one.
#[test]
fn the_sheet_is_clear_of_the_bar_at_every_size() {
    let (_portal, sheet) = a_sheet(ToChoose::Save {
        name: "notes.txt".to_owned(),
    });
    for (size, per_cent) in [((1280, 720), 100), ((1440, 960), 100), ((2880, 1920), 200)] {
        // A band a tenth of the display tall, floating a little above the
        // bottom edge, as the Dock does: a proportion, not a figure.
        let band = Rectangle::new(
            (size.0 / 4, size.1 - size.1 / 10 - size.1 / 60).into(),
            (size.0 / 2, size.1 / 10).into(),
        );
        let on = crate::where_a_sheet_sits::OnTheDisplay::with_the_bar_at(size, Some(band));
        let mut labels = WindowControlLabels::new().unwrap();
        let drawn = drawn(
            sheet.shows().unwrap(),
            &words(),
            &mut labels,
            on,
            crate::settings_testing::light(),
            DisplayScale::of_per_cent(per_cent),
            &called,
        )
        .unwrap();
        assert!(
            drawn.sheet.loc.y + drawn.sheet.size.h <= band.loc.y,
            "{size:?}: the sheet {:?} reaches the Bar {band:?}",
            drawn.sheet
        );
        assert!(
            drawn.button.loc.y + drawn.button.size.h <= band.loc.y,
            "{size:?}"
        );
    }
}

/// **A display too small to hold the sheet is refused**, never drawn in part.
#[test]
fn a_display_too_small_is_refused() {
    let (_portal, sheet) = a_sheet(ToChoose::Open);
    assert!(drawn_at(&sheet, (200, 960), 100).is_err());
    assert!(drawn_at(&sheet, (1440, 250), 100).is_err());
}
