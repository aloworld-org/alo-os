//! Which folders are indexed, and the index for a folder found by its name.
//!
//! The plan's acceptance for task 6: *`alo-finding` keeps the list of
//! folders a person asked to index, beside the indexes, written whole or not
//! at all like an index is; a caller names a folder and gets back its index
//! read from the disk, or a refusal in words saying the folder was never
//! indexed — never a walk of the folder; a folder removed from the list has
//! its index file removed with it; the list answers the same whether an
//! agent or a person asked; and `Searched::of` is unchanged.* And the
//! constraint: *the list is not a grant and grants nothing.*
//!
//! | The acceptance | The test |
//! |---|---|
//! | the list is kept beside the indexes, and a folder's index comes back from the disk | [`a_folder_kept_is_on_the_list_and_its_index_comes_back_from_the_disk_and_not_from_a_walk`] |
//! | a folder never indexed is refused in words, and the folder is never walked | [`a_folder_never_indexed_is_refused_in_words_and_never_walked`] |
//! | a folder forgotten has its index file removed with it | [`a_folder_forgotten_has_its_index_file_removed_with_it`] |
//! | the list is written whole or not at all, and a list that is not one is refused | [`the_list_is_written_whole_or_not_at_all_and_what_is_not_a_list_is_refused`] |
//! | the list holds folders and nothing the record does | [`the_list_holds_folders_and_nothing_the_record_does`] |
//! | the list is not a grant: an indexed folder no grant covers is still refused | [`an_indexed_folder_is_not_a_granted_one_and_a_granted_one_is_not_an_indexed_one`] |
//! | where the list is kept follows the rule an index follows | [`where_the_list_is_kept_follows_the_rule_an_index_follows`] |

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{Authorised, Call, Given, Grant, Grantee, Grants, NotAuthorised, Reach};
use alo_files::{OnThisMachine, Resolving, Touching};
use alo_finding::{Index, Indexed, NotIndexed, Searched, finding_verbs, finding_words};
use alo_strings::Strings;

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The agent everything here is granted to.
fn agent() -> Grantee {
    Grantee::named("@finding")
}

/// The words this machine reads.
fn in_english() -> Strings {
    let mut vocabulary = finding_words().unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A folder of this test's own, resolved, so that what is indexed and what
/// is granted are spelled the way this machine spells them.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-finding-indexed-{}-{what}-{}",
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

/// A few known files: two whose names hold *march*, one that does not.
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

/// How many read calls this thread has made, where the kernel keeps the
/// number per thread; [`None`] on a host that does not. Per thread rather
/// than per process, because the other tests in this file run beside this
/// one and read files of their own.
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

/// A grant to the agent over this folder, made at noon and lasting an hour.
fn granting(folder: &Path) -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            agent().as_str(),
            Reach::Folder(folder.to_path_buf()),
            noon(),
            Duration::from_secs(60 * 60),
        )
        .unwrap(),
    );
    grants
}

/// **A folder kept is on the list, and its index comes back from the
/// disk** — read again by a second reader of the same data home, the way a
/// daemon and a file manager are two readers of one list; and it still
/// comes back after the folder itself is gone, with no file opened, which
/// is what *from the index alone* means.
#[test]
fn a_folder_kept_is_on_the_list_and_its_index_comes_back_from_the_disk_and_not_from_a_walk() {
    let data_home = a_data_home("kept");
    let documents = a_folder_of_our_own("documents");
    a_library(&documents);
    let index = Index::of(&documents, noon()).unwrap();
    assert_eq!(index.opened, 3);

    let mut indexed = the_list_under(&data_home);
    assert!(
        indexed.folders().is_empty(),
        "nothing has been asked for yet"
    );
    assert!(!indexed.holds(&documents));
    indexed.keep(&index).unwrap();
    assert_eq!(indexed.folders(), std::slice::from_ref(&documents));
    assert!(indexed.holds(&documents));
    assert!(indexed.where_index_of(&documents).is_file());
    assert!(
        Indexed::where_kept(Some(data_home.as_os_str()), None)
            .unwrap()
            .is_file()
    );

    // A second reader of the same data home sees the same list.
    let another_reader = the_list_under(&data_home);
    assert_eq!(another_reader, indexed);

    // The folder is gone; the index is not.
    fs::remove_dir_all(&documents).unwrap();
    let back = another_reader.index_of(&documents).unwrap();
    assert_eq!(back.of, documents);
    assert_eq!(back.entries, index.entries);
    assert_eq!(
        back.opened, 0,
        "read from its file, and no file under the folder was opened"
    );
    assert!(matches!(
        Index::of(&documents, noon()),
        Err(NotIndexed::NotWalked { .. })
    ));

    // Keeping the same folder again replaces the index and leaves the list
    // as it was: one folder, once.
    indexed.keep(&back).unwrap();
    assert_eq!(indexed.folders(), std::slice::from_ref(&documents));
    assert_eq!(the_list_under(&data_home).folders(), [documents]);

    let _ = fs::remove_dir_all(&data_home);
}

/// **A folder never indexed is refused in words, and the folder is never
/// walked.** Asked about a folder that is there and full of files, the
/// answer is *never indexed* and not an index; where the kernel counts this
/// process's reads, the count does not move while a walk of the same folder
/// moves it by at least one per file. Asked about a folder that is not
/// there at all, the answer is the same *never indexed* rather than *could
/// not be indexed*, which is what a walk would have said.
#[test]
fn a_folder_never_indexed_is_refused_in_words_and_never_walked() {
    let data_home = a_data_home("never");
    let pictures = a_folder_of_our_own("pictures");
    for number in 0..50 {
        fs::write(pictures.join(format!("{number}.txt")), b"words in a file").unwrap();
    }
    let indexed = the_list_under(&data_home);

    // What reading the counter itself costs, so the answer is held to
    // exactly that and not to a guess.
    let counted_once = reads_so_far();
    let counted_twice = reads_so_far();
    let before = reads_so_far();
    let refused = indexed.index_of(&pictures).unwrap_err();
    let after = reads_so_far();
    assert!(matches!(&refused, NotIndexed::NeverIndexed { at } if at == &pictures));
    let said = refused.said(&in_english()).into_text();
    assert!(said.contains("was never indexed"), "{said}");
    assert!(
        said.contains(&pictures.to_string_lossy().into_owned()),
        "{said}"
    );
    if let (Some(once), Some(twice), Some(before), Some(after)) =
        (counted_once, counted_twice, before, after)
    {
        let the_counter_itself = twice - once;
        assert_eq!(
            after - before,
            the_counter_itself,
            "no file was read to answer about an unindexed folder: the count moved by what \
             reading it costs, and fifty files would be fifty more"
        );
        let walked_before = reads_so_far().unwrap();
        let walked = Index::of(&pictures, noon()).unwrap();
        let walked_after = reads_so_far().unwrap();
        assert_eq!(walked.opened, 50);
        assert!(
            walked_after - walked_before >= 50,
            "a walk of the same folder reads every file, and the count shows it"
        );
    }

    let gone = a_folder_of_our_own("gone");
    fs::remove_dir_all(&gone).unwrap();
    assert!(matches!(
        indexed.index_of(&gone),
        Err(NotIndexed::NeverIndexed { .. })
    ));
    assert!(matches!(
        Index::of(&gone, noon()),
        Err(NotIndexed::NotWalked { .. })
    ));

    assert!(matches!(
        indexed.index_of(Path::new("Pictures")),
        Err(NotIndexed::NotAbsolute { .. })
    ));

    let _ = fs::remove_dir_all(&pictures);
    let _ = fs::remove_dir_all(&data_home);
}

/// **A folder forgotten has its index file removed with it**, the other
/// folder's index stays, the list on the disk names only the other, and
/// forgetting a folder that is not on the list is refused — including one
/// that was, once.
#[test]
fn a_folder_forgotten_has_its_index_file_removed_with_it() {
    let data_home = a_data_home("forgotten");
    let documents = a_folder_of_our_own("documents");
    let pictures = a_folder_of_our_own("pictures");
    a_library(&documents);
    a_library(&pictures);
    let mut indexed = the_list_under(&data_home);
    indexed
        .keep(&Index::of(&documents, noon()).unwrap())
        .unwrap();
    indexed
        .keep(&Index::of(&pictures, noon()).unwrap())
        .unwrap();
    assert_eq!(indexed.folders(), [documents.clone(), pictures.clone()]);
    let documents_file = indexed.where_index_of(&documents);
    let pictures_file = indexed.where_index_of(&pictures);
    assert!(documents_file.is_file() && pictures_file.is_file());

    indexed.forget(&documents).unwrap();
    assert!(
        !documents_file.exists(),
        "the index file went with the folder"
    );
    assert!(pictures_file.is_file(), "the other folder's did not");
    assert_eq!(indexed.folders(), std::slice::from_ref(&pictures));
    assert_eq!(
        the_list_under(&data_home).folders(),
        std::slice::from_ref(&pictures)
    );
    assert!(matches!(
        indexed.index_of(&documents),
        Err(NotIndexed::NeverIndexed { .. })
    ));
    assert!(indexed.index_of(&pictures).is_ok());

    let again = indexed.forget(&documents).unwrap_err();
    assert!(matches!(again, NotIndexed::NeverIndexed { .. }), "{again}");
    assert!(matches!(
        indexed.forget(Path::new("Documents")),
        Err(NotIndexed::NotAbsolute { .. })
    ));

    // An index file already gone is not a refusal: the folder is still
    // forgotten.
    fs::remove_file(&pictures_file).unwrap();
    indexed.forget(&pictures).unwrap();
    assert!(indexed.folders().is_empty());
    assert!(the_list_under(&data_home).folders().is_empty());

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&pictures);
    let _ = fs::remove_dir_all(&data_home);
}

/// **The list is written whole or not at all, and what is not a list is
/// refused whole.** A list that cannot be written leaves the list on the
/// disk and in hand as they were, with no half-written sibling beside it; a
/// file that is not a list is refused with the reason rather than read as an
/// empty list, because an empty list would say every folder was never
/// indexed while its index sat on the disk.
#[test]
fn the_list_is_written_whole_or_not_at_all_and_what_is_not_a_list_is_refused() {
    let data_home = a_data_home("whole");
    let documents = a_folder_of_our_own("documents");
    a_library(&documents);
    let index = Index::of(&documents, noon()).unwrap();
    let mut indexed = the_list_under(&data_home);
    indexed.keep(&index).unwrap();
    let list = Indexed::where_kept(Some(data_home.as_os_str()), None).unwrap();
    let written = fs::read_to_string(&list).unwrap();
    assert_eq!(written.lines().count(), 2, "{written}");
    assert!(written.starts_with("{\"format\":1}\n"), "{written}");

    // The list cannot be written: its directory has been replaced by a file.
    let directory = list.parent().unwrap().to_path_buf();
    let kept_aside = data_home.join("aside");
    fs::rename(&directory, &kept_aside).unwrap();
    fs::write(&directory, b"in the way").unwrap();
    let pictures = a_folder_of_our_own("pictures");
    a_library(&pictures);
    let refused = indexed
        .keep(&Index::of(&pictures, noon()).unwrap())
        .unwrap_err();
    assert!(
        matches!(refused, NotIndexed::NotKept { .. }),
        "the index is written before the list, so it is the index that refuses: {refused}"
    );
    assert_eq!(
        indexed.folders(),
        std::slice::from_ref(&documents),
        "the list in hand is as it was"
    );
    fs::remove_file(&directory).unwrap();
    fs::rename(&kept_aside, &directory).unwrap();
    assert_eq!(
        fs::read_to_string(&list).unwrap(),
        written,
        "and so is the list on the disk"
    );
    assert!(!directory.join("folders.list.new").exists());

    // What is not a list is refused, with the reason.
    fs::write(&list, "hello\n").unwrap();
    let refused = Indexed::read_from(Some(data_home.as_os_str()), None).unwrap_err();
    assert!(
        matches!(&refused, NotIndexed::NotAList { at, .. } if at == &list),
        "{refused}"
    );
    let said = refused.said(&in_english()).into_text();
    assert!(said.contains("is not a list of indexed folders"), "{said}");
    assert!(said.contains("the first line is not a list's"), "{said}");

    // A torn one too: the head and half a line.
    fs::write(&list, "{\"format\":1}\n{\"fold").unwrap();
    assert!(matches!(
        Indexed::read_from(Some(data_home.as_os_str()), None),
        Err(NotIndexed::NotAList { .. })
    ));

    // And a list from a later version.
    fs::write(&list, "{\"format\":2}\n").unwrap();
    let later = Indexed::read_from(Some(data_home.as_os_str()), None).unwrap_err();
    assert!(later.to_string().contains("format 2"), "{later}");

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&pictures);
    let _ = fs::remove_dir_all(&data_home);
}

/// **The list holds folders and nothing the record does**, so it answers
/// the same whoever asked: the file on the disk has a format and folders in
/// it and no other field, and the way in takes the two variables and no
/// caller, no grant and no name — there is no argument through which an
/// agent and a person could be told apart.
#[test]
fn the_list_holds_folders_and_nothing_the_record_does() {
    let read_from: fn(Option<&OsStr>, Option<&OsStr>) -> Result<Indexed, NotIndexed> =
        Indexed::read_from;
    let index_of: fn(&Indexed, &Path) -> Result<Index, NotIndexed> = Indexed::index_of;
    let _ = (read_from, index_of);

    let data_home = a_data_home("record");
    let documents = a_folder_of_our_own("documents");
    a_library(&documents);
    let mut indexed = the_list_under(&data_home);
    indexed
        .keep(&Index::of(&documents, noon()).unwrap())
        .unwrap();
    let list = Indexed::where_kept(Some(data_home.as_os_str()), None).unwrap();
    for (number, line) in fs::read_to_string(&list).unwrap().lines().enumerate() {
        let fields: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(line).unwrap();
        let named: Vec<&String> = fields.keys().collect();
        if number == 0 {
            assert_eq!(named, ["format"]);
        } else {
            assert_eq!(named, ["folder"]);
        }
    }

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&data_home);
}

/// **The list is not a grant.** A folder on the list that no grant covers
/// is refused by the grants before the index is asked; a folder a grant
/// covers that is not on the list has no index to hand the door, and the
/// list says so; and when both hold, the index the list gave is exactly
/// what `Searched::of` takes — unchanged, and still checking the index is
/// the granted folder's.
#[test]
fn an_indexed_folder_is_not_a_granted_one_and_a_granted_one_is_not_an_indexed_one() {
    let strings = in_english();
    let data_home = a_data_home("grants");
    let documents = a_folder_of_our_own("documents");
    let pictures = a_folder_of_our_own("pictures");
    a_library(&documents);
    a_library(&pictures);
    let mut indexed = the_list_under(&data_home);
    indexed
        .keep(&Index::of(&documents, noon()).unwrap())
        .unwrap();
    let grants = granting(&pictures);

    // Indexed, and not granted: the grants refuse, and the index is never
    // asked.
    let refused =
        Authorised::read(&searching(&documents, "march"), &agent(), &grants, noon()).unwrap_err();
    assert!(matches!(refused.why(), NotAuthorised::NotGranted(_)));
    assert!(
        indexed.holds(&documents),
        "being indexed changed nothing about that"
    );

    // Granted, and not indexed: the door is open and there is nothing to
    // hand it.
    let authorised =
        Authorised::read(&searching(&pictures, "march"), &agent(), &grants, noon()).unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap();
    let no_index = indexed.index_of(&pictures).unwrap_err();
    assert!(
        matches!(no_index, NotIndexed::NeverIndexed { .. }),
        "{no_index}"
    );
    // What the caller has is the other folder's index, and the door refuses
    // it for this folder, as it did before there was a list.
    let the_other = indexed.index_of(&documents).unwrap();
    let searched = Searched::of(touching, &the_other);
    assert!(searched.answer().is_none());
    assert!(
        searched
            .not_answered()
            .unwrap()
            .said(&strings)
            .text()
            .contains("not of"),
        "{}",
        searched.not_answered().unwrap().said(&strings)
    );

    // Both: the index the list gave is what the door takes.
    indexed
        .keep(&Index::of(&pictures, noon()).unwrap())
        .unwrap();
    let authorised =
        Authorised::read(&searching(&pictures, "march"), &agent(), &grants, noon()).unwrap();
    let touching = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap();
    let index = indexed.index_of(&pictures).unwrap();
    let searched = Searched::of(touching, &index);
    let found: Vec<&str> = searched
        .answer()
        .unwrap()
        .found
        .iter()
        .map(|entry| entry.below.as_str())
        .collect();
    assert_eq!(found, ["march-notes.txt", "2026/march.pdf"]);

    let _ = fs::remove_dir_all(&documents);
    let _ = fs::remove_dir_all(&pictures);
    let _ = fs::remove_dir_all(&data_home);
}

/// **Where the list is kept follows the rule an index follows**: beside the
/// indexes under `$XDG_DATA_HOME`, under `$HOME/.local/share` when that
/// says nothing, and nowhere for a session with no home — in which case
/// there is no list to read, either.
#[test]
fn where_the_list_is_kept_follows_the_rule_an_index_follows() {
    let documents = Path::new("/home/ada/Documents");
    let list =
        Indexed::where_kept(Some(OsStr::new("/data/ada")), Some(OsStr::new("/home/ada"))).unwrap();
    let index = Index::where_kept(
        Some(OsStr::new("/data/ada")),
        Some(OsStr::new("/home/ada")),
        documents,
    )
    .unwrap();
    assert_eq!(list.parent(), index.parent());
    assert_eq!(list, Path::new("/data/ada/alo/finding/folders.list"));
    assert_eq!(
        Indexed::where_kept(None, Some(OsStr::new("/home/ada"))).unwrap(),
        Path::new("/home/ada/.local/share/alo/finding/folders.list")
    );
    assert!(matches!(
        Indexed::where_kept(None, None),
        Err(NotIndexed::NowhereToKeepIt)
    ));
    assert!(matches!(
        Indexed::read_from(Some(OsStr::new("data")), None),
        Err(NotIndexed::NowhereToKeepIt)
    ));

    // A data home that is there and holds no list yet is an empty list, not
    // a refusal.
    let data_home = a_data_home("empty");
    let indexed = the_list_under(&data_home);
    assert!(indexed.folders().is_empty());
    assert!(
        !Indexed::where_kept(Some(data_home.as_os_str()), None)
            .unwrap()
            .exists()
    );
    let _ = fs::remove_dir_all(&data_home);
}
