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

/// The nth Place.
fn place(n: u64) -> Place {
    Place::numbered(n).unwrap()
}

/// The Place a machine starts on, which most of these tests only need one of.
fn first() -> Place {
    Place::FIRST
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
        first(),
        Camera::new()
            .looking_at(at(-4_200, 1_337))
            .unwrap()
            .zoomed_to(Zoom::of(2_500).unwrap(), (0, 0))
            .unwrap(),
    );
    left.window_was(first(), "org.alo.Notes", at(120, -80), size(800, 600));
    left.window_was(first(), "org.alo.Ledger", at(3_000, 2_000), size(640, 480));

    let back = Arrangement::read(&left.written()).unwrap();

    assert_eq!(
        back, left,
        "the canvas came back different from how it was left"
    );
    assert_eq!(
        back.camera_on(first()).unwrap().zoom(),
        Zoom::of(2_500).unwrap()
    );
    assert_eq!(back.camera_on(first()).unwrap().at(), at(-4_200, 1_337));
    assert_eq!(
        back.where_it_was(first(), "org.alo.Notes"),
        Some((at(120, -80), size(800, 600)))
    );
}

/// **A canvas nobody arranged remembers no Place at all.**
///
/// The first sign-in on a new machine, and the case a missing file has to behave
/// as. **This test changed its claim when the file gained Places, and the change
/// is the point rather than a consequence.** It used to assert that a fresh
/// arrangement's camera *is the origin at life size*, because version 1 had one
/// camera and it always had a value. There is now no camera until a Place has
/// one, and `camera_on` answers [`None`].
///
/// *This Place was never left anywhere* and *this Place was left at the origin*
/// are different answers, and only one of them should move a person's view. The
/// old shape could not tell them apart — which is why a restore on a Place
/// nobody had arranged would have been taken to the origin as though somebody
/// had chosen it.
#[test]
fn a_canvas_nobody_arranged_remembers_no_place() {
    let fresh = Arrangement::fresh();
    assert_eq!(fresh.camera_on(first()), None);
    assert_eq!(fresh.how_many(), 0);
    assert_eq!(fresh.each_place().count(), 0);
    assert_eq!(Arrangement::read(&fresh.written()).unwrap(), fresh);
}

/// **A Place left at the origin is remembered, and is not the same as no Place.**
///
/// The other half of the distinction above, which is worth its own test because
/// the two are one line apart in the implementation and an `unwrap_or_default`
/// would collapse them.
#[test]
fn a_place_left_at_the_origin_is_not_the_same_as_no_place() {
    let mut left = Arrangement::fresh();
    left.looking(first(), Camera::new());

    assert_eq!(left.camera_on(first()), Some(Camera::new()));
    assert_eq!(left.camera_on(place(2)), None);
    assert_eq!(left.each_place().count(), 1);

    let back = Arrangement::read(&left.written()).unwrap();
    assert_eq!(
        back, left,
        "a Place with nothing on it did not survive a round trip"
    );
}

/// **Two Places with different cameras and different frames both come back as
/// they were.**
///
/// Task 5's acceptance sentence, and the thing version 1 could not express: it
/// held one camera, so a person who left two Places at two zooms had one of them
/// restored wrongly and nothing said so.
#[test]
fn two_places_come_back_with_their_own_cameras_and_their_own_frames() {
    let (one, two) = (first(), place(2));
    let mut left = Arrangement::fresh();

    left.looking(
        one,
        Camera::new()
            .looking_at(at(-4_200, 1_337))
            .unwrap()
            .zoomed_to(Zoom::of(2_500).unwrap(), (0, 0))
            .unwrap(),
    );
    left.window_was(one, "org.alo.Notes", at(120, -80), size(800, 600));

    left.looking(
        two,
        Camera::new()
            .looking_at(at(9_000, -9_000))
            .unwrap()
            .zoomed_to(Zoom::of(400).unwrap(), (0, 0))
            .unwrap(),
    );
    left.window_was(two, "org.alo.Ledger", at(3_000, 2_000), size(640, 480));

    let back = Arrangement::read(&left.written()).unwrap();
    assert_eq!(back, left);

    // And named individually, so a round trip that swapped the two would fail
    // rather than compare equal to itself.
    assert_eq!(
        back.camera_on(one).unwrap().zoom(),
        Zoom::of(2_500).unwrap()
    );
    assert_eq!(back.camera_on(two).unwrap().zoom(), Zoom::of(400).unwrap());
    assert_eq!(back.camera_on(one).unwrap().at(), at(-4_200, 1_337));
    assert_eq!(back.camera_on(two).unwrap().at(), at(9_000, -9_000));
    assert_eq!(back.how_many_on(one), 1);
    assert_eq!(back.how_many_on(two), 1);
    assert_eq!(back.how_many(), 2);
}

/// **One application is remembered once per Place, not once in the world.**
///
/// `one_application_has_one_place` asserted the opposite and was right about
/// version 1: there was one map, so a second row for an `app_id` replaced the
/// first. The same application open on two Places now has two remembered places,
/// and writing it twice on *one* Place still replaces.
#[test]
fn one_application_has_one_place_on_each_place() {
    let (one, two) = (first(), place(2));
    let mut left = Arrangement::fresh();

    left.window_was(one, "org.alo.Notes", at(0, 0), size(100, 100));
    left.window_was(two, "org.alo.Notes", at(500, 500), size(200, 200));
    assert_eq!(
        left.how_many(),
        2,
        "one application on two Places is two places"
    );

    left.window_was(one, "org.alo.Notes", at(7, 7), size(300, 300));
    assert_eq!(
        left.how_many_on(one),
        1,
        "a second row on one Place replaces"
    );
    assert_eq!(
        left.where_it_was(one, "org.alo.Notes"),
        Some((at(7, 7), size(300, 300)))
    );
    assert_eq!(
        left.where_it_was(two, "org.alo.Notes"),
        Some((at(500, 500), size(200, 200)))
    );
}

/// **A key that is not a Place is a file this does not read.**
///
/// The boundary rule applied to an identity rather than a position: the raw
/// number crossed the disk, so it is validated here. Zero is refused because it
/// is what serde invents, and a word is refused because it is not a number at
/// all — both by name, so a person can see which line of their file is wrong.
#[test]
fn a_place_key_that_is_not_a_number_is_refused() {
    for key in ["0", "notaplace", "-1", "1.5"] {
        let written =
            format!("version = 2\n\n[places.\"{key}\"]\nlooking-at = [0, 0]\nzoom = 1000\n");
        assert!(
            matches!(
                Arrangement::read(&written),
                Err(NotArranged::NotOnThePlane(_))
            ),
            "a Place keyed {key:?} was read as a Place"
        );
    }
    // And the smallest real one is read.
    let good = "version = 2\n\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n";
    assert_eq!(Arrangement::read(good).unwrap().each_place().count(), 1);
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
    let off_the_plane = "version = 2\n\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n\n[places.\"1\".windows.\"org.alo.Notes\"]\n\
                         x = 99999999\ny = 0\nwidth = 800\nheight = 600\n";
    assert!(matches!(
        Arrangement::read(off_the_plane),
        Err(NotArranged::NotOnThePlane(_))
    ));

    let no_such_zoom = "version = 2\n\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 999999\n";
    assert!(matches!(
        Arrangement::read(no_such_zoom),
        Err(NotArranged::NotOnThePlane(_))
    ));

    let no_width = "version = 2\n\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n\n[places.\"1\".windows.\"org.alo.Notes\"]\n\
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
        Arrangement::read(
            "version = 2\nwallpaper = \"none\"\n\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n"
        ),
        Err(NotArranged::Unreadable(_))
    ));
}

/// **One application has one remembered place *on one Place*.**
///
/// **This test kept its assertions and lost half its claim**, which is worth
/// saying rather than renaming quietly. It was named
/// `one_application_has_one_place` and was right about version 1: there was one
/// map keyed by `app_id`, so a second row anywhere replaced the first. With a map
/// per Place that is now only true *within* a Place, and
/// `one_application_has_one_place_on_each_place` holds the other half. A test
/// whose name outlives its subject is the fault this repository keeps
/// cataloguing, so the name moved with the claim.
#[test]
fn one_application_has_one_remembered_place_on_a_single_place() {
    let mut left = Arrangement::fresh();
    left.window_was(first(), "org.alo.Notes", at(0, 0), size(100, 100));
    left.window_was(first(), "org.alo.Notes", at(500, 500), size(200, 200));
    assert_eq!(left.how_many(), 1);
    assert_eq!(
        left.where_it_was(first(), "org.alo.Notes"),
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
    left.window_was(first(), "org.alo.Notes", at(10, 10), size(100, 100));
    let back = Arrangement::read(&left.written()).unwrap();
    assert_eq!(back.where_it_was(first(), "org.alo.Ledger"), None);
    assert_eq!(back.how_many(), 1);
}

/// **What this crate writes says which shape it is.**
///
/// The whole point of the number: a file with no version is not from a version
/// that wrote one, and that has to be distinguishable from a file of this one.
#[test]
fn what_is_written_says_its_version_and_is_read_back() {
    let mut left = Arrangement::fresh();
    left.window_was(first(), "org.alo.Notes", at(40, 50), size(800, 600));
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
    let was_valid_before_versioning = "\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n";
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
        "version = {}\n\n[places.\"1\"]\nlooking-at = [0, 0]\nzoom = 1000\n",
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
        "version = {}\n\n[places.\"1\"]\nlooking-at = [99999999, 0]\nzoom = 1000\n",
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
