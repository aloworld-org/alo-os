//! A question from a turn to a paired machine found only over IPv6 — bounded by
//! the real programme, shown on the indicator and written in the record — on a
//! real kernel.
//!
//! *One GPU box serves the office — it is still egress, and the indicator still
//! fires.* Task 23 made a paired machine on a network with no IPv4 address
//! found, paired with and dialled at a scoped link-local address; ADR 0041 made
//! the departure for it the address, the port **and the interface**. The rules
//! are each tested with no kernel in them (`alo_bounding_map::Departure`,
//! `crate::bounding`, `alo_turn`'s registration) and on a kernel with no network
//! in them (`alo-bounding`'s `a_link_local_departure_names_its_interface.rs`);
//! this is the join, the way reception's machine has it.
//!
//! # The two machines
//!
//! **Reception** is this test binary run again inside a network namespace of its
//! own, made by util-linux's `unshare` **as root and in no user namespace** —
//! because it opens the boundary the outer test pinned and moves itself into
//! control groups, and both are the first namespace's. **The studio** is the
//! binary run a third time in a namespace nested inside reception's. Between
//! them is one `veth` pair with **no IPv4 address on either end**. Reception also
//! has a second interface, a `dummy` called `elsewhere0`, which is where the
//! studio's own link-local address is tried when nobody showed it there.
//!
//! The studio is not the whole service: what it has to be is a machine that
//! answers discovery the way `alo-nearby` does and a question the way the
//! corridor expects, and it counts every connection that reaches it — so *the
//! kernel refused it* is witnessed by the machine that would have been reached.
//! What the studio does with a question is task 11's and is tested there.
//!
//! The outer test holds `alo_bounding::Waited::on_this_kernel()` for the whole
//! of it and imposes the boundary; the children never take the lock again,
//! which is `docs/autonomy/SHARED_MAIN.md`'s rule for a parent and its test
//! child. Everything else dies with the namespaces.
//!
//! # In this order
//!
//! 1. Reception has no IPv4 network, and finds the studio **only** at its
//!    link-local address, with the cable's interface.
//! 2. An agent's question to the machine its person chose goes down the corridor
//!    from a turn bounded by the real programme — answered, **shown and taken off
//!    the indicator, and written as a departure naming the paired machine** —
//!    and the studio counts one connection and one question.
//! 3. The kernel, asked directly through the same `ByTheKernel`: the registered
//!    interface is reached; the studio's address on `elsewhere0` is refused with
//!    `EACCES`; and a boundary shown some other link-local address is refused the
//!    studio on the cable. The studio counts one connection more, which is the
//!    one that was permitted — nothing refused ever arrived.
//! 4. An IPv4 address on each end of the cable: the studio is found at IPv4
//!    first, the same question goes, and **the departure the record keeps is the
//!    same value** as the one kept over link-local — the same agent, the same
//!    place, the same reason.

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
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, TcpListener, TcpStream};
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdout, Command, Stdio};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    use alo_bounding::{Imposed, Pinned};
    use alo_capability::Grants;
    use alo_context::Context;
    use alo_egress::{Destination, Indicator};
    use alo_files::OnThisMachine;
    use alo_models::Catalogue;
    use alo_nearby::{
        Answering, Found, MachineId, MayAskIts, Presence, THE_ADDRESS, THE_IPV6_ADDRESS, THE_PORT,
    };
    use alo_protocol::ToAnAgent;
    use alo_record::{Happened, Record};
    use alo_turn::{Bounding as _, Machine, Turning};

    use crate::bounding::ByTheKernel;
    use crate::corridor::Corridor;
    use crate::doing::what_an_agent_said;
    use crate::looking::{LookingFor, found_by_name};
    use crate::network::TheNetwork;
    use crate::networks::{discovery_networks, link_local_networks};
    use crate::questions::{Questions, TheBound, WhoseKeyring};
    use crate::route_messages::reported_by_the_kernel;
    use crate::terms::NoNameYet;
    use crate::testing::{
        THE_STUDIOS_ANSWER, a_directory_of_our_own, a_message, hour, in_english, noon,
        paired_between, reception, the_studio,
    };
    use crate::unix::{a_shared_datagram_socket_on, a_shared_ipv6_datagram_socket_on};

    /// Set on the binary run again inside a namespace, naming which machine it is.
    const INSIDE: &str = "ALO_AGENTD_LINK_LOCAL_INSIDE";

    /// Where the outer test pinned the boundary, for reception to open.
    const PINNED: &str = "ALO_AGENTD_LINK_LOCAL_PINNED";

    /// The port the studio advertises and answers questions on.
    const QUESTIONS: u16 = 7_611;

    /// Reception's end of the cable.
    const RECEPTIONS_END: &str = "cable0";
    /// The studio's end of the cable.
    const STUDIOS_END: &str = "cable1";
    /// Reception's other interface, where nobody showed anything.
    const ELSEWHERE: &str = "elsewhere0";

    /// How long anything here waits for the kernel or the other machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// What an agent asks.
    const ASKED: &str = r#"{"ask":{"question":"how many invoices are unpaid?"}}"#;

    /// `EACCES`, which is the only thing a refusal by the boundary looks like.
    const REFUSED: i32 = 13;

    /// **A question from a turn to a paired machine found only over IPv6 is
    /// bounded, shown and recorded as an IPv4 one is**: the outer test, which
    /// takes the kernel, imposes the boundary, makes reception and reads what it
    /// saw.
    #[test]
    fn a_question_to_a_paired_machine_found_only_over_ipv6_leaves_nothing_the_indicator_did_not_show()
     {
        let _kernel = alo_bounding::Waited::on_this_kernel()
            .expect("this kernel can be taken, and nothing is forced if it cannot");
        let pinned = Pinned::beneath(
            &PathBuf::from("/sys/fs/bpf").join(format!("alo-link-local-{}", std::process::id())),
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
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_paired_machine_over_link_local::tests::reception_bounded_by_the_kernel";
        let ran = Command::new("unshare")
            .args(["--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "reception")
            .env(PINNED, pinned.root())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect("util-linux's `unshare` makes the two machines");
        drop(loaded);
        pinned.taken_away();

        let said = String::from_utf8_lossy(&ran.stdout).into_owned();
        assert!(
            ran.status.success(),
            "reception failed ({}):\n{said}",
            ran.status
        );
        for step in [
            "found only over IPv6, on the cable",
            "answered from a bounded turn, shown, and recorded naming the paired machine",
            "the kernel reached the registered interface and refused the same address elsewhere",
            "a boundary shown another address was refused the studio",
            "the departure over IPv4 is the departure over link-local",
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

    /// Run iproute2 with `args`, in the studio's network namespace when there is
    /// a studio named, and fail with what it said if it refuses.
    fn ip(studio: Option<u32>, args: &[&str]) {
        let mut command = match studio {
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
            .expect("iproute2's `ip` and util-linux's `nsenter` make the cable");
        assert!(
            ran.status.success(),
            "ip {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&ran.stderr)
        );
    }

    /// Wait until the studio's process is in a network namespace that is not
    /// this one — until then, a link moved to it would land here.
    fn until_it_has_its_own_network(studio: &Child) {
        let ours = std::fs::read_link("/proc/self/ns/net").unwrap();
        let theirs = format!("/proc/{}/ns/net", studio.id());
        let until = Instant::now() + Duration::from_secs(10);
        while std::fs::read_link(&theirs).ok().as_ref() == Some(&ours)
            || !Path::new(&theirs).exists()
        {
            assert!(Instant::now() < until, "the studio never left this network");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// The kernel's index for the interface `name` and its link-local address,
    /// once the kernel has finished checking it.
    fn the_link_local_on(name: &str) -> (u32, Ipv6Addr) {
        let until = Instant::now() + PATIENCE;
        loop {
            let reported = reported_by_the_kernel().unwrap();
            if let Some(network) = link_local_networks(&reported)
                .into_iter()
                .find(|network| network.name() == name)
                && let IpAddr::V6(address) = network.address()
            {
                return (network.index(), address);
            }
            assert!(
                Instant::now() < until,
                "{name} never had a usable link-local address: {reported:?}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// The kernel's number for the interface `name` in this namespace, read
    /// from `/proc/net/if_inet6` — `/sys/class/net` answers for the namespace
    /// `sysfs` was mounted in, and a `dummy` carries no multicast for
    /// `link_local_networks` to find it by.
    fn the_index_of(name: &str) -> u32 {
        let until = Instant::now() + PATIENCE;
        loop {
            let table = std::fs::read_to_string("/proc/net/if_inet6").unwrap();
            let found = table.lines().find_map(|line| {
                let columns: Vec<&str> = line.split_whitespace().collect();
                match columns.as_slice() {
                    [_, index, _, _, _, interface] if *interface == name => {
                        u32::from_str_radix(index, 16).ok()
                    }
                    _ => None,
                }
            });
            if let Some(index) = found {
                return index;
            }
            assert!(
                Instant::now() < until,
                "{name} never had an address: {table}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// The IPv4 address on the interface `name`, once the kernel reports one.
    fn the_ipv4_on(name: &str) -> Ipv4Addr {
        let until = Instant::now() + PATIENCE;
        loop {
            let reported = reported_by_the_kernel().unwrap();
            if let Some(network) = discovery_networks(&reported)
                .into_iter()
                .find(|network| network.name() == name)
                && let IpAddr::V4(address) = network.address()
            {
                return address;
            }
            assert!(Instant::now() < until, "{name} never had an IPv4 address");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// The next line the studio printed under `prefix`, waiting for it.
    fn the_studio_says(from: &mut BufReader<ChildStdout>, prefix: &str) -> String {
        let mut line = String::new();
        loop {
            line.clear();
            let read = from.read_line(&mut line).unwrap();
            assert!(read > 0, "the studio ended before saying `{prefix}`");
            if let Some(rest) = line.trim_end().strip_prefix(prefix) {
                return rest.to_owned();
            }
        }
    }

    /// How many connections and how many questions have reached the studio.
    fn counted(
        telling: &mut impl std::io::Write,
        hearing: &mut BufReader<ChildStdout>,
    ) -> (usize, usize) {
        telling.write_all(b"count\n").unwrap();
        let said = the_studio_says(hearing, "alo:counted ");
        let (connections, questions) = said.split_once(' ').unwrap();
        (connections.parse().unwrap(), questions.parse().unwrap())
    }

    /// What has reached the studio once it has stopped changing: counted until
    /// it is `expected` or [`PATIENCE`] runs out, and counted again a moment
    /// later — the studio accepts on a thread of its own, so a connection that
    /// arrived is counted a little after it was made, and one that should not
    /// have arrived is given the same moment to show up.
    fn settled(
        telling: &mut impl std::io::Write,
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

    /// Look for the studio until what is found satisfies `enough`.
    fn found_when(enough: impl Fn(&Found) -> bool) -> Found {
        let until = Instant::now() + PATIENCE;
        loop {
            if let Some(found) = OnEveryNetwork.look_for(&the_studio())
                && enough(&found)
            {
                return found;
            }
            assert!(
                Instant::now() < until,
                "the studio was never found as expected"
            );
        }
    }

    /// Reception's settings, choosing the studio to answer its questions.
    fn choosing_the_studio() -> Questions {
        let config = a_directory_of_our_own("link-local");
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

    /// A connection attempt's outcome: `Ok` when it was made, and the error
    /// number otherwise, zero for an error with none.
    fn connecting(to: SocketAddr) -> Result<(), i32> {
        TcpStream::connect_timeout(&to, Duration::from_secs(2))
            .map(drop)
            .map_err(|why| why.raw_os_error().unwrap_or(0))
    }

    /// Reception, inside a network of its own: it starts the studio, lays the
    /// cable with no IPv4 on it, and asks.
    #[test]
    #[ignore = "run inside its own network by a_question_to_a_paired_machine_found_only_over_ipv6_leaves_nothing_the_indicator_did_not_show"]
    fn reception_bounded_by_the_kernel() {
        must_be("reception");
        let pinned = Pinned::beneath(Path::new(&env::var(PINNED).unwrap()));
        let exe = env::current_exe().unwrap();
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_paired_machine_over_link_local::tests::the_studio_answering_on_a_cable";
        let mut studio = Command::new("unshare")
            .args(["--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "studio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("util-linux's `unshare` makes the studio");
        until_it_has_its_own_network(&studio);
        let pid = studio.id();
        let mut hearing = BufReader::new(studio.stdout.take().unwrap());
        let mut telling = studio.stdin.take().unwrap();

        // The cable with no IPv4 on it, and an interface of reception's own
        // where nobody is shown anything.
        ip(
            None,
            &[
                "link",
                "add",
                RECEPTIONS_END,
                "type",
                "veth",
                "peer",
                "name",
                STUDIOS_END,
                "netns",
                &pid.to_string(),
            ],
        );
        ip(None, &["link", "set", RECEPTIONS_END, "up"]);
        ip(Some(pid), &["link", "set", STUDIOS_END, "up"]);
        ip(None, &["link", "add", ELSEWHERE, "type", "dummy"]);
        ip(None, &["link", "set", ELSEWHERE, "up"]);
        ip(
            None,
            &[
                "-6",
                "addr",
                "add",
                "fe80::e1/64",
                "dev",
                ELSEWHERE,
                "nodad",
            ],
        );
        let (cable, _) = the_link_local_on(RECEPTIONS_END);
        let elsewhere = the_index_of(ELSEWHERE);
        telling.write_all(b"cable\n").unwrap();
        let studios_address: Ipv6Addr = the_studio_says(&mut hearing, "alo:serving ")
            .parse()
            .unwrap();

        let mut bounding =
            ByTheKernel::beneath(&pinned).expect("reception opens the boundary the loader pinned");
        // Everything that can fail runs inside this, so that a failure still gives
        // the control group subtree back: one left behind is a directory beside
        // every other test's on this kernel, and the next service to start here
        // would find it in the way.
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

            // 1. No IPv4 anywhere, and the studio found only over link-local.
            assert!(
                discovery_networks(&reported_by_the_kernel().unwrap()).is_empty(),
                "reception has an IPv4 network"
            );
            let found = found_when(|_| true);
            assert!(found.address.is_link_local(), "{found:?}");
            assert_eq!(found.address.scope(), Some(cable), "{found:?}");
            assert!(found.also_at.is_empty(), "{found:?}");
            assert_eq!(found.port, QUESTIONS);
            let on_the_cable =
                SocketAddr::V6(SocketAddrV6::new(studios_address, QUESTIONS, 0, cable));
            assert_eq!(found.where_it_answers(), on_the_cable);
            println!("found only over IPv6, on the cable");

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
            let over_link_local = departures(&record);
            assert_eq!(over_link_local.len(), 1, "{record:?}");
            assert!(
                matches!(
                    over_link_local.first(),
                    Some(Happened::Left {
                        destination: Destination::PairedMachine { machine },
                        ..
                    }) if machine == the_studio().as_str()
                ),
                "{over_link_local:?}"
            );
            assert_eq!(settled(&mut telling, &mut hearing, (1, 1)), (1, 1));
            println!("answered from a bounded turn, shown, and recorded naming the paired machine");

            // 3. The kernel decides on the interface, and the studio is the witness.
            let on_another_interface =
                SocketAddr::V6(SocketAddrV6::new(studios_address, QUESTIONS, 0, elsewhere));
            let (mut reached, mut refused) = (None, None);
            bounding
                .carrying_out_a_departure(&[on_the_cable], &mut || {
                    reached = Some(connecting(on_the_cable));
                    refused = Some(connecting(on_another_interface));
                })
                .expect("a boundary can be put around a request");
            assert_eq!(
                reached,
                Some(Ok(())),
                "the registered interface was not reached"
            );
            assert_eq!(
                refused,
                Some(Err(REFUSED)),
                "the studio's address on an interface nobody showed was not refused by the kernel"
            );
            println!(
                "the kernel reached the registered interface and refused the same address elsewhere"
            );

            let somebody_else = SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0xdead),
                QUESTIONS,
                0,
                cable,
            ));
            let mut unshown = None;
            bounding
                .carrying_out_a_departure(&[somebody_else], &mut || {
                    unshown = Some(connecting(on_the_cable));
                })
                .expect("a boundary can be put around a request");
            assert_eq!(unshown, Some(Err(REFUSED)));
            assert_eq!(
                settled(&mut telling, &mut hearing, (2, 1)),
                (2, 1),
                "a connection the kernel refused reached the studio"
            );
            println!("a boundary shown another address was refused the studio");

            // 4. IPv4 on the same cable: found there first, and the same departure.
            ip(
                None,
                &["addr", "add", "10.63.0.1/24", "dev", RECEPTIONS_END],
            );
            ip(
                Some(pid),
                &["addr", "add", "10.63.0.2/24", "dev", STUDIOS_END],
            );
            telling.write_all(b"ipv4\n").unwrap();
            the_studio_says(&mut hearing, "alo:answering over ipv4");
            let studios_ipv4 = IpAddr::V4(Ipv4Addr::new(10, 63, 0, 2));
            let found = found_when(|found| found.address == studios_ipv4);
            assert_eq!(
                found.where_it_answers(),
                SocketAddr::new(studios_ipv4, QUESTIONS)
            );
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
            assert!(indicator.is_quiet());
            let both = departures(&record);
            assert_eq!(both.len(), 2, "{record:?}");
            assert_eq!(
                both.first(),
                both.get(1),
                "a departure over link-local was shown and recorded differently from one over IPv4"
            );
            assert_eq!(settled(&mut telling, &mut hearing, (3, 2)), (3, 2));
            println!("the departure over IPv4 is the departure over link-local");
        }));

        drop(telling.write_all(b"stop\n"));
        bounding.given_back().expect("the subtree is given back");
        if let Err(failed) = asked {
            std::panic::resume_unwind(failed);
        }
        the_studio_says(&mut hearing, "alo:stopped");
        assert!(studio.wait().unwrap().success(), "the studio failed");
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

    /// Answer questions on `listener` as the studio, counting every connection
    /// and every question, until `stopping`.
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

    /// The studio, inside a network of its own nested in reception's: it waits
    /// for the cable, answers discovery over link-local and questions on its
    /// port, and does what reception tells it.
    #[test]
    #[ignore = "run inside its own network by reception_bounded_by_the_kernel"]
    fn the_studio_answering_on_a_cable() {
        must_be("studio");
        let mut told = std::io::stdin().lines();
        assert_eq!(told.next().unwrap().unwrap(), "cable");
        let (cable, address) = the_link_local_on(STUDIOS_END);

        let stopping = Arc::new(AtomicBool::new(false));
        let over_ipv6 = a_shared_ipv6_datagram_socket_on(THE_PORT).unwrap();
        over_ipv6
            .join_multicast_v6(&THE_IPV6_ADDRESS, cable)
            .unwrap();
        answering_discovery(over_ipv6, Arc::clone(&stopping));
        let connections = Arc::new(AtomicUsize::new(0));
        let questions = Arc::new(AtomicUsize::new(0));
        answering_questions(
            TcpListener::bind(SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), QUESTIONS)).unwrap(),
            Arc::clone(&connections),
            Arc::clone(&questions),
            Arc::clone(&stopping),
        );
        println!("alo:serving {address}");

        for line in told {
            match line.unwrap().as_str() {
                "ipv4" => {
                    let here = the_ipv4_on(STUDIOS_END);
                    let over_ipv4 = a_shared_datagram_socket_on(THE_PORT).unwrap();
                    over_ipv4.join_multicast_v4(&THE_ADDRESS, &here).unwrap();
                    answering_discovery(over_ipv4, Arc::clone(&stopping));
                    println!("alo:answering over ipv4");
                }
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
                other => panic!("the studio was told something it does not do: {other}"),
            }
        }
    }
}
