//! `alo-applying-an-update.service`: the privileged half of `updates.apply`.
//!
//! Every decision is `alo_brokerd::stage_the_update_approved`; this is the
//! process that makes them in order on a real machine, and it is deliberately
//! thin because the only thing here a test cannot reach is the machine's own
//! paths.
//!
//! It says one kind of thing, to the journal, in English for whoever
//! administers the machine. What the *person* reads is said by the surface that
//! asked, in their own language, from the one word the broker answered.

/// Stage what the broker handed over, on the machine alo OS is for.
#[cfg(unix)]
fn main() -> std::process::ExitCode {
    use alo_brokerd::{Handing, stage_the_update_approved, the_road_out};
    use alo_updating::TheBase;

    // Reading the machine's own status reaches no network, so the base that is
    // asked about it goes straight out. The base that is told to *fetch* takes
    // this machine's own proxy, decided for that road and set on it explicitly.
    let reading = TheBase::on_this_machine();
    let fetching =
        |host: &str| the_road_out(host).map(|road| TheBase::on_this_machine().taking(road));

    match stage_the_update_approved(&reading, &Handing::on_this_machine(), fetching) {
        Ok(staging) => {
            eprintln!(
                "alo-applying-an-update: staged, and it applies at the next restart the person \
                 makes ({} arguments, none of them --apply)",
                staging.arguments().len()
            );
            std::process::ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("alo-applying-an-update: nothing was staged: {why}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Anywhere else, and it says so rather than pretending.
#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!("alo-applying-an-update is a systemd unit's program and alo OS is Linux (ADR 0011)");
    std::process::ExitCode::FAILURE
}
