# Native folder selection, so a grant can be made at all

**Date:** 2026-09-10
**Workstream:** v0.01 delivery plan, task 6 (`docs/autonomy/v0-01-delivery-plan.md`)
**Contributor:** Claude, in `C:\dev\alo-os-claude`
**Status:** ready for integration. Gated on Windows and on Linux (WSL); no
machine evidence is claimed and none is owed by this task.

## What changed, in words a person outside this repository can read

alo OS can now be given a folder. Until this change nothing on the machine
could make a grant at all: `alo-capability` had held grants since the
beginning and `alo-agentd` had enforced them, and every verb an agent asked
for was refused — correctly — because nothing had ever been granted and
nothing could be. ADR 0001 §3 says a grant comes from *a folder chosen in a
picker*, and there was no picker.

`crates/alo-picking` is that picker, as a decision rather than as pixels. A
person opens it where they are, sees the folders in front of them, opens one,
and picks it. What comes out is a grant over **that** folder — the one they
picked, never the one they walked through — lasting the time it was given, on
the same list the daemon asks before it touches a file. Closing the picker
without picking grants nothing, and the top of the disk cannot be picked at
all: alo OS has no grant to the whole machine, and the person is told so in a
sentence rather than by a button that does nothing.

## The acceptance, and where each half is measured

`crates/alo-picking/tests/a_person_picks_a_folder.rs` walks a **real
filesystem** through `OnThisDisk`, and ends every criterion on
`alo_capability::Grants::permits` — the question `alo-agentd` asks of every
verb — rather than on a second opinion this crate holds about its own work.

| Acceptance | Test |
|---|---|
| a person picks a folder and a grant exists afterwards that the daemon honours | `a_person_picks_a_folder_and_the_daemon_honours_the_grant` |
| picking nothing grants nothing | `picking_nothing_grants_nothing` |
| the grant's scope is the folder picked rather than its parent | `the_grant_is_over_the_folder_picked_and_not_the_one_it_was_picked_from` |

The refusal paths beside them, in the same file: the top of the disk refused
in the machine's own words with nothing granted by any of it; a name that is
not one of the rows — `..`, `.`, a file, a path, a different capitalisation —
opening nothing and moving the picker nowhere; a file refused as not a folder
and a missing folder refused as gone, which are two different sentences
because they send a person to two different actions.

## What is in the crate, one file at a time

| File | What it decides |
|---|---|
| `folders.rs` | The `Folders` port, the `Inside` listing, and the three ways a folder is not shown |
| `on_this_disk.rs` | The only code here that touches a filesystem |
| `browsing.rs` | `Picker`: where a person stands, and the three moves — in, up, pick |
| `picked.rs` | `Picked` (sealed) and `Chosen`, the picker's two endings |
| `granting.rs` | `Granting`: the pick, become a grant on the machine's list |
| `refusing.rs` | `NotPicked`, and the sentence for each of the seven |
| `words.rs` | Eleven strings under the `picking` area, each with a translator's note |

54 unit tests on Windows, 55 on Linux (one is a symbolic-link test that needs a
POSIX link), 8 integration tests, and the crate-level example.

## Decisions taken here, and why

Nobody was waiting to answer these; each is written down so the next worker can
disagree with a reason rather than a guess.

**A new crate rather than a module of `alo-shell`.** The picker is a decision —
what may be shown, opened, picked, and what a person is told when any of it is
refused — and it is the *only* road to a grant. Inside the compositor it would
be untestable without a screen, unreachable by the daemon-side tests that
matter, and in the desktop worker's live chain, which task 2's constraint keeps
this lane out of. Nothing in `crates/alo-shell` was touched.

**The picker walks; it does not jump.** There is no method that takes a path
except the one that opens the picker, and `go_into` accepts only a name that is
currently being shown, matched exactly. A person therefore cannot be shown — or
made to grant — a folder they did not navigate to, whether by a shell's bug or
by a shell's design. `Inside::these` refuses any "name" that is not exactly one
ordinary path component, so `..`, `/etc` and `a/b` are not names an
implementation of the port can smuggle in.

**Standing at the top of the disk is allowed; picking it is refused.** Going up
from `/home` reaches `/`, because a person navigating upwards should not meet an
invisible floor and be unable to reach a sibling folder. Picking there is
`NotPicked::TheWholeMachine`, in a sentence a person reads. `alo-capability`
refuses it too, and the duplication is deliberate: this one is what a person
reads, that one is what makes it true.

**A symbolic link is not a folder here.** `DirEntry::file_type` is asked, which
does not follow a link, so a link to a folder is neither listed nor stood in. A
grant made over a link would name one place and cover another — reach is decided
lexically by `alo-capability` and resolved by `alo-files` at the moment a verb
runs — and the link can be repointed afterwards by anything that can write the
folder it sits in, turning Monday's grant into a grant over somewhere else on
Tuesday. Somebody who wants the folder a link points at picks the folder.

**`Picked` is sealed and `Chosen::Nothing` is a value.** There is no
constructor for a picked folder outside this crate, so *nothing but a person's
pick can become a grant* is held by the compiler rather than by everybody
remembering. *Picking nothing grants nothing* is likewise not a branch somebody
wrote: `Chosen::Nothing` holds no folder for anything to grant.

**Nothing here is hidden and nothing here is dropped silently.** Dot-folders are
shown, because deciding a row is too technical for a person is a rendering
decision and not this crate's. A listing is bounded at 1000 rows in the one
place every implementation passes through, and a folder with more says so
(`picking.more-than-shown`); entries the machine cannot name or describe are
counted and reachable as `Picker::could_not_be_named` rather than forgotten.

**`Granting` carries the agent and the duration together.** A grant cannot be
made without an end (`alo-capability` refuses zero and refuses no-end), and a
call site that could pass the agent without the duration is a call site that can
pass yesterday's agent with today's duration.

## What this is not, and what is still owed

- **It does not draw.** No size, position, row height or colour; what the picker
  looks like is the compositor's, exactly as `alo_overlay::SurfaceRequest`
  leaves the overlay's appearance to whoever owns the screen. Wiring the picker
  to a surface is desktop work and belongs with the compositor lane.
- **It is not reachable by an agent**, and there is no verb for it.
  `alo-protocol` has nothing that reaches this crate, and a chooser an agent
  could drive would be an agent granting itself a folder.
- **It does not keep the grants.** `Granting::of` adds to a `Grants` it is
  handed. Where a machine's list lives between one sign-in and the next is still
  open, and `crates/alo-agentd/src/starting.rs` now says exactly that: a person
  can make a grant, nothing on the daemon's socket can, and nothing yet carries
  one made in the shell into the daemon's process. That wiring is the natural
  neighbour of plan task 7 and is not claimed here.
- **It is not the portal file chooser**, which `docs/features.md` puts at v0.5.
  This is alo OS's own surface for granting the agent a folder, which is the
  v0.01 promise *grants: pick a folder, see what is granted, revoke it, and it
  expires*. The *see what is granted* half is `alo_overlay::Granted`'s count
  today; a readable, revocable list on a screen is not this task.

## Files touched

- `crates/alo-picking/` — new crate: `Cargo.toml`, seven source files, one
  test fixture module, one integration test.
- `Cargo.toml` — the crate is a workspace member.
- `crates/alo-saying/Cargo.toml`, `crates/alo-saying/src/collecting.rs` — the
  eighteenth list in the machine's one vocabulary. Without this every sentence
  the picker can say would reach a real shell as a key in guillemets, which is
  the bug task 3 found for `alo-overlay`.
- `crates/alo-agentd/src/starting.rs` — documentation only: the paragraph
  saying nothing on this machine can make a grant is now true of the socket
  rather than of the machine.
- `docs/autonomy/v0-01-delivery-plan.md` — task 6 marked done; task 7 was
  already written.

## Verification actually run

Windows 11 (host), `cargo 1.97.1`:

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets` — zero warnings, zero errors.
- `cargo test --workspace` — all suites pass, including the new crate's 54 unit
  tests, 8 integration tests and the crate example.
- `cargo doc --workspace --no-deps` — no new warnings; the pre-existing
  Linux-only `broken_intra_doc_links` warnings from `alo-agentd` and
  `alo-bounding` are unchanged, and `cargo doc -p alo-picking --no-deps` is
  silent.
- Each acceptance test run on its own with `--exact`: one test passing each.

Ubuntu under WSL, target `/root/target-claude`:

- `cargo test -p alo-picking` — 55 unit tests (the symbolic-link refusal runs
  here), 8 integration tests, the example.
- `cargo clippy -p alo-picking -p alo-saying -p alo-agentd --all-targets` —
  clean, which is where the `alo-agentd` documentation change is checked at all,
  that crate being Linux-only.
- `cargo doc -p alo-agentd -p alo-picking --no-deps` — no warnings.

Not run and not claimed: anything on real hardware, anything in a VM, and any
compositor. No box in `ROADMAP.md`'s *On the machine* column is touched by this
work.

## Proposed shared-document updates

For the integration owner (`SHARED_MAIN.md` — this contributor does not edit
these four files):

- **`CHANGELOG.md`** — *Native folder selection. A person can now grant the
  agent a folder: alo OS's own picker walks the machine's folders, and the
  grant it makes covers the folder that was picked and nothing above it.
  Closing the picker grants nothing, and the whole machine can never be
  granted. `crates/alo-picking`.*
- **`ROADMAP.md`** — the code half of *Grants: pick a folder, see what is
  granted, revoke it, and it expires* now has its picking half in
  `crates/alo-picking`; the list a person reads and revokes from is still
  owed. No machine half moves.
- **`docs/autonomy/QUEUE.md`** — nothing new is blocked. The open question this
  names is where a machine's grants are kept between sign-ins, which was
  already an item of its own.
- **`docs/autonomy/STATE.md`** — reference this report, and that the plan's task
  6 is marked done in `docs/autonomy/v0-01-delivery-plan.md` with task 7 next.
