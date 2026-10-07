# alo OS 0.0.6 — no model, and the machine says its own name

The notes the GitHub Release carries, committed here so that nothing about the
release is typed twice. `crates/alo-image` holds every fact below to
`image/pinned.toml` and to what `crates/alo-installer` actually accepts, and the
workflow in `.github/workflows/release.yml` publishes this file as the Release's
body rather than a sentence somebody wrote in the web form.

Download `alo-installer.zip`, unpack it, and run `alo-installer.exe`. It reads
this computer, says what it found, says exactly what it will do, and changes
nothing until you type the name of a disk.

**This release carries no model.** Earlier releases put a model on the disk of
every machine — about 5 GB of it, chosen by us. This one does not, because a
model we chose is a model chosen for you. You bring your own: point alo OS at
weights you already have, use a provider you have an account with, or use it
with no model at all. alo OS chooses none of those for you, and until you
choose, nothing answers.

**So out of the box it answers nothing**, and that is worth saying plainly
rather than leaving you to discover it. The machine is a machine; the agent
waits until you give it something to think with.

**The machine says it is alo OS.** Until this release it called itself *Fedora
Linux 42* at start-up and at the login prompt, and told you on every boot, in
red, that your operating system was past its end of support — a promise this
project never made. That is gone. What it is built on is still stated plainly,
because hiding that would help nobody.

**Windows does not know who made this program, and it will say so.** When you
run it, Windows shows a blue box saying *Windows protected your PC*, and the
only button offered is *Do not run*. To go on, choose **More info**, then **Run
anyway**. We are telling you here, before you download it, because a program
that repartitions your only computer should never be something you were
surprised by.

**You can check for yourself that what you downloaded is what we published.**
`SHA256SUMS`, beside the download, is a short code worked out from every byte of
the file. Work out the same code on your own machine:

    Get-FileHash -Algorithm SHA256 alo-installer.zip

If it matches the line in `SHA256SUMS`, the file on your disk is the file we
put there, byte for byte.

**Windows stays.** alo OS is installed onto an empty disk beside it, and
Windows starts exactly as it did before. Nothing on the Windows volume is
touched except the 1 GB the installer asks Windows itself to free.

**Secure Boot.** This release installs only on a computer where Secure Boot is
already off. Where it is on, or where Windows will not say, the installer stops,
says why, and changes nothing — and it never asks anybody to alter a firmware
setting.

What this release installs, from the registry, by content:

    registry: ghcr.io/aloworld-org/alo-os
    tag: 0.0.6
    digest: sha256:fcc732bb147e02fcb8b8840ce359a3e604295f2e453c5ef52d473ee328a3f8ae
    secure boot: off

Nothing but that digest is pulled, whatever a tag in the registry says later.

**What changed since 0.0.5.** The model is gone, as above. The machine stopped
calling itself Fedora and stopped promising somebody else's support. And the
boot environment can now install alo OS **into one part of a disk** rather than
only over a whole one, which is the half that *alongside Windows on one disk*
needs underneath it — that road is not finished, and this installer does not
offer it yet.

`SHA256SUMS` is published beside the download: it is the checksum of every asset
in this Release, so a download can be compared with what was published.

What this computer needs is in `docs/hardware.md`, and the README's *Try it*
says it in short.
