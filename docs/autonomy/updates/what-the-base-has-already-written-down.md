# What the base has already written down

**Date:** 2026-09-21
**Workstream:** v0.5 — the machine keeps itself
(`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, task 12, *What the base
has already written down*)
**Contributor:** Claude Code worker in `C:\dev\alo-os-3` on the development PC,
for the owner
**Status:** ready for integration. Task 12 is complete and marked done in the
plan.

## What this task was

Task 11 measured the real base refusing an unprivileged caller and decided the
fix without writing it. This is the fix, and it is three things:

1. the **measurement** task 11 left open — whether a machine installed straight
   from the registry records the registry's digest — made before any code;
2. the **road** `alo_updating` gains so that *which build is this machine
   running* is answerable to the person, and the narrowing of ADR 0011 it costs;
3. the **two `ext4` installs** in this crate's own virtual-machine tests, which
   measure a filesystem alo OS no longer ships.

## The machines

Two real bootc machines were installed and booted for this task, and they are
named here so that no line below can be mistaken for a test with a stand-in
behind it.

| | `alo-task-12-older` | `alo-task-12-pinned` |
|---|---|---|
| Image | `ghcr.io/aloworld-org/alo-os@sha256:48bd5f31…` — release 0.0.4, the one task 11 measured | `ghcr.io/aloworld-org/alo-os@sha256:6c9abbc5…` — release 0.0.5, **the digest `image/pinned.toml` pins today** |
| Where the image came from | pulled from `ghcr.io` **by digest** | pulled from `ghcr.io` **by digest** |
| Installed with | `bootc install to-disk --via-loopback --wipe --filesystem btrfs` | the same |
| Booted under | QEMU, `-machine q35 -accel tcg,thread=multi -cpu max -smp 4 -m 4096`, OVMF, `-netdev user` | the same |
| Base | Fedora Linux 42 (Adams), kernel 6.19.14-101.fc42.x86_64, `bootc` 1.15.1 | the same |
| Root filesystem | `btrfs`, which is `alo_image::THE_ONLY_FILESYSTEM` | the same |
| SELinux | enforcing | enforcing |
| The person | `alo`, uid 1000, from the image's own `sysusers.d`, unedited | the same |

**How they differ from task 11's machine, which is the whole point of the first
one.** That machine was installed from an image in the development PC's **local
container store**; both of these were pulled from `ghcr.io` by digest and
installed from that. The first holds the image, the installer command and the
base constant against task 11's and changes only where the image came from. The
second is the newest build, so that *a machine running the newest build says so*
is measured rather than argued.

**Three honest caveats.**

- There is no hardware virtualisation available to this checkout — `/dev/kvm` is
  present in WSL2 and the module cannot be opened (`failed to initialize kvm: No
  such device`) — so both machines were emulated. That changes how long a boot
  takes and nothing about what the base writes on the disk.
- The check program was copied onto each disk by hand, as task 11's was, with
  its SELinux label set to `bin_t` so that it runs under an enforcing policy.
  **The image installation of `alo-looking-once` remains the installer lane's**,
  handed over by task 10 and still owed.
- The measurement ran from a unit of its own added to each machine's `/etc`. It
  runs the shipped program unchanged, as uid 1000, through `runuser`.

## 1. The digest measurement, made before the code

Task 11 found `status.booted.image.imageDigest` (`sha256:2e7ecd95…`)
disagreeing with the image reference, the spec and the origin file
(`sha256:48bd5f31…`) on a machine installed from a local container store, and
named the open question: *whether a machine installed straight from the registry
records the registry's digest instead*.

**It does.** On `alo-task-12-older` — the same release, the same installer, the
image pulled from the registry rather than taken out of the local store — all
four agree:

| what was read, as root | what it said |
|---|---|
| `status.booted.image.imageDigest` | `sha256:48bd5f319abcecfa832eb9a5b0b2f7cd06815b1c30c43b781499500ec14c3858` |
| `spec.image.image` | `ghcr.io/aloworld-org/alo-os@sha256:48bd5f31…` |
| the origin file | `container-image-reference=ostree-unverified-registry:ghcr.io/aloworld-org/alo-os@sha256:48bd5f31…` |
| `image/pinned.toml` for 0.0.4 | `sha256:48bd5f31…` |

And the local store's own view of the same image, on the host that installed it:
`podman images --digests` reports `sha256:48bd5f31…` for a copy **pulled** from
the registry, where task 11's copy — built locally and pushed — carried
`sha256:2e7ecd95…`.

**So the disagreement was the install's and not the product's.** It is the
same trap `image/pinned.toml` already warns about in its own comments — *the
digest is the one the registry reports, not the one podman gives the local copy*
— arriving by a second road: a machine installed out of a local container store
records the local manifest digest, and an offer is compared against what the
registry publishes.

That is worth keeping rather than closing, because **it is a property of how a
machine was installed, which nothing in this repository controls on somebody
else's bench**. Reading the origin file closes it for good: the origin carries
the reference the machine was installed *from*, which is the registry's digest
on every machine installed the way a customer's is, and it is what the code now
reads. Nothing was changed about `alo_keeping_up::Running` or `Standing::between`
on the strength of this — the field the base fills is the base's business, and
the fix is to stop reading it from the one road where it is wrong.

## 2. The road the base already leaves open

### The refusal, reproduced independently

`bootc status` still refuses the person, measured on `alo-task-12-older` as
uid 1000 through `runuser`:

```text
exit=1 said=error: Status: Preparing for write: Querying root privilege: This command must be executed as the root user
```

Nothing on stdout. This is task 11's finding on a second machine, a different
install and a different filesystem, so it is the program's behaviour rather than
anything about that one disk.

### What is readable to the person, and what is not enough

| read as uid 1000 | what came back |
|---|---|
| `/proc/cmdline` | `… root=UUID=961c9317… rw ostree=/ostree/boot.1/default/81fdfdcf8d439e97a9659bff42ce7aac132406e0f2b259ab023888999e469476/0 console=ttyS0,115200n8` |
| the origin file, `0644 root root` | `[origin] container-image-reference=ostree-unverified-registry:ghcr.io/aloworld-org/alo-os@sha256:48bd5f31…` |
| `ostree admin status` | `* default d781e71bac6e…02.0     origin: <unknown origin type>` |

**`ostree admin status` was measured and then not used, and that is a change
from what task 11 decided.** It runs as the person and it names the booted
deployment, exactly as task 11 said — but on a container-image machine its own
rendering of the origin is **`<unknown origin type>`**, so it does not carry the
digest at all. Using it would have meant running a program to learn the
deployment's name and then reading the origin file anyway.

The kernel's command line names the same deployment for free: `ostree=` points
at `/ostree/boot.1/default/<boot checksum>/0`, which is a symlink the base
maintains onto `../../../deploy/default/deploy/<commit>.0`. Resolving it gives
the deployment folder, and the origin file is that folder's name with `.origin`
on the end.

**So the road runs no program at all.** One file read, one path resolved, one
more file read. That is strictly less than the decision task 11 wrote down, and
it is the reason ADR 0011's amendment is as narrow as it is: alo OS does not
speak to a *second* command of the base's, it reads two files the base already
wrote. The checksum in the `ostree=` word is the **boot** checksum and not the
deployment's, which is why `crates/alo-updating/src/booted.rs` resolves the link
rather than assembling a path from its pieces — a test holds that.

### The check, run as the person, on both machines

| | `alo-task-12-older` (0.0.4) | `alo-task-12-pinned` (0.0.5) |
|---|---|---|
| exit | `0` | `0` |
| the answer it kept, `about` | `sha256:48bd5f31…` — the build the origin file names | `sha256:6c9abbc5…` — the build the origin file names |
| `offered` | `sha256:6c9abbc5…`, which is 0.0.5 | `null` — the place offers nothing newer |
| what the unit printed | `asked ghcr.io and kept the answer at /var/lib/alo/an-update-was-found: a newer version of this machine's system is available` — **and there is** | `asked ghcr.io and kept the answer at /var/lib/alo/an-update-was-found: this machine is up to date` |
| the record | one `left-on-its-own` entry, errand `checking-for-an-update`, destination `ghcr.io` | the same, one entry |

Both sentences are true of the machine that said them, which is the whole
change. The left-hand machine really is running an older build than the place
offers, and its kept answer says so:
`{"about":"sha256:48bd5f31…","offered":"sha256:6c9abbc5…"}`. The right-hand one
is running the newest build and is offered nothing.

**Task 11's machine said the left-hand sentence while it was the right-hand
machine.** On 2026-09-20 the newest release was 0.0.4 and that machine was
running it — and it was told a newer version was available, because the digest
it compared came from `imageDigest`, which on that install was the local
store's.

## 3. What changed in the code

| file | what it is |
|---|---|
| `crates/alo-updating/src/booted.rs` | which deployment this machine booted, out of the words the kernel was started with |
| `crates/alo-updating/src/origin.rs` | the build a deployment was made from, read out of the `.origin` file beside it |
| `crates/alo-updating/src/written_down.rs` | the two put together: `WrittenDown::running()`, and `WrittenDown::under()` so a test stands a whole machine up in a folder |
| `crates/alo-updating/src/refusing.rs` | three refusals for the second road, beside the three the first road already had |
| `crates/alo-updating/src/status.rs`, `src/lib.rs` | which road is which, and why there are two |
| `crates/alo-looking-once/src/once.rs`, `src/main.rs`, `src/refusing.rs` | the check reads `WrittenDown` instead of asking `TheBase` |
| `crates/alo-looking-once/tests/a_machine_that_has_never_looked.rs` | the stand-in is a deployment and an origin file on a disk rather than an `impl Base` |
| `docs/decisions/0011-…` | the amendment narrowing *spoken to through its own command* |
| `docs/quirks.md` | the two places reality disagreed with the manual: the base refusing the person, and `imageDigest` on a local-store install |

**One file, one responsibility**, which is why this is three files rather than
one: *which folder did this machine boot out of* and *what was that folder made
from* are separate questions with separate refusals, and a reader who wants to
know how the digest is parsed should not have to read a kernel command line
parser to find it.

**What did not change, deliberately.** `crate::status::running` and `TheBase`
are exactly as they were: every act that **changes** the machine — `apply`,
`go_back`, staging, the broker's unit — still hands the base the arguments
`alo_keeping_up::Staging` decided, with no shell between them, and is still run
by a caller that is root because being root is correct for those. Nothing became
root, nothing gained a capability, no grant widened, and `alo-keeping-up` still
has no clock, no socket and no file.

### The ADR

`docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md` carries
an amendment of this date: *the base is spoken to through its own command* holds
without exception for every act that changes the machine and is narrowed for the
one question the base refuses to answer to the person at all, with the refusal
quoted verbatim and with what the narrowing does **not** permit written down —
reading around the base's command to change anything, reading anything that is
not world-readable, and running any program of the base's other than `bootc`
with the arguments this repository decided.

**No new ADR was owed.** ADR 0001 §2 asks for one where a fix needs a new
privileged component or a widened grant, and this needed neither.

## 4. The two `ext4` installs

`tests/an_update_keeps_the_persons_things.rs` and
`tests/back_to_yesterdays_machine.rs` each stood their virtual machine up with
`--filesystem ext4`, spelt in place. Both now pass
[`alo_image::THE_ONLY_FILESYSTEM`](../../crates/alo-image/src/filesystem.rs),
whose own header asks exactly that of every writer.

`tests/the_filesystem_an_update_is_measured_on.rs` is the half that survives the
next decision: it reads both files back through `alo_image::TheFilesystem` — the
guard that already exists — and refuses a filesystem that is not the one, a file
that names none, and a file that spells the value where it could name the
constant. Changing two strings fixes today; this is what carries them the next
time the value moves.

## Decisions a senior engineer made here, and why

- **`/proc/cmdline` rather than `ostree admin status`.** Task 11 named both.
  Measured, `ostree admin status` does not render a container origin at all
  (`<unknown origin type>`), so it would have cost a subprocess and still needed
  the origin file. Reading the kernel's own words costs nothing and narrows
  ADR 0011 by less.
- **`WrittenDown::under(root)` rather than a trait with a stand-in.** The tests
  now lay a deployment and an origin file out on a disk and read them with the
  code a real machine runs, instead of an `impl Base` that agrees with a program
  nobody runs any more. It is one abstraction fewer and a more faithful test.
- **Three refusals rather than one.** *This machine booted no deployment*, *the
  origin is not there* and *the origin names no build* are three different
  machines and three different things to do about them; the sentence a person
  reads is the same one in all three (`RUNNING_NOT_KNOWN`), so nothing new was
  added to the vocabulary.
- **The transport in front of the image reference is not checked.**
  `ostree-unverified-registry:`, `ostree-remote-image:` and
  `ostree-image-signed:` are the base's business and the signature policy's, and
  which one a machine carries says nothing about *which build is running*. What
  is refused is a reference with no digest on it, because an offer is a
  difference between two digests.

## Limitations and what is still owed

- **The image installation of `alo-looking-once`** — the unit, the program in
  `/usr/libexec` and the `systemctl enable` — is still the installer lane's,
  exactly as task 10 handed it over. Both measurements here copied the program
  on by hand and say so.
- **No hardware virtualisation on this checkout**, so both machines were
  emulated. Nothing measured here depends on it.
- **The digest a machine installed from a local container store records is still
  the local one.** That is the base's behaviour and this task does not change
  it; what changed is that alo OS no longer reads that field on the one road
  where being wrong about it reaches a person.

## Proposed shared-document updates

Not made here — `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` and
`docs/autonomy/STATE.md` belong to the integration owner.

- **CHANGELOG.md**: *The check at a start reads which build this machine is
  running out of what the base already wrote down, rather than asking a command
  that refuses to answer anybody but root — so a machine finds out there is an
  update at every start instead of failing at every start. And the two virtual
  machines that measure an update are installed onto the filesystem alo OS
  actually ships.*
- **QUEUE.md**: task 12 of `v0-5-the-machine-keeps-itself-plan.md` done; task 13
  (*the snapshot nobody removed*) is the next one ready and depends on nothing in
  this plan.
- **STATE.md**: reference this report.

## Verification

Run from `/root/alo-trees/this-machine` — the Linux copy of this checkout that
this machine's gates read — with `CARGO_TARGET_DIR=/root/alo-builds/this-machine`.
Executed, not planned.

| gate | result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy -p alo-updating -p alo-looking-once -p alo-brokerd -p alo-image -p alo-citing --all-targets -- -D warnings` | clean, zero warnings |
| `cargo doc -p alo-updating -p alo-looking-once --no-deps` | clean |
| `cargo test -p alo-updating` | ok — 85 across 13 targets, 5 `#[ignore]`d (the virtual-machine ones) |
| `cargo test -p alo-looking-once` | ok — 17 |
| `cargo test -p alo-brokerd` | ok — 101 (it depends on `alo-updating`, whose `NotRead` gained members) |
| `cargo test -p alo-image` | ok — 315 |
| `cargo test -p alo-citing` | ok — 31 (the citation check, because a decision changed) |
| the supervisor's three, in `tools/kernel-loop` | `fmt` clean, `clippy -D warnings` clean, `test` ok — 152 |

**The whole workspace was deliberately not run here.** It is the supervisor's,
and this change reaches `alo-updating`, its one dependent `alo-brokerd`,
`alo-looking-once`, a decision (`alo-citing`) and a plan (the supervisor's own),
which is what ran.

### One test per acceptance criterion, each run on its own with `--exact`

| the plan's criterion | the test | result |
|---|---|---|
| the check keeps an answer naming the build the machine is actually running — the same one the origin file names | `. alo-updating lib written_down::tests::the_build_running_is_the_one_the_base_wrote_down` | 1 passed |
| …and a machine running the newest build says so rather than offering itself an update | `. alo-looking-once a_machine_that_has_never_looked a_machine_running_the_newest_build_is_not_offered_an_update` | 1 passed |
| the digest measurement is made and written down before the code — and the code reads the origin rather than the field that was wrong | `. alo-updating the_base_is_spoken_to_through_its_own_command the_road_that_reads_what_was_written_down_runs_no_program` | 1 passed |
| what the base is asked is still exactly the arguments this repository decided, with no shell between them | `. alo-updating the_base_is_spoken_to_through_its_own_command each_argument_the_base_is_given_arrives_as_one_argument` | 1 passed |
| …for every act that changes the machine — and nothing else in the crate starts a program | `. alo-updating the_base_is_spoken_to_through_its_own_command the_bases_own_command_is_the_only_program_this_crate_starts` | 1 passed |
| ADR 0011 carries the narrowing in its own text, with the refusal that forced it quoted | `. alo-updating the_base_is_spoken_to_through_its_own_command the_decision_carries_the_narrowing_and_the_refusal_that_forced_it` | 1 passed |
| both `ext4` installs take the value from `alo-image` | `. alo-updating the_filesystem_an_update_is_measured_on every_machine_this_crate_stands_up_is_installed_onto_the_one_filesystem` | 1 passed |
| **the refusal paths**: a machine that booted no deployment | `. alo-updating lib written_down::tests::a_machine_that_booted_no_deployment_is_refused` | 1 passed |
| a deployment named and not on the disk | `. alo-updating lib booted::tests::a_deployment_that_is_not_on_the_disk_is_refused` | 1 passed |
| a check whose machine will not say what it is running asks nothing and nothing leaves | `. alo-looking-once a_machine_that_has_never_looked a_machine_whose_base_will_not_say_what_it_runs_asks_nothing` | 1 passed |

### On real hardware

Both bootc machines above, with the console of each kept. The two lines the
shipped program printed, as uid 1000:

```text
alo-looking-once: asked ghcr.io and kept the answer at /var/lib/alo/an-update-was-found: a newer version of this machine's system is available
alo-looking-once: asked ghcr.io and kept the answer at /var/lib/alo/an-update-was-found: this machine is up to date
```

The first is `alo-task-12-older`, running 0.0.4 while the place offers 0.0.5.
The second is `alo-task-12-pinned`, running the build `image/pinned.toml` pins.
Both exited `0`, both kept an answer, and both wrote one `checking-for-an-update`
departure into `/var/lib/alo/checking-for-updates.jsonl`.

**Not measured here, and not claimed:** anything about a certified machine. These
are virtual machines on a developer's PC, and `docs/hardware.md` is unaffected.
