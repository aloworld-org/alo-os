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
