//! **Which Flatpak instance alo's own copy is**, asked of Flatpak itself rather
//! than caught in flight.
//!
//! The process the desktop starts for alo's copy is `flatpak run`. It moves
//! itself into a scope of Flatpak's — `app-flatpak-<id>-<instance>.scope` —
//! and can end while the application runs on there. Caught by looking at the
//! process once a frame, that scope is missed whenever the process moves and
//! ends between two looks (the review of #1287), and then every window of alo's
//! copy is refused as *not alo's* (the walks of 2026-11-15, runs 43726518930
//! and 43739331457).
//!
//! So the copy is started with `--instance-id-fd`, and Flatpak writes the
//! instance number on that descriptor once it has made the scope and before it
//! starts the sandbox, which never inherits the descriptor. Read
//! in Flatpak's own source, `main` branch, on 2026-11-15 — **not on the
//! version the image ships, and not measured on a booted machine**:
//!
//! - `flatpak-builtins-run.c:181` — the option, *"Write the instance ID to the
//!   given file descriptor"*;
//! - `flatpak-run.c:3219-3221` — the id written after the instance's process
//!   number is;
//! - `flatpak-run.c:1013-1017` — the scope named
//!   `app-flatpak-<escaped app id>-<escaped instance id>.scope`;
//! - `flatpak-instance.c:812` — the instance id is a random number, in decimal;
//! - `flatpak-utils.c`, `flatpak_accept_fd_argument` — the descriptor is
//!   refused only when it is not a number or is 0, 1 or 2, which a pipe the
//!   desktop opens never is.
//!
//! Nothing here starts anything or reads a descriptor: this decides the
//! command and reads the answer, so both are tested without Flatpak.

use crate::the_unit::escaped;

/// The option Flatpak writes its instance number on.
pub const INSTANCE_ID_FD: &str = "--instance-id-fd";

/// The longest instance number read: a 32-bit number in decimal, with room.
const THE_LONGEST_INSTANCE: usize = 20;

/// **`command` told to write its instance number on descriptor `fd`** — the
/// option put straight after `run`, where `flatpak run` reads its own options
/// — or [`None`] when `command` is not `flatpak run`, which has no instance
/// to say.
#[must_use]
pub fn told_its_instance(command: &[String], fd: i32) -> Option<Vec<String>> {
    let (program, rest) = command.split_first()?;
    let is_flatpak = std::path::Path::new(program)
        .file_name()
        .is_some_and(|name| name == "flatpak");
    if !is_flatpak || rest.first().map(String::as_str) != Some("run") || fd < 0 {
        return None;
    }
    let mut told = Vec::with_capacity(command.len() + 1);
    told.push(program.clone());
    told.push("run".to_owned());
    told.push(format!("{INSTANCE_ID_FD}={fd}"));
    told.extend(rest.iter().skip(1).cloned());
    Some(told)
}

/// **The instance number Flatpak wrote**, or [`None`] for anything that is not
/// one: digits only, as Flatpak writes them, with nothing else around them.
#[must_use]
pub fn an_instance_number(said: &[u8]) -> Option<String> {
    let said = std::str::from_utf8(said)
        .ok()?
        .trim_end_matches(['\0', '\n']);
    let fits = !said.is_empty()
        && said.len() <= THE_LONGEST_INSTANCE
        && said.bytes().all(|byte| byte.is_ascii_digit());
    fits.then(|| said.to_owned())
}

/// **The scope Flatpak runs this instance of `identifier` in** —
/// `app-flatpak-<escaped identifier>-<instance>.scope`, escaped as Flatpak
/// escapes it, so [`crate::is_a_unit_of`] knows it as that application's.
/// [`None`] for an instance that is not a number.
#[must_use]
pub fn the_scope_of_an_instance(identifier: &str, instance: &str) -> Option<String> {
    an_instance_number(instance.as_bytes())
        .map(|instance| format!("app-flatpak-{}-{instance}.scope", escaped(identifier)))
}

#[cfg(test)]
#[path = "flatpak_instance_tests.rs"]
mod tests;
