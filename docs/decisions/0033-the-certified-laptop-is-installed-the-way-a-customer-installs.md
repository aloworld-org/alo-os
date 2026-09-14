# ADR 0033 — The certified laptop is installed the way a customer installs

**Status:** accepted
**Date:** 2026-09-14 — decided by the owner: *"I want us to test the OS on the
new laptop with the installer, so it will be downloaded and installed from
GitHub or the website."* Recorded and shaped under the owner's standing
delegation.
**Context:** [ADR 0023](0023-installed-from-the-machine-it-replaces.md) (the
install story: a downloaded program, a staged boot environment, `bootc
install` from a registry), [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)
(the OS is a bootable container), `docs/hardware.md` (the certified laptop,
which arrived on 2026-09-14), `docs/autonomy/v0-01-evidence.md` (the two
promises no VM can evidence), `ROADMAP.md` (*Installer*, v0.5, unbuilt).

## The question in one line

**The certified laptop is here. Does v0.01's hardware acceptance happen by an
expedient boot, as ADR 0023 allowed, or through the installer a customer will
use — which does not exist yet?**

## What was true before this decision

ADR 0023 decided the front door — *download, click, reboot, sign in* — and
said in the same breath that *v0.01 still ships no installer: certification
happens on our own two machines by whatever internal boot method is
expedient*. That was written when the laptop was months away and the
installer was a v0.5 line. Both are now on the desk at once, and the owner
has chosen.

## The decision

1. **Hardware acceptance goes through the installer.** The certified laptop is
   installed by downloading the installer from GitHub and running it on the
   Windows the laptop arrived with. What that run proves is written into
   `docs/autonomy/v0-01-evidence.md` under the two promises no VM can evidence,
   and nothing about those promises is ticked by any other road. ADR 0023's
   *whatever internal boot method is expedient* is withdrawn.
2. **Windows stays.** ADR 0023 already allows *retained alongside*; on the
   certified laptop it is the only mode used, and the installer's default
   everywhere. Replacing Windows remains an explicit, twice-confirmed choice
   that the certification never exercises.
3. **The registry is GitHub's.** The image is published to
   `ghcr.io/aloworld-org/alo-os`, and the installer is a GitHub Release of
   the same repository — *install from GitHub* made literal, one channel for
   install and update alike. Until a workflow builds and pushes the image, it
   is pushed from the machine that built it, and the digest the installer
   pulls is pinned in one file a test reads.
4. **Secure Boot: refused, not disabled — except by the owner, for
   certification, on the record.** The shim review ADR 0023 names as the
   pacing item is external and takes months. Until it lands, the shipped
   installer **refuses to proceed with Secure Boot enabled** and says why, in
   the person's language; it never asks anybody to switch it off. For the
   certification laptop the owner disables Secure Boot in firmware
   themselves, and the evidence ledger records that the machine was certified
   that way, so *boots on one certified machine, firmware to sign-in* is not
   read as *with Secure Boot on* until it is.
5. **Every step leaves Windows bootable until the one named point.** ADR
   0023's constraint, restated because the first machine this runs on is the
   owner's new laptop: the installer's bail-out is tested in a virtual machine
   with a real Windows in it before it touches the laptop, and the point of
   no return is one typed sentence naming what is destroyed — which, alongside
   Windows, is nothing.

## What it costs

- **The installer moves from v0.5 to the critical path**, ahead of everything
  v0.5 that needs a screen. That is the point: the on-the-machine column has
  been empty since the repository began, and this is the one road that fills
  it without a stick.
- **Certification with Secure Boot off is a weaker certification**, and the
  ledger says so rather than hiding it in a footnote. The stronger one arrives
  with shim, and re-runs the same installer.
- **The sign-in is still the desktop lane's.** What the installer test proves
  on the laptop is firmware to the daemon, the boundary on the real kernel,
  the GPU on first boot and the pinned model running on the certified CPU. The
  sign-in screen is ticked the day the desktop lane's shell is in the image,
  and not before.

## Rejected

- **Booting the laptop by an expedient method first, the installer later.**
  It would tick boxes with a road no customer will ever take, and then require
  the same work again. The owner declined it, and the reasoning is theirs.
- **Waiting for v0.5 to finish before touching the laptop.** v0.5's exit gate
  needs the desktop, which is away; the laptop would sit idle behind a
  dependency it does not have. Declined on 2026-09-14.
- **Disabling Secure Boot for customers "for now".** ADR 0023 rejected it and
  this decision keeps the rejection; the owner's own machine is the one place
  the expedient is taken, and it is written down.
