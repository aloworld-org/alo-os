//! A search answers in the time a person will wait for one, and says what
//! it did not read — against the real disk, on this machine.
//!
//! The plan's acceptance for the second half of *without asking anything*:
//! a query over an index of ten thousand files answers by name and kind in
//! under a tenth of a second and by contents in under a second, **measured**
//! by building the index and timing the answer; the answer lists what was
//! not searched as a separate list beside the results; and a query that is
//! not a query is refused in words. The numbers each run measures are
//! printed, so that the report can publish them with the machine named —
//! a number with no machine beside it is a claim.
//!
//! | The promise | The test |
//! |---|---|
//! | ten thousand files answer by name and kind in a tenth of a second, and by contents in a second, timed | [`a_query_over_ten_thousand_files_answers_in_the_time_a_person_will_wait`] |
//! | what was not searched is a separate list beside the results, read from the index and not the disk | [`the_answer_lists_what_it_did_not_search_beside_what_it_found`] |
//! | a query that is not a query is refused in words, and nothing is searched | [`a_query_that_is_not_a_query_is_refused_in_words`] |

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use alo_finding::{A_NAME, A_SENTENCE, Entry, Index, Kind, NotAsked, Query, finding_words};
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

/// How many files the timed index holds.
const TEN_THOUSAND: usize = 10_000;

/// How many of them are PDFs, which have a kind and no words.
const PDFS: usize = 1_000;

/// Ten thousand files in a hundred folders: nine thousand letters of
/// twenty-odd words each, and a thousand PDFs. Every hundredth letter says
/// *contract*, and every letter names Anna.
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

/// The paths below the folder of these entries, in the order given.
fn below(found: &[&Entry]) -> Vec<String> {
    found.iter().map(|entry| entry.below.clone()).collect()
}

/// This crate's words, with nothing translated.
fn in_english() -> Strings {
    Strings::of(finding_words().unwrap())
}

/// One timed answer: the query, the bound it has to answer under, and how
/// many entries it has to find. Timed twice — by the answer's own clock and
/// by this test's around the call — and the slower is the one held to the
/// bound, so the answer cannot flatter itself.
fn timed(index: &Index, what: &str, query: &Query, under: Duration, finds: usize) -> Duration {
    let started = Instant::now();
    let answer = index.answer(query).unwrap();
    let around = started.elapsed();
    let took = around.max(answer.took);
    println!(
        "{what}: {} found in {took:?} (the answer's own clock said {:?}); the bound is {under:?}",
        answer.found.len(),
        answer.took
    );
    assert_eq!(answer.found.len(), finds, "{what}");
    assert!(
        took < under,
        "{what} took {took:?}, and the bound is {under:?}"
    );
    took
}

/// **A query over ten thousand files answers by name and kind in under a
/// tenth of a second, and by contents in under a second**, measured here
/// rather than asserted from elsewhere: the index is built from ten thousand
/// real files, and each answer is timed three times, cold first.
#[test]
fn a_query_over_ten_thousand_files_answers_in_the_time_a_person_will_wait() {
    let root = a_folder_of_our_own("ten-thousand");
    let writing = Instant::now();
    ten_thousand_files(&root);
    let indexing = Instant::now();
    let index = Index::of(&root).unwrap();
    println!(
        "ten thousand files written in {:?} and indexed in {:?}",
        indexing.duration_since(writing),
        indexing.elapsed()
    );
    assert_eq!(index.entries.len(), TEN_THOUSAND + 100);
    assert_eq!(index.opened, TEN_THOUSAND);
    assert!(index.covered.is_everything(), "{:?}", index.covered);

    let a_tenth = Duration::from_millis(100);
    let a_second = Duration::from_secs(1);
    for round in 1..=3 {
        println!("round {round}");
        timed(&index, "by name", &Query::named("letter-0999"), a_tenth, 10);
        timed(&index, "by kind", &Query::of_kind(Kind::Pdf), a_tenth, PDFS);
        timed(
            &index,
            "by contents",
            &Query::saying("Anna contract summer"),
            a_second,
            (TEN_THOUSAND - PDFS) / 100,
        );
        timed(
            &index,
            "by name, kind and contents together",
            &Query::named("letter")
                .and_of_kind(Kind::Text)
                .and_saying("garden lease"),
            a_second,
            TEN_THOUSAND - PDFS,
        );
    }

    let _ = fs::remove_dir_all(&root);
}

/// **The answer lists what was not searched beside what was found**, read
/// from the index and not from the disk: a search by words over a folder
/// holding a PDF and a file larger than an index reads finds neither, and
/// says so in two lists a person can open — and, first, which folder was
/// searched, so that nothing outside it is mistaken for searched. A search
/// by name over the same folder finds both and has nothing unsearched, and
/// an empty answer by name is *nothing matched*.
#[test]
fn the_answer_lists_what_it_did_not_search_beside_what_it_found() {
    let root = a_folder_of_our_own("not-read");
    fs::create_dir_all(root.join("2026")).unwrap();
    fs::write(
        root.join("notes.txt"),
        b"Dear Anna, the Contract from before the summer.",
    )
    .unwrap();
    fs::write(
        root.join("2026").join("contract.pdf"),
        b"%PDF-1.4\n1 0 obj << /Type /Catalog >> endobj\n%%EOF\n",
    )
    .unwrap();
    let big = "contract ".repeat(150_000);
    assert!(big.len() as u64 > alo_files::MOST_READ);
    fs::write(root.join("2026").join("contract-log.txt"), big).unwrap();

    let index = Index::of(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    let by_words = index.answer(&Query::saying("contract")).unwrap();
    assert_eq!(below(&by_words.found), ["notes.txt"]);
    assert_eq!(
        below(&by_words.not_searched.no_reader),
        ["2026/contract.pdf"]
    );
    assert_eq!(
        below(&by_words.not_searched.too_big),
        ["2026/contract-log.txt"]
    );
    assert!(by_words.not_searched.files_unread.is_empty());
    assert_eq!(by_words.not_searched.outside, root);
    assert!(!by_words.not_searched.is_nothing());

    let said = by_words.not_searched.said(&in_english());
    let texts: Vec<&str> = said.iter().map(alo_strings::Said::text).collect();
    assert_eq!(
        texts,
        [
            format!("Nothing outside {} was searched.", root.display()).as_str(),
            "One file is of a kind whose words cannot be read, so it was not searched by its \
             words.",
            "One file is larger than an index reads, so it was not searched by its words.",
        ]
    );
    for said in &said {
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
    }

    let by_name = index.answer(&Query::named("contract")).unwrap();
    assert_eq!(
        below(&by_name.found),
        ["2026/contract-log.txt", "2026/contract.pdf"]
    );
    assert!(
        by_name.not_searched.is_nothing(),
        "a name is searched on every file: {:?}",
        by_name.not_searched
    );

    let nothing = index.answer(&Query::named("invoice")).unwrap();
    assert!(nothing.found.is_empty());
    assert!(nothing.not_searched.is_nothing(), "nothing matched");
    assert_eq!(
        nothing.not_searched.said(&in_english()).len(),
        1,
        "the folder that was searched, and nothing else to say"
    );
}

/// **A query that is not a query is refused in words** — empty, more words
/// than a sentence, or a part longer than any name — and nothing is
/// searched: the refusal is the whole of the answer, not everything in the
/// index under a warning.
#[test]
fn a_query_that_is_not_a_query_is_refused_in_words() {
    let root = a_folder_of_our_own("refused");
    fs::write(root.join("notes.txt"), b"Dear Anna").unwrap();
    let index = Index::of(&root).unwrap();
    let _ = fs::remove_dir_all(&root);
    let strings = in_english();

    let a_paragraph = (0..=A_SENTENCE)
        .map(|n| format!("word{n}"))
        .collect::<Vec<_>>()
        .join(" ");
    let a_long_name = "n".repeat(A_NAME + 1);
    for (query, refusal) in [
        (Query::named(""), NotAsked::Nothing),
        (Query::saying("  ,  "), NotAsked::Nothing),
        (
            Query::saying(&a_paragraph),
            NotAsked::MoreThanASentence {
                words: A_SENTENCE + 1,
                most: A_SENTENCE,
            },
        ),
        (
            Query::named(&a_long_name),
            NotAsked::LongerThanAName {
                chars: A_NAME + 1,
                most: A_NAME,
            },
        ),
    ] {
        let refused = index.answer(&query).unwrap_err();
        assert_eq!(refused, refusal);
        let said = refused.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!said.text().starts_with("finding."), "{said}");
    }
    assert_eq!(
        NotAsked::Nothing.said(&strings).text(),
        "Nothing was asked, so nothing was searched: a search needs a name, a kind, a date or \
         some words."
    );

    let asked = index.answer(&Query::named("notes")).unwrap();
    assert_eq!(below(&asked.found), ["notes.txt"]);
}
