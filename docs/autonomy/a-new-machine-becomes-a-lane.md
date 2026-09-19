# A new machine becomes a lane

> **Current workflow, owner-approved 2026-09-18:**
> [SHARED_MAIN.md](SHARED_MAIN.md) supersedes direct-to-main, main-only,
> per-checkout build-cache and concurrent-build instructions below. Use one
> task branch and draft PR per task; progress pushes are allowed. Either PC
> may hold the shared integration turn and squash-merge its own task after all
> nine gates and acceptance pass on the exact combined tree. Existing direct-to-main publishers remain paused;
> the historical runner recipes below do not implement the new workflow.

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
| `v0-5-where-a-persons-settings-are-kept-plan.md` | **this PC, lane B (`alo-os-b`), active from 2026-09-18** - task 7, while hands-on tasks 2 and 7 wait on display identities | `alo-appearance`, `alo-dock`, `alo-shortcuts`, `alo-choosing`, `alo-changing`, `alo-kept` |
| `v0-5-applications-and-what-they-expect-plan.md` | **the Mac** | `alo-portals`, `alo-granted`, `alo-applications`, `alo-secrets`, and ADR 0040's change to `alo-capability`/`alo-remembering` |
| `v0-5-the-machine-keeps-itself-plan.md` | third PC, behind the installer plan — undo waits on the installer's task 11 for a filesystem that can snapshot | `alo-keeping-up` |
| `v0-5-documents-and-paper-plan.md` | **this PC (`alo-os-shell` checkout), from 2026-09-16** — the owner's three documents arrived, and this plan needs no virtual machine | `alo-printing`, `alo-opening`, `alo-converting` (new) |
| `v0-5-the-shell-plan.md` (tasks 7-14) | **this PC, lane A (`alo-os-claude`), from 2026-09-18** - assigned by the owner after the local-network plan finished; starts with task 8, whose lock-state dependency is done. Other tasks retain their dependencies | `alo-shell`, `tools/graphics-check` |
| `v0-5-access-and-language-plan.md` | **the Mac**, after applications | `alo-access`, `alo-conforming`, `alo-formats` (new), the answering-language clause of `alo-instructing` |
| `v0-5-models-a-person-adapts-and-subscribes-to-plan.md` | **the Mac**, after access and language | `alo-adapting`, `alo-hosted` (new) |
| `v0-5-software-and-the-web-plan.md` | **third PC, second loop** (`C:\dev\alo-os-2`) | `alo-software`, `alo-proxy`, `alo-adapters` (new) |
| `v0-5-the-broker-and-the-disk-plan.md` | third PC, second loop, after software and the web | `alo-broker`, `alo-encrypting` (new) |
| `v0-5-capture-and-the-room-plan.md` | **the Mac, from 2026-09-17**, tasks 3 to 7 — tasks 1 and 2 were published and the plan then sat untouched for twenty-six hours with no machine holding it. Its tasks 4, 5 and 7 waited on the devices plan's codec decision, which the same lane then took and wrote as ADR 0051 | `alo-capturing`, `alo-in-use` |
| `v0-5-the-session-and-the-displays-plan.md` | **third PC, first loop, from 2026-09-16** — it needs no virtual machine, and that loop waits on the installer plan's signed release and a machine with hardware virtualisation | `alo-locking`, `alo-sleeping`, `alo-displays`, `alo-notifying` (new) |
| `v0-5-hands-on-the-desktop-plan.md` | **this PC, lane B (`alo-os-b`), from 2026-09-17** — taken ahead of its queue because `alo-keyboards` is what the Mac's access-and-language tasks 3 and 4 wait on | `alo-dividing`, `alo-desktops`, `alo-keyboards` (new) |
| `v0-5-devices-and-media-plan.md` | **the Mac, from 2026-09-17** — taken for its task 1, the codec decision, which was blocking capture tasks 4, 5 and 7 on the same machine | `alo-sound`, `alo-bluetooth`, `alo-playing`, `alo-power`, `alo-cameras`, `alo-media-server` (all new) |

**Narrow printer producer contribution, authorized 2026-09-18.** The owner told
the third PC, "no you should do all the blockers by yourself so no need to lean
on the other pc". For **broker task 2, Printers, through the broker**, this
releases the preserved `alo-printing` additions: `printers_set_up`,
`Found::as_reported`, `Printer::as_reported`, `remove`, `make_default`,
`CannotChange`, their re-exports, contract comments and producer tests, plus the
measured broker socket authentication fix and its real-CUPS acceptance.
The documents plan records the exact released files for this task alone. The third PC integrates that producer contribution with its receiving
broker task publication; the report is
`docs/autonomy/updates/printers-through-the-broker.md`. The documents plan and
its completed work retain their owner, as do all shell and settings work. This
does not transfer the documents plan or authorize any other producer API.
Worker validation is deferred to the supervisor's nine gates and named task
acceptance checks; the ownership release changes none of those requirements.

**`alo-media-server` is owned here and read elsewhere.** It holds reaching the
rented media server and reading what it says, for the four crates that were each
doing it their own way — `alo-in-use` and `alo-capturing` (the capture plan's),
`alo-sound` and `alo-cameras` (this one's). It is listed here so that nobody adopts
it later as unowned: **the devices and media plan owns it, and a change to it is that
plan's lane's**. A crate that wants to read the media server reads this one rather
than starting a fifth copy.

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

## A machine unblocks itself

**Changed 2026-09-18, by the owner.** A machine no longer reports that it is
waiting on another machine and stops. It takes the thing blocking it.

- **A blocker in a plan nobody holds, or in one that has finished** — take it,
  and edit its row in the table above in the same commit, so the fleet can see
  who holds it now.
- **A blocker that is a decision** — write the ADR. You are the machine that
  understands why it matters; waiting for one with less context to decide it is
  worse rather than safer. Two exceptions: what needs the owner personally, and
  what needs a lawyer.
- **A blocker inside a crate another machine's lane is working right now** —
  the one case to coordinate. Say so, and take your next ready task meanwhile.
- **Never idle.** Stopping while work is available is the one outcome that is
  always wrong. If a lane empties its plan it says so *and takes the next thing*
  rather than waiting to be told.

**Why this is safe now and was not before.** Crate ownership existed because two
lanes editing one crate on a shared `main` corrupted a task on 2026-09-11. Under
`docs/autonomy/SHARED_MAIN.md` each task has its own branch and its own pull
request, so two machines in one crate now meet at merge time as a conflict
somebody can see and resolve. The table remains the record of who holds what; it
is no longer a reason to sit still.

**It works.** On 2026-09-17 the Mac was blocked on a codec decision and on the
media kinds `alo-playing` needed. It took both and published inside the hour,
while three other machines were idle waiting on each other.

When a machine takes a blocker, its report says **whose plan it came from and
why nobody was on it** — so a plan being quietly abandoned is visible, rather
than inferred later from a count.

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

### Where the checkout lives, which is worth more than every other setting here

**Do not gate a checkout that lives under `/mnt/c`.** WSL reads it across the
bridge to the Windows filesystem, file by file, in every gate. Copy it onto the
Linux filesystem instead — `rsync -a --exclude target /mnt/c/dev/<checkout>/
/root/<checkout>/`, twenty-one seconds for a 65 MB repository — and gate there.
Measured on the development PC on 2026-09-17, the same nine gates on the same
commit, with the same cores, linker and job count:

| Gate | under `/mnt/c` | on the Linux filesystem |
|---|---|---|
| fmt | 75s | 3s |
| clippy, warnings denied | 61s | 3s |
| the workspace's tests | 1664s | 988s |
| rustdoc | 88s | 56s |
| the BPF target's formatting | 72s | 6s |
| **all nine** | **1907s** | **1085s** |

Thirty-two minutes to eighteen, and the part that is not actually running tests
fell from about four minutes to ninety seconds. What is left is the
network-namespace tests, which take real time wherever the files sit.

The wall-clock is not the point. **A lane whose gate takes half an hour cannot
win a push race** against a fleet that pushes every ten to twenty minutes: it
gates, `main` moves, it rebases and gates again. One task lost four races that
way in a single morning and published nothing, while nothing at all was wrong
with it. Halving the gate is what makes a lane able to finish.

### The disk only grows unless you tell it not to

`ext4.vhdx` never shrinks by itself. Deleting files inside returns nothing to
Windows, and `diskpart compact vdisk` reclaims nothing at all on a disk that is
not sparse — measured at 155.5 GB before and after. A lane machine therefore
fills up and stops, and every symptom looks like something else: a worker that
died mid-task, a supervisor that could not even write its own lock file, gates
blaming the work twice in a row.

Do this once, before it happens: `wsl --shutdown`, then
`wsl --manage <distro> --set-sparse true --allow-unsafe`, then `fstrim -av`
inside the guest. Microsoft marks the conversion unsafe; with the distro shut
down and nothing inside it but build directories that rebuild, it is worth
taking. On this PC it turned **2.8 GB free into 53.5 GB** once a stale 65 GB
`target` was deleted — space that had been unreachable the day before.

Cargo target directories are what fills the disk: one per checkout, tens of
gigabytes each. A machine that changes where it builds should delete the
directory it stopped building in.

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

### Narrow wallpaper handoff, 2026-09-18

The owner assigned the approved Quiet Horizon wallpaper installation to this
PC together with shell task 8. Lane A may add the default-picture COPY mapping,
its alo-image acceptance test and the shared wallpaper lookup contract. All
other installer/image work remains with the third PC. The source artwork was
already published in c69a044; this handoff does not choose new artwork or
change the default appearance.
