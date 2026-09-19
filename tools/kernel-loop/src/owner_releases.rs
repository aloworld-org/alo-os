//! Exact files released by their owner to one task in another plan.
//!
//! The owning plan records an owner-release fenced block in its header.
//! A release neither transfers a crate nor finishes any task. Malformed
//! records refuse publication; directories and patterns are not files.

use crate::who_owns::Claim;

const FENCE: &str = "```";

/// One explicit contribution recorded by the owning plan.
#[derive(Debug, PartialEq, Eq)]
pub struct Release {
    owner: String,
    receiving: String,
    task: u32,
    files: Vec<String>,
}

/// Read releases before the task boundary, never from task prose.
///
/// # Errors
/// An invalid or unterminated release, naming its owning plan.
pub fn read(owner: &str, written: &str) -> Result<Vec<Release>, String> {
    let header = written.split("\n## Tasks").next().unwrap_or(written);
    let mut lines = header.lines();
    let mut releases = Vec::new();
    while let Some(line) = lines.next() {
        if line.trim() != format!("{FENCE}owner-release") {
            continue;
        }
        let mut block = Vec::new();
        let mut closed = false;
        for line in lines.by_ref() {
            if line.trim() == FENCE {
                closed = true;
                break;
            }
            if !line.trim().is_empty() {
                block.push(line.trim());
            }
        }
        if !closed {
            return Err(format!("{owner}: an owner-release block is not closed"));
        }
        releases.push(parse(owner, &block)?);
    }
    Ok(releases)
}

/// A portable, exact repository file without patterns or traversal.
fn exact_file(file: &str) -> bool {
    !file.is_empty()
        && !file
            .chars()
            .any(|c| c.is_whitespace() || "\\:*?[]".contains(c))
        && file.split('/').all(|part| !matches!(part, "" | "." | ".."))
        && file.contains('/')
}

/// Read the small ordered record; unknown fields are not accepted.
fn parse(owner: &str, lines: &[&str]) -> Result<Release, String> {
    let invalid =
        || format!("{owner}: owner-release needs a plan, positive task number and exact files");
    let [plan, task, "files =", files @ ..] = lines else {
        return Err(invalid());
    };
    let receiving = plan.strip_prefix("plan = ").ok_or_else(invalid)?;
    let task = task
        .strip_prefix("task = ")
        .and_then(|number| number.parse::<u32>().ok())
        .filter(|number| *number > 0)
        .ok_or_else(invalid)?;
    if receiving == owner
        || !exact_file(receiving)
        || !receiving.starts_with("docs/autonomy/")
        || !receiving.ends_with("-plan.md")
        || files.is_empty()
        || files.iter().any(|file| !exact_file(file))
    {
        return Err(invalid());
    }
    let mut named = Vec::new();
    for file in files {
        if named.iter().any(|seen| seen == file) {
            return Err(format!("{owner}: an owner-release names {file} twice"));
        }
        named.push((*file).to_owned());
    }
    Ok(Release {
        owner: owner.to_owned(),
        receiving: receiving.to_owned(),
        task,
        files: named,
    })
}

/// Whether the actual owner released this exact file to this exact task.
///
/// Another unfinished owner's overlapping claim still refuses the file.
/// A receiver cannot grant itself access by writing in its own plan.
#[must_use]
pub fn permits(
    file: &str,
    receiving: &str,
    task: u32,
    claims: &[Claim],
    releases: &[Release],
) -> bool {
    exact_file(file)
        && releases.iter().any(|release| {
            release.receiving == receiving
                && release.task == task
                && release.files.iter().any(|named| named == file)
                && claims
                    .iter()
                    .any(|claim| claim.plan == release.owner && claim.holds(file))
                && claims
                    .iter()
                    .filter(|claim| claim.unfinished && claim.holds(file))
                    .all(|claim| claim.plan == release.owner)
        })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "an unexpected result is a test failure")]
mod tests {
    use super::*;
    use crate::{inside_the_plan, who_owns};

    const OWNER: &str = "docs/autonomy/documents-plan.md";
    const RECEIVER: &str = "docs/autonomy/broker-plan.md";
    const FILE: &str = "crates/alo-printing/src/found.rs";

    fn written(files: &str) -> String {
        format!(
            "**Crates this plan owns:** `alo-printing`.\n\n\
             {FENCE}owner-release\nplan = {RECEIVER}\ntask = 2\nfiles =\n{files}\n{FENCE}\n\n\
             ## Tasks\n\n### 1. Documents\n\n**Status:** ready. **Depends on:** nothing.\n"
        )
    }

    /// Only the released task/file passes; all remaining files still meet both checks.
    #[test]
    fn only_the_named_task_and_files_are_released() {
        let text = written(FILE);
        let claims = [who_owns::claimed_by(OWNER, &text)];
        let releases = read(OWNER, &text).unwrap();
        assert!(permits(FILE, RECEIVER, 2, &claims, &releases));
        assert!(!permits(FILE, RECEIVER, 3, &claims, &releases));
        assert!(!permits(FILE, OWNER, 2, &claims, &releases));
        let other = "crates/alo-printing/src/lib.rs";
        assert!(!permits(other, RECEIVER, 2, &claims, &releases));
        let files = [FILE.to_owned(), other.to_owned()];
        let checked: Vec<String> = files
            .into_iter()
            .filter(|file| !permits(file, RECEIVER, 2, &claims, &releases))
            .collect();
        assert_eq!(checked, [other]);
        assert!(who_owns::refusal(&checked, RECEIVER, &claims).is_some());
        let receiver = "**It reads and never edits** `alo-printing`.\n\n## Tasks\n";
        assert!(inside_the_plan::refusal(&checked, receiver, RECEIVER).is_some());
        assert!(claims.first().unwrap().unfinished);
    }

    /// Neither a stranger nor a duplicate claimant can release an owner's work.
    #[test]
    fn only_the_owner_can_release_and_competing_claims_still_refuse() {
        let text = written(FILE);
        let owner = who_owns::claimed_by(OWNER, &text);
        let releases = read(OWNER, &text).unwrap();
        let stranger = who_owns::claimed_by(
            "docs/autonomy/stranger-plan.md",
            "**Crates this plan owns:** `alo-printing`.\n\n## Tasks\n\n\
             ### 1. A task\n\n**Status:** ready. **Depends on:** nothing.\n",
        );
        assert!(!permits(
            FILE,
            RECEIVER,
            2,
            std::slice::from_ref(&stranger),
            &releases
        ));
        assert!(!permits(FILE, RECEIVER, 2, &[owner, stranger], &releases));
    }

    /// Malformed, broad and traversing records cannot become exceptions.
    #[test]
    fn malformed_or_broad_releases_are_refused() {
        for file in [
            "crates/alo-printing/",
            "crates/alo-printing/*",
            "/crates/alo-printing/src/lib.rs",
            "crates/alo-printing/../alo-shell/src/lib.rs",
            "crates\\alo-printing\\src\\lib.rs",
            "crates/alo-printing//src/lib.rs",
            "crates/alo-printing/src/?.rs",
        ] {
            assert!(read(OWNER, &written(file)).is_err(), "{file}");
        }
        for text in [
            written(FILE).replace("task = 2", "task = 0"),
            written(FILE).replace("task = 2", "task = two"),
            written(FILE).replace("files =", "directory ="),
            written(FILE).replace(&format!("\n{FENCE}\n"), "\n"),
            written(FILE).replace(RECEIVER, OWNER),
            written(&format!("{FILE}\n{FILE}")),
            written(""),
        ] {
            assert!(read(OWNER, &text).is_err(), "{text}");
        }
    }

    /// A task's quoted example is not authority to edit another owner's file.
    #[test]
    fn releases_after_the_task_boundary_are_not_read() {
        let text = format!("# A plan\n\n## Tasks\n\n{}", written(FILE));
        assert!(read(OWNER, &text).unwrap().is_empty());
    }
}
