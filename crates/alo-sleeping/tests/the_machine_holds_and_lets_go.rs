//! A hold asked of this machine's own `logind`, seen in its list, and gone
//! when let go — through this crate's own call.
//!
//! What `crate::logind`'s fixtures cannot show is that the arguments this crate
//! assembles are ones the real service accepts, and that dropping the value
//! really lets go. So this takes one hold of the harmless kind — a block on
//! *idle*, which only stops a machine sleeping on its own for the length of the
//! test — reads `ListInhibitors` for it, drops it, and reads again.
//!
//! **It never asks the machine to sleep.** A test that suspended the machine
//! running the gate would be a shared-system change of exactly the kind
//! `docs/autonomy/SHARED_MAIN.md` says to coordinate; a sleep is owed on
//! certified hardware, and the report says so.
//!
//! A machine with no system bus, no `logind`, or a policy that refuses this
//! caller a hold has nothing to measure. The test says what it skipped and why
//! rather than failing on the absence of a service — the honest floor
//! `alo-sessiond`'s own measurement set.

#![cfg(target_os = "linux")]
#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_sleeping::{Inhibit, Logind, TheMachinesLogind};
use alo_strings::{Filling, Strings};

/// Every inhibitor `logind` lists: what, who, why, mode, uid, pid.
type Listed = Vec<(String, String, String, String, u32, u32)>;

/// What `logind` lists right now, or why it could not be asked.
fn listed() -> Result<Listed, String> {
    let bus = zbus::blocking::Connection::system().map_err(|why| why.to_string())?;
    let reply = bus
        .call_method(
            Some("org.freedesktop.login1"),
            "/org/freedesktop/login1",
            Some("org.freedesktop.login1.Manager"),
            "ListInhibitors",
            &(),
        )
        .map_err(|why| why.to_string())?;
    reply.body().deserialize().map_err(|why| why.to_string())
}

/// **A hold this crate takes is one `logind` lists under alo OS, with the
/// person's sentence, and letting go of the value takes it off the list.**
#[test]
fn a_hold_is_listed_while_held_and_gone_when_let_go() {
    let Ok(mut logind) = TheMachinesLogind::on_the_system_bus() else {
        eprintln!("skipped: no system bus on this machine, so there is no logind to measure");
        return;
    };
    let mut vocabulary = alo_strings::Vocabulary::empty();
    let Ok(()) = alo_sleeping::declare_into(&mut vocabulary) else {
        panic!("this crate's words do not declare");
    };
    let strings = Strings::of(vocabulary);
    let why = strings.say(
        &alo_sleeping::words::KEPT_AWAKE_BY_YOUR_SETTING.key(),
        &Filling::nothing(),
    );

    let held = match logind.hold(Inhibit::Idle, &why) {
        Ok(held) => held,
        Err(not) => {
            eprintln!(
                "skipped: this machine's logind would not give this caller a hold: {}",
                not.machine
            );
            return;
        }
    };

    let ours = |list: &Listed| {
        list.iter().any(|(what, who, said, mode, _, pid)| {
            what == "idle"
                && who == "alo OS"
                && said == why.text()
                && mode == "block"
                && *pid == std::process::id()
        })
    };

    let Ok(while_held) = listed() else {
        panic!("logind took a hold and would not list its holds");
    };
    assert!(ours(&while_held), "{while_held:?}");

    drop(held);
    let Ok(after) = listed() else {
        panic!("logind would not list its holds a second time");
    };
    assert!(!ours(&after), "{after:?}");
}
