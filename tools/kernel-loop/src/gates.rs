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
//! Which directory that is, and how much room is in it, belong to
//! [`crate::where_it_builds`] — the gates ask it once and are handed the same
//! answer for the whole of a run.
//!
//! # A compile error a gate reports is not always in the change
//!
//! Task 26 was refused twice by `the workspace's tests` for a method that was
//! in the tree: `no method named numbers found for struct Accounts`, raised
//! while compiling the crate that calls it. It was not the worker's code. In
//! the same target directory, `cargo check --workspace --all-targets`, `cargo
//! clippy --all-targets` and `cargo test -p <crate>` all passed on the same
//! tree, and only `cargo build --workspace` and `cargo test --workspace`
//! failed — a **cached unit of the callee that Cargo considered fresh and
//! which predated the method**. `cargo clean -p alo-accounts` and the same
//! command passed. Why the fingerprint stayed fresh across a `/mnt/c` source
//! edit is not established here and is not claimed.
//!
//! What follows from it, and it is the reason this is written in the file that
//! prints the refusal: the second run of a gate does nothing about a stale
//! artefact, so *refused twice* does not mean *the work rather than the
//! machine*. A worker handed a compile error naming something their own
//! `-p` gates compile happily should `cargo clean -p` the crate that owns the
//! missing item and run the whole-workspace **build** before concluding
//! anything about the source. Nothing here is weakened for it: the gate is
//! right to refuse a tree it cannot build, whoever broke it.

use std::path::Path;
use std::process::Command;

use crate::where_it_builds;

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
pub fn all_of_them(at: &Path, touched: &[String]) -> Result<Vec<String>, String> {
    the_machine_is_ready(at)?;
    forget_what_was_built_of(at, touched);
    let mut passed = Vec::new();
    for gate in EVERY_GATE {
        // **A gate that fails is run once more before it is believed.**
        //
        // Two loops share this machine, and not every test is indifferent to
        // how much of it it gets. `window_controls::name_fallback` failed in
        // both lanes tonight and passed alone on the same commit, 262 of 262 —
        // timing under contention rather than a defect. The cost of believing
        // it was two tasks parked for a reason that had nothing to do with
        // them, in a crate neither lane touches.
        //
        // **This does not make a failure ignorable.** A gate that fails twice
        // fails, and the second run is the whole of the tolerance: a real break
        // fails both times, and no flag anywhere turns a gate off. What it
        // removes is the transient — the one kind of failure that says nothing
        // about the work.
        let mut refused = match ran(gate, at)? {
            Ok(()) => {
                passed.push(gate.named.to_owned());
                continue;
            }
            Err(why) => why,
        };
        if ran(gate, at)?.is_ok() {
            passed.push(format!("{} (on the second run)", gate.named));
            continue;
        }
        refused.push_str(
            "\n\nRun twice and refused both times, so this is the work rather than the machine.",
        );
        return Err(refused);
    }
    Ok(passed)
}

/// Drop what was built of the crates this change touched, before anything is
/// gated.
///
/// **Four refusals in one day were this and not the work.** A gate reported
/// `no method named numbers found for struct Accounts` while the method was in
/// the tree and had been for half an hour; `cargo check -p`, `cargo clippy` and
/// `cargo test -p` all passed on the same files, and only the whole-workspace
/// build failed. `cargo build --workspace -v` named it: the crate was handed a
/// cached `.rmeta` from before the method existed, and Cargo held that unit
/// fresh, so no `rustc` ran for it at all. Twice that parked finished work, and
/// once it stopped a run.
///
/// A whole-workspace build resolves features differently from `cargo build -p`,
/// so the two are **different units of the same crate** — which is exactly why
/// a worker's own per-crate gates cannot see this and the supervisor's
/// whole-workspace gate can. The checkout also lives on `/mnt/c`, where every
/// source mtime crosses drvfs from Windows and Cargo's freshness test is known
/// to be delicate.
///
/// So the units for the crates a task actually changed are dropped, and only
/// those: a whole `cargo clean` would throw away an afternoon of compilation
/// belonging to forty crates nobody touched, and the tax it is paying off is
/// seconds per task.
///
/// **Best effort on purpose.** A clean that cannot run is not a reason to
/// refuse to gate — the gates are the check, this is only an attempt to make
/// their answer be about the work. Whatever it says is dropped.
fn forget_what_was_built_of(at: &Path, touched: &[String]) {
    for crate_named in every_crate_among(touched) {
        let args = ["clean".to_owned(), "-p".to_owned(), crate_named];
        if let Ok(mut asking) = running(at, ".", "cargo", &args) {
            drop(asking.output());
        }
    }
}

/// The crates a list of changed files belongs to, each named once.
///
/// A path is a crate's when it begins `crates/<name>/`; everything else — a
/// document, an image file, a plan — belongs to no crate and is skipped. The
/// directory name is the crate name in this repository, which `Cargo.toml`'s
/// `members` glob is what makes true.
fn every_crate_among(touched: &[String]) -> Vec<String> {
    let mut named: Vec<String> = Vec::new();
    for path in touched {
        let path = path.replace('\\', "/");
        let Some(rest) = path.strip_prefix("crates/") else {
            continue;
        };
        let Some((crate_named, _)) = rest.split_once('/') else {
            continue;
        };
        if !crate_named.is_empty() && !named.iter().any(|already| already == crate_named) {
            named.push(crate_named.to_owned());
        }
    }
    named
}

/// Run one gate, and say whether it passed without deciding what that means.
///
/// The outer `Result` is *the gate could not be run at all*, which is a
/// different thing from *the gate ran and refused* — and is not retried,
/// because a missing toolchain does not become present on a second attempt.
fn ran(gate: &Gate, at: &Path) -> Result<Result<(), String>, String> {
    let said = asking(gate, at)?
        .output()
        .map_err(|why| format!("`{}` could not be run: {why}", gate.named))?;
    Ok(whether_it_passed(
        gate.named,
        said.status.success(),
        &String::from_utf8_lossy(&said.stdout),
        &String::from_utf8_lossy(&said.stderr),
    ))
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
/// Four preconditions, and every one of them is a thing that makes the gates
/// fail for a reason that has nothing to do with the change:
///
/// - **Room to build in, on the filesystem the build will actually use.** That
///   is [`where_it_builds::there_is_room`], and it is asked first because it is
///   the cheapest and because it is the one that used to be asked about the
///   wrong filesystem.
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
    where_it_builds::there_is_room(at)?;
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
///
/// Room on the disk is **not** in this list, because it is not a constant: it
/// depends on where this checkout builds, and asking it of the wrong filesystem
/// is what parked a finished task on 2026-09-11.
/// [`where_it_builds::there_is_room`] owns both halves of that question.
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
pub fn running(at: &Path, within: &str, program: &str, args: &[String]) -> Result<Command, String> {
    let building_in = where_it_builds::chosen(at).directory.clone();
    bridged(at, within, program, args, building_in.as_deref())
}

/// The same, with no build directory named at all.
///
/// For the two questions that have to be asked **before** there is an answer
/// about where this checkout builds — whether that directory can be made, and
/// how much room is on the filesystem it would be on. Handing them a target
/// directory would be asking [`where_it_builds::chosen`] for the answer it is
/// in the middle of working out.
///
/// # Errors
/// Whatever [`running`] answers with.
pub fn without_a_target_directory(
    at: &Path,
    within: &str,
    program: &str,
    args: &[String],
) -> Result<Command, String> {
    bridged(at, within, program, args, None)
}

/// One command, as this host runs it, building where it is told to.
///
/// # Errors
/// A sentence when a Windows checkout is somewhere `wsl` cannot see.
#[cfg(windows)]
fn bridged(
    at: &Path,
    within: &str,
    program: &str,
    args: &[String],
    building_in: Option<&str>,
) -> Result<Command, String> {
    let within = as_wsl_sees_it(&at.join(within))?;
    // Unquoted on purpose where it is absent, so that a command with no build
    // directory carries no empty variable either. `where_it_builds` guarantees
    // the path has nothing in it a shell would have to be protected from.
    let target = building_in.map_or_else(String::new, |directory| {
        format!("export CARGO_TARGET_DIR=\"{directory}\"; ")
    });
    let mut asking = Command::new("wsl");
    asking
        .args(["-d", "Ubuntu", "--", "bash", "-lc"])
        .arg(format!(
            "export PATH=\"$HOME/.cargo/bin:$PATH\"; \
         {target}export RUSTDOCFLAGS=\"-D warnings\"; \
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
fn bridged(
    at: &Path,
    within: &str,
    program: &str,
    args: &[String],
    building_in: Option<&str>,
) -> Result<Command, String> {
    let mut asking = Command::new(program);
    asking
        .current_dir(at.join(within))
        .args(args)
        .env("RUSTDOCFLAGS", "-D warnings");
    if let Some(directory) = building_in {
        asking.env("CARGO_TARGET_DIR", directory);
    }
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

    /// **No readiness check measures the filesystem the checkout is on.**
    ///
    /// That is the defect this list used to carry: a `df` of `.` refusing a run
    /// over a shortage on a drive the build never wrote to. The room a build
    /// needs is asked of the directory the build uses, in
    /// [`crate::where_it_builds`], and a shell `df` reappearing here would be
    /// the same mistake made again.
    #[test]
    fn no_readiness_check_asks_this_drive_how_much_room_a_build_has() {
        for (check, _) in READY {
            assert!(
                !check.program.contains("df") && !check.args.iter().any(|arg| arg.contains("df ")),
                "`{}` measures a filesystem the build may not be using",
                check.named
            );
        }
    }

    /// **The crates a change touched are the ones whose builds are dropped** —
    /// and nothing else is, because a whole `cargo clean` would throw away an
    /// afternoon of compilation belonging to crates nobody edited.
    #[test]
    fn only_the_crates_a_change_touched_are_forgotten() {
        let touched = vec![
            "crates/alo-accounts/src/store.rs".to_owned(),
            "crates/alo-accounts/tests/a_person_signs_in.rs".to_owned(),
            "crates/alo-sessiond/src/machine.rs".to_owned(),
            "docs/autonomy/updates/something.md".to_owned(),
            "ROADMAP.md".to_owned(),
            "tools/kernel-loop/src/gates.rs".to_owned(),
        ];

        assert_eq!(
            every_crate_among(&touched),
            ["alo-accounts", "alo-sessiond"],
            "a crate is named once however many of its files changed, and a              path outside crates/ belongs to no crate"
        );
    }

    /// And the shapes that are not a crate path, which must not produce one:
    /// an empty list, a bare directory, and Windows' own separator, which is
    /// what a handoff written on this machine can carry.
    #[test]
    fn a_path_that_names_no_crate_produces_no_crate() {
        assert!(every_crate_among(&[]).is_empty());
        assert!(every_crate_among(&["crates/".to_owned()]).is_empty());
        assert!(
            every_crate_among(&["crates/alo-keeping".to_owned()]).is_empty(),
            "a directory with nothing under it does not name a crate to clean"
        );
        assert_eq!(
            every_crate_among(&[r"crates\alo-keeping\src\lib.rs".to_owned()]),
            ["alo-keeping"],
            "a handoff written on Windows names the same crate as one written              anywhere else"
        );
    }
}
