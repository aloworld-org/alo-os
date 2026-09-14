//! An index that fits in hand — the words of a large folder bounded and said,
//! against the real disk, on this machine.
//!
//! The plan's acceptance for task 13: *`alo-finding` holds a whole index of a
//! large folder in a bounded amount of memory per entry — words kept once per
//! file rather than once per occurrence, and a bound on how many distinct
//! words one entry keeps, said in `Contents` so that an answer can say a
//! file's words were not all kept — with a test that indexes a folder of long
//! text files and checks the bytes held in hand against the bytes of the
//! files; a search over a file whose words were bounded still finds every
//! word that was kept and says the words were not all kept, never* nothing
//! matched *quietly; the index file's `format` is still `1`, and if a field is
//! added it is added the way `made` was, so a reader from before still reads.*
//!
//! | The acceptance | The test |
//! |---|---|
//! | a folder of long text files, indexed, holds in hand no more per entry than its words once each and at most `MOST_WORDS` places, measured against the files' bytes, read back from its file the same | [`a_folder_of_long_text_files_is_held_in_hand_in_a_bounded_number_of_bytes_measured`] |
//! | a search finds every kept word and lists a bounded file it did not find as not all searched — through an index, a held set and a sentence | [`a_search_over_a_file_whose_words_were_bounded_finds_every_kept_word_and_says_the_rest`] |
//! | `format` is still `1`, the new field is `unkept` beside `words` and left out when nothing was, a reader from before still reads every entry, and a count that is not one is refused | [`the_index_file_is_format_one_and_a_reader_from_before_still_reads_it`] |
//!
//! The bytes held are counted from what the index holds — every list's and
//! every string's capacity, and the entries themselves — rather than asked of
//! the allocator, which a test here cannot replace without an `unsafe` block.
//! What the allocator adds on top of each allocation is named in the report
//! as an estimate, not measured here.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant, SystemTime};

use alo_finding::{Contents, Entry, Index, Indexed, MOST_WORDS, NotIndexed, Query, finding_words};
use alo_strings::{Said, Strings};
use serde::Deserialize;

/// A folder of this test's own, under this machine's temporary directory,
/// named after the test so a leftover says which test left it.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-finding-in-hand-{}-{what}-{}",
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

/// How many long letters and how many long logs the folder holds, and about
/// how many bytes each is — under the megabyte an index reads.
const LETTERS: usize = 6;
const LOGS: usize = 3;
const LONG: usize = 900_000;

/// How many different words a letter draws from, and how many different
/// identifiers a log says between its first and last word.
const VOCABULARY: usize = 30_000;
const IDENTIFIERS: usize = 100_000;

/// How many different words a log says: *begins*, *log* and its number,
/// the identifiers, and *finale*.
const WORDS_OF_A_LOG: usize = 3 + IDENTIFIERS + 1;

/// A word of the letters' language: syllables, the number spelled in base
/// twenty, at least two of them.
fn a_word(mut number: usize) -> String {
    const SYLLABLES: [&str; 20] = [
        "ka", "lo", "mi", "tu", "re", "sa", "no", "vi", "pe", "da", "zu", "ho", "fa", "ri", "go",
        "le", "wa", "xi", "bo", "ne",
    ];
    let mut word = String::new();
    for _ in 0..2 {
        word.push_str(SYLLABLES.get(number % 20).unwrap());
        number /= 20;
    }
    while number > 0 {
        word.push_str(SYLLABLES.get(number % 20).unwrap());
        number /= 20;
    }
    word
}

/// A long letter: words drawn from the vocabulary, the common ones far more
/// often than the rare, in sentences that begin with a capital — the shape
/// of prose, so that a word is said many times and kept once.
fn a_letter(seed: u64) -> String {
    let mut state = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
    let mut text = String::with_capacity(LONG + 64);
    let mut in_sentence = 0;
    while text.len() < LONG {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // Twenty-four random bits as a fraction of one, to the fourth power,
        // of the vocabulary: in fixed point, so the draw is the same on
        // every machine.
        let unit = u128::from(state >> 40);
        let drawn = (unit.pow(4) * VOCABULARY as u128) >> 96;
        let word = a_word(usize::try_from(drawn).unwrap().min(VOCABULARY - 1));
        if in_sentence == 0 {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                text.extend(first.to_uppercase());
                text.push_str(chars.as_str());
            }
        } else {
            text.push_str(&word);
        }
        in_sentence += 1;
        if in_sentence == 12 {
            text.push_str(". ");
            in_sentence = 0;
        } else {
            text.push(' ');
        }
    }
    text
}

/// A long log: *begins*, then a hundred thousand different identifiers,
/// then *finale* — more different words than an index keeps, so the first
/// ones are kept and the last are not.
fn a_log(which: usize) -> String {
    let mut text = format!("Begins log {which}\n");
    for n in 0..IDENTIFIERS {
        text.push_str(&format!("id{n:06}\n"));
    }
    text.push_str("finale\n");
    text
}

/// The folder: `LETTERS` long letters and `LOGS` long logs, and the paths
/// below it of each.
fn a_folder_of_long_text_files(root: &Path) -> (Vec<String>, Vec<String>) {
    let mut letters = Vec::new();
    for n in 0..LETTERS {
        let below = format!("letter-{n}.txt");
        fs::write(root.join(&below), a_letter(n as u64 + 1)).unwrap();
        letters.push(below);
    }
    let mut logs = Vec::new();
    for n in 0..LOGS {
        let below = format!("log-{n}.txt");
        fs::write(root.join(&below), a_log(n)).unwrap();
        logs.push(below);
    }
    (letters, logs)
}

/// The bytes this entry holds in hand: the entry itself, its path, and for
/// words the list's places and every word's own bytes — each counted by
/// capacity, so that room held and not used is counted too.
fn held_in_hand(entry: &Entry) -> usize {
    size_of::<Entry>()
        + entry.below.capacity()
        + match &entry.contents {
            Contents::Read { words } | Contents::NotAllKept { words, .. } => {
                words.capacity() * size_of::<String>()
                    + words.iter().map(String::capacity).sum::<usize>()
            }
            Contents::NotRead { why } => why.capacity(),
            _ => 0,
        }
}

/// The most one text entry may hold: itself, its path, `MOST_WORDS` places,
/// and no more bytes of words than the file has bytes — which, for text in
/// a script whose lower case is no longer than its upper, is what a set of
/// its words can take.
fn the_bound_for(entry: &Entry) -> usize {
    size_of::<Entry>()
        + entry.below.len()
        + MOST_WORDS * size_of::<String>()
        + usize::try_from(entry.bytes).unwrap()
}

/// An entry by its path below the folder.
fn entry<'a>(index: &'a Index, below: &str) -> &'a Entry {
    index
        .entries
        .iter()
        .find(|entry| entry.below == below)
        .unwrap()
}

/// **What each entry holds is bounded, and what the folder holds is measured
/// against its bytes.** Every word list and every word is held with no room
/// to spare, no entry keeps more than `MOST_WORDS` words, every text entry
/// holds no more than its words' own bytes and `MOST_WORDS` places, a
/// letter's words are all kept, a log's are not and it says how many — and
/// the index written and read back from its file holds the same. The bytes
/// and the time are printed for the report.
#[test]
fn a_folder_of_long_text_files_is_held_in_hand_in_a_bounded_number_of_bytes_measured() {
    let root = a_folder_of_our_own("measured");
    let (letters, logs) = a_folder_of_long_text_files(&root);

    let indexing = Instant::now();
    let index = Index::of(&root, noon()).unwrap();
    println!(
        "Index::of over {LETTERS} letters and {LOGS} logs of about {LONG} bytes each: {:?}",
        indexing.elapsed()
    );
    assert!(index.covered.whole);
    assert_eq!(index.opened, LETTERS + LOGS);

    let data_home = a_folder_of_our_own("measured-data-home");
    let at = Index::where_kept(Some(data_home.as_os_str()), None, &root).unwrap();
    index.kept_at(&at).unwrap();
    let back = Index::read_from(&at, &root).unwrap();
    assert_eq!(back.entries, index.entries);

    for (how, held) in [("made", &index), ("read back from its file", &back)] {
        assert_eq!(
            held.entries.capacity(),
            held.entries.len(),
            "{how}: the list of entries has no room to spare"
        );
        let mut bytes_of_letters = 0;
        let mut held_by_letters = 0;
        let mut bytes_of_logs = 0;
        let mut held_by_logs = 0;
        let mut words_of_letters = 0;
        let mut words_of_logs = 0;
        for entry in &held.entries {
            let words = entry.contents.words();
            assert!(words.len() <= MOST_WORDS, "{how}: {}", entry.below);
            if let Contents::Read { words } | Contents::NotAllKept { words, .. } = &entry.contents {
                assert_eq!(words.capacity(), words.len(), "{how}: {}", entry.below);
                for word in words {
                    assert_eq!(word.capacity(), word.len(), "{how}: {word}");
                }
            }
            let in_hand = held_in_hand(entry);
            assert!(
                in_hand <= the_bound_for(entry),
                "{how}: {} holds {in_hand} bytes, more than {}",
                entry.below,
                the_bound_for(entry)
            );
            let bytes = usize::try_from(entry.bytes).unwrap();
            if letters.contains(&entry.below) {
                bytes_of_letters += bytes;
                held_by_letters += in_hand;
                words_of_letters += words.len();
            } else {
                bytes_of_logs += bytes;
                held_by_logs += in_hand;
                words_of_logs += words.len();
            }
        }
        println!(
            "{how}: the letters are {bytes_of_letters} bytes on the disk and {held_by_letters} \
             bytes in hand for {words_of_letters} words kept; the logs are {bytes_of_logs} \
             bytes on the disk and {held_by_logs} bytes in hand for {words_of_logs} words kept; \
             the index file is {} bytes",
            fs::metadata(&at).unwrap().len()
        );
    }

    for below in &letters {
        let letter = entry(&index, below);
        assert!(
            matches!(letter.contents, Contents::Read { .. }),
            "prose keeps every word: {below} has {} words",
            letter.contents.words().len()
        );
        assert!(letter.contents.all_kept());
    }
    // Every different word of a log, held the way a letter's are, is what
    // the bound saved: the words it did not keep, each with its place.
    let every_word_of_a_log = size_of::<String>() * WORDS_OF_A_LOG
        + "id000000".len() * IDENTIFIERS
        + "begins".len()
        + "log".len()
        + "finale".len()
        + 1;
    for below in &logs {
        let log = entry(&index, below);
        assert!(
            matches!(
                log.contents,
                Contents::NotAllKept { unkept, .. } if unkept == WORDS_OF_A_LOG - MOST_WORDS
            ),
            "{below}: {:?} words kept",
            log.contents.words().len()
        );
        assert_eq!(log.contents.words().len(), MOST_WORDS);
        assert!(!log.contents.all_kept());
        println!(
            "{below}: {} bytes in hand with its words bounded; every different word held \
             would be about {every_word_of_a_log}",
            held_in_hand(log)
        );
        assert!(held_in_hand(log) < every_word_of_a_log);
    }

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&data_home);
}

/// The paths below the folder of these entries.
fn below<'a>(entries: impl IntoIterator<Item = &'a Entry>) -> Vec<String> {
    let mut below: Vec<String> = entries.into_iter().map(|e| e.below.clone()).collect();
    below.sort();
    below
}

/// The text of these sentences.
fn texts(said: &[Said]) -> Vec<String> {
    said.iter().map(|said| said.text().to_owned()).collect()
}

/// **A search over a file whose words were bounded finds every word that was
/// kept, and says the words were not all kept** — never *nothing matched*
/// quietly. The first word a log says and the last it kept are found; a word
/// it says past the bound is not, and each log is then listed beside the
/// answer as not all searched, with a sentence saying so; a log that fails
/// the query on its name is not listed, because its name was searched; a
/// word no file says leaves the letters out of that list, because their words
/// were all kept. The same is said through a set held in hand, and indexed
/// again the logs are not read and still say it.
#[test]
fn a_search_over_a_file_whose_words_were_bounded_finds_every_kept_word_and_says_the_rest() {
    let root = a_folder_of_our_own("searched");
    let (letters, logs) = a_folder_of_long_text_files(&root);
    let index = Index::of(&root, noon()).unwrap();
    let strings = Strings::of(finding_words().unwrap());

    // Kept: the first word, and the last identifier inside the bound —
    // `begins`, `log`, the log's number, then identifiers from the first.
    let last_kept = format!("id{:06}", MOST_WORDS - 4);
    for kept in ["begins", "id000000", last_kept.as_str()] {
        let answer = index.answer(&Query::saying(kept)).unwrap();
        assert_eq!(below(answer.found.iter().copied()), logs, "{kept}");
        assert!(answer.not_searched.not_all_kept.is_empty(), "{kept}");
    }

    // Past the bound: not found, and not *nothing matched*.
    let first_unkept = format!("id{:06}", MOST_WORDS - 3);
    for unkept in [first_unkept.as_str(), "finale"] {
        let answer = index.answer(&Query::saying(unkept)).unwrap();
        assert!(answer.found.is_empty(), "{unkept}");
        assert_eq!(
            below(answer.not_searched.not_all_kept.iter().copied()),
            logs,
            "{unkept}"
        );
        assert!(!answer.not_searched.is_nothing());
        assert!(
            texts(&answer.not_searched.said(&strings)).contains(
                &"3 files hold more different words than an index keeps, so not all of their \
                  words were searched."
                    .to_owned()
            ),
            "{unkept}"
        );
    }
    for below in &logs {
        let said = entry(&index, below).contents.said(&strings).unwrap();
        assert_eq!(
            said.text(),
            "More different words than the 50000 an index keeps, so only the first 50000 were \
             kept."
        );
    }

    // A word nobody says: the logs are listed, the letters are not.
    let nobody = index.answer(&Query::saying("zzzzzz")).unwrap();
    assert!(nobody.found.is_empty());
    assert_eq!(
        below(nobody.not_searched.not_all_kept.iter().copied()),
        logs
    );
    for letter in &letters {
        assert!(
            !nobody
                .not_searched
                .not_all_kept
                .iter()
                .any(|entry| &entry.below == letter)
        );
    }

    // The name was searched whole: a log that fails on its name is not
    // listed as not all searched, and a search by name lists nothing.
    let by_name = index
        .answer(&Query::named("letter").and_saying("finale"))
        .unwrap();
    assert!(by_name.found.is_empty());
    assert!(by_name.not_searched.not_all_kept.is_empty());
    let only_name = index.answer(&Query::named("log")).unwrap();
    assert_eq!(below(only_name.found.iter().copied()), logs);
    assert!(only_name.not_searched.is_nothing());

    // Through a set held in hand, the same.
    let data_home = a_folder_of_our_own("searched-data-home");
    let mut indexed = Indexed::read_from(Some(data_home.as_os_str()), None).unwrap();
    indexed.keep(&index).unwrap();
    let in_hand = indexed.in_hand();
    let everywhere = in_hand.answer(&Query::saying("finale")).unwrap();
    let (folder, held) = everywhere.answered().next().unwrap();
    assert_eq!(folder, &root);
    assert!(held.found.is_empty());
    assert_eq!(below(&held.not_searched.not_all_kept), logs);
    assert!(!held.answer().not_searched.is_nothing());

    // Indexed again, nothing is read, and the logs still say it.
    let again = index.again(noon()).unwrap();
    assert_eq!(again.opened, 0);
    assert_eq!(again.entries, index.entries);
    let answer = again.answer(&Query::saying("finale")).unwrap();
    assert_eq!(
        below(answer.not_searched.not_all_kept.iter().copied()),
        logs
    );

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&data_home);
}

/// Contents as a reader from before `unkept` existed spelled them: exactly
/// the shape `alo-finding` derived for them until this change.
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "were", rename_all = "kebab-case")]
enum ContentsFromBefore {
    /// Words.
    Read {
        /// The words.
        words: Vec<String>,
    },
    /// No reader.
    NotText,
    /// Not read.
    NotRead {
        /// Why.
        why: String,
    },
    /// Too big.
    TooBig {
        /// How big.
        bytes: u64,
    },
    /// Not a file.
    NotAFile,
}

/// An entry as a reader from before read it.
#[derive(Debug, Deserialize)]
struct EntryFromBefore {
    /// Where it is.
    below: String,
    /// Its words.
    contents: ContentsFromBefore,
}

/// **The file is still format `1`, and a reader from before still reads
/// it.** The head says `"format":1`; a log's entry is still `"were":"read"`
/// with its kept words and `unkept` beside them; a letter's entry has no
/// `unkept` at all; every entry line parses with the shape entries had before
/// this field, and the words it reads are the words kept. And a file whose
/// `unkept` is not a count is refused whole as not an index, rather than read
/// as nothing left out.
#[test]
fn the_index_file_is_format_one_and_a_reader_from_before_still_reads_it() {
    let root = a_folder_of_our_own("format");
    let (letters, logs) = a_folder_of_long_text_files(&root);
    let index = Index::of(&root, noon()).unwrap();
    let data_home = a_folder_of_our_own("format-data-home");
    let at = Index::where_kept(Some(data_home.as_os_str()), None, &root).unwrap();
    index.kept_at(&at).unwrap();
    let text = fs::read_to_string(&at).unwrap();

    let mut lines = text.lines();
    assert!(lines.next().unwrap().starts_with(r#"{"format":1,"#));
    let unkept = WORDS_OF_A_LOG - MOST_WORDS;
    let mut read = 0;
    for line in lines {
        let before: EntryFromBefore = serde_json::from_str(line).unwrap();
        let now = entry(&index, &before.below);
        if logs.contains(&before.below) {
            assert!(line.contains(r#""contents":{"were":"read","words":["#));
            assert!(
                line.ends_with(&format!(r#"],"unkept":{unkept}}}}}"#)),
                "{line:.80}"
            );
        } else {
            assert!(letters.contains(&before.below));
            assert!(!line.contains("unkept"));
        }
        assert_eq!(
            before.contents,
            ContentsFromBefore::Read {
                words: now.contents.words().to_vec()
            },
            "{}",
            before.below
        );
        read += 1;
    }
    assert_eq!(read, LETTERS + LOGS);

    for torn in ["\"many\"", "-1"] {
        let not_a_count = text.replacen(
            &format!(r#""unkept":{unkept}"#),
            &format!(r#""unkept":{torn}"#),
            1,
        );
        assert_ne!(not_a_count, text);
        fs::write(&at, not_a_count).unwrap();
        let refused = Index::read_from(&at, &root).unwrap_err();
        assert!(
            matches!(refused, NotIndexed::NotAnIndex { .. }),
            "{torn}: {refused:?}"
        );
    }

    let _ = fs::remove_dir_all(&root);
    let _ = fs::remove_dir_all(&data_home);
}
