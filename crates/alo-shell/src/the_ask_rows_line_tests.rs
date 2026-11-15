//! The request cut inside its own quotation marks, in whatever language puts
//! them where, and the whole line cut where that cannot be done.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on fonts that do not load is the failure being reported"
)]

use super::*;

fn fonts() -> FontSystem {
    crate::WindowControlLabels::new()
        .map(|labels| labels.fonts)
        .expect("the shell's own fonts load")
}

fn name() -> (Metrics, Attrs<'static>) {
    (
        Metrics::new(14.0, 22.0),
        crate::painted_text::alos_own_text().weight(cosmic_text::Weight::SEMIBOLD),
    )
}

const LONG: &str = "Open the file hello.txt in my home folder in VSCodium, type the line: \
                    Written by alo in VSCodium and then save it.";

#[test]
fn a_request_that_fits_is_left_whole() {
    let (metrics, attrs) = name();
    let asking = "Ask alo \u{201c}files\u{201d}";
    let shown = the_request_in_its_room(&mut fonts(), (asking, "files"), 400, metrics, &attrs);
    assert_eq!(shown, asking);
}

#[test]
fn a_long_request_is_cut_inside_its_quotation_marks_and_fits() {
    let (metrics, attrs) = name();
    let mut fonts = fonts();
    let asking = format!("Ask alo \u{201c}{LONG}\u{201d}");
    let shown = the_request_in_its_room(&mut fonts, (&asking, LONG), 300, metrics, &attrs);
    assert!(shown.starts_with("Ask alo \u{201c}Open the"), "{shown}");
    assert!(shown.ends_with("\u{2026}\u{201d}"), "{shown}");
    assert!(
        crate::painted_text::how_wide_as(&mut fonts, &shown, metrics, &attrs) <= 300,
        "{shown}"
    );
}

/// **A language that puts the request first** keeps its words after it.
#[test]
fn the_words_after_the_request_are_kept_too() {
    let (metrics, attrs) = name();
    let asking = format!("\u{ab} {LONG} \u{bb} \u{e0} alo");
    let shown = the_request_in_its_room(&mut fonts(), (&asking, LONG), 300, metrics, &attrs);
    assert!(shown.starts_with("\u{ab} Open"), "{shown}");
    assert!(shown.ends_with("\u{2026} \u{bb} \u{e0} alo"), "{shown}");
}

/// A translation that does not say the request is cut at its end.
#[test]
fn a_line_without_the_request_ends_in_an_ellipsis() {
    let (metrics, attrs) = name();
    let mut fonts = fonts();
    let asking = format!("Ask alo about something else entirely, {LONG}");
    let shown = the_request_in_its_room(&mut fonts, (&asking, "absent"), 200, metrics, &attrs);
    assert!(shown.ends_with('\u{2026}'), "{shown}");
    assert!(crate::painted_text::how_wide_as(&mut fonts, &shown, metrics, &attrs) <= 200);
}

#[test]
fn no_room_is_nothing_drawn() {
    let (metrics, attrs) = name();
    assert_eq!(
        the_request_in_its_room(&mut fonts(), ("Ask alo", "alo"), 0, metrics, &attrs),
        ""
    );
}

#[test]
fn what_follows_the_request_keeps_its_room_until_half_the_row() {
    assert_eq!(the_requests_room(400, 80, 6), 314);
    assert_eq!(the_requests_room(100, 80, 6), 50);
}
