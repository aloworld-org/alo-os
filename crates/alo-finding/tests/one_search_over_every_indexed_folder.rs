//! One search over every indexed folder, each answer saying which folder
//! and how old.
//!
//! The plan's acceptance for task 8: *`alo-finding` answers one `Query` over
//! every folder on the list in one call, as one answer per folder, in the
//! list's order, each carrying the folder it is of, what matched, what was
//! not searched and the moment its index was made, read from each index's
//! file and never by walking; a folder whose index file could not be read or
//! is not an index is a refusal beside the other answers, named, rather than
//! a missing folder or a failed search; a query that is not one is refused
//! once, before any index file is opened, checked by the read count; and an
//! empty list answers with no folders and no refusal.* And the constraint:
//! *nothing ranks across folders; the verb is unchanged; the list is still
//! not a grant.*
//!
//! | The acceptance | The test |
//! |---|---|
//! | one query, every folder, in the list's order, each saying which folder and how old, from the index files and never by walking | [`one_query_is_answered_from_every_folder_in_the_lists_order_each_saying_which_and_how_old`] |
//! | a folder whose index file would not read is a named refusal beside the other answers | [`a_folder_whose_index_file_would_not_read_is_a_named_refusal_beside_the_other_answers`] |
//! | a query that is not one is refused once, before any index file is opened | [`a_query_that_is_not_one_is_refused_once_before_any_index_file_is_opened`] |
//! | an empty list answers with no folders and no refusal | [`an_empty_list_answers_with_no_folders_and_no_refusal`] |
//! | nothing ranks across folders or within one | [`nothing_ranks_across_folders_or_within_one`] |
//! | the verb is unchanged, and the list is still not a grant | [`the_search_over_every_folder_is_not_a_verb_and_the_list_is_still_not_a_grant`] |
//!
//! What is deliberately not tested here: that nothing opens a socket, reads
//! a clock or watches a folder. `nothing_here_opens_a_socket_or_asks_anybody.rs`
//! reads the shipped source — the two new files included — and says so.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Authorised, Call, Given, Grantee, Grants, NotAuthorised};
use alo_files::{OnThisMachine, Resolving, Touching};
use alo_finding::{
    Everywhere, Index, Indexed, Moment, NotAsked, NotIndexed, OfFolder, Query, Searched,
    finding_verbs, finding_words,
};
use alo_strings::Strings;

/// A fixed moment: when the first folder's index is made.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// So many hours after noon: when a later folder's index is made.
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
        "alo-finding-everywhere-{}-{what}-{}",
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

/// A few known files: two whose names hold *march* — one at the top and one
/// in a folder below — and one that does not, whose words hold *contract*.
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

/// Three folders, each a library, kept on this list in this order — not
/// the alphabetical one — at noon, one and two o'clock.
fn three_folders_kept(data_home: &Path) -> (Indexed, Vec<PathBuf>) {
    let folders = vec![
        a_folder_of_our_own("pictures"),
        a_folder_of_our_own("documents"),
        a_folder_of_our_own("music"),
    ];
    let mut indexed = the_list_under(data_home);
    for (hours, folder) in folders.iter().enumerate() {
        a_library(folder);
        indexed
            .keep(&Index::of(folder, hours_after_noon(u64::try_from(hours).unwrap())).unwrap())
            .unwrap();
    }
    assert_eq!(indexed.folders(), folders.as_slice());
    (indexed, folders)
}

/// The answer of the folder at this place on the list.
fn nth(everywhere: &Everywhere, which: usize) -> &OfFolder {
    everywhere.answers.get(which).unwrap()
}

/// The folder at this place on the list.
fn folder_at(folders: &[PathBuf], which: usize) -> &Path {
    folders.get(which).unwrap()
}

/// The paths below the folder of what one folder found.
fn found_below(everywhere: &Everywhere, which: usize) -> Vec<String> {
    nth(everywhere, which)
        .answered
        .as_ref()
        .unwrap()
        .found
        .iter()
        .map(|entry| entry.below.clone())
        .collect()
}

/// How many read calls this thread has made, where the kernel keeps the
/// number per thread; [`None`] on a host that does not.
fn reads_so_far() -> Option<u64> {
    let io = fs::read_to_string("/proc/thread-self/io").ok()?;
    io.lines()
        .find_map(|line| line.strip_prefix("syscr: "))
        .and_then(|count| count.trim().parse().ok())
}

/// A call of the search verb over this folder, for this part of a name.
fn searching(folder: &Path, named: &str) -> Call {
    finding_verbs()
        .unwrap()
        .call(
            "search_files",
            &[
                ("folder", Given::text(folder.to_string_lossy().into_owned())),
                ("named", Given::text(named)),
            ],
        )
        .unwrap()
}

/// Everything removed, at the end.
fn cleared(data_home: &Path, folders: &[PathBuf]) {
    for folder in folders {
        let _ = fs::remove_dir_all(folder);
    }
    let _ = fs::remove_dir_all(data_home);
}

/// **One query is answered from every folder on the list, in the list's
/// order, each answer saying which folder it is of and how old it is** —
/// and each is the answer that folder's own index gives, from the index
/// files alone: the folders themselves are removed and the answers are the
/// same.
#[test]
fn one_query_is_answered_from_every_folder_in_the_lists_order_each_saying_which_and_how_old() {
    let data_home = a_data_home("every");
    let (indexed, folders) = three_folders_kept(&data_home);

    let everywhere = indexed.answer(&Query::named("march")).unwrap();
    assert_eq!(everywhere.answers.len(), 3);
    assert!(everywhere.every_folder_answered());
    assert_eq!(everywhere.refused().count(), 0);
    for (which, (of, folder)) in everywhere.answers.iter().zip(&folders).enumerate() {
        assert_eq!(&of.folder, folder, "the list's order, not the disk's");
        let held = of.answered.as_ref().unwrap();
        assert_eq!(
            held.made,
            Some(Moment::of(hours_after_noon(u64::try_from(which).unwrap()))),
            "each answer says when its own index was made"
        );
        assert_eq!(held.not_searched.outside, *folder);
        assert!(held.not_searched.borrowed().is_nothing());
        let its_own = indexed.index_of(folder).unwrap();
        let alone = its_own.answer(&Query::named("march")).unwrap();
        assert_eq!(
            held.found,
            alone.found.iter().map(|&e| e.clone()).collect::<Vec<_>>()
        );
        assert_eq!(held.not_searched.borrowed(), alone.not_searched);
        assert_eq!(held.made, alone.made, "the folder's own answer, held");
        assert_eq!(held.found.len(), 2);
        let names: Vec<&str> = held.found.iter().map(|entry| entry.name()).collect();
        assert!(names.contains(&"march-notes.txt"), "{names:?}");
        assert!(names.contains(&"march.pdf"), "{names:?}");
    }
    let answered: Vec<&PathBuf> = everywhere.answered().map(|(folder, _)| folder).collect();
    assert_eq!(answered, folders.iter().collect::<Vec<_>>());

    // By words, what could not be searched is said per folder, and the
    // sentences name that folder.
    let by_words = indexed.answer(&Query::saying("contract")).unwrap();
    let strings = in_english();
    for (of, folder) in by_words.answers.iter().zip(&folders) {
        let held = of.answered.as_ref().unwrap();
        assert_eq!(held.found.len(), 1, "one file says contract");
        assert_eq!(
            held.not_searched.no_reader.len(),
            1,
            "the PDF has no reader"
        );
        let said = held.not_searched.borrowed().said(&strings);
        let first = said.first().unwrap();
        assert!(
            first
                .text()
                .contains(&folder.to_string_lossy().into_owned()),
            "{first}"
        );
    }

    // The folders are gone, and the answers are the same: nothing was
    // walked to answer.
    for folder in &folders {
        fs::remove_dir_all(folder).unwrap();
    }
    let again = indexed.answer(&Query::named("march")).unwrap();
    assert_eq!(again.answers.len(), 3);
    for (which, (before, after)) in everywhere.answers.iter().zip(&again.answers).enumerate() {
        assert_eq!(before.folder, after.folder);
        let (before, after) = (
            before.answered.as_ref().unwrap(),
            after.answered.as_ref().unwrap(),
        );
        assert_eq!(before.found, after.found);
        assert_eq!(before.not_searched, after.not_searched);
        assert_eq!(before.made, after.made);
        assert_eq!(found_below(&everywhere, which), found_below(&again, which));
    }

    cleared(&data_home, &folders);
}

/// **A folder whose index file could not be read, or is not an index, is a
/// named refusal beside the other answers** — not a missing folder and not
/// a failed search: three folders with one index file torn still give two
/// answers and one refusal naming the file, and the two answers are what
/// they were before the tear. The same when the file is gone, and when it
/// is another folder's index.
#[test]
fn a_folder_whose_index_file_would_not_read_is_a_named_refusal_beside_the_other_answers() {
    let data_home = a_data_home("torn");
    let (indexed, folders) = three_folders_kept(&data_home);
    let strings = in_english();
    let whole = indexed.answer(&Query::named("march")).unwrap();
    assert!(whole.every_folder_answered());

    // Torn: the middle folder's index file holds something that is not an
    // index.
    let middle = indexed.where_index_of(folder_at(&folders, 1));
    fs::write(&middle, b"hello\n").unwrap();
    let torn = indexed.answer(&Query::named("march")).unwrap();
    assert_eq!(
        torn.answers.len(),
        3,
        "a torn index is not a missing folder"
    );
    assert!(!torn.every_folder_answered());
    let folders_in_order: Vec<&PathBuf> = torn.answers.iter().map(|of| &of.folder).collect();
    assert_eq!(folders_in_order, folders.iter().collect::<Vec<_>>());
    assert!(nth(&torn, 0).answered.is_ok());
    assert!(nth(&torn, 2).answered.is_ok());
    let refused = nth(&torn, 1).answered.as_ref().unwrap_err();
    assert!(
        matches!(refused, NotIndexed::NotAnIndex { at, .. } if at == &middle),
        "{refused}"
    );
    let said = refused.said(&strings).into_text();
    assert!(
        said.contains(&middle.to_string_lossy().into_owned()),
        "{said}"
    );
    assert!(said.contains("not an index"), "{said}");
    let named: Vec<(&PathBuf, &NotIndexed)> = torn.refused().collect();
    assert_eq!(named.len(), 1);
    assert_eq!(named.first().unwrap().0, folder_at(&folders, 1));
    assert_eq!(torn.answered().count(), 2);
    for which in [0, 2] {
        assert_eq!(
            found_below(&torn, which),
            found_below(&whole, which),
            "the other folders answer as they did"
        );
    }

    // Gone: the file is not there at all.
    fs::remove_file(&middle).unwrap();
    let gone = indexed.answer(&Query::named("march")).unwrap();
    assert_eq!(gone.answers.len(), 3);
    assert!(matches!(
        nth(&gone, 1).answered.as_ref().unwrap_err(),
        NotIndexed::NotOpened { at, .. } if at == &middle
    ));
    assert_eq!(gone.answered().count(), 2);

    // Another folder's: the first folder's index copied over the middle's.
    fs::copy(indexed.where_index_of(folder_at(&folders, 0)), &middle).unwrap();
    let swapped = indexed.answer(&Query::named("march")).unwrap();
    assert_eq!(swapped.answers.len(), 3);
    assert!(matches!(
        nth(&swapped, 1).answered.as_ref().unwrap_err(),
        NotIndexed::NotTheSame { asked, indexed }
            if asked == folder_at(&folders, 1) && indexed == folder_at(&folders, 0)
    ));
    assert_eq!(swapped.answered().count(), 2);

    // The list itself is untouched by any of this.
    assert_eq!(indexed.folders(), folders.as_slice());
    assert_eq!(the_list_under(&data_home).folders(), folders.as_slice());

    cleared(&data_home, &folders);
}

/// **A query that is not one is refused once, before any index file is
/// opened**: three folders are indexed, and each refusal moves the kernel's
/// per-thread read count by exactly what reading the counter costs — where
/// a query that is one reads three files.
#[test]
fn a_query_that_is_not_one_is_refused_once_before_any_index_file_is_opened() {
    let data_home = a_data_home("refused");
    let (indexed, folders) = three_folders_kept(&data_home);
    let strings = in_english();
    let forty = (0..40)
        .map(|n| format!("w{n}"))
        .collect::<Vec<_>>()
        .join(" ");
    let not_queries = [
        (Query::named(""), "nothing"),
        (Query::named("   "), "nothing"),
        (Query::saying(&forty), "more than a sentence"),
        (Query::named(&"a".repeat(256)), "longer than a name"),
    ];

    // What reading the counter itself costs, so the answer is held to
    // exactly that and not to a guess.
    let counted_once = reads_so_far();
    let counted_twice = reads_so_far();
    for (query, why) in &not_queries {
        let before = reads_so_far();
        let refused = indexed.answer(query).unwrap_err();
        let after = reads_so_far();
        match *why {
            "nothing" => assert_eq!(refused, NotAsked::Nothing),
            "more than a sentence" => {
                assert!(matches!(
                    refused,
                    NotAsked::MoreThanASentence { words: 40, .. }
                ));
            }
            _ => assert!(matches!(
                refused,
                NotAsked::LongerThanAName { chars: 256, .. }
            )),
        }
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        if let (Some(once), Some(twice), Some(before), Some(after)) =
            (counted_once, counted_twice, before, after)
        {
            assert_eq!(
                after - before,
                twice - once,
                "a query that is not one opened no index file: the count moved by what \
                 reading it costs, and three index files would be three more"
            );
        }
    }

    // And a query that is one reads the three index files, so the counter
    // is known to see what it must not see.
    let before = reads_so_far();
    let answered = indexed.answer(&Query::named("march")).unwrap();
    let after = reads_so_far();
    assert_eq!(answered.answers.len(), 3);
    if let (Some(once), Some(twice), Some(before), Some(after)) =
        (counted_once, counted_twice, before, after)
    {
        assert!(
            after - before >= (twice - once) + 3,
            "three index files were read to answer"
        );
    }

    cleared(&data_home, &folders);
}

/// **An empty list answers with no folders and no refusal**, because nothing
/// asked for is nothing to search — and a query that is not one is still
/// refused over an empty list, because the check comes before the list.
#[test]
fn an_empty_list_answers_with_no_folders_and_no_refusal() {
    let data_home = a_data_home("empty");
    let indexed = the_list_under(&data_home);
    assert!(indexed.folders().is_empty());

    let nothing = indexed.answer(&Query::named("march")).unwrap();
    assert!(nothing.answers.is_empty());
    assert!(nothing.every_folder_answered());
    assert_eq!(nothing.answered().count(), 0);
    assert_eq!(nothing.refused().count(), 0);

    assert_eq!(
        indexed.answer(&Query::named("")).unwrap_err(),
        NotAsked::Nothing
    );

    cleared(&data_home, &[]);
}

/// **Nothing ranks across folders or within one**: the order is the list's,
/// not the alphabet's and not the number of matches; bringing a folder's
/// index up to date does not move it; and within a folder the order is the
/// index's own, which is the walk's.
#[test]
fn nothing_ranks_across_folders_or_within_one() {
    let data_home = a_data_home("order");
    let (mut indexed, folders) = three_folders_kept(&data_home);
    // The first folder has the fewest matches, so an order by matches would
    // put it last; the alphabet would put `documents` first.
    fs::remove_file(folder_at(&folders, 0).join("march-notes.txt")).unwrap();
    for number in 0..5 {
        fs::write(
            folder_at(&folders, 2).join(format!("zz-march-{number}.txt")),
            b"march again",
        )
        .unwrap();
    }
    indexed
        .again(folder_at(&folders, 0), hours_after_noon(3))
        .unwrap();
    indexed
        .again(folder_at(&folders, 2), hours_after_noon(4))
        .unwrap();
    assert_eq!(
        indexed.folders(),
        folders.as_slice(),
        "brought up to date, not moved"
    );

    let everywhere = indexed.answer(&Query::named("march")).unwrap();
    let in_order: Vec<&PathBuf> = everywhere.answers.iter().map(|of| &of.folder).collect();
    assert_eq!(in_order, folders.iter().collect::<Vec<_>>());
    assert_eq!(found_below(&everywhere, 0).len(), 1);
    assert_eq!(found_below(&everywhere, 1).len(), 2);
    assert_eq!(found_below(&everywhere, 2).len(), 7);
    assert_eq!(
        nth(&everywhere, 0).answered.as_ref().unwrap().made,
        Some(Moment::of(hours_after_noon(3)))
    );

    // Within a folder, the index's own order — `Index::find` is the filter
    // with nothing between it and the entries.
    for (which, folder) in folders.iter().enumerate() {
        let its_own = indexed.index_of(folder).unwrap();
        let unranked: Vec<String> = its_own
            .find(&Query::named("march"))
            .iter()
            .map(|entry| entry.below.clone())
            .collect();
        assert_eq!(found_below(&everywhere, which), unranked);
    }

    cleared(&data_home, &folders);
}

/// **The search over every folder is not a verb, and the list is still not
/// a grant**: with no grant at all, `Indexed::answer` answers every indexed
/// folder — because a person searching their own files is not asking
/// anybody — while an agent's `search_files` over one of those folders is
/// refused at the door, and the door itself still takes one permitted touch
/// and one index. The assignments are the test; they do not compile against
/// a shape that changed.
#[test]
fn the_search_over_every_folder_is_not_a_verb_and_the_list_is_still_not_a_grant() {
    let data_home = a_data_home("grant");
    let (indexed, folders) = three_folders_kept(&data_home);
    let no_grants = Grants::default();

    let everywhere = indexed.answer(&Query::named("march")).unwrap();
    assert_eq!(
        everywhere.answered().count(),
        3,
        "no grant, and every folder answers"
    );

    let refused = Authorised::read(
        &searching(folder_at(&folders, 1), "march"),
        &Grantee::named("@finding"),
        &no_grants,
        noon(),
    )
    .unwrap_err();
    assert!(
        matches!(refused.why(), NotAuthorised::NotGranted(_)),
        "indexed, and an agent is still refused: {:?}",
        refused.why()
    );

    fn the_door<'a>(touching: Touching, index: &'a Index) -> Searched<'a> {
        Searched::of(touching, index)
    }
    fn the_search(indexed: &Indexed, query: &Query) -> Result<Everywhere, NotAsked> {
        indexed.answer(query)
    }
    let _ = (the_door, the_search);

    cleared(&data_home, &folders);
}
