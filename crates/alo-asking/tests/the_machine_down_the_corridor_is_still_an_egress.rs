//! *A machine without a GPU discovers the one with it, and the agents just
//! work. The inference never leaves the building; it moves down the corridor.*
//!
//! And the half of that promise which costs something: **it is still egress,
//! and the indicator still fires** (ADR 0003).
//!
//! | The acceptance | The test |
//! |---|---|
//! | a machine with no model of its own asks a paired one, and the answer comes back | [`a_machine_with_no_model_of_its_own_is_answered_down_the_corridor`] |
//! | the indicator fires for it — beside a local answer where it does not | [`the_indicator_fires_for_the_corridor_and_stays_quiet_for_this_machine`] |
//! | what a person is shown names the machine that answered | [`what_a_person_is_shown_names_the_machine_down_the_corridor`] |
//! | an unpaired machine offering inference is not used | [`an_unpaired_machine_offering_inference_is_not_used_however_convenient`] |
//! | the answering machine's record names the machine that asked | [`the_machine_that_answered_writes_down_which_machine_asked`] |
//! | and it is never a fallback for a model that was not there | [`the_machine_down_the_corridor_is_not_a_fallback_for_anything`] |
//!
//! # The sentence being refused
//!
//! *"It only went to the machine down the hall."* Every one of these tests is
//! about that sentence. The corridor is short, the machine belongs to the same
//! office, the question never touches the internet — and none of that makes the
//! question not have left. A person who cannot see the short journeys cannot
//! check any of the claims made about the long ones.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, ToSocketAddrs as _};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_answering::Answering;
use alo_asking::{Asking, DownTheCorridor, Question};
use alo_capability::Grantee;
use alo_egress::Indicator;
use alo_models::{InferenceSource, SourcePolicy};
use alo_nearby::{Deliberating, MachineId, MayAskIts, Pairings, Proposal, Side};
use alo_strings::Strings;

/// This machine — the one in the office with no GPU in it.
fn here() -> MachineId {
    MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
}

/// The machine down the corridor, with the GPU in it.
fn the_studio() -> MachineId {
    MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
}

/// What a person called it when they paired with it.
const CALLED: &str = "the studio machine";

/// A moment to reason from, so no test here depends on when it is run.
fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A pairing two people made, permitting this machine to ask its models.
fn paired_for_a_day() -> Pairings {
    let mut pairings = Pairings::none();
    pairings.keep(
        Deliberating::of(
            Proposal::checked(
                here(),
                the_studio(),
                &[MayAskIts::Models],
                Duration::from_secs(86_400),
            )
            .unwrap(),
        )
        .agreed_at(Side::TheOneAsking)
        .agreed_at(Side::TheOneAsked)
        .agreed(Side::TheOneAsking, a_moment())
        .unwrap(),
    );
    pairings
}

/// A machine on the network answering one question, and handing back what it
/// was sent so the request can be read.
fn a_machine_answering(said: &'static str) -> (SocketAddr, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let at = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
        let mut head = String::new();
        let mut length = 0_usize;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            if let Some(how_long) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = how_long.trim().parse().unwrap_or(0);
            }
            let done = line == "\r\n" || line == "\n";
            head.push_str(&line);
            if done {
                break;
            }
        }
        let mut body = vec![0_u8; length];
        if length > 0 {
            reader.read_exact(&mut body).unwrap();
        }
        head.push_str(&String::from_utf8_lossy(&body));
        let answer =
            format!(r#"{{"choices":[{{"message":{{"role":"assistant","content":"{said}"}}}}]}}"#);
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{answer}",
            answer.len()
        )
        .unwrap();
        stream.flush().unwrap();
        head
    });
    (at, handle)
}

/// Where a door would connect, resolved by the caller before anything is asked
/// (ADR 0020).
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

/// **The promise, end to end.** A machine with no model of its own puts a
/// question to the paired machine that has one, and the answer comes back.
#[test]
fn a_machine_with_no_model_of_its_own_is_answered_down_the_corridor() {
    let (at, studio) = a_machine_answering("Yes, and here is why.");
    let corridor = DownTheCorridor::paired(
        &paired_for_a_day(),
        &the_studio(),
        CALLED,
        at,
        None,
        a_moment(),
    )
    .unwrap();
    let to = resolved(&corridor);

    let mail = Grantee::named("@mail");
    let answering = Answering::chosen(corridor.source(), &SourcePolicy::Anywhere).unwrap();
    let question = Question::asked("may the tenant sublet?", "a-big-model").unwrap();
    let mut indicator = Indicator::default();

    let asked = Asking::by(&mail, answering, &[], &SourcePolicy::Anywhere)
        .to_a_paired_machine(&question, &corridor, &mut indicator, a_moment(), &to)
        .unwrap();
    let answer = asked.ended(&mut indicator);

    let sent = studio.join().unwrap();
    assert!(sent.contains("may the tenant sublet?"), "{sent}");
    assert_eq!(answer.text(), "Yes, and here is why.");
    assert_eq!(
        *answer.source(),
        InferenceSource::PairedMachine {
            machine: CALLED.to_owned()
        }
    );
}

/// **The indicator fires for the corridor, and stays quiet for this machine.**
///
/// The two are asserted in one test on purpose. Either alone proves nothing:
/// an indicator that fires for everything is no signal, and one that is quiet
/// for everything is a lie. What the promise rests on is the difference.
#[test]
fn the_indicator_fires_for_the_corridor_and_stays_quiet_for_this_machine() {
    let (at, studio) = a_machine_answering("Down the corridor.");
    let corridor = DownTheCorridor::paired(
        &paired_for_a_day(),
        &the_studio(),
        CALLED,
        at,
        None,
        a_moment(),
    )
    .unwrap();
    let to = resolved(&corridor);
    let mail = Grantee::named("@mail");
    let question = Question::asked("what is this?", "a-big-model").unwrap();
    let mut indicator = Indicator::default();

    // A question answered on this machine is not a departure at all, so there
    // is nothing to show and nothing is shown.
    assert!(
        alo_egress::Leaving::asking(&mail, &InferenceSource::ThisMachine).is_err(),
        "a question answered on this machine became something that could leave"
    );
    assert!(indicator.is_quiet());

    // One answered down the corridor is, and the person sees it while it
    // happens.
    let answering = Answering::chosen(corridor.source(), &SourcePolicy::Anywhere).unwrap();
    let asked = Asking::by(&mail, answering, &[], &SourcePolicy::Anywhere)
        .to_a_paired_machine(&question, &corridor, &mut indicator, a_moment(), &to)
        .unwrap();

    assert!(
        !indicator.is_quiet(),
        "the question went down the corridor with the indicator quiet"
    );

    drop(asked.ended(&mut indicator));
    drop(studio.join());
    assert!(indicator.is_quiet(), "the line stayed up after it was over");
}

/// **What a person is shown names the machine that answered**, in the name they
/// gave it — and says it is on their network rather than implying it never
/// went anywhere.
#[test]
fn what_a_person_is_shown_names_the_machine_down_the_corridor() {
    let (at, studio) = a_machine_answering("Named.");
    let corridor = DownTheCorridor::paired(
        &paired_for_a_day(),
        &the_studio(),
        CALLED,
        at,
        None,
        a_moment(),
    )
    .unwrap();
    let to = resolved(&corridor);
    let mail = Grantee::named("@mail");
    let strings = Strings::of(alo_egress::egress_words().unwrap());
    let question = Question::asked("who answered?", "a-big-model").unwrap();
    let mut indicator = Indicator::default();

    let answering = Answering::chosen(corridor.source(), &SourcePolicy::Anywhere).unwrap();
    let asked = Asking::by(&mail, answering, &[], &SourcePolicy::Anywhere)
        .to_a_paired_machine(&question, &corridor, &mut indicator, a_moment(), &to)
        .unwrap();

    let shown = indicator.showing().first().unwrap().said(&strings);
    assert_eq!(
        shown.text(),
        format!("@mail is asking a question of {CALLED}, on your network")
    );

    drop(asked.ended(&mut indicator));
    drop(studio.join());
}

/// **An unpaired machine offering inference is not used, however convenient.**
///
/// The machine is there, it has the GPU, it would answer, and the answer would
/// be better than this machine's. None of that is a pairing, and there is no
/// constructor that skips one.
#[test]
fn an_unpaired_machine_offering_inference_is_not_used_however_convenient() {
    let (at, studio) = a_machine_answering("I would have answered.");

    let refused = DownTheCorridor::paired(
        &Pairings::none(),
        &the_studio(),
        CALLED,
        at,
        None,
        a_moment(),
    );
    assert!(
        refused.is_err(),
        "a machine nobody paired with was used because it was there"
    );

    // And nothing was sent: the far side is still waiting, so the test closes
    // it rather than joining a thread that would never return.
    drop(std::net::TcpStream::connect(at));
    drop(studio.join());
}

/// **It is never a fallback.** A question reaches the corridor because the
/// person's permission named that machine, and for no other reason.
///
/// The permission here names a provider. The corridor is right there, paired,
/// and would answer — and the door refuses, because choosing where a question
/// goes is the person's and `docs/features.md` refuses silent substitution four
/// times over.
#[test]
fn the_machine_down_the_corridor_is_not_a_fallback_for_anything() {
    let (at, studio) = a_machine_answering("I should not be asked.");
    let corridor = DownTheCorridor::paired(
        &paired_for_a_day(),
        &the_studio(),
        CALLED,
        at,
        None,
        a_moment(),
    )
    .unwrap();
    let to = resolved(&corridor);
    let mail = Grantee::named("@mail");
    let question = Question::asked("anything", "a-big-model").unwrap();
    let mut indicator = Indicator::default();

    // The person's permission is for a hosted provider, not for the machine
    // down the corridor.
    let elsewhere = InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        region: alo_models::Region::Declared("the EU".to_owned()),
    };
    let answering = Answering::chosen(elsewhere, &SourcePolicy::Anywhere).unwrap();

    let refused = Asking::by(&mail, answering, &[], &SourcePolicy::Anywhere)
        .to_a_paired_machine(&question, &corridor, &mut indicator, a_moment(), &to)
        .unwrap_err();

    assert!(
        refused.nothing_left(),
        "a question permitted elsewhere went down the corridor anyway"
    );
    assert!(
        indicator.is_quiet(),
        "something was shown leaving for a question that was never sent"
    );

    drop(std::net::TcpStream::connect(at));
    drop(studio.join());
}

/// **The machine that answered writes down that it did**, with the machine that
/// asked named as the origin.
///
/// The other end of the corridor. The asking machine's record says something
/// left; the answering machine's says one arrived — so *one GPU box serves the
/// office* is something either person can check from their own machine rather
/// than something one of them has to be told.
///
/// It names the **machine** and not the agent, and that is the decision worth
/// the test. The agent that asked is a name on somebody else's machine, which
/// this one has no way to check; writing it here would put a claim into a record
/// people read as a statement of fact. What this machine can stand behind is
/// which machine it paired with, because its own person agreed to that.
#[test]
fn the_machine_that_answered_writes_down_which_machine_asked() {
    use alo_record::{Entry, Happened};

    let entry = Entry::answered_for("the reception machine", a_moment());

    assert!(
        matches!(
            entry.happened(),
            Happened::AnsweredForAnotherMachine { origin } if origin.as_str() == "the reception machine"
        ),
        "{:?}",
        entry.happened()
    );
    assert!(
        entry.agent().is_none(),
        "this machine's record named an agent it has no way to check"
    );
    assert!(
        entry.what().is_none(),
        "somebody else's question was kept in this machine's record"
    );
    assert!(
        !entry.happened().was_stopped(),
        "answering a paired machine's question was recorded as a refusal"
    );
}

/// A permission naming one paired machine does not open a door to a different
/// one: the person chose which machine, and two machines here would be this one
/// choosing for them.
#[test]
fn a_permission_for_one_machine_does_not_reach_another() {
    let (at, studio) = a_machine_answering("Wrong machine.");
    let corridor = DownTheCorridor::paired(
        &paired_for_a_day(),
        &the_studio(),
        CALLED,
        at,
        None,
        a_moment(),
    )
    .unwrap();
    let to = resolved(&corridor);
    let mail = Grantee::named("@mail");
    let question = Question::asked("anything", "a-big-model").unwrap();
    let mut indicator = Indicator::default();

    let another = InferenceSource::PairedMachine {
        machine: "the reception machine".to_owned(),
    };
    let answering = Answering::chosen(another, &SourcePolicy::Anywhere).unwrap();

    let refused = Asking::by(&mail, answering, &[], &SourcePolicy::Anywhere)
        .to_a_paired_machine(&question, &corridor, &mut indicator, a_moment(), &to)
        .unwrap_err();

    assert!(refused.nothing_left());
    assert!(indicator.is_quiet());

    drop(std::net::TcpStream::connect(at));
    drop(studio.join());
}
