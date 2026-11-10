# ADR 0148 — The kernel counts what an application sends during a person's command

**Status:** **accepted, 2026-11-06, by the owner**, who answered the question
below *"Yes"*, relayed by the coordinator. The wording is the lane's; the
decision is the owner's.
**Date:** 2026-11-07
**Amends:** [ADR 0015](0015-the-kernel-learns-what-a-turn-is.md), *the LSM decides
and forgets*, for this one purpose and no other.
**Context:** [`docs/contracts/terminal-commands.md`](../contracts/terminal-commands.md)
section 5 and its answer 7; [ADR 0018](0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md)
(one privileged loader, taking no input); [ADR 0029](0029-what-the-kernel-writes-down-about-a-turn.md)
(still proposed, and untouched by this); [ADR 0125](0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md)
(a command asks a service that can show an egress); [ADR 0143](0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
fact 5 and question 6; `docs/contracts/application-door.md`.

## What it changes for the person

When a person types a command that makes an application do something — press
*Send* in a mail program, open a page in a browser — the indicator now shows
that the application is sending while the command runs: *Thunderbird is
sending, during your command.* Before this, the indicator could not see what an
application sends at all, so a command that sent an email left the machine with
nothing on the screen. Nothing the application does outside those seconds is
counted, and nothing about where it sent or what it sent is ever read.

## The question, as it was put

The contract decided (answer 7) that what an application sends during a typed
command is made visible **by counting**, with *the kernel counter that already
watches a turn's connections*. Reading the code on 2026-11-06 found no such
counter: the boundary's program decides and keeps nothing, by ADR 0015, and two
tests hold it to two maps. So the lane stopped and put this to the owner, in
these words:

> *"To show on the indicator what an application sends while a person's command
> runs, may alo OS add a second kernel program that **counts bytes and nothing
> else** (no addresses, no contents), for one application's sandbox, only for the
> seconds the command runs, and clears the count when the command ends? This
> changes ADR 0015's 'decides and forgets' rule for that one purpose, through a
> new decision record."*

The owner, 2026-11-06: **"Yes"**.

## The decision

### 1. One program and one map, and nothing else

The programme `alo-boundaryd` loads gains exactly two things, named here so that
a third cannot arrive under this record:

- **The map `COUNTED`**: a per-CPU hash from a control group's id to a number of
  bytes, holding at most 64 entries. **The program never adds an entry.** It
  looks up the sending socket's own control group, and only where an entry is
  already there does it add the packet's length to it. Entries are put there and
  taken away by the person's desktop, and only for the length of a command.
- **The program `counting`**, on the kernel's `cgroup_skb/egress` hook, attached
  once at boot to the root of the control-group tree, allowing others beside it.
  **It always lets the packet through**: it counts and never decides. A packet to
  the machine's own loopback interface is not counted, because nothing left.

What it reads is the socket's control group, the interface the packet leaves
on, and the packet's length. **It reads no address, no port, no contents, and
no process.** There is no ring buffer, no `bpf_printk`, and no counter keyed by
anything but a control group somebody asked to count.

### 2. Who asks, and for how long

A person's command asks **the compositor's door** (`application-door.md`), which
already admits only the person's own login, with one new act, `count
<application>`. The connection stays open while the command runs; the command
says `done` when it has finished, and is answered with the total. **A connection
that closes ends the count**, so a command that crashes cannot leave one
running. **The desktop ends any count after two minutes** whatever the command
does, because a count held open for ever would be the watching this record
exists to prevent.

The desktop is the service that draws the indicator, which is why it is the
one that asks: ADR 0125's rule is that a command never reports an egress itself.

### 3. Which control groups are an application's

The ones the person's own service manager runs it in, under `app.slice`:

- `app-alo-<identifier>@<n>.service` — what alo OS starts when a person presses
  an application (`alo_launching::the_unit_for`), with `-` written `\x2d`;
- `app-flatpak-<identifier>-<n>.scope` — what Flatpak moves an application into
  when it starts;

and every control group beneath those. They are read again on each look while a
command runs, so an application that starts during the command is counted from
the moment its unit appears.

### 4. What the person sees, and what is kept

- **The indicator**, from the first byte counted until the command ends:
  *{application} is sending, during your command* — **during**, not *because
  of*, because the count includes anything else the application sent in those
  seconds. A line that would otherwise vanish within two seconds stays for two,
  so it can be seen.
- **The record**: the total goes into the person's own record entry for the
  command, written by the command (`commands.jsonl`, ADR 0125's one writer).

### 5. What it cannot see, said so it is not mistaken for proof

- **What another process sends for the application.** A name looked up by the
  machine's resolver, a file uploaded through a portal, a sync done by a helper
  started outside the application's units: each is a different control group.
- **What is sent before a newly started unit is found** — at most one look,
  a tenth of a second.
- **That it was the command that sent it.** It counts the application, during
  the command, which is what the line says.

## How the guard tests change, named rather than loosened

- `the_boundary_decides_and_forgets.rs` read the programme's maps as exactly
  `BOUNDS` and `FIELDS`. It now reads exactly `BOUNDS`, `COUNTED` and `FIELDS`,
  each by name. A fourth, or a third under any other name, is still the
  finding. And it gains what this record promises: **after an ordinary day,
  `COUNTED` holds no entry**, and an entry somebody put there grows while
  nothing else does.
- `the_records_source_is_decided_before_it_is_built.rs` held `kernel.rs` to two
  maps while ADR 0029 is proposed. It now holds it to two maps **plus the one
  named here**, `COUNTED`, which is not ADR 0029's map: ADR 0029 asks what the
  kernel writes down *about a turn*, and this map holds nothing about any turn.
  Any other third map beside a proposed ADR 0029 is still refused.

## Amended 2026-11-10: the same count, of alo's own copy of an application

ADR 0143's owner answers of 2026-10-31 (Q6) and 2026-11-10 put alo, by default,
in **a copy of an application of its own**, counted apart from the person's
windows. This record's map and program are unchanged, and nothing is counted
that a person's command could not already have counted: the desktop holds
`COUNTED` entries for **alo's copy's control groups only** — the scope it hands
the copy's process to, `app-alocopy-<id>@<n>.scope`, and the Flatpak scope that
process is moved into, read off the process itself — for as long as alo holds
the door's `work-in-alos-copy` connection, and at most fifteen minutes on one
connection. **The person's own windows are not counted while alo works**, and
alo's copy is left out of a person's own count. At the setting turned up,
`work-in-their-window` holds no entry at all. The wire is in
`docs/contracts/application-door.md`; nothing asks for it until a turn drives
an application.

## Alternatives the lane put beside it

- **Watching the application's connections from outside**, with no kernel
  change. It sees ordinary web connections only: a browser's newer protocol and
  everything else would be missed, and an indicator that looks trustworthy and
  misses things is worse than none.
- **The machine's network card totals.** They would blame the command for every
  byte anything on the machine sent.
- **Counting the application all the time.** Not offered: that is watching the
  person (ADR 0143).
