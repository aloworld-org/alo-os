//! What this crate keeps, and what it refuses to keep.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// A place on the plane.
fn at(x: i32, y: i32) -> At {
    At::checked(x, y).unwrap()
}

/// A size a frame may be.
fn size(width: u32, height: u32) -> Size {
    Size::checked(width, height).unwrap()
}

/// **What was written is what is read back, exactly.**
///
/// Task 9's acceptance says *positions and camera are restored exactly*, so this
/// compares the whole arrangement rather than sampling it: a round trip that
/// dropped the zoom, or rounded a position, would pass a test that only looked at
/// the windows.
#[test]
fn what_was_left_is_what_comes_back() {
    let mut left = Arrangement::fresh();
    left.looking(
        Camera::new()
            .looking_at(at(-4_200, 1_337))
            .unwrap()
            .zoomed_to(Zoom::of(2_500).unwrap(), (0, 0))
            .unwrap(),
    );
    left.window_was("org.alo.Notes", at(120, -80), size(800, 600));
    left.window_was("org.alo.Ledger", at(3_000, 2_000), size(640, 480));

    let back = Arrangement::read(&left.written()).unwrap();

    assert_eq!(
        back, left,
        "the canvas came back different from how it was left"
    );
    assert_eq!(back.camera().zoom(), Zoom::of(2_500).unwrap());
    assert_eq!(back.camera().at(), at(-4_200, 1_337));
    assert_eq!(
        back.where_it_was("org.alo.Notes"),
        Some((at(120, -80), size(800, 600)))
    );
}

/// **A canvas nobody arranged reads back as one nobody arranged.**
///
/// The first sign-in on a new machine, and the case a missing file has to behave
/// as: the origin, life size, nothing placed.
#[test]
fn an_empty_arrangement_is_the_origin_at_life_size() {
    let fresh = Arrangement::fresh();
    assert_eq!(fresh.camera(), Camera::new());
    assert_eq!(fresh.how_many(), 0);
    assert_eq!(Arrangement::read(&fresh.written()).unwrap(), fresh);
}

/// **A file hand-edited off the plane is refused, not clamped.**
///
/// The reason this crate rebuilds every value through `alo-canvas` rather than
/// deserialising a `Camera`: a position read back without being checked is a
/// position nothing validated. Clamping would be worse than refusing, because a
/// person whose file was quietly corrected would have a canvas that disagreed
/// with what they could read in it.
#[test]
fn a_place_no_canvas_has_is_refused() {
    let off_the_plane = "version = 1\nlooking-at = [0, 0]\nzoom = 1000\n\n[windows.\"org.alo.Notes\"]\n\
                         x = 99999999\ny = 0\nwidth = 800\nheight = 600\n";
    assert!(matches!(
        Arrangement::read(off_the_plane),
        Err(NotArranged::NotOnThePlane(_))
    ));

    let no_such_zoom = "version = 1\nlooking-at = [0, 0]\nzoom = 999999\n";
    assert!(matches!(
        Arrangement::read(no_such_zoom),
        Err(NotArranged::NotOnThePlane(_))
    ));

    let no_width = "version = 1\nlooking-at = [0, 0]\nzoom = 1000\n\n[windows.\"org.alo.Notes\"]\n\
                    x = 0\ny = 0\nwidth = 0\nheight = 600\n";
    assert!(matches!(
        Arrangement::read(no_width),
        Err(NotArranged::NotOnThePlane(_))
    ));
}

/// **A file of another shape is refused rather than half-read.**
#[test]
fn something_that_is_not_an_arrangement_is_refused() {
    assert!(matches!(
        Arrangement::read("this is not toml at all ["),
        Err(NotArranged::Unreadable(_))
    ));
    // An unknown key is a file written by something else, or by a later version
    // of this one. Either way it is not read half-way.
    //
    // **Otherwise a valid file of this version**, so that what is being refused
    // is the unknown key and not a missing version. Before `FORMAT` existed
    // these fixtures said nothing about a version and could not tell the two
    // apart; one of them now would be refused for the wrong reason and still
    // pass.
    assert!(matches!(
        Arrangement::read("version = 1\nlooking-at = [0, 0]\nzoom = 1000\nwallpaper = \"none\"\n"),
        Err(NotArranged::Unreadable(_))
    ));
}

/// **One application has one remembered place**, which is the consequence of
/// keying by `app_id` and is stated rather than discovered.
#[test]
fn one_application_has_one_place() {
    let mut left = Arrangement::fresh();
    left.window_was("org.alo.Notes", at(0, 0), size(100, 100));
    left.window_was("org.alo.Notes", at(500, 500), size(200, 200));
    assert_eq!(left.how_many(), 1);
    assert_eq!(
        left.where_it_was("org.alo.Notes"),
        Some((at(500, 500), size(200, 200))),
        "the second window of one application did not take the place"
    );
}

/// **An application that did not come back is simply not there.**
///
/// Task 9's second acceptance sentence lives here and in the shell: this crate's
/// half is that asking about an application nothing kept answers nothing, rather
/// than a default place that would draw an empty frame.
#[test]
fn an_application_that_never_came_back_has_no_place() {
    let mut left = Arrangement::fresh();
    left.window_was("org.alo.Notes", at(10, 10), size(100, 100));
    let back = Arrangement::read(&left.written()).unwrap();
    assert_eq!(back.where_it_was("org.alo.Ledger"), None);
    assert_eq!(back.how_many(), 1);
}

/// **What this crate writes says which shape it is.**
///
/// The whole point of the number: a file with no version is not from a version
/// that wrote one, and that has to be distinguishable from a file of this one.
#[test]
fn what_is_written_says_its_version_and_is_read_back() {
    let mut left = Arrangement::fresh();
    left.window_was("org.alo.Notes", at(40, 50), size(800, 600));
    let written = left.written();

    assert!(
        written.contains(&format!("version = {FORMAT}")),
        "an arrangement was written without its version: {written}"
    );
    assert_eq!(Arrangement::read(&written), Ok(left));
}

/// **A file with no version is refused by name, not as a parse error.**
///
/// This is the case that did not exist before: every file ever written by this
/// crate said `version`, so one that does not is from something else. Refused as
/// [`NotArranged::AnotherVersion`] with zero, because `#[serde(default)]` makes
/// *absent* and *zero* the same arrival and zero is not a version anybody wrote.
#[test]
fn a_file_with_no_version_is_not_this_shape() {
    let was_valid_before_versioning = "looking-at = [0, 0]\nzoom = 1000\n";
    assert_eq!(
        Arrangement::read(was_valid_before_versioning),
        Err(NotArranged::AnotherVersion {
            said: 0,
            reads: FORMAT
        })
    );
}

/// **A later version is refused, and the refusal says which way round it is.**
///
/// A person whose file is newer than their binary has a working file and the
/// wrong program to open it with, which is a different sentence from *this is
/// damaged* — so it is a different variant, carrying both numbers.
#[test]
fn a_version_from_the_future_is_refused_with_both_numbers() {
    let later = format!(
        "version = {}\nlooking-at = [0, 0]\nzoom = 1000\n",
        FORMAT + 1
    );
    assert_eq!(
        Arrangement::read(&later),
        Err(NotArranged::AnotherVersion {
            said: FORMAT + 1,
            reads: FORMAT
        })
    );
}

/// **The version is checked before any value in it is trusted.**
///
/// A file of another shape may parse cleanly and mean something else — the same
/// keys with a Place keyed above them, which is exactly what task 5 will write.
/// So a file whose version is wrong **and** whose values are off the plane is
/// refused for its version, because that is the true reason and the one a person
/// can act on.
#[test]
fn the_version_is_refused_before_the_values_are() {
    let both_wrong = format!(
        "version = {}\nlooking-at = [99999999, 0]\nzoom = 1000\n",
        FORMAT + 1
    );
    assert_eq!(
        Arrangement::read(&both_wrong),
        Err(NotArranged::AnotherVersion {
            said: FORMAT + 1,
            reads: FORMAT
        })
    );
}
