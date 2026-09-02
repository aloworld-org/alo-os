# ADR 0005 — Applications are sandboxed, and ask for what they need

**Status:** accepted
**Date:** 2026-09-02
**Context:** how a person installs software; the XDG Desktop Portal interfaces;
`docs/decisions/0001-the-capability-model.md`;
`docs/contracts/app-adapters.md`

## The decision in one line

Third-party applications install **sandboxed**, as Flatpaks from repositories a
person or an organisation chooses, and reach the system only through the **XDG
Desktop Portal** interfaces — which is not a second permission system, but the
grant model of ADR 0001 applied to applications instead of agents.

## What was actually wrong

`docs/features.md` described an operating system whose agents can drive Blender
and DaVinci Resolve, and never said how Blender gets onto the machine. That is
not a small omission: how software is installed determines what the security
model is actually worth, and it was about to be decided by whatever we
happened to build first.

It also sat badly with an existing non-goal. "No general-purpose distribution,
no package manager for the world" is right — we are not becoming Ubuntu — but a
workstation with no way to install software is not a workstation.

## The decision

**Applications are Flatpaks, sandboxed by default.** A person installs from
Flathub or from a repository their organisation runs; nothing requires us to
package the world, and nothing requires an application to be modified for us.

**Applications reach the system through portals.** File access, screenshots,
screen capture, camera, microphone, printing, notifications, secrets, USB,
location — each is a portal request, and each is a request a person answers.

**And that is the same model we already have.** ADR 0001 says an agent reaches
nothing a person has not granted, and that grants are enumerated, visible,
revocable and expiring. A portal request is that sentence with "application"
substituted for "agent". So alo OS has **one** permission story, told about two
kinds of actor:

| | An agent asks | An application asks |
|---|---|---|
| Through | an enumerated verb | a portal interface |
| Reaches | only granted paths and devices | only granted paths and devices |
| Recorded | every execution and refusal | every grant and use |
| Revoked | one action, immediate | one action, immediate |

The person sees one list of what has been granted to what. Not an agent list
and an application list, because nobody keeps two.

**We implement the portal backend against our native shell.** That is the work
this decision buys: a portal request must be answered by *our* file chooser, our
screenshot flow, our permission surface. Doing it makes every existing Linux
application work on alo OS without knowing we exist.

**Unsandboxed software is possible and is a deliberate act.** A person may
install something that runs without confinement — that is what owning a computer
means, and it is the same instinct as shipping a terminal. It is never the
default, it is clearly marked, and on a managed machine an organisation can
forbid it by policy (ADR 0004).

## Consequences

- **Adapters drive sandboxed applications** (`docs/contracts/app-adapters.md`).
  An adapter's reach is bounded by the application's sandbox and by the grants
  behind it, which makes an adapter meaningfully safer than the automation
  interface it wraps.
- **The portal surface is a contract**, so it belongs in `docs/contracts/` and
  changes additively.
- **Secrets get a real home.** The Secret portal means applications stop
  inventing credential storage, and the keyring becomes something we are
  responsible for doing properly.
- We inherit an ecosystem instead of building one. That is the same doctrine as
  every other rented engine: configured, never patched.

## Alternatives rejected

**A traditional package manager, with unsandboxed system-wide installs.**
Rejected: it contradicts the non-goal directly, and every installed package
would hold the authority of the person who installed it. A product whose
central claim is that reach comes only from a deliberate grant cannot have an
install step that hands out everything at once.

**Snap.** Rejected: the store backend is single-vendor and proprietary, which is
an odd dependency for a sovereignty product to take on.

**Our own application format and store.** Rejected: it is a decade of ecosystem
work to arrive where Flatpak already is, and it would ask every application
vendor to care about us before we have users.
