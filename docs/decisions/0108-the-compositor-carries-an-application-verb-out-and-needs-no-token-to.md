# ADR 0108 — The compositor carries an application verb out, and needs no token to

**Status:** **accepted, 2026-10-13**, by the owner, who chose the socket.

## What is stuck, and it is not the hard part

`docs/features.md` promises four application verbs — **open, focus, arrange,
close** — and `docs/contracts/agent-verbs.md` publishes them for third parties.
`alo-applications` decides all four: the declarations, the arguments, the
approval sentences, and `Reaching`, whose own header says *whatever drives the
compositor holds one of these or it holds nothing*. `alo-keeping-up` can undo
them.

**And they are offered to nobody**, which is exactly where
`install_application` was until #662.

`docs/autonomy/QUEUE.md` item 19b says they are *blocked on the acting half,
which is Wayland and D-Bus … until something can move a window*. **That blocker
has lifted and the item was not updated.** The compositor starts applications
over the bus — the Dock does it through `alo_portals::activated` — and it
activates, switches and closes windows.

What is actually stuck is one process boundary: **the capability model is in
`alo-agentd` and the windows are in the compositor**, and there is no road
between them for a verb.

## The decision this proposes

**The compositor serves a door that only the person's own uid may use, and
`alo-agentd` is its only caller. No token crosses it.**

That is the whole decision, and the reason it is this and not the broker's
shape is the part worth writing down.

### Why no token, when the broker demands one

[ADR 0001](0001-the-capability-model.md) §2 puts the operations that *genuinely
need privilege* behind `alo-broker`, which runs as root. A root service must not
act because a person's process said so, so `alo-broker`'s door takes **five
words and an HMAC** over the exact verb, argument, approval and moment, good
once and for sixty seconds. The token exists because of the privilege gap.

**There is no privilege gap here.** The compositor runs as the person. Asking it
to close a window is a thing any process of the person's can already do on any
desktop, and granting it no new power is the point: a door that accepts the
person's uid hands out nothing the person did not already have.

**And the agent cannot reach it.** `docs/contracts/daemon-protocol.md`: *the two
sides are two Unix users*, decided by `SO_PEERCRED`, and *a machine on which the
person and the agent are one login has no socket at all — the daemon refuses to
start*. The agent is its own login. So a door that admits only the person admits
the daemon and refuses the agent, by the kernel, with nothing on the wire taking
part in the decision.

So the authority an application verb runs under is the **turn's**, checked where
every other verb's is checked, and the compositor is the hand rather than a
second judge. A token would be this machine proving something to itself across a
boundary it already controls — ceremony that reads as a guarantee.

### What the compositor therefore must not do

- **Not decide anything.** It carries out what it is handed and refuses what it
  cannot do. Which applications exist is `alo_applications::Installed`'s, which
  windows exist is the compositor's own, and whether a verb may run is the
  turn's. Three answers, three owners, as now.
- **Not accept a verb name as text to look up.** The door's requests are
  enumerated, as `alo-broker`'s thirteen are, and an argument is a shape rather
  than a string a caller composes — law 2 at the one place a caller can reach.
- **Not answer anybody but the person's uid**, checked from the kernel before a
  line is read, and closing the connection **with nothing written on it**
  otherwise, which is `daemon-protocol.md`'s rule for a stranger and its reason:
  an answer tells whoever knocked that there is an alo OS here.

## A Unix socket, and not a D-Bus interface

The owner chose it, and the reason is the one thing this door must get right.

Everything about its safety rests on whether the caller is the person or the
agent. `daemon-protocol.md` already settled how that question is answered and
why: **the credentials are the kernel's account of the caller, not the caller's
account of itself.** A socket gets that from the kernel at the moment the
connection is accepted, before a byte is read. D-Bus gets it by asking the bus —
a third party standing between this door and the one fact it turns on.

The rest is smaller and points the same way: the daemon, the broker and the
session service all already do one-line-in-one-line-out over a socket, so this
is a fourth instance of a pattern rather than a second pattern; it works whether
or not a session bus is up; and nothing else on the machine can see it.

**The honest cost, measured rather than waved at: the compositor has never
listened on a socket.** There is no `UnixListener` anywhere in `alo-shell`. So
this is new work inside a window system's event loop, not a new file beside an
existing listener — and D-Bus would have been the cheaper build precisely
because `alo_shell::access_bus` is already there. The identity check was worth
more than the saving.

## What this does not decide

**And nothing about `arrange`.** Open, focus and close are a window named and a
thing done. `arrange` takes a *where*, and where a window goes on a screen is
`alo-arranging`'s and the design's; it is the fourth verb for a reason and it is
not in this road's first pass.

## What it costs if it is wrong

If the owner would rather the compositor proved the turn's authority
cryptographically, the change is a token on a door that already exists rather
than a different door — `alo-broker`'s `ApprovingKey` would have to stop being
typed to `SystemVerb` first, which is a change to privileged proof code and
deserves its own decision.

## What it produced

[`docs/contracts/application-door.md`](../contracts/application-door.md) — the
path, the modes, the request line, the answers and which refusal each act can
honestly meet. It is a published surface from its first version rather than
from the version somebody outside starts depending on, as every other contract
in that directory is.

## Not relitigated

ADR 0001 §2's broker and its token, `daemon-protocol.md`'s two doors and its
`SO_PEERCRED` rule, and law 2's enumerated verbs. This proposes a third door for
an unprivileged consumer and asks for no exception to any of them.

## How an approved `open` starts the application — the owner's decision of 2026-10-31

**Asked:** once a person's own press on the Dock started any installed
application the ordinary way (#1045) — over the bus where the entry says
`DBusActivatable=true`, and from the entry's own `Exec` line otherwise, in a
unit of its own with alo's display — should an approved `open_application`
take the same road, or stay on bus activation only, as it had?

**The owner's answer: the same road.** After the person's approval and their
grant for that application, alo starts it exactly as a press would. What that
does and does not widen:

- **It still starts only installed applications, by their identifier, from the
  entry's own `Exec` line.** There is no field on this door for a command to
  arrive in, and none is added: the request line is unchanged, and what runs is
  what the installed desktop entry names — never text the agent wrote.
- **One approval, one execution, one record.** The record of an approved `open`
  says **which road** starts the application — *over the session bus, as its
  entry says*, or *from the command its own entry names* — read off the entry
  with the same decisions the desktop's sender makes (`alo_launching`).
- **An entry that gives no way to start is refused by name before the door is
  asked**, and that refusal is written down like every other.

The record names the road the entry asks for, decided before the door is
asked. The desktop's one further step — using the entry's `Exec` line when the
bus has nothing to start for a `DBusActivatable` entry — is said on the
desktop's journal, not in this record: a door whose answer is one word cannot
carry it, and changing that answer would be a break in a published surface.

**And the door is now served.** Until the same day nothing on a real machine
served `/run/alo/<uid>/shell.sock` — only an example did — so every approved
application act was answered *this machine could not reach* the application.
The desktop now serves it once the daemon's directory is there
(`alo_shell::the_door_on_the_desktop`).
