# The laptop as a development machine

The certified laptop arrived on 2026-09-14 with 32 GB — the most memory this
project has. Until the installer is ready to put alo OS on it
(`v0-5-the-installer-plan.md`), it is the strongest builder we own and it is
idle. This turns it into a fourth lane.

**It is still the certification machine.** When the installer's tasks 1–5 land,
the lanes here stop and it becomes the machine alo OS is installed on
([ADR 0033](../decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)).
Nothing this document sets up is installed outside Windows, and nothing it
does touches the disk layout.

## What to install

1. **WSL2 with Ubuntu 24.04** — `wsl --install -d Ubuntu-24.04` in an
   administrator PowerShell, then set a user when it first starts.
2. **Inside Ubuntu**, the toolchain `docs/autonomy/LOOP.md` and
   `docs/autonomy/GRAPHICS.md` list, which the Mac lane's report of 2026-09-13
   found the complete version of:
   - stable Rust (at least the version `Cargo.toml`'s `rust-version` names),
     the pinned nightly from `crates/alo-bounding-kernel/rust-toolchain.toml`
     with `rust-src`, `rustfmt` and `clippy`;
   - LLVM 22 and `bpf-linker 0.11.0` built `--no-default-features
     --features llvm-22`;
   - `build-essential pkg-config libwayland-dev libegl1-mesa-dev
     libgles2-mesa-dev libxkbcommon-dev libudev-dev libinput-dev libgbm-dev
     libseat-dev`, or clippy stops at `libudev-sys`;
   - `gnome-keyring` and `dbus`, with Ubuntu's `org.freedesktop.secrets`
     activation file set aside as `docs/quirks.md` describes;
   - `bpffs` mounted at `/sys/fs/bpf`, in `/etc/fstab` so a restart keeps it.
   - Run the gates **as root**, as this PC does.
3. **On Windows**: Git with credentials that can push to the repository, the
   `claude` CLI, and Rust for building the supervisor.
4. **`.wslconfig`** in your Windows home, with more than this PC can give:
   `memory=20GB`, `processors=` however many cores it has minus two,
   `swap=8GB`, `autoMemoryReclaim=gradual` under `[experimental]`, and the
   `kernelCommandLine = lsm=capability,landlock,yama,safesetid,selinux,bpf`
   line this PC's copy carries (ADR 0015). Then `wsl --shutdown` once.

## Running the lane

```powershell
git clone https://github.com/aloworld-org/alo-os C:\dev\alo-os
cd C:\dev\alo-os\tools\kernel-loop
cargo build --release
cd C:\dev\alo-os
$env:ALO_LOOP_PLAN = "docs/autonomy/v0-5-applications-and-what-they-expect-plan.md"
$env:ALO_KERNEL_LOOP_WORKER = "$env:APPDATA\npm\claude.cmd --dangerously-skip-permissions -p"
.\tools\kernel-loop\target\release\alo-kernel-loop.exe run
```

`status` says whether a loop is running; `stop` asks it to finish its task and
exit. A second lane on the same machine is a second clone in its own
directory, with its own plan — the supervisor keeps their build directories
apart by the checkout's path.

## The plan it runs, and the ones it must not

**Its plan is `docs/autonomy/v0-5-documents-and-paper-plan.md`** — five tasks:
what this machine can do with a file, `.docx`/`.xlsx`/`.pptx` opened and what
the conversion cost, a printer found and saying what is wrong with it,
*"I can't open this file"* said properly, and the walk through every sentence.
It owns `crates/alo-printing` and `crates/alo-opening`, both new, and **no
other lane is in those**.

Two spare PCs joined on the same day and took the applications and
machine-keeps-itself plans; the table in
`docs/autonomy/a-new-machine-becomes-a-lane.md` is the one place that says who
has what. **If this plan empties, do not pick another** — say so and stop, so
that a person who can see every lane assigns the next one. Two lanes in one
crate corrupted a task on 2026-09-11.

## The rules, which are the same for every lane

One branch, `main`. Pull before starting and again before pushing; a refused
push means `main` moved — rebase, gate the combined tree, push again, and
never resolve it by discarding somebody's work. Push every finished task.
Commit as the configured git user with no `Co-Authored-By` trailer. Before
writing a new ADR or task number, `git pull` and look — numbers are shared and
have collided three times. `CLAUDE.md` is absolute, and its laws hold here
exactly as on the other machines.

**This machine's own rule:** nothing measured on it is a certification.
A grade, a timing or a boot in a virtual machine is a fact about the thing
measured, never about alo OS on certified hardware — that is what the
installer and `docs/autonomy/v0-01-evidence.md` are for, and only a person at
the machine writes into that ledger.
