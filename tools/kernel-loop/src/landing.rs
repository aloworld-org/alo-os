//! Landing a finished commit on `main` through a branch and a pull request.
//!
//! `main` is protected (`docs/autonomy/SHARED_MAIN.md`): nothing is pushed to it
//! directly, and a merge needs the branch to be up to date and to carry a
//! passing `alo/nine-gates` on that exact head. So the last step of publishing
//! is no longer a push — it is *land this commit*: put it on a branch of its
//! own, open a pull request, say that the gates passed on that head, merge it,
//! and remove the branch.
//!
//! # Why this shape, and not a second algorithm
//!
//! [`crate::publishing::gated_and_pushed`] already knows how to lose a race and
//! recover from it: gate, try to publish, and if `origin/main` moved underneath,
//! integrate it, gate the combination and try again. A merge refused because the
//! branch fell behind `main` is exactly that race with a different error
//! message, so it is reported as a failed publish and the loop above handles it
//! unchanged. Nothing here retries, integrates or gates; this module only knows
//! how to land a commit that is already believed good.
//!
//! # Why the fleet has no lock any more
//!
//! A shared claim ref was tried and removed: it was held by a machine that then
//! stopped, and `main` was frozen for eight hours on 2026-09-18 and fifteen on
//! 2026-09-19 with finished work nobody was permitted to merge. Protection is
//! `strict`, so a branch behind `main` cannot merge however green it looked —
//! which is the whole job the lock was doing, with no state that can stick.

use std::path::Path;

use crate::repository::{MAIN, git};

/// The repository every machine publishes to.
const REPOSITORY: &str = "aloworld-org/alo-os";

/// The check `main`'s protection requires before a merge.
const THE_CHECK: &str = "alo/nine-gates";

/// How a branch is named, before the subject: `task/<machine>/<subject>`.
const UNDER: &str = "task";

/// The longest a branch's subject may be.
///
/// Long enough to stay a sentence somebody recognises, short enough that a list
/// of branches is readable in a terminal.
const AT_MOST: usize = 60;

/// What this machine calls itself in a branch name.
///
/// The hostname, lowercased and reduced to what a branch may carry. A branch
/// says which machine made it so that a person reading the list knows who to
/// ask, and so two machines writing the same subject do not collide.
pub fn this_machine() -> String {
    let named = hostname().unwrap_or_else(|| "a-machine".to_owned());
    let named = as_a_branch_may_carry(&named);
    if named.is_empty() {
        "a-machine".to_owned()
    } else {
        named
    }
}

/// What names this machine, in the order worth believing.
///
/// `ALO_MACHINE` first, because a branch says which machine to ask and a
/// hostname often cannot: this development PC calls itself `HYB4GchZ1tnQNqB`,
/// which named a branch nobody could place. A machine told what it is called
/// says so; one that is not falls back to what the operating system thinks.
fn hostname() -> Option<String> {
    for named in ["ALO_MACHINE", "COMPUTERNAME", "HOSTNAME"] {
        if let Ok(found) = std::env::var(named)
            && !found.trim().is_empty()
        {
            return Some(found);
        }
    }
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|read| read.trim().to_owned())
        .filter(|read| !read.is_empty())
}

/// A subject reduced to what a git branch name may carry.
///
/// Lowercase letters, digits and single hyphens. Everything else becomes a
/// hyphen and runs of them collapse, so *`.docx`, `.xlsx` — opened* becomes
/// `docx-xlsx-opened` rather than something a shell needs quoting for.
pub fn as_a_branch_may_carry(subject: &str) -> String {
    let mut carried = String::with_capacity(subject.len());
    for character in subject.chars() {
        if character.is_ascii_alphanumeric() {
            carried.push(character.to_ascii_lowercase());
        } else if !carried.ends_with('-') {
            carried.push('-');
        }
    }
    carried.trim_matches('-').chars().take(AT_MOST).collect()
}

/// The branch a task's commit is landed on.
///
/// Named for the subject rather than the task's number, which is
/// `CLAUDE.md`'s rule for commit subjects and branches alike: a number means
/// nothing to somebody reading a list of branches a week later.
#[must_use]
pub fn the_branch_for(machine: &str, subject: &str) -> String {
    format!("{UNDER}/{machine}/{}", as_a_branch_may_carry(subject))
}

/// Whether a refused merge is a branch that fell behind `main`.
///
/// GitHub answers a stale or unmergeable pull request with 405 and a sentence;
/// that is the lost race, and the publishing loop above integrates and gates
/// again. Anything else — a missing check, no permission, a closed pull request
/// — is a real error, and retrying it would only repeat it.
#[must_use]
pub fn is_a_lost_race(answered: &str) -> bool {
    let answered = answered.to_ascii_lowercase();
    [
        "not mergeable",
        "base branch was modified",
        "merge conflict",
    ]
    .iter()
    .any(|said| answered.contains(said))
}

/// Land a commit that is already gated: branch, pull request, check, merge.
///
/// Returns the sha `main` carries afterwards.
///
/// # Errors
/// When any step is refused. A refusal that reads as a lost race is left for
/// [`crate::publishing::gated_and_pushed`] to integrate and try again; the
/// commit stays in this checkout either way, and nothing is discarded.
pub fn landed(at: &Path, subject: &str, note: &mut dyn FnMut(&str)) -> Result<String, String> {
    let machine = this_machine();
    let branch = the_branch_for(&machine, subject);
    let head = git(at, &["rev-parse", "HEAD"])?;

    note(&format!("landing {} on `{branch}`", short(&head)));

    // A branch of this name may be left from an attempt that lost a race. It
    // holds a commit this machine made and has since rebased, so it is this
    // machine's to remove — and removing it is not a force push over anybody's
    // published history.
    if remote_has(at, &branch)? {
        note("a branch of that name is left from an earlier attempt; removing it");
        drop(git(
            at,
            &["push", "origin", &format!(":refs/heads/{branch}")],
        ));
    }
    git(
        at,
        &["push", "origin", &format!("HEAD:refs/heads/{branch}")],
    )?;

    let number = the_pull_request(at, &branch, subject, &machine)?;
    note(&format!("opened pull request {number}"));

    say_the_gates_passed(at, &head)?;
    let landed = merged(at, number)?;

    drop(git(
        at,
        &["push", "origin", &format!(":refs/heads/{branch}")],
    ));
    note(&format!(
        "landed as {} and removed `{branch}`",
        short(&landed)
    ));
    Ok(landed)
}

/// The first seven characters of a sha, for a sentence somebody reads.
fn short(sha: &str) -> &str {
    let at = sha.char_indices().nth(7).map_or(sha.len(), |(at, _)| at);
    &sha[..at]
}

/// Whether the remote already carries this branch.
fn remote_has(at: &Path, branch: &str) -> Result<bool, String> {
    let found = git(at, &["ls-remote", "--heads", "origin", branch])?;
    Ok(!found.trim().is_empty())
}

/// Open the pull request, or find the one already open for this branch.
fn the_pull_request(at: &Path, branch: &str, subject: &str, machine: &str) -> Result<u64, String> {
    let body = format!(
        "Published by the loop on {machine}, which gated this commit before landing it.\n\n\
         The nine gates ran on the committed tree and the task's own acceptance evidence stood \
         up; `{THE_CHECK}` on this head says so. See the task's report in \
         `docs/autonomy/updates/` for what was built and what is still owed."
    );
    let asked = format!(
        "{{\"title\":{},\"head\":{},\"base\":{},\"body\":{}}}",
        as_json(subject),
        as_json(branch),
        as_json(MAIN),
        as_json(&body)
    );
    let answered = github(at, "POST", "pulls", Some(&asked))?;
    if let Some(number) = number_in(&answered) {
        return Ok(number);
    }
    // Already open for this branch, which happens when a previous attempt
    // opened one and then lost the race at the merge.
    let open = github(
        at,
        "GET",
        &format!("pulls?state=open&head=aloworld-org:{branch}"),
        None,
    )?;
    number_in(&open).ok_or_else(|| {
        format!("the pull request for `{branch}` was neither opened nor found: {answered}")
    })
}

/// The first `"number"` a GitHub answer carries.
///
/// Read by hand rather than parsed: this supervisor has no dependencies, and
/// one number out of an answer does not justify the first one.
fn number_in(answered: &str) -> Option<u64> {
    let at = answered.find("\"number\"")?;
    let rest = answered.get(at + "\"number\"".len()..)?;
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// A string as JSON carries it, with what JSON must escape escaped.
fn as_json(said: &str) -> String {
    let mut written = String::with_capacity(said.len() + 2);
    written.push('"');
    for character in said.chars() {
        match character {
            '"' => written.push_str("\\\""),
            '\\' => written.push_str("\\\\"),
            '\n' => written.push_str("\\n"),
            '\r' => written.push_str("\\r"),
            '\t' => written.push_str("\\t"),
            other if (other as u32) < 0x20 => {
                written.push_str(&format!("\\u{:04x}", u32::from(other)));
            }
            other => written.push(other),
        }
    }
    written.push('"');
    written
}

/// The text of a `"name": "value"` field, where the answer carries one.
fn text_in(answered: &str, named: &str) -> Option<String> {
    let key = format!("\"{named}\"");
    let at = answered.find(&key)?;
    let rest = answered.get(at + key.len()..)?;
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut read = String::new();
    let mut characters = rest.chars();
    while let Some(character) = characters.next() {
        match character {
            '"' => return Some(read),
            '\\' => match characters.next()? {
                'n' => read.push('\n'),
                't' => read.push('\t'),
                'r' => read.push('\r'),
                other => read.push(other),
            },
            other => read.push(other),
        }
    }
    None
}

/// Whether the answer says the merge happened.
fn says_it_merged(answered: &str) -> bool {
    let Some(at) = answered.find("\"merged\"") else {
        return false;
    };
    answered
        .get(at + "\"merged\"".len()..)
        .map(|rest| rest.trim_start().trim_start_matches(':').trim_start())
        .is_some_and(|rest| rest.starts_with("true"))
}

/// Say, on this exact head, that the gates passed.
///
/// The check is the only thing `main`'s protection asks for, and it is reported
/// **after** the gates have run rather than before: a check posted first is a
/// claim about work nobody has looked at.
fn say_the_gates_passed(at: &Path, head: &str) -> Result<(), String> {
    let asked = format!(
        "{{\"state\":\"success\",\"context\":{},\"description\":{}}}",
        as_json(THE_CHECK),
        as_json("the nine gates and this task's evidence passed on this commit")
    );
    github(at, "POST", &format!("statuses/{head}"), Some(&asked)).map(drop)
}

/// Merge it, squashed, and answer with the sha `main` then carries.
fn merged(at: &Path, number: u64) -> Result<String, String> {
    let asked = "{\"merge_method\":\"squash\"}";
    let answered = github(at, "PUT", &format!("pulls/{number}/merge"), Some(asked))?;
    if says_it_merged(&answered) {
        return text_in(&answered, "sha")
            .ok_or_else(|| format!("the merge said it merged and named no sha: {answered}"));
    }
    Err(text_in(&answered, "message").unwrap_or(answered))
}

/// One call to GitHub, with the credential git already holds.
///
/// The token is read from git's credential helper rather than kept anywhere of
/// ours: this machine can already push, so it can already merge, and a second
/// copy of a credential is a second thing to leak.
fn github(at: &Path, how: &str, path: &str, asked: Option<&str>) -> Result<String, String> {
    let token = the_token(at)?;
    let mut arguments = vec![
        "--silent".to_owned(),
        "--show-error".to_owned(),
        "-X".to_owned(),
        how.to_owned(),
        "-H".to_owned(),
        format!("Authorization: token {token}"),
        "-H".to_owned(),
        "Accept: application/vnd.github+json".to_owned(),
    ];
    if let Some(asked) = asked {
        arguments.push("-H".to_owned());
        arguments.push("Content-Type: application/json".to_owned());
        arguments.push("--data".to_owned());
        arguments.push(asked.to_owned());
    }
    arguments.push(format!("https://api.github.com/repos/{REPOSITORY}/{path}"));

    let ran = std::process::Command::new("curl")
        .args(&arguments)
        .current_dir(at)
        .output()
        .map_err(|why| format!("curl could not be run to reach GitHub: {why}"))?;
    if !ran.status.success() {
        return Err(format!(
            "curl refused reaching GitHub: {}",
            String::from_utf8_lossy(&ran.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&ran.stdout).into_owned())
}

/// The token git already holds for `github.com`.
fn the_token(at: &Path) -> Result<String, String> {
    use std::io::Write as _;

    let mut asked = std::process::Command::new("git")
        .args(["credential", "fill"])
        .current_dir(at)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|why| format!("git's credential helper could not be asked: {why}"))?;
    asked
        .stdin
        .as_mut()
        .ok_or_else(|| "git's credential helper took no input".to_owned())?
        .write_all(b"protocol=https\nhost=github.com\n\n")
        .map_err(|why| format!("git's credential helper could not be asked: {why}"))?;
    let answered = asked
        .wait_with_output()
        .map_err(|why| format!("git's credential helper did not answer: {why}"))?;
    String::from_utf8_lossy(&answered.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("password="))
        .map(str::to_owned)
        .ok_or_else(|| {
            "git holds no credential for github.com on this machine, so this loop cannot merge \
             what it gated. Sign in once with git, or push by hand."
                .to_owned()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A branch is named for its subject**, in what a branch may carry.
    #[test]
    fn a_subject_becomes_a_branch_name() {
        assert_eq!(
            the_branch_for("dev-pc", "Keyboards: layouts, dead keys, compose"),
            "task/dev-pc/keyboards-layouts-dead-keys-compose"
        );
        assert_eq!(
            as_a_branch_may_carry("`.docx`, `.xlsx`, `.pptx` — opened"),
            "docx-xlsx-pptx-opened"
        );
    }

    /// **Nothing a branch may not carry survives**, and no name ends in the
    /// separator a run of punctuation would otherwise leave.
    #[test]
    fn a_name_carries_only_what_a_branch_may() {
        for (subject, expected) in [
            ("  spaces  ", "spaces"),
            ("...", ""),
            ("A/B\\C", "a-b-c"),
            ("two--hyphens", "two-hyphens"),
            ("trailing —", "trailing"),
        ] {
            assert_eq!(as_a_branch_may_carry(subject), expected, "{subject}");
        }
    }

    /// **A long subject is cut rather than carried whole**, and is still a
    /// sentence somebody recognises.
    #[test]
    fn a_long_subject_is_cut_to_a_readable_length() {
        let long = "a".repeat(200);
        let carried = as_a_branch_may_carry(&long);
        assert_eq!(carried.len(), AT_MOST);
    }

    /// **A stale branch is a lost race and everything else is not.** The first
    /// is integrated and gated again by the loop above; retrying the second
    /// would repeat it forever.
    #[test]
    fn a_stale_branch_reads_as_a_lost_race() {
        for said in [
            "Pull Request is not mergeable",
            "Base branch was modified. Review and try the merge again.",
            "merge conflict between base and head",
        ] {
            assert!(is_a_lost_race(said), "{said}");
        }
        for said in [
            "Required status check \"alo/nine-gates\" is expected.",
            "Resource not accessible by integration",
            "Pull Request is still a draft",
        ] {
            assert!(!is_a_lost_race(said), "{said}");
        }
    }

    /// **A machine always has a name for a branch**, even where the operating
    /// system will not say what it is.
    #[test]
    fn this_machine_always_has_a_name() {
        let named = this_machine();
        assert!(!named.is_empty());
        assert_eq!(named, as_a_branch_may_carry(&named));
    }

    /// **The number is read from either shape GitHub answers with** — the one
    /// pull request it opened, or the list it was searched for in.
    #[test]
    fn a_pull_requests_number_is_read_from_either_answer() {
        assert_eq!(number_in(r#"{"number":7}"#), Some(7));
        assert_eq!(number_in(r#"[{"number":9},{"number":11}]"#), Some(9));
        assert_eq!(number_in(r#"{"message":"Validation Failed"}"#), None);
        assert_eq!(number_in("not json at all"), None);
    }
}
