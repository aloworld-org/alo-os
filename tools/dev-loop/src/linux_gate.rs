//! Linux phases with a fresh host-space check before each; no shared-host repairs.
use crate::{Result, process::checked};
use std::fs::File;

/// Fixed checkout and private target; no user input is interpolated into commands.
const ENVIRONMENT: &str = "set -eu\n\
export PATH=/root/.cargo/bin:/usr/lib/llvm-22/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\n\
export LLVM_PREFIX=/usr/lib/llvm-22\n\
export CARGO_TARGET_DIR=/root/alo-os-target\n\
cd /mnt/c/dev/alo-os\n";

/// Check, never mount or restart shared state while the other checkout is active.
const READY: &str = "mountpoint -q /sys/fs/bpf";

/// Every previous Linux gate, kept separate so each receives a disk preflight.
const PHASES: &[&str] = &[
    READY,
    "cargo fmt --all --check",
    "cargo clippy --workspace --all-targets --locked -- -D warnings",
    "cargo test --workspace --locked --quiet",
    "RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --locked",
    "cd crates/alo-bounding-kernel\ncargo fmt --all --check",
    "cd crates/alo-bounding-kernel\ncargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings",
];

/// Preserve fail-fast ordering without acquiring the tests' non-reentrant lock.
fn phases(mut check: impl FnMut(&str) -> Result<()>) -> Result<()> {
    for phase in PHASES {
        check(&format!("{ENVIRONMENT}{phase}"))?;
    }
    Ok(())
}

/// Each `checked` call measures Windows host space before launching its phase.
pub fn ready(log: &mut File) -> Result<()> {
    checked("wsl", &["-d", "Ubuntu", "-u", "root", "--", "bash", "-lc", &format!("{ENVIRONMENT}{READY}")], log)
        .map_err(|error| format!("Recovery preflight failed: {error}. Missing bpffs needs coordinated maintenance, not a repair worker.").into())
}

/// Each `checked` call measures Windows host space before launching its phase.
pub fn run(log: &mut File) -> Result<()> {
    phases(|script| {
        checked(
            "wsl",
            &["-d", "Ubuntu", "-u", "root", "--", "bash", "-lc", script],
            log,
        )
    })
    .map_err(|error| {
        format!("Linux gate stopped: {error}. If bpffs is absent, coordinate shared-environment maintenance; never repair it over another worker's tests.").into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_phase_has_the_private_target_and_its_own_invocation() -> Result<()> {
        let mut seen = Vec::new();
        phases(|script| {
            seen.push(script.to_owned());
            Ok(())
        })?;
        assert_eq!(seen.len(), 7);
        for (script, phase) in seen.iter().zip(PHASES) {
            assert_eq!(script, &format!("{ENVIRONMENT}{phase}"));
            assert!(!script.contains("mount -t"));
            assert!(!script.contains("systemctl"));
            assert!(!script.contains("flock"));
        }
        Ok(())
    }

    #[test]
    fn failure_at_any_phase_prevents_every_later_phase() {
        for failure in 0..PHASES.len() {
            let mut calls = 0;
            let outcome = phases(|_| {
                let index = calls;
                calls += 1;
                if index == failure {
                    return Err("synthetic phase failure".into());
                }
                Ok(())
            });
            assert!(outcome.is_err());
            assert_eq!(calls, failure + 1);
        }
    }
}
