# ADR 0130 — What takes the bracket before a person's own command

**Status:** **accepted, 2026-10-22, by the owner** — option **B**, asked for by
name after the three were laid out with what each costs. Proposed 2026-10-21 by
the Mac lane.

**What the acceptance settles and what it does not.** It settles that the
person's own shell asks a service, which takes the bracket — not a mark, and not
a kernel observer yet. It does not settle how a person turns it on; that is
[ADR 0127](0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md)'s,
which says a row in Settings under Terminal and which had not been built when
this was accepted.
**Context:** [ADR 0127](0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md)
(a person's own commands are bracketed in a session they asked for — accepted,
and this is the question it did not reach),
[ADR 0123](0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
(alo learns what a person typed from the shell),
[ADR 0045](0045-what-undoing-rewinds-to.md) (what undoing rewinds to),
`docs/contracts/shell-integration-marks.md` (the public form of the marks),
`crates/alo-terminal/shell-integration/alo.bash`,
`crates/alo-turn/src/the_bracket_a_turn_takes.rs`; task 5 of
`docs/autonomy/the-mac-lanes-order-while-the-owner-is-away.md`.

## The question

ADR 0127 settled **how a person asks** for snapshots — a row in Settings under
Terminal — and **when the rewind happens**. It did not settle **what takes the
snapshot**, because it read as an implementation detail. It is not one, and the
reason is a sentence in a published contract.

## Why this is a decision and not a detail

The only command boundary alo OS can see today is the shell's own OSC 133 mark,
and `docs/contracts/shell-integration-marks.md` says this of a mark, at line
107:

> a forged mark can make alo explain a command nobody typed, and it
> **cannot make alo *do* anything**.

**Taking a snapshot when a mark arrives makes that sentence false.** Any program
in the session can print the sequence, so any program could cause `btrfs
subvolume snapshot` to run — as often as it liked. The contract is a public
surface, third parties emit these marks already (kitty, iTerm2, WezTerm, VS
Code), and `CLAUDE.md`'s *contracts outlive code* puts a change to it behind a
decision rather than behind a commit.

There is a second, independent reason, and it would matter even with no
contract. **The mark is racy.** `alo.bash` says it plainly in its own comment:
*PS0 is printed after you press Enter and before the command runs.* Bash prints
the mark and then execs. alo reads the pseudoterminal on its own schedule, so a
snapshot begun when the mark is *seen* can begin after the command has already
written. *Your files are as they were before it ran* would be false, sometimes,
silently, and a reversibility overstated once is one nobody trusts again.

## What was measured, 2026-10-21, and whether it runs again

**All three ran once, by hand.** Nothing re-measures any of them today; the
first is the one an implementing change should turn into a test, and the last
section says how.

### 1. A `PS0` command substitution is synchronous and holds the command

On the lane's Lima VM — Linux, aarch64, `GNU bash 5.2.21` — on a real
pseudoterminal, with `PS0` set to `$(date; sleep 2; date)`:

| | |
|---|---|
| PS0 began | `…566429194682` |
| PS0 ended, 2 seconds later | `…568447531334` (+2.02 s) |
| the command ran | `…568452961502` (+0.0054 s) |

**The shell waited the full two seconds before running the command.** So work
started from `PS0` finishes *before* the command, by construction rather than by
timing — which is exactly the guarantee the mark cannot give.

A control was needed and the first two attempts lacked one. `PS0` with plain
text was checked to print before the command's output at all, because two
earlier probes reported *no substitution ran* for reasons that had nothing to do
with the question: the first ran on the lane's **macOS host, whose `bash` is
3.2**, where the same probe produced no `PS0` behaviour at all while the VM's
5.2 honoured it and documents it in its manual; `PS0` was added in `bash` 4.4,
which is the documented origin rather than something measured here. The second
passed `--norc` alongside `--rcfile`, so the file under test was never read.
*Both produced a confident negative about the wrong subject.*

A third instrument failed the same way while checking this very paragraph:
`man bash | grep -c PS0` on the host answered **0**, and so did the control
`grep -c PS1`, because the host has no `bash` manual at all. *A null result and
a negative result look the same, and only the control tells them apart.*

### 2. A snapshot costs single-digit milliseconds, and does not grow with the tree

`btrfs subvolume snapshot -r` on a loop-mounted filesystem in the same VM, 20
snapshots each:

| the subvolume | median | p95 | worst |
|---|---|---|---|
| 200 files, 40 MB | **2.3 ms** | 13.2 ms | 13.2 ms |
| 20,200 files and directories | **2.4 ms** | 30.1 ms | 30.1 ms |

**A hundredfold more files cost 0.1 ms.** A snapshot is a metadata operation on
the subvolume root, so this is the shape to expect; it is measured at two sizes
rather than argued from the shape. A person does not feel 2 ms before a command,
and the worst reading observed is 30 ms.

These are one workload on one loop device, and a real home is neither. Treat
them as a dated reading rather than a property.

### 3. There is still nothing that puts files back

ADR 0127 measured *make and remove, no restore* and the figure holds. Every
`btrfs` subvolume verb in the workspace:

| verb | where | what |
|---|---|---|
| `snapshot -r` | `alo-turn` | makes one |
| `delete --commit-after` | `alo-letting-go` | removes an expired one |
| `create`, `list`, `show` | tests only | fixtures |

`alo-changing-undo` was checked rather than assumed from its name: it is the
Settings pane about how far back undo reaches, and the only place it names
`subvolume` is a test asserting its wording **avoids** naming machinery.

## The options

### A — bracket when the OSC 133 mark arrives

**Rejected.** It needs the contract sentence above rewritten so that a mark can
cause a privileged filesystem operation, and it is racy for the independent
reason in *Why this is a decision*. Either fault alone is disqualifying.

### B — the shell's own `PS0` asks a service, which takes the bracket — **recommended**

`alo.bash` gains a `PS0` substitution calling a small command, which asks
`alo-agentd` over the socket that is already one per signed-in person; the
daemon holds the bracket and answers. The command runs to completion before the
person's command, as measured.

**Why this is the shape and not a new one.** It is
[ADR 0125](0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md)'s
accepted pattern — *a command does not do the privileged thing, it asks a
service that can* — and `alo.bash` already ships exactly this: `try` is a shell
function the snippet defines that calls `alo-try`. So the file's structure, the
precedent and the authority model are all in place.

**The marks contract is untouched.** A mark still causes nothing. What causes a
snapshot is the person's own shell running a command as the person, which is the
ordinary way anything happens on this machine.

**What it costs, plainly.** Three things, and none is hidden:

- **It needs the integration installed.** A person who has not added `alo.bash`
  gets no snapshots, and the Settings row must say so rather than appear to work.
  #916 is the row that installs it, so the two belong in one place in Settings.
- **A new verb on the daemon's socket is a public surface**, so the contract
  gains an entry and changes additively. This is the part that needs the
  decision; the rest is implementation.
- **Every command waits for the daemon.** Measured above as 2 ms of snapshot
  plus a socket round trip. If the daemon is not answering, the command must run
  **anyway**, and the person is told the session is no longer undoable rather
  than left believing it is. ADR 0045 term 3 points the same way, and its words
  are *a machine too full to snapshot still runs the **turn***. **It says turn,
  not command**, so applying it here is a reading of the term rather than the
  term itself — which is one more thing for the owner to accept or reject rather
  than something to assume. ADR 0127 quotes it as *command* already.

### C — a non-forgeable boundary from the kernel

Observe `execve` and bracket there. **Rejected for now, not never.** It is the
only option that works with no shell integration at all and cannot be forged by
a program printing bytes. It is also much larger: there is no such observer in
the workspace — measured by looking for a crate and for the hooks, and every
`bpf` reference in `crates/**/*.rs` is in `alo-agentd`'s bounding, which is law
2's sealed box rather than an exec monitor — and `docs/features.md`'s ★ *ask
what a command actually did*, which wants the same machinery, has no crate and
no manifest naming one. Worth revisiting when that is built, and
the stored line B writes would then be read by it unchanged.

## What this would not decide

- **The restore.** Measurement 3 stands: nothing puts files back. B makes
  snapshots exist and a person able to ask for them; *choose one, and your files
  are as they were* still needs the second half ADR 0127 named, and
  `docs/features.md` must not be marked built on the strength of this.
- **The words on the row**, which are the design file's.
- **Whether a session's snapshots expire differently** — ADR 0045's window
  applies unchanged, as ADR 0127 already said.

## If B is accepted, what the implementing change owes

**Measurement 1 becomes a test rather than a paragraph.** A test that runs
`bash` on a pseudoterminal with a `PS0` that records an order, and asserts the
bracket's record precedes the command's — so the property is checked on every
build instead of resting on one evening in a VM. It needs `bash` 4.4 or newer,
which is why it belongs beside the other Linux-only tests rather than in the
unit suite: the lane's own host is `bash` 3.2 and would skip it silently.

And the refusal path is the one that matters: **a session whose bracket could
not be taken runs the command and says it is not undoable.** Tested as
carefully as the success, because that is the path a person meets when their
disk is full.
