# Kernel supervisor runs its gates through WSL

- Date: 2026-09-07
- Workstream: kernel enforcement (`tools/kernel-loop`)
- Contributor: Claude Code, kernel-enforcement workstream
- Status: ready for integration

## What changed and why

A follow-up to `kernel-enforcement-plan-and-supervisor.md`, found by running the
supervisor on a real task rather than by reading it.

**`git push` from inside WSL hangs forever.** Everything before it looks
healthy — cloning and fetching a public repository need no credentials — so the
first symptom is a supervisor sitting on a `git-remote-https` that will never be
answered, with a task gated, committed and unpublished. The credentials on this
machine live on the Windows side.

So the split is now explicit and is the only arrangement that works here:

- **the loop is a Windows program**, because that is where `git` finds its
  credentials;
- **every gate is handed to `wsl`**, because that is where a kernel, a BPF
  target and the pinned nightly are.

On a Linux host each gate runs directly and there is nothing to bridge, which is
the `#[cfg(not(windows))]` half.

`CARGO_TARGET_DIR` is named rather than inherited: the environment a Windows
process hands to `wsl` is not the one the gates need, and two contributors
compiling the same workspace into one directory would overwrite each other's
artefacts.

### And a bug in the supervisor, twice

The check that refuses to publish files no task named parses `git status
--porcelain`. It counted three characters before the path — two status columns
and a space — which is right for the format and wrong here, because the loop
trims the whole output and that eats the leading space of the first line. It
reported a changed file called `rates/alo-bounding/src/lib.rs`.

The obvious fix, splitting at the first space, broke the other lines: an
unstaged line *starts* with a space, so the split lands at nothing. Trimming
each line and then dropping its status field is the one reading that survives
all four spellings — ` M`, `M `, `MM` and `??`.

Both mistakes were made here and both were found by running the loop. Neither
would have been found by reading it, which is the argument for the verification
step rather than a claim about care.

## Acceptance criteria and actual verification

Acceptance: the supervisor publishes a real task end to end on this machine.

Executed:

- The supervisor's own gates on Windows: `cargo fmt --all --check`, `cargo
  clippy --all-targets -- -D warnings`, release build — all clean.
- The supervisor was run on a real task and **passed all six repository gates**
  through WSL, in 234 seconds: formatting; clippy with warnings denied; the
  workspace's tests; rustdoc with warnings denied; the BPF target's formatting;
  the BPF target's clippy on the pinned nightly. It then committed locally.
- That run is what exposed the hang. The commit it made was never lost: the loop
  has no `reset`, no `clean` and no `checkout --`, so stopping it left the work
  exactly where it was, which is the property those absences exist for.

**The hang was in `git push`, and the process stopped was the loop's own.** No
other worker's process was stopped, no pin removed, no service restarted and no
host-wide networking changed. The stale lock left by killing the process was
removed by hand, which is what the lock's own message tells a person to do.

**Windows:** the supervisor itself is built and linted on Windows, and that is
this change's platform. The repository gates it drives are Linux, as before.

**WSL is development evidence and never certified-hardware acceptance.**

## Decisions and approvals

No approval needed. This is contributor tooling; no crate that ships changed, no
grant, capability, contract or ADR is touched.

The decision recorded: **the supervisor does not configure git credentials in
WSL.** Storing a credential inside the Linux side to make a push work there
would be a change to how this machine holds a secret, for a convenience, and the
Windows-side split costs nothing.

## Remaining gaps and hardware obligations

- A failure path still not induced: a genuine mid-attach kernel refusal.
- Everything in `docs/autonomy/kernel-enforcement-plan.md`'s *Incomplete* table.
- All physical acceptance.

## Proposed shared-document updates

**CHANGELOG.md** — nothing; contributor tooling.

**ROADMAP.md**, **QUEUE.md** — nothing.

**docs/autonomy/STATE.md** — reference this report beside the plan. The fact
worth carrying for anybody else automating publication on this machine: **a push
from inside WSL hangs on credentials it has no helper for**, and the answer is
to run git on the Windows side and hand the gates to `wsl`.
