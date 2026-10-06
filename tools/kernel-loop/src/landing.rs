//! Landing a finished commit on `main` through a branch and a pull request.
//!
//! `main` is protected (`docs/autonomy/SHARED_MAIN.md`): nothing is pushed to it
//! directly. So the last step of publishing is not a push — it is *offer this
//! commit*: put it on a branch of its own, open a pull request, **wait for the
//! checks `main` requires**, and put it in the merge queue. The queue merges it.
//!
//! # What this module stopped doing on 2026-10-02, and why both halves were one mistake
//!
//! It used to post its own status — `const THE_CHECK: &str = "alo/nine-gates"` — and then
//! `PUT pulls/{n}/merge` itself. The owner moved `main` to require
//! `alo/gates-on-a-runner`, which CI posts, and **nothing required the loop's status any
//! more**. A loop started that day would have gated for minutes, posted a status nothing
//! reads, and waited on a merge that could not happen: a lane that looks busy and lands
//! nothing.
//!
//! Renaming the constant would have fixed that once. [`what_main_requires`] asks protection
//! instead, so the next rename needs no change here — **the name of a required check is
//! GitHub's state, not this program's opinion.**
//!
//! The direct merge was the same fault from the other end: the thing that measured and the
//! thing that merged were one program, so the one question a direct merge cannot ask went
//! unasked — *does this still pass combined with whatever landed while it waited*. The queue
//! builds a candidate on current `main` and re-runs the checks against it. The loop's job now
//! ends at *this is ready*.
//!
//! **The local gates did not go away and are not the verdict.** They still run before the
//! push, because catching a failure here costs a minute and catching it on a runner costs a
//! queue slot. What changed is which answer decides.
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
//! 2026-09-19 with finished work nobody was permitted to merge.
//!
//! **This paragraph used to say protection was `strict`, and it is not** — measured on
//! 2026-10-02, `strict` is false, because the merge queue builds its own candidate on current
//! `main` and so does not need the branch to be up to date first. What does the lock's old job
//! is the queue itself: it serialises the one thing that has to be serialised, and a branch
//! that has fallen behind is rebuilt rather than refused. There is no state that can stick.

use std::path::Path;

use crate::repository::{MAIN, git};

/// The repository every machine publishes to.
const REPOSITORY: &str = "aloworld-org/alo-os";

/// How long to wait for the checks `main` requires, and how often to look.
///
/// **Sixty minutes because that is the merge queue's own window.** A loop that gave up sooner
/// would abandon an entry the queue was still building, and one that waited for ever would
/// hold a worker on a run that had died.
const LONG_ENOUGH_FOR_A_RUN: std::time::Duration = std::time::Duration::from_secs(60 * 60);

/// How often to ask whether the required checks have finished.
///
/// A hosted run takes minutes, so a tighter poll buys nothing but requests against a rate
/// limit the whole fleet shares.
const BETWEEN_LOOKS: std::time::Duration = std::time::Duration::from_secs(30);

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
///
/// **Five sources, and the last one exists because four were not enough.**
/// On macOS none of the first four answer — the variables are unset in a
/// non-interactive shell and `/etc/hostname` is a Linux file — so every Mac
/// landing was named `a-machine` until 2026-10-06. A fallback chain that ends
/// in a placeholder will reach the placeholder on whatever platform its author
/// did not have in front of them, and nothing goes red when it does.
fn hostname() -> Option<String> {
    for named in ["ALO_MACHINE", "COMPUTERNAME", "HOSTNAME"] {
        if let Ok(found) = std::env::var(named)
            && !found.trim().is_empty()
        {
            return Some(found);
        }
    }
    if let Some(named) = std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|read| read.trim().to_owned())
        .filter(|read| !read.is_empty())
    {
        return Some(named);
    }
    // **And then ask the operating system, because macOS has no
    // `/etc/hostname`.** Every lane on a Mac fell through all four of the
    // sources above — the environment variables are not exported to a
    // non-interactive shell and the file does not exist — and landed on
    // `a-machine`, which is the exact branch nobody can place that this
    // function's own documentation warns about. Measured 2026-10-06 on a
    // branch named `task/a-machine/feat-shell-a-frame-is-drawn-per-display`.
    std::process::Command::new("hostname")
        .output()
        .ok()
        .filter(|ran| ran.status.success())
        .map(|ran| String::from_utf8_lossy(&ran.stdout).trim().to_owned())
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

/// Offer a commit that is already gated: branch, pull request, wait for CI, queue.
///
/// Returns **the head that was offered**, not the sha `main` ends up carrying. It used to
/// return the latter because it merged the thing itself; now the queue merges, later, after
/// building its own candidate — so the sha `main` will carry does not exist yet and inventing
/// one would be a claim about work that has not happened.
///
/// That is a change callers can see, and it is the honest shape: *this is offered* is the last
/// thing this machine knows for certain.
///
/// # Errors
/// When any step is refused, when a required check fails, or when the checks have not finished
/// inside the queue's own window. A refusal that reads as a lost race is left for
/// [`crate::publishing::gated_and_pushed`] to integrate and try again; the commit stays in this
/// checkout either way, and nothing is discarded.
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

    waited_for(at, &head, note)?;
    enqueued(at, number)?;

    // **The branch is not removed here any more.** The queue builds its candidate from this
    // branch, so deleting it now would delete the thing about to be tested. The queue removes
    // it on merge, which is what `delete_branch_on_merge` is for.
    note(&format!(
        "pull request {number} is in the merge queue on `{branch}`; the queue merges it"
    ));
    Ok(head)
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
    // **The body claims what this machine did and nothing about what will happen.** It used to
    // name the status the loop posted itself, which is the sentence that went stale when `main`
    // stopped requiring it. What is said here is only ever this machine's own measurement; the
    // verdict that governs is CI's, on this head and again on the queue's candidate.
    let body = format!(
        "Published by the loop on {machine}, which gated this commit before pushing it.\n\n\
         The nine gates ran on the committed tree and the task's own acceptance evidence stood \
         up **on that machine**. That is a pre-check and not the verdict: the checks `main` \
         requires decide, on this head and again on the queue's own candidate. See the task's \
         report in `docs/autonomy/updates/` for what was built and what is still owed."
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

/// What `main`'s protection requires right now, asked rather than remembered.
///
/// # This is the whole reason the cutover cannot go stale the way the last one did
///
/// Until 2026-10-02 this module held `const THE_CHECK: &str = "alo/nine-gates"` and posted it
/// itself. The owner moved `main` to require `alo/gates-on-a-runner`, which CI posts, and
/// **nothing requires `alo/nine-gates` any more** — so a loop started that day would have
/// gated for minutes, posted a status nothing reads, and waited on a merge that could never
/// happen. A lane that looks busy and lands nothing.
///
/// A renamed constant would have fixed that once. Asking protection fixes it every time: the
/// next rename, or a second required check, needs no change here. **The name of a required
/// check is GitHub's state, not this program's opinion**, and the fault being removed is
/// exactly a program holding its own copy of somebody else's answer.
///
/// An empty answer is returned as such rather than as an error: a repository with no required
/// checks is a real configuration, and the caller decides what to do about it.
fn what_main_requires(at: &Path) -> Result<Vec<String>, String> {
    let answered = github(at, "GET", &format!("branches/{MAIN}/protection"), None)?;
    Ok(contexts_in(&answered))
}

/// The required contexts named in a protection answer.
///
/// **Pure, so it can be tested against the shapes GitHub actually sends.** The I/O above it
/// cannot be, and a parser that is only exercised through a network call is a parser nobody has
/// seen handle an empty list, a null, or two entries.
fn contexts_in(answered: &str) -> Vec<String> {
    let Some(at_contexts) = answered.find("\"contexts\"") else {
        return Vec::new();
    };
    let rest = answered
        .get(at_contexts..)
        .unwrap_or_default()
        .trim_start_matches("\"contexts\"")
        .trim_start()
        .trim_start_matches(':')
        .trim_start();
    let Some(list) = rest.strip_prefix('[') else {
        return Vec::new();
    };
    // **An unclosed list reads as none rather than as an error.** The caller refuses on an
    // empty answer anyway, so a malformed reply and a missing one lead to the same safe place:
    // nothing is queued. Returning an error here would make a parser decide policy.
    let Some(end) = list.find(']') else {
        return Vec::new();
    };
    list.get(..end)
        .unwrap_or_default()
        .split(',')
        .filter_map(|each| {
            let named = each.trim().trim_matches('"').trim();
            (!named.is_empty()).then(|| named.to_owned())
        })
        .collect()
}

/// Whether every required check has succeeded on this head, or why not yet.
///
/// Four answers rather than two, because *not finished*, *failed* and *I could not read the
/// reply* are different things a caller does different things about: the first is worth
/// waiting for and the other two never are. The fourth was added on 2026-10-06, after the
/// third spent fifty minutes wearing the first's clothes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TheChecks {
    /// Every required check has succeeded on this commit.
    AllPassed,
    /// At least one has not finished. Worth another look.
    StillRunning(String),
    /// At least one finished and did not succeed. Waiting cannot help.
    OneFailed(String),
    /// The reply was not a status answer, so nothing can be read from it.
    ///
    /// **Separate from `StillRunning` because waiting cannot fix it**, and
    /// because the two were one answer until 2026-10-06 and that cost an hour.
    /// Bad credentials, a renamed repository and a commit GitHub has never
    /// heard of all reply with a `message` and no `statuses`; read as *has not
    /// reported yet*, each of them spins until the deadline and then reports
    /// that **CI** was slow, which is a true-sounding sentence about the wrong
    /// machine.
    Unreadable(String),
}

/// Ask what the required checks say about this head.
///
/// **A missing context reads as still running, not as passing.** That is the distinction this
/// whole evening turned on: an absent answer is not a negative one, and a loop that treated
/// *no result yet* as *nothing wrong* would enqueue before CI had spoken and be refused —
/// or worse, on a repository with no protection, merge something nobody measured.
fn what_the_checks_say(at: &Path, head: &str, required: &[String]) -> Result<TheChecks, String> {
    let answered = github(at, "GET", &format!("commits/{head}/status"), None)?;
    Ok(read_the_checks(&answered, required))
}

/// What a status answer says about the required checks.
///
/// Pure, for the reason [`contexts_in`] is: the three outcomes are the decision, and a decision
/// exercised only through a network call is one nobody has watched handle *not reported yet*.
fn read_the_checks(answered: &str, required: &[String]) -> TheChecks {
    // **Is this a status answer at all?** Every real one carries `statuses`,
    // even when the list is empty; every refusal carries `message` instead.
    // Asked before any context is looked for, so *I could not read this* can
    // never leave here dressed as *it has not reported yet*.
    if !answered.contains("\"statuses\"") {
        let why = text_in(answered, "message")
            .unwrap_or_else(|| answered.chars().take(200).collect::<String>());
        return TheChecks::Unreadable(format!(
            "GitHub did not answer with a status for this commit: {why}"
        ));
    }
    for named in required {
        let Some(said) = the_state_of(answered, named) else {
            return TheChecks::StillRunning(format!(
                "`{named}` has not reported on this commit yet"
            ));
        };
        match said.as_str() {
            "success" => {}
            "pending" => return TheChecks::StillRunning(format!("`{named}` is {said}")),
            _ => return TheChecks::OneFailed(format!("`{named}` is {said}")),
        }
    }
    TheChecks::AllPassed
}

/// The state reported for one named context, if it has reported at all.
///
/// Reads the statuses list rather than the summary `state`, because the summary is an answer
/// about **every** context including ones protection does not require — so a repository with
/// an unrelated failing check would read as failed when the required ones all passed.
fn the_state_of(answered: &str, named: &str) -> Option<String> {
    // **The key is found, then its value is read** — never a `"key":"value"`
    // pair built by hand and searched for literally. GitHub pretty-prints, so
    // it sends `"context": "alo/gates-on-a-runner"` with a space, and the
    // literal spelled without one matched nothing on every real reply while
    // matching every fixture in this file. See
    // `docs/misreadings/a-fixture-i-wrote-agreed-with-the-parser-i-wrote.md`.
    const KEY: &str = "\"context\"";
    let mut from = 0_usize;
    while let Some(at) = answered.get(from..).and_then(|rest| rest.find(KEY)) {
        let at = from.saturating_add(at);
        let after = answered.get(at..).unwrap_or_default();
        if text_in(after, "context").as_deref() == Some(named) {
            // The state for an entry may be written before or after its context, so look in
            // the object around it rather than only forwards.
            let before = answered.get(..at).unwrap_or_default();
            let from_the_object = before
                .rfind('{')
                .map_or(after, |opened| answered.get(opened..).unwrap_or_default());
            if let Some(state) = text_in(from_the_object, "state") {
                return Some(state);
            }
        }
        from = at.saturating_add(KEY.len());
    }
    None
}

/// Put it in the merge queue, and let the queue be the thing that merges.
///
/// # Why this replaced a direct merge
///
/// This module used to `PUT pulls/{n}/merge` itself, after posting its own status. Both halves
/// were the same mistake from different ends: the loop was deciding that something was safe to
/// land and then landing it, so **the thing that measured and the thing that merged were the
/// same program**. The merge queue builds an integration candidate on current `main` and
/// re-runs the checks against it, which is the one question a direct merge cannot ask — *does
/// this still pass combined with whatever landed while it was waiting*.
///
/// So the loop's job ends at *this is ready*. The queue's job is *and it still works next to
/// everything else*. Nothing here merges anything.
///
/// # Errors
///
/// A refusal naming a required check means the loop got here before CI did, which
/// [`waited_for`] exists to prevent and which is reported rather than retried — retrying a
/// refusal that is about ordering would spin.
fn enqueued(at: &Path, number: u64) -> Result<(), String> {
    let node = the_node_of(at, number)?;
    let asked = format!(
        "{{\"query\":{},\"variables\":{{\"id\":{}}}}}",
        as_json(
            "mutation($id:ID!){enqueuePullRequest(input:{pullRequestId:$id})\
             {mergeQueueEntry{position state}}}"
        ),
        as_json(&node)
    );
    // **The reply is read, not the exit status**, and the transient refusal is retried.
    // `curl` exits 0 on a 200 that carries `"errors"`, so a request that *succeeded* while the
    // enqueue was *refused* would otherwise report a landing that never happened — the fault
    // this lane fixed in its own enqueue script on 2026-10-01, and the one the shell lane spent
    // a night reading `exit 0` out of a pipeline whose `tail` always succeeded.
    let mut attempt = 1;
    loop {
        let answered = graphql(at, &asked)?;
        match the_queue_refused(&answered) {
            None => return Ok(()),
            Some(why) if is_worth_asking_again(&why) && attempt < AT_MOST_ASKS => {
                attempt += 1;
                std::thread::sleep(BETWEEN_LOOKS);
            }
            Some(why) => return Err(format!("the queue refused: {why}")),
        }
    }
}

/// How many times a transient refusal is worth retrying before it is a real one.
const AT_MOST_ASKS: u32 = 5;

/// Why the queue refused, or [`None`] if it took the entry.
///
/// **Three ways a reply can mean *not queued*, and all three are read**: an `errors` array, a
/// null entry under no error at all, and a body with no entry in it. The middle one is the
/// quiet one — a 200, no error, and nothing queued — and a reader that only looked for `errors`
/// would call it a landing.
fn the_queue_refused(answered: &str) -> Option<String> {
    if answered.contains("\"errors\"") {
        return Some(text_in(answered, "message").unwrap_or_else(|| answered.to_owned()));
    }
    if answered.contains("\"mergeQueueEntry\":null")
        || answered.contains("\"enqueuePullRequest\":null")
        || !answered.contains("\"mergeQueueEntry\"")
    {
        return Some(format!(
            "the queue answered without an entry, so nothing is queued: {answered}"
        ));
    }
    None
}

/// Whether a refusal is the one that fixes itself if you wait.
///
/// GitHub answers `UNPROCESSABLE: mergeability check has not yet completed` for a pull request
/// it has not finished testing for conflicts. **That resolves in seconds and is not a
/// refusal**; every other one will not resolve at all. A loop that retried a real refusal would
/// spin, and one that gave up on this would lose a turn to a race it had already won.
fn is_worth_asking_again(why: &str) -> bool {
    why.to_ascii_lowercase()
        .contains("mergeability check has not yet completed")
}

/// The pull request's node id, which the queue mutation needs and REST does not give.
fn the_node_of(at: &Path, number: u64) -> Result<String, String> {
    let answered = github(at, "GET", &format!("pulls/{number}"), None)?;
    text_in(&answered, "node_id").ok_or_else(|| {
        format!("pull request {number} was read and named no node_id, so it cannot be queued")
    })
}

/// Reach GitHub's GraphQL endpoint, which is not under `/repos`.
///
/// Its own function rather than a parameter on [`github`], because that one builds a
/// repository-scoped REST path and this is a different endpoint with a different shape. One
/// function doing both would take a flag that changes what every other argument means.
fn graphql(at: &Path, asked: &str) -> Result<String, String> {
    let token = the_token(at)?;
    let ran = std::process::Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "-X",
            "POST",
            "-H",
            &format!("Authorization: bearer {token}"),
            "-H",
            "Content-Type: application/json",
            "--data",
            asked,
            "https://api.github.com/graphql",
        ])
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

/// Wait until the required checks have spoken about this head, or give up saying why.
///
/// **Waiting is the loop's work now, and it is not the same as gating.** The loop still runs
/// the nine gates before it pushes, because catching a failure here costs a minute and
/// catching it on a runner costs a queue slot. What changed is which verdict *decides*: CI's.
/// So the local gate is a pre-check and this is where the answer comes from.
fn waited_for(at: &Path, head: &str, note: &mut dyn FnMut(&str)) -> Result<(), String> {
    let required = what_main_requires(at)?;
    if required.is_empty() {
        return Err(format!(
            "`{MAIN}` requires no checks, so nothing here can tell a measured commit from an \
             unmeasured one. Refusing to queue rather than guessing."
        ));
    }
    note(&format!(
        "waiting for {} on this head: {}",
        if required.len() == 1 {
            "the check"
        } else {
            "the checks"
        },
        required.join(", ")
    ));

    let began = std::time::Instant::now();
    loop {
        match what_the_checks_say(at, head, &required)? {
            TheChecks::AllPassed => return Ok(()),
            TheChecks::OneFailed(why) => {
                return Err(format!("a required check did not pass: {why}"));
            }
            TheChecks::Unreadable(why) => return Err(why),
            TheChecks::StillRunning(why) => {
                if began.elapsed() >= LONG_ENOUGH_FOR_A_RUN {
                    return Err(format!(
                        "the required checks had not finished after {} minutes: {why}",
                        LONG_ENOUGH_FOR_A_RUN.as_secs() / 60
                    ));
                }
                std::thread::sleep(BETWEEN_LOOKS);
            }
        }
    }
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
    /// **What `main` requires is read from protection, not remembered.**
    ///
    /// The cutover of 2026-10-02 turned on this: the old code held `alo/nine-gates` as a
    /// constant and posted it, and when the owner moved protection to
    /// `alo/gates-on-a-runner` nothing required the loop's status any more. Parsing the live
    /// answer is what stops the next rename being silent.
    #[test]
    fn the_required_checks_are_read_out_of_what_protection_answers() {
        let answered =
            r#"{"required_status_checks":{"strict":false,"contexts":["alo/gates-on-a-runner"]}}"#;
        assert_eq!(
            contexts_in(answered),
            vec!["alo/gates-on-a-runner".to_owned()],
            "the required check was not read out of protection"
        );
    }

    /// **Two required checks are both read**, because a repository may grow a second one and a
    /// reader that took only the first would queue on half an answer.
    #[test]
    fn every_required_check_is_read_and_not_just_the_first() {
        let answered = r#"{"required_status_checks":{"contexts":["one","two"]}}"#;
        assert_eq!(
            contexts_in(answered),
            vec!["one".to_owned(), "two".to_owned()]
        );
    }

    /// **No required checks reads as none**, which the caller refuses on rather than treating
    /// as permission. A repository with nothing required cannot tell a measured commit from an
    /// unmeasured one, and queueing there would be the loop merging something nobody looked at.
    #[test]
    fn protection_with_no_required_checks_reads_as_none_rather_than_as_anything() {
        assert!(contexts_in(r#"{"required_status_checks":null}"#).is_empty());
        assert!(contexts_in("{}").is_empty());
    }

    /// **A check that has not reported is still running, never passing.**
    ///
    /// The distinction the whole fleet spent 2026-10-01 on: an absent answer is not a negative
    /// one. A loop that read *no result yet* as *nothing wrong* would queue before CI had
    /// spoken.
    #[test]
    fn a_check_that_has_not_reported_is_not_a_check_that_passed() {
        let required = vec!["alo/gates-on-a-runner".to_owned()];
        let nothing_yet = r#"{"state":"pending","statuses":[]}"#;
        assert!(matches!(
            read_the_checks(nothing_yet, &required),
            TheChecks::StillRunning(_)
        ));
    }

    /// **The required check is read by name, not from the summary state.**
    ///
    /// The summary is about every context including ones protection does not require, so an
    /// unrelated failing check would read as failed when the required one passed — and an
    /// unrelated passing one could read as success when the required one had not run.
    #[test]
    fn an_unrelated_failing_check_does_not_decide_a_required_one() {
        let required = vec!["alo/gates-on-a-runner".to_owned()];
        let said = r#"{"state":"failure","statuses":[
            {"state":"failure","context":"some/other-thing"},
            {"state":"success","context":"alo/gates-on-a-runner"}]}"#;
        assert_eq!(read_the_checks(said, &required), TheChecks::AllPassed);
    }

    /// **GitHub pretty-prints, and the parser reads what GitHub sends.**
    ///
    /// The shape below is the real one, taken from
    /// `GET /repos/.../commits/<sha>/status` on 2026-10-06 — a space after
    /// every colon. The parser spelled its search `"context":"<name>"` with
    /// no space, so it matched **nothing** on every real reply and
    /// **everything** in the fixtures above, which are hand-written compact.
    ///
    /// The cost, measured: pull request 542's required check passed at 16:52
    /// and the loop was still reporting *has not reported on this commit yet*
    /// fifty minutes later, on its way to announcing that CI had been slow.
    ///
    /// Every other fixture in this file stays compact on purpose. Both shapes
    /// are valid JSON and the parser owes both an answer; what it may not do
    /// is work on only the one its author happened to type.
    #[test]
    fn the_shape_github_actually_sends_is_read() {
        let required = vec!["alo/gates-on-a-runner".to_owned()];
        let as_github_sends_it = r#"{
  "state": "success",
  "statuses": [
    {
      "url": "https://api.github.com/repos/aloworld-org/alo-os/statuses/2ffe6d90",
      "id": 123456789,
      "state": "success",
      "description": "the gates that run on a hosted runner, by the workflow",
      "context": "alo/gates-on-a-runner",
      "created_at": "2026-10-06T16:52:09Z",
      "creator": {
        "login": "github-actions[bot]",
        "state": "irrelevant"
      }
    }
  ],
  "sha": "2ffe6d90489c8cd83361b05e27826709506b3461",
  "total_count": 1
}"#;
        assert_eq!(
            read_the_checks(as_github_sends_it, &required),
            TheChecks::AllPassed,
            "the parser could not read the shape GitHub actually sends"
        );
    }

    /// **A pretty-printed failure is a failure, not a wait.**
    ///
    /// The same blindness in the direction that matters more: unreadable had
    /// previously meant *keep waiting*, so a required check that **failed**
    /// would also have been waited on until the deadline.
    #[test]
    fn a_pretty_printed_failure_is_not_waited_for() {
        let required = vec!["alo/gates-on-a-runner".to_owned()];
        let failed = "{\n  \"statuses\": [\n    {\n      \"state\": \"failure\",\n      \"context\": \"alo/gates-on-a-runner\"\n    }\n  ]\n}";
        assert!(matches!(
            read_the_checks(failed, &required),
            TheChecks::OneFailed(_)
        ));
    }

    /// **A reply that is not a status answer says so, and is never a wait.**
    ///
    /// Bad credentials, a renamed repository and an unknown commit all answer
    /// with a `message` and no `statuses`. Read as *has not reported yet* —
    /// which is what the parser did until 2026-10-06 — each one spins to the
    /// deadline and then blames CI.
    #[test]
    fn a_reply_that_is_not_a_status_is_told_apart_from_one_still_running() {
        let required = vec!["alo/gates-on-a-runner".to_owned()];
        for refusal in [
            r#"{"message":"Bad credentials","documentation_url":"https://docs.github.com"}"#,
            r#"{"message":"Not Found","status":"404"}"#,
        ] {
            let read = read_the_checks(refusal, &required);
            let why = match &read {
                TheChecks::Unreadable(why) => why.as_str(),
                _ => "",
            };
            assert!(
                why.contains("Bad credentials") || why.contains("Not Found"),
                "a refusal was not read as unreadable, or lost what GitHub said: {read:?}"
            );
        }
    }

    /// **An empty status list is still a status answer**, and still a wait.
    ///
    /// The guard above must not turn *CI has not started* into *I cannot read
    /// this*: the first is worth waiting for and the second never is.
    #[test]
    fn an_empty_status_list_is_a_wait_and_not_a_refusal() {
        let required = vec!["alo/gates-on-a-runner".to_owned()];
        assert!(matches!(
            read_the_checks(r#"{"state":"pending","statuses":[]}"#, &required),
            TheChecks::StillRunning(_)
        ));
    }

    /// **A failed required check is not something to wait for.**
    #[test]
    fn a_failed_required_check_is_told_apart_from_one_still_running() {
        let required = vec!["alo/gates-on-a-runner".to_owned()];
        let failed = r#"{"statuses":[{"state":"failure","context":"alo/gates-on-a-runner"}]}"#;
        let running = r#"{"statuses":[{"state":"pending","context":"alo/gates-on-a-runner"}]}"#;
        assert!(matches!(
            read_the_checks(failed, &required),
            TheChecks::OneFailed(_)
        ));
        assert!(matches!(
            read_the_checks(running, &required),
            TheChecks::StillRunning(_)
        ));
    }

    /// **A 200 carrying errors is a refusal, and this is the test for the fault that was
    /// nearly written twice.**
    ///
    /// `curl` exits 0 on a 200 whose body says `"errors"`, so a request that *succeeded*
    /// while the enqueue was *refused* reads as a landing. This lane had that exact bug in its
    /// own enqueue script on 2026-10-01, and the shell lane spent a night reading `exit 0`
    /// from a pipeline whose `tail` always succeeded. Same shape from two directions: **the
    /// signal was correct and the reader discarded it.**
    #[test]
    fn a_reply_carrying_errors_is_a_refusal_however_the_request_went() {
        let refused = r#"{"errors":[{"message":"Pull request is in unstable status"}]}"#;
        assert!(the_queue_refused(refused).is_some());

        let no_entry = r#"{"data":{"enqueuePullRequest":null}}"#;
        assert!(
            the_queue_refused(no_entry).is_some(),
            "a null entry under no error read as queued"
        );

        let queued = r#"{"data":{"enqueuePullRequest":{"mergeQueueEntry":{"position":1,"state":"QUEUED"}}}}"#;
        assert!(the_queue_refused(queued).is_none());
    }

    /// **The transient refusal is told apart from a real one.**
    ///
    /// `mergeability check has not yet completed` resolves in seconds; every other refusal
    /// will not. A loop that retried a real refusal would spin, and one that gave up on the
    /// transient one would lose a turn to a race it had already won.
    #[test]
    fn a_mergeability_check_that_has_not_finished_is_not_a_real_refusal() {
        assert!(is_worth_asking_again(
            "UNPROCESSABLE: mergeability check has not yet completed"
        ));
        assert!(!is_worth_asking_again("Pull request is in unstable status"));
        assert!(!is_worth_asking_again(
            "Required status check \"alo/gates-on-a-runner\" is expected."
        ));
    }
}
