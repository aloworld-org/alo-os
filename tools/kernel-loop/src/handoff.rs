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

use std::path::Path;

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

    /// The file, read.
    fn read(written: &str) -> Result<Self, String> {
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
