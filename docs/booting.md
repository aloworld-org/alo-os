# Turning the image into a disk, and watching it boot

`image/Containerfile` builds alo OS as a bootable container
([ADR 0011](decisions/0011-the-base-is-rented-and-the-image-is-a-container.md)).
That is an image, and an image is not a disk. Until this document existed,
nothing in this repository turned one into the other, so the only thing anybody
had ever seen was a test saying the recipe was right.

This is the missing step: one command that writes the image onto a disk file,
and what to point at that file afterwards.

The disk, in four facts:

    tool: bootc install to-disk
    firmware: uefi
    generation: 2
    disk: alo-os.raw

Those four are not prose. `crates/alo-image` reads them back out of this
document and holds them against the `alo.disk.*` labels in
`image/Containerfile`, because the way this document goes wrong is not that
somebody deletes it — it is that the image changes and the sentence telling
somebody which firmware to select stays where it was.

## What this produces

`alo-os.raw`, a whole disk: a partition table, an EFI system partition with the
bootloader on it, and the root filesystem this image's files are in. It is
written by the `bootc` **inside the base image pinned in
`image/Containerfile`** — so the version of the tool is that base's digest, and
there is no second thing to pin and nothing here that lays out a partition
table itself.

It is the same image, not a second recipe: `bootc install to-disk` installs the
container it is running as, so what lands on the disk is exactly what
`image/Containerfile` produced, or the command fails.

## What you need installed

- **Podman**, on a Linux machine or in WSL2. Upstream's install path runs the
  bootable container as a privileged container and reads the image out of
  `/var/lib/containers`, which is podman's store. Docker can build the image;
  it is podman that writes the disk.
- **`qemu-img`**, from `qemu-utils`. Hyper-V attaches VHDX files and not raw
  ones, and converting between disk formats is an upstream tool's job.
- A Linux kernel with loop devices, which WSL2 has. The install writes through
  `/dev/loop*` rather than to a physical disk.

## Building the image, then the disk

Build the image first, from the root of the repository:

    podman build -f image/Containerfile -t alo-os:dev .

Make the file the disk goes into, and then write it:

    truncate -s 20G alo-os.raw

    podman run --rm --privileged --pid=host \
      --security-opt label=type:unconfined_t \
      -v /var/lib/containers:/var/lib/containers \
      -v .:/output \
      localhost/alo-os:dev \
      bootc install to-disk --via-loopback --wipe \
        --filesystem btrfs /output/alo-os.raw

**The `truncate` is not optional**, and this document said nothing about it
until somebody ran the command: `--via-loopback` attaches a loop device to a
file that is already there and does **not** create one, so without it the whole
thing stops on its first line with `Querying /output/alo-os.raw: No such file
or directory`. Twenty gigabytes is the size the disk is laid out for; the file
is sparse, so what it costs on the host is what the install actually writes,
which measured 2.0 GB on 2026-09-11.

`--via-loopback` is what makes a *file* an acceptable target; without it the
tool expects a block device, which on a workstation means somebody's disk.
`--wipe` is there so that running it twice is running it twice rather than
writing into whatever was left.

**`--filesystem btrfs` is the argument that cannot be taken back.**
[ADR 0045](decisions/0045-what-undoing-rewinds-to.md) rewinds *undo what the
agent did* from the base's own read-only snapshot of a person's home, and a
snapshot needs a filesystem that has them: of the three the pinned `bootc`
accepts — `xfs`, `ext4`, `btrfs` — only `btrfs` does. **A filesystem is chosen
at install and cannot be converted afterwards**, so a disk written here with
anything else is a machine that can never undo anything without being
reinstalled. The value is `alo_image::THE_ONLY_FILESYSTEM` in the code, this
document is held to the same one, and what the base actually makes of it — which
subvolumes, where a home lands, what a snapshot needs — is measured in
`docs/quirks.md` rather than assumed.

And for Hyper-V, one conversion:

    qemu-img convert -f raw -O vhdx -o subformat=dynamic \
      alo-os.raw alo-os.vhdx

## Installing the published image

Everything above builds the image on the machine that writes the disk. A
machine that did not build it — the installer's boot environment, or anybody
following this page — installs the **published** release instead, from the
registry updates will come from
([ADR 0023](decisions/0023-installed-from-the-machine-it-replaces.md),
[ADR 0033](decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
§3), and it installs it by digest:

    registry: ghcr.io/aloworld-org/alo-os
    tag: 0.0.5
    digest: sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1

Those three are `image/pinned.toml`, the one file the digest is written in, and
`crates/alo-image` holds this document to it the way it holds the four disk
facts to the recipe. The tag is how a person reads the release; the digest is
what is pulled, because a tag is a name somebody can move after the owner
signed and a digest is the bytes themselves.

**Verify first, and write nothing if it fails.** Release `0.0.5` was signed by
the owner, by digest, with the key whose public half is committed at
`image/signing/alo-os.pub`
([ADR 0036](decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md)),
and without an upload to a public transparency log — so verification needs this
repository and the registry, and nothing else. From the root of a checkout, with
`cosign` installed:

    cosign verify --key image/signing/alo-os.pub --insecure-ignore-tlog=true \
      ghcr.io/aloworld-org/alo-os@sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1

`--insecure-ignore-tlog=true` is the flag for *there is no log entry to check*,
which is true of every alo OS signature by decision rather than by accident; the
key is still checked. On 2026-09-15 a different key was refused (*Found: 0, Expected 1*), and
anything but a pass here is the end of the procedure.

Then pull that digest into podman's store and install it, the same way as the
local build above — the image in the `podman run` is the only thing that
changed:

    podman pull ghcr.io/aloworld-org/alo-os@sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1

    truncate -s 20G alo-os.raw

    podman run --rm --privileged --pid=host \
      --security-opt label=type:unconfined_t \
      -v /var/lib/containers:/var/lib/containers \
      -v .:/output \
      ghcr.io/aloworld-org/alo-os@sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1 \
      bootc install to-disk --via-loopback --wipe \
        --filesystem btrfs /output/alo-os.raw

It writes a file, not a disk. The installer's boot environment will run this
same invocation against the one disk a person named, and nothing in this
repository points it at a real one.

**Whether a pull needs a login is not yet shown here.** This page said until
2026-09-15 that the package was private, so `podman login ghcr.io` (and
`cosign login ghcr.io`) came first. The boot environment below holds no account
of any kind, so an installer can only work once the package is public; that the
environment has pulled and installed the release is **not** yet measured (see
`docs/autonomy/updates/the-boot-environment-that-installs.md`).

## The boot environment that installs

Everything above is run by a person at a Linux machine. The installer a person
downloads cannot ask that of anybody: it restarts the machine into a small
environment that does the same thing on its own, and says on the screen what it
is doing ([ADR 0023](decisions/0023-installed-from-the-machine-it-replaces.md)
§2–3). That environment is built from `image/installing/Containerfile`, and the
program inside it is `crates/alo-installing`.

It is a kernel and an initramfs, both made from the image's own pinned base by
the base's own tools, and the base's signed shim and loader beside them — so a
machine with Secure Boot on starts it without anybody changing a setting
([ADR 0033](decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
§4). It is not a second operating system: one program runs, nobody is offered a
login or a shell, and it ships only what installing needs.

### Building it

From the root of the repository:

    podman build -f image/installing/Containerfile \
      --output type=local,dest=alo-installing .

What lands in `alo-installing/` is the content of a FAT partition, and nothing
else:

    EFI/BOOT/BOOTX64.EFI              the signed shim
    EFI/BOOT/grubx64.efi              the signed loader
    EFI/BOOT/mmx64.efi                the shim's key manager
    EFI/BOOT/grub.cfg                 the one entry
    EFI/alo-installing/vmlinuz        the signed kernel
    EFI/alo-installing/initramfs.img  the environment

### What the program that stages it must do

The installer plan's task 3 writes these files; this is what the environment
needs of it, and the environment refuses rather than guesses when it is not
done:

1. **A FAT partition labelled `ALO-INSTALL`** holding the files above. The label
   is how the environment recognises the disk it is running from, and it refuses
   to install over that disk.
2. **The chosen disk, in `EFI/BOOT/chosen.cfg`**, one line:
   `set alo_installing_to=` followed by the disk's own name as Linux lists it
   under `/dev/disk/by-id/` — never `sda`, which names whichever disk answered
   first. With no such file the environment says *no disk was chosen* and
   writes nothing.
3. **A boot entry for `\EFI\BOOT\BOOTX64.EFI` on that partition, reached once**,
   through the firmware's next-boot choice rather than by changing the default.
   The environment restarts the machine when it has installed, and a restart
   that landed back in it would install again.
4. **A wired connection.** The environment brings up a network cable and
   nothing else; a machine with only Wi-Fi reaches *alo OS could not be
   downloaded* and nothing is changed. Carrying a wireless network across the
   restart is the staging program's problem to solve, and is not solved yet.

### The program that stages it: `alo-installer`, on Windows

`crates/alo-installer` is that program — what a person downloads and runs on the
Windows the computer came with. It runs Windows' own tools and nothing else
(the storage cmdlets of Windows PowerShell, `bcdedit`, `whoami`, `shutdown`),
each an enumerated program with typed arguments, and never writes to a disk
directly. In order, saying each step before it begins:

1. **It asks for an administrator's rights** before anything else, and without
   them says so and stops.
2. **It holds the environment beside it to the release.** The download is the
   program and, beside it, the `alo-installing/` directory the recipe above
   builds, with one more file in it: `alo-installing.sha256`, the SHA-256 of
   every file, as `sha256sum` writes them. The SHA-256 of *that list* is compiled
   into the program by the release that builds it (`ALO_INSTALLER_ENVIRONMENT_SHA256`,
   the installer plan's task 5), and each file is read once, compared, and those
   same bytes are what is written. A program built without the variable — any
   build that is not a release — says *this download is not a genuine alo OS, so
   nothing was changed*.
3. **It checks the computer, with reads alone, and says everything it found:**
   UEFI or BIOS, Secure Boot, TPM, BitLocker on the Windows volume, its free
   space, memory, and every disk. A question Windows did not answer is said as
   *could not be found out* and is never read as *off*.
4. **It refuses**, each time with *so nothing was changed*: a BIOS computer;
   **Secure Boot on, or not known** — said with the reason and no suggestion
   ([ADR 0033](decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
   §4); disks it could not read; a Windows disk that is not GPT; an entry named
   alo OS or an area labelled `ALO-INSTALL` that an earlier start left; BitLocker
   part way through encrypting or decrypting; less than 17 GB free on the Windows
   volume (the 1 GB area and 16 GB Windows keeps); and **no empty disk of at
   least 24 GB beside the one Windows is on** — the environment replaces one
   whole disk, and putting alo OS on the Windows disk itself is the installer
   plan's task 4.
5. **It says exactly what will happen** — Windows made 1 GB smaller, a 1 GB area
   made in that space, an entry named alo OS added, one restart into the
   installer, which replaces the disk named — and that nothing has changed yet.
6. **It takes a typed consent: the name of the disk alo OS replaces**, as it was
   shown. Nothing typed stops; anything but exactly an offered disk's name is
   refused.
7. **It prepares the computer:** shrinks the Windows partition by exactly the
   area; makes the area where that freed; formats it FAT32 labelled `ALO-INSTALL`
   after checking it is still where it was made; writes the environment and
   `EFI/BOOT/chosen.cfg` and reads every file back; copies `{bootmgr}` into an
   entry named alo OS pointing at `\EFI\BOOT\BOOTX64.EFI` on the area, listed
   last; takes the area's letter away; and, last, makes the entry the firmware's
   next start (`bcdedit /set {fwbootmgr} bootsequence`), which never changes the
   default. **A step that fails puts back every change before it**, newest
   first, and says so; if putting back fails too, it says exactly what remains
   rather than that nothing was changed.
8. **It restarts.**

**The chosen disk's name** is made from what Windows reports: `nvme-eui.` and the
disk's identifier for an NVMe disk (never its model — `quirks.md` says why),
`ata-<model>_<serial>` for SATA, and `wwn-0x<identifier>` for SAS and SCSI,
which is what a Hyper-V generation 2 machine's disks are. Any other bus, USB
above all, is not offered. Every name carries an identifier, so a wrong one
names no disk and the environment refuses it as not connected.

**What has been measured, 2026-09-15:** every decision against a scripted
Windows (`crates/alo-installer/tests/the_installer_checks_consents_and_stages.rs`),
and the checks — reads only — against the development machine's own Windows 11
(`tests/reading_this_windows.rs`). **Not yet:** the installer run on a Windows in
a virtual machine and walked to each step, killed there, and Windows shown still
starting; that is the installer plan's task 10.

### What the environment does, in order

It says each step on every console before it begins:

1. reads which disk was chosen from its kernel command line;
2. waits for that disk to appear, by its own name;
3. refuses a disk that is part of a disk, is not connected, holds this
   installer, holds partitions Windows makes, or is in use;
4. waits for the network, then checks that the pinned release is signed by the
   key in this repository — the same check as `cosign verify` above, with the
   same key and the same digest, and the checker's answer must name that
   digest;
5. writes the chosen disk with `bootc install to-disk`, pulling the release by
   digest straight into the new disk, and says every minute that it is still
   going;
6. says alo OS is installed, and restarts.

Every refusal before step 5 ends with *so nothing was changed* and *you can turn
this computer off or restart it now*, and the environment does not restart by
itself after one. A release whose signature does not verify is said as *this
download is not a genuine alo OS, so nothing was changed*.

### Watching it in a virtual machine

`crates/alo-installing/tests/installed_in_a_virtual_machine.rs` does all of this
on Linux with QEMU and OVMF: it builds the recipe, lays out a first disk the way
Windows does with the environment staged beside it, attaches an empty second
disk, and starts the machine with Secure Boot on under Microsoft's certificates.
It hashes the first disk before and after, then starts the second disk on its
own and reads what is running. A second test gives the environment a key that
is not the owner's and watches it refuse. A third changes one byte of the staged
loader and watches the firmware refuse *that*, so the first two are known to run
under a firmware that really enforces Secure Boot. All three are run by name:

    cargo test -p alo-installing --test installed_in_a_virtual_machine \
      -- --include-ignored --test-threads 1

**The Secure Boot firmware comes out of the pinned base**, not from the host's
packages: Ubuntu's `ovmf` 2025.11-3ubuntu7 page-faults starting the base's own
signed loader, and Fedora's `edk2-ovmf-20250812-21.fc42` starts it with Secure
Boot enabled (`docs/quirks.md`, *EDK II's strict image protection page-faults the
base's signed loader*). The test installs it into a container of the base once
and keeps it in its work directory.

**The install does not pass yet** (2026-09-16). What now works, measured: the
firmware starts the staged shim and loader with **Secure Boot enabled**, the
loader hands over the disk the person chose, the environment says every step,
the signature of the pinned release verifies — *this is a genuine alo OS* — and
`bootc install` begins and deploys the release. It then fails installing the
bootloader, and the environment says *alo OS could not be installed onto <disk>.
That disk may now hold part of alo OS; nothing else on this computer was
changed.* **The serial line now says why**, in the installer's own words:
*Installing bootloader: Probing bootupd --filesystem support* and *bwrap:
pivot_root: Invalid argument* (`docs/quirks.md`, *in the boot environment, the
image deploys and the bootloader's probe dies in `bwrap`'s `pivot_root`*). **The
cause is located** from a boot of the environment's own initramfs: the
environment runs from the kernel's initial root, which has no mount above it, and
`pivot_root(2)` refuses exactly that. So the installer's unit now runs under the
environment bound again at `/run/alo/installing/root`
(`image/installing/run-alo-installing-root.mount`, taken as the unit's
`RootDirectory=`), where the same `bwrap` pivots and starts `bootupctl`. Whether
the install then finishes and the disk boots to `alo-agentd` is the installer
plan's task 13, not yet run. Written up in
`docs/autonomy/updates/with-secure-boot-on-the-staged-loader-starts.md` and
`docs/autonomy/updates/the-boot-environment-says-why-an-install-stopped.md`.

**Why a program failed goes to the serial lines, never the screen.** When the
signature check or the install fails, the environment writes the last lines that
program complained of to the machine's log and to every console the kernel lists
that is not a virtual terminal — `ttyS0`, `hvc0` — each line prefixed with the
program's path. The screen keeps only the sentences a person reads, which never
name the machinery. A machine with no serial line keeps the lines only in the
environment's journal, which lives in memory and is gone at the restart. Every virtual-machine test removes the disks it
made when it ends, pass or fail, and keeps its serial logs.

**The refusal passes.** It is started without Secure Boot and refuses, with the
second disk untouched and the first disk byte-for-byte unchanged. It once changed
the first disk, and the writer was the virtual machine's firmware, not the
environment: the firmware without SMM, given flash only SMM may write, saved its
variables as `NvVars` on the staged FAT (`docs/quirks.md`, *OVMF without SMM saves
its variables onto a FAT disk when its flash is SMM-only*;
`docs/autonomy/updates/a-refusal-writes-nothing.md`). When starting OVMF by hand,
keep a firmware and its machine together the way the test's `Firmware` does.

**Nobody has watched it in Hyper-V yet**: the account the tests ran under on
2026-09-15 is not allowed to manage Hyper-V, and the gates run in Linux. By
hand, the same shape is a generation 2 machine with Secure Boot
under the *Microsoft UEFI Certificate Authority* template, a first disk holding
the `ALO-INSTALL` partition as its boot device, a second empty disk of at least
24 GB, and a network adapter on a switch with a route out. On Hyper-V's SCSI
disks the name under `/dev/disk/by-id/` is the disk's `scsi-` or `wwn-` name,
and that is what `chosen.cfg` must hold.

## Attaching it to Hyper-V

alo OS is installed for UEFI, so it is a **generation 2** virtual machine, and
generation 2 is the only shape this repository tests or supports.

It is worth being exact about why, because this document first said a
generation-1 machine *would not find anything to boot*, and that is untrue. The
disk written on 2026-09-11 carries a 1 MB BIOS boot partition beside its 512 MB
EFI system partition, its master boot record holds the `55aa` signature and the
string `GRUB`, and that BIOS partition holds GRUB's core image — so a BIOS
machine would find a bootloader. **Generation 2 is a choice, not a necessity:**
it is how a certified laptop boots, so it is what gets tested, and a
generation-1 boot is simply a path nobody here has walked.

In Hyper-V Manager, on a Windows 11 Pro host:

1. **New → Virtual Machine**, generation **2**.
2. Memory: 8192 MB or more, **if the host has it to give**. The model runtime
   is on the image
   ([ADR 0025](decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)),
   and a machine that swaps while it answers tells you nothing useful.

   On a host that does not, Hyper-V refuses rather than starting something
   small: on 2026-09-11 a 15.5 GB development machine running a browser, an
   editor, WSL and two build loops had about 1.1 GB it could offer, and
   Hyper-V turned down 4096 MB, then 1024 MB, then 768 MB in turn — *not
   enough memory in the system*, which is a sentence about the host and not
   about this disk. Close what is holding the memory before starting the
   machine. The same shortage is why no local model has been graded yet
   (`quirks.md`), and it is the clearest argument in this repository for the
   certified machine having real memory in it.
3. Networking: leave it disconnected for the first boot. Nothing on this image
   needs the network to start, and a machine sold on
   [nothing leaving silently](../CLAUDE.md) is one whose first boot is worth
   watching with the wire out.
4. **Use an existing virtual hard disk**, and choose `alo-os.vhdx`.
5. Before starting it: **Settings → Security**. Either turn Secure Boot off, or
   set the template to **Microsoft UEFI Certificate Authority** — the Fedora
   bootloader is signed by that authority and not by the Windows one, which is
   what the default template trusts.

What you should see is a text console and a login prompt. There is no
compositor on this image, nothing to draw and nowhere to sign in graphically;
that is the honest state of v0.01 and not a machine that came up wrong.

**What was seen, 2026-09-11.** The first run of this document, on the owner's
15.5 GB Windows 11 Pro host, watched by the owner:

- **It boots.** Generation 2, Secure Boot **on** under the *Microsoft UEFI
  Certificate Authority* template — the step above that nobody had yet watched.
  Fedora's shim passed it. The guest reported a heartbeat through the
  integration services at 92 seconds with the processor idle, and the console
  showed the text login this section promises.
- **768 MB was enough to boot to that console** — dynamic memory, 512 MB floor,
  2 GB ceiling — after the host had refused 4096, 1024 and 768 MB while a
  browser held the memory. The 8192 MB above is for *answering*, not for
  arriving at a prompt, and the two are different numbers.
- **No address came up on the Default Switch.** Expected: nothing on this image
  configures a network yet, and step 3 says to leave it out for the first boot
  anyway. It is noted so nobody reads a blank address as a broken machine.
- **The disk that booted was built from a six-day-old image**, before the
  weights and the serving unit were added; the current image was built the same
  evening. **Its disk booted the next morning, 2026-09-12:** 9.6 GB (the model
  is 4.5 GB of it), the same machine switched to it, heartbeat at 60 seconds,
  768 MB. So the image that carries the weights boots.
- **What neither run can show:** whether each service came up. The image ships
  no accounts (ADR 0024), so nobody can log in at that console to ask systemd;
  the heartbeat proves the kernel and nothing about the units. That stays owed
  until there is a sign-in.

## What a virtual machine cannot show

This is not the hardware acceptance in phase 8 of
`docs/autonomy/DELIVERY.md`, and nobody may quote it as one:

- **The GPU.** A virtual display adapter is not *the GPU works on first boot*.
  What Hyper-V presents is a synthetic framebuffer, and the certified machine's
  graphics path is not exercised by any of it.
- **The firmware.** Hyper-V's firmware is tame and known; a certified laptop's
  is neither, and *firmware to sign-in* is a measurement on that machine.
  Secure Boot answering here says nothing about Secure Boot answering there.
- **Anything physical.** Suspend, the lid, the keyboard's own firmware, the
  wireless card, the battery, the trackpad, and the time from power to a
  prompt. A virtual machine has none of them, and no line in `ROADMAP.md` about
  hardware may be ticked from this document.

What it does show is that the image installs, that the disk boots, and that the
two services come up in the order their units say — which is the difference
between a repository that believes it boots and one that has watched it.

Whatever the first run finds that contradicts this document belongs in
[`quirks.md`](quirks.md), in the same change that finds it.
