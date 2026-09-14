//! An index made whole for a folder larger than one walk — against the real
//! disk, on this machine.
//!
//! The plan's acceptance for task 11: *`alo-finding` makes an index whole
//! for a folder larger than one walk's bound, with every entry's `below`
//! spelled from the folder the person named; `Covered::whole` is true at the
//! end, `not_entered` is empty, and `Covered::most` still says what one
//! walk's bound is; a test builds a folder of more than the bound, in
//! subfolders, and gets one index with every file in it, timed; `Index::again`
//! on that folder reads only what changed, checked by `Index::opened`; an
//! index that still cannot be whole — a subfolder the machine would not read
//! — says so in `Covered::unread` exactly as today; and an index file written
//! by task 3 for a folder that was cut short still reads, and is made whole
//! by `Index::again`.* And the constraint: *the walk is still `alo-files`',
//! nothing is walked that is not below the folder the person named, and the
//! format is still `1`.*
//!
//! | The acceptance | The test |
//! |---|---|
//! | a folder of more than the bound, in subfolders, is one whole index with every file in it, each `below` spelled from the folder named, timed; a link out of it is not followed | [`a_folder_larger_than_one_walk_is_one_whole_index_with_every_file_in_it_timed`] |
//! | indexed again, the large folder reads only what changed | [`indexed_again_a_folder_larger_than_one_walk_reads_only_what_changed`] |
//! | an index cut short by an earlier version still reads, and is made whole by `again` reading only what it had not reached, with `format` still `1` | [`an_index_cut_short_by_an_earlier_version_still_reads_and_is_made_whole_by_again`] |
//! | a subfolder the machine would not read, past the first walk, is in `unread` and every answer says it was not searched | [`a_subfolder_the_machine_would_not_read_past_the_first_walk_is_unread_and_said_so`] |
//!
//! The time each run measures is printed, so that the report can publish it
//! with the machine named — a number with no machine beside it is a claim.
//! What is deliberately not tested here: that the walk is `alo-files`' and
//! one file walks, which `nothing_here_opens_a_socket_or_asks_anybody.rs`
//! reads the shipped source to say.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant, SystemTime};

use alo_files::MOST_WALKED;
use alo_finding::{Contents, Covered, Index, Kind, Query, finding_words};
use alo_strings::Strings;

/// A folder of this test's own, under this machine's temporary directory,
/// named after the test so a leftover says which test left it.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-finding-whole-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    folder.canonicalize().unwrap()
}

/// A fixed moment for every index these tests make.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How many folders the large folder holds, how many letters each holds,
/// and how many older letters each holds in a folder of its own.
const FOLDERS: usize = 200;
const LETTERS: usize = 100;
const OLDER: usize = 20;

/// How many files the large folder holds, and how many things.
const FILES: usize = FOLDERS * (LETTERS + OLDER);
const THINGS: usize = FOLDERS * (2 + LETTERS + OLDER);

/// A folder of more than one walk's bound, in subfolders: `FOLDERS` folders
/// of `LETTERS` letters each, and under each an `older` folder of `OLDER`
/// more. Every letter names Anna; the last letter in the last folder — past
/// where one walk stops — is the one about the contract. The paths of every
/// file, spelled with `/` below the root, are handed back.
fn a_large_folder(root: &Path) -> BTreeSet<String> {
    const {
        assert!(
            THINGS > MOST_WALKED,
            "the folder must be larger than one walk"
        );
    }
    let mut every = BTreeSet::new();
    for f in 0..FOLDERS {
        let folder = format!("folder-{f:03}");
        fs::create_dir_all(root.join(&folder).join("older")).unwrap();
        for n in 0..LETTERS {
            let below = format!("{folder}/letter-{n:03}.txt");
            fs::write(
                root.join(&folder).join(format!("letter-{n:03}.txt")),
                format!("Dear Anna, the invoice number {f}-{n} is attached."),
            )
            .unwrap();
            every.insert(below);
        }
        for n in 0..OLDER {
            let below = format!("{folder}/older/older-{n:03}.txt");
            let subject = if f == FOLDERS - 1 && n == OLDER - 1 {
                "contract"
            } else {
                "receipt"
            };
            fs::write(
                root.join(&folder)
                    .join("older")
                    .join(format!("older-{n:03}.txt")),
                format!("Dear Anna, the {subject} from {f}-{n} is filed."),
            )
            .unwrap();
            every.insert(below);
        }
    }
    assert_eq!(every.len(), FILES);
    every
}

/// The `below` of every entry that is a file.
fn files_in(index: &Index) -> BTreeSet<String> {
    index
        .entries
        .iter()
        .filter(|entry| entry.kind.is_a_file())
        .map(|entry| entry.below.clone())
        .collect()
}

/// This crate's words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(finding_words().unwrap())
}

/// **A folder of more than the bound, in subfolders, is one whole index with
/// every file in it**, timed here and the number printed for the report;
/// `whole` is true, `not_entered` is empty and `most` still says what one
/// walk's bound is; every `below` is spelled from the folder named, so
/// [`Index::where_is`] finds each file on the disk; a query by contents
/// finds the one letter past where one walk stops; a link out of the folder
/// is an entry that is a link and nothing under it is indexed; and the
/// index written to its file reads back the same, the file's size printed
/// beside the time.
#[test]
fn a_folder_larger_than_one_walk_is_one_whole_index_with_every_file_in_it_timed() {
    let root = a_folder_of_our_own("whole");
    let building = Instant::now();
    let every = a_large_folder(&root);
    println!(
        "{FILES} files in {THINGS} things built in {:?}",
        building.elapsed()
    );
    #[cfg(unix)]
    let linked = {
        std::os::unix::fs::symlink(root.parent().unwrap(), root.join("escape")).unwrap();
        1
    };
    #[cfg(not(unix))]
    let linked = 0;

    let indexing = Instant::now();
    let index = Index::of(&root, noon()).unwrap();
    let took = indexing.elapsed();
    println!(
        "Index::of over {THINGS} things, more than one walk's bound of {MOST_WALKED}: {took:?}, \
         {} files opened",
        index.opened
    );

    assert!(index.covered.whole, "{:?}", index.covered);
    assert!(index.covered.not_entered.is_empty());
    assert_eq!(index.covered.most, MOST_WALKED, "one walk's bound, still");
    assert!(index.covered.unread.is_empty());
    assert_eq!(index.covered.unnamed, 0);
    assert!(index.covered.not_the_whole(&in_english()).is_none());
    assert_eq!(index.entries.len(), THINGS + linked);
    assert_eq!(index.opened, FILES);
    assert_eq!(files_in(&index), every, "every file, once");

    // Spelled from the folder the person named: each is on the disk there.
    for entry in &index.entries {
        let at = index.where_is(entry);
        assert!(
            fs::symlink_metadata(&at).is_ok(),
            "{} is not at {}",
            entry.below,
            at.display()
        );
        assert!(!entry.below.starts_with("escape/"), "{}", entry.below);
    }
    if linked == 1 {
        let escape = index
            .entries
            .iter()
            .find(|entry| entry.below == "escape")
            .unwrap();
        assert_eq!(
            escape.kind,
            Kind::Link,
            "a link is a link, and is not followed"
        );
    }

    // The one letter about the contract is in the last folder's older
    // letters, past where the first walk stopped, and a search finds it.
    let about_the_contract = index.answer(&Query::saying("contract")).unwrap();
    assert_eq!(
        about_the_contract
            .found
            .iter()
            .map(|entry| entry.below.as_str())
            .collect::<Vec<_>>(),
        [format!(
            "folder-{:03}/older/older-{:03}.txt",
            FOLDERS - 1,
            OLDER - 1
        )]
    );
    assert!(about_the_contract.not_searched.not_entered.is_empty());
    let every_letter = index.answer(&Query::saying("anna")).unwrap();
    assert_eq!(every_letter.found.len(), FILES);

    // Written whole and read back the same, its size on the disk printed
    // beside what it holds in hand: the bytes of every path and every word.
    let data_home = a_folder_of_our_own("whole-data-home");
    let at = Index::where_kept(Some(data_home.as_os_str()), None, &root).unwrap();
    index.kept_at(&at).unwrap();
    let held: usize = index
        .entries
        .iter()
        .map(|entry| {
            entry.below.len()
                + match &entry.contents {
                    Contents::Read { words } => words.iter().map(String::len).sum(),
                    _ => 0,
                }
        })
        .sum();
    println!(
        "the index file of {} entries is {} bytes on the disk, and holds {held} bytes of \
         paths and words in hand",
        index.entries.len(),
        fs::metadata(&at).unwrap().len()
    );
    let back = Index::read_from(&at, &root).unwrap();
    assert_eq!(back.covered, index.covered);
    assert_eq!(back.entries, index.entries);

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&data_home);
}

/// **Indexed again, the large folder reads only what changed**, checked by
/// [`Index::opened`]: nothing the second time, and exactly the one letter
/// rewritten past where the first walk stopped and the one letter added
/// before it the third time — the index whole each time.
#[test]
fn indexed_again_a_folder_larger_than_one_walk_reads_only_what_changed() {
    let root = a_folder_of_our_own("again");
    let every = a_large_folder(&root);
    let index = Index::of(&root, noon()).unwrap();
    assert!(index.covered.whole);
    assert_eq!(index.opened, FILES);

    let again = Instant::now();
    let unchanged = index.again(noon()).unwrap();
    println!(
        "Index::again over {THINGS} unchanged things: {:?}",
        again.elapsed()
    );
    assert_eq!(unchanged.opened, 0, "nothing changed, nothing read");
    assert!(unchanged.covered.whole);
    assert_eq!(files_in(&unchanged), every);
    assert_eq!(
        unchanged.find(&Query::saying("contract")).len(),
        1,
        "the words were kept"
    );

    let rewritten = root
        .join(format!("folder-{:03}", FOLDERS - 1))
        .join("older")
        .join(format!("older-{:03}.txt", OLDER - 1));
    fs::write(
        &rewritten,
        b"Dear Anna, the contract is signed and the lease with it.",
    )
    .unwrap();
    let added = root.join("folder-000").join("letter-new.txt");
    fs::write(&added, b"Dear Anna, a new letter about the lease.").unwrap();

    let changed = unchanged.again(noon()).unwrap();
    assert_eq!(changed.opened, 2, "one rewritten, one added");
    assert!(changed.covered.whole);
    assert_eq!(changed.entries.len(), THINGS + 1);
    assert_eq!(
        changed
            .find(&Query::saying("lease"))
            .iter()
            .map(|entry| entry.below.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "folder-000/letter-new.txt",
            &*format!("folder-{:03}/older/older-{:03}.txt", FOLDERS - 1, OLDER - 1),
        ])
    );
    let _ = fs::remove_dir_all(&root);
}

/// **An index file written by an earlier version for a folder that was cut
/// short still reads, and is made whole by [`Index::again`]**, reading only
/// the files the earlier index had not reached. The earlier index is one
/// whose walk stopped inside `folder-2` and never entered `folder-3` or
/// `folder-4`, written to its file in the format as it is — `format` still
/// `1`, no field added — and read back as an index that is not whole, whose
/// sentence says so; made again, it is whole, its `not_entered` is empty, and
/// every file the earlier one had is vouched for rather than read.
#[test]
fn an_index_cut_short_by_an_earlier_version_still_reads_and_is_made_whole_by_again() {
    let root = a_folder_of_our_own("cut-short");
    let mut files = 0;
    for f in 0..5 {
        let folder = root.join(format!("folder-{f}"));
        fs::create_dir_all(&folder).unwrap();
        for n in 0..6 {
            fs::write(
                folder.join(format!("letter-{n}.txt")),
                format!(
                    "Dear Anna, letter {f}-{n} about the {}.",
                    if n % 2 == 0 { "roof" } else { "garden" }
                ),
            )
            .unwrap();
            files += 1;
        }
    }
    let whole = Index::of(&root, noon()).unwrap();
    assert!(whole.covered.whole);
    assert_eq!(whole.opened, files);

    // What task 3's walk would have kept had its bound been fifteen things:
    // the five folders, six letters of folder-0 and folder-1, and the first
    // two of folder-2 — stopped inside folder-2, folder-3 and folder-4 found
    // and not entered.
    let reached = |below: &str| {
        !below.contains('/')
            || below.starts_with("folder-0/")
            || below.starts_with("folder-1/")
            || below == "folder-2/letter-0.txt"
            || below == "folder-2/letter-1.txt"
    };
    let earlier = Index {
        of: whole.of.clone(),
        made: whole.made,
        covered: Covered {
            whole: false,
            most: MOST_WALKED,
            unread: Vec::new(),
            elsewhere: Vec::new(),
            not_entered: vec![
                "folder-2".to_owned(),
                "folder-3".to_owned(),
                "folder-4".to_owned(),
            ],
            unnamed: 0,
        },
        entries: whole
            .entries
            .iter()
            .filter(|entry| reached(&entry.below))
            .cloned()
            .collect(),
        opened: 0,
    };
    let not_reached = files - (2 * 6 + 2);
    assert_eq!(earlier.entries.len(), 5 + 2 * 6 + 2);

    let data_home = a_folder_of_our_own("cut-short-data-home");
    let at = Index::where_kept(Some(data_home.as_os_str()), None, &root).unwrap();
    earlier.kept_at(&at).unwrap();
    let text = fs::read_to_string(&at).unwrap();
    let head = text.lines().next().unwrap();
    assert!(head.starts_with(r#"{"format":1,"#), "{head}");
    assert!(head.contains(r#""whole":false"#), "{head}");
    assert!(
        head.contains(r#""not_entered":["folder-2","folder-3","folder-4"]"#),
        "{head}"
    );
    for field in ["format", "of", "made", "covered"] {
        assert!(head.contains(&format!("\"{field}\":")), "{field} in {head}");
    }
    assert_eq!(head.matches("\":").count(), 4 + 6 + 2, "no field was added");

    let read_back = Index::read_from(&at, &root).unwrap();
    assert_eq!(read_back.covered, earlier.covered);
    assert_eq!(read_back.entries, earlier.entries);
    let strings = in_english();
    let said = read_back.covered.not_the_whole(&strings).unwrap();
    assert!(said.text().contains(&MOST_WALKED.to_string()), "{said}");
    let partial = read_back.answer(&Query::saying("garden")).unwrap();
    assert_eq!(partial.found.len(), 3 * 2 + 1);
    assert_eq!(
        partial.not_searched.not_entered,
        ["folder-2", "folder-3", "folder-4"]
    );

    let made_whole = read_back.again(noon()).unwrap();
    assert!(made_whole.covered.whole, "{:?}", made_whole.covered);
    assert!(made_whole.covered.not_entered.is_empty());
    assert_eq!(made_whole.covered.most, MOST_WALKED);
    assert_eq!(
        made_whole.opened, not_reached,
        "only the files the earlier index had not reached are read"
    );
    assert_eq!(files_in(&made_whole), files_in(&whole));
    assert!(made_whole.covered.not_the_whole(&strings).is_none());
    let all = made_whole.answer(&Query::saying("garden")).unwrap();
    assert_eq!(all.found.len(), 5 * 3);
    assert!(all.not_searched.not_entered.is_empty());

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&data_home);
}

/// **A subfolder the machine would not read, found past where the first walk
/// stopped, is in `unread` exactly as one the first walk stepped over**, with
/// what the machine said, and every answer says it was not searched — so
/// that *nothing matched* is never said about a folder nobody looked at.
/// The index is still whole in the sense `whole` has: nothing was left
/// unentered for being too many. Where this process is root, which no
/// permission stops, or on a host without modes, the folder reads and the
/// index says nothing was unread — checked either way.
#[test]
fn a_subfolder_the_machine_would_not_read_past_the_first_walk_is_unread_and_said_so() {
    let root = a_folder_of_our_own("unread");
    a_large_folder(&root);
    // Sorted after every `folder-NNN`, so the first walk finds it at the root
    // and has stopped long before it would enter it.
    let private = root.join("zz-private");
    fs::create_dir_all(private.join("inside")).unwrap();
    fs::write(private.join("secret.txt"), b"Dear Anna, the contract.").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&private, fs::Permissions::from_mode(0o000)).unwrap();
    }
    let stopped = fs::read_dir(&private).is_err();

    let index = Index::of(&root, noon()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&private, fs::Permissions::from_mode(0o755)).unwrap();
    }

    assert!(
        index.covered.whole,
        "unread is not unentered: {:?}",
        index.covered
    );
    assert!(index.covered.not_entered.is_empty());
    let about_the_contract = index.answer(&Query::saying("contract")).unwrap();
    if stopped {
        assert_eq!(index.covered.unread.len(), 1, "{:?}", index.covered.unread);
        let unread = index.covered.unread.first().unwrap();
        assert_eq!(unread.below, "zz-private");
        assert!(!unread.why.is_empty());
        assert_eq!(
            index.entries.len(),
            THINGS + 1,
            "the folder itself is an entry"
        );
        assert!(
            index
                .entries
                .iter()
                .all(|entry| !entry.below.starts_with("zz-private/"))
        );
        assert_eq!(
            about_the_contract.found.len(),
            1,
            "the one past the first walk"
        );
        assert_eq!(
            about_the_contract
                .not_searched
                .folders_unread
                .iter()
                .map(|unread| unread.below.as_str())
                .collect::<Vec<_>>(),
            ["zz-private"]
        );
        let said = about_the_contract
            .not_searched
            .said(&in_english())
            .into_iter()
            .map(|said| said.text().to_owned())
            .collect::<Vec<_>>();
        assert!(
            said.iter().any(|line| line.contains("zz-private")),
            "{said:?}"
        );
    } else {
        assert!(
            index.covered.unread.is_empty(),
            "this process reads anything"
        );
        assert_eq!(index.entries.len(), THINGS + 3);
        assert_eq!(about_the_contract.found.len(), 2);
    }
    let _ = fs::remove_dir_all(&root);
}
