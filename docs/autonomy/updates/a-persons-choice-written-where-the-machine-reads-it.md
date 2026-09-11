# A person's choice about their model, written where the machine reads it

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 24
**Contributor:** Claude (`C:\dev\alo-os-claude`)
**Status:** ready for integration

## What was wrong

`crates/alo-choosing` is the person's half of
[ADR 0016](../../decisions/0016-the-organisation-bounds-and-the-person-chooses.md):
which model answers their questions, which weights they brought to the machine
themselves, which providers they added, and which language they read. It **only
read**. `Settings::at` opened the file under `$XDG_CONFIG_HOME/alo`, `alo-agentd`
asked it once a turn, and nothing anywhere in this repository wrote a byte of it.

`docs/autonomy/v0-01-evidence.md` had said so in as many words since task 11: *a
provider is added by writing the person's own settings file by hand*. So every
choice ADR 0016 gives the person was a choice no surface could carry out — a
settings panel's only route to *save* was to compose somebody's settings as text
itself, which means a second writer of a shape only `alo-choosing` knows, in a
crate that has no business knowing it.

That is the gap task 23 closed for the grants, one file over.

## What changed

`alo_choosing::Choosing` is the written change. It takes a path, holds what the
file at it says, and has one door per thing ADR 0016 gives the person:

| Door | What it changes |
|---|---|
| `answered_by(Option<Picked>)` | What answers this person's questions, chosen or cleared |
| `bringing(Weights)` | Weights they brought to this machine themselves |
| `adding(Provider)` | A provider they added |
| `reading(Vec<Language>)` | The languages they read, best first |

Source:

- `crates/alo-choosing/src/choosing.rs` — the value and its four doors.
- `crates/alo-choosing/src/writing.rs` — settings as the text of a file, and the
  round trip that refuses anything this alo OS would not read back.
- `crates/alo-choosing/src/keeping.rs` — the file on the disk, whole or not at
  all.
- `crates/alo-choosing/src/unwritten.rs` — `NotWritten`, the six ways a change is
  not made.
- `crates/alo-choosing/src/written.rs` — the shape on the disk, now travelling
  both ways.
- `crates/alo-choosing/src/words.rs` — four new strings under `choosing.change.*`.
- `crates/alo-choosing/src/lib.rs`, `src/testing.rs` — exports and the fixture.
- `crates/alo-choosing/tests/a_persons_choice_reaches_the_machine.rs`,
  `tests/no_agents_door_reaches_these_settings.rs`.
- `docs/contracts/person-settings.md` — a new *Writing it* section; the contract
  said this file "is written by a settings panel" and nothing said how.
- `docs/autonomy/v0-01-evidence.md` — *add your own provider in Settings* no
  longer owes *by hand*.
- `docs/autonomy/v0-01-delivery-plan.md` — task 24 marked done. Task 25 was
  already written.

## Decisions, and why

### It is in `alo-choosing` and not a crate of its own

Task 23 made `alo-changing` a separate crate, and this one deliberately is not.
The difference is where the shape lives. `alo-changing` composes two existing
crates — `alo-remembering` writes the grants file and `alo-capability` owns what
a grant means — so it adds an order rather than a shape. Here, **the shape on the
disk is `alo-choosing`'s and nobody else's**. A writer anywhere else would be a
second declaration of `format`, `[answers]`, `[[brought]]`, `[[provider]]` and
`[reading]`, and the acceptance's *no second parser* would have been a rule
somebody kept rather than a thing that was true.

So `crate::written`'s own types carry both directions. A key renamed is renamed
for the reader and the writer in one keystroke. `writing.rs` holds no shape at
all — it holds the conversions into that one shape and the check below — which
keeps law 4 while keeping the declaration single.

### Nothing is written that this alo OS does not read back as the same settings

`writing::written` serialises the changed settings, parses its own output back
through `written::read`, and refuses unless what comes back equals what went in
— **before a byte reaches the disk**, so the file is untouched.

This is the decision I would most expect to be argued with, so here is the
failure it exists for. `alo_models::Provider` has public fields, and this file
has nowhere to say two of them: the list of model names a provider offers
(`Provider::models`), and a credential kept under a name other than the derived
`provider/<name>` — because the settings file deliberately has no `key` field at
all, and the keyring name comes from the provider's own name. A writer that
simply dropped those would answer a settings panel `Ok(())` for a provider that
the machine afterwards describes differently, and **nobody would ever be told**.
The alternatives were to widen the file (which would put a credential's location
in a text file in somebody's home directory, refused) or to trim silently (which
is the class of defect every task in this plan has been built to refuse).

It costs one parse of a few hundred bytes at the moment somebody clicks
something. What it buys is that *a change reads back through `Settings::at`* is a
property of the code rather than of the tests that happened to be written.
`a_provider_this_file_cannot_hold_is_refused_rather_than_trimmed` and
`a_credential_kept_somewhere_this_file_cannot_name_is_refused` are the two real
cases, and `a_choice_the_file_would_change_on_the_way_back_is_refused` is a
third: `Picked::FromAProvider` has public fields, so a name with space around it
can be built by hand and would come back trimmed.

`written::a_key_for` is now the one place the keyring name is derived, asked by
both directions, so the two cannot drift apart.

### What is written is `format = 2`, always

`ALSO_READ` exists so a machine configured before providers keeps working; it is
not a menu a writer picks from. A writer that chose the oldest shape a value
happened to fit would grow one branch per format for ever, each exercised only by
somebody who has downgraded. What downgrading costs is already written into
`docs/contracts/person-settings.md` and has always been *your settings rather
than your choice*.

### Opening settings writes nothing

`Choosing::at` on a machine nobody has configured makes no file, no folder and no
`format` line. ADR 0016 refuses a default nobody chose and ADR 0025's reading
turns on exactly that distinction, so a value that wrote *nothing chosen* into
somebody's file on being opened would be this crate writing in the one file
ADR 0016 keeps for them. `opening_settings_nobody_has_written_writes_nothing` and
`nothing_is_written_for_somebody_who_has_chosen_nothing` are that, and both check
the folder as well as the file.

For the same reason, **adding a provider is not choosing it**. It goes on the
list and what answers this person's questions does not move — measured in
`a_provider_added_and_then_chosen_reads_back_as_both`. A machine that switched to
the last thing somebody added would be choosing on their behalf, and a hosted
provider is the one place that would cost them something no undo returns.

### Settings that do not read are refused rather than replaced

`Choosing::at` returns `NotSet` for a file that is there and does not hold. This
matters more on the way out than on the way in: a value that read a typo as
*nothing chosen* would replace whatever somebody had typed at the first thing
they changed, and the keystroke that was about to fix it would be gone.
`a_file_the_machine_refuses_is_not_replaced_by_a_change` writes a real typo
(`catalog` for `catalogue`) and reads the file back byte for byte afterwards.

### The folder **is** made here, and `alo-remembering`'s is not

`/var/lib/alo` belongs to the image, so a missing one means the machine is not an
alo OS machine and making one would turn a typo into a second list nobody reads.
`$XDG_CONFIG_HOME/alo` is the opposite: it belongs to the person, nothing has
ever made it, and *the first choice somebody makes* is precisely the moment it
does not exist. `0700` for the folder and `0600` for the file on a host with
modes; on Windows the ordinary inherited permissions, which is where this
crate's gates are run and which `crate::place` already records the other half of.

### A fifth sentence, and the clause that makes it different

Four strings arrived, under `choosing.change.*`. The ten this crate already had
end *nothing in the file has been used* or *nothing has been chosen to answer
questions* — the machine is running on no choice at all. The four new ones end
**nothing in your settings has been changed**, which says the opposite: whatever
was chosen is still in force and the thing that failed was the change. A person
who had just clicked something and read the first would reasonably conclude their
machine had forgotten what they chose last month. `words.rs`'s
`every_sentence_says_what_the_machine_did_about_it` gained the third consequence
rather than being loosened; every new string still names the file and still
counts nothing.

Two of `NotWritten`'s six reasons — the same weights twice, a provider name
already taken — are `alo_models::WeightsError` and `ProviderError` carried in
their own words rather than reworded, exactly as `NotSet::NotAProvider` already
does: those refusals are about a **list**, the list is `alo-models`', and a second
sentence here would be two accounts of one moment with only one of them kept up
to date. They are therefore the two that do not name the file, which is stated in
`unwritten.rs` and held by
`the_reasons_this_crate_words_itself_all_name_the_file` rather than discovered
later. `src/testing.rs` gained `alo_models`' list for it, and its header now says
so instead of claiming this crate says nothing of anybody else's.

## The knock question, answered by reading the daemon

The task asked for this to be read rather than assumed either way. **There is
nothing to knock about.**

`alo-agentd` does not hold a person's settings. `crates/alo-agentd/src/questions.rs`
reads the file **once a turn, at the first question of that turn**:
`Questions::a_new_turn` forgets and `Questions::what_answers` looks, and that
file's own header states the reasoning — a service that re-read on every question
pays for a file it almost never needs, and one that read at start-up would tell
somebody who has just picked a model that nothing answers questions until the
machine restarts. The daemon measures it itself, in
`questions.rs`'s `a_new_turn_reads_the_file_the_person_has_just_written`.

So a change written through `Choosing` is in force for the next question anybody
asks. A knock would be a message telling a service to do what it already does.
That is the opposite answer from the grants, which a daemon holds from start-up
and which task 23 therefore had to tell about through
`alo_protocol::FromAPerson::Granted`.

What is held instead is the pair of facts that keep it true:

- **this crate could not knock** — no `alo-protocol`, no daemon, no turn, no
  record, no capability in the manifest
  (`the_persons_writer_has_nothing_to_knock_with`);
- **the daemon cannot write** — `alo-agentd` names this crate on purpose, and
  every `.rs` under `crates/alo-agentd/src` is read off the disk and shown to
  name `Choosing` nowhere (`the_daemon_reads_these_settings_and_never_writes_them`).

The second is the one that matters, and it is read off the repository rather than
asserted about it, in the shape `alo-collected` and `alo-citing` settled: a check
that only ever reads its own fixtures can pass while the disk says something
else. It is the check that fails the day somebody inside the daemon reaches for
the convenient thing, which is to have the machine write a person's choice on
their behalf.

## What this does not do

- **No verb was added, and there is nowhere here for one.** A person picking
  which model answers their own questions is not an agent doing something —
  the answer `alo-clipboard` gave about copy and paste. Held by
  `nothing_here_declares_a_verb`, which checks both the `src/verbs.rs`
  convention `alo-by-hand` walks and that this crate does not name
  `alo-capability`.
- **The organisation's file is not touched.** `/etc/alo/agentd.toml` is
  ADR 0004's and ADR 0016's bound, it has an owner who is not the person, and
  nothing in this crate has ever read or written it.
- **Nothing is chosen for anybody and nothing is pre-selected.** No default is
  written that a person did not choose; ADR 0016 stands and ADR 0025's reading
  stands.
- **No pixels.** There is still no Settings panel; drawing one is the compositor
  lane's, and the evidence ledger now says that is what the promise owes.
- **No removal doors.** Nothing here removes weights or a provider from a
  person's list, because nothing has a surface that would offer it yet. The
  shape that makes removal safe is already measured —
  `the_file_is_written_whole_rather_than_added_to` — so the door, when a surface
  wants one, is three lines and no new argument.

## Verification

Run from `C:\dev\alo-os-claude` on Windows 11 Pro, 2026-09-11.

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy -p alo-choosing --all-targets -- -D warnings` | clean |
| `cargo test -p alo-choosing` | 128 passed, 0 failed |
| `cargo clippy -p alo-saying --all-targets -- -D warnings` | clean |
| `cargo test -p alo-saying` | 68 passed, 0 failed |
| `cargo clippy -p alo-reconciling --all-targets -- -D warnings` | clean |
| `cargo test -p alo-reconciling` | passed |
| `cargo clippy -p alo-citing --all-targets -- -D warnings` | clean |
| `cargo test -p alo-citing` | passed |

`alo-saying`, `alo-reconciling` and `alo-citing` are gated because this change
reaches them without touching their source: four new strings enter the machine's
one vocabulary, `docs/autonomy/v0-01-evidence.md` is the ledger `alo-reconciling`
holds to `docs/features.md`, and the new files and documents carry ADR citations
`alo-citing` resolves.

**Not run here:** the full workspace suite, deliberately — the supervisor runs it
after this, and `docs/autonomy/LOOP.md` records why a worker does not.

### Second pass, 2026-09-11 — re-gated after a machine that could not build

The first handoff of this task was refused before a single gate ran, and not for
anything in the change: the disk this checkout sits on had **11.75 GiB** free and
the preflight in `docs/autonomy/SHARED_MAIN.md` asks for 12 GiB, so the
supervisor stopped rather than start a build that would have failed as a linker
that could not open a file — which reads like a broken change and is not one.
Nothing was staged, committed or pushed, and no source was altered on this pass.

**What was freed, and what deliberately was not.** Only this checkout's own
regenerable build output: `target/doc` (rustdoc, 4 MB), `target/debug/examples`
(example binaries, 377 MB) and the `.pdb` debug-symbol files under
`target/debug/deps` (2.34 GiB). Nothing shared was touched — not the desktop
worker's `C:\dev\alo-os` or its target directory, not the Cargo registry, not
anything under a system folder. All three are outputs this same checkout
reproduces on demand; none is another party's to lose. Free space afterwards:
**13.4 GiB**. Deleting the rest of `target` was available and was not taken: it
would have forced a full workspace rebuild for no gate that needed one, and the
whole point of the preflight is headroom rather than an empty disk.

Every gate was then run from `C:\dev\alo-os-claude` on the tree as it stands:

| Command | Result |
|---|---|
| `cargo fmt --all` | clean, nothing reformatted |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean, exit 0 |
| `cargo test -p alo-choosing` | 128 passed, 0 failed |
| `cargo test -p alo-saying` | 68 passed, 0 failed |
| `cargo test -p alo-reconciling` | 35 passed, 0 failed |
| `cargo test -p alo-citing` | 31 passed, 0 failed |

Each of the seven pieces of evidence was then run **on its own**, the way
`tools/kernel-loop/src/evidence.rs` runs it — `--exact --include-ignored`,
against the named target — and each reported exactly one test passing.

The clippy gate on this pass was the **whole workspace** rather than
`-p alo-choosing`, which is the one thing the first pass did not establish: four
new strings enter a vocabulary every crate loads, and a check that only ever ran
over the crate that declared them would not have seen a crate that reads them.

**Not measured anywhere:** the Unix mode on a real Linux machine. The mode
assertion in `keeping.rs` is `#[cfg(unix)]` and does not compile on the host
these gates ran on, so *the settings go down `0600` and the folder `0700`* is
written and compiled but has been executed on no machine. It is named here rather
than claimed.

## Proposed changelog entry

**A person's choice about their model can be saved.** alo OS could always read
which model answers your questions, which weights you brought, which providers
you added and which language you read — and could never write any of it, so
every one of those was a choice you could only make by editing a file by hand.
Now a change lands in your own settings file whole, or it does not land at all:
a change that fails leaves the file exactly as it was and says so, a file that
alo OS cannot read is never written over, and nothing is chosen or filled in on
your behalf. The change is in force for the next question you ask.

## Proposed queue and roadmap updates

None. `docs/features.md` is unchanged, no v0.01 promise is ticked, and *On the
machine* does not move. The evidence ledger's entry for *add your own provider in
Settings* records what is closed and what is still owed.
