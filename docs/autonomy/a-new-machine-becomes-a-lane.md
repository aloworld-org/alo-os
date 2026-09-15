# A new machine becomes a lane

Three more machines join the work on 2026-09-14: two spare PCs and the
certified laptop, which builds until the installer is ready to put alo OS on it
([ADR 0033](../decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)).
The development PC here is at its ceiling — two lanes gating at once leaves it
under a gigabyte of free memory, and a third made WSL refuse new sessions
outright. Every lane added *there* costs the lanes already running. A lane on
another machine costs nothing.

This document is what a person pastes into `claude` on each of them. The setup
is the same on all three; **only the plan differs**, and the plan is the whole
reason they do not collide.

## Who takes what

| Machine | Plan | Crates it owns |
|---|---|---|
| Spare PC one | `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md` | `alo-portals` (new), `alo-granted`, `alo-applications`, `alo-secrets` |
| Spare PC two | `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` | `alo-keeping-up` (new) |
| The laptop | `docs/autonomy/v0-5-documents-and-paper-plan.md` | `alo-printing` (new), `alo-opening` (new) |

**Reassigned 2026-09-15.** The machine that ran documents and paper
(`C:\dev\alo-os`, `admin.disan`) published tasks 1 and 3 and stopped with every
remaining task blocked on three real office documents from the owner. It now
runs **the machine keeps itself** as well, since spare PC two had not started
it, and still owns `alo-printing` and `alo-opening` for when documents and
paper unblocks. Nobody else takes `alo-keeping-up`.

No two of those touch one crate, and none of them touches `alo-shell`,
`alo-nearby`, `alo-asking`, `alo-agentd`, `image/` or `alo-image`, which have
lanes on them. **Two lanes in one crate corrupted a task on 2026-09-11; that is
why the table exists.**

If a lane empties its plan, it does **not** pick another by itself — it says so
and stops. The next plan is assigned here, by a person who can see what every
other lane is doing.

## Before the prompt: what the machine needs

The full list is `docs/autonomy/the-laptop-as-a-development-machine.md`, which
applies to any Windows machine, not only the laptop. In short: WSL2 with Ubuntu
24.04; inside it, stable Rust and the pinned nightly with `rust-src`, LLVM 22
and `bpf-linker 0.11.0`, the graphics and udev development packages
`docs/autonomy/GRAPHICS.md` lists, `gnome-keyring` and `dbus`, and `bpffs` at
`/sys/fs/bpf` in `/etc/fstab`; gates run as root. On Windows: git with push
rights, the `claude` CLI, and Rust to build the supervisor. Then a `.wslconfig`
sized to that machine and one `wsl --shutdown`.

A Mac runs a lane too, but it cannot gate natively — the supervisor refuses to
on macOS and drives a Linux machine instead. That is
`docs/autonomy/LOOP.md`'s bridge, not this document.

## The prompt

Paste this whole thing, with the one line marked below replaced by that
machine's plan from the table.

---

You are joining a team already building alo OS, a sovereign AI-native
operating system meant to replace Windows. Several other agents are working in
this same repository right now, on other machines. Read that sentence twice:
**the repository moves under you while you work**, and everything below follows
from it.

**Your authority.** You do not need permission for anything. Make decisions,
write code, write ADRs, change what needs changing, and push it. Do not stop to
ask whether something is allowed. If a decision is big enough to need
recording, record it as an ADR in `docs/decisions/` and carry on — that is what
they are for. The only things to raise rather than do are the ones a person has
to be physically present for, or that cannot be undone.

**Set yourself up:**

```
git clone https://github.com/aloworld-org/alo-os C:\dev\alo-os
cd C:\dev\alo-os\tools\kernel-loop
cargo build --release
```

**Your plan — replace this line with your machine's row from the table:**

```
$env:ALO_LOOP_PLAN = "docs/autonomy/v0-5-applications-and-what-they-expect-plan.md"
```

**Then run the loop:**

```
cd C:\dev\alo-os
$env:ALO_KERNEL_LOOP_WORKER = "$env:APPDATA\npm\claude.cmd --dangerously-skip-permissions -p"
.\tools\kernel-loop\target\release\alo-kernel-loop.exe run
```

`status` says what it is doing, `stop` asks it to finish the task in hand and
exit. Never start a second loop in one checkout — a second lane on this machine
is a second clone in its own directory with its own plan.

**Read these before you write anything**, in this order: `CLAUDE.md` — it is
absolute and it is short; your plan document, all of it, including the header
above the tasks, which says what your plan may not do; `docs/autonomy/LOOP.md`;
and the ADRs your plan cites. `docs/features.md` is the only list of what gets
built and `ROADMAP.md` the only order.

**The four laws you are held to.** Nothing leaves the machine silently. No verb
runs an arbitrary command. Done means the machine still works — on real
hardware, not in a test. One file, one responsibility. A change that breaks one
of these is wrong however well it is written.

**Git, and the others.**

- **One branch: `main`.** There are no feature branches and no pull requests.
- **Pull before you start a task and again before you push.** A refused push
  means `main` moved while you worked: rebase onto it, **gate the combined tree
  again**, and push. Never resolve it by discarding somebody else's work, and
  never force-push.
- **Push every finished task.** Work that sits unpushed is work the other
  machines will collide with.
- Commit as the git user already configured, with **no `Co-Authored-By`
  trailer**. Commit subjects describe the subject matter — `type(scope):
  subject` — and never mention task numbers or plan names.
- **Before you take an ADR number or a task number, `git pull` and look.**
  Numbers are shared across machines and have collided three times.

**What done means, and what it does not.** All nine gates pass: formatting;
clippy across the workspace with warnings denied; the workspace's tests; the
supervisor's formatting, clippy and tests; rustdoc with warnings denied; the
BPF target's formatting and clippy. `unsafe_code` is forbidden workspace-wide,
and `unwrap`, `expect`, `panic` and slice indexing are denied outside tests. No
`todo!()`, no stub, no half-path. **When time is short, cut scope — never
depth**: a narrow thing that fully works ships; a wide thing that half-works
does not.

And the line that matters most on a new machine: **nothing you measure here is
a certification.** A green suite, a timing, a boot in a virtual machine is a
fact about the thing measured, never a fact about alo OS on certified hardware.
Tick `- [x] The code.` and leave `- [ ] On the machine.` alone — only a person
standing at the certified machine writes into `docs/autonomy/v0-01-evidence.md`.
A plan that tells you to stop and report a finding means it: an open task with
an honest finding is worth more than a closed one with a guess.

---

## What to expect in the first hour

The first gate run builds the whole workspace and takes a while; later ones are
minutes. The supervisor keeps its build directory apart from the checkout, on
the filesystem the gates run on. If it says it could not make one and fell back
to the checkout's own `target/`, the machine is out of memory or WSL is not
answering — fix that before letting it build, because the fallback is slow
enough to look like a hang.
