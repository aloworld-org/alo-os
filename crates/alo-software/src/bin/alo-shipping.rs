//! **The applications a fresh machine has, installed at first boot.**
//!
//! [ADR 0073](../../../../docs/decisions/0073-a-fresh-machines-applications-are-installed-at-first-boot-from-versions-pinned-by-digest.md)
//! decided this and the owner answered its two questions on 2026-09-28: the
//! version `shipped.toml` names rather than a digest, while the product is being
//! built; and a machine that cannot reach the place says so and is still a
//! machine.
//!
//! # It decides nothing
//!
//! Which applications, where from, and what a refusal means were all settled
//! elsewhere. `alo_software::Shipped` holds the list, `installing` and `install`
//! do the work, `Enabled` says which places may be used, and every refusal is
//! already a sentence in [`alo_software::NotDone`] or [`alo_software::Stopped`].
//! This program is the thing that walks the list, and nothing more — it exists
//! because nothing did.
//!
//! # It does not retry
//!
//! ADR 0073 rejected a first-boot unit that retries for ever: *a machine that is
//! still trying is a machine that never says what happened*. Each application is
//! attempted once. What did not install is named, with the reason, and a person
//! asks again when they have a network.
//!
//! # What it exits with
//!
//! Zero when every application installed, and zero when some did not — because
//! *incomplete* is a state this machine is allowed to be in and systemd should
//! not paint it red. Non-zero is kept for the two faults that mean the program
//! could not do its job at all: the built-in list would not parse, or the places
//! could not be read. Those are defects here, not conditions out there.

use std::process::ExitCode;
use std::time::SystemTime;

use alo_egress::Indicator;
use alo_software::{
    Bound, Enabled, Shipped, TheApplicationsArrived, TheRentedTool, install, installing,
};

fn main() -> ExitCode {
    let shipped = match Shipped::decided() {
        Ok(shipped) => shipped,
        Err(why) => {
            eprintln!("the shipped list could not be read: {why:?}");
            return ExitCode::FAILURE;
        }
    };

    let tool = TheRentedTool::on_this_machine();
    let enabled = match Enabled::read(&tool, Bound::Nobodys) {
        Ok(enabled) => enabled,
        Err(why) => {
            eprintln!("the places to install from could not be read: {why:?}");
            return ExitCode::FAILURE;
        }
    };

    let mut indicator = Indicator::default();
    let mut arrived = 0_usize;
    let mut owed = 0_usize;

    for pinned in shipped.every() {
        let wanted = pinned.wanted();
        // Read off `Pinned`, which says it in public: `Wanted` keeps its
        // application private and is moved into `install` below.
        let application = format!("{:?} from {:?}", pinned.application(), pinned.source());
        match installing(&enabled, &wanted, &tool) {
            Err(why) => {
                owed += 1;
                eprintln!("{application} was not installed: {why:?}");
            }
            Ok(errand) => {
                let underway = indicator.beginning_on_its_own(errand, SystemTime::now());
                let answer = install(&underway, &enabled, wanted, &tool);
                indicator.ended_on_its_own(underway);
                match answer {
                    Ok(_) => {
                        arrived += 1;
                        println!("{application} installed");
                    }
                    Err(why) => {
                        owed += 1;
                        eprintln!("{application} was not installed: {why:?}");
                    }
                }
            }
        }
    }

    println!("{arrived} of this machine's applications arrived, {owed} still owed");

    // **The walk is recorded here, and it is what stops this running every
    // boot.** The unit asked `ConditionFirstBoot=yes` until 2026-10-11 and
    // was skipped on every machine: `bootc install to-disk` populates
    // `/etc/machine-id`, so the first boot is spent before the machine has
    // booted and a written machine shipped with none of its applications.
    //
    // **After the walk and not after a success**, which is this program's own
    // rule one line above: incomplete is a state this machine is allowed to
    // be in, and a machine with no network does not try again for ever (ADR
    // 0073). The two defect exits above return before reaching here on
    // purpose - a program that could not read its own list has not walked
    // anything, and must not be marked as having done so.
    //
    // **A mark that cannot be written is said and not fatal.** The
    // applications really were walked; a machine with a read-only `/var`
    // walks them again next boot, which is a better failure than reporting a
    // walk that did not happen.
    let arrived_before = TheApplicationsArrived::on_this_machine();
    if let Err(why) = arrived_before.record() {
        eprintln!(
            "the applications were walked and it could not be written down at {}: {why} - this              machine will walk them again on its next boot",
            arrived_before.at().display()
        );
    }
    ExitCode::SUCCESS
}
