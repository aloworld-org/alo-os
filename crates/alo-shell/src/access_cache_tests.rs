//! What a reader holding the cache is told, decided without a bus. The cache
//! on a real bus is `access_bus_tests`', and libatspi reading it is
//! `tests/a_screen_readers_own_library_reads_the_tree.rs`'.

use alo_access::{Surface, TurnedOn};
use zbus::zvariant::Type as _;

use super::*;
use crate::access_nodes::ReadAloudTree;
use crate::approval_testing::words;

/// Every entry of a tree with `showing` up.
fn entries(showing: &[Surface]) -> Vec<Entry> {
    let tree = ReadAloudTree::of(&words(), showing, &TurnedOn::nothing());
    tree.nodes()
        .iter()
        .map(|node| entry_of(&ReadAloud::of(node, &tree, ":1.7")))
        .collect()
}

/// The entry is the one shape `Cache.xml` gives, field for field.
#[test]
fn an_entry_is_the_shape_cache_xml_gives() {
    assert_eq!(Entry::SIGNATURE.to_string(), "((so)(so)(so)iiassusau)");
}

/// An entry says what the node says one question at a time.
#[test]
fn an_entry_says_what_the_node_says() {
    let tree = ReadAloudTree::of(&words(), &Surface::ALL, &TurnedOn::nothing());
    let node = &tree.nodes()[1];
    let answers = ReadAloud::of(node, &tree, ":1.7");
    let entry = entry_of(&answers);
    assert_eq!(entry.0, answers.itself);
    assert_eq!(entry.2, answers.parent);
    assert_eq!(entry.3, answers.index_in_parent);
    assert_eq!(entry.4, answers.how_many_inside());
    assert_eq!(entry.5, answers.interfaces());
    assert_eq!(entry.6, node.name);
    assert_eq!(entry.7, node.role);
    assert_eq!(entry.9, node.states.to_vec());
}

/// The same tree twice tells a reader nothing.
#[test]
fn the_same_tree_changes_nothing() {
    let all = entries(&Surface::ALL);
    assert_eq!(what_changed(&all, &all), Changed::default());
}

/// A surface coming up changes what its nodes say, and only those are sent.
#[test]
fn a_surface_coming_up_sends_what_changed_and_nothing_else() {
    let before = entries(&[Surface::Desktop]);
    let after = entries(&Surface::ALL);
    let changed = what_changed(&before, &after);
    assert!(changed.removed.is_empty(), "{:?}", changed.removed);
    assert!(!changed.added.is_empty());
    assert!(changed.added.len() < after.len());
    for entry in &changed.added {
        assert!(!before.contains(entry));
    }
}

/// A path that is gone is removed, by its reference.
#[test]
fn a_path_that_is_gone_is_removed() {
    let all = entries(&Surface::ALL);
    let fewer = &all[..all.len() - 1];
    let changed = what_changed(&all, fewer);
    assert_eq!(changed.removed, vec![all[all.len() - 1].0.clone()]);
    assert!(changed.added.is_empty());
}
