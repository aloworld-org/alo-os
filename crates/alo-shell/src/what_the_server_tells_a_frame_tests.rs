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
const THE_SERVERS_HALF: [&str; 5] = [
    "windows",
    "filling_the_screen",
    "display_scale",
    "on_the_dock",
    "the_overflow_is_open",
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

/// **What the Dock holds is one of the fields**, which is the whole reason this
/// change exists.
///
/// It was absent from `direct_desktop` entirely while its three neighbours were
/// each filled from the server, so the Dock drew a bar sized for nothing
/// however many applications were open. A test naming it is what stops it being
/// dropped again by somebody tidying.
#[test]
fn the_dock_is_told_what_it_holds() {
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
}

/// **And the list is the server's own answer, asked where it is now asked.**
///
/// This assertion read `body.contains("how_many_the_dock_holds")` until
/// 2026-10-10, when the field became the entries rather than a count: the list
/// has to outlive the frame that borrows it, so it is built by the caller and
/// passed in, and the function above no longer names the server for it.
///
/// **So the claim moves rather than weakens.** What it always meant is *this
/// comes from the server and not from arithmetic somebody did here*, and that
/// is now a fact about `direct_desktop` — which is where it is asserted.
/// Dropping it instead would have left the one field with no guard, which is
/// the state that produced this file.
#[test]
fn the_list_is_built_from_the_server_by_the_caller() {
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/direct_desktop.rs");
    let text = std::fs::read_to_string(&at).expect("this file is where it says");
    let built = text.matches("server.what_the_dock_holds(").count();
    assert!(
        built >= 1,
        "`direct_desktop` builds the Dock's list nowhere, so whatever reaches the frame is not \
         the server's answer"
    );
    assert!(
        !text.contains("frame.on_the_dock ="),
        "`direct_desktop` sets the field itself. That is the server's half and
         `what_only_the_server_knows` is the one place it is set"
    );
}
