//! A change to the printers, from the moment a person approves it — or chooses
//! it in Settings — to the printing service's side of the wire, through the
//! privileged broker's real door, and every way it is stopped on the way.
//!
//! Task 2 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`:
//!
//! - *the broker's printer verbs — add a printer `alo-printing` found, remove
//!   one, set the default — take `alo-printing`'s own types, and configure the
//!   rented print system without a free-form URI or driver name;*
//! - *the agent's printers, solved reaches configuration only through these
//!   verbs, each proposed and approved;*
//! - *and a person does the same by hand in Settings through the same verbs.*
//!
//! Everything here is real except two things, and both are said: the printing
//! service is `alo-printing`'s own loopback fixture, which speaks the protocol
//! and remembers every request, and the door is handed to whoever runs the test
//! rather than to the person's login. The broker, its key hand-over, its
//! socket, its record, its composite carriers and this crate's road are the ones a machine
//! runs.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

#[path = "../../alo-printing/tests/serving/mod.rs"]
mod serving;

#[path = "../../alo-brokerd/tests/with_printers/mod.rs"]
mod with_printers;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

use alo_broker::handing_over::hand_over_a_fresh_key;
use alo_broker::listening::Listening;
use alo_broker::{Answer, BY_HAND, Broker, Door, Identity, SystemVerb, our_group, our_user};
use alo_capability::{
    Approvals, Arg, Authorised, Effect, Given, Grantee, Grants, NotAuthorised, Proposal, Requires,
    Takes, Verb, Verbs,
};
use alo_changing_printers::by_hand::picked;
use alo_changing_printers::words::PRINTER;
use alo_changing_printers::{
    ADD_PRINTER, Change, NotChanged, REMOVE_PRINTER, SET_DEFAULT_PRINTER, TheBroker,
    ThisMachinesPrinters, carry_out_approved, carry_out_by_hand, changed_said, printer_verbs,
};
use alo_printing::ipp::{Group, Message};
use alo_printing::{Called, PrintingService};
use alo_record::{AtTheBroker, Happened, Record};
use alo_strings::{Strings, Vocabulary};

use serving::{
    ADD_MODIFY_PRINTER, Answer as Served, DELETE_PRINTER, GET_DEVICES, GET_PRINTERS, SET_DEFAULT,
    Serving, a_printing_service, nothing_listening, ok, status, with_device, with_printer,
};

/// Where the Canon on a cable was found.
const THE_CANON_FOUND: &str = "ippusb://Canon%20G3560/?serial=X";

/// The queue the Brother on the office network is set up under.
const THE_BROTHER_SET_UP: &str = "alo-brother-hl-l2350dw-series-5b1f0c3e9a7d2468";

/// The queue the Canon is set up under.
const THE_CANON_SET_UP: &str = "alo-canon-pixma-g3560-0f1e2d3c4b5a6978";

/// A moment, now: the broker checks a token's age against its own clock.
fn now() -> SystemTime {
    SystemTime::now()
}

/// The words, as a shell holds them.
fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_printing::declare_into(&mut vocabulary).unwrap();
    alo_changing_printers::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The agent a proposal is made under.
fn the_agent() -> Grantee {
    Grantee::named("alo")
}

/// A group this test may hand a file to that is not root's.
fn a_group() -> u32 {
    if our_group() == 0 { 4242 } else { our_group() }
}

/// What an office's printing service answers: two printers found — a Brother on
/// the network and a Canon on a cable — and both set up. `add_answer` is what it
/// says to adding one.
fn an_office(add_answer: u16) -> impl Fn(&Message) -> Served + Send + 'static {
    move |request| match request.code() {
        GET_DEVICES => Served::Ipp(with_device(
            with_device(
                ok(),
                "dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000",
                "Brother HL-L2350DW series",
            ),
            THE_CANON_FOUND,
            "Canon PIXMA G3560",
        )),
        GET_PRINTERS => Served::Ipp(with_printer(
            with_printer(
                ok(),
                THE_BROTHER_SET_UP,
                "dnssd://Brother%20HL-L2350DW._ipp._tcp.local/?uuid=e3248000",
                "Brother HL-L2350DW series",
            ),
            THE_CANON_SET_UP,
            "ippusb://Canon%20G3560/?serial=X",
            "Canon PIXMA G3560",
        )),
        ADD_MODIFY_PRINTER => Served::Ipp(status(add_answer)),
        _ => Served::Ipp(ok()),
    }
}

/// A broker of the test's own, answering `how_many` requests at a real door
/// with the real carrier against `service`, and handing back its record.
struct Running {
    /// Where its door and key are.
    reached: TheBroker,
    /// The broker's thread, which hands back what it wrote down.
    answering: JoinHandle<Record>,
}

/// Start a broker in a directory of its own.
fn a_broker(what: &str, service: PrintingService, how_many: usize) -> Running {
    let here = std::env::temp_dir().join(format!(
        "alo-changing-printers-{}-{what}",
        std::process::id()
    ));
    drop(std::fs::remove_dir_all(&here));
    std::fs::create_dir_all(&here).unwrap();
    let (door, key) = (here.join("door.sock"), here.join("approving.key"));
    let approving = hand_over_a_fresh_key(&key, a_group()).unwrap();
    let listening = Listening::at(&door, a_group()).unwrap();
    let answering = std::thread::spawn(move || {
        let mut broker = Broker::new(
            Door::handed_to(our_user()),
            approving,
            Record::default(),
            with_printers::carriers(service),
        );
        for _ in 0..how_many {
            listening.answer_one(&mut broker).unwrap();
        }
        broker.recording().clone()
    });
    Running {
        reached: TheBroker::at(door, key, our_user()),
        answering,
    }
}

/// What the broker wrote down: each entry's approval and refusal.
fn kept(record: &Record) -> Vec<(Option<u64>, Option<AtTheBroker>)> {
    record
        .everything()
        .map(|entry| match entry.happened() {
            Happened::Brokered {
                from_approval,
                refused,
                ..
            } => (*from_approval, *refused),
            other => panic!("the broker wrote down something else: {other:?}"),
        })
        .collect()
}

/// Propose `verb` for the printer called `called`, approve it, and redeem the
/// approval: the authority a turn hands on, and the approval's number.
fn approved(verb: &str, called: &str) -> (Authorised, u64) {
    let grants = Grants::default();
    let call = printer_verbs()
        .unwrap()
        .call(verb, &[(PRINTER, Given::text(called))])
        .unwrap();
    let mut approvals = Approvals::default();
    let id = approvals.propose(
        Proposal::checked(
            &call,
            &the_agent(),
            &grants,
            now(),
            Duration::from_secs(300),
        )
        .unwrap(),
    );
    let authorised = approvals
        .approve(id, now())
        .unwrap()
        .redeem(&grants, now())
        .unwrap();
    assert!(
        approvals.approve(id, now()).is_err(),
        "one approval was answered twice"
    );
    (authorised, id.as_u64())
}

/// The value of one attribute of the first request with this operation.
fn asked_with(serving: &Serving, operation: u16, attribute: &str) -> Option<String> {
    serving
        .heard()
        .iter()
        .find(|request| request.code() == operation)
        .and_then(|request| {
            request
                .attribute(Group::Operation, attribute)
                .or_else(|| request.attribute(Group::Printer, attribute))
        })
        .and_then(|value| value.text().map(str::to_owned))
}

/// **An agent's proposal, approved, sets up exactly the printer the person
/// approved by name — through the broker, once.** The agent's own call is
/// refused as a change that waits and reaches nothing; the approved one finds
/// the printer, crosses the door, is written down under the same approval
/// number, and the broker's carrier finds the printer again and sets that one
/// up. The printing service is told the address it reported, and nothing a
/// model wrote.
#[test]
fn an_approved_proposal_sets_up_exactly_that_printer_through_the_broker() {
    let serving = a_printing_service(an_office(0));
    let strings = in_english();

    let call = printer_verbs()
        .unwrap()
        .call(ADD_PRINTER, &[(PRINTER, Given::text("Canon PIXMA G3560"))])
        .unwrap();
    assert_eq!(
        call.sentence(&strings).text(),
        "set up the printer Canon PIXMA G3560, so this machine can print on it"
    );
    let by_itself = Authorised::read(&call, &the_agent(), &Grants::default(), now()).unwrap_err();
    assert!(matches!(by_itself.why(), NotAuthorised::ChangeWaits { .. }));
    assert!(
        serving.operations().is_empty(),
        "an unapproved call reached the printing service"
    );

    let running = a_broker("add", serving.service(), 1);
    let (authorised, approval) = approved(ADD_PRINTER, "Canon PIXMA G3560");
    let verb = carry_out_approved(authorised, &serving.service(), &running.reached, now()).unwrap();

    assert_eq!(
        verb,
        SystemVerb::AddPrinter(Identity::of_what_was_reported(THE_CANON_FOUND.as_bytes()))
    );
    assert_eq!(
        serving.operations(),
        [GET_DEVICES, GET_DEVICES, ADD_MODIFY_PRINTER, SET_DEFAULT],
        "found by this crate, found again by the broker, set up once"
    );
    assert_eq!(
        asked_with(&serving, ADD_MODIFY_PRINTER, "device-uri").as_deref(),
        Some(THE_CANON_FOUND)
    );
    assert_eq!(
        asked_with(&serving, ADD_MODIFY_PRINTER, "ppd-name").as_deref(),
        Some("everywhere"),
        "the printer describes itself; nobody chose a driver"
    );
    assert_eq!(
        kept(&running.answering.join().unwrap()),
        [(Some(approval), None)]
    );
    assert_eq!(
        changed_said(
            &verb,
            &Called::Named("Canon PIXMA G3560".to_owned()),
            &strings
        )
        .unwrap()
        .text(),
        "Canon PIXMA G3560 is set up, and this machine prints on it"
    );
}

/// **Removing and choosing the printer this machine prints on are the same
/// road**: each finds the printer set up under the approved name, crosses the
/// door once, and the service is asked about exactly that queue.
#[test]
fn removing_and_choosing_a_printer_go_through_the_broker_to_exactly_that_printer() {
    let strings = in_english();
    for (verb_name, called, queue, operation, said) in [
        (
            REMOVE_PRINTER,
            "Brother HL-L2350DW series",
            THE_BROTHER_SET_UP,
            DELETE_PRINTER,
            "Brother HL-L2350DW series is removed from this machine. Nothing more can be printed \
             on it until it is set up again",
        ),
        (
            SET_DEFAULT_PRINTER,
            "Canon PIXMA G3560",
            THE_CANON_SET_UP,
            SET_DEFAULT,
            "This machine prints on Canon PIXMA G3560 from now on",
        ),
    ] {
        let serving = a_printing_service(an_office(0));
        let running = a_broker(verb_name, serving.service(), 1);
        let (authorised, approval) = approved(verb_name, called);
        let verb =
            carry_out_approved(authorised, &serving.service(), &running.reached, now()).unwrap();

        assert_eq!(
            serving.operations(),
            [GET_PRINTERS, GET_PRINTERS, operation]
        );
        assert_eq!(
            asked_with(&serving, operation, "printer-uri"),
            Some(format!("ipp://localhost/printers/{queue}"))
        );
        assert_eq!(
            kept(&running.answering.join().unwrap()),
            [(Some(approval), None)]
        );
        assert_eq!(
            changed_said(&verb, &Called::Named(called.to_owned()), &strings)
                .unwrap()
                .text(),
            said
        );
    }
}

/// **A person's own choice in Settings is the same verb through the same
/// door.** The printers Settings lists are the ones the printing service
/// reports; the one a person picks becomes exactly the verb an agent's approved
/// proposal became, and the broker writes it down as a change made by hand.
#[test]
fn a_person_in_settings_changes_printers_through_the_same_verbs() {
    let serving = a_printing_service(an_office(0));
    let service = serving.service();

    let found = service.found().unwrap();
    let canon = found
        .iter()
        .find(|printer| printer.called() == &Called::Named("Canon PIXMA G3560".to_owned()))
        .unwrap();
    let chose_by_hand = picked(Change::Add, canon);
    let (authorised, _) = approved(ADD_PRINTER, "Canon PIXMA G3560");
    let the_agents = alo_changing_printers::chosen(
        &alo_changing_printers::approved(&authorised).unwrap(),
        &service,
    )
    .unwrap();
    assert_eq!(
        chose_by_hand, the_agents,
        "two roads to one printer are two verbs"
    );

    let running = a_broker("by-hand", serving.service(), 3);
    carry_out_by_hand(chose_by_hand, canon.called(), &running.reached, now()).unwrap();

    let set_up = service.set_up_here().unwrap();
    let brother = set_up
        .iter()
        .find(|printer| printer.called() == &Called::Named("Brother HL-L2350DW series".to_owned()))
        .unwrap();
    carry_out_by_hand(
        picked(Change::MakeDefault, brother),
        brother.called(),
        &running.reached,
        now(),
    )
    .unwrap();
    carry_out_by_hand(
        picked(Change::Remove, brother),
        brother.called(),
        &running.reached,
        now(),
    )
    .unwrap();

    assert_eq!(
        kept(&running.answering.join().unwrap()),
        [
            (Some(BY_HAND), None),
            (Some(BY_HAND), None),
            (Some(BY_HAND), None)
        ]
    );
    let changes: Vec<u16> = serving
        .operations()
        .into_iter()
        .filter(|operation| [ADD_MODIFY_PRINTER, SET_DEFAULT, DELETE_PRINTER].contains(operation))
        .collect();
    assert_eq!(
        changes,
        [ADD_MODIFY_PRINTER, SET_DEFAULT, SET_DEFAULT, DELETE_PRINTER]
    );
    assert_eq!(
        asked_with(&serving, ADD_MODIFY_PRINTER, "device-uri").as_deref(),
        Some(THE_CANON_FOUND)
    );
    assert_eq!(
        asked_with(&serving, DELETE_PRINTER, "printer-uri"),
        Some(format!("ipp://localhost/printers/{THE_BROTHER_SET_UP}"))
    );
    let chosen_default = serving
        .heard()
        .into_iter()
        .rev()
        .find(|request| request.code() == SET_DEFAULT)
        .and_then(|request| {
            request
                .attribute(Group::Operation, "printer-uri")
                .and_then(|value| value.text().map(str::to_owned))
        });
    assert_eq!(
        chosen_default,
        Some(format!("ipp://localhost/printers/{THE_BROTHER_SET_UP}"))
    );
}

/// **A name no printer has, or two printers share, changes nothing and never
/// reaches the broker.** The person is told which, and sent to Settings.
#[test]
fn a_name_no_printer_has_or_two_share_changes_nothing_and_asks_the_broker_nothing() {
    let strings = in_english();
    let serving = a_printing_service(an_office(0));
    // No broker is running: had anything been asked, the answer would be that
    // nothing makes changes.
    let nobody = TheBroker::at("/nonexistent-door.sock", "/nonexistent.key", our_user());

    let (authorised, _) = approved(ADD_PRINTER, "HP LaserJet Pro M404");
    let refused = carry_out_approved(authorised, &serving.service(), &nobody, now()).unwrap_err();
    assert_eq!(
        refused.said(&strings).text(),
        "No printer called HP LaserJet Pro M404 was found, so nothing was changed. Check that it \
         is switched on and connected, then choose it from the printers in Settings"
    );

    let (authorised, _) = approved(REMOVE_PRINTER, "Canon PIXMA G3560 ");
    let lookalike = a_printing_service(|request| match request.code() {
        GET_PRINTERS => Served::Ipp(with_printer(
            with_printer(
                ok(),
                "alo-canon-a",
                "ippusb://Canon/?serial=A",
                "Canon PIXMA G3560",
            ),
            "alo-canon-b",
            "ipp://192.168.1.40/ipp/print",
            "Canon PIXMA G3560",
        )),
        _ => Served::Ipp(ok()),
    });
    let refused = carry_out_approved(authorised, &lookalike.service(), &nobody, now()).unwrap_err();
    assert_eq!(
        refused,
        NotChanged::MoreThanOneCalled(Called::Named("Canon PIXMA G3560".to_owned()))
    );
    assert_eq!(lookalike.operations(), [GET_PRINTERS]);

    let (authorised, _) = approved(SET_DEFAULT_PRINTER, "Canon PIXMA G3560");
    assert_eq!(
        carry_out_approved(authorised, &nothing_listening(), &nobody, now()),
        Err(NotChanged::PrintingNotAnswering)
    );
    assert_eq!(serving.operations(), [GET_DEVICES]);
}

/// **An approval the broker did not issue the key for changes nothing**: a key
/// planted beside the door proves nothing, the broker refuses and writes the
/// refusal down, and the printing service is never asked to change anything.
#[test]
fn a_token_under_any_key_but_the_brokers_changes_nothing() {
    let serving = a_printing_service(an_office(0));
    let running = a_broker("planted", serving.service(), 1);
    let planted = std::env::temp_dir().join(format!(
        "alo-changing-printers-{}-planted-key",
        std::process::id()
    ));
    drop(std::fs::remove_dir_all(&planted));
    std::fs::create_dir_all(&planted).unwrap();
    let planted_key = planted.join("approving.key");
    let _never_the_brokers = hand_over_a_fresh_key(&planted_key, a_group()).unwrap();
    let (door, _) = paths_of("planted");
    let impostor = TheBroker::at(door, planted_key, our_user());

    let (authorised, _) = approved(ADD_PRINTER, "Canon PIXMA G3560");
    assert_eq!(
        carry_out_approved(authorised, &serving.service(), &impostor, now()),
        Err(NotChanged::ApprovalNotAccepted)
    );
    assert_eq!(serving.operations(), [GET_DEVICES]);
    assert_eq!(
        kept(&running.answering.join().unwrap()),
        [(None, Some(AtTheBroker::NotApproved))]
    );
}

/// The door and key paths of a broker started as `what`.
fn paths_of(what: &str) -> (PathBuf, PathBuf) {
    let here = std::env::temp_dir().join(format!(
        "alo-changing-printers-{}-{what}",
        std::process::id()
    ));
    (here.join("door.sock"), here.join("approving.key"))
}

/// **No broker running, or no key handed over, changes nothing** and says so.
#[test]
fn no_broker_or_no_key_changes_nothing() {
    let strings = in_english();
    let serving = a_printing_service(an_office(0));
    let (authorised, _) = approved(ADD_PRINTER, "Canon PIXMA G3560");
    let nobody = TheBroker::at("/nonexistent-door.sock", "/nonexistent.key", our_user());
    let refused = carry_out_approved(authorised, &serving.service(), &nobody, now()).unwrap_err();
    assert_eq!(refused, NotChanged::NothingMakesChanges);
    assert_eq!(
        refused.said(&strings).text(),
        "The part of this machine that changes its settings is not running, so nothing was \
         changed. Restart the machine"
    );
    assert_eq!(
        carry_out_by_hand(
            SystemVerb::RemovePrinter(Identity::of_what_was_reported(THE_CANON_SET_UP.as_bytes())),
            &Called::Named("Canon PIXMA G3560".to_owned()),
            &nobody,
            now(),
        ),
        Err(NotChanged::NothingMakesChanges)
    );
    assert!(
        serving.operations().iter().all(|operation| ![
            ADD_MODIFY_PRINTER,
            SET_DEFAULT,
            DELETE_PRINTER
        ]
        .contains(operation))
    );
}

/// **A request sent and never answered is not said to have changed nothing.**
/// The broker writes a request down before carrying it out, so a door that took
/// the request and went silent may have made the change: the person is sent to
/// Settings to look, rather than told something nobody knows.
#[test]
fn a_request_never_answered_sends_the_person_to_look_rather_than_saying_nothing_changed() {
    let serving = a_printing_service(an_office(0));
    let (door, key) = paths_of("silent");
    drop(std::fs::remove_dir_all(door.parent().unwrap()));
    std::fs::create_dir_all(door.parent().unwrap()).unwrap();
    let _brokers_key = hand_over_a_fresh_key(&key, a_group()).unwrap();
    let listener = std::os::unix::net::UnixListener::bind(&door).unwrap();
    let silent = std::thread::spawn(move || {
        let (connection, _) = listener.accept().unwrap();
        let mut line = String::new();
        std::io::BufRead::read_line(&mut std::io::BufReader::new(&connection), &mut line).unwrap();
        line
    });

    let (authorised, approval) = approved(SET_DEFAULT_PRINTER, "Canon PIXMA G3560");
    assert_eq!(
        carry_out_approved(
            authorised,
            &serving.service(),
            &TheBroker::at(door, key, our_user()),
            now()
        ),
        Err(NotChanged::CouldNotFinish(Called::Named(
            "Canon PIXMA G3560".to_owned()
        )))
    );
    let asked = silent.join().unwrap();
    assert!(
        asked.starts_with("printers.set-default ") && asked.contains(&format!(" {approval} ")),
        "{asked}"
    );
}

/// **A printer the printing service will not set up, or one that went away
/// between the approval and the change, is written down as not carried out**,
/// and the person is sent to Settings to see how the printers are.
#[test]
fn a_change_the_machine_could_not_finish_is_written_down_and_said() {
    let strings = in_english();

    let serving = a_printing_service(an_office(0x0403));
    let running = a_broker("not-permitted", serving.service(), 1);
    let (authorised, approval) = approved(ADD_PRINTER, "Canon PIXMA G3560");
    let refused =
        carry_out_approved(authorised, &serving.service(), &running.reached, now()).unwrap_err();
    assert_eq!(
        refused.said(&strings).text(),
        "This machine could not finish the change to Canon PIXMA G3560. Open the printers in \
         Settings to see how they are now"
    );
    assert_eq!(
        kept(&running.answering.join().unwrap()),
        [
            (Some(approval), None),
            (Some(approval), Some(AtTheBroker::NotCarried))
        ]
    );

    // Found when the person approved it, gone when the broker looks.
    let looked = Arc::new(AtomicUsize::new(0));
    let counting = Arc::clone(&looked);
    let office = an_office(0);
    let vanishing = a_printing_service(move |request| {
        if request.code() == GET_DEVICES && counting.fetch_add(1, Ordering::SeqCst) > 0 {
            return Served::Ipp(ok());
        }
        office(request)
    });
    let running = a_broker("vanished", vanishing.service(), 1);
    let (authorised, approval) = approved(ADD_PRINTER, "Canon PIXMA G3560");
    assert_eq!(
        carry_out_approved(authorised, &vanishing.service(), &running.reached, now()),
        Err(NotChanged::CouldNotFinish(Called::Named(
            "Canon PIXMA G3560".to_owned()
        )))
    );
    assert_eq!(vanishing.operations(), [GET_DEVICES, GET_DEVICES]);
    assert_eq!(
        kept(&running.answering.join().unwrap()),
        [
            (Some(approval), None),
            (Some(approval), Some(AtTheBroker::NotCarried))
        ]
    );
}

/// **An authority that is not an approved change to the printers is not one**,
/// whatever it was approved for, and it reaches neither the printing service nor
/// the broker.
#[test]
fn an_authority_for_anything_else_changes_no_printer() {
    const LOOKING: alo_strings::Word = alo_strings::Word::saying(
        "testing.looking-at-printers",
        "look at the printer {printer}",
    );
    let mut verbs = Verbs::default();
    verbs
        .declare(
            Verb::checked(
                "look_at_printer",
                LOOKING,
                Effect::Read,
                vec![Arg::taking(PRINTER, LOOKING, Takes::name(127))],
                Requires::nothing_because("a test's own read, which needs no grant at all"),
                LOOKING,
            )
            .unwrap(),
        )
        .unwrap();
    let call = verbs
        .call(
            "look_at_printer",
            &[(PRINTER, Given::text("Canon PIXMA G3560"))],
        )
        .unwrap();
    let read = Authorised::read(&call, &the_agent(), &Grants::default(), now()).unwrap();

    let serving = a_printing_service(an_office(0));
    let nobody = TheBroker::at("/nonexistent-door.sock", "/nonexistent.key", our_user());
    assert_eq!(
        carry_out_approved(read, &serving.service(), &nobody, now()),
        Err(NotChanged::NotAPrinterChange)
    );
    assert!(serving.operations().is_empty());
    assert!(matches!(
        alo_changing_printers::NotChanged::from_the_brokers(Answer::NotKept, &Called::Unshowable),
        Err(NotChanged::NotBeingKept)
    ));
}
