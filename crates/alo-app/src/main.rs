//! `alo-app`, on a real machine: its terminal, its desktop, its accessibility
//! tree, its session bus, its daemon and the person's own record.
//!
//! **What was typed is read before the machine is touched**, exactly as
//! `alo-ask` and `alo-try` do it, so a person who pressed return by mistake is
//! owed *that is not the shape* rather than something about their machine. The
//! person's language is read from their settings the way `alo-ask` reads it.
//!
//! Two English sentences survive, and they are `alo-try`'s: a session with no
//! home, and a process with no current folder, which cannot happen inside a
//! session and are read by whoever is fixing one.

#[cfg(target_os = "linux")]
mod on_this_machine;

#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    on_this_machine::main()
}

/// `alo-app` reaches an alo OS session, which only Linux has.
#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!("alo-app: this command reaches an alo OS session, which runs on Linux");
    std::process::ExitCode::from(alo_app::Ended::NotOffered.code())
}
