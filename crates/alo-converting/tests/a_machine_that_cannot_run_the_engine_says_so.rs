//! A machine that cannot run the engine says so, instead of failing ten times.
//!
//! Task 9 of `docs/autonomy/v0-5-documents-and-paper-plan.md`, and ADR 0063.
//! Two halves, and the second is the one that keeps the first honest:
//!
//! - **The ask refuses correctly.** Nothing there, a file that is not a
//!   program, a wrapper whose binary cannot run, one that answers nothing and
//!   one that never answers — each is a reason naming where the engine was
//!   looked for, and none of them is a crash or a hang.
//! - **Exactly the ten tests that need the engine ask it**, held against the
//!   sources rather than against a list somebody keeps beside them. A
//!   conversion test added without the ask fails here, and so does an ask added
//!   to a test that does not need one — which would be a test quietly skipping
//!   itself on a machine where it ought to run.
//!
//! # Which ten, measured rather than read
//!
//! The plan names nine, from a run on x86_64 with the engine's directory moved
//! aside, and records that the Mac reports ten and that the difference is
//! unresolved. It was measured again on 2026-09-22 with `--no-fail-fast`, and
//! the difference was the measurement: `cargo test` stops at the first test
//! binary that fails, so the original run ended inside
//! `converting_a_real_document.rs` and never reached
//! `the_walk_through_documents_and_paper.rs`. With nothing stopping it the two
//! machines agree, at [`THE_TEN`].
//!
//! # A wrapper is why this is not `test -x`
//!
//! [`a_wrapper_whose_binary_cannot_run_is_a_reason`] is the aarch64 shape
//! written down: an executable shell script that starts a binary which is not
//! there. The executable bit is set, so the proxy this plan has already shipped
//! a defect from would call it an engine, and running it does not.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod asking;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use asking::{NoEngine, asking as ask, the_engine_on_this_machine};

/// Every test on this machine that needs the engine, and the file it is in.
///
/// Load-bearing: [`exactly_these_tests_ask_whether_the_engine_runs`] holds the
/// sources to it in both directions.
const THE_TEN: [(&str, &str); 10] = [
    (
        "converting_a_real_document.rs",
        "a_word_document_is_converted_and_what_it_lost_is_named",
    ),
    (
        "converting_a_real_document.rs",
        "an_excel_workbook_is_converted_and_what_it_lost_is_named",
    ),
    (
        "converting_a_real_document.rs",
        "a_powerpoint_presentation_is_converted_and_what_it_lost_is_named",
    ),
    (
        "converting_a_real_document.rs",
        "an_opendocument_text_is_converted_and_what_it_lost_is_named",
    ),
    (
        "converting_a_real_document.rs",
        "an_opendocument_spreadsheet_is_converted_and_what_it_lost_is_named",
    ),
    (
        "converting_a_real_document.rs",
        "an_opendocument_presentation_is_converted_and_what_it_lost_is_named",
    ),
    (
        "converting_a_real_document.rs",
        "a_document_with_a_macro_library_says_the_macros_were_not_run",
    ),
    (
        "converting_a_real_document.rs",
        "a_pages_document_is_converted_and_the_families_it_is_set_in_are_named",
    ),
    (
        "converting_a_real_document.rs",
        "a_document_that_loses_nothing_says_so",
    ),
    (
        "the_walk_through_documents_and_paper.rs",
        "the_walk_from_a_file_arriving_to_one_that_cannot_be_opened_reads_as_the_table",
    ),
];

/// How many tests each of those two files holds altogether.
///
/// Asserted so that a parser which quietly found nothing cannot report that
/// every test asks correctly.
const ALTOGETHER: [(&str, usize); 2] = [
    ("converting_a_real_document.rs", 14),
    ("the_walk_through_documents_and_paper.rs", 2),
];

/// What a test that needs the engine begins with, exactly.
const THE_ASK: &str = "if asking::this_machine_cannot_run_the_engine() {";

/// A folder of this test's own, removed when it is done with.
struct AFolder {
    /// Where it is.
    at: PathBuf,
}

impl AFolder {
    /// One, made.
    fn made() -> Self {
        /// So that two folders in one run never collide.
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let at = std::env::temp_dir().join(format!(
            "alo-converting-no-engine-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        drop(fs::remove_dir_all(&at));
        fs::create_dir_all(&at).unwrap();
        Self { at }
    }

    /// A file in it holding `text`, made runnable or left as it is.
    fn holding(&self, named: &str, text: &str, runnable: bool) -> PathBuf {
        let at = self.at.join(named);
        fs::write(&at, text).unwrap();
        if runnable {
            fs::set_permissions(&at, fs::Permissions::from_mode(0o755)).unwrap();
        }
        at
    }
}

impl Drop for AFolder {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.at));
    }
}

/// **Asking this machine answers what the engine is, or says why not** — and
/// never an empty answer standing in for a question that could not be asked.
///
/// The half every machine can take, whatever it has on it. It is also what
/// makes the skip visible as a decision rather than an absence: the answer is
/// always a sentence.
#[test]
fn asking_this_machine_answers_what_the_engine_is_or_says_why_not() {
    match the_engine_on_this_machine() {
        Ok(said) => assert!(
            !said.is_empty(),
            "the engine answered, and the answer was nothing"
        ),
        Err(why) => {
            assert!(
                why.at().is_absolute(),
                "a reason that does not say where the engine was looked for: {why}"
            );
            assert!(!why.to_string().is_empty());
            eprintln!("this machine cannot run the engine a conversion needs: {why}");
        }
    }
}

/// **An engine that is not there is a reason naming where it was looked for**,
/// not a panic and not a silent false.
#[test]
fn an_engine_that_is_not_there_is_a_reason_naming_where_it_looked() {
    let folder = AFolder::made();
    let nowhere = folder.at.join("no-engine-here");
    let why = ask(&nowhere, asking::LONGEST).expect_err("nothing there answered as an engine");
    assert!(matches!(why, NoEngine::NotStarted { .. }), "{why:?}");
    assert_eq!(why.at(), nowhere);
    assert!(why.to_string().contains("no-engine-here"), "{why}");
}

/// **A file that is not a program is a reason.** The engine's path pointing at
/// something unrunnable is the same answer as nothing being there at all.
#[test]
fn a_file_that_is_not_a_program_is_a_reason() {
    let folder = AFolder::made();
    let not_a_program = folder.holding("not-a-program", "this is not a program\n", false);
    let why = ask(&not_a_program, asking::LONGEST).expect_err("a text file answered as an engine");
    assert!(matches!(why, NoEngine::NotStarted { .. }), "{why:?}");
}

/// **A wrapper whose binary cannot run is a reason**, and this is the case the
/// ask exists for.
///
/// The engine on a machine is a shell script that starts a binary beside it.
/// On a machine of another architecture the script is there and runnable and
/// the binary cannot run, so the executable bit says yes and the engine says
/// no. `test -x` cannot tell these apart; running it can.
#[test]
fn a_wrapper_whose_binary_cannot_run_is_a_reason() {
    let folder = AFolder::made();
    let wrapper = folder.holding(
        "wrapper",
        "#!/bin/sh\nexec \"$(dirname \"$0\")/for-another-machine\" \"$@\"\n",
        true,
    );
    assert!(
        fs::metadata(&wrapper).unwrap().permissions().mode() & 0o111 != 0,
        "the bit a weaker check would have believed is set"
    );
    let why = ask(&wrapper, asking::LONGEST).expect_err("a wrapper over nothing answered");
    assert!(matches!(why, NoEngine::EndedBadly { .. }), "{why:?}");
    assert!(why.to_string().contains("wrapper"), "{why}");
}

/// **A program that ends well and says nothing is not an engine.** An engine
/// answers what it is; an empty answer is not an answer.
#[test]
fn a_program_that_says_nothing_is_not_an_engine() {
    let folder = AFolder::made();
    let quiet = folder.holding("quiet", "#!/bin/sh\nexit 0\n", true);
    let why = ask(&quiet, asking::LONGEST).expect_err("a program that said nothing answered");
    assert_eq!(why, NoEngine::SaidNothing { at: quiet });
}

/// **One that never answers is stopped, and is a reason** — not a test binary
/// waiting for something that will not come, and not a program left running
/// behind it.
#[test]
fn an_engine_that_never_answers_is_stopped_and_is_a_reason() {
    let folder = AFolder::made();
    let slow = folder.holding(
        "slow",
        "#!/bin/sh\nsleep 4\ntouch \"$(dirname \"$0\")/finished\"\n",
        true,
    );
    let finished = folder.at.join("finished");

    let began = Instant::now();
    let why = ask(&slow, Duration::from_secs(1)).expect_err("a program that never answered passed");
    let waited = began.elapsed();

    assert_eq!(
        why,
        NoEngine::DidNotAnswer {
            at: slow,
            given: Duration::from_secs(1)
        }
    );
    assert!(
        waited < Duration::from_secs(3),
        "the ask waited {waited:?} for a program it had given one second"
    );

    // Stopped rather than left. It writes that file a second after the ask gave
    // up on it, and would have if the ask had merely stopped waiting.
    thread::sleep(Duration::from_secs(6));
    assert!(
        !finished.exists(),
        "the program was left running after the ask gave up on it"
    );
}

/// **Every reason says where the engine was looked for**, because a reason a
/// person fixing the machine cannot act on is not a reason.
#[test]
fn every_reason_names_where_the_engine_was_looked_for() {
    let at = PathBuf::from("/somewhere/a/person/could/look");
    for why in [
        NoEngine::NotStarted {
            at: at.clone(),
            said: "no such file or directory".to_owned(),
        },
        NoEngine::DidNotAnswer {
            at: at.clone(),
            given: Duration::from_secs(60),
        },
        NoEngine::EndedBadly {
            at: at.clone(),
            ended: Some(126),
        },
        NoEngine::EndedBadly {
            at: at.clone(),
            ended: None,
        },
        NoEngine::SaidNothing { at: at.clone() },
    ] {
        assert_eq!(why.at(), at);
        let said = why.to_string();
        assert!(said.contains("/somewhere/a/person/could/look"), "{said}");
        assert!(said.len() > at.as_os_str().len(), "{said}");
    }
}

/// **Exactly the ten tests that need the engine ask whether it runs, and each
/// asks before it does anything else.**
///
/// Held against the sources in both directions. A conversion test added without
/// the ask fails here — it would be a permanently red test on a machine that
/// cannot have an engine, which is what task 9 removed. An ask added to a test
/// that does not need one fails here too — it would be a test skipping itself
/// on a machine where it ought to run, which is what ADR 0039 is right about.
#[test]
fn exactly_these_tests_ask_whether_the_engine_runs() {
    for (file, how_many) in ALTOGETHER {
        let found = tests_in(&reading(file));
        assert_eq!(
            found.len(),
            how_many,
            "{file} holds {} tests and this reads {how_many}, so the reading is wrong rather \
             than the file",
            found.len()
        );
        for (name, body) in found {
            let expected = THE_TEN.contains(&(file, name.as_str()));
            let asks = body.contains("this_machine_cannot_run_the_engine");
            assert_eq!(
                asks,
                expected,
                "{file}: {name} {} the engine, and the measured ten say it {}",
                if asks {
                    "asks about"
                } else {
                    "does not ask about"
                },
                if expected { "should" } else { "should not" }
            );
            if expected {
                assert!(
                    body.trim_start().starts_with(THE_ASK),
                    "{file}: {name} asks about the engine, but not before it does anything else"
                );
            }
        }
    }
}

/// One of this crate's test sources, read.
fn reading(named: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(named);
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Every `#[test]` function in a source, as its name and its body.
///
/// A test begins at `#[test]` on its own line and its body ends at the first
/// closing brace in the first column, which is how these files are written and
/// what `cargo fmt` keeps true.
fn tests_in(source: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for after in source.split("\n#[test]\n").skip(1) {
        let Some((signature, rest)) = after.split_once(" {\n") else {
            continue;
        };
        let Some(name) = signature
            .strip_prefix("fn ")
            .and_then(|named| named.split('(').next())
        else {
            continue;
        };
        let body = rest.split("\n}\n").next().unwrap_or(rest);
        found.push((name.to_owned(), body.to_owned()));
    }
    found
}
