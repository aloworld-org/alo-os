//! A search over every indexed folder, timed: the index files read on every
//! query beside the indexes read once and held — against the real disk, on
//! this machine.
//!
//! The plan's acceptance for task 9's measurement: *a test builds three
//! indexes of ten thousand files, times `Indexed::answer` by name and by
//! contents, and the numbers are in the report with the machine named, as
//! task 4's are; the held form is timed beside the file-reading one, and
//! both numbers are published.* The numbers each run measures are printed,
//! so that the report can publish them with the machine named — a number
//! with no machine beside it is a claim.
//!
//! | The promise | The test |
//! |---|---|
//! | three indexes of ten thousand files, `Indexed::answer` timed by name and by contents, and the held form timed beside it | [`a_search_over_three_folders_of_ten_thousand_files_is_timed_from_the_disk_and_from_hand`] |
//!
//! # What is held, and what is only printed
//!
//! The held form answers from memory, so it is held to the bounds task 4
//! holds one folder to, three times over: by name under three tenths of a
//! second, by contents under three seconds. The file-reading form is held
//! to no bound of its own — its cost is the thing being measured, and a
//! bound nobody measured first is a claim — but the best of its rounds may
//! not be faster than the best of the held form's, because a set held in
//! hand that was slower than reading the files would be a set nobody
//! should hold. Both forms find the same files.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant, SystemTime};

use alo_finding::{Everywhere, InHand, Index, Indexed, Query};

/// A folder of this test's own, under this machine's temporary directory,
/// named after the test so a leftover says which test left it.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-finding-timed-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    folder.canonicalize().unwrap()
}

/// How many files each timed index holds.
const TEN_THOUSAND: usize = 10_000;

/// How many of them are PDFs, which have a kind and no words.
const PDFS: usize = 1_000;

/// How many folders are indexed.
const FOLDERS: usize = 3;

/// A fixed moment for every index this test makes.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// Ten thousand files in a hundred folders, as task 4's test writes them:
/// nine thousand letters of twenty-odd words each, and a thousand PDFs.
/// Every hundredth letter says *contract*, and every letter names Anna.
fn ten_thousand_files(root: &Path) {
    for n in 0..TEN_THOUSAND {
        let folder = root.join(format!("folder-{:02}", n % 100));
        if n < 100 {
            fs::create_dir_all(&folder).unwrap();
        }
        if n < PDFS {
            fs::write(
                folder.join(format!("scan-{n:05}.pdf")),
                b"%PDF-1.4\n1 0 obj << /Type /Catalog >> endobj\n%%EOF\n",
            )
            .unwrap();
        } else {
            let subject = if n % 100 == 0 { "contract" } else { "invoice" };
            fs::write(
                folder.join(format!("letter-{n:05}.txt")),
                format!(
                    "Dear Anna, here is the {subject} number {n} that we discussed on the phone \
                     last week, with the figures for the summer and a note about the garden, \
                     the roof and the lease."
                ),
            )
            .unwrap();
        }
    }
}

/// How many entries one search over every folder found, and that every
/// folder answered.
fn found_everywhere(everywhere: &Everywhere) -> usize {
    assert!(everywhere.every_folder_answered());
    assert_eq!(everywhere.answers.len(), FOLDERS);
    everywhere
        .answered()
        .map(|(_, held)| held.found.len())
        .sum()
}

/// Everything two searches found, folder by folder, so that the two forms
/// can be held to finding the same.
fn what_was_found(everywhere: &Everywhere) -> Vec<(PathBuf, Vec<String>)> {
    everywhere
        .answered()
        .map(|(folder, held)| {
            (
                folder.clone(),
                held.found.iter().map(|entry| entry.below.clone()).collect(),
            )
        })
        .collect()
}

/// One timed search over every folder, whichever way it is asked: how long
/// it took by this test's clock around the call, printed, with what it
/// found held to the number expected.
fn timed(
    how: &str,
    what: &str,
    ask: impl FnOnce() -> Everywhere,
    finds: usize,
) -> (Duration, Everywhere) {
    let started = Instant::now();
    let everywhere = ask();
    let took = started.elapsed();
    let found = found_everywhere(&everywhere);
    println!("{how}, {what}: {found} found over {FOLDERS} folders in {took:?}");
    assert_eq!(found, finds, "{how}, {what}");
    (took, everywhere)
}

/// **A search over three folders of ten thousand files is timed from the
/// disk and from hand**: three indexes built from thirty thousand real
/// files and kept on one list; `Indexed::answer` timed by name and by
/// contents, three rounds, round one cold; the set read into hand, timed;
/// and `InHand::answer` timed the same way. Both forms find the same files,
/// the held form answers inside three times task 4's bounds, and the best
/// held round is no slower than the best disk round.
#[test]
fn a_search_over_three_folders_of_ten_thousand_files_is_timed_from_the_disk_and_from_hand() {
    let data_home = a_folder_of_our_own("data");
    let mut indexed = Indexed::read_from(Some(data_home.as_os_str()), None).unwrap();
    let mut folders = Vec::new();
    for which in 0..FOLDERS {
        let root = a_folder_of_our_own(&format!("ten-thousand-{which}"));
        let writing = Instant::now();
        ten_thousand_files(&root);
        let indexing = Instant::now();
        let index = Index::of(&root, noon()).unwrap();
        let keeping = Instant::now();
        indexed.keep(&index).unwrap();
        println!(
            "folder {which}: ten thousand files written in {:?}, indexed in {:?}, kept in {:?}",
            indexing.duration_since(writing),
            keeping.duration_since(indexing),
            keeping.elapsed()
        );
        assert_eq!(index.entries.len(), TEN_THOUSAND + 100);
        folders.push(root);
    }
    assert_eq!(indexed.folders(), folders.as_slice());

    let by_name = Query::named("letter-0999");
    let by_contents = Query::saying("Anna contract summer");
    let names = FOLDERS * 10;
    let contents = FOLDERS * (TEN_THOUSAND - PDFS) / 100;
    let a_tenth_three_times = Duration::from_millis(300);
    let a_second_three_times = Duration::from_secs(3);

    let mut disk_by_name = Vec::new();
    let mut disk_by_contents = Vec::new();
    let mut found_by_name = None;
    let mut found_by_contents = None;
    for round in 1..=3 {
        println!("round {round}, from the disk");
        let (took, everywhere) = timed(
            "from the disk",
            "by name",
            || indexed.answer(&by_name).unwrap(),
            names,
        );
        disk_by_name.push(took);
        found_by_name = Some(what_was_found(&everywhere));
        let (took, everywhere) = timed(
            "from the disk",
            "by contents",
            || indexed.answer(&by_contents).unwrap(),
            contents,
        );
        disk_by_contents.push(took);
        found_by_contents = Some(what_was_found(&everywhere));
    }

    let reading = Instant::now();
    let in_hand: InHand = indexed.in_hand();
    println!(
        "the three indexes read into hand in {:?}",
        reading.elapsed()
    );
    assert_eq!(in_hand.folders(), folders.as_slice());

    let mut hand_by_name = Vec::new();
    let mut hand_by_contents = Vec::new();
    for round in 1..=3 {
        println!("round {round}, from hand");
        let (took, everywhere) = timed(
            "from hand",
            "by name",
            || in_hand.answer(&by_name).unwrap(),
            names,
        );
        assert!(
            took < a_tenth_three_times,
            "from hand, by name, took {took:?}; the bound is {a_tenth_three_times:?}"
        );
        hand_by_name.push(took);
        assert_eq!(
            Some(what_was_found(&everywhere)),
            found_by_name,
            "the same files"
        );
        let (took, everywhere) = timed(
            "from hand",
            "by contents",
            || in_hand.answer(&by_contents).unwrap(),
            contents,
        );
        assert!(
            took < a_second_three_times,
            "from hand, by contents, took {took:?}; the bound is {a_second_three_times:?}"
        );
        hand_by_contents.push(took);
        assert_eq!(
            Some(what_was_found(&everywhere)),
            found_by_contents,
            "the same files"
        );
    }

    let best = |rounds: &[Duration]| rounds.iter().copied().min().unwrap();
    println!(
        "best rounds — from the disk: by name {:?}, by contents {:?}; from hand: by name {:?}, \
         by contents {:?}",
        best(&disk_by_name),
        best(&disk_by_contents),
        best(&hand_by_name),
        best(&hand_by_contents)
    );
    assert!(
        best(&hand_by_name) <= best(&disk_by_name),
        "a set held in hand answers by name no slower than reading the files"
    );
    assert!(
        best(&hand_by_contents) <= best(&disk_by_contents),
        "a set held in hand answers by contents no slower than reading the files"
    );

    for folder in &folders {
        let _ = fs::remove_dir_all(folder);
    }
    let _ = fs::remove_dir_all(&data_home);
}
