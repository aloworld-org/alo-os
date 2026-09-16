# The gate refusal on "a machine reachable on both networks", diagnosed

**Date:** 2026-09-16
**Workstream:** v0.5 — the local network
(`docs/autonomy/v0-5-the-local-network-plan.md`, task 27)
**Contributor:** Claude Code, in `C:\dev\alo-os-claude` — second worker on the task
**Status:** ready for integration
**Follow-up to:** `a-machine-reachable-on-both-networks.md`, which is this task's
report and which I did not change.

## Why this report exists

The first worker finished task 27 and handed it over. The supervisor refused it on
the gate `the workspace's tests`, twice, and parked nothing: their files stayed in
the tree. I was sent at the refusal with the instruction to fix exactly what it
said and to prefer the smallest change.

**The smallest change turned out to be no change to the code at all.** That is a
claim that has to be earned rather than asserted, so this report is what I ran to
earn it, and what the refusal really was. The work of this task is the first
worker's; what is new here is the diagnosis and a wider set of gates than the
first worker ran.

## What the refusal actually said, and what it did not

The refusal carried, as `tools/kernel-loop/src/gates.rs` builds it, the last
twenty-five lines of the gate's `stderr` followed by its `stdout`. Every one of
those twenty-five lines was a passing `alo-shell` test —
`direct_output::tests::…`, `display_resources::tests::…` — and **there was no
`test result:` line, no `FAILED`, and no panic among them**.

That shape is not a test failing. `cargo test --workspace` stops at the first
failing binary and prints a failure summary; what was captured is a test binary
cut off in the middle of printing, with the tail crowding the `stderr` lines that
would have named the cause out of the twenty-five-line window entirely.

`gates.rs` already documents, in its own module comment, that *refused twice* does
not mean *the work rather than the machine*: the second run of a gate does nothing
about a stale artefact. The same reasoning covers a machine that killed the gate
twice for the same reason.

## What the machine was doing

Two findings, and both are about this machine rather than about the change.

- **The Windows volume the build lives on had nine megabytes free.** `C:` was at
  508 GB of 508 GB used. The WSL2 Ubuntu root filesystem is a sparse
  `ext4.vhdx` on that volume: it reported 802 GB free *inside*, which is what the
  supervisor's twelve-gibibyte preflight measures, while the file holding it could
  not grow by a byte. The loop's own journal shows the matching symptom from the
  same period — `unable to read .cargo-ok file … Input/output error (os error 5)`
  while unpacking a crate from the registry.
- **The WSL virtual machine had been restarted since.** `/tmp` and `dmesg` were
  both empty of anything older than the current boot, and the build directory's
  usage had dropped by twenty-two gigabytes between the refusal and my first
  measurement. A `cargo test --workspace` interrupted by its virtual machine going
  away produces exactly the captured shape: output stopping mid-binary with no
  summary.

I freed about a gigabyte on `C:` from Windows' own reclaimable caches — the
Windows Update download cache, crash dumps, `C:\Windows\Temp`, and user temporary
files older than a day. **That is not enough**, and the rest of what would free
the volume is the owner's to decide, not a worker's: a 14 GB Docker Desktop disk,
a 20 GB Claude desktop virtual-machine bundle, an 11 GB archive on the Desktop,
and the two 40 GB and 34 GB cargo build directories inside WSL, one of which
belongs to the other checkout. **The supervisor's twelve-gibibyte preflight cannot
see this**, because it measures the filesystem inside the virtual disk and not the
volume the virtual disk is a file on. Whether that preflight should also measure
the host volume is a question for a separate task; I have not changed it here.

## What I ran

All from `/mnt/c/dev/alo-os-claude` in WSL Ubuntu (kernel
`6.18.33.2-microsoft-standard-WSL2`) with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-claude-bd192ccccbc3745b`, each in the
foreground and each waited on for its exit code.

| Command | Result |
| --- | --- |
| `cargo fmt --all` | clean; changed no file |
| `cargo clippy --workspace --all-targets -- -D warnings` | zero warnings |
| `cargo build --workspace --all-targets` | built |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` | zero warnings |
| `cargo test -p alo-agentd` | ok — 406 + 1 + 5 + 1 + 1 + 4 + 12 passed, 0 failed |
| `cargo test -p alo-nearby` | ok — 165 + 9 + 12 + 4 + 4 + 1 + 5 + 9 + 2 + 2 passed, 0 failed |
| `cargo test -p alo-shell` | ok — 317 + 262 + … passed, 0 failed, in 28.31 s |
| `cargo test -p alo-approving -p alo-asking -p alo-changing -p alo-corridor -p alo-remembering -p alo-saying -p alo-turn` | ok — 44 test binaries, 0 failed |

Three of these the first worker did not run, and they are the three that matter
for this refusal:

1. **`cargo test -p alo-shell`** — the crate whose output the refusal carried. It
   passes on this tree, on its own, in under half a minute. Whatever stopped the
   workspace suite in the middle of that binary, it was not one of its tests.
2. **The seven crates that depend on `alo-nearby`.** This change alters
   `alo-nearby`'s public surface — `Found::on_the_network`,
   `Waiting::where_the_other_was_heard`, a new argument on `Waiting::begun` and on
   `dialling::put` — and the first worker gated only the two crates they edited. A
   crate that consumes a changed surface is a crate the change touches, whether or
   not its files moved, so I gated all seven. They pass.
3. **`cargo build --workspace --all-targets` and the rustdoc gate**, which are the
   two gates that catch the registration nobody can know about: a module not
   collected in `lib.rs`, a doc link that does not resolve.

I did not run `cargo test --workspace` myself; the supervisor runs it, and the
instruction I was given is explicit that pre-empting it is not a worker's job.

## What I did not do, and why

**I changed no code, and I deleted no test.** Every test the first worker wrote
names something that exists, every crate is registered, every doc link resolves,
and every gate a worker owns is green. Deleting or weakening anything to make a
refusal go away would have been the wrong repair for a refusal whose cause was not
in the tree.

I also did not change the first worker's report. Their account of the kernel
measurements, the decisions and the acceptance is accurate against the code as I
read it, and `docs/autonomy/updates/README.md` says a correction is a new report
rather than an edit of somebody else's.

## The residual risk, stated plainly

If the volume fills again, the workspace suite will be killed again, and the
refusal will look the same. That is the one thing I can neither fix from inside
this task nor test for. What is in my hands — that the tree compiles, lints,
documents and tests clean across every crate this change can reach — is done and
shown above.

## Proposed updates to the shared documents

None beyond those the first worker's report already proposes. This report ticks
nothing on its own; it is the evidence that task 27's tree was refused by the
machine and not by its work.
