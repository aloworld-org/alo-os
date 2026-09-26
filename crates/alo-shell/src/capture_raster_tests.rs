//! What the capture tools draw, and the one thing they must never draw
//! differently from what gets saved.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;

/// The ordinary light look.
fn light() -> CaptureLook {
    CaptureLook {
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
        contrast: Contrast::AsDesigned,
    }
}

/// The tools drawn on a 1920×1080 output.
fn drawn(capturing: Capturing<'_>) -> CapturePicture {
    picture(capturing, (1920, 1080), light()).unwrap()
}

/// Whether any shape covers this point.
fn painted_at(picture: &CapturePicture, (x, y): (i32, i32)) -> bool {
    picture.solids.iter().any(|solid| {
        (solid.area.loc.x..solid.area.loc.x + solid.area.size.w).contains(&x)
            && (solid.area.loc.y..solid.area.loc.y + solid.area.size.h).contains(&y)
    })
}

/// **Nobody capturing draws nothing at all.**
#[test]
fn a_screen_nobody_is_capturing_has_no_tools_on_it() {
    assert!(
        drawn(Capturing {
            choosing: None,
            marked: &[],
        })
        .is_empty()
    );
}

/// **The region is an outline and its middle is untouched.**
///
/// What a person is deciding is what will be in the picture, and a filled
/// rectangle hides exactly that.
#[test]
fn the_region_being_chosen_is_an_outline_and_not_a_fill() {
    let region = Region::of(200, 100, 400, 300).unwrap();
    let picture = drawn(Capturing {
        choosing: Some(&What::APartOfIt(region)),
        marked: &[],
    });

    assert!(painted_at(&picture, (200, 100)), "the corner is not drawn");
    assert!(
        painted_at(&picture, (400, 100)),
        "the top edge is not drawn"
    );
    assert!(
        !painted_at(&picture, (400, 250)),
        "the middle of the region is painted over, hiding what is being captured"
    );
}

/// **Nothing outside the region is dimmed.**
///
/// A dim is a second way of saying where the edge is, and it costs the person
/// the sight of what they are about to capture while they are judging it.
#[test]
fn what_is_outside_the_region_is_left_alone() {
    let picture = drawn(Capturing {
        choosing: Some(&What::APartOfIt(Region::of(200, 100, 400, 300).unwrap())),
        marked: &[],
    });

    for point in [(10, 10), (1900, 1000), (100, 500), (900, 50)] {
        assert!(
            !painted_at(&picture, point),
            "the screen outside the region is dimmed at {point:?}"
        );
    }
}

/// **A blur is drawn as what a blur becomes: a flat block.**
///
/// `crate::capture_flatten` replaces a hidden area with one flat colour, for
/// good. Drawing it soft here would be this machine showing a person one thing
/// and saving another, at the one moment they are deciding whether a colleague
/// may see what is underneath.
#[test]
fn a_blur_is_drawn_as_the_solid_block_it_will_be_saved_as() {
    let over = Region::of(50, 60, 120, 80).unwrap();
    let picture = drawn(Capturing {
        choosing: None,
        marked: &[Mark::Blur { over }],
    });

    // Every point inside it is covered, middle included — the opposite of the
    // region outline above, and deliberately so.
    for point in [(50, 60), (110, 100), (169, 139)] {
        assert!(
            painted_at(&picture, point),
            "the hidden area is not solid at {point:?}: something underneath shows through"
        );
    }
    assert_eq!(
        picture.hidden.len(),
        1,
        "the hidden area was not recorded as hidden"
    );
    let hidden = picture.hidden.first().unwrap();
    assert_eq!((hidden.size.w, hidden.size.h), (120, 80));
}

/// **A box drawn round something does not hide it.**
///
/// The mark a person uses to point at something must not be the mark that
/// destroys it. This and the blur are the two shapes that would be easiest to
/// confuse, so they are held apart here.
#[test]
fn a_box_round_something_is_an_outline_and_a_blur_is_not() {
    let over = Region::of(50, 60, 120, 80).unwrap();
    let boxed = drawn(Capturing {
        choosing: None,
        marked: &[Mark::Rectangle { over }],
    });
    let hidden = drawn(Capturing {
        choosing: None,
        marked: &[Mark::Blur { over }],
    });

    assert!(
        !painted_at(&boxed, (110, 100)),
        "a box round something covered what is inside it"
    );
    assert!(painted_at(&hidden, (110, 100)));
    assert!(
        boxed.hidden.is_empty(),
        "a box was recorded as hiding something"
    );
}

/// **An arrow runs from where it starts to what it points at.**
#[test]
fn an_arrow_is_drawn_between_its_two_ends() {
    let picture = drawn(Capturing {
        choosing: None,
        marked: &[Mark::Arrow {
            from: (100, 100),
            to: (300, 200),
        }],
    });

    assert!(painted_at(&picture, (100, 100)), "the arrow has no start");
    assert!(painted_at(&picture, (300, 200)), "the arrow has no end");
    assert!(
        painted_at(&picture, (200, 150)),
        "the arrow does not pass through the middle of its own run"
    );
}

/// **A line drawn by hand follows the hand.**
#[test]
fn a_freehand_line_passes_through_every_point_it_was_drawn_through() {
    let picture = drawn(Capturing {
        choosing: None,
        marked: &[Mark::Freehand {
            through: vec![(10, 10), (60, 10), (60, 60)],
        }],
    });

    for point in [(10, 10), (60, 10), (60, 60), (35, 10), (60, 35)] {
        assert!(
            painted_at(&picture, point),
            "the hand's line misses {point:?}"
        );
    }
}

/// **Nothing is drawn outside the screen.**
///
/// A mark made near an edge and dragged past it is ordinary, and a shape
/// reaching past the output is how a frame comes to carry pixels nobody
/// allocated.
#[test]
fn a_mark_running_off_the_screen_is_cut_at_it() {
    let picture = picture(
        Capturing {
            choosing: Some(&What::APartOfIt(Region::of(1800, 1000, 400, 300).unwrap())),
            marked: &[Mark::Blur {
                over: Region::of(1900, 1050, 200, 200).unwrap(),
            }],
        },
        (1920, 1080),
        light(),
    )
    .unwrap();

    for solid in &picture.solids {
        assert!(
            solid.area.loc.x >= 0
                && solid.area.loc.y >= 0
                && solid.area.loc.x + solid.area.size.w <= 1920
                && solid.area.loc.y + solid.area.size.h <= 1080,
            "a shape reaches past the output: {:?}",
            solid.area
        );
    }
}

/// **Capturing the whole screen outlines the whole screen.**
#[test]
fn the_whole_screen_is_a_region_like_any_other() {
    let picture = drawn(Capturing {
        choosing: Some(&What::TheWholeScreen),
        marked: &[],
    });

    assert!(
        painted_at(&picture, (0, 0)),
        "the screen's corner is not drawn"
    );
    assert!(
        !painted_at(&picture, (960, 540)),
        "capturing the whole screen painted over the whole screen"
    );
}
