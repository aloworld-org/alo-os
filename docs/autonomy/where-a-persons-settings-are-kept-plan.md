# Where a person's settings are kept

*Named `v0.5 — where a person's settings are kept` until 2026-10-02. **A release code is not a subject** — `v0.5` says *when*, never *what*, which is what `CLAUDE.md`'s *names are for strangers* forbids and what the owner asked be cleared out of this repository. The subject was already in this line; the rename only wrote it down. The release this belongs to is in `ROADMAP.md`, which is where release codes live.*

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

### 9. A person's pinned applications are kept

**Status:** **Open, and owed to this plan by another lane's work.** **Owner:**
this plan's lane — the development PC — for the **persisted representation and
the storage change**. The Dock's lane owns the behaviour, the drawing and the
integration against the contract, and has built its half.

**Written 2026-10-10 by the owner's direction**, who ruled on what a fresh
machine pins and then said where the work divides:

> The existing named owner of `alo-kept` and
> `where-a-persons-settings-are-kept-plan.md` owns the persisted representation
> and storage changes. Your lane owns Dock behaviour, rendering and integration
> with that contract. Agree on the contract and record the responsibilities in
> the plans. Use the existing settings mechanism; do not create competing
> settings files.

**What is already built, so this task starts from a shape rather than a
question.** `alo_dock::pinning` decides which applications a fresh machine pins
— Files, Browser, Apps, Settings, in that order, omitting any role nothing
fills — and `alo_dock::ThePins` holds the distinction the ruling turns on.
`docs/contracts/person-settings.md`'s `dock.toml` section states what must be
stored and why a simpler shape will not do.

- **Acceptance:** `dock.toml` keeps a person's pins, in their order, and keeps
  **whether the pins have ever been set** apart from what they are. A person who
  empties their Dock finds it empty after a restart; a person who removes one
  pin does not get it back; an update does not re-run the defaults; a settings
  file that fails to read does not re-run them either. Verified **in a fresh
  process after restarting**, which is the owner's condition and the only one a
  single-process test cannot meet.
- **Constraint:** the existing mechanism — `alo-kept`, one file, refused whole
  when wrong, written whole or not at all (ADR 0038). **No second settings
  file.** The key's spelling is this lane's; the *shape* is the contract's, and
  a representation that cannot tell *never set* from *set to nothing* does not
  meet it however it is spelled.
- **Depends on:** nothing. `alo_dock::ThePins` is in `main` and the Dock reads
  whatever it is handed.
- **Blocked by:** this lane being offline on 2026-10-10. The owner's words:
  *their being offline is a delivery dependency, not a reason to invent another
  storage mechanism or mark persistence complete.* So the Dock's side is
  finished and says it is waiting, rather than growing a `dock.toml` key of its
  own.


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

**Status:** **Done, 2026-09-15** — `alo_changing::Changing::revoked` takes a
`Row` (a grant's `Seen` or a `SeenPairing`) and answers `Gone` for both; a
pairing is asked of the daemon over `revoke-pairing`; see
`docs/autonomy/updates/a-pairing-is-revoked-the-way-a-grant-is.md`. **Depends on:** 1.

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

**Status:** **Done, 2026-09-15** — `docs/contracts/person-settings.md` has a
section per file, held to each crate by its `tests/the_contract_describes_this_file.rs`;
`alo-choosing`'s `tests/no_english_outside_the_vocabulary.rs` reads all five
crates' shipped source; see
`docs/autonomy/updates/what-a-person-may-edit-and-what-they-are-told.md`.
**Depends on:** 2, 3.

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

### 5. A file that did not read is not written over by the next click

**Status:** **Done, 2026-09-15** — `alo_kept::keep` asks the file as it is
immediately before the rename and refuses `Unwritten::OverAFileThatDidNotRead`;
`alo_kept::put_back_as_shipped` (and each crate's `keeping::put_back_as_shipped`)
is the one door that replaces such a file; see
`docs/autonomy/updates/a-file-that-did-not-read-is-not-written-over.md`.
**Depends on:** 2, 4.

ADR 0038, clause 3: *Settings does not write over a file that did not read*
except when the person puts that section back as shipped, *so a hand-edit with
a typo is never lost silently to the next click.* Task 4 wrote the contract from
the crates and found that none of the three holds it: `keeping::keep` replaces
whatever is at the path, so a surface that draws the release's appearance after
`at_sign_in` refused the file, and then saves the person's next change, throws
away the edit they were about to fix. `alo_choosing::Choosing` already refuses
to open settings that do not read for exactly this reason; the three files
beside it do not. The contract's three *Writing it* sections say so today.

- **Acceptance:** `alo-kept` carries the rule in type, so the three crates
  cannot each forget it: a write is refused, in the owning crate's words and
  naming the file, when the file it would replace is there and does not read —
  held by a test per crate against a real file whose bytes are unchanged
  afterwards; putting the section back as shipped is a distinct, deliberate
  door that does replace such a file, and is the only one; a file that is not
  there, or that reads, is written exactly as today; every new sentence is in
  the vocabulary with a translator's note and collected by `alo-saying`; and
  the three *Writing it* sections of `docs/contracts/person-settings.md` say
  what the crates then do, held by each crate's
  `tests/the_contract_describes_this_file.rs`.
- **Constraint:** no crate learns the folder, and nothing watches a file. The
  check is made at the moment of the write, against the file as it is then —
  never against what was read at sign-in, which a hand edit since may have
  fixed. Nothing here is reachable by an agent: these are a person's own
  settings, written from Settings.

### 6. One person's folder, from sign-in to the next change

**Status:** **Done, 2026-09-15** — `crates/alo-choosing/tests/one_persons_folder_from_sign_in_to_the_next_change.rs`
walks the folder with each of the three files broken in turn, and
`docs/contracts/person-settings.md` has *A Settings surface, from sign-in to the
next change*, held by that test; see
`docs/autonomy/updates/one-persons-folder-from-sign-in-to-the-next-change.md`.
**Depends on:** 1, 2, 3, 5.

Every clause above is held crate by crate. What no test yet holds is the path a
session actually takes through all of them at once, which is the road the shell
plan's task 6 will call: the folder worked out by `alo_choosing`, a path handed
to each of the three keepers, each section drawn at sign-in with its refusal
beside it, a change written, a broken file kept rather than replaced, a section
put back as shipped, and a pairing and a grant revoked from the one list. A
break between two crates that each pass their own suite — a file name two
crates disagree on, a folder one of them makes with the wrong mode — is found
here or by a person.

- **Acceptance:** one integration test, in a crate this plan owns and without
  giving `alo-appearance`, `alo-dock` or `alo-shortcuts` a dependency on
  `alo-choosing`, walks a person's folder under a temporary `$HOME` handed to
  `alo_choosing::where_the_folder_is` (never read from the environment): the
  three files are written beside `settings.toml` under their own `THE_FILE`
  names and read back at a second sign-in as what was written; a hand edit that
  breaks one file leaves the other two drawn as the person left them and that
  one drawn as shipped, with its sentence naming the file; the next change to
  the broken section is refused and the file's bytes are unchanged while a
  change to another section is written; putting the broken section back as
  shipped lets the next change through; and `docs/contracts/person-settings.md`
  names, in one short section for the shell, the calls a Settings surface makes
  for each section — held by the same test reading that section.
- **Constraint:** nothing in `crates/alo-shell`, no new public surface unless
  the walk finds one missing (and then it is named in the report as what the
  shell plan waited on), nothing on the machine, and no watcher.

### 7. A session with no folder says so in Settings

**Status:** **met in this plan's crates, and no person can read it, 2026-10-03.** Was
*Done, 2026-09-18 — implementation complete*, and the part this plan is responsible for holds.
The typed folder result, refusal sentence and Settings contract are held by the folder walk
and refusal tests; see
`docs/autonomy/updates/settings-explain-when-session-changes-cannot-be-kept.md`.

**What holds, measured rather than assumed.** `alo_choosing::where_the_folder_is` is called in
**production** at `crates/alo-shell/src/settings_places.rs:44` — `#[cfg(test)]` in that file
begins at line 78 — and the `SettingsPlaces` it answers into is taken in production by
`settings_window.rs`'s `opened_by_hand`. So a Settings surface genuinely cannot reach a
keeper's path without having met the no-folder case, which is what this task asked for.

**What does not hold is that the sentence can ever be read**, and the reason is one road
further on than this plan goes — **now `the-shell-plan.md` task 19**, which
records that no chord reaches a running desktop at all:

```text
SettingsWindow::closed()          constructed in tests only
SettingsWindow::opened_by_hand    0 production callers — every `.opened_by_hand(`
                                  in production is RecordWindow's different
                                  method of the same name
nested_settings::pump_settings    no caller at all
SettingsKey                       no caller outside its own module
"Settings" in crates/alo-desktop  0 non-comment mentions
```

This workspace builds many binaries and most are daemons. The two that put anything on a
screen are **`alo-desktop`**, a crate of its own, and **`alo-compositor`**, which is a
`[[bin]]` target of `alo-shell` at `src/bin/alo-compositor.rs` rather than a crate — so it is
named in no workspace member list and `ls crates/alo-compositor` finds nothing, which is how
its existence gets mislaid in both directions. `cargo metadata` lists it as
`alo-compositor (alo-shell)`. **Neither of the two creates a `SettingsWindow`**, and there is
no third surface to look in. The window draws — `settings_raster` lays it out — and routes keys, and
refuses correctly in every tested case, and nothing opens one. So every sentence this plan
declared for Settings is reachable by a test and by nobody else.

**Not this plan's road, and named rather than taken.** Opening a Settings window is a shell
surface, and Settings surfaces in `alo-shell` belong to `the-shell-plan.md`'s tasks 7 to 14,
which the charter gives to this PC's lane A. This plan's own acceptance is met; its visibility
waits there.

*Found by auditing this plan the way the panel lane audited canvas task 9 — asking of each
`Done` whether anything reaches it. The sibling finding is task 5 of
`hands-on-the-desktop-plan.md`, where a gesture is recognised and cannot be turned off.*
**Depends on:** 6.

Task 6's walk found the one moment on the road that has no words. A login with
no home directory — or a session whose `$XDG_CONFIG_HOME` is relative and whose
`$HOME` is unset — gets `None` from `alo_choosing::where_the_folder_is`. The
contract's section for the shell says every section is then drawn as the release
ships it and nothing is written, and that is right; but a person who changes the
dock in that session watches it move and finds it back at the next sign-in, with
no sentence anywhere that told them it would not be kept. *Nothing leaves
silently* has a twin here: nothing is forgotten silently either.

- **Acceptance:** `alo-choosing` declares the sentence a Settings surface says in
  a session with no folder — that changes made now are drawn and will not be kept
  past this sign-in, and why (no home directory to keep them in) — in the
  vocabulary with a translator's note and collected by `alo-saying`; a value the
  surface holds (not a bare `Option`) answers either the folder or that refusal,
  so a surface cannot reach a keeper's path without having met the case; a test
  per way the folder is missing (no `$HOME`; a relative `$XDG_CONFIG_HOME` and no
  `$HOME`; a relative `$HOME`) answers the refusal and writes nothing anywhere;
  the contract's *A Settings surface, from sign-in to the next change* names the
  call and the sentence, held by task 6's walk; and the no-English test still
  reads all five crates' shipped source clean.
- **Constraint:** no keeper learns about the folder or the environment, nothing
  in `crates/alo-shell`, nothing is written to a place the session made up (no
  `/tmp` fallback — a folder that belongs to nobody is the thing the contract
  refuses), and no new file in the person's folder.

### 8. Ask for it — the appearance a person asked for in words

**Status:** **the road is declared and nothing calls it yet, 2026-10-04.**
`crates/alo-appearance/src/verbs.rs` declares two verbs — `set_the_scheme` and
`follow_the_clock` — with their nine strings, and both end at `changes.rs`'s
`Changes::follow`, the road a settings panel writes through. What is **not**
paid is the half this task already excludes: nothing on a machine calls them,
because carrying a call out is `alo-agentd`'s. **Depends on:** nothing.

**Moved into this release on 2026-10-03** by
`docs/decisions/0084-seven-promises-move-from-v1-into-v0-5.md`. The promise is
`docs/features.md`'s *★ **Ask for it** — "make the surface warmer", "use dark
after six" — the same propose-then-approve as any other change, because
personalisation is exactly the low-stakes place people first learn to trust the
agent.*

**Half of it is already built, and the crate names the promise itself.**
`crates/alo-appearance/src/time.rs` opens *a time of day, which is all a schedule
needs to know* and explains that **use dark after six means six o'clock where
the person is, every day** — not an instant, so no calendar and no decision about
the night the clocks go back. `scheme.rs` carries `Following`, which answers a
`Scheme` from a moment passed in, tested at the hour and the half hour; nothing
there reads a clock, so a settings panel previewing a schedule and the
compositor obeying it cannot disagree about what it says. `changes.rs` holds
what a person changed as the difference from the running release's defaults.

**So a person who can reach a settings panel can already have a schedule. What
nothing does is the asking.** There is no road from *make the surface warmer*,
said in words, to a proposal naming exactly what would change, to a person
approving it, to `changes.rs`. That road is the promise; the schedule is the
thing it would set.

- **Acceptance:** a person says *use dark after six* and is shown what would
  change — the setting, its old value and its new one — before anything moves;
  approving it writes the same change a settings panel would have written, and
  declining writes nothing. The same for a warmer surface. **A proposal names
  the change in the person's own language**, not a token or a field name.
- **Constraint:** the propose-then-approve road is **`alo-capability`**, and a
  crate reaches it by declaring its own verbs — a `pub fn declare_into(verbs:
  &mut Verbs)` in `src/verbs.rs`, which thirteen crates already have and
  `docs/contracts/agent-verbs.md` asks for. `alo-capability` depends on
  `alo-strings` alone, so the edge adds no cycle, and **nothing is requested from
  another lane**. *This constraint named `alo-asking` until 2026-10-04. That
  crate is "putting a question to a model" — a correct name in a different
  vocabulary — and the dependency it gave as the reason was one `alo-appearance`
  did not declare.*
- **Constraint:** no new way to set appearance. Whatever a proposal applies must
  be the road `changes.rs` already defines, or a person's settings and the
  agent's will drift and only one of them will be written down.

**What this does not include.** Making it work on a machine — the half every
promise in this release owes — and the settings panel itself, which is this
plan's other work. This task is the agent's road to a change, not the change.
