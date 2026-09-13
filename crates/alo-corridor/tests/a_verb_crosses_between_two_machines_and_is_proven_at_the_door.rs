//! *A verb crosses between two machines, and is proven at the door.*
//!
//! Task 8 of the local-network plan, held from outside the crate over real
//! sockets on one host: an agent at reception asks for a verb on the studio
//! at the address discovery measured, the verb carries its proof, and the
//! studio judges the proof before it asks anything else of itself.
//!
//! | The acceptance | The test |
//! |---|---|
//! | a verb reaches the machine it names at the address discovery measured, carrying the verb, its arguments and a proof over exactly those bytes, and nothing else | [`a_verb_reaches_the_machine_it_names_at_the_address_discovery_measured_carrying_exactly_the_verb_its_arguments_and_a_proof`] |
//! | a verb from a stranger presenting a paired machine's identity is refused before any grant is asked, the record staying empty | [`a_verb_from_a_stranger_presenting_a_paired_machines_identity_is_refused_before_any_grant_is_asked`] |
//! | a verb replayed from an earlier exchange is refused, the record staying empty | [`a_verb_replayed_from_an_earlier_exchange_is_refused_and_the_record_gains_nothing`] |
//! | a verb from a pairing since revoked is refused, the record staying empty | [`a_verb_from_a_pairing_since_revoked_is_refused_and_the_record_stays_empty`] |
//! | a verb the receiving machine's person has not granted is refused as *not granted here*, beside the same verb granted there and run | [`a_verb_not_granted_here_is_refused_as_such_beside_the_same_verb_granted_here_and_run`] |
//! | every answer sent back travels holding a `Departing`, and there is no road to the wire without one | [`every_answer_sent_back_travels_holding_a_departing_and_there_is_no_road_to_the_wire_without_one`] |
//! | the receiving machine's record names the origin machine, and the asking machine's record names where the verb went | [`each_machines_record_names_the_other`] |
//! | the second message of an open turn carries a fresh proof, and the first one replayed is refused | [`the_second_message_of_an_open_turn_carries_a_fresh_proof_and_the_first_one_replayed_is_refused`] |
//!
//! And beside them: a change from a paired machine waits for the person at
//! the studio and runs once they approve it there, and a rule at the studio
//! that says nothing leaves holds the answer back.
//!
//! # Two machines, on one host
//!
//! The studio is a real `Machine` with a real folder, a real record and a
//! real indicator, in a thread of its own; reception is a `Crossing` in the
//! test's thread with a record and an indicator of its own. Discovery is done
//! honestly in every test — the studio answers a datagram that it exists at
//! its port, and the wire dials what that measured. **What no test here shows
//! is two chassis**, as every report of this plan has said.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::io::{BufRead as _, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

use alo_asking::THE_PROOF_HEADER;
use alo_capability::{Given, Grant, Grantee, Grants, Reach};
use alo_corridor::{
    AtTheDoor, Crossed, Crossing, Doorway, Heard, NotCrossed, Receiving, THE_READ_PATH, WentBack,
};
use alo_egress::{Destination, EgressPolicy, Indicator};
use alo_files::{OnThisMachine, Reaching, Resolving as _};
use alo_nearby::{
    Answering, Deliberating, Found, Keying, Looking, MachineId, MayAskIts, NotProven, Pairing,
    Pairings, Presence, Proof, Proposal, Proven, Seen, Side,
};
use alo_record::{Asking, Entry, Happened, Only, Record};
use alo_strings::{Strings, Vocabulary};
use alo_turn::{Bounding, Doing, Done, Machine, NoBoundary};

/// The machine at reception, whose agent asks.
fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The studio machine, which is asked.
fn the_studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// Somebody on the office WiFi who read reception's identity off a discovery
/// packet.
fn a_stranger() -> MachineId {
    MachineId::read("99998888777766665555444433332222").unwrap()
}

/// What the person at reception called the studio.
const THE_STUDIO: &str = "the studio machine";

/// What the person at the studio called reception.
const RECEPTION: &str = "the reception machine";

/// Reception's principal on the studio's grants.
const RECEPTIONS_PRINCIPAL: &str = "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0";

/// A moment to reason from.
fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// The moment the `nth` message of a test is sent and judged at: one second
/// apart, so each proof is fresh and every one is within the window.
fn the_moment_of(nth: usize) -> SystemTime {
    a_moment() + Duration::from_secs(u64::try_from(nth).unwrap())
}

/// An hour, for a turn, a grant and a change.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// Two machines paired the way two machines are: the asking one's row first,
/// the asked one's second.
fn paired_between(asking: MachineId, asked: MachineId) -> (Pairing, Pairing) {
    let at_asking = Keying::fresh().unwrap();
    let at_asked = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        asking,
        asked,
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        at_asking.offer().clone(),
    )
    .unwrap();
    let asked_side = Deliberating::asked(proposal.clone(), at_asked);
    let asking_side = Deliberating::asking(proposal, at_asking)
        .unwrap()
        .answered_with(asked_side.answered().unwrap().clone())
        .unwrap();
    assert_eq!(asking_side.code(), asked_side.code());
    (
        asking_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(a_moment())
            .unwrap(),
        asked_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(a_moment())
            .unwrap(),
    )
}

/// Reception and the studio, each holding its row: reception's pairings
/// first, the studio's second.
fn paired() -> (Pairings, Pairings) {
    let (on_reception, on_studio) = paired_between(reception(), the_studio());
    let mut at_reception = Pairings::none();
    at_reception.keep(on_reception);
    let mut at_studio = Pairings::none();
    at_studio.keep(on_studio);
    (at_reception, at_studio)
}

/// Every word the studio has loaded, so that a missing string is a failing
/// test and not a passing one.
fn everything_the_studio_says() -> Vocabulary {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_keeping::declare_into(&mut vocabulary).unwrap();
    alo_models::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_answering::declare_into(&mut vocabulary).unwrap();
    alo_asking::declare_into(&mut vocabulary).unwrap();
    alo_nearby::words::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    alo_corridor::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships — `alo_turn::bounding` says why no library here has one, and why a
/// test writes these lines where its reader can see them.
struct NothingIsBounded;

impl Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }
}

/// A folder of the studio's own with one file in it, both resolved.
fn a_folder_with_an_invoice(what: &str) -> (PathBuf, PathBuf) {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-corridor-wire-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let folder = OnThisMachine.real(&folder).unwrap().into_path_buf();
    let invoice = folder.join("march.pdf");
    fs::write(&invoice, "March, 4180.00").unwrap();
    (folder, invoice)
}

/// A path as a verb's argument arrives: text.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// What the studio's person called reception, and nothing else.
fn naming(machine: &MachineId) -> Option<String> {
    (*machine == reception()).then(|| RECEPTION.to_owned())
}

/// Discovery, honestly: `machine` answers on a datagram socket that it exists
/// at `port`, this host asks who is here, and what is found carries the
/// address the answer came from — which is what the wire then dials.
fn found_by_discovery(machine: MachineId, port: u16) -> Found {
    let there = UdpSocket::bind("127.0.0.1:0").unwrap();
    there
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let at = there.local_addr().unwrap();
    let answering = Answering::on(there, Presence::of(machine.clone(), port));
    let answered = thread::spawn(move || answering.answer_one());

    let here = UdpSocket::bind("127.0.0.1:0").unwrap();
    here.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let looking = Looking::from(here);
    looking.ask(at).unwrap();
    let found = looking.found(Duration::from_secs(5)).unwrap();
    assert!(answered.join().unwrap().unwrap().is_some());
    let one = found
        .into_iter()
        .find(|one| one.machine == machine)
        .unwrap();
    assert_eq!(one.where_it_answers(), SocketAddr::new(at.ip(), port));
    one
}

/// What the studio was: its record, what it heard, when each turn it held
/// ended, whether its indicator was quiet at the end, and its folder.
struct Studio {
    record: Record,
    heard: Vec<Heard>,
    turn_ends: Vec<Option<SystemTime>>,
    quiet_at_the_end: bool,
    folder: PathBuf,
    invoice: PathBuf,
}

/// How the studio is set up for one test.
struct StudioSetting {
    /// Its pairings.
    pairings: Pairings,
    /// Who its person granted the folder to, if anybody.
    granted_to: Option<&'static str>,
    /// The egress rule in force there.
    policy: EgressPolicy,
    /// Before which message, if any, its person revokes the pairing with
    /// reception.
    revoking_before: Option<usize>,
    /// Whether its person approves every change put to them, after the
    /// message that put it.
    approving: bool,
}

impl StudioSetting {
    /// A studio paired with reception, granting reception the folder, with a
    /// rule that keeps things in the building.
    fn granting_reception(pairings: Pairings) -> Self {
        Self {
            pairings,
            granted_to: Some(RECEPTIONS_PRINCIPAL),
            policy: EgressPolicy::InTheBuilding,
            revoking_before: None,
            approving: false,
        }
    }
}

/// The studio, hearing exactly `how_many` connections, each judged at the
/// moment its number names, and reporting what it was.
///
/// Answers where discovery found it — the studio's identity, at the port
/// the listener was bound to — and the handle to join.
fn a_studio(
    what: &str,
    how_many: usize,
    setting: StudioSetting,
) -> (Found, PathBuf, JoinHandle<Studio>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (folder, invoice) = a_folder_with_an_invoice(what);
    let studio_folder = folder.clone();
    let handle = thread::spawn(move || {
        let StudioSetting {
            mut pairings,
            granted_to,
            policy,
            revoking_before,
            approving,
        } = setting;
        let strings = Strings::of(everything_the_studio_says());
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let mut heard = Vec::new();
        let mut turn_ends = Vec::new();
        {
            let mut machine = Machine::carrying_out_file_verbs(
                &strings,
                &OnThisMachine,
                &mut bounding,
                &mut indicator,
                &mut record,
            )
            .unwrap();
            let mut doorway = Doorway::at(the_studio(), &mut machine, hour(), hour()).unwrap();
            let mut grants = Grants::default();
            if let Some(whom) = granted_to {
                grants.grant(
                    Grant::checked(
                        whom,
                        Reach::Folder(studio_folder.clone()),
                        a_moment(),
                        hour(),
                    )
                    .unwrap(),
                );
            }
            let receiving = Receiving::on(listener);
            for nth in 0..how_many {
                if revoking_before == Some(nth) {
                    assert!(pairings.revoke(&reception()));
                }
                let arrived = receiving.accept_one().unwrap();
                let now = the_moment_of(nth);
                let what_happened = arrived
                    .considered(&mut doorway, &pairings, &mut grants, &naming, &policy, now)
                    .unwrap();
                heard.push(what_happened);
                turn_ends.push(doorway.turn().map(|turn| turn.turning().ends()));
                if approving {
                    let waiting: Vec<_> = doorway
                        .turn()
                        .map(|turn| turn.waiting_at(now).map(|change| change.id).collect())
                        .unwrap_or_default();
                    for id in waiting {
                        doorway
                            .turn()
                            .unwrap()
                            .approving(id, &pairings, &grants, now)
                            .unwrap();
                    }
                }
            }
            doorway.ending(&mut grants);
        }
        Studio {
            record,
            heard,
            turn_ends,
            quiet_at_the_end: indicator.is_quiet(),
            folder: studio_folder,
            invoice,
        }
    });
    (found_by_discovery(the_studio(), port), folder, handle)
}

/// Reception's side of one crossing: what came back, and what reception
/// wrote down about it.
struct AtReception {
    outcome: Result<Crossed, NotCrossed>,
    record: Record,
    quiet_at_the_end: bool,
}

/// One read of `folder` put to the studio from reception, as `@files`, at
/// the moment `nth` names, with reception's record and indicator walked the
/// way a turn on reception would walk them.
fn reading_from_reception(
    pairings: &Pairings,
    here: &MachineId,
    found: &Found,
    folder: &Path,
    nth: usize,
) -> AtReception {
    crossing_from_reception(pairings, here, found, nth, |crossing, indicator| {
        crossing.reading(
            "list_folder",
            &[("folder", as_given(folder))],
            &EgressPolicy::InTheBuilding,
            indicator,
            the_moment_of(nth),
        )
    })
}

/// One crossing from reception, whatever `doing` puts.
fn crossing_from_reception(
    pairings: &Pairings,
    here: &MachineId,
    found: &Found,
    nth: usize,
    doing: impl FnOnce(Crossing<'_>, &mut Indicator) -> Result<Crossed, NotCrossed>,
) -> AtReception {
    let files = Grantee::named("@files");
    let crossing = Crossing::to(
        pairings,
        here,
        found,
        THE_STUDIO,
        &files,
        the_moment_of(nth),
    )
    .unwrap();
    assert_eq!(crossing.where_it_would_connect(), found.where_it_answers());
    let mut indicator = Indicator::default();
    let mut record = Record::default();
    let outcome = doing(crossing, &mut indicator);
    // What a turn on reception writes: the departure, whatever came back.
    match &outcome {
        Ok(crossed) => {
            assert_eq!(
                indicator.showing().len(),
                1,
                "the verb left with nothing showing"
            );
            record.keep(Entry::left(crossed.departing()));
        }
        Err(NotCrossed::Left(left)) => {
            assert_eq!(
                indicator.showing().len(),
                1,
                "the verb left with nothing showing"
            );
            record.keep(Entry::left(left.departing()));
        }
        Err(_) => assert!(
            indicator.is_quiet(),
            "nothing left and something is showing"
        ),
    }
    AtReception {
        outcome,
        record,
        quiet_at_the_end: indicator.is_quiet(),
    }
}

/// A read that came back answered, ended on the indicator.
fn answered(at_reception: AtReception) -> alo_corridor::Answered {
    let mut indicator = Indicator::default();
    match at_reception.outcome {
        // The line is ended on a fresh indicator here only to spend the
        // departure; the one it was shown on was checked in the fixture.
        Ok(crossed) => crossed.ended(&mut indicator),
        Err(why) => unreachable!("the read was not answered: {why:?}"),
    }
}

/// A read that came back refused at the studio's door, and the word.
fn turned_away(at_reception: AtReception) -> AtTheDoor {
    match at_reception.outcome {
        Err(NotCrossed::Left(left)) => match left.why() {
            WentBack::AtTheDoor(at_the_door) => *at_the_door,
            other => unreachable!("not the door's word: {other:?}"),
        },
        other => unreachable!("the read was not turned away at the door: {other:?}"),
    }
}

/// A recording of one request, made by standing in the road: what reception
/// sent, byte for byte, passed on to the studio as it was and the answer
/// carried back.
fn recording_one_request(studio_at: SocketAddr) -> (u16, JoinHandle<Vec<u8>>) {
    let recorder = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = recorder.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (mut from_reception, _) = recorder.accept().unwrap();
        let bytes = read_a_whole_request(&from_reception);
        let mut to_the_studio = TcpStream::connect(studio_at).unwrap();
        to_the_studio.write_all(&bytes).unwrap();
        let mut answer = Vec::new();
        to_the_studio.read_to_end(&mut answer).unwrap();
        from_reception.write_all(&answer).unwrap();
        bytes
    });
    (port, handle)
}

/// One request off a socket, head and body, exactly.
fn read_a_whole_request(stream: &TcpStream) -> Vec<u8> {
    let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
    let mut bytes = Vec::new();
    let mut length = 0_usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }
        if let Some(how_long) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            length = how_long.trim().parse().unwrap_or(0);
        }
        bytes.extend_from_slice(line.as_bytes());
        if line == "\r\n" {
            break;
        }
    }
    let mut body = vec![0_u8; length];
    reader.read_exact(&mut body).unwrap();
    bytes.extend_from_slice(&body);
    bytes
}

/// Send recorded bytes to the studio again, exactly, and read the status
/// line and body that come back.
fn replayed(studio_at: SocketAddr, bytes: &[u8]) -> (String, String) {
    let mut again = TcpStream::connect(studio_at).unwrap();
    again.write_all(bytes).unwrap();
    let mut reply = String::new();
    again.read_to_string(&mut reply).unwrap();
    let (head, body) = reply.split_once("\r\n\r\n").unwrap();
    (
        head.lines().next().unwrap().to_owned(),
        body.trim().to_owned(),
    )
}

/// How many entries a record holds that say something left.
fn what_left(record: &Record) -> usize {
    record
        .answering(&Asking::anything().only(Only::Egress))
        .count()
}

/// **A verb reaches the machine it names at the address discovery measured,
/// carrying exactly the verb, its typed arguments and a proof over those
/// bytes — and nothing else.** A recorder stands in the road at the address
/// discovery measured, reads the wire, and the test refuses any header or
/// field not on the list; the proof in the header verifies, with the
/// studio's own row, over exactly the body bytes; and the answer comes back.
#[test]
fn a_verb_reaches_the_machine_it_names_at_the_address_discovery_measured_carrying_exactly_the_verb_its_arguments_and_a_proof()
 {
    let (at_reception, at_studio) = paired();
    let (studio_found, folder, studio) = a_studio(
        "exactly-the-verb",
        1,
        StudioSetting::granting_reception(at_studio.clone()),
    );
    let (recorder_port, recording) = recording_one_request(studio_found.where_it_answers());
    let in_the_road = found_by_discovery(the_studio(), recorder_port);

    let outcome = reading_from_reception(&at_reception, &reception(), &in_the_road, &folder, 0);
    let bytes = recording.join().unwrap();
    let studio = studio.join().unwrap();

    // The answer came back: the studio's folder, with its one file in it.
    let answered = answered(outcome);
    assert!(
        matches!(
            answered.did(),
            Some(alo_protocol::Done::Listed { things, .. }) if things.len() == 1
        ),
        "{answered:?}"
    );
    assert!(matches!(
        studio.heard.as_slice(),
        [Heard::Answered { from, waits: None }] if *from == reception()
    ));

    // What crossed, read off the wire.
    let text = String::from_utf8(bytes).unwrap();
    let (head, body) = text.split_once("\r\n\r\n").unwrap();
    let mut lines = head.lines();
    assert_eq!(
        lines.next().unwrap(),
        format!("POST {THE_READ_PATH} HTTP/1.1")
    );
    let mut headers: Vec<String> = lines
        .map(|line| line.split_once(':').unwrap().0.to_ascii_lowercase())
        .collect();
    headers.sort_unstable();
    assert_eq!(
        headers,
        [
            THE_PROOF_HEADER,
            "connection",
            "content-length",
            "content-type",
            "host"
        ],
        "a header not on the list crossed: {head}"
    );
    let proof_line = head
        .lines()
        .find_map(|line| {
            line.split_once(':').and_then(|(name, value)| {
                (name.eq_ignore_ascii_case(THE_PROOF_HEADER)).then(|| value.trim().to_owned())
            })
        })
        .unwrap();

    // The body is exactly the verb and its arguments.
    let object: serde_json::Value = serde_json::from_str(body).unwrap();
    let mut keys: Vec<&str> = object
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["given", "verb"],
        "a field not on the list crossed: {body}"
    );
    assert_eq!(object.get("verb").unwrap(), "list_folder");
    let given = object.get("given").unwrap().as_array().unwrap();
    for argument in given {
        let mut keys: Vec<&str> = argument
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            ["is", "named"],
            "an argument said more than a name and a value: {body}"
        );
    }
    assert_eq!(given.len(), 1);
    assert!(
        !body.contains("machine"),
        "a name or an identity was in the body: {body}"
    );

    // And the proof is over exactly those bytes, with the key only the pairing holds.
    let proof = Proof::read(&proof_line).unwrap();
    assert_eq!(proof.from(), &reception());
    assert_eq!(proof.to(), &the_studio());
    assert!(
        Proven::checked(
            &at_studio,
            &the_studio(),
            &proof,
            body.as_bytes(),
            the_moment_of(0),
            &mut Seen::nothing()
        )
        .is_ok()
    );
    assert!(
        Proven::checked(
            &at_studio,
            &the_studio(),
            &proof,
            format!("{body} ").as_bytes(),
            the_moment_of(0),
            &mut Seen::nothing()
        )
        .is_err(),
        "the proof is not over exactly the bytes that crossed"
    );
}

/// **A verb from a stranger presenting a paired machine's identity is refused
/// before any grant is asked, and the record stays empty.** The stranger has
/// the address, the shape of the request and reception's `MachineId` —
/// everything the network gives away — and a key of its own from a pairing
/// the studio's person never made. The studio's person *had* granted
/// reception the folder, and it is not consulted.
#[test]
fn a_verb_from_a_stranger_presenting_a_paired_machines_identity_is_refused_before_any_grant_is_asked()
 {
    let (_, at_studio) = paired();
    let (studio_found, folder, studio) = a_studio(
        "a-stranger",
        1,
        StudioSetting::granting_reception(at_studio),
    );

    // The stranger's own row names the studio, so its crossing opens — on
    // its own machine, whose pairings nobody at the studio agreed to — and
    // it presents reception's identity as the sender.
    let (strangers_row, _) = paired_between(a_stranger(), the_studio());
    let mut strangers_pairings = Pairings::none();
    strangers_pairings.keep(strangers_row);
    let outcome =
        reading_from_reception(&strangers_pairings, &reception(), &studio_found, &folder, 0);
    let studio = studio.join().unwrap();

    assert_eq!(
        turned_away(outcome),
        AtTheDoor::NotProven(NotProven::NotFromThatMachine)
    );
    assert!(matches!(
        studio.heard.as_slice(),
        [Heard::TurnedAwayAtTheDoor(AtTheDoor::NotProven(
            NotProven::NotFromThatMachine
        ))]
    ));
    assert_eq!(studio.record.len(), 0, "{:?}", studio.record);
    assert!(studio.quiet_at_the_end);
}

/// **A verb replayed from an earlier exchange is refused, and the record
/// gains nothing.** Whoever recorded reception's bytes sends them again,
/// exactly; the studio has seen that proof and answers *already used*.
#[test]
fn a_verb_replayed_from_an_earlier_exchange_is_refused_and_the_record_gains_nothing() {
    let (at_reception, at_studio) = paired();
    let (studio_found, folder, studio) =
        a_studio("a-replay", 2, StudioSetting::granting_reception(at_studio));
    let (recorder_port, recording) = recording_one_request(studio_found.where_it_answers());
    let in_the_road = found_by_discovery(the_studio(), recorder_port);

    let outcome = reading_from_reception(&at_reception, &reception(), &in_the_road, &folder, 0);
    assert!(answered(outcome).did().is_some());
    let bytes = recording.join().unwrap();

    let (status, word) = replayed(studio_found.where_it_answers(), &bytes);
    let studio = studio.join().unwrap();

    assert!(status.starts_with("HTTP/1.1 403"), "{status}");
    assert_eq!(
        AtTheDoor::off_the_wire(&word),
        Some(AtTheDoor::NotProven(NotProven::AlreadySeen))
    );
    assert!(matches!(
        studio.heard.as_slice(),
        [
            Heard::Answered { .. },
            Heard::TurnedAwayAtTheDoor(AtTheDoor::NotProven(NotProven::AlreadySeen))
        ]
    ));
    // One read ran and its answer left; the replay wrote nothing.
    assert_eq!(studio.record.len(), 2, "{:?}", studio.record);
    assert_eq!(what_left(&studio.record), 1);
}

/// **A verb from a pairing since revoked is refused, and the record stays
/// empty.** The studio's person undid the pairing a moment before the verb
/// arrived, with reception's key still in reception's hand: the key is on
/// the row, and the row is gone.
#[test]
fn a_verb_from_a_pairing_since_revoked_is_refused_and_the_record_stays_empty() {
    let (at_reception, at_studio) = paired();
    let (studio_found, folder, studio) = a_studio(
        "revoked",
        1,
        StudioSetting {
            revoking_before: Some(0),
            ..StudioSetting::granting_reception(at_studio)
        },
    );

    let outcome = reading_from_reception(&at_reception, &reception(), &studio_found, &folder, 0);
    let studio = studio.join().unwrap();

    assert_eq!(
        turned_away(outcome),
        AtTheDoor::NotProven(NotProven::NotWithThatMachine)
    );
    assert_eq!(studio.record.len(), 0, "{:?}", studio.record);
    assert!(matches!(
        studio.heard.as_slice(),
        [Heard::TurnedAwayAtTheDoor(AtTheDoor::NotProven(
            NotProven::NotWithThatMachine
        ))]
    ));
}

/// **A verb the receiving machine's person has not granted is refused as
/// *not granted here***, in a sentence at reception that names the studio and
/// sends nobody to a folder picker — beside the same verb, granted at the
/// studio, which runs. The studio's record keeps the refusal with reception
/// named, and its sentence says *on this machine*.
#[test]
fn a_verb_not_granted_here_is_refused_as_such_beside_the_same_verb_granted_here_and_run() {
    let strings = Strings::of(everything_the_studio_says());
    let (at_reception, at_studio) = paired();

    let (ungranting_found, folder, ungranting) = a_studio(
        "not-granted",
        1,
        StudioSetting {
            granted_to: None,
            ..StudioSetting::granting_reception(at_studio.clone())
        },
    );
    let outcome =
        reading_from_reception(&at_reception, &reception(), &ungranting_found, &folder, 0);
    let ungranting = ungranting.join().unwrap();

    let said = match &outcome.outcome {
        Err(refused) => refused.said(THE_STUDIO, &strings),
        Ok(_) => unreachable!("a verb nobody granted at the studio ran there"),
    };
    assert_eq!(turned_away(outcome), AtTheDoor::NotGrantedThere);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains(THE_STUDIO), "{said}");
    assert!(said.text().contains("not been granted"), "{said}");
    assert!(!said.text().contains("picking a folder"), "{said}");
    assert!(matches!(
        ungranting.heard.as_slice(),
        [Heard::Refused { from, at_the_door: AtTheDoor::NotGrantedThere }] if *from == reception()
    ));
    let only_refusals = Asking::anything().only(Only::Refusals);
    let refusals: Vec<_> = ungranting.record.answering(&only_refusals).collect();
    assert_eq!(refusals.len(), 1, "{:?}", ungranting.record);
    let refusal = refusals.first().unwrap();
    let why = refusal.happened().why_stopped().unwrap();
    assert!(why.as_str().contains("on this machine"), "{why}");
    assert!(why.as_str().contains(RECEPTION), "{why}");
    assert!(refusal.origin().is_some_and(|from| from.is(RECEPTION)));

    // And the same verb, granted at the studio, runs.
    let (granting_found, folder, granting) =
        a_studio("granted", 1, StudioSetting::granting_reception(at_studio));
    let outcome = reading_from_reception(&at_reception, &reception(), &granting_found, &folder, 0);
    let granting = granting.join().unwrap();
    assert!(answered(outcome).did().is_some());
    assert_eq!(
        granting
            .record
            .answering(&Asking::anything().only(Only::Executions))
            .count(),
        1
    );
}

/// **Every answer sent back travels holding a `Departing`**, and there is no
/// road to the wire without one. Over the wire: an answer and a refusal from
/// the door each leave one `left` entry at the studio, naming reception as
/// the destination and as the origin, and the indicator is quiet once each
/// has gone; a rule at the studio that says nothing leaves holds the answer
/// back — written down as such, nothing on the wire, and reception is told
/// the network refused. In the source: one `write_all` in the receiving
/// file, in the one function that takes a `Replying`, whose two carrying
/// constructors take a `Departing`; and the dial spells no address.
#[test]
fn every_answer_sent_back_travels_holding_a_departing_and_there_is_no_road_to_the_wire_without_one()
{
    let (at_reception, at_studio) = paired();

    // An answer, and then a refusal from the door: two departures.
    let (studio_found, folder, studio) = a_studio(
        "departing",
        2,
        StudioSetting::granting_reception(at_studio.clone()),
    );
    let outcome = reading_from_reception(&at_reception, &reception(), &studio_found, &folder, 0);
    assert!(answered(outcome).did().is_some());
    let elsewhere = a_folder_with_an_invoice("departing-elsewhere").0;
    let outcome = reading_from_reception(&at_reception, &reception(), &studio_found, &elsewhere, 1);
    assert_eq!(turned_away(outcome), AtTheDoor::NotGrantedThere);
    let studio = studio.join().unwrap();

    let left: Vec<_> = studio
        .record
        .everything()
        .filter(|entry| matches!(entry.happened(), Happened::Left { .. }))
        .collect();
    assert_eq!(left.len(), 2, "{:?}", studio.record);
    for entry in left {
        assert_eq!(
            entry.happened().destination(),
            Some(&Destination::paired(RECEPTION).unwrap())
        );
        assert_eq!(
            entry.happened().why_it_was_leaving(),
            Some(alo_egress::Why::Sending)
        );
        assert!(entry.origin().is_some_and(|from| from.is(RECEPTION)));
        assert!(
            entry
                .agent()
                .is_some_and(|agent| agent.as_str() == RECEPTIONS_PRINCIPAL),
            "{entry:?}"
        );
    }
    assert!(
        studio.quiet_at_the_end,
        "a line stayed on the studio's indicator"
    );

    // A rule that says nothing leaves: the read ran, and nothing went back.
    let (holding_found, folder, holding) = a_studio(
        "held-back",
        1,
        StudioSetting {
            policy: EgressPolicy::NothingLeaves,
            ..StudioSetting::granting_reception(at_studio)
        },
    );
    let outcome = reading_from_reception(&at_reception, &reception(), &holding_found, &folder, 0);
    let holding = holding.join().unwrap();
    assert!(
        matches!(&outcome.outcome, Err(NotCrossed::Left(left)) if matches!(left.why(), WentBack::TheNetwork(_))),
        "{:?}",
        outcome.outcome
    );
    assert!(matches!(
        holding.heard.as_slice(),
        [Heard::NothingLeft { from, why: alo_turn::NoAnswer::HeldBack(_) }] if *from == reception()
    ));
    assert_eq!(what_left(&holding.record), 0);
    assert!(
        holding
            .record
            .everything()
            .any(|entry| matches!(entry.happened(), Happened::HeldBack { .. }))
    );
    assert!(holding.quiet_at_the_end);

    // In the source.
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let receiving = fs::read_to_string(src.join("receiving.rs")).unwrap();
    assert_eq!(
        receiving.matches("write_all").count(),
        1,
        "receiving.rs writes to the wire from more than one place"
    );
    let the_one_road = receiving
        .find("fn written(stream: &mut TcpStream, replying: &Replying)")
        .unwrap();
    assert!(
        receiving.find("write_all").unwrap() > the_one_road,
        "the write is not inside the function that takes a Replying"
    );
    let replying = fs::read_to_string(src.join("replying.rs")).unwrap();
    assert!(replying.contains("pub const fn answered(departing: Departing, answered: Answered)"));
    assert!(
        replying.contains("pub const fn refused(departing: Departing, at_the_door: AtTheDoor)")
    );
    let dialling = fs::read_to_string(src.join("dialling.rs")).unwrap();
    let shipped = dialling.split("#[cfg(test)]").next().unwrap();
    assert!(!shipped.contains("parse("), "the dial spells an address");
    assert!(
        !shipped.contains("SocketAddr::new"),
        "the dial makes an address"
    );
    for file in [
        "doorway.rs",
        "crossing.rs",
        "carried.rs",
        "answered.rs",
        "door.rs",
        "refusing.rs",
        "replying.rs",
        "holding.rs",
        "naming.rs",
        "words.rs",
        "lib.rs",
    ] {
        let text = fs::read_to_string(src.join(file)).unwrap();
        let shipped = text.split("#[cfg(test)]").next().unwrap();
        assert!(
            !shipped.contains("TcpStream::connect"),
            "{file} opens a connection"
        );
        assert!(!shipped.contains("TcpListener"), "{file} listens");
    }
}

/// **The receiving machine's record names the origin machine, and the asking
/// machine's record names where the verb went.** Every entry at the studio
/// carries reception as its origin, and reception's one entry is a departure
/// to the studio machine, by the name reception's person gave it, under the
/// agent that asked.
#[test]
fn each_machines_record_names_the_other() {
    let (at_reception, at_studio) = paired();
    let (studio_found, folder, studio) =
        a_studio("named", 1, StudioSetting::granting_reception(at_studio));
    let at_reception =
        reading_from_reception(&at_reception, &reception(), &studio_found, &folder, 0);
    let studio = studio.join().unwrap();

    assert_eq!(at_reception.record.len(), 1);
    assert!(
        !at_reception.quiet_at_the_end,
        "the line came off reception's indicator before the departure was spent"
    );
    let went = at_reception.record.everything().next().unwrap();
    assert!(matches!(went.happened(), Happened::Left { .. }), "{went:?}");
    assert_eq!(
        went.happened().destination(),
        Some(&Destination::paired(THE_STUDIO).unwrap())
    );
    assert!(went.agent().is_some_and(|agent| agent.as_str() == "@files"));
    assert_eq!(
        went.origin(),
        None,
        "reception's own verb was recorded as another machine's"
    );
    assert!(
        matches!(&at_reception.outcome, Ok(crossed) if crossed.departing().destination() == &Destination::paired(THE_STUDIO).unwrap())
    );

    assert!(!studio.record.is_empty());
    assert!(
        studio
            .record
            .everything()
            .all(|entry| entry.origin().is_some_and(|from| from.is(RECEPTION))),
        "{:?}",
        studio.record
    );
    assert_eq!(
        studio
            .record
            .answering(&Asking::anything().only(Only::FromAnotherMachine))
            .count(),
        studio.record.len()
    );
}

/// **The second message of an open turn carries a fresh proof, and the first
/// one replayed is refused.** Two reads from reception land in one turn at
/// the studio — the turn's end is the same after each — the second having
/// been judged on its own proof; and the first, sent again exactly, is
/// *already used*.
#[test]
fn the_second_message_of_an_open_turn_carries_a_fresh_proof_and_the_first_one_replayed_is_refused()
{
    let (at_reception, at_studio) = paired();
    let (studio_found, folder, studio) =
        a_studio("open-turn", 3, StudioSetting::granting_reception(at_studio));
    let (recorder_port, recording) = recording_one_request(studio_found.where_it_answers());
    let in_the_road = found_by_discovery(the_studio(), recorder_port);

    let first = reading_from_reception(&at_reception, &reception(), &in_the_road, &folder, 0);
    let first_bytes = recording.join().unwrap();
    let second = reading_from_reception(&at_reception, &reception(), &studio_found, &folder, 1);
    assert!(answered(first).did().is_some());
    assert!(answered(second).did().is_some());

    let (status, word) = replayed(studio_found.where_it_answers(), &first_bytes);
    let studio = studio.join().unwrap();

    assert!(matches!(
        studio.heard.as_slice(),
        [
            Heard::Answered { .. },
            Heard::Answered { .. },
            Heard::TurnedAwayAtTheDoor(AtTheDoor::NotProven(NotProven::AlreadySeen))
        ]
    ));
    let [after_first, after_second, after_replay] = studio.turn_ends.as_slice() else {
        unreachable!("three messages were heard")
    };
    assert!(after_first.is_some());
    assert_eq!(
        after_first, after_second,
        "the second read began a second turn"
    );
    assert_eq!(after_second, after_replay);
    assert!(status.starts_with("HTTP/1.1 403"), "{status}");
    assert_eq!(
        AtTheDoor::off_the_wire(&word),
        Some(AtTheDoor::NotProven(NotProven::AlreadySeen))
    );
    // Two reads ran, two answers left, and the replay added nothing.
    assert_eq!(studio.record.len(), 4, "{:?}", studio.record);
}

/// **A change from a paired machine waits for the person at the studio**,
/// who approves it on their own machine; reception is told the number it
/// waits under and nothing more, and the file moves once.
#[test]
fn a_change_from_a_paired_machine_waits_for_the_person_at_the_studio() {
    let (at_reception, at_studio) = paired();
    let (studio_found, folder, studio) = a_studio(
        "a-change",
        1,
        StudioSetting {
            approving: true,
            ..StudioSetting::granting_reception(at_studio)
        },
    );
    let invoice = folder.join("march.pdf");
    let outcome = crossing_from_reception(
        &at_reception,
        &reception(),
        &studio_found,
        0,
        |crossing, indicator| {
            crossing.changing(
                "rename_file",
                &[
                    ("file", as_given(&invoice)),
                    ("name", Given::text("march-final.pdf")),
                ],
                &EgressPolicy::InTheBuilding,
                indicator,
                the_moment_of(0),
            )
        },
    );
    let studio = studio.join().unwrap();

    let answered = answered(outcome);
    assert!(answered.waits().is_some(), "{answered:?}");
    assert!(answered.did().is_none());
    assert!(matches!(
        studio.heard.as_slice(),
        [Heard::Answered { from, waits: Some(_) }] if *from == reception()
    ));
    assert!(!studio.invoice.is_file(), "the file did not move");
    assert!(studio.folder.join("march-final.pdf").is_file());
    let ran: Vec<_> = studio
        .record
        .everything()
        .filter(|entry| entry.happened().ran())
        .collect();
    assert_eq!(ran.len(), 1);
    let it = ran.first().unwrap();
    assert!(it.happened().from_approval().is_some());
    assert!(it.origin().is_some_and(|from| from.is(RECEPTION)));
}
