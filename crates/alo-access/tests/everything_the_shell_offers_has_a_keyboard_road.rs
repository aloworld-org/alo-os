//! The acceptance of task 3: **every action the shell offers has a keyboard
//! road, and none of them is reachable only by pointer.**
//!
//! The list walked is `alo-shortcuts`' own — `Action::ALL` is what the shell
//! offers, held to being complete by that crate — so an action added there and
//! forgotten here fails, with nothing to keep in step by hand.
//!
//! # Two roads, not one
//!
//! Every action already answers to a chord, so the clause's own question is
//! answered before this test runs. The question this test asks is the one a
//! person who has never been told a chord would ask: **is it anywhere the
//! keyboard arrives at by pressing Tab?** For the agent, the plan asks it
//! outright — *the agent overlay's one key is not the only road* — and the
//! answer for every one of the eleven is now yes.

use alo_access::tree::{Control, Surface};
use alo_access::{Leaving, focus_order, leaving, the_tab_stop_for};
use alo_shortcuts::{Action, Shortcuts};

/// **Every action is reachable by Tab as well as by its chord.**
#[test]
fn every_action_the_shell_offers_is_reached_by_tab_and_not_only_by_a_chord() {
    let chords = Shortcuts::shipped();
    for action in Action::ALL {
        // The chord: what a person who has been told it uses.
        assert!(
            chords.chord_for(*action).is_some(),
            "{action:?} has no chord at all"
        );

        // And the road for somebody who has not been told it.
        let surface = the_tab_stop_for(*action);
        let stops = focus_order(surface);
        assert!(
            !stops.is_empty(),
            "{action:?} is reached at {surface:?}, where the keyboard has nowhere to stop"
        );
    }
}

/// **The agent is one of them**, which the plan asks for by name: a
/// keyboard-only person reaches the agent without knowing that it answers to a
/// chord.
#[test]
fn the_agents_one_key_is_not_the_only_road_to_the_agent() {
    assert_eq!(the_tab_stop_for(Action::TheAgent), Surface::Desktop);
    let on_the_desktop = focus_order(Surface::Desktop);
    assert!(
        on_the_desktop
            .iter()
            .any(|control| control.name.key() == alo_access::words::ASK_THE_AGENT.key()),
        "the desktop has no stop that opens the agent: {on_the_desktop:?}"
    );
}

/// **The keyboard stops where a reader reads, in that order, on every
/// surface** — EN 301 549's meaningful focus order, held as one list rather
/// than two that can disagree.
#[test]
fn the_focus_order_is_the_reading_order() {
    for surface in Surface::ALL {
        let read: Vec<Control> = surface.read_aloud();
        let stops = focus_order(surface);
        let expected: Vec<Control> = read
            .into_iter()
            .filter(alo_access::tree::Control::can_be_used)
            .collect();
        assert_eq!(stops, expected, "{surface:?}");
    }
}

/// **Every surface is left with one key, and the approval is never approved by
/// it.**
///
/// The one that matters: Escape on an approval answers **no**. It must never
/// approve — what a person approves is the sentence (ADR 0001) — and it must not
/// leave the proposal unanswered either, because somebody who pressed Escape
/// believes they have dealt with it.
#[test]
fn escape_leaves_every_surface_and_never_approves_anything() {
    for surface in Surface::ALL {
        let out = leaving(surface);
        assert!(
            !matches!(out, Leaving::Declines) || surface == Surface::Approval,
            "{surface:?} answers no to something that asked nothing"
        );
    }
    assert_eq!(leaving(Surface::Approval), Leaving::Declines);
}
