//! Which mechanism each of the edge's controls reaches, and that none is lost.
//!
//! **The mapping is the part that can be wrong**, and it is the part a machine
//! with a compositor and two real applications would be needed to check if it
//! lived inside the act. Close reaching the put-aside road would compile, run,
//! and put a window away when a person asked to shut it — with every other test
//! in this crate passing.
//!
//! What is **not** here, said rather than implied: a press on a real window's
//! real control, which needs a mapped surface, which needs a client. That is
//! `examples/a_real_application.rs`'s road and the walk's.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — the words \
              `crate::a_key_reaches_settings_tests` uses for the same server fixture"
)]

use std::os::unix::fs::PermissionsExt;

use smithay::backend::input::ButtonState;

use super::*;
use crate::Server;
use crate::window_edge::OnTheEdge;
use smithay::input::keyboard::XkbConfig;

/// A server on a socket of this test's own, with a keyboard.
///
/// **Its own private runtime directory**, which is not politeness: three
/// unrelated tests in this repository have failed a gate for assuming they were
/// alone on the machine, and a fixed socket name in a shared directory is how
/// two of them did it. The directory is a `tempdir`, so the name inside it
/// cannot collide with anything.
fn a_server(socket: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind_keyboard(
        runtime.path(),
        socket,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    (runtime, server)
}

/// **Every control reaches the mechanism that already does its job**, named one
/// by one rather than asserted as a shape.
///
/// A table rather than a loop over the enum, because the thing being checked is
/// precisely the correspondence between a control and a road, and a loop would
/// have to compute the expected answer — from the function under test or from a
/// copy of it.
#[test]
fn each_control_reaches_the_mechanism_that_already_does_its_job() {
    assert_eq!(
        the_road_an_edge_press_takes(OnTheEdge::Minimise),
        TheRoad::PutItAside,
        "Minimise must join the queue a person's other ways of putting a window aside use"
    );
    assert_eq!(
        the_road_an_edge_press_takes(OnTheEdge::Maximise),
        TheRoad::MaximiseOrRestore
    );
    assert_eq!(
        the_road_an_edge_press_takes(OnTheEdge::Close),
        TheRoad::AskItToClose,
        "Close must be the application's own close request, so its unsaved-work handling runs"
    );
    assert_eq!(
        the_road_an_edge_press_takes(OnTheEdge::Menu),
        TheRoad::NothingHasOneYet,
        "nothing in this crate opens a window menu, and claiming a road for it would be a \
         control that silently does nothing"
    );
}

/// **No two controls share a road**, which is what stops a mapping being wrong
/// in the one direction the test above cannot see.
///
/// Every assertion above would still pass if `Close` *and* `Minimise` both
/// answered `PutItAside` — no, they would not, but a future edit that made two
/// of them agree would have to change a line here as well as a line there. Held
/// as a set so that it does.
#[test]
fn no_two_controls_reach_the_same_mechanism() {
    let roads: Vec<TheRoad> = [
        OnTheEdge::Minimise,
        OnTheEdge::Maximise,
        OnTheEdge::Close,
        OnTheEdge::Menu,
    ]
    .into_iter()
    .map(the_road_an_edge_press_takes)
    .collect();

    for (which, road) in roads.iter().enumerate() {
        let same = roads
            .iter()
            .enumerate()
            .filter(|(other, it)| *other != which && *it == road)
            .count();
        assert_eq!(
            same, 0,
            "{road:?} is reached by more than one control, so pressing one does what another \
             asked for. Roads: {roads:?}"
        );
    }
}

/// **A press with no pointer is nobody's**, and not the window whose edge
/// happens to contain the origin.
///
/// The same rule `crate::the_pointer_in_pixels` carries and the reason it gives:
/// a seat that has never had a pointing device has no position, and answering
/// `(0, 0)` would press whatever is in the corner of the screen. Asserted here
/// because this is the one of the two readers that *acts* on the answer.
#[test]
fn a_press_with_no_pointer_is_not_the_edges() {
    let (_runtime, mut server) = a_server("a-press-with-no-pointer");
    assert!(
        server.surfaces.pointer.is_none(),
        "the premise: a freshly bound server has no pointer, so the branch under test is the \
         one that runs"
    );
    assert!(
        !server.a_press_on_an_edge(0x110, ButtonState::Pressed),
        "a press with no pointer was taken as some window's edge"
    );
    assert!(
        server.asked_on_an_edge.is_empty(),
        "and it recorded an ask for a window nobody pressed"
    );
}

/// **A release is never the edge's.**
///
/// A drag's release is consumed by `Surfaces::window_move_button`, which
/// `pointer_button` asks first, and a control acts on its press. An edge that
/// claimed releases would swallow the release of a press a client had been told
/// about, and *a client must never see a release for a press it did not see* has
/// a mirror: a client that saw the press must see the release.
#[test]
fn a_release_is_never_the_edges() {
    let (_runtime, mut server) = a_server("a-release-is-never-the-edges");
    assert!(!server.a_press_on_an_edge(0x110, ButtonState::Released));
}

/// **Meeting the asks empties the list**, whatever became of them.
///
/// A press that failed is not retried: a person pressed Close once, a refusal is
/// an answer, and an ask that stayed would be a close request sent sixty times a
/// second. Checked with an empty list, which is the only case reachable without
/// a client — and it is the case that would break if `meet_…` ever stopped
/// taking the list.
#[test]
fn meeting_nothing_asks_for_nothing_and_leaves_nothing_behind() {
    let (_runtime, mut server) = a_server("meeting-nothing");
    assert!(server.meet_what_was_pressed_on_an_edge().is_empty());
    assert!(server.asked_on_an_edge.is_empty());
    assert!(
        server.asked_to_put_aside.is_empty(),
        "meeting no asks put a window aside"
    );
}
