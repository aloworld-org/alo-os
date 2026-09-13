//! An index brought up to date by its name, and an answer that says how old
//! it is.
//!
//! The plan's acceptance for task 7: *`alo-finding` records in the index's
//! first line the moment the index was made, passed in by the caller rather
//! than read from a clock inside the crate, so a test can make an index at
//! noon and read noon back; the moment is in the file additively, and an
//! index file without it still reads; an `Answer` carries the moment the
//! index it answered from was made; and `Indexed::again` takes a folder's
//! name and a moment, reads the kept index, indexes again reading only files
//! whose size or time changed, and keeps the result whole, in one call. A
//! folder never indexed is refused with `NeverIndexed` and is not indexed
//! for the first time; a folder that is gone since it was indexed is refused
//! with `NotWalked`, and the index it had is kept.* And the constraint:
//! *nothing here watches a folder, and nothing here reads a clock.*
//!
//! | The acceptance | The test |
//! |---|---|
//! | an index made at noon says noon, in its first line and in every answer from it | [`an_index_made_at_noon_says_noon_in_its_first_line_and_in_every_answer`] |
//! | an index file written before the moment was kept still reads, and answers with no moment | [`an_index_file_without_the_moment_still_reads_and_answers_with_none`] |
//! | brought up to date by its name, reading only what changed, kept whole, in one call | [`an_index_is_brought_up_to_date_by_its_name_reading_only_what_changed_and_kept_whole`] |
//! | a folder never indexed is refused and is not indexed for the first time | [`a_folder_never_indexed_is_refused_and_is_not_indexed_for_the_first_time`] |
//! | a folder gone since it was indexed is refused, and its index is kept | [`a_folder_gone_since_it_was_indexed_is_refused_and_its_index_is_kept`] |
//! | the moment is the caller's, and `Searched::of` is unchanged | [`the_moment_is_the_callers_and_the_door_is_unchanged`] |
//!
//! What is deliberately not tested here: that nothing watches. A test cannot
//! wait for something that must never happen; instead
//! `nothing_here_opens_a_socket_or_asks_anybody.rs` reads the shipped source
//! and refuses every name that would let a watcher, a thread or a timer in,
//! and holds the one `now` to the stopwatch around a search.

#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_files::{OnThisMachine, Resolving, Touching};
use alo_finding::{Index, Indexed, Moment, NotIndexed, Query, Searched, finding_words};
use alo_strings::Strings;

/// A fixed moment: when the first index in each test is made.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A fixed moment an hour after noon: when an index is brought up to date.
fn one_oclock() -> SystemTime {
    noon() + Duration::from_secs(60 * 60)
}

/// The words this machine reads.
fn in_english() -> Strings {
    let mut vocabulary = finding_words().unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A folder of this test's own, resolved, so that what is indexed is spelled
/// the way this machine spells it.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-finding-again-{}-{what}-{}",
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

/// A few known files: two text files and a PDF in a subfolder.
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

/// The entries of an index that are files. Two walks of one folder are
/// compared by these: a folder's own time settles a moment after a write
/// inside it on NTFS, and a folder has no contents to vouch for.
fn files_of(index: &Index) -> Vec<&alo_finding::Entry> {
    index
        .entries
        .iter()
        .filter(|entry| entry.kind.is_a_file())
        .collect()
}

/// The first line of the file this index is kept in.
fn first_line_of(at: &Path) -> String {
    fs::read_to_string(at)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_owned()
}

/// How many read calls this thread has made, where the kernel keeps the
/// number per thread; [`None`] on a host that does not.
fn reads_so_far() -> Option<u64> {
    let io = fs::read_to_string("/proc/thread-self/io").ok()?;
    io.lines()
        .find_map(|line| line.strip_prefix("syscr: "))
        .and_then(|count| count.trim().parse().ok())
}

/// **An index made at noon says noon**: in the index in hand, in the first
/// line of its file as the contract spells it, in the index read back by a
/// second reader of the same data home, and beside every answer from it —
/// so a window can say *as of noon* next to the results.
#[test]
fn an_index_made_at_noon_says_noon_in_its_first_line_and_in_every_answer() {
    let data_home = a_data_home("noon");
    let documents = a_folder_of_our_own("documents");
    a_library(&documents);
    let at_noon = Moment::of(noon());

    let index = Index::of(&documents, noon()).unwrap();
    assert_eq!(index.made, Some(at_noon));

    let mut indexed = the_list_under(&data_home);
    indexed.keep(&index).unwrap();
    let first = first_line_of(&indexed.where_index_of(&documents));
    assert!(
        first.contains(r#""made":{"secs":1760000000,"nanos":0}"#),
        "{first}"
    );
    assert!(first.starts_with(r#"{"format":1,"of":"#), "{first}");

    let back = the_list_under(&data_home).index_of(&documents).unwrap();
    assert_eq!(back.made, Some(at_noon), "read back as it was said");
    assert_eq!(back.made.unwrap().as_time(), noon());

    for index in [&index, &back] {
        let by_name = index.answer(&Query::named("notes")).unwrap();
        assert_eq!(by_name.found.len(), 2);
        assert_eq!(by_name.made, Some(at_noon), "an answer says how old it is");
        let by_words = index.answer(&Query::saying("contract")).unwrap();
        assert_eq!(by_words.made, Some(at_noon));
        let nothing = index
            .answer(&Query::named("nothing-is-called-this"))
            .unwrap();
        assert!(nothing.found.is_empty());
        assert_eq!(
            nothing.made,
            Some(at_noon),
            "an empty answer still says when it is from"
        );
    }

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&data_home);
}

/// **An index file written before the moment was kept still reads**, and an
/// answer from it says it has no moment rather than inventing one. The file
/// is an index of this version with the field taken out by hand, which is
/// exactly the file an earlier version wrote; and `opened` is zero, because
/// nothing was walked to fill the gap in.
#[test]
fn an_index_file_without_the_moment_still_reads_and_answers_with_none() {
    let data_home = a_data_home("earlier");
    let documents = a_folder_of_our_own("documents");
    a_library(&documents);

    let mut indexed = the_list_under(&data_home);
    indexed
        .keep(&Index::of(&documents, noon()).unwrap())
        .unwrap();
    let at = indexed.where_index_of(&documents);
    let text = fs::read_to_string(&at).unwrap();
    let earlier = text.replacen(r#""made":{"secs":1760000000,"nanos":0},"#, "", 1);
    assert_ne!(earlier, text, "the moment was in the file to take out");
    assert!(!earlier.contains("made"));
    fs::write(&at, earlier).unwrap();

    let back = the_list_under(&data_home).index_of(&documents).unwrap();
    assert_eq!(back.made, None);
    assert_eq!(back.opened, 0);
    assert_eq!(back.entries.len(), 4, "a folder and three files, all read");
    let answer = back.answer(&Query::saying("contract")).unwrap();
    assert_eq!(answer.found.len(), 1);
    assert_eq!(answer.made, None, "no moment rather than an invented one");

    // Brought up to date, it gains the moment the caller names.
    let fresh = indexed.again(&documents, one_oclock()).unwrap();
    assert_eq!(fresh.made, Some(Moment::of(one_oclock())));
    assert!(
        first_line_of(&at).contains(r#""made":{"secs":1760003600,"nanos":0}"#),
        "{}",
        first_line_of(&at)
    );

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&data_home);
}

/// **An index is brought up to date by its folder's name, in one call**: the
/// kept index is read, the folder is indexed again reading only the files
/// whose size or time changed — counted by `Index::opened` — and the result
/// is kept whole, so that a second reader of the data home sees the new
/// moment and the new file. Brought up to date again with nothing changed,
/// nothing is read, and the moment is still the caller's.
#[test]
fn an_index_is_brought_up_to_date_by_its_name_reading_only_what_changed_and_kept_whole() {
    let data_home = a_data_home("again");
    let documents = a_folder_of_our_own("documents");
    a_library(&documents);

    let mut indexed = the_list_under(&data_home);
    let first = Index::of(&documents, noon()).unwrap();
    assert_eq!(first.opened, 3, "every file read the first time");
    indexed.keep(&first).unwrap();

    fs::write(
        documents.join("notes.txt"),
        b"Dear Anna, the contract is signed.",
    )
    .unwrap();
    fs::write(documents.join("2026").join("april.txt"), b"new in April").unwrap();

    let fresh = indexed.again(&documents, one_oclock()).unwrap();
    assert_eq!(fresh.of, documents);
    assert_eq!(fresh.made, Some(Moment::of(one_oclock())));
    assert_eq!(
        fresh.opened, 2,
        "the rewritten file and the new one, and no other"
    );
    assert_eq!(fresh.entries.len(), 5, "a folder and four files");
    assert_eq!(
        fresh.answer(&Query::saying("signed")).unwrap().found.len(),
        1
    );
    assert_eq!(
        fresh.answer(&Query::saying("april")).unwrap().found.len(),
        1
    );

    // Kept whole: a second reader of the same data home reads the fresh
    // index, with its moment, and the list is as it was.
    let second_reader = the_list_under(&data_home);
    assert_eq!(second_reader.folders(), std::slice::from_ref(&documents));
    let back = second_reader.index_of(&documents).unwrap();
    assert_eq!(back.made, fresh.made);
    assert_eq!(back.entries, fresh.entries);
    let answer = back.answer(&Query::named("april")).unwrap();
    assert_eq!(answer.found.len(), 1);
    assert_eq!(answer.made, Some(Moment::of(one_oclock())));

    // Nothing changed: nothing read, and the moment is whatever the caller
    // says — here, earlier than the last one, because nothing here looks at
    // a clock to disagree.
    let again = indexed.again(&documents, noon()).unwrap();
    assert_eq!(again.opened, 0);
    assert_eq!(again.made, Some(Moment::of(noon())));
    assert_eq!(files_of(&again), files_of(&fresh), "the words were kept");
    assert_eq!(again.entries.len(), fresh.entries.len());
    assert_eq!(
        the_list_under(&data_home)
            .index_of(&documents)
            .unwrap()
            .made,
        Some(Moment::of(noon()))
    );

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&data_home);
}

/// **A folder never indexed is refused, in words, and is not indexed for
/// the first time** by a call meant to refresh one: no index file appears,
/// the list is as it was, and on a host whose kernel counts reads per
/// thread, the folder's files were not read. A folder not named from the
/// root is refused before the list is asked.
#[test]
fn a_folder_never_indexed_is_refused_and_is_not_indexed_for_the_first_time() {
    let data_home = a_data_home("never");
    let documents = a_folder_of_our_own("documents");
    let pictures = a_folder_of_our_own("pictures");
    a_library(&documents);
    for n in 0..50 {
        fs::write(pictures.join(format!("{n}.txt")), b"a picture, in words").unwrap();
    }

    let mut indexed = the_list_under(&data_home);
    indexed
        .keep(&Index::of(&documents, noon()).unwrap())
        .unwrap();
    let folders_before = indexed.folders().to_vec();

    // What reading the counter itself costs, so the answer is held to
    // exactly that and not to a guess.
    let counted_once = reads_so_far();
    let counted_twice = reads_so_far();
    let before = reads_so_far();
    let refused = indexed.again(&pictures, one_oclock()).unwrap_err();
    let after = reads_so_far();
    match &refused {
        NotIndexed::NeverIndexed { at } => assert_eq!(at, &pictures),
        other => panic!("{other:?}"),
    }
    if let (Some(once), Some(twice), Some(before), Some(after)) =
        (counted_once, counted_twice, before, after)
    {
        assert_eq!(
            after - before,
            twice - once,
            "no file was read while refusing a folder never indexed: the count moved by what              reading it costs, and fifty files would be fifty more"
        );
    }
    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert_eq!(
        said.text(),
        format!(
            "{} was never indexed, so there is no index of it to search.",
            pictures.display()
        )
    );

    assert!(
        !indexed.where_index_of(&pictures).exists(),
        "no index was made for the first time"
    );
    assert_eq!(indexed.folders(), folders_before);
    assert_eq!(the_list_under(&data_home).folders(), folders_before);
    assert!(matches!(
        the_list_under(&data_home).index_of(&pictures),
        Err(NotIndexed::NeverIndexed { .. })
    ));

    assert!(matches!(
        indexed.again(Path::new("Pictures"), one_oclock()),
        Err(NotIndexed::NotAbsolute { .. })
    ));

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&pictures);
    let _ = fs::remove_dir_all(&data_home);
}

/// **A folder that is gone since it was indexed is refused, and its index is
/// kept**: the file is still there, the list still names the folder, the
/// index still answers about the folder as it was, and it still says when
/// that was. An unplugged disk is not a request to forget it.
#[test]
fn a_folder_gone_since_it_was_indexed_is_refused_and_its_index_is_kept() {
    let data_home = a_data_home("gone");
    let documents = a_folder_of_our_own("documents");
    a_library(&documents);

    let mut indexed = the_list_under(&data_home);
    let first = Index::of(&documents, noon()).unwrap();
    indexed.keep(&first).unwrap();
    let at = indexed.where_index_of(&documents);
    let file_before = fs::read_to_string(&at).unwrap();

    fs::remove_dir_all(&documents).unwrap();
    assert!(!documents.exists());

    match indexed.again(&documents, one_oclock()) {
        Err(NotIndexed::NotWalked { at: folder, .. }) => assert_eq!(folder, documents),
        other => panic!("{other:?}"),
    }

    assert!(at.exists(), "the index is kept");
    assert_eq!(
        fs::read_to_string(&at).unwrap(),
        file_before,
        "byte for byte"
    );
    assert!(indexed.holds(&documents));
    assert_eq!(
        the_list_under(&data_home).folders(),
        std::slice::from_ref(&documents)
    );
    let kept = the_list_under(&data_home).index_of(&documents).unwrap();
    assert_eq!(kept.entries, first.entries);
    assert_eq!(kept.made, Some(Moment::of(noon())), "and it says when");
    let answer = kept.answer(&Query::saying("contract")).unwrap();
    assert_eq!(answer.found.len(), 1);
    assert_eq!(answer.made, Some(Moment::of(noon())));

    // Forgetting it is a separate request, and still works.
    indexed.forget(&documents).unwrap();
    assert!(!at.exists());

    let _ = fs::remove_dir_all(&data_home);
}

/// **The moment is the caller's, and the door an agent takes is unchanged.**
/// Two indexes of one unchanged folder made at two moments differ in their
/// moment and nothing else, which is only possible if nothing in the crate
/// looked at a clock; and `Searched::of` still takes a permitted touch and
/// an index, so the answer an agent is given is the same function a person
/// is given, moment included. The assignment is the test; it does not
/// compile against a door that changed shape.
#[test]
fn the_moment_is_the_callers_and_the_door_is_unchanged() {
    let documents = a_folder_of_our_own("documents");
    a_library(&documents);

    let at_noon = Index::of(&documents, noon()).unwrap();
    let at_one = Index::of(&documents, one_oclock()).unwrap();
    assert_ne!(at_noon.made, at_one.made);
    assert_eq!(at_noon.of, at_one.of);
    assert_eq!(at_noon.covered, at_one.covered);
    assert_eq!(at_noon.opened, at_one.opened);
    assert_eq!(
        files_of(&at_noon),
        files_of(&at_one),
        "the moment and nothing else"
    );
    assert_eq!(at_noon.entries.len(), at_one.entries.len());

    fn the_door<'a>(touching: Touching, index: &'a Index) -> Searched<'a> {
        Searched::of(touching, index)
    }
    let _ = the_door;

    let _ = fs::remove_dir_all(&documents);
}
