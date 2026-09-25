//! What the in-use indicator draws, and what it never draws.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::*;
use alo_appearance::Token;
use alo_capability::Grantee;
use alo_in_use::{By, Use, UseId, Used};

/// Everything this machine can say, with nothing translated.
fn words() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// The ordinary light look, read left to right.
fn light() -> InUseLook {
    InUseLook {
        scheme: Scheme::Light,
        scale: TextScale::ordinary(),
        reading: Direction::LeftToRight,
        contrast: Contrast::AsDesigned,
    }
}

/// One use of `what`, by an application.
fn used(what: Used, number: u32) -> Use {
    Use::of(
        UseId::recorded(number),
        what,
        By::an_application(
            alo_applications::Application::called("com.example.call", "A Video Call").unwrap(),
        ),
    )
}

/// One use of `what`, by the agent.
fn the_agents(what: Used, number: u32) -> Use {
    Use::of(
        UseId::recorded(number),
        what,
        By::the_agent(&Grantee::named("@mail")).unwrap(),
    )
}

/// The indicator drawn for these uses on a 1920×1080 output.
fn drawn(uses: &[Use]) -> InUsePicture {
    let lines: Vec<Line> = uses.iter().map(Use::line).collect();
    let mut labels = WindowControlLabels::new().unwrap();
    picture(
        &lines,
        &words(),
        &Dock::shipped(),
        &mut labels,
        (1920, 1080),
        light(),
    )
    .unwrap()
}

/// **Nothing in use draws nothing at all.**
///
/// Not a faint row, not an empty box: no pixel of this surface exists while
/// nothing on the machine is watching or listening.
#[test]
fn a_machine_watching_nothing_draws_no_indicator() {
    let quiet = drawn(&[]);

    assert!(quiet.is_empty(), "something was drawn for nothing in use");
    assert_eq!(quiet.height, 0);
}

/// **The order is the screen, then the camera, then the microphone** — whatever
/// order the uses arrive in.
///
/// `alo_in_use::Position` decides it and this file cannot: a person who has
/// glanced at the camera twice knows where to look the third time.
#[test]
fn the_order_is_the_crates_and_not_the_order_they_arrived_in() {
    let backwards = drawn(&[
        used(Used::Microphone, 1),
        used(Used::Camera, 2),
        used(Used::Screen, 3),
    ]);
    let forwards = drawn(&[
        used(Used::Screen, 3),
        used(Used::Camera, 2),
        used(Used::Microphone, 1),
    ]);

    let sentences = |picture: &InUsePicture| -> Vec<String> {
        picture
            .rows
            .iter()
            .map(|row| row.sentence.clone())
            .collect()
    };
    assert_eq!(
        sentences(&backwards),
        sentences(&forwards),
        "the rows came out in the order the uses were handed over"
    );
    assert_eq!(backwards.rows.len(), 3);
}

/// **Who is using something never moves it.**
///
/// The microphone is in the same place whether an application or the agent has
/// it. If position depended on who, the glance a person had learned would stop
/// working at exactly the moment it mattered most.
#[test]
fn an_agent_picking_something_up_does_not_move_it() {
    let anybodys = drawn(&[
        used(Used::Screen, 1),
        used(Used::Camera, 2),
        used(Used::Microphone, 3),
    ]);
    let the_agents = drawn(&[
        used(Used::Screen, 1),
        the_agents(Used::Camera, 2),
        used(Used::Microphone, 3),
    ]);

    let places: Vec<i32> = anybodys.rows.iter().map(|row| row.area.loc.y).collect();
    let moved: Vec<i32> = the_agents.rows.iter().map(|row| row.area.loc.y).collect();
    assert_eq!(
        places, moved,
        "a line moved because the agent was the one using it"
    );
}

/// **Every row says exactly what `alo-in-use` worded**, never a sentence
/// assembled here.
#[test]
fn the_words_are_the_crates_own() {
    let strings = words();
    let uses = [used(Used::Camera, 1), the_agents(Used::Microphone, 2)];
    let expected: Vec<String> = uses
        .iter()
        .map(|one| one.line().said(&strings).text().to_owned())
        .collect();

    let picture = drawn(&uses);
    let drawn_sentences: Vec<String> = picture
        .rows
        .iter()
        .map(|row| row.sentence.clone())
        .collect();

    for sentence in &expected {
        assert!(
            drawn_sentences.contains(sentence),
            "the indicator drew {drawn_sentences:?} rather than {sentence}"
        );
    }
}

/// **The agent's line is terracotta and carries the dot; nobody else's does.**
///
/// ADR 0010: the two arrive together or not at all, and the decision is
/// `alo-in-use`'s rather than this file's.
#[test]
fn terracotta_and_the_dot_arrive_together_and_only_for_the_agent() {
    let terracotta = Contrast::AsDesigned.accent(Scheme::Light, Token::Terracotta.colour());

    let anybodys = drawn(&[used(Used::Camera, 1)]);
    let the_agents_own = drawn(&[the_agents(Used::Camera, 1)]);

    let has_terracotta = |picture: &InUsePicture| {
        picture
            .solids
            .iter()
            .any(|solid| solid.colour == terracotta)
    };
    assert!(
        !has_terracotta(&anybodys),
        "an application's camera was drawn in the colour that means the agent"
    );
    assert!(
        has_terracotta(&the_agents_own),
        "the agent's camera was not drawn in terracotta"
    );
    assert!(
        the_agents_own.solids.len() > anybodys.solids.len(),
        "the agent's line carries no more shapes than anybody's: the dot is missing"
    );
}

/// **An output too small to lay a dock on refuses rather than drawing nothing.**
///
/// A machine that cannot say its camera is on may not put a desktop up instead.
#[test]
fn an_output_that_cannot_hold_the_indicator_is_refused() {
    let lines = vec![used(Used::Camera, 1).line()];
    let mut labels = WindowControlLabels::new().unwrap();

    let refused = picture(
        &lines,
        &words(),
        &Dock::shipped(),
        &mut labels,
        (LARGEST_SIDE + 1, 1080),
        light(),
    );

    assert!(
        matches!(refused, Err(RenderError::InUseScene)),
        "{refused:?}"
    );
}
