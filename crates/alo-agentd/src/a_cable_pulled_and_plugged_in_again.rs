//! A cable pulled is a network this machine is no longer found on, and one
//! plugged in is found at once — on a real kernel, with the service running
//! throughout as `src/main.rs` runs it.
//!
//! *Machines find each other with zero configuration.* Three sets of sockets
//! follow the kernel's network notifications — the IPv6 joins
//! (`crate::joining`), the port's listeners (`crate::listeners`) and discovery's
//! responders (`crate::responding`) — and each has its rule tested by handing it
//! a list. What no test measured until this one is a network that **goes** and
//! **comes back** while the service is running: the thing a person reports as
//! *it only works if I reboot after docking*.
//!
//! This test found such a bug, and it is fixed beside it: the thread that
//! answers discovery (`crate::answering_discovery`) took the responders at the
//! top of a round and slept until a question arrived on one of them. A cable
//! plugged in again takes a new responder, which that thread was not waiting on
//! — so a question on the new cable was answered only once somebody on another
//! network happened to ask something. The responders now say when they move
//! (`crate::told_of_a_move`), and the thread wakes on it.
//!
//! # The three machines
//!
//! **Reception** is this test binary run again in a network namespace of its own,
//! made by util-linux's `unshare` inside a user namespace — so this needs no
//! root, takes no kernel-global state, and every cable dies with the namespaces
//! (`docs/autonomy/SHARED_MAIN.md`).
//!
//! - At the far end of `cable0` is **the studio**, at `10.69.1.2`.
//! - At the far end of `cable1` is **the colleague**, at `10.69.2.2`.
//!
//! Each far end is the binary run a third time in a namespace of its own, and
//! does three things when told: *ask* who is here and print the answer's bytes,
//! *reach* reception's port and print whether the service answered, and *stop*.
//!
//! # In this order
//!
//! 1. **Both cables up**: reception serves; the studio and the colleague each
//!    find it and reach it.
//! 2. **A cable pulled by its far end going away**: the colleague stops, its
//!    namespace ends, and `cable1` with it. Reception answers, listens and is
//!    joined on `cable0` and not on `cable1`, the studio still finds and reaches
//!    it, and the person's door still answers.
//! 3. **The same cable plugged in again**: a new colleague, a new `cable1` — with
//!    a new index, as a cable re-laid really is — and without the service
//!    restarting the colleague finds and reaches reception.
//! 4. **A cable pulled by its link going down**: `cable0` set down on reception's
//!    side. Reception is found and reached on `cable1` and nowhere on `cable0`.
//! 5. **A failure on the way back**: somebody else holds the port on `cable0`
//!    when it comes up. Discovery is answered there, the port is not listened on,
//!    and that is a line in the service log — the service still answers the door
//!    and the colleague. Once somebody else lets go and the cable is re-seated,
//!    the studio finds and reaches reception again.
//!
//! **What is said is the same bytes in every step**: every answer either far end
//! heard is compared with the first.

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
    use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
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
    use crate::route_messages::reported_by_the_kernel;
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
    const INSIDE: &str = "ALO_AGENTD_A_CABLE_PULLED_INSIDE";

    /// Reception's cable to the studio.
    const THE_STUDIOS_CABLE: &str = "cable0";
    /// Reception's cable to the colleague.
    const THE_COLLEAGUES_CABLE: &str = "cable1";
    /// The far end of either cable.
    const FAR_END: &str = "far0";

    /// Reception's end of each cable, and the machine at the far end of it.
    const RECEPTION_TO_THE_STUDIO: &str = "10.69.1.1/24";
    const THE_STUDIO: &str = "10.69.1.2/24";
    const RECEPTION_TO_THE_COLLEAGUE: &str = "10.69.2.1/24";
    const THE_COLLEAGUE: &str = "10.69.2.2/24";

    /// How long anything here waits for the kernel or another machine.
    const PATIENCE: Duration = Duration::from_secs(30);

    /// What the service log says when the port will not bind on a network.
    const THE_PORT_WOULD_NOT_BIND_ON_THE_STUDIOS_CABLE: &str =
        "the port presence advertises could not be bound on cable0";

    /// **A cable pulled is a network this machine is no longer found on, and one
    /// plugged in is found at once**: the outer test, which makes reception and
    /// reads what it saw and what its service log said.
    #[test]
    fn a_cable_pulled_is_a_network_no_longer_found_on_and_one_plugged_in_is_found_at_once() {
        let exe = env::current_exe().expect("a test binary knows where it is");
        let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_cable_pulled_and_plugged_in_again::tests::reception_while_cables_come_and_go";
        let ran = Command::new("unshare")
            .args(["--map-root-user", "--net", "--", "sh", "-c", script])
            .arg(exe)
            .env(INSIDE, "reception")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("util-linux's `unshare` makes the machines");

        let said = String::from_utf8_lossy(&ran.stdout).into_owned();
        let log = String::from_utf8_lossy(&ran.stderr).into_owned();
        assert!(
            ran.status.success(),
            "reception failed ({}):\n{said}\n{log}",
            ran.status
        );
        for step in [
            "with both cables up it is found and reached on both",
            "a cable pulled by its far end going away leaves it found and reached on the other alone",
            "the same cable plugged in again is found and reached without the service restarting",
            "a cable pulled by its link going down leaves it found and reached on the other alone",
            "a network the port would not bind on left the service answering",
            "the cable re-seated is found and reached again",
            "what was said was the same bytes throughout",
        ] {
            assert!(
                said.contains(step),
                "reception never said `{step}`:\n{said}\n{log}"
            );
        }
        assert!(
            log.contains(THE_PORT_WOULD_NOT_BIND_ON_THE_STUDIOS_CABLE),
            "the network that would not bind is not a line in the service log:\n{log}"
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

    /// A machine at the far end of a cable.
    struct FarEnd {
        /// Its process, whose network namespace is the far end of the cable.
        process: Child,
        /// What it is told on.
        telling: ChildStdin,
        /// What it says on.
        hearing: BufReader<ChildStdout>,
    }

    impl FarEnd {
        /// A far end in a network namespace nested inside reception's, at
        /// `address`, with `cable` laid to it and reception's end at `here`.
        fn laid(which: &str, cable: &str, here: &str, address: &str) -> Self {
            let exe = env::current_exe().unwrap();
            let script = "ip link set lo up && exec \"$0\" --exact --ignored --nocapture a_cable_pulled_and_plugged_in_again::tests::a_machine_at_the_far_end";
            let mut process = Command::new("unshare")
                .args(["--net", "--", "sh", "-c", script])
                .arg(exe)
                .env(INSIDE, "far end")
                .env("ALO_AGENTD_FAR_END_ADDRESS", ip_of(address))
                .env("ALO_AGENTD_RECEPTION_ADDRESS", ip_of(here))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("util-linux's `unshare` makes the far end");
            let ours = std::fs::read_link("/proc/self/ns/net").unwrap();
            let theirs = format!("/proc/{}/ns/net", process.id());
            let until = Instant::now() + Duration::from_secs(10);
            while std::fs::read_link(&theirs).ok().as_ref() == Some(&ours)
                || !Path::new(&theirs).exists()
            {
                assert!(Instant::now() < until, "{which} never left this network");
                std::thread::sleep(Duration::from_millis(20));
            }
            let pid = process.id().to_string();
            ip(
                None,
                &[
                    "link", "add", cable, "type", "veth", "peer", "name", FAR_END, "netns", &pid,
                ],
            );
            ip(
                Some(process.id()),
                &["addr", "add", address, "dev", FAR_END],
            );
            ip(Some(process.id()), &["link", "set", FAR_END, "up"]);
            ip(None, &["addr", "add", here, "dev", cable]);
            ip(None, &["link", "set", cable, "up"]);
            let telling = process.stdin.take().unwrap();
            let hearing = BufReader::new(process.stdout.take().unwrap());
            Self {
                process,
                telling,
                hearing,
            }
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
                assert!(read > 0, "a far end ended before answering `{what}`");
                if let Some(rest) = line.trim_end().strip_prefix(&prefix) {
                    return rest.trim().to_owned();
                }
            }
        }

        /// What it heard when it asked who is here: the bytes of reception's
        /// answer, or `nothing`.
        fn asks(&mut self) -> String {
            self.told("ask")
        }

        /// Whether it reached the service on reception's port.
        fn reaches(&mut self) -> bool {
            self.told("reach") == "yes"
        }

        /// Stop it, and wait for its namespace — and the cable — to end with it.
        fn stopped(mut self) {
            assert_eq!(self.told("stop"), "");
            drop(self.telling);
            assert!(self.process.wait().unwrap().success(), "a far end failed");
        }
    }

    /// The address in `with_prefix`, without its prefix length.
    fn ip_of(with_prefix: &str) -> &str {
        with_prefix.split('/').next().unwrap_or(with_prefix)
    }

    /// The names of `networks`, over IPv4.
    fn named(networks: &[Network]) -> Vec<String> {
        networks
            .iter()
            .filter(|network| network.address().is_ipv4())
            .map(|network| network.name().to_owned())
            .collect()
    }

    /// Where the wire answers discovery, listens and is joined over IPv4, by
    /// name.
    fn where_the_wire_is(wire: &Wire) -> [Vec<String>; 3] {
        [
            named(&wire.answered_on()),
            named(&wire.listened_on()),
            named(&wire.joined()),
        ]
    }

    /// Wait until the wire answers, listens and is joined on `cable` exactly
    /// when `on` says so — the service following the kernel, measured from
    /// outside it.
    fn until_the_wire_follows(wire: &Wire, cable: &str, on: [bool; 3]) {
        let until = Instant::now() + PATIENCE;
        loop {
            let now = where_the_wire_is(wire);
            let is: Vec<bool> = now
                .iter()
                .map(|names| names.iter().any(|name| name == cable))
                .collect();
            if is == on {
                return;
            }
            assert!(
                Instant::now() < until,
                "the wire never followed {cable} to {on:?}: answered, listened, joined = {now:?}"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Wait until the kernel reports no interface called `cable`: a far end's
    /// namespace is torn down a moment after its last process ends.
    fn until_the_kernel_has_no(cable: &str) {
        let until = Instant::now() + PATIENCE;
        while reported_by_the_kernel()
            .unwrap()
            .iter()
            .any(|interface| interface.name == cable)
        {
            assert!(
                Instant::now() < until,
                "{cable} outlived the namespace at its far end"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// The kernel's number for the interface called `cable`.
    fn index_of(cable: &str) -> u32 {
        reported_by_the_kernel()
            .unwrap()
            .iter()
            .find(|interface| interface.name == cable)
            .map(|interface| interface.index)
            .unwrap_or_else(|| panic!("the kernel reports no {cable}"))
    }

    /// Wait until `far` hears reception's answer, and hand back its bytes.
    ///
    /// A cable just come up is not joined in the instant it is running, so the
    /// first question may go unanswered; a far end that never hears within
    /// [`PATIENCE`] is the failure.
    fn heard_by(far: &mut FarEnd, who: &str) -> String {
        let until = Instant::now() + PATIENCE;
        loop {
            let heard = far.asks();
            if heard != "nothing" {
                return heard;
            }
            assert!(Instant::now() < until, "{who} never found reception");
        }
    }

    /// Wait until `far` reaches the service on reception's port.
    fn reached_by(far: &mut FarEnd, who: &str) {
        let until = Instant::now() + PATIENCE;
        while !far.reaches() {
            assert!(Instant::now() < until, "{who} never reached reception");
            std::thread::sleep(Duration::from_millis(100));
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
                .expect("a cable coming or going stopped the service");
            driving.join().unwrap();
        });
    }

    /// A listener somebody else holds at the port on the interface numbered
    /// `index`, which is what makes a network refuse the service's bind.
    fn somebody_else_holding_the_port_on(index: u32) -> TcpListener {
        let socket =
            socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None).unwrap();
        socket
            .bind_device_by_index_v4(NonZeroU32::new(index))
            .unwrap();
        socket.set_reuse_address(true).unwrap();
        let at: SocketAddr = (Ipv4Addr::UNSPECIFIED, THE_WIRE_PORT).into();
        socket.bind(&at.into()).unwrap();
        socket.listen(8).unwrap();
        socket.into()
    }

    /// Reception, while its cables come and go: the whole of it.
    #[test]
    #[ignore = "run inside its own network by a_cable_pulled_is_a_network_no_longer_found_on_and_one_plugged_in_is_found_at_once"]
    fn reception_while_cables_come_and_go() {
        must_be("reception");
        let mut studio = FarEnd::laid(
            "the studio",
            THE_STUDIOS_CABLE,
            RECEPTION_TO_THE_STUDIO,
            THE_STUDIO,
        );
        let mut colleague = Some(FarEnd::laid(
            "the colleague",
            THE_COLLEAGUES_CABLE,
            RECEPTION_TO_THE_COLLEAGUE,
            THE_COLLEAGUE,
        ));

        serving_as(&reception(), "a-cable-pulled", |wire, door, stop| {
            let mut person = Talking::to(door);
            let everything = [true, true, true];
            let nothing = [false, false, false];

            // 1. Both cables up.
            until_the_wire_follows(wire, THE_STUDIOS_CABLE, everything);
            until_the_wire_follows(wire, THE_COLLEAGUES_CABLE, everything);
            let said = heard_by(&mut studio, "the studio");
            {
                let colleague = colleague.as_mut().unwrap();
                assert_eq!(heard_by(colleague, "the colleague"), said);
                reached_by(colleague, "the colleague");
            }
            reached_by(&mut studio, "the studio");
            assert!(person.answers());
            println!("with both cables up it is found and reached on both");

            // 2. The colleague's cable pulled: its far end goes away, and the
            //    cable with it.
            colleague.take().unwrap().stopped();
            until_the_kernel_has_no(THE_COLLEAGUES_CABLE);
            until_the_wire_follows(wire, THE_COLLEAGUES_CABLE, nothing);
            until_the_wire_follows(wire, THE_STUDIOS_CABLE, everything);
            assert_eq!(studio.asks(), said);
            assert!(studio.reaches());
            assert!(person.answers(), "the person's door stopped answering");
            println!(
                "a cable pulled by its far end going away leaves it found and reached on the other alone"
            );

            // 3. And plugged in again, with a new index, as a cable re-laid is.
            let mut colleague = FarEnd::laid(
                "the colleague, again",
                THE_COLLEAGUES_CABLE,
                RECEPTION_TO_THE_COLLEAGUE,
                THE_COLLEAGUE,
            );
            until_the_wire_follows(wire, THE_COLLEAGUES_CABLE, everything);
            // Found **at once**: nobody else asks anything in between, so an
            // answering thread still asleep on the sockets it had before the
            // cable came back is a colleague who never hears. And the **first**
            // question once the wire has followed the kernel is answered: the
            // socket is bound and joined before it is counted as answered on, so
            // a question asked the moment after waits on it and is read.
            assert_eq!(
                colleague.asks(),
                said,
                "the first question on a cable plugged in again went unanswered"
            );
            assert!(
                colleague.reaches(),
                "the first connection on a cable plugged in again was not answered"
            );
            assert_eq!(studio.asks(), said);
            assert!(studio.reaches());
            assert!(person.answers());
            println!(
                "the same cable plugged in again is found and reached without the service restarting"
            );

            // 4. The studio's cable pulled at this end: the link set down.
            ip(None, &["link", "set", THE_STUDIOS_CABLE, "down"]);
            until_the_wire_follows(wire, THE_STUDIOS_CABLE, nothing);
            assert_eq!(studio.asks(), "nothing", "found on a cable that was pulled");
            assert!(!studio.reaches(), "reached on a cable that was pulled");
            assert_eq!(colleague.asks(), said);
            assert!(colleague.reaches());
            assert!(person.answers());
            println!(
                "a cable pulled by its link going down leaves it found and reached on the other alone"
            );

            // 5. Somebody else holds the port on the studio's cable when it
            //    comes back: discovery is answered there, the port is not
            //    listened on, and the service goes on.
            let somebody_else = somebody_else_holding_the_port_on(index_of(THE_STUDIOS_CABLE));
            ip(None, &["link", "set", THE_STUDIOS_CABLE, "up"]);
            until_the_wire_follows(wire, THE_STUDIOS_CABLE, [true, false, true]);
            assert_eq!(heard_by(&mut studio, "the studio"), said);
            assert!(
                !named(&wire.listened_on())
                    .iter()
                    .any(|name| name == THE_STUDIOS_CABLE),
                "the port was listened on where somebody else held it"
            );
            assert_eq!(colleague.asks(), said);
            assert!(colleague.reaches());
            assert!(
                person.answers(),
                "a network that would not bind stopped the door"
            );
            println!("a network the port would not bind on left the service answering");

            // Somebody else lets go, and the cable is re-seated.
            drop(somebody_else);
            ip(None, &["link", "set", THE_STUDIOS_CABLE, "down"]);
            until_the_wire_follows(wire, THE_STUDIOS_CABLE, nothing);
            ip(None, &["link", "set", THE_STUDIOS_CABLE, "up"]);
            until_the_wire_follows(wire, THE_STUDIOS_CABLE, everything);
            assert_eq!(heard_by(&mut studio, "the studio"), said);
            reached_by(&mut studio, "the studio");
            assert_eq!(colleague.asks(), said);
            assert!(person.answers());
            println!("the cable re-seated is found and reached again");
            println!("what was said was the same bytes throughout");

            colleague.stopped();
            assert!(stop.stop());
        });
        studio.stopped();
    }

    /// A machine at the far end of a cable, doing what it is told: asking who is
    /// here, reaching reception's port, and stopping.
    #[test]
    #[ignore = "run inside its own network by reception_while_cables_come_and_go"]
    fn a_machine_at_the_far_end() {
        must_be("far end");
        let here: Ipv4Addr = env::var("ALO_AGENTD_FAR_END_ADDRESS")
            .unwrap()
            .parse()
            .unwrap();
        let reception: Ipv4Addr = env::var("ALO_AGENTD_RECEPTION_ADDRESS")
            .unwrap()
            .parse()
            .unwrap();
        let mut line = String::new();
        loop {
            line.clear();
            let read = std::io::stdin().read_line(&mut line).unwrap();
            assert!(read > 0, "reception ended without stopping this machine");
            match line.trim_end() {
                "ask" => println!("alo:ask {}", asked(here, reception)),
                "reach" => println!(
                    "alo:reach {}",
                    if reached(reception) { "yes" } else { "no" }
                ),
                "stop" => {
                    println!("alo:stop");
                    return;
                }
                said => panic!("a far end was told something it does not do: {said}"),
            }
        }
    }

    /// Ask the group who is here from `here`, and the bytes `reception` answered
    /// with in hexadecimal — or `nothing` when it said nothing within a moment.
    ///
    /// The answer's bytes rather than what they parse to, so that *the same
    /// thing is said* is compared byte for byte.
    fn asked(here: Ipv4Addr, reception: Ipv4Addr) -> String {
        let socket = UdpSocket::bind((here, 0)).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        let question = alo_nearby::advertising::a_question().unwrap();
        // A cable whose link is down refuses the send; that is nothing heard.
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

    /// Whether the service on `reception`'s port answered a message on it — a
    /// handshake completed **and** the wire's own refusal read back, so a port
    /// held by anybody else is not counted as reception reached.
    fn reached(reception: Ipv4Addr) -> bool {
        let at = SocketAddr::from((reception, THE_WIRE_PORT));
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
}
