# One place for settings waits on where a person's settings are kept

**Date:** 2026-09-14
**Workstream:** v0.5 — the shell (`docs/autonomy/v0-5-the-shell-plan.md`, task 6)
**Responsible contributor:** Claude worker in `C:\dev\alo-os-shell`, for the owner
**Status:** ready for integration — **the decision, not the surface.** The
settings surface is not built. Task 6 is marked `blocked` in the plan on
ADR 0038 (proposed) and on the keeping it describes, which lives in crates the
shell plan may not edit.

## What changed

| File | What it is |
|---|---|
| `docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md` | The decision task 6 needs: where appearance, the dock and shortcuts are kept, who writes them, and how a pairing is revoked the way a grant is. Proposed. |
| `crates/alo-shell/tests/settings_source.rs` | Holds, from the shipped source, that the shell keeps no settings file of its own while the decision waits — and that the check catches one that would. |
| `docs/autonomy/v0-5-the-shell-plan.md` | Task 6 marked `blocked`, naming ADR 0038 and this report. Its acceptance is unchanged. |
| `crates/alo-bounding/src/cgroup.rs` | Second worker: removing a control group waits, briefly and only on `EBUSY`, for a thread still leaving it — the teardown failure the workspace gate refused this change on. |
| `docs/quirks.md` | The existing entry on that failure, with what refused this task and what changed. |

**User-readable change description:** *Nothing a person can see changes yet.
Settings, as one place, needs somewhere to keep a changed background, dock
position and keyboard shortcut — and alo OS had never decided where. A proposed
decision now says: each in its own small file in the person's own settings
folder, written only by the part of alo OS that understands it, and never
overwritten silently when the file was edited by hand. Settings will be built
on that once it is accepted.*

## What the worker found

The acceptance asks that one surface hold seven sections, **each reading and
writing the same file its crate already owns**, and that the surface decide
nothing. Read crate by crate:

| Section | File | Writer | Ready? |
|---|---|---|---|
| What was chosen at setup (`alo-setting-up`) | person's `settings.toml` | `alo_choosing::Choosing` | yes |
| Model and provider (`alo-choosing`) | person's `settings.toml` | `alo_choosing::Choosing` | yes |
| Grants (`alo-granted`) | `/var/lib/alo/grants.toml` | `alo_changing::Changing` (write, then knock) | yes |
| Pairings (`alo-nearby`) | `/var/lib/alo/pairings.toml` | `alo-agentd`, on the person's door | **no person-side road** |
| Appearance (`alo-appearance`) | **none** | **nobody** | **no** |
| Dock (`alo-dock`) | **none** | **nobody** | **no** |
| Shortcuts (`alo-shortcuts`) | **none** | **nobody** | **no** |

- **Three sections have no file.** `alo-appearance`, `alo-dock` and
  `alo-shortcuts` each hold a serde `Changes` and stop there;
  `alo-appearance`'s header says *which file it lives in and who writes it is
  the shell's*. No crate in the workspace reads or writes one — the compositor
  draws `…::shipped()` for all three (checked by searching every crate for a
  reader or writer of the three `Changes`). The shell plan says the opposite of
  that header: a decision a surface needs that is not in a crate *is a finding
  and the task stays open*, never made in a drawing crate. Where a person's
  settings are kept also touches ADR 0016, which names the person's store and
  says what may go into `settings.toml`. That makes it an ADR.
- **A pairing cannot be revoked the way a grant is.** `docs/features.md` ★
  promises one list, *revoked the same way*. A grant is revoked through
  `alo_changing::Changing::revoked`; a pairing only through the daemon's
  `revoke-pairing` wire message, which no crate on the person's side sends
  outside tests. Writing that client into the compositor is the order-sensitive
  glue `alo-changing` was made to hold instead.
- **Building the four ready sections and handing them over as task 6** would
  be a partial task marked done: the acceptance names all seven and says a
  setting the machine does not have is *absent*, and appearance, the dock and
  shortcuts are settings the machine has — only not kept. Splitting them into a
  new task, the way task 5 split out the clock, would narrow what *one place
  for settings* means in the plan without anybody deciding it should. So the
  decision is the work, as the worker's instructions say.

## Decisions, and why

- **An ADR, recommending that each crate keeps its own file in the person's
  folder** (`appearance.toml`, `dock.toml`, `shortcuts.toml` beside
  `settings.toml` under `$XDG_CONFIG_HOME/alo/`). ADR 0016's table names the
  person's store as that folder, so one owner still has one store; the crate
  declaring a shape is the one that reads and writes it (Law 4); the daemon
  never parses wallpaper paths; and the v0.5 portals can reach the same files
  through the same crates. The rejected options — everything in
  `settings.toml`, a shell-owned file, a new aggregating crate — and the rules
  for a missing file, a broken file, writing whole, applying only after the
  write, no folder watcher and no organisation bound yet are in the ADR.
- **Pairings revoked through `alo-changing`**, answered as the same `Gone` a
  grant's revocation is, so a surface revokes any row of the one list with one
  call. Put in the same ADR because it is the same question — what a settings
  surface writes through — and task 6 needs both answers.
- **A source test in the shell rather than nothing.** The rejected option C —
  the compositor serialising a `Changes` into a file of its own — is the easy
  thing for the next person to build while the decision waits, so
  `tests/settings_source.rs` makes it a failing build: no shipped file in
  `src/` names `serde`/`toml`, writes a file's contents (`fs::write`,
  `File::create`, `OpenOptions`, `fs::rename`, `fs::copy`) or names a settings
  location (`XDG_CONFIG_HOME`, `THE_SETTINGS`, `.toml`, `.config/`), and the
  shipped dependencies include no `serde`, `serde_json`, `toml` or `toml_edit`.
  It stays true after ADR 0038 lands as recommended, because the files are
  then the owning crates'. It was written against the tree as it is — the shell
  writes nothing today except removing its own socket lock, which the test
  deliberately allows.
- **The plan's acceptance is not edited.** Only the status line moves, to
  `blocked`, so the loop steps over the task rather than selecting it again.
  Task 7, the next in the plan, was already written and stays blocked on its
  own crates.

## Acceptance criteria and evidence

| Criterion (plan, task 6) | Where it stands |
|---|---|
| One surface holds every setting the machine has today | **Not built** — waits on ADR 0038 and the keeping lane below |
| Each section reads and writes the same file its crate already owns, with a test per section | **Not built** — three sections have no file (ADR 0038 §B) |
| Grants and pairings in one list, revoked the same way | **Not built** — pairings have no person's road (ADR 0038, recommendation 8) |
| A setting the machine does not have yet is absent, not greyed out | **Not built** |
| Constraint: this surface decides nothing; every value goes through the crate that owns it | Held for the shell as it is today: `alo-shell settings_source the_shell_keeps_no_settings_file_of_its_own`; `alo-shell settings_source the_shell_is_built_with_nothing_that_writes_a_settings_file` |
| — and the refusal path of that check | `alo-shell settings_source a_shell_file_that_kept_a_setting_is_refused`; `alo-shell settings_source a_settings_writer_among_the_shipped_dependencies_is_refused` |

The refusal tests plant each road — `toml::to_string`, `use serde`,
`#[derive(Serialize)]`, `fs::write`, `File::create`, `OpenOptions`,
`fs::rename`, `XDG_CONFIG_HOME` in a string, an `appearance.toml` path, a
`.config/alo` path, `THE_SETTINGS` — and require each to be caught on its line,
and plant what only looks like one — a comment, a trailing comment, a longer
identifier, an unrelated string, `XDG_RUNTIME_DIR`, the socket lock's
`remove_file`, a write inside a `#[cfg(test)]` module — and require none to be.
A string spanning two lines is followed to the line the write is really on. A
manifest with `toml` in the shipped table is caught, and `serde_json` in the
dev-dependencies is not.

## Verification

Executed on 2026-09-14, Windows 11 host, Ubuntu under WSL 2,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`. The
distribution stopped answering (`Wsl/Service/0x8007274c`) for some minutes
before the first run. It was waited for, not restarted.

- `cargo fmt --all -- --check` — clean.
- `cargo clippy -p alo-shell --all-targets -- -D warnings` — exit 0, no
  warnings.
- `cargo test -p alo-shell` — exit 0: lib 317, `approval_source` 5,
  `client_lifecycle` 262, `desktop_source` 4, `egress_status_source` 3,
  `record_source` 5, `settings_source` 4 (new), `sign_in_source` 4,
  `socket_ownership` 3, doc tests passed; 0 failed.
- `cargo test -p alo-citing` — exit 0 (21 and 10 passed). ADR 0038's status line
  and every citation of it resolve.
- Each of the four evidence tests run on its own with `--exact` — 1 passed each.
- **Mutation:** `const _SETTINGS_PROBE: &str = "appearance.toml";` added to
  `src/cursor.rs` made `the_shell_keeps_no_settings_file_of_its_own` fail with
  `src/cursor.rs:69 names .toml`. The line was removed and the test passes
  again.

Not run: the full workspace suite (the supervisor runs it). No surface was
drawn, so nothing was measured under WSLg.

## Limitations

- **No settings surface exists.** A person still cannot change a background,
  move the dock, rebind a key, see their grants and pairings in one list, or
  change their model from the shell.
- **ADR 0038 is proposed, not accepted.** If the owner chooses another option,
  `tests/settings_source.rs` may need to follow it (option C would retire it).

## The second worker: why the gates refused this, and what fixed it

The supervisor's workspace gate refused this handoff twice, both times on a
test in a crate this task never touched: `alo-bounding`'s
`a_turn_without_a_boundary_does_not_run::a_turn_runs_where_the_boundary_is_in_place`,
panicking in the fixture's teardown — *cannot take away the control group at
…/home: Device or resource busy*. So the work was not wrong, but it could not be
published, and the fix is in the tree the gate runs.

**What was happening.** `docs/quirks.md` already recorded this flake on the Mac
lane ("A boundary fixture takes a control group away while its process is still
leaving") and left it for the crate's owner. The group from the refused run was
still on the machine, reading `populated 0` with nothing in it. The in-place
turn is the only one in that file that reaches `Turns::doing`, whose keeper
thread ends inside `home`. Moving the process out through `cgroup.procs` leaves
behind a thread that has begun to exit. That thread counts, so `rmdir` answers
`EBUSY`, until it is gone. It is also still listed in `cgroup.threads`, so the
list cannot be used to tell it from a live thread.

**The change** is in `crates/alo-bounding/src/cgroup.rs`, not in the test.
Every group the service takes away goes through `Cgroup::removed`: `home` in
`Turns::given_back`, and each turn's group in `Turns::doing`. On `EBUSY`, and
only then, it now waits up to five seconds for `cgroup.events` to say
`populated 0`, which is the count `rmdir` checks, and asks once more. A group
that does not empty, and every refusal that is not `EBUSY`, is reported with the
kernel's first answer, exactly as before. Nothing is removed that the kernel
would not remove, and no refusal becomes a success.

**Decided rather than asked:**
- *In the crate, not the fixture.* `alo-agentd` removes `home` the same way when
  it stops, and a daemon that failed to stop cleanly on a loaded machine would
  be the same bug outside a test.
- *Wait on `populated`, not on the thread list.* A first draft waited only when
  `cgroup.threads` was empty. That would have refused exactly the case it was
  written for, because a thread that has begun to exit is still listed.
- *Five seconds.* Much longer than a thread takes to exit, and short enough that
  a group which really is held is still reported quickly.

**Tests.** There are new unit tests beside the code. The refusal paths are that
a group that stays populated is refused once the wait is over (and not before),
a group whose events file cannot be read is not waited on, and a refusal that
is not `EBUSY` is not waited on. The legitimate paths are that `EBUSY` is waited
for and an empty group ends the wait at once. Whole `alo-bounding` suite, on the
real kernel in WSL (root, BPF LSM, 6.18.33.2): 23 test binaries, all passing,
including the refused one.

**What was not shown.** The flake did not reproduce on demand. The refused test
passed alone three times without the change. A std-only probe ran the fixture's
sequence 400 times, idle and beside eight spinning processes: threaded `home`, a
thread joined inside it, the process moved out, `rmdir` straight away. It never
saw `EBUSY`. The explanation comes from the kernel's migration path, the quirk's
two earlier failures and the empty group left behind, not from a reproduction.
The supervisor's workspace run, under the load that produced it, is the
measurement that settles it.

**Debris left on the machine, untouched:** `/sys/fs/cgroup/alo-in-place-in-place-649482`
and `/sys/fs/cgroup/alo-two-places-201326`, both empty and left by earlier
processes. Their names include those processes' ids, so they block nothing.

## Proposed changes to shared documents

**CHANGELOG.md** — nothing a person can see; optionally: *A decision is proposed
for where a person's appearance, dock and keyboard settings are kept (ADR 0038),
which Settings waits on.*

**ROADMAP.md** — under v0.5 *Settings, as one place*: unchanged, not ticked.

**docs/autonomy/QUEUE.md** — the shell plan's task 6 blocked on ADR 0038.
Proposed, once ADR 0038 is accepted, a **keeping lane** outside the shell plan:
1. `alo-choosing` answers the person's folder on its own, beside `where_it_is`.
2. `alo-appearance`, `alo-dock` and `alo-shortcuts` each gain a keeping module:
   a file-name constant, a `format` number, read at a handed path (missing is
   untouched, broken is refused whole in the crate's words), written whole and
   read back, with the refusal words collected by `alo-saying`.
3. `alo-changing` gains a pairing's revocation over `revoke-pairing`, answered
   as `Gone`.
4. `docs/contracts/person-settings.md` gains the three files.
Then task 6 is unblocked and is wiring.

**docs/autonomy/STATE.md** — reference this report and ADR 0038.
