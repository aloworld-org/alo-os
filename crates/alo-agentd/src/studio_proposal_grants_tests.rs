//! Application permission is checked before the actual daemon creates a card.
//!
//! These use the production request router and one held turn, not scripted
//! answers supplied to the private probe. Transport observations are test-only;
//! this is not installed native approval or application-state evidence.

use std::cell::RefCell;

use alo_capability::{Grant, Grants, HowCodeRuns, Reach};
use alo_protocol::{Done, ToAnAgent};
use alo_record::Record;
use alo_turn::{Machine, Turning};

use super::{Elsewhere, THE_AGENT, the_agent_asked, the_number, the_person_approved};
use crate::testing::{NothingIsBounded, a_message, hour, in_english, noon};

/// Fixed application identity from the public Studio control contract.
const STUDIO: &str = "org.aloworld.Studio";

/// A literal next request, independent of adapter request-building helpers.
const ENABLE: &str = r#"{"propose":{"verb":"studio.enable_microphone","given":[]}}"#;
/// Full access disables the microphone without an application grant or approval.
const DISABLE: &str = r#"{"propose":{"verb":"studio.disable_microphone","given":[]}}"#;
/// A read must also refuse once the same application grant is gone.
const STATUS: &str = r#"{"read":{"verb":"studio.status","given":[]}}"#;

/// Record exactly which application methods the real executor attempts.
#[derive(Default)]
struct Transport(RefCell<Vec<String>>);

impl alo_adapters::Delivers for Transport {
    fn deliver(&self, message: &alo_adapters::Message) -> Result<(), alo_adapters::NotDelivered> {
        self.0.borrow_mut().push(message.method().to_owned());
        Ok(())
    }

    fn read_text(
        &self,
        message: &alo_adapters::Message,
    ) -> Result<String, alo_adapters::NotDelivered> {
        self.0.borrow_mut().push(message.method().to_owned());
        Ok(
            r#"{"version":1,"phase":"ready","microphone_enabled":false,"microphone_active":false}"#
                .to_owned(),
        )
    }
}

/// Unrelated application, accessibility and publishing roads must not be used.
struct NotUsed;

impl alo_turn::TheDoorTheCompositorServes for NotUsed {
    fn asking(
        &mut self,
        _: &alo_acting::Request,
    ) -> Result<alo_acting::Answer, alo_turn::NotAtTheDoor> {
        unreachable!("a semantic Studio control must not use a compositor gesture")
    }
}

impl alo_turn::SaysHowAnApplicationStarts for NotUsed {
    fn how_it_starts(&self, _: &str) -> Result<alo_turn::TheRoad, alo_turn::NoRoad> {
        unreachable!("a semantic Studio control must not launch an application")
    }
}

impl alo_turn::TheWindowsOnTheScreen for NotUsed {
    fn opening(
        &mut self,
    ) -> Result<Box<dyn alo_adapters::AccessibilityTree>, alo_adapters::TreeFault> {
        unreachable!("a semantic Studio control must not open an accessibility tree")
    }

    fn working_fully(&self, _: &str) -> alo_adapters::WorkingFully {
        unreachable!("a semantic Studio control must not ask about screen-road settings")
    }
}

impl alo_publishing::Runs for NotUsed {
    fn run(
        &self,
        _: &alo_capability::ACommand,
        _: &mut dyn alo_capability::AroundACommand,
    ) -> Result<alo_capability::WhatACommandDid, alo_capability::ItDidNotStart> {
        unreachable!("a semantic Studio control must not publish or run a command")
    }
}

/// A normal machine inventory with installed Studio and real semantic routing.
fn on_studio_turn(
    record: &mut Record,
    transport: &Transport,
    observe: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &alo_strings::Strings),
) {
    let strings = in_english();
    let tool = alo_software::rented::TheRentedTool::at("/studio-test/no-installer");
    let mut door = NotUsed;
    let mut windows = NotUsed;
    let mut boundary = NothingIsBounded;
    let mut indicator = alo_egress::Indicator::default();
    let mut machine = Machine::carrying_out_file_software_and_application_verbs(
        &strings,
        &alo_files::OnThisMachine,
        alo_turn::WhatASoftwareVerbNeeds {
            tool: &tool,
            bound: alo_software::Bound::Nobodys,
            publishing: alo_turn::WhatPublishingNeeds {
                runs: &NotUsed,
                through: alo_publishing::Through::on_github_com(),
                rule: alo_egress::EgressPolicy::Anywhere,
            },
        },
        alo_turn::WhatAnApplicationVerbNeeds {
            installed: alo_applications::Installed::holding([
                alo_applications::Application::called(STUDIO, "alo Studio").unwrap(),
            ])
            .into(),
            door: &mut door,
            starting: &NotUsed,
            windows: &mut windows,
        },
        &mut boundary,
        &mut indicator,
        record,
    )
    .unwrap()
    .with_application_controls(transport)
    .unwrap();
    let mut grants = Grants::default();
    let mut turn = Turning::beginning(
        alo_context::Context::at_invocation(noon()),
        THE_AGENT,
        hour(),
        &mut grants,
        &mut machine,
    )
    .unwrap();
    observe(&mut turn, &mut grants, &strings);
}

/// Discovery and both valid fixed operations refuse immediately, with no card.
fn require_no_studio_access(
    stage: &str,
    turn: &mut Turning<'_, '_>,
    grants: &Grants,
    strings: &alo_strings::Strings,
    here: &crate::running_a_command::CommandsHere,
) {
    let discovery = the_agent_asked(
        &a_message(r#"{"capabilities":{}}"#),
        turn,
        grants,
        strings,
        here,
    );
    let ToAnAgent::Capabilities { verbs } = discovery else {
        unreachable!("{stage}: discovery did not return capabilities: {discovery:?}");
    };
    assert!(
        !verbs.iter().any(|name| name.starts_with("studio.")),
        "{stage}: {verbs:?}"
    );
    for request in [STATUS, ENABLE] {
        let reply = the_agent_asked(&a_message(request), turn, grants, strings, here);
        assert!(
            matches!(reply, ToAnAgent::Refused(_)),
            "{stage}: {request} must explicitly refuse, got {reply:?}"
        );
        assert_eq!(
            turn.waiting_at(noon()).count(),
            0,
            "{stage}: refusal left a proposal waiting"
        );
    }
}

/// A no-argument application control must not bypass the proposal grant check.
#[test]
fn an_ungranted_studio_change_is_refused_before_a_card_at_each_restricted_level() {
    for level in [HowCodeRuns::AskingEachTime, HowCodeRuns::InASealedBox] {
        let elsewhere = Elsewhere::with_the_level("studio-ungranted-proposal", Some(level));
        let mut record = Record::default();
        let transport = Transport::default();
        on_studio_turn(&mut record, &transport, |turn, grants, strings| {
            require_no_studio_access("before grant", turn, grants, strings, &elsewhere.here());
            assert!(
                transport.0.borrow().is_empty(),
                "an ungranted request reached Studio"
            );
        });
        let entries: Vec<_> = record.everything().collect();
        assert_eq!(
            entries.len(),
            2,
            "both valid refused operations need records: {entries:?}"
        );
        assert!(
            entries.iter().all(|entry| entry.happened().was_stopped()),
            "{entries:?}"
        );
    }
}

/// Grant/revoke and Full/Ask changes affect the same live turn, never a restart.
#[test]
fn studio_proposal_refusal_tracks_withdrawal_without_breaking_ask_or_full_execution() {
    let elsewhere = Elsewhere::with_the_level(
        "studio-proposal-withdrawal",
        Some(HowCodeRuns::AskingEachTime),
    );
    let mut record = Record::default();
    let transport = Transport::default();
    on_studio_turn(&mut record, &transport, |turn, grants, strings| {
        let grant = grants.grant(
            Grant::checked(
                THE_AGENT,
                Reach::Application(STUDIO.to_owned()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        let status = the_agent_asked(&a_message(STATUS), turn, grants, strings, &elsewhere.here());
        assert!(
            matches!(status, ToAnAgent::Did(Done::ApplicationControlled { .. })),
            "granted status: {status:?}"
        );
        let proposal =
            the_agent_asked(&a_message(ENABLE), turn, grants, strings, &elsewhere.here());
        let number = the_number(&proposal);
        assert_eq!(turn.waiting_at(noon()).count(), 1);
        assert_eq!(
            *transport.0.borrow(),
            ["GetStatus"],
            "Ask change ran before approval"
        );
        let approved = the_person_approved(number, turn, grants, strings, &elsewhere.here());
        assert!(approved.done().is_some(), "granted approval: {approved:?}");
        assert_eq!(*transport.0.borrow(), ["GetStatus", "EnableMicrophone"]);
        assert_eq!(turn.waiting_at(noon()).count(), 0);
        assert!(grants.revoke(grant));
        require_no_studio_access("revoked", turn, grants, strings, &elsewhere.here());
        assert_eq!(
            *transport.0.borrow(),
            ["GetStatus", "EnableMicrophone"],
            "withdrawn request reached Studio"
        );

        elsewhere.choosing(HowCodeRuns::AtFullTrust);
        let full = the_agent_asked(
            &a_message(DISABLE),
            turn,
            grants,
            strings,
            &elsewhere.here(),
        );
        assert!(
            matches!(
                full,
                ToAnAgent::DoneWithoutAsking {
                    did: Done::ApplicationControlled { .. },
                    ..
                }
            ),
            "Full change: {full:?}"
        );
        assert_eq!(turn.waiting_at(noon()).count(), 0);
        assert_eq!(
            *transport.0.borrow(),
            ["GetStatus", "EnableMicrophone", "DisableMicrophone"]
        );
        elsewhere.choosing(HowCodeRuns::AskingEachTime);
        require_no_studio_access("Ask again", turn, grants, strings, &elsewhere.here());
        assert_eq!(
            *transport.0.borrow(),
            ["GetStatus", "EnableMicrophone", "DisableMicrophone"],
            "Ask again retained Full access"
        );
    });
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.happened().ran())
            .count(),
        3,
        "{entries:?}"
    );
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry.happened().was_stopped())
            .count(),
        4,
        "{entries:?}"
    );
}

/// The actual application grant must survive both Ask operations into the record.
#[test]
fn studio_status_and_approved_change_record_the_actual_application_grant() {
    let elsewhere =
        Elsewhere::with_the_level("studio-ask-provenance", Some(HowCodeRuns::AskingEachTime));
    let mut record = Record::default();
    let transport = Transport::default();
    let mut expected = None;
    on_studio_turn(&mut record, &transport, |turn, grants, strings| {
        let grant = grants.grant(
            Grant::checked(
                THE_AGENT,
                Reach::Application(STUDIO.to_owned()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        let status = the_agent_asked(&a_message(STATUS), turn, grants, strings, &elsewhere.here());
        assert!(
            matches!(status, ToAnAgent::Did(Done::ApplicationControlled { .. })),
            "granted status: {status:?}"
        );
        let proposal =
            the_agent_asked(&a_message(ENABLE), turn, grants, strings, &elsewhere.here());
        let approval = the_number(&proposal);
        let ToAnAgent::Proposed(card) = &proposal else {
            unreachable!("the granted Ask change needs its actual card: {proposal:?}");
        };
        assert_eq!(
            card.touches(),
            [alo_protocol::Touched::Application(STUDIO.to_owned())],
            "the actual approval card must name exactly Studio: {card:?}"
        );
        assert_eq!(
            *transport.0.borrow(),
            ["GetStatus"],
            "Ask change ran before approval"
        );
        let approved = the_person_approved(approval, turn, grants, strings, &elsewhere.here());
        assert!(approved.done().is_some(), "granted approval: {approved:?}");
        assert_eq!(turn.waiting_at(noon()).count(), 0);
        expected = Some((grant.as_u64(), approval));
    });
    assert_eq!(*transport.0.borrow(), ["GetStatus", "EnableMicrophone"]);
    let (grant, approval) = expected.unwrap();
    let entries: Vec<_> = record.everything().collect();
    let [status, change] = entries.as_slice() else {
        unreachable!("one status and one approved change need two entries: {entries:?}");
    };
    for entry in [status, change] {
        assert!(entry.happened().ran(), "{entry:?}");
        assert_eq!(
            entry.happened().against(),
            [grant],
            "the actual application grant was lost: {entry:?}"
        );
        assert!(
            !entry.happened().by_full_access(),
            "Ask used Full authority: {entry:?}"
        );
        assert!(
            !entry.was_at_full_access(),
            "Ask record carries the wrong policy: {entry:?}"
        );
    }
    assert_eq!(status.happened().from_approval(), None);
    assert_eq!(change.happened().from_approval(), Some(approval));
}

/// With no application grant, both Full operations must record that authority.
#[test]
fn studio_status_and_full_change_record_full_access_without_a_grant_or_approval() {
    let elsewhere =
        Elsewhere::with_the_level("studio-full-provenance", Some(HowCodeRuns::AtFullTrust));
    let mut record = Record::default();
    let transport = Transport::default();
    on_studio_turn(&mut record, &transport, |turn, grants, strings| {
        assert!(grants.is_empty());
        let status = the_agent_asked(&a_message(STATUS), turn, grants, strings, &elsewhere.here());
        assert!(
            matches!(status, ToAnAgent::Did(Done::ApplicationControlled { .. })),
            "Full status: {status:?}"
        );
        let changed = the_agent_asked(
            &a_message(DISABLE),
            turn,
            grants,
            strings,
            &elsewhere.here(),
        );
        assert!(
            matches!(
                changed,
                ToAnAgent::DoneWithoutAsking {
                    did: Done::ApplicationControlled { .. },
                    ..
                }
            ),
            "Full change: {changed:?}"
        );
        assert_eq!(turn.waiting_at(noon()).count(), 0);
        assert!(
            grants.is_empty(),
            "Full execution created a stored application grant"
        );
    });
    assert_eq!(*transport.0.borrow(), ["GetStatus", "DisableMicrophone"]);
    let entries: Vec<_> = record.everything().collect();
    assert_eq!(
        entries.len(),
        2,
        "one status and one Full change: {entries:?}"
    );
    for entry in entries {
        assert!(entry.happened().ran(), "{entry:?}");
        assert!(
            entry.happened().against().is_empty(),
            "Full invented a grant: {entry:?}"
        );
        assert!(
            entry.happened().by_full_access(),
            "the actual Full authority was lost: {entry:?}"
        );
        assert!(
            entry.was_at_full_access(),
            "Full record carries the wrong policy: {entry:?}"
        );
        assert_eq!(
            entry.happened().from_approval(),
            None,
            "Full invented approval: {entry:?}"
        );
    }
}
