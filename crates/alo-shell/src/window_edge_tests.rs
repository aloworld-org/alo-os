//! The band, held against the design file's own coordinates.
//!
//! **These assert the numbers in
//! `docs/design/figma-snapshot/window-controls-XbSmsBYix0TrcQ0Qzaup64.xml`, not
//! numbers this file chose.** The Notes window of `11:24` is a 1056-wide
//! specimen, so feeding 1056 in must give back the node positions the designer
//! drew — 898, 946, 994 and the rest. A test that asserted what the code
//! computes would pass whatever the code computed.
//!
//! The specimen is placed so the band lands at the origin: the band sits 54
//! above its window, so a window at `y = 54` has a band at `y = 0` and every
//! figure below reads directly against the capture.

use super::*;

/// A display large enough that nothing is clamped.
fn roomy() -> Size<i32, Logical> {
    Size::from((4000, 3000))
}

/// The Notes window of `11:24`, its band at the origin.
fn specimen() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((0, 54)), Size::from((1056, 466)))
}

/// The band of a shell-drawn window, its controls revealed.
fn shown(window: Rectangle<i32, Logical>) -> WindowEdge {
    edge_of(window, Decorations::TheShellDraws, true, roomy())
}

/// **The band is 54, and all of it is drawn.** `11:31` is 1056 × 54 in every
/// frame, at rest and revealed.
#[test]
fn the_band_is_fifty_four_and_all_of_it_is_drawn() {
    for revealed in [false, true] {
        let edge = edge_of(specimen(), Decorations::TheShellDraws, revealed, roomy());
        let band = Rectangle::new(Point::from((0, 0)), Size::from((1056, 54)));
        assert_eq!(edge.region, band);
        assert_eq!(edge.strip, band, "revealed: {revealed}");
    }
}

/// **The three controls are where `11:41` draws them**: 140 wide at 898, y 5,
/// each 44 and 4 apart — Window actions at 898, Minimise at 946, Close at 994.
#[test]
fn the_three_controls_are_where_the_design_draws_them() {
    let edge = shown(specimen());
    let at: Vec<(OnTheEdge, i32, i32)> = edge
        .controls
        .iter()
        .map(|it| (it.does, it.target.loc.x, it.target.loc.y))
        .collect();
    assert_eq!(
        at,
        vec![
            (OnTheEdge::WindowActions, 898, 5),
            (OnTheEdge::Minimise, 946, 5),
            (OnTheEdge::Close, 994, 5),
        ]
    );
    for control in &edge.controls {
        assert_eq!(control.target.size, Size::from((44, 44)));
    }
    let last = edge.controls.last().map(|it| it.target);
    assert_eq!(
        last.map(|it| 1056 - (it.loc.x + it.size.w)),
        Some(18),
        "the last control ends 18 in from the trailing edge"
    );
}

/// **Exactly three, and no others**: no Maximise and no full screen on the
/// band, which the specification says by name.
#[test]
fn there_are_exactly_three_and_none_of_them_is_maximise() {
    let edge = shown(specimen());
    assert_eq!(edge.controls.len(), 3);
}

/// **At rest nothing is drawn to press, so nothing answers** — and the whole
/// band drags, because a control nobody can see is never pressed.
#[test]
fn at_rest_there_are_no_controls_and_the_whole_band_drags() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, false, roomy());
    assert!(edge.controls.is_empty());
    assert_eq!(edge.drag, edge.region);
}

/// **Revealed, the band drags up to one gap before the first control**, so a
/// press on a control never starts a drag.
#[test]
fn revealed_the_drag_stops_before_the_controls() {
    let edge = shown(specimen());
    assert_eq!(edge.drag.loc, Point::from((0, 0)));
    assert_eq!(edge.drag.size, Size::from((894, 54)));
    for control in &edge.controls {
        assert!(
            edge.drag.intersection(control.target).is_none(),
            "{:?} overlaps the drag",
            control.does
        );
    }
}

/// **The title is at (22, 18)**, `11:32`, and its room ends one gap before
/// where Window actions is.
#[test]
fn the_title_starts_where_the_design_puts_it() {
    let edge = shown(specimen());
    assert_eq!(edge.title, Some(Point::from((22, 18))));
    assert_eq!(edge.the_room_a_title_has(), 898 - 4 - 22);
}

/// **Revealing the controls never reflows the title**: the room is the same
/// concealed and revealed, which the specification asks for in as many words.
#[test]
fn revealing_the_controls_does_not_move_or_narrow_the_title() {
    let concealed = edge_of(specimen(), Decorations::TheShellDraws, false, roomy());
    let revealed = shown(specimen());
    assert_eq!(concealed.title, revealed.title);
    assert_eq!(concealed.title_room, revealed.title_room);
}

/// **An application with its own header gets the controls it lacks**, laid
/// out at the trailing end like the shell's, and no title of alo's over its
/// own.
#[test]
fn an_application_with_its_own_header_gets_the_controls_it_lacks() {
    let close_only = Decorations::TheApplicationDraws(crate::window_edge::ItsOwnButtons {
        minimise: false,
        close: true,
    });
    let edge = edge_of(specimen(), close_only, true, roomy());
    let does: Vec<OnTheEdge> = edge.controls.iter().map(|it| it.does).collect();
    assert_eq!(does, vec![OnTheEdge::WindowActions, OnTheEdge::Minimise]);
    assert_eq!(edge.title, None);
    assert_eq!(edge.the_room_a_title_has(), 0);
    // The last one ends where the shell's Close ends: the trailing inset.
    let last = edge.controls.last().expect("two controls");
    assert_eq!(
        last.target.loc.x + last.target.size.w,
        edge.region.loc.x + edge.region.size.w - THE_TRAILING_INSET
    );
}

/// **Revealing an application's added controls never moves its content**:
/// the band is above the window in every mode, concealed or revealed.
#[test]
fn revealing_an_applications_added_controls_never_moves_its_window() {
    for its_own in [
        crate::window_edge::ItsOwnButtons::NOT_KNOWN,
        crate::window_edge::ItsOwnButtons {
            minimise: false,
            close: true,
        },
        crate::window_edge::ItsOwnButtons {
            minimise: false,
            close: false,
        },
    ] {
        for revealed in [false, true] {
            let edge = edge_of(
                specimen(),
                Decorations::TheApplicationDraws(its_own),
                revealed,
                roomy(),
            );
            assert_eq!(
                edge.region.loc.y + edge.region.size.h,
                specimen().loc.y,
                "the band does not end where the window begins"
            );
            assert!(!edge.region.overlaps(specimen()), "the band covers the window");
        }
    }
}

/// **A task window carries Minimise and Close; alo's own windows Close alone.**
#[test]
fn a_task_window_carries_minimise_and_alos_own_windows_close_alone() {
    let does = |decorations| -> Vec<OnTheEdge> {
        edge_of(specimen(), decorations, true, roomy())
            .controls
            .iter()
            .map(|it| it.does)
            .collect()
    };
    assert_eq!(
        does(Decorations::ATaskWindow),
        vec![OnTheEdge::Minimise, OnTheEdge::Close]
    );
    assert_eq!(does(Decorations::AlosOwnWindow), vec![OnTheEdge::Close]);
    assert_eq!(
        edge_of(specimen(), Decorations::ATaskWindow, true, roomy()).title,
        None
    );
    assert!(
        edge_of(specimen(), Decorations::AlosOwnWindow, true, roomy())
            .title
            .is_some()
    );
}

/// **The band sits above the window, and the window is not moved.**
#[test]
fn the_band_is_above_the_content_and_takes_nothing_from_it() {
    let window = Rectangle::new(Point::from((300, 400)), Size::from((640, 480)));
    let edge = shown(window);
    assert_eq!(edge.region.loc.y + edge.region.size.h, window.loc.y);
    assert_eq!(edge.region.loc.x, window.loc.x);
    assert_eq!(edge.region.size.w, window.size.w);
}

/// **A window narrower than the three controls gets a band that overhangs it,
/// centred** — never a control cut off and never the content resized.
#[test]
fn a_narrow_window_gets_a_band_that_overhangs_it_centred() {
    let window = Rectangle::new(Point::from((500, 300)), Size::from((100, 200)));
    let edge = shown(window);
    assert_eq!(edge.region.size.w, THE_NARROWEST_EDGE_IS);
    assert_eq!(edge.region.loc.x, 500 - (180 - 100) / 2);
    assert_eq!(edge.controls.len(), 3);
    for control in &edge.controls {
        assert!(
            edge.region.contains_rect(control.target),
            "{:?} is outside the band",
            control.does
        );
    }
}

/// **An overhanging band is clamped onto the display** rather than hanging off
/// its left side.
#[test]
fn an_overhanging_band_at_the_left_of_the_display_is_clamped_onto_it() {
    let window = Rectangle::new(Point::from((0, 300)), Size::from((60, 200)));
    let edge = shown(window);
    assert_eq!(edge.region.loc.x, 0);
}

/// **The narrowest band is derived, not chosen**: the title's inset, three
/// targets, two gaps and the trailing inset.
#[test]
fn the_narrowest_band_holds_the_title_inset_and_three_controls() {
    assert_eq!(THE_NARROWEST_EDGE_IS, 180);
    let window = Rectangle::new(Point::from((0, 54)), Size::from((180, 200)));
    let edge = shown(window);
    assert_eq!(edge.the_room_a_title_has(), 0, "no room left for a title");
    assert_eq!(
        edge.controls.first().map(|it| it.target.loc.x),
        Some(22),
        "the controls take everything after the title's inset"
    );
}

/// **A scale below one-to-one is read as one-to-one**, and a doubled display
/// halves the logical room.
#[test]
fn the_usable_output_is_divided_by_the_scale_once() {
    assert_eq!(
        the_usable_output(Size::from((1920, 1080)), 0),
        Size::from((1920, 1080))
    );
    assert_eq!(
        the_usable_output(Size::from((2880, 1920)), 200),
        Size::from((1440, 960))
    );
}
