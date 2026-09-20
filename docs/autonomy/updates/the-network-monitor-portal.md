# The network monitor portal

**Date:** 2026-09-20
**Workstream:** v0.5 applications and what they expect — task 12 of
`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, second half.
The first half is `docs/autonomy/updates/how-far-this-machine-reaches.md`.
**Contributor:** Claude, as a lane in `/root/alo-os-lane-b`
**Status:** ready for integration. The code and its tests, run in WSL, on a
private session bus. **No machine in this lane has NetworkManager**, so the
backend is handed a reading of the test's own; nothing here claims what
NetworkManager reports on real hardware.

## What changed, for a person

An application can now ask this machine whether it is worth trying to send
anything — and is told only if the person granted it that.

What it is told is deliberately small: **whether anything is reached, and
whether the connection costs money to use.** Not which network this machine is
on, not its name, not its strength. A sandboxed application learns that it is
online; it does not learn where its owner is sitting.

## What changed, in the repository

**`crates/alo-portals/src/network_state.rs`** (new). The decision, without the
bus, so it is tested on any machine: who may be answered, and what the answer
is.

**`crates/alo-portals/src/network_monitor_portal.rs`** (new).
`org.freedesktop.portal.NetworkMonitor`, version 3, served at
`/org/freedesktop/portal/desktop` beside the other three. `GetAvailable`,
`GetMetered`, `GetConnectivity` and `GetStatus`.

`Portal::NetworkMonitor::answered_on_the_bus` is no longer `None`, and
`docs/contracts/portals.md` moves it out of *not answered yet* in the same
change. The answers file gains `network-read` and `network-unread`,
additively, in `docs/contracts/portal-answers-file.md`.

## The finding this task existed to make

**Two services number the same four states differently, and they swap two of
them.**

| The state | NetworkManager | GLib, which the portal answers |
|---|---|---|
| reaching nothing | `NONE` = 1 | `LOCAL` = 1 |
| this network, no further | `LIMITED` = **3** | `LIMITED` = **2** |
| a sign-in page in the way | `PORTAL` = **2** | `PORTAL` = **3** |
| reaching what was asked for | `FULL` = 4 | `FULL` = 4 |

A backend that passed the number through would tell every application on the
machine that a café's sign-in page is a working connection, and that a
working-but-local network is a captive portal. Neither number is written in the
portal: `alo_networks::HowFar` is words, `network_state` maps words to GLib's
numbers, and one test holds the mapping to both tables at once.

GLib's enum has **no *unknown***. A network manager that has not worked out how
far the machine reaches is answered `LOCAL`, the least it could be, and
`GetAvailable` is `false`. Answering *full* on no evidence would be this machine
telling an application it is online because nobody has said otherwise.

## Two refusals worth reading

**`CanReach` is never answered, to anybody.** The specification's fifth method
takes a hostname and a port and says whether they can be reached. Answering it
honestly means **sending a packet to a host an application named**, on a machine
whose first law is that nothing leaves silently. It would also be a road around
every other boundary: an application could read a person's network one name at a
time by asking whether each is reachable. It is
`org.freedesktop.portal.Error.NotAllowed` — and refused to a *granted*
application as well, so the refusal says nothing about the grants either.

**`GetMetered` is true where the machine does not know.** An application reads
that key before spending somebody's data. Being wrong the cheerful way costs a
person money on their own connection; being wrong the careful way costs an
update that waits.

## What a refused application is told

`org.freedesktop.portal.Error.NotAllowed`, in the same words as a machine whose
network manager is not answering. An application not granted the network state
learns nothing from being refused that it would not learn from a machine that
cannot see its own network. The record tells the two apart; the application is
not told, and a test asserts the two sentences are the same string.

## What was not done

- **Nothing was measured against NetworkManager.** WSL has none. The bus tests
  serve the real backend on a real bus and ask it with a real `zbus` client, and
  what the machine reports is the test's own `Reaching`.
- **The `changed` signal is declared and never sent.** Nothing here watches the
  network manager for changes, so an application that reads on its own schedule
  is answered correctly and one that waits for the signal waits. The contract
  says so.
- **Nothing draws a network icon.** The status area is the shell plan's, and
  task 7 there is now unblocked rather than done.

## Tests

`network_state.rs` — seven, on any machine: the two services number these states
differently; every state is one of GLib's four; nothing worked out is answered as
the least it could be; a sign-in page is reachable and is not a working
connection; what might be metered is answered metered; only an application
granted the facility is allowed, and no refusal reads the network; a network that
cannot be read is refused rather than guessed.

`tests/the_network_monitor_portal_answers_the_network.rs` — six, on a real bus: a
granted application is answered what the machine reads, including both sides of
the 2/3 swap; an ungranted application is refused and told nothing more, in the
same words a machine with no network gives, while the record tells the two apart;
`CanReach` is refused to a granted application as well; a machine that cannot read
its network refuses and is answered again when it can; a grant of another facility
does not reach the network; and the contract lists the portal as answered.
