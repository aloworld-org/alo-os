//! `alo-going-back.service`: the privileged half of `updates.roll-back`.
//!
//! Every decision is `alo_brokerd::set_going_back`; this is the process that
//! makes them in order on a real machine, and it is thin for the same reason
//! its sister program is.

/// Set the machine to start the build before, on the machine alo OS is for.
#[cfg(unix)]
fn main() -> std::process::ExitCode {
    use std::path::Path;

    use alo_brokerd::{Handing, THE_MACHINES_RECORD, set_going_back};
    use alo_updating::{AcrossRestarts, TheBase};

    // No road out: the build is already on the disk, so there is no proxy to
    // read and nothing leaves this machine.
    match set_going_back(
        &TheBase::on_this_machine(),
        &Handing::on_this_machine(),
        Path::new(THE_MACHINES_RECORD),
        &AcrossRestarts::on_this_machine(),
    ) {
        Ok(returning) => {
            eprintln!(
                "alo-going-back: set, and it happens at the next restart the person makes ({} \
                 arguments, none of them --apply)",
                returning.arguments().len()
            );
            std::process::ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("alo-going-back: nothing was set: {why}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Anywhere else, and it says so rather than pretending.
#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!("alo-going-back is a systemd unit's program and alo OS is Linux (ADR 0011)");
    std::process::ExitCode::FAILURE
}
