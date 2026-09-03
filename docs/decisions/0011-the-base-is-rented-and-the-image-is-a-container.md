# ADR 0011 — The base is rented, and the image is a container

**Status:** accepted — settles what `docs/features.md`'s "image built as an OCI
container image" is actually built on, and answers *why not our own base*
**Date:** 2026-09-03
**Context:** `docs/features.md` (the system and the image), `ROADMAP.md` (v0.01's
image line, v0.5's atomic updates), `docs/hardware.md`,
[ADR 0002](0002-the-shell-is-native.md),
[ADR 0005](0005-applications-are-sandboxed-and-ask.md),
`CLAUDE.md` ("engines are configured, never patched")

## The decision in one line

alo OS is built as a **bootable container image** on a rented, unmodified Linux
base — **not a distribution of our own** — because everything that makes alo OS
*alo* lives in layers we already own, and everything below them is the same code
every distribution ships from the same upstreams.

## What was actually being asked

Not "which distribution do we like". The question was **why we do not build our
own base**, asked by somebody who had already, correctly, refused to fork
Chromium and built a browser engine instead. That refusal was right, and the
same instinct applied here gives the opposite answer. Saying so requires a test
rather than a preference.

## The test

**Does building it buy something nobody sells?**

- **The browser engine: yes.** ADR 0002 of `alo-browser` needs the layout tree to
  *be* the agent's tree — declared roles, typed verbs, never a coordinate. No
  engine on earth offers that and it cannot be bolted on afterwards, which is
  exactly why forking was wrong there. Building it buys a capability that does
  not exist in the world.
- **A base distribution: no.** We would compile the same glibc, the same
  systemd, the same Mesa that Fedora and SUSE compile, from the same upstream
  sources, and arrive at what we already have for free — several years later,
  having spent the hours that would have built the shell.

`alo-browser`'s own non-goals say it in one line: *we rent the physics, and not
out of timidity*. A base distribution is physics.

## What "our own base" would actually mean

Not writing software. Nobody writes their own glibc, systemd, PipeWire or Mesa.
It means owning **the packaging and the security response for the whole lower
half of the machine, permanently**: a build farm and a package repository,
CVE tracking across roughly fifteen hundred packages with fixes shipped in hours
rather than days, hardware enablement for the long tail of devices, release
engineering, mirrors and signing. Five to fifteen engineers doing nothing else,
forever, for **no feature a customer would ever notice**.

`CLAUDE.md` already decided this in one line — *engines are configured, never
patched; Linux, Mesa, systemd run as pinned upstream components* — and a base
distribution is precisely those components packaged. This ADR is that rule
applied to the question rather than a new position.

## The decision

**The image is a bootable container (`bootc`), on a Fedora-derived base.**

The reason is not taste. Four sentences were already written before this ADR
existed, and bootc is the only option that makes all four true at once:

| Already promised | What bootc gives |
|---|---|
| "Image built as an OCI container image" | the OS **is** an OCI image, literally |
| "Atomic updates with rollback" (v0.5) | `bootc upgrade` / `bootc rollback`, built in |
| Flatpaks, not a vendor store (ADR 0005) | Flatpak is first-class |
| "Signed images verified before boot, Secure Boot with our key" (v1) | we build, host and sign our own image |

**We still own the supply chain.** Our own image, our own registry, our own
signing key. The dependency is on packages, not on a company.

**Ubuntu is rejected for the shipped image**, and this is worth stating because
it was the intuitive answer. Canonical's atomic path is Ubuntu Core with
**snaps**, whose store backend is proprietary and single-vendor — a contradiction
inside a sovereignty product, and a direct conflict with ADR 0005's choice of
Flatpak. Building a bootable OS from a container image is also not Ubuntu's
paved path, so we would be working against the one sentence in `features.md`
that constrains this most. *Ubuntu as the development environment is a different
question and a fine answer; this decision is about the image customers boot.*

## Renting the base does not cost us being AI-native

The objection worth taking seriously is that owning the base would make alo more
of an AI-native system. Traced layer by layer, it would not:

| What an agent must do | Which layer provides it | Whose |
|---|---|---|
| See windows and act on them | the compositor | **ours** |
| Read the interface as a tree of what it *is* | the compositor, ADR 0002 | **ours** |
| Touch files only under a grant | `alo-agentd`, `alo-files` | **ours** |
| Drive installed applications | the XDG portal *backend*, which we write (ADR 0005) | **ours** |
| Ask a model on this machine | `alo-models` over a pinned runtime | **ours** |
| Have every action recorded | `alo-record`, `alo-keeping` | **ours** |
| Let nothing leave silently | `alo-egress`, over the kernel's own filtering | **ours** |
| **Undo what the agent did this afternoon** | **atomic snapshots and rollback** | **the base** |

Seven of eight are ours whatever the base is. Being AI-native is decided in the
**compositor, the daemon and the portal backend**, which is exactly what ADR 0002
was protecting when it insisted the shell be native.

The eighth is real, and it points the same way this ADR already does: *undo what
the agent did* leans on atomic rollback, bootc has it natively, and Ubuntu Core
is the weakest fit for it. **The AI-native argument does not favour building our
own base; it favours not choosing Ubuntu.**

## Consequences

- `docs/features.md`'s OCI line stops being a shape and becomes a mechanism, and
  v0.5's *atomic updates with rollback* is largely inherited rather than built.
- **The boundary is expected to move.** A bootc image is not a distribution
  adopted whole — we choose what goes in it, and we already refuse GNOME and
  write our own shell. Each layer is owned at the moment owning it buys
  something, and not before.
- **Forking stays cheap, and that is what makes renting safe.** If the base ever
  breaks a promise we cannot work around — forced telemetry, a proprietary
  store, a licence change — we fork it, from a position of already having one.
- `docs/hardware.md`'s two certified machines are unaffected: the base changes
  nothing about which hardware we stand behind.
- Nothing here touches the kernel. *No kernel* remains a non-goal.

## The named trigger to revisit

**A procurement conversation that turns on the base vendor being European.**
That is openSUSE MicroOS's case — transactional updates with btrfs rollback are
mature, Flatpak is first-class, and SUSE is a German company one can buy support
from, which is a real asset in public-sector procurement. It is weaker on the
OCI-image requirement and stronger on sovereignty optics.

This trigger is written down rather than left to memory, because it is the one
condition under which this decision is wrong rather than merely arguable.

## Alternatives rejected

**Build our own base.** Rejected on opportunity cost, not difficulty: it buys no
capability that is not already free, and it competes for the exact engineering
hours that build the shell and `alo-agentd`. The sovereignty gain is real and is
obtained more cheaply by owning the image, the registry and the signing key.

**Ubuntu Core.** Rejected: snaps conflict with ADR 0005, the store backend is
proprietary and single-vendor, and building from an OCI image is not its path.

**openSUSE MicroOS.** Not rejected — held, with the trigger above.

**NixOS.** Rejected on `CLAUDE.md`'s two-languages rule: Nix would be a third
language in the repository, which that document calls a bug.

**Debian.** Rejected reluctantly: the most neutral governance of the options and
excellent for sovereignty optics, but no native bootable-container story, so the
atomic-update machinery would be ours to build — which is the cost this ADR
exists to avoid.
