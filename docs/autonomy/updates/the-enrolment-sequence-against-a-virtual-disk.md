# The enrolment sequence, against a virtual disk — and the sentences that go with it

**Date:** 2026-09-20
**Workstream:** `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`, task 6 —
*Enrolled at install, and recovered — everything a virtual disk can show*
**Contributor:** the broker-and-the-disk lane, on the third PC (`AGAI01`) —
built by its first worker and re-verified by its second, whose account is
*Verified again* below
**Decision:** [ADR 0056](../../decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md),
accepted option C, **moved from *proposed* to *accepted* in the same change that
built it**, exactly as the plan's two-commit rule describes.
**Status:** ready for integration.

## What this is, in one sentence

The installer plan is handed what it needs to enrol encryption during an
install — `alo-encrypting`'s types, one tested sequence of runs of the two rented
tools, and a sentence for every refusal on the road — and the sequence was run
against a **real LUKS2 volume** in the pinned base rather than described.

## What is deliberately not claimed

**`docs/features.md`'s v0.5 *Full-disk encryption, enrolled at install* line is
not ticked by this, and this report is where that is said rather than left for a
reader to infer from a green suite.** ADR 0056 point 2, accepted most
deliberately of all: an install that has never happened on a machine with a chip
has enrolled nothing.

Three facts about the sequence are about a chip rather than about LUKS, and no
virtual disk can show any of them. They are the plan's **task 9**, blocked on a
certified machine existing (`docs/hardware.md` lists none):

- the chip releases the key when the PIN is typed, and the machine reaches its
  desktop;
- a new deployment does not stop it;
- a change to what register 7 measures **does** stop it, and the recovery key
  answers.

Nothing here is an emulated chip standing in for a real one. ADR 0056 rejected
that by name as option D, and measurement 8 is why: the chip a guest gets reports
its manufacturer as *IBM / SW* and carries the simulator's own lockout defaults,
so a run against one would be a program agreeing with itself.

## What changed

### `crates/alo-encrypting` — the sequence, as closed types

Four files added beside the shape task 5 gave the crate. The crate still
**depends on nothing**, opens no file, starts no program and reaches no network:
it *builds* the runs and takes none of them.

- **`src/volume.rs`** — `TheDisk` and `TheVolume`. A disk by the identity udev
  gave it, under `/dev/disk/by-id`; a volume is that disk and a partition
  **number**. Anything that is not one component of udev's characters is refused,
  including `/dev/sda`, `../../sda`, `disk$(reboot)` and a name with a space in
  it; a partition's *name* is refused as a partition, because a partition is a
  disk and a number here. `IT_OPENS_AS` is one fixed name, so the name the volume
  opens under is not a caller's free string either.
- **`src/handing_over.rs`** — `ASecretOnItsWay`: the four secrets an enrolment
  moves (the installer's first key, the person's secret, their new secret, the
  recovery key), each in one named file under `/run/alo-encrypting`. **`/run` is
  a `tmpfs`: it is memory with a path.** A recovery key written there is not on
  the disk it recovers whether or not anybody remembered to remove it, and
  `ONLY_ITS_OWNER_MAY_READ_IT` is `0o600` because the rented tool complains in as
  many words when it is not. There is no constructor taking a path, so a caller
  cannot ask for a file of its own.
- **`src/sequence.rs`** — `TheTool`, `Run`, `OnTheRoad`, `TheSequence`. Four
  things can be asked of an encrypted disk and there is no fifth: enrol at
  install, open, close, change what the person unlocks with. Each is a list of
  runs: which of the two rented tools, which arguments, and which named secret
  has to be in its file first. `Run::what_it_prints_is_a_secret` is true for
  exactly one run on the whole road.
- **`src/refusing.rs`** — `TheDiskRefused`, read off how the tool ended. Exit 2
  is *what was given does not open it*; every other non-zero answer is *the
  rented tool refused and did not say why*, deliberately vague, because guessing
  would mean telling somebody at a machine that will not open that their correct
  recovery key was wrong.

### `crates/alo-enrolling` — new, and small

Twenty-four sentences: one for every refusal `alo-encrypting` can make, and the
five the road itself has to ask for (choose a PIN, choose a passphrase, write
this down, type it back, the disk is encrypted). Each carries a translator's
note. `src/said.rs` maps each refusal to its sentence as an **exhaustive match**,
so a variant added to any refusal stops the crate compiling until somebody has
written the sentence for it — *every refusal on the road is a sentence in the
vocabulary* held by the compiler rather than by a test that counts.

`alo-saying` collects it, so the sentences are in the machine's **one**
vocabulary and a translation file covering the machine accepts them.

### The decision, the plan and the documents

- ADR 0056's status line moved to **accepted**, and its *What the code waits on*
  now says what was built and what still waits on a machine.
- The plan marks task 6 **Done, 2026-09-20**, records what was built, and adds
  *What task 9 inherits from 6*. Task 7's forward reference to *the encryption
  sentences join the table once task 6 lands* now names them.
- `docs/hardware.md` gains **What the security chip must be able to do** — which
  road a machine takes, the three promises, and a table that currently reads
  *not shown on any machine*. ADR 0056 points 3 and 4.
- `docs/quirks.md` gains one entry: exit 2 is the only answer that is a fact
  about a secret, and `systemd-cryptenroll` complains about a key file anybody
  but its owner could read.

## Decisions I made, and why

**1. The sentences are a separate crate rather than a module of
`alo-encrypting`.** The obvious thing was `alo-encrypting/src/words.rs`. It is
wrong: `alo-encrypting`'s manifest says *nothing, and the absence is the
argument*, and `tests/the_key_is_never_kept_on_the_disk_it_recovers.rs` refuses a
dependency added there — a crate that holds a recovery key for the length of one
screen must not have a serialiser, a client or a logger in reach. `alo-strings`
depends on `serde`. Weakening that guard to save a crate would have traded a real
property for a small convenience, so the sentences live in `alo-enrolling` and
the dependency goes one way: nothing in `alo-enrolling` is reachable from a value
holding a secret.

**2. The sequence is data, and the test is what runs it.** ADR 0056 asks
`alo-encrypting` to gain *the sequence as closed types* and the integration test
that runs it. Those are two different capabilities and only one of them may be in
that crate, so `TheSequence` is a description — program and arguments — and
`tests/the_sequence_against_a_virtual_disk.rs` is what starts anything. It is
also what the installer wants: it runs the sequence itself, and would have had to
unpick a crate that ran it.

**3. The volume is a disk's udev identity and a partition number, and the test
makes a virtual disk answer to one.** The alternative — a `TheVolume::at(path)`
— is exactly the free string ADR 0056 forbids, and a *virtual disk* variant in
the production type would have put test scaffolding in the road a root program
walks. So there is one road, and the acceptance makes
`/dev/disk/by-id/virtio-alo-target-part4` a symlink to the disk image inside the
container. The test therefore exercises the real path construction rather than a
second one written for it.

**4. Secrets travel in key files under `/run`, not on standard input.** ADR
0056's measurement 6 is that `systemd-cryptenroll` has no option for the secret
being enrolled and waits forever for a terminal; `cryptsetup` takes `--key-file`
and `--new-keyfile`, and a run needing both cannot have two standard inputs. A
`tmpfs` file is what a real installer uses, it is memory rather than disk, and it
gives every run of both tools one uniform way to be handed a secret.

**5. `changing_what_the_person_unlocks_with` adds the new secret before removing
the old one.** Two runs on the passphrase road rather than `luksChangeKey`, so
that a machine interrupted between them opens with either secret rather than with
neither. Held by a test.

**6. Three of the four `NotARecoveryKey` variants share one sentence.** *Not
eight groups*, *a group is not eight* and *not the alphabet* are the same fact to
the person in front of the machine — alo OS made something that is not a key, and
nothing is enrolled — and the shapes stay in the refusal for the log.

## Verification

Run from `/root/alo-trees/this-machine` on `AGAI01` (Windows Server 2022, WSL 2
Ubuntu, kernel `6.18.33.2-microsoft-standard-WSL2`), with
`CARGO_TARGET_DIR=/root/alo-builds/this-machine`, `cargo 1.98.1`. The Windows
host cannot build `ring` (no `gcc.exe`), so `-p` tests for crates that reach
`alo-saying` are run on the Linux side, which is where the gates run anyway.

| What | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo doc --workspace --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean |
| `cargo test -p alo-encrypting` | 72 passed, 2 ignored |
| `cargo test -p alo-enrolling` | 10 passed |
| `cargo test -p alo-saying` | 63 passed |
| `cargo test -p alo-collected` | pass |
| `cargo test -p alo-citing` | pass |
| `cargo test -p alo-conforming` | pass |
| `cargo test -p alo-reconciling` | pass |
| the supervisor's `fmt`, `clippy` and `test` in `tools/kernel-loop` | clean, 151 passed |
| `cargo test -p alo-encrypting --test the_sequence_against_a_virtual_disk -- --ignored` | 2 passed, 96 s |

The supervisor's own tests are in the list because this change edits
`tools/kernel-loop/src/who_owns.rs`: that file holds a test asserting which
crates the broker plan's header says it owns, and the plan now owns three rather
than two. Changing the plan without changing that line is a `main` that does not
gate, which is the failure the file exists to prevent.

**The virtual-disk acceptance**, run under `podman` in
`quay.io/fedora/fedora-bootc:42@sha256:077182b6…` with `cryptsetup 2.8.4` and
`systemd-cryptenroll` 257, on a 64 MiB LUKS2 volume:

- the four runs of the enrolment happen in `THE_ROAD`'s order;
- `systemd-cryptenroll --recovery-key` wrote **72 bytes to stdout** — and nothing
  else on the road printed anything — which `RecoveryKey::as_printed` read, the
  person typed back, and `Enrolment::made` could not have been written without;
- the person's secret **opened** the volume; the recovery key **opened** it;
- the installer's own first key, a secret that was never this disk's, and a
  recovery key one character wrong were each refused, each **exit 2**, each read
  as `TheDiskRefused::WhatWasGivenDoesNotOpenIt`;
- the secret was changed: the new one opened it, the old one stopped opening it,
  and the recovery key still opened it;
- the recovery key is **nowhere in the disk's 64 MiB** — nor with its dashes
  removed, nor is the person's passphrase — while `LUKS` is, so the search read
  the right file.

That is the passphrase road — ADR 0054's option B, what every machine with no
usable chip gets — shown **end to end**, and five of the chip road's six runs
with it.

Each of the ten evidence lines in the handoff was also run **on its own**, with
`--exact --include-ignored`, and each reported exactly one test passing.

**Not run, and not claimed:** `cargo test --workspace` (the supervisor's, and
deliberately not this worker's); anything on a machine with a chip; anything that
installs onto a real disk; the nine gates. No physical hardware was touched, and
no real disk was encrypted by any test.

## Verified again, 2026-09-20, by the second worker on this task

**Contributor:** the broker-and-the-disk lane, second worker, same machine
(`AGAI01`). **Why there is a second worker:** the first handoff was refused
before a single crate was compiled, and not for anything in this change. The
supervisor's pre-flight said, verbatim:

> there is less than 12 GiB free on `/root/alo-builds`, which is the filesystem
> the gates build on — they build in `$HOME/alo-builds/this-machine`, and
> `/root/alo-builds` has 8 GiB free. A build needs more than that. Nothing was
> staged, committed or pushed.

That is `tools/kernel-loop/src/where_it_builds.rs` doing exactly what it was
written to do: refusing to start a run that would fail as *a linker that could
not open a file*, which reads like a broken change and is not one. **No code,
test or document was changed to get past it, and nothing was deleted.** The
refused handoff is kept, as that module intends, at
`.kernel-loop/refused/1789937628.toml`.

**Why this is not a second report.** `docs/autonomy/updates/README.md` is one
task, one report, one writer, and this is the same task on the same lane, still
unpublished — the plan and ADR 0056 already name this file. A second file would
have meant a second name in the plan for one piece of work, and a reader looking
for the task's evidence finding half of it. So the account of the second run is
here, attributed, rather than beside it.

### What was re-run, and what it said

Everything below was run in the foreground, on its own, and waited for. Same
place as the first run — `/root/alo-trees/this-machine` under WSL 2 Ubuntu
(`6.18.33.2-microsoft-standard-WSL2`), `CARGO_TARGET_DIR=/root/alo-builds/this-machine`,
`cargo 1.98.1` — after confirming that tree is byte-for-byte the checkout for
every path this change touches (`diff -r` over `crates/alo-encrypting`,
`crates/alo-enrolling`, `crates/alo-saying`, `tools/kernel-loop/src`, `docs`,
`Cargo.toml` and `Cargo.lock`: no difference).

| What | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy -p alo-encrypting -p alo-enrolling -p alo-saying --all-targets -- -D warnings` | clean |
| `cargo clippy -p alo-collected -p alo-citing -p alo-conforming -p alo-reconciling --all-targets -- -D warnings` | clean |
| `cargo doc -p alo-encrypting -p alo-enrolling -p alo-saying --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean |
| `cargo test -p alo-encrypting` | 72 passed, 2 ignored |
| `cargo test -p alo-enrolling` | 10 passed |
| `cargo test -p alo-saying` | 68 passed |
| `cargo test -p alo-collected` | 19 passed |
| `cargo test -p alo-citing` | 31 passed |
| `cargo test -p alo-conforming` | 9 passed |
| `cargo test -p alo-reconciling` | 35 passed |
| `tools/kernel-loop`: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` | clean, clean, 151 passed |

The four vocabulary crates are in the list because this change declares a new
crate's words into `alo-saying`, and what reads that vocabulary back — the
collected list, the citations, the conformance check and the reconciliation — is
where a word added in the wrong shape shows up.

**The virtual-disk acceptance, run again, each test on its own**, under `podman
4.9.3` in `quay.io/fedora/fedora-bootc:42@sha256:077182b6…`:

- `the_whole_sequence_runs_against_a_real_virtual_disk` — 1 passed, 133 s;
- `the_recovery_key_is_not_on_the_disk_it_recovers` — 1 passed, 78 s.

**Every one of the ten evidence lines in the handoff was run on its own**, with
`--exact --include-ignored`, and each reported exactly one test passing and
nothing else selected.

### One thing the Windows side cannot do, recorded so nobody re-measures it

`cargo fmt --all` **cannot be run from the Windows checkout**: this workspace has
97 members, `cargo fmt` hands rustfmt every path in one command line, and Windows
answers `The filename or extension is too long. (os error 206)` before rustfmt
starts. It is not a formatting failure and there is nothing in the tree to fix.
The equivalent on that side is `cargo fmt -p <member>` for each member — run for
all 97 here, which changed nothing — and `cargo fmt --all --check` on the Linux
side, which is where the gates run anyway, is clean.

### What still needs a person, and it is not in this change

`/root/alo-builds` is a 79 GiB ext4 filesystem on `/root/alo-builds.ext4`, and it
had **8.0 GiB free** when this task's gates were refused and **8.0 GiB free**
after they were all re-run here — this change's own compilation cost about half a
gibibyte, because the shared build directory already held the workspace. What
holds the rest:

| | |
|---|---|
| `/root/alo-builds/this-machine` | 65 GiB — the per-machine build directory every lane's gates share |
| `/root/alo-builds/this-machine-installing` | 694 MiB — a per-checkout directory from the scheme `where_it_builds.rs` replaced |
| `/root/alo-builds/lost+found` | 16 KiB |

**Nothing here removes any of it, deliberately.** `where_it_builds.rs` says so in
its own words — *a build directory holding an afternoon of compilation may belong
to a lane that is merely idle, and a supervisor that tidied up would be a
supervisor that can throw work away* — so whether one goes is a person's
decision, and the supervisor's refusal said the same thing. Lowering
`THE_RESERVE` would be weakening a gate and is not on the table.

Two ways a person can make the room, for whoever picks this up:

1. **Grow the filesystem**, which deletes nothing:
   `truncate -s +8G /root/alo-builds.ext4 && losetup -c /dev/loop0 && resize2fs /dev/loop0`.
   It costs the same 8 GiB from the Windows volume behind WSL's own disk, which
   had 32 GiB free.
2. **Give back a build directory**, which costs a recompilation — the 694 MiB one
   is from a scheme nothing uses any more; the 65 GiB one is what makes every
   lane's next gate run take minutes instead of an hour.

The first is the cheaper of the two by a long way, and neither is this worker's
to make.

## Limitations

- The chip road's fifth run is built and never run. That is task 9, and it is
  `- [x] The code.` until a certified machine exists.
- The integration test is `#[ignore]`d and run by name, like every other test in
  this repository that needs a container. It **fails** rather than skips when
  `podman` is missing.
- The sequence assumes the pinned base's paths (`/usr/sbin/cryptsetup`,
  `/usr/sbin/systemd-cryptenroll`). A test holds that this file's base and
  `image/Containerfile`'s are the same digest.

## Proposed shared-document updates

Not edited here (`docs/autonomy/SHARED_MAIN.md`), for the integration owner:

- **`CHANGELOG.md`** — *Full-disk encryption now has a tested sequence: the
  machine's disk is made into an encrypted volume, a recovery key is made and
  shown once for the person to write down and type back, and the way they open
  the machine is enrolled only after that. Every refusal on the way is a sentence
  in the person's own language. Shown against a real encrypted volume; not yet
  shown on a machine with a security chip.*
- **`ROADMAP.md` / `docs/features.md`** — **no line ticked.** The v0.5 encryption
  line waits on the plan's task 9.
- **`docs/autonomy/QUEUE.md`** — task 6 done; task 9 remains blocked on a
  certified machine.
- **`docs/autonomy/STATE.md`** — reference this report.
