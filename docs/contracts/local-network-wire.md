# Contract — what one alo machine says to another on the port it advertises

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is what crosses the network between two alo machines: the port one
advertises, the six paths on it, the one framing every message shares, the
header a proof travels in, and what comes back. Another alo machine speaks it,
and so would anything somebody builds to pair with one — so it is a public
surface from the first version rather than from the version somebody outside
starts depending on it.

Read `docs/decisions/0003-the-network-is-not-authority.md` first and
`docs/decisions/0031-the-pairing-is-the-key.md` beside it: what a pairing *is*
and what a proof proves belong to those, and this describes only how the
messages are put and answered. `crates/alo-nearby`, `crates/alo-corridor` and
`crates/alo-agentd` are this document as working code.

## The port, and who answers on it

A machine advertises itself over mDNS/DNS-SD (`crates/alo-nearby`,
`docs/features.md` *machines find each other*) with one port, **7610**, and
`alo-agentd` binds it on every interface. There is no setting for either: no
*advertise as*, no *discovery off*. What a machine says about itself is its
identity and this port, the same whether anything is paired or a turn is under
way.

One process answers on the port, and it tells the messages apart **by path**.
Anything on another path is answered `404` with the body `not-for-this-wire`
and reaches nothing on the machine. Anything that is not a message at all is
answered `400` with the body `not-a-message`.

## The framing

The least of HTTP/1.1 that carries one message there and one back
(`alo_nearby::http`):

- a request is a `POST` with `content-length` and `connection: close`, a
  `content-type` of `text/plain; charset=utf-8`, at most thirty-two headers,
  and a body of text;
- a reply is a status line with the same two headers and a body of text;
- no chunked bodies, no keep-alive, no pipelining, no redirects. A
  `transfer-encoding` header refuses the message.

Bodies are bounded: four kibibytes on the pairing wire, sixteen on the verb
wire. A message that says it is longer is refused before its body is read.
Either end waits ten seconds for the other on one connection.

## The six paths

| Path | Method | Body | Answered by |
|---|---|---|---|
| `/alo-os/1/pairing/proposal` | `POST` | one line, `alo-os/1 proposal <asking> <asked> <seconds> <offer> <may,…>` | `200` and the asked machine's offer, one line; or a refusal word |
| `/alo-os/1/pairing/confirmation` | `POST` | one line, the confirming machine's confirmation | `204`; or a refusal word |
| `/alo-os/1/verb/read` | `POST` | JSON `{"verb":…,"given":[…]}` | `200` and an answer; or a word at the door |
| `/alo-os/1/verb/change` | `POST` | the same | `200` and the number the change waits under; or a word at the door |
| `/alo-os/1/verb/outcome` | `POST` | JSON `{"number":…}` | `200` and what became of it; or a word at the door |
| `/v1/chat/completions` | `POST` | an OpenAI-compatible question, exactly one message from the person | `200` and the answer in the same shape, as JSON; or a word, see below |

The two pairing paths are `crates/alo-nearby`'s (`Proposal::said`,
`Confirmation::said`, `NotProposed::on_the_wire`); the three verb paths are
`crates/alo-corridor`'s (`Carried`, `AskedAbout`, `Answered`, `AtTheDoor`);
the question path is `crates/alo-asking`'s (`THE_QUESTION_PATH`). A field a
body has no place for is refused rather than read around.

## The proof

Every verb, outcome and question carries a proof in the header `alo-pairing`
(`alo_asking::THE_PROOF_HEADER`): `alo_nearby::Proof::said`, one line, an
HMAC over the sending machine, the receiving machine, the moment and the
SHA-256 of exactly the body bytes, keyed by the pairing (ADR 0031). The
receiving machine judges it **before anything else** — before the body is
read as anything, before a grant is asked — against its own pairings at the
moment, and refuses:

| Word | Meaning |
|---|---|
| `no-proof` | the header was absent or did not read |
| `not-paired` | no pairing with the machine named stands now — never made, expired, or revoked |
| `not-for-this-machine` | the proof names another machine as the receiver |
| `not-from-that-machine` | the tag does not verify: a stranger, or a body altered in transit |
| `from-another-moment` | the moment is more than two minutes from this machine's clock |
| `already-used` | a proof this machine has accepted within that window |

A refusal before the door is one of the words `alo_corridor::AtTheDoor` spells
on the wire, with a status that is never `200`; it carries nothing of the
machine and nothing is written down for it. The two pairing paths carry no
proof — a proposal is what makes the key — and a confirmation proves itself
with the key both offers agreed.

## What the proofs are judged against, and where the answer leaves

The pairings, the proposals, the door onto the machine and the proofs seen
are **one each** on a machine: a pairing revoked on the person's own surface
refuses the very next verb and the very next question alike, and a proof
spent on a question is a replay as a verb. A verb from a paired machine is
judged against the **receiving** machine's grants, a change waits for the
**receiving** machine's person, and every answer that carries anything of the
machine leaves under its egress indicator and is written in its record with
the origin machine named (ADR 0003). A verb arriving while the person's own
agent holds a turn waits for that turn to end.

**Named** is the name the receiving machine's person gave the other machine
(`name-machine`, `docs/contracts/daemon-protocol.md`), or its identity until they
give one. A name is kept on the machine whose person gave it
(`docs/contracts/machine-names-file.md`) and **nothing on this wire carries one**
— no header, no body, no proof — so neither machine learns what the other calls
it, and nothing is found, dialled or proven by a name: the identity is what the
proof, the pairing and every grant are about.

## The question path

A question is told apart, proven, judged against the pairing's own list, and
then answered by **the machine's own model and nothing else**. In that order:

1. **The proof**, as above, through the same memory the verb wire refuses
   replays with. A proof spent on a question is a replay as a verb.
2. **The pairing's list.** A pairing that does not permit asking this
   machine's models (`alo_nearby::MayAskIts::Models`) is answered `403` with
   `not-permitted`, and the body is not read.
3. **What the person on the answering machine chose**, read at every
   question exactly as it is read for their own questions
   (`docs/contracts/person-settings.md`) — so a model picked in Settings this
   morning answers for the machine down the corridor this afternoon, and no
   default decides it. A machine where nothing is chosen, nothing is running,
   or the settings file does not hold is answered `503` with
   `not-answered-here`. A machine whose person chose a **provider**, or a
   **paired machine** of their own, is answered `503` with `answers-elsewhere`: a question from a paired machine
   is never forwarded to a provider or to a third machine (ADR 0003, ADR
   0008), and nothing is written.
4. **The body**, now that the proof over it held: exactly what
   `alo-asking` sends — `model`, one `messages` entry in the `user` role, and
   `stream: false`. A field the shape has no place for, a stream asked for,
   no message, more than one, one in another role, or one that asks nothing
   is answered `400` with `not-a-question`. **The model the body names is
   checked and not used**: the question is put to the model the answering
   machine's person chose, and the answer names that model.
5. **The answer**, put to the model on that machine inside no turn of the
   asking machine's, recorded there as *a question answered for another
   machine* with the asking machine named (`docs/contracts/record-file.md`),
   and sent back as `200` with the body in the OpenAI-compatible reply shape
   — `object`, `model`, one `choices` entry with `message.role` `assistant`,
   `message.content` the answer, and `finish_reason` `stop` — under
   `content-type: application/json`. The answer leaves under the answering
   machine's egress indicator and is written in its record as having left,
   with the asking machine named; a rule on that machine that says nothing
   leaves holds the answer back, writes that down, and closes the connection
   with nothing on it. A model that was asked and did not answer is `503`
   with `nothing-answered-here`, and one that was not there to answer is
   `404` with `no-model-here`; neither writes anything.

## What the asking machine reads

The machine that asked reads three of those words and no others, and only on
their own status: `not-permitted` on `403`, `answers-elsewhere` and
`not-answered-here` on `503` (`alo_asking::NOT_PERMITTED`, `ANSWERS_ELSEWHERE`,
`NOT_ANSWERED_HERE`, spelled once for both ends). Each reaches the agent as
`alo_answering::RefusedThere`, rendered as a sentence in the language the person
**on the asking machine** reads — never as text the answering machine wrote.
Any other body on any other status is read as the status alone, as before. The
question left either way, so the asking machine's record keeps it as a
departure. The body names the model `what-that-machine-chose`
(`alo_choosing::WHAT_THAT_MACHINE_CHOSE`), which the answering machine checks
and sets aside. Reading these words is additive: a machine speaking the version
before it read every refusal as the status alone.

The `200` answer is additive to the version that answered every proven
question `503`: a machine speaking that version reads a `503` as it always
did, and reads a `200` as the answer it was always going to read.

## A workspace on the network

Added 2026-09-14, additively. *A self-hosted workspace on the network is
discovered, not configured — no DNS step* (`docs/features.md`, ADR 0003). What a
workspace **is** and what serves one belongs to `alo-workplace`; this section is
what that repository advertises against, and what every alo machine reads.
`crates/alo-nearby` (`WORKSPACE_SERVICE`, `WorkspacePresence`,
`advertising::about_a_workspace`, `reading::a_workspace_in`) is it as working
code.

**The service** is `_alo-workspace._tcp.local` — its own, beside `_alo-os._tcp.local`,
so an alo machine advertises exactly the same presence whether or not it serves a
workspace, and a workspace served by something that is not an alo machine is not
advertised as one.

**The advertisement** is an mDNS answer of three records and nothing else:

| Record | Name | Carries |
|---|---|---|
| `PTR` | `_alo-workspace._tcp.local` | the instance, `<identity>._alo-workspace._tcp.local` |
| `SRV` | the instance | priority `0`, weight `0`, the **port** the workspace answers on, and the target `<identity>.local` |
| `TXT` | the instance | exactly one entry, `v=1` |

That is the closed list: **which workspace** (the identity), **where it answers**
(the port — the address is measured, see below), and **the version it speaks**
(`v=1`). `<identity>` is thirty-two lowercase hexadecimal characters nobody chose:
on an alo machine it is that machine's own identity (`/var/lib/alo/machine-id`),
so a workspace hosted by a machine the person is paired with can be spoken of by
the name they gave it; a host that is not an alo machine keeps a random identity
of its own in the same shape, made once and kept across restarts, and never a
serial, a hostname or the organisation's name. One host serves one workspace.

**What a reader refuses.** A `TXT` entry other than `v=1` — any other key, or a
version this machine does not speak — refuses the whole advertisement rather than
being read around; so does an instance that is not an identity. A workspace that
wants to say more (its organisation, a login address, a certificate) is not found
at all, which is the point: an advertisement is read by everything on the network,
including machines nobody paired with. No `A` record is required or read.

**Where it answers is measured**: the source address of the answer, and the port
from the `SRV` record. The `SRV` target name is never resolved or used, so nothing
in a packet becomes an address a machine goes to.

**The question** a machine asks is one `PTR` question for
`_alo-workspace._tcp.local`, naming no workspace. An alo machine asks it beside the
question for `_alo-os._tcp.local`, on the same socket, and reads both kinds of
answer in one window.

**Finding a workspace confers nothing.** It is listed for the person
(`workspaces`, `docs/contracts/daemon-protocol.md`) and nothing more: no request,
verb or question reaches a workspace because it was found, no alo machine
connects to one until a person acts, and no alo machine dials an address a person
typed as a workspace. What reaching a found workspace takes is recorded in
`docs/autonomy/updates/a-self-hosted-workspace-is-found-not-configured.md`. The
person's act is `open-workspace` (`docs/contracts/daemon-protocol.md`), by identity
alone: the machine asks this question again at that moment, and hands the person's
session the one address that workspace answered from — refusing when none did, or
when more than one address answered for the same identity. The machine still
connects to nothing; the workspace client in the person's session does.

**An alo machine that hosts a workspace answers for it** (added 2026-09-14,
additively). `alo-agentd` is the one responder on an alo machine: it answers the
`_alo-workspace._tcp.local` question with the three records above, under **its own
identity**, when root has said where the hosted workspace answers in
`/etc/alo/workspace.toml` (`docs/contracts/hosted-workspace-file.md`), and steps
over the question when nothing is hosted — exactly as it steps over a printer's.
`alo-workplace`'s server on an alo machine therefore runs no mDNS responder of its
own and invents no identity; it is installed with that file. The answer to the
`_alo-os._tcp.local` question is the same bytes either way, and one packet asking
both questions is answered with two packets, one each. Hosting a workspace pairs
nothing and grants nothing.

## A machine on more than one network

Added 2026-09-15, additively. A machine is found on **every** network it is on,
not on whichever one the kernel would pick:

- `alo-agentd` joins the discovery group `224.0.0.251` on every interface that is
  up and running, carries multicast, has an IPv4 address and is not loopback, and
  joins again when the kernel says a network appeared. An interface that cannot
  be joined is a line in the service log; the others are still joined.
- **What is said on each network is the same bytes** — the same identity, the same
  port `7610`, and the same workspace answer. Presence never differs by network,
  and an answer to a question goes back to whoever asked, from the address on
  their network.
- A machine looking asks on each network it is on, from that network's own
  address, and a machine heard on two networks is **one machine with the address
  it answered from on each** (`alo_nearby::Found::also_at`). A workspace heard
  from one address on each of two networks is likewise one workspace; a workspace
  heard from two addresses on the **same** network is still two claims, and
  `open-workspace` still refuses it.
- There is no setting: no list of networks to advertise on, and no interface a
  person or an agent chooses (ADR 0003). A reader of this wire needs to change
  nothing; a responder on a machine with several networks answers on each.

## A network with no IPv4 address

Added 2026-09-15, additively. Two machines on one cable with no DHCP server, an
office whose router is down, and a network run IPv6-only have no IPv4 address in
common, and each interface on them still has an IPv6 link-local address. An alo
machine is found on those too:

- **The IPv6 group.** `alo-agentd` answers discovery on a second socket, IPv6
  only, at port `5353`, joined to **`ff02::fb`** (RFC 6762 §3) on every interface
  that is up and running, carries multicast, has an IPv6 link-local address
  (`fe80::/10`) the kernel has finished checking, and is not loopback — and again
  whenever the kernel says an address appeared. A machine looking asks `ff02::fb`
  on each such interface, from that interface's own link-local address. An
  interface that cannot be joined is a line in the service log; a kernel with no
  IPv6 in it is a line in the log and a machine discovered over IPv4 alone.
- **The same bytes in both families.** The question, the machine's answer (its
  identity and port `7610`) and the workspace answer are byte for byte what they
  are over IPv4, and a packet saying more than presence is refused whichever
  family carried it. An answer still carries no address record.
- **The scope rule.** A link-local address names no network by itself — every
  interface has one in `fe80::/10` — so an address measured off a link-local
  answer or connection is kept **with the interface it was heard on** (its scope
  id, RFC 4007 §11: `fe80::a406:e5ff:fe4b:ac9e%3`), and that is how it is dialled
  (`alo_nearby::HeardFrom`). An answer from a link-local address with no scope is
  refused (`alo_nearby::NotNearby::NamesNoNetwork`, nothing sent back) rather
  than written down, and a proposal from one is measured nowhere and so refused
  as not found. A scope is local to the machine that measured it and never
  crosses the wire.
- **The port in both families.** The port `7610` is answered in both families (an
  IPv4 peer is read as its IPv4 address, never as `::ffff:a.b.c.d`), so a
  proposal, a confirmation, a verb and a question arrive over link-local exactly
  as over IPv4, and a proposal's measurement is made in the family and on the
  interface its connection came from. Since 2026-09-16 that is one IPv4 listener
  per IPv4 network beside one IPv6-only listener rather than one listener in both
  families — see *The port is answered on every network*, below; nothing a reader
  of this wire sends or reads changes.
- **One machine in both families.** A machine heard over IPv4 and over IPv6 on
  one network is one machine with an address in each (`alo_nearby::Found::also_at`),
  and a workspace likewise. **Where both answered, the IPv4 address is the one
  written first and so the one a pairing dials**; where only IPv6 answered, the
  scoped link-local address is.
- **A scoped address is a departure on its interface.** A question from a turn to
  a paired machine at a scoped link-local address leaves under a departure holding
  the address, the port **and the interface** (ADR 0041), so the kernel permits it
  on the interface it was found on and refuses the same address on any other; a
  link-local address with no interface is not registered and the question is not
  put. The indicator and the record name the machine as over IPv4, by the name its
  person gave it, with no address.
- **A private IPv4 address is a departure on the network it was found on.** A
  machine heard at an IPv4 address on one network is written down with that
  network's interface (never spelled in the address), and a question from a turn to
  it connects from a socket held to that interface under a departure holding the
  address, the port **and the interface** (ADR 0044) — so the same address on
  another network, and a socket held to none, are refused. A provider's departure
  holds no interface and is decided as it always was. What crosses the wire, and
  what the indicator and the record say, is unchanged.
- There is still no setting: no *IPv6 on/off*, and no family chosen by a person or
  an agent (ADR 0003). A request naming one is refused on either door.

## The port is answered on every network

Added 2026-09-16, additively. A machine on two networks whose routers hand out the
same private range — the commonest office and the commonest home — was reachable
on its port only from the network its own route pointed at, because a TCP listener
held to no interface answers every handshake by the route (`docs/quirks.md`). So
the port is bound once per network:

- **One IPv4 listener per IPv4 network this machine is on**, each held to that
  network's interface (`SO_BINDTOIFINDEX`), beside **one IPv6-only listener** held
  to nothing. Which networks is what the kernel reports — every interface that is
  up and running and has an IPv4 address, **loopback included**, multicast not
  asked for — and the listeners follow the kernel's network notifications as the
  discovery joins do. A network that will not bind is a line in the service log
  and the others are still bound; a machine whose interfaces cannot be read binds
  one listener held to nothing and says so.
- **The port is still `7610` and nothing about it is a setting** (ADR 0003): which
  interfaces are listened on is what the machine is plugged into, never a list
  anybody writes down.
- **What crosses the wire is unchanged.** The framing, the six paths, the proofs
  and every reply are what they were; a machine speaking this wire sends and reads
  exactly what it did.
- **The network a connection arrived on is the listener that accepted it**, not
  the address it was made to — a machine's own address can belong to an interface
  the packet did not come in on (Linux's weak host model). A proposal is measured
  on that network, so `192.168.1.20` on the cable is not judged against
  `192.168.1.20` on the Wi-Fi.
- **A proposal and a confirmation are dialled held to the network the other
  machine was heard on** (ADR 0044), as a question from a turn already was — so
  two machines that found each other on a network the route does not point at
  pair, rather than sending the confirmation to whoever the route reaches at the
  same address.
- **And a discovery answer leaves on the network its question arrived on**, which
  is the section below: being reachable on a network and being *found* on it are
  two halves, and this section is the first.

## A discovery answer leaves on the network the question arrived on

Added 2026-09-16, additively. The same two networks that could not reach this
machine's port could not find it either: the socket it answered *who is here* on
was held to no network, so its answer to `192.168.1.20` on the cable left by the
route — to somebody else at that address, or to nobody.

- **Discovery is answered on one datagram socket per network this machine is on**,
  each held to that network's interface (`SO_BINDTOIFINDEX`) and joined to
  `224.0.0.251` on the networks that carry multicast. Which networks is what the
  kernel reports — the same rule the port is listened on by, loopback included —
  and they follow the kernel's network notifications. A network that will not take
  a socket is a line in the service log and the others still answer.
- **An answer to a question that arrived on one network leaves on that network.**
  A machine asking on the cable is answered on the cable, whatever this machine's
  route says, and nobody else is sent anything.
- **A question whose network this machine cannot read is answered on no network**
  rather than by the route: an interface the kernel numbers zero, and a machine
  that cannot read its own interfaces at all, answer nobody and say so in the
  service log. Silence rather than an answer sent to a machine that did not ask.
- **Over IPv6 nothing changes.** A question over IPv6 discovery comes from a
  link-local address carrying the interface it was heard on, and the kernel answers
  back out of that one; the IPv6 socket is one, held to nothing, as before.
- **What is said is unchanged, byte for byte, on every network and in both
  families**: the same identity, the same port and the same workspace answer. A
  machine that said something different on one network would be two machines to
  whoever asked on both. Nothing a reader of this wire sends or reads changes, and
  which interfaces are answered on is still no setting (ADR 0003).

## A cable pulled, and plugged in again

Added 2026-09-16, additively. Nothing on this wire changes; this says what a
reader of it can rely on while a machine's networks come and go.

- **A network this machine is taken off is a network it is no longer found or
  reached on.** Discovery stops being answered there, the port stops being
  listened on there and the group is left, when the kernel says the network went —
  and the machine stays found and reachable on every network it is still on, with
  the service running and its person's door answering.
- **A network it is put on again is found and reached at once**, with no restart:
  the first question asked there after the kernel says the network is up is
  answered, and the port answers there. A cable re-laid is a new interface to the
  kernel, and it is treated as one.
- **What is said is the same bytes before, during and after**, on every network.
- **A network that will not take the port when it comes back** — somebody else
  holding it there — is a line in the service log; discovery is still answered on
  that network, and the port is listened on there the next time the kernel says
  the network changed and it is free.

## A cable with no IPv4 address, pulled and plugged in again

Added 2026-09-16, additively. Nothing on this wire changes; this says what a
reader of it can rely on when the network that comes and goes carries IPv6
link-local addresses only.

- **Pulled — its link set down, or the cable gone altogether — the two machines
  are not found by each other**, a proposal to the other machine is refused before
  anything is sent, and both services go on answering their person's door.
- **Plugged in again, each finds the other with no restart**, at the link-local
  address it answers from now and **with the interface it was heard on now**. A
  cable re-laid is a new interface, usually with a new number, and a link-local
  address is written down with the new one — including where the new interface
  happens to have the number an old one had.
- **Nothing is dialled at an interface that is gone.** No address is kept: a
  proposal after the cable comes back is measured at that moment, on the
  interface the machine was heard on, and the machine asked measures the
  proposer at the address its connection came from.
- **What is said is the same bytes before and after.**

## A cable deleted and laid again before the machine looks

Added 2026-09-16, additively. Nothing on this wire changes; this says what a
reader of it can rely on when a network goes and comes back faster than a machine
reads its interfaces — a dock re-enumerating its adapters, a namespace rebuilt.

- **A cable deleted and laid again is a new network, even at the old interface
  number.** A machine does not keep answering on the strength of a number it saw
  before: when the kernel says an interface was deleted, discovery on that network
  is answered again from the start, and the group joined afresh, whether or not
  the number is back by the time the machine looks.
- **The first question asked there once the machine has followed the kernel is
  answered, and the port answers there**, over IPv4 as over IPv6, with no
  restart — at the old number and at a new one alike.
- **An adapter that comes back identical is still a new network.** Added
  2026-09-17, additively: a cable with no IPv4 address laid again with its number,
  its name and its hardware address — and so its `fe80::` address — is joined
  afresh, found by the first question once the machine has followed the kernel,
  and a pairing proposed there is measured and completed on that interface.
- **Where a machine cannot tell what went** — the kernel's notices were lost or
  unreadable — it answers every network again from the start once, rather than
  assume nothing went.
- **What is said is the same bytes before and after.**

## Versioning

The `1` in every path is the version of this wire. Anything that would stop a
machine speaking version `1` from being answered correctly gets a new number
beside it; anything additive — a new path, a new word at the door, a new
field an older reader ignores — does not.
