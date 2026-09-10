//! What a real sign-in really hands a daemon, on a machine with `logind`.
//!
//! `environment.rs` says what the strings are; this says that they are the ones
//! a machine actually makes. Three states, reached from somebody signing in
//! rather than from a fixture that made a directory and called it a session:
//!
//! 1. **Started under the session.** After a sign-in, `/run/user/<uid>` is
//!    there, it is the person's, and it is what `TheSessionsEnvironment` says.
//! 2. **Reaching that session's bus.** The address the environment names is a
//!    socket that accepts a connection, and it is the one `alo-secrets` derives
//!    from the uid with no environment variable anywhere near it.
//! 3. **Stopping when the session ends.** After a logout, that same environment
//!    names nothing: the bus is gone, and a process started with it would reach
//!    no session at all.
//!
//! # It is a sign-in, and not a number typed into a test
//!
//! The uid comes through `alo_accounts::Session` — a store on the disk, a
//! password verified against it, and the session that opens. So what is compared
//! against the machine is derived from a sign-in the way alo OS derives it, and
//! a change that broke the join between the two crates would fail here rather
//! than pass on a constant that happened to match.
//!
//! The store is the **test's own**, in a temporary directory. Nothing here
//! writes `/etc/alo/accounts.toml`: the account this machine really has is
//! whatever it has, and a test that installed one would be changing the machine
//! it is measuring.
//!
//! # Why it is `#[ignore]`d
//!
//! It signs a person in and out of the machine it runs on. That needs `logind`,
//! needs to be root, and — because sessions are machine-wide — needs to be the
//! only thing testing at the time, exactly as
//! `crates/alo-secrets/tests/a_session_that_really_ended.rs` says at length.
//!
//! ```text
//! cargo test -p alo-entering --test a_daemon_in_the_persons_session -- --ignored --test-threads=1
//! ```
//!
//! # What it does not cover, said rather than implied
//!
//! **It does not start `alo-agentd`.** The daemon reads `/etc/alo/agentd.toml`
//! before anything else it could be asked about here, so starting it would need
//! a machine description installed on whatever machine this runs on — which is
//! the image's job and `docs/autonomy/v0-01-delivery-plan.md`'s task 10, not a
//! file a test may write into `/etc`. What the daemon does with the environment
//! it is handed is `crates/alo-agentd/src/session.rs`, tested there against
//! every string this file measures; that the unit hands it these strings at all
//! is `crates/alo-image`, checked against this crate's derivation.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#![expect(
    clippy::panic,
    reason = "a bounded wait that gives up is a failing test, and it says what it waited for"
)]

use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use alo_accounts::{Accounts, Session};
use alo_entering::TheSessionsEnvironment;
use alo_secrets::{NotStored, TheBus};

/// The person whose sessions these are: the login alo OS's own image creates,
/// so nothing here invents an account on somebody's machine.
const THEIRS: u32 = 1000;

/// The password this test's own store keeps for them. It is a store in a
/// temporary directory that nothing outside this file reads, and it is deleted
/// with the directory.
const THE_PASSWORD: &str = "correct horse battery staple";

/// The longest anything here waits for the machine to catch up.
const AT_MOST: Duration = Duration::from_secs(20);

/// How often it looks while waiting.
const LOOKING_EVERY: Duration = Duration::from_millis(100);

/// What the account at [`THEIRS`] is called, asked of the machine.
///
/// `su` takes a name and never a number, and the name is looked up once so that
/// the uid stays the thing this file is about.
fn their_name() -> String {
    let said = Command::new("getent")
        .args(["passwd", &THEIRS.to_string()])
        .output()
        .expect("getent runs");
    String::from_utf8_lossy(&said.stdout)
        .split(':')
        .next()
        .map(str::to_owned)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| panic!("this machine has no account at uid {THEIRS} to sign in"))
}

/// Refuse to run at all unless this really is a machine where the question has
/// an answer.
fn on_a_machine_that_can_answer() {
    assert_eq!(
        rustix::process::getuid().as_raw(),
        0,
        "this signs a person in and out, which takes root; run it deliberately and alone"
    );
    let running = Command::new("systemctl")
        .arg("is-system-running")
        .output()
        .expect("this machine has systemctl");
    let said = String::from_utf8_lossy(&running.stdout).trim().to_owned();
    assert!(
        said == "running" || said == "degraded",
        "systemd is `{said}`, so logind cannot be asked about sessions"
    );
}

/// Look until this is true, or give up and say what was being waited for.
fn until(what: &str, mut looking: impl FnMut() -> bool) {
    let deadline = Instant::now() + AT_MOST;
    while Instant::now() < deadline {
        if looking() {
            return;
        }
        std::thread::sleep(LOOKING_EVERY);
    }
    panic!("waited {AT_MOST:?} for {what}, and it never happened");
}

/// **The sign-in**: a store of this test's own on a real disk, an account in it,
/// a password verified against it, and the session that opens.
///
/// This is `alo-accounts` used exactly as a sign-in surface would use it, and it
/// is where the uid in everything below comes from.
fn they_sign_in() -> Session {
    let folder = std::env::temp_dir().join(format!("alo-entering-store-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).expect("a directory of our own");
    let at = folder.join("accounts.toml");

    let mut store = Accounts::none().expect("an empty store");
    store
        .created(&their_name(), THEIRS, THE_PASSWORD)
        .expect("an account for the person this machine has");
    alo_accounts::kept(&at, &store).expect("the store is written");

    let store = alo_accounts::found(&at).expect("the store is read back");
    let who = store
        .signs_in(&their_name(), THE_PASSWORD)
        .expect("the password verifies against the store");
    let session = Session::opened(who, THEIRS).expect("the described person is the one signing in");

    drop(std::fs::remove_dir_all(&folder));
    session
}

/// A session of the kind a person really has, held open until it is dropped.
struct ASessionOfTheirs {
    /// The shell holding it open.
    holding: std::process::Child,
}

impl ASessionOfTheirs {
    /// Sign them in to the machine, and wait until it agrees.
    fn opened() -> Self {
        let holding = Command::new("setsid")
            .arg("su")
            .arg("-s")
            .arg("/bin/sh")
            .arg(their_name())
            .arg("-c")
            .arg("sleep 300")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("su runs");
        until("their runtime directory to appear", || {
            TheBus::of(THEIRS).exists()
        });
        Self { holding }
    }
}

impl Drop for ASessionOfTheirs {
    fn drop(&mut self) {
        drop(self.holding.kill());
        drop(self.holding.wait());
    }
}

/// Remove them from the machine altogether, and wait until nothing of theirs is
/// left. How a case starts from nothing, and how it ends.
fn nobody_is_signed_in_as_them() {
    drop(
        Command::new("loginctl")
            .args(["terminate-user", &THEIRS.to_string()])
            .output(),
    );
    until("their session and its bus to be gone", || {
        TheBus::found(THEIRS).is_err()
    });
}

/// Which boot this is, so that a restart cannot be mistaken for a logout.
fn boot() -> String {
    std::fs::read_to_string("/proc/sys/kernel/random/boot_id").expect("this machine says its boot")
}

/// **The three states, from a sign-in.**
///
/// One test rather than three, and deliberately: they are one sequence on one
/// machine-wide thing, and three tests would be three runs taking each other's
/// sessions away — which is the same reason `a_session_that_really_ended.rs`
/// asks for `--test-threads=1`.
#[test]
#[ignore = "signs a person in and out of this machine; needs root, logind, and nothing else testing"]
fn the_environment_a_sign_in_hands_a_daemon_is_the_session_the_machine_made() {
    on_a_machine_that_can_answer();
    let began = boot();
    nobody_is_signed_in_as_them();

    // The sign-in, and the environment derived from *it* rather than from a
    // number written here.
    let session = they_sign_in();
    let environment = TheSessionsEnvironment::of(&session);
    assert_eq!(environment.person(), THEIRS);

    // 1. Started under the session: after a sign-in, the runtime directory the
    //    environment names is there and it is the person's.
    let signed_in = ASessionOfTheirs::opened();
    let directory = PathBuf::from(environment.runtime_directory());
    assert!(
        directory.is_dir(),
        "{} is not there after a sign-in, so nothing started into that session would find one",
        directory.display()
    );

    // 2. Reaching that session's bus. The address the environment names is a
    //    socket that answers, and it is the bus `alo-secrets` derives from the
    //    uid — the two crates meeting on one real machine rather than in a
    //    formatting rule.
    let bus = TheBus::found(THEIRS).expect("their bus is reachable while they are signed in");
    assert_eq!(bus.as_an_address(), environment.bus_address());
    assert_eq!(bus.at_path(), PathBuf::from(environment.bus()));
    assert!(
        environment.names_their_bus(&bus.as_an_address()),
        "the bus this machine made is not the one the daemon would be told about"
    );
    drop(UnixStream::connect(environment.bus()).expect("their bus accepts a connection"));

    // And nobody else's. The refusal matters as much as the reach: this is the
    // comparison `alo-agentd` stops on when a unit file has been edited.
    assert!(!environment.names_their_bus("unix:path=/run/user/0/bus"));

    // 3. Stopping when the session ends. What `BindsTo=user@<uid>.service` is
    //    bound to is exactly this going away.
    drop(signed_in);
    nobody_is_signed_in_as_them();
    assert_eq!(TheBus::found(THEIRS), Err(NotStored::Unavailable));
    assert!(
        UnixStream::connect(environment.bus()).is_err(),
        "their bus still accepts connections after they signed out"
    );

    // A restart removes /run/user/<uid> exactly as a logout does, so a case that
    // spanned one is a case that measured nothing.
    assert_eq!(
        began,
        boot(),
        "the machine restarted during this case, so nothing observed here is evidence"
    );
}
