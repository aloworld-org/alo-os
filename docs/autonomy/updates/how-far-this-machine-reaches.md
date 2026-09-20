# How far this machine reaches

**Date:** 2026-09-20
**Workstream:** v0.5 applications and what they expect — task 12 of
`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, first half. The
task exists because task 7 of `docs/autonomy/v0-5-the-shell-plan.md` — the status
area's clock, battery, network and volume — named the network monitor portal as
one of its four blockers.
**Contributor:** Claude, as a lane in `/root/alo-os-lane-b`
**Status:** ready for integration. The code and its tests, run in WSL against a
stand-in network manager on a private bus. **No machine in this lane has
NetworkManager**, so nothing here claims what NetworkManager reports on a real
machine; what is proven is the asking and the reading.

## What changed, for a person

Nothing yet, and deliberately. This is the reading the network monitor portal and
the status area will both be answered from, and neither exists yet. What it makes
possible is that when a status area does show a network, it shows **what the
machine actually reaches** rather than whether a cable is in.

The difference is the one everybody has met: a laptop joined to a café's access
point, showing full bars, reaching nothing until somebody agrees to the terms. The
machine already knew it had joined a network. It did not know whether joining had
got it anywhere.

## What changed, in the repository

**`crates/alo-networks/src/reaching.rs`** (new). Two answers read from the rented
network manager, and nothing worked out from them:

- **`HowFar`** — how far this machine reaches through the connection it is on:
  `Nowhere`, `APageInTheWay` (the café, the hotel, the airport), `OnlyThisNetwork`,
  `AllOfIt`, and `NotSaid`. NetworkManager's `Connectivity`, mapped into words.
- **`Metered`** — whether the way out is somebody's meter: `Metered`, `Unmetered`,
  `ProbablyMetered`, `ProbablyUnmetered`, `NotSaid`. NetworkManager's `Metered`,
  which knows the difference between somebody having said so and its own guess,
  and so does this.
- **`Reaching`** — the two together, because they are read together, so a caller
  cannot be told about two different moments.
- **`WhatIsReached`** — a trait apart from `Networks`, because it is a different
  question with a different caller. What there is to join is Settings' and the
  broker's; how far the machine reaches is the portal's and the status area's. A
  surface that only shows a network icon implements this and never the other.

**`crates/alo-networks/src/network_manager.rs`.** The client reads both
properties off the network manager's own object, in one pair of reads. A property
that cannot be read fails the whole reading rather than standing beside a
made-up other.

## Two decisions worth reading

**Nothing understood is never rounded to the convenient end.** Both of
NetworkManager's properties have a value meaning *it has not worked that out*, and
both keep it. A number the service has never published — a sixth answer added in
some future release — is also *nothing said*, not the reachable end of the range.
A machine that has not been told is not a machine that is online.

**One question, one answer.** `HowFar::reaches_anything` is the single place
*connected* is decided, so the portal, the status area and whatever asks next
cannot hold three different views of it. A sign-in page in the way counts as
reaching something, because an application that wants to show one and a person
who wants to get past it both need the connection used.

`Metered::should_hold_off` is answered the careful way round, and it is the only
thing here that is: a guess that the connection is metered holds off, and so does
nothing said at all. Being wrong there spends a person's own money; being wrong
the other way costs an update that waits until the machine is somewhere cheaper.

## Why this crate, and not the portal's

`alo-networks` is where what the network manager reports already lives, made by
task 3 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`. Reading two more
of its properties belongs beside the others, and a portal that read NetworkManager
itself would be a second road to a rented service that ADR 0011 says is asked in
one place. The portal plan owns the answering; this owns the reading.

**This is a cross-plan edit and is recorded as one.** The applications plan reads
`alo-portals`' neighbours and does not normally edit `alo-networks`; the broker
plan, which made that crate, owns no portal. Nothing here changes a broker verb,
the broker's door, or anything a person approves — it adds a read beside reads
that already exist.

## What was not done

- **Nothing was measured on a machine with NetworkManager.** WSL has none. The
  bus test serves a stand-in under the network manager's own name, objects and
  interfaces, and proves the client asks for exactly what it says it asks for.
  What NetworkManager reports for a real café sign-in page is untested here.
- **Nothing shows this to anybody.** No portal answers it yet and no status area
  draws it. That is the second half of the task.
- **Nothing is said in the vocabulary yet.** `alo-networks` has no `alo-strings`
  dependency and carries English for a service log, not for a person. The
  sentences a person reads belong to whatever surfaces this.

## Tests

`crates/alo-networks/src/reaching.rs` — six, about behaviour a person meets:
every number the network manager publishes is its own answer; a number we do not
understand is nothing said, at both ends and in both properties; nothing worked
out yet is not something reached; a sign-in page in the way is still something
reached; what might be metered holds off; and a network manager that has decided
nothing says so in both answers rather than looking like a machine that is
offline and free.

`crates/alo-networks/tests/the_network_manager_on_a_bus.rs` — one more, over a
real bus: the stand-in reports a sign-in page in the way and a *guess* that the
connection is metered, both deliberately mid-range, and the client reads them
without flattening either. It also has a primary connection at the same moment,
so the test shows the two questions answered apart.
