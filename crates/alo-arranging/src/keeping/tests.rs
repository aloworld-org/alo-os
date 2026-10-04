//! Whether a layout reaches a person's folder, and what happens when it does not.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
use super::*;

/// A fixed moment, so a series' order is the order the test made it in rather
/// than the order a clock happened to tick.
fn noon() -> std::time::SystemTime {
    std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000)
}
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

    keep(&at_file, &left, noon()).unwrap();
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
    keep(&at_file, &before, noon()).unwrap();

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
    // **The bytes survive, at the slot beside it.** This asserted that the file
    // stayed where it was, which was true when written and is wrong by design
    // since 2026-10-03: a layout that does not read is moved aside so the next
    // save cannot overwrite it. What the assertion was reaching for is that
    // nothing is destroyed, and that is what it says now.
    assert!(
        !at_file.exists(),
        "a layout that did not read was left where the next save would overwrite it"
    );
    assert_eq!(
        std::fs::read_to_string(at_file.with_file_name(THE_ONE_THAT_DID_NOT_READ)).unwrap(),
        "version = 1\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n",
        "the bytes that did not read were not kept beside the file"
    );
}

/// **And the next save mends it**, so the damage costs a remembered layout and
/// nothing more.
#[test]
fn the_next_save_replaces_a_damaged_file() {
    let at_file = a_folder("mended").join(THE_FILE);
    std::fs::write(&at_file, "not a layout at all [").unwrap();

    keep(&at_file, &two_places(), noon()).unwrap();
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

/// **A layout from an older version of our own format is kept, not lost.**
///
/// The case the owner decided on 2026-10-03, and the one that forces this to
/// exist. `read` refuses a version that is not this one by strict equality, so
/// without the move the first format change after a release would discard every
/// person's arrangement at their next save — and **a file written by an older
/// version of our own format is not wrong, it is old.**
///
/// What is asserted is not the refusal, which `arranging_tests` already holds.
/// It is that **the bytes are still on the disk afterwards**, which is the whole
/// of what preserving means and the only thing a later migration, a support
/// question or an explanation could ever be built on.
#[test]
fn a_layout_from_another_version_is_kept_beside_itself() {
    let folder = a_folder("another-version");
    let at = folder.join(THE_FILE);
    let older = "version = 2\n\n[places.\"1\"]\nlooking-at = [40, 50]\nzoom = 1000\n";
    std::fs::write(&at, older).unwrap();

    let (back, why) = at_sign_in(&at);

    assert_eq!(
        back,
        Arrangement::fresh(),
        "an older file stopped the canvas"
    );
    assert!(
        matches!(why, Some(NotReadBack::NotALayout(_))),
        "an older version was not named: {why:?}"
    );
    assert_eq!(
        std::fs::read_to_string(folder.join(THE_ONE_THAT_DID_NOT_READ)).unwrap(),
        older,
        "the older layout was not kept, so the format change cost it"
    );

    // And the next save writes cleanly rather than being refused forever,
    // which is the half `alo_kept::keep`'s own rule would have got wrong here.
    keep(&at, &two_places(), noon()).unwrap();
    let (now, why) = at_sign_in(&at);
    assert_eq!(why, None);
    assert_eq!(now, two_places());
    assert_eq!(
        std::fs::read_to_string(folder.join(THE_ONE_THAT_DID_NOT_READ)).unwrap(),
        older,
        "writing a good layout destroyed the one that had been kept"
    );
}

/// **The slot holds the most recent file that did not read.**
///
/// Overwriting an older set-aside copy is the decision rather than an accident:
/// between two failures this crate wrote at least one file that *did* read, so
/// the earlier copy has already been superseded by a layout the person then
/// used. Asserted because the alternative — keeping the first forever — is just
/// as defensible and somebody will wonder which was chosen.
#[test]
fn the_slot_holds_the_most_recent_one_that_did_not_read() {
    let folder = a_folder("most-recent");
    let at = folder.join(THE_FILE);

    std::fs::write(&at, "version = 1\n").unwrap();
    let _first = at_sign_in(&at);
    keep(&at, &two_places(), noon()).unwrap();
    std::fs::write(&at, "version = 2\n").unwrap();
    let _second = at_sign_in(&at);

    assert_eq!(
        std::fs::read_to_string(folder.join(THE_ONE_THAT_DID_NOT_READ)).unwrap(),
        "version = 2\n",
        "the slot kept the older failure rather than the most recent one"
    );
}

/// A moment after [`noon`], so two saves are two stops on a ribbon.
fn later() -> std::time::SystemTime {
    noon() + std::time::Duration::from_secs(3_600)
}

/// **A Place remembers what it was, through the file.**
///
/// Task 8 of `docs/autonomy/the-canvas-and-its-places.md`, and the half of it
/// a disk can show: two saves, and the first layout is on the second's ribbon
/// with the moment it stopped being current.
#[test]
fn a_place_keeps_what_it_was_when_the_next_layout_is_kept() {
    let folder = a_folder("remembers");
    let at_file = folder.join("canvas.toml");

    keep(&at_file, &two_places(), noon()).unwrap();

    let mut moved = two_places();
    moved.looking(first(), a_camera(0, 0, 1_000));
    keep(&at_file, &moved, later()).unwrap();

    let (read_back, why) = at_sign_in(&at_file);
    assert!(why.is_none(), "{why:?}");
    assert_eq!(read_back.camera_on(first()), moved.camera_on(first()));

    let ribbon = read_back.earlier_on(first());
    assert_eq!(ribbon.len(), 1, "one save ago is one stop");
    assert_eq!(
        ribbon[0].when(),
        later(),
        "held when it stopped being current"
    );
    assert_eq!(
        ribbon[0].camera(),
        two_places().camera_on(first()).unwrap(),
        "and it is the camera that was there"
    );
    assert_eq!(ribbon[0].how_many(), 1);

    // **The Place that did not move has an empty ribbon.** A save is about the
    // canvas, and a Place nobody rearranged has nothing new to remember —
    // otherwise every save would put a repeat on every Place.
    assert!(read_back.earlier_on(second()).is_empty());
}

/// **A ribbon is capped, and the oldest stop is the one that goes.**
#[test]
fn a_place_remembers_no_more_than_the_cap_and_forgets_the_oldest() {
    let folder = a_folder("capped");
    let at_file = folder.join("canvas.toml");
    let saves = crate::HOW_MANY_A_PLACE_REMEMBERS + 5;

    for n in 0..saves {
        let mut moved = two_places();
        // A different camera each time, so every save is a real change and the
        // stop it leaves can be told from the others.
        moved.looking(first(), a_camera(i32::try_from(n).unwrap(), 0, 1_000));
        keep(
            &at_file,
            &moved,
            noon() + std::time::Duration::from_secs(n as u64),
        )
        .unwrap();
    }

    let (read_back, _) = at_sign_in(&at_file);
    let ribbon = read_back.earlier_on(first());
    assert_eq!(ribbon.len(), crate::HOW_MANY_A_PLACE_REMEMBERS);
    assert_eq!(
        ribbon.first().unwrap().when(),
        noon() + std::time::Duration::from_secs((saves - crate::HOW_MANY_A_PLACE_REMEMBERS) as u64),
        "the oldest stops went and the newest stayed"
    );
}

/// **A version-3 file reads, and its Places have empty ribbons.**
///
/// The first migration this crate has had, and the reason it is one:
/// `alo-desktop` has been writing version 3 to real disks since before 0.0.1
/// went public, so refusing it would cost a person the canvas they left
/// (`docs/misreadings/nothing-is-on-a-disk-is-a-fact-with-a-date-on-it.md`).
#[test]
fn a_file_from_before_a_place_remembered_anything_still_reads() {
    let folder = a_folder("version-three");
    let at_file = folder.join("canvas.toml");
    let three = "version = 3\n\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n";
    std::fs::write(&at_file, three).unwrap();

    let (read_back, why) = at_sign_in(&at_file);
    assert!(
        why.is_none(),
        "a version-3 file is read rather than refused: {why:?}"
    );
    assert!(read_back.camera_on(first()).is_some());
    assert!(
        read_back.earlier_on(first()).is_empty(),
        "a Place whose earlier states were never kept has an empty ribbon"
    );

    // And the next save starts its ribbon rather than refusing the file.
    let mut moved = Arrangement::fresh();
    moved.looking(first(), a_camera(10, 10, 2_000));
    keep(&at_file, &moved, later()).unwrap();
    let (after, _) = at_sign_in(&at_file);
    assert_eq!(after.earlier_on(first()).len(), 1);
}
