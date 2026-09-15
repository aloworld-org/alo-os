//! Two machines with no IPv4 address between them find each other and pair —
//! on a real kernel, each running the whole service.
//!
//! *Machines find each other with zero configuration — no addresses typed.* Two
//! machines joined by one cable with no DHCP server between them have no IPv4
//! address in common, and every interface still gives itself an IPv6 link-local
//! one. The rules are each tested with no kernel in them (`crate::networks`,
//! `crate::route_messages`, `alo_nearby::HeardFrom`, `alo_nearby::reading`);
//! this is the join, the way two machines have it.
//!
//! # The two machines
//!
//! **Reception** is this test binary run again inside a network namespace of its
//! own, made by util-linux's `unshare` inside a user namespace. **The studio**
//! is the binary run a third time, in a namespace nested inside reception's.
//! Between them is one `veth` pair — the cable — brought up with **no IPv4
//! address on either end**, so each machine has exactly one network and the
//! only address on it is the link-local one the kernel made. Each machine runs
//! `crate::serving::Serving` over `Wire::bound`, as `src/main.rs` does, with a
//! person's door a thread of the test talks to.
//!
//! Nothing here touches kernel-global state: both namespaces are private to a
//! user namespace this test made and the cable dies with them, so this does
//! not take `alo_bounding::Waited::on_this_kernel()`.
//!
//! # In this order
//!
//! 1. Both machines serve; reception checks it has no IPv4 network at all, and
//!    finds the studio at its link-local address **with the interface it was
//!    heard on**.
//! 2. Reception's person asks to pair with the studio by its identity — task
//!    12's request — and is shown a code; the studio's person is shown the same
//!    code on their own door and confirms; reception's person confirms; each
//!    machine lists the other as paired. The studio measured reception over
//!    link-local while reception's service waited on the studio's reply, which
//!    only works because discovery is answered beside the service
//!    (`crate::answering_discovery`).
//! 3. An IPv4 address is put on each end of the cable. The studio's service
//!    follows the kernel and joins discovery over IPv4 as well; reception looks
//!    again and hears **one machine and one workspace with an address in each
//!    family**, the IPv4 one first, which is what a pairing dials; and what the
//!    studio answers is the same bytes over IPv4 and over IPv6.

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::collections::BTreeSet;
    use std::env;
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket};
    use std::num::NonZeroU16;
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdout, Command, Stdio};
    use std::time::{Duration, Instant, SystemTime};

    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_keeping::Keeping;
    use alo_nearby::{
        MachineId, Presence, THE_ADDRESS, THE_IPV6_ADDRESS, THE_PORT, WorkspacePresence,
        advertising,
    };
    use alo_protocol::{AfterConfirming, ToAPerson};
    use alo_record::Record;
    use alo_turn::Machine;

    use crate::hosting::Hosted;
    use crate::looking::{around_at, found_by_name};
    use crate::network::TheNetwork;
    use crate::networks::{discovery_networks, link_local_networks};
    use crate::rereading::WhatIsGranted;
    use crate::route_messages::reported_by_the_kernel;
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

    /// Set on the binary run again inside a namespace, naming which machine it is.
    const INSIDE: &str = "ALO_AGENTD_NO_IPV4_INSIDE";

    /// The port the studio's workspace answers on.
    const WORKSPACE: u16 = 8_443;

    /// Reception's end of the cable.
    const RECEPTIONS_END: &str = "cable0";
    /// The studio's end of the cable.
    const STUDIOS_END: &str = "cable1";

    /// How long anything here waits for the kernel or the other machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// **Two machines with no IPv4 address between them find each other and
    /// pair**: the outer test, which makes reception and reads what it saw.
    #[test]
    fn two_machines_with_no_ipv4_address_between_them_find_each_other_and_pair() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture two_machines_with_no_ipv4::tests::reception_on_a_cable_with_no_ipv4";
        let ran = Command::new("unshare")
            .args(["--map-root-user", "--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "reception")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .output()
            .expect(
                "util-linux's `unshare` makes the two machines, and this machine does not have it",
            );
        let said = String::from_utf8_lossy(&ran.stdout).into_owned();
        assert!(
            ran.status.success(),
            "reception failed ({}):\n{said}",
            ran.status
        );
        for step in [
            "no IPv4 network between them",
            "found at a link-local address with its interface",
            "the same code on both machines",
            "paired over link-local on both machines",
            "one machine and one workspace in both families, IPv4 first",
            "the same bytes in both families",
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

    /// The kernel's index for the interface `name` and its link-local address,
    /// once the kernel has finished checking it.
    fn the_cable(name: &str) -> (u32, Ipv6Addr) {
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

    /// Run this machine as the service runs it — `Wire::bound` as `here`,
    /// hosting `hosted`, with a person's door — while `person` drives the door
    /// from a thread that may borrow the wire, and stop when `person` stops it.
    fn serving_as(
        here: &MachineId,
        what: &str,
        hosted: Hosted,
        person: impl FnOnce(&Wire, &Path, Stop) + Send,
    ) {
        let strings = in_english();
        let (folder, _) = a_folder_with_an_invoice(what);
        let (waking, stop) = Waking::made().unwrap();
        let knocking = Pretending::handing_out(what, &[Some(Side::Person)]);
        let door: PathBuf = knocking.at();
        let wire = Wire::bound(here.clone()).unwrap().hosting(hosted);
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

    /// Everything that came back to `socket` within a second, from `from`.
    fn heard_from(socket: &UdpSocket, from: IpAddr) -> BTreeSet<Vec<u8>> {
        socket
            .set_read_timeout(Some(Duration::from_millis(1_000)))
            .unwrap();
        let mut heard = BTreeSet::new();
        let mut datagram = [0_u8; 1_500];
        while let Ok((read, who)) = socket.recv_from(&mut datagram) {
            if who.ip() == from {
                heard.insert(datagram.get(..read).unwrap().to_vec());
            }
        }
        heard
    }

    /// Both questions, asked from `here` at `group`.
    fn both_questions(here: SocketAddr, group: SocketAddr) -> UdpSocket {
        let asking = UdpSocket::bind(here).unwrap();
        asking
            .send_to(&advertising::a_question().unwrap(), group)
            .unwrap();
        asking
            .send_to(&advertising::a_question_for_workspaces().unwrap(), group)
            .unwrap();
        asking
    }

    /// Reception, inside a network of its own: it starts the studio, lays the
    /// cable with no IPv4 on it, and pairs.
    #[test]
    #[ignore = "run inside its own network by two_machines_with_no_ipv4_address_between_them_find_each_other_and_pair"]
    fn reception_on_a_cable_with_no_ipv4() {
        must_be("reception");
        let exe = env::current_exe().unwrap();
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture two_machines_with_no_ipv4::tests::the_studio_on_a_cable_with_no_ipv4";
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

        // The cable, with no IPv4 address on either end.
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
        let (cable, receptions_address) = the_cable(RECEPTIONS_END);
        telling.write_all(b"cable\n").unwrap();

        serving_as(
            &reception(),
            "no-ipv4-reception",
            Hosted::Nothing,
            |wire, door, stop| {
                // 1. No IPv4 anywhere, and the studio found over link-local.
                assert!(
                    discovery_networks(&reported_by_the_kernel().unwrap()).is_empty(),
                    "reception has an IPv4 network"
                );
                println!("no IPv4 network between them");
                the_studio_says(&mut hearing, "serving");
                let until = Instant::now() + PATIENCE;
                let found = loop {
                    if let Some(found) = found_by_name(&the_studio(), wire.looks_at()) {
                        break found;
                    }
                    assert!(Instant::now() < until, "the studio was never found");
                };
                assert!(found.address.is_link_local(), "{found:?}");
                assert_eq!(found.address.scope(), Some(cable), "{found:?}");
                assert!(found.also_at.is_empty(), "{found:?}");
                assert_eq!(found.port, THE_WIRE_PORT);
                assert!(
                    matches!(found.where_it_answers(), SocketAddr::V6(at) if at.scope_id() == cable),
                    "{found:?}"
                );
                println!("found at a link-local address with its interface");

                // 2. Task 12's request, from reception's person.
                let mut person = Talking::to(door);
                let proposed = person.asking(&format!(
                    r#"{{"pair":{{"machine":"{}","may":["models"],"seconds":600}}}}"#,
                    the_studio().as_str()
                ));
                let waiting = proposed
                    .proposed_pairing()
                    .expect("the proposal came back with the code");
                let code = waiting.code().unwrap().to_owned();
                assert_eq!(the_studio_says(&mut hearing, "code "), code);
                println!("the same code on both machines");
                the_studio_says(&mut hearing, "confirmed");
                let confirmed = person.asking(&format!(
                    r#"{{"confirm-pairing":{{"machine":"{}","code":"{code}"}}}}"#,
                    the_studio().as_str()
                ));
                assert_eq!(
                    confirmed.became_of_confirming(),
                    Some(AfterConfirming::Paired),
                    "{confirmed:?}"
                );
                let (paired, waiting) = person.pairings();
                assert_eq!(paired, vec![the_studio().as_str().to_owned()]);
                assert!(waiting.is_empty(), "{waiting:?}");
                assert_eq!(
                    the_studio_says(&mut hearing, "paired "),
                    reception().as_str()
                );
                println!("paired over link-local on both machines");

                // 3. An IPv4 address on each end of the same cable.
                ip(
                    None,
                    &["addr", "add", "10.62.0.1/24", "dev", RECEPTIONS_END],
                );
                ip(
                    Some(pid),
                    &["addr", "add", "10.62.0.2/24", "dev", STUDIOS_END],
                );
                let studios_ipv4 = IpAddr::V4(Ipv4Addr::new(10, 62, 0, 2));
                let until = Instant::now() + PATIENCE;
                let around = loop {
                    let around = around_at(wire.looks_at());
                    let heard_in_both = |addresses: Vec<alo_nearby::HeardFrom>| {
                        addresses.iter().any(|at| at.ip() == studios_ipv4)
                            && addresses.iter().any(alo_nearby::HeardFrom::is_link_local)
                    };
                    let machine = around
                        .machines
                        .iter()
                        .find(|one| one.machine == the_studio());
                    let workspace = around.workspaces.first();
                    if machine.is_some_and(|one| heard_in_both(one.addresses().collect()))
                        && workspace.is_some_and(|one| heard_in_both(one.addresses().collect()))
                    {
                        break around;
                    }
                    assert!(
                        Instant::now() < until,
                        "the studio was never heard in both families: {around:?}"
                    );
                };
                let machine = around
                    .machines
                    .iter()
                    .find(|one| one.machine == the_studio())
                    .unwrap();
                assert_eq!(machine.address, studios_ipv4, "{machine:?}");
                assert_eq!(
                    machine.where_it_answers(),
                    SocketAddr::new(studios_ipv4, THE_WIRE_PORT)
                );
                assert_eq!(machine.also_at.len(), 1, "{machine:?}");
                assert_eq!(machine.also_at.first().unwrap().scope(), Some(cable));
                assert_eq!(around.workspaces.len(), 1, "{around:?}");
                let workspace = around.workspaces.first().unwrap();
                assert_eq!(workspace.host(), &the_studio());
                assert_eq!(workspace.address(), studios_ipv4);
                assert_eq!(workspace.port(), WORKSPACE);
                println!("one machine and one workspace in both families, IPv4 first");

                let studios_link_local = machine.also_at.first().unwrap().ip();
                let expected: BTreeSet<Vec<u8>> = [
                    advertising::about(&Presence::of(the_studio(), THE_WIRE_PORT)).unwrap(),
                    advertising::about_a_workspace(&WorkspacePresence::of(the_studio(), WORKSPACE))
                        .unwrap(),
                ]
                .into();
                let over_ipv4 = both_questions(
                    SocketAddr::new(Ipv4Addr::new(10, 62, 0, 1).into(), 0),
                    SocketAddr::new(THE_ADDRESS.into(), THE_PORT),
                );
                let over_ipv6 = both_questions(
                    SocketAddr::V6(SocketAddrV6::new(receptions_address, 0, 0, cable)),
                    SocketAddr::V6(SocketAddrV6::new(THE_IPV6_ADDRESS, THE_PORT, 0, cable)),
                );
                assert_eq!(heard_from(&over_ipv4, studios_ipv4), expected, "over IPv4");
                assert_eq!(
                    heard_from(&over_ipv6, studios_link_local),
                    expected,
                    "over IPv6"
                );
                println!("the same bytes in both families");

                telling.write_all(b"stop\n").unwrap();
                the_studio_says(&mut hearing, "stopped");
                assert!(stop.stop());
            },
        );
        assert!(studio.wait().unwrap().success(), "the studio failed");
    }

    /// The studio, inside a network of its own nested in reception's: it waits
    /// for the cable, serves, and its person confirms what they are shown.
    #[test]
    #[ignore = "run inside its own network by reception_on_a_cable_with_no_ipv4"]
    fn the_studio_on_a_cable_with_no_ipv4() {
        must_be("studio");

        assert_eq!(std::io::stdin().lines().next().unwrap().unwrap(), "cable");
        the_cable(STUDIOS_END);
        let hosted = Hosted::At(NonZeroU16::new(WORKSPACE).unwrap());

        serving_as(&the_studio(), "no-ipv4-studio", hosted, |_, door, stop| {
            let mut person = Talking::to(door);
            // Asked once so the service holds this door before anything arrives.
            let (paired, _) = person.pairings();
            assert!(paired.is_empty());
            println!("serving");

            // The proposal, shown here with its code.
            let until = Instant::now() + PATIENCE;
            let code = loop {
                let (_, waiting) = person.pairings();
                if let Some((machine, Some(code))) = waiting.first() {
                    assert_eq!(machine, reception().as_str());
                    break code.clone();
                }
                assert!(Instant::now() < until, "no proposal was shown here");
                std::thread::sleep(Duration::from_millis(100));
            };
            println!("code {code}");
            let confirmed = person.asking(&format!(
                r#"{{"confirm-pairing":{{"machine":"{}","code":"{code}"}}}}"#,
                reception().as_str()
            ));
            assert_eq!(
                confirmed.became_of_confirming(),
                Some(AfterConfirming::WaitingForTheOtherPerson),
                "{confirmed:?}"
            );
            println!("confirmed");

            let until = Instant::now() + PATIENCE;
            loop {
                let (paired, _) = person.pairings();
                if let Some(machine) = paired.first() {
                    println!("paired {machine}");
                    break;
                }
                assert!(
                    Instant::now() < until,
                    "reception's confirmation never arrived"
                );
                std::thread::sleep(Duration::from_millis(100));
            }

            assert_eq!(std::io::stdin().lines().next().unwrap().unwrap(), "stop");
            assert!(stop.stop());
            println!("stopped");
        });
    }
}
