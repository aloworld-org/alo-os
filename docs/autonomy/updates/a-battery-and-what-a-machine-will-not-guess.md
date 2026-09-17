# A battery, and what a machine will not guess

**Date:** 2026-09-17
**Workstream:** v0.5 — devices and media
**Task:** [task 5](../v0-5-devices-and-media-plan.md) — the battery, and power
profiles
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
measured in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6 CPUs, 3 GB of
memory**, kernel 7.0.0-31, power-profiles-daemon 0.21.
**Egress:** `apt-get install power-profiles-daemon` from Ubuntu's own archive into
the VM, 2026-09-17. The battery itself needed nothing fetched: the kernel's own
`test_power` module was already there.
**Status:** done.

## What this is

`crates/alo-power`. The battery, read **straight from the kernel** — not through a
daemon, and that is a decision rather than a shortcut: a reading that goes through
a service is a reading that stops when the service does, and *why is my battery
gone* is a question a machine should be able to answer while things are going
wrong.

| File | What it is for |
|---|---|
| `reading.rs` | charge, what it is doing, one reading |
| `battery.rs` | the kernel's own files, and whether this machine stops charging early |
| `steady.rs` | how long is left — and the four rules for when this machine will not say |
| `telling.rs` | once at low, once at nearly gone, and never again |
| `profiles.rs` | the rented daemon's closed set, and which this machine really has |
| `the_daemon.rs` | that daemon, reached (Linux) |
| `draining.rs` | *why is my battery gone*, with the model named when it is the model |
| `keeping.rs` | the profile a person chose (ADR 0038) |
| `words.rs` | the ten sentences |

Thirty-one tests, four of them on a machine.

## The part worth arguing about: the machine refuses to guess

Everyone has watched *2 hours 14 minutes* become *31 minutes* by opening a
browser, and what that teaches a person is to stop reading the number — which
costs them the one moment it would have been useful. So `steady.rs` holds a rule,
in one place, and **each part of it is a test that names it**:

1. the machine must be running **on** the battery — a time to empty while it is
   charging is a number about a situation that is not happening;
2. at least **three** readings;
3. spanning at least **a minute**, so a burst of activity is not the whole sample;
4. the draw across them varying by no more than **a quarter**.

Rule 4 does the work: a machine whose draw doubled while this was measured has no
steady draw to extrapolate from, and the honest answer is that it does not know.
The sentence for that is in the vocabulary — *this machine will not guess how long
is left while what it is doing keeps changing* — because an admission a person can
read beats a number they learn to ignore.

## Told once, and plugging in forgets

Once at a fifth, once near the end, nothing at all on the mains. A fifth rather
than a tenth because the number is not about the battery, it is about how long
somebody has to find a cable, and a tenth of a worn-out battery is a few minutes.
A battery that falls past both marks between two readings gets the sentence that
matters rather than neither. **Plugging in forgets both**, so the next run down is
told about from the beginning — the one case where saying it twice is right.

Taken on a machine, through the kernel's own **test** battery (`modprobe
test_power`), driven down 80 → 50 → 21 → 20 → 19 → 6 → 5 → 4:

> on the way down a person heard: `[(20, "power.getting-low"), (5, "power.nearly-gone")]`

The test refuses to run against anything but that fake battery, and puts every
dial back: a test that wrote to a real battery's controls would be damaging
somebody's laptop to prove a point about a message.

## The profiles are the daemon's, and this machine has two

`power-saver`, `balanced`, `performance` — read from the rented daemon rather than
declared here, and a name it offers that alo OS has no sentence for is passed over
rather than shown untranslated. On this machine:

> this machine offers `[Saver, Balanced]` and is in `Balanced`

There is no `performance` here, so it is **absent** — not greyed out. A switch a
person cannot move is a thing to wonder about every time they open the page, and
*why is this grey* is a question the machine has no answer to. The on-a-machine
test chooses the other profile, reads it back, puts the machine back where it
found it, and then checks that asking for the profile this machine does not have
is refused **before** anything is told to the daemon.

A charge limit is the same shape: read from
`charge_control_end_threshold`, offered where the hardware has one, absent where
it does not. This machine has none, and says so.

## Why is my battery gone

`draining.rs` answers it from a reading of what is running (`alo-measuring`, read
and never edited), and names **one** thing or nothing:

- the person's own **model**, as itself — alo OS started it, so it knows which
  process it is, and it says *your model is what is using this machine*. On a
  machine whose first-class workload is a model the customer owns, a machine that
  would not say so is leaving its owner to guess about the one process it knows
  the most about;
- or something else, named;
- or **nothing in particular**, which is an answer and often the true one. A
  machine can be flat because it is old, and saying so beats blaming whatever
  happened to sort first.

Nothing here watches anything: it is handed a reading taken when somebody asked,
and keeps no history.

## One thing that went wrong, which is worth passing on

The first version of the on-a-machine profile test was **two** tests: one that
chose a profile and put it back, and one that checked a profile this machine does
not have is refused. Cargo runs the tests in a file at the same time, so the
second read the machine's profile while the first had it set to *save power*, and
the gate failed with `a refused profile change moved this machine anyway`. It was
a real race and the test was right to catch it.

They are one test now, because both are about **this machine's** profile and a
machine is one machine. It is a fair warning for every other test that changes
something a machine only has one of: the audio default, the radio, a card's
profile. Two tests moving it at once are two tests measuring each other.

## Two notes for other lanes

- **`alo-measuring`'s `Running` cannot be built from outside the crate** —
  `running` is a private module and `Running::of` is `pub(crate)` — so a crate
  that wants to decide something about what is running can only be tested against
  a whole real machine. `what_is_draining_it` therefore takes
  `running.processes()` rather than the reading, which is fine and is arguably
  better; but a `#[doc(hidden)]` constructor for tests, of the kind `alo-sound`
  and `alo-bluetooth` now have, would let the next crate test against a machine
  it can describe. Not changed here: another plan's crate.
- **Nothing in this crate suspends anything.** Suspend is `alo-sleeping`'s and is
  not a power profile, per the plan's own constraint. Where a lid-close or an idle
  timeout wants to know the battery is nearly gone, this crate's reading is the
  thing to read.

## Not ticked

Nothing here is *on the machine* in `ROADMAP.md`'s sense. The battery was a
kernel's test battery in a virtual machine and the profiles' platform driver was a
placeholder: what has been shown is that the roads work, not that they work on
certified hardware with a real battery in it.
