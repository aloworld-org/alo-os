//! A task publishes only what its plan may change.
//!
//! Every plan's header names the crates it owns and the ones it **reads and never
//! edits**, and most add *Nothing in `crates/alo-shell`*. Those lines are how
//! several machines work in one repository without two of them editing one crate,
//! which corrupted a task on 2026-09-11. Until now nothing read them but people.
//! On 2026-09-16 a person caught three handoffs that edited a crate their plan
//! said it never edits: `alo-agentd`, `alo-printing` and `alo-capability`. Each was
//! found only after its gates, and one only minutes before it would have pushed.
//!
//! So before a task is gated, the files its handoff names are read against its
//! plan's header, and a file in a crate the plan never edits is refused with the
//! crate named. Nothing is gated, committed or pushed. The refusal says what a
//! worker can do about it: hand over only what is inside the plan, and mark the
//! task blocked on the crate's owner.
//!
//! # The registrations every crate makes
//!
//! A crate that declares words is listed in `alo-saying`'s collected list, and a
//! crate that declares verbs in `alo-by-hand`'s, because workspace tests refuse the
//! workspace otherwise. Every plan that adds such a crate has to touch those files,
//! whatever its header says about the crates they are in, so those files alone are
//! not refused.

/// Files every plan may change, because a workspace test makes every new crate
/// with words or verbs register there.
const REGISTRATIONS: [&str; 4] = [
    "crates/alo-saying/src/collecting.rs",
    "crates/alo-saying/Cargo.toml",
    "crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs",
    "crates/alo-by-hand/Cargo.toml",
];

/// Where a plan's tasks begin, and so where its header ends.
const THE_TASKS: &str = "\n## Tasks";

/// The words a header introduces its forbidden crates with.
const NEVER_EDITS: &str = "never edits";

/// The words a header introduces a crate it keeps entirely out of with.
const NOTHING_IN: &str = "Nothing in";

/// What a header says a plan may not change: crate names like `alo-record`, and
/// paths ending in `/` like `image/`.
#[must_use]
pub fn never_edited(plan: &str) -> Vec<String> {
    let header = plan.split(THE_TASKS).next().unwrap_or(plan);
    let mut refused: Vec<String> = Vec::new();
    let mut owned: Vec<String> = Vec::new();

    for paragraph in header.split("\n\n") {
        // Headers are wrapped, and *never edits* is as likely as anything to be
        // split across two lines.
        let paragraph = paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
        let paragraph = paragraph.as_str();
        if let Some((before, after)) = paragraph.split_once(NEVER_EDITS) {
            owned.extend(named_in(before));
            refused.extend(named_in(after));
        }
        let mut rest = paragraph;
        while let Some((_, after)) = rest.split_once(NOTHING_IN) {
            let sentence = after.split('.').next().unwrap_or(after);
            refused.extend(named_in(sentence));
            rest = after;
        }
    }

    let mut kept: Vec<String> = Vec::new();
    for name in refused {
        if !owned.contains(&name) && !kept.contains(&name) {
            kept.push(name);
        }
    }
    kept
}

/// The crates and directories written in backticks in some text.
fn named_in(text: &str) -> Vec<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .filter_map(|quoted| {
            let quoted = quoted.trim();
            let name = quoted.strip_prefix("crates/").unwrap_or(quoted);
            let a_crate = name.starts_with("alo-")
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
            let a_directory = name.ends_with('/') && !name.contains(' ') && !name.starts_with('.');
            (a_crate || a_directory).then(|| name.to_owned())
        })
        .collect()
}

/// Each file among `files` that is in something `refused` names, with the name it
/// is in.
#[must_use]
pub fn outside(files: &[String], refused: &[String]) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for file in files {
        let file = file.replace('\\', "/");
        if REGISTRATIONS.contains(&file.as_str()) {
            continue;
        }
        for name in refused {
            let inside = if name.ends_with('/') {
                file.starts_with(name.as_str())
            } else {
                file.starts_with(&format!("crates/{name}/"))
            };
            if inside {
                found.push((file.clone(), name.clone()));
            }
        }
    }
    found
}

/// The refusal a supervisor gives for files outside the plan, or `None`.
#[must_use]
pub fn refusal(files: &[String], plan: &str, plan_named: &str) -> Option<String> {
    let found = outside(files, &never_edited(plan));
    if found.is_empty() {
        return None;
    }
    let listed = found
        .iter()
        .map(|(file, name)| format!("{file} (in `{name}`)"))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!(
        "this task changes what {plan_named}'s header says the plan never edits: {listed}. \
         Nothing was gated or published. Another lane may own that crate, and two lanes in one \
         crate is what the plans' headers exist to prevent. If the task cannot be done without \
         that change, hand over only what is inside the plan and mark the task blocked on the \
         crate's owner, saying what change is needed"
    ))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A header written the way the broker plan's is.
    const A_HEADER: &str = "# a plan\n\n**Crates this plan owns, all new:** `crates/alo-broker` \
        (the broker) and `crates/alo-encrypting`. **It reads and never\nedits** `alo-capability`, \
        `alo-printing` (the documents plan's), `alo-record`, `image/`, and `alo-saying`. \
        **Nothing in `crates/alo-shell`.**\n\n**What this plan may not do:** mention `alo-broker` \
        again.\n\n## Tasks\n\n### 1. Something\n\nIt edits `alo-turn` in its prose.\n";

    fn files(named: &[&str]) -> Vec<String> {
        named.iter().map(|&name| name.to_owned()).collect()
    }

    /// **The header's forbidden crates are read, and the owned ones are not
    /// among them**, nor anything named in the tasks below the header.
    #[test]
    fn the_header_names_what_the_plan_never_edits() {
        let refused = never_edited(A_HEADER);
        for name in [
            "alo-capability",
            "alo-printing",
            "alo-record",
            "image/",
            "alo-saying",
            "alo-shell",
        ] {
            assert!(
                refused.contains(&name.to_owned()),
                "{name} missing from {refused:?}"
            );
        }
        for name in ["alo-broker", "alo-encrypting", "alo-turn"] {
            assert!(
                !refused.contains(&name.to_owned()),
                "{name} wrongly refused: {refused:?}"
            );
        }
    }

    /// **A handoff inside its plan is not refused.**
    #[test]
    fn work_inside_the_plan_passes() {
        let handed = files(&[
            "crates/alo-broker/src/lib.rs",
            "docs/quirks.md",
            "Cargo.lock",
        ]);
        assert_eq!(refusal(&handed, A_HEADER, "the plan"), None);
    }

    /// **A file in a crate the plan never edits is refused, naming the file and
    /// the crate**, which is what the three handoffs of 2026-09-16 needed.
    #[test]
    fn a_file_in_a_crate_the_plan_never_edits_is_refused_by_name() {
        let handed = files(&[
            "crates/alo-broker/src/lib.rs",
            "crates/alo-printing/src/found.rs",
        ]);
        let refused = refusal(&handed, A_HEADER, "the broker plan").unwrap();
        assert!(
            refused.contains("crates/alo-printing/src/found.rs"),
            "{refused}"
        );
        assert!(refused.contains("`alo-printing`"), "{refused}");
        assert!(
            refused.contains("Nothing was gated or published"),
            "{refused}"
        );

        let under_a_directory = files(&["image/usr/lib/tmpfiles.d/alo.conf"]);
        assert!(refusal(&under_a_directory, A_HEADER, "the plan").is_some());
    }

    /// **The registrations every crate with words or verbs must make are not
    /// refused**, though the crates they are in are.
    #[test]
    fn the_registrations_every_new_crate_makes_pass() {
        let handed = files(&[
            "crates/alo-saying/src/collecting.rs",
            "crates/alo-saying/Cargo.toml",
            "crates/alo-broker/src/words.rs",
        ]);
        assert_eq!(refusal(&handed, A_HEADER, "the plan"), None);

        let not_a_registration = files(&["crates/alo-saying/src/rented.rs"]);
        assert!(refusal(&not_a_registration, A_HEADER, "the plan").is_some());
    }

    /// **A plan with no such lines refuses nothing.**
    #[test]
    fn a_plan_whose_header_forbids_nothing_refuses_nothing() {
        let handed = files(&["crates/alo-anything/src/lib.rs"]);
        assert_eq!(refusal(&handed, "# a plan\n\n## Tasks\n", "the plan"), None);
    }

    /// **A real plan's header is read the way a person reads it.** The broker
    /// plan's worker changed `alo-printing` on 2026-09-16, which its header says it
    /// never edits; that handoff is refused, and the broker's own crate is not.
    #[test]
    fn the_broker_plans_header_refuses_the_printing_change_it_says_it_never_makes() {
        let plan = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../docs/autonomy/v0-5-the-broker-and-the-disk-plan.md"),
        )
        .unwrap();
        let refused = never_edited(&plan);
        assert!(refused.contains(&"alo-printing".to_owned()), "{refused:?}");
        assert!(!refused.contains(&"alo-broker".to_owned()), "{refused:?}");
        assert!(
            refusal(
                &files(&["crates/alo-printing/src/found.rs"]),
                &plan,
                "the broker plan"
            )
            .is_some()
        );
    }
}
