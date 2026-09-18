//! A port another program held over IPv6 when the service started is listened on
//! over IPv6 once it is let go of — on a real kernel, with the service running
//! throughout as `src/main.rs` runs it and holding no capabilities, as the unit
//! runs it, on a network with no IPv4 address at all.
//!
//! *Machines find each other with zero configuration.* The IPv6-only listener is
//! the only way a machine whose one network is link-local reaches this one's port.
//! Before `crate::listening_over_ipv6` it was bound once, at start, and a program
//! holding `[::]` at the port then left the machine found on that network and
//! unreachable there until the service restarted. This measures that the service
//! now tries it again when the kernel says the port was let go of
//! (`crate::told_of_a_port_let_go`), with **no network changing**.
//!
//! # The two machines
//!
//! As `crate::a_port_another_program_let_go_of` makes them. **The far end** is
//! this test binary run again in a network namespace of its own inside a user
//! namespace, so this needs no root and touches no kernel-global state and does
//! not take `alo_bounding::Waited::on_this_kernel()`. **Reception** is the binary
//! run a third time in a network namespace nested inside the far end's, started
//! through `setpriv` with an **empty capability bounding set**, as
//! `alo-agentd.service` runs the real service, and it asserts its own `CapEff`
//! and `CapBnd` are zero. One `veth` cable joins them carrying **link-local IPv6
//! only**: `cable0` at reception's end, numbered 40, and `far0` at the far end's,
//! numbered 60.
//!
//! # How the port is taken before the service starts
//!
//! Reception waits to be told to serve. Before it is, the cable is laid, both of
//! its link-local addresses finish duplicate address detection, and *the squatter*
//! — this binary once more, in reception's network with `nsenter` — listens on the
//! wire's port over IPv6 alone, held to nothing, as another program binding
//! `[::]` with `IPV6_V6ONLY` does. So the squatter certainly holds the port over
//! IPv6 before the service first tries to. The squatter accepts every connection
//! and closes it at once, saying so, and lets go when its standard input closes.
//!
//! # In this order
//!
//! 1. **The port held over IPv6 at start**: once reception has joined discovery on
//!    the cable, the question is answered over link-local, the port is **not**
//!    reached there, and the door answers. The knock that finds the port not
//!    reached is accepted and closed by the squatter — **a let-go that leaves the
//!    port held over IPv6**, which the kernel says and the service hears, tries,
//!    and is refused again.
//! 2. **The port let go of, with no network changing**: `ip -o monitor link
//!    address` watches reception's network from before the squatter lets go, as
//!    task 35's fixture does. Reception listens over IPv6, the port is reached over
//!    link-local, the monitor printed **nothing**, the question is answered with
//!    the same bytes as before, and the door answers.
//! 3. **The service log** says the IPv6 refusal exactly once — though a let-go
//!    left the port held — and the IPv6 bind exactly once, refuses and binds no
//!    IPv4 network, never says the kernel would not tell it, and the service stops
//!    when it is told to and not before.

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
    use std::fmt::Write as _;
    use std::io::{BufRead as _, BufReader, Read as _, Write as _};
    use std::net::{IpAddr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV6, TcpStream, UdpSocket};
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::time::{Duration, Instant, SystemTime};

    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_keeping::Keeping;
    use alo_nearby::{MachineId, THE_IPV6_ADDRESS, THE_PORT};
    use alo_protocol::ToAPerson;
    use alo_record::Record;
    use alo_turn::Machine;

    use crate::hearing::NOT_FOR_THIS_WIRE;
    use crate::hosting::Hosted;
    use crate::network::TheNetwork;
    use crate::networks::Network;
    use crate::rereading::WhatIsGranted;
    use crate::serving::Serving;
    use crate::side::Side;
    use crate::stopping::{Stop, Waking};
    use crate::surface::AtThePersonsDoor;
    use crate::terms::Terms;
    use crate::testing::{
        NothingIsBounded, NothingIsRemembered, Pretending, a_folder_with_an_invoice, a_message,
        granting, hour, in_english, nothing_has_been_chosen, reception,
    };
    use crate::wire::{THE_WIRE_PORT, Wire};

    /// Set on the binary run again inside a namespace, naming which machine it
    /// is.
    const INSIDE: &str = "ALO_AGENTD_A_PORT_HELD_OVER_IPV6_AT_START_INSIDE";

    /// Reception's end of the cable.
    const RECEPTIONS_END: &str = "cable0";

    /// The far end's end of the cable.
    const FAR_END: &str = "far0";

    /// The number reception's end is laid at.
    const RECEPTIONS_NUMBER: u32 = 40;

    /// The number the far end's end is laid at, which is the scope every
    /// link-local address the far end dials carries.
    const FAR_NUMBER: u32 = 60;

    /// How long anything here waits for the kernel or another machine.
    const PATIENCE: Duration = Duration::from_secs(60);

    /// A routing socket subscribed to `RTMGRP_LINK`, `RTMGRP_IPV4_IFADDR` and
    /// `RTMGRP_IPV6_IFADDR`, as `/proc/net/netlink` spells its groups — which is
    /// how `ip monitor link address` subscribes, and how the service does.
    const THE_ROUTING_GROUPS: &str = "00000111";

    /// `ff02::fb` as `/proc/net/igmp6` spells a group.
    const THE_GROUP_IN_IGMP6: &str = "ff0200000000000000000000000000fb";

    /// What the service log says once when the port is held over IPv6 at start.
    const REFUSED: &str = "the port presence advertises could not be bound over IPv6 (";

    /// What the service log says once when it binds over IPv6 after all.
    const BOUND: &str = "the port presence advertises is bound over IPv6 now";

    /// **A port another program held over IPv6 at start is listened on over IPv6
    /// once it is let go of**: the outer test, which makes the far end and reads
    /// what it saw and what reception's service log said.
    #[test]
    fn a_port_held_over_ipv6_at_start_is_listened_on_over_ipv6_once_let_go_of() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_port_held_over_ipv6_at_start::tests::the_far_end_on_a_link_local_cable";
        let ran = Command::new("unshare")
            .args(["--map-root-user", "--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "far end")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("util-linux's `unshare` makes the machines");

        let said = String::from_utf8_lossy(&ran.stdout).into_owned();
        let log = String::from_utf8_lossy(&ran.stderr).into_owned();
        assert!(
            ran.status.success(),
            "the far end failed ({}):\n{said}\n{log}",
            ran.status
        );
        for step in [
            "reception serves with no capabilities",
            "with its port held over IPv6 at start it is found over link-local and not reached there",
            "a let-go that left the port held over IPv6 was refused again",
            "with the port let go of and no network changing, it is reached over link-local",
            "what was said was the same bytes throughout",
        ] {
            assert!(
                said.contains(step),
                "the far end never said `{step}`:\n{said}\n{log}"
            );
        }

        let service: Vec<&str> = log
            .lines()
            .filter(|line| line.starts_with("alo-agentd: "))
            .collect();
        assert_eq!(
            service.iter().filter(|line| line.contains(REFUSED)).count(),
            1,
            "the service log did not say exactly once that the port was refused over IPv6:\n{log}"
        );
        assert!(
            service
                .iter()
                .filter(|line| line.contains(REFUSED))
                .all(|line| line.contains("as soon as the kernel says a program let go")),
            "the IPv6 refusal does not say it is tried again when the port is let go of:\n{log}"
        );
        assert_eq!(
            service.iter().filter(|line| line.contains(BOUND)).count(),
            1,
            "the service log did not say exactly once that the port is bound over IPv6:\n{log}"
        );
        let per_network: Vec<&str> = service
            .iter()
            .copied()
            .filter(|line| {
                ["could not be bound on ", " is bound on "]
                    .iter()
                    .any(|what| line.contains(what))
            })
            .collect();
        assert!(
            per_network.is_empty(),
            "an IPv4 network was refused or bound again:\n{per_network:?}"
        );
        assert!(
            !log.contains("will not say when a program lets go"),
            "the kernel would not tell a service with no capabilities:\n{log}"
        );
    }

    /// Refuse to run unless this is the binary re-run as `which`.
    fn must_be(which: &str) {
        assert_eq!(
            env::var(INSIDE).ok().as_deref(),
            Some(which),
            "this test runs inside the namespace the outer test makes; run the outer one"
        );
    }

    /// `ip`, or `ip` in the network namespace of `pid` when one is named.
    fn ip_command(pid: Option<u32>) -> Command {
        match pid {
            Some(pid) => {
                let mut command = Command::new("nsenter");
                command.args(["-t", &pid.to_string(), "-n", "ip"]);
                command
            }
            None => Command::new("ip"),
        }
    }

    /// Run iproute2 with `args`, in the network namespace of `pid` when one is
    /// named, and hand back what it printed — failing with what it said if it
    /// refuses.
    fn ip_said(pid: Option<u32>, args: &[&str]) -> String {
        let ran = ip_command(pid)
            .args(args)
            .output()
            .expect("iproute2's `ip` and util-linux's `nsenter` make the cable");
        assert!(
            ran.status.success(),
            "ip {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&ran.stderr)
        );
        String::from_utf8_lossy(&ran.stdout).into_owned()
    }

    /// The link-local address on `name`, in the network of `pid` when one is
    /// named — or `None` while it is missing or still being checked.
    fn link_local_on(pid: Option<u32>, name: &str) -> Option<Ipv6Addr> {
        let addresses = ip_said(
            pid,
            &["-6", "-o", "addr", "show", "dev", name, "scope", "link"],
        );
        let line = addresses.lines().find(|line| line.contains("inet6"))?;
        if line.contains("tentative") || line.contains("dadfailed") {
            return None;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        words
            .iter()
            .position(|word| *word == "inet6")
            .and_then(|at| words.get(at + 1))?
            .split('/')
            .next()?
            .parse()
            .ok()
    }

    /// Lay the cable between here and reception at `pid`, with no IPv4 address
    /// on either end, and hand back both link-local addresses — reception's
    /// first — once each has finished duplicate address detection.
    fn laid(pid: u32) -> (Ipv6Addr, Ipv6Addr) {
        ip_said(
            None,
            &[
                "link",
                "add",
                FAR_END,
                "index",
                &FAR_NUMBER.to_string(),
                "type",
                "veth",
                "peer",
                "name",
                RECEPTIONS_END,
                "index",
                &RECEPTIONS_NUMBER.to_string(),
                "netns",
                &pid.to_string(),
            ],
        );
        ip_said(None, &["link", "set", FAR_END, "up"]);
        ip_said(Some(pid), &["link", "set", RECEPTIONS_END, "up"]);
        let until = Instant::now() + PATIENCE;
        loop {
            if let (Some(receptions), Some(far)) = (
                link_local_on(Some(pid), RECEPTIONS_END),
                link_local_on(None, FAR_END),
            ) {
                assert!(
                    !ip_said(
                        Some(pid),
                        &["-4", "-o", "addr", "show", "dev", RECEPTIONS_END]
                    )
                    .contains("inet"),
                    "the cable carries IPv4"
                );
                return (receptions, far);
            }
            assert!(
                Instant::now() < until,
                "the cable never had a usable link-local address at both ends"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Whether `igmp6` — `/proc/net/igmp6` of some network — lists `ff02::fb` on
    /// the interface numbered `index`.
    fn in_the_ipv6_group(igmp6: &Path, index: u32) -> bool {
        std::fs::read_to_string(igmp6)
            .unwrap()
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .any(|words| {
                words.first() == Some(&index.to_string().as_str())
                    && words.get(2) == Some(&THE_GROUP_IN_IGMP6)
            })
    }

    /// The inodes of the sockets `pid` holds open.
    fn sockets_of(pid: u32) -> BTreeSet<String> {
        std::fs::read_dir(format!("/proc/{pid}/fd"))
            .map(|open| {
                open.filter_map(|fd| std::fs::read_link(fd.ok()?.path()).ok())
                    .filter_map(|target| {
                        target
                            .to_str()?
                            .strip_prefix("socket:[")?
                            .strip_suffix(']')
                            .map(str::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether `pid` holds a routing socket subscribed to the groups the service
    /// follows, as the kernel lists it in `/proc/<pid>/net/netlink`.
    fn holds_a_routing_socket(pid: u32) -> bool {
        let open = sockets_of(pid);
        std::fs::read_to_string(format!("/proc/{pid}/net/netlink"))
            .unwrap_or_default()
            .lines()
            .skip(1)
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .any(|row| {
                row.len() == 10
                    && row.get(1) == Some(&"0")
                    && row.get(3) == Some(&THE_ROUTING_GROUPS)
                    && row.get(9).is_some_and(|inode| open.contains(*inode))
            })
    }

    /// `ip monitor` watching reception's network for any link or address that
    /// appears, changes or goes.
    struct Monitor {
        /// Its process.
        process: Child,
    }

    impl Monitor {
        /// Started in the network of `pid`, and subscribed before this returns.
        fn on(pid: u32) -> Self {
            let process = Command::new("nsenter")
                .args([
                    "-t",
                    &pid.to_string(),
                    "-n",
                    "ip",
                    "-o",
                    "monitor",
                    "link",
                    "address",
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("iproute2's `ip monitor` watches reception's network");
            let monitor = Self { process };
            let until = Instant::now() + PATIENCE;
            // `nsenter` executes `ip` in place, so the process is the monitor.
            while !holds_a_routing_socket(monitor.process.id()) {
                assert!(Instant::now() < until, "the monitor never subscribed");
                std::thread::sleep(Duration::from_millis(10));
            }
            monitor
        }

        /// Stop it, and hand back everything it printed.
        fn stopped(mut self) -> String {
            drop(self.process.kill());
            let mut printed = String::new();
            if let Some(mut out) = self.process.stdout.take() {
                drop(out.read_to_string(&mut printed));
            }
            drop(self.process.wait());
            printed
        }
    }

    impl Drop for Monitor {
        fn drop(&mut self) {
            drop(self.process.kill());
            drop(self.process.wait());
        }
    }

    /// Reception, run as a process of its own in a network of its own.
    struct Reception {
        /// Its process, whose network namespace is reception's end of the cable.
        process: Child,
        /// What it is told on.
        telling: ChildStdin,
        /// What it says on.
        hearing: BufReader<ChildStdout>,
    }

    impl Reception {
        /// Reception, started in a network namespace nested inside this one, with
        /// an empty capability bounding set — and not serving until it is told to.
        fn started() -> Self {
            let exe = env::current_exe().unwrap();
            let script = "ip link set lo up && exec setpriv --bounding-set=-all --inh-caps=-all --ambient-caps=-all \"$0\" --exact --ignored --nocapture a_port_held_over_ipv6_at_start::tests::reception_on_a_link_local_cable";
            let mut process = Command::new("unshare")
                .args(["--net", "--", "sh", "-c", script])
                .arg(exe)
                .env(INSIDE, "reception")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("util-linux's `unshare` and `setpriv` make reception");
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

        /// Tell it one thing, and read the one line it answers with.
        fn told(&mut self, what: &str) -> String {
            self.telling
                .write_all(format!("{what}\n").as_bytes())
                .unwrap();
            let prefix = format!("alo:{what}");
            let mut line = String::new();
            loop {
                line.clear();
                let read = self.hearing.read_line(&mut line).unwrap();
                assert!(read > 0, "reception ended before answering `{what}`");
                if let Some(rest) = line.trim_end().strip_prefix(&prefix) {
                    return rest.trim().to_owned();
                }
            }
        }

        /// Whether its person's door answers — which it does only from the
        /// service's own round, so an answer also says every round before it is
        /// finished.
        fn door_answers(&mut self) -> bool {
            self.told("door") == "yes"
        }

        /// Wait until the service has joined discovery on the cable over IPv6, listens on no
        /// IPv4 cable and listens over IPv6 as `over_ipv6` says, **and** the
        /// kernel lists the cable in the discovery group — and then until a round
        /// of the service after that has finished.
        fn until_it_holds(&mut self, over_ipv6: bool) {
            let igmp6 = PathBuf::from(format!("/proc/{}/net/igmp6", self.pid()));
            let wanted = format!(
                "{RECEPTIONS_END}|-|{}",
                if over_ipv6 { "yes" } else { "no" }
            );
            let until = Instant::now() + PATIENCE;
            loop {
                let held = self.told("where");
                let joined = in_the_ipv6_group(&igmp6, RECEPTIONS_NUMBER);
                if held == wanted && joined {
                    break;
                }
                assert!(
                    Instant::now() < until,
                    "reception never held `{wanted}`: it holds `{held}`, and the cable in the discovery group is {joined}"
                );
                std::thread::sleep(Duration::from_millis(50));
            }
            assert!(self.door_answers(), "reception's door stopped answering");
            assert_eq!(
                self.told("where"),
                wanted,
                "what reception holds moved after the round that followed its cable"
            );
        }

        /// Stop the service, and wait for reception to end.
        fn stopped(mut self) {
            assert_eq!(self.told("stop"), "");
            assert!(self.process.wait().unwrap().success(), "reception failed");
        }
    }

    impl Drop for Reception {
        /// A far end that fails takes reception with it.
        fn drop(&mut self) {
            drop(self.process.kill());
            drop(self.process.wait());
        }
    }

    /// A program in reception's network holding the port presence advertises over
    /// IPv6, for as long as it lives.
    struct Squatter {
        /// Its process.
        process: Child,
        /// What it says on, held open until it ends.
        hearing: BufReader<ChildStdout>,
    }

    impl Squatter {
        /// The squatter, in the network of `pid` — started, and holding the port
        /// over IPv6, before this returns.
        fn over_ipv6(pid: u32) -> Self {
            let exe = env::current_exe().unwrap();
            let mut process = Command::new("nsenter")
                .args(["-t", &pid.to_string(), "-n", "--"])
                .arg(exe)
                .args([
                    "--exact",
                    "--ignored",
                    "--nocapture",
                    "a_port_held_over_ipv6_at_start::tests::a_squatter_on_the_port_over_ipv6",
                ])
                .env(INSIDE, "squatter")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("util-linux's `nsenter` puts the squatter in reception's network");
            let hearing = BufReader::new(process.stdout.take().unwrap());
            let mut squatter = Self { process, hearing };
            squatter.until_it_says("alo:holding");
            squatter
        }

        /// Wait until the squatter prints `what`.
        fn until_it_says(&mut self, what: &str) {
            let mut line = String::new();
            loop {
                line.clear();
                let read = self.hearing.read_line(&mut line).unwrap();
                assert!(read > 0, "the squatter ended before saying `{what}`");
                if line.trim_end() == what {
                    return;
                }
            }
        }

        /// Let go of the port, and wait for the squatter to end.
        fn gone(mut self) {
            drop(self.process.stdin.take());
            let mut rest = Vec::new();
            drop(self.hearing.read_to_end(&mut rest));
            assert!(
                self.process.wait().unwrap().success(),
                "the squatter failed"
            );
        }
    }

    impl Drop for Squatter {
        /// A far end that fails takes the squatter with it.
        fn drop(&mut self) {
            drop(self.process.kill());
            drop(self.process.wait());
        }
    }

    /// Ask the discovery group on the cable who is here, from the far end's
    /// link-local address `far`, and hand back the bytes reception at
    /// `receptions` answered with in hexadecimal — or `nothing`.
    fn asked(far: Ipv6Addr, receptions: Ipv6Addr) -> String {
        let socket = UdpSocket::bind(SocketAddrV6::new(far, 0, 0, FAR_NUMBER)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        let question = alo_nearby::advertising::a_question().unwrap();
        let group = SocketAddrV6::new(THE_IPV6_ADDRESS, THE_PORT, 0, FAR_NUMBER);
        if socket.send_to(&question, group).is_err() {
            return "nothing".to_owned();
        }
        let until = Instant::now() + Duration::from_secs(3);
        let mut heard = [0_u8; 1_500];
        while Instant::now() < until {
            if let Ok((how_many, from)) = socket.recv_from(&mut heard)
                && from.ip() == IpAddr::V6(receptions)
            {
                return heard.get(..how_many).unwrap().iter().fold(
                    String::new(),
                    |mut hex, byte| {
                        let _ = write!(hex, "{byte:02x}");
                        hex
                    },
                );
            }
        }
        "nothing".to_owned()
    }

    /// Whether the service on reception's port at its link-local address
    /// `receptions` answered a message — a handshake completed **and** the
    /// wire's own refusal read back.
    fn reached(receptions: Ipv6Addr) -> bool {
        let at = SocketAddr::V6(SocketAddrV6::new(receptions, THE_WIRE_PORT, 0, FAR_NUMBER));
        let Ok(mut connection) = TcpStream::connect_timeout(&at, Duration::from_secs(2)) else {
            return false;
        };
        if connection
            .set_read_timeout(Some(Duration::from_secs(3)))
            .is_err()
        {
            return false;
        }
        let request = alo_nearby::http::a_request("/somewhere-else", "reception", "");
        if connection.write_all(request.as_bytes()).is_err() {
            return false;
        }
        let mut back = Vec::new();
        drop(connection.read_to_end(&mut back));
        String::from_utf8_lossy(&back).contains(NOT_FOR_THIS_WIRE)
    }

    /// The first question on the cable, failing where it went unanswered.
    fn the_answer(far: Ipv6Addr, receptions: Ipv6Addr) -> String {
        let heard = asked(far, receptions);
        assert_ne!(
            heard, "nothing",
            "the question over link-local went unanswered"
        );
        heard
    }

    /// The far end: lays the cable and takes the port over IPv6 before reception
    /// serves, then lets the port go with no network changing.
    #[test]
    #[ignore = "run inside its own network by a_port_held_over_ipv6_at_start_is_listened_on_over_ipv6_once_let_go_of"]
    fn the_far_end_on_a_link_local_cable() {
        must_be("far end");
        let mut reception = Reception::started();
        assert_eq!(
            reception.told("capabilities"),
            "none",
            "reception holds capabilities the service never has"
        );
        println!("reception serves with no capabilities");

        // 1. The port held over IPv6 before the service first tries it.
        let pid = reception.pid();
        let (receptions, far) = laid(pid);
        let mut squatter = Squatter::over_ipv6(pid);
        assert_eq!(reception.told("serve"), "");
        assert!(reception.door_answers(), "reception never served");
        reception.until_it_holds(false);
        let said = the_answer(far, receptions);
        assert!(
            !reached(receptions),
            "the port was reached over link-local while another program held it over IPv6"
        );
        assert!(reception.door_answers());
        println!(
            "with its port held over IPv6 at start it is found over link-local and not reached there"
        );

        // The knock that was not reached was the squatter's to accept and close:
        // a socket at the port destroyed while the port stays held.
        squatter.until_it_says("alo:closed");
        assert!(reception.door_answers());
        reception.until_it_holds(false);
        println!("a let-go that left the port held over IPv6 was refused again");

        // 2. Let go of, with nothing in reception's network changing.
        let monitor = Monitor::on(pid);
        squatter.gone();
        reception.until_it_holds(true);
        assert!(
            reached(receptions),
            "the port was not reached over link-local once it was let go of"
        );
        let printed = monitor.stopped();
        assert_eq!(
            printed, "",
            "a network changed in reception's network while the port was let go of"
        );
        assert_eq!(
            the_answer(far, receptions),
            said,
            "an answer after the port was let go of was not the same bytes"
        );
        assert!(reception.door_answers());
        println!("with the port let go of and no network changing, it is reached over link-local");
        println!("what was said was the same bytes throughout");

        reception.stopped();
    }

    /// The squatter: listens on the port presence advertises over IPv6 alone,
    /// held to nothing, accepting and closing every connection, until its
    /// standard input closes.
    #[test]
    #[ignore = "run in reception's network by the_far_end_on_a_link_local_cable"]
    fn a_squatter_on_the_port_over_ipv6() {
        must_be("squatter");
        let listener = crate::unix::an_ipv6_only_listener_on(THE_WIRE_PORT).unwrap();
        println!("alo:holding");
        std::thread::scope(|scope| {
            let listener = &listener;
            scope.spawn(move || {
                // Ends when the listener is shut, which makes accept refuse.
                while let Ok((connection, _)) = listener.accept() {
                    drop(connection);
                    println!("alo:closed");
                }
            });
            let mut rest = Vec::new();
            drop(std::io::stdin().read_to_end(&mut rest));
            socket2::SockRef::from(listener)
                .shutdown(Shutdown::Read)
                .unwrap();
        });
        drop(listener);
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

        /// Ask for the pairings, and say whether the door answered with them.
        fn answers(&mut self) -> bool {
            self.writing
                .write_all(format!("{}\n", a_message(r#"{"pairings":{}}"#)).as_bytes())
                .unwrap();
            let mut back = String::new();
            self.reading.read_line(&mut back).unwrap();
            ToAPerson::read(back.trim_end())
                .ok()
                .and_then(|told| told.paired().map(|_| ()))
                .is_some()
        }
    }

    /// The cables among `networks` over `ipv4` or over IPv6, sorted and joined
    /// with commas, and `-` for none.
    fn cables_among(networks: &[Network], ipv4: bool) -> String {
        let names: BTreeSet<&str> = networks
            .iter()
            .filter(|network| network.address().is_ipv4() == ipv4)
            .map(Network::name)
            .filter(|name| name.starts_with("cable"))
            .collect();
        if names.is_empty() {
            "-".to_owned()
        } else {
            names.into_iter().collect::<Vec<_>>().join(",")
        }
    }

    /// Whether this process holds no capability at all, effective or bounding,
    /// as the kernel reports it.
    fn holds_no_capabilities() -> bool {
        let status = std::fs::read_to_string("/proc/self/status").unwrap();
        ["CapEff:", "CapBnd:"].iter().all(|which| {
            status
                .lines()
                .find_map(|line| line.strip_prefix(which))
                .is_some_and(|mask| mask.trim().chars().all(|digit| digit == '0'))
        })
    }

    /// Run this machine as the service runs it — `Wire::bound` as `here`, with a
    /// person's door — while `person` drives it from a thread that may borrow
    /// the wire, and stop when `person` stops it.
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
                .expect("a port held over IPv6 and let go of stopped the service");
            driving.join().unwrap();
        });
    }

    /// The next line the far end tells reception, ending the test if it ended.
    fn the_next_line() -> String {
        let mut line = String::new();
        let read = std::io::stdin().read_line(&mut line).unwrap();
        assert!(read > 0, "the far end ended without stopping reception");
        line.trim_end().to_owned()
    }

    /// Reception, waiting to be told to serve, then serving on the cable the far
    /// end lays and answering the far end on its standard output.
    #[test]
    #[ignore = "run inside its own network by the_far_end_on_a_link_local_cable"]
    fn reception_on_a_link_local_cable() {
        must_be("reception");
        let capabilities = if holds_no_capabilities() {
            "none"
        } else {
            "some"
        };
        loop {
            match the_next_line().as_str() {
                "capabilities" => println!("alo:capabilities {capabilities}"),
                "serve" => {
                    println!("alo:serve");
                    break;
                }
                said => panic!("reception was told something before serving: {said}"),
            }
        }
        serving_as(
            &reception(),
            "a-port-held-over-ipv6-at-start",
            |wire, door, stop| {
                let mut person = Talking::to(door);
                loop {
                    match the_next_line().as_str() {
                        "where" => println!(
                            "alo:where {}|{}|{}",
                            cables_among(&wire.joined(), false),
                            cables_among(&wire.listened_on(), true),
                            if wire.listening_over_ipv6() {
                                "yes"
                            } else {
                                "no"
                            },
                        ),
                        "door" => {
                            println!("alo:door {}", if person.answers() { "yes" } else { "no" })
                        }
                        "stop" => {
                            println!("alo:stop");
                            break;
                        }
                        said => panic!("reception was told something it does not do: {said}"),
                    }
                }
                assert!(stop.stop());
            },
        );
    }
}
