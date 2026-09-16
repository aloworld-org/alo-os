//! A machine on two networks that hand out one private range, reached on
//! **both** of them — on a real kernel, over two `veth` cables.
//!
//! *Machines find each other with zero configuration, and trust none of them
//! for it.* Until `crate::listeners` the port presence advertises was one
//! listener held to nothing, and an unheld TCP listener answers every handshake
//! by the route (`docs/quirks.md`, measured 2026-09-16). So on the commonest
//! office and the commonest home — two routers handing out the same private
//! range — only the machine on the network the route points at could reach this
//! one's port at all: the studio on the cable proposes, its SYN arrives, the
//! reply leaves by the other network, and the handshake never completes. The
//! rules with no kernel in them are `crate::networks`' and
//! `crate::arrived_on`'s; this is the join, over cables, with two machines
//! really at one address.
//!
//! # The three machines
//!
//! **Reception** is this test binary run again in a network namespace of its
//! own, made by util-linux's `unshare` inside a user namespace — so this needs
//! no root, takes no kernel-global state, and both cables die with the
//! namespaces (`docs/autonomy/SHARED_MAIN.md`).
//!
//! - At the far end of `cable0` is **the studio**, at `10.67.0.2`, running the
//!   whole service as `src/main.rs` does. Reception's end is `10.67.0.1`.
//! - At the far end of `other0` is **somebody else**, also at `10.67.0.2`,
//!   answering discovery **as the studio** and counting every connection made to
//!   its port. Reception's end is `10.67.0.3`.
//!
//! **Reception's route to `10.67.0.2` goes over `other0`** — the network the
//! studio is *not* on. Everything reception does that is not held to a network
//! therefore reaches somebody else.
//!
//! # What the route is made to do, and what is left alone
//!
//! The route to `10.67.0.2` is moved to `other0` in the **main** table, which is
//! the machine this task is about. One thing is put back by a policy rule:
//! **discovery's answers, which are UDP, go over the cable**. A discovery answer
//! leaves from the socket `crate::wire` answers on, which is held to no network,
//! so on a real machine on two such networks it would leave by the route as
//! well — a gap of discovery's own, written down in `docs/quirks.md` and left to
//! the next task. Holding the answers here with a rule is what makes this test
//! about the **handshake**, which is the thing task 27 changed, rather than
//! about a second thing nothing in this change touched.
//!
//! Everything else reception does reaches the studio because the **code** holds
//! it: the look on each network is held to that network (task 22), the
//! measurement is held to the network the connection arrived on (task 26), and
//! the confirmation is dialled held to the network the studio was heard on
//! (`alo_nearby::dialling`, ADR 0044).
//!
//! # In this order
//!
//! 1. Reception lays both cables and points the route at the other network. It
//!    binds **an unheld listener** at one spare port and **the wire's held
//!    listeners** at another, and the studio dials each: the unheld one is never
//!    reached, and the held one is — accepted with the cable as the network it
//!    arrived on. That is the measurement the whole change rests on, and the
//!    refusal beside it.
//! 2. Both machines serve. Reception is listening on **both** cables and on
//!    loopback.
//! 3. The studio's person proposes a pairing by identity — task 12's request —
//!    over the network the route does not point at. Reception's person is shown
//!    it with a code, both people confirm, and each machine lists the other as
//!    paired.
//! 4. Somebody else, at the same address on the network the route *does* point
//!    at, was asked nothing and connected to never.

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
    use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
    use std::os::fd::BorrowedFd;
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::{Duration, Instant, SystemTime};

    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_keeping::Keeping;
    use alo_nearby::{Answering, MachineId, Presence, THE_PORT};
    use alo_protocol::{AfterConfirming, ToAPerson};
    use alo_record::Record;
    use alo_turn::Machine;

    use crate::arrived_on::ArrivedOn;
    use crate::hosting::Hosted;
    use crate::listeners::{Listeners, Listening};
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
    use crate::unix::ready_and;
    use crate::wire::{THE_WIRE_PORT, Wire};

    /// Set on the binary run again inside a namespace, naming which machine it
    /// is.
    const INSIDE: &str = "ALO_AGENTD_BOTH_NETWORKS_INSIDE";

    /// Reception's cable to the studio — the network the route does **not**
    /// point at.
    const THE_CABLE: &str = "cable0";
    /// Reception's cable to somebody else — the network the route points at.
    const THE_OTHER_NETWORK: &str = "other0";
    /// The far end of either cable.
    const FAR_END: &str = "far0";

    /// Reception's address on the cable, and on the other network: two networks
    /// out of one private range, as two routers handing it out look.
    const RECEPTION_ON_THE_CABLE: &str = "10.67.0.1/24";
    const RECEPTION_ELSEWHERE: &str = "10.67.0.3/24";
    /// The address at the far end of both cables.
    const FAR_ADDRESS: &str = "10.67.0.2/24";

    /// A spare port for the listener held to nothing, and one for the wire's
    /// held listeners — neither is the wire's own port, which the service binds.
    const UNHELD_PORT: u16 = 7_710;
    const HELD_PORT: u16 = 7_711;

    /// How long anything here waits for the kernel or another machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// **A machine on two networks with one private range is reachable on
    /// both**: the outer test, which makes reception and reads what it saw.
    #[test]
    fn a_machine_on_two_networks_with_one_private_range_is_reachable_on_both() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_machine_reachable_on_both_networks::tests::reception_on_two_networks_with_one_range";
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
            "a listener held to nothing was never reached from the cable",
            "a listener held to the cable was reached, and the connection arrived on the cable",
            "reception listens on both cables",
            "paired over the network the route does not point at",
            "somebody else at the same address was asked nothing and connected to never",
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

    /// Point reception's route to `10.67.0.2` at the other network, and put
    /// discovery's answers back on the cable with a rule of their own.
    ///
    /// See this module's documentation: the main table is the machine this task
    /// is about, and the UDP rule stands in for a hold discovery does not have
    /// yet, so that what is measured here is the handshake.
    fn the_route_points_at_the_other_network() {
        ip(
            None,
            &[
                "route",
                "replace",
                "10.67.0.2/32",
                "dev",
                THE_OTHER_NETWORK,
                "src",
                "10.67.0.3",
            ],
        );
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
                "10.67.0.2/32",
                "dev",
                THE_CABLE,
                "src",
                "10.67.0.1",
                "table",
                "100",
            ],
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
            "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_machine_reachable_on_both_networks::tests::{runs}"
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

    /// The next thing a far end was told, read a line at a time — rather than
    /// through an iterator over standard input, which holds a lock a thread
    /// cannot take with it.
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

    /// Reception, on two networks with one private range: the whole of it.
    #[test]
    #[ignore = "run inside its own network by a_machine_on_two_networks_with_one_private_range_is_reachable_on_both"]
    fn reception_on_two_networks_with_one_range() {
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

        // 1. The measurement the change rests on: a listener held to nothing is
        //    never reached from the cable, and one held to the cable is.
        let unheld = TcpListener::bind((Ipv4Addr::UNSPECIFIED, UNHELD_PORT)).unwrap();
        tell(
            &mut telling_studio,
            &format!("dial 10.67.0.1:{UNHELD_PORT}"),
        );
        assert_eq!(
            said_by(&mut hearing_studio, "alo:dialled"),
            "nowhere",
            "an unheld listener answered a handshake from the network the route does not point at"
        );
        drop(unheld);
        println!("a listener held to nothing was never reached from the cable");

        let held = Listeners::bound(HELD_PORT, &mut |line| println!("alo-agentd: {line}")).unwrap();
        let cable = held
            .listened_on()
            .into_iter()
            .find(|network| network.name() == THE_CABLE)
            .expect("the cable is one of the networks the port is listened on");
        tell(&mut telling_studio, &format!("dial 10.67.0.1:{HELD_PORT}"));
        let listening = Listening::of(&held);
        let nothing: [Option<BorrowedFd<'_>>; 0] = [];
        let waiting_on = listening.waiting_on();
        let (_, ready) = ready_and(&nothing, &waiting_on, Some(PATIENCE)).unwrap();
        let knocked = listening
            .accept_one(&ready)
            .expect("a listener held to the cable was never reached");
        assert_eq!(
            knocked.arrived,
            ArrivedOn::TheNetwork(std::num::NonZeroU32::new(cable.index()).unwrap()),
            "the connection was not read as having arrived on the cable"
        );
        assert_eq!(said_by(&mut hearing_studio, "alo:dialled"), "reached");
        drop(knocked);
        drop(listening);
        drop(held);
        println!(
            "a listener held to the cable was reached, and the connection arrived on the cable"
        );

        // 2 to 4: both machines serve, and the studio pairs over the cable.
        tell(&mut telling_studio, "serve");
        said_by(&mut hearing_studio, "alo:serving");
        serving_as(
            &reception(),
            "both-networks-reception",
            |wire, door, stop| {
                let listened: Vec<String> = wire
                    .listened_on()
                    .iter()
                    .map(|network| network.name().to_owned())
                    .collect();
                for network in [THE_CABLE, THE_OTHER_NETWORK, "lo"] {
                    assert!(
                        listened.iter().any(|name| name == network),
                        "the port is not listened on {network}: {listened:?}"
                    );
                }
                println!("reception listens on both cables");

                // The person's door is held before anything arrives: a proposal
                // nobody can be shown is refused as such.
                let mut person = Talking::to(door);
                let (paired, _) = person.pairings();
                assert!(paired.is_empty(), "{paired:?}");

                // 3. The studio proposes over the network the route does not point
                //    at — the thing that cannot happen without a held listener.
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

                // 4. And nothing of this reached the machine at the same address on
                //    the network the route does point at.
                tell(&mut telling_other, "count");
                let counted = said_by(&mut hearing_other, "alo:counted");
                assert_eq!(
                    counted, "0 0",
                    "somebody else at the same address was asked or connected to"
                );
                println!(
                    "somebody else at the same address was asked nothing and connected to never"
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

    /// The studio, at the far end of the cable: it dials what it is told to,
    /// then runs the whole service and proposes a pairing to reception.
    #[test]
    #[ignore = "run inside its own network by reception_on_two_networks_with_one_range"]
    fn the_studio_at_the_end_of_the_cable() {
        must_be("studio");
        loop {
            let line = next_line();
            match line.split_once(' ') {
                Some(("dial", at)) => {
                    let at: SocketAddr = at.parse().unwrap();
                    let reached = TcpStream::connect_timeout(&at, Duration::from_secs(5)).is_ok();
                    println!(
                        "alo:dialled {}",
                        if reached { "reached" } else { "nowhere" }
                    );
                }
                None if line == "serve" => break,
                _ => panic!("the studio was told something it does not do: {line}"),
            }
        }

        serving_as(&the_studio(), "both-networks-studio", |_, door, stop| {
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

    /// Somebody else, at the same address on the network the route points at:
    /// it answers discovery **as the studio** and counts what reaches it.
    ///
    /// Nothing about an advertisement proves who sent it (ADR 0003), so the only
    /// thing that tells the two apart is which network reached which — and this
    /// machine is the witness that nothing reached it.
    #[test]
    #[ignore = "run inside its own network by reception_on_two_networks_with_one_range"]
    fn somebody_else_at_the_same_address() {
        must_be("somebody else");
        assert_eq!(next_line(), "serve");

        let stopping = Arc::new(AtomicBool::new(false));
        let answers = Arc::new(AtomicUsize::new(0));
        let connections = Arc::new(AtomicUsize::new(0));

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
        println!("alo:serving");

        loop {
            match next_line().as_str() {
                "count" => println!(
                    "alo:counted {} {}",
                    answers.load(Ordering::SeqCst),
                    connections.load(Ordering::SeqCst)
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
