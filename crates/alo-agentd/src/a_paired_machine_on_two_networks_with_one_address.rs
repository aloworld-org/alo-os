//! A question from a turn to a paired machine found at a private IPv4 address,
//! on a machine where another network carries the same address — bounded by the
//! real programme, shown on the indicator and written in the record — on a real
//! kernel.
//!
//! *One GPU box serves the office — it is still egress, and the indicator still
//! fires.* [ADR 0042](../../../docs/decisions/0042-a-private-ipv4-departure-is-held-to-the-network-it-was-found-on.md)
//! holds a paired machine's IPv4 departure to the interface of the network it
//! was found on. The rules are each tested with no kernel in them
//! (`alo_bounding_map::Departure::permits`, `crate::bounding`,
//! `alo_nearby::HeardFrom::on_the_network`, `crate::corridor`) and on a kernel
//! with no daemon in it (`alo-bounding`'s
//! `a_private_ipv4_departure_is_held_to_its_network.rs`); this is the join.
//!
//! # The three machines
//!
//! **Reception** is this test binary run again in a network namespace of its own,
//! made by util-linux's `unshare` as root and in no user namespace, because it
//! opens the boundary the outer test pinned and moves itself into control groups.
//! It has two cables, and both of its ends are `10.65.0.1/24`, which is what two
//! routers handing out the same range look like from the machine on both.
//!
//! - At the far end of `cable0` is **the studio**, at `10.65.0.2`: it answers
//!   discovery as `alo-nearby` does and questions as the corridor expects.
//! - At the far end of `other0` is **somebody else**, also at `10.65.0.2`, on the
//!   same port: it answers any connection and any question, and does not answer
//!   discovery — it is not an alo machine, and it is not the one the person
//!   paired with.
//!
//! Both count every connection and every question that reaches them, so *the
//! kernel refused it* is witnessed by the machine that would have been reached.
//! The outer test holds `alo_bounding::Waited::on_this_kernel()` for the whole of
//! it; the children never take it again (`docs/autonomy/SHARED_MAIN.md`).
//!
//! # In this order
//!
//! 1. Reception looks on both networks and finds the studio **on the cable
//!    alone**, at its IPv4 address held to the cable's interface.
//! 2. An agent's question to the machine its person chose goes from a turn
//!    bounded by the real programme — answered, **shown and taken off the
//!    indicator, and written as a departure naming the paired machine**, exactly
//!    the shape it always had — and the studio counts one connection and one
//!    question, and somebody else counts nothing.
//! 3. The kernel, asked directly through the same `ByTheKernel` with the studio
//!    held to the cable: the cable is reached; the same address on `other0` is
//!    refused with `EACCES`; and a socket held to no interface, which the route
//!    would send somewhere, is refused. Somebody else still counts nothing.
//! 4. A provider's departure, held to no interface, is reached by the route as it
//!    always was.

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::env;
    use std::io::{BufRead as _, BufReader, Read as _, Write as _};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
    use std::num::NonZeroU32;
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    use alo_bounding::{Imposed, Pinned};
    use alo_capability::Grants;
    use alo_context::Context;
    use alo_egress::{Destination, Indicator};
    use alo_files::OnThisMachine;
    use alo_models::Catalogue;
    use alo_nearby::{Answering, Found, MachineId, MayAskIts, Presence, THE_ADDRESS, THE_PORT};
    use alo_protocol::ToAnAgent;
    use alo_record::{Happened, Record};
    use alo_turn::{Bounding as _, Machine, Turning};

    use crate::bounding::ByTheKernel;
    use crate::corridor::Corridor;
    use crate::doing::what_an_agent_said;
    use crate::looking::{LookingFor, found_by_name};
    use crate::network::TheNetwork;
    use crate::networks::discovery_networks;
    use crate::questions::{Questions, TheBound, WhoseKeyring};
    use crate::route_messages::reported_by_the_kernel;
    use crate::terms::NoNameYet;
    use crate::testing::{
        THE_STUDIOS_ANSWER, a_directory_of_our_own, a_message, hour, in_english, noon,
        paired_between, reception, the_studio,
    };
    use crate::unix::a_shared_datagram_socket_on;

    /// Set on the binary run again inside a namespace, naming which machine it is.
    const INSIDE: &str = "ALO_AGENTD_TWO_NETWORKS_INSIDE";

    /// Where the outer test pinned the boundary, for reception to open.
    const PINNED: &str = "ALO_AGENTD_TWO_NETWORKS_PINNED";

    /// The port the studio advertises, and both far ends answer questions on.
    const QUESTIONS: u16 = 7_612;

    /// Reception's cable to the studio.
    const THE_CABLE: &str = "cable0";
    /// Reception's cable to somebody else.
    const THE_OTHER_NETWORK: &str = "other0";
    /// The far end of either cable.
    const FAR_END: &str = "far0";

    /// Reception's address, on both networks.
    const RECEPTION: &str = "10.65.0.1/24";
    /// The address at the far end of both cables.
    const FAR_ADDRESS: &str = "10.65.0.2/24";
    /// The same, as an address.
    const THE_STUDIOS_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 65, 0, 2);

    /// How long anything here waits for the kernel or the other machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// What an agent asks.
    const ASKED: &str = r#"{"ask":{"question":"how many invoices are unpaid?"}}"#;

    /// `EACCES`, which is the only thing a refusal by the boundary looks like.
    const REFUSED: i32 = 13;

    /// **A question from a turn to a paired machine at a private IPv4 address is
    /// held to the network it was found on**: the outer test, which takes the
    /// kernel, imposes the boundary, makes reception and reads what it saw.
    #[test]
    fn a_question_to_a_paired_machine_at_a_private_address_reaches_only_the_network_it_was_found_on()
     {
        let _kernel = alo_bounding::Waited::on_this_kernel()
            .expect("this kernel can be taken, and nothing is forced if it cannot");
        let pinned = Pinned::beneath(
            &PathBuf::from("/sys/fs/bpf").join(format!("alo-two-networks-{}", std::process::id())),
        );
        pinned.taken_away();
        pinned
            .made()
            .expect("this machine has a BPF filesystem at /sys/fs/bpf");
        let loaded = Imposed::once(&pinned).expect(
            "no boundary could be imposed, so nothing here would be tested: this needs root and \
             a kernel that started the BPF LSM",
        );

        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_paired_machine_on_two_networks_with_one_address::tests::reception_on_two_networks";
        let ran = Command::new("unshare")
            .args(["--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "reception")
            .env(PINNED, pinned.root())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect("util-linux's `unshare` makes the machines");
        drop(loaded);
        pinned.taken_away();

        let said = String::from_utf8_lossy(&ran.stdout).into_owned();
        assert!(
            ran.status.success(),
            "reception failed ({}):\n{said}",
            ran.status
        );
        for step in [
            "found on the cable alone, held to its interface",
            "answered from a bounded turn, shown, and recorded naming the paired machine",
            "the kernel reached the cable and refused the same address on the other network",
            "a departure held to no interface went by the route",
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

    /// The kernel's number for the interface `name`, from what `ip` prints first.
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

    /// A far end in a network namespace nested inside reception's, answering as
    /// `which`, with the cable `cable` laid to it and both ends addressed.
    fn a_far_end(which: &str, cable: &str) -> (Child, ChildStdin, BufReader<ChildStdout>) {
        let exe = env::current_exe().unwrap();
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_paired_machine_on_two_networks_with_one_address::tests::a_machine_at_the_far_end";
        let mut far = Command::new("unshare")
            .args(["--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, which)
            .env_remove(PINNED)
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
        let pid = far.id();
        let pid_text = pid.to_string();
        ip(
            None,
            &[
                "link", "add", cable, "type", "veth", "peer", "name", FAR_END, "netns", &pid_text,
            ],
        );
        ip(Some(pid), &["addr", "add", FAR_ADDRESS, "dev", FAR_END]);
        ip(Some(pid), &["link", "set", FAR_END, "up"]);
        ip(None, &["addr", "add", RECEPTION, "dev", cable]);
        ip(None, &["link", "set", cable, "up"]);
        let mut telling = far.stdin.take().unwrap();
        let mut hearing = BufReader::new(far.stdout.take().unwrap());
        telling.write_all(b"cable\n").unwrap();
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

    /// How many connections and how many questions have reached a far end.
    fn counted(telling: &mut ChildStdin, hearing: &mut BufReader<ChildStdout>) -> (usize, usize) {
        telling.write_all(b"count\n").unwrap();
        let said = said_by(hearing, "alo:counted");
        let (connections, questions) = said.split_once(' ').unwrap();
        (connections.parse().unwrap(), questions.parse().unwrap())
    }

    /// What has reached a far end once it has stopped changing: counted until it
    /// is `expected` or [`PATIENCE`] runs out, and again a moment later, so a
    /// connection that should not have arrived is given the same moment to.
    fn settled(
        telling: &mut ChildStdin,
        hearing: &mut BufReader<ChildStdout>,
        expected: (usize, usize),
    ) -> (usize, usize) {
        let until = Instant::now() + PATIENCE;
        while counted(telling, hearing) != expected && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(50));
        }
        std::thread::sleep(Duration::from_millis(500));
        counted(telling, hearing)
    }

    /// Discovery as the service asks it: the group, on every network this
    /// machine is on at the moment.
    #[derive(Debug)]
    struct OnEveryNetwork;

    impl LookingFor for OnEveryNetwork {
        fn look_for(&self, machine: &MachineId) -> Option<Found> {
            found_by_name(machine, SocketAddr::new(THE_ADDRESS.into(), THE_PORT))
        }

        fn look_around(&self) -> alo_nearby::Around {
            crate::looking::around_at(SocketAddr::new(THE_ADDRESS.into(), THE_PORT))
        }
    }

    /// Reception's settings, choosing the studio to answer its questions.
    fn choosing_the_studio() -> Questions {
        let config = a_directory_of_our_own("two-networks");
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join(alo_choosing::THE_SETTINGS),
            format!(
                "format = 3\n\n[answers]\nmachine = \"{}\"\n",
                the_studio().as_str()
            ),
        )
        .unwrap();
        Questions::of_a_session(
            Some(config.into_os_string()),
            None,
            Catalogue::built_in().unwrap(),
            TheBound::Nobodys,
            WhoseKeyring::Nobodys,
        )
    }

    /// One agent's question, put from a turn on a machine bounded by the
    /// kernel, and what the agent was told.
    fn one_question(
        bounding: &mut ByTheKernel,
        indicator: &mut Indicator,
        record: &mut Record,
        questions: &mut Questions,
        corridor: &Corridor<'_>,
    ) -> ToAnAgent {
        let strings = in_english();
        let mut machine =
            Machine::carrying_out_file_verbs(&strings, &OnThisMachine, bounding, indicator, record)
                .unwrap();
        let mut grants = Grants::default();
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        let said = what_an_agent_said(
            &a_message(ASKED),
            &mut turning,
            questions,
            Some(corridor),
            &Grants::default(),
            &strings,
            hour(),
            noon(),
        );
        let _ = turning.ending(&mut grants);
        said
    }

    /// Every departure the record keeps.
    fn departures(record: &Record) -> Vec<Happened> {
        record
            .everything()
            .map(|entry| entry.happened().clone())
            .filter(|happened| matches!(happened, Happened::Left { .. }))
            .collect()
    }

    /// A connection to the far address from a socket held to `interface`, or
    /// held to none for `None`: `Ok` when it was made, and the error number
    /// otherwise.
    fn connecting(held_to: Option<u32>) -> Result<(), i32> {
        let to = SocketAddr::new(THE_STUDIOS_ADDRESS.into(), QUESTIONS);
        let made = match held_to {
            None => TcpStream::connect_timeout(&to, Duration::from_secs(2)).map(drop),
            Some(interface) => {
                socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None).and_then(
                    |socket| {
                        socket.bind_device_by_index_v4(NonZeroU32::new(interface))?;
                        socket.connect_timeout(&to.into(), Duration::from_secs(2))
                    },
                )
            }
        };
        made.map_err(|why| why.raw_os_error().unwrap_or(0))
    }

    /// Reception, inside a network of its own: two cables with the same address
    /// at the far end of each, and the studio at one of them.
    #[test]
    #[ignore = "run inside its own network by a_question_to_a_paired_machine_at_a_private_address_reaches_only_the_network_it_was_found_on"]
    fn reception_on_two_networks() {
        must_be("reception");
        let pinned = Pinned::beneath(Path::new(&env::var(PINNED).unwrap()));
        // Somebody else's cable first, so the route to the shared address is
        // theirs: a question that was not held to the cable would reach them.
        let (mut other, mut telling_other, mut hearing_other) =
            a_far_end("somebody else", THE_OTHER_NETWORK);
        let (mut studio, mut telling_studio, mut hearing_studio) = a_far_end("studio", THE_CABLE);
        let (cable, elsewhere) = (index_of(THE_CABLE), index_of(THE_OTHER_NETWORK));

        let mut bounding =
            ByTheKernel::beneath(&pinned).expect("reception opens the boundary the loader pinned");
        // Everything that can fail runs inside this, so that a failure still gives
        // the control group subtree back.
        let asked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let bounding = &mut bounding;
            let mut indicator = Indicator::default();
            let mut record = Record::default();
            let mut questions = choosing_the_studio();
            let network = TheNetwork::on(reception());
            let (on_reception, _) =
                paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
            network.locked().pairings_mut().keep(on_reception);
            let corridor = Corridor {
                network: &network,
                looking: &OnEveryNetwork,
                naming: &NoNameYet,
            };

            // 1. Two networks with one address, and the studio found on the cable.
            let until = Instant::now() + PATIENCE;
            let networks = loop {
                let networks = discovery_networks(&reported_by_the_kernel().unwrap());
                if networks.len() == 2 || Instant::now() > until {
                    break networks;
                }
                std::thread::sleep(Duration::from_millis(50));
            };
            assert_eq!(networks.len(), 2, "{networks:?}");
            let studios_address = IpAddr::V4(THE_STUDIOS_ADDRESS);
            let found = loop {
                if let Some(found) = OnEveryNetwork.look_for(&the_studio())
                    && found.address == studios_address
                {
                    break found;
                }
                assert!(Instant::now() < until, "the studio was never found");
            };
            assert_eq!(found.address.interface(), Some(cable), "{found:?}");
            assert!(found.also_at.is_empty(), "{found:?}");
            assert_eq!(
                found.where_it_answers(),
                SocketAddr::new(studios_address, QUESTIONS)
            );
            println!("found on the cable alone, held to its interface");

            // 2. The question, from a turn the kernel bounds.
            let said = one_question(
                bounding,
                &mut indicator,
                &mut record,
                &mut questions,
                &corridor,
            );
            assert!(
                matches!(&said, ToAnAgent::Answered { text, .. } if text == "Three are unpaid."),
                "{said:?}"
            );
            assert!(
                indicator.is_quiet(),
                "the departure was left on the indicator"
            );
            let left = departures(&record);
            assert_eq!(left.len(), 1, "{record:?}");
            assert!(
                matches!(
                    left.first(),
                    Some(Happened::Left {
                        destination: Destination::PairedMachine { machine },
                        ..
                    }) if machine == the_studio().as_str()
                ),
                "{left:?}"
            );
            assert_eq!(
                settled(&mut telling_studio, &mut hearing_studio, (1, 1)),
                (1, 1)
            );
            assert_eq!(
                counted(&mut telling_other, &mut hearing_other),
                (0, 0),
                "a question to the studio reached somebody else at its address"
            );
            println!("answered from a bounded turn, shown, and recorded naming the paired machine");

            // 3. The kernel decides on the interface, and both far ends witness it.
            let at = SocketAddr::new(studios_address, QUESTIONS);
            let (mut reached, mut elsewhere_refused, mut route_refused) = (None, None, None);
            bounding
                .carrying_out_a_departure_on(&[at], NonZeroU32::new(cable).unwrap(), &mut || {
                    reached = Some(connecting(Some(cable)));
                    elsewhere_refused = Some(connecting(Some(elsewhere)));
                    route_refused = Some(connecting(None));
                })
                .expect("a boundary can be put around a request");
            assert_eq!(reached, Some(Ok(())), "the cable was not reached");
            assert_eq!(
                elsewhere_refused,
                Some(Err(REFUSED)),
                "the studio's address on the other network was not refused by the kernel"
            );
            assert_eq!(
                route_refused,
                Some(Err(REFUSED)),
                "a socket held to no interface was permitted a departure held to one"
            );
            assert_eq!(
                settled(&mut telling_studio, &mut hearing_studio, (2, 1)),
                (2, 1)
            );
            assert_eq!(
                counted(&mut telling_other, &mut hearing_other),
                (0, 0),
                "a connection the kernel refused reached somebody else"
            );
            println!(
                "the kernel reached the cable and refused the same address on the other network"
            );

            // 4. A provider's departure is held to nothing, and goes by the route —
            // which, on this machine, is somebody else's network.
            let mut by_the_route = None;
            bounding
                .carrying_out_a_departure(&[at], &mut || {
                    by_the_route = Some(connecting(None));
                })
                .expect("a boundary can be put around a request");
            assert_eq!(by_the_route, Some(Ok(())));
            assert_eq!(
                settled(&mut telling_other, &mut hearing_other, (1, 0)),
                (1, 0),
                "the connection by the route did not go where the route goes"
            );
            assert_eq!(counted(&mut telling_studio, &mut hearing_studio), (2, 1));
            println!("a departure held to no interface went by the route");
        }));

        for telling in [&mut telling_studio, &mut telling_other] {
            drop(telling.write_all(b"stop\n"));
        }
        bounding.given_back().expect("the subtree is given back");
        if let Err(failed) = asked {
            std::panic::resume_unwind(failed);
        }
        said_by(&mut hearing_studio, "alo:stopped");
        said_by(&mut hearing_other, "alo:stopped");
        assert!(studio.wait().unwrap().success(), "the studio failed");
        assert!(other.wait().unwrap().success(), "somebody else failed");
    }

    /// Answer discovery on `socket` as the studio, until `stopping`.
    fn answering_discovery(socket: std::net::UdpSocket, stopping: Arc<AtomicBool>) {
        socket
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let answering = Answering::on(socket, Presence::of(the_studio(), QUESTIONS));
        std::thread::spawn(move || {
            while !stopping.load(Ordering::SeqCst) {
                // A timeout is a quiet moment on the link, not a failure.
                let _ = answering.answer_one();
            }
        });
    }

    /// Answer questions on `listener`, counting every connection and every
    /// question, until `stopping`.
    fn answering_questions(
        listener: TcpListener,
        connections: Arc<AtomicUsize>,
        questions: Arc<AtomicUsize>,
        stopping: Arc<AtomicBool>,
    ) {
        listener.set_nonblocking(true).unwrap();
        std::thread::spawn(move || {
            while !stopping.load(Ordering::SeqCst) {
                let Ok((stream, _)) = listener.accept() else {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                };
                connections.fetch_add(1, Ordering::SeqCst);
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
                    continue;
                }
                let mut length = 0_usize;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 || line.trim().is_empty() {
                        break;
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                        length = value.trim().parse().unwrap_or(0);
                    }
                }
                let mut body = vec![0_u8; length];
                drop(reader.read_exact(&mut body));
                if request_line.starts_with("POST ") {
                    questions.fetch_add(1, Ordering::SeqCst);
                }
                let mut stream = stream;
                drop(stream.write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{THE_STUDIOS_ANSWER}",
                        THE_STUDIOS_ANSWER.len()
                    )
                    .as_bytes(),
                ));
            }
        });
    }

    /// A machine at the far end of one cable: the studio, which answers
    /// discovery and questions, or somebody else, which answers only questions.
    #[test]
    #[ignore = "run inside its own network by reception_on_two_networks"]
    fn a_machine_at_the_far_end() {
        let which = env::var(INSIDE).unwrap();
        assert!(
            which == "studio" || which == "somebody else",
            "this test runs inside the namespace reception makes; run the outer one"
        );
        let mut told = std::io::stdin().lines();
        assert_eq!(told.next().unwrap().unwrap(), "cable");

        let stopping = Arc::new(AtomicBool::new(false));
        let connections = Arc::new(AtomicUsize::new(0));
        let questions = Arc::new(AtomicUsize::new(0));
        answering_questions(
            TcpListener::bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), QUESTIONS)).unwrap(),
            Arc::clone(&connections),
            Arc::clone(&questions),
            Arc::clone(&stopping),
        );
        if which == "studio" {
            let over_ipv4 = a_shared_datagram_socket_on(THE_PORT).unwrap();
            over_ipv4
                .join_multicast_v4(&THE_ADDRESS, &THE_STUDIOS_ADDRESS)
                .unwrap();
            answering_discovery(over_ipv4, Arc::clone(&stopping));
        }
        println!("alo:serving");

        for line in told {
            match line.unwrap().as_str() {
                "count" => println!(
                    "alo:counted {} {}",
                    connections.load(Ordering::SeqCst),
                    questions.load(Ordering::SeqCst)
                ),
                "stop" => {
                    stopping.store(true, Ordering::SeqCst);
                    println!("alo:stopped");
                    return;
                }
                other => panic!("a far end was told something it does not do: {other}"),
            }
        }
    }
}
