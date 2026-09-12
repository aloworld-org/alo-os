//! What a contributor hands the loop when a task's code is finished.
//!
//! One file, `.kernel-loop/handoff.toml`, naming the task, the files it
//! touched, the commit it becomes, the report it published, and **the test
//! behind each acceptance criterion**. The loop stages **exactly those files**
//! and no others — `git add -A` would sweep up whatever
//! else happened to be in the tree, which is how an unrelated change gets
//! published under somebody else's commit message.
//!
//! The evidence is not decoration and is not the loop taking a worker's word:
//! `crate::evidence` runs each named test on its own and refuses one whose file
//! is not part of the change. A task whose only claim is that the existing
//! suite still passes has not shown that it was done.
//!
//! ```text
//! task = Reproducing unrestricted network access from inside a bound turn
//! report = docs/autonomy/updates/network-egress-reproduction.md
//! subject = test(bounding): reproduce a bound turn opening any socket
//! evidence =
//!   . alo-bounding a_turn_reaches_the_network a_bound_turn_opens_any_socket
//! files =
//!   crates/alo-bounding/tests/a_turn_reaches_the_network.rs
//!   docs/autonomy/updates/network-egress-reproduction.md
//! body =
//!   The first paragraph of the message.
//!
//!   The second.
//! ```
//!
//! # A hand-written parser, and why there is no dependency here
//!
//! Four keys and a list. A crate to read that would be a dependency in a tool
//! whose whole job is to be trustworthy about what it publishes, and the format
//! is small enough that the parser below is shorter than the argument for
//! renting one.

use std::path::{Path, PathBuf};

/// What the loop was handed.
#[derive(Debug)]
pub struct Handed {
    /// The task's descriptive name, which must be one the plan knows.
    pub task: String,

    /// The report this task published, under `docs/autonomy/updates/`.
    ///
    /// Read to insist it is among the files being staged: a task that gated,
    /// committed and pushed without its report would be one whose evidence
    /// stayed on somebody's disk.
    pub report: String,

    /// The commit subject, in conventional style.
    pub subject: String,

    /// The commit body.
    pub body: String,

    /// Every file to stage, and nothing else is staged.
    pub files: Vec<String>,

    /// One test per acceptance criterion, each held up by [`crate::evidence`].
    ///
    /// Refused when empty, because the gates a task passes are the state of
    /// everything except the thing it just wrote.
    pub evidence: Vec<crate::evidence::Shown>,
}

/// What the file is called inside the loop's own directory.
const THE_HANDOFF: &str = "handoff.toml";

/// Where a finished handoff is kept, so a task is never published twice.
const ALREADY_DONE: &str = "published";

/// Where a refused handoff is kept, under the moment it was refused.
const REFUSED: &str = "refused";

/// A handoff the gates refused, found again for the task it was written for.
///
/// Both the text and the reading of it: the text is what goes back into place
/// byte for byte when a parked task is recovered from it, and the reading is
/// what says which task and which files that text claims.
#[derive(Debug)]
pub struct Refused {
    /// The file it was found in, so a person can be told where the handoff
    /// they are holding came from.
    pub at: PathBuf,

    /// The file's text, exactly.
    pub written: String,

    /// The file, read.
    pub handed: Handed,
}

impl Handed {
    /// Whatever is waiting, or [`None`] if nothing is.
    ///
    /// # Errors
    /// A sentence naming what is missing when the file is there and does not
    /// say everything a commit needs. A half-written handoff is refused rather
    /// than published with an empty subject.
    pub fn waiting(ours: &Path) -> Result<Option<Self>, String> {
        let at = ours.join(THE_HANDOFF);
        let written = match std::fs::read_to_string(&at) {
            Ok(written) => written,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(why) => return Err(format!("{} could not be read: {why}", at.display())),
        };
        Self::read(&written).map(Some)
    }

    /// Where a handoff waits.
    ///
    /// One name, in one place, so that recovering a parked task puts its
    /// handoff back exactly where the loop looks for one rather than somewhere
    /// that reads the same.
    #[must_use]
    pub fn where_one_waits(ours: &Path) -> std::path::PathBuf {
        ours.join(THE_HANDOFF)
    }

    /// Move the handoff aside, so the next iteration does not publish it again.
    ///
    /// Kept rather than deleted: it is the record of what a commit was made
    /// from, and the one place a person can look to see what the loop was told.
    ///
    /// # Errors
    /// Whatever the machine said. This matters and is not best effort — a
    /// handoff left in place would be republished, which is the one mistake a
    /// publishing loop must not make.
    pub fn put_away(ours: &Path, sha: &str) -> Result<(), String> {
        let done = ours.join(ALREADY_DONE);
        std::fs::create_dir_all(&done)
            .map_err(|why| format!("{} could not be made: {why}", done.display()))?;
        std::fs::rename(ours.join(THE_HANDOFF), done.join(format!("{sha}.toml")))
            .map_err(|why| format!("the handoff could not be put away: {why}"))
    }

    /// Move a refused handoff out of the way so a second worker can write its
    /// own, keeping it under the refusal it earned.
    ///
    /// **Kept, like every other one**, and for a sharper reason: it is the only
    /// account of what the first worker believed it had finished, and the pair
    /// of it and the repaired one is how anybody later sees what the gates
    /// actually caught. Nothing here deletes a file.
    ///
    /// # Errors
    /// Whatever the machine said. A handoff left in place would be read as the
    /// second worker's own, and the loop would gate the same refused work twice
    /// and call it an answer.
    pub fn put_aside(ours: &Path) -> Result<(), String> {
        let refused = ours.join(REFUSED);
        std::fs::create_dir_all(&refused)
            .map_err(|why| format!("{} could not be made: {why}", refused.display()))?;
        let when = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        std::fs::rename(ours.join(THE_HANDOFF), refused.join(format!("{when}.toml")))
            .map_err(|why| format!("the refused handoff could not be put aside: {why}"))
    }

    /// Where refused handoffs are kept, so a refusal can name it.
    #[must_use]
    pub fn where_refused_ones_are(ours: &Path) -> PathBuf {
        ours.join(REFUSED)
    }

    /// The newest refused handoff for this task, or [`None`] when there is
    /// none.
    ///
    /// **Newest by the name [`Self::put_aside`] gave it**, which is the second
    /// the refusal happened, rather than by a modification time: a checkout
    /// copied or restored from somewhere gets new times on every file and
    /// keeps every name. A file in the directory whose name is not a number is
    /// treated as older than every one that is — it was not put there by this
    /// program, and the order this program wrote is the order that means
    /// something.
    ///
    /// **An entry that does not read is passed over rather than reported.**
    /// Everything [`Self::put_aside`] moves here was read successfully first —
    /// the loop gated it — so an unreadable file in this directory is not a
    /// refused handoff, whatever else it is.
    ///
    /// # Errors
    /// A sentence when the directory is there and cannot be listed. A directory
    /// that is not there is not an error: it means no handoff has ever been
    /// refused on this checkout, and the answer is [`None`].
    pub fn newest_refused_for(ours: &Path, task: &str) -> Result<Option<Refused>, String> {
        let refused = Self::where_refused_ones_are(ours);
        let listed = match std::fs::read_dir(&refused) {
            Ok(listed) => listed,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(why) => {
                return Err(format!("{} could not be listed: {why}", refused.display()));
            }
        };
        let mut entries: Vec<(u64, PathBuf)> = Vec::new();
        for entry in listed {
            let path = entry
                .map_err(|why| format!("{} could not be listed: {why}", refused.display()))?
                .path();
            if path.extension().is_none_or(|ext| ext != "toml") {
                continue;
            }
            let moment = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.parse::<u64>().ok())
                .unwrap_or(0);
            entries.push((moment, path));
        }
        entries.sort_by(|a, b| b.cmp(a));

        for (_, at) in entries {
            let Ok(written) = std::fs::read_to_string(&at) else {
                continue;
            };
            let Ok(handed) = Self::read(&written) else {
                continue;
            };
            if handed.task == task {
                return Ok(Some(Refused {
                    at,
                    written,
                    handed,
                }));
            }
        }
        Ok(None)
    }

    /// The file, read.
    ///
    /// # Errors
    /// A sentence naming what a commit needs and the text does not say.
    pub fn read(written: &str) -> Result<Self, String> {
        let mut task = String::new();
        let mut report = String::new();
        let mut subject = String::new();
        let mut body = String::new();
        let mut files = Vec::new();
        let mut evidence = Vec::new();
        let mut inside: Option<&str> = None;

        for line in written.lines() {
            let indented = line.starts_with([' ', '\t']);
            if indented || line.trim().is_empty() {
                match inside {
                    Some("files") => {
                        let named = line.trim();
                        if !named.is_empty() {
                            files.push(named.to_owned());
                        }
                    }
                    Some("evidence") => {
                        let named = line.trim();
                        if !named.is_empty() {
                            evidence.push(crate::evidence::Shown::read(named)?);
                        }
                    }
                    Some("body") => {
                        body.push_str(line.strip_prefix("  ").unwrap_or(line.trim_start()));
                        body.push('\n');
                    }
                    _ => {}
                }
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "task" => task = value.to_owned(),
                "report" => report = value.to_owned(),
                "subject" => subject = value.to_owned(),
                "files" | "body" | "evidence" => {}
                _ => {}
            }
            inside = match key {
                "files" => Some("files"),
                "evidence" => Some("evidence"),
                "body" => Some("body"),
                _ => None,
            };
        }

        for (what, named) in [(&task, "task"), (&report, "report"), (&subject, "subject")] {
            if what.is_empty() {
                return Err(format!(
                    "the handoff does not say `{named}`, and a commit needs it"
                ));
            }
        }
        if files.is_empty() {
            return Err(
                "the handoff names no files, and the loop stages only what a task names".to_owned(),
            );
        }
        if !files.iter().any(|named| named == &report) {
            return Err(format!(
                "the handoff publishes {report} and does not list it among its files"
            ));
        }
        if evidence.is_empty() {
            return Err(
                "the handoff shows no `evidence`, and a green suite is the state of the \
                 repository rather than proof this task was done. Name one test per \
                 acceptance criterion: crate, test target, test name."
                    .to_owned(),
            );
        }
        Ok(Self {
            task,
            report,
            subject,
            body: body.trim_end().to_owned(),
            files,
            evidence,
        })
    }

    /// The whole commit message.
    #[must_use]
    pub fn message(&self) -> String {
        if self.body.is_empty() {
            self.subject.clone()
        } else {
            format!("{}\n\n{}", self.subject, self.body)
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A handoff with everything a commit needs, a line at a time so that a
    /// test can take one of them away.
    const WHOLE: &[&str] = &[
        "task = Auditing what a bound turn can still reach",
        "report = docs/autonomy/updates/an-audit.md",
        "subject = test(bounding): reproduce what a bound turn can still reach",
        "evidence =",
        "  . alo-bounding what_a_bound_turn_can_still_reach a_datagram_leaves_unchecked",
        "files =",
        "  crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs",
        "  docs/autonomy/updates/an-audit.md",
        "body =",
        "  What it says.",
    ];

    /// The whole thing.
    fn whole() -> String {
        WHOLE.join(
            "
",
        )
    }

    /// The same with one key gone, and whatever was indented under it —
    /// which is the shape of what a worker that stopped part way through
    /// leaves behind.
    fn without(key: &str) -> String {
        let mut kept = Vec::new();
        let mut dropping = false;
        for line in WHOLE {
            if !line.starts_with(' ') {
                dropping = line.starts_with(key);
            }
            if !dropping {
                kept.push(*line);
            }
        }
        kept.join(
            "
",
        )
    }

    /// A loop directory of this test's own, with these refused entries in it.
    fn a_refused_directory(called: &str, entries: &[(&str, &str)]) -> std::path::PathBuf {
        let ours = std::env::temp_dir().join(format!(
            "alo-handoff-refused-{}-{called}",
            std::process::id()
        ));
        drop(std::fs::remove_dir_all(&ours));
        let refused = Handed::where_refused_ones_are(&ours);
        std::fs::create_dir_all(&refused).expect("a refused directory");
        for (named, written) in entries {
            std::fs::write(refused.join(named), written).expect("an entry to be written");
        }
        ours
    }

    /// The whole handoff, for a different task.
    fn whole_for(task: &str) -> String {
        whole().replace("Auditing what a bound turn can still reach", task)
    }

    /// **The newest refused handoff for a task is found by the moment in its
    /// name**, past newer ones for other tasks and older ones for the same.
    #[test]
    fn the_newest_refused_handoff_for_a_task_is_found_by_its_moment() {
        let ours = a_refused_directory(
            "newest",
            &[
                ("100.toml", &whole_for("This one")),
                ("300.toml", &whole_for("Another")),
                (
                    "200.toml",
                    &whole_for("This one").replace("What it says.", "The newer."),
                ),
                ("notes.txt", "not a handoff"),
            ],
        );
        let found = Handed::newest_refused_for(&ours, "This one")
            .expect("the directory to be read")
            .expect("a refused handoff to be found");
        assert!(found.at.ends_with("200.toml"), "{:?}", found.at);
        assert_eq!(found.handed.body, "The newer.");
        assert!(found.written.contains("The newer."));
        drop(std::fs::remove_dir_all(&ours));
    }

    /// **No refused handoff for the task means [`None`]**, both when the
    /// directory holds only other tasks' and when it was never made.
    #[test]
    fn no_refused_handoff_for_the_task_is_none() {
        let ours = a_refused_directory("none", &[("100.toml", &whole_for("Another"))]);
        assert!(
            Handed::newest_refused_for(&ours, "This one")
                .expect("the directory to be read")
                .is_none()
        );
        drop(std::fs::remove_dir_all(&ours));

        let never = std::env::temp_dir().join(format!(
            "alo-handoff-refused-{}-never-made",
            std::process::id()
        ));
        drop(std::fs::remove_dir_all(&never));
        assert!(
            Handed::newest_refused_for(&never, "This one")
                .expect("a missing directory to be no refused handoffs")
                .is_none()
        );
    }

    /// **An entry that does not read is passed over**, rather than stopping
    /// the search or being taken for the task's.
    #[test]
    fn a_refused_entry_that_does_not_read_is_passed_over() {
        let ours = a_refused_directory(
            "unreadable",
            &[
                ("100.toml", &whole_for("This one")),
                ("200.toml", "task = This one\n"),
            ],
        );
        let found = Handed::newest_refused_for(&ours, "This one")
            .expect("the directory to be read")
            .expect("the readable one to be found");
        assert!(found.at.ends_with("100.toml"), "{:?}", found.at);
        drop(std::fs::remove_dir_all(&ours));
    }

    /// **A whole handoff reads**, so that the refusals below are about what is
    /// missing rather than about a parser that never worked.
    #[test]
    fn a_handoff_with_everything_a_commit_needs_reads() {
        let read = Handed::read(&whole()).expect("a whole handoff reads");
        assert_eq!(read.task, "Auditing what a bound turn can still reach");
        assert_eq!(read.report, "docs/autonomy/updates/an-audit.md");
        assert_eq!(read.files.len(), 2);
        assert_eq!(read.evidence.len(), 1);
        assert_eq!(read.body, "What it says.");
    }

    /// **A partial handoff publishes nothing, one missing piece at a time.**
    ///
    /// Every one of these is a task that does not reach `main`. The evidence
    /// block is the newest of them and the one that matters most: without it a
    /// task's whole claim is that the suite which was already there still
    /// passes.
    #[test]
    fn a_partial_handoff_publishes_nothing() {
        for key in ["task", "report", "subject", "evidence", "files"] {
            let refused = Handed::read(&without(key));
            assert!(
                refused.is_err_and(|why| why.contains(key)),
                "a handoff with no `{key}` was accepted"
            );
        }
    }

    /// **A handoff that does not publish its own report is refused**, because a
    /// task whose evidence stayed on somebody's disk is a task nobody can check.
    #[test]
    fn a_report_that_is_not_being_published_is_refused() {
        let elsewhere = whole().replace(
            "  docs/autonomy/updates/an-audit.md",
            "  docs/autonomy/updates/a-different-one.md",
        );
        assert!(Handed::read(&elsewhere).is_err_and(|why| why.contains("among its files")));
    }

    /// **A line of evidence that does not name one test is refused**, rather
    /// than guessed at — a guess would run some other test and pass.
    ///
    /// The line here is the likeliest mistake rather than nonsense: a crate, a
    /// target and a name, with the workspace left off. It used to be the whole
    /// format, and reading it as one would run the test from the wrong
    /// directory or not at all.
    #[test]
    fn evidence_that_does_not_name_one_test_is_refused() {
        for instead in [
            "  alo-bounding what_a_bound_turn_can_still_reach a_datagram_leaves_unchecked",
            "  it is tested",
            "  .",
        ] {
            let vague = whole().replace(
                "  . alo-bounding what_a_bound_turn_can_still_reach a_datagram_leaves_unchecked",
                instead,
            );
            assert!(
                Handed::read(&vague).is_err_and(|why| why.contains("not a piece of evidence")),
                "`{instead}` was accepted as evidence"
            );
        }
    }
}
