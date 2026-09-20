//! What the shape must hold to, whatever `alo-access` names next.

#![expect(
    clippy::indexing_slicing,
    reason = "in a test, a panic on a thing the tree said was there is the failure being reported"
)]

use alo_access::{Control, Surface};

use super::*;
use crate::approval_testing::words;

/// The tree a person reading English is given, with every surface up — which
/// is not a machine, and is what makes the shape easy to read here. What is
/// really up is [`the_showing_bit_says_which_surfaces_are_up`]'s subject.
fn tree() -> ReadAloudTree {
    ReadAloudTree::of(&words(), &Surface::ALL)
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
        let tree = ReadAloudTree::of(&strings, &[surface]);
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
    let tree = ReadAloudTree::of(&strings, &Surface::ALL);
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
