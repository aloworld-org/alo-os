# ADR 0119 — The capability to give a directory away goes to the smallest thing that can hold it

**Status:** accepted — **its second measurement named a parent the then-specified
path did not have**, and
[ADR 0120](0120-the-undo-folder-is-roots-own-and-not-inside-the-persons.md) moved
the path rather than widening the grant. The one capability stands.
**Date:** 2026-10-17
**Context:** [ADR 0118](0118-more-than-one-person-on-a-machine.md) (more than one
person on a machine) left this as *the next record*, and said how to write it:
*one component must gain it, narrowly, with a test holding it to that. Which one
is the next record, and it should be written against a measurement rather than a
preference.* Also
[ADR 0045](0045-what-undoing-rewinds-to.md) term 8 (who writes a bracket),
[ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
(a snapshot before each run),
[ADR 0018](0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md) (one
privileged component), `image/`, `CLAUDE.md` law 5

## The decision in one line

A **single new privileged helper**, `alo-giving-a-folder`, gains
**`CAP_CHOWN` and nothing else**, does one thing — make a person's directory
under `/var/lib/alo/undo` and give it to them — and is held to that by a test
over its own source; **neither `alo-compositor` nor `alo-sessiond` gains any
capability.**

## What is blocked, and why nothing on the machine can do it

ADR 0045 term 8 says `/var/lib/alo/undo` is `0711 root:root`, each person's
directory inside it is `0700` and theirs, and a privileged step makes both. That
folder is where the agent's undo snapshots go. Without it, ADR 0064's *a
snapshot before each run, so it can be undone* cannot be kept — a person who
lets the agent run a command cannot put their files back.

**Nothing on this machine can make it.** Measured, and ADR 0118 term 4 records
that both lanes measured it independently:

```text
alo-sessiond.service     User=root   CapabilityBoundingSet=   (empty)
alo-compositor.service   User=root   CapabilityBoundingSet=   (empty)
grep -rl CAP_CHOWN image/                                     0 files
```

The only unit in the image holding any capability is `alo-boundaryd`
(`CAP_BPF CAP_SYS_ADMIN`), which is ADR 0018's loader and is neither.

## What was measured, and it decides two things rather than one

**2026-10-17, on kernel 7.0.0-31-generic, with `capsh`.**

**One: `CAP_CHOWN` is necessary.** With every capability *except* that one, root
cannot give a directory away:

```text
capsh --drop=cap_chown -- -c "chown 65534 <dir>"
  chown: changing ownership of '<dir>': Operation not permitted
  owner afterwards: 0
```

**Two: `CAP_CHOWN` alone is sufficient — for the whole job, not just the
chown.** With only that one capability in the bounding set, all three steps
work:

```text
capsh --caps="cap_chown+eip" -- -c "mkdir <0711 root dir>/ada
                                    && chmod 0700 <dir>/ada
                                    && chown 65534 <dir>/ada"
  owner afterwards: 65534    mode afterwards: 700
```

`mkdir` and `chmod` need nothing, because the helper owns what it has just
made. **So the grant is one capability and not a set**, and that is a
measurement rather than an aspiration: there is no second capability to argue
about.

**Read the parent in that transcript.** It is `<0711 root dir>` — root-owned and
root-writable — and that is load-bearing. `/var/lib/alo/undo`, the path ADR 0045
term 8 specified when this was written, had no such parent: `/var/lib/alo` is
`0700` and the **person's**, so the first thing the built helper did on a real
machine was fail with `Permission denied` **as root**, because this bounding set
drops `CAP_DAC_OVERRIDE`. The sentence above is true of what it measured and was
not true of where the folder was going to be, which is the gap between a correct
measurement and the right one.
[ADR 0120](0120-the-undo-folder-is-roots-own-and-not-inside-the-persons.md) has
the failure and the four readbacks, and it moved the folder to a parent root owns
instead of granting a second capability — so *one capability and not a set* came
out of it intact, and the transcript above is now a measurement of where the
folder actually lives.

## Why a new helper rather than either component that is already root

The question ADR 0118 left is *which component*, and the three candidates differ
by two orders of magnitude. Measured in this repository on the same day:

| candidate | lines of Rust | direct dependencies | what else it does |
|---|---|---|---|
| `alo-compositor` (from `alo-shell`) | **113,587** | 51 | draws, reads input devices, takes a person's password |
| `alo-sessiond` | **2,160** | 5 | answers one knock on one socket |
| a new helper | **~100** | 1 or 2 | makes one directory and gives it away |

`alo-enter-the-box`, written yesterday for the same kind of job, is **62 lines**,
which is what *~100* is estimated from rather than hoped for.

**A capability belongs to the smallest thing that can hold it**, and the numbers
are what make that sentence actionable rather than a slogan:

- **`alo-compositor` is the worst of the three** and it is the tempting one,
  because the account-creating write already happens inside it
  (`alo_accounts::kept`, reached from `alo-greeting::makes_an_account`). It is
  the screen a person types a password into, it parses input devices and fonts,
  and it is fifty-two times the size of the next candidate. Giving *that* the
  power to take ownership of any file on the machine, in order to save a
  process, is the trade this decision exists to refuse.
- **`alo-sessiond` is better and still wrong.** Its unit says
  `CapabilityBoundingSet=` with nothing after it, and
  `tests/this_crate_cannot_do_what_its_unit_forbids.rs` exists because that
  claim was once broken: its header records `Command::uid()` shipping on
  2026-10-14, passing clippy, rustdoc, 72 tests and CI, and failing on a booted
  machine. **That test's failure message already answers this ADR**: *adding the
  capability is a decision and not a fix.* This is that decision, and it says no.
- **A helper can be read in a minute**, which is the only review that scales.
  `CLAUDE.md` law 4 is about the file nobody wants to open being where the
  security bug lives; the same argument applied to a capability says it belongs
  in the file everybody can read.

## What the helper is, and what holds it narrow

- **`alo-giving-a-folder`**, in `/usr/libexec` beside the loader and the opener,
  because nobody runs it by hand.
- **`AmbientCapabilities=CAP_CHOWN`** and
  **`CapabilityBoundingSet=CAP_CHOWN`**, so it can hold that one and can never
  hold another.
- **Two arguments and no options**: the account number, and the folder the
  machine keeps undo in. No configuration file, nothing from a network, nothing
  from a person.
- **It refuses anything that is not under the folder it was told**, resolving
  both paths before comparing — the fault
  `alo-running`'s `entering.rs` records, where `Path::starts_with` is lexical
  and `inside/../elsewhere` passes.
- **It makes a directory and gives it away. It never removes one**, which is
  `alo-letting-go`'s, needs `CAP_SYS_ADMIN`, and is a different grant to a
  different unit.
- **A test over its own source holds it to that**, in the shape
  `alo-bounding`'s `a_turn_is_this_thread.rs` and `alo-running`'s
  `only_one_place_in_this_crate_starts_a_program.rs` already have: the one place
  that may `chown` is named, and a second one fails the build.

## Who calls it, and why that is two roads and not one

ADR 0118's term is that **adding a person is a different road with different
authority** — authorised by somebody already signed in, at a screen reached
after sign-in — and that `makes_an_account`'s refusal once an account exists
stays, because it is what stops a surface making a second *first* account.

So the helper is started from **both** roads and widens neither:

1. the first account, from `makes_an_account`;
2. a person added later, from whatever ADR 0118's route becomes.

**It is idempotent, and that is what makes two callers safe.** Making a
directory that is already there is not a failure; what it does then is put the
mode and the owner back, so a folder somebody widened by hand is narrowed again
at the next call. *Measured while building this: a `DirBuilder`'s own `mode` is
ignored for a directory that already exists, so the mode is set explicitly
afterwards or a widened folder stays wide for ever.*

## Consequences

- **One unit in the image gains one capability**, and `image/` goes from zero
  mentions of `CAP_CHOWN` to one. That is a real increase in what the machine
  can do and is the price of the undo working at all.
- **`alo-sessiond` and `alo-compositor` keep their empty bounding sets**, and the
  tests that hold them to that keep passing unchanged. Nothing about this
  weakens either claim.
- **`alo-letting-go` is unaffected.** It needs `CAP_SYS_ADMIN` to remove a
  read-only snapshot (ADR 0045's amendment of 2026-09-21), it is a different
  unit, and this grants it nothing.
- **A second person's folder is not a case this defers.** The helper takes an
  account number, so the second road needs no second mechanism — which is what
  ADR 0118 asked for when it said the obligation should be compiler-visible
  rather than remembered.
- **`docs/hardware.md` gains nothing.** `CAP_CHOWN` is not a kernel feature a
  certified machine might lack.

## Alternatives rejected

**Grant it to `alo-compositor`**, where the account is already written.
Rejected on the measurement: 113,587 lines, 51 dependencies, a password prompt
and input-device parsing. The saving is one process; the cost is the power to
take ownership of any file, held by the largest and most exposed component on
the machine.

**Grant it to `alo-sessiond`.** Rejected, and its own test's words are the
reason: *adding the capability is a decision and not a fix.* 2,160 lines is
better than 113,587 and worse than 100, and that unit's whole claim is that it
grants nothing.

**No capability anywhere: let the person's own session make its directory**,
with the parent mode allowing it. Rejected because the parent would have to be
writable by others, and then one person can pre-create a directory named for
another — which the sticky bit does not prevent, and which ADR 0118 makes a real
case rather than a hypothetical one. **It is the only alternative that costs no
capability, and it trades a power for a squatting hole**, which is the worse of
the two on a machine that now expects more than one person.

**`systemd-tmpfiles`.** Rejected: it runs at boot and does not know who the
people will be, which is ADR 0045 term 8's own argument for why image build time
cannot do this.
