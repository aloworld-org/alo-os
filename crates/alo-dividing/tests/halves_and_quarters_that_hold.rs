//! A split: halves and quarters that hold — each acceptance criterion of the
//! task, walked through this crate's public surface as the shell will reach it.
//!
//! The crate's own unit tests take each operation apart. This file is the
//! other half: one test per promise, written against nothing but what a
//! compositor can call, and holding a refusal to the sentence a person reads.
//!
//! It is not the hardware verification `CLAUDE.md` asks for. Nothing here has
//! been dragged: there is no compositor drawing these shares yet, and a person
//! at a certified machine is who says a split holds under a hand.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_dividing::{
    Area, Division, Offer, Place, Point, Proposal, Refused, Scale, Side, Size, Window, WindowId,
    dividing_words, side_bound_to,
};
use alo_shortcuts::{Action, Chord, Key, Modifier, Modifiers, Shortcuts};
use alo_strings::Strings;

/// A window stating no minimum.
fn any(id: u64) -> Window {
    Window::any_size(WindowId::from_compositor(id))
}

/// The window the compositor calls `id`.
fn id(id: u64) -> WindowId {
    WindowId::from_compositor(id)
}

/// An area in logical units.
fn area(x: u32, y: u32, width: u32, height: u32) -> Area {
    Area::of(Point::at(x, y), Size::of(width, height)).unwrap()
}

/// A 2560 by 1440 display.
fn display() -> Area {
    area(0, 0, 2560, 1440)
}

/// The proposal in an offer.
fn proposed(offer: Offer) -> Proposal {
    match offer {
        Offer::Proposed(proposal) => proposal,
        other => {
            assert!(matches!(other, Offer::Proposed(_)), "{other:?}");
            unreachable!()
        }
    }
}

/// Every share as a window number and its area.
fn laid_out(division: &Division) -> Vec<(u64, Area)> {
    division
        .shares()
        .into_iter()
        .map(|share| (share.window().to_compositor(), share.area()))
        .collect()
}

/// A display in halves by drag: window 1 dropped on the left edge over 2.
fn in_halves() -> Division {
    let mut division = Division::of(display());
    let half = proposed(division.propose_drop(Point::at(0, 700), any(1), Some(any(2))));
    division.commit(half).unwrap();
    division
}

/// **A division is a tree of shares — halves, quarters, and a split of an
/// existing half — each naming the window in it.**
#[test]
fn a_division_is_halves_quarters_and_a_split_half_each_naming_its_window() {
    let mut division = in_halves();
    assert_eq!(
        laid_out(&division),
        vec![(1, area(0, 0, 1280, 1440)), (2, area(1280, 0, 1280, 1440))]
    );

    // The right half split into quarters, and one quarter split again.
    division.divide(id(2), any(3), Side::Bottom).unwrap();
    division.divide(id(3), any(4), Side::Right).unwrap();
    assert_eq!(
        laid_out(&division),
        vec![
            (1, area(0, 0, 1280, 1440)),
            (2, area(1280, 0, 1280, 720)),
            (3, area(1280, 720, 640, 720)),
            (4, area(1920, 720, 640, 720)),
        ]
    );
    for (window, place) in [
        (1, Place::LeftHalf),
        (2, Place::TopRightQuarter),
        (3, Place::Part),
    ] {
        assert_eq!(
            Place::of(division.share_of(id(window)).unwrap(), display()),
            place
        );
    }
}

/// **Resizing the boundary between two shares resizes both.**
#[test]
fn resizing_the_boundary_between_two_shares_resizes_both() {
    let mut division = in_halves();
    division.divide(id(2), any(3), Side::Bottom).unwrap();

    division.move_boundary(id(1), Side::Right, 1000).unwrap();
    assert_eq!(division.share_of(id(1)), Some(area(0, 0, 1000, 1440)));
    assert_eq!(division.share_of(id(2)), Some(area(1000, 0, 1560, 720)));
    assert_eq!(division.share_of(id(3)), Some(area(1000, 720, 1560, 720)));

    division.move_boundary(id(3), Side::Top, 1000).unwrap();
    assert_eq!(division.share_of(id(2)), Some(area(1000, 0, 1560, 1000)));
    assert_eq!(division.share_of(id(3)), Some(area(1000, 1000, 1560, 440)));
    assert_eq!(division.share_of(id(1)), Some(area(0, 0, 1000, 1440)));
}

/// **A window dragged to an edge proposes a half and to a corner a quarter,
/// and the proposal is shown before it is committed so a drop can be
/// abandoned.**
#[test]
fn an_edge_proposes_a_half_a_corner_a_quarter_and_either_can_be_abandoned() {
    let mut division = in_halves();
    let before = laid_out(&division);

    let half = proposed(division.propose_drop(Point::at(2559, 700), any(3), None));
    assert_eq!(half.place(), Place::RightHalf);
    assert_eq!(half.area(), area(1280, 0, 1280, 1440));
    assert_eq!(laid_out(&division), before, "a proposal is not a change");

    let quarter = proposed(division.propose_drop(Point::at(2559, 1439), any(3), None));
    assert_eq!(quarter.place(), Place::BottomRightQuarter);
    assert_eq!(quarter.area(), area(1280, 720, 1280, 720));
    assert_eq!(laid_out(&division), before, "a proposal is not a change");

    // Let go away from the edge: abandoned, and nothing moved.
    drop(half);
    assert_eq!(
        division.propose_drop(Point::at(1300, 700), any(3), None),
        Offer::Nothing
    );
    assert_eq!(laid_out(&division), before);

    // Let go in the corner: committed, once.
    division.commit(quarter).unwrap();
    assert_eq!(division.share_of(id(3)), Some(area(1280, 720, 1280, 720)));
    assert_eq!(division.share_of(id(2)), Some(area(1280, 0, 1280, 720)));
}

/// **A keyboard split of what is already open divides the focused share with
/// the next window, bound through `alo-shortcuts`** — so it answers to the
/// person's chord, moves when they rebind it, and is gone when they clear it.
#[test]
fn a_keyboard_split_divides_the_focused_share_with_the_next_window_through_the_shortcuts() {
    let mut shortcuts = Shortcuts::shipped();
    let super_left = Chord::checked(Modifiers::just(Modifier::Super), Key::Left).unwrap();
    let super_right = Chord::checked(Modifiers::just(Modifier::Super), Key::Right).unwrap();
    assert_eq!(side_bound_to(&shortcuts, super_left), Some(Side::Left));
    assert_eq!(side_bound_to(&shortcuts, super_right), Some(Side::Right));

    // Pressed with mail focused and notes next: the display in two.
    let mut division = Division::of(display());
    let side = side_bound_to(&shortcuts, super_left).unwrap();
    division
        .divide_with_next(any(1), Some(any(2)), side)
        .unwrap();
    assert_eq!(division.share_of(id(1)), Some(area(0, 0, 1280, 1440)));
    assert_eq!(division.share_of(id(2)), Some(area(1280, 0, 1280, 1440)));

    // Pressed again with notes focused and a third window next: notes' half
    // is divided, not the display.
    let side = side_bound_to(&shortcuts, super_right).unwrap();
    division
        .divide_with_next(any(2), Some(any(3)), side)
        .unwrap();
    assert_eq!(division.share_of(id(1)), Some(area(0, 0, 1280, 1440)));
    assert_eq!(division.share_of(id(3)), Some(area(1280, 0, 640, 1440)));
    assert_eq!(division.share_of(id(2)), Some(area(1920, 0, 640, 1440)));

    // Rebound: the split follows the person's chord, and the old one no
    // longer divides anything.
    let ctrl_alt_h =
        Chord::checked(Modifiers::just(Modifier::Ctrl).and(Modifier::Alt), Key::H).unwrap();
    shortcuts.unbind(Action::SnapLeft);
    assert_eq!(side_bound_to(&shortcuts, super_left), None);
    shortcuts.bind(Action::SnapLeft, ctrl_alt_h).unwrap();
    assert_eq!(side_bound_to(&shortcuts, ctrl_alt_h), Some(Side::Left));

    // A chord that does something else is not a split.
    let super_a = Chord::checked(Modifiers::just(Modifier::Super), Key::A).unwrap();
    assert_eq!(side_bound_to(&shortcuts, super_a), None);

    // And with nothing else open, a split says so rather than dividing.
    let mut alone = Division::of(display());
    assert_eq!(
        alone.divide_with_next(any(1), None, Side::Left),
        Err(Refused::NothingToShareWith)
    );
    assert!(alone.is_empty());
}

/// **A window closed inside a division gives its share to its neighbour
/// rather than leaving a hole.**
#[test]
fn a_window_closed_inside_a_division_gives_its_share_to_its_neighbour() {
    let mut division = in_halves();
    division.divide(id(2), any(3), Side::Bottom).unwrap();
    division.move_boundary(id(1), Side::Right, 1000).unwrap();

    division.close(id(2)).unwrap();
    assert_eq!(
        laid_out(&division),
        vec![(1, area(0, 0, 1000, 1440)), (3, area(1000, 0, 1560, 1440))]
    );

    division.close(id(1)).unwrap();
    assert_eq!(laid_out(&division), vec![(3, display())]);

    assert_eq!(division.close(id(1)), Err(Refused::NotDivided(id(1))));
}

/// **A window with a minimum size larger than its share is not squeezed below
/// it — the division refuses and says why.**
#[test]
fn a_window_larger_than_its_share_is_not_squeezed_and_the_refusal_says_why() {
    let strings = Strings::of(dividing_words().unwrap());
    let editor = Window::at_least(id(1), Size::of(1100, 900));
    let mut division = Division::of(display());
    division
        .divide_with_next(editor, Some(any(2)), Side::Left)
        .unwrap();
    let before = laid_out(&division);

    // The boundary dragged into the editor's minimum.
    let refused = division.move_boundary(id(2), Side::Left, 1099).unwrap_err();
    assert_eq!(refused, Refused::TooNarrow(id(1)));
    assert_eq!(refused.window(), Some(id(1)));
    assert_eq!(
        refused.said(&strings).text(),
        "this window cannot be made that narrow, so the screen has been left as it was"
    );
    assert_eq!(laid_out(&division), before);

    // Its half split into quarters: 720 of the 900 it needs.
    let refused = division.divide(id(1), any(3), Side::Bottom).unwrap_err();
    assert_eq!(refused, Refused::TooShort(id(1)));
    assert_eq!(
        refused.said(&strings).text(),
        "this window cannot be made that short, so the screen has been left as it was"
    );
    assert_eq!(laid_out(&division), before);

    // Dropped into a corner quarter it cannot fit.
    let offer = division.propose_drop(
        Point::at(2559, 0),
        Window::at_least(id(4), Size::of(1, 800)),
        None,
    );
    assert_eq!(offer, Offer::Refused(Refused::TooShort(id(4))));
    assert_eq!(laid_out(&division), before);
}

/// **Geometry is in logical units the display's scale turns into pixels, so a
/// division means the same thing on a scaled screen.** The same division on a
/// display at 100 % and at 150 % is the same shares, and at 150 % its halves
/// still meet at one pixel.
#[test]
fn a_division_means_the_same_thing_on_a_scaled_screen() {
    let division = in_halves();
    let left = division.share_of(id(1)).unwrap();
    let right = division.share_of(id(2)).unwrap();
    assert_eq!(Place::of(left, display()), Place::LeftHalf);

    let at_150 = Scale::in_120ths(180).unwrap();
    let (l, r) = (at_150.to_pixels(left), at_150.to_pixels(right));
    let whole = at_150.to_pixels(display());
    assert_eq!((whole.width, whole.height), (3840, 2160));
    assert_eq!((l.x, l.width), (0, 1920));
    assert_eq!(l.x + l.width, r.x);
    assert_eq!(l.width + r.width, whole.width);

    let at_100 = Scale::WHOLE.to_pixels(left);
    assert_eq!((at_100.width, at_100.height), (1280, 1440));
}
