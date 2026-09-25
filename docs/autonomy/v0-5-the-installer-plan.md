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

**A test that makes disks cleans them up, and a build that makes an image names
or removes it.** On 2026-09-15 this plan's virtual-machine work filled this PC's
C: drive from 33 GB free to nothing in about an hour: every rebuild of the boot
environment left an unnamed image of 1.3–1.7 GB in the container store, and the
tests left multi-gigabyte copies of Windows-shaped disks behind. The distribution
crashed with the drive full, and a lane's gates failed with an I/O error. So,
from then on: every virtual-machine test writes its disks under Cargo's
`CARGO_TARGET_TMPDIR` and **removes them when it finishes, pass or fail**, keeping
only the logs it names in its report; a worker that builds an image **tags it or
removes it** before it hands over, and runs `podman image prune -f` when it is
done; nothing is left in `/root` or `/tmp` between runs; and a worker checks that
at least 15 GB is free on the drive the distribution's disk lives on before it
starts a virtual machine, and says so and stops if not.

**The weights on the image, released to the v0.01 lane-B plan, owner-authorized
2026-09-22.** The owner instructed the third PC to build that plan's task 10, *a
model on the disk, sized for the machine it lands on*, and **that task's own
acceptance names this plan's crate**: the weights are put on the image and
`crates/alo-image` holds the recipe to them, digest-checked the way the runtime
already is. There is nowhere else they could go — the recipe is what the image
is made from — so the task cannot be built at all without these files.

This plan keeps `image/` and `crates/alo-image`, and nothing else moves: the
files below are the weights' own additions, and the checks that hold the recipe
to them. A change to the image for any other reason is still this plan's alone.

```owner-release
plan = docs/autonomy/v0-01-lane-b-plan.md
task = 10
files =
  image/Containerfile
  crates/alo-image/src/arrives_with.rs
  crates/alo-image/src/checking.rs
  crates/alo-image/src/lib.rs
  crates/alo-image/src/weights.rs
  crates/alo-image/src/wrong.rs
```

## Tasks

### 1. The image is published from GitHub, signed, and pinned

**Status:** **Done, 2026-09-15** (`updates/the-published-image-pinned-by-digest.md`).
The signed digest below is pinned in `image/pinned.toml`, which `crates/alo-image`
reads and holds to the recipe's release, the committed key and `docs/booting.md`
(its registry, tag, digest, and the verify-then-install invocation);
`.github/workflows/image.yml` pushes a candidate on request, never signs, and
records why it is not yet the road. What stays the owner's: making the `ghcr.io`
package public, so an installer pulls without an account.

The owner has published five times, beginning 2026-09-15 under
[ADR 0036](../decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md).
The pinned one, signed 2026-09-21, is **the first release that is complete for
media**: it opens a document, plays a video and makes one. 0.0.4 was the first
that could open a document at all — each before it shipped a converter that
died at launch for want of twelve shared libraries — and every release up to
and including it could decode the sound of a file and nothing else, carrying no
video decoder, no encoder and nothing to reach the media server with:

| | |
|---|---|
| Release | `0.0.5` (`org.opencontainers.image.version`) |
| Built from | `97c970c96f2c13f2341ed94fb264247fb6837442` (`org.opencontainers.image.revision`) |
| Pushed to | `ghcr.io/aloworld-org/alo-os:0.0.5` |
| Digest | `sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1` |
| Signed | by the owner, with the private half, by digest, no transparency log (`cosign sign --use-signing-config=false --tlog-upload=false`, cosign 3.1.3) |
| Verified | `cosign verify --key image/signing/alo-os.pub --insecure-ignore-tlog=true` passes; a different key is refused (*Found: 0, Expected 1*) |

**Both the digest and the revision above were read back from the registry**, not
from the machine that built the image — the digest from `docker-content-digest`,
the revision from the image's own label in its config blob. A local copy's
digest is not the one a registry stores, and signing the local one produces a
signature that verifies on the builder's machine and nowhere else.

The package on
`ghcr.io` is still private until the owner makes it public, so a pull today
needs a login; that is named in the report, not worked around.
**Depends on:** nothing.

**What the first worker found, 2026-09-14** (`updates/who-signs-the-image.md`):
the machine that builds the image has no `cosign`, no login to `ghcr.io` and
no GitHub command-line tool, and nothing in this repository says who holds the
key every installed machine will trust. ADR 0036 puts that to the owner and
recommends a key the owner generates and holds, with signing by digest done
by a person and never by an agent or the loop. What landed with it is the one
piece a push needs first: `image/Containerfile` names its release in
`org.opencontainers.image.version`, and `crates/alo-image` refuses a recipe
that names none, two, or a word that moves.

**When it unblocks**, the owner does the five steps in ADR 0036 — build at a
published commit, push, sign the digest, verify with the committed public half,
hand over the version and digest — and this task is then the repository's
half, which is the acceptance below with a real digest to pin. The key is
settled (accepted 2026-09-15); a worker that reaches this task with no digest
handed over launches nothing and says so.

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

### 2. The boot environment that installs: written, and its refusals held

**Status:** **Done, 2026-09-15**, for the part of the original acceptance that is
proven, and **split** for the rest. Three workers in a row reached the 90-minute
deadline on this task, and the supervisor's rule is that such a task is a phase,
not a task (`tools/kernel-loop/src/worker.rs`). What is done: the environment's
recipe in `image/installing/`, the program inside it (`crates/alo-installing`)
with every refusal decided in `sequence.rs` and tested against a scripted machine
(no choice, two choices, a path, a disk that never appears, the installer's own
disk, a disk holding Windows, a disk in use or read-only, no network, not genuine,
a damaged environment — each saying *so nothing was changed* in words
`alo-saying` collects), `crates/alo-image` holding that recipe to the image's base,
pin and key, and `docs/booting.md` saying honestly that neither virtual-machine
test passes yet. **What is not done moved to two tasks of its own:** task 8 (the
refusal road must write nothing the installer does not own) and task 9 (with
Secure Boot on, the staged loader must start). The two virtual-machine tests stay
in `crates/alo-installing/tests/installed_in_a_virtual_machine.rs`, ignored in the
suite, and are those tasks' acceptance. Report:
`updates/the-boot-environment-that-installs.md`. **Depends on:** 1.

**What the second worker found, 2026-09-15**
(`updates/the-boot-environment-that-installs.md`): the environment, its recipe,
the program inside it (`crates/alo-installing`) and its refusal tests are
written and gate; the two virtual-machine tests that are this task's acceptance
**do not pass**. Under Secure Boot (OVMF with Microsoft's certificates) the
firmware page-faults starting the staged shim/loader, before Linux; and the
not-genuine refusal says the right words and writes nothing to the second disk
but changes the first disk — located to three mebibytes inside the staged
`ALO-INSTALL` FAT itself, Windows' partitions untouched, cause not yet known
(the report lists what was ruled out and the next two experiments). The next
worker starts from those two, not
from the code. This machine's account cannot manage Hyper-V, so the VM is QEMU.

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

**Status:** **Done, 2026-09-15**, for the program and every decision in it, and
**split** for the one part of the acceptance no machine this repository has can
run: the walk in a Hyper-V VM with a real Windows in it, killed at each step, is
task 10. What is done (`updates/the-windows-installer-program.md`):
`crates/alo-installer`, a Windows program that asks for an administrator's
rights; holds the environment beside it to the list its release was built with;
reads UEFI, Secure Boot, TPM, BitLocker, free space, memory and every disk from
Windows' own tools and says each, *could not be found out* never read as *off*;
**refuses Secure Boot on (or not known) with the reason and no suggestion**; says
exactly what will happen; takes the name of the disk as a typed consent; shrinks
Windows by exactly the 1 GB area through `Resize-Partition`, makes and formats
the area, writes the environment and `chosen.cfg` and reads them back, adds an
entry named alo OS with `bcdedit`, makes it the next start once, and restarts —
putting back every change, newest first, when any step fails, and saying exactly
what remains when putting back fails too. Every refusal and every step's failure
is tested against a scripted Windows, and the checks were run, reads only, against
the development machine's own Windows 11, which found two things the design had
wrong (`docs/quirks.md`). **What it does not do, and says so:** install onto the
disk Windows is on — the environment of task 2 replaces one whole empty disk, so
a one-disk laptop is refused with *no empty disk … beside the one Windows is on*
until task 4 puts alo OS beside Windows on the same disk. **Depends on:** 2.

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
  a virtual one with a Windows the test installed. **A person never sees a
  key** (ADR 0036, as the owner accepted it): the public half ships inside the
  installer, the signature is checked without being shown or offered as a
  choice, and no screen, prompt or sentence names a key, a password or a
  signature — a person downloads, clicks and restarts. A failed check is said
  as *this download is not a genuine alo OS, so nothing was changed*, and there
  is no way past it.

### 4. Alongside Windows, switching between them easily, and back again

**Status:** in progress — **and no longer scheduled on hardware.** It installs
beside a real Windows in a virtual machine and walks the switching both ways,
which is the largest disk of the three. **Depends on:** 3, 8, 9, 10.

**Landed so far, 2026-09-22 to 2026-09-25**, three pieces of the Windows side,
each walked in the guest except where it says otherwise:

- **The Fast Startup question** (ADR 0064 term 9). The installer reads Windows'
  own values, says what it found like every other check, and — only when Fast
  Startup is on — asks the owner's words after the consent and before anything
  is changed. *Turn off* sets `HiberbootEnabled` to `0`, is journalled, and is
  put back if a later step fails. No program of the installer may name
  `powercfg`, and a test holds that. **Walked on a real Windows** on a second
  base with hibernation on: the question asked, *turn off* typed, and the value
  read back `1 → 0` (`docs/quirks.md` carries what that guest will and will not
  keep).
- **The way back in**, from inside Windows. Staging leaves a copy of the
  installer under Windows' own place for programs with shortcuts in the Start
  menu; started with the switch's word it sets the firmware's **next start**
  only and restarts. **Walked**: the firmware started alo OS, and the start
  after that was Windows, so the default was never touched.
- **Which system starts by default, from Windows** (ADR 0066 term 3). It calls
  `alo-starting` rather than copying the format: the environment block, the
  path on the EFI system partition, the name the choice is kept under and the
  two systems all come from the crate that owns them. Six tests, including that
  what Windows writes is what alo OS's own reader reads back, and that the file
  keeps the length it was read at. **Against the scripted Windows**; the guest
  walk of a person changing it is owed.

- **What the install leaves behind, put right** (ADR 0062 term 1). The
  environment has a sixth step, after alo OS is on the disk and before the
  restart: the entry named *alo OS* rather than the base's own name, Windows
  Boot Manager directly behind it, the entry the installer staged taken away
  with its area, and the area itself removed from the Windows disk. alo OS is
  installed before any of it runs and stays installed if all of it fails.
  **Walked**, 2026-09-25: the installed machine read its own firmware and its
  own disks back —

  ```
  BootOrder: 000C,0004,0003,0000,0001,0002,0005,0006,0007,0008,0009
  Boot0004* Windows Boot Manager  HD(1,GPT,…)/\EFI\Microsoft\Boot\bootmgfw.efi
  Boot000C* alo OS               HD(2,GPT,…)/\EFI\fedora\shimx64.efi
  sda   ├─sda1 SYSTEM  ├─sda2  ├─sda3 Windows  └─sda5      (no ALO-INSTALL)
  ```

**Still owed here, and this task is not done until they are:**
1. **The fall-through test** ADR 0062 term 1 asks for: alo OS's loader made
   unstartable, the computer restarted, and Windows coming up **with no
   keypress**, read from the machine's own console rather than counted.
2. The rest of the acceptance below that neither piece covers: *remove alo OS*,
   and the walk of the default being changed from either side.

> **One of this task's two hardware conditions was cleared on 2026-09-20, on the
> development PC** (Intel Core Ultra 7 155U). *Hardware virtualisation, which the
> third PC does not have*: `/dev/kvm` exists here and accelerates — task 10's
> note carries the measurements, and a Windows 11 evaluation ISO booted and ran
> its unattended answer file in a KVM guest there, with a TPM 2.0 and a second
> empty disk.
>
> **The disk condition stands.** *A machine with 50 GB free* is still not met:
> `df` inside WSL reports 805 GB, but that is the vhdx's virtual size — the
> host's C: is 474 GB with about **13 GB** genuinely free, and the run above
> took it to zero. This task is *the largest disk of the three*, so it is the
> one least able to ignore that. Blocked on **9 and 10, and on a disk**.

> **Which program shows the start-up menu is decided, 2026-09-21:**
> [ADR 0062](../decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md),
> alo OS's own loader, with Windows directly behind it in the firmware's
> order and a test that proves the fall-through. Neither ADR 0023 nor ADR
> 0033 answered it, and this task could not be built without the answer.
> Its three terms are part of this task's acceptance. **Fast Startup is
> decided, 2026-09-22:**
> [ADR 0064](../decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
> term 9 — **the installer asks**. The words are in the acceptance below. And
> the test that alo OS never mounts the Windows partition read-write holds
> whichever the person answers.

ADR 0023 §4 and ADR 0033 §2: *Windows is retained alongside* — the default,
and on the certified laptop the only mode. **The owner's words on 2026-09-14:
*by the time we test, run the two operating systems and switch from one to
the other easily.*** Two systems on one disk that a person has to reach a
firmware key to choose between are not that; switching is a thing each
system offers from inside itself. And ADR 0023's constraint: *a tested
bail-out that leaves Windows bootable at every step until the final, named
point of no return* — which, alongside Windows, is nothing destroyed.

- **Acceptance:** after task 2's environment has installed alo OS beside a
  Windows in a VM, the firmware's boot menu offers both **on every start,
  with a short countdown and the last-chosen system preselected**, so a
  person who touches nothing gets what they had and a person who wants the
  other has one keypress; **from inside Windows** the installer, left in
  place as a small program after installing, offers *Restart into alo OS* —
  one click, a confirmation, and the next boot is alo OS, done through the
  firmware's next-boot entry rather than by changing the default, so the
  choice is for one restart and the default is untouched; **from inside alo
  OS** *Restart into Windows* is a setting and a verb an agent may ask under
  a grant, done the same way, with the person's confirmation as any verb has;
  which system is the **default** is the person's choice, made once in the
  installer and changeable from either side, and both sides show the same
  answer; Windows boots exactly as before (the same hash test), alo OS boots,
  and a test in the VM walks Windows → alo OS → Windows → alo OS through the
  in-system switches alone, never the firmware menu; *remove alo OS* exists
  as a documented, tested road back — the partition freed, the boot entry
  gone, Windows as it was — because a person who can install from a download
  must be able to uninstall from one; and the whole journey is recorded in
  `docs/booting.md` as the steps a person takes, in order, with what they
  see at each.
- **Fast Startup, and what the installer asks** (ADR 0064 term 9, the owner's
  decision of 2026-09-22). When Windows' Fast Startup is on, the installer
  **asks**, in these words, externalised like every other sentence:

  > Windows' Fast Startup is on. It can make Windows and alo OS disagree about
  > the disk. Turn it off? (Recommended when sharing a disk.)

  with **[Turn off]** and **[Leave on]**. *Turn off* sets `HiberbootEnabled` to
  `0` and **never** runs `powercfg /h off`, which removes hibernation
  altogether and is a different act from the one the person agreed to. Both
  answers are safe, because alo OS never mounts the Windows partition
  read-write (`crates/alo-starting/tests/windows_is_never_mounted.rs`, task
  16). The question is only asked when the value is on, the answer is the
  person's, and neither answer stops the install.
- **What the install leaves behind, tidied.** Measured on 2026-09-22
  (`docs/quirks.md`): after the install the firmware's first entry is bootupd's,
  named *Fedora*, and the installer's staging entry *alo OS* and its area are
  left behind. This task names the entry as alo OS, puts Windows Boot Manager
  directly behind it (term 2), and removes the staging area once the installed
  system has started.
- **Constraint:** replacing Windows is not built here. It is an explicit,
  twice-confirmed mode ADR 0023 allows, and it waits for a task of its own
  after the alongside road has been certified.

### 5. Released from GitHub, and the page a person downloads from

**Status:** **Done, 2026-09-16** (`updates/released-from-github-and-the-page-a-person-downloads-from.md`),
with **one half of one line decided rather than built**:
[ADR 0046](../decisions/0046-the-installer-is-signed-by-a-certificate-a-person-holds.md)
answers *signs the executable* the way ADR 0036 answered it for the image — **a
machine builds, a person signs** — because Authenticode needs a certificate this
repository does not have and nothing in it said who should hold one. So
`.github/workflows/release.yml` runs on a tag `image/pinned.toml` names, builds
the boot environment and `alo-installer` with that environment's list compiled
into it, writes `SHA256SUMS`, and creates a **draft** Release whose body is the
committed `image/release-notes.md`; it never signs and never reaches for a
secret, and `crates/alo-image` (`releasing.rs`) fails this repository's build if
it ever does. The notes name the registry, the tag, the pinned digest and the
Secure Boot state the installer accepts, each held to `image/pinned.toml` and to
what `alo_installer::decide` really refuses; the README's *Try it* is held to
`docs/hardware.md`'s table, to *Windows stays*, and to one Secure Boot sentence
that never advises. **What stays the owner's:** obtain a code-signing
certificate, and sign `alo-installer.exe` and publish the draft for each Release
(ADR 0046, *What it costs*) — and accept or amend ADR 0046 itself.
**Depends on:** 3.

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

**Status:** blocked — **on task 4 alone**, and on the owner at the laptop;
nothing in this repository can tick it. **Narrowed 2026-09-25:** this line named
tasks 1–5, 8, 9, 10 and 11, and eight of those nine are finished — 1, 2, 3, 5,
8 and 9 on 2026-09-15 and 2026-09-16, 11 on 2026-09-21 and 10 on 2026-09-22.
Only task 4 is unfinished, and it is in progress on the development PC. A
blocker that outlives its cause makes takeable work look untakeable; this one
also made the laptop look far away when one task stands between this repository
and being ready for one. Task 11 is not optional before
this one: a filesystem is chosen at install and cannot be converted, so a laptop
installed before it lands could never undo what an agent did without being
reinstalled ([ADR 0045](../decisions/0045-what-undoing-rewinds-to.md), accepted
2026-09-16). **Depends on:** 4, 5.

**Sequenced by the owner on 2026-09-25 as one of the last two things in the
release** — see [what closes this release, and in what
order](updates/what-closes-v0-0-5-and-in-what-order.md). No lane is sent at
this until it is unblocked; it closes nothing else, and it is the owner's to
perform.

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

**The machine available has no discrete graphics, said 2026-09-21 by the owner
before the install rather than discovered at it.** So *the GPU works on first
boot* cannot be answered there, and it is not answered by a machine that has no
card to fail on: the other three observations are taken, that one stays open
with the machine still needed named beside it, and `ROADMAP.md`'s box for it
does not move. **An observation nobody could have made is not a pass**, and a
run that ticked it because nothing went wrong would be the same mistake as a
build that checked an executable bit.

Task 22 of `v0-5-the-models-measured-plan.md` is the code half of that line —
written so that the processor road is measured on a machine with no card, and
the card's road waits for hardware the way the chip's half of encryption does.

### 7. Replace Windows — the road with no way back

**Status:** ready. **Depends on:** 4.

[ADR 0023](../decisions/0023-installed-from-the-machine-it-replaces.md) §4:
Windows is *either retained alongside (default where disk allows) or replaced
(explicit choice, twice confirmed)*. Task 4 built the first. This is the
second, and it is the only task in this repository that destroys somebody's
data on purpose.

It depends on task 4 rather than on the laptop's certification because it
reuses all of it — the checks, the staging, the boot entry, the environment
that pulls and installs. What is new is the one thing that cannot be undone,
and it is written as its own task so that nobody adds it to another one as a
flag.

- **Acceptance:** *Replace Windows* is a mode a person chooses in the
  installer, never a default, never preselected, and never reachable by
  clicking past a screen: it asks twice, and the second time the person
  **types the name of the disk and the word that names what is lost**, in
  their own language, with the sentence saying what will be destroyed —
  Windows, the applications on it, and every file on that volume — in the
  vocabulary rather than in English written into the program; before anything
  is written it says, from what it read rather than from assumption, how much
  is on the Windows volume and the date of the newest file there, because
  *20 GB of documents last touched this morning* is what makes a person stop;
  it refuses outright, with a reason, when there is a BitLocker volume it
  cannot confirm is unlocked and backed up, when the machine has one disk and
  no recovery media exists, and when the person's answer does not match; the
  whole-disk install then runs through task 2's environment — `bootc install`
  over the whole disk rather than beside it — and the machine boots to alo OS
  with no boot entry for anything else; and a test in a virtual machine with
  a real Windows and known files on it walks the refusals one at a time and
  then the accepted road once, checking after each refusal that Windows still
  boots and the files are byte-for-byte what they were.
- **Constraint:** **the point of no return is one place in the code, named,
  and everything before it is reversible** — the same shape ADR 0023 asks of
  the alongside road, with the difference that after it there is no bail-out
  and the program says so in the sentence that precedes it. No telemetry, no
  "are you sure?" checkbox, no timer that proceeds on silence. This task
  never runs on the certified laptop: ADR 0033 §2 keeps that machine on the
  alongside road, and the evidence ledger records certification made that
  way. Recovery media is named as the thing a person needs if they change
  their mind afterwards, honestly, because after this there is no Windows to
  run a program from.

### 8. A refusal writes nothing the installer does not own

**Status:** **Done, 2026-09-15** (`updates/a-refusal-writes-nothing.md`). The first
experiment found it: the writer was the test's virtual machine. OVMF's build
without SMM was started with flash only SMM code may write, so its variable writes
never took and it saved them as `NvVars` onto the first FAT it found, the staged
`ALO-INSTALL` partition (`docs/quirks.md`, *OVMF without SMM saves its variables
onto a FAT disk when its flash is SMM-only*). It was neither alo OS nor a
laptop's firmware, so the plan's third outcome applies: the machine is fixed, the
test holds that **the whole first disk** is unchanged, and it also holds that the
firmware wrote its own flash. The sentence is unchanged. **Depends on:** 2.

Split from task 2 on 2026-09-15. The not-genuine refusal said *this download is
not a genuine alo OS, so nothing was changed*, wrote nothing to the second disk —
and the first disk's contents changed, at three mebibytes, all inside the staged
`ALO-INSTALL` FAT partition (its first two mebibytes and one cluster about 218 MiB
in); Windows' partitions and the partition table were untouched. A sentence that
says *nothing was changed* while a disk changed is the most serious finding the
installer has had, whoever did the writing.

- **Acceptance:** the cause is found and written into `docs/quirks.md` with its
  evidence, starting from the report's two experiments in order — (1) the
  diagnostic *installer's own disk* boot run with the test's exact machine flags
  (`q35,smm=on`, `-global cfi.pflash01.secure=on`, 4 CPUs, 3 GB), which tests
  whether the firmware's FAT driver writes to a FAT it enumerates; (2) if not, the
  not-genuine road with no appended initramfs archive. **If the writer is the
  firmware**, a real laptop's firmware does it too: the test's assertion becomes
  *Windows' partitions and the partition table are byte-for-byte unchanged, and the
  installer's own partition changes only as a firmware booting from it changes it*,
  with the evidence that it is the firmware, and the sentence stays true because
  alo OS changed nothing. **If the writer is ours** — the environment, `cosign`, a
  unit, a generator — it is stopped, and the test holds that the whole first disk
  is unchanged. Either way
  `a_release_signed_by_another_key_writes_nothing_and_says_so` passes when run
  (`--include-ignored`), and the run is pasted into the report.
- **Constraint:** the sentence *so nothing was changed* is not reworded to fit a
  change. If something of ours writes, the code changes, not the words.

### 9. With Secure Boot on, the staged loader starts

**Status:** **Done, 2026-09-16**, for the part this task ends at since its split
(`updates/the-boot-environment-says-why-an-install-stopped.md`): the environment
notes the last lines a failing program complained of on the machine's serial
lines and its log, never on the screen; the `bwrap` `pivot_root` failure is in
`docs/quirks.md` with the run's console as its evidence; the refusal tests pass —
the scripted ones, and both refusals in a virtual machine run by name, the
not-genuine one now also finding the checker's own complaint on the serial line;
and every virtual-machine test removes the disks it made, pass or fail. **Found on
the way:** the test of Secure Boot's refusal could never have passed, because the
stager copied the built environment whatever it was handed, so the changed loader
was never staged; it copies what it is handed now, and the firmware refuses the
changed loader (*Access Denied*). The install that finishes is task 12.

*Before it was done:* taken on 2026-09-16 by the third PC, whose lane has the disk.
It was scheduled for a machine with 50 GB free, which the development PC is not.
The supervisor there clears the idle lane's build directory before this task's
virtual machines start, and every run checks for 15 GB free first. Measured on
the development PC on 2026-09-16: one run of this task left 12 GB of virtual
disks and 11 GB of container images and took the drive from 20 GB free to 0.4 GB,
twice in one night, crashing the distribution and failing another lane's gates
both times. The rule added to this plan's header did not prevent it, because a
rule in a document is not a bound on a running test. So this task waits for a
machine that can hold it, and the lane on the development PC steps over it rather
than filling the disk again. **Depends on:** 2.

**Most of it is answered, and published on 2026-09-16**
(`updates/with-secure-boot-on-the-staged-loader-starts.md`), so whichever machine
takes this task starts from a measurement rather than from the fault:

- **The fault is the firmware build — not our files, and not Secure Boot.** Nine
  boots of the same staged partition, one variable at a time: Ubuntu's
  `ovmf 2025.11-3ubuntu7` page-faults even with no certificates enrolled, while
  Fedora's `edk2-ovmf-20250812-21.fc42` starts the same signed chain **with
  Secure Boot enabled**, saying *Page fault fixups needed … the guest OS boot
  chain is not NX clean … shim is older than v16*. The remedy upstream names is
  shim 16, which is upstream's to ship and never ours to build (ADR 0011).
- **So the test takes its firmware out of the pinned base** rather than from
  whatever the host packages, and a new test changes one byte of the staged
  loader and finds that firmware refusing it — *Access Denied -- rejected
  probably by Secure Boot* — with no kernel started and the first disk unchanged.
  Secure Boot is never switched off (ADR 0033 §4).
- **And it found what nothing else could have:** the loader's entry read the
  person's chosen disk from `${cmdpath}`, which the base's own loader leaves
  **empty** — so no install could ever have succeeded, under any firmware. It now
  reads `${config_directory}`, with tests in both halves refusing the other.
- **Where it now stops:** with Secure Boot on, the environment reads the choice,
  verifies the signature and begins installing; `bootc install` then ends without
  finishing, and **the console does not say why**, because the environment says
  its own sentences and not the installer's. Making it say them is the first step
  for whoever takes this task, and is worth doing for its own sake — the person
  watching has no other window.
- **And now it says why — measured on the third PC, 2026-09-16.** With the
  environment passing the installer's own lines through, the same run under Secure
  Boot said: *Deploying container image...done (3 minutes)*, then *error:
  Installing to disk: Installing bootloader: Probing bootupd --filesystem support:
  Subprocess failed* and *bwrap: pivot_root: Invalid argument*, and then the
  environment's own sentence that the chosen disk may hold part of alo OS and
  nothing else on the computer changed. So the image deploys, and the install
  stops at the bootloader: `bootc` probes `bootupd` inside a `bwrap` sandbox, and
  `pivot_root` is refused where the environment runs it. The worker that measured
  this reached the supervisor's ninety-minute limit before handing over, so the
  work is in the tree and this task was split rather than given longer.

**Split again, 2026-09-16.** A task that cannot finish inside the worker's limit
is a phase (`tools/kernel-loop/src/worker.rs`), and one emulated install takes
most of that limit on its own. **This task ends** at the environment saying the
installer's own failure, the fault written into `docs/quirks.md` with the run's
console as evidence, and the refusal tests passing. **The install that finishes
and boots to `alo-agentd` with Secure Boot on is task 12**, below.

Split from task 2 on 2026-09-15. Under QEMU q35 with OVMF's Secure Boot build and
Microsoft's enrolled certificates, the firmware page-faults (`#PF`, a write to a
present page, `W:1 P:1`) starting the staged loader from the installer's
partition, before Linux, and hangs.

- **Acceptance:** the fault is located and written into `docs/quirks.md` with its
  evidence, starting from the report's steps in order — boot the same partition
  with OVMF's build without Secure Boot, to separate *the files* from *Secure
  Boot*; then shim alone with a trivial second stage, to separate shim from the
  loader; and if it is this OVMF build's memory protection rather than the files,
  show it with a second firmware build and name both versions. Then
  `the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`
  passes **with Secure Boot on** — the staged loader starts, the environment
  pulls the pinned release, verifies it, installs onto the second disk with the
  first disk's Windows partitions unchanged, and the second disk boots to
  `alo-agentd` — and the run is pasted into the report. If the only way through is
  a different signed shim or loader than the base ships, that is a decision record
  first, because what the laptop's firmware trusts is ADR 0033 §4's.
- **Constraint:** Secure Boot is never switched off to make the test pass (ADR
  0033 §4). A person is never told to disable it. No shim or loader is patched or
  built by us (ADR 0011).

### 10. The installer, walked on a real Windows in a virtual machine and killed at every step

**Status:** **Done, 2026-09-22.** **For the part it ends at since its split** —
everything below under *Measured*. The Windows partition byte for byte after
each kill is **task 19** and is **not** done, and so are the NVMe and Hyper-V
SCSI names and the MSVC build, carried there; this line carries the mark the
supervisor reads, and the qualifier beside it so no person reads the mark as
the whole task (`updates/the-installer-walked-on-a-real-windows.md`).
**Depends on:** 3.

**Measured, under the Rust test run by name:**
- **The install.** A Windows 11 Enterprise Evaluation installed itself
  unattended into a QEMU/KVM machine and reached a desktop session (the test
  passed). It is QEMU because this account cannot manage Hyper-V. Secure Boot is
  off because the shipped installer refuses it on.
- **The kills.** The installer, cross-built for Windows with a genuine
  environment, ran elevated there. It was killed after each of the seven steps,
  and all seven landed exactly: 1–3 by freezing, and 4–7 by holding the next
  program it starts. Windows restarted to its desktop session after every kill.
- **The start partition** never changed beyond the controls, and nothing on
  either partition was unreadable.
- **The entry.** The installer now writes its start-up entry itself, with no
  optional data, and reads it back from the firmware variable. On its own
  restart the firmware started it from the area, and shim went straight to
  GRUB.
- **The road.** It passed: the environment found
  `ata-QEMU_HARDDISK_ALOTARGET1`, the name the installer wrote.

**Split off, not done — task 19:** the Windows partition byte for byte. The
run found 14 changes beyond the controls, under the user's profile and also
under `ProgramData` and `Windows\` (Defender's scan history, a WMI file), which
no control explains yet. The kill test no longer asserts that claim; it prints
what it finds for task 19. Also carried there: the NVMe and Hyper-V SCSI
names, and the release's MSVC build.

**Found for task 18:** on the road, `bootc` stops with *Creating rootfs: No
such file or directory* after the environment has found the disk.

> *Before 2026-09-21:* **one of the two conditions this task waited on was
> measured away on the development PC (Intel Core Ultra 7 155U), 2026-09-20.
> The other — the disk — was then cleared too: the host had 62 GB free on
> 2026-09-21, and the walk ran with 31–42 GB free throughout.**
>
> - **Disk — still a real blocker, and the plan was right.** An earlier draft of
>   this note claimed 805 GB free. That number is `df` **inside WSL**, and it is
>   the ext4 vhdx's *virtual* size. The host's C: is **474 GB with about 13 GB
>   actually free**; writing ~10 GB of ISOs and disk images took it to **zero**,
>   which stopped WSL from starting and put every lane on the machine at risk.
>   `fstrim -v /` returned the space. **This task's *tens of gigabytes per run*
>   does not fit on the development PC either** — `docs/quirks.md` carries the
>   trap under *`df` inside WSL reports the virtual disk's size*.
> - **Hardware virtualisation — cleared.** `/proc/cpuinfo` shows `vmx`, `/dev/kvm` exists,
>   and it accelerates rather than merely initialising: the same Alpine 3.21
>   image reached a login prompt in **12.4 s under `-accel kvm` against 27.7 s
>   under `-accel tcg`**, and an Ubuntu 24.04 guest under OVMF went from cold
>   start to an SSH login in **23 s**. QEMU here is 10.2.1, not 8.2.2.
> - **The Windows this task needs is obtainable, and Setup runs.** The Windows 11
>   Enterprise *Evaluation* ISO is a direct, unauthenticated **4.5 GB** download
>   from `software-static.download.prss.microsoft.com` — no key and no form. On
>   2026-09-20 it booted in a KVM guest on that machine — q35 with OVMF, AHCI
>   disks (virtio would need drivers Windows Setup does not carry), a `swtpm`
>   TPM 2.0, a 64 GB system disk and **a second empty 32 GB disk**, the shape
>   this task's acceptance asks for — and the `autounattend.xml` was accepted:
>   Setup partitioned disk 0 and reached **15% of *Installing Windows 11*** with
>   no prompt. **It did not finish.** It was stopped there because the host's C:
>   ran out of space, per the disk note above. So *an unattended Windows 11
>   installs in a KVM guest here* is **not** yet measured; what is measured is
>   that the media, the firmware, the answer file and the accelerator all work,
>   and that the disk is what stops it.
>
> **What remains** is this task's own work — copying a release build of the
> installer in, killing it at each of `staging.rs`'s seven steps, and restarting
> — **plus a machine with real room**, which is still not this one. Nothing here
> ticks the task.

~~**For a machine with 50 GB free**, for task 9's reason: a
real Windows in a virtual machine is tens of gigabytes of disk per run, and the
development PC has about 25 GB at its best.~~

~~**And for hardware virtualisation, as found on the third PC, 2026-09-16.** The third
PC has the disk but no hardware virtualisation: it is a VMware guest, and
`qemu -accel kvm` refuses there (`docs/quirks.md`), so every virtual machine it runs
is emulated. Measured there, an emulated install of alo OS alone took forty-six
minutes. This task installs a Windows unattended and restarts it to its desktop at
least eight times, once after each step the installer is killed at and once for the
whole road. That has not been timed under emulation, but at the measured speed one
run is many hours, far past a worker's ninety-minute limit, so the third PC does
not take it. It needs a machine where Hyper-V or KVM works.~~

Split from task 3 on 2026-09-15. `crates/alo-installer` is written and every
decision in it is tested against a scripted Windows; its checks have been run,
reads only, on a real Windows 11. What no test has yet done is let it change a
real Windows: the account the tests run under on the development machine cannot
manage Hyper-V (task 2's report), and nothing in this repository installs a
Windows into a virtual machine. Task 3's report names the three things the
scripted machine cannot show, which this task is for: that the storage cmdlets
and `bcdedit` do what `crates/alo-installer/src/program.rs` asks of them on a
Windows that is really running; that the NVMe, SATA and Hyper-V SCSI names
`naming.rs` makes are the names the environment finds under `/dev/disk/by-id/`;
and that a copy of `{bootmgr}` with a `device` and `path` is an entry the
firmware starts.

- **Acceptance:** a test in `crates/alo-installer/tests/`, ignored in the suite
  and run by name, starts a Hyper-V generation 2 VM (or QEMU with OVMF, if
  Hyper-V is still out of reach, saying which) holding a Windows the test
  installed unattended, with a second empty disk; it copies a release build of
  the installer and a built environment into the guest and runs it elevated,
  typing the second disk's name; and for **each** step of
  `crates/alo-installer/src/staging.rs` — after the shrink, after the area is
  made, after it is prepared, after the copy, after the entry, after its letter
  is taken, and after the next start is set — kills the installer there,
  restarts the VM, and shows Windows starts to its desktop session, with the
  Windows partition's files byte-for-byte what they were. Then it runs the whole
  road once, and shows the firmware starts the environment on the next restart
  and the environment finds the disk by the name the installer wrote. The run is
  pasted into the report. Whatever reality says differently from `program.rs`
  or `naming.rs` goes into `docs/quirks.md` and the code, in the same change.
- **Constraint:** nothing here runs on the laptop, or on the development
  machine's own disks; every destructive step is in a virtual machine the test
  made. Secure Boot is off in that VM only because the shipped installer refuses
  it on (ADR 0033 §4), and the report says so.

### 11. The disk alo OS is installed onto can hold an undo

**Status:** **Done, 2026-09-21.** On the development PC.
`crates/alo-installing/src/writing.rs` names `btrfs`, from the one place it is
named (`alo_image::THE_ONLY_FILESYSTEM`); `docs/booting.md` names the same one;
and `crates/alo-image/tests/one_filesystem_and_it_can_hold_an_undo.rs` counts
both, so a second filesystem on the road to a person's disk is a failing test.

**What the base makes of it was measured, not assumed**, on the pinned release
`0.0.5` installed with `--filesystem btrfs` and booted with KVM. Four entries in
`docs/quirks.md` carry the commands: **the base makes no subvolume of its own** —
`btrfs subvolume list -a -p -u /sysroot` prints nothing, and `/boot`, `/etc`,
`/sysroot` and `/var` are four binds of four directories in subvolume 5;
**a person's home lands in `/var/home`, which is an ordinary directory** and
empty until somebody signs in, so making each home a subvolume is still the
accounts lane's; and **taking a read-only snapshot needs no capability while
removing one needs `CAP_SYS_ADMIN`**, because `bootc install` sets no
`user_subvol_rm_allowed` and we add no mount option of our own. That last one is
a cost ADR 0045 did not anticipate: its expiry window, its oldest-go-first under
disk pressure and its *forgetting is one act* all need a privileged remover. The
fourth entry is the trap that cost this task two boots and a wrong reading — a
snapshot into a destination that already exists is made *inside* it and answers
`Read-only file system`.

**The machine did it, not a document.** `alo-agentd` came up `ActiveState=active
SubState=running` on the btrfs install; a read-only snapshot of a home subvolume
was taken, held its bytes, refused a write, and was removed again, with the whole
run in `updates/a-disk-that-can-hold-an-undo.md`. And
`crates/alo-installing/tests/a_btrfs_disk_keeps_what_an_update_passes_over.rs`
holds the rest: a btrfs install, known bytes written into a home subvolume and
into `/var/lib/alo`, a `bootc switch` to a second build, a `bootc rollback` back,
and every byte, the subvolume and the read-only snapshot found on both sides.
`test result: ok. 1 passed; 0 failed`, in 161.84 s with KVM, exit 0. The run's
disks and images were removed and `fstrim` returned 12.5 GiB.

**A machine already on ext4 is not converted and does not claim an undo**:
`crates/alo-keeping-up/src/putting_back.rs` answers *not yet on this machine*,
and goes on doing so until the home subvolume, the bracket and the broker's verb
land in their own lanes.

*Before it was done:* ready. **Depends on:** 2, 15 — its acceptance boots a virtual-machine
install to `alo-agentd`: task 13 saw an install finish and the disk boot under
Secure Boot on 2026-09-16, and `alo-agentd` failed there. Task 14 found why and
fixed the image, and a release carrying that fix is task 15.

Added 2026-09-16 by [ADR 0045](../decisions/0045-what-undoing-rewinds-to.md),
accepted that day: ★ *undo what the agent did* rewinds from the base's own
snapshot, and a snapshot needs a filesystem that has them. `crates/alo-installing`
installs with `--filesystem ext4`, which has no subvolume and no snapshot, and
**a filesystem is chosen at install and cannot be converted afterwards**. So a
machine installed today can never undo without being reinstalled — and the
certified laptop must not be the first machine in that position, which is why
this task comes before task 6.

- **Acceptance:** `bootc install to-disk` names **btrfs**, and what it makes is
  measured rather than assumed — which subvolumes the base creates of its own
  accord, where a person's home lands, and what capability taking a snapshot needs
  on the pinned kernel — each written into `docs/quirks.md` with the command that
  showed it; a virtual-machine install then boots to `alo-agentd`, takes a
  read-only snapshot of the home subvolume and removes it, with the whole run in
  the report; **an update and a return to the build before leave the home
  subvolume and `/var/lib/alo` untouched**, held by a test that writes known files,
  updates, rolls back and finds them; and `crates/alo-image` holds the installer to
  naming one filesystem, so a second one cannot appear in a later change unnoticed.
- **Constraint:** the base is rented and unmodified (ADR 0011) — no partitioning,
  no mkfs and no subvolume layout of ours beyond the arguments `bootc install`
  takes. Nothing here decides what a snapshot is for; that is ADR 0045's. A machine
  already installed on ext4 is not converted and not silently left claiming undo:
  it answers *not yet on this machine*.

### 12. With Secure Boot on, the install finishes and boots

**Status:** **Done, 2026-09-16**, for the part this task ends at since its split
(`updates/the-installers-sandbox-pivots-under-its-own-root.md`): the reason
`bwrap` cannot `pivot_root` is located from a run and written into
`docs/quirks.md`, and the environment is changed, by configuration only, so the
installer runs where the base's own sandboxing works — shown in a boot of the
environment's own initramfs. **The install that finishes and boots with Secure
Boot on is task 13**, below. **Depends on:** 9.

**What the run found.** The environment runs from the kernel's initial root
file system — it never switches root, because there is no root to switch to —
and `pivot_root(2)` refuses any caller whose root is that absolute root. Booting
the base's kernel with an initramfs made from `alo-installing.conf` by the base's
own `dracut`, the same `bwrap` line `bootc` runs said *bwrap: pivot_root: Invalid
argument* from a unit on that root (`/proc/self/mountinfo`: `1 1 0:2 / /`, its own
parent), and started `bootupctl` from a unit whose root is the environment bound
again (`122 110 0:2 / /`). `image/installing/run-alo-installing-root.mount` binds
it (`rbind,rslave`) and `alo-installing.service` takes it as `RootDirectory=`;
`crates/alo-installing/tests/what_the_environment_carries.rs` refuses the unit on
the initramfs's root again, a plain or shared bind, and a root bound elsewhere.

**Split, 2026-09-16.** The acceptance's install run did not fit the machine that
took this task: the drive holding the distribution's disk had 5.4 GB free, under
this plan's 15 GB, and one emulated run is most of an hour on top of building the
environment. The plan's own rule is to say so and stop rather than start the run,
and *a task that cannot finish inside the worker's limit is a phase*.

Split from task 9 on 2026-09-16, when the environment first said why the install
stopped: the image deploys, then `bootc install` fails *Installing bootloader:
Probing bootupd --filesystem support* with *bwrap: pivot_root: Invalid argument*.
`pivot_root` is refused for a process whose root is the initial RAM filesystem,
which is where the environment may be running the installer — the task finds out
rather than assumes it.

- **Acceptance:** the reason `bwrap` cannot `pivot_root` in the environment is
  located and written into `docs/quirks.md` with its evidence, from a run rather
  than from reading; the environment is changed so the installer runs where the
  base's own sandboxing works — configuring how the environment starts, never
  patching `bootc`, `bootupd` or `bwrap` (ADR 0011). *(The install run that was
  also here is task 13.)*
- **Constraint:** Secure Boot is never switched off to make the test pass (ADR 0033
  §4). No shim, loader or installer is patched or built by us (ADR 0011).

### 13. With Secure Boot on, the install onto the second disk finishes and boots to the agent service

**Status:** **Done, 2026-09-16**, for the part this task ends at since its split
(`updates/the-install-finishes-under-secure-boot-and-the-installed-disk-boots.md`):
**with Secure Boot on, the install onto the second disk finishes, and the second
disk boots.** The *No such file or directory* was `bootc` starting `fstrim` to
finish the file systems it made, which nothing put in the initramfs; it is in
`docs/quirks.md` with both consoles, `alo-installing.conf` carries it, and
`crates/alo-installing/tests/what_the_environment_carries.rs` refuses a list
without it or the three programs beside it. The run after the change, emulated,
said *alo OS is installed* after 46 minutes with the first disk unchanged; the
installed disk then started through the base's signed shim with Secure Boot on
and reported `alo-boundaryd` active — and **`alo-agentd` failed**, which is task
14. The test now ends the moment an install says it could not finish, and prints
the installed machine's own account of why a service is not running. **Split,
2026-09-16:** one emulated run is most of an hour, and the worker had spent its
limit on it. **Depends on:** 12.

*Before it was done:* ready on the third PC, with the emulated run already done
outside any worker's limit (below), so the worker starts from its serial line. It
was scheduled for a machine with at least 15 GB free and either hardware
virtualisation or a worker limit that holds one emulated run beside building the
environment.

Split from task 12 on 2026-09-16. The bootloader's sandbox now pivots in the
environment (task 12); nothing after that step has yet been seen to run, so the
next failure, if there is one, is this task's to read from the serial line.

**The run, measured on the third PC on 2026-09-16, by the supervisor rather than
a worker.** Under emulation, with Secure Boot on and 159 GB free before it began,
`the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`
went further than any run before it. The installer's root was mounted, the
download was checked as genuine, and `bootc` said *Deploying container
image...done (3 minutes)*, with no `bwrap` error. Then it said **`error: Installing
to disk: No such file or directory (os error 2)`**, and the environment said its
own sentence that the chosen disk may hold part of alo OS. So task 12's change
holds, and the next failure is something the install looks for after deploying
and does not find from inside the bound root — the likely places are the firmware
and EFI paths, but that is for this task to show from a run, not to assume. The
whole console is kept on that machine as `C:\dev\setup\task13-install-run.log`.

**And a defect the run showed in the test itself.** After an install that fails,
the environment waits for a person to turn the computer off, and never powers off
by itself. The test waits for power-off for
`processor.allowing(Duration::from_secs(60 * 60))`, which under emulation is
several hours, so it sat for 71 minutes after the failure was already on the
serial line until the supervisor stopped the machine. The test should end as soon
as the environment says the install could not be finished, and report that line,
the way `alo-updating`'s test ends on *Freezing execution*.

- **Acceptance:**
  `the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`
  passes **with Secure Boot on** — the staged loader starts, the environment pulls
  the pinned release, verifies it, installs onto the second disk with the first
  disk's Windows partitions unchanged, and the second disk boots to `alo-agentd` —
  with the run pasted into the report. Whatever stops it next goes into
  `docs/quirks.md` with its console, and is changed by configuring the
  environment, not by patching an engine.
- **Constraint:** Secure Boot is never switched off to make the test pass (ADR 0033
  §4). No shim, loader or installer is patched or built by us (ADR 0011). A worker
  checks for 15 GB free before every run and removes the run's disks and images
  when it ends, pass or fail. If a single run cannot fit inside the worker's limit,
  that is a finding for the plan, not a reason to leave a run behind.

### 14. The disk installed under Secure Boot runs the agent service

**Status:** **Done, 2026-09-16**, for the part this task ends at since its split
(`updates/the-agent-service-can-reach-the-boundary-on-an-installed-disk.md`). **Why
`alo-agentd` failed is located from the installed machine's own account, and fixed
in the image.** systemd mounts `/sys/fs/bpf` `1700 root:root`, so the person's
service could not pass through it to the boundary the loader had pinned beneath it,
and said there was none. It is in `docs/quirks.md` with both consoles.
`image/usr/lib/tmpfiles.d/alo.conf` gives the agent's group passage and nothing more
(`z /sys/fs/bpf 0710 root alo-agent -`), and `crates/alo-image/src/reaching.rs`
refuses an image without it or with anything wider. The same release, with only that
line added and Secure Boot on, ran `alo-agentd`.
**The named test does not pass yet, and cannot inside this task.** It installs the
*pinned* release, `0.0.1`, which does not carry the line, and only the owner builds,
signs and pins a release (ADR 0036). The run that passes whole is task 15, below.
**Depends on:** 13.

*Before it was done:* ready on the third PC, or any machine with 15 GB free that
holds one emulated install run (about an hour) inside a worker's limit, or has
hardware virtualisation.

**A quicker road to the installed machine, measured on the third PC.** A worker does
not need the emulated install to read an installed disk. `bootc install to-disk
--via-loopback --wipe --filesystem ext4` from the pinned release itself, run
natively in `podman --privileged`, writes the same disk in about 7 minutes. With
`console=ttyS0` added to a scratch copy's boot entry, that disk boots emulated under
Secure Boot to the watching unit in about 5. Use it for diagnosis only. The
acceptance is still the named test, which installs through the environment.

Split from task 13 on 2026-09-16. With Secure Boot on, the environment installed
the pinned release onto the second disk (*alo OS is installed*, the first disk
unchanged), and the installed disk started through the base's own signed shim and
loader. The test's watching unit, handed over through the firmware tables with a
drop-in that starts `user@1000.service` the way a sign-in would, then said:

```
Id=alo-boundaryd.service
ActiveState=active
SubState=exited

Id=alo-agentd.service
ActiveState=failed
SubState=failed
```

and nothing more, because the installed machine's console is not the serial line.
From 2026-09-16 the same unit prints `systemctl status` and the boot's journal for
`alo-boundaryd`, `alo-agentd` and `user@1000.service` between `ALO-WHY-BEGIN` and
`ALO-WHY-END`, and the test's failure quotes it — so the next run says why rather
than that. Whether the fault is the image (its units, `alo` user, tmpfiles, the
machine description), the test's stand-in for a sign-in, or the boundary on this
kernel under emulation is this task's to show from that run, not to assume.

- **Acceptance:**
  `the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`
  passes **with Secure Boot on** — the whole test, including *the first disk is
  unchanged when alo OS booted*, which the run of 2026-09-16 never reached — with
  the run pasted into the report. Why `alo-agentd` failed goes into
  `docs/quirks.md` with the machine's own account of it. If the fault is in the
  image, it is fixed there with its own test in the crate that holds that file;
  if it is the test's stand-in for a sign-in, the stand-in changes and the report
  says why a real sign-in is not affected.
- **Constraint:** Secure Boot is never switched off to make the test pass (ADR 0033
  §4). No shim, loader or installer is patched or built by us (ADR 0011), and
  nothing on the installed disk is changed by the test to make a service start. A
  worker checks for 15 GB free before every run and removes the run's disks and
  images when it ends, pass or fail.

### 15. A release that carries the way to the boundary, installed under Secure Boot to the agent service

**Status:** done. **Depends on:** 14.

**Done, 2026-09-20, on the development PC.** The named test passed whole, with
Secure Boot on, against the release pinned now — `0.0.4` at
`sha256:48bd5f319abcecfa832eb9a5b0b2f7cd06815b1c30c43b781499500ec14c3858`,
revision `b41b4b5e`, verified against `signing/alo-os.pub` **inside the machine**:
`test result: ok. 1 passed; 0 failed`, in `1303.36s`, exit 0. The guest kernel's
own account of its firmware, twice — `secureboot: Secure boot enabled` — and the
environment said every sentence in order with none of the four refusals anywhere
in the log. **The two assertions no run had reached are both in it:**
`alo-agentd` is `active (running)`, Main PID 1137, on the disk the installer
wrote, and *the first disk is unchanged when alo OS booted*. So task 14's passage
through `/sys/fs/bpf` is in a released image and works **through the installer**,
which is the whole of what this task existed to show. Nothing on the installed
disk was changed by the test to make a service start.

**What the machine changed, which is the finding worth keeping.** The 2026-09-18
run stopped because every guest on the third PC is emulated. This PC's hardware
virtualisation was used (`-accel kvm -cpu host`, the test's own probe having
started a machine and seen it stay up), and the install that ran at about 124 MB
a minute emulated — 4.7 GB in 38 minutes, unfinished — finished **whole in about
eleven minutes**. The previous report's *does not fit a worker's window* is
therefore true of an emulated machine and false of this one: it ran to the end
with lane A's `cargo test --workspace` beside it at load average 8.3 on twelve
processors.

**One line in the log is not a defect, and is recorded so nobody files it as
one.** On the installed disk the agent service says `no translations were
loaded: /usr/share/alo/translations could not be read`. This is the first time
that has been seen on a machine installed from a **released** image, but task 14
already recorded it with the reason and the reason still holds: **no translation
exists in this repository yet**, checked again here, so there is nothing for the
image to carry and `alo-saying` is correctly reporting an empty case. Nothing
goes to `docs/quirks.md` — nothing stopped, and that file is for others'
misbehaviour. Whether the image should ship the empty directory remains the open
question task 14 put to `alo-saying`'s owner. Report:
`updates/the-install-under-secure-boot-reaches-the-agent-service.md`; the run's
disks were removed and `fstrim` returned 20.5 GiB to Windows.

**Re-pointed at 0.0.4 on 2026-09-20.** This line named `0.0.2` until then, and
by that morning it was naming the release before last: 0.0.3 was pinned on
2026-09-19 and 0.0.4 that evening. Anybody following it would have installed
**the release whose document converter cannot start** — 0.0.2 and 0.0.3 both
shipped an engine that died at launch for want of twelve shared libraries, which
0.0.4 is the fix for. A record that names a superseded release is not merely out
of date; it sends somebody to the wrong bytes.

**It wanted a machine with a virtual machine that is not emulated, and it got
one — this condition is gone.** Cleared 2026-09-20 by the run above: on the
development PC the test's own probe started a machine with `-accel kvm` and it
stayed up, the guests ran `-accel kvm -cpu host`, and the whole named test
finished in `1303.36s`. What follows is why the condition was real, kept because
it still describes the third PC. The 2026-09-18
run stopped because every guest on the third PC is emulated — it has no `vmx`,
no `svm`, and `qemu -accel kvm` there answers *failed to initialize kvm*. The
development PC has a working `/dev/kvm` (measured 2026-09-20: the same guest boots
in 12.4 s accelerated against 27.7 s emulated), so this belongs on that machine
rather than on the one that first took it.

**The run of 2026-09-18, and why it did not finish inside a worker's window.** A
worker on the third PC (`AGAI01`) started the named test at 11:44 and had to stop
it, and the measurement is the finding this plan asks for rather than a reason to
try again unchanged. The environment built in 16 minutes; the install machine
started at 12:01 with Secure Boot on, and **against release `0.0.2` the environment
said every sentence the acceptance asks for, in order** — the choice read, the disk
found and checked, the network reached, and *This is a genuine alo OS*, which is the
owner's signature over the pinned digest verified inside the machine. Then it wrote
the second disk at **about 124 MB a minute** — 4.7 GB in the thirty-eight minutes
before it was stopped — against the 46-minute whole install task 13 measured on the
same PC on 2026-09-16. The image is 5.79 GB compressed over 80 layers and lands
decompressed, so at that rate the install alone needs an hour or more after it
starts, and the second boot follows it; sixteen minutes had already gone on building
the environment. **What differs from 2026-09-16 is not the release
but the machine:** WSL has four processors here, the run is emulated (TCG), and
another lane was running `cargo test --workspace` and an `apt-get` beside it the
whole time — load average 8.5 on four processors. So *this PC holds one emulated
install run inside a worker's limit* is true only when this PC is not also gating
another lane. A worker taking this task should have the machine to itself, or the
run belongs to a supervisor outside a worker's window the way task 13's did.

**So, before starting this run:** look at what else is on the machine
(`uptime`, `pgrep -af cargo`). On four processors, with another lane's suite beside
it, the window is not enough and the honest thing is to say so and stop rather than
spend it — the way this plan already asks a worker to stop for want of disk. It also
costs sixteen minutes to build the environment from nothing before the machine can
start, so a worker that finds one already built in `CARGO_TARGET_TMPDIR` has most of
an hour more for the run than one that does not.
The measurement, the environment's own sentences under Secure Boot against `0.0.2`,
and where the serial line is kept are in
`updates/the-install-under-secure-boot-does-not-fit-a-workers-window.md`.

Split from task 14 on 2026-09-16. Task 14 found why `alo-agentd` failed on the disk
installed under Secure Boot. systemd mounts `/sys/fs/bpf` so that only root can pass
through it, so the person's service could not reach the boundary pinned beneath it.
The image now gives the agent's group passage, and the same release with only that
line added ran `alo-agentd` under Secure Boot. The line is in `image/`, and **not in
release `0.0.1`**, which is what the installer pulls and what the named test
installs. So a machine installed today still boots with `alo-agentd` failed.

Only the owner makes a release worth pinning (ADR 0036): built from a commit that
carries `image/usr/lib/tmpfiles.d/alo.conf`'s passage, pushed to
`ghcr.io/aloworld-org/alo-os`, signed by digest with the owner's key, and pinned in
`image/pinned.toml`. A worker may prepare the candidate (declare `next` and move the
recipe's `org.opencontainers.image.version` in one change, as that file says), and
never pushes, signs or pins.

- **Acceptance:**
  `the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`
  passes **with Secure Boot on**, against the release pinned after this change, as a
  whole: the install, *the first disk is unchanged during the install*, `alo-agentd`
  active and running on the installed disk, and *the first disk is unchanged when
  alo OS booted*, which no run has yet reached. Paste the run into the report. If
  something stops it after `alo-agentd` starts, it goes into `docs/quirks.md` with
  the machine's own account, and this task is split again rather than extended.
- **Constraint:** Secure Boot is never switched off to make the test pass (ADR 0033
  §4). No shim, loader or installer is patched or built by us (ADR 0011). Nothing on
  the installed disk is changed by the test to make a service start, and that
  includes the `tmpfiles.extra` credential task 14 used on a scratch disk to show
  the fix. No worker signs or pins a release (ADR 0036). A worker checks for 15 GB
  free before every run, and removes the run's disks and images when it ends, pass
  or fail.

### 16. Restarting into Windows, and the menu a machine starts at

**Status:** **Done, 2026-09-22.** On the third PC
(`updates/restarting-into-windows-and-the-menu-a-machine-starts-at.md`).
**Nothing in it is ticked on a machine**, which is what this task's own
constraint asks: it is built so it can be walked, and the walk is the
development PC's.

**What landed.** A new crate `crates/alo-starting` — the generated menu, the
loader's environment block read and written, which system starts when nobody
chooses, what the firmware reports and the one thing it is ever told, and every
sentence a person reads with a translator's note on each. `alo_broker::SystemVerb`
gained a twelfth member, `starting.windows-next`, carried out by
`alo_brokerd::NextStart` against `alo_starting::Firmware`; `docs/contracts/agent-verbs.md`
carries it additively and `docs/booting.md` carries the alongside journey with
term 2's honest line and the boot-menu key named. `alo-letting-go`'s count of the
broker's list moved from eleven to twelve, in this change, with the reason beside
it.

**Three decisions a reader should know were made rather than found.**
**(a) The identity of a start-up entry does not include the number the firmware
keeps it under** — an entry names what it starts, not the slot it is in — so a
firmware that renumbers does not turn an approval into a refusal, and the same
entry written twice is one identity and two matches, which is refused rather
than guessed at. **(b) Windows is recognised by the program its entry starts and
never by the entry's name**, because a firmware's names are whatever was typed
when the entries were made and on a reinstalled machine they are regularly
wrong. **(c) The menu's Windows entry is found by searching for Windows's own
loader** rather than by a partition identifier written into the configuration:
an identifier learned once at install and never checked again would be a second
copy of a fact about somebody's disk, of exactly the kind term 3 refuses.

**What the alo OS side owes, and who owes it.** No agent verb was declared:
declaring one requires a promise in `docs/features.md` with a tier for
`alo-by-hand` to answer against (ADR 0009), and adding a promise there is the
owner's rather than a worker's. The broker verb is the road, which is the shape
`updates.apply` and `storage.mount` already have. And whether `GRUB_SAVEDEFAULT`
behaves as this configuration expects across a `bootupd` update is **not
measured** — ADR 0062's consequences already put that answer in
`docs/quirks.md` when task 4 walks it.

Formerly: ready — **taken by the third PC on 2026-09-22 at the owner's
instruction**, to get the certified laptop installed by the evening of
2026-09-23. It is **the part of task 4 that needs no virtual machine**, split
out so that what is this machine's and what stays with the development PC is
plain rather than inferred. **Depends on:**
[ADR 0062](../decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md),
accepted 2026-09-21.

**What is here.** The alo OS side of *alongside Windows*: the way out of alo OS,
the menu a machine starts at, and the journey written down.

- **a. *Restart into Windows*, from inside alo OS.** A setting, and a verb an
  agent may ask for under a grant, with the person's confirmation as any verb
  has. It sets the firmware's **`BootNext`** to Windows Boot Manager for **one
  restart** and leaves the default untouched, so the choice is for that restart
  and nothing about the machine's ordinary behaviour changes.
  `alo_broker::SystemVerb` is a closed enum and adding a member is a deliberate
  act (ADR 0001 §1–2): it gets its name, its words in the vocabulary with a
  translator's note, and **the same tests every other verb has** — approved in a
  turn, one approval causing exactly one execution, recorded permitted or
  refused, and its refusal path tested as carefully as its happy one.
- **b. The start-up menu, as ADR 0062 decided it.** The base's own GRUB,
  **configured and never patched** (ADR 0011), offering alo OS and Windows, the
  latter by chainloading `\EFI\Microsoft\Boot\bootmgfw.efi`; a short countdown;
  and **term 3** — the last choice kept as GRUB's own saved default, in its
  environment block, as **the only copy**. The default is changeable from alo
  OS's settings, which read and write *that* and keep **no second copy**: two
  copies drift, and a menu that preselects one thing while a setting says
  another is the bug the term exists to prevent.
- **c. `docs/booting.md`'s alongside journey.** The steps a person takes, in
  order, with what they see at each — including **term 2's honest line**: a
  loader that *starts* and is then broken is not passed over by the firmware,
  because to the firmware it started, so the machine's boot-menu key is named as
  the way to Windows in that case, and nothing claims the fall-through reaches
  it.
- **And the test that holds whichever way Fast Startup is answered:** alo OS
  **never mounts the Windows partition read-write**. Fast Startup was decided
  on 2026-09-22, after this task landed — ADR 0064 term 9, *the installer
  asks* — and the asking is task 4's, on the Windows side. This test holds for
  either answer.

**What is not here, and stays with the development PC.** `crates/alo-installer`
and `crates/alo-installing` are **not edited by this task** — that machine is
working in both, on the Windows-side program and on a *Creating rootfs* failure,
and two lanes in one crate is the collision the lane table exists to prevent. If
this part needs a change there, it is **written down and passed across**, never
made here. Also that machine's, because each needs a virtual machine: **term
1's** firmware-order fall-through test, the install beside a real Windows, the
*Restart into alo OS* side from within Windows, the Windows-unchanged hash test,
*remove alo OS*, and the full Windows → alo OS → Windows → alo OS walk through
the in-system switches alone. **This part is built so it can be walked there.**

- **Acceptance:** the verb exists with its name, words and the full set of tests
  every `SystemVerb` has, and sets `BootNext` for one restart with the default
  provably untouched; GRUB's configuration is generated rather than patched,
  offers both systems, chainloads Windows by the path above, counts down, and
  saves the last choice in its own environment block; alo OS's setting reads and
  writes that one place, held by a test that **finds no second copy** anywhere;
  `docs/booting.md` carries the journey with term 2's line in it; and alo OS
  never mounts the Windows partition read-write, held by its own test.
- **Constraint:** nothing here is ticked *on the machine* — none of this has run
  on the certified laptop, and the walk that proves it belongs to the virtual
  machine on the development PC. GRUB is configured and never patched. No change
  to `crates/alo-installer` or `crates/alo-installing`. Fast Startup was not
  decided here; the owner decided it on 2026-09-22 (ADR 0064 term 9) and the
  asking is task 4's.

### 17. The default a machine starts at, changed by the person who owns it

**Status:** **Done, 2026-09-23.** On the third PC
(`updates/the-default-a-machine-starts-at-changed-by-the-person-who-owns-it.md`).
**Nothing in it is ticked on a machine**, which is what this task's own
constraint asks: no hardware, no certified laptop, and no acceptance claimed
from a machine.

What *was* run on a machine, and is written down rather than ticked, is the
base's own half. ADR 0066 term 1 puts the last choice on the EFI system
partition, and where that file lives on the pinned base was measured instead of
assumed: the base was installed by its own `bootc install to-disk`, read, and
booted under OVMF with no KVM. It keeps `grubenv` as a plain file on `/boot`
rather than as the symlink onto the ESP older Fedora layouts have, carries no
block on the ESP at all, and **mounts the ESP nowhere**. The loader's ability to
save on that partition's FAT was proved rather than believed, and the generated
drop-in was booted verbatim. `docs/quirks.md` has the measurement, the versions
and the date. What it costs — the ESP mounted at `/boot/efi`, and the block
created at install time, neither of which the base does — is written above as
owed by the lane that owns the installer.

**The decision a. asked for was already made, and it is not this task's.**
[ADR 0066](../decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md),
accepted 2026-09-23, weighs the alternatives this task lists and decides: one
copy of the answer, on the partition both systems can read, and **a thirteenth
member of `alo_broker::SystemVerb`** whose argument is the identity of a system
the menu already offers. So this task is b. and c. — the road and the surface —
built on that decision, with `alo-letting-go`'s count moved from twelve to
thirteen in this change and the reason beside it.

**What landed.** `starting.default`, the broker's thirteenth verb, carried out
by `alo_brokerd::ByDefault` against a new `alo_starting::TheLoader` — three
methods over bytes, so that nothing on the privileged side knows the shape of
the loader's files. `alo_starting::TheLoadersFiles` is that trait over
`/boot/grub2/`, writing the environment block **in place**: opened for writing,
never truncated, never created, never written to a new file and renamed over,
because the loader saves into that file from inside the loader and a file it
can no longer reach is the last choice silently stopping being kept.
`to_start_by_default(System)` is the person's half in Settings, and
`starts_at_said` reads what is in the file at the moment it is shown. Five new
sentences in the vocabulary, each with a translator's note.

**The verb is not this plan's crate, and the change carries the records that
say who released it.** `v0-5-the-broker-and-the-disk-plan.md` releases the
broker's two files and the two carriers' lines to this task, and
`v0-5-the-machine-keeps-itself-plan.md` releases the count test — both as
owner-release blocks in their own headers, both naming ADR 0066 as the owner
decision they record, and neither transferring a crate. The first handover of
this task was refused for want of them: they existed in the working tree and
were not among the files the handoff named, and the check reads what a task
publishes. A release block is part of the change that uses it.

**Three decisions a reader should know were made rather than found.**
**(a) The identity is over what the *menu* offers, not over what the firmware
reports** — it is the same on every machine, deliberately, because what is being
named is *which of the two systems* and not *which entry on this computer*;
whether this machine has that system at all is asked of the machine's own menu
where the change is made. **(b) Whether the machine offers Windows is read off
the generated menu file**, not from a setting beside it: a record of *there is a
Windows here* kept anywhere else would be a second copy of a fact about
somebody's disk, of exactly the kind term 3 refuses. **(c) The change is carried
out even when the file already says so.** An execution that quietly did nothing
would make what the record shows depend on a state nobody can see.

**What it does not do.** The asking road — a surface issuing a token at the
broker's door under `alo_broker::BY_HAND` — is **not** built here, which is the
shape `updates.apply`, `storage.mount` and `starting.windows-next` already have:
those crates produce the verb and the sentences, and something else asks. And no
agent verb was declared — **but not for task 16's reason any more.** The promise
it was waiting on now exists: the change that accepted ADR 0066 added a `[v0.5]`
line to `docs/features.md`, and ADR 0066 §2 says an agent may ask for this under
a grant. What stops it here is ownership — `alo-capability`, `alo-agentd` and
`alo-by-hand` are named on this plan's own first page as lane A's — so it is
**written down and passed across**: everything the agent verb needs on this side
is in place, and declaring it is a small change in lane A's crates.

**What is here.** The half of ADR 0062's third term that task 16 could not
finish: **the road a person's choice travels to reach the loader's own file.**
Task 16 built the reader and the writer over GRUB's environment block
(`alo_starting::TheStartingChoice`), and a test that nothing anywhere in the
crates or the image keeps a second copy of the answer. What it did not build is
how a person in Settings, who is not root, changes a file under `/boot` that is.

- **a. The decision, if one is needed — and it probably is.** The obvious road
  is a thirteenth member of `alo_broker::SystemVerb`, and **adding a member of
  that enum is a deliberate act** (ADR 0001 §1–2): `alo-letting-go`'s count of
  the list is the tripwire, and moving it takes a decision named beside it. The
  alternatives are real and should be weighed rather than skipped: a verb whose
  argument is a `Switch` naming which of the two systems; the person's own act
  going through the broker under `alo_broker::BY_HAND` with no agent verb at
  all; or the loader's saved default being writable only by the loader, with
  Settings offering nothing but *choose at the menu*. **If the answer is a new
  member, write the ADR first**, with the options, a recommendation and the
  consequences, and hand that over as this task — that is a finished piece of
  work, and it is what the next worker needs.
- **b. The road itself, whatever a. decides**, with the full set of tests every
  change to the machine has: approved once, one approval causing exactly one
  execution, recorded permitted and refused, and each refusal path tested as
  carefully as the happy one. The refusals are already known and each is a
  sentence task 16 wrote or owes: a file that is not an environment block, one
  that will not hold another setting, one that could not be written, and a
  machine with no Windows on it to start.
- **c. The surface in Settings.** *This computer starts alo OS / Windows when
  nobody chooses*, from `alo_starting::starts_at_said`, with the change beside
  it. It reads what is in the loader's file at the moment it is shown — never a
  value kept anywhere else, which is the whole of term 3 — and it says what a
  person reads when the change was refused.

**What is not here.** `crates/alo-installer` and `crates/alo-installing` are not
edited: **who writes `custom.cfg` and the environment block onto a machine at
install time, and what happens to them across a `bootupd` update**, is task 4's
and belongs with the machine that has the virtual machine to walk it.

**Two things the installer is owed, measured here and written down rather than
built.** Both come out of the base itself, on the pinned digest, installed and
booted — `docs/quirks.md` carries the measurement.

1. **The EFI system partition has to be mounted at `/boot/efi`.** The base
   mounts it nowhere: no `/etc/fstab` at all, no `/efi`, `/boot/efi` an empty
   directory, and no vfat mounted anywhere on a booted machine. Until something
   mounts it, the block is missing and this road refuses — correctly, and with a
   sentence a person can read, but it refuses.
2. **The block has to be created at install time**, with `grub2-editenv create`
   or the same 1024 bytes by another name. GRUB's `save_env` writes a block *in
   place* and cannot make one, and neither does anything here: a file that is
   not there is not created, which is this crate's rule and its test. A machine
   whose ESP has no block has nothing for either system to write. Nor is the
*Restart into Windows* verb, which task 16 built and which is a different act: it
sets the next start, and this one sets the default. Nothing here is ticked on a
machine.

- **Acceptance:** a person's change to which system the machine starts at
  reaches the loader's one environment block on the EFI system partition
  (`/boot/efi/EFI/fedora/grubenv`) and nothing else, under one approval, recorded
  either way; the test that finds no second copy of the answer still passes and
  now covers the new road; every refusal on the road is a sentence in the
  vocabulary with a translator's note; and if a member was added to
  `alo_broker::SystemVerb`, the ADR that decided it is in `docs/decisions/` and
  `alo-letting-go`'s count moved in the same change with the reason beside it.
- **Constraint:** GRUB is configured and never patched (ADR 0011). **No second
  copy of the last choice** — not a file of ours, not a cache, not a value
  carried in a session (ADR 0062 term 3). No change to `crates/alo-installer` or
  `crates/alo-installing`. No verb that writes the machine's start-up **order**,
  adds a start-up entry or removes one. Nothing is ticked on the certified
  laptop.

### 18. `bootc` stops at *Creating rootfs* when the installer's own restart reaches the environment

**Status:** **Done, 2026-09-22.** On the development PC
(`updates/the-install-finishes-on-the-installers-own-road.md`). **The cause:**
the environment's initramfs had no `mkfs.btrfs`. The base has it at
`/usr/sbin/mkfs.btrfs`, and bootc starts it by name for `--filesystem btrfs`,
which task 11 asked for. Task 11 had measured btrfs by running `bootc install`
from the release image, not through the environment. `alo-installing.conf` now
carries it, and a test holds the list to `mkfs.` followed by
`alo_image::THE_ONLY_FILESYSTEM`. That was the only change. **Then, on the
installer's own road** (Windows 11 in QEMU/KVM, Fedora's firmware, Secure Boot
off because the Windows side refuses it on, no boot order given):
the install finished (*alo OS is installed*), the firmware started the
installed system on its own (`Boot000B "Fedora"`), and the installed system
said its root was `/dev/sdb3 btrfs` on the disk with serial `ALOTARGET1`. The
test that shows it,
`the_whole_road_installs_alo_os_and_the_installed_system_starts`, passed by
name in 1 815 s. **Found on the way:**
- One run's download stalled for over 30 minutes, with no disk or network
  traffic, and the environment said *Still installing* for ever. That is task 20.
- bootupd's entry is named *Fedora* and is put first in `BootOrder`, and the
  installer's own *alo OS* entry is left behind, last. That belongs to task 4.
**Depends on:** nothing.
**Found by** task 10's walk, 2026-09-21, and not fixed there on purpose: the
owner's laptop takes the same road, so it gets a task of its own.

On the installer's own road — a Windows 11 in a QEMU/KVM machine, the installer
run to the end, and the computer restarted by the installer — the firmware
started the `alo OS` entry from the installer's area, shim fell back to GRUB,
the environment's kernel booted with
`alo.installing.to=ata-QEMU_HARDDISK_ALOTARGET1`, and the environment said, in
order (the run's serial line, `/root/t10/logs/road-fedora-kept.log` lines
779–804 on the development PC):

```
alo OS is being installed on this computer. Each step is written here as it happens
Reading which disk you chose before the restart
Looking for the disk you chose: ata-QEMU_HARDDISK_ALOTARGET1
Checking that ata-QEMU_HARDDISK_ALOTARGET1 is safe to install onto
Connecting to the internet
Checking over the internet that this download is a genuine alo OS
This is a genuine alo OS
Installing alo OS onto ata-QEMU_HARDDISK_ALOTARGET1. Everything that was on that disk is being replaced. This takes a while, and this screen will say when it is done
[    6.850278]  sdb: sdb1 sdb2 sdb3
/usr/bin/bootc: error: Installing to disk: Creating rootfs: No such file or directory (os error 2)
alo OS could not be installed onto ata-QEMU_HARDDISK_ALOTARGET1. That disk may now hold part of alo OS; nothing else on this computer was changed. Restart to try again
```

So the image is found, verified, and the disk partitioned (`sdb1 sdb2 sdb3`),
and `bootc install to-disk` then fails creating the root filesystem, 3–4 s
after partitioning — **before** *Deploying container image*, which task 9's
run reached.

**What differs from task 15's run that finished**, none of it yet separated
and each a candidate: the release (0.0.5 pinned now, 0.0.4 then); the
filesystem (`--filesystem btrfs` since task 11; see `docs/quirks.md`, *`bootc
install --filesystem btrfs` makes no subvolume of its own*); the disk (SATA
`sdb` here, virtio there); the firmware (Fedora's `edk2-ovmf` 20250812-21 in
both). **Not the road in**, measured on 2026-09-22: with the installer's entry
written without Windows' optional data, shim started GRUB directly with no
fallback, and `bootc` stopped at exactly the same line.

**To reproduce:** on a machine with KVM and 25 GB free, check out `main`, then
`cargo test -p alo-installer --test the_installer_walked_on_a_real_windows --
--ignored --test-threads=1 a_windows_installs_itself_and_reaches_a_desktop_session
the_whole_road_starts_the_environment_on_the_next_restart`, and read the serial
line the second test prints. Faster, without Windows: task 9's
`crates/alo-installing/tests/installed_in_a_virtual_machine.rs`
`the_environment_installs_onto_the_second_disk_and_it_boots_to_the_agent_service`
on today's pin, and — to separate the disk — the same test with its second disk
given as SATA with a serial rather than virtio.

- **Acceptance:** the cause is found by changing one of the candidates above at
  a time and written into `docs/quirks.md` with the run's own output; the fix
  lands with the test that shows it; and the installer's own road — task 10's
  road test — then runs past *Creating rootfs* to *Deploying container image*
  and on. Whether it then finishes and boots is task 12's.
- **Constraint:** as task 12's. Secure Boot is never switched off to make a run
  pass; no shim, loader or `bootc` is patched (ADR 0011); a worker checks for
  15 GB free before every run and removes its disks when it ends.

### 19. Killed at every step, Windows' own partition byte for byte — or the reason it is not

**Status:** scheduled — **on the development PC**, recorded 2026-09-22 at the
owner's word. It needs the Windows guest, and the third PC has no working
`/dev/kvm`: every guest there is emulated, and one virtual-machine acceptance on
it has already cost 3 779 seconds. The work is ready; the machine that can run
it is what it waits for.

*scheduled* rather than *ready* is deliberate and is the word the supervisor
reads: a lane skips a task only on `blocked` or `scheduled` in this line, so a
sentence naming the machine — however plain — would be read by a person and
stepped over by the program, and the next lane to start on this plan would take
this task merely because it is next. **Depends on:** 10.
**Split from** task 10 on 2026-09-22. Task 10 is done for what it proved: all
seven kills landing exactly, Windows restarting to its desktop after each, the
start partition unchanged beyond the controls, nothing unreadable, and the
install and road tests passing. The claim this task owns is the one task 10's
run could not make: **the Windows partition is unchanged, byte for byte,
beyond what the controls change.** `killed_at_every_step_the_computer_still_starts_windows`
no longer asserts it; it prints what it finds for this task.

**What the run of 2026-09-22 found** (the fourth start of the Rust walk, head
`cdee38b6`, 17 261 s, no host sleep; the controls changed 5 331 paths after one
start and 5 424 after two and disagreed in 721 directories; nothing unread).
After every kill and every restart that followed — 14 findings — the Windows
partition changed beyond the controls:

| after | kill | restart |
|---|---|---|
| step 1 | 7 paths | 50 |
| step 2 | 21 | 46 |
| step 3 | 19 | 41 |
| step 4 | 51 | 60 |
| step 5 | 60 | 60 |
| step 6 | 60 | 60 |
| step 7 | 20 | 41 |

(60 is where that run's test stopped listing, so those findings may hold more.)
Every distinct path, with the number of the 14 findings that name it
(random 8-character directories written `<8>`; files under one cache folded):

| path | findings |
|---|---|
| `ProgramData/Microsoft/Windows Defender/Scans/History/ReportLatency/Latency/…` | 8 |
| `ProgramData/Microsoft/Windows Defender/Scans/History/Results/Resource/{…}` | 8 |
| `ProgramData/Microsoft/Windows/SystemData/<SID>/ReadOnly/LockScreen_O/…` | 1 |
| `ProgramData/Packages/Microsoft.WindowsTerminal_8wekyb3d8bbwe/<SID>/SystemAppData/Helium/Cache/…` | 7 |
| `Windows/System32/wbem/Performance/WmiApRpl_new.ini` | 2 |
| `Users/alo/AppData/Local/Microsoft/OneDrive/StandaloneUpdater/*.json` | 2 |
| `Users/alo/AppData/Local/Microsoft/Windows/ActionCenterCache/microsoft-skydrive-desktop_3_0.png` | 4 |
| `Users/alo/AppData/Local/Microsoft/Windows/INetCache/IE/<8>/08b7573ae3ef7b6b30f35fd702bdfa9bf754ff1f[1].xml` | 14 |
| `Users/alo/…/Microsoft.Windows.ContentDeliveryManager_cw5n1h2txyewy/LocalState/TargetedContentCache/v3/8800016{1,3,5}/…` | 4 |
| `Users/alo/…/Microsoft.Windows.ShellExperienceHost_cw5n1h2txyewy/Settings/{roaming.lock,settings.dat}` | 10 |
| `Users/alo/…/Microsoft.WindowsTerminal_8wekyb3d8bbwe/LocalState/{settings,state}.json` | 7 |
| `Users/alo/…/Microsoft.WindowsTerminal_8wekyb3d8bbwe/SystemAppData/Helium/{User,UserClasses}.dat*` | 7 |
| `Users/alo/…/MicrosoftWindows.Client.CBS_cw5n1h2txyewy/AC/INetCache/<8>/th[1].svg` | 14 |
| `Users/alo/…/MicrosoftWindows.Client.CBS_cw5n1h2txyewy/AC/Temp/edge_BITS_*/…` | 3 |
| `Users/alo/…/MicrosoftWindows.Client.CBS_cw5n1h2txyewy/LocalState/EBWebView/Default/Service Worker/CacheStorage/…` | 14 |
| `Users/alo/…/MicrosoftWindows.Client.CBS_cw5n1h2txyewy/LocalState/EBWebView/Speech Recognition/1.15.0.1/…` | 3 |
| `Users/alo/…/MicrosoftWindows.Client.CBS_cw5n1h2txyewy/LocalState/EBWebView/ZxcvbnData/3.2.0.0/…` | 6 |

**Not only the user's profile.** Defender's scan history and a lock screen
image are under `ProgramData`, and `WmiApRpl_new.ini` is under `Windows\`.
They are not waved away as caches: a file under `Windows\` that changes after a
kill and not after a control is exactly what this claim is about, until a
control shows otherwise.

**The census** (the shell harness's 21 readings of 2026-09-21/22 — base, two
plain boots, two refusals, four kills at the consent, twelve step readings):
every kind above is also in the base or the controls. `settings.dat`,
`state.json`, `ZxcvbnData`, `TargetedContentCache` and
`OneDrive/…/ECSConfig.json` are in all 21; the IE-cache `.xml` is in all six
refusal and kill-at-consent readings, each time under a *different* random
directory; Defender's latency history is in every reading in which the
installer ran and in neither plain control; `WmiApRpl_new.ini` is in both
refusal readings and after steps 5 and 7, and not in the kill-at-consent
controls. So these are files Windows itself writes over time. What no control
yet shows is that *these* changes are Windows' schedule and not the kills.

**Two hypotheses, neither proved.**
1. **Random directory names.** The IE cache and the `AC/INetCache` directory
   are named with random 8 characters, like the TPM key hash that made the
   `<id>` rule necessary; the rule does not cover them, so the same file under
   a new directory reads as a new path.
2. **The guest is online, and the controls ran hours before the later steps.**
   The walk's guest has QEMU's user-mode network. Edge WebView's components,
   the content delivery cache, OneDrive's updater and BITS downloads
   (`edge_BITS_*`) arrive from Microsoft's servers when Windows chooses, and in
   the fourth start the controls ran around 08:10–09:00 and step 7 around
   12:20.

**The run that decides it.** Change one thing at a time, each a run of about
five hours on the development PC:
- **The guest offline** — the machine started with no network device
  (`walking::machine`), since the installer and the walk need none; if the
  findings under the user's profile vanish, hypothesis 2 held for them.
- **A control beside each step** — a plain boot and a kill-at-the-consent boot
  of a fresh overlay immediately before each step's kill, the step held to its
  own neighbours' controls; if the remaining findings vanish, they were
  Windows' schedule.
- Only then, if a random directory is still all that differs, **a measured
  rule**: the random-name shape added to `reading::directory_of`'s
  identifiers, with the run that shows it — never a path listed by hand.

**Also carried from task 10**, unmeasured there: the NVMe and Hyper-V SCSI
names `naming.rs` makes, seen from the Linux side (an NVMe device that reports
an identifier, and a Hyper-V machine); and the installer's own MSVC release
build walked the same way (the walk cross-builds `x86_64-pc-windows-gnu`).

- **Acceptance:** `killed_at_every_step_the_computer_still_starts_windows`
  asserts the Windows partition beyond the controls again, and passes, run by
  name end to end with its run pasted; or, if a change survives both runs, it
  is named with its evidence in `docs/quirks.md` and `staging.rs` is fixed or
  the claim is narrowed in the product's own words, in the same change.
- **Constraint:** the ignore rule is never widened by hand; nothing the
  release installer obeys exists only for the test; nothing runs against the
  host's own disks; a worker keeps 15 GB free on the host.

### 20. A download that stops arriving ends the install in words, rather than *Still installing* for ever

**Status:** ready. **Depends on:** 18.
**Found by** task 18's second run on 2026-09-22
(`updates/the-install-finishes-on-the-installers-own-road.md`). On the
installer's own road, with the environment carrying `mkfs.btrfs`, `bootc
install` went quiet after about 20 minutes. It did not end. The environment said
*Still installing alo OS. Leave the computer on* 65 times, and the screen showed
`Job alo-installing.service/start running (54min … / no limit)`. For the last 30
minutes of that the machine did no disk I/O: QEMU's `read_bytes` and
`write_bytes` moved by a few kilobytes, and the target disk's image stayed at
6 762 MB. The host received nothing for it either. QEMU's own table of the
guest's connections showed three TCP connections `ESTABLISHED` with empty
queues, one of them to `185.199.110.154:443`. The host itself reached `ghcr.io`
in 0.19 s. The first and third runs of the same test, on the same PC the same
afternoon, finished in about 12 minutes and 8 minutes of *Still installing*.
Kept on the development PC: `/root/t10/logs/installed-second/`.

A person's laptop would show *Still installing* until the battery ran out. The
environment has no point after which it says anything else.

**What is here.**
- Find where the pull waits with no deadline: `skopeo` or `podman`'s copy
  under bootc, or the TCP connection itself.
- Decide, as configuration of the engines and never a patch (ADR 0011), what
  bounds it. Candidates: a registry or containers configuration timeout, a
  `TimeoutStartSec=` on `alo-installing.service`, or the environment's own
  watch on progress.
- Make the environment say a sentence of its own when the bound is reached,
  with a translator's note, and end as the refusals already end: nothing else
  on the computer changed, and restart to try again.

- **Acceptance:** a test holds the bound. It is shown with a download stopped
  on purpose in a virtual machine, and the environment says the new sentence
  within the bound, not *Still installing* after it. The road test from task 18
  still passes.
- **Constraint:** no engine is patched (ADR 0011). A bound short enough to cut
  off a slow but moving download is a bug too, so the bound is on *no
  progress*, not on total time, unless measurement shows the two cannot be
  told apart.
