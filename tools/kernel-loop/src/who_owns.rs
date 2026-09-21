//! Which plan owns a crate, and whether that plan still holds it.
//!
//! [`crate::inside_the_plan`] reads the publishing plan's own header: what it
//! says it never edits. That list cannot know about a crate the header forgot.
//! On 2026-09-17 a session task edited `alo-turn` and `alo-record` while
//! another machine's lane was working in them, and passed, because the session
//! plan's header did not name them.
//!
//! So this reads **every** plan's `**Crates this plan owns:**` paragraph. A task
//! that edits a crate another plan owns is refused while that plan still has an
//! unfinished task. When the owning plan's last task is done, its crates are
//! released, and nobody has to edit anything for that to happen. The prose lane
//! table in `a-new-machine-becomes-a-lane.md` is deliberately not read: it is
//! written for people, and a parser of it would break on the next rewording.
//!
//! **Two plans claiming one crate is an error in itself**, and nothing detected it
//! before this. A task that edits such a crate while more than one claimant is
//! unfinished is refused with every claimant named. Every other double claim is
//! said in the log on each publication, without refusing, because refusing work
//! over a claim it does not touch would stop every lane on every machine for a
//! sentence none of them wrote.
//!
//! # Reading the paragraph
//!
//! The owned crates are the backticked `alo-*` names and `dir/` paths in the
//! sentence the label begins, with three things left out:
//! - text in parentheses, which describes a crate and often names others;
//! - a name followed by `'s` (`` `alo-saying`'s list entries``), which is a part of
//!   somebody else's crate rather than a claim on it;
//! - everything from *reads and never edits* on.

use crate::plan;

/// The label a plan's header introduces its own crates with, in any of its
/// spellings (`**Crates this plan owns:**`, `**Crates this plan owns, all new:**`).
const THE_LABEL: &str = "**Crates this plan owns";

/// Where a plan's tasks begin, and so where its header ends.
const THE_TASKS: &str = "\n## Tasks";

/// One plan, as far as ownership goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    /// The plan's path, as a person would name it.
    pub plan: String,

    /// The crates and directories it says it owns.
    pub owns: Vec<String>,

    /// Whether any of its tasks is not done.
    pub unfinished: bool,
}

impl Claim {
    /// Whether this file is inside a crate or directory this plan claims.
    #[must_use]
    pub fn holds(&self, file: &str) -> bool {
        !is_in(file, &self.owns).is_empty()
    }
}

/// What one plan claims, read from its text.
#[must_use]
pub fn claimed_by(plan_named: &str, written: &str) -> Claim {
    let tasks = plan::tasks_in(written);
    Claim {
        plan: plan_named.to_owned(),
        owns: owned(written),
        unfinished: tasks.iter().any(|task| !task.done),
    }
}

/// The crates and directories a plan's header says it owns.
#[must_use]
pub fn owned(written: &str) -> Vec<String> {
    let header = written.split(THE_TASKS).next().unwrap_or(written);
    let Some(paragraph) = header
        .split("\n\n")
        .find(|paragraph| paragraph.contains(THE_LABEL))
    else {
        return Vec::new();
    };
    let paragraph = paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
    let after_label = paragraph
        .split_once(THE_LABEL)
        .map_or("", |(_, rest)| rest)
        .split_once("**")
        .map_or("", |(_, rest)| rest);
    let before_never = after_label
        .split("never edits")
        .next()
        .unwrap_or_default()
        .split("**It reads")
        .next()
        .unwrap_or_default();
    let unbracketed = without_parentheses(before_never);
    let sentence = unbracketed.split(". ").next().unwrap_or_default();

    let mut found: Vec<String> = Vec::new();
    let mut pieces = sentence.split('`');
    // Every other piece is inside backticks; the one after it says whether a
    // possessive follows.
    pieces.next();
    while let (Some(quoted), after) = (pieces.next(), pieces.next()) {
        if after.is_some_and(|after| after.starts_with("'s")) {
            continue;
        }
        let name = quoted.trim();
        let name = name.strip_prefix("crates/").unwrap_or(name);
        let a_crate = name.starts_with("alo-")
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        let a_directory = name.ends_with('/') && !name.contains(' ');
        if (a_crate || a_directory) && !found.iter().any(|seen| seen == name) {
            found.push(name.to_owned());
        }
    }
    found
}

/// Text with every parenthesised part removed, nested ones included.
fn without_parentheses(text: &str) -> String {
    let mut depth = 0_u32;
    let mut kept = String::with_capacity(text.len());
    for letter in text.chars() {
        match letter {
            '(' => depth = depth.saturating_add(1),
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => kept.push(letter),
            _ => {}
        }
    }
    kept
}

/// Every crate more than one plan claims, with the plans claiming it.
#[must_use]
pub fn claimed_twice(claims: &[Claim]) -> Vec<(String, Vec<String>)> {
    let mut twice: Vec<(String, Vec<String>)> = Vec::new();
    for claim in claims {
        for name in &claim.owns {
            if twice.iter().any(|(seen, _)| seen == name) {
                continue;
            }
            let claimants: Vec<String> = claims
                .iter()
                .filter(|other| other.owns.contains(name))
                .map(|other| other.plan.clone())
                .collect();
            if claimants.len() > 1 {
                twice.push((name.clone(), claimants));
            }
        }
    }
    twice
}

/// The crate or directory a changed file is in, when it is in one a plan could
/// own.
fn is_in<'n>(file: &str, names: &'n [String]) -> Vec<&'n String> {
    let file = file.replace('\\', "/");
    names
        .iter()
        .filter(|name| {
            if name.ends_with('/') {
                file.starts_with(name.as_str())
            } else {
                file.starts_with(&format!("crates/{name}/"))
            }
        })
        .collect()
}

/// The refusal for a task whose files are in a crate another unfinished plan
/// owns, or that more than one unfinished plan claims; `None` when there is
/// nothing to refuse.
#[must_use]
pub fn refusal(files: &[String], publishing: &str, claims: &[Claim]) -> Option<String> {
    let mut reasons: Vec<String> = Vec::new();
    for file in files {
        for claim in claims.iter().filter(|claim| claim.unfinished) {
            for name in is_in(file, &claim.owns) {
                let holders: Vec<&str> = claims
                    .iter()
                    .filter(|other| other.unfinished && other.owns.contains(name))
                    .map(|other| other.plan.as_str())
                    .collect();
                let reason = if holders.len() > 1 {
                    format!(
                        "`{name}` is claimed by more than one unfinished plan ({}), which is an \
                         error in the plans themselves",
                        holders.join(", ")
                    )
                } else if claim.plan == publishing {
                    continue;
                } else {
                    format!(
                        "`{name}` belongs to {}, which still has an unfinished task",
                        claim.plan
                    )
                };
                if !reasons.contains(&reason) {
                    reasons.push(reason);
                }
            }
        }
    }
    if reasons.is_empty() {
        return None;
    }
    Some(format!(
        "this task changes a crate that is not {publishing}'s to change right now: {}. Nothing \
         was gated or published. A plan's crates are released when its last task is done; until \
         then, hand over only what is inside this plan and mark the task blocked on that plan, \
         saying what change is needed",
        reasons.join("; ")
    ))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A plan in the shape the real ones are written in.
    fn a_plan(owns: &str, statuses: &[&str]) -> String {
        let mut written = format!(
            "# A plan\n\n**Crates this plan owns, all new:** {owns} **It reads and never\nedits** \
             `alo-record`.\n\n## Tasks\n"
        );
        for (number, status) in statuses.iter().enumerate() {
            written.push_str(&format!(
                "\n### {}. Something\n\n**Status:** {status} **Depends on:** nothing.\n",
                number + 1
            ));
        }
        written
    }

    fn files(named: &[&str]) -> Vec<String> {
        named.iter().map(|&name| name.to_owned()).collect()
    }

    /// **What a header owns is read, and what it only describes is not**: the
    /// names in parentheses, a possessive, and the never-edits list are left out.
    #[test]
    fn the_owned_crates_are_read_and_the_described_ones_are_not() {
        let written = a_plan(
            "`crates/alo-broker` (the third component after `alo-boundaryd`), \
             `crates/alo-encrypting`, `alo-saying`'s list entries and `image/`.",
            &["ready."],
        );
        assert_eq!(owned(&written), ["alo-broker", "alo-encrypting", "image/"]);
    }

    /// **A task editing a crate another plan owns is refused while that plan has
    /// work left, and allowed once it has none.** Finishing is what releases.
    #[test]
    fn a_finished_plan_releases_its_crates() {
        let files = files(&["crates/alo-nearby/src/lib.rs"]);
        let working = claimed_by(
            "network",
            &a_plan("`crates/alo-nearby`.", &["**Done, 1.**", "ready."]),
        );
        let refused = refusal(&files, "session", &[working]).unwrap();
        assert!(
            refused.contains("`alo-nearby` belongs to network"),
            "{refused}"
        );

        let finished = claimed_by(
            "network",
            &a_plan("`crates/alo-nearby`.", &["**Done, 1.**", "**Done, 2.**"]),
        );
        assert_eq!(refusal(&files, "session", &[finished]), None);
    }

    /// **A plan's own crates are its own**, and a crate nobody claims is not
    /// refused here.
    #[test]
    fn a_plans_own_crates_and_unclaimed_ones_pass() {
        let ours = claimed_by("session", &a_plan("`crates/alo-sleeping`.", &["ready."]));
        let handed = files(&[
            "crates/alo-sleeping/src/lib.rs",
            "crates/alo-turn/src/turning.rs",
        ]);
        assert_eq!(refusal(&handed, "session", &[ours]), None);
    }

    /// **Two unfinished plans claiming one crate refuses a task in it, naming
    /// both**, whichever of them is publishing, and the double claim is found
    /// whether or not anything touches it.
    #[test]
    fn two_plans_claiming_one_crate_is_an_error_in_itself() {
        let one = claimed_by(
            "models",
            &a_plan("`alo-models` and `alo-choosing`.", &["ready."]),
        );
        let two = claimed_by("settings", &a_plan("`crates/alo-choosing`.", &["ready."]));
        let claims = [one, two];

        let refused = refusal(
            &files(&["crates/alo-choosing/src/lib.rs"]),
            "models",
            &claims,
        )
        .unwrap();
        assert!(
            refused.contains("more than one unfinished plan"),
            "{refused}"
        );
        assert!(
            refused.contains("models") && refused.contains("settings"),
            "{refused}"
        );

        assert_eq!(
            claimed_twice(&claims),
            [(
                "alo-choosing".to_owned(),
                vec!["models".to_owned(), "settings".to_owned()]
            )]
        );
    }

    /// **The real plans are read the way a person reads them.** The broker plan
    /// owns its three crates and not the ones its paragraph mentions in
    /// passing.
    ///
    /// It owned two until 2026-09-20, when its task 6 added `alo-enrolling` —
    /// the sentences a person meets while their disk is being encrypted, which
    /// cannot live in `alo-encrypting` because that crate holds a recovery key
    /// and depends on nothing. A plan that gains a crate changes this line, and
    /// that is the point of the line.
    #[test]
    fn the_real_broker_plan_owns_what_its_header_says() {
        let written = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/autonomy/v0-5-the-broker-and-the-disk-plan.md"),
        )
        .unwrap();
        assert_eq!(
            owned(&written),
            ["alo-broker", "alo-encrypting", "alo-enrolling"]
        );
    }
}
