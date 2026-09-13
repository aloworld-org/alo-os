# A loop on a Mac

How to run a third `tools/kernel-loop` lane on a Mac, what it can take up, and
the one thing it must never do.

## What a Mac is for

**A machine that can hold a 7B model.** The development PC has 15.5 GB and a
6 GB WSL cap; `mistral-7b-instruct` ran there against swap at a quarter of a
token a second (`dev-machine-limits`, 2026-09-11). An Apple Silicon Mac with
16–32 GB of unified memory runs the same weights at conversational speed with
Ollama installed natively. That is the whole reason to have one in the loop: it
can answer the questions every report so far has ended with — *is this model
`not-measured` because nobody could load it, or because it cannot drive the
verbs?* — and it can do so on lane B's old crates, which are pure Rust and
currently idle.

The plan a Mac lane runs is `docs/autonomy/v0-5-the-models-measured-plan.md`.

## The rule it is held to

**The gates run in Linux, or they do not run.** On Windows the loop hands every
gate to `wsl`. On a Mac it hands them to a Linux virtual machine you name, and
**refuses to gate natively** if you do not: a macOS-green suite hides Linux-red
exactly the way a Windows-green one does (`alo-recounting` fails on Windows and
passes on Linux, every day), and the BPF target has nothing to build against on
macOS at all. There is no flag that runs the gates on the Mac itself, on
purpose.

## Setting it up

1. **A Linux VM.** [OrbStack](https://orbstack.dev) or [Lima](https://lima-vm.io),
   with an Ubuntu 24.04 machine called `alo`. Either mounts your home directory
   inside Linux **at the same path**, which is what lets the loop hand a gate a
   path without translating it — so the checkout must live under `~`. With Lima,
   make the mount writable (`writable: true` for `~` in the instance's YAML);
   OrbStack's is writable already.

2. **Inside the VM**, what WSL has on the PC: `rustup` with the stable
   toolchain, the pinned nightly named by `crates/alo-bounding-kernel/rust-toolchain.toml`,
   `bpf-linker`, `clang`, `llvm`, `pkg-config`, `libssl-dev`, and
   `/sys/fs/bpf` mounted (`bpffs` in `/etc/fstab`, as the PC's WSL has). The
   gates build in `$HOME/alo-builds/<checkout>-<hash>` **inside the VM's own
   home**, never on the shared mount — the loop chooses that directory itself.

   That list was not enough, and the first run found what else, one refusal at
   a time (`docs/autonomy/updates/one-catalogue-entry-graded-on-a-machine-that-can-hold-it.md`
   has each refusal's words):

   - **The desktop's libraries**, the list `docs/autonomy/GRAPHICS.md` gives —
     `libwayland-dev libegl1-mesa-dev libgles2-mesa-dev libxkbcommon-dev
     libudev-dev libinput-dev libgbm-dev libseat-dev` — or clippy stops at
     `libudev-sys`.
   - **`gnome-keyring` and `dbus`**, which `alo-keyring-fixture` starts.
   - **Ubuntu's `org.freedesktop.secrets` activation file set aside**, with
     `dpkg-divert --local --rename --add
     /usr/share/dbus-1/services/org.freedesktop.secrets.service`, or the keyring
     fixture's bus starts the machine's keyring instead of its own
     (`docs/quirks.md`).
   - **The HWE kernel**, `linux-generic-hwe-24.04` (7.0 on 2026-09-13): on the
     release kernel, 6.8, the verifier refuses the boundary for its stack.
   - **`lsm=…,bpf`** on the kernel command line, in `/etc/default/grub.d/`, which
     Ubuntu's cloud image does not start.
   - **Root.** The PC runs every gate as root (`wsl -u root`), and the tests that
     impose a boundary need it. Point root's login shell at the same toolchain
     (`RUSTUP_HOME`, `CARGO_HOME` and `PATH` in `/root/.profile`) and name the VM
     as `limactl shell alo sudo`, so the loop's `bash -lc` runs as root.
   - **Swap.** Four gigabytes of VM links the workspace's tests against swap;
     an 8 GB swap file keeps the linker from being killed.

3. **On the Mac itself**: `git` with credentials that can push (the loop runs
   `git` where the credentials are, which is why it is a host program and not a
   VM one), the `claude` CLI, `cargo` for building the supervisor, and
   [Ollama](https://ollama.com) with the catalogue's models pulled. That pull
   is the first real egress the measurement work has ever needed; it is
   `alo-egress`'s *fetching a model* errand and shows on the indicator on an
   alo machine, and on the Mac it is a person typing `ollama pull`.

4. **The supervisor**: `cd tools/kernel-loop && cargo build --release`.

5. **Run it**, from the checkout:

   ```sh
   export ALO_LOOP_PLAN=docs/autonomy/v0-5-the-models-measured-plan.md
   export ALO_KERNEL_LOOP_LINUX='limactl shell alo sudo'   # or: orb -m alo -u root
   export ALO_KERNEL_LOOP_WORKER="$(command -v claude)"
   ./tools/kernel-loop/target/release/alo-kernel-loop run
   ```

   `tools/kernel-loop/on-a-mac.sh` is those five lines with a `git pull` in
   front, idempotent under the loop's lock, for the same use as
   `resume-lane-a.cmd` on the PC.

`ALO_KERNEL_LOOP_LINUX` is **a command that runs a shell inside the VM**; the
loop appends `bash -lc "<the gate's script>"` to it. Anything that takes that
shape works — a different Lima instance name, `ssh` to a Linux box on the
desk — and the loop never guesses one.

## What the first run owes

Nobody has run the gates on aarch64 Linux. The workspace's tests are pure Rust
and should not care; **the BPF target's gates might**, because
`crates/alo-bounding`'s `build.rs` builds against the running kernel's BTF and
the hook's shape in ADR 0030 was read from an x86_64 kernel. If those two gates
refuse a tree whose other seven pass, that is a finding about the Mac and not
about the tree: record it in `docs/quirks.md` with the exact refusal, and the
Mac lane's plan already keeps it off the kernel crates. Do not weaken a gate to
get past it.

**What it found, on 2026-09-13**, on an Apple M3 with 8 GB running Ubuntu
24.04 aarch64 under Lima: **both BPF gates pass**, and so do six of the other
seven. The workspace's tests pass except for one on the untouched tree, in a crate
this lane does not own — `alo-bounding`'s file-flag test, refused as unsupported
on aarch64 kernels 6.17 and 7.0 — and one that fails or passes by the order a
filesystem lists a directory in, `alo-citing`'s. Both are in
`docs/quirks.md` with their words, for their owners.

## A Mac with 8 GB

This document assumed 16–32 GB. The first Mac in the loop has 8, and it holds a
7B model at four bits anyway — **with the VM stopped**. The VM is given 4 GB and
the weights want 4.7 GB of the same unified memory, so a measurement is run with
`limactl stop alo` first and the gates are run after it. Measured that way, the
fixed ten against `qwen2.5:7b-instruct-q4_K_M` took thirty-three seconds.

## What it must not take up

Anything Linux-only or hardware-bound: `alo-bounding`, `alo-boundaryd`,
`alo-agentd`, the BPF target, `image/`, `alo-sessiond`, `alo-secrets`,
portals, printers, hardware acceptance, and everything in `crates/alo-shell`
(the desktop lane's). The image is x86_64; an Apple Silicon Mac cannot boot it
and this document does not pretend otherwise. A Mac lane's plan names its
crates, and they are the model crates: `alo-models`, `alo-driving`,
`alo-choosing`, `alo-answering`, `alo-telling`, `alo-asking`'s hosted and
served doors.

## The one thing it cannot do for the product

A measurement on a Mac is a measurement of the **model**, and it says so:
whether these weights emit a valid verb call nine times in ten is a fact about
the weights that holds wherever they run. It is not a measurement of alo OS on
certified hardware, moves no *On the machine* box, and every report names the
machine it ran on.
