//! What is filling a folder, as the rows of a panel: the tree `alo-measuring`
//! counted, with the folders a person opened open.
//!
//! The folder asked about is always the first row. Under every open folder,
//! what is inside it, in the order `alo_measuring::Holding` holds it — the
//! order a person reads a folder — each set in one level further. A row is the
//! thing's name, its size in bytes as `alo-measuring` counted it, and, when the
//! size is not the whole truth, `alo-measuring`'s sentence saying why.
//!
//! **Nothing here makes a number.** A size is the digits of the size the count
//! gave, never scaled into a unit, added or taken away from, and never sorted
//! by; the tree is not reordered and nothing in it is left out.

use std::collections::BTreeSet;
use std::path::PathBuf;

use alo_measuring::{Holding, Kind, Node};
use alo_strings::{Said, Strings};

use crate::desktop_list::ListRow;

/// One node in view, and how deep it is.
pub(crate) struct InView<'a> {
    /// The node.
    pub(crate) node: &'a Node,
    /// How deep: zero for the folder asked about.
    pub(crate) depth: usize,
}

/// Every node in view, in order: the folder asked about, and under each open
/// folder what is inside it.
///
/// Walked with a list of its own rather than by calling itself, because a
/// folder can be nested deeper than a stack.
pub(crate) fn in_view<'a>(holding: &'a Holding, opened: &BTreeSet<PathBuf>) -> Vec<InView<'a>> {
    let mut seen = Vec::new();
    let mut pending = vec![InView {
        node: &holding.tree,
        depth: 0,
    }];
    while let Some(next) = pending.pop() {
        if opened.contains(&next.node.at) {
            pending.extend(next.node.children.iter().rev().map(|child| InView {
                node: child,
                depth: next.depth + 1,
            }));
        }
        seen.push(next);
    }
    seen
}

/// Whether a node is a folder a person can open.
pub(crate) fn can_open(node: &Node) -> bool {
    node.kind == Kind::Folder
}

/// Every node in view, as rows.
pub(crate) fn rows(
    holding: &Holding,
    opened: &BTreeSet<PathBuf>,
    strings: &Strings,
) -> Vec<ListRow> {
    in_view(holding, opened)
        .into_iter()
        .map(|seen| {
            let mut cells = vec![seen.node.name.clone(), seen.node.size.to_string()];
            cells.extend(seen.node.counted.said(strings).map(Said::into_text));
            ListRow {
                depth: seen.depth,
                opened: can_open(seen.node).then(|| opened.contains(&seen.node.at)),
                cells,
            }
        })
        .collect()
}

/// What `alo-measuring` says above the tree about the count as a whole.
pub(crate) fn remarks(holding: &Holding, strings: &Strings) -> Vec<String> {
    holding
        .not_the_whole(strings)
        .into_iter()
        .chain(holding.left_unnamed(strings))
        .map(Said::into_text)
        .collect()
}
