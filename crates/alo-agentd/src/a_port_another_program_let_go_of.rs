//! A port another program let go of on one network is listened on there again —
//! on a real kernel, with the service running throughout as `src/main.rs` runs it
//! and holding no capabilities, as the unit runs it.
//!
//! *Machines find each other with zero configuration.* Task 34 made one network
//! fail: another program held the port presence advertises on one interface, and
//! the service's listener there was refused. The service log said a machine on
//! that network could not reach this one *until it can be*, and nothing tried
//! again when it could be, because a program closing a socket is not a network
//! change. This measures that the service now hears the kernel say so
//! (`crate::told_of_a_port_let_go`) and listens there again, with **no network
//! changing**.
//!
//! # The two machines
//!
//! **The far end** is this test binary run again in a network namespace of its
//! own, inside a user namespace — so this needs no root and touches no
//! kernel-global state (`docs/autonomy/SHARED_MAIN.md`), and does not take
//! `alo_bounding::Waited::on_this_kernel()`. **Reception** is the binary run a
//! third time in a network namespace nested inside the far end's, serving as
//! `src/main.rs` does and answering on its standard output what it is asked on
//! its standard input. It is started through `setpriv` with an **empty
//! capability bounding set**, which is what `alo-agentd.service` gives the real
//! service (`CapabilityBoundingSet=`), and asserts its own `CapEff` and `CapBnd`
//! are zero before it serves: a reading that only a privileged process gets would
//! pass in a user namespace and fail on the machine. Two `veth` cables join them,
//! both carrying IPv4: `cable0` and `cable1` at reception's end, numbered 40 and
//! 41, and `far0` and `far1` at the far end's, numbered 60 and 61.
//!
//! # How the port is taken, and let go of
//!
//! Both cables are laid while reception is **held still** (`SIGSTOP`, and wait
//! until the kernel reports every thread stopped), and before it is let go *the
//! squatter* — this binary once more, in reception's network with `nsenter` —
//! listens on the wire's port held to `cable1` alone, sharing it with nothing. So
//! the squatter certainly holds the port before the service first tries to. The
//! squatter lets go when its standard input closes, and ends.
//!
//! # How *with no network changing* is made certain
//!
//! Before the squatter lets go, the far end starts `ip -o monitor link address`
//! in reception's network — the same three routing groups the service's routing
//! socket joins — and waits until the kernel lists that monitor's routing socket,
//! subscribed, among its descriptors (`/proc/<pid>/net/netlink`). Every
//! link-local address on both cables has finished duplicate address detection
//! before then. Once the port is reached on `cable1`, the monitor is stopped, and
//! **it must have printed nothing**: no link and no address in reception's network
//! appeared, changed or went between the squatter letting go and the service
//! listening there.
//!
//! # In this order
//!
//! 1. **The port taken on one cable**: once reception has followed its cables, the
//!    first question on each is answered with the same bytes, the port is reached
//!    on `cable0` and not on `cable1`, and the door answers.
//! 2. **The port let go of, with no network changing**: reception listens on
//!    `cable1`, the port is reached there and still on `cable0`, both questions
//!    are answered with the same bytes as before, and the door answers.
//! 3. **The service log** names `cable1` in exactly one refusal and exactly one
//!    line saying it is bound there now, names no other network in either, and the
//!    service stops when it is told to and not before.

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
    use std::net::{Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
    use std::num::NonZeroU32;
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::time::{Duration, Instant, SystemTime};

    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_keeping::Keeping;
    use alo_nearby::{MachineId, THE_ADDRESS, THE_PORT};
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
    const INSIDE: &str = "ALO_AGENTD_A_PORT_ANOTHER_PROGRAM_LET_GO_OF_INSIDE";

    /// Set on the squatter, naming the interface number it holds the port on.
    const SQUATTING_ON: &str = "ALO_AGENTD_A_PORT_ANOTHER_PROGRAM_LET_GO_OF_SQUATTING_ON";

    /// The cable the port is always reached on.
    const THE_FREE_ONE: Cable = Cable(0);

    /// The cable another program holds the port on, and lets go of it.
    const THE_TAKEN_ONE: Cable = Cable(1);

    /// How long anything here waits for the kernel or another machine.
    const PATIENCE: Duration = Duration::from_secs(60);

    /// A routing socket subscribed to `RTMGRP_LINK`, `RTMGRP_IPV4_IFADDR` and
    /// `RTMGRP_IPV6_IFADDR`, as `/proc/net/netlink` spells its groups — which is
    /// how `ip monitor link address` subscribes, and how the service does.
    const THE_ROUTING_GROUPS: &str = "00000111";

    /// One cable between the far end and reception, laid identically every time.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    struct Cable(u8);

    impl Cable {
        /// Reception's end.
        fn receptions_end(self) -> String {
            format!("cable{}", self.0)
        }

        /// The far end's end.
        fn far_end(self) -> String {
            format!("far{}", self.0)
        }

        /// The numbers each end is laid at, reception's first.
        fn numbers(self) -> (u32, u32) {
            (40 + u32::from(self.0), 60 + u32::from(self.0))
        }

        /// The IPv4 addresses on each end, reception's first.
        const fn ipv4(self) -> (Ipv4Addr, Ipv4Addr) {
            (
                Ipv4Addr::new(10, 75, self.0, 1),
                Ipv4Addr::new(10, 75, self.0, 2),
            )
        }
    }

    /// **A port another program let go of on one network is listened on there
    /// again**: the outer test, which makes the far end and reads what it saw
    /// and what reception's service log said.
    #[test]
    fn a_port_another_program_let_go_of_on_one_network_is_listened_on_there_again() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_port_another_program_let_go_of::tests::the_far_end_with_two_cables_to_reception";
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
            "with its port taken on cable1 it is found on both cables, reached on cable0 and not on cable1",
            "with the port let go of and no network changing, it is reached on cable1 and still on cable0",
            "what was said was the same bytes on both cables, throughout",
        ] {
            assert!(
                said.contains(step),
                "the far end never said `{step}`:\n{said}\n{log}"
            );
        }

        let taken = THE_TAKEN_ONE.receptions_end();
        let (receptions, _) = THE_TAKEN_ONE.ipv4();
        let refused =
            format!("the port presence advertises could not be bound on {taken} ({receptions}): ");
        let bound = format!("the port presence advertises is bound on {taken} ({receptions}) now");
        let service: Vec<&str> = log
            .lines()
            .filter(|line| line.starts_with("alo-agentd: "))
            .collect();
        assert_eq!(
            service
                .iter()
                .filter(|line| line.contains(&refused))
                .count(),
            1,
            "the service log did not name the network its port was refused on exactly once:\n{log}"
        );
        assert_eq!(
            service.iter().filter(|line| line.contains(&bound)).count(),
            1,
            "the service log did not say exactly once that the port is bound where it was let go of:\n{log}"
        );
        let per_network: Vec<&str> = service
            .iter()
            .copied()
            .filter(|line| {
                ["could not be bound on ", " is bound on "]
                    .iter()
                    .any(|what| line.contains(what))
                    && !line.contains(&format!(" on {taken} ("))
            })
            .collect();
        assert!(
            per_network.is_empty(),
            "a network other than {taken} was refused or bound again:\n{per_network:?}"
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
    /// named, and hand back what it printed.
    fn ip_said(pid: Option<u32>, args: &[&str]) -> String {
        let ran = ip_command(pid)
            .args(args)
            .output()
            .expect("iproute2's `ip` and util-linux's `nsenter` make the cables");
        assert!(
            ran.status.success(),
            "ip {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&ran.stderr)
        );
        String::from_utf8_lossy(&ran.stdout).into_owned()
    }

    /// Run `lines` through one `ip -batch`, in the network namespace of `pid`
    /// when one is named, and fail with what it said if it refuses any of them.
    fn batch(pid: Option<u32>, lines: &str) {
        let mut batching = ip_command(pid)
            .args(["-batch", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("iproute2's `ip -batch` lays the cables");
        batching
            .stdin
            .take()
            .unwrap()
            .write_all(lines.as_bytes())
            .unwrap();
        let done = batching.wait_with_output().unwrap();
        assert!(
            done.status.success(),
            "one batch was refused: {}\n{lines}",
            String::from_utf8_lossy(&done.stderr)
        );
    }

    /// Lay `cables` between here and reception at `pid`, addressed and up, and
    /// wait until no address at either end is still being checked.
    fn laid(cables: &[Cable], pid: u32) {
        let (mut here, mut there) = (String::new(), String::new());
        for cable in cables {
            let (receptions_number, far_number) = cable.numbers();
            let (receptions, far) = cable.ipv4();
            let (receptions_end, far_end) = (cable.receptions_end(), cable.far_end());
            let _ = writeln!(
                here,
                "link add {far_end} index {far_number} type veth peer name {receptions_end} index {receptions_number} netns {pid}"
            );
            let _ = writeln!(here, "addr add {far}/24 dev {far_end}");
            let _ = writeln!(here, "link set {far_end} up");
            let _ = writeln!(there, "addr add {receptions}/24 dev {receptions_end}");
            let _ = writeln!(there, "link set {receptions_end} up");
        }
        batch(None, &here);
        batch(Some(pid), &there);
        let until = Instant::now() + PATIENCE;
        loop {
            let checking = [None, Some(pid)].into_iter().any(|at| {
                cables.iter().any(|cable| {
                    let name = if at.is_some() {
                        cable.receptions_end()
                    } else {
                        cable.far_end()
                    };
                    let addresses = ip_said(at, &["-6", "-o", "addr", "show", "dev", &name]);
                    !addresses.contains("inet6") || addresses.contains("tentative")
                })
            });
            if !checking {
                return;
            }
            assert!(
                Instant::now() < until,
                "a cable's link-local address never finished being checked"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Whether the interface numbered `index` is in `224.0.0.251`, as `igmp` —
    /// `/proc/net/igmp` of some network — lists it.
    fn in_the_ipv4_group(igmp: &Path, index: u32) -> bool {
        let group = format!("{:08X}", u32::from_ne_bytes(THE_ADDRESS.octets()));
        let listed = std::fs::read_to_string(igmp).unwrap();
        let mut on = None;
        for line in listed.lines() {
            if line.starts_with(|first: char| first.is_ascii_digit()) {
                on = line
                    .split_whitespace()
                    .next()
                    .and_then(|at| at.parse::<u32>().ok());
            } else if on == Some(index) && line.split_whitespace().next() == Some(group.as_str()) {
                return true;
            }
        }
        false
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

    /// Reception, run as a process of its own so it can be held still.
    struct Reception {
        /// Its process, whose network namespace is reception's end of the cables.
        process: Child,
        /// What it is told on.
        telling: ChildStdin,
        /// What it says on.
        hearing: BufReader<ChildStdout>,
    }

    impl Reception {
        /// Reception, started in a network namespace nested inside this one, with
        /// an empty capability bounding set.
        fn started() -> Self {
            let exe = env::current_exe().unwrap();
            let script = "ip link set lo up && exec setpriv --bounding-set=-all --inh-caps=-all --ambient-caps=-all \"$0\" --exact --ignored --nocapture a_port_another_program_let_go_of::tests::reception_on_two_cables";
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

        /// What the service holds, as `answered|listened`, each the cables it is
        /// on there.
        fn where_it_is(&mut self) -> String {
            self.told("where")
        }

        /// Wait until the service answers on both cables and listens on
        /// `listened`, **and** the kernel lists both in the discovery group — and
        /// then until a round of the service after that has finished.
        fn until_it_holds(&mut self, listened: &str) {
            let igmp = PathBuf::from(format!("/proc/{}/net/igmp", self.pid()));
            let wanted = format!("cable0,cable1|{listened}");
            let until = Instant::now() + PATIENCE;
            loop {
                let held = self.where_it_is();
                let joined = [THE_FREE_ONE, THE_TAKEN_ONE]
                    .iter()
                    .all(|cable| in_the_ipv4_group(&igmp, cable.numbers().0));
                if held == wanted && joined {
                    break;
                }
                assert!(
                    Instant::now() < until,
                    "reception never held `{wanted}`: it holds `{held}`, and both cables in the discovery group is {joined}"
                );
                std::thread::sleep(Duration::from_millis(50));
            }
            assert!(self.door_answers(), "reception's door stopped answering");
            assert_eq!(
                self.where_it_is(),
                wanted,
                "what reception holds moved after the round that followed its cables"
            );
        }

        /// Stop the whole process, wait until the kernel says every thread of it
        /// is stopped, do `what`, and let it go on.
        fn held_still<T>(&mut self, what: impl FnOnce(u32) -> T) -> T {
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
            let done = what(pid);
            signalled("-CONT", pid);
            done
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

    /// Send `signal` to `pid`, through procps' `kill`.
    fn signalled(signal: &str, pid: u32) {
        let ran = Command::new("kill")
            .args([signal, &pid.to_string()])
            .status()
            .expect("procps' `kill` signals reception");
        assert!(ran.success(), "kill {signal} {pid} failed");
    }

    /// A program in reception's network holding the port presence advertises on
    /// one interface, for as long as it lives.
    struct Squatter {
        /// Its process.
        process: Child,
        /// What it says on, held open until it ends.
        hearing: BufReader<ChildStdout>,
    }

    impl Squatter {
        /// The squatter, in the network of `pid`, holding the port on `cable`'s
        /// interface there — started, and holding it, before this returns.
        fn on(cable: Cable, pid: u32) -> Self {
            let exe = env::current_exe().unwrap();
            let mut process = Command::new("nsenter")
                .args(["-t", &pid.to_string(), "-n", "--"])
                .arg(exe)
                .args([
                    "--exact",
                    "--ignored",
                    "--nocapture",
                    "a_port_another_program_let_go_of::tests::a_squatter_on_the_port",
                ])
                .env(INSIDE, "squatter")
                .env(SQUATTING_ON, cable.numbers().0.to_string())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("util-linux's `nsenter` puts the squatter in reception's network");
            let hearing = BufReader::new(process.stdout.take().unwrap());
            let mut squatter = Self { process, hearing };
            let mut line = String::new();
            loop {
                line.clear();
                let read = squatter.hearing.read_line(&mut line).unwrap();
                assert!(read > 0, "the squatter ended before it held the port");
                if line.trim_end() == "alo:holding" {
                    return squatter;
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

    /// Ask the discovery group on `cable` who is here, from the far end's own
    /// address on it, and hand back the bytes reception answered with in
    /// hexadecimal — or `nothing`.
    fn asked(cable: Cable) -> String {
        let (receptions, far) = cable.ipv4();
        let socket = UdpSocket::bind((far, 0)).unwrap();
        socket2::SockRef::from(&socket)
            .set_multicast_if_v4(&far)
            .unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        let question = alo_nearby::advertising::a_question().unwrap();
        if socket
            .send_to(&question, SocketAddr::from((THE_ADDRESS, THE_PORT)))
            .is_err()
        {
            return "nothing".to_owned();
        }
        let until = Instant::now() + Duration::from_secs(3);
        let mut heard = [0_u8; 1_500];
        while Instant::now() < until {
            if let Ok((how_many, from)) = socket.recv_from(&mut heard)
                && from.ip() == receptions
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

    /// Whether the service on reception's port on `cable` answered a message —
    /// a handshake completed **and** the wire's own refusal read back.
    fn reached(cable: Cable) -> bool {
        let (receptions, _) = cable.ipv4();
        let at = SocketAddr::from((receptions, THE_WIRE_PORT));
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

    /// The first question on each cable, failing where one went unanswered.
    fn both_answers() -> [String; 2] {
        [THE_FREE_ONE, THE_TAKEN_ONE].map(|cable| {
            let heard = asked(cable);
            assert_ne!(
                heard,
                "nothing",
                "the question on {} went unanswered",
                cable.receptions_end()
            );
            heard
        })
    }

    /// The far end: lays two cables while reception is held still with the port
    /// taken on one, and lets the port go with no network changing.
    #[test]
    #[ignore = "run inside its own network by a_port_another_program_let_go_of_on_one_network_is_listened_on_there_again"]
    fn the_far_end_with_two_cables_to_reception() {
        must_be("far end");
        let mut reception = Reception::started();
        assert_eq!(
            reception.told("capabilities"),
            "none",
            "reception holds capabilities the service never has"
        );
        println!("reception serves with no capabilities");
        assert!(reception.door_answers(), "reception never served");

        // 1. The port taken on one cable, before the service first tries it.
        let squatter = reception.held_still(|pid| {
            laid(&[THE_FREE_ONE, THE_TAKEN_ONE], pid);
            Squatter::on(THE_TAKEN_ONE, pid)
        });
        reception.until_it_holds("cable0");
        let said = both_answers();
        assert_eq!(
            said[0], said[1],
            "the two cables were answered with different bytes"
        );
        assert!(reached(THE_FREE_ONE), "the port was not reached on cable0");
        assert!(
            !reached(THE_TAKEN_ONE),
            "the port was reached on the cable another program holds it on"
        );
        assert!(reception.door_answers());
        println!(
            "with its port taken on cable1 it is found on both cables, reached on cable0 and not on cable1"
        );

        // 2. Let go of, with nothing in reception's network changing.
        let pid = reception.pid();
        let monitor = Monitor::on(pid);
        squatter.gone();
        reception.until_it_holds("cable0,cable1");
        assert!(
            reached(THE_TAKEN_ONE),
            "the port was not reached on cable1 once it was let go of"
        );
        assert!(
            reached(THE_FREE_ONE),
            "the port was no longer reached on cable0"
        );
        let printed = monitor.stopped();
        assert_eq!(
            printed, "",
            "a network changed in reception's network while the port was let go of"
        );
        assert_eq!(
            both_answers(),
            said,
            "an answer after the port was let go of was not the same bytes"
        );
        assert!(reception.door_answers());
        println!(
            "with the port let go of and no network changing, it is reached on cable1 and still on cable0"
        );
        println!("what was said was the same bytes on both cables, throughout");

        reception.stopped();
    }

    /// The squatter: listens on the port presence advertises, held to the
    /// interface [`SQUATTING_ON`] names and sharing it with nothing, until its
    /// standard input closes.
    #[test]
    #[ignore = "run in reception's network by the_far_end_with_two_cables_to_reception"]
    fn a_squatter_on_the_port() {
        must_be("squatter");
        let interface: NonZeroU32 = env::var(SQUATTING_ON).unwrap().parse().unwrap();
        let socket =
            socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None).unwrap();
        socket.bind_device_by_index_v4(Some(interface)).unwrap();
        let at: SocketAddr = (Ipv4Addr::UNSPECIFIED, THE_WIRE_PORT).into();
        socket.bind(&at.into()).unwrap();
        socket.listen(1).unwrap();
        println!("alo:holding");
        let mut rest = Vec::new();
        drop(std::io::stdin().read_to_end(&mut rest));
        drop(socket);
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

    /// The cables among `networks` over IPv4, sorted and joined with commas, and
    /// `-` for none.
    fn cables_among(networks: &[Network]) -> String {
        let names: BTreeSet<&str> = networks
            .iter()
            .filter(|network| network.address().is_ipv4())
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
                .expect("a port taken and let go of stopped the service");
            driving.join().unwrap();
        });
    }

    /// Reception, serving on the cables the far end lays, answering the far end
    /// on its standard output.
    #[test]
    #[ignore = "run inside its own network by the_far_end_with_two_cables_to_reception"]
    fn reception_on_two_cables() {
        must_be("reception");
        serving_as(
            &reception(),
            "a-port-another-program-let-go-of",
            |wire, door, stop| {
                let mut person = Talking::to(door);
                let mut line = String::new();
                loop {
                    line.clear();
                    let read = std::io::stdin().read_line(&mut line).unwrap();
                    assert!(read > 0, "the far end ended without stopping reception");
                    match line.trim_end() {
                        "capabilities" => println!(
                            "alo:capabilities {}",
                            if holds_no_capabilities() {
                                "none"
                            } else {
                                "some"
                            }
                        ),
                        "where" => println!(
                            "alo:where {}|{}",
                            cables_among(&wire.answered_on()),
                            cables_among(&wire.listened_on()),
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
