//! Two machines with no IPv4 address between them find each other again when
//! the cable comes back — on a real kernel, each running the whole service
//! throughout.
//!
//! *Machines find each other with zero configuration — no addresses typed.*
//! `crate::two_machines_with_no_ipv4` measured two machines on a cable with only
//! IPv6 link-local addresses finding each other and pairing, and
//! `crate::a_cable_pulled_and_plugged_in_again` measured a cable pulled and
//! plugged in again over IPv4. Over link-local the same cable is a different
//! thing: a link set down is a network whose address is usable again only once
//! the kernel has checked it, and a cable re-laid is a new interface index — and
//! a link-local address is only an address together with that index (ADR 0041).
//! This is that measurement.
//!
//! # The two machines
//!
//! As in `crate::two_machines_with_no_ipv4`: **reception** is this test binary
//! run again inside a network namespace of its own, inside a user namespace, and
//! **the studio** is the binary run a third time in a namespace nested inside
//! reception's, with one `veth` between them carrying no IPv4 address. Nothing
//! here touches kernel-global state, so this does not take
//! `alo_bounding::Waited::on_this_kernel()`. The studio does what it is told on
//! its standard input and answers each on one line.
//!
//! # A cable re-laid, and why the studio's namespace does not end
//!
//! The plan names two pulls: the far end's link set down, and the far end's
//! namespace ended with the cable re-laid at a new index. The studio *is* the far
//! end, and a namespace ends only once the last process in it has — so ending
//! the studio's would restart the studio's service, which the same criterion
//! forbids. What a namespace ending does to a `veth` is delete it
//! (`docs/quirks.md`, measured by `crate::a_cable_pulled_and_plugged_in_again`,
//! where the far end had no service to keep), and that is what the second pull
//! does, from the studio's side: the cable is deleted, both ends with it, and a
//! new one is laid — a new interface at each end, each at a new index and a new
//! link-local address, with both services running across it.
//!
//! # In this order
//!
//! 1. **The cable laid**: each machine finds the other at its link-local address
//!    with the interface it was heard on, and they pair through task 12's
//!    request.
//! 2. **The far end's link set down**: neither finds the other, a proposal to the
//!    studio is refused before anything is sent, and both services answer their
//!    person's door.
//! 3. **Set up again**: each finds the other on the same interface, once the
//!    kernel has finished checking the address — with no restart.
//! 4. **The cable deleted**: neither finds the other, and both services answer.
//! 5. **Re-laid**: each finds the other at the new link-local address **with the
//!    new interface**, and the old index is gone — the kernel refuses a
//!    connection to it.
//! 6. **A proposal to the paired machine** is measured on the new interface —
//!    the studio measures reception at the address its connection came from —
//!    and the two machines pair again.
//! 7. **Deleted again and re-laid at the numbers it first had**: each finds the
//!    other.
//!
//! **What is said is the same bytes throughout**, and at each step the studio
//! measures what the kernel did with an IPv6 membership, for `docs/quirks.md`.
//!
//! # What it found
//!
//! Step 7 failed at first: reception never found the studio. A link deleted
//! takes its interface out of the group and leaves the **socket's** membership
//! behind at that number, and `crate::joining` read the `EADDRINUSE` a join there
//! answers as *already joined* — so an interface given the number again was
//! counted joined with nobody in the group. `crate::joining` now leaves a
//! network that goes, and takes a refused join afresh.

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::collections::BTreeSet;
    use std::env;
    use std::io::{BufRead as _, BufReader, Write as _};
    use std::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6, TcpStream, UdpSocket};
    use std::num::NonZeroU16;
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::time::{Duration, Instant, SystemTime};

    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_keeping::Keeping;
    use alo_nearby::{
        MachineId, Presence, THE_IPV6_ADDRESS, THE_PORT, WorkspacePresence, advertising,
    };
    use alo_protocol::{AfterConfirming, ToAPerson};
    use alo_record::Record;
    use alo_turn::Machine;

    use crate::hosting::Hosted;
    use crate::looking::found_by_name;
    use crate::network::TheNetwork;
    use crate::networks::link_local_networks;
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
    use crate::words::NO_SUCH_MACHINE_ON_THE_NETWORK;

    /// Set on the binary run again inside a namespace, naming which machine it is.
    const INSIDE: &str = "ALO_AGENTD_NO_IPV4_AGAIN_INSIDE";

    /// The port the studio's workspace answers on.
    const WORKSPACE: u16 = 8_443;

    /// Reception's end of the cable.
    const RECEPTIONS_END: &str = "cable0";
    /// The studio's end of the cable.
    const STUDIOS_END: &str = "cable1";

    /// How long anything here waits for the kernel or the other machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// `ff02::fb` as `/proc/net/igmp6` spells a group.
    const THE_GROUP_IN_IGMP6: &str = "ff0200000000000000000000000000fb";

    /// **Two machines with no IPv4 address between them find each other again
    /// when the cable comes back**: the outer test, which makes reception and
    /// reads what it saw.
    #[test]
    fn two_machines_with_no_ipv4_address_find_each_other_again_when_the_cable_comes_back() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture two_machines_with_no_ipv4_find_each_other_again::tests::reception_while_the_cable_comes_and_goes";
        let ran = Command::new("unshare")
            .args(["--map-root-user", "--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "reception")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect(
                "util-linux's `unshare` makes the two machines, and this machine does not have it",
            );
        let said = String::from_utf8_lossy(&ran.stdout).into_owned();
        let log = String::from_utf8_lossy(&ran.stderr).into_owned();
        assert!(
            ran.status.success(),
            "reception failed ({}):\n{said}\n{log}",
            ran.status
        );
        for step in [
            "each found the other over link-local with its interface, and they paired",
            "the far end's link set down leaves each not found by the other, both services running",
            "a proposal to a machine on a pulled cable was refused before anything was sent",
            "set up again, each finds the other on the same interface without a restart",
            "the cable deleted leaves each not found by the other, both services running",
            "re-laid, each finds the other at the new address with the new interface",
            "the old interface is gone, and nothing is dialled at it",
            "re-laid at the interfaces it first had, each finds the other",
            "a proposal after the cable was re-laid was measured on the new interface and paired",
            "the same bytes throughout",
        ] {
            assert!(
                said.contains(step),
                "reception never said `{step}`:\n{said}\n{log}"
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

    /// Lay the cable between reception and the studio at `pid`, with no IPv4
    /// address on either end, and bring both ends up.
    fn lay_the_cable(pid: u32) {
        let pid = pid.to_string();
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
                &pid,
            ],
        );
        brought_up(&pid);
    }

    /// Lay the cable as [`lay_the_cable`] does, with reception's end numbered
    /// `receptions` and the studio's `studios`.
    fn lay_the_cable_at(pid: u32, receptions: u32, studios: u32) {
        let pid = pid.to_string();
        let (receptions, studios) = (receptions.to_string(), studios.to_string());
        ip(
            None,
            &[
                "link",
                "add",
                RECEPTIONS_END,
                "index",
                &receptions,
                "type",
                "veth",
                "peer",
                "name",
                STUDIOS_END,
                "index",
                &studios,
                "netns",
                &pid,
            ],
        );
        brought_up(&pid);
    }

    /// Bring both ends of the cable up, the studio's in the namespace of `pid`.
    fn brought_up(pid: &str) {
        let pid = pid.parse().unwrap();
        ip(None, &["link", "set", RECEPTIONS_END, "up"]);
        ip(Some(pid), &["link", "set", STUDIOS_END, "up"]);
    }

    /// The kernel's index for the interface `name` and its link-local address,
    /// once the link is running and the kernel has finished checking the
    /// address.
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

    /// Wait until the wire's IPv6 group is joined on `cable` exactly when `on`
    /// says so — the service following the kernel, measured from outside it.
    fn until_joined(wire: &Wire, cable: &str, on: bool) {
        let until = Instant::now() + PATIENCE;
        loop {
            let joined = wire.joined();
            if joined.iter().any(|network| network.name() == cable) == on {
                return;
            }
            assert!(
                Instant::now() < until,
                "the wire never followed {cable} to joined = {on}: {joined:?}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Wait until the kernel reports no interface called `cable`.
    fn until_the_kernel_has_no(cable: &str) {
        let until = Instant::now() + PATIENCE;
        while reported_by_the_kernel()
            .unwrap()
            .iter()
            .any(|interface| interface.name == cable)
        {
            assert!(Instant::now() < until, "{cable} outlived its deletion");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// The studio, as reception drives it.
    struct TheStudio {
        /// Its process, whose network namespace is the far end of the cable.
        process: Child,
        /// What it is told on.
        telling: ChildStdin,
        /// What it says on.
        hearing: BufReader<ChildStdout>,
    }

    impl TheStudio {
        /// Tell it `what` without waiting for an answer.
        fn tell(&mut self, what: &str) {
            self.telling
                .write_all(format!("{what}\n").as_bytes())
                .unwrap();
        }

        /// The rest of the next line it says under `prefix`, waiting for it.
        fn says(&mut self, prefix: &str) -> String {
            let prefix = format!("alo:{prefix}");
            let mut line = String::new();
            loop {
                line.clear();
                let read = self.hearing.read_line(&mut line).unwrap();
                assert!(read > 0, "the studio ended before saying `{prefix}`");
                if let Some(rest) = line.trim_end().strip_prefix(&prefix) {
                    return rest.trim().to_owned();
                }
            }
        }

        /// Tell it `what`, and read the one line it answers with.
        fn told(&mut self, what: &str) -> String {
            self.tell(what);
            let command = what.split_whitespace().next().unwrap_or(what).to_owned();
            self.says(&command)
        }

        /// The interface and address it found reception at just now, or
        /// `None` when it did not find it.
        fn finds_reception(&mut self) -> Option<(u32, IpAddr)> {
            let said = self.told("look");
            let (scope, address) = said.split_once(' ')?;
            Some((scope.parse().unwrap(), address.parse().unwrap()))
        }

        /// Wait until it finds reception, and where.
        fn until_it_finds_reception(&mut self) -> (u32, IpAddr) {
            let until = Instant::now() + PATIENCE;
            loop {
                if let Some(found) = self.finds_reception() {
                    return found;
                }
                assert!(Instant::now() < until, "the studio never found reception");
            }
        }

        /// Whether its person's door answered, with reception among its
        /// pairings.
        fn door_answers_paired_with_reception(&mut self) -> bool {
            self.told("door") == reception().as_str()
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

        /// The machines the door lists as paired, and how many proposals wait.
        fn pairings(&mut self) -> (Vec<String>, Vec<(String, Option<String>)>) {
            let listed = self.asking(r#"{"pairings":{}}"#);
            let paired = listed
                .paired()
                .expect("the door answers with the pairings")
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

        /// Propose to pair with `machine`, as task 12's request does.
        fn proposing_to(&mut self, machine: &MachineId) -> ToAPerson {
            self.asking(&format!(
                r#"{{"pair":{{"machine":"{}","may":["models"],"seconds":600}}}}"#,
                machine.as_str()
            ))
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
                .expect("a cable coming or going stopped the service");
            driving.join().unwrap();
        });
    }

    /// Both questions, asked over IPv6 from reception's link-local address on
    /// `cable`, and everything the studio at `studio` answered within a second.
    fn what_the_studio_says(cable: u32, here: Ipv6Addr, studio: IpAddr) -> BTreeSet<Vec<u8>> {
        let asking = UdpSocket::bind(SocketAddr::V6(SocketAddrV6::new(here, 0, 0, cable))).unwrap();
        let group = SocketAddr::V6(SocketAddrV6::new(THE_IPV6_ADDRESS, THE_PORT, 0, cable));
        asking
            .send_to(&advertising::a_question().unwrap(), group)
            .unwrap();
        asking
            .send_to(&advertising::a_question_for_workspaces().unwrap(), group)
            .unwrap();
        asking
            .set_read_timeout(Some(Duration::from_millis(1_000)))
            .unwrap();
        let mut heard = BTreeSet::new();
        let mut datagram = [0_u8; 1_500];
        while let Ok((read, who)) = asking.recv_from(&mut datagram) {
            if who.ip() == studio {
                heard.insert(datagram.get(..read).unwrap().to_vec());
            }
        }
        heard
    }

    /// Wait until reception finds the studio on the wire, and hand back what it
    /// found.
    fn until_reception_finds_the_studio(wire: &Wire) -> alo_nearby::Found {
        let until = Instant::now() + PATIENCE;
        loop {
            if let Some(found) = found_by_name(&the_studio(), wire.looks_at()) {
                return found;
            }
            assert!(Instant::now() < until, "reception never found the studio");
        }
    }

    /// Pair with the studio through reception's door, the studio's person
    /// confirming the code they are shown, and hand back what reception found
    /// the studio at when proposing.
    fn paired_with_the_studio(person: &mut Talking, studio: &mut TheStudio) {
        studio.tell("confirm");
        let proposed = person.proposing_to(&the_studio());
        let waiting = proposed
            .proposed_pairing()
            .unwrap_or_else(|| panic!("the proposal was not measured: {proposed:?}"));
        let code = waiting.code().unwrap().to_owned();
        assert_eq!(studio.says("code"), code, "the codes differ");
        studio.says("confirmed");
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
    }

    /// Reception, while the cable between it and the studio comes and goes.
    #[test]
    #[ignore = "run inside its own network by two_machines_with_no_ipv4_address_find_each_other_again_when_the_cable_comes_back"]
    fn reception_while_the_cable_comes_and_goes() {
        must_be("reception");
        let exe = env::current_exe().unwrap();
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture two_machines_with_no_ipv4_find_each_other_again::tests::the_studio_while_the_cable_comes_and_goes";
        let mut process = Command::new("unshare")
            .args(["--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "studio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("util-linux's `unshare` makes the studio");
        let ours = std::fs::read_link("/proc/self/ns/net").unwrap();
        let theirs = format!("/proc/{}/ns/net", process.id());
        let until = Instant::now() + Duration::from_secs(10);
        while std::fs::read_link(&theirs).ok().as_ref() == Some(&ours)
            || !Path::new(&theirs).exists()
        {
            assert!(Instant::now() < until, "the studio never left this network");
            std::thread::sleep(Duration::from_millis(20));
        }
        let pid = process.id();
        let mut studio = TheStudio {
            telling: process.stdin.take().unwrap(),
            hearing: BufReader::new(process.stdout.take().unwrap()),
            process,
        };

        lay_the_cable(pid);
        let (first_cable, first_address) = the_cable(RECEPTIONS_END);
        let studios_first_cable: u32 = studio.told("cable").parse().unwrap();

        serving_as(
            &reception(),
            "no-ipv4-again-reception",
            Hosted::Nothing,
            |wire, door, stop| {
                let mut person = Talking::to(door);
                studio.says("serving");
                let expected: BTreeSet<Vec<u8>> = [
                    advertising::about(&Presence::of(the_studio(), THE_WIRE_PORT)).unwrap(),
                    advertising::about_a_workspace(&WorkspacePresence::of(the_studio(), WORKSPACE))
                        .unwrap(),
                ]
                .into();

                // 1. The cable laid: each finds the other, and they pair.
                until_joined(wire, RECEPTIONS_END, true);
                let found = until_reception_finds_the_studio(wire);
                assert!(found.address.is_link_local(), "{found:?}");
                assert_eq!(found.address.scope(), Some(first_cable), "{found:?}");
                let studios_first_address = found.address.ip();
                let (scope, at) = studio.until_it_finds_reception();
                assert_eq!(scope, studios_first_cable);
                assert_eq!(at, IpAddr::V6(first_address));
                paired_with_the_studio(&mut person, &mut studio);
                assert!(studio.door_answers_paired_with_reception());
                let said = what_the_studio_says(first_cable, first_address, studios_first_address);
                assert_eq!(said, expected, "what the studio said over link-local");
                assert_eq!(studio.told("membership"), "joined");
                println!(
                    "each found the other over link-local with its interface, and they paired"
                );

                // 2. The far end's link set down.
                ip(Some(pid), &["link", "set", STUDIOS_END, "down"]);
                until_joined(wire, RECEPTIONS_END, false);
                assert!(
                    found_by_name(&the_studio(), wire.looks_at()).is_none(),
                    "reception found the studio on a pulled cable"
                );
                assert_eq!(studio.told("look"), "nothing");
                // What the kernel did with the studio's membership on a link set
                // down: kept on the socket, and on the interface.
                assert_eq!(studio.told("membership"), "kept");
                let refused = person.proposing_to(&the_studio());
                assert!(refused.proposed_pairing().is_none(), "{refused:?}");
                assert_eq!(
                    refused.refusal().map(alo_protocol::Wording::text),
                    Some(NO_SUCH_MACHINE_ON_THE_NETWORK.says()),
                    "{refused:?}"
                );
                let (paired, waiting) = person.pairings();
                assert_eq!(paired, vec![the_studio().as_str().to_owned()]);
                assert!(waiting.is_empty(), "a refused proposal left one waiting");
                assert!(studio.door_answers_paired_with_reception());
                println!(
                    "the far end's link set down leaves each not found by the other, both services running"
                );
                println!(
                    "a proposal to a machine on a pulled cable was refused before anything was sent"
                );

                // 3. Set up again: the same interface, once the address is checked.
                ip(Some(pid), &["link", "set", STUDIOS_END, "up"]);
                assert_eq!(the_cable(RECEPTIONS_END).0, first_cable);
                assert_eq!(
                    studio.told("cable").parse::<u32>().unwrap(),
                    studios_first_cable
                );
                until_joined(wire, RECEPTIONS_END, true);
                let found = until_reception_finds_the_studio(wire);
                assert_eq!(found.address.scope(), Some(first_cable), "{found:?}");
                assert_eq!(found.address.ip(), studios_first_address);
                assert_eq!(
                    studio.until_it_finds_reception(),
                    (studios_first_cable, IpAddr::V6(first_address))
                );
                assert_eq!(
                    what_the_studio_says(first_cable, first_address, studios_first_address),
                    expected
                );
                println!(
                    "set up again, each finds the other on the same interface without a restart"
                );

                // 4. The cable deleted, both ends with it.
                ip(Some(pid), &["link", "del", STUDIOS_END]);
                until_the_kernel_has_no(RECEPTIONS_END);
                until_joined(wire, RECEPTIONS_END, false);
                assert!(
                    found_by_name(&the_studio(), wire.looks_at()).is_none(),
                    "reception found the studio with no cable at all"
                );
                assert_eq!(studio.told("look"), "nothing");
                // What the kernel did with the studio's membership on a link
                // deleted: no interface is in the group at that number, and the socket
                // still holds a membership there all the same.
                assert_eq!(studio.told("membership"), "kept on the socket alone");
                assert!(
                    person
                        .pairings()
                        .0
                        .contains(&the_studio().as_str().to_owned())
                );
                assert!(studio.door_answers_paired_with_reception());
                println!(
                    "the cable deleted leaves each not found by the other, both services running"
                );

                // 5. Re-laid: a new interface at each end.
                lay_the_cable(pid);
                let (cable, address) = the_cable(RECEPTIONS_END);
                let studios_cable: u32 = studio.told("cable").parse().unwrap();
                assert_ne!(cable, first_cable, "a re-laid cable kept its index");
                assert_ne!(studios_cable, studios_first_cable);
                until_joined(wire, RECEPTIONS_END, true);
                let found = until_reception_finds_the_studio(wire);
                assert!(found.address.is_link_local(), "{found:?}");
                assert_eq!(found.address.scope(), Some(cable), "{found:?}");
                assert_ne!(found.address.ip(), studios_first_address);
                assert!(
                    matches!(found.where_it_answers(), SocketAddr::V6(at) if at.scope_id() == cable),
                    "{found:?}"
                );
                assert_eq!(
                    studio.until_it_finds_reception(),
                    (studios_cable, IpAddr::V6(address))
                );
                assert_eq!(
                    what_the_studio_says(cable, address, found.address.ip()),
                    expected
                );
                // The studio's membership on the new interface, and nothing to
                // keep on the old one.
                assert_eq!(studio.told("membership"), "joined");
                println!("re-laid, each finds the other at the new address with the new interface");

                // The old index names nothing: a connection to the studio at it is
                // refused by the kernel before anything leaves.
                let IpAddr::V6(studios_address) = found.address.ip() else {
                    panic!("a link-local address that is not IPv6: {found:?}");
                };
                let at_the_old_index = SocketAddr::V6(SocketAddrV6::new(
                    studios_address,
                    THE_WIRE_PORT,
                    0,
                    first_cable,
                ));
                assert!(
                    TcpStream::connect_timeout(&at_the_old_index, Duration::from_secs(2)).is_err(),
                    "the studio was reached at an interface that no longer exists"
                );
                assert!(
                    crate::looking::found_at(
                        alo_nearby::HeardFrom::of(at_the_old_index),
                        crate::arrived_on::ArrivedOn::ItsOwnNetwork,
                        THE_PORT
                    )
                    .is_empty(),
                    "the studio was measured at an interface that no longer exists"
                );
                println!("the old interface is gone, and nothing is dialled at it");

                // 6. A proposal to the paired machine, after the cable was re-laid.
                // The studio shows a code only once its service has measured
                // reception at the address the proposal's connection came from —
                // an address with the new interface, since no other exists — and
                // found the machine that proposed there; a proposal measured
                // nowhere, or somewhere else, is refused before anybody is shown
                // anything, and this pairing would not complete.
                paired_with_the_studio(&mut person, &mut studio);
                assert!(studio.door_answers_paired_with_reception());
                println!(
                    "a proposal after the cable was re-laid was measured on the new interface and paired"
                );

                // 7. Deleted again, and re-laid at the interfaces it first had —
                //    the numbers every socket that joined in step 1 still holds a
                //    membership at, with no interface behind it.
                ip(Some(pid), &["link", "del", STUDIOS_END]);
                until_the_kernel_has_no(RECEPTIONS_END);
                until_joined(wire, RECEPTIONS_END, false);
                assert_eq!(studio.told("look"), "nothing");
                lay_the_cable_at(pid, first_cable, studios_first_cable);
                let (cable, address) = the_cable(RECEPTIONS_END);
                assert_eq!(cable, first_cable);
                assert_eq!(
                    studio.told("cable").parse::<u32>().unwrap(),
                    studios_first_cable
                );
                until_joined(wire, RECEPTIONS_END, true);
                let found = until_reception_finds_the_studio(wire);
                assert_eq!(found.address.scope(), Some(first_cable), "{found:?}");
                assert_eq!(
                    studio.until_it_finds_reception(),
                    (studios_first_cable, IpAddr::V6(address))
                );
                assert_eq!(
                    what_the_studio_says(cable, address, found.address.ip()),
                    expected
                );
                assert!(studio.door_answers_paired_with_reception());
                println!("re-laid at the interfaces it first had, each finds the other");
                println!("the same bytes throughout");

                assert_eq!(studio.told("stop"), "");
                assert!(stop.stop());
            },
        );
        drop(studio.telling);
        assert!(
            studio.process.wait().unwrap().success(),
            "the studio failed"
        );
    }

    /// Whether `/proc/net/igmp6` in this namespace lists the discovery group on
    /// the interface numbered `index`.
    fn the_interface_is_in_the_group(index: u32) -> bool {
        std::fs::read_to_string("/proc/net/igmp6")
            .unwrap()
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .any(|words| {
                words.first() == Some(&index.to_string().as_str())
                    && words.get(2) == Some(&THE_GROUP_IN_IGMP6)
            })
    }

    /// What joining the group again on `index` from a socket already joined
    /// there says: `kept` when the kernel still holds the membership, `joined`
    /// when it took a new one, or the error.
    fn joined_again(probe: &UdpSocket, index: u32) -> String {
        match probe.join_multicast_v6(&THE_IPV6_ADDRESS, index) {
            Ok(()) => "joined".to_owned(),
            Err(why) if why.kind() == std::io::ErrorKind::AddrInUse => "kept".to_owned(),
            Err(why) => format!("refused {why}"),
        }
    }

    /// The next line reception told this machine, waiting for it.
    fn next() -> String {
        let mut line = String::new();
        let read = std::io::stdin().read_line(&mut line).unwrap();
        assert!(read > 0, "reception ended without stopping the studio");
        line.trim_end().to_owned()
    }

    /// The studio, inside a network of its own nested in reception's: it serves
    /// throughout, and does what reception tells it.
    #[test]
    #[ignore = "run inside its own network by reception_while_the_cable_comes_and_goes"]
    fn the_studio_while_the_cable_comes_and_goes() {
        must_be("studio");

        assert_eq!(next(), "cable");
        let (mut cable, _) = the_cable(STUDIOS_END);
        println!("alo:cable {cable}");
        let hosted = Hosted::At(NonZeroU16::new(WORKSPACE).unwrap());
        // A socket of the studio's own, joined as the service's is, to measure
        // what the kernel does with a membership across each pull.
        let probe = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).unwrap();

        serving_as(
            &the_studio(),
            "no-ipv4-again-studio",
            hosted,
            |wire, door, stop| {
                let mut person = Talking::to(door);
                // Asked once so the service holds this door before anything arrives.
                assert!(person.pairings().0.is_empty());
                println!("alo:serving");
                loop {
                    match next().as_str() {
                        "cable" => {
                            cable = the_cable(STUDIOS_END).0;
                            println!("alo:cable {cable}");
                        }
                        "look" => match found_by_name(&reception(), wire.looks_at()) {
                            Some(found) => println!(
                                "alo:look {} {}",
                                found.address.scope().unwrap_or(0),
                                found.address.ip()
                            ),
                            None => println!("alo:look nothing"),
                        },
                        "door" => println!("alo:door {}", person.pairings().0.join(",")),
                        "membership" => {
                            // The first time on an interface this joins; afterwards
                            // it says whether the kernel kept the membership, and
                            // whether the interface itself is still in the group.
                            let said = joined_again(&probe, cable);
                            let listed = the_interface_is_in_the_group(cable);
                            match (said.as_str(), listed) {
                                ("joined", true) => println!("alo:membership joined"),
                                ("kept", true) => println!("alo:membership kept"),
                                ("kept", false) => {
                                    println!("alo:membership kept on the socket alone")
                                }
                                (said, listed) => {
                                    println!(
                                        "alo:membership {said}, listed on the interface: {listed}"
                                    );
                                }
                            }
                        }
                        "confirm" => {
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
                            println!("alo:code {code}");
                            let confirmed = person.asking(&format!(
                                r#"{{"confirm-pairing":{{"machine":"{}","code":"{code}"}}}}"#,
                                reception().as_str()
                            ));
                            assert_eq!(
                                confirmed.became_of_confirming(),
                                Some(AfterConfirming::WaitingForTheOtherPerson),
                                "{confirmed:?}"
                            );
                            println!("alo:confirmed");
                        }
                        "stop" => {
                            println!("alo:stop");
                            assert!(stop.stop());
                            return;
                        }
                        said => panic!("the studio was told something it does not do: {said}"),
                    }
                }
            },
        );
    }
}
