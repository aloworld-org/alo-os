# alo OS 0.0.3 — the installer

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
    tag: 0.0.3
    digest: sha256:41d43c7ea491990eea602493e5c645bd7bf8e7d0d9d7a8e9a000bc895a9dd0d7
    secure boot: off

Nothing but that digest is pulled, whatever a tag in the registry says later.

**What changed since 0.0.1.** The disk this installs carries the door on the way
to the kernel boundary, which 0.0.1 did not — without it an installed machine
runs the agent service but the boundary it is meant to be held inside has no
way through. It also carries the document engine, so a `.docx`, `.xlsx` or
`.pptx` is converted on the machine itself, by a service that can reach nothing,
and what the copy could not carry is said by name rather than lost quietly.

`SHA256SUMS` is published beside the download: it is the checksum of every asset
in this Release, so a download can be compared with what was published.

What this computer needs is in `docs/hardware.md`, and the README's *Try it*
says it in short.
