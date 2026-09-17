//! A link-local cable deleted and laid again with the same hardware address
//! between two readings of the interfaces is still joined — on a real kernel,
//! with both machines serving as `src/main.rs` serves throughout.
//!
//! *Machines find each other with zero configuration — no addresses typed.*
//! `crate::two_machines_with_no_ipv4_find_each_other_again` re-laid a link-local
//! cable at its old numbers, but with a **new** hardware address, so its
//! link-local address changed and the network compared unequal whatever
//! `crate::joining` read. `crate::a_cable_re_laid_between_two_readings` held a
//! machine still across a re-laying, over IPv4. What neither measured is the cable
//! that comes back **identical** — the same number, the same name, the same
//! hardware address and so the same `fe80::` address, because the adapter is the
//! same adapter: a USB dongle pulled and pushed back in. Reading the dumps alone,
//! that network never went, and the only thing that says it did is the kernel's
//! `RTM_DELLINK` (`crate::interfaces_that_went`).
//!
//! # The two machines, and how the ordering is made certain
//!
//! **The studio** is this test binary run again in a network namespace of its own,
//! inside a user namespace; it serves, and it drives. **Reception** is the binary
//! run a third time in a namespace nested inside the studio's, serving too, and
//! answering on its standard output what it is told on its standard input. One
//! `veth` joins them, with no IPv4 address on it, laid with both interface numbers
//! **and both hardware addresses** given. Nothing here touches kernel-global
//! state, so this does not take `alo_bounding::Waited::on_this_kernel()`.
//!
//! Reception is the machine that is asked, and it is **held still** as
//! `crate::a_cable_re_laid_between_two_readings` holds its machine: the studio stops
//! it with `SIGSTOP`, waits until the kernel reports every thread of it stopped,
//! deletes the cable, lays it again identically, and waits until **both
//! link-local addresses have finished duplicate address detection** before it sends
//! `SIGCONT`. So everything the kernel says about the re-laying is already queued on
//! reception's routing socket, and the first dump reception reads is of an interface
//! with the number, name and link-local address it had — nothing a dump could tell
//! apart from the interface that went. The studio is not held, and follows its own
//! end of the cable in rounds of its own: only reception is between two readings.
//!
//! *Once it has followed the kernel* is read from the kernel: nothing in
//! reception's network but its service joins `ff02::fb`, so the re-laid interface
//! listed in the group in `/proc/<pid>/net/igmp6` is the service having joined it.
//! Until it is, nothing is asked; the studio's **first** question after that is the
//! one that must find reception.
//!
//! # In this order
//!
//! 1. **The cable laid**: each finds the other over link-local with its interface,
//!    and the studio proposes and they pair.
//! 2. **Held still, the cable deleted**: the studio finds nothing, and a proposal to
//!    reception is refused before anything is sent.
//! 3. **Laid again identically, still held**: the same numbers, names, hardware
//!    addresses and link-local addresses at both ends; reception's re-laid interface
//!    is **not** in the group although its service joined at that number; and the
//!    studio still finds nothing.
//! 4. **Let go**: once reception has followed the kernel the studio's first question
//!    finds it on the same interface at the same address, both questions are
//!    answered with the same bytes as before, and the studio proposes and they pair
//!    on that interface.
//!
//! # What it shows
//!
//! With `crate::joining`'s reading of what went removed — `kept_and_gone` keeping
//! every network that is reported — reception keeps the network in its list, never
//! joins the re-laid interface, and the fixture fails at step 4 with *reception never
//! followed its cable to 40 … the interface is not in the discovery group*.

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
    use std::net::{IpAddr, Ipv6Addr, SocketAddr, SocketAddrV6, UdpSocket};
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
    use crate::networks::{Network, link_local_networks};
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
    const INSIDE: &str = "ALO_AGENTD_SAME_HARDWARE_ADDRESS_INSIDE";

    /// The port reception's workspace answers on.
    const WORKSPACE: u16 = 8_443;

    /// Reception's end of the cable.
    const RECEPTIONS_END: &str = "cable0";
    /// The studio's end of the cable.
    const STUDIOS_END: &str = "cable1";

    /// The numbers the cable is laid at every time, reception's end first.
    const NUMBERS: (u32, u32) = (40, 41);
    /// The hardware addresses it is laid with every time, reception's end first
    /// — locally administered, so they name no real adapter.
    const HARDWARE: (&str, &str) = ("02:a1:0c:00:00:40", "02:a1:0c:00:00:41");

    /// How long anything here waits for the kernel or the other machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// `ff02::fb` as `/proc/net/igmp6` spells a group.
    const THE_GROUP_IN_IGMP6: &str = "ff0200000000000000000000000000fb";

    /// **A link-local cable re-laid with the same hardware address between two
    /// readings is still joined**: the outer test, which makes the studio and
    /// reads what it saw.
    #[test]
    fn a_link_local_cable_re_laid_with_its_hardware_address_between_two_readings_is_still_joined() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_link_local_cable_re_laid_with_its_hardware_address::tests::the_studio_while_receptions_cable_is_re_laid";
        let ran = Command::new("unshare")
            .args(["--map-root-user", "--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "studio")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("util-linux's `unshare` makes the machines");
        let said = String::from_utf8_lossy(&ran.stdout).into_owned();
        let log = String::from_utf8_lossy(&ran.stderr).into_owned();
        assert!(
            ran.status.success(),
            "the studio failed ({}):\n{said}\n{log}",
            ran.status
        );
        for step in [
            "each found the other over link-local with its interface, and they paired",
            "held still with its cable deleted, reception is found nowhere and a proposal is refused before anything is sent",
            "laid again identically, the interface is not in the group its socket joined at that number, and nothing answers",
            "once reception followed the kernel, the first question found it on the same interface at the same address",
            "the pairing then proposed and paired on that interface",
            "the same bytes throughout",
        ] {
            assert!(
                said.contains(step),
                "the studio never said `{step}`:\n{said}\n{log}"
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
    /// named, and hand back what it printed or what it said when it refused.
    fn ip_said(pid: Option<u32>, args: &[&str]) -> Result<String, String> {
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
            .expect("iproute2's `ip` and util-linux's `nsenter` make the cable");
        if ran.status.success() {
            Ok(String::from_utf8_lossy(&ran.stdout).into_owned())
        } else {
            Err(format!(
                "ip {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&ran.stderr)
            ))
        }
    }

    /// Run iproute2 as [`ip_said`] does, and fail with what it said if it
    /// refuses.
    fn ip(pid: Option<u32>, args: &[&str]) {
        if let Err(why) = ip_said(pid, args) {
            panic!("{why}");
        }
    }

    /// Lay the cable between the studio and reception at `pid` — always at
    /// [`NUMBERS`] and with [`HARDWARE`], no IPv4 address on either end — and
    /// bring both ends up.
    ///
    /// A number an interface deleted a moment ago held may not be free the
    /// instant the deletion returns, so the laying is tried again until it is.
    fn lay_the_cable(pid: u32) {
        let (receptions, studios) = (NUMBERS.0.to_string(), NUMBERS.1.to_string());
        let pid_named = pid.to_string();
        let args = [
            "link",
            "add",
            STUDIOS_END,
            "index",
            &studios,
            "address",
            HARDWARE.1,
            "type",
            "veth",
            "peer",
            "name",
            RECEPTIONS_END,
            "index",
            &receptions,
            "address",
            HARDWARE.0,
            "netns",
            &pid_named,
        ];
        let until = Instant::now() + PATIENCE;
        while let Err(why) = ip_said(None, &args) {
            assert!(Instant::now() < until, "the cable could not be laid: {why}");
            std::thread::sleep(Duration::from_millis(50));
        }
        ip(None, &["link", "set", STUDIOS_END, "up"]);
        ip(Some(pid), &["link", "set", RECEPTIONS_END, "up"]);
    }

    /// Delete the cable — both ends go with it — and wait until neither end is
    /// reported.
    fn delete_the_cable(pid: u32) {
        ip(None, &["link", "del", STUDIOS_END]);
        let until = Instant::now() + PATIENCE;
        while ip_said(None, &["link", "show", STUDIOS_END]).is_ok()
            || ip_said(Some(pid), &["link", "show", RECEPTIONS_END]).is_ok()
        {
            assert!(Instant::now() < until, "the cable outlived its deletion");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// The studio's end of the cable, as the kernel reports it once the link is
    /// running and its link-local address has finished duplicate address
    /// detection: its number and that address.
    fn the_studios_end() -> (u32, Ipv6Addr) {
        let until = Instant::now() + PATIENCE;
        loop {
            let reported = reported_by_the_kernel().unwrap();
            if let Some(network) = link_local_networks(&reported)
                .into_iter()
                .find(|network| network.name() == STUDIOS_END)
                && let IpAddr::V6(address) = network.address()
            {
                return (network.index(), address);
            }
            assert!(
                Instant::now() < until,
                "{STUDIOS_END} never had a usable link-local address: {reported:?}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Reception's end of the cable, read from its network while reception
    /// itself may be held still: its number, its hardware address and its
    /// link-local address, once that address is no longer tentative.
    fn receptions_end(pid: u32) -> (u32, String, Ipv6Addr) {
        let until = Instant::now() + PATIENCE;
        loop {
            if let Some(end) = receptions_end_now(pid) {
                return end;
            }
            assert!(
                Instant::now() < until,
                "{RECEPTIONS_END} never had a usable link-local address in reception's network"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Reception's end of the cable as [`receptions_end`] reads it, or `None`
    /// while it is missing or its link-local address is still being checked.
    fn receptions_end_now(pid: u32) -> Option<(u32, String, Ipv6Addr)> {
        let link = ip_said(Some(pid), &["-o", "link", "show", RECEPTIONS_END]).ok()?;
        let number = link.split(':').next()?.trim().parse().ok()?;
        let words: Vec<&str> = link.split_whitespace().collect();
        let hardware = words
            .iter()
            .position(|word| *word == "link/ether")
            .and_then(|at| words.get(at + 1))?
            .to_string();
        let addresses = ip_said(
            Some(pid),
            &[
                "-6",
                "-o",
                "addr",
                "show",
                "dev",
                RECEPTIONS_END,
                "scope",
                "link",
            ],
        )
        .ok()?;
        let line = addresses.lines().find(|line| line.contains("inet6"))?;
        if line.contains("tentative") || line.contains("dadfailed") {
            return None;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        let address = words
            .iter()
            .position(|word| *word == "inet6")
            .and_then(|at| words.get(at + 1))?
            .split('/')
            .next()?
            .parse()
            .ok()?;
        Some((number, hardware, address))
    }

    /// The studio's end's hardware address, as the kernel reports it.
    fn the_studios_hardware() -> String {
        let link = ip_said(None, &["-o", "link", "show", STUDIOS_END]).unwrap();
        let words: Vec<&str> = link.split_whitespace().collect();
        words
            .iter()
            .position(|word| *word == "link/ether")
            .and_then(|at| words.get(at + 1))
            .unwrap()
            .to_string()
    }

    /// Whether `igmp6` — `/proc/net/igmp6` of some network — lists the discovery
    /// group on the interface numbered `index`.
    fn in_the_group(igmp6: &Path, index: u32) -> bool {
        std::fs::read_to_string(igmp6)
            .unwrap()
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .any(|words| {
                words.first() == Some(&index.to_string().as_str())
                    && words.get(2) == Some(&THE_GROUP_IN_IGMP6)
            })
    }

    /// The number of `cable` among `networks` over IPv6, or `-` where it is not
    /// among them.
    fn numbered(networks: &[Network], cable: &str) -> String {
        networks
            .iter()
            .find(|network| network.address().is_ipv6() && network.name() == cable)
            .map_or_else(|| "-".to_owned(), |network| network.index().to_string())
    }

    /// Wait until the studio's own wire is joined on its end of the cable exactly
    /// when `on` says so.
    fn until_the_studio_is_joined(wire: &Wire, on: bool) {
        let until = Instant::now() + PATIENCE;
        loop {
            let joined = numbered(&wire.joined(), STUDIOS_END);
            let listed = in_the_group(Path::new("/proc/self/net/igmp6"), NUMBERS.1);
            let followed = if on {
                joined == NUMBERS.1.to_string() && listed
            } else {
                joined == "-"
            };
            if followed {
                return;
            }
            assert!(
                Instant::now() < until,
                "the studio never followed its cable to joined = {on}: joined at `{joined}`, listed in the group: {listed}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Reception, run as a process of its own so it can be held still.
    struct Reception {
        /// Its process, whose network namespace is reception's end of the cable.
        process: Child,
        /// What it is told on.
        telling: ChildStdin,
        /// What it says on.
        hearing: BufReader<ChildStdout>,
    }

    impl Reception {
        /// Reception, started in a network namespace nested inside this one.
        fn started() -> Self {
            let exe = env::current_exe().unwrap();
            let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_link_local_cable_re_laid_with_its_hardware_address::tests::reception_while_its_cable_is_re_laid";
            let mut process = Command::new("unshare")
                .args(["--net", "--", "sh", "-c", script])
                .arg(exe)
                .env(INSIDE, "reception")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("util-linux's `unshare` makes reception");
            let ours = std::fs::read_link("/proc/self/ns/net").unwrap();
            let theirs = format!("/proc/{}/ns/net", process.id());
            let until = Instant::now() + Duration::from_secs(10);
            while std::fs::read_link(&theirs).ok().as_ref() == Some(&ours)
                || !Path::new(&theirs).exists()
            {
                assert!(Instant::now() < until, "reception never left this network");
                std::thread::sleep(Duration::from_millis(20));
            }
            let telling = process.stdin.take().unwrap();
            let hearing = BufReader::new(process.stdout.take().unwrap());
            Self {
                process,
                telling,
                hearing,
            }
        }

        /// Its process, which is also how its network is named.
        fn pid(&self) -> u32 {
            self.process.id()
        }

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
                assert!(read > 0, "reception ended before saying `{prefix}`");
                if let Some(rest) = line.trim_end().strip_prefix(&prefix) {
                    return rest.trim().to_owned();
                }
            }
        }

        /// Tell it `what`, and read the one line it answers with.
        fn told(&mut self, what: &str) -> String {
            self.tell(what);
            self.says(what)
        }

        /// Whether its person's door answered, with the studio among its
        /// pairings.
        fn door_answers_paired_with_the_studio(&mut self) -> bool {
            self.told("door") == the_studio().as_str()
        }

        /// Wait until it finds the studio, and where: the interface and the
        /// address.
        fn until_it_finds_the_studio(&mut self) -> (u32, IpAddr) {
            let until = Instant::now() + PATIENCE;
            loop {
                let said = self.told("look");
                if let Some((scope, address)) = said.split_once(' ') {
                    return (scope.parse().unwrap(), address.parse().unwrap());
                }
                assert!(Instant::now() < until, "reception never found the studio");
            }
        }

        /// Wait until its service is joined on its end of the cable at `number`
        /// **and** the kernel lists that interface in the discovery group —
        /// which, in reception's network, only the service joins.
        fn until_it_follows(&mut self, number: u32) {
            let igmp6 = PathBuf::from(format!("/proc/{}/net/igmp6", self.pid()));
            let until = Instant::now() + PATIENCE;
            loop {
                let joined = self.told("where");
                let listed = in_the_group(&igmp6, number);
                if joined == number.to_string() && listed {
                    return;
                }
                assert!(
                    Instant::now() < until,
                    "reception never followed its cable to {number}: joined at `{joined}`, and the interface {} in the discovery group",
                    if listed { "is" } else { "is not" }
                );
                std::thread::sleep(Duration::from_millis(50));
            }
        }

        /// Stop the whole process, wait until the kernel says every thread of it
        /// is stopped, do `what`, and let it go on.
        fn held_still(&mut self, what: impl FnOnce(u32)) {
            let pid = self.pid();
            signalled("-STOP", pid);
            let tasks = PathBuf::from(format!("/proc/{pid}/task"));
            let until = Instant::now() + PATIENCE;
            while !std::fs::read_dir(&tasks).unwrap().all(|task| {
                let stat =
                    std::fs::read_to_string(task.unwrap().path().join("stat")).unwrap_or_default();
                stat.rsplit(')')
                    .next()
                    .and_then(|rest| rest.split_whitespace().next())
                    .is_some_and(|state| state == "T" || state == "t")
            }) {
                assert!(Instant::now() < until, "reception was never held still");
                std::thread::sleep(Duration::from_millis(10));
            }
            what(pid);
            signalled("-CONT", pid);
        }

        /// Stop the service, and wait for reception to end.
        fn stopped(mut self) {
            assert_eq!(self.told("stop"), "");
            assert!(self.process.wait().unwrap().success(), "reception failed");
        }
    }

    impl Drop for Reception {
        /// A studio that fails takes reception with it, so a reception left held
        /// still never keeps the outer test waiting on its output.
        fn drop(&mut self) {
            drop(self.process.kill());
            drop(self.process.wait());
        }
    }

    /// Send `signal` to `pid`, through procps' `kill`.
    fn signalled(signal: &str, pid: u32) {
        let ran = Command::new("kill")
            .args([signal, &pid.to_string()])
            .status()
            .expect("procps' `kill` signals reception");
        assert!(ran.success(), "kill {signal} {pid} failed");
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

        /// The machines the door lists as paired, and the proposals waiting,
        /// each with its code where it has one.
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
                .expect("a cable re-laid stopped the service");
            driving.join().unwrap();
        });
    }

    /// Both questions, asked over IPv6 from the studio's link-local address on
    /// `cable`, and everything reception at `reception` answered within a
    /// second.
    fn what_reception_says(cable: u32, here: Ipv6Addr, reception: IpAddr) -> BTreeSet<Vec<u8>> {
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
            if who.ip() == reception {
                heard.insert(datagram.get(..read).unwrap().to_vec());
            }
        }
        heard
    }

    /// Wait until the studio finds reception on the wire, and hand back what it
    /// found.
    ///
    /// For the first laying only: a cable just come up is not joined in the
    /// instant it is running.
    fn until_the_studio_finds_reception(wire: &Wire) -> alo_nearby::Found {
        let until = Instant::now() + PATIENCE;
        loop {
            if let Some(found) = found_by_name(&reception(), wire.looks_at()) {
                return found;
            }
            assert!(Instant::now() < until, "the studio never found reception");
        }
    }

    /// Propose to reception through the studio's door, reception's person
    /// confirming the code they are shown, and the studio's person confirming
    /// after.
    fn paired_with_reception(person: &mut Talking, asked: &mut Reception) {
        asked.tell("confirm");
        let proposed = person.proposing_to(&reception());
        let waiting = proposed
            .proposed_pairing()
            .unwrap_or_else(|| panic!("the proposal was not measured: {proposed:?}"));
        let code = waiting.code().unwrap().to_owned();
        assert_eq!(asked.says("code"), code, "the codes differ");
        asked.says("confirmed");
        let confirmed = person.asking(&format!(
            r#"{{"confirm-pairing":{{"machine":"{}","code":"{code}"}}}}"#,
            reception().as_str()
        ));
        assert_eq!(
            confirmed.became_of_confirming(),
            Some(AfterConfirming::Paired),
            "{confirmed:?}"
        );
        let (paired, waiting) = person.pairings();
        assert_eq!(paired, vec![reception().as_str().to_owned()]);
        assert!(waiting.is_empty(), "{waiting:?}");
    }

    /// The studio: lays the cable, holds reception still while it deletes the
    /// cable and lays it again identically, and asks.
    #[test]
    #[ignore = "run inside its own network by a_link_local_cable_re_laid_with_its_hardware_address_between_two_readings_is_still_joined"]
    fn the_studio_while_receptions_cable_is_re_laid() {
        must_be("studio");
        let mut receiving = Reception::started();
        let pid = receiving.pid();
        lay_the_cable(pid);

        serving_as(
            &the_studio(),
            "same-hardware-address-studio",
            Hosted::Nothing,
            |wire, door, stop| {
                let mut person = Talking::to(door);
                receiving.says("serving");
                let expected: BTreeSet<Vec<u8>> = [
                    advertising::about(&Presence::of(reception(), THE_WIRE_PORT)).unwrap(),
                    advertising::about_a_workspace(&WorkspacePresence::of(reception(), WORKSPACE))
                        .unwrap(),
                ]
                .into();

                // 1. The cable laid: each finds the other, and they pair.
                let (studios_cable, studios_address) = the_studios_end();
                let (receptions_cable, receptions_hardware, receptions_address) =
                    receptions_end(pid);
                assert_eq!((receptions_cable, studios_cable), NUMBERS);
                assert_eq!(receptions_hardware, HARDWARE.0);
                assert_eq!(the_studios_hardware(), HARDWARE.1);
                until_the_studio_is_joined(wire, true);
                receiving.until_it_follows(NUMBERS.0);
                let found = until_the_studio_finds_reception(wire);
                assert_eq!(found.address.scope(), Some(studios_cable), "{found:?}");
                assert_eq!(found.address.ip(), IpAddr::V6(receptions_address));
                assert_eq!(
                    receiving.until_it_finds_the_studio(),
                    (receptions_cable, IpAddr::V6(studios_address))
                );
                paired_with_reception(&mut person, &mut receiving);
                assert!(receiving.door_answers_paired_with_the_studio());
                assert_eq!(
                    what_reception_says(studios_cable, studios_address, found.address.ip()),
                    expected,
                    "what reception said over link-local"
                );
                println!(
                    "each found the other over link-local with its interface, and they paired"
                );

                // 2 and 3. Held still: deleted, then laid again identically.
                receiving.held_still(|pid| {
                    delete_the_cable(pid);
                    until_the_studio_is_joined(wire, false);
                    assert!(
                        found_by_name(&reception(), wire.looks_at()).is_none(),
                        "the studio found reception with no cable at all"
                    );
                    let refused = person.proposing_to(&reception());
                    assert!(refused.proposed_pairing().is_none(), "{refused:?}");
                    assert_eq!(
                        refused.refusal().map(alo_protocol::Wording::text),
                        Some(NO_SUCH_MACHINE_ON_THE_NETWORK.says()),
                        "{refused:?}"
                    );
                    let (paired, waiting) = person.pairings();
                    assert_eq!(paired, vec![reception().as_str().to_owned()]);
                    assert!(waiting.is_empty(), "a refused proposal left one waiting");
                    println!(
                        "held still with its cable deleted, reception is found nowhere and a proposal is refused before anything is sent"
                    );

                    lay_the_cable(pid);
                    assert_eq!(the_studios_end(), (studios_cable, studios_address));
                    assert_eq!(the_studios_hardware(), HARDWARE.1);
                    assert_eq!(
                        receptions_end(pid),
                        (
                            receptions_cable,
                            receptions_hardware.clone(),
                            receptions_address
                        ),
                        "the cable was not laid again identically"
                    );
                    // What the kernel did with the membership reception's socket
                    // holds at that number: the interface laid there is not in the
                    // group, and nothing reception has run since could change it.
                    assert!(
                        !in_the_group(
                            &PathBuf::from(format!("/proc/{pid}/net/igmp6")),
                            receptions_cable
                        ),
                        "the kernel put an interface laid at a deleted one's number in its group"
                    );
                    until_the_studio_is_joined(wire, true);
                    assert!(
                        found_by_name(&reception(), wire.looks_at()).is_none(),
                        "reception answered while it was held still"
                    );
                    println!(
                        "laid again identically, the interface is not in the group its socket joined at that number, and nothing answers"
                    );
                });

                // 4. Let go: the first question once it has followed.
                receiving.until_it_follows(receptions_cable);
                let found = found_by_name(&reception(), wire.looks_at()).expect(
                    "the first question on a cable re-laid with its hardware address went unanswered",
                );
                assert_eq!(found.address.scope(), Some(studios_cable), "{found:?}");
                assert_eq!(found.address.ip(), IpAddr::V6(receptions_address));
                assert!(
                    matches!(found.where_it_answers(), SocketAddr::V6(at) if at.scope_id() == studios_cable),
                    "{found:?}"
                );
                println!(
                    "once reception followed the kernel, the first question found it on the same interface at the same address"
                );
                assert_eq!(
                    what_reception_says(studios_cable, studios_address, found.address.ip()),
                    expected
                );
                // Reception shows a code only once its service has measured the
                // studio at the address the proposal's connection came from, on
                // the re-laid interface, and found the machine that proposed.
                paired_with_reception(&mut person, &mut receiving);
                assert!(receiving.door_answers_paired_with_the_studio());
                assert_eq!(
                    receiving.until_it_finds_the_studio(),
                    (receptions_cable, IpAddr::V6(studios_address))
                );
                println!("the pairing then proposed and paired on that interface");
                println!("the same bytes throughout");

                assert!(stop.stop());
            },
        );
        receiving.stopped();
    }

    /// The next line the studio told this machine, waiting for it.
    fn next() -> String {
        let mut line = String::new();
        let read = std::io::stdin().read_line(&mut line).unwrap();
        assert!(read > 0, "the studio ended without stopping reception");
        line.trim_end().to_owned()
    }

    /// Reception, inside a network of its own nested in the studio's: it serves
    /// throughout, and does what the studio tells it.
    #[test]
    #[ignore = "run inside its own network by the_studio_while_receptions_cable_is_re_laid"]
    fn reception_while_its_cable_is_re_laid() {
        must_be("reception");
        let hosted = Hosted::At(NonZeroU16::new(WORKSPACE).unwrap());
        serving_as(
            &reception(),
            "same-hardware-address-reception",
            hosted,
            |wire, door, stop| {
                let mut person = Talking::to(door);
                // Asked once so the service holds this door before anything arrives.
                assert!(person.pairings().0.is_empty());
                println!("alo:serving");
                loop {
                    match next().as_str() {
                        "where" => {
                            println!("alo:where {}", numbered(&wire.joined(), RECEPTIONS_END))
                        }
                        "look" => match found_by_name(&the_studio(), wire.looks_at()) {
                            Some(found) => println!(
                                "alo:look {} {}",
                                found.address.scope().unwrap_or(0),
                                found.address.ip()
                            ),
                            None => println!("alo:look nothing"),
                        },
                        "door" => println!("alo:door {}", person.pairings().0.join(",")),
                        "confirm" => {
                            let until = Instant::now() + PATIENCE;
                            let code = loop {
                                let (_, waiting) = person.pairings();
                                if let Some((machine, Some(code))) = waiting.first() {
                                    assert_eq!(machine, the_studio().as_str());
                                    break code.clone();
                                }
                                assert!(Instant::now() < until, "no proposal was shown here");
                                std::thread::sleep(Duration::from_millis(100));
                            };
                            println!("alo:code {code}");
                            let confirmed = person.asking(&format!(
                                r#"{{"confirm-pairing":{{"machine":"{}","code":"{code}"}}}}"#,
                                the_studio().as_str()
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
                        said => panic!("reception was told something it does not do: {said}"),
                    }
                }
            },
        );
    }
}
