//! A question this service puts to a provider, inside a boundary the kernel
//! imposes, on a running machine with the real programme loaded.
//!
//! `a_turn_is_bounded_by_the_kernel.rs` is this file's sibling and covers what a
//! turn does to a disk. This is law 1's other half: **nothing leaves that the
//! person was not shown**, refused by the machine rather than promised by the
//! daemon. ADR 0020 is the decision; this is whether it is true.
//!
//! # Two addresses, because the two halves are different claims
//!
//! Loopback is deliberately unchecked (ADR 0007), so what a request to
//! `127.0.0.1` proves is that **the production path runs inside the boundary and
//! still works** — resolve, register, enter, connect, answer — and not that the
//! kernel permitted anything. `Provider::checked` permits unencrypted `http://`
//! only to this machine, which is why the test that reads a whole answer is the
//! loopback one.
//!
//! What the kernel decides about is a destination that is **not** loopback, so
//! the other tests bind **this machine's own address on `eth0`** and the
//! connection really goes there: a registered address is reached, an
//! unregistered one is refused with `EACCES`, and a provider at this machine's
//! own address is connected to *through the production path* — the connection
//! being accepted is the kernel having permitted what the request registered.
//!
//! Between them: the path works, and the machine is deciding. Neither test is
//! asked to prove the other's half.
//!
//! Everything is answered inside this virtual machine and reaches no network: it
//! is the machine talking to itself over a real interface. Nothing here changes
//! host-wide networking — it binds a port the operating system chooses, on an
//! address the machine already has, and gives it back.
//!
//! # One at a time, because a turn subtree is this whole process's
//!
//! `alo_bounding::Turns::of_this_service` makes one control group beside where
//! *this process* is and moves the process into it, so two of them in one test
//! binary are two attempts at the same directory. These tests take a lock and
//! give their subtree back inside it, which is the same arrangement a real
//! machine has: one service, one subtree.
//!
//! # It needs root, a BPF filesystem, and a kernel that started the BPF LSM
//!
//! The same as its sibling, for the same reasons, and it fails loudly on a
//! machine without them rather than skipping itself.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::io::{BufRead as _, Read as _, Write as _};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, SystemTime};

use alo_agentd::{ByTheKernel, starting};
use alo_answering::Answering;
use alo_asking::Hosted;
use alo_bounding::{Imposed, Pinned};
use alo_capability::Grants;
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::OnThisMachine;
use alo_keeping::Writing;
use alo_models::{InferenceSource, Provider, Region, Secret, SourcePolicy};
use alo_turn::{Answers, Bounding, Machine, Places, Turning};

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long this turn lasts.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// What the canned provider answers with.
const AN_ANSWER: &str =
    r#"{"choices":[{"message":{"content":"No, not without written consent."}}]}"#;

/// This machine's own address on the interface it has, which is not loopback.
///
/// Not loopback because loopback is not checked, and not a documentation range
/// because a test that proves a request *works* needs something that answers.
/// Everything here stays inside this machine.
fn our_own_address() -> std::net::IpAddr {
    // Connecting a datagram socket to somewhere unreachable picks the interface
    // the machine would use and tells us its address, without sending anything.
    let asking = UdpSocket::bind("0.0.0.0:0").expect("a socket of our own");
    asking
        .connect("192.0.2.1:9")
        .expect("a route to somewhere can be chosen");
    asking
        .local_addr()
        .expect("a socket knows its own address")
        .ip()
}

/// Held for the length of a test that makes a subtree of this process's own.
///
/// Poisoning is stepped over deliberately: a panicking test leaves nothing
/// behind that a later one reads, and turning one failure into three would hide
/// which test actually broke.
///
/// The second half is `alo_bounding::Waited`, which keeps **the other
/// checkout** out — the same machine runs two of them, and before this existed
/// a gate run here failed all five of these tests because the other one was
/// running its own suite at that moment.
fn the_only_one_on_this_machine() -> (MutexGuard<'static, ()>, alo_bounding::Waited) {
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    let ours = ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner);
    // This binary first, then the machine — `alo_bounding::waiting` says why
    // that order and not the other. The second one is what keeps the other
    // checkout's kernel tests out while these run.
    let kernel = alo_bounding::Waited::on_this_kernel()
        .expect("this kernel can be taken, and nothing is forced if it cannot");
    (ours, kernel)
}

/// A socket that accepts one connection and says nothing.
///
/// For the tests where the question is *did the connection happen*, not what
/// came back over it: a request whose handshake cannot complete still had to be
/// permitted by the kernel to get as far as being accepted here.
fn accepting() -> (SocketAddr, thread::JoinHandle<bool>) {
    let listener =
        TcpListener::bind(SocketAddr::new(our_own_address(), 0)).expect("a listener of our own");
    let at = listener.local_addr().expect("a listener knows where it is");
    listener
        .set_nonblocking(true)
        .expect("a listener can be made not to wait");
    let handle = thread::spawn(move || {
        let until = std::time::Instant::now() + Duration::from_secs(10);
        while std::time::Instant::now() < until {
            if listener.accept().is_ok() {
                return true;
            }
            thread::sleep(Duration::from_millis(20));
        }
        false
    });
    (at, handle)
}

/// One request, one canned reply, on a real socket bound to a real interface.
///
/// Answers with what it was sent, so a test can assert nothing was sent at all.
fn serving() -> (SocketAddr, thread::JoinHandle<Option<String>>) {
    serving_at(our_own_address())
}

/// The same, somewhere a caller names.
fn serving_at(address: std::net::IpAddr) -> (SocketAddr, thread::JoinHandle<Option<String>>) {
    let listener = TcpListener::bind(SocketAddr::new(address, 0)).expect("a listener of our own");
    let at = listener.local_addr().expect("a listener knows where it is");
    listener
        .set_nonblocking(false)
        .expect("a listener can be made to wait");
    let handle = thread::spawn(move || {
        // A short wait, because one of these tests is about nothing arriving.
        listener
            .set_nonblocking(true)
            .expect("a listener can be made not to wait");
        let until = std::time::Instant::now() + Duration::from_secs(10);
        let mut accepted = None;
        while std::time::Instant::now() < until {
            match listener.accept() {
                Ok((stream, _)) => {
                    accepted = Some(stream);
                    break;
                }
                Err(_) => thread::sleep(Duration::from_millis(20)),
            }
        }
        let mut stream = accepted?;
        let mut reader = std::io::BufReader::new(stream.try_clone().ok()?);
        let mut head = String::new();
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).ok()? == 0 {
                break;
            }
            if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = value.trim().parse().unwrap_or(0);
            }
            let done = line == "\r\n" || line == "\n";
            head.push_str(&line);
            if done {
                break;
            }
        }
        let mut body = vec![0u8; length];
        if length > 0 {
            reader.read_exact(&mut body).ok()?;
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{AN_ANSWER}",
            AN_ANSWER.len()
        );
        stream.write_all(response.as_bytes()).ok()?;
        stream.flush().ok()?;
        Some(head + &String::from_utf8_lossy(&body))
    });
    (at, handle)
}

/// A boundary of this test's own, loaded and pinned where nothing else is.
struct AsAMachineHasIt {
    /// Where it is pinned, which is never the real machine's place.
    pinned: Pinned,

    /// The loader's handles, which hold nothing once the pins are made.
    _loaded: Imposed,
}

impl AsAMachineHasIt {
    /// Impose one, and fail loudly on a machine that cannot take it.
    fn on_this_kernel(what: &str) -> Self {
        let pinned = Pinned::beneath(
            &PathBuf::from("/sys/fs/bpf")
                .join(format!("alo-question-{}-{what}", std::process::id())),
        );
        pinned.taken_away();
        pinned
            .made()
            .expect("this machine has a BPF filesystem at /sys/fs/bpf");
        let loaded = Imposed::once(&pinned).unwrap_or_else(|why| {
            panic!("no boundary could be imposed, so nothing here is being tested: {why}")
        });
        Self {
            pinned,
            _loaded: loaded,
        }
    }
}

impl Drop for AsAMachineHasIt {
    fn drop(&mut self) {
        self.pinned.taken_away();
    }
}

/// Somewhere of this test's own for the record and the strings.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let at = PathBuf::from("/tmp").join(format!(
        "alo-question-bounded-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&at);
    fs::create_dir_all(&at).expect("a temporary directory can be made");
    at
}

/// Where an answer says it came from.
fn mistral() -> InferenceSource {
    InferenceSource::Hosted {
        provider: "Mistral".to_owned(),
        region: Region::Declared("the EU".to_owned()),
    }
}

/// **A provider-style request, through the production path, under the real
/// boundary — and it works.**
///
/// The whole point of the integration: `Turning::asking` resolves the endpoint,
/// registers what it resolved, enters a control group, makes the request from
/// inside it, and comes back with the answer. Nothing about the shape of the
/// call changed; what changed is that the machine was refusing everywhere else
/// while it happened.
///
/// The request really crosses a socket, and what the server received is
/// asserted, so this is not a mock agreeing with itself.
///
/// **What this half does not prove** is that the kernel permitted anything: the
/// endpoint is loopback and loopback is unchecked (ADR 0007).
/// `a_provider_at_an_address_the_kernel_decides_about_is_connected_to` is the
/// other half, and it is a destination the machine really rules on.
#[test]
fn a_question_this_service_puts_to_a_provider_answers_inside_the_boundary() {
    let _one = the_only_one_on_this_machine();
    let kernel = AsAMachineHasIt::on_this_kernel("answers");
    let folder = a_folder_of_our_own("answers");
    // Loopback, because this is the test that reads a whole answer back and
    // `Provider::checked` carries a key over `http://` only to this machine. The
    // kernel's decision about a destination is the *next* test's half.
    let (at, server) = serving_at(std::net::IpAddr::from([127, 0, 0, 1]));

    let strings = starting::what_this_machine_says()
        .expect("this machine's own words")
        .into_strings();
    let mut writing =
        Writing::opening(&folder.join("record.jsonl")).expect("a record can be opened");
    let mut indicator = Indicator::default();
    let mut grants = Grants::default();
    let mut bounding =
        ByTheKernel::beneath(&kernel.pinned).expect("a service can open the map a loader pinned");

    let provider = Provider::checked(
        "Mistral",
        &format!("http://{at}"),
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider at an address of our own");
    let key = Secret::typed("sk-live-0123456789").expect("a key");

    let answer = {
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut writing,
        )
        .expect("the six declare");
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@mail",
            hour(),
            &mut grants,
            &mut machine,
        )
        .expect("a turn can begin");

        let permitted = Answering::chosen(mistral(), &SourcePolicy::Anywhere).expect("permitted");
        let answer = turning
            .asking(
                "may the tenant sublet?",
                "mistral-small-latest",
                permitted,
                &Answers::Provider(Hosted::provider(&provider, Some(&key))),
                &Places::under(&SourcePolicy::Anywhere),
                noon(),
            )
            .expect("the question was put and answered");
        let _ = turning.ending(&mut grants);
        answer
    };

    assert_eq!(answer.text(), "No, not without written consent.");
    assert_eq!(answer.source(), &mistral());

    let sent = server
        .join()
        .expect("the server thread finishes")
        .expect("the request really reached the server");
    assert!(sent.contains("may the tenant sublet?"), "{sent}");
    assert!(
        sent.to_ascii_lowercase().contains("authorization:"),
        "the key did not travel with the request"
    );

    bounding.given_back().expect("the subtree is given back");
    let _ = fs::remove_dir_all(&folder);
}

/// **The production path reaches a destination the kernel decides about.**
///
/// The other half of the pair above. The endpoint is `https://` at this
/// machine's own address on `eth0` — not loopback, so `socket_connect` really
/// rules on it — and the whole production path runs: the daemon resolves it,
/// registers what it resolved, enters a control group and connects from inside.
///
/// **The connection being accepted is the assertion.** The server speaks no TLS,
/// so the handshake fails and the question is not answered; that is expected and
/// is not what is being measured. A kernel that had refused the destination would
/// have failed the `connect` and the server would have seen nothing at all — so
/// *the server saw a connection* is the registration having been honoured on the
/// real path, which no `socket_connect` test on its own can say.
#[test]
fn a_provider_at_an_address_the_kernel_decides_about_is_connected_to() {
    let _one = the_only_one_on_this_machine();
    let kernel = AsAMachineHasIt::on_this_kernel("elsewhere");
    let folder = a_folder_of_our_own("elsewhere");
    let (at, server) = accepting();

    let strings = starting::what_this_machine_says()
        .expect("this machine's own words")
        .into_strings();
    let mut writing =
        Writing::opening(&folder.join("record.jsonl")).expect("a record can be opened");
    let mut indicator = Indicator::default();
    let mut grants = Grants::default();
    let mut bounding =
        ByTheKernel::beneath(&kernel.pinned).expect("a service can open the map a loader pinned");

    let provider = Provider::checked(
        "Mistral",
        &format!("https://{at}"),
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider at an address of our own");
    let key = Secret::typed("sk-live-0123456789").expect("a key");

    {
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut writing,
        )
        .expect("the six declare");
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@mail",
            hour(),
            &mut grants,
            &mut machine,
        )
        .expect("a turn can begin");

        let permitted = Answering::chosen(mistral(), &SourcePolicy::Anywhere).expect("permitted");
        let outcome = turning.asking(
            "may the tenant sublet?",
            "mistral-small-latest",
            permitted,
            &Answers::Provider(Hosted::provider(&provider, Some(&key))),
            &Places::under(&SourcePolicy::Anywhere),
            noon(),
        );
        assert!(
            outcome.is_err(),
            "a server that speaks no TLS answered a question: {outcome:?}"
        );
        let _ = turning.ending(&mut grants);
    }

    assert!(
        server.join().expect("the server thread finishes"),
        "the production request never reached this machine's own address, so the          destination it registered was not the one the kernel permitted"
    );

    bounding.given_back().expect("the subtree is given back");
    let _ = fs::remove_dir_all(&folder);
}

/// **An address nobody registered is refused while the registered one works**,
/// through the same `Bounding` the service uses.
///
/// The production path cannot itself attempt an unregistered address — the
/// registration is derived from the endpoint it is about to reach, which is the
/// design — so this asks the layer underneath directly: inside a boundary
/// carrying one address, that address answers and another does not.
///
/// Two servers, both on this machine's own interface, and only one of them
/// registered.
#[test]
fn inside_a_bounded_request_only_the_registered_address_can_be_reached() {
    let _one = the_only_one_on_this_machine();
    let kernel = AsAMachineHasIt::on_this_kernel("registered");
    let mut bounding =
        ByTheKernel::beneath(&kernel.pinned).expect("a service can open the map a loader pinned");

    let (registered, one) = serving();
    let (unregistered, other) = serving();

    let mut reached = None;
    let mut refused = None;
    bounding
        .carrying_out_a_departure(&[registered], &mut || {
            reached = Some(
                TcpStream::connect_timeout(&registered, Duration::from_secs(3))
                    .map_err(|why| why.raw_os_error().unwrap_or(0)),
            );
            refused = Some(
                TcpStream::connect_timeout(&unregistered, Duration::from_secs(3))
                    .map_err(|why| why.raw_os_error().unwrap_or(0)),
            );
        })
        .expect("a boundary can be put around a request");

    assert!(
        matches!(reached, Some(Ok(_))),
        "the registered address was not reachable from inside its own boundary: {reached:?}"
    );
    assert_eq!(
        refused.map(Result::err),
        Some(Some(13)),
        "an address nobody registered was reachable, so a departure is permission for \
         more than it showed"
    );

    // Both servers are let go; one was never connected to.
    drop(one);
    drop(other);
    bounding.given_back().expect("the subtree is given back");
}

/// **A rule that forbids it sends nothing**, and the server is the witness.
///
/// The policy is asked before anything is registered or bounded, so a question
/// the person's own rule refuses never reaches a socket at all. What makes this
/// worth a kernel test rather than a unit one is that it is asserted from the
/// other end of a real connection that never happened.
#[test]
fn a_question_the_rule_refuses_never_reaches_the_server() {
    let _one = the_only_one_on_this_machine();
    let kernel = AsAMachineHasIt::on_this_kernel("refused");
    let folder = a_folder_of_our_own("refused");
    let (at, server) = serving();

    let strings = starting::what_this_machine_says()
        .expect("this machine's own words")
        .into_strings();
    let mut writing =
        Writing::opening(&folder.join("record.jsonl")).expect("a record can be opened");
    let mut indicator = Indicator::default();
    let mut grants = Grants::default();
    let mut bounding =
        ByTheKernel::beneath(&kernel.pinned).expect("a service can open the map a loader pinned");

    let provider = Provider::checked(
        "Mistral",
        &format!("https://{at}"),
        Region::Declared("the EU".to_owned()),
        None,
    )
    .expect("a provider at an address of our own");

    {
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut writing,
        )
        .expect("the six declare");
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@mail",
            hour(),
            &mut grants,
            &mut machine,
        )
        .expect("a turn can begin");

        // The person's machine keeps its questions on it.
        let only_here = SourcePolicy::ThisMachineOnly;
        let permitted = Answering::chosen(mistral(), &SourcePolicy::Anywhere).expect("permitted");
        let refused = turning
            .asking(
                "may the tenant sublet?",
                "mistral-small-latest",
                permitted,
                &Answers::Provider(Hosted::provider(&provider, None)),
                &Places::under(&only_here),
                noon(),
            )
            .expect_err("a rule that forbids it does not answer");
        assert!(
            refused.nothing_left(),
            "a refused question was recorded as having left: {refused:?}"
        );
        let _ = turning.ending(&mut grants);
    }

    assert!(
        server.join().expect("the server thread finishes").is_none(),
        "a question the rule refused still reached the server"
    );
    assert!(
        indicator.is_quiet(),
        "the indicator was left showing egress"
    );

    bounding.given_back().expect("the subtree is given back");
    let _ = fs::remove_dir_all(&folder);
}

/// **A request that fails leaves no permission behind**, and the next one is
/// bounded on its own terms.
///
/// The control group and its entry are made and taken away by the same call
/// that carries the request, so a failure inside cannot leave a turn registered
/// — and the kernel is asked afterwards rather than trusted about it.
#[test]
fn a_request_that_fails_leaves_the_kernel_holding_nothing() {
    let _one = the_only_one_on_this_machine();
    let kernel = AsAMachineHasIt::on_this_kernel("cleanup");
    let mut bounding =
        ByTheKernel::beneath(&kernel.pinned).expect("a service can open the map a loader pinned");

    // Nothing listens here, so the work inside fails. The boundary is still
    // entered, and still has to be left.
    let nowhere = SocketAddr::new(our_own_address(), 9);
    let mut what_happened = None;
    bounding
        .carrying_out_a_departure(&[nowhere], &mut || {
            what_happened = Some(
                TcpStream::connect_timeout(&nowhere, Duration::from_secs(2))
                    .map_err(|why| why.raw_os_error().unwrap_or(0)),
            );
        })
        .expect("a boundary can be put around a request that fails");

    assert!(
        matches!(what_happened, Some(Err(_))),
        "something answered where nothing listens: {what_happened:?}"
    );

    // **And no permission survived it.** A second request, registering a
    // different address, must be refused the first one — which is the kernel
    // being asked rather than this test believing that a control group was
    // taken away.
    let (elsewhere, server) = serving();
    let mut afterwards = None;
    bounding
        .carrying_out_a_departure(&[elsewhere], &mut || {
            afterwards = Some(
                TcpStream::connect_timeout(&nowhere, Duration::from_secs(2))
                    .map_err(|why| why.raw_os_error().unwrap_or(0)),
            );
        })
        .expect("a second boundary can be put around a second request");
    assert_eq!(
        afterwards.map(Result::err),
        Some(Some(13)),
        "the first request's address was still permitted during the second, so a          permission outlived the request it belonged to"
    );
    drop(server);

    bounding.given_back().expect("the subtree is given back");
}
