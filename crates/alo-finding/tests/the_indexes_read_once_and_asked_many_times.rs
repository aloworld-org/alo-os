//! The indexes read once and asked many times: a search over every folder
//! from memory, with the disk's refusals held in the same place, and one
//! folder brought up to date by its name without the others being read.
//!
//! The plan's acceptance for task 9, apart from the timing: *`alo-finding`
//! offers the list's indexes read from their files once and held in hand,
//! answering any number of queries over every folder from memory in the
//! same shape `Indexed::answer` answers, with a test counting reads that
//! shows the second query opens no file; a folder whose index file would
//! not read is the same named refusal, held in the same place beside the
//! others, until the caller reads again; and one folder is brought up to
//! date by its name through the held set in one call — in hand and on the
//! disk, through `Indexed::again` — without the other folders being read
//! again, checked by the read count.* And the constraint: *holding the
//! indexes is the caller's choice for the caller's lifetime; nothing here
//! decides when to read again; nothing ranks; the verb is unchanged; the
//! list is still not a grant; `Indexed::answer` stays as it is.*
//!
//! | The acceptance | The test |
//! |---|---|
//! | the indexes read once and held, answering from memory in the shape `Indexed::answer` answers, the second query opening no file | [`the_second_query_opens_no_file_and_answers_what_the_disk_answered`] |
//! | a folder whose index file would not read is the same named refusal, held in the same place, until the caller reads again | [`a_folder_whose_index_file_would_not_read_is_the_same_refusal_in_the_same_place_until_the_caller_reads_again`] |
//! | one folder brought up to date by its name through the held set, in hand and on the disk, without the others being read again | [`one_folder_is_brought_up_to_date_by_its_name_without_the_others_being_read_again`] |
//! | bringing a folder up to date through the held set is refused the way the disk refuses, and the set is as it was | [`a_refusal_to_bring_a_folder_up_to_date_leaves_the_set_as_it_was`] |
//! | nothing ranks, `Indexed::answer` is unchanged, the verb is unchanged, the list is still not a grant | [`nothing_ranks_the_disk_is_still_asked_every_time_and_the_list_is_still_not_a_grant`] |
//!
//! The timing of both forms over three folders of ten thousand files is
//! `a_search_over_every_folder_timed.rs`. That nothing opens a socket,
//! reads a clock or watches a folder is
//! `nothing_here_opens_a_socket_or_asks_anybody.rs`, which reads the new
//! file's shipped source with every other.

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
    Entry, Everywhere, InHand, Index, Indexed, Moment, NotAsked, NotIndexed, OfFolder, Query,
    Unsearched, finding_verbs, finding_words,
};
use alo_strings::Strings;

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
        "alo-finding-in-hand-{}-{what}-{}",
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

/// Three folders, each a library, kept on this list in this order at
/// noon, one and two o'clock.
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

/// The folder at this place on the list.
fn folder_at(folders: &[PathBuf], which: usize) -> &Path {
    folders.get(which).unwrap()
}

/// What one search said, folder by folder, with the timing left out: two
/// searches timed separately are two measurements, and everything else
/// about them has to be the same.
type Said = Vec<(
    PathBuf,
    Result<(Vec<Entry>, Unsearched, Option<Moment>), NotIndexed>,
)>;

/// Everything a search over every folder said but how long it took.
fn what_was_said(everywhere: &Everywhere) -> Said {
    everywhere
        .answers
        .iter()
        .map(|of| {
            (
                of.folder.clone(),
                of.answered
                    .as_ref()
                    .map(|held| (held.found.clone(), held.not_searched.clone(), held.made))
                    .map_err(NotIndexed::clone),
            )
        })
        .collect()
}

/// The answer of the folder at this place on the list.
fn nth(everywhere: &Everywhere, which: usize) -> &OfFolder {
    everywhere.answers.get(which).unwrap()
}

/// The paths below the folder of what one folder found.
fn found_below(everywhere: &Everywhere, which: usize) -> Vec<String> {
    everywhere
        .answers
        .get(which)
        .unwrap()
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

/// **The second query opens no file, and answers what the disk answered**:
/// reading the set costs at least one read per index file; the first query
/// from hand and every one after it cost exactly what reading the counter
/// does; each is what `Indexed::answer` says, folder for folder, but for
/// the timing; and with the folders and their index files gone from the
/// disk the set still answers, because it is asked from memory.
#[test]
fn the_second_query_opens_no_file_and_answers_what_the_disk_answered() {
    let data_home = a_data_home("memory");
    let (indexed, folders) = three_folders_kept(&data_home);

    let (in_hand, reading) = reads_during(|| indexed.in_hand());
    assert_eq!(in_hand.folders(), folders.as_slice());
    assert_eq!(in_hand.indexed(), &indexed);
    assert_eq!(in_hand.each().count(), 3);
    for (folder, held) in in_hand.each() {
        assert_eq!(&held.unwrap().of, folder);
    }
    if let Some(reading) = reading {
        assert!(reading >= 3, "three index files were read once: {reading}");
    }

    let by_name = Query::named("march");
    let by_words = Query::saying("contract");
    let from_disk_by_name = what_was_said(&indexed.answer(&by_name).unwrap());
    let from_disk_by_words = what_was_said(&indexed.answer(&by_words).unwrap());
    for round in 1..=3 {
        let (answered, reads) = reads_during(|| in_hand.answer(&by_name).unwrap());
        assert_eq!(reads, reads.map(|_| 0), "round {round}: no file was opened");
        assert_eq!(answered.answers.len(), 3);
        assert!(answered.every_folder_answered());
        assert_eq!(what_was_said(&answered), from_disk_by_name, "round {round}");
        let (answered, reads) = reads_during(|| in_hand.answer(&by_words).unwrap());
        assert_eq!(reads, reads.map(|_| 0), "round {round}: no file was opened");
        assert_eq!(
            what_was_said(&answered),
            from_disk_by_words,
            "round {round}"
        );
    }
    for (which, folder) in folders.iter().enumerate() {
        assert_eq!(
            in_hand.index_of(folder).unwrap(),
            &indexed.index_of(folder).unwrap()
        );
        assert_eq!(
            nth(&in_hand.answer(&by_name).unwrap(), which)
                .answered
                .as_ref()
                .unwrap()
                .made,
            Some(Moment::of(hours_after_noon(u64::try_from(which).unwrap())))
        );
    }

    // A query that is not one is refused from hand as it is from the disk,
    // and searches nothing.
    let (refused, reads) = reads_during(|| in_hand.answer(&Query::named("")).unwrap_err());
    assert_eq!(refused, NotAsked::Nothing);
    assert_eq!(reads, reads.map(|_| 0));

    // The folders and the index files are gone; the disk now refuses, and
    // the set answers as it did.
    cleared(&data_home, &folders);
    let disk_now = indexed.answer(&by_name).unwrap();
    assert_eq!(disk_now.refused().count(), 3, "the disk has nothing to say");
    let (still, reads) = reads_during(|| in_hand.answer(&by_name).unwrap());
    assert_eq!(reads, reads.map(|_| 0));
    assert_eq!(what_was_said(&still), from_disk_by_name);
    assert_eq!(found_below(&still, 1).len(), 2);
}

/// **A folder whose index file would not read is the same named refusal,
/// held in the same place beside the others, until the caller reads
/// again**: one index file of three torn before the set is read; every
/// query from hand gives two answers and the one refusal, equal each time,
/// naming the file, at the torn folder's place; the file mended on the
/// disk changes nothing in hand, while the disk answers again; and a set
/// read again answers from every folder. A file gone, and another folder's
/// index in its place, are held the same way.
#[test]
fn a_folder_whose_index_file_would_not_read_is_the_same_refusal_in_the_same_place_until_the_caller_reads_again()
 {
    let data_home = a_data_home("torn");
    let (indexed, folders) = three_folders_kept(&data_home);
    let strings = in_english();
    let middle = folder_at(&folders, 1);
    let middle_file = indexed.where_index_of(middle);
    let whole = indexed.index_of(middle).unwrap();

    fs::write(&middle_file, b"hello\n").unwrap();
    let in_hand = indexed.in_hand();
    let first = in_hand.answer(&Query::named("march")).unwrap();
    let (second, reads) = reads_during(|| in_hand.answer(&Query::named("march")).unwrap());
    assert_eq!(reads, reads.map(|_| 0));
    for asked in [&first, &second] {
        assert_eq!(
            asked.answers.len(),
            3,
            "a torn index is not a missing folder"
        );
        assert!(!asked.every_folder_answered());
        assert_eq!(asked.answered().count(), 2);
        let refused: Vec<(&PathBuf, &NotIndexed)> = asked.refused().collect();
        assert_eq!(refused.len(), 1);
        let (at, why) = refused.first().unwrap();
        assert_eq!(*at, middle, "at the torn folder's place");
        assert!(
            matches!(why, NotIndexed::NotAnIndex { at, .. } if at == &middle_file),
            "{why}"
        );
        let said = why.said(&strings).into_text();
        assert!(
            said.contains(&middle_file.to_string_lossy().into_owned()),
            "{said}"
        );
        assert!(said.contains("not an index"), "{said}");
        assert_eq!(&nth(asked, 1).folder, middle);
    }
    assert_eq!(
        what_was_said(&first),
        what_was_said(&second),
        "the same refusal each time"
    );
    assert_eq!(
        what_was_said(&first),
        what_was_said(&indexed.answer(&Query::named("march")).unwrap()),
        "what the disk said when the set was read"
    );
    assert!(matches!(
        in_hand.index_of(middle).unwrap_err(),
        NotIndexed::NotAnIndex { at, .. } if at == middle_file
    ));

    // Mended on the disk: the disk answers, and the set is as it was until
    // the caller reads again.
    whole.kept_at(&middle_file).unwrap();
    assert!(
        indexed
            .answer(&Query::named("march"))
            .unwrap()
            .every_folder_answered()
    );
    let (still_refused, reads) = reads_during(|| in_hand.answer(&Query::named("march")).unwrap());
    assert_eq!(reads, reads.map(|_| 0));
    assert_eq!(what_was_said(&still_refused), what_was_said(&first));
    let read_again = indexed.in_hand();
    assert!(
        read_again
            .answer(&Query::named("march"))
            .unwrap()
            .every_folder_answered()
    );
    assert_ne!(read_again, in_hand);

    // Gone, and another folder's: held the same way, in the same place.
    fs::remove_file(&middle_file).unwrap();
    let gone = indexed.in_hand();
    for _ in 0..2 {
        let asked = gone.answer(&Query::named("march")).unwrap();
        assert_eq!(asked.answered().count(), 2);
        assert!(matches!(
            nth(&asked, 1).answered.as_ref().unwrap_err(),
            NotIndexed::NotOpened { at, .. } if at == &middle_file
        ));
    }
    fs::copy(indexed.where_index_of(folder_at(&folders, 0)), &middle_file).unwrap();
    let swapped = indexed.in_hand();
    for _ in 0..2 {
        let asked = swapped.answer(&Query::named("march")).unwrap();
        assert_eq!(asked.answered().count(), 2);
        assert!(matches!(
            nth(&asked, 1).answered.as_ref().unwrap_err(),
            NotIndexed::NotTheSame { asked, indexed }
                if asked == middle && indexed == folder_at(&folders, 0)
        ));
    }

    // The list itself is untouched by any of this.
    assert_eq!(indexed.folders(), folders.as_slice());
    assert_eq!(the_list_under(&data_home).folders(), folders.as_slice());

    cleared(&data_home, &folders);
}

/// **One folder is brought up to date by its name through the held set, in
/// hand and on the disk, without the other folders being read again**: a
/// file is added under the first folder and the other two index files are
/// torn on the disk after the set was read; bringing the first up to date
/// through the set finds the new file, in hand and in the file the disk
/// now holds, and the other two still answer from hand — they would have
/// refused, had they been read again. Then, with nothing changed, the
/// same refresh through the set costs exactly the reads `Indexed::again`
/// costs on its own, and fewer than reading the set did.
#[test]
fn one_folder_is_brought_up_to_date_by_its_name_without_the_others_being_read_again() {
    let data_home = a_data_home("again");
    let (mut indexed, folders) = three_folders_kept(&data_home);
    let first = folder_at(&folders, 0);
    let (mut in_hand, reading_the_set) = reads_during(|| indexed.in_hand());
    let before = what_was_said(&in_hand.answer(&Query::named("march")).unwrap());

    fs::write(first.join("march-again.txt"), b"March, once more.").unwrap();
    for which in [1, 2] {
        fs::write(
            indexed.where_index_of(folder_at(&folders, which)),
            b"hello\n",
        )
        .unwrap();
    }
    let fresh = in_hand.again(first, hours_after_noon(5)).unwrap().clone();
    assert_eq!(fresh.of, first);
    assert_eq!(fresh.made, Some(Moment::of(hours_after_noon(5))));
    assert_eq!(fresh.opened, 1, "only the new file was read");
    assert_eq!(in_hand.index_of(first).unwrap(), &fresh, "in hand");
    let on_the_disk = Index::read_from(&indexed.where_index_of(first), first).unwrap();
    assert_eq!(on_the_disk.entries, fresh.entries, "and on the disk");
    assert_eq!(on_the_disk.made, fresh.made, "and on the disk");
    assert_eq!(on_the_disk.covered, fresh.covered, "and on the disk");
    assert_eq!(on_the_disk.opened, 0, "read back, nothing was opened");
    assert_eq!(in_hand.indexed().folders(), folders.as_slice(), "not moved");

    let (after, reads) = reads_during(|| in_hand.answer(&Query::named("march")).unwrap());
    assert_eq!(reads, reads.map(|_| 0));
    assert!(
        after.every_folder_answered(),
        "the other two folders answer from hand; read again, they would have refused"
    );
    assert_eq!(found_below(&after, 0).len(), 3);
    assert!(found_below(&after, 0).contains(&"march-again.txt".to_owned()));
    assert_eq!(nth(&after, 0).answered.as_ref().unwrap().made, fresh.made);
    let after = what_was_said(&after);
    assert_eq!(
        after.get(1),
        before.get(1),
        "the second folder, as it was in hand"
    );
    assert_eq!(
        after.get(2),
        before.get(2),
        "the third folder, as it was in hand"
    );
    let disk = indexed.answer(&Query::named("march")).unwrap();
    assert_eq!(
        disk.refused().count(),
        2,
        "the disk, asked, refuses the two torn"
    );
    assert_eq!(
        found_below(&disk, 0),
        found_below(&in_hand.answer(&Query::named("march")).unwrap(), 0)
    );

    // Nothing changed now: the refresh through the set reads what the
    // refresh on the disk reads — one index file, and no other — and less
    // than reading the whole set did.
    let (_, through_the_set) = reads_during(|| in_hand.again(first, hours_after_noon(6)).unwrap());
    let (_, on_its_own) = reads_during(|| indexed.again(first, hours_after_noon(7)).unwrap());
    let (_, the_set_again) = reads_during(|| indexed.in_hand());
    if let (Some(through_the_set), Some(on_its_own), Some(reading_the_set), Some(the_set_again)) =
        (through_the_set, on_its_own, reading_the_set, the_set_again)
    {
        assert_eq!(
            through_the_set, on_its_own,
            "bringing one folder up to date through the set reads exactly what doing it on the \
             disk reads: that folder's index file, and nothing of the others"
        );
        assert!(
            through_the_set < reading_the_set.min(the_set_again),
            "one index file read, not three: {through_the_set} against {reading_the_set} and \
             {the_set_again}"
        );
    }
    assert_eq!(
        in_hand.index_of(first).unwrap().made,
        Some(Moment::of(hours_after_noon(6))),
        "the set holds the moment it was brought up to date through it, not the later one on \
         the disk, until the caller reads again"
    );

    cleared(&data_home, &folders);
}

/// **A refusal to bring a folder up to date through the held set leaves
/// the set as it was, the way it leaves the disk as it was**: a folder
/// never indexed is refused with no file read; a folder named from
/// somewhere is refused; a folder gone since it was indexed is refused
/// and still answers from hand as it did; and a folder whose kept index
/// file was torn after the set was read is refused and still answers from
/// hand — then, the file mended, is brought up to date and answers anew.
#[test]
fn a_refusal_to_bring_a_folder_up_to_date_leaves_the_set_as_it_was() {
    let data_home = a_data_home("refused");
    let (indexed, folders) = three_folders_kept(&data_home);
    let strings = in_english();
    let mut in_hand = indexed.in_hand();
    let as_it_was = in_hand.clone();
    let before = what_was_said(&in_hand.answer(&Query::named("march")).unwrap());
    let never = a_folder_of_our_own("never");
    a_library(&never);

    let (refused, reads) = reads_during(|| in_hand.again(&never, hours_after_noon(5)).unwrap_err());
    assert!(matches!(&refused, NotIndexed::NeverIndexed { at } if at == &never));
    assert_eq!(
        reads,
        reads.map(|_| 0),
        "nothing read for a folder never indexed"
    );
    assert!(!refused.said(&strings).is_a_bug());
    assert!(matches!(
        in_hand.again(Path::new("Documents"), hours_after_noon(5)).unwrap_err(),
        NotIndexed::NotAbsolute { at } if at == Path::new("Documents")
    ));
    assert_eq!(in_hand, as_it_was);

    // Gone: the folder is unplugged, and the set and the disk keep its
    // index.
    let third = folder_at(&folders, 2);
    fs::remove_dir_all(third).unwrap();
    let refused = in_hand.again(third, hours_after_noon(5)).unwrap_err();
    assert!(
        matches!(&refused, NotIndexed::NotWalked { at, .. } if at == third),
        "{refused}"
    );
    assert_eq!(in_hand, as_it_was);
    assert_eq!(
        what_was_said(&in_hand.answer(&Query::named("march")).unwrap()),
        before
    );
    assert!(indexed.index_of(third).is_ok(), "the disk keeps it too");

    // Torn after the set was read: the disk's word is asked first, and
    // refuses; the set still answers as it did.
    let second = folder_at(&folders, 1);
    let second_file = indexed.where_index_of(second);
    let whole = indexed.index_of(second).unwrap();
    fs::write(&second_file, b"hello\n").unwrap();
    let refused = in_hand.again(second, hours_after_noon(5)).unwrap_err();
    assert!(
        matches!(&refused, NotIndexed::NotAnIndex { at, .. } if at == &second_file),
        "{refused}"
    );
    assert_eq!(in_hand, as_it_was);
    assert_eq!(
        what_was_said(&in_hand.answer(&Query::named("march")).unwrap()),
        before
    );

    // Mended, and a file added: brought up to date through the set, the
    // place that held the old index holds the fresh one.
    whole.kept_at(&second_file).unwrap();
    fs::write(second.join("march-again.txt"), b"March, once more.").unwrap();
    let fresh = in_hand.again(second, hours_after_noon(6)).unwrap().clone();
    assert_eq!(fresh.entries.len(), whole.entries.len() + 1);
    assert_ne!(in_hand, as_it_was);
    let after = in_hand.answer(&Query::named("march")).unwrap();
    assert_eq!(found_below(&after, 1).len(), 3);
    assert_eq!(nth(&after, 1).answered.as_ref().unwrap().made, fresh.made);

    // And a place that held a refusal holds the fresh index once the
    // folder is brought up to date through the set.
    fs::write(&second_file, b"hello\n").unwrap();
    let mut torn_in_hand = indexed.in_hand();
    assert_eq!(
        torn_in_hand
            .answer(&Query::named("march"))
            .unwrap()
            .refused()
            .count(),
        1
    );
    fresh.kept_at(&second_file).unwrap();
    torn_in_hand.again(second, hours_after_noon(7)).unwrap();
    assert!(
        torn_in_hand
            .answer(&Query::named("march"))
            .unwrap()
            .every_folder_answered()
    );

    let _ = fs::remove_dir_all(&never);
    cleared(&data_home, &folders);
}

/// **Nothing ranks, the disk is still asked every time by `Indexed::answer`,
/// the verb is unchanged, and the list is still not a grant**: the answers
/// from hand are in the list's order and within a folder the index's own;
/// `Indexed::answer` reads the index files on every call, as before; with
/// no grant at all the set answers every folder while an agent's
/// `search_files` over one of them is refused at the door; and the shapes
/// are pinned by assignment.
#[test]
fn nothing_ranks_the_disk_is_still_asked_every_time_and_the_list_is_still_not_a_grant() {
    let data_home = a_data_home("order");
    let (mut indexed, folders) = three_folders_kept(&data_home);
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
    let in_hand = indexed.in_hand();

    let everywhere = in_hand.answer(&Query::named("march")).unwrap();
    let in_order: Vec<&PathBuf> = everywhere.answers.iter().map(|of| &of.folder).collect();
    assert_eq!(in_order, folders.iter().collect::<Vec<_>>());
    assert_eq!(found_below(&everywhere, 0).len(), 1);
    assert_eq!(found_below(&everywhere, 1).len(), 2);
    assert_eq!(found_below(&everywhere, 2).len(), 7);
    for (which, folder) in folders.iter().enumerate() {
        let unranked: Vec<String> = in_hand
            .index_of(folder)
            .unwrap()
            .find(&Query::named("march"))
            .iter()
            .map(|entry| entry.below.clone())
            .collect();
        assert_eq!(found_below(&everywhere, which), unranked);
    }

    // `Indexed::answer` is as it was: the disk's word every time.
    for _ in 0..2 {
        let (_, reads) = reads_during(|| indexed.answer(&Query::named("march")).unwrap());
        if let Some(reads) = reads {
            assert!(reads >= 3, "three index files read on every call: {reads}");
        }
    }

    // No grant, and the set answers; an agent over one of its folders is
    // refused at the door.
    assert_eq!(everywhere.answered().count(), 3);
    let call = finding_verbs()
        .unwrap()
        .call(
            "search_files",
            &[
                (
                    "folder",
                    Given::text(folder_at(&folders, 1).to_string_lossy().into_owned()),
                ),
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

    fn the_set(indexed: &Indexed) -> InHand {
        indexed.in_hand()
    }
    fn from_hand(in_hand: &InHand, query: &Query) -> Result<Everywhere, NotAsked> {
        in_hand.answer(query)
    }
    fn from_disk(indexed: &Indexed, query: &Query) -> Result<Everywhere, NotAsked> {
        indexed.answer(query)
    }
    fn through_the_set<'a>(
        in_hand: &'a mut InHand,
        folder: &Path,
        made: SystemTime,
    ) -> Result<&'a Index, NotIndexed> {
        in_hand.again(folder, made)
    }
    let _ = (the_set, from_hand, from_disk, through_the_set);

    cleared(&data_home, &folders);
}
