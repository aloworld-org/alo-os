# v0.5 — where a person's settings are kept

**Workstream:** the half of `ROADMAP.md`'s *Settings, as one place* that is not
a screen. On 2026-09-14 the shell lane's worker was sent to draw seven sections
and found that **three of them have nowhere to keep anything**: `alo-appearance`,
`alo-dock` and `alo-shortcuts` each declare what a person changed and stop, and
every one of their headers says *which file it lives in and who writes it is
the shell's*. Nothing in the workspace reads or writes one. A changed
background is forgotten at the next sign-in.
**Why it exists:** the shell plan may not edit those crates, which is why its
task 6 is blocked rather than half-built. This plan is the crates' half, and it
is the whole reason task 6 can be finished afterwards.

**The decision it implements** is
[ADR 0038](../decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md):
each crate keeps its own file in the person's folder, one writer per file,
written whole and read back before it counts, a file that is there and wrong
refused whole in the owning crate's words, and no file at all meaning *the
person has changed nothing* rather than an error. Read it before the first
task; the tasks below implement it and do not re-argue it.

**Crates this plan owns:** `crates/alo-appearance`, `crates/alo-dock`,
`crates/alo-shortcuts`, `crates/alo-choosing` and `crates/alo-changing`. **It
reads and never edits** `alo-capability`, `alo-granted`, `alo-nearby`,
`alo-agentd` (the `revoke-pairing` door already exists in
`docs/contracts/daemon-protocol.md` — this plan *speaks* it and does not change
it), `alo-saying` and `alo-strings`. Nothing in `crates/alo-shell`: the
surface is the shell plan's and what this owes it is a road to call.

**What this plan may not do:** tick anything *on the machine*; write a second
writer of `/var/lib/alo/pairings.toml`, which stays the daemon's; put anything
new into the person's `settings.toml`, which [ADR 0016](../decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
says is about the model and nothing else; make any of these crates depend on
`alo-choosing` or read an environment variable of its own — a crate is handed
the path it keeps its file at; or decide what an organisation may bound, which
is ADR 0004's and is deliberately left open. Before writing the next task,
`git pull` and read the plan as published.

## Tasks

### 1. What keeping one of these files means

**Status:** **Done, 2026-09-15** — `crates/alo-kept` holds the rule in type and
`alo_choosing::where_the_folder_is` hands out the folder; see
`docs/autonomy/updates/how-a-file-in-a-persons-folder-is-kept.md`. **Depends on:** nothing.

Four small files written by four different crates will go wrong in the same
four ways, and a rule invented separately in each is four chances to get it
wrong. The rule is decided once, here, and the crates in task 2 follow it.

- **Acceptance:** `alo-choosing` gains the person's **folder** on its own —
  `where_it_is` already works out `$XDG_CONFIG_HOME` and `$HOME` without
  reading the environment, and this exposes the folder beside the file it
  already names, so no other crate reads an environment or depends on this one;
  and the keeping rule is carried in type rather than in habit, with a test per
  clause: **only the difference is written** (which is what each crate's
  `Changes` already is) under a `format` number of its own per file;
  **no file means the person has changed nothing** and is never an error;
  **a file that is there and wrong is refused whole** in the owning crate's own
  words and nothing in it is honoured; and a write is **whole or not at all and
  read back before it counts** — to a sibling file, renamed over the old, and
  refused unless the text written reads back as the same value, the way
  `alo_choosing::Choosing` already writes.
- **Constraint:** nothing here reads or writes appearance, the dock or
  shortcuts — it decides how, and task 2 does it. Nothing new goes into
  `settings.toml`. No crate gains a dependency on `alo-choosing` for this: the
  folder is handed to a caller, and each crate is handed the path it keeps at.

### 2. Appearance, the dock and shortcuts, each keeping its own

**Status:** **Done, 2026-09-15** — `alo_appearance::keeping`, `alo_dock::keeping`
and `alo_shortcuts::keeping` read and write their own file at a path they are
handed; see `docs/autonomy/updates/appearance-dock-and-shortcuts-keep-their-own-files.md`.
**Depends on:** 1.

Three crates, one shape each, one file each. The header sentence every one of
them carries — *who writes it is the shell's* — stops being true here, and each
crate becomes the reader and writer of the shape it declares.

- **Acceptance:** `alo-appearance`, `alo-dock` and `alo-shortcuts` each gain a
  keeping module, a constant naming its file (`appearance.toml`, `dock.toml`,
  `shortcuts.toml`), a `format` number, and reading and writing **at a path it
  is handed**; a value read back from a file written by the same crate is the
  value that was written, held per crate against a real file rather than a
  round trip in memory; a hand-edited file with a key that is not on the list is
  refused whole with the key named, which each crate's refusal already carries
  (`alo_appearance::NotRead`), and the machine then draws what the release
  ships; each crate's header loses the sentence that said this was somebody
  else's, because a header that describes a state the crate has left is worse
  than none; and every word a person could read from these refusals is in the
  vocabulary `alo-saying` collects.
- **Constraint:** no crate here learns the folder — it is handed a path. None
  of them watches a file: a change made in Settings is drawn at once because
  Settings and the compositor are one process, and a file edited by hand is
  read at the next sign-in. A background reader of the person's folder is a
  mechanism nobody asked for.

### 3. A pairing is revoked the way a grant is

**Status:** ready. **Depends on:** 1.

`docs/features.md`, ★: what has been granted to what, in **one list, revoked
the same way**. A grant has a person's road — `alo_changing::Changing::revoked`
writes the file whole and knocks. A pairing has only the daemon's door, and no
crate on the person's side speaks it outside tests. Two revocations that are
two code paths in a compositor would not be one list.

- **Acceptance:** `alo-changing` gains a pairing's revocation over the daemon's
  existing `revoke-pairing` door, answered as the **same `Gone`** a grant's
  revocation is answered with, so that a surface revokes a row of the one list
  with one call whichever kind of row it is — held by a test that both kinds
  answer the same type; the pairings a surface lists come from the daemon's
  `pairings` answer or `alo_remembering::pairings_remembered`, and a test reads
  the shipped source for a second writer of `/var/lib/alo/pairings.toml`, which
  stays the daemon's alone; and a revocation that the daemon refuses says so in
  the daemon's words rather than being reported as done.
- **Constraint:** `alo-nearby` and `alo-agentd` are read and not edited — the
  door exists and this speaks it. Nothing here revokes anything without the
  person's act; there is no *revoke everything* and no revocation an agent may
  ask for, because a grant an agent could remove is a grant an agent could
  have chosen to keep.

### 4. What a person may edit, and what they are told when it did not read

**Status:** ready. **Depends on:** 2, 3.

These files are in a person's own folder, which means a person will open one in
an editor, and a portal will one day read one. Both of those need the shape
written down, and the moment a file does not read needs a sentence.

- **Acceptance:** `docs/contracts/person-settings.md` gains a section per file —
  the keys, the `format` number, what a missing file means, and what happens to
  a file that does not read — as three sections of the existing contract rather
  than a new one; every sentence these crates can say to a person is in the
  vocabulary with a translator's note, including the one that names **which
  file** did not read and **which key** was wrong, because *your settings could
  not be read* without naming the file is a sentence a person cannot act on;
  and a test reads the shipped source of all five crates for English written
  outside `alo-strings`.
- **Constraint:** nothing here changes what the sections describe. The contract
  follows the crates; where they disagree the crate is right and the document
  is wrong, and it is the document that changes.
