//! That the server's half of a frame is filled in one place, and all of it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a file this crate ships is the failure being reported"
)]

/// Every field this function is responsible for.
///
/// Named here rather than derived, because the question is whether anything
/// sets one of them *somewhere else* — which a list of names can ask and the
/// compiler cannot.
const THE_SERVERS_HALF: [&str; 4] = [
    "windows",
    "filling_the_screen",
    "display_scale",
    "dock_holds",
];

/// `direct_desktop.rs`, as text.
fn the_direct_desktop() -> String {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/direct_desktop.rs");
    std::fs::read_to_string(&at).expect("direct_desktop.rs is beside this file")
}

/// **Both sites ask, and neither sets a field itself.**
///
/// `direct_desktop` builds a frame twice — for the display the loop holds and
/// for every other display — and the fields drifting apart between them is the
/// fault this function exists to make impossible. A one-line patch at each site
/// would have been correct code in the wrong shape.
///
/// This is a text guard because the property is *where the assignment is*, and
/// there is no value to assert: two sites each setting four fields correctly
/// would pass every behavioural test and still be the shape that drifts.
#[test]
fn the_servers_half_of_a_frame_is_set_in_one_place() {
    let text = the_direct_desktop();

    // The count first: a file that stopped calling it at all would otherwise
    // satisfy every assertion below by having no assignments either.
    let asks = text.matches("what_only_the_server_knows(").count();
    assert_eq!(
        asks, 2,
        "direct_desktop builds a frame for the held display and for every other \
         one, so it asks twice; it asked {asks} times"
    );

    for field in THE_SERVERS_HALF {
        let set_here = format!("frame.{field} =");
        assert!(
            !text.contains(&set_here),
            "direct_desktop sets `{field}` itself. That is the server's half, and \
             two places setting it is how the two displays come to disagree - put \
             it in `what_only_the_server_knows` with the other three"
        );
    }
}

/// **The Dock's count is one of the fields**, which is the whole reason this
/// change exists.
///
/// `dock_holds` was absent from `direct_desktop` entirely while its three
/// neighbours were each filled from the server, so the Dock drew a bar sized
/// for nothing however many applications were open. A test naming it is what
/// stops it being dropped again by somebody tidying.
#[test]
fn the_dock_is_told_how_many_it_holds() {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/what_the_server_tells_a_frame.rs");
    let text = std::fs::read_to_string(&at).expect("this file is where it says");
    let body = text
        .split("pub(crate) fn what_only_the_server_knows")
        .nth(1)
        .expect("the function is in its own file");
    for field in THE_SERVERS_HALF {
        assert!(
            body.contains(&format!("frame.{field} =")),
            "`{field}` is the server's half and this function does not set it"
        );
    }
    assert!(
        body.contains("how_many_the_dock_holds"),
        "the count comes from the server's own answer, not from arithmetic here"
    );
}
