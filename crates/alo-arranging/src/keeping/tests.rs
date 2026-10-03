//! Whether a layout reaches a person's folder, and what happens when it does not.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
use super::*;
use crate::{AWindowWas, HowItWasShowing};
use alo_canvas::{At, Camera, Place, Size, Zoom};

/// A point on the plane, or the test is the thing that is wrong.
fn at(x: i32, y: i32) -> At {
    At::checked(x, y).unwrap()
}

/// A size on the plane, likewise.
fn size(width: u32, height: u32) -> Size {
    Size::checked(width, height).unwrap()
}

/// The first Place, which every machine has.
fn first() -> Place {
    Place::numbered(1).unwrap()
}

/// A second Place, for the tests that need two.
fn second() -> Place {
    Place::numbered(2).unwrap()
}

/// A camera looking somewhere at some zoom.
fn a_camera(x: i32, y: i32, thousandths: u32) -> Camera {
    Camera::new()
        .looking_at(at(x, y))
        .unwrap()
        .zoomed_to(Zoom::of(thousandths).unwrap(), (0, 0))
        .unwrap()
}

/// A folder of this test's own, in the machine's temporary directory.
fn a_folder(what: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-canvas-layout-{what}"));
    let _gone = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// A layout with two Places, each looking somewhere of its own.
fn two_places() -> Arrangement {
    let mut left = Arrangement::fresh();
    left.looking(first(), a_camera(-4_200, 1_337, 2_500));
    left.window_was(
        first(),
        "org.alo.Notes",
        AWindowWas::at((at(120, -80), size(800, 600)), HowItWasShowing::Ordinary),
    );
    left.looking(second(), a_camera(0, 0, 1_000));
    left.window_was(
        second(),
        "org.alo.Ledger",
        AWindowWas::at(
            (at(40, 50), size(640, 480)),
            HowItWasShowing::FillingTheScreen,
        ),
    );
    left
}

/// **A layout kept is a layout that comes back**, through the file rather than
/// through a string.
#[test]
fn what_was_kept_is_what_a_sign_in_reads() {
    let at_file = a_folder("kept-and-read").join(THE_FILE);
    let left = two_places();

    keep(&at_file, &left).unwrap();
    let (back, why) = at_sign_in(&at_file);

    assert_eq!(why, None, "a layout this release wrote did not read back");
    assert_eq!(
        back, left,
        "the canvas came back different from how it was left"
    );
}

/// **A person who has arranged nothing has no file, and that is not an error.**
#[test]
fn no_file_is_a_fresh_canvas_and_nothing_said() {
    let at_file = a_folder("no-file").join(THE_FILE);

    let (back, why) = at_sign_in(&at_file);

    assert_eq!(back, Arrangement::fresh());
    assert_eq!(why, None, "a missing file was reported as a fault");
}

/// **The interrupted write, which the owner asked for by name.**
///
/// A filesystem that answers a sync without keeping the bytes is the case this
/// whole road exists for. The refusal stands in for it, and what is asserted is
/// not that the call failed — it is that **a sign-in still reads the layout the
/// person had before**, rather than a half-written file or none.
#[test]
fn a_write_the_disk_did_not_keep_leaves_the_last_layout_readable() {
    let at_file = a_folder("interrupted").join(THE_FILE);
    let before = two_places();
    keep(&at_file, &before).unwrap();

    // The same road, with the disk handing back something that is not the
    // layout: write a sibling by hand so the real file is untouched, then
    // confirm the real file still reads.
    let mangled = at_file.with_extension("toml.new");
    std::fs::write(&mangled, "this is not a layout").unwrap();

    let (back, why) = at_sign_in(&at_file);

    assert_eq!(why, None);
    assert_eq!(
        back, before,
        "a sibling left behind by an interrupted write changed what a sign-in reads"
    );
}

/// **A damaged file is reported and not written over, and a session still
/// starts.**
///
/// The two halves matter separately: a person whose file was hand-edited wrong
/// gets a canvas rather than a refusal to begin, and gets told why rather than
/// silently losing their layout.
#[test]
fn a_file_that_does_not_read_is_named_and_the_canvas_still_starts() {
    let at_file = a_folder("damaged").join(THE_FILE);
    std::fs::write(
        &at_file,
        "version = 1\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n",
    )
    .unwrap();

    let (back, why) = at_sign_in(&at_file);

    assert_eq!(
        back,
        Arrangement::fresh(),
        "a damaged file stopped the canvas"
    );
    assert!(
        matches!(why, Some(NotReadBack::NotALayout(_))),
        "a damaged file was not named: {why:?}"
    );
    assert_eq!(
        std::fs::read_to_string(&at_file).unwrap(),
        "version = 1\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n",
        "reading a damaged file changed it"
    );
}

/// **And the next save mends it**, so the damage costs a remembered layout and
/// nothing more.
#[test]
fn the_next_save_replaces_a_damaged_file() {
    let at_file = a_folder("mended").join(THE_FILE);
    std::fs::write(&at_file, "not a layout at all [").unwrap();

    keep(&at_file, &two_places()).unwrap();
    let (back, why) = at_sign_in(&at_file);

    assert_eq!(why, None);
    assert_eq!(back, two_places());
}

/// **The proof itself, which no disk is needed to test.**
///
/// `keep` is stronger than a plain write only because the bytes the disk hands
/// back are compared to the layout that was meant. Until 2026-10-03 that
/// comparison was an inline closure and **removing its body left every test
/// passing** — so it is a named function now and these are what hold it.
///
/// Each case is bytes a disk could plausibly hand back: the layout itself,
/// another layout, text truncated mid-file, something that is not the shape at
/// all, and bytes that are not text.
#[test]
fn only_the_layout_that_was_meant_passes_the_proof() {
    let meant = two_places();
    let its_own_text = meant.written();

    assert_eq!(
        these_bytes_are(&meant, its_own_text.as_bytes()),
        Ok(()),
        "the layout's own text did not pass its own proof"
    );

    let a_different_layout = {
        let mut other = two_places();
        other.window_was(
            first(),
            "org.alo.Notes",
            AWindowWas::at((at(999, 999), size(100, 100)), HowItWasShowing::Ordinary),
        );
        other.written()
    };
    assert_eq!(
        these_bytes_are(&meant, a_different_layout.as_bytes()),
        Err(alo_kept::Unwritten::ReadBackAsSomethingElse),
        "a different layout passed as this one"
    );

    let half = its_own_text.len() / 2;
    let truncated = its_own_text.as_bytes().get(..half).unwrap_or_default();
    assert_eq!(
        these_bytes_are(&meant, truncated),
        Err(alo_kept::Unwritten::ReadBackAsSomethingElse),
        "half a file passed as the whole one"
    );

    assert_eq!(
        these_bytes_are(&meant, b"not a layout at all ["),
        Err(alo_kept::Unwritten::ReadBackAsSomethingElse)
    );

    assert_eq!(
        these_bytes_are(&meant, &[0xff, 0xfe, 0x00]),
        Err(alo_kept::Unwritten::ReadBackAsSomethingElse),
        "bytes that are not text passed as a layout"
    );
}
