//! Fixed private permission probe, launched only as the confined image agent.
//!
//! No command-line options, alternate identities, paths, verbs or approvals.
//! Its five short stdout tokens go to the exclusively owned person-controller
//! pipe. The person makes the native Settings choice, then echoes that token.

use std::{fs::File, io::Read, path::Path, process::ExitCode, time::Instant};

use alo_agent::{AtTheDaemonsSocket, TheDoor};
use alo_walking::{
    studio_agent_launch::{
        DeadlineGuard, before_deadline, exchange_stage, read_description, remaining_budget,
    },
    studio_agent_probe::{require_confinement, run},
};

/// Run the fixed private probe and disclose only a bounded diagnostic on error.
fn main() -> ExitCode {
    match probe() {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            eprintln!("Studio private agent check: {why}");
            ExitCode::FAILURE
        }
    }
}

/// Observe actual identity, then keep one daemon connection across the stages.
fn probe() -> Result<(), String> {
    let start = Instant::now();
    if std::env::var("STUDIO_PRIVATE_AGENT_WALK").as_deref() != Ok("1") {
        return Err("the exact private opt-in is required".to_owned());
    }
    if std::env::args_os().nth(1).is_some() {
        return Err("command-line arguments are not accepted".to_owned());
    }
    let status = bounded(
        File::open("/proc/self/status")
            .map_err(|_| "the process identity could not be read".to_owned())?,
    )?;
    require_confinement(&status)
        .map_err(|_| "the process identity is not the confined image agent".to_owned())?;
    read_description(Path::new("/etc/alo/agentd.toml"))?;
    let deadline = start + std::time::Duration::from_secs(300);
    remaining_budget(start, Instant::now())?;
    let mut door = before_deadline(deadline, || {
        AtTheDaemonsSocket::opened(Path::new("/run/alo/1000/agentd.sock"))
            .map_err(|_| "the fixed agent connection could not be opened".to_owned())
    })?;
    let watchdog = door
        .a_second_handle()
        .ok_or_else(|| "the same connection could not be watched".to_owned())?;
    let _guard = DeadlineGuard::on(watchdog, deadline)?;
    let input = std::io::stdin();
    let mut output = std::io::stdout().lock();
    run(
        &mut |request| {
            remaining_budget(start, Instant::now())?;
            let reply = door
                .asking(request)
                .map_err(|_| "the agent connection did not answer the fixed request".to_owned())?;
            remaining_budget(start, Instant::now())?;
            Ok(reply)
        },
        &mut |stage| exchange_stage(stage, &input, &mut output, deadline),
    )?;
    remaining_budget(start, Instant::now())?;
    Ok(())
}

/// Read one bounded public observation without disclosing its content on error.
fn bounded(file: File) -> Result<String, String> {
    let mut text = String::new();
    file.take(65_537)
        .read_to_string(&mut text)
        .map_err(|_| "the public observation could not be read".to_owned())?;
    if text.len() > 65_536 {
        return Err("the public observation exceeds its size limit".to_owned());
    }
    Ok(text)
}
