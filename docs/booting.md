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
        --filesystem ext4 /output/alo-os.raw

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

And for Hyper-V, one conversion:

    qemu-img convert -f raw -O vhdx -o subformat=dynamic \
      alo-os.raw alo-os.vhdx

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
