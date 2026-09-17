//! A dock with a dozen adapters is found on every one of them at once — on a real
//! kernel, with the service running throughout as `src/main.rs` runs it.
//!
//! *Machines find each other with zero configuration.* Every fixture before this
//! one put a machine on one or two cables. A docking station or a lab switch with
//! many ports brings a dozen interfaces up in one moment, and the service then
//! holds one responder, one listener and one IPv6 join per network: a socket, a
//! descriptor or a join that ran out part of the way through would show as *found
//! on eleven networks of twelve*. This measures that it does not, that half the
//! dock unplugged leaves the machine on the other half and on nothing at the half
//! that went, and that a network that does fail is a line in the service log
//! naming it — never a stopped service, never a silent miss.
//!
//! # The two machines
//!
//! **The far end** is this test binary run again in a network namespace of its
//! own, inside a user namespace — so this needs no root and touches no
//! kernel-global state (`docs/autonomy/SHARED_MAIN.md`), and does not take
//! `alo_bounding::Waited::on_this_kernel()`. **Reception** is the binary run a
//! third time in a namespace nested inside the far end's, serving as
//! `src/main.rs` does and answering on its standard output what it is asked on
//! its standard input. Twelve `veth` cables join them — `cable0` to `cable11` at
//! reception's end, numbered 40 to 51, and `far0` to `far11` at the far end's,
//! numbered 60 to 71, each with a hardware address given. `cable0` to `cable5`
//! carry IPv4; `cable6` to `cable11` carry link-local IPv6 only.
//!
//! # How *in one burst* is made certain rather than hoped for
//!
//! Reception is **held still** as `crate::a_machine_that_missed_what_the_kernel_said`
//! holds it — `SIGSTOP`, and wait until the kernel reports every thread of it
//! stopped — and all twelve cables are laid in one `ip -batch` at the far end
//! and addressed and brought up in one `ip -batch` in reception's network. Every
//! link-local address on every cable finishes duplicate address detection before
//! reception is let go, so the first thing it reads is all twelve at once. Whether
//! that burst overflowed its routing socket is read from the kernel's own drop
//! count for that socket (`/proc/<pid>/net/netlink`) and said either way: the
//! fixture passes on both, and the service log must say `DROPPED` exactly when the
//! kernel counted a drop.
//!
//! *Once it has followed the kernel* is read from the kernel as well, per
//! interface: nothing in reception's network but its service joins `224.0.0.251`
//! or `ff02::fb`, so each cable listed in its group (`/proc/<pid>/net/igmp`,
//! `/proc/<pid>/net/igmp6`) is the service having joined it. Until every one is,
//! nothing is asked; the far end's **first** question on each cable after that is
//! the one that must be answered.
//!
//! # How a network is made to fail
//!
//! A thirteenth cable, `cable12` at 52 carrying IPv4, is laid while reception is
//! held still, and before it is let go a process in reception's network — this
//! binary once more, as *the squatter* — listens on the port presence advertises,
//! held to that one interface, without sharing it. The kernel then refuses the
//! service's listener there with `EADDRINUSE`, which is a failure nothing in this
//! fixture fakes: a port another program took is what one looks like on a real
//! machine.
//!
//! # In this order
//!
//! 1. **Twelve cables in one burst**, laid while reception is held still: once it
//!    has followed, the first question on every cable is answered, the port is
//!    reached on every cable, and every answer is the same bytes.
//! 2. **Half unplugged in one batch** — three of each kind: reception is found and
//!    reached on the other six, holds nothing at the six that went, and its door
//!    answers.
//! 3. **One network that fails**: reception is still found on `cable12`, the port
//!    is not reached there, it is reached on every other cable carrying IPv4, and
//!    the door answers.
//! 4. **The service log** names `cable12` in every per-network failure line and no
//!    other network in any, said `DROPPED` exactly when the kernel counted a drop,
//!    and the service stops when it is told to and not before.
//!
//! # What it shows
//!
//! No limit bites at a dozen networks on this kernel, and none is imposed by the
//! service: one datagram socket and one listener per network, each joined once
//! (`net.ipv4.igmp_max_memberships` counts memberships per socket), and one IPv6
//! socket joined once per network, whose memberships are charged to
//! `net.core.optmem_max`. `docs/quirks.md` records what was measured.

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
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, TcpStream, UdpSocket};
    use std::num::NonZeroU32;
    use std::os::unix::net::UnixStream;
    use std::path::{Path, PathBuf};
    use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
    use std::time::{Duration, Instant, SystemTime};

    use alo_egress::{EgressPolicy, Indicator};
    use alo_files::OnThisMachine;
    use alo_keeping::Keeping;
    use alo_nearby::{MachineId, THE_ADDRESS, THE_IPV6_ADDRESS, THE_PORT};
    use alo_protocol::ToAPerson;
    use alo_record::Record;
    use alo_turn::Machine;

    use crate::hearing::NOT_FOR_THIS_WIRE;
    use crate::hosting::Hosted;
    use crate::joining::DROPPED;
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
    const INSIDE: &str = "ALO_AGENTD_A_DOCK_WITH_A_DOZEN_ADAPTERS_INSIDE";

    /// Set on the squatter, naming the interface number it holds the port on.
    const SQUATTING_ON: &str = "ALO_AGENTD_A_DOCK_WITH_A_DOZEN_ADAPTERS_SQUATTING_ON";

    /// How many cables the dock brings up at once.
    const A_DOZEN: u32 = 12;

    /// The cable a network is made to fail on.
    const THE_ONE_THAT_FAILS: Cable = Cable(12);

    /// The cables unplugged in one batch: three carrying IPv4 and three carrying
    /// link-local IPv6 only, the last one enumerated among them.
    const UNPLUGGED: [Cable; 6] = [Cable(3), Cable(4), Cable(5), Cable(9), Cable(10), Cable(11)];

    /// How long anything here waits for the kernel or another machine.
    const PATIENCE: Duration = Duration::from_secs(60);

    /// `ff02::fb` as `/proc/net/igmp6` spells a group.
    const THE_GROUP_IN_IGMP6: &str = "ff0200000000000000000000000000fb";

    /// A routing socket subscribed to `RTMGRP_LINK`, `RTMGRP_IPV4_IFADDR` and
    /// `RTMGRP_IPV6_IFADDR`, as `/proc/net/netlink` spells its groups.
    const THE_SERVICES_GROUPS: &str = "00000111";

    /// The line the far end ends with, carrying the kernel's drop count for
    /// reception's routing socket.
    const IN_ALL: &str = "in all, the kernel dropped for its routing socket:";

    /// One cable between the far end and reception, named by its place on the
    /// dock and laid identically every time.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    struct Cable(u32);

    impl Cable {
        /// Every cable the dock brings up at once.
        fn the_dozen() -> Vec<Self> {
            (0..A_DOZEN).map(Self).collect()
        }

        /// Reception's end.
        fn receptions_end(self) -> String {
            format!("cable{}", self.0)
        }

        /// The far end's end.
        fn far_end(self) -> String {
            format!("far{}", self.0)
        }

        /// The numbers each end is laid at, reception's first.
        const fn numbers(self) -> (u32, u32) {
            (40 + self.0, 60 + self.0)
        }

        /// The hardware addresses each end is laid with, reception's first —
        /// locally administered, so they name no real adapter.
        fn hardware(self) -> (String, String) {
            let (receptions, far) = self.numbers();
            (
                format!("02:a1:0d:00:01:{receptions:02}"),
                format!("02:a1:0d:00:01:{far:02}"),
            )
        }

        /// Whether it carries IPv4 as well as link-local IPv6.
        const fn carries_ipv4(self) -> bool {
            self.0 < A_DOZEN / 2 || self.0 == THE_ONE_THAT_FAILS.0
        }

        /// The IPv4 addresses on each end, reception's first — none on a cable
        /// carrying link-local IPv6 only.
        fn ipv4(self) -> Option<(Ipv4Addr, Ipv4Addr)> {
            let network = u8::try_from(self.0).unwrap();
            self.carries_ipv4().then_some((
                Ipv4Addr::new(10, 74, network, 1),
                Ipv4Addr::new(10, 74, network, 2),
            ))
        }
    }

    /// **A dock with a dozen adapters is found on every one of them at once**:
    /// the outer test, which makes the far end and reads what it saw and what
    /// reception's service log said.
    #[test]
    fn a_dock_with_a_dozen_adapters_is_found_on_every_one_of_them_at_once() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_dock_with_a_dozen_adapters::tests::the_far_end_with_a_dozen_cables_to_reception";
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
            "twelve cables laid in one burst while it was held still",
            "once it followed the kernel, the first question on every one of the twelve cables was answered",
            "the port was reached on every one of the twelve cables",
            "with six cables unplugged in one batch it is found and reached on the other six, and holds nothing at the six that went",
            "with its port taken on cable12 it is still found there, reached on every other cable carrying IPv4, and its door answers",
            "what was said was the same bytes on every cable, throughout",
        ] {
            assert!(
                said.contains(step),
                "the far end never said `{step}`:\n{said}\n{log}"
            );
        }

        let failing = THE_ONE_THAT_FAILS.receptions_end();
        let (receptions, _) = THE_ONE_THAT_FAILS.ipv4().unwrap();
        let named = format!(
            "the port presence advertises could not be bound on {failing} ({receptions}): "
        );
        assert!(
            log.lines()
                .any(|line| line.starts_with("alo-agentd: ") && line.contains(&named)),
            "the service log never named the network its port could not be bound on:\n{log}"
        );
        let per_network: Vec<&str> = log
            .lines()
            .filter(|line| {
                [
                    "could not be answered on ",
                    "could not be answered again on ",
                    "could not be bound on ",
                    "could not be joined on ",
                ]
                .iter()
                .any(|failure| line.contains(failure))
            })
            .collect();
        assert!(
            per_network
                .iter()
                .all(|line| line.contains(&format!(" on {failing} ("))),
            "a network other than {failing} failed:\n{}",
            per_network.join("\n")
        );

        let dropped: u64 = said
            .lines()
            .find_map(|line| line.strip_prefix(IN_ALL))
            .and_then(|count| count.trim().parse().ok())
            .unwrap_or_else(|| panic!("the far end never said what the kernel dropped:\n{said}"));
        let logged = format!("alo-agentd: {DROPPED}");
        let said_dropped = log.lines().filter(|line| *line == logged).count();
        assert_eq!(
            said_dropped > 0,
            dropped > 0,
            "the kernel dropped {dropped} messages for the routing socket and the service log said so {said_dropped} times:\n{log}"
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
    /// named, and hand back what it printed or what it said when it refused.
    fn ip_said(pid: Option<u32>, args: &[&str]) -> Result<String, String> {
        let ran = ip_command(pid)
            .args(args)
            .output()
            .expect("iproute2's `ip` and util-linux's `nsenter` make the cables");
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

    /// Lay `cables` between here and reception at `pid` in one batch at each end:
    /// every pair made at its numbers with its hardware addresses in one, and
    /// reception's ends addressed and brought up in another.
    fn laid(cables: &[Cable], pid: u32) {
        let (mut here, mut there) = (String::new(), String::new());
        for cable in cables {
            let (receptions_number, far_number) = cable.numbers();
            let (receptions_hardware, far_hardware) = cable.hardware();
            let (receptions_end, far_end) = (cable.receptions_end(), cable.far_end());
            let _ = writeln!(
                here,
                "link add {far_end} index {far_number} address {far_hardware} type veth peer name {receptions_end} index {receptions_number} address {receptions_hardware} netns {pid}"
            );
            if let Some((receptions, far)) = cable.ipv4() {
                let _ = writeln!(here, "addr add {far}/24 dev {far_end}");
                let _ = writeln!(there, "addr add {receptions}/24 dev {receptions_end}");
            }
            let _ = writeln!(here, "link set {far_end} up");
            let _ = writeln!(there, "link set {receptions_end} up");
        }
        batch(None, &here);
        batch(Some(pid), &there);
    }

    /// Unplug `cables` in one batch — both ends go with each — and wait until no
    /// end of any of them is reported.
    fn unplugged(cables: &[Cable], pid: u32) {
        let lines = cables.iter().fold(String::new(), |mut lines, cable| {
            let _ = writeln!(lines, "link del {}", cable.far_end());
            lines
        });
        batch(None, &lines);
        let until = Instant::now() + PATIENCE;
        while cables.iter().any(|cable| {
            ip_said(None, &["link", "show", &cable.far_end()]).is_ok()
                || ip_said(Some(pid), &["link", "show", &cable.receptions_end()]).is_ok()
        }) {
            assert!(Instant::now() < until, "a cable outlived its unplugging");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// One end of a cable as the kernel reports it, in the network of `pid` when
    /// one is named: its number, its hardware address and its link-local address
    /// — or `None` while it is missing or that address is still being checked.
    fn an_end_now(pid: Option<u32>, name: &str) -> Option<(u32, String, Ipv6Addr)> {
        let link = ip_said(pid, &["-o", "link", "show", name]).ok()?;
        let number = link.split(':').next()?.trim().parse().ok()?;
        let words: Vec<&str> = link.split_whitespace().collect();
        let hardware = words
            .iter()
            .position(|word| *word == "link/ether")
            .and_then(|at| words.get(at + 1))?
            .to_string();
        let addresses = ip_said(
            pid,
            &["-6", "-o", "addr", "show", "dev", name, "scope", "link"],
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

    /// Both link-local addresses of `cable`, reception's at `pid` first, once each
    /// has finished duplicate address detection — with both ends asserted at the
    /// cable's numbers and hardware addresses.
    fn both_ends(cable: Cable, pid: u32) -> (Ipv6Addr, Ipv6Addr) {
        let until = Instant::now() + PATIENCE;
        let (receptions, far) = loop {
            if let (Some(receptions), Some(far)) = (
                an_end_now(Some(pid), &cable.receptions_end()),
                an_end_now(None, &cable.far_end()),
            ) {
                break (receptions, far);
            }
            assert!(
                Instant::now() < until,
                "{} never had a usable link-local address at both ends",
                cable.receptions_end()
            );
            std::thread::sleep(Duration::from_millis(50));
        };
        assert_eq!(
            (receptions.0, far.0),
            cable.numbers(),
            "{} was laid at other numbers",
            cable.receptions_end()
        );
        assert_eq!(
            (receptions.1, far.1),
            cable.hardware(),
            "{} was laid with other hardware addresses",
            cable.receptions_end()
        );
        (receptions.2, far.2)
    }

    /// Whether the interface numbered `index` is in `224.0.0.251`, as `igmp` —
    /// `/proc/net/igmp` of some network — lists it: a line naming each
    /// interface, then one line per group it is in, each the address's four
    /// bytes as the kernel holds them.
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

    /// How many messages the kernel has dropped for reception's routing socket,
    /// as it counts them.
    ///
    /// Read from `/proc/<pid>/net/netlink` — `sk Eth Pid Groups Rmem Wmem Dump
    /// Locks Drops Inode` — on the one row that is a routing socket subscribed
    /// as the service subscribes, whose inode is one of reception's open
    /// descriptors. Exactly one such row is insisted on: a drop count read off
    /// somebody else's socket would prove nothing.
    fn dropped_for(pid: u32) -> u64 {
        let open: BTreeSet<String> = std::fs::read_dir(format!("/proc/{pid}/fd"))
            .unwrap()
            .filter_map(|fd| std::fs::read_link(fd.ok()?.path()).ok())
            .filter_map(|target| {
                target
                    .to_str()?
                    .strip_prefix("socket:[")?
                    .strip_suffix(']')
                    .map(str::to_owned)
            })
            .collect();
        let table = std::fs::read_to_string(format!("/proc/{pid}/net/netlink")).unwrap();
        let rows: Vec<Vec<&str>> = table
            .lines()
            .skip(1)
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .filter(|row| {
                row.len() == 10
                    && row.get(1) == Some(&"0")
                    && row.get(3) == Some(&THE_SERVICES_GROUPS)
                    && row.get(9).is_some_and(|inode| open.contains(*inode))
            })
            .collect();
        assert_eq!(
            rows.len(),
            1,
            "reception does not hold exactly one routing socket subscribed as the service subscribes:\n{table}"
        );
        rows.first()
            .and_then(|row| row.get(8))
            .and_then(|drops| drops.parse().ok())
            .unwrap()
    }

    /// The names of `cables` as reception's service reports networks: sorted,
    /// joined with commas, and `-` for none.
    fn named(cables: impl Iterator<Item = Cable>) -> String {
        let names: BTreeSet<String> = cables.map(Cable::receptions_end).collect();
        if names.is_empty() {
            "-".to_owned()
        } else {
            names.into_iter().collect::<Vec<_>>().join(",")
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
        /// Reception, started in a network namespace nested inside this one.
        fn started() -> Self {
            let exe = env::current_exe().unwrap();
            let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_dock_with_a_dozen_adapters::tests::reception_on_a_dozen_cables";
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
        /// service's own round, so an answer also says every round before it,
        /// a change of networks included, is finished.
        fn door_answers(&mut self) -> bool {
            self.told("door") == "yes"
        }

        /// What the service holds, as `answered|listened|joined over IPv4|joined
        /// over IPv6`, each the cables it is on there.
        fn where_it_is(&mut self) -> String {
            self.told("where")
        }

        /// What [`where_it_is`](Self::where_it_is) must say when the service is on
        /// every one of `plugged_in`, listening on every one carrying IPv4 but
        /// `not_listened_on`.
        fn on(plugged_in: &[Cable], not_listened_on: Option<Cable>) -> String {
            let ipv4 = || {
                plugged_in
                    .iter()
                    .copied()
                    .filter(|cable| cable.carries_ipv4())
            };
            format!(
                "{}|{}|{}|{}",
                named(ipv4()),
                named(ipv4().filter(|cable| Some(*cable) != not_listened_on)),
                named(ipv4()),
                named(plugged_in.iter().copied()),
            )
        }

        /// Wait until the service is on every one of `plugged_in` as
        /// [`on`](Self::on) says, **and** the kernel lists each of them in its
        /// groups — which, in reception's network, only the service joins — and
        /// then until a round of the service after that has finished.
        fn until_it_follows(&mut self, plugged_in: &[Cable], not_listened_on: Option<Cable>) {
            let igmp = PathBuf::from(format!("/proc/{}/net/igmp", self.pid()));
            let igmp6 = PathBuf::from(format!("/proc/{}/net/igmp6", self.pid()));
            let wanted = Self::on(plugged_in, not_listened_on);
            let until = Instant::now() + PATIENCE;
            loop {
                let held = self.where_it_is();
                let not_joined: Vec<String> = plugged_in
                    .iter()
                    .filter(|cable| {
                        let index = cable.numbers().0;
                        (cable.carries_ipv4() && !in_the_ipv4_group(&igmp, index))
                            || !in_the_ipv6_group(&igmp6, index)
                    })
                    .map(|cable| cable.receptions_end())
                    .collect();
                if held == wanted && not_joined.is_empty() {
                    break;
                }
                assert!(
                    Instant::now() < until,
                    "reception never followed its cables: it holds `{held}` where `{wanted}` was wanted, and the kernel does not list {not_joined:?} in their groups"
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
        /// A far end that fails takes reception with it, so a reception left held
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

    /// A program in reception's network holding the port presence advertises on
    /// one interface, for as long as it lives.
    struct Squatter {
        /// Its process.
        process: Child,
        /// What it says on, held open until it ends: a harness that reports its
        /// test to a pipe nobody holds fails with `EPIPE`.
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
                    "a_dock_with_a_dozen_adapters::tests::a_squatter_on_the_port",
                ])
                .env(INSIDE, "squatter")
                .env(SQUATTING_ON, cable.numbers().0.to_string())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("util-linux's `nsenter` puts the squatter in reception's network");
            let hearing = BufReader::new(process.stdout.take().unwrap());
            // Held before anything can fail, so that a failure here still ends
            // the squatter and waits on it.
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

    /// Where the far end asks on one cable, and who must answer.
    #[derive(Clone, Copy)]
    struct Asking {
        /// The far end's own address on the cable, bound to ask from.
        from: SocketAddr,
        /// The group the question goes to on the cable.
        group: SocketAddr,
        /// Reception's address on the cable.
        reception: IpAddr,
        /// Reception's port on the cable.
        port: SocketAddr,
    }

    impl Asking {
        /// On `cable` over IPv4, where it carries it, and over link-local IPv6
        /// otherwise — from the far end's `fe80::` address `here` to reception's
        /// at `there`.
        fn on(cable: Cable, (there, here): (Ipv6Addr, Ipv6Addr)) -> Self {
            match cable.ipv4() {
                Some((receptions, far)) => {
                    let reception = IpAddr::V4(receptions);
                    Self {
                        from: SocketAddr::from((far, 0)),
                        group: SocketAddr::from((THE_ADDRESS, THE_PORT)),
                        reception,
                        port: SocketAddr::new(reception, THE_WIRE_PORT),
                    }
                }
                None => {
                    let scope = cable.numbers().1;
                    Self {
                        from: SocketAddr::V6(SocketAddrV6::new(here, 0, 0, scope)),
                        group: SocketAddr::V6(SocketAddrV6::new(
                            THE_IPV6_ADDRESS,
                            THE_PORT,
                            0,
                            scope,
                        )),
                        reception: IpAddr::V6(there),
                        port: SocketAddr::V6(SocketAddrV6::new(there, THE_WIRE_PORT, 0, scope)),
                    }
                }
            }
        }

        /// Ask the group who is here, and the bytes reception answered with in
        /// hexadecimal — or `nothing` when it said nothing within a moment.
        ///
        /// Over IPv4 the question is sent out of the far end's own address on
        /// the cable, named as the interface to send multicast from: with six
        /// cables carrying IPv4 at the far end, the route alone would choose one.
        fn asked(self) -> String {
            let socket = UdpSocket::bind(self.from).unwrap();
            if let SocketAddr::V4(from) = self.from {
                socket2::SockRef::from(&socket)
                    .set_multicast_if_v4(from.ip())
                    .unwrap();
            }
            socket
                .set_read_timeout(Some(Duration::from_millis(500)))
                .unwrap();
            let question = alo_nearby::advertising::a_question().unwrap();
            if socket.send_to(&question, self.group).is_err() {
                return "nothing".to_owned();
            }
            let until = Instant::now() + Duration::from_secs(3);
            let mut heard = [0_u8; 1_500];
            while Instant::now() < until {
                if let Ok((how_many, from)) = socket.recv_from(&mut heard)
                    && from.ip() == self.reception
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

        /// Whether the service on reception's port answered a message on it — a
        /// handshake completed **and** the wire's own refusal read back.
        fn reached(self) -> bool {
            let Ok(mut connection) = TcpStream::connect_timeout(&self.port, Duration::from_secs(2))
            else {
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
    }

    /// Ask on every one of `cables` once, and hand back what each was answered
    /// with — failing, naming the cable, where the first question went
    /// unanswered.
    fn first_questions(asking: &[(Cable, Asking)]) -> Vec<String> {
        asking
            .iter()
            .map(|(cable, asking)| {
                let heard = asking.asked();
                assert_ne!(
                    heard,
                    "nothing",
                    "the first question on {} went unanswered",
                    cable.receptions_end()
                );
                heard
            })
            .collect()
    }

    /// Fail, naming the cable, unless the port is reached on every one of
    /// `asking`.
    fn reached_on_every_one(asking: &[(Cable, Asking)]) {
        for (cable, asking) in asking {
            assert!(
                asking.reached(),
                "the port was not reached on {}",
                cable.receptions_end()
            );
        }
    }

    /// The far end: lays twelve cables in one burst while reception is held
    /// still, unplugs half, makes one network fail, and asks on every cable.
    #[test]
    #[ignore = "run inside its own network by a_dock_with_a_dozen_adapters_is_found_on_every_one_of_them_at_once"]
    fn the_far_end_with_a_dozen_cables_to_reception() {
        must_be("far end");
        let mut reception = Reception::started();
        assert!(reception.door_answers(), "reception never served");
        let pid = reception.pid();
        assert_eq!(dropped_for(pid), 0, "the socket overflowed before the dock");

        // 1. Twelve cables in one burst, laid while reception is held still.
        let dozen = Cable::the_dozen();
        let addresses = reception.held_still(|pid| {
            laid(&dozen, pid);
            let addresses: Vec<(Ipv6Addr, Ipv6Addr)> =
                dozen.iter().map(|cable| both_ends(*cable, pid)).collect();
            let igmp = PathBuf::from(format!("/proc/{pid}/net/igmp"));
            let igmp6 = PathBuf::from(format!("/proc/{pid}/net/igmp6"));
            for cable in &dozen {
                assert!(
                    !in_the_ipv4_group(&igmp, cable.numbers().0)
                        && !in_the_ipv6_group(&igmp6, cable.numbers().0),
                    "something joined {} while reception was held still",
                    cable.receptions_end()
                );
            }
            println!(
                "twelve cables laid in one burst while it was held still: {} messages dropped for its routing socket",
                dropped_for(pid)
            );
            addresses
        });
        reception.until_it_follows(&dozen, None);
        let asking: Vec<(Cable, Asking)> = dozen
            .iter()
            .zip(&addresses)
            .map(|(cable, ends)| (*cable, Asking::on(*cable, *ends)))
            .collect();
        let answers = first_questions(&asking);
        let said = answers.first().unwrap().clone();
        assert!(
            answers.iter().all(|answer| *answer == said),
            "the cables were not all answered with the same bytes: {answers:?}"
        );
        println!(
            "once it followed the kernel, the first question on every one of the twelve cables was answered"
        );
        reached_on_every_one(&asking);
        println!("the port was reached on every one of the twelve cables");

        // 2. Half unplugged in one batch.
        unplugged(&UNPLUGGED, pid);
        let plugged_in: Vec<Cable> = dozen
            .iter()
            .copied()
            .filter(|cable| !UNPLUGGED.contains(cable))
            .collect();
        reception.until_it_follows(&plugged_in, None);
        let still: Vec<(Cable, Asking)> = asking
            .iter()
            .copied()
            .filter(|(cable, _)| plugged_in.contains(cable))
            .collect();
        assert!(
            first_questions(&still).iter().all(|answer| *answer == said),
            "an answer after the unplugging was not the same bytes"
        );
        reached_on_every_one(&still);
        let held = reception.where_it_is();
        let holds: BTreeSet<&str> = held.split(['|', ',']).collect();
        for cable in UNPLUGGED {
            assert!(
                !holds.contains(cable.receptions_end().as_str()),
                "reception still holds something at {}: {held}",
                cable.receptions_end()
            );
        }
        assert!(reception.door_answers());
        println!(
            "with six cables unplugged in one batch it is found and reached on the other six, and holds nothing at the six that went"
        );

        // 3. One network that fails: the port taken on a cable laid while
        //    reception is held still.
        let (squatter, failing) = reception.held_still(|pid| {
            laid(&[THE_ONE_THAT_FAILS], pid);
            let ends = both_ends(THE_ONE_THAT_FAILS, pid);
            (
                Squatter::on(THE_ONE_THAT_FAILS, pid),
                Asking::on(THE_ONE_THAT_FAILS, ends),
            )
        });
        let with_it: Vec<Cable> = plugged_in
            .iter()
            .copied()
            .chain([THE_ONE_THAT_FAILS])
            .collect();
        reception.until_it_follows(&with_it, Some(THE_ONE_THAT_FAILS));
        assert_eq!(
            first_questions(&[(THE_ONE_THAT_FAILS, failing)]),
            std::slice::from_ref(&said),
            "the answer on the cable whose port was taken was not the same bytes"
        );
        assert!(
            !failing.reached(),
            "the port was reached on the cable another program holds it on"
        );
        let ipv4: Vec<(Cable, Asking)> = still
            .iter()
            .copied()
            .filter(|(cable, _)| cable.carries_ipv4())
            .collect();
        reached_on_every_one(&ipv4);
        assert!(reception.door_answers());
        println!(
            "with its port taken on cable12 it is still found there, reached on every other cable carrying IPv4, and its door answers"
        );
        println!("what was said was the same bytes on every cable, throughout");

        squatter.gone();
        println!("{IN_ALL} {}", dropped_for(pid));
        reception.stopped();
    }

    /// The squatter: listens on the port presence advertises, held to the
    /// interface [`SQUATTING_ON`] names and sharing it with nothing, until its
    /// standard input closes.
    #[test]
    #[ignore = "run in reception's network by the_far_end_with_a_dozen_cables_to_reception"]
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

    /// The cables among `networks` in the family `ipv6` says, as [`named`] spells
    /// them.
    fn cables_among(networks: &[Network], ipv6: bool) -> String {
        let names: BTreeSet<&str> = networks
            .iter()
            .filter(|network| network.address().is_ipv6() == ipv6)
            .map(Network::name)
            .filter(|name| name.starts_with("cable"))
            .collect();
        if names.is_empty() {
            "-".to_owned()
        } else {
            names.into_iter().collect::<Vec<_>>().join(",")
        }
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
                .expect("a dozen networks, or one of them failing, stopped the service");
            driving.join().unwrap();
        });
    }

    /// Reception, serving on however many cables the far end lays, answering
    /// the far end on its standard output.
    #[test]
    #[ignore = "run inside its own network by the_far_end_with_a_dozen_cables_to_reception"]
    fn reception_on_a_dozen_cables() {
        must_be("reception");
        serving_as(
            &reception(),
            "a-dock-with-a-dozen-adapters",
            |wire, door, stop| {
                let mut person = Talking::to(door);
                let mut line = String::new();
                loop {
                    line.clear();
                    let read = std::io::stdin().read_line(&mut line).unwrap();
                    assert!(read > 0, "the far end ended without stopping reception");
                    match line.trim_end() {
                        "where" => println!(
                            "alo:where {}|{}|{}|{}",
                            cables_among(&wire.answered_on(), false),
                            cables_among(&wire.listened_on(), false),
                            cables_among(&wire.joined(), false),
                            cables_among(&wire.joined(), true),
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
