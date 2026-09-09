# One keyring fixture, two crates

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-keyring-fixture`)
- Contributor: Claude Code
- Task: The real-keyring fixture made shareable, without changing what it does
- Status: **moved, unchanged, and its original tests rerun before anything new
  was written.** Authenticated HTTPS through the daemon is the next task and is
  not in this change.

## Why it moved

`alo-secrets` owns the credential store; `alo-agentd` asks it for a provider's
key. Both need a **real** keyring to ask questions of, and the fixture lived in
`alo-secrets/tests/`, where only that crate could reach it.

The alternative was writing it twice. Two fixtures for one thing drift, and the
one that drifts is the one whose tests still pass.

So `crates/alo-keyring-fixture`, reached **only through `dev-dependencies`**,
sharing exactly what these two crates need and nothing more. It is not a testing
framework and has no ambition to become one.

## Moved, not rewritten

`git mv`, so the history follows the file. What changed is only what a library
requires rather than a test module: `#![cfg(target_os = "linux")]` in place of
the `dead_code` allowance, and a header saying who uses it now.

**Every isolation property is exactly as it was**, because none of it was
touched: its own `dbus-daemon` on a private socket, `XDG_DATA_DIRS` pointed at an
empty directory so the bus can activate nothing, its own `XDG_DATA_HOME` and
`XDG_RUNTIME_DIR`, a `0700` control directory, `--components=secrets`, a
synthetic password, and **never `--replace`**. Cleanup still kills only the two
processes it started and removes only the directory it made.

The original tests were rerun before a line of new work was written:
**8 real-keyring tests and 9 unit tests, all passing, unchanged.** That is the
whole evidence that this was a move and not a rewrite.

## Keeping it out of the image, as a check

The fixture starts a keyring daemon with a password written in its own source.
Correct for a test, a hole in a shipped image — so *it is only ever a
dev-dependency* is now asserted rather than remembered.

`nothing_ships_the_fixture.rs` holds up both halves:

- **no manifest names it outside a `dev-dependencies` table**, so it cannot
  become a real dependency by somebody moving one line;
- **`image/Containerfile` still builds `--package alo-agentd --package
  alo-boundaryd`**, and a `--package` release build compiles no dev-dependency
  of those packages. If that ever became `--workspace`, every dev-dependency in
  the repository would be compiled into the build that produces the image.

Both tests failed on first run, and both failures were **mine, not findings**:
the fixture's own manifest names itself under `[package]`, the workspace root
lists it as a member, and the `--workspace` the Containerfile check tripped on
was inside the comment explaining why it is not used. A check that reads
comments fails on the sentence saying it is right.

## The supervisor could not publish a move at all

Trying to publish this found a real gap in `tools/kernel-loop`.

Porcelain writes a rename as **one line holding two paths** — `old -> new`. The
cleanliness check compared that whole pair against the task's file list, so it
matched nothing however carefully the task named its files, and reported the
move as changed-but-unnamed. **A task that moved a file could not publish,
ever.** It had never come up because no task had moved one.

`accounted_for` now reads a rename as what it is and requires **both** ends to
be named. That is the honest rule rather than the convenient one: a move deletes
a path and creates another, a reader of the commit needs to see both, and naming
only the destination would let a file disappear from a crate without the task
that did it saying so.

Staging needed the matching change. `git add -- <path>` on the end that no
longer exists is a pathspec matching nothing, so it is now `git add --all --
<path>` — still one named path at a time, never across the tree.

Three tests pin it: both ends named is accounted for, either end alone is
refused, and a path that merely *contains* the arrow is still matched whole.
Nothing in this repository is named like that, which is exactly why it is worth
pinning.

## What is unchanged

The production credential architecture, entirely. `TheBus` is still derived from
a uid, `TheKeyring` still opens on the connection this crate made, the session is
still `EncryptionType::Dh`, and the four refusals are still four. The fixture
builds its `TheBus` through the **same checked constructor** production uses, so
a test cannot be handed a bus production would refuse.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — credential store: fixture shared; authenticated
HTTPS through the daemon next.

**docs/autonomy/STATE.md** — the real-keyring fixture is now
`crates/alo-keyring-fixture`, dev-dependency only, moved without behavioural
change and with its original tests rerun; two tests assert it can never reach a
shipped image.
