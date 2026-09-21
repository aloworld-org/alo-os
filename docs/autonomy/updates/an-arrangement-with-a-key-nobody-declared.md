# An arrangement with a key nobody declared

**Date:** 2026-09-21
**Workstream:** `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`, task 10
— *An arrangement with a key nobody declared*
**Contributor:** this development PC (Windows host, gates run in its Ubuntu)
**Status:** ready for integration.

## What changed, in words a person outside this repository can read

`displays.toml` is the file in a person's own folder that remembers where their
screens are. Until today it was read in two minds. A key alo OS does not know at
the **top** of the file refused the whole file and named the key, the way it does
in every other file in that folder. A key alo OS does not know **inside** an
arrangement — in the `[[arrangements]]` table, or in one of the
`[[arrangements.screens]]` rows that describe a screen — was read straight past.
`brightness = 50` on a screen row read, and the arrangement around it was
honoured.

Now it does not. An arrangement table and a screen row have exactly the keys the
contract lists, and one alo OS does not know refuses the whole file, as every
other way of getting that file wrong already does.

**This changes what a person experiences, and it is worth saying plainly.** A
hand-edited `displays.toml` with a stray key inside an arrangement stops reading,
where yesterday it read and the arrangement was used. What the person is told is
the sentence `displays.kept.not-understood`: *your screen arrangements at
`<file>` are not arrangements alo OS can read, so nothing in the file has been
used and your screens have been laid out side by side.* The file itself is left
byte for byte as they wrote it — a file that does not read is never written over
— so taking the stray line out by hand keeps everything else in it, and
Settings' *put this back as alo OS ships it* is the one door that replaces it.

The reason for making that trade is not tidiness. A row read past is a row a
later release, or a person's own hand, can quietly put something into, and a
screen's row is the one place in this file where a screen is *described* —
exactly the sort of thing something would be tempted to grow a field on. Every
other nested shape in these crates already refuses one; this was the gap, and
nothing suggests it was decided rather than missed.

A file this alo OS wrote reads exactly as it did yesterday. No key was added,
renamed or removed, and `format` did not move.

## What changed, with paths

### `crates/alo-displays/src/arrangement.rs`

Two attributes, and the documentation around them.

- `Written` — what an `[[arrangements]]` table is — gains
  `#[serde(deny_unknown_fields)]`.
- `WrittenScreen` — what an `[[arrangements.screens]]` row is — gains the same,
  beside the `rename_all` it already had.
- A module section, *A key nobody declared refuses the file*, says which
  sentence a person gets and why it is that one rather than the `unknown-key`
  sentence, and says outright that a hand-edited file with a stray key in it
  stops reading and that this is the point rather than a cost of it.
- One unit test, `a_key_nobody_declared_refuses_the_arrangement`, asks the shape
  itself for both tables and holds that a row with only declared keys still
  reads.

**Which sentence a person gets, and why.** The key is inside a *value*, so it is
serde that refuses it and `alo_kept::Unread::NotItsShape` that carries the
refusal — which `alo_displays::FileNotRead::word` already maps to
`displays.kept.not-understood`, the sentence a value alo OS cannot take gives.
It is not `displays.kept.unknown-key`, and that is the right answer rather than
a compromise: that sentence names the key at the top of the file for the person
to go and find, and there is no top-level key here to name.
`FileNotRead::key()` answers `None`, which is what it already answered for every
refusal that is not a top-level key, so nothing downstream had to learn a new
shape.

### `crates/alo-displays/src/keeping.rs`

Four tests, each through a real file on a real disk, because that is how a
person's edit arrives:

- `a_key_nobody_declared_in_an_arrangement_refuses_the_whole_file` — `desk`
  inside `[[arrangements]]`: refused, `displays.kept.not-understood`, `key()` is
  `None`, and `at_sign_in` answers with a machine that has arranged nothing.
- `a_key_nobody_declared_on_a_screen_row_refuses_the_whole_file` —
  `brightness = 50` on a screen row: the same, and it is the exact file that
  read yesterday.
- `the_same_file_with_only_the_keys_it_declares_reads` — the same file with
  nothing stray in it reads, with two screens, the second one's place and the
  main screen all honoured. Without this the two above would pass equally well
  against a crate that had stopped reading arrangements at all.
- `a_file_this_alo_os_wrote_still_reads` — an arrangement and night light
  written by `keep` and read back unchanged, which is the constraint the task
  put on the change.

### `docs/contracts/person-settings.md`

The `displays.toml` section's *Values* part loses the paragraph headed **One
check `appearance.toml` has that this file does not yet** — written by task 9,
which found the gap and deliberately left it — and gains the ordinary sentence
the other sections have: an `[[arrangements]]` table, and each
`[[arrangements.screens]]` row inside it, has exactly its own keys, and one alo
OS does not know refuses the whole file. It names what somebody would most
plausibly try to add — how bright a screen is, what the desk is called, the
resolution it was running at, which of two identical monitors this one is — and
says none of them is a key.

*A file that does not read* gains two refused examples, one per table, each
named to `displays.kept.not-understood`.

### `crates/alo-displays/tests/the_contract_describes_this_file.rs`

One test, `the_contract_says_a_key_inside_an_arrangement_refuses_the_whole_file`.
The file's existing test over every refused example already covers the two new
ones generically; this one holds the part that is specific and is the part that
could rot:

- the *Values* part no longer says *does not yet*, and does say *has exactly its
  own keys* — the same shape of check the *Writing it* part already had for the
  rule that landed before it;
- the section shows a refused example with a stray key inside
  `[[arrangements]]` and another inside `[[arrangements.screens]]`;
- each is refused from a real file, in the sentence the prose names, with no
  top-level key named and nothing in the file honoured;
- and **each reads with the stray line taken out**, so what the contract is
  demonstrating is that line and not the shape around it.

Nothing in that file was loosened. It gained a test; it lost nothing.

### `docs/autonomy/v0-5-the-session-and-the-displays-plan.md`

Task 10 marked **Done, 2026-09-21** with what it did and what a person notices,
and **task 11 written**, because the plan named none after it and a plan with no
next task reads to the loop as a workstream that is finished.

## Decisions this task made, and why

**`deny_unknown_fields` rather than a hand-written key list.** `alo-kept`
already checks the top of the file against `Kept::KEYS`, and the same shape
could have been copied inward. It was not, for two reasons: the nested keys are
serde's to know and a second list beside the struct is a thing two changes can
disagree about; and every other nested shape in these four crates —
`night_light::Written`, `Between`, `Whereabouts`, `alo_leaving::open::Written`,
`alo_notifying::quiet_hours::Written` — already uses the attribute, so this is
the two stragglers joining the rule rather than a sixth way of doing it.

**The sentence is `displays.kept.not-understood`, and no new string was
declared.** The plan's acceptance named it, and it is also the honest answer:
the person is being told that their arrangements could not be read and what the
machine did instead, which is exactly what this sentence says. A new string
naming the nested key would have been a better sentence in the abstract and a
worse one in practice — `alo-kept` hands the owning crate a reason without
words, and reaching the key out of a serde message means parsing English out of
a library's error text, which is not something a translated product should do.

**Task 11 is *The desk a machine wakes up at*.** The plan had no task after this
one, and the instruction to write the next one is not an invitation to invent
work. What was chosen is the one path through this workstream that nothing
decides: `alo_displays::Attached` learns about screens from `unplugged` and
`plugged_in` events, one cable at a time, and a machine that was asleep saw none
of them — so a laptop suspended at home and opened at the office wakes holding
the set of screens it went to sleep with. Task 7's walk docks a screen *after*
the resume, which is the hotplug road working as built; the step before it is
missing. It needs no new layout rule, it lives entirely in `alo-displays` and
`alo-sleeping` (both this plan's), and its refusal path — a machine that wakes
with nothing plugged in at all — is as concrete as its road.

The per-display dock edge was the other candidate, and it was rejected on
ownership rather than on merit: `docs/features.md`'s *Per display, so the dock
can sit along the bottom of the laptop and down the side of the external screen*
is still unmet, task 3 says so, and `alo_displays::Wearing::of` is the one
function that changes when it is met — but `alo-dock` belongs to
`v0-5-where-a-persons-settings-are-kept-plan.md`, and this plan's header says it
reads that crate and never edits it. It stays where task 3 left it: a finding
written down, waiting on its owner.

## Acceptance, and the evidence for each

The workspace is the product's (`.`). Each was run on its own, on the tree
being published.

| Acceptance criterion | Crate | Target | Test |
|---|---|---|---|
| An unknown key inside an `[[arrangements]]` table refuses the whole file, and `at_sign_in` answers with a machine that has arranged nothing | `alo-displays` | `lib` | `keeping::tests::a_key_nobody_declared_in_an_arrangement_refuses_the_whole_file` |
| …and inside an `[[arrangements.screens]]` row | `alo-displays` | `lib` | `keeping::tests::a_key_nobody_declared_on_a_screen_row_refuses_the_whole_file` |
| …and a key that *is* declared still reads | `alo-displays` | `lib` | `keeping::tests::the_same_file_with_only_the_keys_it_declares_reads` |
| Both tables refuse at the shape itself | `alo-displays` | `lib` | `arrangement::tests::a_key_nobody_declared_refuses_the_arrangement` |
| Nothing else about the shape changed: a file this alo OS wrote yesterday reads today | `alo-displays` | `lib` | `keeping::tests::a_file_this_alo_os_wrote_still_reads` |
| The contract loses the *does not yet* paragraph, gains the ordinary sentence, and shows a refused example per table | `alo-displays` | `the_contract_describes_this_file` | `the_contract_says_a_key_inside_an_arrangement_refuses_the_whole_file` |
| `tests/the_contract_describes_this_file.rs` goes on passing without being loosened | `alo-displays` | `the_contract_describes_this_file` | `every_example_the_contract_refuses_is_refused_in_the_words_it_names` |

## Verification

Run from this checkout's Windows host, in its Ubuntu, against the synchronized
Linux source copy at `/root/alo-trees/this-machine` with
`CARGO_TARGET_DIR=/root/alo-builds/this-machine`, as `SHARED_MAIN.md` requires.

| Check | Result |
|---|---|
| `cargo fmt --check`, every workspace member | clean |
| `cargo clippy -p alo-displays --all-targets -- -D warnings` | clean, zero warnings |
| `cargo test -p alo-displays` | 144 passed, 0 failed |
| `cargo test -p alo-leaving -p alo-sleeping -p alo-saying` (the dependants that use this crate) | passed |
| `cargo doc -p alo-displays --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean |
| `cargo test -p alo-kernel-loop plan` (the plan file changed) | 26 passed |
| Each evidence test above, run alone with `--exact` | 1 passed each |

**`cargo fmt --all` cannot be run as one command on this host.** It fails with
`os error 206` — Windows' command-line length limit, reached when `cargo fmt`
passes every file in a hundred-crate workspace to `rustfmt` in one invocation.
It is an environment limit and not a fault in the tree, and it is not a gate
weakened: the same check was run as `cargo fmt -p <member> --check` for **every
one of the 100 workspace members**, which covers exactly the same files. The
supervisor's own gate runs `cargo fmt --all --check` from the Linux side, where
the limit does not apply.

**The full workspace suite was not run here**, by instruction: the supervisor
runs it after this hand-over, and it takes the better part of an hour on this
machine. What was run is every crate this change touches and every crate that
depends on it.

**Not on hardware.** Nothing here opens a device, sets a mode or draws. It is a
serde clause, a contract paragraph and the tests around both.

## Remaining limitations

- **A file that a person, or an earlier alo OS, wrote with a stray key inside an
  arrangement will stop reading.** There is no migration and there should not
  be one: the whole point is that the machine stops honouring a file it does not
  understand, and guessing which stray keys are safe to drop would be the same
  fault in a friendlier coat. The person is told what happened, what the machine
  did instead, and the file is theirs to mend.
- **Nothing mechanical holds every nested shape in these crates to the rule.**
  All of them now carry `deny_unknown_fields`, and a seventh added tomorrow
  without it would be found by a reader rather than by a test. A workspace-wide
  check — every shape reachable from an `impl Kept` held to it — would be the
  proper fix and belongs with the repository-wide checks in the
  `alo-citing`/`alo-collected` family, not inside a crate about screens. It is
  named here rather than built because it needs a home this task has no standing
  to choose, and because `alo-kept` is read and never edited by this plan.

## Proposed changelog entry

> **A settings file for your screens is no longer read past.** A key alo OS does
> not understand inside one of your saved screen arrangements now refuses the
> whole `displays.toml` file, the way a key it does not understand anywhere else
> in that file already did. You are told that your arrangements could not be
> read and that your screens have been laid out side by side; the file is left
> exactly as you wrote it, so a stray line is yours to take out. A file alo OS
> wrote itself is unaffected.

## Proposed queue and roadmap updates

- `docs/autonomy/QUEUE.md`: mark *An arrangement with a key nobody declared*
  done, and add *The desk a machine wakes up at* (task 11 of this plan) as ready.
- `ROADMAP.md`: no change. This does not complete or alter a v0.5 exit gate; it
  closes a finding task 9 published against the *Multi-monitor, scaling,
  hotplug* line's file format.
