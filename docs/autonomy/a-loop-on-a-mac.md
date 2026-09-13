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
   export ALO_KERNEL_LOOP_LINUX='limactl shell alo'   # or: orb -m alo
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
