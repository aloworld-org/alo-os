//! A folder kept or forgotten in hand and on the disk in one call: what
//! `Indexed::keep` and `Indexed::forget` do on the disk and the list, done
//! through the held set, and then in hand what the disk's change means.
//!
//! The plan's acceptance for task 10: *`alo-finding` offers, through the
//! held set, a folder kept and a folder forgotten in one call each, each
//! doing on the disk and the list exactly what `Indexed::keep` and
//! `Indexed::forget` do, and then in hand what the disk's change means: a
//! kept folder's index in the set at the end of the list, or in its place
//! if it was already there; a forgotten folder gone from the set, its
//! place and its entries with it, so that the next `InHand::answer` has no
//! folder for it and nothing of its words is held, checked by a test that
//! reads the set's own list of what it holds; a refusal — a folder never
//! indexed, a folder not named from the root, an index file that could not
//! be written or removed — leaves the set as it was, as it leaves the
//! disk; the list's order is unchanged by any of this; and neither call
//! reads any other folder's index file, checked by the read count the task
//! 9 test uses.* And the constraint: *nothing reads the disk that
//! `Indexed::keep` and `Indexed::forget` do not read, nothing writes what
//! they do not write; nothing ranks; the verb is unchanged; the list is
//! still not a grant; `Indexed::answer`, `Indexed::keep` and
//! `Indexed::forget` stay as they are.*
//!
//! | The acceptance | The test |
//! |---|---|
//! | a folder kept through the set: on the disk and the list as `Indexed::keep` does it, and in hand at the end of the set or in its place; no other folder's index file read | [`a_folder_kept_through_the_set_is_on_the_disk_and_at_the_end_of_the_set_or_in_its_place`] |
//! | a folder forgotten through the set: off the disk and the list as `Indexed::forget` does it, and gone from the set with its place and its words; no other folder's index file read | [`a_folder_forgotten_through_the_set_is_gone_from_the_disk_and_from_hand_with_its_words`] |
//! | every refusal leaves the set as it was, as it leaves the disk — and the one refusal in which the disk changed leaves the set holding what the disk holds | [`a_refusal_to_keep_or_forget_through_the_set_leaves_the_set_as_it_leaves_the_disk`] |
//! | the list's order is unchanged; the disk forms are unchanged; the verb is unchanged; the list is still not a grant | [`the_lists_order_is_unchanged_the_disk_forms_are_unchanged_and_the_list_is_still_not_a_grant`] |
//!
//! That nothing opens a socket, reads a clock or watches a folder is
//! `nothing_here_opens_a_socket_or_asks_anybody.rs`, which reads the
//! shipped source with the two new methods in it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Authorised, Given, Grantee, Grants, NotAuthorised};
use alo_files::{OnThisMachine, Resolving};
use alo_finding::{
    InHand, Index, Indexed, Moment, NotIndexed, Query, finding_verbs, finding_words,
};
use alo_strings::Strings;

/// A word that is in exactly one folder's files and nowhere else, so that
/// whether the set still holds that folder's words is one question.
const ONLY_THERE: &str = "quetzalcoatl";

/// A fixed moment: when the first folder's index is made.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// So many hours after noon.
fn hours_after_noon(hours: u64) -> SystemTime {
    noon() + Duration::from_secs(hours * 60 * 60)
}

/// The words this machine reads.
fn in_english() -> Strings {
    let mut vocabulary = finding_words().unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A folder of this test's own, resolved, so that what is indexed is
/// spelled the way this machine spells it.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-finding-kept-forgotten-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// A data home of this test's own, standing in for `$XDG_DATA_HOME`.
fn a_data_home(what: &str) -> PathBuf {
    a_folder_of_our_own(&format!("data-{what}"))
}

/// The list as it is on the disk under this data home.
fn the_list_under(data_home: &Path) -> Indexed {
    Indexed::read_from(Some(data_home.as_os_str()), None).unwrap()
}

/// Where the list's file is under this data home.
fn the_list_file_under(data_home: &Path) -> PathBuf {
    Indexed::where_kept(Some(data_home.as_os_str()), None).unwrap()
}

/// The sibling `keeping.rs` stages a file at before renaming it over the
/// real one; a directory put there is a write that cannot happen.
fn the_staging_sibling_of(at: &Path) -> PathBuf {
    let mut named = at.as_os_str().to_owned();
    named.push(".new");
    PathBuf::from(named)
}

/// A few known files: two whose names hold *march* and one that does not,
/// whose words hold *contract*.
fn a_library(root: &Path) {
    fs::create_dir_all(root.join("2026")).unwrap();
    fs::write(root.join("notes.txt"), b"Dear Anna, the contract.").unwrap();
    fs::write(root.join("march-notes.txt"), b"March, as it was.").unwrap();
    fs::write(
        root.join("2026").join("march.pdf"),
        b"%PDF-1.4\n1 0 obj << /Type /Catalog >> endobj\n%%EOF\n",
    )
    .unwrap();
}

/// Three folders, each a library and the middle one holding the word that
/// is only there, kept on this list in this order at noon, one and two
/// o'clock.
fn three_folders_kept(data_home: &Path) -> (Indexed, Vec<PathBuf>) {
    let folders = vec![
        a_folder_of_our_own("pictures"),
        a_folder_of_our_own("documents"),
        a_folder_of_our_own("music"),
    ];
    let mut indexed = the_list_under(data_home);
    for (hours, folder) in folders.iter().enumerate() {
        a_library(folder);
        if hours == 1 {
            fs::write(
                folder.join("feathered.txt"),
                format!("The {ONLY_THERE} of the second folder."),
            )
            .unwrap();
        }
        indexed
            .keep(&Index::of(folder, hours_after_noon(u64::try_from(hours).unwrap())).unwrap())
            .unwrap();
    }
    assert_eq!(indexed.folders(), folders.as_slice());
    (indexed, folders)
}

/// The folder at this place on the list.
fn folder_at(folders: &[PathBuf], which: usize) -> &Path {
    folders.get(which).unwrap()
}

/// The folders in this set, in order, as the set itself says.
fn held_folders(in_hand: &InHand) -> Vec<PathBuf> {
    in_hand.each().map(|(folder, _)| folder.clone()).collect()
}

/// Every index file of these folders but this one torn on the disk, so
/// that a folder read again would refuse, and one answering from hand
/// shows it was not read.
fn every_other_index_file_torn(indexed: &Indexed, folders: &[PathBuf], but: &Path) {
    for folder in folders {
        if folder.as_path() != but {
            fs::write(indexed.where_index_of(folder), b"hello\n").unwrap();
        }
    }
}

/// How many read calls this thread has made, where the kernel keeps the
/// number per thread; [`None`] on a host that does not.
fn reads_so_far() -> Option<u64> {
    let io = fs::read_to_string("/proc/thread-self/io").ok()?;
    io.lines()
        .find_map(|line| line.strip_prefix("syscr: "))
        .and_then(|count| count.trim().parse().ok())
}

/// What reading the counter itself costs, so that a count can be held to
/// exactly that and not to a guess.
fn the_counters_own_cost() -> Option<u64> {
    let once = reads_so_far()?;
    let twice = reads_so_far()?;
    Some(twice - once)
}

/// How many reads this call made, beyond what reading the counter costs;
/// [`None`] where the kernel does not count.
fn reads_during<T>(call: impl FnOnce() -> T) -> (T, Option<u64>) {
    let cost = the_counters_own_cost();
    let before = reads_so_far();
    let answer = call();
    let after = reads_so_far();
    let reads = match (cost, before, after) {
        (Some(cost), Some(before), Some(after)) => Some(after - before - cost),
        _ => None,
    };
    (answer, reads)
}

/// Everything removed, at the end.
fn cleared(data_home: &Path, folders: &[PathBuf]) {
    for folder in folders {
        let _ = fs::remove_dir_all(folder);
    }
    let _ = fs::remove_dir_all(data_home);
}

/// **A folder kept through the set is on the disk and at the end of the
/// set, or in its place if it was already there, and no other folder's
/// index file is read**: with every other index file torn after the set
/// was read, a fourth folder kept through the set is its index file on the
/// disk, the fourth folder on the list on the disk, and the fourth place
/// in the set, costing exactly the reads `Indexed::keep` costs on its own
/// and fewer than reading the set; the next answer has four folders, the
/// torn three still answering from hand. Then the second folder, indexed
/// again with a file added and kept through the set, is in its own place
/// with the fresh index, the list's order unchanged and its length too,
/// in hand and on the disk.
#[test]
fn a_folder_kept_through_the_set_is_on_the_disk_and_at_the_end_of_the_set_or_in_its_place() {
    let data_home = a_data_home("kept");
    let (indexed, folders) = three_folders_kept(&data_home);
    let (mut in_hand, reading_the_set) = reads_during(|| indexed.in_hand());
    let fourth = a_folder_of_our_own("videos");
    a_library(&fourth);
    let fourth_index = Index::of(&fourth, hours_after_noon(3)).unwrap();
    every_other_index_file_torn(&indexed, &folders, &fourth);

    let (kept, reads_through_the_set) = reads_during(|| in_hand.keep(&fourth_index));
    kept.unwrap();
    let mut on_the_list = folders.clone();
    on_the_list.push(fourth.clone());
    assert_eq!(held_folders(&in_hand), on_the_list, "at the end of the set");
    assert_eq!(in_hand.folders(), on_the_list.as_slice());
    assert_eq!(in_hand.indexed().folders(), on_the_list.as_slice());
    assert_eq!(in_hand.index_of(&fourth).unwrap(), &fourth_index, "in hand");
    assert_eq!(
        the_list_under(&data_home).folders(),
        on_the_list.as_slice(),
        "and on the list on the disk"
    );
    let on_the_disk = Index::read_from(&indexed.where_index_of(&fourth), &fourth).unwrap();
    assert_eq!(on_the_disk.entries, fourth_index.entries, "and on the disk");
    assert_eq!(on_the_disk.made, Some(Moment::of(hours_after_noon(3))));
    let (answered, reads) = reads_during(|| in_hand.answer(&Query::named("march")).unwrap());
    assert_eq!(reads, reads.map(|_| 0));
    assert_eq!(answered.answers.len(), 4);
    assert!(
        answered.every_folder_answered(),
        "the three torn on the disk still answer from hand: none was read again"
    );
    assert_eq!(
        answered
            .answers
            .iter()
            .map(|of| &of.folder)
            .collect::<Vec<_>>(),
        on_the_list.iter().collect::<Vec<_>>()
    );

    // What keeping costs on its own, on a list of its own, is what keeping
    // through the set cost: no index file read, and fewer than reading the
    // set.
    let other_home = a_data_home("kept-alone");
    let (mut other, other_folders) = three_folders_kept(&other_home);
    let fifth = a_folder_of_our_own("letters");
    a_library(&fifth);
    let fifth_index = Index::of(&fifth, hours_after_noon(3)).unwrap();
    let (kept, reads_on_its_own) = reads_during(|| other.keep(&fifth_index));
    kept.unwrap();
    if let (Some(through_the_set), Some(on_its_own), Some(reading_the_set)) =
        (reads_through_the_set, reads_on_its_own, reading_the_set)
    {
        assert_eq!(
            through_the_set, on_its_own,
            "keeping through the set reads exactly what keeping on the disk reads"
        );
        assert!(
            through_the_set < reading_the_set,
            "and not the set's index files: {through_the_set} against {reading_the_set}"
        );
    }

    // Already there: in its own place, the fresh index, and the list as
    // long as it was and in the same order.
    let second = folder_at(&folders, 1);
    fs::write(second.join("march-again.txt"), b"March, once more.").unwrap();
    let before = in_hand.index_of(second).unwrap().clone();
    let fresh = before.again(hours_after_noon(4)).unwrap();
    assert_eq!(fresh.entries.len(), before.entries.len() + 1);
    let (kept, reads) = reads_during(|| in_hand.keep(&fresh));
    kept.unwrap();
    assert_eq!(reads, reads.map(|_| 0), "no index file read");
    assert_eq!(
        held_folders(&in_hand),
        on_the_list,
        "in its place, not moved"
    );
    assert_eq!(in_hand.index_of(second).unwrap(), &fresh);
    assert_eq!(the_list_under(&data_home).folders(), on_the_list.as_slice());
    let on_the_disk = Index::read_from(&indexed.where_index_of(second), second).unwrap();
    assert_eq!(on_the_disk.entries, fresh.entries);
    assert_eq!(on_the_disk.made, Some(Moment::of(hours_after_noon(4))));
    let answered = in_hand.answer(&Query::named("march")).unwrap();
    assert_eq!(answered.answers.len(), 4);
    assert_eq!(
        answered
            .answers
            .get(1)
            .unwrap()
            .answered
            .as_ref()
            .unwrap()
            .found
            .len(),
        3
    );

    let _ = fs::remove_dir_all(&fourth);
    let _ = fs::remove_dir_all(&fifth);
    cleared(&other_home, &other_folders);
    cleared(&data_home, &folders);
}

/// **A folder forgotten through the set is gone from the disk and from
/// hand, its place and its words with it, and no other folder's index
/// file is read**: the set holds the middle folder's word before; with the
/// other two index files torn after the set was read, forgetting the
/// middle folder through the set removes its index file, takes it off the
/// list on the disk, and leaves the set with two places, in the list's
/// order, costing exactly the reads `Indexed::forget` costs on its own;
/// the next answer has no folder for it, finds its word nowhere, and the
/// set's own account of itself no longer holds the word; the other two
/// still answer from hand; and asking the set about the forgotten folder
/// is refused as never indexed.
#[test]
fn a_folder_forgotten_through_the_set_is_gone_from_the_disk_and_from_hand_with_its_words() {
    let data_home = a_data_home("forgotten");
    let (indexed, folders) = three_folders_kept(&data_home);
    let (mut in_hand, reading_the_set) = reads_during(|| indexed.in_hand());
    let middle = folder_at(&folders, 1);
    let middle_file = indexed.where_index_of(middle);
    let before = in_hand.answer(&Query::saying(ONLY_THERE)).unwrap();
    assert_eq!(before.answers.len(), 3);
    assert_eq!(
        before
            .answers
            .get(1)
            .unwrap()
            .answered
            .as_ref()
            .unwrap()
            .found
            .len(),
        1,
        "the word is held, in the middle folder's place"
    );
    assert!(format!("{in_hand:?}").contains(ONLY_THERE));
    every_other_index_file_torn(&indexed, &folders, middle);

    let (forgotten, reads_through_the_set) = reads_during(|| in_hand.forget(middle));
    forgotten.unwrap();
    let remaining = vec![
        folder_at(&folders, 0).to_path_buf(),
        folder_at(&folders, 2).to_path_buf(),
    ];
    assert_eq!(
        held_folders(&in_hand),
        remaining,
        "its place gone from the set"
    );
    assert_eq!(in_hand.folders(), remaining.as_slice());
    assert_eq!(in_hand.each().count(), 2);
    assert!(
        !format!("{in_hand:?}").contains(ONLY_THERE),
        "nothing of its words is held"
    );
    assert!(matches!(
        in_hand.index_of(middle).unwrap_err(),
        NotIndexed::NeverIndexed { at } if at == middle
    ));
    assert!(!middle_file.exists(), "its index file is off the disk");
    assert_eq!(
        the_list_under(&data_home).folders(),
        remaining.as_slice(),
        "and it is off the list on the disk"
    );

    let (after, reads) = reads_during(|| in_hand.answer(&Query::saying(ONLY_THERE)).unwrap());
    assert_eq!(reads, reads.map(|_| 0));
    assert_eq!(after.answers.len(), 2, "no folder for it");
    assert!(
        after.every_folder_answered(),
        "the other two, torn on the disk, still answer from hand: neither was read"
    );
    assert!(after.answers.iter().all(|of| of.folder != middle));
    assert!(
        after.answered().all(|(_, held)| held.found.is_empty()),
        "the word is found nowhere"
    );

    // What forgetting costs on its own, on a list of its own, is what
    // forgetting through the set cost: no index file read.
    let other_home = a_data_home("forgotten-alone");
    let (mut other, other_folders) = three_folders_kept(&other_home);
    let (forgotten, reads_on_its_own) = reads_during(|| other.forget(folder_at(&other_folders, 1)));
    forgotten.unwrap();
    if let (Some(through_the_set), Some(on_its_own), Some(reading_the_set)) =
        (reads_through_the_set, reads_on_its_own, reading_the_set)
    {
        assert_eq!(
            through_the_set, on_its_own,
            "forgetting through the set reads exactly what forgetting on the disk reads"
        );
        assert!(
            through_the_set < reading_the_set,
            "and not the set's index files: {through_the_set} against {reading_the_set}"
        );
    }

    // Forgotten, it can be kept again, and comes back at the end.
    a_library(middle);
    in_hand
        .keep(&Index::of(middle, hours_after_noon(5)).unwrap())
        .unwrap();
    let mut at_the_end = remaining.clone();
    at_the_end.push(middle.to_path_buf());
    assert_eq!(held_folders(&in_hand), at_the_end);
    assert_eq!(the_list_under(&data_home).folders(), at_the_end.as_slice());

    cleared(&other_home, &other_folders);
    cleared(&data_home, &folders);
}

/// **A refusal to keep or forget through the set leaves the set as it was,
/// as it leaves the disk**: a folder never indexed and one not named from
/// the root are refused by `forget` with no file read; an index of a
/// folder not named from the root is refused by `keep`; an index file that
/// cannot be removed refuses `forget`, naming it, with the folder still
/// held and still on the list; an index file that cannot be written
/// refuses `keep`, with no place added and the list as it was; a list that
/// cannot be written after a new folder's index was refuses `keep`, with
/// no place added, the list as it was, and the index written being
/// nothing's. And the one refusal in which the disk changed — the index
/// file removed and then the list not written — leaves the set holding for
/// that folder what the disk holds: the folder still named, no index, and
/// none of its words; the list mended, forgetting again finishes the job.
#[test]
fn a_refusal_to_keep_or_forget_through_the_set_leaves_the_set_as_it_leaves_the_disk() {
    let data_home = a_data_home("refused");
    let (indexed, folders) = three_folders_kept(&data_home);
    let strings = in_english();
    let list_file = the_list_file_under(&data_home);
    let list_on_the_disk = fs::read_to_string(&list_file).unwrap();
    let mut in_hand = indexed.in_hand();
    let as_it_was = in_hand.clone();
    let never = a_folder_of_our_own("never");
    a_library(&never);

    // Never indexed, and not named from the root: refused with no file
    // read, and nothing changed.
    let (refused, reads) = reads_during(|| in_hand.forget(&never).unwrap_err());
    assert!(matches!(&refused, NotIndexed::NeverIndexed { at } if at == &never));
    assert_eq!(reads, reads.map(|_| 0));
    assert!(!refused.said(&strings).is_a_bug());
    assert!(matches!(
        in_hand.forget(Path::new("Documents")).unwrap_err(),
        NotIndexed::NotAbsolute { at } if at == Path::new("Documents")
    ));
    let mut sideways = Index::of(&never, hours_after_noon(3)).unwrap();
    sideways.of = PathBuf::from("Documents");
    let (refused, reads) = reads_during(|| in_hand.keep(&sideways).unwrap_err());
    assert!(matches!(&refused, NotIndexed::NotAbsolute { at } if at == Path::new("Documents")));
    assert_eq!(reads, reads.map(|_| 0));
    assert_eq!(in_hand, as_it_was);
    assert_eq!(fs::read_to_string(&list_file).unwrap(), list_on_the_disk);

    // An index file that cannot be removed: a directory stands where it
    // was. The folder is still held, still answers, and is still listed.
    let third = folder_at(&folders, 2);
    let third_file = indexed.where_index_of(third);
    let third_index = indexed.index_of(third).unwrap();
    fs::remove_file(&third_file).unwrap();
    fs::create_dir(&third_file).unwrap();
    let refused = in_hand.forget(third).unwrap_err();
    assert!(
        matches!(&refused, NotIndexed::NotRemoved { at, .. } if at == &third_file),
        "{refused}"
    );
    assert!(
        refused
            .said(&strings)
            .into_text()
            .contains(&third_file.to_string_lossy().into_owned())
    );
    assert_eq!(in_hand, as_it_was);
    assert_eq!(in_hand.index_of(third).unwrap(), &third_index);
    assert_eq!(fs::read_to_string(&list_file).unwrap(), list_on_the_disk);
    fs::remove_dir(&third_file).unwrap();
    third_index.kept_at(&third_file).unwrap();

    // An index file that cannot be written: a directory stands where it
    // would be staged. No place is added, and the list is as it was.
    let fourth = a_folder_of_our_own("videos");
    a_library(&fourth);
    let fourth_index = Index::of(&fourth, hours_after_noon(3)).unwrap();
    let fourth_file = indexed.where_index_of(&fourth);
    fs::create_dir_all(the_staging_sibling_of(&fourth_file)).unwrap();
    let refused = in_hand.keep(&fourth_index).unwrap_err();
    assert!(
        matches!(&refused, NotIndexed::NotKept { at, .. } if at == &fourth_file),
        "{refused}"
    );
    assert_eq!(in_hand, as_it_was);
    assert!(!fourth_file.exists());
    assert_eq!(fs::read_to_string(&list_file).unwrap(), list_on_the_disk);
    fs::remove_dir(the_staging_sibling_of(&fourth_file)).unwrap();

    // The list cannot be written after a new folder's index was: no place
    // is added, the list is as it was, and the index on the disk is
    // nothing's — never indexed, on the disk and in hand alike.
    fs::create_dir_all(the_staging_sibling_of(&list_file)).unwrap();
    let refused = in_hand.keep(&fourth_index).unwrap_err();
    assert!(
        matches!(&refused, NotIndexed::ListNotKept { at, .. } if at == &list_file),
        "{refused}"
    );
    assert_eq!(in_hand, as_it_was);
    assert!(fourth_file.exists(), "the index was written first");
    assert_eq!(fs::read_to_string(&list_file).unwrap(), list_on_the_disk);
    assert!(matches!(
        the_list_under(&data_home).index_of(&fourth).unwrap_err(),
        NotIndexed::NeverIndexed { .. }
    ));
    assert!(matches!(
        in_hand.index_of(&fourth).unwrap_err(),
        NotIndexed::NeverIndexed { .. }
    ));
    assert_eq!(held_folders(&in_hand), folders);

    // The list cannot be written after the middle folder's index file was
    // removed: the disk changed, and the set holds what the disk holds —
    // the folder still named, its index not there, and none of its words.
    let middle = folder_at(&folders, 1);
    let middle_file = indexed.where_index_of(middle);
    assert!(format!("{in_hand:?}").contains(ONLY_THERE));
    let refused = in_hand.forget(middle).unwrap_err();
    assert!(
        matches!(&refused, NotIndexed::ListNotKept { at, .. } if at == &list_file),
        "{refused}"
    );
    assert!(!middle_file.exists(), "the file went first");
    assert_eq!(fs::read_to_string(&list_file).unwrap(), list_on_the_disk);
    assert_eq!(
        held_folders(&in_hand),
        folders,
        "still named, as on the list"
    );
    assert!(
        !format!("{in_hand:?}").contains(ONLY_THERE),
        "nothing of its words is held"
    );
    let on_the_disk = the_list_under(&data_home).index_of(middle).unwrap_err();
    assert!(
        matches!(&on_the_disk, NotIndexed::NotOpened { at, .. } if at == &middle_file),
        "{on_the_disk}"
    );
    assert_eq!(in_hand.index_of(middle).unwrap_err(), on_the_disk);
    let answered = in_hand.answer(&Query::saying(ONLY_THERE)).unwrap();
    assert_eq!(answered.answers.len(), 3);
    assert_eq!(answered.answered().count(), 2);
    assert!(matches!(
        answered.refused().next().unwrap(),
        (at, NotIndexed::NotOpened { .. }) if at == middle
    ));
    assert_ne!(in_hand, as_it_was);

    // The list mended, forgetting again finishes the job.
    fs::remove_dir(the_staging_sibling_of(&list_file)).unwrap();
    in_hand.forget(middle).unwrap();
    let remaining = vec![folder_at(&folders, 0).to_path_buf(), third.to_path_buf()];
    assert_eq!(held_folders(&in_hand), remaining);
    assert_eq!(the_list_under(&data_home).folders(), remaining.as_slice());

    let _ = fs::remove_dir_all(&never);
    let _ = fs::remove_dir_all(&fourth);
    cleared(&data_home, &folders);
}

/// **The list's order is unchanged by any of this, the disk forms are
/// unchanged, the verb is unchanged, and the list is still not a grant**:
/// a folder kept again stays where it was and one forgotten and kept
/// again goes to the end, in hand and on the disk, the same as through
/// `Indexed` alone; `Indexed::keep`, `Indexed::forget` and
/// `Indexed::answer` still work with nothing in hand, the last reading the
/// index files on every call; with no grant at all the set keeps, forgets
/// and answers while an agent's `search_files` over one of its folders is
/// refused at the door; and the two new shapes are pinned by assignment.
#[test]
fn the_lists_order_is_unchanged_the_disk_forms_are_unchanged_and_the_list_is_still_not_a_grant() {
    let data_home = a_data_home("order");
    let (indexed, folders) = three_folders_kept(&data_home);
    let mut in_hand = indexed.in_hand();
    let first = folder_at(&folders, 0);
    let second = folder_at(&folders, 1);

    // Kept again: in its place. Forgotten and kept again: at the end.
    in_hand
        .keep(&Index::of(first, hours_after_noon(3)).unwrap())
        .unwrap();
    assert_eq!(held_folders(&in_hand), folders);
    in_hand.forget(second).unwrap();
    in_hand
        .keep(&Index::of(second, hours_after_noon(4)).unwrap())
        .unwrap();
    let reordered = vec![
        first.to_path_buf(),
        folder_at(&folders, 2).to_path_buf(),
        second.to_path_buf(),
    ];
    assert_eq!(held_folders(&in_hand), reordered);
    assert_eq!(the_list_under(&data_home).folders(), reordered.as_slice());
    let answered = in_hand.answer(&Query::named("march")).unwrap();
    assert_eq!(
        answered
            .answers
            .iter()
            .map(|of| &of.folder)
            .collect::<Vec<_>>(),
        reordered.iter().collect::<Vec<_>>()
    );

    // The disk forms, with nothing in hand, do the same thing in the same
    // order — and `Indexed::answer` reads the files on every call.
    let mut disk = the_list_under(&data_home);
    disk.forget(first).unwrap();
    disk.keep(&Index::of(first, hours_after_noon(5)).unwrap())
        .unwrap();
    let disk_order = vec![
        folder_at(&folders, 2).to_path_buf(),
        second.to_path_buf(),
        first.to_path_buf(),
    ];
    assert_eq!(disk.folders(), disk_order.as_slice());
    assert_eq!(the_list_under(&data_home).folders(), disk_order.as_slice());
    for _ in 0..2 {
        let (answered, reads) = reads_during(|| disk.answer(&Query::named("march")).unwrap());
        assert!(answered.every_folder_answered());
        if let Some(reads) = reads {
            assert!(reads >= 3, "three index files read on every call: {reads}");
        }
    }
    assert_eq!(
        held_folders(&in_hand),
        reordered,
        "the set is the list as it was when it was read, until the caller reads again"
    );

    // No grant, and the set keeps, forgets and answers; an agent over one
    // of its folders is refused at the door.
    assert_eq!(
        in_hand
            .answer(&Query::named("march"))
            .unwrap()
            .answered()
            .count(),
        3
    );
    let call = finding_verbs()
        .unwrap()
        .call(
            "search_files",
            &[
                ("folder", Given::text(second.to_string_lossy().into_owned())),
                ("named", Given::text("march")),
            ],
        )
        .unwrap();
    let refused = Authorised::read(
        &call,
        &Grantee::named("@finding"),
        &Grants::default(),
        noon(),
    )
    .unwrap_err();
    assert!(
        matches!(refused.why(), NotAuthorised::NotGranted(_)),
        "{:?}",
        refused.why()
    );

    fn kept_through_the_set(in_hand: &mut InHand, index: &Index) -> Result<(), NotIndexed> {
        in_hand.keep(index)
    }
    fn forgotten_through_the_set(in_hand: &mut InHand, folder: &Path) -> Result<(), NotIndexed> {
        in_hand.forget(folder)
    }
    fn kept_on_the_disk(indexed: &mut Indexed, index: &Index) -> Result<(), NotIndexed> {
        indexed.keep(index)
    }
    fn forgotten_on_the_disk(indexed: &mut Indexed, folder: &Path) -> Result<(), NotIndexed> {
        indexed.forget(folder)
    }
    let _ = (
        kept_through_the_set,
        forgotten_through_the_set,
        kept_on_the_disk,
        forgotten_on_the_disk,
    );

    cleared(&data_home, &folders);
}
