//! An application cannot notify with an action that answers an approval.
//!
//! ADR 0001: a change to the machine is proposed with a sentence and waits for
//! one approval, and **what a person approves is that sentence**. They read it
//! on the approval surface, which they went to in order to approve something. A
//! notification is the opposite kind of thing: it arrives uninvited, beside
//! whatever they were doing, and a person tapping a button on one has not read
//! a proposal. A notification action that could answer one would be an approval
//! obtained somewhere nobody went to approve anything — and it is the easiest
//! feature in the world to add by accident, because *let the notification carry
//! the buttons* is what every other system does.
//!
//! So this holds it three ways, and the first is the one that matters:
//!
//! 1. against a change that is **really waiting** — proposed by a real turn,
//!    put to a real approval surface — which is still waiting after the person
//!    has picked every action a notification could offer;
//! 2. against this crate's **manifest**: `alo-approving` is a dev-dependency of
//!    these tests and not a dependency of the crate, so nothing shipped can name
//!    an approval at all;
//! 3. against this crate's **shipped source**, for any spelling of answering
//!    one.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod common;

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use alo_approving::{Approving, Asks, Compositor, SurfaceRefused};
use alo_capability::{Given, Grant, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching};
use alo_notifying::Picked;
use alo_record::Record;
use alo_turn::{Bounding, Doing, Done, Machine, NoBoundary, Turning};

use common::{a_notification_offering, the_machines_words};

/// Everything this crate depends on, exactly.
///
/// A closed list rather than a search for a name: the way a crate about
/// notifications gains the ability to answer an approval is by gaining a
/// dependency that can, and adding anything here is a deliberate act with this
/// test in front of it. `alo-approving` appears in this crate's manifest only
/// under `[dev-dependencies]`, which is where this very test reaches it from.
const EVERYTHING_IT_DEPENDS_ON: [&str; 10] = [
    "alo-locking",
    "alo-portals",
    "alo-capability",
    "alo-applications",
    "alo-appearance",
    "alo-in-use",
    "alo-kept",
    "alo-strings",
    "serde",
    "thiserror",
];

/// A screen that keeps whatever question it was last given.
#[derive(Default)]
struct AScreen {
    /// The question on it.
    showing: Option<alo_approving::Asked>,
}

impl Compositor for AScreen {
    fn ask(&mut self, asked: alo_approving::Asked) -> Result<(), SurfaceRefused> {
        self.showing = Some(asked);
        Ok(())
    }
}

/// A boundary that bounds nothing, because what is being tested is the
/// approval rather than the kernel.
struct NothingIsBounded;

impl Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _r: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), NoBoundary> {
        doing();
        Ok(())
    }
}

/// **A change that is waiting is still waiting after the person has picked
/// everything a notification could offer them.**
///
/// The notification is a real one, sent by a real application under a real
/// grant, and it offers three things whose names are every spelling somebody
/// would reach for: *approve*, *yes*, *allow*. Picking each of them produces a
/// [`Picked`] addressed to the mail client — and the proposal on the approval
/// surface has not moved.
#[test]
fn a_change_that_is_waiting_is_still_waiting_after_every_action_is_picked() {
    let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    let hour = Duration::from_secs(60 * 60);
    let strings = the_machines_words();

    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder("/home/anna/Invoices".into()),
            now,
            hour,
        )
        .unwrap(),
    );
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_verbs(
        &strings,
        &OnThisMachine,
        &mut bounding,
        &mut indicator,
        &mut record,
    )
    .unwrap();
    let mut turning = Turning::beginning(
        Context::at_invocation(now),
        "@files",
        hour,
        &mut grants,
        &mut machine,
    )
    .unwrap();

    let id = turning
        .proposing(
            "rename_file",
            &[
                ("file", Given::text("/home/anna/Invoices/march.pdf")),
                ("name", Given::text("march-final.pdf")),
            ],
            &grants,
            hour,
            now,
        )
        .unwrap();

    let mut screen = AScreen::default();
    let mut approving = Approving::nothing_to_answer();
    let Asks::Asked(asked) = approving.ask(Some(&mut screen), &turning, id, now) else {
        panic!("a change that was waiting was not put to anybody");
    };
    assert_eq!(asked.id(), id);
    assert!(approving.is_asking());

    // The person's mail client sends a notification offering every spelling of
    // *yes* somebody would reach for, and the person picks all three.
    let notification = a_notification_offering(
        "Anna Pärt",
        &[("approve", "Approve"), ("yes", "Yes"), ("allow", "Allow")],
    );
    for action in notification.offering() {
        let picked = Picked::of(&notification, action).unwrap();
        assert_eq!(
            picked
                .to()
                .application()
                .map(alo_applications::Application::identifier),
            Some(common::THE_MAIL_CLIENT),
            "what was picked went somewhere other than the sender"
        );
        assert!(
            picked.to().agent().is_none(),
            "it was addressed to an agent"
        );
    }

    // And the change is exactly where it was: on the approval surface, waiting.
    assert!(
        approving.is_asking(),
        "picking something on a notification answered a waiting change"
    );
    assert_eq!(approving.asking(), Some(id));
    assert!(screen.showing.is_some(), "the surface lost its question");
}

/// **Nothing that comes out of a notification carries a proposal.** What a
/// picked action holds is who sent the notification and the sender's own name
/// for it, and there is no third field for an approval to travel in.
#[test]
fn nothing_that_comes_out_of_a_notification_carries_a_proposal() {
    let notification = a_notification_offering("Anna Pärt", &[("approve", "Approve")]);
    let action = notification.offering().first().unwrap();
    let picked = Picked::of(&notification, action).unwrap();
    assert_eq!(picked.named(), "approve");
    assert_eq!(
        picked
            .to()
            .application()
            .map(alo_applications::Application::identifier),
        Some(common::THE_MAIL_CLIENT)
    );

    // And an action this notification never offered cannot be picked at all,
    // so a name a shell read off anything else goes nowhere: what comes out is
    // always one of the things this notification itself put in front of the
    // person, addressed to the sender that offered it.
    let never_offered = alo_notifying::Action::offering("delete", "Delete").unwrap();
    assert_eq!(Picked::of(&notification, &never_offered), None);

    // An action is the sender's own name and the words the person read, and
    // one that differs in either is a different action — the label included,
    // because a shell that relabelled a button would be reporting that the
    // person picked something they were never shown.
    let relabelled = alo_notifying::Action::offering("approve", "Approve everything").unwrap();
    assert_eq!(Picked::of(&notification, &relabelled), None);
}

/// Every source file this crate ships, with its test modules cut off.
fn the_shipped_source() -> Vec<(String, String)> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    for entry in fs::read_dir(&src).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "testing.rs" {
            continue;
        }
        let text = fs::read_to_string(entry.path()).unwrap();
        let shipped = text.split("#[cfg(test)]").next().unwrap_or("").to_owned();
        found.push((name, shipped));
    }
    assert!(found.len() > 12, "this crate's source was not found");
    found
}

/// The lines of a file that are code rather than what somebody wrote about it.
fn the_code_of(text: &str) -> String {
    text.lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with("//"))
        .collect::<Vec<&str>>()
        .join("\n")
}

/// This crate's manifest, and every dependency line in it that is not a
/// comment.
fn what_it_depends_on() -> Vec<String> {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = fs::read_to_string(&at).unwrap();
    let mut named = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == "[dependencies]";
            continue;
        }
        if !inside || line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(name) = line.split_whitespace().next() {
            named.push(name.to_owned());
        }
    }
    assert!(!named.is_empty(), "no dependency was read at all");
    named
}

/// **Nothing in this crate's shipped code names an approval**, in any
/// spelling. A crate that cannot say the word cannot answer one by accident.
#[test]
fn nothing_in_this_crates_code_names_an_approval() {
    let never = [
        "alo_approving",
        "Approving",
        "Proposal",
        "ProposalId",
        "approve",
        "Approve",
        "decline",
        "Decline",
        "Answered",
        "alo_turn",
        "Turning",
    ];
    for (name, shipped) in the_shipped_source() {
        let code = the_code_of(&shipped);
        for what in never {
            assert!(
                !code.contains(what),
                "src/{name} names `{what}` in its code: a notification never answers an approval"
            );
        }
    }
}

/// **It depends on nothing that could answer an approval for it**, and the
/// list of what it depends on is closed.
#[test]
fn it_depends_on_nothing_that_could_answer_an_approval() {
    let mut depends = what_it_depends_on();
    let mut expected: Vec<String> = EVERYTHING_IT_DEPENDS_ON
        .iter()
        .map(|it| (*it).to_owned())
        .collect();
    depends.sort();
    expected.sort();
    assert_eq!(
        depends, expected,
        "this crate's dependencies changed: a crate about notifications that can reach an \
         approval is how *approval is only ever the approval surface* stops being true"
    );
    assert!(
        !depends.contains(&"alo-approving".to_owned()),
        "alo-approving is a dependency of this crate rather than of its tests"
    );
}

/// Whether one thing a notification could offer is one this crate accepts,
/// used by the test below to say what it means to offer something.
fn offers(named: &str, label: &str) -> bool {
    alo_notifying::Action::offering(named, label).is_ok()
}

/// **An action's name is the sender's and means nothing to alo OS.** There is
/// no reserved word here and there does not need to be one: *approve* is
/// accepted exactly as *reply* is, because neither can reach anything but the
/// program that sent it. A crate that kept a list of forbidden names would be
/// a crate that believed a name could do something.
#[test]
fn an_actions_name_is_the_senders_and_means_nothing_to_alo_os() {
    for named in ["reply", "approve", "yes", "allow", "grant", "confirm"] {
        assert!(offers(named, "Something"), "{named} was refused as a name");
    }
    let notification =
        a_notification_offering("Anna Pärt", &[("grant", "Grant"), ("confirm", "Confirm")]);
    assert_eq!(notification.offering().len(), 2);
}
