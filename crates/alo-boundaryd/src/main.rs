//! The loader, as a machine really runs it: once, at boot, before `alo-agentd`.
//!
//! `alo_boundaryd` is every decision this loader is made of; this is the process
//! that makes them in order. It is deliberately thin, because what a process is
//! — the order, and what an exit code means — is the only thing here a test
//! cannot reach: it asks the kernel who it is and it pins into
//! `/sys/fs/bpf/alo`, which is the real machine's boundary rather than a
//! directory a test may write in.
//!
//! # It exits, and the boundary stays
//!
//! The capabilities are needed to load and to pin; they are not needed
//! afterwards, and ADR 0018 says so. So the loader finishes and the pinned link
//! is what holds the programme on `file_open` — which means the machine's
//! boundary is not a running process anybody can stop. `Type=oneshot` with
//! `RemainAfterExit=yes` is the shape of the unit, and the unit is the image's
//! (queue item 28).
//!
//! # It says one kind of thing, and it says it to a service log
//!
//! Nothing this file writes is read by the person using the machine. A kernel
//! with no BPF LSM, a `Group=` line naming root, a `/sys/fs/bpf` that is not a
//! BPF filesystem: those are read by whoever is standing the machine up, in
//! English, which is `alo_boundaryd::NotLoaded`'s argument. What the person
//! reads is `alo-turn`'s sentence about their agent not running, in their own
//! language, and this is what the log says underneath it.
//!
//! # And it runs on Linux
//!
//! On any other host there is no BPF LSM and nothing to impose. It ends in
//! failure rather than in success, because a process that exited cleanly would
//! be telling a supervisor that a machine had been given its boundary.

/// What this process does on the machine alo OS is for.
#[cfg(target_os = "linux")]
mod running {
    use std::process::ExitCode;

    use alo_boundaryd::{imposed, our_group, our_user};
    use alo_bounding::Pinned;

    /// Impose the boundary and say what happened.
    ///
    /// `SUCCESS` when the machine has a boundary it did not have before,
    /// `FAILURE` when it has none. There is no third code and there will not be
    /// one: a machine either can bound a turn or cannot, and a supervisor either
    /// carries on to `alo-agentd` or does not.
    ///
    /// A machine that **already** has one is a failure here, and deliberately:
    /// two programmes on `file_open` are two boundaries, so a loader run twice
    /// is somebody fixing something and the second run says so rather than
    /// quietly agreeing.
    pub fn main() -> ExitCode {
        let pinned = Pinned::on_this_machine();
        match imposed(our_user(), our_group(), &pinned) {
            Ok(loaded) => {
                eprintln!(
                    "alo-boundaryd: the boundary is on this kernel, pinned at {}, with {} to \
                     write it and nobody else; this process holds nothing and is done",
                    pinned.root().display(),
                    our_group(),
                );
                // Said out loud because it is the whole argument for this
                // component: what is dropped here is descriptors, and the
                // machine keeps the boundary because the pin holds it.
                drop(loaded);
                ExitCode::SUCCESS
            }
            Err(why) => {
                eprintln!("alo-boundaryd: this machine has no boundary: {why}");
                ExitCode::FAILURE
            }
        }
    }
}

/// The loader, on the machine it is for.
#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    running::main()
}

/// Say that this is not the machine, and end.
///
/// alo OS boots Linux and this is a BPF LSM programme in a kernel, so on any
/// other host there is nothing to impose.
#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!(
        "alo-boundaryd imposes the boundary alo OS's grants are enforced by, which is a BPF LSM \
         programme in a Linux kernel; there is nothing for it to load here"
    );
    std::process::ExitCode::FAILURE
}
