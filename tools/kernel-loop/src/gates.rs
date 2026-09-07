//! The checks the repository requires, run for real, with what they actually
//! said kept.
//!
//! # Every gate, every time, and none of them optional
//!
//! There is no flag here that skips one. A supervisor with a *quick* mode is a
//! supervisor that publishes work nobody gated, and the first time it matters
//! nobody will remember the flag was on.
//!
//! # Linux runs them; Windows is asked separately and is not pretended
//!
//! The kernel half of this workstream cannot be built or run on Windows at all
//! — `alo-bounding` compiles to nothing there and there is no BPF target. So
//! this runs the Linux gates, and what it records about Windows is that it did
//! not run them here. `docs/autonomy/updates/` reports say the same thing in
//! words. **WSL is development evidence and never certified-hardware
//! acceptance**, and nothing in this file may be read as the latter.

use std::path::Path;
use std::process::Command;

/// One check, and what it is called in a report.
pub struct Gate {
    /// What a person calls it.
    pub named: &'static str,

    /// The program to run.
    pub program: &'static str,

    /// Its arguments.
    pub args: &'static [&'static str],

    /// Where to run it, relative to the checkout.
    pub within: &'static str,
}

/// Every gate this workstream is held to, in the order they are cheapest to
/// fail in.
///
/// The last two are the BPF target's own, on the pinned nightly, which is why
/// they are run inside `crates/alo-bounding-kernel`: `rustup` picks a toolchain
/// from where it is invoked, and that directory's `rust-toolchain.toml` is the
/// pin whose LLVM matches the `bpf-linker` on the machine.
pub const EVERY_GATE: &[Gate] = &[
    Gate {
        named: "formatting",
        program: "cargo",
        args: &["fmt", "--all", "--check"],
        within: ".",
    },
    Gate {
        named: "clippy, warnings denied",
        program: "cargo",
        args: &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        within: ".",
    },
    Gate {
        named: "the workspace's tests",
        program: "cargo",
        args: &["test", "--workspace"],
        within: ".",
    },
    Gate {
        named: "rustdoc, warnings denied",
        program: "cargo",
        args: &["doc", "--workspace", "--no-deps"],
        within: ".",
    },
    Gate {
        named: "the BPF target's formatting",
        program: "cargo",
        args: &["fmt", "--all", "--check"],
        within: "crates/alo-bounding-kernel",
    },
    Gate {
        named: "the BPF target's clippy",
        program: "cargo",
        args: &[
            "clippy",
            "--release",
            "--target",
            "bpfel-unknown-none",
            "-Z",
            "build-std=core",
            "--",
            "-D",
            "warnings",
        ],
        within: "crates/alo-bounding-kernel",
    },
];

/// Run every gate, stopping at the first that fails.
///
/// # Errors
/// A sentence naming the gate and the last of what it printed. Stopping at the
/// first is deliberate: the others' output would be noise around the one thing
/// that has to be fixed.
pub fn all_of_them(at: &Path) -> Result<Vec<String>, String> {
    let mut passed = Vec::new();
    for gate in EVERY_GATE {
        let said = Command::new(gate.program)
            .current_dir(at.join(gate.within))
            .args(gate.args)
            .env("RUSTDOCFLAGS", "-D warnings")
            .output()
            .map_err(|why| format!("`{}` could not be run: {why}", gate.named))?;
        if !said.status.success() {
            let err = String::from_utf8_lossy(&said.stderr);
            let out = String::from_utf8_lossy(&said.stdout);
            let tail: Vec<&str> = err.lines().chain(out.lines()).collect();
            let from = tail.len().saturating_sub(25);
            return Err(format!(
                "the gate `{}` did not pass, so nothing was published:\n{}",
                gate.named,
                tail.get(from..).unwrap_or_default().join("\n")
            ));
        }
        passed.push(gate.named.to_owned());
    }
    Ok(passed)
}
