//! What a contributor hands the loop when a task's code is finished.
//!
//! One file, `.kernel-loop/handoff.toml`, naming the task, the files it
//! touched, the commit it becomes and the report it published. The loop stages
//! **exactly those files** and no others — `git add -A` would sweep up whatever
//! else happened to be in the tree, which is how an unrelated change gets
//! published under somebody else's commit message.
//!
//! ```text
//! task = Reproducing unrestricted network access from inside a bound turn
//! report = docs/autonomy/updates/network-egress-reproduction.md
//! subject = test(bounding): reproduce a bound turn opening any socket
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
                "files" | "body" => {}
                _ => {}
            }
            inside = match key {
                "files" => Some("files"),
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
        Ok(Self {
            task,
            report,
            subject,
            body: body.trim_end().to_owned(),
            files,
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
