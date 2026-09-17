//! A machine that missed what the kernel said about its networks is still found
//! and reached on every one — on a real kernel, with the service running
//! throughout as `src/main.rs` runs it.
//!
//! *Machines find each other with zero configuration.*
//! `crate::a_cable_re_laid_between_two_readings` and
//! `crate::a_link_local_cable_re_laid_with_its_hardware_address` measured the
//! service reading the kernel's `RTM_DELLINK` after it was held still across a
//! cable re-laid. What neither measured is the kernel **not delivering** it. A
//! routing socket nobody reads in time overflows: the kernel drops what it would
//! have queued and says `ENOBUFS` once, `crate::unix::emptied` hands that on, and
//! `crate::interfaces_that_went::Went` reads it as *any interface may have gone*.
//! A dock with a dozen adapters, or a container host rebuilding its bridges while
//! the service is descheduled, is exactly that burst.
//!
//! # The two machines
//!
//! **The far end** is this test binary run again in a network namespace of its
//! own, inside a user namespace — so this needs no root and touches no
//! kernel-global state (`docs/autonomy/SHARED_MAIN.md`), and does not take
//! `alo_bounding::Waited::on_this_kernel()`. **Reception** is the binary run a
//! third time in a namespace nested inside the far end's, serving as
//! `src/main.rs` does and answering on its standard output what it is asked on
//! its standard input. Two `veth` cables join them, each laid every time at the
//! same numbers **and with the same hardware addresses**: one carrying IPv4, and
//! one carrying link-local IPv6 only.
//!
//! # How the overflow is made certain rather than hoped for
//!
//! Reception is **held still** as the fixtures before this hold it: `SIGSTOP`, and
//! wait until the kernel reports every thread of it stopped. Then, in reception's
//! network, batches of `veth` pairs are made and deleted — each a handful of
//! routing messages to reception's socket, which nothing is reading — until the
//! kernel's **own drop count for that socket** rises. That count is the `Drops`
//! column of `/proc/<pid>/net/netlink`, on the one row that is reception's socket:
//! a routing socket (`Eth` 0) subscribed to links and to both families' addresses
//! (`Groups` `00000111`), whose inode is among reception's open descriptors.
//!
//! Only then are **both cables deleted and laid again identically**, and the drop
//! count is read a second time and must have risen again: what the kernel said
//! about the cables was dropped too, not merely queued behind the burst. Both
//! link-local addresses on the IPv6 cable finish duplicate address detection
//! before reception is let go with `SIGCONT`, so the first dump reception reads is,
//! for each cable, exactly the network it had — nothing a dump could tell apart —
//! and the only thing it was told is that something was lost.
//!
//! *Once it has followed the kernel* is read from the kernel as well: nothing in
//! reception's network but its service joins `224.0.0.251` or `ff02::fb`, so each
//! re-laid interface listed in its group (`/proc/<pid>/net/igmp`,
//! `/proc/<pid>/net/igmp6`) is the service having joined it. Until both are,
//! nothing is asked; the far end's **first** question on each cable after that is
//! the one that must be answered.
//!
//! # In this order
//!
//! 1. **Both cables laid**: the far end finds reception on each, and reaches its
//!    port on each.
//! 2. **Held still**: the routing socket overflowed, measured; both cables deleted
//!    and laid again at the same numbers with the same hardware addresses, the drops
//!    measured again; neither re-laid interface is in its group.
//! 3. **Let go**: once reception has followed the kernel, the first question on
//!    each cable is answered with the same bytes as in step 1, the port is reached
//!    on each, and the person's door answers.
//! 4. **The service log** said what was lost exactly once, and the service stops
//!    when it is told to and not before.
//!
//! # What it shows
//!
//! With `Went::lost` read as nothing having gone, reception keeps the responder
//! and the IPv6 join it held at each number, never joins the re-laid interfaces,
//! and the fixture fails at step 3 with *reception never followed its cables …
//! 40 in the IPv4 group: false; 44 in the IPv6 group: false*.

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
    const INSIDE: &str = "ALO_AGENTD_A_MACHINE_THAT_MISSED_WHAT_THE_KERNEL_SAID_INSIDE";

    /// One cable between the far end and reception, laid identically every
    /// time.
    struct Cable {
        /// Reception's end.
        receptions_end: &'static str,
        /// The far end's end.
        far_end: &'static str,
        /// The numbers each end is laid at, reception's first.
        numbers: (u32, u32),
        /// The hardware addresses each end is laid with, reception's first —
        /// locally administered, so they name no real adapter.
        hardware: (&'static str, &'static str),
        /// The IPv4 addresses on each end with their prefixes, reception's
        /// first — none on a cable carrying link-local IPv6 only.
        ipv4: Option<(&'static str, &'static str)>,
    }

    /// The cable carrying IPv4.
    const OVER_IPV4: Cable = Cable {
        receptions_end: "cable0",
        far_end: "far0",
        numbers: (40, 41),
        hardware: ("02:a1:0d:00:00:40", "02:a1:0d:00:00:41"),
        ipv4: Some(("10.73.1.1/24", "10.73.1.2/24")),
    };

    /// The cable carrying link-local IPv6 only.
    const LINK_LOCAL: Cable = Cable {
        receptions_end: "cable1",
        far_end: "far1",
        numbers: (44, 45),
        hardware: ("02:a1:0d:00:00:44", "02:a1:0d:00:00:45"),
        ipv4: None,
    };

    /// How long anything here waits for the kernel or another machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// How many `veth` pairs one batch of the burst makes and deletes.
    const A_BATCH: usize = 64;
    /// How many batches the burst may take before the socket is declared
    /// impossible to overflow — a failure, never a skip.
    const AT_MOST: usize = 50;

    /// `ff02::fb` as `/proc/net/igmp6` spells a group.
    const THE_GROUP_IN_IGMP6: &str = "ff0200000000000000000000000000fb";

    /// A routing socket subscribed to `RTMGRP_LINK`, `RTMGRP_IPV4_IFADDR` and
    /// `RTMGRP_IPV6_IFADDR`, as `/proc/net/netlink` spells its groups.
    const THE_SERVICES_GROUPS: &str = "00000111";

    /// **A machine that missed what the kernel said about its networks is still
    /// found on every one**: the outer test, which makes the far end and reads
    /// what it saw and what reception's service log said.
    #[test]
    fn a_machine_that_missed_what_the_kernel_said_about_its_networks_is_still_found_on_every_one() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_machine_that_missed_what_the_kernel_said::tests::the_far_end_while_reception_misses_what_the_kernel_said";
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
            "with both cables laid it is found and reached on each",
            "held still, its routing socket overflowed as the kernel counts it",
            "both cables re-laid identically while the socket was full, and what the kernel said about them was dropped too",
            "once it followed the kernel, the first question on the cable carrying IPv4 was answered and the port reached there",
            "once it followed the kernel, the first question on the link-local cable was answered and the port reached there",
            "the service kept running, and its door answered",
            "what was said was the same bytes throughout",
        ] {
            assert!(
                said.contains(step),
                "the far end never said `{step}`:\n{said}\n{log}"
            );
        }
        let logged = format!("alo-agentd: {DROPPED}");
        assert_eq!(
            log.lines().filter(|line| *line == logged).count(),
            1,
            "the service log did not say once what was lost:\n{log}"
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

    /// Run iproute2 as [`ip_said`] does, and fail with what it said if it
    /// refuses.
    fn ip(pid: Option<u32>, args: &[&str]) {
        if let Err(why) = ip_said(pid, args) {
            panic!("{why}");
        }
    }

    /// Lay `cable` between here and reception at `pid`, at its numbers and with
    /// its hardware addresses, addressed where it carries IPv4, and up.
    ///
    /// A number an interface deleted a moment ago held may not be free the
    /// instant the deletion returns, so the laying is tried again until it is.
    fn lay(cable: &Cable, pid: u32) {
        let (receptions, far) = (cable.numbers.0.to_string(), cable.numbers.1.to_string());
        let pid_named = pid.to_string();
        let args = [
            "link",
            "add",
            cable.far_end,
            "index",
            &far,
            "address",
            cable.hardware.1,
            "type",
            "veth",
            "peer",
            "name",
            cable.receptions_end,
            "index",
            &receptions,
            "address",
            cable.hardware.0,
            "netns",
            &pid_named,
        ];
        let until = Instant::now() + PATIENCE;
        while let Err(why) = ip_said(None, &args) {
            assert!(Instant::now() < until, "the cable could not be laid: {why}");
            std::thread::sleep(Duration::from_millis(50));
        }
        if let Some((receptions, far)) = cable.ipv4 {
            ip(None, &["addr", "add", far, "dev", cable.far_end]);
            ip(
                Some(pid),
                &["addr", "add", receptions, "dev", cable.receptions_end],
            );
        }
        ip(None, &["link", "set", cable.far_end, "up"]);
        ip(Some(pid), &["link", "set", cable.receptions_end, "up"]);
    }

    /// Delete `cable` — both ends go with it — and wait until neither end is
    /// reported.
    fn delete(cable: &Cable, pid: u32) {
        ip(None, &["link", "del", cable.far_end]);
        let until = Instant::now() + PATIENCE;
        while ip_said(None, &["link", "show", cable.far_end]).is_ok()
            || ip_said(Some(pid), &["link", "show", cable.receptions_end]).is_ok()
        {
            assert!(Instant::now() < until, "the cable outlived its deletion");
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

    /// One end of a cable as [`an_end_now`] reads it, waiting until its
    /// link-local address has finished duplicate address detection.
    fn an_end(pid: Option<u32>, name: &str) -> (u32, String, Ipv6Addr) {
        let until = Instant::now() + PATIENCE;
        loop {
            if let Some(end) = an_end_now(pid, name) {
                return end;
            }
            assert!(
                Instant::now() < until,
                "{name} never had a usable link-local address"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Both ends of `cable`, reception's at `pid` first, once each is usable —
    /// asserted at the cable's numbers and hardware addresses.
    fn both_ends(cable: &Cable, pid: u32) -> ((u32, String, Ipv6Addr), (u32, String, Ipv6Addr)) {
        let receptions = an_end(Some(pid), cable.receptions_end);
        let far = an_end(None, cable.far_end);
        assert_eq!(
            (receptions.0, far.0),
            cable.numbers,
            "{} was laid at other numbers",
            cable.receptions_end
        );
        assert_eq!(
            (receptions.1.as_str(), far.1.as_str()),
            cable.hardware,
            "{} was laid with other hardware addresses",
            cable.receptions_end
        );
        (receptions, far)
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

    /// Make and delete `veth` pairs in reception's network at `pid`, a batch at a
    /// time, until the kernel counts messages dropped for its routing socket —
    /// and hand back that count.
    fn overflowed(pid: u32) -> u64 {
        let before = dropped_for(pid);
        for batch in 0..AT_MOST {
            let mut lines = String::new();
            for pair in 0..A_BATCH {
                let _ = writeln!(
                    lines,
                    "link add burst{batch}x{pair} type veth peer name burst{batch}y{pair}"
                );
            }
            for pair in 0..A_BATCH {
                let _ = writeln!(lines, "link del burst{batch}x{pair}");
            }
            let mut batching = ip_command(Some(pid))
                .args(["-batch", "-"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .expect("iproute2's `ip -batch` makes the burst");
            batching
                .stdin
                .take()
                .unwrap()
                .write_all(lines.as_bytes())
                .unwrap();
            let done = batching.wait_with_output().unwrap();
            assert!(
                done.status.success(),
                "the burst could not be made: {}",
                String::from_utf8_lossy(&done.stderr)
            );
            let now = dropped_for(pid);
            if now > before {
                return now;
            }
        }
        panic!(
            "{} veth pairs made and deleted never overflowed reception's routing socket",
            A_BATCH * AT_MOST
        );
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
            let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_machine_that_missed_what_the_kernel_said::tests::reception_while_it_misses_what_the_kernel_said";
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

        /// Whether its person's door answers.
        fn door_answers(&mut self) -> bool {
            self.told("door") == "yes"
        }

        /// Wait until the service answers, listens and is joined on the cable
        /// carrying IPv4 at its number, joined on the link-local
        /// cable at its number, **and** the kernel lists each interface in its
        /// group — which, in reception's network, only the service joins.
        fn until_it_follows(&mut self) {
            let (four, six) = (OVER_IPV4.numbers.0, LINK_LOCAL.numbers.0);
            let igmp = PathBuf::from(format!("/proc/{}/net/igmp", self.pid()));
            let igmp6 = PathBuf::from(format!("/proc/{}/net/igmp6", self.pid()));
            let wanted = format!("{four} {four} {four} {six}");
            let until = Instant::now() + PATIENCE;
            loop {
                let wire = self.told("where");
                let in_four = in_the_ipv4_group(&igmp, four);
                let in_six = in_the_ipv6_group(&igmp6, six);
                if wire == wanted && in_four && in_six {
                    return;
                }
                assert!(
                    Instant::now() < until,
                    "reception never followed its cables: answered, listened and joined over IPv4 and joined over link-local at `{wire}`; {four} in the IPv4 group: {in_four}; {six} in the IPv6 group: {in_six}"
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

    /// The address in `with_prefix`, without its prefix length.
    fn ip_of(with_prefix: &str) -> Ipv4Addr {
        with_prefix
            .split('/')
            .next()
            .and_then(|address| address.parse().ok())
            .unwrap()
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
        /// On the cable carrying IPv4.
        fn over_ipv4() -> Self {
            let (receptions, far) = OVER_IPV4.ipv4.unwrap();
            let reception = IpAddr::V4(ip_of(receptions));
            Self {
                from: SocketAddr::from((ip_of(far), 0)),
                group: SocketAddr::from((THE_ADDRESS, THE_PORT)),
                reception,
                port: SocketAddr::new(reception, THE_WIRE_PORT),
            }
        }

        /// On the link-local cable, from the far end's `fe80::` address `here` on
        /// its interface numbered `scope`, to reception at `there`.
        fn over_link_local(here: Ipv6Addr, scope: u32, there: Ipv6Addr) -> Self {
            Self {
                from: SocketAddr::V6(SocketAddrV6::new(here, 0, 0, scope)),
                group: SocketAddr::V6(SocketAddrV6::new(THE_IPV6_ADDRESS, THE_PORT, 0, scope)),
                reception: IpAddr::V6(there),
                port: SocketAddr::V6(SocketAddrV6::new(there, THE_WIRE_PORT, 0, scope)),
            }
        }

        /// Ask the group who is here, and the bytes reception answered with in
        /// hexadecimal — or `nothing` when it said nothing within a moment.
        fn asked(self) -> String {
            let socket = UdpSocket::bind(self.from).unwrap();
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

        /// Wait until reception answers, and hand back its bytes.
        ///
        /// For the first laying only: a cable just come up is not joined in the
        /// instant it is running, and nothing here says when it has been.
        fn heard_at_last(self) -> String {
            let until = Instant::now() + PATIENCE;
            loop {
                let heard = self.asked();
                if heard != "nothing" {
                    return heard;
                }
                assert!(Instant::now() < until, "the far end never found reception");
            }
        }

        /// Whether the service on reception's port answered a message on it — a
        /// handshake completed **and** the wire's own refusal read back.
        fn reached(self) -> bool {
            let Ok(mut connection) = TcpStream::connect_timeout(&self.port, Duration::from_secs(2))
            else {
                return false;
            };
            if connection
                .set_read_timeout(Some(Duration::from_secs(5)))
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

        /// Wait until the port is reached, for the first laying only.
        fn reached_at_last(self) {
            let until = Instant::now() + PATIENCE;
            while !self.reached() {
                assert!(
                    Instant::now() < until,
                    "the far end never reached reception"
                );
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    /// The far end: lays both cables, holds reception still while it overflows
    /// reception's routing socket and re-lays both, and asks.
    #[test]
    #[ignore = "run inside its own network by a_machine_that_missed_what_the_kernel_said_about_its_networks_is_still_found_on_every_one"]
    fn the_far_end_while_reception_misses_what_the_kernel_said() {
        must_be("far end");
        let mut reception = Reception::started();
        let pid = reception.pid();

        // 1. Both cables laid.
        lay(&OVER_IPV4, pid);
        lay(&LINK_LOCAL, pid);
        both_ends(&OVER_IPV4, pid);
        let (receptions_link_local, far_link_local) = both_ends(&LINK_LOCAL, pid);
        reception.until_it_follows();
        let over_ipv4 = Asking::over_ipv4();
        let over_link_local = Asking::over_link_local(
            far_link_local.2,
            LINK_LOCAL.numbers.1,
            receptions_link_local.2,
        );
        let said_over_ipv4 = over_ipv4.heard_at_last();
        let said_over_link_local = over_link_local.heard_at_last();
        over_ipv4.reached_at_last();
        over_link_local.reached_at_last();
        assert!(reception.door_answers());
        assert_eq!(
            dropped_for(pid),
            0,
            "the socket overflowed before the burst"
        );
        println!("with both cables laid it is found and reached on each");

        // 2. Held still: the socket overflowed, and both cables re-laid while it
        //    is full.
        reception.held_still(|pid| {
            let overflowed = overflowed(pid);
            println!(
                "held still, its routing socket overflowed as the kernel counts it: {overflowed} dropped"
            );
            for cable in [&OVER_IPV4, &LINK_LOCAL] {
                delete(cable, pid);
            }
            for cable in [&OVER_IPV4, &LINK_LOCAL] {
                lay(cable, pid);
            }
            assert_eq!(
                both_ends(&LINK_LOCAL, pid),
                (receptions_link_local.clone(), far_link_local.clone()),
                "the link-local cable was not laid again identically"
            );
            both_ends(&OVER_IPV4, pid);
            let after = dropped_for(pid);
            assert!(
                after > overflowed,
                "what the kernel said about the cables was not dropped: {after} dropped, as before"
            );
            assert!(
                !in_the_ipv4_group(
                    &PathBuf::from(format!("/proc/{pid}/net/igmp")),
                    OVER_IPV4.numbers.0
                ),
                "the kernel put the interface laid at a deleted one's number in its IPv4 group"
            );
            assert!(
                !in_the_ipv6_group(
                    &PathBuf::from(format!("/proc/{pid}/net/igmp6")),
                    LINK_LOCAL.numbers.0
                ),
                "the kernel put the interface laid at a deleted one's number in its IPv6 group"
            );
            println!(
                "both cables re-laid identically while the socket was full, and what the kernel said about them was dropped too: {after} dropped"
            );
        });

        // 3. Let go: the first question on each cable once it has followed.
        reception.until_it_follows();
        assert_eq!(
            over_ipv4.asked(),
            said_over_ipv4,
            "the first question on the cable carrying IPv4 went unanswered"
        );
        assert!(
            over_ipv4.reached(),
            "the first connection on the cable carrying IPv4 was not answered"
        );
        println!(
            "once it followed the kernel, the first question on the cable carrying IPv4 was answered and the port reached there"
        );
        assert_eq!(
            over_link_local.asked(),
            said_over_link_local,
            "the first question on the link-local cable went unanswered"
        );
        assert!(
            over_link_local.reached(),
            "the first connection on the link-local cable was not answered"
        );
        println!(
            "once it followed the kernel, the first question on the link-local cable was answered and the port reached there"
        );
        assert!(reception.door_answers());
        println!("the service kept running, and its door answered");
        println!("what was said was the same bytes throughout");

        reception.stopped();
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

    /// The number of `cable` among `networks` in the family `ipv6` says, or `-`
    /// where it is not among them.
    fn numbered(networks: &[Network], cable: &str, ipv6: bool) -> String {
        networks
            .iter()
            .find(|network| network.address().is_ipv6() == ipv6 && network.name() == cable)
            .map_or_else(|| "-".to_owned(), |network| network.index().to_string())
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
                .expect("a routing socket that overflowed stopped the service");
            driving.join().unwrap();
        });
    }

    /// Reception, serving while its routing socket overflows and its cables are
    /// re-laid, answering the far end on its standard output.
    #[test]
    #[ignore = "run inside its own network by the_far_end_while_reception_misses_what_the_kernel_said"]
    fn reception_while_it_misses_what_the_kernel_said() {
        must_be("reception");
        serving_as(
            &reception(),
            "missed-what-the-kernel-said",
            |wire, door, stop| {
                let mut person = Talking::to(door);
                let mut line = String::new();
                loop {
                    line.clear();
                    let read = std::io::stdin().read_line(&mut line).unwrap();
                    assert!(read > 0, "the far end ended without stopping reception");
                    match line.trim_end() {
                        "where" => println!(
                            "alo:where {} {} {} {}",
                            numbered(&wire.answered_on(), OVER_IPV4.receptions_end, false),
                            numbered(&wire.listened_on(), OVER_IPV4.receptions_end, false),
                            numbered(&wire.joined(), OVER_IPV4.receptions_end, false),
                            numbered(&wire.joined(), LINK_LOCAL.receptions_end, true),
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
