//! What the journal is told about where bands are, frame after frame, and
//! that the walk reads it back as it was said.

use alo_acting::said_of_a_band::{
    OnTheScreen, TheKind, read_a_band_is_drawn, read_a_band_is_no_longer_drawn,
    read_a_bands_controls_are_drawn,
};
use alo_put_aside::AlosOwnWindow;

use super::{SaidOfOneBand, what_changed};
use crate::the_bands_over_alos_windows::AlosBand;
use crate::which_band::ABand;
use crate::window_edge::OnTheEdge;

/// A rectangle on the screen.
const fn at(x: i32, y: i32, w: i32, h: i32) -> OnTheScreen {
    OnTheScreen { x, y, w, h }
}

/// The terminal's band at `x`, with these controls.
fn the_terminal(x: i32, controls: Vec<(OnTheEdge, OnTheScreen)>) -> SaidOfOneBand {
    SaidOfOneBand {
        band: ABand::Alos(AlosBand::Its(AlosOwnWindow::TheTerminal)),
        kind: TheKind::Terminal,
        id: "terminal".to_owned(),
        at: at(x, 46, 640, 54),
        controls,
    }
}

/// Its three controls, revealed.
fn its_three(x: i32) -> Vec<(OnTheEdge, OnTheScreen)> {
    vec![
        (OnTheEdge::WindowActions, at(x + 482, 51, 44, 44)),
        (OnTheEdge::Minimise, at(x + 530, 51, 44, 44)),
        (OnTheEdge::Close, at(x + 578, 51, 44, 44)),
    ]
}

/// **A band is said once when it appears, not again while it stays, again
/// when it moves, and once when it goes**, each line read back by the walk.
#[test]
fn a_band_is_said_when_it_appears_moves_and_goes() {
    let (lines, now) = what_changed(&[], vec![the_terminal(100, Vec::new())]);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(
        read_a_band_is_drawn(&lines[0]),
        Some((
            TheKind::Terminal,
            "terminal".to_owned(),
            at(100, 46, 640, 54)
        ))
    );

    let (lines, now) = what_changed(&now, vec![the_terminal(100, Vec::new())]);
    assert!(lines.is_empty(), "a frame with nothing new said {lines:?}");

    let (lines, now) = what_changed(&now, vec![the_terminal(140, Vec::new())]);
    assert_eq!(
        lines
            .iter()
            .filter_map(|line| read_a_band_is_drawn(line))
            .map(|(_, _, at)| at)
            .collect::<Vec<_>>(),
        vec![at(140, 46, 640, 54)],
        "a band that moved was not said where it went"
    );

    let (lines, now) = what_changed(&now, Vec::new());
    assert_eq!(
        lines
            .iter()
            .filter_map(|line| read_a_band_is_no_longer_drawn(line))
            .collect::<Vec<_>>(),
        vec![(TheKind::Terminal, "terminal".to_owned())]
    );
    assert!(now.is_empty());
}

/// **Its controls are said once when they appear, with each one's English
/// word and where it is, and not while the band is at rest.**
#[test]
fn a_bands_controls_are_said_when_they_appear() {
    let (_, now) = what_changed(&[], vec![the_terminal(100, Vec::new())]);
    let (lines, now) = what_changed(&now, vec![the_terminal(100, its_three(100))]);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let (kind, id, controls) =
        read_a_bands_controls_are_drawn(&lines[0]).expect("the walk reads the line back");
    assert_eq!((kind, id.as_str()), (TheKind::Terminal, "terminal"));
    assert_eq!(
        controls
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["Window actions", "Minimise", "Close"]
    );
    assert_eq!(controls[2].1, at(678, 51, 44, 44));

    let (lines, _) = what_changed(&now, vec![the_terminal(100, its_three(100))]);
    assert!(lines.is_empty(), "controls still shown were said again");
}
