# v0.5 — the installer: download, click, reboot

**Workstream:** `ROADMAP.md`'s *Installer* line, brought forward to the
critical path by [ADR 0033](../decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md):
the certified laptop arrived on 2026-09-14 and is installed the way a customer
installs — a program downloaded from GitHub, run on the Windows the machine
came with, which stages a boot environment, reboots, and pulls alo OS from a
registry ([ADR 0023](../decisions/0023-installed-from-the-machine-it-replaces.md)).
**Why it exists:** every on-the-machine box in `ROADMAP.md` is empty and this
is the one road that fills them without a stick. It is the most dangerous code
the product will ship — it repartitions somebody's only computer from a
download — and it gets kernel-boundary paranoia throughout.

**Crates this plan owns:** a new `crates/alo-installer` (a Windows program;
compiles to its types on any other host, the way `alo-agentd` does on
Windows), `image/`, `docs/booting.md`, `.github/workflows/`, and
`crates/alo-image` where the recipe and the disk facts are held. Nothing in
`crates/alo-shell`, nothing in `alo-nearby`, `alo-asking`, `alo-record`,
`alo-capability`, `alo-turn`, `alo-egress` (lane A's), nothing in the model
crates (the Mac's), nothing in `alo-finding`, `alo-measuring`,
`alo-portals`, `alo-granted`, `alo-applications`, `alo-secrets` (lane B's
other plan, which yields to this one).

**What this plan may not do:** tick anything *on the machine* — that is done
by a person at the laptop, into `docs/autonomy/v0-01-evidence.md`, with what
they saw; move any v0.01 wording; ask a person to disable Secure Boot (ADR
0033 §4); or take a destructive step in any test on real hardware. Every test
that repartitions runs against a virtual machine. Before writing the next
task, `git pull` and read the plan as published.

**The gates cannot run this crate's Windows tests.** They run in Linux, where
`alo-installer` compiles to its types. A worker on this Windows PC runs the
crate's own tests by hand (`cargo test -p alo-installer`) and pastes the
result into the report; a handoff whose report has no such paste is not
done.

## Tasks

### 1. The image is published from GitHub, signed, and pinned

**Status:** ready. **Depends on:** nothing.

ADR 0023: *the installer's reboot environment pulls the same signed, versioned
image CI built, from the same registry updates come from.* Today the image is
built by hand under WSL and lives nowhere a laptop can pull it from.

- **Acceptance:** `image/Containerfile` is built and pushed to
  `ghcr.io/aloworld-org/alo-os` with a version tag and a digest, by a
  workflow in `.github/workflows/` — or, until GitHub's runners can hold the
  image (it carries 4.5 GB of weights), by `podman push` from the machine
  that built it, with the workflow file written and the reason it is not yet
  the road recorded in it; the image is signed with `cosign` and the public
  key is in the repository, so a puller can verify before writing (ADR 0023
  §3); the digest the installer pulls is pinned in one file `alo-image`
  reads, and a test fails if the recipe's version and the pinned digest
  disagree; and `docs/booting.md` gains the registry, the tag and the one
  `bootc install` invocation that pulls from it, held to the code the same
  way its four disk facts already are.
- **Constraint:** ADR 0011 throughout — the base is rented and unmodified;
  what is pushed is exactly what the recipe builds. Nothing here changes what
  the image contains. A push that cannot be verified is not a publish.

### 2. The boot environment that installs, tested in a virtual machine

**Status:** ready. **Depends on:** 1.

ADR 0023 §2–3: *a minimal boot environment and a UEFI boot entry*, which runs
`bootc install`, *pulling alo OS from the registry over HTTPS, verifying
signatures before writing.* This is that environment — the part that runs
after the reboot and before alo OS exists on the disk.

- **Acceptance:** a small UEFI-bootable environment (a kernel, an initramfs
  holding `bootc` and its container tooling, and nothing else) is built from a
  recipe in `image/` beside the OS's own; booted in a Hyper-V generation-2
  VM with a **second** virtual disk attached, it pulls the image from the
  registry of task 1, verifies the signature, runs `bootc install to-disk`
  onto that second disk with Windows-style partitions on the first left
  untouched (a test hashes the first disk before and after), and the second
  disk then boots to `alo-agentd` running; a pull whose signature does not
  verify writes nothing and says so on the console in words `alo-saying`
  collects; and the environment reports what it is doing to the console at
  every step, because the person watching has no other window.
- **Constraint:** the environment is not a second operating system; it is
  the installer's reboot half and ships only what `bootc install` needs. It
  never touches a disk it was not told to. Measured in a VM; nothing here
  runs on the laptop.

### 3. The installer program: check, say, consent, stage, reboot

**Status:** ready. **Depends on:** 2.

ADR 0023 §1–2, and ADR 0033 §4–5. A Windows program in Rust —
`crates/alo-installer` — that a person downloads and runs.

- **Acceptance:** it checks the machine and says what it found — UEFI or
  not, Secure Boot state, TPM, BitLocker state of the Windows volume, free
  space, memory — each check read from Windows' own tools rather than
  guessed, and each with a sentence in the vocabulary; **with Secure Boot on
  it refuses to proceed and says why**, never suggesting the setting be
  changed (ADR 0033 §4); it says exactly what will happen — shrink the Windows
  volume by *N* GB, create a partition, add a boot entry named alo OS, reboot
  — and takes a **typed** consent naming the disk, not a checkbox; it shrinks
  the Windows volume through Windows' own tooling, stages the boot
  environment of task 2 on the new partition, adds the UEFI entry, and
  reboots; and every step before the reboot is reversible and tested so: a
  test in a Hyper-V VM with a real Windows in it walks the installer to each
  step, kills it there, and shows Windows still boots.
- **Constraint:** `unsafe_code = "forbid"` holds. Disk and boot operations go
  through Windows' own programs and documented APIs behind safe wrappers; a
  raw IOCTL this crate would have to write itself is a finding, not a line of
  `unsafe`. Nothing here runs on the laptop; the machine every test uses is
  a virtual one with a Windows the test installed.

### 4. Alongside Windows, and back again

**Status:** ready. **Depends on:** 3.

ADR 0023 §4 and ADR 0033 §2: *Windows is retained alongside* — the default,
and on the certified laptop the only mode. And ADR 0023's constraint: *a
tested bail-out that leaves Windows bootable at every step until the final,
named point of no return* — which, alongside Windows, is nothing destroyed.

- **Acceptance:** after task 2's environment has installed alo OS beside a
  Windows in a VM, the firmware's boot menu offers both, Windows boots
  exactly as before (the same hash test), alo OS boots, and which is the
  default is a choice the installer made explicit and recorded; *remove alo
  OS* exists as a documented, tested road back — the partition freed, the
  boot entry gone, Windows as it was — because a person who can install from
  a download must be able to uninstall from one; and the whole journey is
  recorded in `docs/booting.md` as the steps a person takes, in order, with
  what they see at each.
- **Constraint:** replacing Windows is not built here. It is an explicit,
  twice-confirmed mode ADR 0023 allows, and it waits for a task of its own
  after the alongside road has been certified.

### 5. Released from GitHub, and the page a person downloads from

**Status:** ready. **Depends on:** 3.

*Install from GitHub or the website.* The installer is a GitHub Release of
this repository, and the download page says what a person needs to know
before running it.

- **Acceptance:** a workflow builds `alo-installer` for Windows on a tag,
  signs the executable, and publishes it as a Release asset with a checksum
  beside it; the Release notes say which image digest it installs and which
  Secure Boot state it accepts; the README gains a *Try it* section — the
  requirements from `docs/hardware.md`, the fact that Windows stays, and the
  one sentence about Secure Boot — and nothing else, because the README's
  rule since 2026-09-13 is no noise; and a test in the repository refuses a
  Release whose notes name a digest the pinned file does not.
- **Constraint:** no telemetry, no update check in the installer beyond the
  one pull it exists to make, and nothing leaves the person's machine that the
  screen did not show (law 1). The website's copy is the website repository's;
  this task ends at the Release and the README.

### 6. The certified laptop, firmware to the daemon

**Status:** blocked — on tasks 1–5, and on the owner at the laptop; nothing in
this repository can tick it. **Depends on:** 4, 5.

ADR 0033 §1: hardware acceptance goes through the installer. This task is the
document the owner follows at the laptop and the ledger entries their
observations fill.

- **Acceptance, when unblocked:** `docs/booting.md` (or a page beside it)
  holds the steps: download from the Release, run, read what the installer
  found, type the consent, reboot, watch the boot environment pull and
  install, choose alo OS in the boot menu, and reach `alo-agentd` on the
  console; for each of *the GPU works on first boot*, *the boundary attaches
  on this kernel*, *the pinned model answers on this CPU* and *boots on one
  certified machine, firmware to the daemon*, the exact thing to look for and
  the command to type; the owner's observations are written into
  `docs/autonomy/v0-01-evidence.md` by hand with the date, the machine and
  the Secure Boot state, and the roadmap's on-the-machine boxes move only
  for what was seen; and *firmware to sign-in* stays open with one sentence:
  the sign-in screen is the desktop lane's and is not in the image yet.
- **Constraint:** nothing is ticked from this repository. A loop that
  reaches this task launches nothing and says so.
