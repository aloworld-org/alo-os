//! That a conversation opens at its newest words, scrolls back as far as its
//! oldest and no further, and comes back to the newest when something is said.

use super::TheScroll;

/// **A conversation opens at its newest words.**
#[test]
fn a_conversation_opens_at_the_newest() {
    let scroll = TheScroll::at_the_newest();
    assert_eq!(scroll.the_view_starts(600, 3), 600);
}

/// **Scrolling back moves the view up the content**, and forward brings it
/// down again.
#[test]
fn scrolling_back_and_forward_moves_the_view() {
    let mut scroll = TheScroll::at_the_newest();
    scroll.the_frame_measured(600, 300);

    assert!(scroll.scrolled(150, 3));
    assert_eq!(scroll.the_view_starts(600, 3), 450);

    assert!(scroll.scrolled(-100, 3));
    assert_eq!(scroll.the_view_starts(600, 3), 550);
}

/// **It stops at the oldest words, and says it did not move.** The refusal
/// path: a scroll past the top neither runs on into nothing nor claims to
/// have moved.
#[test]
fn it_stops_at_the_oldest_and_says_so() {
    let mut scroll = TheScroll::at_the_newest();
    scroll.the_frame_measured(200, 300);

    assert!(scroll.scrolled(500, 3));
    assert_eq!(scroll.the_view_starts(200, 3), 0, "it went past the top");
    assert!(!scroll.scrolled(50, 3), "a scroll at the top said it moved");
    // **And one notch forward moves at once**, rather than first paying back
    // the distance scrolled past the top.
    assert!(scroll.scrolled(-50, 3));
    assert_eq!(scroll.the_view_starts(200, 3), 50);
}

/// **At the newest, scrolling forward moves nothing**, and says so.
#[test]
fn at_the_newest_forward_moves_nothing() {
    let mut scroll = TheScroll::at_the_newest();
    scroll.the_frame_measured(200, 300);
    assert!(!scroll.scrolled(-50, 3));
}

/// **Nothing to scroll before a frame has measured anything.** A conversation
/// that fits has nowhere to go.
#[test]
fn nothing_scrolls_before_it_is_measured() {
    let mut scroll = TheScroll::at_the_newest();
    assert!(!scroll.scrolled(100, 3));
    assert_eq!(scroll.the_view_starts(0, 3), 0);
}

/// **Something new being said brings the view back to the newest.**
#[test]
fn something_new_brings_the_view_back() {
    let mut scroll = TheScroll::at_the_newest();
    scroll.the_frame_measured(600, 300);
    assert!(scroll.scrolled(300, 3));
    assert_eq!(scroll.the_view_starts(600, 3), 300);

    assert_eq!(
        scroll.the_view_starts(700, 4),
        700,
        "a new turn left the view behind"
    );
}

/// **A conversation that measured shorter takes the view with it.**
#[test]
fn a_shorter_measurement_pulls_the_view_in() {
    let mut scroll = TheScroll::at_the_newest();
    scroll.the_frame_measured(600, 300);
    assert!(scroll.scrolled(500, 3));
    scroll.the_frame_measured(100, 300);
    assert_eq!(scroll.the_view_starts(100, 3), 0);
}

/// **A page is the view less one line**, so the line being read stays in sight.
#[test]
fn a_page_keeps_one_line_in_sight() {
    let mut scroll = TheScroll::at_the_newest();
    scroll.the_frame_measured(600, 300);
    assert_eq!(
        scroll.a_page(),
        300 - crate::measures::A_LINE_OF_WHAT_WAS_SAID
    );
}
