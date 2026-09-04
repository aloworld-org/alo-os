# The alo OS build loop

One iteration builds **one queue item**, completely, and stops. A supervisor
runs iterations until the journal says the queue is done or that something is
wrong.

The loop exists because the work in `ROADMAP.md` is long and mostly independent,
not because it is unsupervised. Every iteration ends in a commit that met the
gate in `CLAUDE.md`, or in a halt that says why it could not.

## There are no tracks here

The supervisor's prompt names a "track", because the repository it came from has
several. **This repository has one queue and one loop.** Whatever track the
prompt names, the work is `docs/autonomy/QUEUE.md` — treat the word as noise
rather than as scope you have been asked to invent.

Halting on it once was right: an unknown track could have meant real work
somebody expected. It is written down now, so it is answered.

## What one iteration does

1. **Read `docs/autonomy/QUEUE.md`.** Take the first item that is not done and
   is not blocked. If every remaining item is blocked, write `LOOP COMPLETE`
   into the journal with the blocked list, and stop — a loop that keeps
   re-reading impossible work is a loop burning somebody's money.
2. **Read what the item names.** The ADR it implements, the contract it must
   satisfy, the section of `docs/features.md` that promised it. An item that
   cannot name those is not ready to build; mark it `needs design` and take the
   next one.
3. **Build it whole** — input, validation, policy, execution, record, error
   paths. Law 3: no stubs, no `todo!()`, no `unwrap()` outside tests. If it is
   turning out larger than one iteration, **cut its scope, never its depth**,
   and write the cut into the queue as a new item rather than leaving a
   half-built one.
4. **Pass the gate**, all of it. `cargo fmt`, `cargo clippy` with zero warnings
   *and* zero errors, tests including the refusal paths, documentation in the
   same change, a `CHANGELOG.md` line.
5. **Commit and push.** One item, one commit, a message that says what changed
   and why somebody would care.
6. **Update the queue, the roadmap and the journal.** Tick the queue item.
   Append to `docs/autonomy/STATE.md`: what was built, what the gate said, and
   anything the next iteration should know.

   **Then open `ROADMAP.md` and move the line this item served** — in the same
   commit. A part-done capability there carries two boxes of its own, described
   at the top of that file: **The code**, which this repository can finish, and
   **On the machine**, which it cannot. Write into the code half, naming the
   crate; add the line's two boxes if it has none yet.

   **You may tick the code half** when it is whole and gated — that is what it
   is for, and leaving it empty while the crate is finished is how this file
   came to report one done item out of eighty. You may never tick **On the
   machine**, and never the parent.

   If the item served no roadmap line, **say so in `STATE.md` and say why** —
   the way iteration 10 did when it found *test a provider* lived only inside
   "Settings, as one place". That is a real answer. Silence is not, and silence
   is what happened: eight consecutive iterations left `ROADMAP.md` untouched
   while eight crates landed, so the file reported that nothing had been built.

   **Never resolve this by ticking a capability or a machine half.** Both mean
   law 3 on real hardware, this loop has none, and a loop that learns to tick in
   order to discharge an obligation is worse than one that never updated the
   file. The code half is the honest place to record what was finished.
7. **Stop.** One item per iteration. Two is how a bad decision gets made twice
   before anybody reads the first one.

## What stops the loop

Write one of these as a line of its own in `STATE.md`:

- **`LOOP COMPLETE`** — every item is done or blocked, with the blocked ones
  listed and why.
- **`LOOP HALT`** — something is wrong that the loop must not work around: the
  gate fails for a reason the iteration did not cause, a decision is needed that
  is not ours, a test that used to pass has started failing, or the same item
  has failed twice.

**Halting is not failure.** An iteration that halts with a clear reason is worth
more than one that invents a way past a problem nobody has looked at.

## What the loop may never do

- **Never weaken the gate to pass it.** Not a lowered lint, not an ignored test,
  not a `#[expect]` added to silence something real. If the gate is wrong, halt
  and say so.
- **Never tick an item it did not finish.** `ROADMAP.md` says a tick means done,
  not written; the same rule applies here, and the loop is exactly where that
  rule would erode first.
- **Never add a verb that runs an arbitrary command** (law 2), or an escape
  hatch that amounts to one.
- **Never claim hardware verification it did not do.** Most of v0.01 ends on a
  certified machine that this loop does not have. Code that is built and unit
  tested is *built and unit tested*, and the item says so.
- **Never touch another repository.** Items that belong to `alo-workplace` are
  marked as such and are not this loop's to do.

## Linux is reachable, and some items need it

This loop runs on Windows, and a few v0.01 items cannot be built there — a Unix
socket's peer credentials being the one that stopped it. **That is no longer a
blocker: the machine has Ubuntu in WSL2, and it can build this very checkout.**

```
wsl -d Ubuntu -u root -- bash -c '
  export PATH="/root/.cargo/bin:/usr/lib/llvm-22/bin:$PATH"
  export LLVM_PREFIX=/usr/lib/llvm-22
  cd /mnt/c/dev/alo-os
  CARGO_TARGET_DIR=/root/alo-os-target cargo test -p <crate>
'
```

The two LLVM lines are only needed for `alo-bounding`, which builds a BPF
programme on the way; they are harmless everywhere else and are in the snippet
so nobody has to remember which crate is which. **Quote the `PATH` assignment** —
without the quotes the Windows `PATH` WSL passes through arrives with spaces in
it and `export` fails a dozen times before the command runs.

Three things about it, all measured rather than assumed:

- **It is the same working tree.** `/mnt/c/dev/alo-os` is this checkout, so an
  edit made on Windows is compiled by Linux with no copying and nothing to keep
  in step.
- **`CARGO_TARGET_DIR` is separate on purpose.** Linux and Windows artefacts in
  one `target/` invalidate each other constantly; two directories cost disk and
  save the whole build every time you switch.
- **It is fast enough** — a single crate checks in about seven seconds across
  the filesystem boundary.

**Use it for the Linux half and nothing else.** An item that builds on Windows
is built and gated on Windows; reaching for WSL by habit would mean the ordinary
path stops being tested. And an item that needs Linux says so in the queue, with
the reason, so nobody has to guess which is which.

**The BPF toolchain is installed, and getting there was not obvious.** Items 26
and 27 build a BPF programme with `aya`, which needs nightly with `rust-src` and
`bpf-linker`. Both are on the WSL box as of 2026-09-04:

```
nightly-2026-06-01, with rust-src, rustfmt and clippy   # pinned by the repository
bpf-linker 0.11.0, built against LLVM 22
export LLVM_PREFIX=/usr/lib/llvm-22
export PATH=/root/.cargo/bin:/usr/lib/llvm-22/bin:$PATH
```

Three things cost five attempts, and every one of them pointed at the wrong
cause:

- **The variable is `LLVM_PREFIX`, bpf-linker's own** — not `llvm-sys`'s
  `LLVM_SYS_<version>_PREFIX`, which is the one every search result names.
  Setting the `llvm-sys` variable correctly changes nothing.
- **The binary must be called exactly `llvm-config` and be on `PATH`.** Ubuntu
  installs `/usr/bin/llvm-config-22`, which does not match, and a plain
  `llvm-config` only under `/usr/lib/llvm-22/bin`, which is not on `PATH`. The
  error says *could not find llvm-config in … `PATH`*, so it reads as a missing
  package — and installing more packages never fixes it.
- **`cargo install bpf-linker` with default features wants an LLVM the
  distribution may not ship.** The feature has to be pinned to the one that is
  installed: `--no-default-features --features llvm-22`.

**And a fourth, which cost more than the other three together: the LLVM must
match the compiler.** `bpf-linker` reads the bitcode `rustc` emits and runs
LLVM's passes over it, so a `bpf-linker` built against an older LLVM than the
compiler's does not refuse the input — it segmentation-faults, in a message
naming LLVM's bug tracker and one of our own functions, on any programme with a
map in it. A programme without a map links perfectly, which is what sends you
looking at the code.

So **the nightly is pinned in the repository**, at
`crates/alo-bounding-kernel/rust-toolchain.toml`, and
`crates/alo-bounding/build.rs` starts the nested build inside that directory so
the file is what decides. Moving the pin means rebuilding `bpf-linker` against
the new compiler's LLVM: `rustc +<channel> -vV` says which one that is.

**`clang` and `bpftool` are still not installed and are still not needed.**

**A BPF filesystem has to be mounted, and on this box it is not by default.**
Since item 26e the boundary is *pinned* rather than held by whoever loaded it
(ADR 0018), and pinning needs `bpffs`. WSL's Ubuntu has `/sys/fs/bpf` as an
ordinary read-only sysfs directory, so every test that imposes a boundary fails
naming a directory. One line, and it does not survive a restart of the
distribution:

```
mount -t bpf bpffs /sys/fs/bpf
```

On a real machine `systemd` has already done it, which is why this is a fact
about the development box rather than a quirk of the product —
`docs/hardware.md` asks the question as the fifth of its kernel checks.

**Nothing is left pinned after a test run**, and that is deliberate rather than
lucky: a pinned programme outlives the process, so a fixture that leaked one
would leave a boundary attached to `file_open` on whoever ran the tests until
they rebooted. Every test that imposes one pins under a name of its own and takes
it away, and `ls /sys/fs/bpf` after a run is how you check that stayed true.

The two halves have their own gate, because neither is in the main workspace's
`--all-targets` sweep:

```
cd crates/alo-bounding-kernel
cargo fmt --all --check
cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings
```

`--all-targets` is deliberately absent from that clippy line: it forces a test
target for a crate that cannot link one, and the failure is
`can't find crate for test` rather than anything about the code.

**The BPF LSM attaches on this machine, and `cargo test --workspace` is the
whole gate again.** It was not always: on kernel `6.6.87.2` every attach hung
unkillably, because the kernel's RCU-tasks grace periods stalled a minute after
boot, and two iterations ran `--lib` for `alo-bounding` and excluded it from the
rest. `wsl --update` to `6.18.33.2` ended that. **Run the whole workspace**, and
if an attach ever hangs again the machine has failed one of the checks in
`docs/hardware.md` — ask it those four questions rather than narrowing what is
run. Do not mark the test `#[ignore]` to make the suite green; that is the gate
being weakened to pass it.

Do **not** install `clang` or `bpftool` to get around any of this. `aya` builds
the programme from Rust and neither is needed; reaching for them is how C would
enter the toolchain unnoticed. LLVM here is a build tool on the machine, in the
same category as the linker cargo already uses — it puts no C in the repository.

**`alo-agentd` and `alo-boundaryd` are never gated on Windows.** Every module in
them is
`#[cfg(target_os = "linux")]`, so on Windows the crate compiles to almost
nothing, runs **no tests, and exits 0** — which is the same exit code, and the
same green, as a full pass. This is not a slow path or a partial one; it is a
result that cannot be told apart from success while being worth nothing. It has
already produced one wrong entry in `STATE.md`, where *zero tests* was written
down as a fact about the code when it was a fact about the platform, and the
correction sits under that entry.

So for those crates the WSL command above is **the** gate, not a supplement to
one, and the number of tests it ran is part of what gets reported. A crate whose
test count silently drops to zero is the failure this rule exists to catch —
check the count, not just the colour. Any crate that acquires a
`[target.'cfg(target_os = "linux")']` block joins this rule the day it does.

**`cargo doc` is the same rule read backwards, and it fails the other way.**
The gate includes rustdoc on public items, and on Windows
`cargo doc --workspace --no-deps` emits *unresolved link* warnings in every one
of the Linux-only crates, because a crate header linking `[`starting`]` or
`[`Boundary`]` links to a module that is `cfg`'d out on this host. Every one of them is about code that is correct, and
on Linux the same command is silent. So where tests on Windows say *green* and
mean *nothing ran*, rustdoc on Windows says *warning* and means *not this
platform's file*, and both are the same fact wearing opposite colours.

`Cargo.toml` therefore denies `rustdoc::private_intra_doc_links` — the real one,
which fires identically on both hosts — and leaves `broken_intra_doc_links` at a
warning, because denying it would fail the gate on the host the loop runs on for
a reason that is not about the code. **Read the rustdoc gate on Linux**, and on
Windows check only that the count is the one the platform explains: one more
than that is somebody's new broken link hiding in the noise.

**The number moves for honest reasons, and it has moved five times.** It was
twenty-eight until item 26a gave `alo-bounding` four more Linux-only things for
its own crate header to link, thirty-two until item 26b gave it two more,
thirty-four until item 26d gave it four more again — the crate header now names
the doors the daemon reaches it by — forty-one once item 27 added the three
read-backs its test counts through, and **forty-nine** since item 26e split the
crate in two and added a third Linux-only crate. **Do not read the total, and do
not trust the last number written down either.** 27's own journal entry reports
thirty-eight, because it was read before the crate header was finished, and the
iteration that closed the queue found the real figure by measuring rather than
by carrying the sentence forward. `cargo doc --workspace --no-deps 2>&1 | grep
generated` names the crates and their counts one line each — today twenty-four
for `alo-agentd`, twenty-one for `alo-bounding` and four for `alo-boundaryd` —
and a count appearing against a crate that is **not** one of those three is the
thing to look at. A number in this paragraph is worth less than the command above
it.

## Where things are

| | |
|---|---|
| `docs/autonomy/QUEUE.md` | The work, in order, with what each is blocked on |
| `docs/autonomy/STATE.md` | The journal: one entry per iteration, newest last |
| `ROADMAP.md` | What a person outside the loop reads to know where the product is. Moved every iteration, per step 6 |
| `CLAUDE.md` | The four laws and the gate |
| `docs/decisions/` | Why things are the way they are. Read before proposing otherwise |

## Running it

The supervisor lives in `alo-workplace` and takes a repository path, so this
repository stays Rust and Containerfiles:

```
powershell -ExecutionPolicy Bypass -File C:\dev\Ficina-orders\scripts\run-loop.ps1 -RepoPath "C:\dev\alo-os"
```

Stop it any time. Every finished item was committed and pushed by the iteration
that built it, so nothing is lost by interrupting one.
