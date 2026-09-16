# alo OS 0.0.1 — the installer

The notes the GitHub Release carries, committed here so that nothing about the
release is typed twice. `crates/alo-image` holds every fact below to
`image/pinned.toml` and to what `crates/alo-installer` actually accepts, and the
workflow in `.github/workflows/release.yml` publishes this file as the Release's
body rather than a sentence somebody wrote in the web form.

Download `alo-installer.zip`, unpack it, and run `alo-installer.exe`. It reads
this computer, says what it found, says exactly what it will do, and changes
nothing until you type the name of a disk.

**Windows stays.** alo OS is installed onto an empty disk beside it, and
Windows starts exactly as it did before. Nothing on the Windows volume is
touched except the 1 GB the installer asks Windows itself to free.

**Secure Boot.** This release installs only on a computer where Secure Boot is
already off. Where it is on, or where Windows will not say, the installer stops,
says why, and changes nothing — and it never asks anybody to alter a firmware
setting.

What this release installs, from the registry, by content:

    registry: ghcr.io/aloworld-org/alo-os
    tag: 0.0.1
    digest: sha256:d3f05b60975edcff51a44c1f21e764a32b286677e306ba24631bad6a00b6a13c
    secure boot: off

Nothing but that digest is pulled, whatever a tag in the registry says later.

`SHA256SUMS` is published beside the download: it is the checksum of every asset
in this Release, so a download can be compared with what was published.

What this computer needs is in `docs/hardware.md`, and the README's *Try it*
says it in short.
