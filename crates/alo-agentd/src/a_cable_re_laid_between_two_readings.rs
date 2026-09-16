//! A cable deleted and laid again between two readings of the interfaces is
//! still found and reached over IPv4 — on a real kernel, with the service
//! running throughout as `src/main.rs` runs it.
//!
//! *Machines find each other with zero configuration.* The responders
//! (`crate::responding`) and the listeners (`crate::listeners`) match what they
//! hold to what the kernel reports by the interface's **number**, and
//! `crate::a_cable_pulled_and_plugged_in_again` measured a cable re-laid at a new
//! number, in a round of its own. What it could not measure is a cable deleted
//! and laid again **at the same number before the service reads the interfaces
//! again** — a dock re-enumerating, a namespace rebuilt — which reads, before
//! and after, as nothing having changed at all.
//!
//! # How the ordering is made certain
//!
//! A service reads the interfaces a moment after the kernel says anything, so a
//! cable deleted and re-laid by a test racing it lands between two readings only
//! when the test happens to be faster. So reception is **held still**: it is its
//! own process, and the far end stops it with `SIGSTOP`, waits until the kernel
//! reports every one of its threads stopped, deletes the cable and lays it again,
//! and only then lets it go on with `SIGCONT`. Every message the kernel sent in
//! between is waiting on reception's routing socket, and the next reading it
//! takes is of the re-laid cable. Nothing about reception is changed to allow
//! it: a process stopped and continued is the same process, and the service
//! inside it does not restart.
//!
//! *After the service has followed the kernel* is read from the kernel too:
//! nothing in reception's network but the service joins the discovery group, so
//! the re-laid interface in the group (`/proc/<pid>/net/igmp`) is the service
//! having answered on it — and until it is, the question is not asked. The
//! **first** question after that is the one that must be answered.
//!
//! # The two machines
//!
//! **The far end** is this test binary run again in a network namespace of its
//! own, inside a user namespace — so this needs no root and touches no
//! kernel-global state (`docs/autonomy/SHARED_MAIN.md`). **Reception** is the
//! binary run a third time in a namespace nested inside the far end's, serving
//! as `src/main.rs` does and answering on its standard output what it is asked
//! on its standard input: where the wire is, whether the person's door answers,
//! and to stop.
//!
//! # In this order
//!
//! 1. **The cable laid**, reception's end numbered 40: the far end finds reception
//!    and reaches its port.
//! 2. **Deleted and laid again at the same numbers while reception is held
//!    still.** On the far end's own side, a probe joined on its end measures what
//!    the kernel did with the membership, and `crate::responding::joined` is
//!    measured taking a join refused `EADDRINUSE` afresh. Once reception has
//!    followed the kernel, the far end's first question is answered and the port
//!    is reached.
//! 3. **Deleted and laid again at new numbers while reception is held still**:
//!    the same, at 42.
//!
//! **What is said is the same bytes throughout**: every answer is compared with
//! the first.
//!
//! # What it found
//!
//! Step 2 fails without `crate::interfaces_that_went`: the responder held to 40
//! was kept because 40 was still reported, the kernel had taken the deleted
//! interface out of the group, and reception never joined the one laid at its
//! number — so the far end never found reception again, while the port, held to
//! the same number, was reached throughout.

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::env;
    use std::fmt::Write as _;
    use std::io::{BufRead as _, BufReader, Read as _, Write as _};
    use std::net::{Ipv4Addr, SocketAddr, TcpStream, UdpSocket};
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
    use crate::responding::joined;
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
    const INSIDE: &str = "ALO_AGENTD_A_CABLE_RE_LAID_INSIDE";

    /// Reception's end of the cable.
    const RECEPTIONS_END: &str = "cable0";
    /// The far end's end of the cable.
    const FAR_END: &str = "far0";
    /// Reception's address on the cable, with its prefix.
    const RECEPTION: &str = "10.72.1.1/24";
    /// The far end's address on the cable, with its prefix.
    const THE_FAR_END: &str = "10.72.1.2/24";

    /// The numbers the cable is first laid at, reception's end first.
    const FIRST_NUMBERS: (u32, u32) = (40, 41);
    /// The numbers it is laid at when they are new.
    const NEW_NUMBERS: (u32, u32) = (42, 43);

    /// How long anything here waits for the kernel or another machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// **A cable deleted and re-laid between two readings is still found and
    /// reached**: the outer test, which makes the far end and reads what it saw
    /// and what reception's service log said.
    #[test]
    fn a_cable_deleted_and_re_laid_between_two_readings_is_still_found_and_reached() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_cable_re_laid_between_two_readings::tests::the_far_end_while_the_cable_is_re_laid";
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
            "with the cable laid it is found and reached",
            "a deleted interface's membership is not on the one laid at its number, and a join there is refused as already held",
            "a join refused as already held is taken afresh and the interface is in the group",
            "deleted and re-laid at its numbers between two readings, it is found by the first question and reached",
            "deleted and re-laid at new numbers between two readings, it is found by the first question and reached",
            "what was said was the same bytes throughout",
        ] {
            assert!(
                said.contains(step),
                "the far end never said `{step}`:\n{said}\n{log}"
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
    /// named, and say whether it did what it was asked.
    fn ip_did(pid: Option<u32>, args: &[&str]) -> Result<(), String> {
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
            Ok(())
        } else {
            Err(format!(
                "ip {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&ran.stderr)
            ))
        }
    }

    /// Run iproute2 as [`ip_did`] does, and fail with what it said if it refuses.
    fn ip(pid: Option<u32>, args: &[&str]) {
        if let Err(why) = ip_did(pid, args) {
            panic!("{why}");
        }
    }

    /// Lay the cable between here and reception, at `pid`, with reception's end
    /// and this end numbered as `numbers` says, addressed, and up.
    ///
    /// A number an interface deleted a moment ago held may not be free the
    /// instant the deletion returns, so the laying is tried again until it is.
    fn lay_the_cable(pid: u32, numbers: (u32, u32)) {
        let (receptions, far) = (numbers.0.to_string(), numbers.1.to_string());
        let pid_named = pid.to_string();
        let args = [
            "link",
            "add",
            FAR_END,
            "index",
            &far,
            "type",
            "veth",
            "peer",
            "name",
            RECEPTIONS_END,
            "index",
            &receptions,
            "netns",
            &pid_named,
        ];
        let until = Instant::now() + PATIENCE;
        while let Err(why) = ip_did(None, &args) {
            assert!(Instant::now() < until, "the cable could not be laid: {why}");
            std::thread::sleep(Duration::from_millis(50));
        }
        ip(None, &["addr", "add", THE_FAR_END, "dev", FAR_END]);
        ip(None, &["link", "set", FAR_END, "up"]);
        ip(
            Some(pid),
            &["addr", "add", RECEPTION, "dev", RECEPTIONS_END],
        );
        ip(Some(pid), &["link", "set", RECEPTIONS_END, "up"]);
    }

    /// Delete the cable — both ends go with it — and wait until neither end is
    /// reported.
    fn delete_the_cable(pid: u32) {
        ip(None, &["link", "del", FAR_END]);
        let until = Instant::now() + PATIENCE;
        while ip_did(None, &["link", "show", FAR_END]).is_ok()
            || ip_did(Some(pid), &["link", "show", RECEPTIONS_END]).is_ok()
        {
            assert!(Instant::now() < until, "the cable outlived its deletion");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// The address in `with_prefix`, without its prefix length.
    fn ip_of(with_prefix: &str) -> Ipv4Addr {
        with_prefix
            .split('/')
            .next()
            .and_then(|address| address.parse().ok())
            .unwrap()
    }

    /// Whether the interface numbered `index` is in the discovery group, as the
    /// kernel lists it in `igmp` — `/proc/net/igmp` of some network: a line
    /// naming each interface, then one line per group it is in, each the
    /// address's four bytes as the kernel holds them.
    fn in_the_group(igmp: &Path, index: u32) -> bool {
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

    /// The group memberships of this network, the far end's.
    fn here() -> PathBuf {
        PathBuf::from("/proc/self/net/igmp")
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
            let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_cable_re_laid_between_two_readings::tests::reception_while_the_cable_is_re_laid";
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

        /// Wait until the service answers, listens and is joined on its end of
        /// the cable at `number` and at no other, **and** the kernel lists that
        /// interface in the discovery group — which, in reception's network,
        /// only the service joins.
        fn until_it_follows(&mut self, number: u32) {
            let igmp = PathBuf::from(format!("/proc/{}/net/igmp", self.pid()));
            let wanted = [number.to_string(), number.to_string(), number.to_string()].join(" ");
            let until = Instant::now() + PATIENCE;
            loop {
                let wire = self.told("where");
                let joined = in_the_group(&igmp, number);
                if wire == wanted && joined {
                    return;
                }
                assert!(
                    Instant::now() < until,
                    "reception never followed its cable to {number}: answered, listened and joined at `{wire}`, and the interface {} in the discovery group",
                    if joined { "is" } else { "is not" }
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

    /// Wait until the far end hears reception's answer, and hand back its bytes.
    ///
    /// For the first laying only: a cable just come up is not joined in the
    /// instant it is running, and nothing here says when it has been.
    fn heard_at_last() -> String {
        let until = Instant::now() + PATIENCE;
        loop {
            let heard = asked();
            if heard != "nothing" {
                return heard;
            }
            assert!(Instant::now() < until, "the far end never found reception");
        }
    }

    /// The far end: lays the cable, holds reception still while it re-lays it,
    /// and asks.
    #[test]
    #[ignore = "run inside its own network by a_cable_deleted_and_re_laid_between_two_readings_is_still_found_and_reached"]
    fn the_far_end_while_the_cable_is_re_laid() {
        must_be("far end");
        let mut reception = Reception::started();
        let pid = reception.pid();

        // 1. The cable laid.
        lay_the_cable(pid, FIRST_NUMBERS);
        reception.until_it_follows(FIRST_NUMBERS.0);
        let said = heard_at_last();
        let until = Instant::now() + PATIENCE;
        while !reached() {
            assert!(
                Instant::now() < until,
                "the far end never reached reception"
            );
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(reception.door_answers());
        println!("with the cable laid it is found and reached");

        // 2. Deleted and laid again at the same numbers, while reception reads
        //    nothing — and a probe on this end measuring what the kernel does
        //    with a membership on the way.
        let far_address = ip_of(THE_FAR_END);
        let probe = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        probe.join_multicast_v4(&THE_ADDRESS, &far_address).unwrap();
        assert!(in_the_group(&here(), FIRST_NUMBERS.1));
        reception.held_still(|pid| {
            delete_the_cable(pid);
            lay_the_cable(pid, FIRST_NUMBERS);
        });
        assert!(
            !in_the_group(&here(), FIRST_NUMBERS.1),
            "the kernel put the interface laid at a deleted one's number in its group"
        );
        let refused = probe
            .join_multicast_v4(&THE_ADDRESS, &far_address)
            .expect_err("a socket joined on a deleted interface joined its successor afresh");
        assert_eq!(refused.kind(), std::io::ErrorKind::AddrInUse, "{refused}");
        println!(
            "a deleted interface's membership is not on the one laid at its number, and a join there is refused as already held"
        );
        joined(&probe, far_address).unwrap();
        assert!(
            in_the_group(&here(), FIRST_NUMBERS.1),
            "a join refused as already held was counted joined with the interface out of the group"
        );
        drop(probe);
        println!(
            "a join refused as already held is taken afresh and the interface is in the group"
        );

        reception.until_it_follows(FIRST_NUMBERS.0);
        assert_eq!(
            asked(),
            said,
            "the first question on a cable re-laid at its number went unanswered"
        );
        assert!(
            reached(),
            "the first connection on a cable re-laid at its number was not answered"
        );
        assert!(reception.door_answers());
        println!(
            "deleted and re-laid at its numbers between two readings, it is found by the first question and reached"
        );

        // 3. Deleted and laid again at new numbers, while reception reads
        //    nothing.
        reception.held_still(|pid| {
            delete_the_cable(pid);
            lay_the_cable(pid, NEW_NUMBERS);
        });
        reception.until_it_follows(NEW_NUMBERS.0);
        assert_eq!(
            asked(),
            said,
            "the first question on a cable re-laid at a new number went unanswered"
        );
        assert!(
            reached(),
            "the first connection on a cable re-laid at a new number was not answered"
        );
        assert!(reception.door_answers());
        println!(
            "deleted and re-laid at new numbers between two readings, it is found by the first question and reached"
        );
        println!("what was said was the same bytes throughout");

        reception.stopped();
    }

    /// Ask the group who is here from the far end's address, and the bytes
    /// reception answered with in hexadecimal — or `nothing` when it said nothing
    /// within a moment.
    fn asked() -> String {
        let reception = ip_of(RECEPTION);
        let socket = UdpSocket::bind((ip_of(THE_FAR_END), 0)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        let question = alo_nearby::advertising::a_question().unwrap();
        if socket.send_to(&question, (THE_ADDRESS, THE_PORT)).is_err() {
            return "nothing".to_owned();
        }
        let until = Instant::now() + Duration::from_secs(3);
        let mut heard = [0_u8; 1_500];
        while Instant::now() < until {
            if let Ok((how_many, from)) = socket.recv_from(&mut heard)
                && from.ip() == reception
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
    fn reached() -> bool {
        let at = SocketAddr::from((ip_of(RECEPTION), THE_WIRE_PORT));
        let Ok(mut connection) = TcpStream::connect_timeout(&at, Duration::from_secs(2)) else {
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

    /// The number of reception's end of the cable among `networks` over IPv4, or
    /// `-` where it is not among them.
    fn numbered(networks: &[Network]) -> String {
        networks
            .iter()
            .find(|network| network.address().is_ipv4() && network.name() == RECEPTIONS_END)
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
                .expect("a cable re-laid stopped the service");
            driving.join().unwrap();
        });
    }

    /// Reception, serving while its cable is deleted and laid again, answering
    /// the far end on its standard output.
    #[test]
    #[ignore = "run inside its own network by the_far_end_while_the_cable_is_re_laid"]
    fn reception_while_the_cable_is_re_laid() {
        must_be("reception");
        serving_as(&reception(), "a-cable-re-laid", |wire, door, stop| {
            let mut person = Talking::to(door);
            let mut line = String::new();
            loop {
                line.clear();
                let read = std::io::stdin().read_line(&mut line).unwrap();
                assert!(read > 0, "the far end ended without stopping reception");
                match line.trim_end() {
                    "where" => println!(
                        "alo:where {} {} {}",
                        numbered(&wire.answered_on()),
                        numbered(&wire.listened_on()),
                        numbered(&wire.joined()),
                    ),
                    "door" => println!("alo:door {}", if person.answers() { "yes" } else { "no" }),
                    "stop" => {
                        println!("alo:stop");
                        break;
                    }
                    said => panic!("reception was told something it does not do: {said}"),
                }
            }
            assert!(stop.stop());
        });
    }
}
