//! What applications were answered is kept as long as the machine's record, and
//! no longer.
//!
//! Task 9 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`,
//! each clause of its acceptance a test here:
//!
//! - **shortened under the same `[record].keeping` rule, read through
//!   `alo_keeping::Keeping`; an answer past the rule is gone and one inside it
//!   is not; `since` and `under` go into the format line and `format` stays
//!   `1`** — [`the_file::an_answer_past_the_rule_is_gone_and_one_inside_it_is_not`];
//! - **`"forever"` removes nothing** —
//!   [`the_file::forever_removes_nothing_and_touches_nothing`];
//! - **never a line that did not read; a torn line survives** —
//!   [`the_file::a_line_that_did_not_read_survives_a_shortening`];
//! - **a file shortened twice still says it was** —
//!   [`the_file::a_file_shortened_twice_still_says_it_was`];
//! - **whole or not at all; a shortening that cannot write leaves the file as
//!   it was** —
//!   [`the_file::a_shortening_that_cannot_write_leaves_the_file_as_it_was`];
//! - **the backend answers nothing while the file is replaced** —
//!   [`the_file::no_answer_is_kept_while_the_file_is_replaced`];
//! - **the contract gains the two fields additively** —
//!   [`the_contract_names_where_a_shortened_file_starts`].

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

/// **The contract names both fields a shortening writes, the rule they come
/// from, and keeps `format` at `1`.**
#[test]
fn the_contract_names_where_a_shortened_file_starts() {
    let contract = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/contracts/portal-answers-file.md"),
    )
    .expect("docs/contracts/portal-answers-file.md is there");
    for named in [
        "`since`",
        "`under`",
        "`[record].keeping`",
        "## Shortening it",
    ] {
        assert!(contract.contains(named), "{named}");
    }
    assert!(contract.contains("`1` today"), "format stays 1");
}

#[cfg(unix)]
mod the_file {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use alo_capability::{Applicant, Grants};
    use alo_keeping::Keeping;
    use alo_portals::{
        Answered, AnswersFile, KeptAnswer, NotRecorded, Outcome, Portal, Recording, Request,
        Unanswered,
    };

    const FRACTAL: &str = "org.gnome.Fractal";
    const STRANGER: &str = "org.example.Stranger";
    const A_DAY: Duration = Duration::from_secs(24 * 60 * 60);

    /// A folder of this test's own, empty.
    fn a_folder(what: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "alo-portal-answers-shortened-{what}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn week() -> Keeping {
        Keeping::for_days(7).unwrap()
    }

    /// An answer given at `at`, of one of three kinds by `which`: a refusal by
    /// the grants, a caller nobody could name, and an answer the keyring could
    /// not give — so what a shortening removes and keeps is refusals as much as
    /// anything.
    fn answered_at(at: SystemTime, which: u32) -> Answered {
        match which % 3 {
            0 => Answered::new(
                at,
                Some(Applicant::named(STRANGER)),
                Portal::Secret,
                Outcome::Refused(
                    Request::of(STRANGER, Portal::Secret)
                        .unwrap()
                        .judged(&Grants::default(), at)
                        .unwrap_err(),
                ),
            ),
            1 => Answered::new(
                at,
                None,
                Portal::OpenWith,
                Outcome::Unanswered(Unanswered::NotIdentified),
            ),
            _ => Answered::new(
                at,
                Some(Applicant::named(FRACTAL)),
                Portal::Secret,
                Outcome::Unanswered(Unanswered::KeyringUnavailable),
            ),
        }
    }

    /// A fortnight of answers, one a day at noon, oldest first.
    fn a_fortnight() -> Vec<Answered> {
        (0..14_u32)
            .rev()
            .map(|days_ago| answered_at(noon() - A_DAY * days_ago, days_ago))
            .collect()
    }

    fn kept(answers: &[Answered]) -> Vec<KeptAnswer> {
        answers.iter().map(KeptAnswer::from).collect()
    }

    fn a_file_with(at: &Path, answers: &[Answered]) -> AnswersFile {
        let record = AnswersFile::opened(at).unwrap();
        for answered in answers {
            record.keep(answered.clone()).unwrap();
        }
        record
    }

    /// **An answer past the rule is gone after a shortening, and one inside it
    /// is not** — the answer at the rule's own edge among those kept — and the
    /// first line says where the file now starts and under which rule, in the
    /// notation the agent's record writes them in, with `format` still `1`.
    #[test]
    fn an_answer_past_the_rule_is_gone_and_one_inside_it_is_not() {
        let folder = a_folder("past-the-rule");
        let at = folder.join("portal-answers.jsonl");
        let answers = a_fortnight();
        let record = a_file_with(&at, &answers);

        let shortened = record.shortened(week(), noon()).unwrap();
        assert_eq!(
            shortened.removed(),
            6,
            "the thirteenth to the eighth day ago"
        );
        assert_eq!(shortened.kept(), 8, "the seventh day ago to today");
        assert_eq!(shortened.since(), Some(noon() - A_DAY * 7));
        assert!(shortened.anything_removed());

        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.answers, kept(answers.get(6..).unwrap()));
        assert_eq!(read.unreadable, Vec::<usize>::new());
        assert_eq!(read.since, Some(noon() - A_DAY * 7));
        assert_eq!(read.under, Some(week()));
        let text = std::fs::read_to_string(&at).unwrap();
        assert_eq!(
            text.lines().next(),
            Some(
                r#"{"format":1,"since":{"secs_since_epoch":1759395200,"nanos_since_epoch":0},"under":{"for-days":7}}"#
            )
        );

        // The backend goes on keeping answers, into the file that replaced it.
        record.keep(answered_at(noon(), 0)).unwrap();
        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.answers.len(), 9);
        assert_eq!(read.since, Some(noon() - A_DAY * 7));
        // And a backend started again over it adds to it, and still reads it.
        a_file_with(&at, &[answered_at(noon(), 1)]);
        assert_eq!(AnswersFile::read_back(&at).unwrap().answers.len(), 10);

        let left: Vec<String> = std::fs::read_dir(&folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(left, ["portal-answers.jsonl"], "nothing is left beside it");
        let mode =
            std::os::unix::fs::PermissionsExt::mode(&std::fs::metadata(&at).unwrap().permissions());
        assert_eq!(
            mode & 0o777,
            0o600,
            "the shortened file is its owner's alone"
        );
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **`"forever"` removes nothing, and a rule nothing is old enough for yet
    /// removes nothing** — and neither rewrites a byte, the first line
    /// included, because a file that lost nothing has nothing to say about
    /// where it starts.
    #[test]
    fn forever_removes_nothing_and_touches_nothing() {
        let folder = a_folder("forever");
        let at = folder.join("portal-answers.jsonl");
        let record = a_file_with(&at, &a_fortnight());
        let before = std::fs::read(&at).unwrap();
        let inode = std::os::unix::fs::MetadataExt::ino(&std::fs::metadata(&at).unwrap());

        for (keeping, now) in [
            (Keeping::Forever, noon()),
            (Keeping::Forever, noon() + A_DAY * 36_500),
            (Keeping::for_days(90).unwrap(), noon()),
            // A clock that says it is 1970 is never a way to empty the file.
            (week(), SystemTime::UNIX_EPOCH + Duration::from_secs(60)),
        ] {
            let shortened = record.shortened(keeping, now).unwrap();
            assert!(!shortened.anything_removed(), "{keeping:?}");
            assert_eq!(shortened.kept(), 14);
            assert_eq!(shortened.since(), None);
            assert_eq!(std::fs::read(&at).unwrap(), before, "{keeping:?}");
        }
        assert_eq!(
            std::os::unix::fs::MetadataExt::ino(&std::fs::metadata(&at).unwrap()),
            inode,
            "the file was never replaced"
        );
        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.since, None);
        assert_eq!(read.under, None);
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **A line that did not read is never removed**: one in the middle, dated
    /// long before the rule's window but naming no identifier, and a torn last
    /// line nobody can date. Both stay, byte for byte and in their places; the
    /// torn one is ended so it stays one line; and the next answer is whole.
    #[test]
    fn a_line_that_did_not_read_survives_a_shortening() {
        let folder = a_folder("torn");
        let at = folder.join("portal-answers.jsonl");
        let answers = a_fortnight();
        drop(a_file_with(&at, answers.get(..3).unwrap()));

        let unreadable_old = r#"{"at":{"secs_since_epoch":1,"nanos_since_epoch":0},"application":"org.gnome Fractal","portal":"secret","answer":"unanswered","why":"not-identified"}"#;
        let torn = r#"{"at":{"secs_since_epoch":17600"#;
        let mut text = std::fs::read_to_string(&at).unwrap();
        text.push_str(unreadable_old);
        text.push('\n');
        std::fs::write(&at, &text).unwrap();
        drop(a_file_with(&at, answers.get(3..).unwrap()));
        let mut text = std::fs::read_to_string(&at).unwrap();
        text.push_str(torn);
        std::fs::write(&at, &text).unwrap();
        assert_eq!(
            AnswersFile::read_back(&at).unwrap().unreadable,
            [5, 17],
            "the two lines that do not read, before"
        );

        let record = AnswersFile::opened(&at).unwrap();
        let shortened = record.shortened(week(), noon()).unwrap();
        assert_eq!(shortened.removed(), 6);
        assert_eq!(shortened.kept(), 8);

        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.answers, kept(answers.get(6..).unwrap()));
        let text = std::fs::read_to_string(&at).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.get(1), Some(&unreadable_old), "kept in its place");
        assert_eq!(lines.last(), Some(&torn), "kept, and last");
        assert!(text.ends_with(&format!("{torn}\n")), "and ended");
        assert_eq!(read.unreadable, [2, 11]);

        record.keep(answered_at(noon(), 2)).unwrap();
        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.answers.len(), 9, "the next answer is whole");
        assert_eq!(read.unreadable, [2, 11]);
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **A file shortened twice still says it was**, starting where the second
    /// rule left it — and a later round that removes nothing, or a longer rule,
    /// never makes it read as whole again.
    #[test]
    fn a_file_shortened_twice_still_says_it_was() {
        let folder = a_folder("twice");
        let at = folder.join("portal-answers.jsonl");
        let record = a_file_with(&at, &a_fortnight());

        record.shortened(week(), noon()).unwrap();
        let three = Keeping::for_days(3).unwrap();
        let again = record.shortened(three, noon() + A_DAY).unwrap();
        assert!(again.anything_removed());
        let edge = noon() + A_DAY - A_DAY * 3;
        assert_eq!(again.since(), Some(edge));

        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.since, Some(edge));
        assert_eq!(read.under, Some(three));
        assert!(read.answers.iter().all(|answer| answer.at() >= edge));

        for (keeping, now) in [
            (three, noon() + A_DAY),
            (Keeping::Forever, noon() + A_DAY),
            (Keeping::for_days(90).unwrap(), noon() + A_DAY * 2),
        ] {
            let nothing = record.shortened(keeping, now).unwrap();
            assert!(!nothing.anything_removed());
            assert_eq!(nothing.since(), Some(edge), "{keeping:?}");
            let read = AnswersFile::read_back(&at).unwrap();
            assert_eq!(read.since, Some(edge));
            assert_eq!(read.under, Some(three));
        }
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **A shortening that cannot write leaves the file as it was**, byte for
    /// byte, and the backend goes on keeping answers into it: when the
    /// replacement cannot be made beside it, when the path no longer holds the
    /// file the backend has open, and when its first line is not one this
    /// backend shortens.
    #[test]
    fn a_shortening_that_cannot_write_leaves_the_file_as_it_was() {
        let folder = a_folder("cannot-write");
        let at = folder.join("portal-answers.jsonl");
        let record = a_file_with(&at, &a_fortnight());
        let before = std::fs::read(&at).unwrap();

        // Something that will not be removed where the replacement goes. A
        // folder with something in it, because a folder that will not be
        // written into does not stop root.
        let in_the_way = folder.join("portal-answers.jsonl.shortening");
        std::fs::create_dir(&in_the_way).unwrap();
        std::fs::write(in_the_way.join("somebody's"), "left here").unwrap();
        let refused = record.shortened(week(), noon()).unwrap_err();
        assert!(
            matches!(refused, NotRecorded::NotWritten { .. }),
            "{refused:?}"
        );
        assert_eq!(std::fs::read(&at).unwrap(), before, "as it was");
        assert_eq!(
            std::fs::read_to_string(in_the_way.join("somebody's")).unwrap(),
            "left here"
        );
        std::fs::remove_dir_all(&in_the_way).unwrap();

        // A link where the replacement goes is removed, never written through.
        let elsewhere = folder.join("elsewhere");
        std::fs::write(&elsewhere, "not the answers").unwrap();
        std::os::unix::fs::symlink(&elsewhere, &in_the_way).unwrap();

        // The file moved away and another put at its path.
        let moved = folder.join("moved.jsonl");
        std::fs::rename(&at, &moved).unwrap();
        std::fs::write(&at, "{\"format\":1}\n").unwrap();
        let refused = record.shortened(week(), noon()).unwrap_err();
        assert!(
            matches!(refused, NotRecorded::Replaced { .. }),
            "{refused:?}"
        );
        assert_eq!(std::fs::read(&moved).unwrap(), before);
        assert_eq!(std::fs::read_to_string(&at).unwrap(), "{\"format\":1}\n");
        std::fs::remove_file(&at).unwrap();
        let refused = record.shortened(week(), noon()).unwrap_err();
        assert!(
            matches!(refused, NotRecorded::Replaced { .. }),
            "{refused:?}"
        );
        std::fs::rename(&moved, &at).unwrap();

        // A first line in a newer format, written into the same file.
        let mut newer = b"{\"format\":2}\n".to_vec();
        newer.extend_from_slice(before.get(13..).unwrap());
        std::fs::write(&at, &newer).unwrap();
        let refused = record.shortened(week(), noon()).unwrap_err();
        assert!(
            matches!(refused, NotRecorded::ANewerFormat { format: 2, .. }),
            "{refused:?}"
        );
        assert_eq!(std::fs::read(&at).unwrap(), newer);
        std::fs::write(&at, &before).unwrap();

        // And after all of it, the backend still keeps an answer into the file.
        record.keep(answered_at(noon(), 0)).unwrap();
        assert_eq!(AnswersFile::read_back(&at).unwrap().answers.len(), 15);
        assert_eq!(
            std::fs::read_to_string(&elsewhere).unwrap(),
            "not the answers"
        );
        // Once nothing is in the way, the same shortening goes through, and
        // removes the link rather than what it named.
        assert_eq!(record.shortened(week(), noon()).unwrap().removed(), 6);
        assert_eq!(
            std::fs::read_to_string(&elsewhere).unwrap(),
            "not the answers"
        );
        assert!(!in_the_way.exists());
        std::fs::remove_dir_all(&folder).unwrap();
    }

    /// **No answer is kept while the file is replaced** — so none is sent, and
    /// none is written into the file renamed away. Four doors keep answers
    /// while the file is shortened over and over; every answer a door was told
    /// is kept is in the file afterwards, and every one past the rule is not.
    #[test]
    fn no_answer_is_kept_while_the_file_is_replaced() {
        let folder = a_folder("meanwhile");
        let at = folder.join("portal-answers.jsonl");
        let record = Arc::new(AnswersFile::opened(&at).unwrap());

        let doors: Vec<_> = (0..4_u32)
            .map(|door| {
                let record = Arc::clone(&record);
                std::thread::spawn(move || {
                    (0..100_u32)
                        .map(|n| {
                            let at = noon() + Duration::from_nanos(u64::from(door * 1000 + n));
                            record.keep(answered_at(at, n)).unwrap();
                            at
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let mut rounds = 0;
        for n in 0..100_u32 {
            record.keep(answered_at(noon() - A_DAY * 30, n)).unwrap();
            if record.shortened(week(), noon()).unwrap().anything_removed() {
                rounds += 1;
            }
        }
        let told_kept: BTreeSet<SystemTime> = doors
            .into_iter()
            .flat_map(|door| door.join().unwrap())
            .collect();
        assert_eq!(rounds, 100, "every round replaced the file");
        assert_eq!(told_kept.len(), 400);

        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.unreadable, Vec::<usize>::new());
        let in_the_file: BTreeSet<SystemTime> = read.answers.iter().map(KeptAnswer::at).collect();
        assert_eq!(in_the_file, told_kept);
        assert_eq!(read.answers.len(), 400);
        std::fs::remove_dir_all(&folder).unwrap();
    }
}
