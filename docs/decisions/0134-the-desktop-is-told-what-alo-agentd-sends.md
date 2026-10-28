# ADR 0134 — The desktop is told what alo-agentd sends

**Status:** **accepted, 2026-10-27, by the owner**, who chose option 1 below
and asked for the best road rather than the quickest.

**Amends:** [ADR 0125](0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md),
whose table called `alo-agentd`'s indicator *the single one law 1's surface is
drawn from*. It was not drawn anywhere.

**Context:** law 1; `docs/contracts/daemon-protocol.md`;
`docs/contracts/native-egress-indicator.md`; `crates/alo-egress`,
`crates/alo-indicator`, `crates/alo-agentd`, `crates/alo-desktop`.

## What was found

The sixth Demo 1 walk, on a booted machine on 2026-10-27, asked alo through a
provider (OpenAI). The Dock's status area stayed dark.

- **The desktop's indicator** is `alo-desktop`'s own `alo_egress::Indicator`.
  Its only errand is an installation started from the market window.
- **The indicator a provider call lights** belongs to `alo-agentd`, which opens
  that connection: `Indicator::beginning` is the only way to the `Departing` a
  question needs, and agentd calls it.
- **Nothing carried the second to the first.** There was no message on either
  door, and `alo_indicator::Lamp` deliberately could not be built from a number
  somebody reported.

So on every machine with a provider, a question left with nothing on the screen.
The record wrote it down, but nobody could open the record, which was a
separate gap closed the same day.

## The options put to the owner

1. **The desktop asks agentd, and agentd tells it on every change.** The
   service that opens the connection is the one that knows.
2. **Turns go through the desktop**, which would then show its own indicator.
   This moves network access into the process that sees every keystroke and
   pixel.
3. **Show it only in the task window.** You see it only when that window is in
   front, which keeps half of *visible at the moment it happens*.

The owner chose 1.

## The decision

- **agentd's indicator tells a list of watchers, inside the call that changes
  it** (`alo_egress::Telling`). A provider call blocks agentd's one thread for
  the length of the question, so a message deferred to the next round would
  arrive after the request had gone. Told inside `beginning`, the desktop's
  socket holds the line before the `Departing` exists.
- **A desktop asks `watch-what-leaves` on the person's door.** It is answered
  with the list as it stands, then given over to the watchers, so Settings can
  still knock. Every change writes **the whole list**, never a change to it.
- **A desktop that cannot take a list at once is dropped, never waited for.** A
  frozen screen must not hold an agent's question up. With nobody watching, a
  departure still happens and the record still writes it down (the owner's
  default, said the same day).
- **What crosses is `alo_egress::Reported`.** It deserialises and can be said,
  and has no road to a `Leaving`, a `Departing` or a line on an indicator.
  Nothing holding one may open a connection because of it.
- **Only `alo_indicator::watching` makes the type the lines arrive in**
  (`ToldByTheDaemon`). It reads from the person's own door after checking that
  the directory is theirs alone and that the process listening runs as them
  (`SO_PEERCRED`). This is the *different type with the report in its name*
  that `Lamp`'s documentation promised.
- **Not told is never dark.** Before the service has said anything, and from
  the moment it stops, the status area reads *alo OS cannot tell whether your
  agent is sending anything — its service is not answering*. It never refuses
  the frame.

## What this costs, said plainly

- **The line names the provider and its region, not the model.** It reads
  *@alo is asking a question of openai, in the US*. Naming the model is a
  separate change; it was promised on 2026-10-27 in a reply and was wrong.
- **A program running as the same person could bind the socket instead.** It
  would have to remove the service's socket first, and anything running as
  the person can already do whatever the person can. `alo_egress::Departing`
  documents the same edge for its own reasons.
- **The screen can trail the network by one frame.** The line is in the
  desktop's socket before the connection opens, and is drawn on the next frame.
- **The nested backend and the overlay's `Quiet` are not fed yet.** Only the
  direct desktop, which is what a machine runs.

## The test that decides whether this works

On a booted machine with a provider: ask alo something, and the status area
shows the line while the question is out. Stop `alo-agentd`, and it reads
*cannot tell*. Neither has been seen on a machine yet. The tests in
`alo-egress`, `alo-agentd`, `alo-indicator` and `alo-protocol` hold the
mechanism, and run on every build.
