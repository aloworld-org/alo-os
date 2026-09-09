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
    // **The supervisor's own.** Its workspace is separate from the product's,
    // so `cargo test --workspace` above never reaches it — which meant the
    // program that decides what gets published was the one thing nothing
    // checked. Cheap, and it runs before the slow BPF gates.
    Gate {
        named: "the supervisor's formatting",
        program: "cargo",
        args: &["fmt", "--all", "--check"],
        within: "tools/kernel-loop",
    },
    Gate {
        named: "the supervisor's clippy",
        program: "cargo",
        args: &["clippy", "--all-targets", "--", "-D", "warnings"],
        within: "tools/kernel-loop",
    },
    Gate {
        named: "the supervisor's own tests",
        program: "cargo",
        args: &["test"],
        within: "tools/kernel-loop",
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
        whether_it_passed(
            gate.named,
            said.status.success(),
            &String::from_utf8_lossy(&said.stdout),
            &String::from_utf8_lossy(&said.stderr),
        )?;
        passed.push(gate.named.to_owned());
    }
    Ok(passed)
}

/// One gate's result, read.
///
/// Separate from running it so that *what a failure does* is a thing this
/// crate's own tests can state: it stops, it names the gate, and it carries the
/// tail of what the gate printed so the sentence is diagnosable without going
/// to look for a log.
///
/// # Errors
/// A sentence whenever the gate did not exit successfully. There is no gate
/// whose failure is ignorable and no flag anywhere that makes one so.
fn whether_it_passed(named: &str, ok: bool, out: &str, err: &str) -> Result<(), String> {
    if ok {
        return Ok(());
    }
    let tail: Vec<&str> = err.lines().chain(out.lines()).collect();
    let from = tail.len().saturating_sub(25);
    Err(format!(
        "the gate `{named}` did not pass, so nothing was published:\n{}",
        tail.get(from..).unwrap_or_default().join("\n")
    ))
}

/// That this machine can run the gates at all, before it spends four minutes
/// discovering that it cannot.
///
/// Three preconditions, and every one of them is a thing that makes the gates
/// fail for a reason that has nothing to do with the change:
///
/// - **A BPF filesystem mounted at `/sys/fs/bpf`.** Every test that loads the
///   boundary pins to it, and WSL forgets the mount across a restart — so the
///   symptom is a kernel test panicking about a directory, three gates and
///   several minutes in, which reads like a broken boundary rather than an
///   unmounted filesystem. This is not hypothetical: it is what made a gate run
///   fail on 2026-09-07, and the publication that followed it went out because
///   the result was never looked at.
/// - **A toolchain the bridge can reach.** If `cargo` cannot be run where the
///   gates run, every gate fails identically and none of the messages says why.
/// - **The pinned nightly the BPF target needs.** Its LLVM is what matches the
///   `bpf-linker` on this machine, and a missing toolchain fails the last two
///   gates only — after the slow ones have already run.
///
/// **It reports rather than fixing.** A supervisor that mounted filesystems or
/// installed toolchains would be changing shared state on a machine another
/// worker is using, and `docs/hardware.md` asks the question of a person
/// instead.
///
/// # Errors
/// A sentence naming what is missing and the command that fixes it.
fn the_machine_is_ready(at: &Path) -> Result<(), String> {
    for (check, why) in READY {
        let said = asking(check, at)?.output().map_err(|it| {
            format!(
                "this machine could not be asked about {}: {it}",
                check.named
            )
        })?;
        if !said.status.success() {
            return Err(format!(
                "this machine is not ready to be gated, so nothing was published: {why}"
            ));
        }
    }
    Ok(())
}

/// What has to be true before a gate run means anything, and what to say when
/// it is not.
const READY: &[(Gate, &str)] = &[
    (
        Gate {
            named: "/sys/fs/bpf",
            program: "mountpoint",
            args: &["-q", "/sys/fs/bpf"],
            within: ".",
        },
        "/sys/fs/bpf is not a mounted BPF filesystem, so every test that loads the boundary \
         would fail for that reason rather than for anything in the change. Nothing was staged, \
         committed or pushed, and the work is where it was. \
         **Do not mount it in order to get a task out.** This mount is shared with whoever else \
         is testing on this kernel — a boot makes it and WSL forgets it across a restart — and a \
         checkout that remounts whenever it wants to publish is one that changes another \
         worker's ground while their tests are running. Ask for a coordinated maintenance \
         handoff, and let it be done once while nothing else is in flight.",
    ),
    (
        Gate {
            named: "the toolchain the gates run with",
            program: "cargo",
            args: &["--version"],
            within: ".",
        },
        "`cargo` could not be run where the gates run, so every gate would fail for the same \
         reason and none of them would say which. Check that the Linux side of this machine is \
         reachable and that its toolchain is installed.",
    ),
    (
        Gate {
            named: "the pinned nightly the BPF target needs",
            program: "cargo",
            args: &["+nightly-2026-06-01", "--version"],
            within: "crates/alo-bounding-kernel",
        },
        "the pinned nightly toolchain is not installed, so the BPF target's gates would fail \
         after the slow ones had already run. `crates/alo-bounding-kernel/rust-toolchain.toml` \
         names it, and its LLVM is what matches the `bpf-linker` on this machine.",
    ),
];

/// One gate, as a command this host can actually run.
///
/// # Errors
/// Whatever [`running`] answers with.
fn asking(gate: &Gate, at: &Path) -> Result<Command, String> {
    let args: Vec<String> = gate.args.iter().map(|&arg| arg.to_owned()).collect();
    running(at, gate.within, gate.program, &args)
}

/// Anything else this workstream needs run where the gates run.
///
/// The same bridge, exported, because a check that ran somewhere other than
/// where the gates ran would be a check of a different machine.
/// [`crate::evidence`] is the caller: it runs one named test on its own, in the
/// same toolchain, target directory and kernel the suite just passed in.
///
/// # Errors
/// A sentence when a Windows checkout is somewhere `wsl` cannot see.
#[cfg(windows)]
pub fn running(at: &Path, within: &str, program: &str, args: &[String]) -> Result<Command, String> {
    let within = as_wsl_sees_it(&at.join(within))?;
    let mut asking = Command::new("wsl");
    asking
        .args(["-d", "Ubuntu", "--", "bash", "-lc"])
        .arg(format!(
            "export PATH=\"$HOME/.cargo/bin:$PATH\"; \
         export CARGO_TARGET_DIR=\"{ITS_OWN_TARGET}\"; \
         export RUSTDOCFLAGS=\"-D warnings\"; \
         cd {within} && {program} {}",
            args.join(" ")
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

/// The same, run where it stands.
///
/// # Errors
/// None on a Linux host; the signature matches the Windows half so the caller
/// has one shape.
#[cfg(not(windows))]
pub fn running(at: &Path, within: &str, program: &str, args: &[String]) -> Result<Command, String> {
    let mut asking = Command::new(program);
    asking
        .current_dir(at.join(within))
        .args(args)
        .env("RUSTDOCFLAGS", "-D warnings");
    Ok(asking)
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected Ok is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A gate that did not exit successfully stops publication**, and says
    /// which gate and what it printed.
    ///
    /// There is no flag here that skips one and no result that is looked at and
    /// stepped over. The sentence carries the tail of the output because a
    /// refusal somebody has to go and investigate is a refusal somebody
    /// overrides.
    #[test]
    fn a_gate_that_failed_stops_and_says_which() {
        let refused = whether_it_passed(
            "the workspace's tests",
            false,
            "test result: FAILED. 0 passed; 5 failed\n",
            "error: test failed\n",
        );
        let Err(why) = refused else {
            panic!("a failed gate was treated as a pass")
        };
        assert!(why.contains("the workspace's tests"), "{why}");
        assert!(why.contains("nothing was published"), "{why}");
        assert!(why.contains("0 passed; 5 failed"), "{why}");
    }

    /// And a gate that passed says nothing at all, which is what lets the
    /// caller add it to the list and go on to the next one.
    #[test]
    fn a_gate_that_passed_is_simply_a_pass() {
        assert_eq!(whether_it_passed("formatting", true, "", ""), Ok(()));
    }

    /// **Every readiness check has a sentence a person can act on.**
    ///
    /// A precondition that failed with `false` and no explanation would send
    /// whoever hit it to read this file, and the one that actually bit this
    /// machine — an unmounted BPF filesystem — looks like a broken boundary
    /// until somebody says otherwise.
    #[test]
    fn each_readiness_check_names_what_to_do_about_it() {
        assert!(!READY.is_empty());
        for (check, why) in READY {
            assert!(!check.named.is_empty());
            assert!(
                why.len() > 60,
                "`{}` fails with too little to act on",
                check.named
            );
        }
        assert!(
            READY.iter().any(|(check, _)| check.named == "/sys/fs/bpf"),
            "the precondition this machine actually loses is not checked"
        );
    }
}
