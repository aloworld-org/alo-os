//! **Writes the person's password into the system's list, once, for the
//! sign-in screen** (ADR 0150).
//!
//! Started by `alo-keeping-the-system-password.socket` for one connection,
//! with the connection as its standard input and output. Only root may open
//! the socket, and the kernel is asked again here, because the sign-in screen
//! is the one caller this exists for. It reads `<number>\n<password>\n`, hands
//! the password to `chpasswd` on its standard input, writes whether it did to
//! `/run/alo-the-system-password/<number>`, and answers one line.
//!
//! **It never prints the password**, and never the line it arrived in: the
//! journal is read by more people than `/etc/shadow` is.

#![cfg(target_os = "linux")]

use std::io::{BufRead, Read, Write};
use std::os::fd::AsFd;
use std::process::{Command, ExitCode, Stdio};

use alo_administrator::in_step;
use alo_administrator::keeping::{self, NotKept};
use alo_administrator::logins;

/// Reads one request from the connection, keeps it or says why not, answers.
fn main() -> ExitCode {
    let kept = kept();
    match kept {
        Ok(()) => println!("{}", keeping::answer_of(kept).trim_end()),
        Err(why) => {
            eprintln!("alo-keeping-the-system-password: not kept: {why}");
            println!("{}", keeping::answer_of(kept).trim_end());
        }
    }
    let _ = std::io::stdout().flush();
    if kept.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The whole request, from the kernel's answer about the caller to the state
/// file.
fn kept() -> Result<(), NotKept> {
    let caller = rustix::net::sockopt::socket_peercred(std::io::stdin().as_fd())
        .map_err(|_| NotKept::NotTheSignInScreen)?;
    if caller.uid.as_raw() != 0 {
        return Err(NotKept::NotTheSignInScreen);
    }
    let mut sent = String::new();
    let mut reading = std::io::stdin()
        .lock()
        .take(u64::try_from(keeping::THE_LONGEST_LINE).unwrap_or(u64::MAX));
    for _ in 0..2 {
        if reading.read_line(&mut sent).is_err() {
            forget(sent);
            return Err(NotKept::NotTwoLines);
        }
    }
    let came = match keeping::what_was_sent(&sent) {
        Ok((person, password)) => written(person, password).map(|()| person),
        Err(why) => Err(why),
    };
    let person = keeping::what_was_sent(&sent).map(|(person, _)| person).ok();
    forget(sent);
    if let Some(person) = person {
        said_in_step(person, came.map(|_| ()));
    }
    came.map(|_| ())
}

/// `chpasswd`, given `name:password` on its standard input and nothing else.
fn written(person: u32, password: &str) -> Result<(), NotKept> {
    let logins = std::fs::read_to_string(logins::THE_LOGINS).map_err(|_| NotKept::NotAPerson)?;
    let name = logins::a_person(&logins, person).ok_or(NotKept::NotAPerson)?;
    let mut chpasswd = Command::new(keeping::CHPASSWD)
        .args(keeping::THE_METHOD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|_| NotKept::DidNotStart)?;
    let handed = chpasswd.stdin.take().is_some_and(|mut stdin| {
        stdin
            .write_all(name.as_bytes())
            .and_then(|()| stdin.write_all(b":"))
            .and_then(|()| stdin.write_all(password.as_bytes()))
            .and_then(|()| stdin.write_all(b"\n"))
            .is_ok()
    });
    let ended = chpasswd.wait().map_err(|_| NotKept::Refused)?;
    if handed && ended.success() {
        Ok(())
    } else {
        Err(NotKept::Refused)
    }
}

/// The state file the desktop's sheet reads, replaced whole. A file that could
/// not be written is said in the journal and is not a reason to answer
/// otherwise: the password was kept or it was not, whatever this file says.
fn said_in_step(person: u32, kept: Result<(), NotKept>) {
    let at = in_step::where_for(std::path::Path::new(in_step::THE_FOLDER), person);
    let fresh = at.with_extension("new");
    let wrote = std::fs::write(&fresh, in_step::as_written(kept)).and_then(|()| {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fresh, std::fs::Permissions::from_mode(0o644))?;
        std::fs::rename(&fresh, &at)
    });
    if let Err(why) = wrote {
        eprintln!(
            "alo-keeping-the-system-password: {} could not be written: {why}",
            at.display()
        );
    }
}

/// Overwrite what arrived before it is let go, so the password does not stay
/// in memory the allocator has taken back.
fn forget(sent: String) {
    let mut bytes = sent.into_bytes();
    bytes.fill(0);
    std::hint::black_box(&bytes);
}
