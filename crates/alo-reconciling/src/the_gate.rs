//! The exit gate held to itself, to the definition, and to the plans.
//!
//! `crates/alo-reconciling`'s older half answers one question — is every promise
//! in `docs/features.md` reconciled by an entry in a ledger. This half answers
//! the questions a reading of `ROADMAP.md` on 2026-09-26 answered by hand, over
//! six sections and a day, and got wrong in four places. Every check below was
//! specified by something that pass found, and the pass is what says why each is
//! worth the code.
//!
//! | check | what specified it |
//! |---|---|
//! | [`orphaned_boxes`] | the v0.5 cut moved nine promises to v1 by their first line and left four boxes behind, two of them ticked, under promises they were not about |
//! | [`promises_with_no_box`] | three `[v0.5]` promises had no box in the gate at all, one of them with finished code and nothing to tick it in |
//! | [`denials_by_task_number`] | the gate said pairing waited on task 12, *open*, twelve days after that task was done |
//! | [`statuses_their_own_section_contradicts`] | two plans, fourteen tasks, reading `ready` above their own published reports — and 74 across six plans reading `ready` above their own `**Done,` marker |
//! | [`tiers_that_disagree`] | the camera sat at `[v0.5]` in the definition, inside *Devices* in the roadmap's v1, and *carried to v2* in its plan |
//! | [`refusals_a_report_contradicts`] | four refusals written from searches narrower than the claim, each contradicted by a report that had been published for days |
//! | [`counts_that_drifted`] | *five designed hues* stayed ticked as working code when `Accent::ALL` became four |
//!
//! # None of these reads a plan's status to decide whether work is done
//!
//! That is the one thing the pass established twice and then forgot twice. A
//! plan's status line is a claim somebody has to remember to update, and
//! fourteen of them were behind their own evidence on the day this was written.
//! So a status is read **only** to find a disagreement with something else — a
//! report that exists, a box that denies it — and never as the answer.

use std::collections::{BTreeMap, BTreeSet};

use crate::finding::Finding;
use crate::promise::promises_at;
use crate::tier::Tier;

/// A box in the gate: a promise's own, or one of the two halves under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Box_ {
    /// The line, trimmed of its marker.
    words: String,
    /// Whether it is ticked.
    ticked: bool,
    /// Whether it is a half — indented under a promise — rather than a promise.
    half: bool,
    /// Which line of the document it is on, so a finding can name it.
    line: usize,
}

impl Box_ {
    /// The box's words.
    #[must_use]
    pub fn words(&self) -> &str {
        &self.words
    }

    /// Whether it is ticked.
    #[must_use]
    pub const fn ticked(&self) -> bool {
        self.ticked
    }

    /// Whether it is one of a promise's two halves.
    #[must_use]
    pub const fn half(&self) -> bool {
        self.half
    }

    /// Which line it is on.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }
}

/// One promise in the gate, with the halves written under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Promised {
    /// The promise's own box.
    pub promise: Box_,
    /// The halves under it, in the order they are written.
    pub halves: Vec<Box_>,
    /// Every line of its body and its halves' bodies, for the checks that read
    /// what a box says rather than whether it is ticked.
    pub body: String,
}

/// The heading a section of the gate is under, and the promises in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// The `## ` heading.
    pub heading: String,
    /// Its promises, in order.
    pub promises: Vec<Promised>,
}

/// Which words mark a half as *the code* and which as *on the machine*.
///
/// Read as a prefix of the box's own words rather than by position, because the
/// order of the two halves is a house style and not a rule, and a check that
/// depended on the order would break on the first promise somebody wrote the
/// other way round.
const THE_CODE: &str = "**The code.**";
/// As above, for the machine half.
const ON_THE_MACHINE: &str = "**On the machine.**";
/// The same two, as the handful of lines in the file write them without bold.
const THE_CODE_PLAIN: &str = "The code.";
/// As above.
const ON_THE_MACHINE_PLAIN: &str = "On the machine.";

/// Every section of a roadmap, with its promises and their halves.
///
/// A line beginning `- [` opens a promise; a line beginning with two spaces and
/// `- [` is one of its halves; anything else is body, and belongs to whichever
/// box was opened last. That is exactly how the document reads to a person, and
/// it is why a promise moved by its first line alone leaves its halves attached
/// to the promise above — which is [`orphaned_boxes`]' whole subject.
#[must_use]
pub fn sections_in(roadmap: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    for (at, line) in roadmap.lines().enumerate() {
        if let Some(heading) = line.strip_prefix("## ") {
            sections.push(Section {
                heading: format!("## {heading}"),
                promises: Vec::new(),
            });
            continue;
        }
        let Some(section) = sections.last_mut() else {
            continue;
        };
        if let Some(rest) = line.strip_prefix("- [") {
            section.promises.push(Promised {
                promise: a_box(rest, false, at + 1),
                halves: Vec::new(),
                body: String::new(),
            });
        } else if let Some(rest) = line.strip_prefix("  - [") {
            if let Some(promised) = section.promises.last_mut() {
                promised.halves.push(a_box(rest, true, at + 1));
            }
        } else if let Some(promised) = section.promises.last_mut() {
            promised.body.push_str(line.trim());
            promised.body.push('\n');
        }
    }
    sections
}

/// One box, from the text after its `- [`.
fn a_box(rest: &str, half: bool, line: usize) -> Box_ {
    let ticked = rest.starts_with('x');
    let words = rest
        .get(1..)
        .unwrap_or_default()
        .trim_start_matches(']')
        .trim()
        .to_owned();
    Box_ {
        words,
        ticked,
        half,
        line,
    }
}

/// **A half under the wrong promise, or a promise carrying two of one half.**
///
/// The v0.5 cut moved nine promises to v1 by deleting nine first lines and
/// writing nine new ones, and two of those promises had halves. The halves
/// stayed, and markdown does not mind: they attached themselves to the promise
/// above, which then carried two *The code* boxes and two *On the machine*
/// boxes. One of the strays was ticked, so the gate reported a promise as having
/// finished code that has never had a code box of its own.
///
/// A promise may carry at most one of each half. Nothing else here is a rule
/// about how many halves a promise should have — a promise with none is an
/// unsplit promise and is [`promises_with_no_box`]' business, not this one's.
#[must_use]
pub fn orphaned_boxes(roadmap: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for section in sections_in(roadmap) {
        for promised in &section.promises {
            let mut code = 0_usize;
            let mut machine = 0_usize;
            let mut neither = 0_usize;
            for half in &promised.halves {
                if half.words.starts_with(THE_CODE) || half.words.starts_with(THE_CODE_PLAIN) {
                    code += 1;
                } else if half.words.starts_with(ON_THE_MACHINE)
                    || half.words.starts_with(ON_THE_MACHINE_PLAIN)
                {
                    machine += 1;
                } else {
                    neither += 1;
                }
            }
            if code > 1 || machine > 1 {
                findings.push(Finding::AHalfUnderTheWrongPromise {
                    promise: promised.promise.words.clone(),
                    line: promised.promise.line,
                    code,
                    machine,
                });
            }
            if neither > 0 {
                findings.push(Finding::AHalfThatIsNeither {
                    promise: promised.promise.words.clone(),
                    line: promised.promise.line,
                    how_many: neither,
                });
            }
        }
    }
    findings
}

/// **A promise the definition makes with no box in the gate.**
///
/// Three `[v0.5]` promises had none on 2026-09-26 — the dock's size, the dock
/// per display, and night light, the last of which had a finished crate, two
/// reports and nothing to tick it in. `ROADMAP.md`'s own preamble says the rule
/// runs both ways and that six v0.01 promises had been found this way, one at a
/// time, with the reading twice believing it had found the last.
///
/// A promise is answered by any box in its tier's section whose words carry a
/// long enough run of the promise's own. Matching is on a phrase rather than the
/// whole line because the gate groups: *Settings, as one place* is one box over
/// nine areas, and a box is allowed to say less than the definition does.
#[must_use]
pub fn promises_with_no_box(features: &str, roadmap: &str, tier: Tier) -> Vec<Finding> {
    let Some(heading) = tier.heading() else {
        return Vec::new();
    };
    let section = sections_in(roadmap)
        .into_iter()
        .find(|section| section.heading.starts_with(heading));
    let Some(section) = section else {
        return vec![Finding::ATierWithNoSection {
            tier: tier.named(),
            heading,
        }];
    };

    let everything: String = section
        .promises
        .iter()
        .map(|promised| format!("{} {}", promised.promise.words, promised.body))
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();

    promises_at(features, tier)
        .into_iter()
        .filter(|promise| !is_answered(promise.words(), &everything))
        .map(|promise| Finding::APromiseWithNoBox {
            promise: promise.words().to_owned(),
            tier: tier.named(),
        })
        .collect()
}

/// The shortest run of a promise's own distinctive words that a box must carry
/// for the promise to count as answered.
///
/// Four. Both halves of that number were measured against these documents rather
/// than chosen:
///
/// - fewer lets a box answer a promise it only shares a subject with —
///   *Printing* would answer *Printing. Unglamorous, and it decides public-sector
///   deals*;
/// - more makes a grouped box fail to answer promises it plainly covers, and
///   the v0.5 gate groups on purpose: one box over nine settings areas.
const ENOUGH_WORDS: usize = 4;

/// The shortest word this counts as the promise's own.
///
/// Five characters. **This is the number that decides whether the check works**,
/// and the first version got it wrong in a way worth writing down: it counted
/// every word over two characters and then accepted a promise as answered when
/// half of them appeared *anywhere* in the section. Half the words of *Audio in
/// and out, with device switching that works mid-call* are `and`, `out`, `with`,
/// `that` — which appear in any prose at all — so the gate was reported as
/// answering a promise it has no box for, and two real findings were missed.
///
/// A check biased towards silence is the reading it replaces.
const A_WORD_OF_ITS_OWN: usize = 5;

/// Whether the gate says enough of a promise for it to be answered.
///
/// Matched on a run of consecutive distinctive words rather than on a set,
/// because a set of common words is satisfied by prose and a run is not.
fn is_answered(promise: &str, gate: &str) -> bool {
    let words = its_own_words(promise);
    let said = its_own_words(gate).join(" ");
    if words.len() < ENOUGH_WORDS {
        return !words.is_empty() && words.iter().all(|word| said.contains(word.as_str()));
    }
    words
        .windows(ENOUGH_WORDS)
        .any(|run| said.contains(&run.join(" ")))
}

/// A passage's own words: lowercased, stripped of punctuation, and only the ones
/// long enough to mean something on their own.
fn its_own_words(passage: &str) -> Vec<String> {
    passage
        .to_lowercase()
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|what: char| !what.is_alphanumeric())
                .to_owned()
        })
        .filter(|word| word.chars().count() >= A_WORD_OF_ITS_OWN)
        .collect()
}

/// **A box that says a task is open, about a task that is done.**
///
/// *Machines find each other* read *Most of it, and not whole*, and named what
/// was missing: the person's door to propose, confirm and revoke, and a pairing
/// that outlives a restart — **task 12 of `v0-5-the-local-network-plan.md`,
/// open**. That task had been done for twelve days. The clause was precise
/// enough to sound authoritative, which is what made it worse than vagueness.
///
/// A box that names a plan and a task number, in a sentence that also says the
/// work is not there, is checked against that plan's own status for that task.
/// This is the one check that reads a status — and it reads it only to
/// contradict a denial, never to believe a claim.
#[must_use]
pub fn denials_by_task_number(roadmap: &str, plans: &BTreeMap<String, String>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for section in sections_in(roadmap) {
        for promised in &section.promises {
            let said = format!(
                "{} {}",
                promised.promise.words,
                what_a_box_still_claims(&promised.body)
            );
            if !denies_something(&said) {
                continue;
            }
            for (plan, task) in tasks_named_in(&said) {
                let Some(text) = plans.get(&plan) else {
                    continue;
                };
                if is_done(text, task) {
                    findings.push(Finding::ADenialOfFinishedWork {
                        promise: promised.promise.words.clone(),
                        line: promised.promise.line,
                        plan,
                        task,
                    });
                }
            }
        }
    }
    findings
}

/// How a box records a claim it used to make and has withdrawn.
///
/// `ROADMAP.md`'s *Settings, as one place* carries this, and it is the most
/// valuable sentence in that box:
///
/// ```text
/// *This box said **the one place does not exist** on 2026-09-26, which was
/// **wrong**: it was drawn on 2026-09-16*
/// ```
///
/// A check reading the box's words found *does not exist* there and reported the
/// box as denying work its own plan says is done. It is the opposite: the box
/// withdrew that denial and kept the record of having made it, which is what the
/// reconciliation log exists for. **Deleting the record to satisfy a check would
/// be the check making the documents worse**, so the convention is named here
/// instead: from this marker to the end of a box's body is the record of a claim
/// no longer made, and nothing reads it as a claim.
const A_WITHDRAWN_CLAIM: &str = "*This box said";

/// A box's body with any withdrawn claim cut off it.
fn what_a_box_still_claims(body: &str) -> &str {
    match body.find(A_WITHDRAWN_CLAIM) {
        Some(at) => body.get(..at).unwrap_or(body),
        None => body,
    }
}

/// The words a box uses when it says something is not there.
const DENIALS: [&str; 9] = [
    "not whole",
    "not yet",
    "not built",
    "is open",
    ", open",
    "unstarted",
    "does not exist",
    "no such",
    "not assessed",
];

/// Whether a box denies that something is there.
fn denies_something(said: &str) -> bool {
    let said = said.to_lowercase();
    DENIALS.iter().any(|denial| said.contains(denial))
}

/// Every plan and task number a passage names, as `("v0-5-….md", 12)`.
fn tasks_named_in(said: &str) -> Vec<(String, u32)> {
    let mut found = Vec::new();
    for (at, _) in said.match_indices("task ") {
        let rest = said.get(at + "task ".len()..).unwrap_or_default();
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        let Ok(task) = digits.parse::<u32>() else {
            continue;
        };
        // The plan is whichever `.md` the same passage names after it, which is
        // how the gate writes it: "task 12 of `v0-5-the-local-network-plan.md`".
        let after = rest.get(digits.len()..).unwrap_or_default();
        if let Some(plan) = a_plan_named_in(after) {
            found.push((plan, task));
        }
    }
    found
}

/// The first plan filename a passage names, if it names one within reach.
fn a_plan_named_in(after: &str) -> Option<String> {
    const HOW_FAR: usize = 160;
    let near = after.get(..after.len().min(HOW_FAR)).unwrap_or(after);
    let at = near.find("v0-5-")?;
    let rest = near.get(at..)?;
    let end = rest.find(".md")? + ".md".len();
    Some(rest.get(..end)?.to_owned())
}

/// Whether a plan says a task is done.
///
/// The status line under the task's heading, and nothing else. A task whose
/// status says anything but done — `ready`, `blocked`, `scheduled`, `in
/// progress` — is not done, which is the reading `docs/autonomy/SHARED_MAIN.md`
/// gives and the one the supervisor gives.
#[must_use]
pub fn is_done(plan: &str, task: u32) -> bool {
    status_of(plan, task).is_some_and(|status| status.to_lowercase().contains("done"))
}

/// The status line of a task in a plan, if the plan has that task.
#[must_use]
pub fn status_of(plan: &str, task: u32) -> Option<String> {
    let heading = format!("### {task}. ");
    let mut lines = plan.lines().skip_while(|line| !line.starts_with(&heading));
    lines.next()?;
    lines
        .take_while(|line| !line.starts_with("### "))
        .find(|line| line.starts_with("**Status:**"))
        .map(str::to_owned)
}

/// The marker a plan puts in a task's body when the task is finished.
///
/// `docs/autonomy/SHARED_MAIN.md` names this one, not the status word, as the
/// one to trust when surveying plans for free work — a rule written after a
/// reading of the status words alone "nearly handed a lane thirteen completed
/// tasks".
const THE_DONE_MARKER: &str = "**Done, ";

/// **A task whose status word its own section contradicts.**
///
/// A plan states whether a task is finished twice, in two places, and nothing
/// made them agree:
///
/// ```text
/// ### 1. What an update is, and what it may never do
///
/// **Status:** ready. **Depends on:** nothing.
///
/// **Done, 2026-09-14.** `crates/alo-keeping-up`: `Digest` is a whole, lowercase
/// ```
///
/// The second line says the work is free to take; the fourth says it was
/// finished thirteen days ago and then describes the code. **74 tasks across six
/// plans** read like that on 2026-09-27 — 32 of them in `v0-01-delivery-plan.md`,
/// a release that shipped.
///
/// # Why the marker and not a published report
///
/// The finding this check was specified by was worded *a plan reading `ready`
/// over its own published report*, and that is what was tried first. It is a
/// worse signal in two ways that the real documents show immediately:
///
/// - `updates/what-closes-v0-0-5-and-in-what-order.md` is named by task sections
///   in **six** plans. An index a task cross-references is not that task's
///   evidence, and counting it reported five tasks that are correctly unfinished.
/// - A task held on purpose can have a report *about the hold* — the installer
///   plan's stopped download does — and its status is right to say `blocked`.
///
/// The `**Done,` marker has neither problem: it is in the task's own body, it is
/// written by whoever finished the task, and a held task does not have one. Every
/// one of the fourteen the report-based reading found is also found here, and
/// sixty more.
///
/// # And it still does not let a status decide whether work is done
///
/// It reads the status **only** to find the disagreement, which is the rule
/// stated at the top of this module. The marker is the claim about the work; the
/// status word is the line to correct.
#[must_use]
pub fn statuses_their_own_section_contradicts(plan: &str, named: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (task, section) in sections_of(plan) {
        let Some(status) = section.lines().find(|line| line.starts_with("**Status:**")) else {
            continue;
        };
        if status.to_lowercase().contains("done") {
            continue;
        }
        let Some(marker) = section
            .lines()
            .find(|line| line.starts_with(THE_DONE_MARKER))
        else {
            continue;
        };
        findings.push(Finding::AStatusItsOwnSectionContradicts {
            plan: named.to_owned(),
            task,
            status: shortened(status),
            marker: shortened(marker),
        });
    }
    findings
}

/// The first few words of a line, so a finding names it without quoting a
/// paragraph.
fn shortened(line: &str) -> String {
    const ENOUGH: usize = 60;
    let mut said: String = line.chars().take(ENOUGH).collect();
    if line.chars().count() > ENOUGH {
        said.push('…');
    }
    said
}

/// Each task number in a plan, with the text of its section.
fn sections_of(plan: &str) -> Vec<(u32, String)> {
    let mut found: Vec<(u32, String)> = Vec::new();
    for line in plan.lines() {
        if let Some(rest) = line.strip_prefix("### ") {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(task) = digits.parse::<u32>() {
                found.push((task, String::new()));
                continue;
            }
        }
        if let Some((_, text)) = found.last_mut() {
            text.push_str(line);
            text.push('\n');
        }
    }
    found
}

/// Every `updates/….md` a passage names, by filename alone.
fn reports_named_in(said: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for (at, _) in said.match_indices("updates/") {
        let rest = said.get(at + "updates/".len()..).unwrap_or_default();
        let Some(end) = rest.find(".md") else {
            continue;
        };
        if let Some(name) = rest.get(..end + ".md".len())
            && !name.contains(' ')
            && !name.contains('(')
        {
            found.insert(name.to_owned());
        }
    }
    found
}

/// **A promise the definition records as withdrawn, still open in the roadmap.**
///
/// `docs/features.md` records a withdrawal as a paragraph beginning
/// `**Withdrawn by [<record>](…):**`, naming in italics the promises it takes
/// away. This reads those paragraphs and reports any whose promise still has an
/// open `- [ ]` box in `ROADMAP.md`.
///
/// # Why this one can gate where its neighbours cannot
///
/// [`promises_with_no_box`] and [`tiers_that_disagree`] cannot gate, and the
/// test that runs them says at length why: whether a box *covers* a promise is a
/// judgement, 31 boxes group 92 promises, and every threshold measured either
/// missed real cases or flagged most of the document.
///
/// **This asks a narrower question with a definite answer.** A withdrawal names
/// the promise in the words the roadmap uses for it, because the roadmap's box
/// was written from the same sentence — so the match is between two copies of
/// one phrase, not between a promise and a paragraph that might be about it.
/// And the right number of findings is **zero**: a withdrawn promise left open
/// is never correct, where a promise with no box may simply be grouped under
/// one.
///
/// # What it cannot see
///
/// **A withdrawal the definition never recorded.** If a record withdraws
/// something and `docs/features.md` is not updated, nothing here notices — the
/// definition is this check's only source for what was withdrawn, and a
/// withdrawal that never reached it is invisible to every instrument in this
/// crate.
#[must_use]
pub fn withdrawals_left_open(features: &str, roadmap: &str) -> Vec<Finding> {
    let open: Vec<String> = roadmap
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("- [ ] "))
        .map(str::to_owned)
        .collect();

    let mut findings = Vec::new();
    for (record, withdrawn) in withdrawals_in(features) {
        for promise in withdrawn {
            if open
                .iter()
                .any(|still_open| is_answered(&promise, still_open))
            {
                findings.push(Finding::AWithdrawalLeftOpen {
                    promise,
                    record: record.clone(),
                });
            }
        }
    }
    findings
}

/// Every withdrawal the definition records: the record that made it, and the
/// promises it names in italics.
///
/// Italics rather than the whole paragraph, because a withdrawal also says
/// *why*, and the why is prose that would match half the roadmap. Only spans
/// long enough to be a promise are taken: a withdrawal italicises qualifiers
/// too — *in both orientations* — and a short span has no business matching a
/// box.
fn withdrawals_in(features: &str) -> Vec<(String, Vec<String>)> {
    let mut found = Vec::new();
    for line in features.lines() {
        let Some(rest) = line.trim_start().strip_prefix("**Withdrawn by ") else {
            continue;
        };
        let record = rest.split_once(']').map_or_else(
            || rest.to_owned(),
            |(named, _)| named.trim_start_matches('[').to_owned(),
        );
        let promises: Vec<String> = italics_in(rest)
            .into_iter()
            .filter(|span| its_own_words(span).len() >= ENOUGH_WORDS)
            .collect();
        if !promises.is_empty() {
            found.push((record, promises));
        }
    }
    found
}

/// Every `*…*` span in a line, without the asterisks.
///
/// `**bold**` is stepped over: the marker is doubled, and reading the inside of
/// a bold span would hand this check the words *Withdrawn by* on every
/// paragraph it is looking at.
fn italics_in(line: &str) -> Vec<String> {
    let characters: Vec<char> = line.chars().collect();
    let mut spans = Vec::new();
    let mut at = 0;
    while at < characters.len() {
        if characters.get(at) != Some(&'*') {
            at = at.saturating_add(1);
            continue;
        }
        if characters.get(at.saturating_add(1)) == Some(&'*') {
            at = at.saturating_add(2);
            continue;
        }
        let opened = at.saturating_add(1);
        let mut closes = opened;
        while closes < characters.len() && characters.get(closes) != Some(&'*') {
            closes = closes.saturating_add(1);
        }
        if closes >= characters.len() {
            break;
        }
        let span: String = characters
            .get(opened..closes)
            .map(|inside| inside.iter().collect())
            .unwrap_or_default();
        if !span.is_empty() {
            spans.push(span);
        }
        at = closes.saturating_add(1);
    }
    spans
}

/// **A promise at one tier in the definition and another in the roadmap.**
///
/// *Camera and microphone* was `[v0.5]` in `docs/features.md`, inside *Devices*
/// in `ROADMAP.md`'s **v1** section, and *carried to v2* in its own plan — three
/// documents, three answers, after a change that moved it in the plan alone and
/// touched neither of the others.
///
/// The definition is the scope gate, so it is the one that decides. A promise it
/// makes at one tier, answered by a box in a different tier's section, is a
/// finding against the roadmap.
#[must_use]
pub fn tiers_that_disagree(features: &str, roadmap: &str) -> Vec<Finding> {
    let sections = sections_in(roadmap);
    let mut findings = Vec::new();
    for tier in Tier::EVERY {
        let Some(heading) = tier.heading() else {
            continue;
        };
        for promise in promises_at(features, tier) {
            for section in &sections {
                if section.heading.starts_with(heading) {
                    continue;
                }
                let Some(other) = Tier::EVERY.iter().find(|one| {
                    one.heading()
                        .is_some_and(|h| section.heading.starts_with(h))
                }) else {
                    continue;
                };
                let said: String = section
                    .promises
                    .iter()
                    .map(|promised| format!("{} {}", promised.promise.words, promised.body))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .to_lowercase();
                if is_answered(promise.words(), &said) {
                    findings.push(Finding::ATierThatDisagrees {
                        promise: promise.words().to_owned(),
                        definition: tier.named(),
                        roadmap: other.named(),
                    });
                }
            }
        }
    }
    findings
}

/// **A box saying something does not exist, in a repository that holds a report
/// about it.**
///
/// Four refusals written on 2026-09-26 were wrong, and every one was a negative
/// claim from a search narrower than the claim: *no portal backend exists* from
/// grepping one spelling of an interface, *no touch gesture is read* from
/// reading one file, *not assessed* without searching for the standard's number,
/// *Settings is not drawn* without opening the plan that drew it. All four were
/// contradicted by reports that had been published for days, and the index that
/// answered them was read an hour later for a different purpose.
///
/// So a box that denies something, and names no report, is checked against the
/// reports whose own filenames carry the promise's words. It is deliberately a
/// weak signal reported loudly: it cannot know whether the report is about the
/// same thing, and it does not have to — it only has to make somebody look
/// before writing *nothing exists*.
#[must_use]
pub fn refusals_a_report_contradicts(roadmap: &str, reports: &BTreeSet<String>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for section in sections_in(roadmap) {
        for promised in &section.promises {
            // Per promise rather than per half. A half is one line — `**The
            // code.**` — and everything it says is in the body below it, which
            // `sections_in` keeps against the promise; asking the half alone
            // caught only the handful of halves written on a single line, which is
            // the shape this function had until 2026-09-27 and is why it reported
            // three refusals out of eight.
            let said = format!(
                "{} {} {}",
                promised.promise.words,
                promised
                    .halves
                    .iter()
                    .map(Box_::words)
                    .collect::<Vec<_>>()
                    .join(" "),
                what_a_box_still_claims(&promised.body)
            );
            if !denies_something(&said) || !reports_named_in(&said).is_empty() {
                continue;
            }
            for report in reports {
                if looks_like(report, promised.promise.words()) {
                    findings.push(Finding::ARefusalAReportContradicts {
                        promise: promised.promise.words.clone(),
                        line: promised.promise.line,
                        report: report.clone(),
                    });
                }
            }
        }
    }
    findings
}

/// How many of a promise's words a report's filename must carry before it is
/// worth showing somebody.
const ENOUGH_TO_LOOK: usize = 4;

/// Whether a report's filename reads like it is about a promise.
fn looks_like(report: &str, promise: &str) -> bool {
    let slug = report.trim_end_matches(".md");
    let words: BTreeSet<String> = promise
        .to_lowercase()
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|what: char| !what.is_alphanumeric())
                .to_owned()
        })
        .filter(|word| word.len() > 3)
        .collect();
    let landed = words.iter().filter(|word| slug.contains(*word)).count();
    landed >= ENOUGH_TO_LOOK
}

/// **A number a promise states, against the number the code has.**
///
/// *Making it yours* stayed ticked, claiming *the accent set as working code:
/// five hues*, after `Accent::ALL` became `[Self; 4]` — under a doc comment that
/// still read *All five*. The change that falsified it touched neither
/// `ROADMAP.md` nor `docs/features.md`, passed review, and left a ticked box
/// saying something untrue for as long as nobody counted.
///
/// A count is a fact a crate can hold. Each pair below is a number written in
/// the definition and the expression in this repository that has to equal it;
/// the caller supplies what the code actually says, because reading Rust is not
/// this crate's business and a caller that already has the value is the honest
/// place for it.
#[must_use]
pub fn counts_that_drifted(counted: &[(&str, usize, usize)]) -> Vec<Finding> {
    counted
        .iter()
        .filter(|(_, promised, actual)| promised != actual)
        .map(|(what, promised, actual)| Finding::ACountThatDrifted {
            what: (*what).to_owned(),
            promised: *promised,
            actual: *actual,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A roadmap written the way `ROADMAP.md` is written.
    const A_ROADMAP: &str = "\
## v0.5 — a person can work on it all day

- [ ] Lock screen, suspend and resume
  - [x] **The code.**
        `alo-locking` and `alo-sleeping`
  - [ ] **On the machine.**
        no lid has ever closed
- [ ] Bluetooth: pairing, audio, keyboards, mice
  - [x] **The code.**
        `alo-bluetooth`, and nothing pairs that a person did not choose
  - [ ] **On the machine.**
        a radio
";

    /// **A box somebody is working on is not a box that is done.**
    ///
    /// `- [~]` is how a lane says it has this in hand, so three machines reading
    /// one roadmap can tell a task nobody has picked up from one already being
    /// worked on. Two lanes took ADR 0069 twenty minutes apart for want of
    /// exactly that.
    ///
    /// It needs no parsing of its own — [`a_box`] decides with `starts_with('x')`
    /// and anything else is untricked, which is the conservative reading and the
    /// right one. This test exists so it stays that way: a later change that
    /// counted `~` as done would report work in hand as work finished, and the
    /// gate would say a promise was kept while somebody was still writing it.
    #[test]
    fn a_box_being_worked_on_is_not_a_box_that_is_done() {
        const IN_HAND: &str = "\
## v0.5 — a person can work on it all day

- [ ] Lock screen, suspend and resume
  - [~] **The code.**
        `alo-locking` and `alo-sleeping`, in hand
  - [ ] **On the machine.**
        no lid has ever closed
";

        let sections = sections_in(IN_HAND);
        let halves: Vec<&Box_> = sections
            .iter()
            .flat_map(|section| &section.promises)
            .flat_map(|promised| &promised.halves)
            .collect();
        assert_eq!(halves.len(), 2, "both halves were read: {halves:?}");
        assert!(
            halves.iter().all(|half| !half.ticked()),
            "a box being worked on was counted as done: {halves:?}"
        );
        assert!(
            halves
                .iter()
                .any(|half| half.words().starts_with("**The code.**")),
            "the marker did not eat the words beside it: {halves:?}"
        );
    }

    /// A features document at two tiers.
    const A_DEFINITION: &str = "\
- [v0.5] Lock screen, suspend and resume
- [v0.5] Bluetooth: pairing, audio, keyboards, mice
- [v0.5] Input methods for non-Latin scripts
- [v1] Serving more than one person from one workstation
";

    /// **A promise the gate has no box for is found, and one it has a box for is
    /// not.**
    ///
    /// The finding this check exists for: three `[v0.5]` promises had no box on
    /// 2026-09-27, one of them with a finished crate and nothing to tick it in.
    #[test]
    fn a_promise_with_no_box_is_found_and_one_with_a_box_is_not() {
        let findings = promises_with_no_box(A_DEFINITION, A_ROADMAP, Tier::V0_5);
        let said: String = findings.iter().map(ToString::to_string).collect();
        assert!(
            said.contains("Input methods for non-Latin scripts"),
            "the promise with no box was not found: {said}"
        );
        assert!(
            !said.contains("Lock screen"),
            "a promise the gate has a box for was reported as having none: {said}"
        );
        assert_eq!(findings.len(), 1, "{said}");
    }

    /// **A tier with no section in the roadmap is said once**, rather than every
    /// promise at that tier being reported as missing a box.
    #[test]
    fn a_tier_the_roadmap_has_no_section_for_is_said_once() {
        let findings = promises_with_no_box(A_DEFINITION, A_ROADMAP, Tier::V1);
        assert!(
            matches!(findings.as_slice(), [Finding::ATierWithNoSection { .. }]),
            "{findings:?}"
        );
        assert!(
            promises_with_no_box(A_DEFINITION, A_ROADMAP, Tier::V2).is_empty(),
            "v2 has no gate in ROADMAP.md, so nothing can be said about its boxes"
        );
    }

    /// **A promise answered in another release's section is found.**
    ///
    /// *Camera and microphone* was `[v0.5]` in the definition, inside *Devices* in
    /// the roadmap's v1, and *carried to v2* in its plan — three documents, three
    /// answers.
    /// **The one that really happened.** ADR 0076 withdrew the per-display
    /// dock; `ROADMAP.md` kept the box open for three days with its code half
    /// ticked, citing a function the same change had deleted.
    #[test]
    fn a_withdrawn_promise_left_open_is_found() {
        let features = "\
**Withdrawn by [ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md):** *the person decides where it goes*, \
and its **[v0.5]** descendant *per display, so the dock can sit along the bottom of the laptop \
and down the side of the external screen*. The Dock is fixed to the bottom edge.
";
        let roadmap = "\
## v0.5 — a person can work on it all day

- [ ] **Per display, so the dock can sit along the bottom of the laptop and down
      the side of the external screen**
  - [x] **The code.**
        it was built, and then the promise was withdrawn
";
        let findings = withdrawals_left_open(features, roadmap);
        let said: String = findings.iter().map(ToString::to_string).collect();
        assert!(
            said.contains("per display"),
            "a withdrawn promise left open was not found: {said}"
        );
        assert!(
            said.contains("ADR 0076"),
            "the record was not named: {said}"
        );
    }

    /// **And a withdrawal the roadmap took out is not reported**, or the check
    /// would fail for ever on every withdrawal this repository ever makes.
    #[test]
    fn a_withdrawal_the_roadmap_honoured_is_not_found() {
        let features = "\
**Withdrawn by [ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md):** *per display, so the dock can sit along \
the bottom of the laptop and down the side of the external screen*.
";
        let roadmap = "\
## v0.5 — a person can work on it all day

- **Per display, so the dock can sit along the bottom** — WITHDRAWN by ADR 0076
- [ ] Lock screen, suspend and resume
";
        assert!(
            withdrawals_left_open(features, roadmap).is_empty(),
            "a box that says it was withdrawn was reported anyway"
        );
    }

    /// **A qualifier in italics is not a promise.** A withdrawal italicises
    /// short asides too, and a three-word span has no business matching a box.
    #[test]
    fn a_short_italic_aside_is_not_taken_for_a_promise() {
        let features = "\
**Withdrawn by [ADR 0076](decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md):** *in both orientations*.
";
        let roadmap = "- [ ] Something about orientations in both directions entirely unrelated\n";
        assert!(
            withdrawals_left_open(features, roadmap).is_empty(),
            "a short aside was matched against a box"
        );
    }

    #[test]
    fn a_promise_answered_at_another_tier_is_found() {
        let roadmap = "\
## v0.5 — a person can work on it all day

- [ ] Lock screen, suspend and resume

## v1 — an organisation can buy it

- [ ] Bluetooth: pairing, audio, keyboards, mice
";
        let findings = tiers_that_disagree(A_DEFINITION, roadmap);
        let said: String = findings.iter().map(ToString::to_string).collect();
        assert!(
            said.contains("Bluetooth"),
            "a v0.5 promise answered in the v1 gate was not found: {said}"
        );
    }

    /// **A half under the wrong promise is found**, and a promise with one of each
    /// is not.
    #[test]
    fn a_stray_half_is_found() {
        let stray = "\
## v0.5 — a person can work on it all day

- [ ] Lock screen, suspend and resume
  - [x] **The code.**
        its own
  - [ ] **On the machine.**
        its own
  - [x] **The code.**
        a half left behind by a promise that moved
";
        let findings = orphaned_boxes(stray);
        assert!(
            matches!(
                findings.as_slice(),
                [Finding::AHalfUnderTheWrongPromise {
                    code: 2,
                    machine: 1,
                    ..
                }]
            ),
            "{findings:?}"
        );
        assert!(
            orphaned_boxes(A_ROADMAP).is_empty(),
            "a promise with one of each half was reported as carrying a stray"
        );
    }

    /// **A box denying work its own named task says is done is found** — and one
    /// that withdrew the denial and kept the record of it is not.
    #[test]
    fn a_denial_of_finished_work_is_found_and_a_withdrawn_one_is_not() {
        let plans = BTreeMap::from([(
            "v0-5-a-plan.md".to_owned(),
            "### 12. Pairing\n\n**Status:** done. Report: elsewhere.\n".to_owned(),
        )]);

        let denying = "\
## v0.5 — a person can work on it all day

- [ ] Machines find each other
      Most of it, and not whole: the person's door is task 12 of
      `v0-5-a-plan.md`, open
";
        let findings = denials_by_task_number(denying, &plans);
        assert!(
            matches!(
                findings.as_slice(),
                [Finding::ADenialOfFinishedWork { task: 12, .. }]
            ),
            "{findings:?}"
        );

        let withdrawn = "\
## v0.5 — a person can work on it all day

- [ ] Machines find each other
      The person's door is task 12 of `v0-5-a-plan.md`, and it is there.
      *This box said the door was **not whole** and task 12 **open** on 2026-09-26,
      which was wrong*
";
        assert!(
            denials_by_task_number(withdrawn, &plans).is_empty(),
            "a box that withdrew a denial and kept the record of having made it was read as still \
             making it. Deleting that record to satisfy a check would be the check making the \
             documents worse"
        );
    }

    /// **A status word its own body contradicts is found**, and a task whose
    /// status agrees with its body is not.
    #[test]
    fn a_status_its_own_body_contradicts_is_found() {
        let plan = "\
### 1. What an update is

**Status:** ready. **Depends on:** nothing.

**Done, 2026-09-14.** `crates/alo-keeping-up` holds it.

### 2. Something nobody has started

**Status:** ready. **Depends on:** 1.

Nothing has been written for this yet.

### 3. Something finished and said so

**Status:** **Done, 2026-09-16.** `crates/alo-somewhere`.

**Done, 2026-09-16.** The same date, said twice on purpose.
";
        let findings = statuses_their_own_section_contradicts(plan, "a-plan.md");
        assert!(
            matches!(
                findings.as_slice(),
                [Finding::AStatusItsOwnSectionContradicts { task: 1, .. }]
            ),
            "{findings:?}"
        );
    }

    /// **A count that drifted is found, and one that has not is not.**
    #[test]
    fn a_count_that_drifted_is_found() {
        assert_eq!(counts_that_drifted(&[("the accents", 5, 5)]).len(), 0);
        assert!(matches!(
            counts_that_drifted(&[("the accents", 5, 4)]).as_slice(),
            [Finding::ACountThatDrifted {
                promised: 5,
                actual: 4,
                ..
            }]
        ));
    }

    /// **A refusal is shown a report whose filename reads like the promise**, and
    /// a refusal that already names a report is not shown anything.
    #[test]
    fn a_refusal_is_shown_the_report_that_looks_like_it() {
        let reports = BTreeSet::from([
            "the-portal-backend-answers-on-a-real-bus.md".to_owned(),
            "something-else-entirely.md".to_owned(),
        ]);
        let denying = "\
## v0.5 — a person can work on it all day

- [ ] **The portal backend answers on a real bus**
  - [ ] **The code.**
        the portal backend does not exist
";
        let findings = refusals_a_report_contradicts(denying, &reports);
        assert!(
            matches!(
                findings.as_slice(),
                [Finding::ARefusalAReportContradicts { .. }]
            ),
            "{findings:?}"
        );

        let naming = "\
## v0.5 — a person can work on it all day

- [ ] **The portal backend answers on a real bus**
  - [ ] **The code.**
        the portal backend does not exist, and
        `docs/autonomy/updates/the-portal-backend-answers-on-a-real-bus.md` says why
";
        assert!(
            refusals_a_report_contradicts(naming, &reports).is_empty(),
            "a refusal that already names its report was shown it again"
        );
    }
}
