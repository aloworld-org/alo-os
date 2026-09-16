# The screen reader and the tree it reads

**Date:** 2026-09-16
**Workstream:** v0.5 — access and language
**Task:** *The screen reader and the tree it reads*
(`docs/autonomy/v0-5-access-and-language-plan.md`, task 2)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written and tested on an **Apple M3 with 8 GB unified memory**,
macOS 26.5.2; the voices were read in the Lima VM (Ubuntu 24.04 aarch64, kernel
7.0.0-31-generic), where the gates also run.
**Egress:** one — `apt-get install espeak-ng` in the VM, version
`1.51+dfsg-12build1`, to read what voices exist rather than assume them. Nothing
was installed on the Mac and nothing was added to the image.
**Status:** done.

## What changed, for somebody outside this repository

- **Every surface this machine draws now has something to say about itself** —
  what kind of thing each control is, what it is called in the person's own
  language, and what can be done with it. One description, used by the screen
  reader and by the agent, because `docs/contracts/app-adapters.md` has the
  agent read applications through the same tree: if it is good enough for one it
  is good enough for the other.
- **A surface added to the shell with nothing to say about it fails a test in
  this lane.** The person who adds a screen is told that somebody who cannot see
  it would have no way to know it exists.
- **The approval surface is read as the sentence the turn wrote, then *no*, then
  *approve*, with nothing chosen for the person** — ADR 0001 read aloud.
- **What is leaving the machine, and what the agent is doing, are announced when
  they change** rather than found by looking. Law 1's indicator is a feature, and
  for somebody who cannot see it, a feature they can hear.
- **All 24 official languages have a voice** in the engine this was measured
  against.

## The surfaces, and what a reader is told

| Surface | Drawn by | Read as |
|---|---|---|
| sign-in | `SignInScreen` | the screen, who is signing in, the password field, sign in — **and the access settings** |
| desktop | `DesktopFrame` | the desktop, the windows open |
| dock | *inside the desktop's frame* | the dock, and each application in it |
| status area | `EgressStatusFrame` | what this machine is doing; *something is leaving*, *the agent is working*, both announced |
| approval | `ApprovalFrame`, `ApprovalScreen` | the question, the sentence, no, approve |
| record | `RecordFrame` | the record, what happened, each entry |
| settings | `SettingsFrame` | settings, what can be changed, each setting on or off |
| window controls | `WindowControlFrame` | close, move |

The dock has no frame of its own — it is drawn inside the desktop's — and it is
listed anyway, because it is a surface a person meets and a reader must be able
to name it. That is the one entry in this table that is not held to an export.

**The access settings are in the sign-in surface**, held by its own test: task 1
put every accessibility setting where there is no account yet, and this puts the
button that reaches them where a reader can find it. The two halves of the same
promise.

## How the list is held to the shell's

`alo-access` names, for each surface, the frame or screen the shell exports for
it; the test reads `crates/alo-shell/src/lib.rs` **as text** and fails when an
exported frame has no entry. It is read and never imported: a crate that decided
what a reader is told *and* depended on the crate that draws would have the arrow
pointing both ways, and this plan owns nothing in `crates/alo-shell`.

Five of that crate's exports are not surfaces a person meets, and are named in
the test so nobody has to guess: `DirectFrame` and `XrgbFrame` are how a frame
reaches a screen, and **`NestedReaderFrame`, `WindowControlLabelFrame` and
`WindowControlReaderFrame` are the keyboard's own labels** — the *reader* in
those names is the label reader for keyboard navigation, not a screen reader.
The next person to read those names will assume otherwise, which is why it is
written down.

`SettingsFrame` landed in the shell while task 1 was publishing, so Settings is a
surface with a role here rather than the finding it would have been an hour
earlier.

## The voices, per language

Read from `espeak-ng --voices` on 2026-09-16: **all 24 have a voice**, English
with eight regional ones, of which the first the engine lists is recorded —
which regional voice a person is given is their setting, not this table's claim.

| | |
|---|---|
| Bulgarian, Croatian, Czech, Danish, Dutch, English | ✓ |
| Estonian, Finnish, French, German, Greek, Hungarian | ✓ |
| Irish, Italian, Latvian, Lithuanian, Maltese, Polish | ✓ |
| Portuguese, Romanian, Slovak, Slovene, Spanish, Swedish | ✓ |

**But the finding under it: alo OS pins no speech engine.** ADR 0011 rents
engines — Orca, AT-SPI and a speech engine among them — and `alo-image` names
none of the three. So this table is what **eSpeak NG 1.51** offers, the engine
Orca speaks through by default on the distribution alo OS is built from, and it
is *not yet* a statement about a shipped machine. When the image pins one, the
table is re-read against that engine and whatever it lacks becomes a language
written down as unspoken. The test is shaped for that: it fails naming the
languages with no voice, rather than passing quietly at 24.

That is a task for whoever owns the image, and it is named here rather than
assumed away.

## What is not done here

- **Nothing draws and nothing speaks.** The shell exposes this through AT-SPI
  and Orca speaks it — both rented, neither written by us (ADR 0011).
- **Starting the reader from the setting** is the shell's, from `alo-access`'s
  value: task 1 decided *screen reader on*, and this decides what it would read.
- **Keyboard-only operation** is task 3, still blocked on the desktop plan's
  keyboard crate.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, run by the Mac lane's publish script on the tree combined with `main`,
which publishes only on all nine green with nothing failing.
