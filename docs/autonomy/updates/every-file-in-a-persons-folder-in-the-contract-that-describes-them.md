# Every file in a person's folder, in the contract that describes them

**Date:** 2026-09-21
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 9
— *The four files this plan keeps, in the contract that describes them*
**Contributor:** this development PC (Windows host, gates run in its Ubuntu)
**Status:** ready for integration.

## What changed, in words a person outside this repository can read

`docs/contracts/person-settings.md` is the published surface a third party
builds against when it reads a person's own settings folder. Until today it
said there were **four** files in that folder and listed them. There were ten.
Somebody writing a backup tool, a migration or an organisation's provisioning
from that sentence would have written something that silently dropped more than
half of what a person had chosen — and would have had no reason to look, because
the contract read as complete.

The contract now describes the folder as it is: ten files in its table, and a
section of its own for eight of them, each with its keys, the values each key
takes, what a missing file means, what a file that will not read is told and in
whose words, and a refused example for every way the file can be wrong. The four
new sections are `sleeping.toml`, `displays.toml`, `leaving.toml` and
`notifying.toml` — the files this plan's crates keep. Each is held to its crate
by a new test in that crate, so a key added to one of those files without a line
in the contract fails in the change that adds it.

## What changed, with paths

### The contract

`docs/contracts/person-settings.md`

- *The folder, and the files beside this one*: the counting sentence moves from
  *Since 2026-09-15 there are four* to *Since 2026-09-21 there are **ten***, the
  table gains six rows — `sleeping.toml`, `displays.toml`, `leaving.toml`,
  `notifying.toml`, `keyboards.toml` and `gestures.toml` — and two new
  paragraphs say how the count stays true and which two rows have no section
  yet.
- Four new sections, in the shape the existing four have and in the order this
  plan gained the files: `sleeping.toml`, `displays.toml`, `leaving.toml`,
  `notifying.toml`. Each has *Keys*, *`format`*, *What alo OS writes*,
  *Values*, *What a missing file means*, *A file that does not read* and
  *Writing it*.
- The four existing sections are untouched, as the task's constraint requires.

### The four crates

- `crates/alo-sleeping/tests/the_contract_describes_this_file.rs` (new)
- `crates/alo-displays/tests/the_contract_describes_this_file.rs` (new)
- `crates/alo-leaving/tests/the_contract_describes_this_file.rs` (new)
- `crates/alo-notifying/tests/the_contract_describes_this_file.rs` (new)

Each is modelled on `crates/alo-appearance/tests/the_contract_describes_this_file.rs`
— it **reads the contract's own text** rather than restating it — and holds six
things:

1. the folder's table names this file, with the crate that keeps it and its
   `format`, and **the number in the counting sentence equals the number of rows
   in the table under it**;
2. the keys the section's *Keys* table lists are exactly the keys the crate
   writes;
3. the file the section shows under *What alo OS writes* is byte for byte the
   file the crate writes, and every `toml` example in the section reads;
4. every `toml refused` example is refused, in the `<crate>.kept.*` sentence the
   prose after it names, naming the key or the line where the prose says it does
   — and then `keeping::at_sign_in` answers with what the release ships and
   **nothing in the file honoured** (ADR 0038);
5. a missing file is a person who changed nothing, and reading one writes
   nothing;
6. *Writing it* is true: a change over a file that did not read is refused with
   `<crate>.kept.not-replaced`, the file's bytes are unchanged,
   `FileNotWritten::did_not_read` says what is wrong with it, and
   `put_back_as_shipped` replaces it with the `format` line alone.

Every example goes through a real file on a real disk, because that is how a
person's edit arrives.

- `crates/alo-sleeping/src/keeping.rs`, `crates/alo-displays/src/keeping.rs`,
  `crates/alo-leaving/src/keeping.rs`, `crates/alo-notifying/src/keeping.rs`:
  one paragraph of module rustdoc apiece, pointing at the contract section and
  at the test that holds the two together. No code changed in any of the four
  crates.

### The plan

`docs/autonomy/v0-5-the-session-and-the-displays-plan.md`: task 9 marked
**Done, 2026-09-21**, and **task 10** written, because the plan named nothing
after task 9 and one thing this task found is not this task's to fix.

## Decisions, and why

### The count is ten, not eight — and that is the finding

The task said four files were owed and the contract would then describe eight.
Reading the workspace for every `impl Kept for` found ten real ones: the four
described, the four this plan owes, and two nobody had mentioned —

| File | Kept by |
|---|---|
| `keyboards.toml` | `alo_keyboards::keeping` |
| `gestures.toml` | `alo_desktops::gesture_files` |

Writing *eight* would have repeated, in the same sentence, the exact mistake the
task exists to correct. So the table has all ten rows, the sentence says ten,
and the contract states plainly that two of the ten have no section yet and
points at the crates that declare them. A reader now knows those files exist,
which crate keeps each, and what their keys are — which is most of what a backup
tool needs and all of what it needs in order not to lose data.

**Their sections were not written here**, and this is deliberate rather than an
omission of convenience. A section unheld by a test in its own crate is the
drift this task was written to end, and adding
`tests/the_contract_describes_this_file.rs` to `alo-keyboards` and
`alo-desktops` means working inside two crates this plan does not own and whose
lanes were not consulted. It is one task's work for whoever owns them, and the
table now makes it visible rather than invisible.

### How the count stays true

Prose cannot hold a number; a test can. Each of the four new tests reads the
number word out of the counting sentence, maps it through a small table of
whole numbers, and asserts it equals the rows of the table beneath it. A row
added without the number moving, or the number moved without a row, fails in the
change that does it — on four separate crates, so removing the check means
noticing it four times.

What that does **not** catch is an eleventh crate keeping an eleventh file
without a row at all. The honest answer is that nothing mechanical catches that
today, and the contract does not pretend otherwise: ADR 0038's rule that the
crate declaring a shape gains that test in the change that declares it is what
stands there, and it is prose. A workspace-wide check — every `impl Kept` held
to a row — would be the proper fix and belongs with the
`alo-citing`/`alo-collected`/`alo-reconciling` family of repository-wide checks
rather than inside a crate about screens. It is not written here because it
needs a home this task has no standing to choose, and the task's constraint is
explicit that `alo-kept` is read and never edited.

### `displays.toml` reads past a key inside an arrangement, and the contract says so

Probing every refusal path before describing it found one the contract could not
have described truthfully as uniform. In all eight described files, a key alo OS
does not know at the **top** of the file refuses it whole with the key named.
Inside a value, seven of the eight refuse too, because every nested shape carries
`deny_unknown_fields` — `alo_displays::night_light::Written`, `Between`,
`Whereabouts`, `alo_leaving::open::Written` (the clause that keeps a window title
out of a person's folder) and `alo_notifying::quiet_hours::Written` all do.
`alo_displays::arrangement`'s two `Written` structs do not, so this reads, and
the arrangement is honoured:

```toml
format = 1

[[arrangements]]
name = "office"

[[arrangements.screens]]
socket = "eDP-1"
at = [0, 0]
scale = 100
main = true
brightness = 50
```

**It was not fixed here.** The task's constraint says the four crates are not
rewritten for the contract, and making a hand-edited file that reads today stop
reading tomorrow is a change to what a person experiences: it deserves its own
argument, its own refusal tests and its own line in `CHANGELOG.md`, not a quiet
arrival inside a documentation task. What the contract does instead is say
exactly what happens today, under the heading *One check `appearance.toml` has
that this file does not yet*, and tell nobody to rely on it. **Task 10** is
written in the plan to close it.

### Smaller choices

- **Section order and headings.** The four new sections sit after
  `what-opens-what.toml` and before *A Settings surface*, in the order the plan
  gained the files (task 2, 3/4, 5, 6), and the table lists them in that same
  order. Each heading is `` ## `<file>` — <what it is about> ``, matching the
  existing four, and the folder's list of links was extended to all eight.
- **`leaving.toml`'s example is written by a log-out.** `Changes::set_what_was_open`
  is `pub(crate)`, because the one moment a list is written is a log-out. The
  test therefore walks a real one — a real sign-in, an unlocked `alo_locking::Seat`,
  a compositor where every application closes, then `keeping::at_sign_out` —
  rather than reaching around that door. Testing the example any other way would
  have been testing something the crate does not do.
- **The section examples are real files, dumped from the crates.** Every
  *What alo OS writes* block was produced by running the crate's own `keep` (or
  `at_sign_out`) and copying the bytes, not written by hand; the tests then hold
  them byte for byte.
- **`A Settings surface, from sign-in to the next change` was not extended.**
  It describes the three sections `alo-shell` draws and is held by
  `crates/alo-choosing/tests/one_persons_folder_from_sign_in_to_the_next_change.rs`.
  Whether the shell grows sections for these four files is the shell plan's
  decision, not this task's.

## Acceptance, and the evidence for each

The workspace is the product's (`.`). Each was run on its own.

| Acceptance clause | Where it is held |
|---|---|
| The table gains a row per file, with keeper, `format` and keys; the counting sentence is true and says how it stays true | `alo-sleeping`, `alo-displays`, `alo-leaving`, `alo-notifying`, test target `the_contract_describes_this_file`, `the_folders_table_names_this_file_and_its_count_is_the_rows_it_has` |
| Each file gains a section with every key and the values it takes | same target, `the_contract_lists_every_key_this_crate_writes_and_no_other` |
| The section's file is the file the crate writes, and every example it offers reads | same target, `what_the_contract_says_alo_os_writes_is_what_it_writes_and_every_example_reads` |
| A refused example per way a file can be wrong, refused **whole**, in the words the contract names, with nothing honoured (ADR 0038) | same target, `every_example_the_contract_refuses_is_refused_in_the_words_it_names` |
| What a missing file means | same target, `a_missing_file_is_a_person_who_changed_nothing` (`alo-displays`: `a_missing_file_is_a_person_who_arranged_nothing`) |
| What a file that will not read is told when a change is made over it | same target, `the_contract_says_a_file_that_did_not_read_is_not_written_over` |

## Verification

Run from this checkout, in its Ubuntu, against the copy the gates build from
(`/root/alo-trees/this-machine`, `CARGO_TARGET_DIR=/root/alo-builds/this-machine`,
mold, `RUSTDOCFLAGS=-D warnings`) — the same environment
`tools/kernel-loop/src/gates.rs` uses.

- `cargo fmt --all --check` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean, 1 m 40 s.
- `cargo test -p alo-sleeping` — 62 passed, 0 failed, across 8 targets.
- `cargo test -p alo-displays` — 138 passed, 0 failed, across 8 targets.
- `cargo test -p alo-leaving` — 73 passed, 0 failed, across 9 targets.
- `cargo test -p alo-notifying` — 115 passed, 0 failed, across 11 targets.

And, because this change touches `docs/**` and a plan — the rows
`docs/autonomy/SHARED_MAIN.md`'s *Gate what the change can reach* names:

- `cargo test -p alo-citing`, the citation check — **it failed twice before it
  passed, and rightly both times.** The first time, the `leaving.toml` section
  pointed at ADR 0001 by a filename built out of what that decision is *about*
  rather than what it is called (`0001-the-capability-model.md`). The second
  time, this report — narrating that fix — spelled the wrong filename out
  again, and the check reads every `.md` in the checkout, this directory
  included: a citation quoted is a citation carried, and a reader who follows
  it arrives at the same missing file either way. So the sentence above names
  no decision file but the real one. That is the class of fault a renamed ADR
  leaves behind, and it is why the gate exists;
  `crates/alo-citing/tests/every_decision_this_repository_points_at.rs` has the
  same problem in its own header and solves it the same way, by building its
  fixtures' pointers out of a constant rather than writing them. Clean on the
  third run — 31 passed, 0 failed.
- `cargo test` in `tools/kernel-loop`, the supervisor's own tests, which
  include the checks that read every plan — 152 passed, 0 failed.
- `cargo doc -p alo-sleeping -p alo-displays -p alo-leaving -p alo-notifying
  --no-deps` with `RUSTDOCFLAGS=-D warnings` — clean, for the four module
  paragraphs added to the `keeping.rs` files.

The four crate suites above were run again after the citation fix, since the
contract each of them reads had changed; the counts are from those runs.

**Not run here, and not claimed:** the full workspace suite, which the
supervisor runs after this task, and the BPF target's gates. **Nothing on
hardware**, and nothing is claimed about hardware: this change is a contract and
four tests, and it opens no device.

## What the supervisor's suite caught, and what fixed it

This change was handed over once and refused. The gate was the whole
workspace's tests, and the one that failed was `alo-citing`'s
`every_decision_this_repository_points_at_exists`, with one finding: line 229
of **this report** named `0001-` followed by a description of ADR 0001 instead
of its filename, and no such file exists under `docs/decisions/`.

It was the first fault repeating itself one layer out. The citation had been
found and fixed in the contract, and then written a second time into the
sentence that described fixing it — which the check reads, because it reads
every `.md` in the checkout. Quoting a dead link still leaves a dead link: a
reader who follows it lands nowhere, and the check has no way to tell a
citation from a citation being talked about, nor should it try. The gate
section above now names only the real file.

Nothing else was changed to make the gate pass. No test was loosened, no
exemption taken, and the contract, the four tests and the four `keeping.rs`
paragraphs are byte for byte what the first pass wrote. The gates were then run
again from the top — `cargo fmt --all`, `cargo clippy --all-targets -- -D
warnings`, the four crate suites, `alo-citing`, the supervisor's own tests, and
each of the twenty-four evidence tests on its own.

## Limitations

- `keyboards.toml` and `gestures.toml` have a row and no section. Named in the
  contract as such; one task's work for the lanes that own `alo-keyboards` and
  `alo-desktops`.
- Nothing mechanical stops an eleventh file appearing with no row. See *How the
  count stays true* above; a repository-wide check is the fix and needs a home.
- `displays.toml` reads past an unknown key inside an arrangement. Described in
  the contract, task 10 in the plan.

## Proposed shared-document updates

Not written here — `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` and
`docs/autonomy/STATE.md` are the integration owner's.

**For `CHANGELOG.md`:**

> The contract that describes a person's own settings folder now describes all
> of it. It said there were four files in the folder; there are ten, and a
> backup or a migration written from that sentence would have dropped what a
> person chose about sleep, their screens, logging out and notifications without
> saying so. Four of those files — `sleeping.toml`, `displays.toml`,
> `leaving.toml` and `notifying.toml` — gain a section apiece: every key, the
> values it takes, what a missing file means, and what alo OS tells you when the
> file will not read, with a worked example of each way it can be wrong. A test
> in each crate holds the document to the code, and reads the number that counts
> the files against the table under it, so the count cannot quietly go stale
> again.

**For `docs/autonomy/QUEUE.md` / `STATE.md`:** task 9 of
`v0-5-the-session-and-the-displays-plan.md` is done; task 10 (*An arrangement
with a key nobody declared*) is published in that plan and ready, depending on
tasks 3 and 9. A sentence for whoever holds the `alo-keyboards` and
`alo-desktops` lanes: `keyboards.toml` and `gestures.toml` are in the contract's
table without sections, and owe one each with a
`tests/the_contract_describes_this_file.rs` to hold it.
