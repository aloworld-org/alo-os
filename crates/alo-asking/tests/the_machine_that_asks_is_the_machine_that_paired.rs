//! *The machine that asks is the machine that paired* — the corridor's end of
//! [ADR 0031], walked over a real socket.
//!
//! Task 3 built the corridor and said outright that nothing proved the
//! question came from the machine the studio wrote down. This file is that
//! proof, seen from the studio: it names the reception machine in its record
//! **because the connection proved it**, and a record that would have been
//! written on the strength of an identity alone stays empty.
//!
//! | The acceptance | The test |
//! |---|---|
//! | the studio's record names reception because the connection proved it | [`the_studio_names_reception_in_its_record_because_the_connection_proved_it`] |
//! | a stranger presenting reception's identity is refused, and the record stays empty | [`a_stranger_presenting_receptions_identity_is_refused_and_the_record_stays_empty`] |
//! | a question replayed off the wire is refused, and the record gains nothing | [`a_question_replayed_off_the_wire_is_refused_and_the_record_gains_nothing`] |
//! | a pairing revoked at the studio refuses the very next question | [`a_pairing_revoked_at_the_studio_refuses_the_very_next_question`] |
//!
//! [ADR 0031]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0031-the-pairing-is-the-key.md

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs as _};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::{Answering, WentWrong};
use alo_asking::{Asking, DownTheCorridor, NotAsked, Question, THE_PROOF_HEADER};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::SourcePolicy;
use alo_nearby::{
    Deliberating, Keying, MachineId, MayAskIts, Pairing, Pairings, Proof, Proposal, Proven, Seen,
    Side,
};
use alo_record::{Entry, Only, Record};

/// The machine at reception, with no GPU in it.
fn reception() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The studio machine, with the GPU in it.
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

/// A moment to reason from.
fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
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

/// The studio, answering `how_many` questions, judging each against
/// `pairings` at `now` before answering, and writing its record only for the
/// ones that proved where they came from. Hands back the record.
///
/// This is what a studio does with a question (ADR 0031): the proof in the
/// header is checked over the exact bytes that arrived, and a question that
/// does not prove itself is refused with nothing answered and nothing written.
fn a_studio_answering(
    how_many: usize,
    mut pairings: Pairings,
    revoking_after: Option<usize>,
    now: SystemTime,
) -> (SocketAddr, thread::JoinHandle<Record>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let at = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let mut record = Record::default();
        let mut seen = Seen::nothing();
        for nth in 0..how_many {
            if revoking_after == Some(nth) {
                assert!(pairings.revoke(&reception()));
            }
            let (mut stream, _) = listener.accept().unwrap();
            let (proof, body) = read_a_request(&stream);
            let proven = proof.as_ref().and_then(|proof| {
                Proven::checked(&pairings, &the_studio(), proof, &body, now, &mut seen).ok()
            });
            let (status, answer) = match proven {
                Some(proven) => {
                    assert_eq!(proven.machine(), &reception());
                    record.keep(Entry::answered_for(RECEPTION, now));
                    (
                        "200 OK",
                        r#"{"choices":[{"message":{"role":"assistant","content":"Yes."}}]}"#,
                    )
                }
                None => ("400 Bad Request", "{}"),
            };
            write!(
                stream,
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{answer}",
                answer.len()
            )
            .unwrap();
            stream.flush().unwrap();
        }
        record
    });
    (at, handle)
}

/// One request off a socket: the proof in its header, if it carried one, and
/// the exact bytes of its body.
fn read_a_request(stream: &TcpStream) -> (Option<Proof>, Vec<u8>) {
    let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
    let mut length = 0_usize;
    let mut proof = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }
        let lowered = line.to_ascii_lowercase();
        if let Some(how_long) = lowered.strip_prefix("content-length:") {
            length = how_long.trim().parse().unwrap_or(0);
        }
        if let Some(said) = lowered.strip_prefix(&format!("{THE_PROOF_HEADER}:")) {
            proof = Proof::read(said.trim()).ok();
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
    }
    let mut body = vec![0_u8; length];
    if length > 0 {
        reader.read_exact(&mut body).unwrap();
    }
    (proof, body)
}

/// Where a door would connect, resolved by the caller before anything is
/// asked (ADR 0020).
fn resolved(corridor: &DownTheCorridor<'_>) -> Vec<SocketAddr> {
    corridor
        .where_it_would_connect()
        .into_iter()
        .flat_map(|(host, port)| {
            (host.as_str(), port)
                .to_socket_addrs()
                .into_iter()
                .flatten()
        })
        .collect()
}

/// One question down the corridor from `pairings`, as `here`, at `now`.
fn asked_from(
    pairings: &Pairings,
    here: &MachineId,
    at: SocketAddr,
    now: SystemTime,
) -> Result<String, NotAsked> {
    let corridor =
        DownTheCorridor::paired(pairings, here, &the_studio(), THE_STUDIO, at, None, now).unwrap();
    let to = resolved(&corridor);
    let mail = Grantee::named("@mail");
    let answering = Answering::chosen(corridor.source(), &SourcePolicy::InTheBuilding).unwrap();
    let question = Question::asked("may the tenant sublet?", "a-big-model").unwrap();
    let mut indicator = Indicator::default();
    Asking::by(&mail, answering, &[], &SourcePolicy::InTheBuilding)
        .to_a_paired_machine(&question, &corridor, &mut indicator, now, &to)
        .map(|asked| asked.ended(&mut indicator).text().to_owned())
}

/// How many entries a record holds about other machines.
fn from_another_machine(record: &Record) -> usize {
    record
        .answering(&alo_record::Asking::anything().only(Only::FromAnotherMachine))
        .count()
}

/// **The studio names reception because the connection proved it.** The
/// question travelled with a proof, the studio checked it against its own
/// pairing, and only then wrote down that it answered for the reception
/// machine.
#[test]
fn the_studio_names_reception_in_its_record_because_the_connection_proved_it() {
    let (on_reception, on_studio) = paired_between(reception(), the_studio());
    let mut at_reception = Pairings::none();
    at_reception.keep(on_reception);
    let mut at_studio = Pairings::none();
    at_studio.keep(on_studio);
    let (at, studio) = a_studio_answering(1, at_studio, None, a_moment());

    let answer = asked_from(&at_reception, &reception(), at, a_moment()).unwrap();
    let record = studio.join().unwrap();

    assert_eq!(answer, "Yes.");
    assert_eq!(from_another_machine(&record), 1);
    let entry = record.everything().next().unwrap();
    assert!(
        matches!(
            entry.happened(),
            alo_record::Happened::AnsweredForAnotherMachine { origin } if origin.as_str() == RECEPTION
        ),
        "{:?}",
        entry.happened()
    );
}

/// **A stranger presenting reception's identity is refused, and the record
/// stays empty.** The stranger has the address, the shape of the request and
/// reception's `MachineId` — everything the network gives away — and a key of
/// its own from a pairing the studio's person never made.
#[test]
fn a_stranger_presenting_receptions_identity_is_refused_and_the_record_stays_empty() {
    let (_, on_studio) = paired_between(reception(), the_studio());
    let mut at_studio = Pairings::none();
    at_studio.keep(on_studio);
    let (at, studio) = a_studio_answering(1, at_studio, None, a_moment());

    // The stranger's own row names the studio, so its corridor opens — on its
    // own machine, whose pairings nobody at the studio agreed to.
    let (strangers_row, _) = paired_between(a_stranger(), the_studio());
    let mut strangers_pairings = Pairings::none();
    strangers_pairings.keep(strangers_row);
    let refused = asked_from(&strangers_pairings, &reception(), at, a_moment()).unwrap_err();
    let record = studio.join().unwrap();

    assert!(
        matches!(&refused, NotAsked::DidNotAnswer(did_not) if did_not.failed().why() == WentWrong::NothingUsable),
        "{refused:?}"
    );
    assert_eq!(
        from_another_machine(&record),
        0,
        "the studio named reception on the strength of an identity alone"
    );
    assert_eq!(record.everything().count(), 0);
}

/// **A question replayed off the wire is refused, and the record gains
/// nothing.** Whoever recorded reception's bytes sends them again, exactly;
/// the studio has seen that proof and answers nothing.
#[test]
fn a_question_replayed_off_the_wire_is_refused_and_the_record_gains_nothing() {
    let (on_reception, on_studio) = paired_between(reception(), the_studio());
    let mut at_reception = Pairings::none();
    at_reception.keep(on_reception);
    let mut at_studio = Pairings::none();
    at_studio.keep(on_studio);
    let (at, studio) = a_studio_answering(2, at_studio, None, a_moment());

    // A recording of the real question, made by standing in the road.
    let recorder = TcpListener::bind("127.0.0.1:0").unwrap();
    let recorded_at = recorder.local_addr().unwrap();
    let recording = thread::spawn(move || {
        let (mut from_reception, _) = recorder.accept().unwrap();
        let mut bytes = Vec::new();
        let mut reader = std::io::BufReader::new(from_reception.try_clone().unwrap());
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
        // Passed on to the studio as it was, and the answer carried back.
        let mut to_the_studio = TcpStream::connect(at).unwrap();
        to_the_studio.write_all(&bytes).unwrap();
        let mut answer = Vec::new();
        to_the_studio.read_to_end(&mut answer).unwrap();
        from_reception.write_all(&answer).unwrap();
        bytes
    });

    let answer = asked_from(&at_reception, &reception(), recorded_at, a_moment()).unwrap();
    assert_eq!(answer, "Yes.");
    let bytes = recording.join().unwrap();

    // And sent again, exactly, a moment later.
    let mut again = TcpStream::connect(at).unwrap();
    again.write_all(&bytes).unwrap();
    let mut reply = String::new();
    again.read_to_string(&mut reply).unwrap();
    let record = studio.join().unwrap();

    assert!(reply.starts_with("HTTP/1.1 400"), "{reply}");
    assert_eq!(
        from_another_machine(&record),
        1,
        "the replayed question was written down as a second one"
    );
}

/// **A pairing revoked at the studio refuses the very next question**, with
/// reception's key still in reception's hand: the key is on the row, and the
/// row is gone.
#[test]
fn a_pairing_revoked_at_the_studio_refuses_the_very_next_question() {
    let (on_reception, on_studio) = paired_between(reception(), the_studio());
    let mut at_reception = Pairings::none();
    at_reception.keep(on_reception);
    let mut at_studio = Pairings::none();
    at_studio.keep(on_studio);
    let (at, studio) = a_studio_answering(2, at_studio, Some(1), a_moment());

    let before = asked_from(&at_reception, &reception(), at, a_moment()).unwrap();
    let after = asked_from(
        &at_reception,
        &reception(),
        at,
        a_moment() + Duration::from_secs(1),
    )
    .unwrap_err();
    let record = studio.join().unwrap();

    assert_eq!(before, "Yes.");
    assert!(
        matches!(&after, NotAsked::DidNotAnswer(did_not) if did_not.failed().why() == WentWrong::NothingUsable),
        "{after:?}"
    );
    assert_eq!(from_another_machine(&record), 1);
}
