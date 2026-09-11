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

Then the one command that turns it into a disk:

    podman run --rm --privileged --pid=host \
      --security-opt label=type:unconfined_t \
      -v /var/lib/containers:/var/lib/containers \
      -v .:/output \
      localhost/alo-os:dev \
      bootc install to-disk --via-loopback --wipe \
        --filesystem ext4 /output/alo-os.raw

`--via-loopback` is what makes a *file* an acceptable target; without it the
tool expects a block device, which on a workstation means somebody's disk.
`--wipe` is there so that running it twice is running it twice rather than
writing into whatever was left.

And for Hyper-V, one conversion:

    qemu-img convert -f raw -O vhdx -o subformat=dynamic \
      alo-os.raw alo-os.vhdx

## Attaching it to Hyper-V

alo OS is installed for UEFI, so it is a **generation 2** virtual machine.
Generation 1 is the BIOS one and is not a shape this repository tests or
supports; a generation-1 machine pointed at this disk will not find anything to
boot.

In Hyper-V Manager, on a Windows 11 Pro host:

1. **New → Virtual Machine**, generation **2**.
2. Memory: 8192 MB or more. The model runtime is on the image
   ([ADR 0025](decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)),
   and a machine that swaps while it answers tells you nothing useful.
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
