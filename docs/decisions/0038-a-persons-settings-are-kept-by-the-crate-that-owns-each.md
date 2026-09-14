# ADR 0038 — A person's settings are kept by the crate that owns each, in the person's own folder

**Status:** **accepted**, 2026-09-15, option **B**. Task 6 of
`docs/autonomy/v0-5-the-shell-plan.md` (*one place for settings*) no longer
waits on this decision, only on the keeping it describes landing in the crates
it names — which is `docs/autonomy/v0-5-where-a-persons-settings-are-kept-plan.md`,
a lane of its own, since the shell plan may not edit those crates.
**Date:** 2026-09-14, accepted 2026-09-15
**Context:** [ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md)
(*two settings, two owners, two files*; the person's settings under
`$XDG_CONFIG_HOME/alo/`, and *inventing two stores for one owner is how a
settings system becomes six settings systems*),
[ADR 0001](0001-the-capability-model.md) §3 (grants are visible, revocable and
expiring), [ADR 0003](0003-the-network-is-not-authority.md) (a pairing is a
grant across a machine boundary), [ADR 0004](0004-the-organisations-machine.md)
(an organisation sets policy and never acts as the person),
`crates/alo-appearance`, `crates/alo-dock`, `crates/alo-shortcuts`,
`crates/alo-choosing`, `crates/alo-changing`, `crates/alo-remembering`,
`docs/features.md` v0.5 *Settings, as one place* and ★ *one list of what has
been granted to what*.

## The question in one line

**When a person changes their background, moves their dock or rebinds a key in
Settings, which file does the change go into, and which crate writes it?**

## What was true before this decision

The shell plan's task 6 asks for one surface holding every setting the machine
has — setup, the model and provider, appearance, the dock, shortcuts, grants and
pairings — **each section reading and writing the same file its crate already
owns**, and a surface that decides nothing. The worker sent to it on 2026-09-14
found that four of the seven have such a file and three do not:

| Section | Owner of the value | File | Who writes it today |
|---|---|---|---|
| What was chosen at setup | `alo-setting-up` | the person's `settings.toml` | `alo_choosing::Choosing` |
| The model and provider | `alo-choosing` | the person's `settings.toml` | `alo_choosing::Choosing` |
| Grants | `alo-capability`, listed by `alo-granted` | `/var/lib/alo/grants.toml` | `alo_changing::Changing`, then a knock |
| Pairings | `alo-nearby` | `/var/lib/alo/pairings.toml` | `alo-agentd`, on the person's door |
| **Appearance** | `alo-appearance` | **none** | **nobody** |
| **The dock** | `alo-dock` | **none** | **nobody** |
| **Shortcuts** | `alo-shortcuts` | **none** | **nobody** |

The three crates each built the half that matters — a `Changes` value holding
*only what the person changed*, serde-shaped, refusing a hand-edited value in
its own words, resolved against what the release ships by `…::shipped().with(changes)`
— and each stopped at the same sentence. `alo-appearance`'s own header says it
most plainly: *which file it lives in and who writes it is the shell's, and the
shell does not exist yet.* Nothing in the workspace reads or writes one of these
values from a disk; the compositor draws `Appearance::shipped()`,
`Dock::shipped()` and `Shortcuts::shipped()`.

The shell exists now, and its plan says the opposite of what those headers
expected: *every decision these surfaces render is already made in a crate this
plan reads and never edits*, and a decision that is not there *is a finding in
the report and the task stays open*. Where a person's settings are kept is not
a drawing decision, and ADR 0016 already says something about it — so it is
decided here rather than in a compositor.

A second, smaller gap sits in the same task. `docs/features.md` promises grants
and pairings **in one list, revoked the same way**. A grant has a person's road
— `alo_changing::Changing::revoked` takes an `alo_granted::Seen`, writes the
file whole and knocks. A pairing has the daemon's door (`revoke-pairing` in
`docs/contracts/daemon-protocol.md`) and **no crate on the person's side that
speaks it**; the only callers of `FromAPerson::RevokePairing` are tests.

## The options

### A. Everything goes into the person's `settings.toml`

One file for one owner, which is ADR 0016's *one store* argument taken
literally. `alo-choosing` would gain `appearance`, `dock` and `shortcuts` keys
and depend on the three crates.

- **ADR 0016 says the opposite about this file**: *nothing else goes in it*,
  because an administrator reading it is reading a person's choice of model and
  nothing more.
- The file is read by `alo-agentd`, with `deny_unknown_fields` and a format
  number. Every new appearance key would be a daemon format change, and a
  privileged service that answers agents would parse wallpaper paths it has no
  use for.
- `alo-choosing` would gain three more reasons to change — Law 4.

### B. Each crate keeps its own file, in the person's folder (recommended)

ADR 0016's table names the person's store as a **folder**,
`$XDG_CONFIG_HOME/alo/`, and the choice of model is one file in it. The other
three become files beside it:

| File in the person's folder | Kept by | Holds |
|---|---|---|
| `settings.toml` | `alo-choosing` | unchanged |
| `appearance.toml` | `alo-appearance` | `alo_appearance::Changes` |
| `dock.toml` | `alo-dock` | `alo_dock::Changes` |
| `shortcuts.toml` | `alo-shortcuts` | `alo_shortcuts::Changes` |

One store, one owner, one folder — and one writer per file, which is the crate
that already declares the shape.

### C. The shell keeps one file for the desktop

What the three crate headers anticipated. `alo-shell` would serialise the three
`Changes` into a file of its own.

- A drawing crate would be the only writer of three crates' shapes, which is the
  plan's forbidden decision and a second declaration of each shape.
- `docs/features.md` v0.5 promises portals for *wallpaper* and *settings*, and a
  workspace reaching appearance through a portal would have to read a
  compositor's private file.

### D. A new crate that keeps all three

Tidier than C and still a crate with three reasons to change. It saves three
small modules and costs the property that the crate declaring a shape is the
crate that reads and writes it.

## Recommendation

**B.** In detail:

1. **The folder is `alo-choosing`'s rule, the file name is each crate's.**
   `alo_choosing::where_it_is` already works out the folder from
   `$XDG_CONFIG_HOME` and `$HOME` without reading the environment. It gains the
   folder on its own; each of the three crates gains a constant naming its file
   and a way to read and write **at a path it is handed**, so none of them
   depends on `alo-choosing` or reads an environment.
2. **Only the difference is written**, which is what `Changes` already is, under
   a `format` number of its own per file. An untouched machine has no file, and
   no file means the person has changed nothing — never an error.
3. **A file that is there and wrong is refused whole**, in the owning crate's
   words (`alo_appearance::NotRead` already carries the key), and nothing in it
   is honoured. The machine draws what the release ships and Settings says, in
   that section, that the file did not read. **Settings does not write over a
   file that did not read** except when the person puts that section back as
   shipped, which is a deliberate act (`put_everything_back` already exists) —
   so a hand-edit with a typo is never lost silently to the next click.
4. **Written whole or not at all, and read back before it counts**, the way
   `alo_choosing::Choosing` writes: to a sibling file, renamed over the old one,
   and refused unless the text written reads back as the same `Changes`.
5. **The change is applied after the write answers**, in `alo-changing`'s order:
   a copy is changed, the copy is kept, and only then does it replace what the
   compositor draws from. A write that fails leaves the screen, the file and
   the running session as they were.
6. **Nothing watches the folder.** Settings and the compositor are one process,
   so a change made in Settings is drawn at once; a file edited by hand is read
   at the next sign-in. A background reader of the person's folder would be a
   mechanism nobody asked for.
7. **No organisation bound over these, now.** ADR 0004 gives policy to an
   organisation and ADR 0016 shows the shape a bound would take — a separate
   file in `/etc/alo/`, never a key in the person's. Whether an organisation
   may bound a lock screen or a shortcut is not decided here and nothing in B
   makes it harder.
8. **Pairings are revoked the way grants are: through `alo-changing`.** It is
   already *the person's half of a change*. It gains a pairing's revocation over
   the daemon's `revoke-pairing` door, answered as the same `Gone` a grant's
   revocation is, and the pairings a surface lists come from the daemon's
   `pairings` answer or `alo_remembering::pairings_remembered` — never from a
   second writer of `/var/lib/alo/pairings.toml`, which stays the daemon's.
   A surface then revokes a row of the one list with one call whichever kind of
   row it is, and *revoked the same way* is one crate's shape rather than two
   code paths in a compositor.

## What it costs, and what follows

- `alo-appearance`, `alo-dock` and `alo-shortcuts` each gain a keeping module,
  a file-name constant, a `format` number, `toml` as a dependency, and the
  refusal words for a file that did not read or was not written, collected by
  `alo-saying`. `alo-choosing` gains the folder on its own. `alo-changing` gains
  a pairing's revocation. None of those is a crate the shell plan may edit, so
  that work is a lane of its own — proposed in the report that accompanies this
  decision.
- `docs/contracts/person-settings.md` gains the three files, since a person may
  edit them and a portal will read them; three contract sections rather than a
  new contract.
- The shell's task 6 then becomes wiring: every section reads and writes through
  the crate that owns it, and the shell still names no file, serialises nothing
  and writes nothing to a disk itself — which
  `crates/alo-shell/tests/settings_source.rs` holds today, so option C cannot
  arrive by accident while this waits.
- A person's desktop settings survive a sign-out for the first time. Until this
  lands, a changed background would be forgotten at the next sign-in, which is
  why task 6 does not draw a control that changes one.

## Rejected, if B is accepted

- **A** — the person's model file stays about the model, as ADR 0016 says.
- **C** — the compositor draws settings; it does not keep them.
- **D** — one crate per shape, not one crate per folder.
- **Greyed-out sections until then** — the shell plan's acceptance already
  rejects a dialogue full of disabled controls, and a section that shows a value
  it cannot keep would be one.
