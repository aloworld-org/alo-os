//! Search your own files, against the real disk.
//!
//! The unit tests hold each piece against text a test wrote out. These build
//! a folder of known files on this machine, index it, and ask — which is the
//! whole of the plan's acceptance for the index: *by name, kind, date and
//! contents, from the index alone, never by walking again, on this machine in
//! a file the person owns, and incremental.*
//!
//! | The promise | The test |
//! |---|---|
//! | a folder is indexed by name, kind, date and contents, and a query over any of the four is answered | [`a_folder_is_indexed_by_name_kind_date_and_contents_and_answers_each`] |
//! | a kind is read from the bytes, and a `.pdf` that is a text file is text | [`a_kind_is_read_from_the_bytes_and_a_pdf_that_is_a_text_file_is_text`] |
//! | the answer comes from the index alone, checked by removing the folder and asking | [`the_answer_comes_from_the_index_alone_after_the_folder_is_gone`] |
//! | the index is on this machine, in a file the person owns under their own directory, in the contract's format | [`the_index_is_kept_in_a_file_under_the_persons_own_directory_in_the_contract_format`] |
//! | a file unchanged since last time is not read again, with the reads counted | [`a_file_unchanged_since_last_time_is_not_read_again`] |
//! | a folder that is not there, is a file, or is not named from the root is refused in words | [`a_folder_that_is_not_there_or_is_a_file_or_is_relative_is_refused_in_words`] |
//! | a file that is not an index, is not there, or is another folder's is refused | [`a_file_that_is_not_an_index_or_not_there_or_another_folders_is_refused`] |
//! | a link is an entry that is a link, and is never followed | [`a_link_is_indexed_as_a_link_and_never_followed`] |

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_finding::{Contents, Entry, Index, Kind, Moment, NotIndexed, Query, finding_words};
use alo_strings::Strings;

/// A folder of this test's own, under this machine's temporary directory,
/// named after the test so a leftover says which test left it.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-finding-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    folder.canonicalize().unwrap()
}

/// A small library of known files: two text files, one of them named
/// `.pdf`; a real PDF; a JPEG; an empty file; and an empty folder.
fn a_library(root: &Path) {
    fs::create_dir_all(root.join("2026").join("April")).unwrap();
    fs::write(
        root.join("notes.txt"),
        b"Dear Anna, the Contract from before the summer.",
    )
    .unwrap();
    fs::write(
        root.join("2026").join("march.pdf"),
        b"%PDF-1.4\n1 0 obj << /Type /Catalog >> endobj\n%%EOF\n",
    )
    .unwrap();
    fs::write(
        root.join("2026").join("not-really.pdf"),
        b"a shopping list, not a pdf: bread, milk",
    )
    .unwrap();
    fs::write(
        root.join("2026").join("photo.jpg"),
        b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00\x01",
    )
    .unwrap();
    fs::write(root.join("empty.bin"), b"").unwrap();
}

/// A time, as seconds since the epoch.
fn at(secs: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
}

/// The paths below the folder of these entries, in the order given.
fn below(found: &[&Entry]) -> Vec<String> {
    found.iter().map(|entry| entry.below.clone()).collect()
}

/// Both crates' words, with nothing translated.
fn in_english() -> Strings {
    let mut vocabulary = finding_words().unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// Sets when this file was last written.
fn written_at(path: &Path, when: SystemTime) {
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(when)
        .unwrap();
}

/// **A folder is indexed by name, kind, date and contents, and a query over
/// any of the four is answered** — the date from the filesystem, checked
/// against what the filesystem says about each path.
#[test]
fn a_folder_is_indexed_by_name_kind_date_and_contents_and_answers_each() {
    let root = a_folder_of_our_own("library");
    a_library(&root);
    written_at(&root.join("notes.txt"), at(1_000_000_000));

    let index = Index::of(&root).unwrap();
    assert_eq!(index.of, root);
    assert!(index.covered.is_everything(), "{:?}", index.covered);
    assert_eq!(
        index.opened, 5,
        "five files were read, two folders were not"
    );
    assert_eq!(
        below(&index.entries.iter().collect::<Vec<_>>()),
        [
            "2026",
            "empty.bin",
            "notes.txt",
            "2026/April",
            "2026/march.pdf",
            "2026/not-really.pdf",
            "2026/photo.jpg",
        ]
    );

    // By name, whichever case either side is in.
    assert_eq!(
        below(&index.find(&Query::named("march"))),
        ["2026/march.pdf"]
    );
    assert_eq!(
        below(&index.find(&Query::named(".PDF"))),
        ["2026/march.pdf", "2026/not-really.pdf"]
    );
    assert!(index.find(&Query::named("invoice")).is_empty());

    // By kind, which is the bytes and not the name.
    assert_eq!(
        below(&index.find(&Query::of_kind(Kind::Pdf))),
        ["2026/march.pdf"]
    );
    assert_eq!(
        below(&index.find(&Query::of_kind(Kind::Text))),
        ["notes.txt", "2026/not-really.pdf"]
    );
    assert_eq!(
        below(&index.find(&Query::of_kind(Kind::Jpeg))),
        ["2026/photo.jpg"]
    );
    assert_eq!(
        below(&index.find(&Query::of_kind(Kind::Empty))),
        ["empty.bin"]
    );
    assert_eq!(
        below(&index.find(&Query::of_kind(Kind::Folder))),
        ["2026", "2026/April"]
    );

    // By date, from the filesystem: one file was written in 2001 and the
    // rest just now.
    assert_eq!(
        below(&index.find(&Query::changed_before(at(1_500_000_000)))),
        ["notes.txt"]
    );
    let recent = index.find(&Query::changed_since(at(1_500_000_000)));
    assert_eq!(recent.len(), 6, "{:?}", below(&recent));
    // Every file's time is the filesystem's, to the nanosecond. A folder's
    // own time is not compared: on NTFS it settles a moment after a write
    // inside it, and a folder is not what a person searches by date.
    for entry in index.entries.iter().filter(|entry| entry.kind.is_a_file()) {
        let on_disk = fs::symlink_metadata(index.where_is(entry))
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(entry.modified, Moment::of(on_disk), "{}", entry.below);
    }

    // By contents, which are the words in the file.
    assert_eq!(
        below(&index.find(&Query::saying("contract summer"))),
        ["notes.txt"]
    );
    assert_eq!(below(&index.find(&Query::saying("ANNA"))), ["notes.txt"]);
    assert_eq!(
        below(&index.find(&Query::saying("bread"))),
        ["2026/not-really.pdf"]
    );
    assert_eq!(
        below(&index.find(&Query::saying("pdf"))),
        ["2026/not-really.pdf"],
        "the word is in the text file; the real PDF's words were not read"
    );
    assert!(index.find(&Query::saying("contract bread")).is_empty());

    // And any two at once.
    assert_eq!(
        below(&index.find(&Query::named("pdf").and_saying("milk"))),
        ["2026/not-really.pdf"]
    );
    assert!(
        index
            .find(&Query::of_kind(Kind::Pdf).and_saying("milk"))
            .is_empty()
    );

    let _ = fs::remove_dir_all(&root);
}

/// **A kind is read from the file's own bytes, not its extension**: a `.pdf`
/// that is a text file is text and its words are indexed; a `.txt` that is a
/// PNG is a PNG; a real PDF is a PDF whose words are not read, and says so.
#[test]
fn a_kind_is_read_from_the_bytes_and_a_pdf_that_is_a_text_file_is_text() {
    let root = a_folder_of_our_own("kinds");
    a_library(&root);
    fs::write(
        root.join("picture.txt"),
        b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0DIHDR",
    )
    .unwrap();
    fs::write(root.join("archive.docx"), b"PK\x03\x04\x14\x00\x06\x00").unwrap();

    let index = Index::of(&root).unwrap();
    let entry = |name: &str| {
        index
            .entries
            .iter()
            .find(|entry| entry.name() == name)
            .unwrap_or_else(|| panic!("{name}"))
    };

    let text_named_pdf = entry("not-really.pdf");
    assert_eq!(text_named_pdf.kind, Kind::Text);
    assert!(text_named_pdf.contents.say("shopping"));

    let real_pdf = entry("march.pdf");
    assert_eq!(real_pdf.kind, Kind::Pdf);
    assert_eq!(real_pdf.contents, Contents::NotText);
    let strings = in_english();
    let said = real_pdf.contents.said(&strings).unwrap();
    assert!(!said.is_a_bug(), "{said}");
    assert_eq!(said.text(), "Not a kind of file whose words can be read.");
    assert_eq!(real_pdf.kind.said(&strings).text(), "A PDF document");

    assert_eq!(entry("picture.txt").kind, Kind::Png);
    assert_eq!(entry("archive.docx").kind, Kind::Zip);
    assert_eq!(entry("photo.jpg").kind, Kind::Jpeg);
    assert_eq!(entry("empty.bin").kind, Kind::Empty);
    assert_eq!(entry("2026").kind, Kind::Folder);
    assert_eq!(entry("2026").contents, Contents::NotAFile);

    let _ = fs::remove_dir_all(&root);
}

/// **The answer comes from the index alone, never by walking again.** The
/// folder is removed, and every one of the four questions is still answered
/// — from the index in memory and from the index read back from its file.
/// Indexing *again* is what walks, and is refused.
#[test]
fn the_answer_comes_from_the_index_alone_after_the_folder_is_gone() {
    let root = a_folder_of_our_own("gone");
    a_library(&root);
    let kept = a_folder_of_our_own("gone-kept").join("library.index");

    let index = Index::of(&root).unwrap();
    index.kept_at(&kept).unwrap();
    fs::remove_dir_all(&root).unwrap();
    assert!(!root.exists());

    let back = Index::read_from(&kept, &root).unwrap();
    for index in [&index, &back] {
        assert_eq!(below(&index.find(&Query::named("notes"))), ["notes.txt"]);
        assert_eq!(
            below(&index.find(&Query::of_kind(Kind::Pdf))),
            ["2026/march.pdf"]
        );
        assert_eq!(index.find(&Query::changed_since(at(1))).len(), 7);
        assert_eq!(below(&index.find(&Query::saying("summer"))), ["notes.txt"]);
    }

    match index.again() {
        Err(NotIndexed::NotWalked { at, .. }) => assert_eq!(at, root),
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(kept.parent().unwrap());
}

/// **The index is on this machine, in a file the person owns, under their
/// own directory, in the format the contract describes.** `$XDG_DATA_HOME`
/// stands in for the person's directory; the file is one JSON line saying
/// what it is and one per entry; on a Unix host it is readable by its owner
/// alone; and what is read back is what was written.
#[test]
fn the_index_is_kept_in_a_file_under_the_persons_own_directory_in_the_contract_format() {
    let root = a_folder_of_our_own("kept");
    a_library(&root);
    let data_home = a_folder_of_our_own("data-home");

    let at = Index::where_kept(Some(data_home.as_os_str()), None, &root).unwrap();
    assert!(
        at.starts_with(data_home.join("alo").join("finding")),
        "{}",
        at.display()
    );
    assert_eq!(at.extension().unwrap(), "index");
    assert!(!at.exists(), "nothing has been kept yet");

    let index = Index::of(&root).unwrap();
    index.kept_at(&at).unwrap();

    let text = fs::read_to_string(&at).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 1 + index.entries.len());
    let head: serde_json::Value = serde_json::from_str(lines.first().unwrap()).unwrap();
    assert_eq!(head.get("format"), Some(&serde_json::json!(1)));
    assert_eq!(head.get("of"), Some(&serde_json::json!(root)));
    assert_eq!(
        head.get("covered").and_then(|covered| covered.get("whole")),
        Some(&serde_json::json!(true))
    );
    for line in lines.iter().skip(1) {
        let entry: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(
            entry.get("format").is_none(),
            "only the head says what the file is"
        );
        for field in ["below", "kind", "bytes", "modified", "contents"] {
            assert!(entry.get(field).is_some(), "{field} in {line}");
        }
        assert!(!line.contains('\n'));
    }
    let notes = lines
        .iter()
        .find(|line| line.contains("\"notes.txt\""))
        .unwrap();
    assert!(notes.contains(r#""kind":"text""#), "{notes}");
    assert!(notes.contains(r#""were":"read""#), "{notes}");
    assert!(notes.contains(r#""contract""#), "{notes}");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&at).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    let back = Index::read_from(&at, &root).unwrap();
    assert_eq!(back.of, index.of);
    assert_eq!(back.covered, index.covered);
    assert_eq!(back.entries, index.entries);
    assert_eq!(back.opened, 0);

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&data_home);
}

/// **A file unchanged since last time is not read again**, counted by the
/// index itself: the first index reads every file, the next reads none, a
/// file whose bytes changed is read, a file whose time changed is read, and
/// a new file is read — each exactly once.
#[test]
fn a_file_unchanged_since_last_time_is_not_read_again() {
    let root = a_folder_of_our_own("incremental");
    a_library(&root);

    let first = Index::of(&root).unwrap();
    assert_eq!(first.opened, 5);

    let unchanged = first.again().unwrap();
    assert_eq!(unchanged.opened, 0);
    assert_eq!(
        unchanged.find(&Query::saying("summer")).len(),
        1,
        "the words were kept"
    );

    fs::write(
        root.join("notes.txt"),
        b"Dear Anna, the contract is signed.",
    )
    .unwrap();
    let one_rewritten = unchanged.again().unwrap();
    assert_eq!(one_rewritten.opened, 1);
    assert!(one_rewritten.find(&Query::saying("summer")).is_empty());
    assert_eq!(
        below(&one_rewritten.find(&Query::saying("signed"))),
        ["notes.txt"]
    );

    written_at(&root.join("2026").join("march.pdf"), at(1_000_000_000));
    let one_touched = one_rewritten.again().unwrap();
    assert_eq!(one_touched.opened, 1);

    fs::write(root.join("2026").join("april.txt"), b"new").unwrap();
    let one_new = one_touched.again().unwrap();
    assert_eq!(one_new.opened, 1);
    assert_eq!(
        below(&one_new.find(&Query::saying("new"))),
        ["2026/april.txt"]
    );

    let _ = fs::remove_dir_all(&root);
}

/// **A folder that is not there, is a file, or is not named from the root is
/// refused in words** that name it — and the refusal is the whole question,
/// with no index to search.
#[test]
fn a_folder_that_is_not_there_or_is_a_file_or_is_relative_is_refused_in_words() {
    let root = a_folder_of_our_own("refused");
    fs::write(root.join("a-file.txt"), b"not a folder").unwrap();
    let strings = in_english();

    let missing = root.join("Missing");
    match Index::of(&missing) {
        Err(refusal @ NotIndexed::NotWalked { .. }) => {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.text().contains("Missing"), "{said}");
            assert!(
                said.text().starts_with(&missing.display().to_string()),
                "{said}"
            );
        }
        other => panic!("{other:?}"),
    }
    match Index::of(&root.join("a-file.txt")) {
        Err(refusal @ NotIndexed::NotWalked { .. }) => {
            let said = refusal.said(&strings);
            assert!(said.text().contains("a-file.txt"), "{said}");
            assert!(
                !said.text().contains("files."),
                "the file half's sentence, not its key: {said}"
            );
        }
        other => panic!("{other:?}"),
    }
    match Index::of(Path::new("Documents")) {
        Err(refusal @ NotIndexed::NotAbsolute { .. }) => {
            let said = refusal.said(&strings);
            assert!(said.text().starts_with("Documents does not say"), "{said}");
        }
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&root);
}

/// **A file that is not an index, is not there, or is another folder's is
/// refused**, each in its own words, and none of them is searched.
#[test]
fn a_file_that_is_not_an_index_or_not_there_or_another_folders_is_refused() {
    let root = a_folder_of_our_own("not-an-index");
    a_library(&root);
    let other = a_folder_of_our_own("another");
    let strings = in_english();

    match Index::read_from(&root.join("nothing.index"), &root) {
        Err(refusal @ NotIndexed::NotOpened { .. }) => {
            assert!(refusal.said(&strings).text().contains("nothing.index"));
        }
        other => panic!("{other:?}"),
    }

    let garbage = root.join("garbage.index");
    fs::write(&garbage, b"this is somebody's shopping list\n").unwrap();
    match Index::read_from(&garbage, &root) {
        Err(refusal @ NotIndexed::NotAnIndex { .. }) => {
            let said = refusal.said(&strings);
            assert!(said.text().contains("garbage.index"), "{said}");
            assert!(said.text().contains("first line"), "{said}");
        }
        other => panic!("{other:?}"),
    }

    let torn = root.join("torn.index");
    let whole = {
        let at = root.join("whole.index");
        Index::of(&other).unwrap().kept_at(&at).unwrap();
        fs::read_to_string(at).unwrap()
    };
    fs::write(&torn, whole.trim_end().as_bytes()).unwrap();
    assert!(
        Index::read_from(&torn, &other).is_ok(),
        "a missing final newline is not a tear"
    );
    let (cut, _) = whole.split_at(whole.len() - 3);
    fs::write(&torn, cut.as_bytes()).unwrap();
    assert!(matches!(
        Index::read_from(&torn, &other),
        Err(NotIndexed::NotAnIndex { .. })
    ));

    match Index::read_from(&root.join("whole.index"), &root) {
        Err(refusal @ NotIndexed::NotTheSame { .. }) => {
            let said = refusal.said(&strings);
            assert!(said.text().contains(&other.display().to_string()), "{said}");
            assert!(said.text().contains(&root.display().to_string()), "{said}");
        }
        other => panic!("{other:?}"),
    }

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&other);
}

/// **A link is an entry that is a link, and is never followed.** A link to a
/// file outside the folder holds words the index must not have; a link to a
/// folder outside it has things the index must not list.
#[cfg(unix)]
#[test]
fn a_link_is_indexed_as_a_link_and_never_followed() {
    let root = a_folder_of_our_own("links");
    let elsewhere = a_folder_of_our_own("elsewhere");
    fs::write(elsewhere.join("secret.txt"), b"the secret word is xyzzy").unwrap();
    fs::write(root.join("mine.txt"), b"my own words").unwrap();
    std::os::unix::fs::symlink(
        elsewhere.join("secret.txt"),
        root.join("looks-like-a-file.txt"),
    )
    .unwrap();
    std::os::unix::fs::symlink(&elsewhere, root.join("looks-like-a-folder")).unwrap();

    let index = Index::of(&root).unwrap();
    assert_eq!(
        below(&index.entries.iter().collect::<Vec<_>>()),
        ["looks-like-a-file.txt", "looks-like-a-folder", "mine.txt"]
    );
    for name in ["looks-like-a-file.txt", "looks-like-a-folder"] {
        let link = index
            .entries
            .iter()
            .find(|entry| entry.name() == name)
            .unwrap();
        assert_eq!(link.kind, Kind::Link, "{name}");
        assert_eq!(link.contents, Contents::NotAFile, "{name}");
    }
    assert!(index.find(&Query::saying("xyzzy")).is_empty());
    assert!(index.find(&Query::named("secret")).is_empty());
    assert_eq!(below(&index.find(&Query::of_kind(Kind::Link))).len(), 2);
    assert_eq!(index.opened, 1, "only the real file was read");

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&elsewhere);
}
