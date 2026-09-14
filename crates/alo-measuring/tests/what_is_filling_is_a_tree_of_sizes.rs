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
//! | a folder larger than one walk's bound is counted whole, to the byte, and timed | [`a_folder_larger_than_one_walk_is_counted_whole_to_the_byte_and_timed`] |
//! | a hard link is counted once per tree, not once per name — across the walks too | [`a_file_with_two_names_is_counted_once_and_the_second_name_says_where`] |
//! | a link is the bytes of the link and is never followed | [`a_link_is_the_bytes_of_the_link_and_is_never_followed`] |
//! | a folder the person may not read is a node saying so, not a zero | [`a_folder_that_cannot_be_read_is_a_node_saying_so_rather_than_a_zero`] |
//! | only a folder wider than the bound at one level is cut short, and it says so | [`a_count_that_reaches_the_bound_says_so_on_the_folder_and_above_the_tree`] |
//! | naming the root is answered with a tree that stops at each mount point | [`naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so`] |
//! | a folder that is not there, or not a folder, is refused in words | [`a_folder_that_is_not_there_or_not_a_folder_is_refused_in_words`] |
//!
//! Linux only, because [`alo_measuring::Holding::of`] is: every test here
//! opens the real disk.
//!
//! # The bound, and the tests that cross it
//!
//! Two tests here build more than `alo_files::MOST_WALKED` things in
//! subfolders and read the total back to the byte: the count walks on until
//! the folder is whole, and *cut short* is said only about a folder holding
//! more than the bound directly inside it. The timed one prints what it
//! measured, for the report.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use alo_files::MOST_WALKED;
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

/// Every node of the tree, the root first, each before its children.
fn every_node(tree: &Node) -> Vec<&Node> {
    let mut all = Vec::new();
    let mut pending = vec![tree];
    while let Some(node) = pending.pop() {
        all.push(node);
        pending.extend(node.children.iter().rev());
    }
    all
}

/// `folders` folders named `f-NNN`, each holding `files` files of
/// `bytes_each` bytes and a folder `deeper` holding `deeper_files` more of
/// one byte each. Answers how many things were made and how many bytes.
fn a_tree_of_known_bytes(
    root: &Path,
    folders: usize,
    files: usize,
    bytes_each: usize,
    deeper_files: usize,
) -> (usize, u64) {
    for f in 0..folders {
        let folder = root.join(format!("f-{f:03}"));
        fs::create_dir_all(folder.join("deeper")).unwrap();
        for m in 0..files {
            fs::write(folder.join(format!("{m:04}.bin")), vec![7; bytes_each]).unwrap();
        }
        for m in 0..deeper_files {
            fs::write(folder.join("deeper").join(format!("d-{m:04}.bin")), b"1").unwrap();
        }
    }
    let things = folders * (2 + files + deeper_files);
    let bytes = (folders * (files * bytes_each + deeper_files)) as u64;
    (things, bytes)
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

/// **A folder larger than one walk's bound is counted whole, to the byte.**
/// More than `MOST_WALKED` things in subfolders, of known bytes; the total
/// read back is exactly the bytes written, every folder is whole, nothing is
/// said above the tree, and the time it took is printed for the report.
#[test]
fn a_folder_larger_than_one_walk_is_counted_whole_to_the_byte_and_timed() {
    let root = a_folder_of_our_own("larger-than-one-walk");
    let writing = Instant::now();
    let (things, bytes) = a_tree_of_known_bytes(&root, 120, 150, 13, 50);
    let written = writing.elapsed();
    assert!(
        things > MOST_WALKED,
        "{things} things is more than one walk"
    );
    assert_eq!(things, 24_240);
    assert_eq!(bytes, 240_000);

    let counting = Instant::now();
    let holding = Holding::of(&root).unwrap();
    let counted = counting.elapsed();
    eprintln!(
        "{things} things of {bytes} bytes written in {written:?}, counted whole in {counted:?}"
    );

    assert!(holding.finished, "walked on until whole");
    assert_eq!(holding.most, MOST_WALKED);
    assert_eq!(holding.unnamed, 0);
    assert!(holding.not_the_whole(&in_english()).is_none());
    assert_eq!(holding.tree.size, bytes, "to the byte");
    let nodes = every_node(&holding.tree);
    assert_eq!(nodes.len(), things + 1, "every thing once, and the root");
    for node in &nodes {
        assert_eq!(node.counted, Counted::Whole, "{}", node.at.display());
        if node.kind == Kind::Folder {
            let of_children: u64 = node.children.iter().map(|child| child.size).sum();
            assert_eq!(node.size, of_children, "{}", node.at.display());
        }
    }
    // A folder found by a later walk is where it is, the way one found by
    // the first walk is.
    let last = child(&holding.tree, "f-119");
    assert_eq!(last.at, root.join("f-119"));
    assert_eq!(last.size, 150 * 13 + 50);
    let deeper = child(last, "deeper");
    assert_eq!(deeper.at, root.join("f-119").join("deeper"));
    assert_eq!(deeper.size, 50);
    assert_eq!(deeper.children.len(), 50);

    let _ = fs::remove_dir_all(&root);
}

/// **A file with two names is counted once per tree, not once per name.** Two
/// names for one file of a hundred bytes make a folder of a hundred bytes,
/// and the second name says which name the bytes are under — and it is
/// still one file when its two names are met by two different walks of a
/// folder larger than the bound.
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

    // Past the bound: three folders of seven thousand one-byte files, so the
    // first walk stops inside the last of them and a second walk lists it
    // on. A file named first in the first folder has a second name last in
    // the last folder, met only by the later walk; and a file in the second
    // folder has its second name in the last one, both past the first
    // walk's stop.
    let root = a_folder_of_our_own("two-names-past-the-bound");
    for folder in ["a-first", "b-middle", "c-last"] {
        fs::create_dir(root.join(folder)).unwrap();
        for which in 0..7_000 {
            fs::write(root.join(folder).join(format!("{which:05}.txt")), b"1").unwrap();
        }
    }
    fs::write(root.join("a-first").join("00000-big.bin"), vec![2; 5_000]).unwrap();
    fs::hard_link(
        root.join("a-first").join("00000-big.bin"),
        root.join("c-last").join("zz-big-again.bin"),
    )
    .unwrap();
    fs::write(root.join("b-middle").join("mid.bin"), vec![3; 300]).unwrap();
    fs::hard_link(
        root.join("b-middle").join("mid.bin"),
        root.join("c-last").join("zz-mid-again.bin"),
    )
    .unwrap();

    let holding = Holding::of(&root).unwrap();
    assert!(holding.finished, "more than one walk, and whole");
    assert_eq!(holding.tree.size, 21_000 + 5_000 + 300, "each file once");
    let big = child(child(&holding.tree, "a-first"), "00000-big.bin");
    assert_eq!(big.own, 5_000);
    assert_eq!(big.counted, Counted::Whole);
    let last = child(&holding.tree, "c-last");
    assert_eq!(last.children.len(), 7_002, "the later walk listed it whole");
    assert_eq!(last.size, 7_000, "its two second names hold no bytes");
    let big_again = child(last, "zz-big-again.bin");
    assert_eq!(big_again.own, 0);
    assert_eq!(
        big_again.counted,
        Counted::Elsewhere {
            at: root.join("a-first").join("00000-big.bin")
        }
    );
    let mid_again = child(last, "zz-mid-again.bin");
    assert_eq!(mid_again.own, 0);
    assert_eq!(
        mid_again.counted,
        Counted::Elsewhere {
            at: root.join("b-middle").join("mid.bin")
        }
    );
    assert!(
        every_node(&holding.tree)
            .iter()
            .all(|node| node.counted != Counted::NotFinished),
        "large is not wide"
    );

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

/// **Only a folder holding more than the bound at one level is cut short,
/// and it says so** — on that folder and once above the tree, rather than
/// reporting the part it saw as the whole. A folder beside it that is merely
/// large, with more than the bound in subfolders, is counted whole in the
/// same tree.
#[test]
fn a_count_that_reaches_the_bound_says_so_on_the_folder_and_above_the_tree() {
    let root = a_folder_of_our_own("bounded");
    let many = root.join("many");
    fs::create_dir(&many).unwrap();
    for which in 0..=MOST_WALKED {
        fs::write(many.join(format!("{which:05}.txt")), b"x").unwrap();
    }
    fs::write(root.join("first.txt"), b"first").unwrap();
    let (things, bytes) = a_tree_of_known_bytes(&root.join("large"), 110, 150, 1, 50);
    assert!(things > MOST_WALKED);

    let holding = Holding::of(&root).unwrap();
    assert!(!holding.finished);
    assert_eq!(holding.most, MOST_WALKED);
    let many = child(&holding.tree, "many");
    assert_eq!(many.counted, Counted::NotFinished);
    assert_eq!(
        many.children.len(),
        MOST_WALKED,
        "exactly one walk's worth of names, which tells a folder too wide from one merely large"
    );
    assert_eq!(many.size, MOST_WALKED as u64, "what was seen, and no more");
    assert_eq!(
        holding.tree.counted,
        Counted::Whole,
        "the folder asked about was itself fully listed"
    );
    let large = child(&holding.tree, "large");
    assert_eq!(large.counted, Counted::Whole, "large is not wide");
    assert_eq!(large.size, bytes, "counted whole, to the byte");
    assert_eq!(
        every_node(&holding.tree)
            .iter()
            .filter(|node| node.counted == Counted::NotFinished)
            .count(),
        1,
        "one folder is cut short, and only that one"
    );
    assert_eq!(holding.tree.size, MOST_WALKED as u64 + 5 + bytes);
    let strings = in_english();
    let said = holding.not_the_whole(&strings).unwrap();
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("20"), "{said}");
    assert!(said.text().contains("single level"), "{said}");

    let _ = fs::remove_dir_all(&root);
}

/// **Naming the root of the machine is answered with a tree that stops at
/// each mount point and says so.** `/proc` is on every Linux machine and is
/// not the disk; nothing under it is walked, and its node says why. The
/// root filesystem itself is counted whole — a machine is more than one
/// walk, and the count walks on — and the only folder that may be cut short
/// is one holding more than the bound directly inside it. Prints what it
/// measured, for the report.
#[test]
fn naming_the_root_of_the_machine_stops_at_each_mount_point_and_says_so() {
    let counting = Instant::now();
    let holding = Holding::of(Path::new("/")).unwrap();
    let counted = counting.elapsed();
    assert_eq!(holding.tree.name, "/");
    let proc = child(&holding.tree, "proc");
    assert_eq!(proc.counted, Counted::OnAnotherFilesystem);
    assert!(proc.children.is_empty());
    assert_eq!(proc.size, 0);
    let said = proc.counted.said(&in_english()).unwrap();
    assert!(!said.is_a_bug(), "{said}");

    let nodes = every_node(&holding.tree);
    let not_finished: Vec<&&Node> = nodes
        .iter()
        .filter(|node| node.counted == Counted::NotFinished)
        .collect();
    eprintln!(
        "/ counted in {counted:?}: {} things, {} bytes, finished {}, {} folders too wide",
        nodes.len() - 1,
        holding.tree.size,
        holding.finished,
        not_finished.len()
    );
    assert_eq!(holding.finished, not_finished.is_empty());
    for wide in not_finished {
        assert_eq!(
            wide.children.len(),
            holding.most,
            "{} is cut short only because it holds more than the bound at one level",
            wide.at.display()
        );
    }
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
