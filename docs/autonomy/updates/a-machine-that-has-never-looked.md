# A machine that has never looked

**Date:** 2026-09-19
**Workstream:** v0.5 — the machine keeps itself
(`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, task 10, *A machine that
has never looked*)
**Contributor:** Claude Code worker in `C:\dev\alo-os-3` on this development PC,
for the owner
**Status:** ready for integration. Task 10 is complete and marked in the plan,
and task 11 is written there because the plan named nothing after it. Two things
are handed over rather than done here and are named below: the **image
installation**, which belongs to the lane that owns `image/`, and the **base's
answer to an unprivileged caller**, which needs a machine with a real `bootc` on
it and is task 11's first half. **That second one is closed, 2026-09-20**: it
was measured on `alo-lane-b-bootc` and the base refuses uid 1000, so the unit
this report describes fails at every boot until task 12 lands — see
`docs/autonomy/updates/the-base-answers-only-root.md`.

## What changed

Four fifths of *updates that never interrupt* was built and **nothing on any alo
OS machine had ever looked**. `alo_looking::look` has been able to ask since
task 6 and `Because` has named two occasions since then — *the person asked* and
*this machine started* — and neither had a caller. From where the person sits, a
machine that is never told an update exists cannot be told apart from one that
has none.

**`crates/alo-looking-once`** is the second occasion, and the only caller of it
on a booted machine.

- `src/once.rs` — `AtAStart::look_once`, the whole act in the order it happens:
  ask the base what this machine is running, open the record, check, decide what
  to say through `alo_looking::SaidOnce`, keep the answer. It is the one place in
  this repository that names `Because::ThisMachineStarted`.
- `src/record.rs` — `IntoTheRecord`, the `Noting` that writes the check's one
  departure into the machine's record. It cannot refuse, for
  `alo_record::Record::keep`'s reason, so it keeps what went wrong and the
  program says so afterwards.
- `src/road.rs` — the way out, decided from `/etc/alo-proxy/proxy.json` through
  `alo_proxy::the_way` for `Road::CheckingForAnUpdate`. A machine with no proxy
  file goes straight out; a machine whose proxy file is there and cannot be read
  is **refused**, because reading an unreadable rule as *no rule* is how a
  company's traffic goes around its own proxy with nobody told.
- `src/refusing.rs` — `DidNotLook`, the eight roads where a start does not end
  with an answer kept, and `nothing_left_this_machine` for the first question
  whoever reads the journal asks.
- `src/main.rs` — the program: one line to the journal, `0` when the answer was
  kept and `1` when it was not.
- `alo-looking-once.service` — **the unit, handed over rather than installed**
  (see *What is handed over*). It is `THE_UNIT` in the crate, so a test reads it
  with `alo_image::Service` — the same reader `crates/alo-image` holds the
  image's own units to.

**`crates/alo-looking` gained `Noting`** (`src/noting.rs`), and `look` takes it.
Until this change **no check anywhere was ever written into a record**: law 1 is
two halves, *visible at the moment it happens* **and** *afterwards in a record*,
and the one errand alo OS makes on its own had only the first. `look` now writes
the check into the record before it comes off the indicator, on every road out of
it **including every refusal** — a record with a gap exactly where a check failed
would answer *this machine did nothing on its own* about a week of failed checks.

A user-readable description of the change: *An alo OS machine now finds out
whether there is a newer version of its system, once, on its way up — and the
check appears on the egress indicator while it happens and in the machine's
record afterwards, like everything else that leaves.*

## The decisions, and what each one refused

### Where it runs and under whose privilege

**A system unit, started once at `multi-user.target`, running as the person's
own login, holding no capability at all.** The task said this was the hard half
and that one road might need something no worker may decide. It did not — and
the reason it did not is the reason the other roads were refused.

**Not a login of its own** (the shape `alo-modeld` and `alo-convertd` take).
Both files a check writes are inside `/var/lib/alo`, which
`image/usr/lib/tmpfiles.d/alo.conf` makes `0700 alo alo` because what an agent
did on somebody's machine is theirs. A login of its own could not write there
unless that folder were widened, and the folder the record lives in is not one a
worker widens to make a check convenient.

**Not root.** This process reads what a public registry sent it. ADR 0018's
argument is that one privileged component is acceptable because of how little it
is trusted with; a root process parsing a registry's answer is that argument
quietly stopping being true.

**So the person's login**, which needs nothing the person does not have
themselves (ADR 0001 §2) and writes two files that are already theirs. Nothing
here needed a new privileged component or a widened grant, so no ADR was owed.

### A system unit rather than a session hook

The plan named this as two different machines being described, and it is. The
answer a check keeps is **one per machine** (`/var/lib/alo/an-update-was-found`)
and a session is not one per machine. A session hook would make two sign-ins two
checks — two departures and two lines on somebody's indicator for one question —
and would leave a machine nobody signs into never looking at all. *This machine
started* is a fact about the machine, so the thing that says it belongs to the
machine.

### Its own record file

`/var/lib/alo/checking-for-updates.jsonl`, beside the person's record rather
than inside it. `alo_keeping::Writing` has exactly one writer per file because
two processes appending to one interleave, and `alo-agentd` is the person's
record's writer. **`alo-brokerd` met this first and answered it the same way**,
and a check at a start cannot be ordered against a sign-in tightly enough to
promise the two never overlap. The cost is honest and is written down: somebody
asking *what did this machine do on its own?* reads two files rather than one,
exactly as they already do for the broker.

**And it is opened before the first question.** A machine that cannot write down
what it is about to do does not do it: there is no road through this crate that
reaches the network without somewhere to account for it afterwards.

### `SaidOnce` is honoured and not persisted

The acceptance says a machine with no way out says so once and that `SaidOnce`
already decides this and must not be undone. The way to undo it would be to
write the silence to a disk, and that crate's own reason forbids it: a machine
that remembered across restarts would stay silent about a network that was down
last week. So the memory lasts one check, which is the whole life of this
process, and what keeps a person from meeting the same line at every start is
that **a refused check keeps nothing** — no surface has anything new to put in
front of them. The line each start leaves is in the journal, for whoever
administers the machine.

## Measured on a booted machine

**What the machine was.** Ubuntu 24.04 under WSL2 — a Hyper-V virtual machine
with systemd as PID 1 — restarted with `wsl --shutdown` between starts. The
login `alo` at uid 1000, `/var/lib/alo` `0700 alo alo`, the release binary at
`/usr/libexec/alo-looking-once`, and `alo-looking-once.service` installed
**byte for byte as this crate hands it over** and `systemctl enable`d. The place
asked was the real `ghcr.io/aloworld-org/alo-os` that `image/pinned.toml` names.

**What it was not, stated plainly.** It is not the shipped image: no bootc, no
SELinux, no signature policy, and the base was a stand-in answering the status of
a machine running the digest `image/pinned.toml` pins
(`sha256:48bd5f31…`). What is measured here is the act around the base — where
it runs, how often, what it keeps, what it writes down and what leaves the
machine — and never the base. A real alo OS boot is task 8's and task 11's.

**Two starts, each one check.**

| | first start | second start |
|---|---|---|
| the unit | `active (exited)`, `ExecMainStatus=0`, `NRestarts=0` | the same |
| what it said | `asked ghcr.io and kept the answer at /var/lib/alo/an-update-was-found: this machine is up to date` | the same |
| the kept answer | `because: this-machine-started`, `about: sha256:48bd5f31…`, `at: 1789877348` | the same, `at: 1789877374` |
| the record | one entry, `left-on-its-own`, `checking-for-an-update`, `ghcr.io` | a second entry appended |
| `systemd-analyze blame` | `420ms` | `530ms` |

*Up to date* is the right answer: `0.0.4` is what the repository pins and what
the place holds.

**Started again on a machine that had already looked: nothing at all.** The
record stayed at two entries and the departure count did not move —
`RemainAfterExit=yes` is what makes *once per start* true of a unit rather than
of a sentence.

**Departures counted at the network boundary: six per check.** `/proc/net/snmp`
`Tcp: ActiveOpens` read after a start where the unit was **masked** gave `1`;
after a start where it ran, `7`. The same six appear when the check is run by
hand on a quiet machine. It is two questions, each answered `401` and asked
again with a token the registry named — so three connections per question, and
the vouching still costs no request at all, as task 7 promised.

**A machine with no way out.** The program run as uid 1000 inside a network
namespace with no route:

```
alo-looking-once: this machine did not find out whether there is an update:
  nothing was found out: there is no way out of this machine to the place its
  updates come from
exit=1
before: 7 lines in the record, answer ea19ca8afbf6
after:  8 lines in the record, answer ea19ca8afbf6
```

One line, said once, exit `1`, the refusal **in the record**, and the kept
answer **untouched** — so a surface goes on showing the last answer that was
true rather than a gap.

### Found and fixed inside this task: the check was on the boot's critical path

The first unit was `Type=oneshot`, which is the obvious shape and is wrong here.
A unit wanted by a target is implicitly ordered **before** it, and a oneshot's
start job is not finished until the process has exited — so the check held
`multi-user.target` open for as long as it took, and `graphical.target` behind
it:

```
graphical.target @4.247s
└─multi-user.target @4.246s
  └─alo-looking-once.service @1.891s +1.458s
```

That is the constraint *a check at a start never delays a person's sign-in*
broken, and on a machine whose network answers slowly it would have been the
whole twenty seconds a request waits rather than 1.5 s. `Type=exec` finishes the
start job at the moment the program is exec'd. Measured twice more afterwards,
the check is off the chain entirely — `snapd.seeded.service` is the critical
path on that machine, and `multi-user.target` is reached at 5.31 s and 5.61 s
with the check running in parallel behind it.

The test now asserts `Type=exec` **with the reason in the assertion**, so
somebody tidying it back to `oneshot` is told why it is not one.

## What is handed over

**The image installation is not done here, and this is deliberate.** The plan
that owns this work reads `image/` and `crates/alo-image` and never edits them.
What the lane that owns the image owes, in one change:

1. `image/usr/lib/systemd/system/alo-looking-once.service` — the file
   `crates/alo-looking-once/alo-looking-once.service` is, unchanged.
2. `image/Containerfile` — `--package alo-looking-once` on the build line, a
   `COPY` to `/usr/libexec/alo-looking-once`, that path in the `chmod 0755`
   list, the unit in the `chmod 0644` list, and `alo-looking-once.service` in
   the `systemctl enable` line.
3. `crates/alo-image` — a check holding the installed unit to what this crate
   hands over, so the two cannot drift. `alo_looking_once::THE_UNIT`,
   `THE_UNITS_NAME` and `THE_PROGRAM` exist for exactly that, and
   `crates/alo-looking-once/tests/a_machine_that_has_never_looked.rs` already
   reads the unit with `alo_image::Service`.

Nothing on a machine checks for updates until that lands. The crate, the unit
and the measurement are finished; the installation is one change in somebody
else's tree.

**The base's answer to an unprivileged caller is task 11's first half**, written
into the plan in this change. Nothing in this repository has ever run
`bootc status` as anybody but root: this machine had no base at all, and every
other caller of `alo_updating::running` is a test with `/bin/echo` behind it. If
the real base refuses uid 1000, the check answers *the base would not say which
build this machine is running*, keeps nothing, and the unit fails at every boot
— and the answer is **not** to make this component root. It is named rather than
guessed, and the code that meets it says so in its own sentence rather than
panicking.

**Measured 2026-09-20, and it refuses.** On `alo-lane-b-bootc` — the pinned
0.0.4 image installed to disk and booted under KVM, `bootc` 1.15.1 — every form
of `bootc status` asked as uid 1000 exits 1 with nothing on stdout and
`This command must be executed as the root user` on stderr, and this report's
own unit, run as the person, says *the base would not say which build this
machine is running … nothing left this machine* and keeps nothing. The answer
is still not to make it root: the base's own origin file is world-readable and
names the build, read as uid 1000 on that machine. Task 11 is the measurement
and task 12 is the fix; `docs/autonomy/updates/the-base-answers-only-root.md`
has both.

## Verification

Run from this checkout, gates in WSL against the serialized Linux copy at
`/root/alo-trees/this-machine` with `CARGO_TARGET_DIR=/root/alo-builds/this-machine`,
as `docs/autonomy/SHARED_MAIN.md` requires.

| Command | Result |
|---|---|
| `cargo fmt --all` then `cargo fmt --all --check` | clean |
| `cargo clippy -p alo-looking -p alo-looking-once -p alo-saying --all-targets -- -D warnings` | clean |
| `cargo test -p alo-looking-once` | 13 passed, 0 failed |
| `cargo test -p alo-looking` | 66 + 5 passed, 4 ignored (the network ones), 0 failed |
| `cargo test -p alo-saying` | run for the crate that collects `alo-looking`'s words |
| `cargo doc -p alo-looking-once -p alo-looking --no-deps` | no warnings |

**Not run here, deliberately:** the whole workspace suite, which the supervisor
runs after this. **Not run at all:** anything on real hardware or on a real
bootc machine — see *What is handed over*.

The four `#[ignore]`d tests in `alo-looking` reach the public internet and stay
ignored; this change does not touch what they measure beyond the one argument
`look` gained.

## Limitations

- **The measurement machine is not an alo OS machine.** Stated above in full. It
  shows the unit, the once-per-start, the two files, the record, the refusal and
  the departures; it shows nothing about bootc, SELinux or the signature policy.
- **`bootc status` as the person is unmeasured.** Task 11.
- **A proxy is unmeasured**, as task 6's report also said. Task 11.
- **The login `alo` at uid 1000 was left on the WSL machine** after the
  measurement; the unit, the program, the stand-in base and `/var/lib/alo` were
  all removed. A login that matches the one a real alo OS machine has is closer
  to the truth than none, and nothing in the suite depends on uid 1000 being
  absent.
- **Six connections per check** is more than the two questions suggest, because
  the registry answers each one `401` first. It is not a fault and it is not
  hidden: it is what a check costs, and law 1 says we publish the measurement
  rather than the promise.

## Proposed shared-document updates

Not made here — `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` and
`docs/autonomy/STATE.md` are the integration owner's.

**CHANGELOG.md:** *An alo OS machine now looks for a newer version of its system
once, on its way up. The check runs as the person and holds nothing, appears on
the egress indicator while it happens and in the machine's record afterwards,
and never delays a sign-in. A machine that cannot reach the place its updates
come from says so once and is left exactly as it was.*

**QUEUE.md:** task 10 of the machine-keeps-itself plan is done; task 11 is new
and ready. The installer lane gains one item: install and enable
`alo-looking-once` in the image.

**ROADMAP.md:** *updates that never interrupt* now has a machine that finds out
there is an update. It is not finished until the image starts it, which is the
installer lane's item above.
