//! What is filling a folder, against the real disk.
//!
//! The unit tests in `counting.rs` hold the arithmetic against a walk a test
//! wrote out. These build folders of known bytes on this machine and read the
//! tree back, which is the whole of the plan's acceptance: *true to the byte
//! for the one directory a person opens up rather than roughly right for the
//! whole disk.*
//!
//! | The promise | The test |
//! |---|---|
//! | each node's size is its children plus its own files, to the byte | [`every_size_is_the_sum_of_its_children_plus_its_own_files_to_the_byte`] |
//! | a hard link is counted once per tree, not once per name | [`a_file_with_two_names_is_counted_once_and_the_second_name_says_where`] |
//! | a link is the bytes of the link and is never followed | [`a_link_is_the_bytes_of_the_link_and_is_never_followed`] |
//! | a folder the person may not read is a node saying so, not a zero | [`a_folder_that_cannot_be_read_is_a_node_saying_so_rather_than_a_zero`] |
//! | a walk cut short says so rather than reporting a partial sum as a total | [`a_count_that_reaches_the_bound_says_so_on_the_folder_and_above_the_tree`] |
//! | naming the root is answered with a tree that stops at each mount point | [`naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so`] |
//! | a folder that is not there, or not a folder, is refused in words | [`a_folder_that_is_not_there_or_not_a_folder_is_refused_in_words`] |
//!
//! Linux only, because [`alo_measuring::Holding::of`] is: every test here
//! opens the real disk.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use alo_measuring::{Counted, Holding, Kind, Node, NotMeasured, measuring_words};
use alo_strings::Strings;

/// A folder of this test's own, under this machine's temporary directory,
/// named after the test so a leftover says which test left it.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-measuring-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    folder.canonicalize().unwrap()
}

/// The child of this node with this name.
fn child<'a>(node: &'a Node, name: &str) -> &'a Node {
    node.children
        .iter()
        .find(|child| child.name == name)
        .unwrap_or_else(|| panic!("{name} under {}", node.name))
}

/// Both crates' words, with nothing translated.
fn in_english() -> Strings {
    let mut vocabulary = measuring_words().unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// **Every size is the sum of its children plus its own files, to the
/// byte.** A folder of known bytes, read back; and every node's own bytes
/// agree with what `stat` says about its path.
#[test]
fn every_size_is_the_sum_of_its_children_plus_its_own_files_to_the_byte() {
    let root = a_folder_of_our_own("known-bytes");
    fs::write(root.join("notes.txt"), b"hello").unwrap();
    fs::create_dir_all(root.join("2026/April/empty")).unwrap();
    fs::write(root.join("2026/march.pdf"), b"an invoice").unwrap();
    fs::write(root.join("2026/April/scan.tiff"), vec![7; 1000]).unwrap();

    let holding = Holding::of(&root).unwrap();
    assert!(holding.finished);
    assert_eq!(holding.unnamed, 0);
    assert_eq!(holding.folder, root);
    let tree = &holding.tree;
    assert_eq!(tree.at, root);
    assert_eq!(tree.kind, Kind::Folder);
    assert_eq!(tree.own, 0);
    assert_eq!(tree.size, 1015);
    assert_eq!(tree.counted, Counted::Whole);
    let names: Vec<_> = tree
        .children
        .iter()
        .map(|child| child.name.as_str())
        .collect();
    assert_eq!(names, ["2026", "notes.txt"]);

    let year = child(tree, "2026");
    assert_eq!(year.size, 1010);
    let april = child(year, "April");
    assert_eq!(april.size, 1000);
    let empty = child(april, "empty");
    assert_eq!(empty.size, 0);
    assert_eq!(empty.counted, Counted::Whole, "empty, and known to be");
    assert_eq!(child(april, "scan.tiff").own, 1000);
    assert_eq!(child(year, "march.pdf").own, 10);
    assert_eq!(child(tree, "notes.txt").own, 5);

    // Named for where it came from: `stat` on the path says the same.
    let mut pending = vec![tree];
    while let Some(node) = pending.pop() {
        if node.kind != Kind::Folder {
            assert_eq!(
                node.own,
                fs::symlink_metadata(&node.at).unwrap().len(),
                "{}",
                node.at.display()
            );
        }
        pending.extend(&node.children);
    }

    let _ = fs::remove_dir_all(&root);
}

/// **A file with two names is counted once per tree, not once per name.** Two
/// names for one file of a hundred bytes make a folder of a hundred bytes,
/// and the second name says which name the bytes are under.
#[test]
fn a_file_with_two_names_is_counted_once_and_the_second_name_says_where() {
    let root = a_folder_of_our_own("two-names");
    fs::write(root.join("one.txt"), vec![1; 100]).unwrap();
    fs::hard_link(root.join("one.txt"), root.join("two.txt")).unwrap();
    fs::create_dir(root.join("inside")).unwrap();
    fs::hard_link(root.join("one.txt"), root.join("inside/three.txt")).unwrap();
    fs::write(root.join("other.txt"), b"x").unwrap();

    let holding = Holding::of(&root).unwrap();
    assert_eq!(holding.tree.size, 101, "not 301");
    let one = child(&holding.tree, "one.txt");
    assert_eq!(one.own, 100);
    assert_eq!(one.counted, Counted::Whole);
    for other in [
        child(&holding.tree, "two.txt"),
        child(child(&holding.tree, "inside"), "three.txt"),
    ] {
        assert_eq!(other.own, 0, "{}", other.name);
        assert_eq!(
            other.counted,
            Counted::Elsewhere {
                at: root.join("one.txt")
            },
            "{}",
            other.name
        );
        let said = other.counted.said(&in_english()).unwrap();
        assert!(said.text().contains("one.txt"), "{said}");
    }
    assert_eq!(child(&holding.tree, "inside").size, 0);

    let _ = fs::remove_dir_all(&root);
}

/// **A link is the bytes of the link and is never followed.** A link to a
/// large file elsewhere, and a link to a whole folder elsewhere, add the
/// length of the link and nothing behind it.
#[test]
fn a_link_is_the_bytes_of_the_link_and_is_never_followed() {
    let top = a_folder_of_our_own("links");
    let counted = top.join("Counted");
    let elsewhere = top.join("Elsewhere");
    fs::create_dir_all(&counted).unwrap();
    fs::create_dir_all(elsewhere.join("deep")).unwrap();
    fs::write(elsewhere.join("big.bin"), vec![0; 100_000]).unwrap();
    fs::write(elsewhere.join("deep/more.bin"), vec![0; 50_000]).unwrap();
    std::os::unix::fs::symlink(elsewhere.join("big.bin"), counted.join("big.bin")).unwrap();
    std::os::unix::fs::symlink(&elsewhere, counted.join("everything")).unwrap();
    fs::write(counted.join("mine.txt"), b"mine").unwrap();

    let holding = Holding::of(&counted).unwrap();
    let big = child(&holding.tree, "big.bin");
    let everything = child(&holding.tree, "everything");
    for link in [big, everything] {
        assert_eq!(link.kind, Kind::Link, "{}", link.name);
        assert_eq!(
            link.own,
            fs::read_link(&link.at).unwrap().as_os_str().len() as u64,
            "{}",
            link.name
        );
        assert!(link.children.is_empty(), "{}", link.name);
        assert_eq!(link.counted, Counted::Whole);
    }
    assert_eq!(holding.tree.size, big.own + everything.own + 4);
    assert!(
        holding.tree.size < 1000,
        "nothing behind a link was counted"
    );

    let _ = fs::remove_dir_all(&top);
}

/// **A folder the machine will not read is a node saying so, not a zero.**
///
/// Made unreadable by nesting it deeper than the kernel will open a path —
/// rather than by taking its permissions away, because the supervisor's
/// gates run as the administrator, whom permissions do not refuse. The
/// refusal reaches the walk as the same error from the same call, and the
/// node says what the machine said.
#[test]
fn a_folder_that_cannot_be_read_is_a_node_saying_so_rather_than_a_zero() {
    let root = a_folder_of_our_own("unreadable");
    fs::write(root.join("counted.txt"), b"seven b").unwrap();
    let deepest = a_folder_too_deep_to_open(&root);

    let holding = Holding::of(&root).unwrap();
    assert!(holding.finished);
    assert_eq!(holding.tree.size, 7, "what could be seen");
    assert_eq!(child(&holding.tree, "counted.txt").own, 7);

    // Down the chain to the folder the machine would not open.
    let mut node = child(&holding.tree, "deep");
    let mut depth = 1;
    while let Some(next) = node.children.first() {
        assert_eq!(node.counted, Counted::Whole, "at depth {depth}");
        node = next;
        depth += 1;
    }
    assert!(node.at.starts_with(&root));
    assert!(deepest.starts_with(&node.at));
    let Counted::NotRead { why } = &node.counted else {
        panic!("{:?} at depth {depth}", node.counted);
    };
    assert!(!why.is_empty());
    assert_eq!(node.size, 0);
    let said = node.counted.said(&in_english()).unwrap();
    assert!(!said.is_a_bug(), "{said}");

    let _ = fs::remove_dir_all(&root);
}

/// A folder nested so deep that its full path is longer than the kernel
/// will open, made one level at a time from inside the previous one so that
/// making it needs no path that long. Answers the deepest folder's path.
fn a_folder_too_deep_to_open(root: &Path) -> PathBuf {
    let was = std::env::current_dir().unwrap();
    let mut at = root.join("deep");
    fs::create_dir(&at).unwrap();
    std::env::set_current_dir(&at).unwrap();
    // Each level adds two bytes; the kernel's limit is 4096.
    while at.as_os_str().len() <= 4200 {
        fs::create_dir("d").unwrap();
        std::env::set_current_dir("d").unwrap();
        at.push("d");
    }
    std::env::set_current_dir(was).unwrap();
    at
}

/// **A count that reaches the walk's bound says so** on the folder it had
/// not finished and once above the tree, rather than reporting the part it
/// saw as the whole.
#[test]
fn a_count_that_reaches_the_bound_says_so_on_the_folder_and_above_the_tree() {
    let root = a_folder_of_our_own("bounded");
    let many = root.join("many");
    fs::create_dir(&many).unwrap();
    for which in 0..=alo_files::MOST_WALKED {
        fs::write(many.join(format!("{which}.txt")), b"x").unwrap();
    }
    fs::write(root.join("first.txt"), b"first").unwrap();

    let holding = Holding::of(&root).unwrap();
    assert!(!holding.finished);
    assert_eq!(holding.most, alo_files::MOST_WALKED);
    let many = child(&holding.tree, "many");
    assert_eq!(many.counted, Counted::NotFinished);
    assert!(
        many.size < alo_files::MOST_WALKED as u64 + 1,
        "{}",
        many.size
    );
    assert_eq!(
        holding.tree.counted,
        Counted::Whole,
        "the folder asked about was itself fully listed"
    );
    let strings = in_english();
    let said = holding.not_the_whole(&strings).unwrap();
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("20"), "{said}");

    let _ = fs::remove_dir_all(&root);
}

/// **Naming the root of the machine is answered with a tree that stops at
/// each mount point and says so.** `/proc` is on every Linux machine and is
/// not the disk; nothing under it is walked, and its node says why.
#[test]
fn naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so() {
    let holding = Holding::of(Path::new("/")).unwrap();
    assert_eq!(holding.tree.name, "/");
    let proc = child(&holding.tree, "proc");
    assert_eq!(proc.counted, Counted::OnAnotherFilesystem);
    assert!(proc.children.is_empty());
    assert_eq!(proc.size, 0);
    let said = proc.counted.said(&in_english()).unwrap();
    assert!(!said.is_a_bug(), "{said}");
    // A machine has more than one bound's worth of files, and the answer
    // says so rather than calling the part it saw the whole.
    assert!(!holding.finished);
}

/// **A folder that is not there, or is not a folder, is refused in words**
/// — the refusal of the whole question, since there is no tree to put a
/// node in — and the words name the folder.
#[test]
fn a_folder_that_is_not_there_or_not_a_folder_is_refused_in_words() {
    let root = a_folder_of_our_own("refused");
    fs::write(root.join("march.pdf"), b"an invoice").unwrap();
    let strings = in_english();

    let gone = Holding::of(&root.join("Taxes")).unwrap_err();
    let NotMeasured::NotCounted { at, why } = &gone else {
        panic!("{gone:?}");
    };
    assert_eq!(at, &root.join("Taxes"));
    assert!(matches!(why, alo_files::Failed::Gone { .. }), "{why:?}");
    let said = gone.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("Taxes"), "{said}");

    let not_a_folder = Holding::of(&root.join("march.pdf")).unwrap_err();
    let NotMeasured::NotCounted { why, .. } = &not_a_folder else {
        panic!("{not_a_folder:?}");
    };
    assert!(!matches!(why, alo_files::Failed::Gone { .. }), "{why:?}");
    let said = not_a_folder.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("march.pdf"), "{said}");

    let _ = fs::remove_dir_all(&root);
}
