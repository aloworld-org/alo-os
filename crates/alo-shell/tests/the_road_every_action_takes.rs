//! **Every action a chord can be bound to has a road through this crate**, or is
//! named here as one that does not, with why.
//!
//! `alo_shortcuts::Action` is what each shortcut is **called**; this crate is
//! what each one **does**. A variant on that list with no arm here is a chord
//! that is shipped, bound, worded, translated — and indistinguishable from
//! pressing an unbound key.
//!
//! # This test was cited before it existed
//!
//! `alo_shortcuts::Action::Launcher`'s own documentation says:
//!
//! > `alo_shell::the_road_every_action_takes` names it as the single exception
//! > rather than letting it hide among the others.
//!
//! **There was no such thing.** Measured 2026-10-17: no module, no test, no file
//! of that name anywhere in this crate — so nothing named `Launcher` as the
//! exception, and nothing held the other actions to having a road at all. The
//! sentence was true about an intention and read as true about a mechanism.
//!
//! It is written now because `CLAUDE.md` asks for exactly this: *work a task
//! needs is part of that task — a caller that does not exist, a module with no
//! consumer, **a test that never ran** — build it, in the same change, and say in
//! the record that you did.* Adding `Action::WhyIsItWaiting` is what found it:
//! its own dispatch had nothing holding it either.
//!
//! # Why it reads the source
//!
//! Because the fault cannot be provoked. A missing arm does not fail, panic or
//! log — the chord is simply not recognised and falls through to whatever is
//! behind it, which is what pressing an unbound key does. There is no input that
//! separates *not bound* from *bound to nothing*, so the thing to read is the
//! file.
//!
//! **And it matches dispatch rather than the word.** A search for an action's
//! name finds the doc comments that explain it — and in this repository every
//! arm is explained in prose directly above itself, so a name-only search would
//! pass on the explanation of an arm that had been deleted. Comment lines are
//! excluded, and `the_search_can_tell_an_arm_from_a_sentence_about_one` is the
//! control.

#![expect(
    clippy::panic,
    clippy::expect_used,
    reason = "in a test, a panic on a crate that no longer routes an action is the failure being \
              reported"
)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use alo_shortcuts::Action;

/// Actions this crate deliberately does not route yet, and why.
///
/// **Every line here is a decision somebody has to make, not a fault to fix.**
/// A chord bound to a surface that does not exist is worse than an unbound one:
/// it teaches a person a road and then takes it away.
/// **Empty, and that is the point of it.**
///
/// It held `Action::Launcher` until 2026-10-20, with the reason that *which
/// surface a launcher is* had never been decided. The owner decided it — the
/// alo Bar, the field already on the Dock — and `crate::a_chord_opens_the_bar`
/// is the arm, so the excuse went with the decision rather than outliving it.
/// **That is this file's whole argument**: an entry here is a dated claim about
/// something that does not exist, nothing about building the thing touches the
/// claim, and `nothing_named_as_having_no_road_is_routed_after_all` is what
/// makes it expire. It caught this one.
///
/// An addition needs a sentence of over eighty characters saying what decision
/// is missing — not that the work is unfinished, which is true of everything.
const NO_ROAD_YET: [(Action, &str); 0] = [];

/// Every `.rs` file of this crate's own source.
fn the_source() -> Vec<(PathBuf, String)> {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    walk(&at, &mut found);
    assert!(
        found.len() > 40,
        "only {} source files were read, so this test is reading less of the crate than it thinks",
        found.len()
    );
    found
}

/// Every file beneath here, read as text.
fn walk(at: &Path, into: &mut Vec<(PathBuf, String)>) {
    for entry in std::fs::read_dir(at).expect("a directory of this crate's source") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            walk(&path, into);
        } else if path.extension().is_some_and(|it| it == "rs")
            // **Tests are not roads, and this was found by this file's own
            // sweep.** The first version counted every `.rs` under `src/`, and
            // reported `Launcher` as routed because three test modules name it —
            // two listing it as a Settings row and one asserting it is *not*
            // dispatched. A test naming an action is evidence that somebody
            // thought about it, never that a chord reaches it, and the test that
            // mentions it most is the one saying it does not.
            && !path
                .file_name()
                .is_some_and(|it| it.to_string_lossy().ends_with("_tests.rs"))
        {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|why| panic!("{} could not be read: {why}", path.display()));
            into.push((path, text));
        }
    }
}

/// Whether this action is named on a line of code rather than in a comment.
///
/// Comments are excluded for the reason in this file's header: every arm in this
/// crate is explained in prose directly above itself, so a name-only search
/// would pass on the explanation of an arm somebody had deleted.
fn routed(action: Action, source: &[(PathBuf, String)]) -> Vec<String> {
    let named = format!("Action::{action:?}");
    let mut found = Vec::new();
    for (path, text) in source {
        for (number, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") || code.starts_with("/*") || code.starts_with('*') {
                continue;
            }
            if code.contains(&named) {
                found.push(format!("{}:{}", path.display(), number + 1));
            }
        }
    }
    found
}

/// **Every action is routed, or named as one that is not.**
#[test]
fn every_action_has_a_road_or_is_named_as_having_none() {
    let source = the_source();
    let excused: BTreeSet<Action> = NO_ROAD_YET.iter().map(|(action, _)| *action).collect();

    let mut unrouted = Vec::new();
    let mut walked = 0_usize;
    for action in Action::ALL.iter().copied() {
        walked += 1;
        if excused.contains(&action) {
            continue;
        }
        if routed(action, &source).is_empty() {
            unrouted.push(format!("{action:?}"));
        }
    }

    assert!(
        walked >= 20,
        "only {walked} actions were walked, and this crate's keyboard has more than that"
    );
    assert!(
        unrouted.is_empty(),
        "these actions are shipped, bound, worded and translated, and nothing in this crate \
         routes them — pressing their chord is indistinguishable from pressing an unbound \
         key:\n  {}\n\nEither route it, or add it to `NO_ROAD_YET` in this file with the \
         decision that is missing.",
        unrouted.join("\n  ")
    );
}

/// **Nothing named as having no road has quietly got one.**
///
/// The other half, and the reason the list is not a comment: an excuse that has
/// stopped being true is one somebody will read as current.
#[test]
fn nothing_named_as_having_no_road_is_routed_after_all() {
    let source = the_source();
    let mut routed_after_all = Vec::new();
    for (action, _) in NO_ROAD_YET {
        let found = routed(action, &source);
        if !found.is_empty() {
            routed_after_all.push(format!("{action:?} at {}", found.join(", ")));
        }
    }
    assert!(
        routed_after_all.is_empty(),
        "these are named as having no road and this crate routes them after all, so the reason \
         beside each is no longer true and should go: {routed_after_all:?}"
    );
}

/// **Each excuse says enough to act on.**
#[test]
fn each_action_with_no_road_says_what_is_missing() {
    for (action, why) in NO_ROAD_YET {
        assert!(
            why.len() > 80,
            "{action:?} is named as having no road and says too little about why: {why:?}"
        );
    }
}

/// **The control: the search can tell an arm from a sentence about one.**
///
/// Without this, a `routed` that matched comments would report every action as
/// routed — including one whose arm had been deleted and whose explanation
/// remained. Both directions are asserted on the same name.
#[test]
fn the_search_can_tell_an_arm_from_a_sentence_about_one() {
    let prose = vec![(
        PathBuf::from("pretend.rs"),
        "        // `Action::CloseWindow` is answered by the window dispatcher.\n".to_owned(),
    )];
    assert!(
        routed(Action::CloseWindow, &prose).is_empty(),
        "a doc comment naming an action was counted as routing it"
    );

    let code = vec![(
        PathBuf::from("pretend.rs"),
        "        if shortcuts.action_for(chord) == Some(Action::CloseWindow) {\n".to_owned(),
    )];
    assert_eq!(
        routed(Action::CloseWindow, &code).len(),
        1,
        "a line of code naming an action was not counted as routing it"
    );
}

/// **And the action this change added is routed**, named rather than left to the
/// sweep.
///
/// The sweep would catch it, and a named test says which change is responsible:
/// `Action::WhyIsItWaiting` was added on 2026-10-17 for a promise whose answers
/// were built and whose asking had no road.
#[test]
fn asking_why_a_command_is_waiting_is_routed() {
    let source = the_source();
    let found = routed(Action::WhyIsItWaiting, &source);
    assert!(
        !found.is_empty(),
        "nothing in this crate routes the chord that asks why a command is waiting"
    );
}

/// **Every chord handler is called by the chain**, which is the half the sweep
/// above cannot see.
///
/// # Why this exists, and it is the more important of the two
///
/// The sweep above asks whether an action is *named* in code. That is not the
/// same as *reached*, and the difference was found by mutation rather than by
/// reasoning: deleting the arm that calls
/// `a_chord_asks_why_a_command_is_waiting` from the chain left **all five tests
/// in this file passing**, because the handler still names its own action in its
/// own first line. The chord was shipped, bound, worded, translated, handled —
/// and unreachable, which is precisely the fault this file was written to catch.
///
/// So the second field: a handler named `a_chord_…` is a road only if the chain
/// calls it. Two fields that must agree, which is what caught every one of this
/// week's faults; one field is a check, two are a check that can fail for the
/// right reason.
#[test]
fn every_chord_handler_is_called_by_the_chain() {
    let source = the_source();
    // **The chain's code, with its prose removed.** Not an optimisation: the
    // first version of this test searched the whole file and passed the mutation
    // it was written for, because the comment *above* the arm names the handler
    // the arm calls. In this repository every arm is explained directly above
    // itself, so the explanation and the thing are always adjacent and a
    // `contains` over the file cannot tell them apart. Third time today.
    let chain: String = source
        .iter()
        .find(|(path, _)| path.ends_with("a_chord_reaches_its_action.rs"))
        .map(|(_, text)| {
            text.lines()
                .filter(|line| {
                    let code = line.trim_start();
                    !code.starts_with("//") && !code.starts_with("/*") && !code.starts_with('*')
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .expect("this crate has a chord dispatch chain");

    // Every handler the crate declares, by the name the chain would call.
    let mut handlers: Vec<String> = Vec::new();
    for (path, text) in &source {
        if path.ends_with("a_chord_reaches_its_action.rs") {
            continue;
        }
        for line in text.lines() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            if let Some(rest) = code.strip_prefix("pub(crate) fn a_chord_")
                && let Some(name) = rest.split('(').next()
            {
                handlers.push(format!("a_chord_{name}"));
            }
        }
    }

    assert!(
        handlers.len() >= 4,
        "only {} chord handlers were found, and this crate has more than that: {handlers:?}",
        handlers.len()
    );

    let uncalled: Vec<&String> = handlers
        .iter()
        .filter(|named| !chain.contains(named.as_str()))
        .collect();
    assert!(
        uncalled.is_empty(),
        "these chord handlers exist and the dispatch chain never calls them, so the chords they \
         answer are indistinguishable from unbound keys:\n  {uncalled:?}\n\n\
         A handler naming its own action is not a road. Add it to the chain in \
         `a_chord_reaches_its_action.rs`."
    );
}

/// **The control for the chain search: a comment naming a handler is not a
/// call.**
///
/// This is the control the previous version of `every_chord_handler_is_called_by_the_chain`
/// did not have, and the absence cost two attempts. The mutation it is written
/// for — deleting an arm and leaving the comment above it — passed twice before
/// this existed, because `contains` over the file matched the explanation.
///
/// Both directions on the same name, because a filter that stripped everything
/// would also pass the negative.
#[test]
fn a_comment_naming_a_handler_is_not_a_call_to_it() {
    let only_prose = "        // `crate::a_chord_opens_a_terminal`'s header has the four roads.\n";
    let code_only: String = only_prose
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code_only.contains("a_chord_opens_a_terminal"),
        "a comment naming a handler survived the filter, so the chain search can be satisfied by \
         prose"
    );

    let a_real_call = "        if self.a_chord_opens_a_terminal(&shortcuts, chord) {\n";
    let kept: String = a_real_call
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        kept.contains("a_chord_opens_a_terminal"),
        "the filter removed a real call, so the chain search would report every handler uncalled"
    );
}
