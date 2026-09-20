//! No English is written in the three crates of documents and paper outside
//! the vocabulary `alo-strings` answers from.
//!
//! Task 5 of `docs/autonomy/v0-5-documents-and-paper-plan.md`. `alo-opening`,
//! `alo-converting` and `alo-printing` say things at the moment a person is
//! already frustrated — a file that will not open, a copy that lost something,
//! a printer that stopped — and `CLAUDE.md` calls hardcoded English a bug in a
//! European product. Each crate's own tests hold its declared list, and none of
//! them can see a sentence written somewhere that is **not** on the list: a
//! `format!` in a refusal, an `#[error]` somebody reached for, a line the
//! converting service writes that a surface will one day show. So this reads
//! the shipped source of all three for one, by the rules
//! [`reading_source`] sets out.
//!
//! Anything the rules do not accept is refused unless it is on
//! [`NOT_READ_BY_A_PERSON`] with the argument for it. That list is checked in
//! both directions: an entry the source no longer holds is refused as well,
//! because a list with a dead line in it is a list nobody reads as a check.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod reading_source;

use reading_source::{english_in, is_english, shipped_files_of, shipped_literals};

/// The three crates whose source is read: the ones the documents-and-paper plan
/// owns.
const THE_THREE: [&str; 3] = ["alo-opening", "alo-converting", "alo-printing"];

/// Why the converting service's standard error is not read by a person.
const THE_SERVICE_LOG: &str = "alo-convertd's standard error, which systemd keeps in the service's \
     log for whoever stands the machine up; the verb that reaches the service reads only its \
     answers, and says NothingHereConverts in words";

/// Why the service's reasons for not converting are not read by a person.
const MAPPED_TO_A_REFUSAL: &str = "the Display of a reason inside the converting service, which \
     serving.rs discards with map_err(|_| ...) for a Refusal, which NotConverted::said words \
     for a person; nothing formats it";

/// Why a missing part's name is not read by a person.
const A_MISSING_PART: &str = "fills NotInventoried::Missing's Display, which serving.rs discards for \
     Refusal::OriginalNotChecked; the person is told that refusal in words";

/// Why a lint expectation's reason is not read by a person.
const A_LINT_EXPECTATION: &str = "the reason on a lint expectation: the compiler prints it to \
     whoever builds this crate when the expectation stops being fulfilled, and nothing else \
     ever reads it";

/// English in shipped source that no person using the machine reads, each with
/// the reason — `(crate, file under src/, a fragment of the literal, why)`.
const NOT_READ_BY_A_PERSON: &[(&str, &str, &str, &str)] = &[
    (
        "alo-converting",
        "bin/alo-convertd.rs",
        "standard input could not be taken",
        THE_SERVICE_LOG,
    ),
    (
        "alo-converting",
        "bin/alo-convertd.rs",
        "standard input is not a socket",
        THE_SERVICE_LOG,
    ),
    (
        "alo-converting",
        "bin/alo-convertd.rs",
        "documents are converted on a Linux machine",
        THE_SERVICE_LOG,
    ),
    (
        "alo-converting",
        "engine.rs",
        "the engine could not be started",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "engine.rs",
        "the engine took too long",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "engine.rs",
        "the engine wrote no copy",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "inventory/copy.rs",
        "the copy is not a PDF",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "inventory/copy.rs",
        "the copy's fonts cannot be read",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "inventory/original.rs",
        "the document has no",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "inventory/original.rs",
        "sets text in more fonts than are listed",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "inventory/excel.rs",
        "workbook part",
        A_MISSING_PART,
    ),
    (
        "alo-converting",
        "inventory/opendocument.rs",
        "content part",
        A_MISSING_PART,
    ),
    (
        "alo-converting",
        "inventory/pages.rs",
        "Pages document",
        A_MISSING_PART,
    ),
    (
        "alo-converting",
        "inventory/pages.rs",
        "the shape of an inventory nobody has taken",
        A_LINT_EXPECTATION,
    ),
    (
        "alo-converting",
        "inventory/original.rs",
        "has been inventoried on this machine",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "inventory/powerpoint.rs",
        "presentation part",
        A_MISSING_PART,
    ),
    (
        "alo-converting",
        "inventory/word.rs",
        "main document part",
        A_MISSING_PART,
    ),
    (
        "alo-converting",
        "xml.rs",
        "is not XML that could be read",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "zip.rs",
        "does not hold together as a zip",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "zip.rs",
        "decompresses past what is read",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "zip.rs",
        "stored in a way that is not read here",
        MAPPED_TO_A_REFUSAL,
    ),
    (
        "alo-converting",
        "wire.rs",
        "converted everything",
        "a line of the protocol between the verb and the converting service, written and read by \
         wire.rs on both sides; a person is told Carried::said",
    ),
    (
        "alo-printing",
        "http.rs",
        "Connection: close",
        "the head of the HTTP request this machine's printing service is sent over its own socket; \
         bytes on a wire, which no sentence is made from",
    ),
];

/// **No shipped source of the three crates writes English outside the
/// vocabulary**, except the English on [`NOT_READ_BY_A_PERSON`] — and every
/// entry on that list is still English somewhere, so the list cannot rot into
/// permission nobody needs.
#[test]
fn the_three_crates_write_no_english_outside_the_vocabulary() {
    let mut unexplained = Vec::new();
    let mut explained = vec![false; NOT_READ_BY_A_PERSON.len()];
    for member in THE_THREE {
        for (under, source) in shipped_files_of(member) {
            let file_name = under.rsplit('/').next().unwrap_or(&under).to_owned();
            for (line, text) in english_in(&file_name, &source) {
                let explaining =
                    NOT_READ_BY_A_PERSON
                        .iter()
                        .position(|(crate_, file, fragment, _)| {
                            *crate_ == member && *file == under && text.contains(fragment)
                        });
                match explaining {
                    Some(entry) => {
                        if let Some(seen) = explained.get_mut(entry) {
                            *seen = true;
                        }
                    }
                    None => unexplained.push(format!("{member}/src/{under}:{line}: {text:?}")),
                }
            }
        }
    }
    assert!(
        unexplained.is_empty(),
        "English a person could read, written outside alo-strings — declare it in the crate's \
         words.rs, or argue it onto NOT_READ_BY_A_PERSON: {unexplained:#?}"
    );
    let stale: Vec<_> = NOT_READ_BY_A_PERSON
        .iter()
        .zip(&explained)
        .filter(|(_, seen)| !**seen)
        .map(|(entry, _)| entry)
        .collect();
    assert!(
        stale.is_empty(),
        "an exception the source no longer needs: {stale:#?}"
    );
}

/// **Every exception carries its argument.** A fragment with a shrug beside it
/// is English a person might read, with permission.
#[test]
fn every_exception_says_why_nobody_reads_it() {
    for (member, file, fragment, why) in NOT_READ_BY_A_PERSON {
        assert!(
            THE_THREE.contains(member),
            "{member} is not one of the three"
        );
        assert!(
            is_english(fragment),
            "{fragment:?} is not what the search finds"
        );
        assert!(
            why.split_whitespace().count() >= 6,
            "{member}/src/{file}: {fragment:?} has no argument beside it"
        );
    }
}

/// **The search reads what it says it reads.** Every sentence each of the
/// three crates declares is found in its `words.rs` as the vocabulary — a key
/// and a sentence in a `saying` for each phrase the machine collects under that
/// crate's area — so the test above finding nothing is a measurement rather
/// than a search that looked nowhere.
#[test]
fn the_search_finds_every_crates_declared_vocabulary() {
    let vocabulary =
        alo_saying::everything_this_machine_can_say().expect("the machine's own words collect");
    for member in THE_THREE {
        let area = member.trim_start_matches("alo-");
        let declared = vocabulary
            .phrases()
            .filter(|phrase| phrase.key().area() == area)
            .count();
        assert!(
            declared > 0,
            "{member} declares nothing the machine collects"
        );
        let (literals, _) = shipped_files_of(member)
            .into_iter()
            .find(|(under, _)| under == "words.rs")
            .map(|(_, source)| shipped_literals(&source))
            .unwrap_or_else(|| panic!("{member} has no src/words.rs"));
        let sayings = literals
            .iter()
            .filter(|literal| literal.callee.as_deref() == Some("saying"))
            .count();
        assert_eq!(
            sayings,
            declared * 2,
            "{member}: a key and a sentence for each of its {declared} phrases"
        );
    }
}

/// **The search reads every shipped file of the three, and no test's**: the
/// service's `main`, the files under a folder, and not a module only a test
/// declares.
#[test]
fn the_search_reads_every_shipped_file_and_no_test_module() {
    let converting: Vec<String> = shipped_files_of("alo-converting")
        .into_iter()
        .map(|(under, _)| under)
        .collect();
    for shipped in ["bin/alo-convertd.rs", "words.rs", "engine.rs", "serving.rs"] {
        assert!(
            converting.iter().any(|under| under == shipped),
            "{shipped} is not read: {converting:?}"
        );
    }
    assert!(
        converting
            .iter()
            .any(|under| under.starts_with("inventory/")),
        "the files under inventory/ are not read: {converting:?}"
    );
    for member in THE_THREE {
        assert!(
            shipped_files_of(member)
                .iter()
                .all(|(under, _)| under != "testing.rs"),
            "{member}: a module only its tests declare is read as shipped"
        );
    }
}

/// **A sentence written anywhere else is found**, however it is spelt — held
/// against text, so the rule is shown refusing without planting English in the
/// repository.
#[test]
fn a_sentence_outside_the_vocabulary_is_found_however_it_is_spelt() {
    let refusal = "fn f() -> String { format!(\"the printer is out of paper\") }";
    assert_eq!(english_in("printing.rs", refusal).len(), 1);

    let displayed = "#[derive(thiserror::Error)]\nenum E {\n    #[error(\"the copy was not made\")]\n    NotMade,\n}";
    assert_eq!(
        english_in("converting.rs", displayed).len(),
        1,
        "an #[error] is English"
    );

    let raw = "const SAID: &str = r#\"this \"file\" cannot be opened\"#;";
    assert_eq!(
        english_in("outcome.rs", raw),
        [(1, "this \"file\" cannot be opened".to_owned())]
    );

    let declared_elsewhere =
        "pub const W: Word = Word::saying(\"printing.ready\", \"The printer is ready\");";
    assert!(english_in("words.rs", declared_elsewhere).is_empty());
    assert_eq!(
        english_in("stopped.rs", declared_elsewhere).len(),
        1,
        "the vocabulary is only the vocabulary in words.rs"
    );
}

/// **What is not a sentence on a screen is not found**: keys, the keywords a
/// printer reports and a media type, a lint's reason, and a test module's
/// English.
#[test]
fn what_is_not_a_sentence_on_a_screen_is_not_found() {
    let source = r#"
//! A file that says "something in prose".
#[expect(clippy::unwrap_used, reason = "in a test, a panic is the failure")]
fn f() {
    let _ = ("printing.stopped.jammed", "media-empty-error", "application/pdf", "word/document.xml");
}
#[cfg(test)]
mod tests {
    fn t() { let _ = "a sentence in a test"; }
}
"#;
    assert_eq!(
        english_in("stopped.rs", source),
        Vec::<(usize, String)>::new()
    );
}
