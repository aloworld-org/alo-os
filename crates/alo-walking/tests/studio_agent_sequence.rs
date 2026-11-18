//! The fixed permission sequence, against injected typed requests only.
//!
//! These scripted answers test orchestration and fail-fast behavior. A future
//! real launcher must supply one held agent connection and native Settings
//! boundaries; these tests provide neither and certify no installed execution.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "an unexpected scripted request or malformed fixture must fail the test"
)]

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use alo_protocol::{Done, FromAnAgent, ToAnAgent};
use alo_walking::studio_agent_probe::{Stage, run};
use serde_json::{Value, json};

const ACTIONS: [&str; 19] = [
    "studio.status",
    "studio.new_recording",
    "studio.choose_screen",
    "studio.choose_window",
    "studio.start",
    "studio.pause",
    "studio.resume",
    "studio.finish",
    "studio.enable_microphone",
    "studio.disable_microphone",
    "studio.set_quality",
    "studio.countdown_off",
    "studio.countdown_three",
    "studio.countdown_five",
    "studio.cancel_countdown",
    "studio.show_library",
    "studio.review",
    "studio.rename",
    "studio.export",
];

#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Boundary(Stage),
    Request(FromAnAgent),
}

#[derive(Clone)]
struct Expected {
    event: Event,
    answer: Result<Option<ToAnAgent>, String>,
}

fn read_status() -> FromAnAgent {
    FromAnAgent::Read {
        verb: "studio.status".to_owned(),
        given: Vec::new(),
    }
}

fn enable_microphone() -> FromAnAgent {
    FromAnAgent::Propose {
        verb: "studio.enable_microphone".to_owned(),
        given: Vec::new(),
    }
}

fn refused() -> ToAnAgent {
    ToAnAgent::read(
        r#"{"format":1,"tells":{"refused":{"text":"No access.","came_from":"the-source"}}}"#,
    )
    .expect("typed refusal fixture")
}

fn capabilities(allowed: bool) -> ToAnAgent {
    ToAnAgent::Capabilities {
        verbs: if allowed {
            ACTIONS.into_iter().map(str::to_owned).collect()
        } else {
            Vec::new()
        },
    }
}

fn ready() -> Value {
    json!({"version":1,"phase":"ready","microphone_enabled":false,"microphone_active":false,
        "source_kind":"none","quality":"full_hd","countdown_seconds":0,"elapsed_ms":0,
        "recording_id":null,"preview_ready":false,"export_pending":false,
        "can_start":false,"can_pause":false,"can_resume":false,"can_finish":false})
}

fn status(state: Value) -> ToAnAgent {
    ToAnAgent::Did(Done::ApplicationControlled {
        application: "org.aloworld.Studio".to_owned(),
        action: "studio.status".to_owned(),
        reply: Some(state.to_string()),
        unanswered: false,
    })
}

fn sequence() -> Vec<Expected> {
    let mut plan = Vec::new();
    for (stage, allowed) in [
        (Stage::AskBeforeGrant, false),
        (Stage::Granted, true),
        (Stage::Revoked, false),
        (Stage::Full, true),
        (Stage::AskAgain, false),
    ] {
        plan.push(Expected {
            event: Event::Boundary(stage),
            answer: Ok(None),
        });
        plan.push(Expected {
            event: Event::Request(FromAnAgent::Capabilities),
            answer: Ok(Some(capabilities(allowed))),
        });
        plan.push(Expected {
            event: Event::Request(read_status()),
            answer: Ok(Some(if allowed { status(ready()) } else { refused() })),
        });
        if !allowed {
            plan.push(Expected {
                event: Event::Request(enable_microphone()),
                answer: Ok(Some(refused())),
            });
        }
    }
    plan
}

fn exercise(plan: Vec<Expected>) -> (Result<(), String>, Vec<Event>, usize) {
    let pending = Rc::new(RefCell::new(VecDeque::from(plan)));
    let seen = Rc::new(RefCell::new(Vec::new()));
    let request_pending = Rc::clone(&pending);
    let request_seen = Rc::clone(&seen);
    let boundary_pending = Rc::clone(&pending);
    let boundary_seen = Rc::clone(&seen);
    let result = run(
        &mut |request| {
            let event = Event::Request(request.clone());
            request_seen.borrow_mut().push(event.clone());
            let expected = request_pending
                .borrow_mut()
                .pop_front()
                .expect("no extra request/poll");
            assert_eq!(
                event, expected.event,
                "fixed request order and native boundary must be honored"
            );
            expected
                .answer
                .map(|answer| answer.expect("request answer"))
        },
        &mut |stage| {
            let event = Event::Boundary(stage);
            boundary_seen.borrow_mut().push(event.clone());
            let expected = boundary_pending
                .borrow_mut()
                .pop_front()
                .expect("no extra native boundary");
            assert_eq!(
                event, expected.event,
                "person completes each fixed native boundary before checking it"
            );
            expected.answer.map(|answer| assert!(answer.is_none()))
        },
    );
    let trace = seen.borrow().clone();
    let remaining = pending.borrow().len();
    (result, trace, remaining)
}

#[test]
fn exact_five_stage_sequence_uses_only_discovery_fixed_reads_and_denied_fixed_changes() {
    let plan = sequence();
    let expected: Vec<_> = plan.iter().map(|entry| entry.event.clone()).collect();
    let (result, trace, remaining) = exercise(plan);
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(trace, expected);
    assert_eq!(remaining, 0);
}

#[test]
fn every_transport_or_native_boundary_failure_stops_before_any_later_progress() {
    let baseline = sequence();
    for (at, _) in baseline.iter().enumerate() {
        let mut plan = baseline.clone();
        let marker = format!("test-only failure at event {at}");
        plan.get_mut(at).expect("event exists").answer = Err(marker.clone());
        let (result, trace, remaining) = exercise(plan);
        assert!(result.is_err_and(|error| error.contains(&marker)));
        assert_eq!(
            trace,
            baseline
                .iter()
                .take(at + 1)
                .map(|entry| entry.event.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            remaining,
            baseline.len() - at - 1,
            "no later stage may be announced"
        );
    }
}

fn assert_refuses_at(at: usize, answer: ToAnAgent) {
    let mut plan = sequence();
    let total = plan.len();
    plan.get_mut(at).expect("event exists").answer = Ok(Some(answer));
    let (result, trace, remaining) = exercise(plan);
    assert!(
        result.is_err(),
        "invalid observation must not advance the fixed probe"
    );
    assert_eq!(trace.len(), at + 1);
    assert_eq!(remaining, total - at - 1);
}

#[test]
fn wrong_discovery_fails_on_first_post_boundary_reply_without_a_grace_poll() {
    for (at, entry) in sequence().iter().enumerate() {
        if matches!(entry.event, Event::Request(FromAnAgent::Capabilities)) {
            let Some(ToAnAgent::Capabilities { verbs }) =
                entry.answer.as_ref().expect("valid plan")
            else {
                panic!("capability fixture")
            };
            assert_refuses_at(at, capabilities(verbs.is_empty()));
        }
    }
}

#[test]
fn ungranted_read_and_change_must_each_refuse_in_every_ungranted_stage() {
    for (at, entry) in sequence().iter().enumerate() {
        if matches!(entry.answer, Ok(Some(ToAnAgent::Refused(_)))) {
            // A valid response/state, pending approval or empty discovery is
            // not refusal of this exact valid operation.
            for wrong in [status(ready()), capabilities(false),
                ToAnAgent::read(r#"{"format":1,"tells":{"proposed":{"number":7,"sentence":{"text":"Enable microphone.","came_from":"the-source"},"lapses_in":300}}}"#).expect("pending fixture")] {
                assert_refuses_at(at, wrong);
            }
        }
    }
}

#[test]
fn allowed_status_must_establish_ready_and_disabled_inactive_microphone_before_continuing() {
    let statuses: Vec<_> = sequence()
        .iter()
        .enumerate()
        .filter_map(|(at, entry)| matches!(entry.answer, Ok(Some(ToAnAgent::Did(_)))).then_some(at))
        .collect();
    assert_eq!(
        statuses.len(),
        2,
        "both Granted and Full need an actual precondition read"
    );
    for at in statuses {
        for field in [
            "version",
            "phase",
            "microphone_enabled",
            "microphone_active",
        ] {
            let mut missing = ready();
            missing.as_object_mut().expect("object").remove(field);
            assert_refuses_at(at, status(missing));
        }
        for (field, bad) in [
            ("version", json!(0)),
            ("version", json!(2)),
            ("version", json!(1.0)),
            ("version", json!("1")),
            ("phase", json!("Ready")),
            ("phase", json!("selected")),
            ("phase", json!("recording")),
            ("microphone_enabled", json!(true)),
            ("microphone_enabled", json!("false")),
            ("microphone_active", json!(true)),
            ("microphone_active", json!(null)),
        ] {
            let mut changed = ready();
            changed[field] = bad;
            assert_refuses_at(at, status(changed));
        }
        for wrong in [
            refused(),
            capabilities(true),
            status(json!(null)),
            status(json!([ready()])),
            status(json!({"status":ready()})),
            ToAnAgent::read(r#"{"format":1,"tells":{"proposed":{"number":7,"sentence":{"text":"read status","came_from":"the-source"},"lapses_in":300}}}"#).expect("proposal is not a status read"),
            ToAnAgent::read(r#"{"format":1,"tells":{"answered":{"text":"Studio is ready with its microphone off.","came_from":{"text":"this machine","came_from":"the-source"},"model":"test-only"}}}"#).expect("model prose is not a status read"),
        ] {
            assert_refuses_at(at, wrong);
        }
        for change in [
            "application",
            "action",
            "unanswered",
            "missing-reply",
            "bad-json",
        ] {
            let mut wrong = status(ready());
            if let ToAnAgent::Did(Done::ApplicationControlled {
                application,
                action,
                reply,
                unanswered,
            }) = &mut wrong
            {
                match change {
                    "application" => *application = "org.aloworld.Foreign".to_owned(),
                    "action" => *action = "studio.enable_microphone".to_owned(),
                    "unanswered" => *unanswered = true,
                    "missing-reply" => *reply = None,
                    _ => *reply = Some("{unfinished".to_owned()),
                }
            }
            assert_refuses_at(at, wrong);
        }
        let wrapped = ToAnAgent::read(r#"{"format":1,"tells":{"done-without-asking":{"did":{"application-controlled":{"application":"org.aloworld.Studio","action":"studio.status","reply":"{\"version\":1,\"phase\":\"ready\",\"microphone_enabled\":false,\"microphone_active\":false}","unanswered":false}},"said":{"text":"done","came_from":"the-source"}}}}"#).expect("wrapped fixture");
        assert_refuses_at(at, wrapped);
    }
}
