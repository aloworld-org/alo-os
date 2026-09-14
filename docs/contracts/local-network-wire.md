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
| `/v1/chat/completions` | `POST` | an OpenAI-compatible question | proven and judged; see below |

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

## The question path, today

A question is told apart, proven, and judged against the pairing's own list:
a pairing that does not permit asking this machine's models
(`alo_nearby::MayAskIts::Models`) is answered `403` with `not-permitted`. A
question that proves itself and is permitted is answered `503` with
`not-answered-here` and nothing is written, because the door that answers it
from this machine's own model is not built yet; the asking machine's person
is told the machine down the corridor could not answer, which is true. When
that door exists the same path answers `200` in the OpenAI-compatible shape,
additively.

## Versioning

The `1` in every path is the version of this wire. Anything that would stop a
machine speaking version `1` from being answered correctly gets a new number
beside it; anything additive — a new path, a new word at the door, a new
field an older reader ignores — does not.
