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

## Every v0.5 plan, and who has it — as of 2026-09-15

| Plan | Machine | Crates it owns |
|---|---|---|
| `v0-5-the-local-network-plan.md` | this PC, lane A (`alo-os-claude`) | `alo-nearby`, parts of `alo-agentd`/`alo-turn`/`alo-egress`/`alo-bounding*` for pairing |
| `v0-5-the-installer-plan.md` | **third PC, first loop, from 2026-09-16** — it needs 50 GB free for the three virtual-machine tasks, which the development PC has not | `alo-installer`, `alo-installing`, `image/`, `alo-image`, `.github/workflows/` |
| `v0-5-where-a-persons-settings-are-kept-plan.md` | this PC, lane B (`alo-os-b`), when a slot frees | `alo-appearance`, `alo-dock`, `alo-shortcuts`, `alo-choosing`, `alo-changing`, `alo-kept` |
| `v0-5-applications-and-what-they-expect-plan.md` | **the Mac** | `alo-portals`, `alo-granted`, `alo-applications`, `alo-secrets`, and ADR 0040's change to `alo-capability`/`alo-remembering` |
| `v0-5-the-machine-keeps-itself-plan.md` | third PC, behind the installer plan — undo waits on the installer's task 11 for a filesystem that can snapshot | `alo-keeping-up` |
| `v0-5-documents-and-paper-plan.md` | **this PC (`alo-os-shell` checkout), from 2026-09-16** — the owner's three documents arrived, and this plan needs no virtual machine | `alo-printing`, `alo-opening`, `alo-converting` (new) |
| `v0-5-the-shell-plan.md` (tasks 7–14) | **paused 2026-09-16**, its settings task published and every remaining task blocked on another plan's crates; the next machine to free a lane takes it, and it stays the one owner of `alo-shell` | `alo-shell`, `tools/graphics-check` |
| `v0-5-access-and-language-plan.md` | **the Mac**, after applications | `alo-access`, `alo-conforming`, `alo-formats` (new), the answering-language clause of `alo-instructing` |
| `v0-5-models-a-person-adapts-and-subscribes-to-plan.md` | **the Mac**, after access and language | `alo-adapting`, `alo-hosted` (new) |
| `v0-5-software-and-the-web-plan.md` | **third PC, second loop** (`C:\dev\alo-os-2`) | `alo-software`, `alo-proxy`, `alo-adapters` (new) |
| `v0-5-the-broker-and-the-disk-plan.md` | third PC, second loop, after software and the web | `alo-broker`, `alo-encrypting` (new) |
| `v0-5-capture-and-the-room-plan.md` | **spare PC two**, first — its task 1 unblocks two other plans | `alo-capturing`, `alo-in-use` (new) |
| `v0-5-the-session-and-the-displays-plan.md` | spare PC two, second | `alo-locking`, `alo-sleeping`, `alo-displays`, `alo-notifying` (new) |
| `v0-5-hands-on-the-desktop-plan.md` | spare PC two, third | `alo-dividing`, `alo-desktops`, `alo-keyboards` (new) |
| `v0-5-devices-and-media-plan.md` | spare PC two, fourth — or whichever machine empties first | `alo-sound`, `alo-bluetooth`, `alo-playing`, `alo-power` (new) |

**Why this division.** The Mac holds the small model and runs no virtual machine
well, so it takes the plans that need a model and no hardware: access and language
(the agent answering in each language is measured on the small model) and models a
person adapts. The third PC has the memory for two loops and already runs a virtual
machine, so its second loop takes software and the broker, whose tests use virtual
disks. This PC stays on the installer, which is the critical path, and inherits the
shell plan because it already gates the compositor. Capture goes first on spare PC
two because its task 1 is what the session plan's notifications and the devices
plan's camera wait on. **If spare PC two is not started, its four plans are the queue
for whichever machine empties its own list first.**

The eight plans written on 2026-09-15 own **new crates only** (plus one clause of
`alo-instructing`, whose plan had finished), so any of them can go to any free
machine without meeting a lane. The one exception is the shell plan: exactly one
machine may hold it at a time. **Assigning a plan means editing its row here in the
same commit that starts the lane**, so that the table is always the truth.

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

### Sizing that machine, so the gates are not the slow part

A lane spends most of its wall-clock inside the nine gates, and on 2026-09-16
this PC was found to have been running them on under half of itself: the
`.wslconfig` said `processors=6`, above a comment claiming that was "six of the
eight cores", on a machine with fourteen. Check the real number rather than the
one in the file — `nproc` inside the guest against the host's own count —
because that arithmetic quietly taxes every task the machine ever runs.

Two settings, kept apart on purpose:

- **Cores** (`processors` in `.wslconfig`) go to all but two, leaving the host
  git, the editor and the supervisors themselves. Cores are cheap: the ones
  above the job count are used *inside* each `rustc` and by the linker.
- **Concurrent compiles** (`CARGO_BUILD_JOBS` in the environment a gate runs
  in) are what set peak memory, and memory is what actually breaks a machine —
  `Wsl/Service/0x8007274c`, the guest paging, a finished task losing its lane.
  Pin it, at about half the cores where two lanes share one guest.

Raising the jobs along with the cores raises both, and that is the change that
ran this machine out of memory before. Raise the cores; pin the jobs.

Install **mold** (`apt install mold`) and give the gates
`RUSTFLAGS="-C link-arg=-fuse-ld=mold"`. Linking is the serial tail of every
crate and very nearly the whole of a rebuild that changed one line. Unset
`RUSTFLAGS` for the two BPF gates: that target is not linked by anything of
ours, and the flag would be handed to a linker that is not there.

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
