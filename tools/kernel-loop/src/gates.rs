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
//!
//! # And why the loop runs on Windows while its gates run in WSL
//!
//! That split is not tidiness; it is the only arrangement that works on this
//! machine, and it was found by the loop hanging. **`git push` from inside WSL
//! waits forever for credentials it has no helper for** — fetching a public
//! repository needs none, so everything before the push looks healthy, and the
//! first symptom is a supervisor sitting on a `git-remote-https` that will
//! never be answered. The credentials live on the Windows side.
//!
//! So the loop is a Windows program: `git` runs where the credentials are, and
//! every gate is handed to `wsl` because that is where a kernel, a BPF target
//! and a pinned nightly are. On a Linux host each gate is run directly and
//! there is nothing to bridge.
//!
//! `CARGO_TARGET_DIR` is set to this checkout's own, which is what keeps two
//! contributors compiling the same workspace from writing into one directory.

use std::path::Path;
use std::process::Command;

/// Where this checkout's Linux builds go, kept apart from any other checkout's.
///
/// Named rather than inherited: the environment a Windows process hands to
/// `wsl` is not the environment the gates need, and a shared target directory
/// is two workers overwriting each other's artefacts while both watch a build
/// that should not be rebuilding.
#[cfg(windows)]
const ITS_OWN_TARGET: &str = "$HOME/target-claude";

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
    the_machine_is_ready(at)?;
    let mut passed = Vec::new();
    for gate in EVERY_GATE {
        let said = asking(gate, at)?
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

/// That this machine can run the gates at all, before it spends four minutes
/// discovering that it cannot.
///
/// One precondition, and it is the one this machine actually loses: **a BPF
/// filesystem mounted at `/sys/fs/bpf`**. Every test that loads the boundary
/// pins to it, and WSL forgets the mount across a restart — so the symptom is
/// a kernel test panicking about a directory, three gates and several minutes
/// in, which reads like a broken boundary rather than an unmounted filesystem.
///
/// **It reports rather than mounting.** A supervisor that mounted filesystems
/// would be changing shared kernel state on a machine another worker is using,
/// and `docs/hardware.md` asks the question of a person instead.
///
/// # Errors
/// A sentence naming what is missing and the command that fixes it.
fn the_machine_is_ready(at: &Path) -> Result<(), String> {
    let mounted = Gate {
        named: "a BPF filesystem to pin to",
        program: "mountpoint",
        args: &["-q", "/sys/fs/bpf"],
        within: ".",
    };
    let said = asking(&mounted, at)?
        .output()
        .map_err(|why| format!("this machine could not be asked about /sys/fs/bpf: {why}"))?;
    if said.status.success() {
        return Ok(());
    }
    Err(
        "/sys/fs/bpf is not a mounted BPF filesystem, so every test that loads the boundary          would fail for that reason rather than for anything in the change. Nothing was          published. On this machine: `mount -t bpf bpf /sys/fs/bpf`, which a boot does for          itself and WSL forgets across a restart."
            .to_owned(),
    )
}

/// One gate, as a command this host can actually run.
///
/// # Errors
/// A sentence when a Windows checkout is somewhere `wsl` cannot see.
#[cfg(windows)]
fn asking(gate: &Gate, at: &Path) -> Result<Command, String> {
    let within = as_wsl_sees_it(&at.join(gate.within))?;
    let mut asking = Command::new("wsl");
    asking
        .args(["-d", "Ubuntu", "--", "bash", "-lc"])
        .arg(format!(
            "export PATH=\"$HOME/.cargo/bin:$PATH\"; \
         export CARGO_TARGET_DIR=\"{ITS_OWN_TARGET}\"; \
         export RUSTDOCFLAGS=\"-D warnings\"; \
         cd {within} && {} {}",
            gate.program,
            gate.args.join(" ")
        ));
    Ok(asking)
}

/// A Windows path as the Linux side of this machine names it.
///
/// `C:\dev\alo-os-claude` is `/mnt/c/dev/alo-os-claude`, which is the only
/// shape this needs to handle: the loop runs from a checkout on a drive.
///
/// # Errors
/// A sentence when the path is not of that shape, rather than a guess that
/// would send a gate to the wrong directory and pass.
#[cfg(windows)]
fn as_wsl_sees_it(path: &Path) -> Result<String, String> {
    let written = path.to_string_lossy().replace('\\', "/");
    let (drive, rest) = written
        .split_once(":/")
        .ok_or_else(|| format!("{written} is not a path on a drive, so wsl cannot be told it"))?;
    let letter = drive.to_lowercase();
    Ok(format!("/mnt/{letter}/{rest}"))
}

/// One gate, run where it stands.
///
/// # Errors
/// None on a Linux host; the signature matches the Windows half so the caller
/// has one shape.
#[cfg(not(windows))]
fn asking(gate: &Gate, at: &Path) -> Result<Command, String> {
    let mut asking = Command::new(gate.program);
    asking
        .current_dir(at.join(gate.within))
        .args(gate.args)
        .env("RUSTDOCFLAGS", "-D warnings");
    Ok(asking)
}
