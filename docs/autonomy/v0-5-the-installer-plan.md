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

**Status:** scheduled — **and no longer scheduled on hardware.** It installs
beside a real Windows in a virtual machine and walks the switching both ways,
which is the largest disk of the three. **Depends on:** 3, 8, 9, 10.

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

**Status:** blocked — on tasks 1–5, 8, 9, 10 **and 11**, and on the owner at the
laptop; nothing in this repository can tick it. Task 11 is not optional before
this one: a filesystem is chosen at install and cannot be converted, so a laptop
installed before it lands could never undo what an agent did without being
reinstalled ([ADR 0045](../decisions/0045-what-undoing-rewinds-to.md), accepted
2026-09-16). **Depends on:** 4, 5.

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

**Status:** scheduled — **and no longer scheduled on hardware, as of
2026-09-20.** **Depends on:** 3.

> **One of the two conditions this task waited on was measured away on the
> development PC (Intel Core Ultra 7 155U), 2026-09-20. The other was not, and
> the disk paragraph below stands.**
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

**Status:** ready. **Depends on:** 2, 15 — its acceptance boots a virtual-machine
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
