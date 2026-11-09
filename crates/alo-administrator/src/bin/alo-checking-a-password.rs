//! **Hands one connection to polkit's own helper, for the caller's own
//! login** (ADR 0150).
//!
//! Started by `alo-checking-a-password.socket` for one connection, with the
//! connection as its standard input and output. The desktop that connected
//! runs with no capability and no new privileges, so polkit's setuid helper
//! could do nothing for it; this is the helper run as root on its behalf.
//!
//! **Who is checked is the kernel's answer, never the caller's.** The login is
//! the one `SO_PEERCRED` names, so a caller can only ever be checked as
//! itself — and the agent's login and every service's are refused before the
//! helper starts. The helper then reads polkit's cookie from the connection,
//! asks PAM, and tells polkit itself; nothing here reads the password.

#![cfg(target_os = "linux")]

use std::os::fd::AsFd;
use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode};

use alo_administrator::logins;

/// polkit's own helper, from the base.
const THE_HELPER: &str = "/usr/lib/polkit-1/polkit-agent-helper-1";

/// Becomes polkit's helper for the caller's login, or says why not.
fn main() -> ExitCode {
    let caller = match rustix::net::sockopt::socket_peercred(std::io::stdin().as_fd()) {
        Ok(caller) => caller.uid.as_raw(),
        Err(why) => {
            eprintln!("alo-checking-a-password: the kernel would not say who asked: {why}");
            println!("FAILURE");
            return ExitCode::FAILURE;
        }
    };
    let logins = std::fs::read_to_string(logins::THE_LOGINS).unwrap_or_default();
    let Some(name) = logins::a_person(&logins, caller) else {
        eprintln!("alo-checking-a-password: number {caller} is not a person, so nothing is checked");
        println!("FAILURE");
        return ExitCode::FAILURE;
    };
    // Standard input and output are the connection, inherited.
    let why = Command::new(THE_HELPER).arg(name).exec();
    eprintln!("alo-checking-a-password: {THE_HELPER} would not start: {why}");
    println!("FAILURE");
    ExitCode::FAILURE
}
