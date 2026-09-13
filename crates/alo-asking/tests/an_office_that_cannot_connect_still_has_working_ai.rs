//! *The whole of it works with no internet at all.* An office that cannot
//! connect still has working AI — measured, in the shape
//! `a_day_that_never_left.rs` uses for one machine, across two.
//!
//! | The acceptance | The test |
//! |---|---|
//! | with every non-local address unreachable, two machines discover each other, pair, ask and answer, and the record and the indicator are as they would be with a connection | [`with_no_route_off_the_network_two_machines_still_find_each_other_pair_ask_and_answer`] |
//! | nothing anywhere in the road addresses a single packet off-network | [`nothing_on_the_road_addresses_a_single_packet_off_the_network`] |
//! | the unreachability is enforced in the test rather than assumed | [`a_public_address_really_is_unreachable_where_this_is_measured`] |
//! | what a person is told about the machine's connectedness is true, and said once, through `alo-telling` | [`a_question_bound_for_the_internet_is_told_once_and_truthfully`] |
//! | and it is not said when it is not true | [`a_far_end_that_refused_is_not_said_to_be_out_of_reach`] |
//!
//! # How the office is made, and why nothing here is a mock
//!
//! Every measured test in this file runs **inside a network namespace of its
//! own** with only the loopback interface in it. That is a real kernel with a
//! real routing table that has no route to anything beyond this machine, and
//! it is made by util-linux's `unshare` rather than by this file, because the
//! one crate in this workspace that could make one from Rust marks the safe
//! call deprecated and the other needs `unsafe`, which this repository does
//! not add for a test. The test binary re-runs itself inside the namespace
//! — the way `alo-bounding`'s tests re-run themselves inside a control group —
//! and the outer test reads what the inner one measured.
//!
//! **The measurement is the kernel's own.** `/proc/net/snmp` carries
//! `OutNoRoutes`, the count of packets this namespace was asked to send and
//! had no route for, and the kernel increments it at the moment a `connect`
//! or a `sendto` is refused for want of a route — before a byte goes anywhere.
//! A road that addressed one packet off the network raises it by one. So the
//! second line of the acceptance is not reasoned about the code: it is that
//! number, read before the day and after it, unchanged.
//!
//! **The control comes first.** A counter that never moves proves nothing on
//! its own, so [`a_public_address_really_is_unreachable_where_this_is_measured`]
//! addresses one stream and one datagram at an address reserved for
//! documentation and watches the kernel refuse both and count both. Only then
//! does a zero mean what it says.
//!
//! # What is on this host, and what is owed to two machines
//!
//! Both machines are in one process on one host, over real sockets — which is
//! what task 1 and task 3 already measured, and is what shows the packets are
//! right and the road is walked. The reception machine dials the studio at
//! what discovery measured — [`alo_nearby::Found::where_it_answers`], the
//! address the studio's answer came from and the port it advertised — and
//! every question carries a proof the studio checks against its own pairing
//! before it answers (ADR 0031). Whether an office switch carries the same
//! packets between two chassis is owed to two machines, as every report in
//! this plan has said.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::env;
use std::io::{BufRead as _, Read as _, Write as _};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::{Answering, WentWrong};
use alo_asking::{Asking, DownTheCorridor, Hosted, NotAsked, Question, THE_PROOF_HEADER};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::{InferenceSource, Provider, Region, SourcePolicy};
use alo_nearby::{
    Answering as AnsweringWhoIsHere, Deliberating, Keying, Looking, MachineId, MayAskIts, Pairings,
    Presence, Proof, Proposal, Proven, Seen, Side, Standing,
};
use alo_record::{Entry, Only, Record};
use alo_strings::Strings;
use alo_telling::{Tell, Telling, WhoAsked};

/// Set on the test binary when it is running inside the office, so an inner
/// test run by hand on a host with a connection refuses rather than measures.
const INSIDE: &str = "ALO_INSIDE_AN_OFFICE_THAT_CANNOT_CONNECT";

/// What an inner test prints before the day it measured.
const THE_DAY: &str = "alo:day ";

/// What an inner test prints before the number of packets addressed off the
/// network during whatever it measured.
const OFF_NETWORK: &str = "alo:off-network ";

/// What the telling test prints when the whole of it held.
const TOLD_ONCE: &str = "alo:told-once";

/// An address reserved for documentation (RFC 5737), so that if the namespace
/// somehow failed to take effect the control packet would still reach nothing
/// anybody owns.
const BEYOND_THE_BUILDING: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 443);

/// The machine at reception — the one with no GPU in it.
fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").expect("an identity")
}

/// The machine down the corridor, with the GPU in it.
fn the_studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").expect("an identity")
}

/// What the person at reception called the studio when they paired with it.
const THE_STUDIO: &str = "the studio machine";

/// What the person at the studio called reception when they paired with it.
const RECEPTION: &str = "the reception machine";

/// How many questions a working day is, here.
const A_WORKING_DAY: u64 = 8;

/// A moment to reason from, so nothing here depends on when it is run.
fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// Everything this machine can say, which is what a shell holds.
fn strings() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("the machine's own words"))
}

/// The studio, listening on a port of this host's before anybody has paired
/// with it — which is what discovery advertises.
fn a_studio_listening() -> (u16, TcpListener) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("a port on this host");
    let port = listener.local_addr().expect("an address").port();
    (port, listener)
}

/// The studio, answering this many questions in turn and handing back every
/// request it was sent so the test can read what travelled.
///
/// **Each question is judged before it is answered** (ADR 0031): the proof
/// in its header is checked against the studio's own pairings, over the exact
/// bytes that arrived, at the moment the studio's clock has for that
/// question, and a question that does not prove where it came from is
/// refused with nothing answered. `moment_of` is the studio's clock, which
/// ticks once per question the way the day does.
fn a_studio_answering(
    listener: TcpListener,
    how_many: u64,
    said: &'static str,
    pairings: Pairings,
    moment_of: impl Fn(u64) -> SystemTime + Send + 'static,
) -> thread::JoinHandle<Vec<String>> {
    thread::spawn(move || {
        let mut sent = Vec::new();
        let mut seen = Seen::nothing();
        for nth in 0..how_many {
            let (mut stream, _) = listener.accept().expect("one question");
            let mut reader = std::io::BufReader::new(stream.try_clone().expect("the same socket"));
            let mut request = String::new();
            let mut length = 0_usize;
            let mut proof: Option<Proof> = None;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).expect("a line") == 0 {
                    break;
                }
                let lowered = line.to_ascii_lowercase();
                if let Some(how_long) = lowered.strip_prefix("content-length:") {
                    length = how_long.trim().parse().unwrap_or(0);
                }
                if let Some(said) = lowered.strip_prefix(&format!("{THE_PROOF_HEADER}:")) {
                    proof = Proof::read(said.trim()).ok();
                }
                let done = line == "\r\n" || line == "\n";
                request.push_str(&line);
                if done {
                    break;
                }
            }
            let mut body = vec![0_u8; length];
            if length > 0 {
                reader.read_exact(&mut body).expect("the body");
            }
            request.push_str(&String::from_utf8_lossy(&body));
            let proven = proof.as_ref().and_then(|proof| {
                Proven::checked(
                    &pairings,
                    &the_studio(),
                    proof,
                    &body,
                    moment_of(nth),
                    &mut seen,
                )
                .ok()
            });
            let (status, answer) = match proven {
                Some(proven) => {
                    assert_eq!(proven.machine(), &reception());
                    (
                        "200 OK",
                        format!(
                            r#"{{"choices":[{{"message":{{"role":"assistant","content":"{said}"}}}}]}}"#
                        ),
                    )
                }
                None => ("400 Bad Request", "{}".to_owned()),
            };
            write!(
                stream,
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{answer}",
                answer.len()
            )
            .expect("a reply");
            stream.flush().expect("a flush");
            sent.push(request);
        }
        sent
    })
}

/// A datagram socket of this test's own, on this host and nowhere else.
fn a_socket() -> UdpSocket {
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).expect("a socket on this host");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("a timeout");
    socket
}

/// What one working day came to, in a shape that is the same wherever the
/// day was walked: what each machine wrote down, what the person at reception
/// was shown while each question left, and what came back.
///
/// Ports and moments are deliberately not in it — a port is chosen by the
/// host and a moment is fixed by the test — so two days walked on two
/// different networks compare on what matters.
#[derive(Debug, PartialEq, Eq)]
struct Day {
    /// Everything the reception machine's record says happened, in order.
    reception_wrote: Vec<String>,
    /// Everything the studio's record says happened, in order.
    studio_wrote: Vec<String>,
    /// The line on reception's indicator while each question was away.
    shown: Vec<String>,
    /// What came back, and where each answer says it came from.
    answered: Vec<String>,
}

/// A working day in an office with one GPU: reception finds the studio, the
/// two people pair their machines, and eight questions go down the corridor
/// and come back — with the indicator firing for each and both records
/// written.
///
/// The same function is walked inside the office that cannot connect and on
/// this host as it is, so that the two days can be compared rather than
/// described.
fn a_working_day() -> Day {
    let strings = strings();
    let now = a_moment();
    let (studio_port, studio_listening) = a_studio_listening();

    // **Discovery.** The studio says it exists; reception asks who is here and
    // finds one machine, not paired.
    let studio_socket = a_socket();
    let where_the_studio_listens = studio_socket.local_addr().expect("an address");
    let advertising =
        AnsweringWhoIsHere::on(studio_socket, Presence::of(the_studio(), studio_port));
    let answered = thread::spawn(move || advertising.answer_one());
    let looking = Looking::from(a_socket());
    looking
        .ask(where_the_studio_listens)
        .expect("the question goes out");
    let found = looking
        .found(Duration::from_secs(5))
        .expect("a search ends");
    answered
        .join()
        .expect("the studio's thread")
        .expect("the studio answered");
    assert_eq!(found.len(), 1, "{found:?}");
    let found = found.first().expect("one machine");
    assert_eq!(found.machine, the_studio());
    assert_eq!(found.port, studio_port);
    assert_eq!(found.standing, Standing::NotPaired);

    // **Being found confers nothing.** The studio is right there and would
    // answer, and there is no door to it until two people say so. Where it
    // would be dialled is what discovery measured — the address the answer
    // came from and the port it advertised — rather than an address typed
    // here.
    let at = found.where_it_answers();
    assert_eq!(at.port(), studio_port);
    assert!(
        DownTheCorridor::paired(
            &Pairings::none(),
            &reception(),
            &found.machine,
            THE_STUDIO,
            at,
            None,
            now
        )
        .is_err(),
        "a machine that was merely found was usable"
    );

    // **Pairing, by two people**, each on their own machine (ADR 0031): a
    // keying on each side, the offers crossed, the same six digits read on
    // both, and each machine keeping a row naming the other with the same key
    // on it.
    let at_reception = Keying::fresh().expect("randomness");
    let at_the_studio = Keying::fresh().expect("randomness");
    let proposal = Proposal::checked(
        reception(),
        the_studio(),
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        at_reception.offer().clone(),
    )
    .expect("a proposal");
    let studio_side = Deliberating::asked(proposal.clone(), at_the_studio);
    let reception_side = Deliberating::asking(proposal, at_reception)
        .expect("the keying whose offer the proposal carries")
        .answered_with(studio_side.answered().expect("the studio's offer").clone())
        .expect("the studio's own offer, not a reflection");
    assert_eq!(reception_side.code(), studio_side.code());
    let mut reception_pairings = Pairings::none();
    reception_pairings.keep(
        reception_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(now)
            .expect("both agreed"),
    );
    let mut studio_pairings = Pairings::none();
    studio_pairings.keep(
        studio_side
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(now)
            .expect("both agreed"),
    );
    assert!(reception_pairings.permits(&the_studio(), MayAskIts::Models, now));
    assert!(studio_pairings.paired_with(&reception(), now));

    // The studio now answers, judging each question against its own pairings
    // with its clock at the hour the question is asked.
    let studio = a_studio_answering(
        studio_listening,
        A_WORKING_DAY,
        "Yes, and here is why.",
        studio_pairings.clone(),
        move |hour| now + Duration::from_secs(60 * 60 * hour),
    );

    // **The day.** Under the rule an office like this one would set: nothing
    // leaves the building. The corridor stays inside it.
    let policy = SourcePolicy::InTheBuilding;
    let mail = Grantee::named("@mail");
    let mut indicator = Indicator::default();
    let mut reception_record = Record::default();
    let mut studio_record = Record::default();
    let mut shown = Vec::new();
    let mut answered = Vec::new();

    for hour in 0..A_WORKING_DAY {
        let when = now + Duration::from_secs(60 * 60 * hour);
        let corridor = DownTheCorridor::paired(
            &reception_pairings,
            &reception(),
            &the_studio(),
            THE_STUDIO,
            at,
            None,
            when,
        )
        .expect("the pairing permits asking the studio's models");
        let question =
            Question::asked("may the tenant sublet?", "a-big-model").expect("a question");
        let answering = Answering::chosen(corridor.source(), &policy)
            .expect("the corridor stays in the building");

        let asked = Asking::by(&mail, answering, &[], &policy)
            .to_a_paired_machine(&question, &corridor, &mut indicator, when, &[at])
            .expect("the studio answered");

        // While the question is away, the person at reception sees it go.
        assert!(
            !indicator.is_quiet(),
            "the question left with the indicator quiet"
        );
        shown.push(
            indicator
                .showing()
                .first()
                .expect("one line")
                .said(&strings)
                .text()
                .to_owned(),
        );
        reception_record.keep(Entry::left(asked.departing()));
        let answer = asked.ended(&mut indicator);
        assert!(
            indicator.is_quiet(),
            "the line stayed up after the answer came back"
        );
        answered.push(format!(
            "{} ({})",
            answer.text(),
            answer.came_from(&strings).text()
        ));

        // And the studio writes down that it answered for reception.
        studio_record.keep(Entry::answered_for(RECEPTION, when));
    }

    // Every question really travelled, whole.
    let sent = studio.join().expect("the studio finished");
    assert_eq!(
        sent.len(),
        usize::try_from(A_WORKING_DAY).expect("a small number")
    );
    for request in &sent {
        assert!(request.contains("may the tenant sublet?"), "{request}");
    }

    // The record at reception says eight things left, every one of them to
    // the studio; the record at the studio says eight arrived from reception.
    assert_eq!(
        reception_record
            .answering(&alo_record::Asking::anything().only(Only::Egress))
            .count(),
        usize::try_from(A_WORKING_DAY).expect("a small number")
    );
    assert_eq!(
        studio_record
            .answering(&alo_record::Asking::anything().only(Only::FromAnotherMachine))
            .count(),
        usize::try_from(A_WORKING_DAY).expect("a small number")
    );

    Day {
        reception_wrote: reception_record
            .everything()
            .map(|entry| format!("{:?}", entry.happened()))
            .collect(),
        studio_wrote: studio_record
            .everything()
            .map(|entry| format!("{:?}", entry.happened()))
            .collect(),
        shown,
        answered,
    }
}

/// How many packets this network namespace has been asked to send and had no
/// route for, as the kernel counts them — over both address families.
///
/// `/proc/net/snmp` is the namespace's own: a process reads the counters of
/// the namespace it is in, so a number read inside the office is about the
/// office and nothing else on this host.
fn packets_addressed_off_the_network() -> u64 {
    let four = std::fs::read_to_string("/proc/net/snmp").expect("the kernel's IPv4 counters");
    let mut lines = four.lines().filter(|line| line.starts_with("Ip:"));
    let names = lines.next().expect("a line naming the counters");
    let values = lines.next().expect("a line of values");
    let which = names
        .split_whitespace()
        .position(|name| name == "OutNoRoutes")
        .expect("the kernel counts packets with no route");
    let v4: u64 = values
        .split_whitespace()
        .nth(which)
        .expect("a value for it")
        .parse()
        .expect("a number");

    let six = std::fs::read_to_string("/proc/net/snmp6").expect("the kernel's IPv6 counters");
    let v6: u64 = six
        .lines()
        .find_map(|line| line.strip_prefix("Ip6OutNoRoutes"))
        .expect("the kernel counts IPv6 packets with no route")
        .trim()
        .parse()
        .expect("a number");
    v4 + v6
}

/// Refuse to run unless this is the binary re-run inside the office: an inner
/// test run by hand on a host with a connection would measure the wrong
/// network and say so in a confusing way.
fn must_be_inside_the_office() {
    assert!(
        env::var_os(INSIDE).is_some(),
        "this test runs inside the network namespace the outer test makes; run the outer one"
    );
}

/// Run one inner test of this binary inside an office that cannot connect,
/// and hand back everything it printed.
///
/// The office is a network namespace with only loopback in it, made by
/// util-linux, with loopback brought up by iproute2; the test binary is then
/// run again inside it with the inner test named. Its standard error is this
/// process's, so an assertion that fails inside reads exactly as one that
/// fails outside.
fn inside_an_office_that_cannot_connect(inner: &str) -> String {
    let exe = env::current_exe().expect("a test binary knows where it is");
    let script = format!("ip link set lo up && exec \"$0\" --exact --ignored --nocapture {inner}");
    let ran = Command::new("unshare")
        .args(["--map-root-user", "--net", "--", "sh", "-c", &script])
        .arg(exe)
        .env(INSIDE, "yes")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .expect(
            "util-linux's `unshare` makes the office that cannot connect, and this machine does \
             not have it",
        );
    let said = String::from_utf8_lossy(&ran.stdout).into_owned();
    assert!(
        ran.status.success(),
        "the measurement inside the office failed ({}):\n{said}",
        ran.status
    );
    said
}

/// The line an inner test printed under this prefix.
fn the_line_after<'a>(said: &'a str, prefix: &str) -> &'a str {
    said.lines()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap_or_else(|| panic!("the inner test never printed `{prefix}`:\n{said}"))
}

// ---------------------------------------------------------------------------
// The inner tests: what runs inside the office. Each is ignored so that an
// ordinary run of this binary never measures the wrong network, and each is
// run by exactly one outer test below.
// ---------------------------------------------------------------------------

/// The working day, inside the office, with the kernel's count of packets
/// addressed off the network read before and after it.
#[test]
#[ignore = "run inside the office by the outer tests"]
fn the_day_inside_the_office() {
    must_be_inside_the_office();
    let before = packets_addressed_off_the_network();
    let day = a_working_day();
    let after = packets_addressed_off_the_network();
    println!("{OFF_NETWORK}{}", after - before);
    println!("{THE_DAY}{day:?}");
}

/// The control, inside the office: one stream and one datagram addressed
/// beyond the building, both refused by the kernel for want of a route, and
/// both counted.
#[test]
#[ignore = "run inside the office by the outer tests"]
fn the_control_inside_the_office() {
    must_be_inside_the_office();
    let before = packets_addressed_off_the_network();

    let stream = TcpStream::connect_timeout(&BEYOND_THE_BUILDING, Duration::from_secs(5))
        .expect_err("a stream left an office with no route out of it");
    assert_eq!(
        stream.kind(),
        std::io::ErrorKind::NetworkUnreachable,
        "{stream}"
    );

    let datagram = a_socket()
        .send_to(b"is anybody there?", BEYOND_THE_BUILDING)
        .expect_err("a datagram left an office with no route out of it");
    assert_eq!(
        datagram.kind(),
        std::io::ErrorKind::NetworkUnreachable,
        "{datagram}"
    );

    let after = packets_addressed_off_the_network();
    println!("{OFF_NETWORK}{}", after - before);
}

/// A question bound for a provider, inside the office: the kernel says there
/// is no way there, the person is told so once in words that are true, the
/// machine's own retries say nothing, and the person asking again is answered.
#[test]
#[ignore = "run inside the office by the outer tests"]
fn the_telling_inside_the_office() {
    must_be_inside_the_office();
    let strings = strings();
    let now = a_moment();
    let before = packets_addressed_off_the_network();

    // A provider somebody set up before the office lost its connection, and
    // the studio down the corridor, paired.
    let mistral = Provider::checked(
        "Mistral",
        "https://192.0.2.1",
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider");
    let hosted = Hosted::provider(&mistral, None);
    let the_corridor = InferenceSource::PairedMachine {
        machine: THE_STUDIO.to_owned(),
    };
    let policy = SourcePolicy::Anywhere;
    let mail = Grantee::named("@mail");
    let question =
        Question::asked("may the tenant sublet?", "mistral-small-latest").expect("a question");
    let mut telling = Telling::nothing_said_yet();
    let mut record = Record::default();
    let mut indicator = Indicator::default();

    // One attempt to ask the provider, which the office cannot reach.
    let asking = |indicator: &mut Indicator, record: &mut Record| -> alo_answering::Failed {
        let answering =
            Answering::chosen(hosted.named_source(), &policy).expect("the rule permits it");
        let not_asked = Asking::by(
            &mail,
            answering,
            std::slice::from_ref(&the_corridor),
            &policy,
        )
        .to_a_provider(&question, &hosted, indicator, now, &[BEYOND_THE_BUILDING])
        .expect_err("the office cannot connect");
        let NotAsked::DidNotAnswer(unanswered) = not_asked else {
            panic!("the question was never attempted: {not_asked:?}");
        };
        // It was shown leaving and attempted, so it is written down as such;
        // what the kernel then said about it is in the failure.
        record.keep(Entry::left(unanswered.departing()));
        let failed = unanswered.ended(indicator);
        assert!(indicator.is_quiet());
        failed
    };

    // The person asks, and is told — the true thing, once, in four lines.
    let failed = asking(&mut indicator, &mut record);
    assert_eq!(failed.why(), WentWrong::NoWayThere);
    let tell = telling.about(failed, WhoAsked::ThePerson);
    let told = tell.to_say().expect("nobody has been told this yet");
    let lines = told.lines(&strings);
    let [heading, what_happened, nothing_was_sent, carry_on] = &lines;
    assert_eq!(heading.text(), "The agent cannot answer right now");
    assert_eq!(
        what_happened.text(),
        "nothing was answered by Mistral, in the EU — there is no way there from the network \
         this machine is on, so the question was not sent"
    );
    assert_eq!(
        nothing_was_sent.text(),
        "nothing was sent anywhere, and nothing will be unless you say so"
    );
    assert!(
        carry_on.text().starts_with("You can carry on."),
        "{carry_on}"
    );
    for line in &lines {
        for claim in ["offline", "internet", "disconnected"] {
            assert!(
                !line.text().to_ascii_lowercase().contains(claim),
                "the machine claimed something it cannot know: {line}"
            );
        }
    }
    // The studio is offered, and nothing takes it: that is the person's.
    assert_eq!(told.elsewhere().offers().len(), 1);
    assert_eq!(
        told.elsewhere()
            .offers()
            .first()
            .map(alo_answering::Offer::source),
        Some(&the_corridor)
    );

    // The machine tries three more times by itself over the afternoon, and
    // says nothing each time: it has said this.
    for _ in 0..3 {
        let again = asking(&mut indicator, &mut record);
        assert_eq!(
            telling.about(again, WhoAsked::TheMachine),
            Tell::SaidAlready,
            "the machine nagged"
        );
    }

    // The person asks again themselves, and is answered rather than ignored.
    let once_more = asking(&mut indicator, &mut record);
    assert!(
        telling.about(once_more, WhoAsked::ThePerson).was_said(),
        "the person asked and was not answered"
    );
    assert_eq!(telling.how_many_it_remembers(), 1);

    // Five attempts: five entries saying what the agent tried, and exactly
    // five packets the kernel refused a route for — one per attempt, and no
    // retry anywhere in the road.
    assert_eq!(
        record
            .answering(&alo_record::Asking::anything().only(Only::Egress))
            .count(),
        5
    );
    let after = packets_addressed_off_the_network();
    println!("{OFF_NETWORK}{}", after - before);
    println!("{TOLD_ONCE}");
}

// ---------------------------------------------------------------------------
// The outer tests: one per line of the acceptance, each running its inner
// test inside an office that cannot connect and reading what it measured.
// ---------------------------------------------------------------------------

/// **With every non-local address unreachable, two machines discover each
/// other, pair, ask and answer — and the record and the indicator are as they
/// would be with a connection.** The same day is walked inside the office and
/// on this host as it is, and the two are compared entry for entry and line
/// for line.
#[test]
fn with_no_route_off_the_network_two_machines_still_find_each_other_pair_ask_and_answer() {
    let inside = inside_an_office_that_cannot_connect("the_day_inside_the_office");
    let measured = the_line_after(&inside, THE_DAY);

    let with_a_connection = a_working_day();
    assert_eq!(
        measured,
        format!("{with_a_connection:?}"),
        "the day in the office that cannot connect was not the day with a connection"
    );
    // And it was a day: eight things left reception, eight arrived at the
    // studio, eight lines were shown and eight answers came back.
    assert_eq!(with_a_connection.reception_wrote.len(), 8);
    assert_eq!(with_a_connection.studio_wrote.len(), 8);
    assert_eq!(with_a_connection.shown.len(), 8);
    assert_eq!(with_a_connection.answered.len(), 8);
    assert!(
        with_a_connection
            .shown
            .iter()
            .all(|line| line == "@mail is asking a question of the studio machine, on your network"),
        "{:?}",
        with_a_connection.shown
    );
    assert!(
        with_a_connection
            .answered
            .iter()
            .all(|answer| answer.contains("the studio machine")),
        "{:?}",
        with_a_connection.answered
    );
}

/// **Nothing anywhere in the road addresses a single packet off-network.**
/// The kernel's own count of packets it refused a route for, read before the
/// day and after it inside the office, and unchanged — a retry against a
/// public address anywhere in discovery, pairing, asking or answering would
/// be one.
#[test]
fn nothing_on_the_road_addresses_a_single_packet_off_the_network() {
    let inside = inside_an_office_that_cannot_connect("the_day_inside_the_office");
    assert_eq!(
        the_line_after(&inside, OFF_NETWORK),
        "0",
        "a packet was addressed off the network during the day"
    );
}

/// **The unreachability is enforced, not assumed.** Where the day is
/// measured, a stream and a datagram addressed beyond the building are
/// refused by the kernel with *network unreachable* and counted, two for two
/// — so the zero above is a zero from a counter that moves.
#[test]
fn a_public_address_really_is_unreachable_where_this_is_measured() {
    let inside = inside_an_office_that_cannot_connect("the_control_inside_the_office");
    assert_eq!(
        the_line_after(&inside, OFF_NETWORK),
        "2",
        "the kernel did not count the two packets it was asked to send off the network"
    );
}

/// **What a person is told about the machine's connectedness is true and
/// said once, through `alo-telling`.** Inside the office, a question bound
/// for a provider is refused by the kernel for want of a route, and the
/// person reads that there is no way there from this network — not that the
/// provider failed, not that the machine is offline — once; the machine's own
/// retries say nothing; and five attempts are five packets the kernel
/// counted, so nothing retried in between.
#[test]
fn a_question_bound_for_the_internet_is_told_once_and_truthfully() {
    let inside = inside_an_office_that_cannot_connect("the_telling_inside_the_office");
    assert!(inside.lines().any(|line| line == TOLD_ONCE), "{inside}");
    assert_eq!(
        the_line_after(&inside, OFF_NETWORK),
        "5",
        "five attempts were not five packets"
    );
}

/// **And the sentence is not said when it is not true.** On this host, a
/// provider at an address that is reachable and refuses the connection is
/// reported as nothing having answered — the machine does not claim to have
/// no way somewhere it just reached. This one runs outside the office on
/// purpose: it is the refusal path of the test above.
#[test]
fn a_far_end_that_refused_is_not_said_to_be_out_of_reach() {
    // A port on this host that nothing is listening on: the kernel reaches it
    // and the far end says no.
    let closed = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("a port");
    let at = closed.local_addr().expect("an address");
    drop(closed);

    let provider = Provider::checked(
        "Mistral",
        &format!("http://{at}"),
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider");
    let hosted = Hosted::provider(&provider, None);
    let policy = SourcePolicy::Anywhere;
    let mail = Grantee::named("@mail");
    let question = Question::asked("anything", "mistral-small-latest").expect("a question");
    let mut indicator = Indicator::default();

    let not_asked = Asking::by(
        &mail,
        Answering::chosen(hosted.named_source(), &policy).expect("permitted"),
        &[],
        &policy,
    )
    .to_a_provider(&question, &hosted, &mut indicator, a_moment(), &[at])
    .expect_err("nothing is listening");
    let NotAsked::DidNotAnswer(unanswered) = not_asked else {
        panic!("the question was never attempted: {not_asked:?}");
    };
    let failed = unanswered.ended(&mut indicator);
    assert_eq!(failed.why(), WentWrong::NothingAnswered);

    let mut telling = Telling::nothing_said_yet();
    let tell = telling.about(failed, WhoAsked::ThePerson);
    let told = tell.to_say().expect("nobody has been told this yet");
    let what_happened = told.what_happened(&strings());
    assert_eq!(
        what_happened.text(),
        "nothing answered by Mistral, in the EU"
    );
    assert!(!what_happened.text().contains("no way there"));
}
