//! Turning a walk into a tree of sizes, with the arithmetic that makes each
//! size true.
//!
//! `alo-files`' walk answers a flat list — every folder before the things in
//! it, each folder's things in the order a person reads them — plus what it
//! could not read, would not enter and did not finish. This file makes the
//! tree: a node per step, each moved under its parent, each size its own
//! bytes plus its children's, and each of the walk's notes written on the node
//! it is about.
//!
//! # One look at every file, after the walk
//!
//! The walk read a size for each file as it went. The tree looks once more at
//! each one, through a function the caller hands in, and for two reasons.
//! Only a look at the file itself says **how many names it has**, which is
//! what makes a hard link countable once; and the look is *after* the walk,
//! so a file that became a link in between is seen as a link — its own bytes,
//! nothing followed — rather than as the file the walk thought it was. The
//! caller hands the look in so that this file is tested on every host against
//! a walk a test wrote out, and on Linux against the disk.
//!
//! # No recursion
//!
//! A folder can be nested deeper than a stack: the test for a folder the
//! machine will not read makes one two thousand levels down. So the tree is
//! built from an arena in the walk's own order, and assembled by moving each
//! node under its parent from the last step to the first — a parent always
//! comes before its children in the walk, so by the time a parent is moved its
//! children are already in it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use alo_files::{Kind, Walked};

use crate::holding::{Counted, Holding, Node};
use crate::looking::Looked;

/// One look at a thing on the disk, by path: what the machine says about it
/// now, or why it would not say.
pub(crate) type Looking<'a> = &'a mut dyn FnMut(&Path) -> Result<Looked, String>;

/// The tree of sizes under `folder`, from what a measuring walk found there.
///
/// `most` is the bound the walk had, for the sentence the answer says when it
/// reached it.
#[cfg_attr(
    all(not(target_os = "linux"), not(test)),
    expect(
        dead_code,
        reason = "on any other host `Holding::of` refuses before it would count, and this is \
                  reached by the unit tests below"
    )
)]
pub(crate) fn tree_of(folder: &Path, walked: Walked, most: usize, look: Looking<'_>) -> Holding {
    let mut root = Node {
        name: folder.file_name().map_or_else(
            || folder.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        ),
        at: folder.to_path_buf(),
        kind: Kind::Folder,
        own: 0,
        size: 0,
        counted: Counted::Whole,
        children: Vec::new(),
    };

    // The arena: one node per step, in the walk's order, and for each the
    // index of its parent — `None` for the folder asked about.
    let mut nodes: Vec<Option<Node>> = Vec::with_capacity(walked.things.len());
    let mut parents: Vec<Option<usize>> = Vec::with_capacity(walked.things.len());
    let mut index: HashMap<PathBuf, Option<usize>> = HashMap::new();
    index.insert(PathBuf::new(), None);
    // The first name each many-named file was met under.
    let mut first_name_of: HashMap<(u64, u64), PathBuf> = HashMap::new();

    for step in walked.things {
        let parent = step
            .below
            .parent()
            .and_then(|above| index.get(above).copied())
            .unwrap_or(None);
        let name = step
            .below
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let (kind, own, counted) = match step.kind {
            Kind::Folder => (Kind::Folder, 0, Counted::Whole),
            Kind::Link => (Kind::Link, step.bytes, Counted::Whole),
            // A file, or something that is none of the three; either way a
            // thing with bytes of its own, looked at once more.
            _ => match look(&step.at) {
                Err(why) => (step.kind, 0, Counted::NotRead { why }),
                Ok(looked) if looked.is_link => (Kind::Link, looked.bytes, Counted::Whole),
                Ok(looked) if looked.names > 1 => {
                    match first_name_of.get(&(looked.on, looked.inode)) {
                        Some(first) => (step.kind, 0, Counted::Elsewhere { at: first.clone() }),
                        None => {
                            first_name_of.insert((looked.on, looked.inode), step.at.clone());
                            (step.kind, looked.bytes, Counted::Whole)
                        }
                    }
                }
                Ok(looked) => (step.kind, looked.bytes, Counted::Whole),
            },
        };
        let which = nodes.len();
        nodes.push(Some(Node {
            name,
            at: step.at,
            kind,
            own,
            size: own,
            counted,
            children: Vec::new(),
        }));
        parents.push(parent);
        index.insert(step.below, Some(which));
    }

    // The walk's notes, each on the folder it is about. A folder the walk
    // could not read or would not enter was never listed, so it cannot also
    // be one it did not finish.
    let mut counted_as = |below: &Path, counted: Counted, root: &mut Node| match index.get(below) {
        Some(Some(which)) => {
            if let Some(Some(node)) = nodes.get_mut(*which) {
                node.counted = counted;
            }
        }
        Some(None) => root.counted = counted,
        None => {}
    };
    for unread in walked.unread {
        counted_as(
            &unread.below,
            Counted::NotRead { why: unread.why },
            &mut root,
        );
    }
    for below in walked.elsewhere {
        counted_as(&below, Counted::OnAnotherFilesystem, &mut root);
    }
    for below in walked.not_entered {
        counted_as(&below, Counted::NotFinished, &mut root);
    }

    // Assembly, last step first: a node's children were all pushed into it
    // before it is moved under its own parent, in reverse order — put right
    // as it goes.
    for which in (0..nodes.len()).rev() {
        let Some(mut node) = nodes.get_mut(which).and_then(Option::take) else {
            continue;
        };
        node.children.reverse();
        node.size = node
            .children
            .iter()
            .fold(node.own, |sum, child| sum.saturating_add(child.size));
        match parents.get(which).copied().flatten() {
            Some(parent) => {
                if let Some(Some(parent)) = nodes.get_mut(parent) {
                    parent.children.push(node);
                }
            }
            None => root.children.push(node),
        }
    }
    root.children.reverse();
    root.size = root
        .children
        .iter()
        .fold(0, |sum, child| sum.saturating_add(child.size));

    Holding {
        folder: folder.to_path_buf(),
        tree: root,
        finished: !walked.cut_short,
        most,
        unnamed: walked.could_not_be_named,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::SystemTime;

    use alo_files::{Step, Unread};

    use super::*;

    /// A step, as the walk would have written it.
    fn step(below: &str, kind: Kind, bytes: u64) -> Step {
        Step {
            at: Path::new("Documents").join(below),
            below: PathBuf::from(below),
            kind,
            bytes,
            when: SystemTime::UNIX_EPOCH,
        }
    }

    /// A walk that found these things and nothing else to say.
    fn walk_of(things: Vec<Step>) -> Walked {
        Walked {
            things,
            links: 0,
            could_not_be_named: 0,
            cut_short: false,
            unread: Vec::new(),
            elsewhere: Vec::new(),
            not_entered: Vec::new(),
        }
    }

    /// A look that finds every file with one name and the bytes the walk
    /// saw.
    fn one_name_each(bytes: u64) -> impl FnMut(&Path) -> Result<Looked, String> {
        move |_| {
            Ok(Looked {
                bytes,
                names: 1,
                on: 1,
                inode: 1,
                is_link: false,
            })
        }
    }

    /// The child of this node with this name.
    fn child<'a>(node: &'a Node, name: &str) -> &'a Node {
        node.children
            .iter()
            .find(|child| child.name == name)
            .unwrap_or_else(|| panic!("{name} under {}", node.name))
    }

    /// **Each size is its own bytes plus its children's**, all the way up,
    /// and children are in the order the walk listed them.
    #[test]
    fn each_size_is_its_own_bytes_plus_its_childrens() {
        let walked = walk_of(vec![
            step("a", Kind::Folder, 0),
            step("y.txt", Kind::File, 7),
            step("a/b", Kind::Folder, 0),
            step("a/x.txt", Kind::File, 5),
            step("a/b/z.txt", Kind::File, 11),
        ]);
        let mut sizes = HashMap::from([
            (PathBuf::from("Documents/y.txt"), 7),
            (PathBuf::from("Documents/a/x.txt"), 5),
            (PathBuf::from("Documents/a/b/z.txt"), 11),
        ]);
        let mut look = |at: &Path| {
            Ok(Looked {
                bytes: sizes.remove(at).unwrap(),
                names: 1,
                on: 1,
                inode: 1,
                is_link: false,
            })
        };

        let holding = tree_of(Path::new("Documents"), walked, 20_000, &mut look);
        assert!(holding.finished);
        assert_eq!(holding.unnamed, 0);
        let root = &holding.tree;
        assert_eq!(root.name, "Documents");
        assert_eq!(root.size, 23);
        assert_eq!(root.own, 0);
        let names: Vec<_> = root
            .children
            .iter()
            .map(|child| child.name.as_str())
            .collect();
        assert_eq!(names, ["a", "y.txt"]);
        let a = child(root, "a");
        assert_eq!(a.size, 16);
        assert_eq!(a.kind, Kind::Folder);
        let b = child(a, "b");
        assert_eq!(b.size, 11);
        assert_eq!(child(b, "z.txt").own, 11);
        assert_eq!(child(root, "y.txt").counted, Counted::Whole);
        assert!(sizes.is_empty(), "every file was looked at once");
    }

    /// **A file with two names is counted once**, under the first name the
    /// walk met, and the second name says where.
    #[test]
    fn a_file_with_two_names_is_counted_once_and_the_other_name_says_where() {
        let walked = walk_of(vec![
            step("one.txt", Kind::File, 100),
            step("two.txt", Kind::File, 100),
            step("three.txt", Kind::File, 1),
        ]);
        let mut look = |at: &Path| {
            Ok(if at.ends_with("three.txt") {
                Looked {
                    bytes: 1,
                    names: 1,
                    on: 1,
                    inode: 3,
                    is_link: false,
                }
            } else {
                Looked {
                    bytes: 100,
                    names: 2,
                    on: 1,
                    inode: 42,
                    is_link: false,
                }
            })
        };

        let holding = tree_of(Path::new("Documents"), walked, 20_000, &mut look);
        assert_eq!(holding.tree.size, 101, "not 201");
        let one = child(&holding.tree, "one.txt");
        assert_eq!(one.own, 100);
        assert_eq!(one.counted, Counted::Whole);
        let two = child(&holding.tree, "two.txt");
        assert_eq!(two.own, 0);
        assert_eq!(
            two.counted,
            Counted::Elsewhere {
                at: Path::new("Documents").join("one.txt")
            }
        );
    }

    /// **A link is its own bytes and is never looked through.** And a file
    /// that became a link between the walk and the look is a link too.
    #[test]
    fn a_link_is_its_own_bytes_and_nothing_behind_it_is_looked_at() {
        let walked = walk_of(vec![
            step("elsewhere", Kind::Link, 12),
            step("was-a-file.txt", Kind::File, 5000),
        ]);
        let mut looked_at = Vec::new();
        let mut look = |at: &Path| {
            looked_at.push(at.to_path_buf());
            Ok(Looked {
                bytes: 9,
                names: 1,
                on: 1,
                inode: 1,
                is_link: true,
            })
        };

        let holding = tree_of(Path::new("Documents"), walked, 20_000, &mut look);
        let link = child(&holding.tree, "elsewhere");
        assert_eq!(link.kind, Kind::Link);
        assert_eq!(link.own, 12);
        let swapped = child(&holding.tree, "was-a-file.txt");
        assert_eq!(swapped.kind, Kind::Link);
        assert_eq!(swapped.own, 9, "the link's bytes, not the file's");
        assert_eq!(holding.tree.size, 21);
        assert_eq!(
            looked_at,
            [Path::new("Documents").join("was-a-file.txt")],
            "the link was never looked at"
        );
    }

    /// **A file the machine would not look at says so**, and is not a zero
    /// that looks like an empty file.
    #[test]
    fn a_file_that_could_not_be_looked_at_says_so_rather_than_being_a_zero() {
        let walked = walk_of(vec![step("gone.txt", Kind::File, 5000)]);
        let mut look = |_: &Path| Err("no such file".to_owned());

        let holding = tree_of(Path::new("Documents"), walked, 20_000, &mut look);
        let gone = child(&holding.tree, "gone.txt");
        assert_eq!(gone.own, 0);
        assert_eq!(
            gone.counted,
            Counted::NotRead {
                why: "no such file".to_owned()
            }
        );
    }

    /// **What the walk could not read, would not enter, and did not finish
    /// is written on the node it is about** — including the folder asked
    /// about — and the count says it is not the whole.
    #[test]
    fn what_the_walk_could_not_read_enter_or_finish_is_marked_where_it_is() {
        let mut walked = walk_of(vec![
            step("private", Kind::Folder, 0),
            step("mounted", Kind::Folder, 0),
            step("later", Kind::Folder, 0),
            step("seen.txt", Kind::File, 3),
        ]);
        walked.unread.push(Unread {
            below: PathBuf::from("private"),
            why: "permission denied".to_owned(),
        });
        walked.elsewhere.push(PathBuf::from("mounted"));
        walked.cut_short = true;
        walked.not_entered = vec![PathBuf::new(), PathBuf::from("later")];
        walked.could_not_be_named = 2;

        let holding = tree_of(Path::new("Documents"), walked, 4, &mut one_name_each(3));
        assert!(!holding.finished);
        assert_eq!(holding.most, 4);
        assert_eq!(holding.unnamed, 2);
        let root = &holding.tree;
        assert_eq!(root.counted, Counted::NotFinished);
        assert_eq!(root.size, 3, "what was seen, and no more");
        assert_eq!(
            child(root, "private").counted,
            Counted::NotRead {
                why: "permission denied".to_owned()
            }
        );
        assert_eq!(child(root, "mounted").counted, Counted::OnAnotherFilesystem);
        assert_eq!(child(root, "later").counted, Counted::NotFinished);
        assert_eq!(child(root, "seen.txt").counted, Counted::Whole);
    }

    /// A folder nested deeper than a stack would allow is a tree all the
    /// same: nothing here recurses.
    #[test]
    fn a_folder_nested_thousands_deep_is_a_tree_without_recursion() {
        let mut things = Vec::new();
        let mut below = PathBuf::new();
        for _ in 0..5000 {
            below.push("d");
            things.push(Step {
                at: Path::new("Documents").join(&below),
                below: below.clone(),
                kind: Kind::Folder,
                bytes: 0,
                when: SystemTime::UNIX_EPOCH,
            });
        }
        below.push("leaf.txt");
        things.push(Step {
            at: Path::new("Documents").join(&below),
            below,
            kind: Kind::File,
            bytes: 1,
            when: SystemTime::UNIX_EPOCH,
        });

        let holding = tree_of(
            Path::new("Documents"),
            walk_of(things),
            20_000,
            &mut one_name_each(1),
        );
        assert_eq!(holding.tree.size, 1);
        let mut depth = 0;
        let mut node = &holding.tree;
        while let Some(first) = node.children.first() {
            assert_eq!(node.size, 1, "at depth {depth}");
            node = first;
            depth += 1;
        }
        assert_eq!(depth, 5001);
        assert_eq!(node.name, "leaf.txt");
        // Dropping a tree that deep must not recurse off the stack either.
        drop(holding);
    }
}
