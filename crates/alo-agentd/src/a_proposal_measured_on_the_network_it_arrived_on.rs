//! A proposal arriving from a private IPv4 address, on a machine where another
//! network carries the same address, measured on the network it arrived on —
//! on a real kernel, over two `veth` cables.
//!
//! *Machines find each other with zero configuration, and trust none of them
//! for it.* A proposal is judged against what discovery on this machine has
//! **just** measured about the machine that sent it (`crate::looking`), and
//! since [ADR 0044](../../../docs/decisions/0044-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md)
//! a machine dialled at a private IPv4 address is held to the network it was
//! found on. A connection that *arrives* from `10.66.0.2` was measured from a
//! socket held to nothing, so the question *who are you* left by whatever the
//! route said — and on a machine on two networks whose routers hand out the
//! same range, that is another machine at the same address.
//!
//! The rule with no kernel in it is `crate::arrived_on`'s tests and
//! `crate::looking`'s; this is the join, over cables, with two machines really
//! at one address.
//!
//! # The three machines
//!
//! **Reception** is this test binary run again in a network namespace of its
//! own, made by util-linux's `unshare` inside a user namespace — so this needs
//! no root, takes no kernel-global state, and both cables die with the
//! namespaces (`docs/autonomy/SHARED_MAIN.md`).
//!
//! - At the far end of `cable0` is **the studio**, at `10.66.0.2`. Reception's
//!   end is `10.66.0.1`.
//! - At the far end of `other0` is **somebody else**, also at `10.66.0.2`, and
//!   answering discovery **as the studio** — which is the whole point: nothing
//!   about an advertisement proves who sent it (ADR 0003), so the only thing
//!   that tells the two apart is which network the question was asked on.
//!   Reception's end is `10.66.0.3`.
//!
//! Each far end counts every discovery question it answers, so *it was asked
//! nothing* is witnessed by the machine that would have answered.
//!
//! # The route changes while the proposal is open
//!
//! The plan's own words: *through a route that changed while the question was
//! open*. The studio proposes while the route to `10.66.0.2` goes over the
//! cable, so the connection is made; reception accepts it; the route is then
//! moved to the other network, as a Wi-Fi coming up moves it. Everything after
//! that is measured with the route pointing at the impostor.
//!
//! # In this order
//!
//! 1. The studio proposes over the cable and reception accepts the connection.
//!    The route to `10.66.0.2` is moved to the other network.
//! 2. Reception reads the network the connection arrived on — the cable — and
//!    measures there: the studio answers, is written down **on the cable's
//!    interface**, and somebody else is asked nothing.
//! 3. The proposal itself goes through `crate::hearing`, which measures the
//!    same way and shows it: the machine that sent it is the machine it was
//!    judged against, and somebody else is still asked nothing.
//! 4. The same measurement made the way it was made before this file — held to
//!    nothing, by the route — reaches **somebody else**, who answers as the
//!    studio. That is the failure, witnessed: the hold is what told them apart.
//! 5. A connection whose arriving network could not be read is measured
//!    nowhere: nothing is asked of either machine.

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
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
    use std::num::NonZeroU32;
    use std::path::Path;
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    use alo_capability::Grants;
    use alo_corridor::{AT_MOST_A_VERB, Doorway};
    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_nearby::{
        Answering, HeardFrom, Keying, MachineId, MayAskIts, Presence, Proposal, THE_PORT,
        THE_PROPOSAL_PATH, Waiting, http,
    };
    use alo_record::Record;
    use alo_turn::Machine;

    use crate::arrived_on::{ArrivedOn, the_network_it_arrived_on};
    use crate::hearing::{Heard, Judging, heard};
    use crate::looking::found_at;
    use crate::network::TheNetwork;
    use crate::terms::NoNameYet;
    use crate::testing::{
        NothingIsBounded, hour, in_english, nothing_has_been_chosen, reception, the_studio,
    };
    use crate::wire::{Knocked, THE_WIRE_PORT};

    /// Set on the binary run again inside a namespace, naming which machine it
    /// is.
    const INSIDE: &str = "ALO_AGENTD_ARRIVED_ON_INSIDE";

    /// Reception's cable to the studio.
    const THE_CABLE: &str = "cable0";
    /// Reception's cable to somebody else.
    const THE_OTHER_NETWORK: &str = "other0";
    /// The far end of either cable.
    const FAR_END: &str = "far0";

    /// Reception's address on the cable, and on the other network: two networks
    /// out of one private range, as two routers handing it out look.
    const RECEPTION_ON_THE_CABLE: &str = "10.66.0.1/24";
    const RECEPTION_ELSEWHERE: &str = "10.66.0.3/24";
    /// The address at the far end of both cables.
    const FAR_ADDRESS: &str = "10.66.0.2/24";
    /// The same, as an address.
    const AT_THE_FAR_END: Ipv4Addr = Ipv4Addr::new(10, 66, 0, 2);

    /// How long anything here waits for the kernel or another machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// **A proposal from a private IPv4 address is measured on the network it
    /// arrived on**: the outer test, which makes reception and reads what it
    /// saw.
    #[test]
    fn a_proposal_from_a_private_address_is_measured_on_the_network_it_arrived_on() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_proposal_measured_on_the_network_it_arrived_on::tests::reception_between_two_machines_at_one_address";
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
            "the proposal arrived on the cable, and the route now goes elsewhere",
            "measured on the cable and written down on its interface, and nobody else was asked",
            "the proposal was judged against the machine that sent it",
            "the route would have measured somebody else",
            "a connection whose network could not be read was measured nowhere",
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

    /// The kernel's number for the interface `name`, from what `ip` prints
    /// first.
    fn index_of(name: &str) -> u32 {
        let ran = Command::new("ip")
            .args(["-o", "link", "show", "dev", name])
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&ran.stdout);
        said.split_once(':')
            .and_then(|(index, _)| index.trim().parse().ok())
            .unwrap_or_else(|| panic!("no interface called {name}: {said}"))
    }

    /// Where questions to `10.66.0.2` go: out `interface`, from `from`, as a
    /// host route, which beats either network's own.
    fn the_route_goes(interface: &str, from: &str) {
        ip(
            None,
            &[
                "route",
                "replace",
                "10.66.0.2/32",
                "dev",
                interface,
                "src",
                from,
            ],
        );
    }

    /// Send every **question** — every UDP datagram — to `10.66.0.2` out
    /// `interface` from `from`, leaving connections already open where they
    /// are.
    ///
    /// A policy rule and a table of its own rather than a route in the main
    /// one: an established connection re-looks-up its route when the main
    /// table changes, so moving it would send the reply to this proposal to
    /// the other machine as well, and what is being tested here is where the
    /// *measurement* goes. This is the machine on two networks whose route to
    /// a private address is not the network the connection came in on.
    fn questions_go_to(interface: &str, from: &str) {
        ip(
            None,
            &[
                "rule", "add", "ipproto", "udp", "table", "100", "priority", "100",
            ],
        );
        ip(
            None,
            &[
                "route",
                "replace",
                "10.66.0.2/32",
                "dev",
                interface,
                "src",
                from,
                "table",
                "100",
            ],
        );
    }

    /// A far end in a network namespace nested inside reception's, with the
    /// cable `cable` laid to it and both ends addressed.
    fn a_far_end(
        which: &str,
        cable: &str,
        here: &str,
    ) -> (Child, ChildStdin, BufReader<ChildStdout>) {
        let exe = env::current_exe().unwrap();
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_proposal_measured_on_the_network_it_arrived_on::tests::a_machine_at_the_far_end";
        let mut far = Command::new("unshare")
            .args(["--net", "--", "sh", "-c", script])
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
        let pid_text = far.id().to_string();
        ip(
            None,
            &[
                "link", "add", cable, "type", "veth", "peer", "name", FAR_END, "netns", &pid_text,
            ],
        );
        ip(
            Some(far.id()),
            &["addr", "add", FAR_ADDRESS, "dev", FAR_END],
        );
        ip(Some(far.id()), &["link", "set", FAR_END, "up"]);
        ip(None, &["addr", "add", here, "dev", cable]);
        ip(None, &["link", "set", cable, "up"]);
        let mut telling = far.stdin.take().unwrap();
        let mut hearing = BufReader::new(far.stdout.take().unwrap());
        telling.write_all(b"serve\n").unwrap();
        said_by(&mut hearing, "alo:serving");
        (far, telling, hearing)
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

    /// How many discovery questions a far end has answered.
    fn asked(telling: &mut ChildStdin, hearing: &mut BufReader<ChildStdout>) -> usize {
        telling.write_all(b"count\n").unwrap();
        said_by(hearing, "alo:counted").parse().unwrap()
    }

    /// What reception replied to the proposal a far end sent, waiting for it:
    /// zero while the reply has not arrived there yet.
    fn replied_to(telling: &mut ChildStdin, hearing: &mut BufReader<ChildStdout>) -> usize {
        let until = Instant::now() + PATIENCE;
        loop {
            telling
                .write_all(
                    b"replied
",
                )
                .unwrap();
            let said: usize = said_by(hearing, "alo:replied").parse().unwrap();
            if said != 0 {
                return said;
            }
            assert!(Instant::now() < until, "the proposal was never replied to");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// What a far end has answered once it has stopped changing: counted until
    /// it is `expected` or [`PATIENCE`] runs out, and again a moment later, so
    /// a question that should not have arrived is given the same moment to.
    fn settled(
        telling: &mut ChildStdin,
        hearing: &mut BufReader<ChildStdout>,
        expected: usize,
    ) -> usize {
        let until = Instant::now() + PATIENCE;
        while asked(telling, hearing) != expected && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(50));
        }
        std::thread::sleep(Duration::from_millis(300));
        asked(telling, hearing)
    }

    /// Reception, between two machines at one address: the whole of it.
    #[test]
    #[ignore = "run inside its own network by a_proposal_from_a_private_address_is_measured_on_the_network_it_arrived_on"]
    fn reception_between_two_machines_at_one_address() {
        must_be("reception");
        let (mut studio, mut telling_studio, mut hearing_studio) =
            a_far_end("studio", THE_CABLE, RECEPTION_ON_THE_CABLE);
        let (mut other, mut telling_other, mut hearing_other) =
            a_far_end("somebody else", THE_OTHER_NETWORK, RECEPTION_ELSEWHERE);
        let cable = index_of(THE_CABLE);

        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, THE_WIRE_PORT)).unwrap();

        // 1. The studio proposes while the route to its address goes over the
        //    cable, and the route a question would take moves to the other
        //    network before anything is measured.
        the_route_goes(THE_CABLE, "10.66.0.1");
        telling_studio.write_all(b"propose 10.66.0.1\n").unwrap();
        let (stream, from) = listener.accept().unwrap();
        questions_go_to(THE_OTHER_NETWORK, "10.66.0.3");
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let message = http::read_message_of_at_most(&stream, AT_MOST_A_VERB);
        assert_eq!(from.ip(), IpAddr::V4(AT_THE_FAR_END), "{from}");
        let arrived = the_network_it_arrived_on(&stream);
        assert_eq!(
            arrived,
            ArrivedOn::TheNetwork(NonZeroU32::new(cable).unwrap()),
            "the connection was not read as having arrived on the cable"
        );
        println!("the proposal arrived on the cable, and the route now goes elsewhere");

        // 2. Measured on the cable: the studio answers, written down there.
        let measured = found_at(HeardFrom::of(from), arrived, THE_PORT);
        let one = measured
            .iter()
            .find(|found| found.machine == the_studio())
            .unwrap_or_else(|| panic!("the studio was not measured on the cable: {measured:?}"));
        assert_eq!(one.address, IpAddr::V4(AT_THE_FAR_END));
        assert_eq!(one.address.interface(), Some(cable), "{one:?}");
        assert_eq!(settled(&mut telling_studio, &mut hearing_studio, 1), 1);
        assert_eq!(
            asked(&mut telling_other, &mut hearing_other),
            0,
            "a measurement on the cable asked the machine on the other network"
        );
        println!(
            "measured on the cable and written down on its interface, and nobody else was asked"
        );

        // 3. The proposal itself, through the daemon's own door.
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut record = Record::default();
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let mut doorway = Doorway::at(reception(), &mut machine, hour(), hour()).unwrap();
        let mut grants = Grants::default();
        let network = TheNetwork::on(reception());
        let mut questions = nothing_has_been_chosen();
        let mut shown: Vec<MachineId> = Vec::new();
        let mut surface = |waiting: &Waiting| {
            shown.push(waiting.other().clone());
            true
        };
        let judged = heard(
            Knocked {
                stream,
                from: HeardFrom::of(from),
                message,
            },
            &mut doorway,
            &mut grants,
            &mut Judging {
                network: &network,
                surface: &mut surface,
                naming: &NoNameYet,
                policy: &EgressPolicy::InTheBuilding,
                asking_at: THE_PORT,
                questions: &mut questions,
            },
            std::time::SystemTime::now(),
        )
        .unwrap();
        assert!(
            matches!(
                &judged,
                Heard::OnThePairingWire(alo_nearby::Heard::AProposal { from }) if *from == the_studio()
            ),
            "{judged:?}"
        );
        assert_eq!(shown, vec![the_studio()], "the proposal was not shown");
        assert_eq!(settled(&mut telling_studio, &mut hearing_studio, 2), 2);
        assert_eq!(
            asked(&mut telling_other, &mut hearing_other),
            0,
            "judging the proposal asked the machine on the other network"
        );
        assert_eq!(replied_to(&mut telling_studio, &mut hearing_studio), 200);
        println!("the proposal was judged against the machine that sent it");

        // 4. The same measurement held to nothing goes by the route, which is
        //    now somebody else — answering as the studio.
        let by_the_route = found_at(HeardFrom::of(from), ArrivedOn::ItsOwnNetwork, THE_PORT);
        assert!(
            by_the_route
                .iter()
                .any(|found| found.machine == the_studio()),
            "the route reached nobody at all, so nothing is being told apart: {by_the_route:?}"
        );
        assert_eq!(settled(&mut telling_other, &mut hearing_other, 1), 1);
        assert_eq!(
            asked(&mut telling_studio, &mut hearing_studio),
            2,
            "the route reached the studio, so the route and the cable are the same network"
        );
        println!("the route would have measured somebody else");

        // 5. And a connection whose network could not be read is measured
        //    nowhere: neither machine is asked.
        assert!(
            found_at(HeardFrom::of(from), ArrivedOn::NothingCouldSay, THE_PORT).is_empty(),
            "a connection measured nowhere found something"
        );
        assert_eq!(asked(&mut telling_studio, &mut hearing_studio), 2);
        assert_eq!(asked(&mut telling_other, &mut hearing_other), 1);
        println!("a connection whose network could not be read was measured nowhere");

        for telling in [&mut telling_studio, &mut telling_other] {
            drop(telling.write_all(b"stop\n"));
        }
        said_by(&mut hearing_studio, "alo:stopped");
        said_by(&mut hearing_other, "alo:stopped");
        assert!(studio.wait().unwrap().success(), "the studio failed");
        assert!(other.wait().unwrap().success(), "somebody else failed");
    }

    /// A machine at the far end of one cable: both of them answer discovery as
    /// the studio, and only the studio is told to propose.
    #[test]
    #[ignore = "run inside its own network by reception_between_two_machines_at_one_address"]
    fn a_machine_at_the_far_end() {
        let which = env::var(INSIDE).unwrap();
        assert!(
            which == "studio" || which == "somebody else",
            "this test runs inside the namespace reception makes; run the outer one"
        );
        let mut told = std::io::stdin().lines();
        assert_eq!(told.next().unwrap().unwrap(), "serve");

        let stopping = Arc::new(AtomicBool::new(false));
        let answers = Arc::new(AtomicUsize::new(0));
        // What reception replied to the proposal this machine sent, or zero
        // while none has been sent or answered — asked for rather than
        // printed, because a line printed while reception is counting is a
        // line reception has already read past.
        let replied = Arc::new(AtomicUsize::new(0));
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
        println!("alo:serving");

        for line in told {
            let line = line.unwrap();
            match line.split_once(' ') {
                // Sent from a thread of its own, so this machine goes on
                // answering *count* while reception measures and judges: the
                // reply to the proposal is written only once it has.
                Some(("propose", at)) => {
                    let at = at.to_owned();
                    let said = Arc::clone(&replied);
                    std::thread::spawn(move || {
                        said.store(proposed(&at).into(), Ordering::SeqCst);
                    });
                }
                None if line == "count" => {
                    println!("alo:counted {}", answers.load(Ordering::SeqCst));
                }
                None if line == "replied" => {
                    println!("alo:replied {}", replied.load(Ordering::SeqCst));
                }
                None if line == "stop" => {
                    stopping.store(true, Ordering::SeqCst);
                    println!("alo:stopped");
                    return;
                }
                _ => panic!("a far end was told something it does not do: {line}"),
            }
        }
    }

    /// Propose a pairing to the machine at `at`, and answer with the status it
    /// replied with.
    fn proposed(at: &str) -> u16 {
        let proposal = Proposal::checked(
            the_studio(),
            reception(),
            &[MayAskIts::Models],
            Duration::from_secs(3_600),
            Keying::fresh().unwrap().offer().clone(),
        )
        .unwrap();
        let to = SocketAddr::new(at.parse().unwrap(), THE_WIRE_PORT);
        let mut stream = TcpStream::connect_timeout(&to, PATIENCE).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .unwrap();
        stream
            .write_all(
                http::a_request_carrying(
                    THE_PROPOSAL_PATH,
                    at,
                    &[],
                    &format!("{}\n", proposal.said()),
                )
                .as_bytes(),
            )
            .unwrap();
        let reply = http::read_message(&stream).unwrap();
        http::status_of(&reply.first).unwrap()
    }
}
