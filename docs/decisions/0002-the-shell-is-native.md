# ADR 0002 — The shell is native; the workspace is an application on it

**Status:** accepted
**Date:** 2026-09-02
**Context:** the session shell, the compositor, `alo-workplace` (the workspace
client), `alo-engine` (the rendering engine — since started; see the last
section)

## The decision in one line

Everything a person needs to reach the machine — the compositor, sign-in, the
lock screen, the launcher, window management, recovery, and the agent's
invocation overlay — is **native Rust and depends on no web engine**; the
workspace is one application running on that shell, like any other.

## What was actually wrong

An early plan had alo OS booting into the existing Tauri desktop app
full-screen. It was attractive because it works today and costs nothing: a year
of polished TypeScript, running as the session, immediately.

It is the wrong architecture, for a reason that has nothing to do with taste.
**A lock screen that needs a browser engine to draw itself is a lock screen that
cannot be trusted to let you back in.** The same applies to sign-in and to
recovery: when the engine wedges, misrenders or fails to start, the person needs
a way into their machine that does not depend on the thing that broke.

No serious operating system does this. Windows' shell is native. GNOME and KDE
draw with native toolkits — GNOME Shell's *logic* is JavaScript and KDE uses
QML, so a scripting language driving a native surface is entirely normal, but
neither renders its session as an HTML document inside a browser.

There is also a slower failure mode. On Linux there is no system Chromium
webview; a webview shell would sit on WebKitGTK, the weakest engine available
for performance and GPU acceleration, as the foundation of every session.

## The decision

**Native, in Rust:**

- the compositor (Wayland, via Smithay)
- sign-in and the lock screen
- the launcher and window management
- the recovery and rollback screen
- the agent's invocation overlay — it has to work when the workspace does not

**An application on top:** the alo workplace client, which today is a web
application, and is intended one day to be rendered by `alo-engine` — a
decision taken, and — **since this ADR was accepted** — started: `alo-engine`
is a repository of its own (`aloworld-org/alo-browser`) with an engine that
already renders an alo screen and diffs it on every run. It is one program among
others, exactly as a browser is one program on GNOME. That the workspace is
currently a web app is a fact about that application, not about this operating
system.

**And the swap, if it comes, is incremental.** Nothing here depends on it: the
shell is native whether or not the engine is ever built. When `alo-engine` can
render a workspace module, it does, behind a stable boundary — screen by screen,
with no visible break and nothing thrown away.

## Consequences

- The native shell is real work that must exist before the system boots to
  anything, so v0.01 is further out than a full-screen webview would have been.
  That cost is accepted; it buys a system that is recoverable when its most
  complex component fails.
- The design tokens must leave CSS. `alo-workplace` keeps its visual truth in a
  stylesheet, which a native Rust shell cannot read. The tokens become a
  language-neutral source generating both the CSS the workspace uses and the
  constants the shell uses — otherwise the two drift apart within months.
- The UI runtime built for the shell is the same runtime `alo-engine` would
  need for its first stage. If the engine is ever scheduled, that work is its
  foundation arriving early rather than a detour; if it never is, the shell
  still needed it.
- The agent overlay being native is what lets the invocation rule in ADR 0001
  hold under failure: the one key that summons an agent works even when the
  workspace is wedged.

## Alternatives rejected

**Full-screen webview shell.** Rejected above: unrecoverable failure modes at
sign-in, lock and recovery, on the weakest available engine.

**Native shell *and* a native workspace, immediately.** Rejected: it means
rewriting sixteen modules before the system does anything useful, throwing away
a year of working product to arrive at something worse. The workspace migrates
when the engine earns it.

**Adopt an existing desktop environment (GNOME, KDE) and add our agent.**
Rejected: the shell is where the agent lives and where the sovereignty guarantees
are enforced. Building the product's defining surface as a plugin to somebody
else's shell puts our roadmap behind theirs.

## Since it was accepted

**The engine is started.** This ADR said `alo-engine` was "a decision taken,
but not started and not scheduled". It is now `aloworld-org/alo-browser`: a
Rust engine with its own DOM, cascade, layout, text, paint and agent tree, which
renders `alo-workplace`'s sign-in screen and diffs it against a committed
reference on every run. Its stage 1 needs no operating system and no hardware,
so it does not wait on this repository and this repository does not wait on it —
the incremental swap described above is unchanged, and still incremental.

**The design tokens have not left CSS.** That consequence is real work and had
no line in `ROADMAP.md` and no entry in `docs/features.md` until an audit of
these ADRs found it missing. It is now listed at v0.01, because the shell cannot
draw an alo screen in alo's colours until it exists, and because the longer two
sources of visual truth run in parallel the further apart they drift.

### The default browser, 2026-11-11

**alo's own browser becomes alo OS's default web browser.** The owner, in
their words: *"Send a message to Mac-browser and tell him that that browser
will also be the default browser for alo-os, give him instructions on how it
should be built to be used on alo os"*. *That browser* is `alo-engine`, the
repository `aloworld-org/alo-browser` described in the section above. This is
recorded here rather than in a new ADR because it is the engine this ADR
already describes, reaching one more surface.

Two conditions bind it, set under the constitution when it was recorded:

1. **Firefox stays the shipped default until alo's browser passes the bar
   below, measured on an alo OS image** (law 3: done means the machine still
   works). Until then nothing about the browser a person gets changes.
2. **The person can always choose another default browser, and Firefox is
   never removed from the image** (law 5). This sets a default; it takes no
   choice away.

**The bar.** Every item is measured on an alo OS image, and the report names
the machine it was measured on:

- it installs and starts on a clean alo OS image, with no graphics card, in
  4 GB of memory;
- it opens real sites, including signing in to GitHub and playing a video full
  screen;
- alo's window band, and Minimise, Maximise and Full screen, work on its
  windows;
- downloading and uploading a file go through the file portal;
- the launcher finds it when a person types *browser*;
- a link opened from another application opens in it;
- `alo-app` and alo can open an address in it and read the page, through its
  adapter and its accessibility (AT-SPI) tree;
- the separate copy alo works in, with its own profile, starts apart from the
  person's own, and what it sends is counted on the indicator (pull request
  #1163);
- at idle it sends nothing, measured at the network boundary.

Until every line holds, alo's browser may be installed and chosen by a person
like any other; it is not made the default.

**Owed by alo OS, not by the browser.** *A link opened from another application
opens in it* cannot pass today for any browser: `docs/contracts/portals.md`
answers `OpenURI` (a web link) with `2`, because nothing on this machine yet
decides what opens one. alo OS owes an `OpenURI.OpenURI` answer for `http` and
`https` that opens the person's chosen default browser. It is recorded here as
owed and is not built by the change that wrote this section.

**Its colour and its type** follow the owner's rulings of the same day, quoted
as chosen in a question dialog: *"Deep teal everywhere (Recommended)"* — deep
teal `#0F6B72` means *alo is acting* in the browser as on every alo product,
retiring terracotta for that role; *"Allowed: a logo is a mark"* — the browser's
logo may use teal, since the reservation covers interface states only; and
*"Manrope (Recommended)"* — its interface is set in Manrope. All three are
recorded in `docs/features.md`, *alo's visual language*.

### alo's own applications are built on a shared alo UI kit, 2026-11-11

**alo's own applications draw with the shell's own drawing.** Asked in a
question dialog which road alo's own applications take for their interface —
the first of them alo Mails, from `alo-workplace`, shipped as the Flatpak
`com.alomails.Mails` under AGPL-3.0 — the owner chose *"Build a shared alo UI
kit (Recommended)"*. The option, as it was put: take the drawing, the text
(Manrope, through cosmic-text), the accessibility and the controls alo's
desktop already uses out of the shell and into crates every alo application
builds on; it looks exactly like the system; it is the most work up front; and
alo Mails waits on the kit's first version.

This is recorded here rather than in a new ADR because it is the consequence
above coming due. *The UI runtime built for the shell is the same runtime
`alo-engine` would need* said the shell's drawing was more than the shell's;
this decision gives it its second user before the engine. It does not change
the decision in one line: the shell stays native, and an application on it is
still one program among others.

What binds it:

1. **The desktop keeps drawing exactly what it drew.** Every move out of
   `alo-shell` is held to hashes of the pictures the shell drew before it, so
   a person sees no difference while the kit is being made (law 3).
2. **The kit meets the criteria `alo-workplace` set for its own interface**
   (its ADR 0061, as relayed to this repository): Wayland only, no X11; drawn
   in software with no graphics card
   ([ADR 0104](0104-any-pc-that-works-is-done-and-no-gpu-is-required.md)); a
   complete accessibility (AT-SPI) tree; portals for what a sandbox cannot
   reach; per-display scaling of logical measures; pointer shapes asked for
   by name.
3. **It is GPL-3.0-or-later, like the rest of this repository.** alo Mails is
   AGPL-3.0; section 13 of each licence allows a work under one to be combined
   with a work under the other, so the kit can be linked into alo Mails as it
   is. **Other people may build GPL-compatible applications on it, and closed
   ones may not**: asked on 2026-11-12, the owner chose *"alo's apps and
   open-source apps (GPL) (Recommended)"*, and the kit lives in this
   repository (*"Inside alo-os (Recommended)"*). The plan records all five
   answers.
4. **It is for what alo makes.** An application somebody else wrote keeps its
   own look, and nothing on this machine asks it to change (law 5).
5. **Licence runs the other way too, 2026-11-12.** The kit goes from this
   repository into alo Mails; alo Mails' adapter — its commands for alo and the
   terminal — comes from `alo-workplace` into alo OS. Asked which licence that
   adapter takes and where it lives, the owner chose *"Keep AGPL, separate
   crate alo-os uses"*: it is published by the `alo-workplace` lane as its own
   crate under AGPL-3.0, alo OS pins a tagged release, and **that part of alo OS
   carries AGPL terms**. The contract is `docs/contracts/app-adapters.md`,
   *Amended 2026-11-12*. Not built on the day this was written.

The work is `docs/autonomy/the-alo-ui-kit-plan.md`, and the promise is in
`docs/features.md`, *The shell*. Not built on the day this was written.
