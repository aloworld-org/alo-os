//! What one node answers, read off the node rather than off a bus — the
//! signatures on a real bus are `access_bus_tests`', and what libatspi itself
//! makes of them is `tests/a_screen_readers_own_library_reads_the_tree.rs`'.

use alo_access::{Surface, TurnedOn};
use zbus::zvariant::Type as _;

use super::*;
use crate::approval_testing::words;

/// The whole tree, every surface up.
fn tree() -> ReadAloudTree {
    ReadAloudTree::of(&words(), &Surface::ALL, &TurnedOn::nothing())
}

/// **The shell hangs under the desktop it was embedded in, not under
/// itself.** Until 2026-11-13 the root's `Parent` named the root.
#[test]
fn the_shell_hangs_under_the_registrys_desktop() {
    let tree = tree();
    let root = ReadAloud::of(&tree.nodes()[0], &tree, ":1.7");
    assert_eq!(root.parent.0, REGISTRY);
    assert_eq!(root.parent.1.as_str(), ROOT);
    assert_eq!(root.index_in_parent, -1);
    assert_eq!(root.itself, (":1.7".to_owned(), root.application.1.clone()));
}

/// Every other node hangs under the node that lists it, held by the shell's
/// own name, and says where among its parent's children it is.
#[test]
fn every_node_hangs_where_its_parent_lists_it() {
    let tree = tree();
    let all: Vec<ReadAloud> = tree
        .nodes()
        .iter()
        .map(|node| ReadAloud::of(node, &tree, ":1.7"))
        .collect();
    for answers in all.iter().skip(1) {
        let parent = all
            .iter()
            .find(|other| other.itself == answers.parent)
            .unwrap();
        let at = usize::try_from(answers.index_in_parent).unwrap();
        assert_eq!(parent.children[at], answers.itself);
        assert_eq!(answers.application.1.as_str(), ROOT);
    }
}

/// A child that is not there is the null reference, not an error and not a
/// panic — and the answer is one `(so)` structure, not two loose values.
#[test]
fn a_child_at_an_index_is_one_structure() {
    let tree = tree();
    let root = ReadAloud::of(&tree.nodes()[0], &tree, ":1.7");
    assert_eq!(
        <(Reference,)>::SIGNATURE.to_string(),
        "((so))",
        "the reply's single argument is a (so) structure"
    );
    let (missing,) = root.get_child_at_index(10_000);
    assert_eq!(missing, (String::new(), nothing()));
    let (negative,) = root.get_child_at_index(-1);
    assert_eq!(negative, (String::new(), nothing()));
    let (first,) = root.get_child_at_index(0);
    assert_eq!(first, root.children[0]);
}
