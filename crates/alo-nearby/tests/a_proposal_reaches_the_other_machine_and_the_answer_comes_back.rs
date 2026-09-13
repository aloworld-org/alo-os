//! *A proposal reaches the other machine, and the answer comes back.*
//!
//! Task 7 of the local-network plan, held from outside the crate over real
//! sockets on one host: reception proposes to the studio at the address
//! discovery measured, the studio's person is shown the proposal and the
//! code before the studio answers with its offer, reception's person is
//! shown the same code, and a pairing is kept on each machine only after both
//! people have confirmed on their own. As every file in this plan does, it
//! carries the refusals beside the agreement, each under its own name.
//!
//! | The acceptance | The test |
//! |---|---|
//! | reaches the machine it names at the address discovery measured, carrying exactly the `Proposal` and nothing else | [`a_proposal_reaches_the_machine_it_names_at_the_address_discovery_measured_carrying_exactly_the_proposal`] |
//! | the asked machine answers with its own `Offer` and nothing else, only once its person has been shown the proposal and the `Code` | [`the_asked_machine_answers_with_its_own_offer_and_nothing_else_once_its_person_has_been_shown_it`], [`a_proposal_nobody_can_be_shown_is_refused_and_nothing_waits`] |
//! | the asking machine's person is shown the same `Code`, and a pairing is kept on each machine only after both confirmed | [`both_people_are_shown_the_same_code_and_a_pairing_is_kept_on_each_machine_only_after_both_confirmed`] |
//! | the asking side confirming alone | [`the_asking_side_confirming_alone_keeps_nothing_on_either_machine`] |
//! | the asked side confirming alone | [`the_asked_side_confirming_alone_keeps_nothing_on_either_machine`] |
//! | a proposal from an address discovery never measured | [`a_proposal_from_an_address_discovery_never_measured_is_refused_before_anybody_is_shown_anything`] |
//! | refused when it names a machine other than the one it arrived at | [`a_proposal_naming_another_machine_is_refused_before_anybody_is_shown_anything`] |
//! | refused when its `Offer` does not read | [`a_proposal_whose_offer_does_not_read_is_refused_before_anybody_is_shown_anything`] |
//! | refused when a second from the same machine arrives while the first waits | [`a_second_proposal_from_the_same_machine_while_the_first_waits_is_refused`] |
//! | a proposal nobody answered expires from both machines within a stated time | [`a_proposal_nobody_answered_lapses_from_both_machines_within_the_stated_time`] |
//! | nothing is written in either record until a pairing is kept | [`nothing_is_kept_on_either_machine_until_a_pairing_is_made`] |

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

use alo_nearby::{
    Answering, Code, Deliberating, Found, Heard, Keying, Looking, MachineId, MayAskIts, NotNearby,
    NotProposed, Offer, Pairings, Presence, Proof, Proposal, Proposals, Proven, Receiving, Seen,
    Surface, THE_PROPOSAL_PATH, WHILE_A_PROPOSAL_WAITS, Waiting, crossing,
};

/// The machine at reception, with no model of its own.
fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The studio machine, with the GPU in it.
fn studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// Somebody on the office network who nobody paired with.
fn a_stranger() -> MachineId {
    MachineId::read("99998888777766665555444433332222").unwrap()
}

/// What one machine holds, behind one lock, so the thread that hears the
/// wire and the person in front of it share it.
struct Machine {
    /// Every proposal waiting here.
    proposals: Proposals,
    /// Every pairing kept here, which is the record this crate has.
    pairings: Pairings,
    /// What discovery here measured.
    found: Vec<Found>,
}

/// A surface that remembers what it was shown, and shows it or does not.
struct Shown {
    /// Each proposal and code it was handed, in order.
    what: Vec<(Proposal, Option<Code>)>,
    /// Whether there is anybody here to show it to.
    somebody_here: bool,
}

impl Surface for Shown {
    fn show(&mut self, waiting: &Waiting) -> bool {
        self.what.push((waiting.proposal().clone(), waiting.code()));
        self.somebody_here
    }
}

/// A surface with somebody in front of it.
fn somebody() -> Arc<Mutex<Shown>> {
    Arc::new(Mutex::new(Shown {
        what: Vec::new(),
        somebody_here: true,
    }))
}

/// A machine listening on this host: what it holds, its end of the wire,
/// and the port it listens on.
fn a_machine(id: MachineId) -> (Arc<Mutex<Machine>>, Receiving, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let machine = Arc::new(Mutex::new(Machine {
        proposals: Proposals::on(id),
        pairings: Pairings::none(),
        found: Vec::new(),
    }));
    (machine, Receiving::on(listener), port)
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
    let answered = std::thread::spawn(move || answering.answer_one());

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

/// The machine hears exactly `how_many` connections, each judged at the
/// moment it is considered, and reports what it heard.
fn hearing(
    receiving: Receiving,
    machine: Arc<Mutex<Machine>>,
    surface: Arc<Mutex<Shown>>,
    how_many: usize,
) -> JoinHandle<Vec<Heard>> {
    std::thread::spawn(move || {
        (0..how_many)
            .map(|_| {
                let arrived = receiving.accept_one().unwrap();
                let mut held = machine.lock().unwrap();
                let Machine {
                    proposals,
                    pairings,
                    found,
                } = &mut *held;
                let mut surface = surface.lock().unwrap();
                arrived
                    .considered(proposals, pairings, found, &mut *surface, SystemTime::now())
                    .unwrap()
            })
            .collect()
    })
}

/// Reception and the studio, each listening, each having found the other by
/// discovery, with the studio's person in front of it: reception first.
#[expect(
    clippy::type_complexity,
    reason = "a fixture handing a test both machines and both ends of the wire"
)]
fn both_machines() -> (
    (Arc<Mutex<Machine>>, Receiving),
    (Arc<Mutex<Machine>>, Receiving),
    Arc<Mutex<Shown>>,
) {
    let (at_reception, reception_hears, reception_port) = a_machine(reception());
    let (at_studio, studio_hears, studio_port) = a_machine(studio());
    at_reception
        .lock()
        .unwrap()
        .found
        .push(found_by_discovery(studio(), studio_port));
    at_studio
        .lock()
        .unwrap()
        .found
        .push(found_by_discovery(reception(), reception_port));
    (
        (at_reception, reception_hears),
        (at_studio, studio_hears),
        somebody(),
    )
}

/// The one machine `held` found.
fn the_one_found(held: &Arc<Mutex<Machine>>) -> Found {
    held.lock().unwrap().found.first().unwrap().clone()
}

/// Reception proposes to the studio for a day, for its models, over the wire.
fn reception_proposes(at_reception: &Arc<Mutex<Machine>>) -> Result<Option<Code>, NotProposed> {
    let to = the_one_found(at_reception);
    let mut held = at_reception.lock().unwrap();
    crossing::propose(
        &mut held.proposals,
        &to,
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        SystemTime::now(),
    )
    .map(Waiting::code)
}

/// The person at `here` confirms the proposal with `other`, over the wire.
fn confirms(here: &Arc<Mutex<Machine>>, other: &MachineId) -> Result<bool, NotProposed> {
    let mut held = here.lock().unwrap();
    let Machine {
        proposals,
        pairings,
        ..
    } = &mut *held;
    let kept = crossing::confirm(proposals, other, SystemTime::now())?;
    let was_kept = kept.is_some();
    if let Some(pairing) = kept {
        pairings.keep(pairing);
    }
    Ok(was_kept)
}

/// One request put on the wire by hand, and the reply read back by hand:
/// the status and the body.
fn put_by_hand(at: SocketAddr, path: &str, body: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(at).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(
        stream,
        "POST {path} HTTP/1.1\r\nhost: {at}\r\ncontent-type: text/plain; charset=utf-8\r\n\
         content-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).unwrap();
    let (head, body) = reply.split_once("\r\n\r\n").unwrap();
    let status: u16 = head.split(' ').nth(1).unwrap().parse().unwrap();
    (status, body.to_owned())
}

/// A proposal from reception to `asked`, for a day, for its models, with the
/// keying whose offer it carries.
fn a_proposal_to(asked: MachineId) -> (Proposal, Keying) {
    let keying = Keying::fresh().unwrap();
    let proposal = Proposal::checked(
        reception(),
        asked,
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        keying.offer().clone(),
    )
    .unwrap();
    (proposal, keying)
}

/// **A proposal reaches the machine it names at the address discovery
/// measured, carrying exactly the `Proposal` and nothing else.** The studio
/// here is a bare socket the test reads itself, so what is held is the bytes
/// on the wire: one request, four framing headers, and a body that is the
/// proposal's one line — both identities, the list, the duration and the
/// offer — with any field beyond those refused by the reader.
#[test]
fn a_proposal_reaches_the_machine_it_names_at_the_address_discovery_measured_carrying_exactly_the_proposal()
 {
    let the_wire = TcpListener::bind("127.0.0.1:0").unwrap();
    let at = the_wire.local_addr().unwrap();
    let found = found_by_discovery(studio(), at.port());
    assert_eq!(found.where_it_answers(), at);

    let studios_offer = Keying::fresh().unwrap().offer().clone();
    let answering_offer = studios_offer.said();
    let read_off_the_wire = std::thread::spawn(move || {
        let (mut stream, who) = the_wire.accept().unwrap();
        let mut request = Vec::new();
        let mut byte = [0_u8; 1];
        while !request.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            request.extend_from_slice(&byte);
        }
        let head = String::from_utf8(request).unwrap();
        let length: usize = head
            .lines()
            .find_map(|line| line.strip_prefix("content-length: "))
            .unwrap()
            .parse()
            .unwrap();
        let mut body = vec![0_u8; length];
        stream.read_exact(&mut body).unwrap();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: text/plain; charset=utf-8\r\ncontent-length: {}\r\n\
             connection: close\r\n\r\n{answering_offer}\n",
            answering_offer.len() + 1
        )
        .unwrap();
        (who, head, String::from_utf8(body).unwrap())
    });

    let mut at_reception = Proposals::on(reception());
    let waiting = crossing::propose(
        &mut at_reception,
        &found,
        &[MayAskIts::Models],
        Duration::from_secs(86_400),
        SystemTime::now(),
    )
    .unwrap();
    let proposal = waiting.proposal().clone();
    assert!(
        waiting.code().is_some(),
        "no code once the answer came back"
    );

    let (who, head, body) = read_off_the_wire.join().unwrap();
    assert_eq!(
        who.ip(),
        at.ip(),
        "dialled from somewhere other than this host"
    );
    let mut lines = head.lines();
    assert_eq!(
        lines.next().unwrap(),
        format!("POST {THE_PROPOSAL_PATH} HTTP/1.1")
    );
    for header in lines.filter(|line| !line.is_empty()) {
        let (name, _) = header.split_once(':').unwrap();
        assert!(
            ["host", "content-type", "content-length", "connection"].contains(&name),
            "the request carries `{name}`, which is more than framing"
        );
    }

    // The body is the proposal's one line and nothing else.
    assert_eq!(body, format!("{}\n", proposal.said()));
    let fields: Vec<&str> = body.trim().split(' ').collect();
    assert_eq!(fields.len(), 7, "{body}");
    assert_eq!(fields.first().copied(), Some("alo-os/1"));
    assert_eq!(fields.get(1).copied(), Some("proposal"));
    assert_eq!(fields.get(2).copied(), Some(reception().as_str()));
    assert_eq!(fields.get(3).copied(), Some(studio().as_str()));
    assert_eq!(fields.get(4).copied(), Some("86400"));
    assert_eq!(
        fields.get(5).copied(),
        Some(proposal.offered().said().as_str())
    );
    assert_eq!(fields.get(6).copied(), Some("models"));
    assert_eq!(Proposal::read(&body).unwrap(), proposal);
    // And a field not on that list is refused by whoever reads it.
    assert!(matches!(
        Proposal::read(&format!("{} name=disan", body.trim())).unwrap_err(),
        NotNearby::NotAProposal(_)
    ));
}

/// **The asked machine answers with its own `Offer` and nothing else, and
/// only once its person has been shown the proposal and the `Code`.** The
/// request is put by hand so the reply is read off the wire: its body is
/// sixty-four hexadecimal characters and a newline, it is not reception's
/// offer reflected, and the surface was handed the proposal and the code —
/// the code reception then derives from that answer.
#[test]
fn the_asked_machine_answers_with_its_own_offer_and_nothing_else_once_its_person_has_been_shown_it()
{
    let (at_studio, studio_hears, studio_port) = a_machine(studio());
    at_studio
        .lock()
        .unwrap()
        .found
        .push(found_by_discovery(reception(), 7_611));
    let shown = somebody();
    let heard = hearing(studio_hears, Arc::clone(&at_studio), Arc::clone(&shown), 1);

    let (proposal, keying) = a_proposal_to(studio());
    let (status, body) = put_by_hand(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), studio_port),
        THE_PROPOSAL_PATH,
        &format!("{}\n", proposal.said()),
    );
    assert_eq!(status, 200);
    assert_eq!(body.len(), 65, "{body:?}");
    assert!(body.ends_with('\n'));
    let offer = Offer::read(body.trim()).unwrap();
    assert_ne!(
        &offer,
        proposal.offered(),
        "reception's offer reflected back"
    );

    assert_eq!(
        heard.join().unwrap(),
        [Heard::AProposal { from: reception() }]
    );
    let shown = shown.lock().unwrap();
    assert_eq!(shown.what.len(), 1);
    let (what, code) = shown.what.first().unwrap();
    assert_eq!(what, &proposal);
    let at_reception = Deliberating::asking(proposal, keying)
        .unwrap()
        .answered_with(offer)
        .unwrap();
    assert_eq!(code.as_ref(), at_reception.code().as_ref());
    assert!(at_studio.lock().unwrap().pairings.every().is_empty());
}

/// A proposal the surface cannot show to anybody is refused rather than
/// answered to an empty room, and nothing waits.
#[test]
fn a_proposal_nobody_can_be_shown_is_refused_and_nothing_waits() {
    let (at_studio, studio_hears, studio_port) = a_machine(studio());
    at_studio
        .lock()
        .unwrap()
        .found
        .push(found_by_discovery(reception(), 7_611));
    let nobody = Arc::new(Mutex::new(Shown {
        what: Vec::new(),
        somebody_here: false,
    }));
    let heard = hearing(studio_hears, Arc::clone(&at_studio), Arc::clone(&nobody), 1);

    let (proposal, _) = a_proposal_to(studio());
    let (status, body) = put_by_hand(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), studio_port),
        THE_PROPOSAL_PATH,
        &format!("{}\n", proposal.said()),
    );
    assert_eq!(status, 503);
    assert_eq!(
        NotProposed::off_the_wire(&body),
        NotProposed::NobodyToShowItTo
    );
    assert_eq!(
        heard.join().unwrap(),
        [Heard::Refused(NotProposed::NobodyToShowItTo)]
    );
    assert!(at_studio.lock().unwrap().proposals.every().is_empty());
    assert_eq!(nobody.lock().unwrap().what.len(), 1);
}

/// **Both people are shown the same code, and a pairing is kept on each
/// machine only after both have confirmed on their own** — and the pairing
/// works: a proof made at reception verifies at the studio.
#[test]
fn both_people_are_shown_the_same_code_and_a_pairing_is_kept_on_each_machine_only_after_both_confirmed()
 {
    let ((at_reception, reception_hears), (at_studio, studio_hears), shown) = both_machines();
    let reception_heard = hearing(reception_hears, Arc::clone(&at_reception), somebody(), 1);
    let studio_heard = hearing(studio_hears, Arc::clone(&at_studio), Arc::clone(&shown), 2);

    let code_at_reception = reception_proposes(&at_reception).unwrap().unwrap();
    let code_at_the_studio = shown
        .lock()
        .unwrap()
        .what
        .first()
        .unwrap()
        .1
        .clone()
        .unwrap();
    assert_eq!(code_at_reception, code_at_the_studio);
    assert_eq!(code_at_reception.digits().len(), 6);

    // Reception's person confirms; nothing is kept anywhere yet.
    assert!(!confirms(&at_reception, &studio()).unwrap());
    {
        let held = at_studio.lock().unwrap();
        assert!(held.pairings.every().is_empty());
        let waiting = held.proposals.with(&reception()).unwrap();
        assert!(waiting.confirmed_there() && !waiting.confirmed_here());
    }
    assert!(at_reception.lock().unwrap().pairings.every().is_empty());

    // The studio's person confirms: kept there, and kept at reception once
    // the confirmation arrives.
    assert!(confirms(&at_studio, &reception()).unwrap());
    let heard = reception_heard.join().unwrap();
    assert!(
        matches!(heard.as_slice(), [Heard::AConfirmation { from, kept: Some(_) }] if *from == studio()),
        "{heard:?}"
    );
    let studio_heard = studio_heard.join().unwrap();
    assert!(
        matches!(
            studio_heard.as_slice(),
            [
                Heard::AProposal { .. },
                Heard::AConfirmation { kept: None, .. }
            ]
        ),
        "{studio_heard:?}"
    );

    let at_reception = at_reception.lock().unwrap();
    let at_studio = at_studio.lock().unwrap();
    assert!(at_reception.proposals.every().is_empty());
    assert!(at_studio.proposals.every().is_empty());
    let on_reception = at_reception
        .pairings
        .with(&studio(), SystemTime::now())
        .unwrap();
    assert!(
        at_studio
            .pairings
            .paired_with(&reception(), SystemTime::now())
    );
    assert!(
        at_studio
            .pairings
            .permits(&reception(), MayAskIts::Models, SystemTime::now())
    );

    let proof = Proof::made(on_reception, &reception(), b"a question", SystemTime::now());
    assert!(
        Proven::checked(
            &at_studio.pairings,
            &studio(),
            &proof,
            b"a question",
            SystemTime::now(),
            &mut Seen::nothing()
        )
        .is_ok()
    );
}

/// **The asking side confirming alone keeps nothing on either machine.**
#[test]
fn the_asking_side_confirming_alone_keeps_nothing_on_either_machine() {
    let ((at_reception, _), (at_studio, studio_hears), shown) = both_machines();
    let studio_heard = hearing(studio_hears, Arc::clone(&at_studio), shown, 3);

    reception_proposes(&at_reception).unwrap().unwrap();
    assert!(!confirms(&at_reception, &studio()).unwrap());
    // Twice, even.
    assert!(!confirms(&at_reception, &studio()).unwrap());
    studio_heard.join().unwrap();

    let at_reception = at_reception.lock().unwrap();
    let at_studio = at_studio.lock().unwrap();
    assert!(at_reception.pairings.every().is_empty());
    assert!(at_studio.pairings.every().is_empty());
    assert_eq!(at_reception.proposals.every().len(), 1);
    assert_eq!(at_studio.proposals.every().len(), 1);
}

/// **The asked side confirming alone keeps nothing on either machine.**
#[test]
fn the_asked_side_confirming_alone_keeps_nothing_on_either_machine() {
    let ((at_reception, reception_hears), (at_studio, studio_hears), shown) = both_machines();
    let reception_heard = hearing(reception_hears, Arc::clone(&at_reception), somebody(), 1);
    let studio_heard = hearing(studio_hears, Arc::clone(&at_studio), shown, 1);

    reception_proposes(&at_reception).unwrap().unwrap();
    assert!(!confirms(&at_studio, &reception()).unwrap());
    reception_heard.join().unwrap();
    studio_heard.join().unwrap();

    let at_reception = at_reception.lock().unwrap();
    let at_studio = at_studio.lock().unwrap();
    assert!(at_reception.pairings.every().is_empty());
    assert!(at_studio.pairings.every().is_empty());
    let waiting = at_reception.proposals.with(&studio()).unwrap();
    assert!(waiting.confirmed_there() && !waiting.confirmed_here());
}

/// **A proposal from an address discovery never measured is refused before
/// anybody is shown anything** — the studio never found reception, or found
/// it somewhere else — and nothing waits on either machine.
#[test]
fn a_proposal_from_an_address_discovery_never_measured_is_refused_before_anybody_is_shown_anything()
{
    for what_the_studio_found in [
        Vec::new(),
        vec![Found::seen(
            reception(),
            7_611,
            Ipv4Addr::new(192, 168, 1, 40).into(),
        )],
    ] {
        let (at_reception, _, _) = a_machine(reception());
        let (at_studio, studio_hears, studio_port) = a_machine(studio());
        at_reception
            .lock()
            .unwrap()
            .found
            .push(found_by_discovery(studio(), studio_port));
        at_studio.lock().unwrap().found = what_the_studio_found;
        let shown = somebody();
        let heard = hearing(studio_hears, Arc::clone(&at_studio), Arc::clone(&shown), 1);

        assert_eq!(
            reception_proposes(&at_reception).unwrap_err(),
            NotProposed::NotFromWhereItWasFound
        );
        assert_eq!(
            heard.join().unwrap(),
            [Heard::Refused(NotProposed::NotFromWhereItWasFound)]
        );
        assert!(shown.lock().unwrap().what.is_empty());
        assert!(at_studio.lock().unwrap().proposals.every().is_empty());
        assert!(at_reception.lock().unwrap().proposals.every().is_empty());
    }
}

/// **A proposal naming a machine other than the one it arrived at is
/// refused before anybody is shown anything.**
#[test]
fn a_proposal_naming_another_machine_is_refused_before_anybody_is_shown_anything() {
    let (at_studio, studio_hears, studio_port) = a_machine(studio());
    at_studio
        .lock()
        .unwrap()
        .found
        .push(found_by_discovery(reception(), 7_611));
    let shown = somebody();
    let heard = hearing(studio_hears, Arc::clone(&at_studio), Arc::clone(&shown), 1);

    let (for_a_stranger, _) = a_proposal_to(a_stranger());
    let (status, body) = put_by_hand(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), studio_port),
        THE_PROPOSAL_PATH,
        &format!("{}\n", for_a_stranger.said()),
    );
    assert_eq!(status, 400);
    assert_eq!(
        NotProposed::off_the_wire(&body),
        NotProposed::ForAnotherMachine
    );
    assert_eq!(
        heard.join().unwrap(),
        [Heard::Refused(NotProposed::ForAnotherMachine)]
    );
    assert!(shown.lock().unwrap().what.is_empty());
    assert!(at_studio.lock().unwrap().proposals.every().is_empty());
}

/// **A proposal whose `Offer` does not read is refused before anybody is
/// shown anything.**
#[test]
fn a_proposal_whose_offer_does_not_read_is_refused_before_anybody_is_shown_anything() {
    let (at_studio, studio_hears, studio_port) = a_machine(studio());
    at_studio
        .lock()
        .unwrap()
        .found
        .push(found_by_discovery(reception(), 7_611));
    let shown = somebody();
    let heard = hearing(studio_hears, Arc::clone(&at_studio), Arc::clone(&shown), 1);

    let (proposal, _) = a_proposal_to(studio());
    let said = proposal.said();
    let with_no_offer = said.replace(&proposal.offered().said(), "not-an-offer");
    assert_ne!(with_no_offer, said);
    let (status, body) = put_by_hand(
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), studio_port),
        THE_PROPOSAL_PATH,
        &format!("{with_no_offer}\n"),
    );
    assert_eq!(status, 400);
    assert!(matches!(
        NotProposed::off_the_wire(&body),
        NotProposed::Underneath(_)
    ));
    assert!(matches!(
        heard.join().unwrap().as_slice(),
        [Heard::Refused(NotProposed::Underneath(
            NotNearby::NotAnOffer(_)
        ))]
    ));
    assert!(shown.lock().unwrap().what.is_empty());
    assert!(at_studio.lock().unwrap().proposals.every().is_empty());
}

/// **A second proposal from the same machine while the first waits is
/// refused**, at the studio and on the wire: reception withdraws its own
/// copy and proposes again, and the studio, still holding the first, says
/// so — with its person shown nothing the second time.
#[test]
fn a_second_proposal_from_the_same_machine_while_the_first_waits_is_refused() {
    let ((at_reception, _), (at_studio, studio_hears), shown) = both_machines();
    let heard = hearing(studio_hears, Arc::clone(&at_studio), Arc::clone(&shown), 2);

    reception_proposes(&at_reception).unwrap().unwrap();
    // Before withdrawing, reception's own list refuses without the wire.
    assert_eq!(
        reception_proposes(&at_reception).unwrap_err(),
        NotProposed::AlreadyWaiting
    );
    assert!(at_reception.lock().unwrap().proposals.withdrawn(&studio()));
    assert_eq!(
        reception_proposes(&at_reception).unwrap_err(),
        NotProposed::AlreadyWaiting
    );

    let heard = heard.join().unwrap();
    assert!(
        matches!(
            heard.as_slice(),
            [
                Heard::AProposal { .. },
                Heard::Refused(NotProposed::AlreadyWaiting)
            ]
        ),
        "{heard:?}"
    );
    assert_eq!(shown.lock().unwrap().what.len(), 1);
    assert_eq!(at_studio.lock().unwrap().proposals.every().len(), 1);
    assert!(at_reception.lock().unwrap().proposals.every().is_empty());
}

/// **A proposal nobody answered lapses from both machines within the stated
/// time** — stated once, as ten minutes, and read back from the value a
/// surface shows.
#[test]
fn a_proposal_nobody_answered_lapses_from_both_machines_within_the_stated_time() {
    assert_eq!(WHILE_A_PROPOSAL_WAITS, Duration::from_secs(600));
    let ((at_reception, _), (at_studio, studio_hears), shown) = both_machines();
    let heard = hearing(studio_hears, Arc::clone(&at_studio), shown, 1);
    let began = SystemTime::now();
    reception_proposes(&at_reception).unwrap().unwrap();
    heard.join().unwrap();

    let mut at_reception = at_reception.lock().unwrap();
    let mut at_studio = at_studio.lock().unwrap();
    for held in [&at_reception, &at_studio] {
        let waiting = held.proposals.every().first().unwrap();
        assert!(waiting.until() >= began + WHILE_A_PROPOSAL_WAITS);
        assert!(waiting.until() <= SystemTime::now() + WHILE_A_PROPOSAL_WAITS);
    }
    let still = SystemTime::now();
    assert_eq!(at_reception.proposals.lapsed(still), 0);
    assert_eq!(at_studio.proposals.lapsed(still), 0);

    let the_stated_time_later = SystemTime::now() + WHILE_A_PROPOSAL_WAITS;
    assert_eq!(at_reception.proposals.lapsed(the_stated_time_later), 1);
    assert_eq!(at_studio.proposals.lapsed(the_stated_time_later), 1);
    assert!(at_reception.proposals.every().is_empty());
    assert!(at_studio.proposals.every().is_empty());
    assert!(at_reception.pairings.every().is_empty());
    assert!(at_studio.pairings.every().is_empty());
}

/// **Nothing is kept on either machine until a pairing is made.** The record
/// this crate has is the list of pairings, and through a proposal refused,
/// one answered and left, and one confirmed on one side only, both lists
/// stay empty; what the wire reports for each is a refusal or a proposal,
/// never something kept.
#[test]
fn nothing_is_kept_on_either_machine_until_a_pairing_is_made() {
    let ((at_reception, reception_hears), (at_studio, studio_hears), shown) = both_machines();
    let reception_heard = hearing(reception_hears, Arc::clone(&at_reception), somebody(), 1);
    let studio_heard = hearing(studio_hears, Arc::clone(&at_studio), Arc::clone(&shown), 3);

    // Refused: a proposal naming somebody else, put by hand.
    let studio_at = the_one_found(&at_reception).where_it_answers();
    let (for_a_stranger, _) = a_proposal_to(a_stranger());
    put_by_hand(
        studio_at,
        THE_PROPOSAL_PATH,
        &format!("{}\n", for_a_stranger.said()),
    );
    // Answered, and then confirmed at reception only.
    reception_proposes(&at_reception).unwrap().unwrap();
    assert!(!confirms(&at_reception, &studio()).unwrap());
    // And confirmed at the studio too — which is the first thing kept.
    assert!(confirms(&at_studio, &reception()).unwrap());

    let studio_heard = studio_heard.join().unwrap();
    let reception_heard = reception_heard.join().unwrap();
    let kept_before_the_last = studio_heard
        .iter()
        .any(|heard| matches!(heard, Heard::AConfirmation { kept: Some(_), .. }));
    assert!(!kept_before_the_last, "{studio_heard:?}");
    assert!(
        matches!(
            reception_heard.as_slice(),
            [Heard::AConfirmation { kept: Some(_), .. }]
        ),
        "{reception_heard:?}"
    );
    assert_eq!(at_reception.lock().unwrap().pairings.every().len(), 1);
    assert_eq!(at_studio.lock().unwrap().pairings.every().len(), 1);
}
