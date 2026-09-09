# A loop that says whether it is running

- Date: 2026-09-09
- Workstream: model selection and configuration (`tools/kernel-loop`)
- Contributor: Claude Code
- Task: The backend loop's safeguards
- Status: **the supervisor changes are built and tested.** Whether a loop can
  usefully be *started* is a separate question, answered at the end and not
  favourably.

## Extended, not replaced

`tools/kernel-loop` already did most of what a backend loop needs, and this adds
to it rather than standing up a competitor. What was already there: it selects
the next task the plan does not mark done, launches at most one worker when
`ALO_KERNEL_LOOP_WORKER` names one, reads **the working tree and the handoff**
rather than anything the worker says, runs every gate, refuses evidence whose
test file is not part of the change or that selects zero tests, commits only the
named files under the repository's own identity, integrates `main`, re-gates the
combined tree, pushes, and has no force-push or reset anywhere in it.

`tools/dev-loop` remains the desktop worker's and is untouched, as is
`C:\dev\alo-os`.

### Completion is read from evidence, not from headings

Worth stating because it was nearly got wrong by a reader — me. `plan.rs` takes a
task as done when its section contains a line beginning `**Done,`, **not** from
its `Status:` line. Those disagree today: task 6's status says *ready* while its
body records it finished on 2026-09-08. The supervisor reads the body. A blocked
task is stepped over rather than counted complete.

## What was missing, and is now there

**A disk reserve before each build.** Twelve gibibytes, checked in the readiness
step that already runs before any gate. A build that runs out partway does not
fail as a build — it fails as a linker that cannot open a file, which reads like
a broken change and is not one. That happened on 2026-09-09 and cost an
afternoon. The check is `df` on the filesystem the checkout is on, so it measures
the disk that actually fills, and its message says **not** to delete anything
shared to get past it.

**A loop that says whether it is running.** `status` printed the journal, and a
journal's last line reads identically whether the loop is still working or was
killed an hour ago mid-sentence. It now answers *a loop is running here, process
N* first, and distinguishes three states — running, nothing, and **a lock left by
a process that is gone**. Liveness is asked of the operating system rather than
inferred from a heartbeat this program would have to keep writing: a heartbeat is
a second thing that can be wrong, and a loop legitimately busy for forty minutes
inside one worker is exactly when one would look like a death. When the question
cannot be asked at all, the answer is *alive* — the safe way to be wrong, because
it refuses to start a second loop rather than starting one beside a live one.

**Guarded restart.** A lock left by a dead process is taken over, and only after
the operating system says that process is gone — never while it is running, never
when the question failed. The takeover is written into the journal, because a
supervisor that silently took a lock is one nobody can tell had restarted after a
kill. Before this, a killed loop left a checkout that refused to start for a
process that no longer existed, which is how deleting locks by hand becomes a
habit.

**Ubuntu held up for the loop's lifetime.** One `sleep` inside the distribution,
started with the run and killed when the value owning it drops, including out of
a failure. WSL stops a distribution nothing is using and takes `/sys/fs/bpf` with
it, so a long run could pass readiness, publish, and then fail the next task for
a reason that has nothing to do with the work.

**It mounts nothing, restarts nothing and starts no service.** Keeping a
distribution from stopping is not the same as changing what is inside it, and the
mount is shared with whoever else is testing on this kernel. The readiness check
still asks for a coordinated handoff instead.

## Tested

33 in the supervisor, six of them new: no lock is no loop; a lock this process
holds is a running loop; a lock left by a dead process is not; such a lock is
taken over and the takeover reported; **a lock a living process holds is
refused**; and this process is alive — the one answer that must never be wrong,
because a false there would let a second loop start beside a running one.

The dead process is one the test **starts and waits for**, rather than a large
number guessed at: a guess can be somebody else's process, and the test would
then be asserting something about a program it knows nothing about.

The helper's test asks the operating system whether the process is gone *after*
the drop. Its first version compared a boolean with itself — `held || !held` —
which passes everywhere and means nothing; that is replaced.

## And whether a loop can be started

**Not usefully, and this is the honest part of the report.**

`run` selects the next executable task from `docs/autonomy/kernel-enforcement-plan.md`.
Tasks 1 to 9 all record `**Done,`; task 10 — what a credential does when a
session really ends — is blocked on a machine with `logind` and two sessions,
which this one is not. So a run would select nothing and stop with *the plan has
no executable task left*, which the supervisor is careful to distinguish from a
workstream being finished.

The question-refusal recording work named as the thing to continue was completed
earlier today, in its stated order: reader (`2c296ec`), turn (`91a9806`), daemon
(`29739b9`), against the existing record-file compatibility contract.

**Nothing was added to the plan to give the loop something to select.** Scope is
gated on `docs/features.md` and the current phase, the plan is a person's
document, and inventing an entry so a supervisor has work is precisely the
failure that gating exists to prevent.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible; this is development machinery.
**ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — the backend supervisor now has a disk reserve, live
status, guarded restart after a kill, and a lifetime-scoped WSL helper; the
backend plan has no executable task left.

**docs/autonomy/STATE.md** — `alo-kernel-loop status` distinguishes a running
loop from a stale journal; a lock left by a killed process is taken over and the
takeover recorded; builds are refused below a 12 GiB reserve.
