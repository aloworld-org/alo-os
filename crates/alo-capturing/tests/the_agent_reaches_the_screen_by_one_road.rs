//! **An agent reaches the screen through an approved verb, or not at all.**
//!
//! Task 6 of `docs/autonomy/v0-5-capture-and-the-room-plan.md`, and the reason
//! that plan exists. `docs/features.md` promises *context on invocation* and
//! never harvesting; a screenshot is the most harvest-shaped thing a machine
//! has, because one call hands over everything a person was looking at.
//!
//! Two things are tested here, and the second is tested **directly** rather
//! than by trusting that the agent's capture goes down the same path as an
//! application's:
//!
//! 1. no crate the daemon ships reaches a capture except through the one road;
//! 2. **a capture by the agent lights the indicator, as the agent** — because a
//!    capture the indicator did not show is the failure this whole workstream
//!    is built to prevent.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_appearance::Token;
use alo_capturing::{ForTheAgent, Picture};
use alo_in_use::{By, Line, Use, UseId, Used};

/// The crates the daemon ships, which an agent's turn runs inside.
const WHAT_THE_DAEMON_SHIPS: [&str; 4] =
    ["alo-agentd", "alo-turn", "alo-context", "alo-capability"];

/// Ways to reach a picture of the screen.
const A_ROAD_TO_THE_SCREEN: [&str; 6] = [
    "Screenshot",
    "screen_cast",
    "ScreenCast",
    "alo_capturing",
    "grab_the_screen",
    "take_a_picture",
];

/// **No crate the daemon ships captures the screen at all.**
///
/// Not by a portal, not by a helper, not by this crate: the agent's road is a
/// verb, and the verb does not exist yet (see below), so today the honest state
/// is that nothing there can capture anything.
#[test]
fn nothing_the_daemon_ships_reaches_the_screen_by_another_road() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crates folder");
    let mut roads: Vec<String> = Vec::new();
    for named in WHAT_THE_DAEMON_SHIPS {
        for file in every_file(&crates.join(named).join("src")) {
            let text = std::fs::read_to_string(&file).unwrap_or_default();
            for road in A_ROAD_TO_THE_SCREEN {
                for line in text.lines() {
                    let code = line.trim_start();
                    if code.starts_with("//") {
                        continue;
                    }
                    if code.contains(road) {
                        roads.push(format!("{}: {}", file.display(), line.trim()));
                    }
                }
            }
        }
    }
    assert!(
        roads.is_empty(),
        "a crate the daemon ships can reach the screen:\n{roads:#?}\n\nAn agent sees the screen \
         through a verb whose proposal says `a picture of your screen` and which a person \
         approved, or it does not see the screen. A road added here would be the harvest this \
         plan exists to prevent, arriving as a convenience."
    );
}

/// **A capture by the agent lights the indicator, as the agent.**
///
/// Tested on the value the capture itself produces, not on the general shape of
/// `alo-in-use`: the question is whether *this* road lights it, and the answer
/// has to come from this road.
#[test]
fn a_capture_by_the_agent_shows_in_the_indicator_in_the_agents_colour() {
    let capture = a_capture_the_agent_holds();
    let shown = capture
        .in_use(UseId::recorded(7))
        .expect("the agent is somebody a grant can name");
    let line = Line::of(&shown);

    assert_eq!(
        line.what(),
        Used::Screen,
        "the screen is in use and the line says otherwise"
    );
    assert!(
        line.by().is_the_agents(),
        "the agent took a picture of the screen and the indicator does not say it was the agent"
    );
    assert_eq!(
        line.colour(),
        Token::Terracotta,
        "ADR 0010 gives the agent terracotta, and this line is not it"
    );
    assert!(
        line.the_agents_dot(),
        "ADR 0010's mark is not drawn beside it"
    );
}

/// **And an application's capture is not terracotta**, so the colour still
/// means one thing.
#[test]
fn an_application_using_the_screen_is_not_drawn_as_the_agent() {
    let theirs = Use::of(
        UseId::recorded(8),
        Used::Screen,
        By::an_application(
            alo_applications::Application::identified("org.alo.Writer").expect("an application"),
        ),
    );
    let line = Line::of(&theirs);
    assert!(!line.by().is_the_agents());
    assert_ne!(
        line.colour(),
        Token::Terracotta,
        "an application was drawn in the agent's colour, so the colour no longer means the agent"
    );
    assert!(!line.the_agents_dot());
}

/// **What the record says is that it happened, and nothing of what was seen.**
#[test]
fn the_record_says_a_picture_was_taken_and_not_what_was_in_it() {
    let capture = a_capture_the_agent_holds();
    let said = capture.as_the_record_says_it();
    assert!(said.contains("a picture of the screen was taken"));
    for what_was_on_the_screen in ["window", "title", "text", "pixels", "bytes", "size"] {
        assert!(
            !said.contains(what_was_on_the_screen),
            "the record describes what was on the screen: {said}"
        );
    }
}

/// A capture, built from the parts because **no verb exists that could make
/// one** — which is itself this task's finding, recorded in the report.
fn a_capture_the_agent_holds() -> ForTheAgent {
    alo_capturing::for_the_agent::for_a_test(
        "@the-agent",
        alo_capturing::Screen::measuring(1920, 1080).expect("a screen"),
        Picture::of(vec![4, 5, 6]).expect("a picture"),
    )
}

/// Every `.rs` under a folder.
fn every_file(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(every_file(&path));
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            found.push(path);
        }
    }
    found
}
