//! The daemon's production transport reaches one private interface, not our session.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "an unexpected fixture failure must fail the test"
)]

use std::{
    io::{BufRead as _, BufReader},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant, SystemTime},
};

use alo_adapters::{ApplicationSession, Driving};
use alo_applications::{Application, Installed};
use alo_capability::{Approvals, Authorised, Grant, Grantee, Grants, Proposal, Reach};
use alo_strings::{Strings, Vocabulary};

/// Distinct opaque transport data; this fixture does not certify app state parsing.
const STATUS: &str = r#"{"version":1,"phase":"ready","fixture":"private-session-93"}"#;
/// Recursion is confined to this test's own child process.
const CHILD: &str = "ALO_APPLICATION_SESSION_TEST_CHILD";

/// A daemon with no activation directories or connection to another session.
struct PrivateBus {
    /// Exclusive temporary folder, removed after the daemon exits.
    folder: PathBuf,
    /// The one owned daemon process.
    daemon: Child,
    /// Address supplied only to the child test process.
    address: String,
}

impl PrivateBus {
    /// Start an actual bus and bound the wait for its address.
    fn start() -> Self {
        let folder = std::env::temp_dir().join(format!(
            "alo-appsession-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("read the fixture clock")
                .as_nanos()
        ));
        std::fs::create_dir(&folder).expect("create an exclusive bus folder");
        let config = folder.join("bus.conf");
        std::fs::write(
            &config,
            format!(
                "<busconfig><type>session</type><listen>unix:path={}</listen>\
                 <auth>EXTERNAL</auth><policy context=\"default\">\
                 <allow own=\"*\"/><allow send_destination=\"*\"/>\
                 <allow receive_sender=\"*\"/></policy></busconfig>",
                folder.join("bus").display()
            ),
        )
        .expect("write only this bus's policy");
        let daemon = Command::new("dbus-daemon")
            .arg(format!("--config-file={}", config.display()))
            .args(["--nofork", "--print-address"])
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("the integration test requires dbus-daemon");
        let mut bus = Self {
            folder,
            daemon,
            address: String::new(),
        };
        let output = bus
            .daemon
            .stdout
            .take()
            .expect("read the owned bus address");
        let (sent, received) = mpsc::channel();
        std::thread::spawn(move || {
            let mut address = String::new();
            let result = BufReader::new(output)
                .read_line(&mut address)
                .map(|_| address);
            let _ = sent.send(result);
        });
        bus.address = received
            .recv_timeout(Duration::from_secs(5))
            .expect("the bus must announce its address promptly")
            .expect("read the actual address")
            .trim()
            .to_owned();
        assert!(!bus.address.is_empty());
        bus
    }
}

impl Drop for PrivateBus {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
        let _ = std::fs::remove_dir_all(&self.folder);
    }
}

/// Calls observed by the independent public-interface fixture.
struct Recorder(Arc<Mutex<Vec<&'static str>>>);

#[zbus::interface(name = "org.aloworld.Studio.Recorder1")]
impl Recorder {
    /// The changing method must be delivered exactly once.
    fn enable_microphone(&self) {
        self.0
            .lock()
            .expect("retain the call trace")
            .push("EnableMicrophone");
    }

    /// A successful unit reply cannot fabricate this distinctive text.
    fn get_status(&self) -> String {
        self.0
            .lock()
            .expect("retain the call trace")
            .push("GetStatus");
        STATUS.to_owned()
    }
}

/// Deliver real checked declarations through the same transport the daemon installs.
fn child_delivers() {
    let at = SystemTime::now();
    let agent = Grantee::named("@alo");
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@alo",
            Reach::Application(alo_adapters::studio::APPLICATION.to_owned()),
            at,
            Duration::from_secs(3600),
        )
        .expect("grant the fixture application"),
    );
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).expect("declare capability words");
    alo_applications::words::declare_into(&mut vocabulary).expect("declare application words");
    alo_adapters::words::declare_into(&mut vocabulary).expect("declare adapter words");
    let strings = Strings::of(vocabulary);
    let adapters = alo_adapters::shipped_adapters().expect("load public adapters");
    let verbs = alo_adapters::adapter_verbs().expect("load public controls");
    let installed =
        Installed::holding([
            Application::called(alo_adapters::studio::APPLICATION, "alo Studio")
                .expect("declare the installed fixture"),
        ]);
    let call = verbs
        .call("studio.enable_microphone", &[])
        .expect("build the change");
    let mut approvals = Approvals::default();
    let pending = approvals.propose(
        Proposal::checked(&call, &agent, &grants, at, Duration::from_secs(300))
            .expect("propose a checked change"),
    );
    let authorised = approvals
        .approve(pending, at)
        .expect("approve the fixture change")
        .redeem(&grants, at)
        .expect("redeem its one approval");
    let changed = Driving::of(authorised, &adapters, &grants, &installed, &strings)
        .expect("check the installed application")
        .deliver(&ApplicationSession, &strings)
        .expect("deliver the change through the production wrapper");
    assert!(!changed.unanswered());
    assert_eq!(changed.reply(), None);
    let call = verbs
        .call("studio.status", &[])
        .expect("build the status read");
    let authorised = Authorised::read(&call, &agent, &grants, at).expect("authorise the read");
    let read = Driving::of(authorised, &adapters, &grants, &installed, &strings)
        .expect("check the status target")
        .deliver(&ApplicationSession, &strings)
        .expect("read through the production wrapper");
    assert_eq!(read.reply(), Some(STATUS));
    assert_eq!(read.application(), "org.aloworld.Studio");
    assert!(!read.unanswered());
}

/// Own the child until it has exited, including when a parent assertion fails.
struct OwnedChild(Child);

impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Process-local environment selection must preserve actual calls and exact text.
#[test]
fn production_application_session_delivers_one_change_and_the_actual_status() {
    if std::env::var_os(CHILD).as_deref() == Some(std::ffi::OsStr::new("1")) {
        child_delivers();
        return;
    }
    let bus = PrivateBus::start();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let _service = zbus::blocking::connection::Builder::address(bus.address.as_str())
        .expect("connect the service to its private bus")
        .name("org.aloworld.Studio")
        .expect("own the public fixture name")
        .serve_at("/org/aloworld/Studio/Recorder", Recorder(calls.clone()))
        .expect("publish the fixture interface")
        .build()
        .expect("start the actual service");
    let mut child = OwnedChild(
        Command::new(std::env::current_exe().expect("find this integration test"))
            .args([
                "--exact",
                "production_application_session_delivers_one_change_and_the_actual_status",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("DBUS_SESSION_BUS_ADDRESS", &bus.address)
            .stdin(Stdio::null())
            .spawn()
            .expect("run the production-wrapper check in its own environment"),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    let status = loop {
        if let Some(status) = child.0.try_wait().expect("observe the owned child") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "the wrapper child did not finish"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(status.success(), "the wrapper child failed: {status}");
    assert_eq!(
        *calls.lock().expect("inspect the service-side trace"),
        ["EnableMicrophone", "GetStatus"]
    );
}
