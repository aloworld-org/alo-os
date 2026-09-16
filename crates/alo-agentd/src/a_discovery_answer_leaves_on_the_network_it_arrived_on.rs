//! A discovery answer leaves on the network its question arrived on — on a real
//! kernel, over two `veth` cables carrying one private range, with a machine at
//! the same address at each far end.
//!
//! *Machines find each other with zero configuration, and trust none of them for
//! it.* `crate::listeners` made this machine **reachable** on a network the route
//! does not point at. What still left by the route was the other half of being
//! found — **the answer to *who is here***: `crate::wire` answered discovery on
//! one socket held to no network, and an answer to `10.68.0.2` on the cable went
//! out whichever interface the route picked, to somebody else at that address or
//! to nobody. `crate::responding` holds it: one socket per network, each held to
//! that network's interface. This is that hold, over cables.
//!
//! # The three machines
//!
//! **Reception** is this test binary run again in a network namespace of its own,
//! made by util-linux's `unshare` inside a user namespace — so this needs no
//! root, takes no kernel-global state, and every cable dies with the namespaces
//! (`docs/autonomy/SHARED_MAIN.md`).
//!
//! - At the far end of `cable0` is **the studio**, at `10.68.0.2`, which asks who
//!   is here and then runs the whole service as `src/main.rs` does. Reception's
//!   end is `10.68.0.1`.
//! - At the far end of `other0` is **somebody else**, also at `10.68.0.2`,
//!   answering discovery **as the studio** and counting everything that reaches
//!   it — including, at the port the studio asks from, an answer that was meant
//!   for the studio and left by the route.
//!
//! **Reception's route to `10.68.0.2` goes over `other0`**, the network the
//! studio is *not* on. So everything reception sends that is not held to a
//! network reaches somebody else.
//!
//! # And no rule puts anything back
//!
//! Task 27's fixture kept discovery's answers on the cable with an
//! `ip rule ipproto udp` and a table of its own, because discovery had no hold of
//! its own yet (`docs/quirks.md`). **There is no such rule here.** The main table
//! is the only table, it points at the other network, and what keeps each answer
//! on the cable is the code.
//!
//! # In this order
//!
//! 1. Reception lays both cables, points the route at the other network, and
//!    serves as `src/main.rs` does.
//! 2. The studio asks *who is here* from a socket on a port somebody else is
//!    listening on too. **The studio hears reception's answer and somebody else
//!    hears nothing** — the measurement this task exists for.
//! 3. The studio's person proposes a pairing by identity, which needs reception
//!    to be *found* first: both people confirm, and each machine lists the other
//!    as paired.
//! 4. Somebody else, at the same address on the network the route does point at,
//!    was asked nothing, connected to never, and sent nothing at all.

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::env;
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::net::{Ipv4Addr, TcpListener, UdpSocket};
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::{Duration, Instant, SystemTime};

    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_keeping::Keeping;
    use alo_nearby::{Answering, Looking, MachineId, Presence, THE_ADDRESS, THE_PORT};
    use alo_protocol::{AfterConfirming, ToAPerson};
    use alo_record::Record;
    use alo_turn::Machine;

    use crate::hosting::Hosted;
    use crate::network::TheNetwork;
    use crate::rereading::WhatIsGranted;
    use crate::serving::Serving;
    use crate::side::Side;
    use crate::stopping::{Stop, Waking};
    use crate::surface::AtThePersonsDoor;
    use crate::terms::Terms;
    use crate::testing::{
        NothingIsBounded, NothingIsRemembered, Pretending, a_folder_with_an_invoice, a_message,
        granting, hour, in_english, nothing_has_been_chosen, reception, the_studio,
    };
    use crate::wire::{THE_WIRE_PORT, Wire};

    /// Set on the binary run again inside a namespace, naming which machine it
    /// is.
    const INSIDE: &str = "ALO_AGENTD_DISCOVERY_ANSWER_INSIDE";

    /// Reception's cable to the studio — the network the route does **not**
    /// point at.
    const THE_CABLE: &str = "cable0";
    /// Reception's cable to somebody else — the network the route points at.
    const THE_OTHER_NETWORK: &str = "other0";
    /// The far end of either cable.
    const FAR_END: &str = "far0";

    /// Reception's address on the cable, and on the other network: two networks
    /// out of one private range, as two routers handing it out look.
    const RECEPTION_ON_THE_CABLE: &str = "10.68.0.1/24";
    const RECEPTION_ELSEWHERE: &str = "10.68.0.3/24";
    /// The address at the far end of both cables.
    const FAR_ADDRESS: &str = "10.68.0.2/24";

    /// The port the studio asks *who is here* from, and the one somebody else
    /// listens on to catch an answer that was meant for the studio.
    ///
    /// A fixed port rather than an ephemeral one for exactly that reason: an
    /// answer that leaves by the route goes to `10.68.0.2` at the port it was
    /// asked from, and the only machine that can say whether one arrived there is
    /// a machine listening at it.
    const ASKED_FROM: u16 = 7_712;

    /// How long anything here waits for the kernel or another machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// **A discovery answer leaves on the network the question arrived on**: the
    /// outer test, which makes reception and reads what it saw.
    #[test]
    fn a_discovery_answer_leaves_on_the_network_the_question_arrived_on() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_discovery_answer_leaves_on_the_network_it_arrived_on::tests::reception_answering_on_two_networks";
        let ran = Command::new("unshare")
            .args(["--map-root-user", "--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "reception")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect("util-linux's `unshare` makes the machines");

        let said = String::from_utf8_lossy(&ran.stdout).into_owned();
        assert!(
            ran.status.success(),
            "reception failed ({}):\n{said}",
            ran.status
        );
        for step in [
            "reception answers discovery on both cables",
            "the machine that asked heard the answer",
            "the machine at the same address on the other network heard nothing",
            "paired over the network the route does not point at",
            "somebody else at the same address was asked nothing, connected to never and sent nothing",
        ] {
            assert!(
                said.contains(step),
                "reception never said `{step}`:\n{said}"
            );
        }
    }

    /// Refuse to run unless this is the binary re-run as `which`.
    fn must_be(which: &str) {
        assert_eq!(
            env::var(INSIDE).ok().as_deref(),
            Some(which),
            "this test runs inside the namespace the outer test makes; run the outer one"
        );
    }

    /// Run iproute2 with `args`, in the network namespace of `pid` when one is
    /// named, and fail with what it said if it refuses.
    fn ip(pid: Option<u32>, args: &[&str]) {
        let mut command = match pid {
            Some(pid) => {
                let mut command = Command::new("nsenter");
                command.args(["-t", &pid.to_string(), "-n", "ip"]);
                command
            }
            None => Command::new("ip"),
        };
        let ran = command
            .args(args)
            .output()
            .expect("iproute2's `ip` and util-linux's `nsenter` make the cables");
        assert!(
            ran.status.success(),
            "ip {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&ran.stderr)
        );
    }

    /// A far end in a network namespace nested inside reception's, running the
    /// test named `runs`, with the cable `cable` laid to it and both ends
    /// addressed.
    fn a_far_end(
        which: &str,
        runs: &str,
        cable: &str,
        here: &str,
    ) -> (Child, ChildStdin, BufReader<ChildStdout>) {
        let exe = env::current_exe().unwrap();
        let script = format!(
            "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_discovery_answer_leaves_on_the_network_it_arrived_on::tests::{runs}"
        );
        let mut far = Command::new("unshare")
            .args(["--net", "--", "sh", "-c", &script])
            .arg(exe)
            .env(INSIDE, which)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("util-linux's `unshare` makes the far end");
        let ours = std::fs::read_link("/proc/self/ns/net").unwrap();
        let theirs = format!("/proc/{}/ns/net", far.id());
        let until = Instant::now() + Duration::from_secs(10);
        while std::fs::read_link(&theirs).ok().as_ref() == Some(&ours)
            || !Path::new(&theirs).exists()
        {
            assert!(Instant::now() < until, "{which} never left this network");
            std::thread::sleep(Duration::from_millis(20));
        }
        let pid = far.id().to_string();
        ip(
            None,
            &[
                "link", "add", cable, "type", "veth", "peer", "name", FAR_END, "netns", &pid,
            ],
        );
        ip(
            Some(far.id()),
            &["addr", "add", FAR_ADDRESS, "dev", FAR_END],
        );
        ip(Some(far.id()), &["link", "set", FAR_END, "up"]);
        ip(None, &["addr", "add", here, "dev", cable]);
        ip(None, &["link", "set", cable, "up"]);
        let telling = far.stdin.take().unwrap();
        let hearing = BufReader::new(far.stdout.take().unwrap());
        (far, telling, hearing)
    }

    /// Point reception's route to `10.68.0.2` at the other network, and put
    /// **nothing** back with a rule: this test is about what the code holds.
    fn the_route_points_at_the_other_network() {
        ip(
            None,
            &[
                "route",
                "replace",
                "10.68.0.2/32",
                "dev",
                THE_OTHER_NETWORK,
                "src",
                "10.68.0.3",
            ],
        );
    }

    /// Wait until the kernel says both cables are up and running with an address,
    /// which is what a network the port is listened on and discovery answered on
    /// is made of (`crate::networks::listening_networks`).
    ///
    /// A `veth` is not running until its far end is, and the kernel takes a
    /// moment to say so — a service started in that moment is a service on one
    /// cable, which is a race in the fixture and not a thing about this machine.
    fn both_cables_are_up() {
        let until = Instant::now() + PATIENCE;
        loop {
            let reported = crate::route_messages::reported_by_the_kernel().unwrap();
            let networks = crate::networks::listening_networks(&reported);
            let named: Vec<&str> = networks
                .iter()
                .map(crate::networks::Network::name)
                .collect();
            if [THE_CABLE, THE_OTHER_NETWORK]
                .iter()
                .all(|cable| named.contains(cable))
            {
                return;
            }
            assert!(
                Instant::now() < until,
                "the cables never both came up: {reported:?}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// The next line a far end printed under `prefix`, waiting for it.
    fn said_by(from: &mut BufReader<ChildStdout>, prefix: &str) -> String {
        let mut line = String::new();
        loop {
            line.clear();
            let read = from.read_line(&mut line).unwrap();
            assert!(read > 0, "a far end ended before saying `{prefix}`");
            if let Some(rest) = line.trim_end().strip_prefix(prefix) {
                return rest.trim().to_owned();
            }
        }
    }

    /// Tell a far end one thing.
    fn tell(telling: &mut ChildStdin, what: &str) {
        telling.write_all(format!("{what}\n").as_bytes()).unwrap();
    }

    /// The next thing a far end was told, read a line at a time.
    fn next_line() -> String {
        let mut line = String::new();
        let read = std::io::stdin().read_line(&mut line).unwrap();
        assert!(read > 0, "the machine that drives this one ended");
        line.trim_end().to_owned()
    }

    /// A person's shell, on one connection to the door.
    struct Talking {
        /// What is written to.
        writing: UnixStream,
        /// What is read back.
        reading: BufReader<UnixStream>,
    }

    impl Talking {
        /// Connect to the door at `at`.
        fn to(at: &Path) -> Self {
            let connection = UnixStream::connect(at).unwrap();
            Self {
                reading: BufReader::new(connection.try_clone().unwrap()),
                writing: connection,
            }
        }

        /// Ask one thing and read the answer.
        fn asking(&mut self, asks: &str) -> ToAPerson {
            self.writing
                .write_all(format!("{}\n", a_message(asks)).as_bytes())
                .unwrap();
            let mut back = String::new();
            self.reading.read_line(&mut back).unwrap();
            ToAPerson::read(back.trim_end()).expect("the door answers in its own protocol")
        }

        /// The pairings as the door lists them: who is paired, and each waiting
        /// proposal's machine and code.
        fn pairings(&mut self) -> (Vec<String>, Vec<(String, Option<String>)>) {
            let listed = self.asking(r#"{"pairings":{}}"#);
            let paired = listed
                .paired()
                .unwrap()
                .iter()
                .map(|one| one.machine().to_owned())
                .collect();
            let waiting = listed
                .waiting_to_pair()
                .unwrap()
                .iter()
                .map(|one| (one.machine().to_owned(), one.code().map(str::to_owned)))
                .collect();
            (paired, waiting)
        }
    }

    /// Run this machine as the service runs it — `Wire::bound` as `here`, with a
    /// person's door — while `person` drives the door from a thread that may
    /// borrow the wire, and stop when `person` stops it.
    fn serving_as(here: &MachineId, what: &str, person: impl FnOnce(&Wire, &Path, Stop) + Send) {
        let strings = in_english();
        let (folder, _) = a_folder_with_an_invoice(what);
        let (waking, stop) = Waking::made().unwrap();
        let knocking = Pretending::handing_out(what, &[Some(Side::Person)]);
        let door: PathBuf = knocking.at();
        let wire = Wire::bound(here.clone()).unwrap().hosting(Hosted::Nothing);
        let network = TheNetwork::on(here.clone());
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
        let mut grants = granting(&folder, SystemTime::now());
        let mut questions = nothing_has_been_chosen();
        let terms = Terms {
            for_agent: "@files",
            lasting: hour(),
            standing: hour(),
            keeping: Keeping::Forever,
            policy: EgressPolicy::InTheBuilding,
            naming: network.names(),
        };
        std::thread::scope(|scope| {
            let wire = &wire;
            let door = door.as_path();
            let driving = scope.spawn(move || person(wire, door, stop));
            Serving::of(&knocking, &waking, wire, &network, terms)
                .until_stopped(
                    &mut machine,
                    &mut WhatIsGranted::of(&mut grants, &NothingIsRemembered),
                    &mut questions,
                    &mut AtThePersonsDoor,
                )
                .unwrap();
            driving.join().unwrap();
        });
    }

    /// Reception, answering discovery on two networks with one private range:
    /// the whole of it.
    #[test]
    #[ignore = "run inside its own network by a_discovery_answer_leaves_on_the_network_the_question_arrived_on"]
    fn reception_answering_on_two_networks() {
        must_be("reception");
        let (mut studio, mut telling_studio, mut hearing_studio) = a_far_end(
            "studio",
            "the_studio_at_the_end_of_the_cable",
            THE_CABLE,
            RECEPTION_ON_THE_CABLE,
        );
        let (mut other, mut telling_other, mut hearing_other) = a_far_end(
            "somebody else",
            "somebody_else_at_the_same_address",
            THE_OTHER_NETWORK,
            RECEPTION_ELSEWHERE,
        );
        the_route_points_at_the_other_network();
        tell(&mut telling_other, "serve");
        said_by(&mut hearing_other, "alo:serving");
        both_cables_are_up();

        serving_as(
            &reception(),
            "discovery-answer-reception",
            |wire, door, stop| {
                let answered: Vec<String> = wire
                    .answered_on()
                    .iter()
                    .map(|network| network.name().to_owned())
                    .collect();
                for network in [THE_CABLE, THE_OTHER_NETWORK, "lo"] {
                    assert!(
                        answered.iter().any(|name| name == network),
                        "discovery is not answered on {network}: {answered:?}"
                    );
                }
                let joined: Vec<String> = wire
                    .joined()
                    .iter()
                    .filter(|network| network.address().is_ipv4())
                    .map(|network| network.name().to_owned())
                    .collect();
                assert!(
                    joined.iter().any(|name| name == THE_CABLE)
                        && joined.iter().any(|name| name == THE_OTHER_NETWORK),
                    "the group is not joined on both cables: {joined:?}"
                );
                assert!(
                    !joined.iter().any(|name| name == "lo"),
                    "the group was joined on loopback: {joined:?}"
                );
                println!("reception answers discovery on both cables");

                // 2. The studio asks who is here, from a port somebody else is
                //    listening on too. This is the measurement.
                tell(&mut telling_studio, "ask");
                let heard = said_by(&mut hearing_studio, "alo:heard");
                assert_eq!(
                    heard,
                    reception().as_str(),
                    "the machine that asked did not hear this machine's answer"
                );
                println!("the machine that asked heard the answer");
                tell(&mut telling_other, "strays");
                assert_eq!(
                    said_by(&mut hearing_other, "alo:strays"),
                    "nothing",
                    "an answer meant for the machine on the cable arrived at the same address on the other network"
                );
                println!("the machine at the same address on the other network heard nothing");

                // 3. And end to end: the studio's person proposes a pairing by
                //    identity, which is refused outright unless this machine is
                //    found first.
                tell(&mut telling_studio, "serve");
                said_by(&mut hearing_studio, "alo:serving");
                let mut person = Talking::to(door);
                let (paired, _) = person.pairings();
                assert!(paired.is_empty(), "{paired:?}");

                tell(&mut telling_studio, "propose");
                let until = Instant::now() + PATIENCE;
                let code = loop {
                    let (_, waiting) = person.pairings();
                    if let Some((machine, Some(code))) = waiting.first() {
                        assert_eq!(machine, the_studio().as_str());
                        break code.clone();
                    }
                    assert!(
                        Instant::now() < until,
                        "no proposal from the studio was ever shown here"
                    );
                    std::thread::sleep(Duration::from_millis(100));
                };
                assert_eq!(said_by(&mut hearing_studio, "alo:code"), code);

                let confirmed = person.asking(&format!(
                    r#"{{"confirm-pairing":{{"machine":"{}","code":"{code}"}}}}"#,
                    the_studio().as_str()
                ));
                assert_eq!(
                    confirmed.became_of_confirming(),
                    Some(AfterConfirming::WaitingForTheOtherPerson),
                    "{confirmed:?}"
                );
                tell(&mut telling_studio, "confirm");
                assert_eq!(
                    said_by(&mut hearing_studio, "alo:paired"),
                    reception().as_str()
                );
                let until = Instant::now() + PATIENCE;
                loop {
                    let (paired, _) = person.pairings();
                    if !paired.is_empty() {
                        assert_eq!(paired, vec![the_studio().as_str().to_owned()]);
                        break;
                    }
                    assert!(
                        Instant::now() < until,
                        "the studio's confirmation never arrived"
                    );
                    std::thread::sleep(Duration::from_millis(100));
                }
                println!("paired over the network the route does not point at");

                // 4. And nothing of any of it reached the machine at the same
                //    address on the network the route does point at.
                tell(&mut telling_other, "count");
                assert_eq!(
                    said_by(&mut hearing_other, "alo:counted"),
                    "0 0 0",
                    "somebody else at the same address was asked, connected to or sent something"
                );
                println!(
                    "somebody else at the same address was asked nothing, connected to never and sent nothing"
                );

                tell(&mut telling_studio, "stop");
                said_by(&mut hearing_studio, "alo:stopped");
                tell(&mut telling_other, "stop");
                said_by(&mut hearing_other, "alo:stopped");
                assert!(stop.stop());
            },
        );
        assert!(studio.wait().unwrap().success(), "the studio failed");
        assert!(other.wait().unwrap().success(), "somebody else failed");
    }

    /// The studio, at the far end of the cable: it asks who is here from the
    /// port somebody else is also listening on, then runs the whole service and
    /// proposes a pairing to reception.
    #[test]
    #[ignore = "run inside its own network by reception_answering_on_two_networks"]
    fn the_studio_at_the_end_of_the_cable() {
        must_be("studio");
        loop {
            match next_line().as_str() {
                "ask" => {
                    let socket = UdpSocket::bind(("10.68.0.2", ASKED_FROM)).unwrap();
                    let looking = Looking::from(socket);
                    looking
                        .ask(std::net::SocketAddr::new(THE_ADDRESS.into(), THE_PORT))
                        .unwrap();
                    let found = looking.found(Duration::from_secs(5)).unwrap_or_default();
                    println!(
                        "alo:heard {}",
                        found.first().map_or_else(
                            || "nothing".to_owned(),
                            |one| one.machine.as_str().to_owned()
                        )
                    );
                }
                "serve" => break,
                said => panic!("the studio was told something it does not do: {said}"),
            }
        }

        serving_as(&the_studio(), "discovery-answer-studio", |_, door, stop| {
            let mut person = Talking::to(door);
            let (paired, _) = person.pairings();
            assert!(paired.is_empty(), "{paired:?}");
            println!("alo:serving");

            assert_eq!(next_line(), "propose");
            let proposed = person.asking(&format!(
                r#"{{"pair":{{"machine":"{}","may":["models"],"seconds":600}}}}"#,
                reception().as_str()
            ));
            let waiting = proposed
                .proposed_pairing()
                .expect("the proposal came back with the code");
            println!("alo:code {}", waiting.code().unwrap());

            assert_eq!(next_line(), "confirm");
            let confirmed = person.asking(&format!(
                r#"{{"confirm-pairing":{{"machine":"{}","code":"{}"}}}}"#,
                reception().as_str(),
                waiting.code().unwrap()
            ));
            assert_eq!(
                confirmed.became_of_confirming(),
                Some(AfterConfirming::Paired),
                "{confirmed:?}"
            );
            let (paired, _) = person.pairings();
            assert_eq!(paired, vec![reception().as_str().to_owned()]);
            println!("alo:paired {}", reception().as_str());

            assert_eq!(next_line(), "stop");
            assert!(stop.stop());
            println!("alo:stopped");
        });
    }

    /// Somebody else, at the same address on the network the route points at: it
    /// answers discovery **as the studio**, and counts everything that reaches
    /// it — a question, a connection, and a datagram at the port the studio asks
    /// from, which is where an answer that left by the route would land.
    ///
    /// Nothing about an advertisement proves who sent it (ADR 0003), so the only
    /// thing that tells the two machines apart is which network reached which —
    /// and this machine is the witness that nothing reached it.
    #[test]
    #[ignore = "run inside its own network by reception_answering_on_two_networks"]
    fn somebody_else_at_the_same_address() {
        must_be("somebody else");
        assert_eq!(next_line(), "serve");

        let stopping = Arc::new(AtomicBool::new(false));
        let answers = Arc::new(AtomicUsize::new(0));
        let connections = Arc::new(AtomicUsize::new(0));
        let strays = Arc::new(AtomicUsize::new(0));

        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, THE_PORT)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let answering = Answering::on(socket, Presence::of(the_studio(), THE_WIRE_PORT));
        let counting = Arc::clone(&answers);
        let until = Arc::clone(&stopping);
        std::thread::spawn(move || {
            while !until.load(Ordering::SeqCst) {
                // A timeout is a quiet moment on the link, not a failure.
                if matches!(answering.answer_one(), Ok(Some(_))) {
                    counting.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, THE_WIRE_PORT)).unwrap();
        let counting = Arc::clone(&connections);
        std::thread::spawn(move || {
            while let Ok((connection, _)) = listener.accept() {
                counting.fetch_add(1, Ordering::SeqCst);
                drop(connection);
            }
        });

        // The witness: the port the studio asks from. An answer to the studio
        // that left reception by the route would arrive here instead.
        let ear = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, ASKED_FROM)).unwrap();
        ear.set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let counting = Arc::clone(&strays);
        let until = Arc::clone(&stopping);
        std::thread::spawn(move || {
            let mut heard = [0_u8; 1_500];
            while !until.load(Ordering::SeqCst) {
                if ear.recv_from(&mut heard).is_ok() {
                    counting.fetch_add(1, Ordering::SeqCst);
                }
            }
        });
        println!("alo:serving");

        loop {
            match next_line().as_str() {
                "strays" => {
                    let heard = strays.load(Ordering::SeqCst);
                    println!(
                        "alo:strays {}",
                        if heard == 0 {
                            "nothing".to_owned()
                        } else {
                            heard.to_string()
                        }
                    );
                }
                "count" => println!(
                    "alo:counted {} {} {}",
                    answers.load(Ordering::SeqCst),
                    connections.load(Ordering::SeqCst),
                    strays.load(Ordering::SeqCst)
                ),
                "stop" => {
                    stopping.store(true, Ordering::SeqCst);
                    println!("alo:stopped");
                    return;
                }
                said => panic!("somebody else was told something it does not do: {said}"),
            }
        }
    }
}
