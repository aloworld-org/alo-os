//! What the **bus** does when a session really ends, on a machine with
//! `logind`.
//!
//! `connections_come_and_go.rs` proves **disconnection handling**: a keyring
//! handle whose bus has stopped refuses, promptly, and hands back no key. Its
//! fixture stops the private bus and keyring daemon *that the fixture itself
//! started*, and that is all it stops. ADR 0022 says in as many words that this
//! is not real logout.
//!
//! This is the part of the difference that can be measured from here: what
//! `/run/user/<uid>` and the person's bus actually do across a real sign-in and
//! sign-out, and what `alo-secrets` answers beside them. **It is not the whole
//! of the question** — see *What this does not cover* below, which is written
//! out rather than implied.
//!
//! # Why it is `#[ignore]`d
//!
//! It signs a user in and out of the machine it runs on. That needs `logind`,
//! needs to be root, and — because sessions are machine-wide — needs to be the
//! only thing testing at the time. None of those are true of an ordinary
//! `cargo test`, so it is run deliberately:
//!
//! ```text
//! cargo test -p alo-secrets --test a_session_that_really_ended -- --ignored --test-threads=1
//! ```
//!
//! `--test-threads=1` is not tidiness. Every case signs the same person in and
//! out, so two at once would be two tests taking each other's sessions away.
//!
//! # Three cases, and no second seat for any of them
//!
//! The plan had these as *reachable with one or two `ssh` logins*. **This
//! machine has no `sshd` at all** — the binary is absent and the unit is
//! `not-found` — so as written every case was blocked on installing a network
//! service. `su` reaches the same sessions: `pam_systemd` is in
//! `common-session`, so a login through PAM registers with `logind` and gets a
//! `/run/user/<uid>` and a user bus, with no listener anywhere. Nothing here
//! installs one, and no case needs a second seat.
//!
//! 1. **One session, ended.** No lingering. The bus goes, and `alo-secrets`
//!    says `Unavailable`.
//! 2. **Logged out while lingering is on.** The user manager and its bus
//!    survive with nobody signed in — which is a **policy question for the
//!    owner**, not a bug to fix quietly. This only measures it.
//! 3. **Two sessions, one ended.** The bus survives, because the other holds
//!    it.
//!
//! # The boot id is checked, not assumed
//!
//! An earlier look appeared to show sessions surviving a logout. They had not:
//! the distribution had restarted between the two observations, and a restart
//! tidies up everything a logout would have. So every case records the boot id
//! either side of what it did and **refuses to draw a conclusion across a
//! restart** rather than reporting one that happens to look right.
//!
//! # What this does not cover
//!
//! **A keyring handle held across the logout**, which is the other half of case
//! 1 and is not here. Putting a real Secret Service on the person's *session*
//! bus needs a process running **as the person** — root does not complete the
//! D-Bus handshake on their bus, measured, not assumed — and such a process is
//! itself killed by `loginctl terminate-user`. So the shape that test needs is
//! a helper re-executed as them which survives, or reports across, the very
//! event being measured. That is a piece of work of its own and it is not
//! pretended here.
//!
//! What is established meanwhile is the thing the daemon actually branches on:
//! `alo-agentd` asks `TheBus` before it opens any store, so a bus that is gone
//! is a lookup that never happens. These cases say when it is gone.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#![expect(
    clippy::panic,
    reason = "a bounded wait that gives up is a failing test, and it says what it waited for — \
              which an assert after the loop could not"
)]

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use alo_secrets::{NotStored, TheBus};

/// The person whose sessions these are: the login alo OS's own image creates,
/// so nothing here invents an account on somebody's machine.
const THEIRS: u32 = 1000;

/// The longest anything here waits for the machine to catch up.
const AT_MOST: Duration = Duration::from_secs(20);

/// How often it looks while waiting.
const LOOKING_EVERY: Duration = Duration::from_millis(100);

/// What the account at [`THEIRS`] is called.
///
/// `su` takes a **name**, never a number — `su 1000` is *user 1000 does not
/// exist*, which is a sentence about an account rather than about a uid. The
/// uid stays the thing this file is about, because `/run/user/<uid>` and
/// `TheBus` are both keyed by it; the name is looked up once, here, so the two
/// cannot drift apart in a hard-coded pair.
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

/// Which boot this is, so that a restart cannot be mistaken for a logout.
fn boot() -> String {
    std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .expect("this machine says which boot it is on")
}

/// Refuse to run at all unless this really is a machine where the question has
/// an answer — rather than passing somewhere it could not have been asked.
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

/// Whatever `loginctl` says, as its own words.
fn loginctl(args: &[&str]) -> String {
    let said = Command::new("loginctl")
        .args(args)
        .output()
        .expect("loginctl runs");
    String::from_utf8_lossy(&said.stdout).trim().to_owned()
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

/// Every session `logind` currently has for them, of any class.
fn their_sessions() -> Vec<String> {
    loginctl(&[
        "show-user",
        &THEIRS.to_string(),
        "-p",
        "Sessions",
        "--value",
    ])
    .split_whitespace()
    .map(str::to_owned)
    .collect()
}

/// What class `logind` gives a session.
fn class_of(session: &str) -> String {
    loginctl(&["show-session", session, "-p", "Class", "--value"])
}

/// What state it is in: `active`, `online`, or `closing` on its way out.
fn state_of(session: &str) -> String {
    loginctl(&["show-session", session, "-p", "State", "--value"])
}

/// The sessions that are a **login** — somebody signed in — rather than the
/// `manager` session that represents the user manager itself.
///
/// The distinction is the whole of case 2 and it took a failing test to find.
/// `su` produces two sessions: a login (class `background` here, `user` on a
/// seat) and a `manager`. *Logging out* ends the login; whether the `manager`
/// and its bus go too is `logind`'s decision, and lingering is how an
/// administrator changes that decision. Counting the `manager` as *still
/// signed in* made logout look like it never finished.
fn their_logins() -> Vec<String> {
    their_sessions()
        .into_iter()
        .filter(|session| {
            // A session listed a moment ago can be gone by the time its class
            // is asked for, and then `loginctl` answers with nothing. **Nothing
            // is not a login.** Counting it as one made *logged out* flicker
            // back to *still signed in* between polls, and a wait for the
            // logins to end never finished — which is a test failing on the gap
            // between two `loginctl` calls rather than on anything the machine
            // did. So a login is a session whose class was read **and** is not
            // the user manager's.
            let class = class_of(session);
            // **And it is not on its way out.** A terminated session stays
            // listed, in state `closing`, while its processes are reaped —
            // Ubuntu's `KillUserProcesses=no` makes that stretch. Counting a
            // closing session as somebody signed in made a logout that had
            // plainly happened look like it never did, intermittently: it
            // passed one run and timed out the next, on the same machine and
            // the same code. Somebody is signed in when a session is *up*.
            !class.is_empty() && class != "manager" && state_of(session) != "closing"
        })
        .collect()
}

/// Whether anybody is signed in as them.
fn signed_in() -> bool {
    !their_logins().is_empty()
}

/// A session of the kind a person really has, held open until it is dropped.
///
/// `su` rather than `ssh`, for the reason in this file's header. `setsid` so
/// the shell is not in this test's own process group and cannot be taken down
/// with it by accident — what ends this session is `logind`, which is the event
/// being measured.
struct ASessionOfTheirs {
    holding: std::process::Child,
}

impl ASessionOfTheirs {
    /// Sign them in, and wait until the machine agrees they are.
    fn opened() -> Self {
        let before = their_logins().len();
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
        until("another login to appear for them", || {
            their_logins().len() > before
        });
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

/// Remove them from the machine altogether — every session **and** the user
/// manager.
///
/// This is how a case starts from nothing, not how a person logs out.
/// `terminate-user` takes the manager down too, which is precisely what
/// lingering exists to prevent, so using it to "log out" would defeat case 2 by
/// definition rather than measure it. [`log_them_out`] is the logout.
fn sign_them_out() {
    drop(
        Command::new("loginctl")
            .args(["terminate-user", &THEIRS.to_string()])
            .output(),
    );
    until("their sessions to end when removed altogether", || {
        !signed_in()
    });
}

/// Log them out: end each session they have, by name, and leave whatever
/// `logind` decides to keep.
///
/// The difference from [`sign_them_out`] is the whole of case 2. Ending a
/// session is what a person does; whether the user manager and its bus go with
/// it is `logind`'s decision, and lingering is how an administrator changes
/// that decision. A test that killed the manager itself would be answering its
/// own question.
fn log_them_out() {
    for session in their_logins() {
        drop(
            Command::new("loginctl")
                .args(["terminate-session", &session])
                .output(),
        );
    }
    until("their logins to end when they logged out", || !signed_in());
}

/// Whether lingering is on for them, as logind says.
fn lingering() -> bool {
    loginctl(&["show-user", &THEIRS.to_string(), "-p", "Linger", "--value"]) == "yes"
}

/// Lingering, turned on for the length of one case and **off again after**.
///
/// `enable-linger` is a machine-wide change that outlives the process that made
/// it — a user manager with no session is exactly what it is for. So it is a
/// value with a `Drop`: the case that needs it cannot leave it on by returning
/// early or by failing.
struct Lingering;

impl Lingering {
    fn on() -> Self {
        drop(
            Command::new("loginctl")
                .args(["enable-linger", &THEIRS.to_string()])
                .output(),
        );
        until("lingering to be on", lingering);
        Self
    }
}

impl Drop for Lingering {
    fn drop(&mut self) {
        drop(
            Command::new("loginctl")
                .args(["disable-linger", &THEIRS.to_string()])
                .output(),
        );
        // And the manager it was keeping up goes with it, so the next case
        // starts where this one found the machine.
        drop(
            Command::new("loginctl")
                .args(["terminate-user", &THEIRS.to_string()])
                .output(),
        );
    }
}

/// Whatever a previous case left, cleared before this one looks — and the boot
/// it started on, to be compared with the boot it ends on.
fn from_a_machine_nobody_is_signed_in_to() -> String {
    on_a_machine_that_can_answer();
    if lingering() {
        drop(
            Command::new("loginctl")
                .args(["disable-linger", &THEIRS.to_string()])
                .output(),
        );
    }
    sign_them_out();
    // The manager stops on its own schedule rather than the moment the last
    // login goes, so this waits for the bus itself rather than asserting on it
    // straight away — a previous case's user manager still shutting down is not
    // the same as a bus that outlived a logout, and only one of those is worth
    // failing over.
    until("any previous session's bus to be gone", || {
        TheBus::found(THEIRS).is_err()
    });
    assert!(
        matches!(TheBus::found(THEIRS), Err(NotStored::Unavailable)),
        "their bus was reachable before anybody signed in, so what follows would be measuring \
         somebody else's session"
    );
    boot()
}

/// The machine did not restart while this case was looking.
///
/// A restart removes `/run/user/<uid>` exactly as a logout does, so without
/// this every case here could pass on a machine that never ended a session at
/// all.
fn nothing_restarted_underneath(began: &str) {
    assert_eq!(
        began,
        boot(),
        "the machine restarted during this case, which tidies up everything a logout would have — \
         so nothing observed here is evidence about sessions"
    );
}

/// **Case 1 — one session, ended.** `/run/user/<uid>` and the bus go with it,
/// and `alo-secrets` answers `Unavailable`.
///
/// This is the case that says whether `connections_come_and_go.rs` is
/// representative of a real logout at the level the daemon cares about. It is:
/// the bus a real logout takes away is gone as completely as the fixture's
/// stopped one, and `TheBus` refuses for the same reason with no fallback.
#[test]
#[ignore = "signs a person in and out of this machine; needs root, logind, and nothing else testing"]
fn one_session_ended_takes_the_bus_with_it() {
    let began = from_a_machine_nobody_is_signed_in_to();

    let session = ASessionOfTheirs::opened();
    assert!(
        TheBus::found(THEIRS).is_ok(),
        "their bus is not reachable while they are signed in, so what follows would prove nothing"
    );
    assert!(
        TheBus::of(THEIRS).exists(),
        "the bus socket is not on the disk"
    );

    // A logout, not a removal: `logind` decides what goes with it, which is the
    // thing being measured. **Before** the shell is killed, not after — killing
    // it first orphans the `sleep` into a session nobody is ending, and then
    // logging out has nothing left to end and never completes. Ending the
    // session is what takes its processes with it.
    log_them_out();
    drop(session);

    // A bounded wait, and it *is* the assertion: the user manager stops on its
    // own schedule, so what is claimed is that the bus goes — not that it goes
    // within one syscall of the logout. If it never goes, this fails loudly
    // rather than passing on a technicality.
    until("their bus to go when they logged out", || {
        TheBus::found(THEIRS).is_err()
    });
    assert!(
        matches!(TheBus::found(THEIRS), Err(NotStored::Unavailable)),
        "their bus outlived their session with no lingering"
    );
    assert!(
        !TheBus::of(THEIRS).exists(),
        "the bus socket is still on the disk after a real logout"
    );
    nothing_restarted_underneath(&began);
}

/// **Case 2 — logged out while lingering is on.** The user manager and its bus
/// **survive with nobody signed in at all**.
///
/// This is the case a daemon's author would not predict from case 1, and it is
/// why the plan lists it separately. **What it measures is not a bug**: a bus —
/// and so a credential store — reachable while nobody is signed in is a
/// *policy*, and whether alo OS wants it is the owner's question. This
/// establishes that it is what the machine does.
///
/// Lingering is turned off again by [`Lingering`]'s `Drop`, so a failure here
/// cannot leave the machine changed.
#[test]
#[ignore = "signs a person in and out of this machine; needs root, logind, and nothing else testing"]
fn lingering_keeps_the_bus_after_everybody_signs_out() {
    let began = from_a_machine_nobody_is_signed_in_to();
    let _lingering = Lingering::on();

    let session = ASessionOfTheirs::opened();
    assert!(
        TheBus::found(THEIRS).is_ok(),
        "their bus is not reachable while they are signed in"
    );

    // Ended before the shell is killed, for the reason case 1 gives.
    log_them_out();
    drop(session);

    assert!(
        !signed_in(),
        "somebody is still signed in, so this is not the case it claims to be"
    );
    // Long enough that a manager which was going to stop would have. Without
    // this, *survives* and *has not got round to stopping yet* look identical —
    // and case 1 measures how long the going takes, which is well under this.
    std::thread::sleep(Duration::from_secs(5));
    assert!(
        !signed_in(),
        "somebody signed back in while this was waiting"
    );
    assert!(
        TheBus::found(THEIRS).is_ok(),
        "with lingering on, their bus did not survive everybody signing out"
    );
    nothing_restarted_underneath(&began);
}

/// **Case 3 — two sessions, one ended.** The bus stays, because the other
/// session holds it.
///
/// Two concurrent logins for the same person and **no second seat**: the plan
/// assumed two physical seats were needed, and that assumption is part of what
/// kept this unscheduled. One of the two is ended by name; the other is left
/// alone.
#[test]
#[ignore = "signs a person in and out of this machine; needs root, logind, and nothing else testing"]
fn one_of_two_sessions_ending_leaves_the_bus_where_it_is() {
    let began = from_a_machine_nobody_is_signed_in_to();

    let first = ASessionOfTheirs::opened();
    let after_one = their_logins();
    let second = ASessionOfTheirs::opened();
    let after_two = their_logins();
    assert!(
        after_two.len() > after_one.len(),
        "the second login did not make a session of its own: {after_one:?} then {after_two:?}"
    );
    assert!(TheBus::found(THEIRS).is_ok());

    // End exactly one of them, by name: the one the second login added.
    let ending = after_two
        .iter()
        .find(|session| !after_one.contains(session))
        .expect("the second login added a session")
        .clone();
    drop(
        Command::new("loginctl")
            .args(["terminate-session", &ending])
            .output(),
    );
    until("that one login to end", || {
        !their_logins().contains(&ending)
    });

    assert!(
        signed_in(),
        "ending one of two sessions signed them out altogether"
    );
    assert!(
        TheBus::found(THEIRS).is_ok(),
        "their bus went away while they were still signed in elsewhere"
    );
    nothing_restarted_underneath(&began);

    drop(first);
    drop(second);
    sign_them_out();
}
