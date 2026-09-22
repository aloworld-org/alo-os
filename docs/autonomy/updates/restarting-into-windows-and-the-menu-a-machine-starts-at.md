# Restarting into Windows, and the menu a machine starts at

**Date:** 2026-09-22
**Workstream:** `docs/autonomy/v0-5-the-installer-plan.md`, task 16 — the alo OS
side of *alongside Windows*, split out of task 4 as the part that needs no
virtual machine.
**Contributor:** the third PC (`AGAI01`), one worker, one working tree.
**Decision it rests on:** [ADR 0062](../../decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md),
accepted by the owner 2026-09-21, with its three terms.
**Status:** ready for integration. Nothing in it is ticked on a machine, which
is what this task's own constraint asks.

## What a person gets out of this

A computer that has both alo OS and the Windows it came with now has one place
to choose between them. When it is switched on it draws a short list — alo OS,
Windows — with a five-second countdown, starting at whichever was chosen last.
Choosing Windows hands the machine to Windows's own start-up program and Windows
starts exactly as it always did. From inside alo OS a person can say *start
Windows the next time I switch this on*, which is for that one start and changes
nothing else about the machine. And alo OS never mounts the Windows volume: it
reaches Windows by handing Windows's own loader over, never by reading its disk.

## What changed

### A new crate, `crates/alo-starting`

One subject — the menu a machine starts at, and the way out into the Windows
beside it — in ten files beside its root, each with one reason to change.

| file | what it is |
|---|---|
| `systems.rs` | The two systems a machine can start, and the one file Windows is started by, in the loader's spelling and the firmware's |
| `menu.rs` | `Menu`, the start-up menu generated as configuration for the base's own loader, and the refusals for a title or a countdown that could not be written |
| `saved.rs` | `EnvironmentBlock`: GRUB's own environment block, read and written, with every way it can be wrong refused by name |
| `chosen.rs` | `TheStartingChoice`: which system starts when nobody chooses, kept in that block and in no second place |
| `firmware.rs` | `Entry` and the `Firmware` trait — what the firmware reports, and the one thing it is ever told |
| `efi_variables.rs` | `TheFirmware`: that trait over the kernel's own window onto the firmware's variables |
| `wanted.rs` | `Change::RestartIntoWindows`, and the broker verb it is |
| `refusing.rs` | `NotChanged`, `changed_said`, `starts_at_said` — what a person reads, either way |
| `words.rs` | Thirteen sentences, each with a note for whoever translates it |
| `testing.rs` | A start-up entry written the way a firmware reports one, shared by the tests on both sides of the broker's door so the two cannot drift |

### The broker's twelfth verb

`alo_broker::SystemVerb::RestartIntoWindows(Identity)`, named
`starting.windows-next`. It sets the firmware's `BootNext` to the Windows
already on the disk, for **one** start, and leaves the machine's ordinary
start-up order exactly as it was. Writing a firmware variable is privileged,
which is what ADR 0001 §2 puts behind this list; ADR 0062's *what stays as it
was* names `BootNext` from alo OS as the alo OS half of the two one-restart
switches; and task 16 says in as many words that the verb is added, gets its
words, and gets the tests every other verb has.

It **restarts nothing.** The restart is the person's own, afterwards, exactly as
it is for the update verbs — no unit on this road holds `CAP_SYS_BOOT` and the
broker holds no capability at all.

Carried out by `alo_brokerd::NextStart` (`crates/alo-brokerd/src/next_start.rs`)
against `alo_starting::Firmware`, supplied additively through
`Carriers::with_next_start(…)`; the published `Carriers::of(network, proxy,
storage)` keeps its signature and refuses the verb until a firmware is supplied.
`crates/alo-brokerd/src/main.rs` supplies the machine's own.

### Everything the new member of a closed enum touched

`crates/alo-changing-drives`, `-printers`, `-updates` each gained the new arm in
their exhaustive match; `crates/alo-brokerd/src/printers.rs` likewise;
`crates/alo-saying` collects the new vocabulary;
`crates/alo-letting-go/tests/nothing_here_is_a_verb.rs` moved its count of the
broker's list from eleven to twelve, in this change, with the decision named
beside it. That count is a tripwire and it did its job: the list cannot grow
quietly.

### Documentation

`docs/contracts/agent-verbs.md` gains the verb additively — the row in the
table, how the identity of a start-up entry is made, how it is carried out, and
each way it is refused. `docs/booting.md` gains *Alongside Windows: the journey,
step by step*, with **term 2's honest line** in the steps a person reads: a
loader that starts and is then broken is not passed over by the firmware,
because to the firmware it started, so the machine's own boot-menu key is named
as the way to Windows in that case and nothing claims the fall-through reaches
it. Both say plainly that none of it has run on a machine.

## Decisions taken, and why

Nobody was waiting to answer these, so they were decided the way a senior
engineer would and are written down here.

**1. A start-up entry's identity does not include the number the firmware keeps
it under.** The identity is the SHA-256 of `alo-starting entry 1`, a zero byte,
and exactly the bytes the firmware reported — the whole load option, description
and path. An entry names what it starts; the slot it is in is where it is. A
firmware that renumbers its entries would otherwise turn a person's approval
into a refusal. The cost is that the same entry written twice is one identity and
two matches, and that case is **refused** rather than guessed at, which is the
same rule `storage.mount` already follows for two drives of one name.

**2. Windows is recognised by the program its entry starts, never by the entry's
name.** `Entry::starts_windows` looks for `\EFI\Microsoft\Boot\bootmgfw.efi` in
what the firmware reported, in any case. A firmware's names for what it can start
are whatever was typed when the entries were made, in whatever language, and on a
reinstalled machine they are regularly wrong — so a verb called *start Windows
next* that trusted the name would set the machine to start whatever was approved.
This is the refusal that makes the verb's name true, and it is tested.

**3. The menu finds Windows by searching for Windows's own loader**, rather than
by a partition identifier written into the configuration. An identifier learned
once at install and never checked again would be a second copy of a fact about
somebody's disk — exactly the kind ADR 0062's third term refuses — and searching
for the program itself is the same answer computed at every start.

**4. The last choice is written as the Windows entry's identifier, or as
nothing.** Only the Windows entry is ours to name: alo OS's entries are the
base's, made from the kernel each starts, and their identifiers change with every
update. So the reading is *the saved entry is the Windows one, or the machine
starts alo OS* — true of a block the loader wrote when a person chose alo OS at
the menu, of a block where nothing was ever set, and of a block that came through
an update carrying a kernel nobody has seen. Writing *alo OS* writes an empty
value, which the loader reads as *no saved choice* and starts its first entry.
Keeping an identifier for alo OS here would have been a second copy that goes
stale at the next update.

**5. The menu is one generated file and no settings file beside it.** An earlier
shape of this change also wrote `/etc/default/grub` keys (`GRUB_DEFAULT=saved`,
`GRUB_TIMEOUT`). That was dropped: the countdown and the default would then be
written in two places, which is the drift ADR 0062's third term exists to
prevent, with the term's own subject. `custom.cfg` sets both itself, and it is a
whole file of ours beside the base's rather than an edit to anything the base
wrote (ADR 0011).

**6. An entry the reader cannot parse is left out of the list rather than
failing the whole read.** A firmware regularly keeps entries nothing in this
repository has seen, and one of them failing to parse must not take away a
person's way into their own Windows. Leaving it out cannot cause a wrong answer —
an entry that is not in the list can only fail to match what was approved, and a
failure to match is a refusal that changes nothing.

**7. No agent verb was declared, and this is the one thing a reviewer should
look at hardest.** Task 16 frames the change as *a setting, and a verb an agent
may ask for under a grant*, and its acceptance names *the full set of tests every
`SystemVerb` has* — which is what was built. An `alo_capability::Verb` on top
would need a promise in `docs/features.md` with a tier for `alo-by-hand` to
answer against (ADR 0009, and the check is arithmetic rather than a rule), and
**adding a promise to `docs/features.md` is the owner's, not a worker's.** The
broker verb is the road, and it is the shape `updates.apply`, `updates.roll-back`,
`storage.mount` and `storage.eject` already have: carried out from an approved
authority, with a person-facing crate in front of it and no verb of their own on
the agent's list. If the owner wants the agent able to propose it, the promise
comes first and the verb is a small change after it.

## What this does not do, and who owes it

- **Nothing here has run on a machine.** No hardware, no virtual machine, no
  certified laptop. The walk — Windows → alo OS → Windows → alo OS through the
  in-system switches alone, term 1's firmware-order fall-through, the install
  beside a real Windows, the Windows-unchanged hash, and *remove alo OS* — is
  the development PC's, and `docs/booting.md` says so where a reader finds it.
- **Whether the base's loader honours this configuration across a `bootupd`
  update is not measured.** ADR 0062's consequences already put that answer in
  `docs/quirks.md` when task 4 measures it. Nothing was written to
  `docs/quirks.md` here: that file is for where reality and a specification
  disagreed, and nothing has been run for reality to disagree with.
- **Who writes these two files onto a machine, and when, is not built.** It is
  `crates/alo-installing`'s and `crates/alo-installer`'s, which this task may not
  edit.
- **The privileged road for changing the menu's default is not built.** Task 16's
  acceptance asks that alo OS's setting *reads and writes that one place, held by
  a test that finds no second copy anywhere*, and that is what
  `TheStartingChoice` and
  `crates/alo-starting/tests/the_last_choice_has_one_place_and_no_copy.rs` are.
  What they do not answer is how a person in Settings, who is not root, writes a
  file under `/boot`. That is a genuine gap, it probably needs a thirteenth
  member of the closed enum and therefore a decision, and it is written up as
  **task 17 of the plan** rather than left for somebody to find.
- **Fast Startup is not decided here**, as the task says. The sentence that holds
  either way — alo OS never mounts the Windows partition read-write — is written
  and tested.

## Verification

Run from the checkout, in WSL Ubuntu, against a copy of the committed tree at
`/root/alo-trees/this-machine`, building in `/root/alo-builds/this-machine`.
Toolchain `rustc 1.98.1 (48a229cea 2026-09-01)`.

| what | result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets --all-features -- -D warnings` (workspace) | clean, zero warnings |
| `cargo test -p alo-starting` | pass |
| `cargo test -p alo-broker` | pass |
| `cargo test -p alo-brokerd` | pass |
| `cargo test -p alo-changing-drives` | pass |
| `cargo test -p alo-changing-printers` | pass |
| `cargo test -p alo-changing-updates` | pass |
| `cargo test -p alo-letting-go` | pass |
| `cargo test -p alo-saying` | pass |
| `cargo test -p alo-collected` | pass |
| `cargo test -p alo-citing` | pass |
| `cargo test -p alo-image` | pass |
| `tools/kernel-loop` own suite | pass, 152 tests |

**Not run by this worker, deliberately:** the whole workspace suite, which the
supervisor runs after every task; and anything on hardware or in a virtual
machine, which this task may not tick.

### The acceptance, criterion by criterion

| the plan asks | the test |
|---|---|
| the verb exists with its name and words | `alo-broker`, `verbs::tests::every_verb_reads_back_from_its_name_and_argument`; `alo-starting`, `words::tests::every_key_is_one_of_this_crates_and_names_one_string` |
| and the full set of tests every `SystemVerb` has | `alo-brokerd`, `tests/only_the_next_start_approved_is_set.rs` — approved once, one approval one execution, recorded either way, and five refusal paths |
| sets `BootNext` for one restart with the default provably untouched | `the_windows_approved_is_the_next_start_and_the_default_is_untouched`, and `alo-starting`'s `writing_the_next_start_leaves_every_other_variable_alone` |
| GRUB's configuration is generated rather than patched | `it_is_configuration_beside_the_base_rather_than_a_change_to_it` |
| offers both systems, chainloads Windows by the path, counts down | `it_offers_windows_by_handing_windowss_own_program_over`, `it_counts_down_visibly_and_briefly` |
| saves the last choice in its own environment block | `it_starts_at_the_last_choice_and_saves_the_next_one` |
| the setting reads and writes that one place, and no second copy exists | `nothing_outside_this_crate_keeps_a_second_copy_of_the_last_choice`, `the_menu_and_the_setting_agree_about_what_windows_is` |
| `docs/booting.md` carries the journey with term 2's line | the *Alongside Windows* section, step 5 |
| alo OS never mounts the Windows partition read-write | `crates/alo-starting/tests/windows_is_never_mounted.rs`, three tests |

### The refusal paths, which are half the suite

Setting the next start: an identity no entry has; two entries that are the same
entry; an entry that does not start Windows; a firmware that will not say what it
can start; a firmware that will not be written; an approval issued for another
verb; an approval spent already; and a broker built with no firmware at all.

The environment block: bytes that are not a block; a line that is not a setting;
a name no name may be; a value that would end its own line; and more than fits,
where nothing is truncated and nothing is dropped.

The menu: an empty title, one too long, and one carrying a quote, a backslash, a
brace, a dollar, a backtick or a control character — refused rather than escaped,
because the failure it would cause is a machine that starts at a loader's prompt,
which is the one screen with nothing on it to look the answer up with. A
countdown of zero or of more than a minute.

A start-up entry: too short, a name that never ends, a name that is not text, and
a path shorter than the entry says it is.

## Proposed updates to the shared documents

The integration owner's to make; written here as the plan asks.

**`CHANGELOG.md`**, under the current release:

> **Alongside Windows.** A computer with both alo OS and the Windows it came
> with now starts at a short list of the two, with a five-second countdown,
> beginning at whichever was chosen last. Choosing Windows hands the machine to
> Windows's own start-up program, and Windows starts as it always did. From
> inside alo OS, *Restart into Windows* sets the next start to Windows for that
> one start and changes nothing else about the machine. The last choice is kept
> in one place — the loader's own — so the list and the setting can never
> disagree. alo OS never mounts the Windows volume. None of this has yet been
> watched on a machine.

**`ROADMAP.md`:** nothing to tick. Nothing here ran on hardware.

**`docs/autonomy/QUEUE.md`:** task 16 of the installer plan is done; task 17,
*The default a machine starts at, changed by the person who owns it*, is written
in the plan and ready, and depends on 16.

**`docs/autonomy/STATE.md`:** reference this report. The one thing worth carrying
forward is decision 7 above — the agent-verb layer waits on a promise in
`docs/features.md`, which is the owner's.

## Files touched

- `Cargo.lock`
- `Cargo.toml`
- `crates/alo-starting/**` (new crate: manifest, ten source files, three tests)
- `crates/alo-broker/src/verbs.rs`
- `crates/alo-brokerd/Cargo.toml`
- `crates/alo-brokerd/src/carrying.rs`
- `crates/alo-brokerd/src/lib.rs`
- `crates/alo-brokerd/src/main.rs`
- `crates/alo-brokerd/src/next_start.rs` (new)
- `crates/alo-brokerd/src/printers.rs`
- `crates/alo-brokerd/tests/only_the_next_start_approved_is_set.rs` (new)
- `crates/alo-changing-drives/src/wanted.rs`
- `crates/alo-changing-printers/src/wanted.rs`
- `crates/alo-changing-updates/src/wanted.rs`
- `crates/alo-letting-go/tests/nothing_here_is_a_verb.rs`
- `crates/alo-saying/Cargo.toml`
- `crates/alo-saying/src/collecting.rs`
- `docs/autonomy/v0-5-the-installer-plan.md`
- `docs/booting.md`
- `docs/contracts/agent-verbs.md`
- this report
