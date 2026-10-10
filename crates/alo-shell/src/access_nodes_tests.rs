//! What the shape must hold to, whatever `alo-access` names next.

#![expect(
    clippy::indexing_slicing,
    clippy::expect_used,
    reason = "in a test, a panic on a thing the tree said was there is the failure being reported"
)]

use alo_access::{Control, Surface};

use crate::window_edge_reading::the_controls_of;

use super::*;
use crate::approval_testing::words;
use alo_access::TurnedOn;

/// The tree a person reading English is given, with every surface up — which
/// is not a machine, and is what makes the shape easy to read here. What is
/// really up is [`the_showing_bit_says_which_surfaces_are_up`]'s subject.
fn tree() -> ReadAloudTree {
    ReadAloudTree::of(&words(), &Surface::ALL, &TurnedOn::nothing())
}

/// `ATSPI_STATE_SHOWING`.
const SHOWING: u32 = 25;

/// Whether a set of state words carries a bit.
fn carries(words: [u32; 2], bit: u32) -> bool {
    let set = u64::from(words[0]) | (u64::from(words[1]) << 32);
    set & (1 << bit) != 0
}

/// **A surface that is not up is described and not shown.**
///
/// The whole of what this machine has is in the tree, so a person can be told
/// about a screen they are not looking at; only what is up reads as showing,
/// so a reader does not announce one that is not.
#[test]
fn the_showing_bit_says_which_surfaces_are_up() {
    let strings = words();
    for surface in Surface::ALL {
        let tree = ReadAloudTree::of(&strings, &[surface], &TurnedOn::nothing());
        for (which, at) in tree.surfaces() {
            let node = &tree.nodes()[*at];
            assert_eq!(
                carries(node.states, SHOWING),
                *which == surface,
                "{which:?} while {surface:?} is up"
            );
            for child in &node.children {
                assert_eq!(
                    carries(tree.nodes()[*child].states, SHOWING),
                    *which == surface,
                    "{:?} while {surface:?} is up",
                    tree.nodes()[*child].name
                );
            }
        }
    }
}

/// **Every surface `alo-access` names is in the tree**, and each is there
/// once.
#[test]
fn every_surface_is_somewhere_in_the_tree() {
    let tree = tree();
    let named: Vec<Surface> = tree
        .surfaces()
        .iter()
        .map(|(surface, _)| *surface)
        .collect();
    assert_eq!(named, Surface::ALL.to_vec());
}

/// **Every control a reader is told about is a thing on the bus**, with the
/// name `alo-access` gave it, in the order it is read.
#[test]
fn every_control_is_a_thing_with_the_name_it_was_given() {
    let strings = words();
    let tree = ReadAloudTree::of(&strings, &Surface::ALL, &TurnedOn::nothing());
    for (surface, at) in tree.surfaces() {
        let drawn = surface.read_aloud();
        let mut said: Vec<String> = Vec::new();
        let node = &tree.nodes()[*at];
        if node.role != crate::access_roles::FILLER {
            said.push(node.name.clone());
        }
        for child in &node.children {
            said.push(tree.nodes()[*child].name.clone());
        }
        let expected: Vec<String> = drawn
            .iter()
            .map(|control| {
                strings
                    .say(&control.name.key(), &Filling::nothing())
                    .text()
                    .to_owned()
            })
            .collect();
        assert_eq!(said, expected, "{surface:?} is read in another order");
    }
}

/// **Nothing is named nothing, except what nobody decided a name for.**
///
/// Two things have none: the box a surface's controls hang in where
/// `alo-access` decided no container, and the shell itself, which no crate
/// words. Every control does, because a thing a reader announces with no name
/// is a thing a person is told nothing about.
#[test]
fn everything_but_a_filler_and_the_shell_itself_is_named() {
    let tree = tree();
    for (at, node) in tree.nodes().iter().enumerate() {
        assert_eq!(
            node.name.is_empty(),
            at == 0 || node.role == crate::access_roles::FILLER,
            "{node:?}"
        );
    }
}

/// **Every thing is addressable, and no two are at one address.**
#[test]
fn nothing_is_at_the_same_place_as_anything_else() {
    let tree = tree();
    let mut paths: Vec<&str> = tree.nodes().iter().map(|node| node.path.as_str()).collect();
    let how_many = paths.len();
    paths.sort_unstable();
    paths.dedup();
    assert_eq!(paths.len(), how_many, "two things are at one address");
    assert_eq!(tree.nodes()[0].path, ROOT, "the shell is not at the root");
}

/// **It is a tree**: everything hangs under something that says it is there,
/// and everything is reachable from the shell itself.
#[test]
fn everything_hangs_under_something_that_says_so() {
    let tree = tree();
    for (at, node) in tree.nodes().iter().enumerate().skip(1) {
        assert!(
            tree.nodes()[node.parent].children.contains(&at),
            "{} hangs under something that does not say so",
            node.path
        );
    }
    let mut reached = vec![false; tree.how_many()];
    let mut from = vec![0_usize];
    while let Some(at) = from.pop() {
        if reached[at] {
            continue;
        }
        reached[at] = true;
        from.extend(tree.nodes()[at].children.iter().copied());
    }
    assert!(
        reached.iter().all(|one| *one),
        "something in the tree cannot be reached from the shell"
    );
}

/// **What is announced is the two indicators and nothing else** — law 1's, and
/// the agent's.
#[test]
fn what_is_announced_is_the_two_indicators() {
    let tree = tree();
    let announced: Vec<&str> = tree
        .nodes()
        .iter()
        .filter(|node| node.announced)
        .map(|node| node.name.as_str())
        .collect();
    assert_eq!(
        announced,
        vec!["something is leaving this machine", "the agent is working"],
        "what a reader is told without looking for it has changed"
    );
}

/// **The whole tree is one description**: as many controls on the bus as
/// `alo-access` decided, plus the shell itself and a box for each surface that
/// needed one.
#[test]
fn the_tree_holds_every_control_and_nothing_else() {
    let tree = tree();
    let controls: usize = Surface::ALL
        .into_iter()
        .map(|surface| surface.read_aloud().len())
        .sum();
    let fillers = Surface::ALL
        .into_iter()
        .filter(|surface| {
            surface
                .read_aloud()
                .first()
                .is_none_or(|first| !crate::access_roles::holds_others(first.role))
        })
        .count();
    assert_eq!(
        tree.how_many(),
        controls + fillers + 1,
        "the tree has things in it nobody decided"
    );
}

/// **A control that cannot be used is read and passed over**, which on the bus
/// is the same answer `alo_access::focus_order` gives.
#[test]
fn the_keyboards_stops_are_the_controls_that_can_be_used() {
    for surface in Surface::ALL {
        let stops: Vec<Control> = alo_access::focus_order(surface);
        let usable: Vec<Control> = surface
            .read_aloud()
            .into_iter()
            .filter(Control::can_be_used)
            .collect();
        assert_eq!(stops, usable, "{surface:?}");
    }
}

/// **The buttons a reader is told about are the buttons that are drawn** —
/// the set comparison that was missing, in the one crate that can see both
/// lists.
///
/// [ADR 0089](../../../docs/decisions/0089-what-a-control-is-called.md) was
/// written about what the two lists had drifted into: `alo-access` announced
/// *close this window* and *move this window* while this crate drew minimise,
/// maximise and close. Two buttons drawn and never announced, one announced
/// **`Surface::WindowControls` is a surface nothing draws**, and that is
/// recorded here rather than left for a reader to discover.
///
/// This test asserted that the three tiles a window drew and the three a reader
/// was told about were the same **set** — both ways round, because a
/// containment check would have passed while the strip was short by two. The
/// strip was retired on 2026-10-10 by
/// `docs/design/the-external-window-edge.md`, so there is nothing left to draw
/// and the comparison has one empty side.
///
/// **The guarantee moved and did not go.** `window_edge_reading` and
/// `access_nodes`' own
/// `what_a_reader_is_told_is_what_the_edge_lays_out` hold exactly it for the
/// edge: one function answers which controls a window has, and the layout and
/// the reader both ask it, so a control drawn that no reader can name is
/// unrepresentable rather than merely tested for.
///
/// **What is still owed**, and this is what the assertion below is for:
/// `alo_access::Surface::WindowControls` still declares those controls as a
/// surface of this machine. `alo-access` is a public contract, so retiring an
/// entry from it is versioning and deprecation and not a deletion inside
/// somebody's removal. Until that happens the tree carries a surface no
/// machine draws, and this fails the day somebody wires one again.
#[test]
fn the_retired_control_surface_is_still_declared_and_nothing_draws_it() {
    let announced: Vec<alo_shortcuts::Action> = Surface::WindowControls
        .read_aloud()
        .into_iter()
        .filter_map(|control| control.does)
        .collect();
    assert_eq!(
        announced.len(),
        3,
        "the contract still declares three window controls: {announced:?}"
    );
    // **Nothing in this crate lays them out.** Checked as an absence of the
    // type rather than of a call, because the layout was the only thing that
    // could produce one and it is gone.
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let laid_out = std::fs::read_dir(&src)
        .expect("the source directory is beside this file")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|it| it == "rs"))
        .filter(|entry| {
            // **Not the file asking.** This names the thing it forbids in its
            // own message, so counting itself made the guard fail for the
            // reason it exists to catch. Test files are out of scope anyway:
            // the question is what a machine runs.
            !entry.file_name().to_string_lossy().ends_with("_tests.rs")
        })
        .filter(|entry| {
            std::fs::read_to_string(entry.path())
                .is_ok_and(|text| text.contains("WindowControlLayout"))
        })
        .count();
    assert_eq!(
        laid_out, 0,
        "a file lays out the retired control strip again; the edge is          `crate::window_edge::edge_of` and this surface is owed a deprecation"
    );
}

/// **And each of those buttons is said in the words it is drawn with**, which
/// is clause 11.2.5.3 — the programmatic name contains the visible label —
/// held by there being one string rather than two that agree.
#[test]
fn a_buttons_spoken_name_is_the_label_drawn_on_it() {
    let strings = words();
    for control in Surface::WindowControls.read_aloud() {
        let action = control.does.expect("a button performs something");
        assert_eq!(
            strings
                .say(&control.name.key(), &alo_strings::Filling::nothing())
                .text(),
            action.said(&strings).text(),
            "{action:?} is read aloud as something other than its label"
        );
    }
}

/// **Each open window carries what can be done to it**, which is the owner's
/// *without first hovering* clause of 2026-10-09 as a test.
///
/// Before this, the list of windows held names and nothing under them: a reader
/// was told a window was open and never that it could be closed. The tree is
/// built from what is open and never from where a pointer is, so there is no
/// state in which a person can hear one of these and not reach it.
#[test]
fn every_open_window_carries_the_controls_of_its_edge() {
    let strings = words();
    let tree = ReadAloudTree::of(&strings, &Surface::ALL, &TurnedOn::nothing())
        .with_the_frames_open(
            &strings,
            &[
                FrameName::Given("Ledger for March".to_owned()),
                FrameName::AnApplication,
            ],
            &TurnedOn::nothing(),
        );

    let expected: Vec<String> = what_a_reader_is_told(who_draws_a_frame())
        .into_iter()
        .map(|control| said(&strings, control.name))
        .collect();
    assert_eq!(expected.len(), 3, "{expected:?}");

    let list = tree
        .the_windows_open()
        .expect("the desktop has a list of windows");
    let windows = &tree.nodes()[list].children;
    assert_eq!(windows.len(), 2, "two windows opened");
    for at in windows {
        let window = &tree.nodes()[*at];
        let under: Vec<String> = window
            .children
            .iter()
            .map(|child| tree.nodes()[*child].name.clone())
            .collect();
        assert_eq!(
            under, expected,
            "{} was told to a reader with nothing that could be done to it",
            window.name
        );
    }
}

/// **A window's controls are the ones its edge draws**, asked of the one
/// function both sides ask.
///
/// Two answers to *which controls does this window have* is the drift this
/// guards: a reader naming a control nobody drew, or a control drawn that no
/// reader can name, are the same bug read from two ends.
#[test]
fn what_a_reader_is_told_is_what_the_edge_lays_out() {
    let window = smithay::utils::Rectangle::new(
        smithay::utils::Point::from((40, 100)),
        smithay::utils::Size::from((600, 400)),
    );
    let drawn: Vec<_> = crate::window_edge::edge_of(window, who_draws_a_frame(), true)
        .controls
        .into_iter()
        .map(|control| control.does)
        .collect();
    assert_eq!(drawn, the_controls_of(who_draws_a_frame()));
}
