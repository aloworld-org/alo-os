//! What a bound turn can reach on the network, measured rather than assumed.
//!
//! The boundary watches four things a turn does to files. It watches **nothing**
//! a turn does to a socket, and `crates/alo-bounding/src/lib.rs` states the
//! policy that will close it:
//!
//! > A turn opens no socket unless the person has been shown that it is about
//! > to.
//!
//! This file is that gap, reproduced the way the rename and the delete gaps
//! were: with a turn bound on a running kernel and the real programme loaded,
//! before anything is built to close it. Its assertions are what this machine
//! does **today**, so the day the programme lands they fail and whoever landed
//! it has to come here and say what changed.
//!
//! # Nothing here reaches the network
//!
//! The address the child connects to is a listener **this test owns**, on
//! loopback, on a port the operating system chose. Nothing resolves a name,
//! nothing leaves the machine, and no host-wide networking is touched — which
//! matters more than usual here, because this checkout shares a kernel with
//! another worker's.
//!
//! # The control comes first, and it is not decoration
//!
//! *A bound turn opened a socket* means nothing on its own: it is also what a
//! turn with no boundary at all would do, and a fixture that quietly failed to
//! bind anything would report the gap it was written to find. So the same child
//! in the same turn is first refused a file outside its grant. If that refusal
//! does not happen, the boundary was not in force and the network result is
//! thrown away rather than believed.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use alo_bounding::{Cgroup, places_of};

mod on_this_kernel;

use on_this_kernel::AsAMachineHasIt;

/// Where the child is told its turn's control group is.
const THE_CGROUP: &str = "ALO_NETWORK_TEST_CGROUP";

/// The file the child tries to open, which nobody granted it.
const THE_FILE: &str = "ALO_NETWORK_TEST_OPEN";

/// The address the child tries to connect to.
const THE_ADDRESS: &str = "ALO_NETWORK_TEST_ADDRESS";

/// What the child says when it is inside the cgroup and waiting.
const READY: &str = "alo:in-the-turn";

/// What the child says about the file, followed by `opened` or a number.
const THE_FILE_WENT: &str = "alo:file ";

/// What the child says about the socket, followed by `connected` or a number.
const THE_SOCKET_WENT: &str = "alo:socket ";

/// How long the child waits for a connection it is not going to be refused
/// quickly, so a hung test fails rather than hanging a whole run.
const NOT_FOREVER: Duration = Duration::from_secs(5);

/// What one attempt came to.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The machine allowed it.
    Allowed,

    /// The machine refused, with the number it gave.
    Refused(i32),
}

impl Outcome {
    /// What the child wrote, read back.
    fn from(text: &str) -> Self {
        match text.parse() {
            Ok(number) => Self::Refused(number),
            Err(_) => Self::Allowed,
        }
    }
}

/// A granted folder and a private one beside it that nobody granted.
fn a_machine_with_something_worth_protecting() -> PathBuf {
    let root = PathBuf::from("/tmp").join(format!("alo-network-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Invoices")).expect("a temporary directory can be made");
    fs::create_dir_all(root.join("Private")).expect("a temporary directory can be made");
    fs::write(root.join("Private/secret.txt"), b"not an invoice").expect("a file can be written");
    root
}

/// One turn: a cgroup, a grant, a child inside it, one open and one connection.
fn a_turn(named: &str, granted: &[&Path], opening: &Path, address: &str) -> (Outcome, Outcome) {
    let _order = on_this_kernel::one_at_a_time();
    let mut kernel = AsAMachineHasIt::on_this_kernel(named);
    let boundary = &mut kernel.boundary;
    let cgroup = Cgroup::made(named).expect("a control group can be made");
    let turn = cgroup.id().expect("a control group has an identifier");

    let mut child = Command::new(env::current_exe().expect("a test binary knows where it is"))
        .args([
            "--exact",
            "--ignored",
            "--nocapture",
            "the_work_a_turn_does",
        ])
        .env(THE_CGROUP, cgroup.at())
        .env(THE_FILE, opening)
        .env(THE_ADDRESS, address)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("this binary can be run again");

    let mut saying = BufReader::new(child.stdout.take().expect("stdout was asked for"));
    let mut telling = child.stdin.take().expect("stdin was asked for");

    let ready = next_line_from(&mut saying);
    assert_eq!(ready.trim(), READY, "the child never reached its cgroup");

    boundary
        .bound(
            turn,
            places_of(granted).expect("the granted folders are there"),
        )
        .expect("the kernel takes the entry");

    telling.write_all(b"go\n").expect("the child is listening");
    telling.flush().expect("the child is listening");

    let file = said_about(&mut saying, THE_FILE_WENT);
    let socket = said_about(&mut saying, THE_SOCKET_WENT);

    child.wait().expect("the child finishes");
    boundary.released(turn).expect("the kernel takes it back");
    cgroup
        .removed()
        .expect("an empty control group can be taken away");
    (file, socket)
}

/// The next line the child says that begins with this, read as an outcome.
fn said_about(saying: &mut BufReader<std::process::ChildStdout>, about: &str) -> Outcome {
    let said = next_line_from(saying);
    let said = said.trim();
    match said.strip_prefix(about) {
        Some(what) => Outcome::from(what),
        None => panic!("the child said `{said}` where `{about}...` was expected"),
    }
}

/// The next line the child says that is meant for us.
fn next_line_from(saying: &mut BufReader<std::process::ChildStdout>) -> String {
    let mut line = String::new();
    loop {
        line.clear();
        let read = saying.read_line(&mut line).expect("the child is talking");
        assert!(
            read > 0,
            "the child stopped talking before it said anything"
        );
        if line.starts_with("alo:") {
            return line;
        }
    }
}

/// **The gap, as this machine has it today.**
///
/// One turn, bound to one folder. The same child in the same turn is refused a
/// file nobody granted — which is the control, and without it the second half
/// would be indistinguishable from a boundary that was never applied — and then
/// opens a socket, which nothing stops.
///
/// **This test failing is the good news.** It means the programme
/// `crates/alo-bounding/src/lib.rs` describes has landed, and whoever landed it
/// should come here, change the assertion to a refusal, and say so in
/// `docs/quirks.md` and their report.
#[test]
fn a_bound_turn_is_refused_a_file_and_is_not_refused_a_socket() {
    let machine = a_machine_with_something_worth_protecting();

    // A listener this test owns, on loopback, on a port the machine chose.
    // Nothing here reaches a network, resolves a name, or touches anything the
    // other worker on this kernel shares.
    let ours = TcpListener::bind("127.0.0.1:0").expect("a loopback listener can be made");
    let address = ours
        .local_addr()
        .expect("a listener knows its own address")
        .to_string();

    let (file, socket) = a_turn(
        "alo-network-gap",
        &[&machine.join("Invoices")],
        &machine.join("Private/secret.txt"),
        &address,
    );

    assert_eq!(
        file,
        Outcome::Refused(13),
        "the boundary was not in force, so what the socket did means nothing"
    );
    assert_eq!(
        socket,
        Outcome::Allowed,
        "a bound turn was refused a socket, which means the network programme has \
         landed — say so in docs/quirks.md, in crates/alo-bounding/src/lib.rs, and turn \
         this assertion into the refusal it should now be"
    );

    let _ = fs::remove_dir_all(&machine);
}

/// The work of a turn, which is a second process because a cgroup holds
/// processes.
///
/// Never run by an ordinary pass — it is ignored, and the parent asks for it by
/// name. Run without the environment that names a cgroup it does nothing.
#[test]
#[ignore = "this is the child half of the test above, and the parent runs it by name"]
fn the_work_a_turn_does() {
    let (Ok(cgroup), Ok(opening), Ok(address)) = (
        env::var(THE_CGROUP),
        env::var(THE_FILE),
        env::var(THE_ADDRESS),
    ) else {
        return;
    };

    fs::write(
        Path::new(&cgroup).join("cgroup.procs"),
        std::process::id().to_string(),
    )
    .expect("a process can put itself in a control group");

    let mut saying = std::io::stdout();
    writeln!(saying, "{READY}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    let mut go = String::new();
    std::io::stdin()
        .read_line(&mut go)
        .expect("the parent says when");

    // From here the boundary is in force. The file first, because it is what
    // says the boundary is in force at all; then the socket, which is the
    // subject. Both answers go out on a descriptor opened before this process
    // joined anything.
    let file = match fs::File::open(&opening) {
        Ok(_) => "opened".to_owned(),
        Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
    };
    writeln!(saying, "{THE_FILE_WENT}{file}").expect("the parent is listening");

    let socket = match address.parse() {
        Err(_) => "unparsed".to_owned(),
        Ok(where_to) => match TcpStream::connect_timeout(&where_to, NOT_FOREVER) {
            Ok(_) => "connected".to_owned(),
            Err(why) => why.raw_os_error().unwrap_or(0).to_string(),
        },
    };
    writeln!(saying, "{THE_SOCKET_WENT}{socket}").expect("the parent is listening");
    saying.flush().expect("the parent is listening");

    // Left rather than returned to the harness, which would print a summary and
    // may touch files this process is no longer allowed to open.
    std::process::exit(0);
}
