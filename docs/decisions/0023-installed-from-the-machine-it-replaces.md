# ADR 0023 — Installed from the machine it replaces

**Status:** ACCEPTED, 2026-09-10 — decided by the owner: *"I do not want to use
USB flash. I want the users to be able to install it from GitHub, website."*
Recorded and shaped by the workstream under the owner's standing delegation.
**Date:** 2026-09-10
**Context:** `ROADMAP.md` (v0.01 ships **no installer** — unchanged by this),
`docs/features.md` (the Windows 10 fleet is the market), `image/` (the OS is a
`bootc` bootable container), ADR 0007 (the ordinary laptop decides the market)

## The decision in one line

**alo OS is installed by a small program the person downloads and runs on the
Windows machine it replaces** — which stages a minimal boot environment,
reboots into it, and pulls the operating system itself from a container
registry over HTTPS. No USB stick, no ISO burning, no firmware ceremony.

## Why this and not the usual ISO

**The target machine already has an operating system on it.** This product
exists to catch the Windows 10 fleet Microsoft is abandoning; by definition,
every machine we want is running Windows and can run a downloaded program.
Asking that person to find an 8 GB stick, a flashing tool and the boot-menu key
is asking them to become a hobbyist first — and the ones who would are not the
market.

**The image is already the right artifact.** alo OS is a `bootc` bootable
container, and `bootc install` is *designed* to install from a registry.
"Install from GitHub" is therefore not marketing over a download link: the
installer's reboot environment pulls the same signed, versioned image CI
built, from the same registry updates come from afterwards. One artifact, one
channel, install and upgrade alike.

The person's whole experience: **download, click, reboot, sign in.**

## The shape

1. A signed Windows executable, downloaded from the website. It checks the
   machine (UEFI, disk, memory, BitLocker state), says exactly what will happen,
   and takes a typed consent — not a checkbox — for anything destructive.
2. It stages a minimal boot environment and a UEFI boot entry, then reboots.
3. The environment runs `bootc install`, pulling alo OS from the registry over
   HTTPS, verifying signatures before writing.
4. First boot lands in alo OS's sign-in. Windows is either retained alongside
   (default where disk allows) or replaced (explicit choice, twice confirmed).

## Three constraints that are part of the decision

**Secure Boot is respected, never a thing the person disables.** The boot chain
signs through the established shim review process, and that request **starts
well before the installer ships** — it is external review with lead time, and
it is the pacing item.

**This is the most dangerous code the product will ever ship.** It repartitions
somebody's only computer, likely BitLocker-encrypted, from a download. It gets
kernel-boundary paranoia: verify before write, no destructive step without
typed consent naming what is destroyed, and a tested bail-out that leaves
Windows bootable at every step until the final, named point of no return.

**A dead machine still needs media.** A machine with no working OS cannot run a
downloaded installer; physics offers only existing-OS, network boot, or
removable media. Recovery media therefore continues to exist as the fallback —
it is simply not the front door, and the front door never requires it.

## What this does not change

- **v0.01 still ships no installer.** Certification happens on our own two
  machines by whatever internal boot method is expedient. This lands in v0.5,
  riding on the update/rollback promise already there.
- Nothing about what the OS does once installed; this is distribution only.
- UEFI HTTP boot (no Windows, no media, firmware pulls from a URL) is noted for
  the v1 fleet story and deliberately not the consumer path — consumer firmware
  is too inconsistent to promise on.

## Rejected

- **ISO + USB as the primary path.** Right for enthusiasts, wrong for the
  market, and every competitor's front door already looks like that.
- **Shipping without shim signing and telling people to disable Secure Boot.**
  A sovereignty product must not open by asking people to switch a protection
  off.
- **A web page that flashes a stick via WebUSB.** Still a stick.
