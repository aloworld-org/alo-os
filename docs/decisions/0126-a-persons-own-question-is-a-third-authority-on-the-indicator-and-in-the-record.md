# ADR 0126 — A person's own question is a third authority on the indicator and in the record

**Status:** **proposed, 2026-10-19, by the Mac lane.** Nothing is built in the
change that proposes it. It blocks the remainder of
[ADR 0125](0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md),
which is task 11 of `docs/autonomy/the-terminal-in-full-plan.md`.
**Context:** law 1; law 2 (*the law binds the agent, never the person*);
[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md) (an organisation's policy on machines it
owns); ADR 0125; `docs/contracts/record-file.md`;
`docs/contracts/native-egress-indicator.md`; `crates/alo-egress`,
`crates/alo-record`, `crates/alo-asking`, `crates/alo-capability`.

## The question

ADR 0125 decided that a person's own question goes to their provider **through
the daemon**, because the daemon holds the one indicator and writes the record.
It left the message shape to the daemon contract and said nothing about this,
which only became visible when somebody started building it:

**Who does the indicator name, and who does the record name, when the thing
that caused the egress is the person?**

Every type on law 1's road answers *whose authority is this under* with an
`alo_capability::Grantee`, and a `Grantee` is an agent or an application. A
person is neither. `crates/alo-ask/src/main.rs` already says so, in the comment
on the grantee it carries and never reads:

> `alo_capability::Grantee` is an agent or an application, and a command a
> person typed is neither. It is named for itself rather than for the person,
> because the alternative is writing a login name into a type whose two kinds
> are both software. **That is a finding about the shape of `Asking::by` rather
> than something to fix by inventing a third kind here**, and it costs nothing
> today precisely because this door never reads it.

**The provider road is the door that reads it.** The indicator shows it and the
record keeps it, so *costs nothing today* expires exactly here.

## What was measured, 2026-10-19

| asked | answer |
|---|---|
| what `alo_egress::Leaving` holds | `agent: Grantee`, doc'd *whose authority this is under* |
| what the egress policy actually reads | **the destination only** — `permits` branches on `destination().stays_in_the_building()` / `.is_in(region)`, never on the agent |
| doors onto the indicator today | **two** — `Showing::Agent(Leaving)` and `Showing::Itself(OnItsOwn)` |
| what `Showing::agent()` answers for an errand | `None`, doc'd *the honest answer rather than a placeholder … naming an authority would be inventing one* |
| what the errand door does about policy | **nothing** — *this cannot refuse, and it can never fail to show* |
| `Happened` variants in `alo-record` | 21, of which 10 arms name `NotBounded` and 11 name `LeftOnItsOwn` |
| `Entry` constructors | 29 |
| where a person's name would land | `Entry::left` does `Line::of(departing.agent().as_str())` |

## The precedent is already here, and it already argued this case

`crates/alo-egress/src/underway.rs` exists because alo OS's **own** errands are
an egress with no agent behind them, and its header makes this ADR's argument
for the second authority before a third was wanted:

> A `Departing` answers *whose authority is this under* with a `Grantee`, and
> `alo-record` writes that into every entry it makes about something leaving.
> **Widening it to answer *nobody* would make every one of those answers an
> `Option`, in the crate whose whole job is saying who did what** — so the two
> authorities stay two types, and the thing they share is the indicator, which
> is the thing a person actually looks at.

So the shape of the answer is not an open question. It is: **a third authority,
as its own type, sharing the indicator.**

## The decision proposed, in one line

**A person's own question is a third authority: it goes on the same indicator,
under no agent's name, and the record keeps it as its own kind of entry — and
unlike alo OS's own errands, it asks the egress policy first.**

## What that means, concretely

| alo OS on its own | **a person's own question** | an agent |
|---|---|---|
| `Showing::Itself(OnItsOwn)` | `Showing::ThePersons(TheirOwn)` | `Showing::Agent(Leaving)` |
| `Indicator::beginning_on_its_own` | `Indicator::beginning_for_the_person` | `Indicator::beginning` |
| hands back an `Underway` | hands back a departure of its own | hands back a `Departing` |
| **cannot refuse** | **can refuse** — asks `EgressPolicy` | can refuse |
| `Happened::LeftOnItsOwn { errand, … }` | a kind naming no agent | `Happened::Left { agent, … }` |
| `Showing::agent()` → `None` | `Showing::agent()` → `None` | `Some(&Grantee)` |

### The one difference that is not symmetry, and it is the important one

**The person's door asks the policy; the errand door does not.** The errand door
cannot refuse for a stated reason — a rule about where questions may be answered
would, applied to alo OS fetching a model, stop a machine set to answer on its
own hardware from ever obtaining the model it would answer with.

**None of that reasoning transfers to a person's question.** On a machine an
organisation manages, a rule about where inference may happen is *precisely*
about a person's question, and law 5 says an organisation's policy on machines it
owns may truly forbid. So:

> **Routing a person's question through the errand door would send it around the
> organisation's own rule, and the door could not have refused it even in
> principle.** That is the mistake this ADR exists to make unavailable, and it is
> the cheap-looking shortcut: `beginning_on_its_own` needs no policy argument,
> so it compiles first.

### Why not widen `Grantee` with a third kind

The owner gave this lane permission to widen `alo-capability` on 2026-10-18, so
this is declined on its merits rather than for want of permission.

- **A person holds no grants.** `Grantee` is *who a grant is for*; the person is
  who *makes* them. A third kind would put the grantor in the list of grantees,
  in the crate law 2 is enforced in.
- **`Grantee::as_str()` would have to answer something**, and the only candidates
  are a login name — which is the thing `alo-ask`'s comment declined to write
  into a software type — or a placeholder, which is a name invented for a person
  who has one.
- **Every `permitting` and `allowing` door would gain an unreachable case.** Two
  kinds that *never answer for each other* is a property of that file; a third
  that answers for neither and asks for nothing is a case every reader has to
  rule out.

### Why not reuse `Happened::Left` with the person's name in `agent`

Because then a record says an agent asked, and law 2's record is the one place
that distinction is load-bearing. `docs/contracts/record-file.md`'s own answer to
a new writer is a new kind, and *a new kind of `happened` is additive* is already
the decision for the two most recent kinds.

## What this costs, said plainly

- **Three new kinds in `alo-record`**, not one: something left for the person, the
  policy held it back, and there was no boundary to put it inside. Each needs an
  arm in the ten-to-eleven `impl Happened` matches, and each arm is a judgement
  (*was something stopped? whose authority? did it cause egress?*) rather than a
  line to copy.
- **A third authority in `alo-egress`**: the egress, the departure, and the
  refusal, mirroring `itself.rs` / `underway.rs` / the agent's `NotPermitted`.
- **`alo-asking`'s `Asking::by` takes a `Grantee`** and the person's road cannot
  supply one, so that door needs the same treatment or a sibling.
- **Two contracts gain a section**: the record file's new kinds, and the egress
  indicator's third line. The indicator's **drawing surface does not change** —
  it is one more line on the one list, naming no agent, exactly as an errand's
  line names none today. That is what keeps this inside
  `docs/contracts/native-egress-indicator.md`'s *this document describes only
  the drawing surface*.
- **It is a day's work, not an afternoon's**, and it is why ADR 0125's remainder
  did not land on 2026-10-19. Said here rather than discovered twice.

## What this does not decide

- **The words on the indicator's third line.** `alo_egress::Leaving::said`
  composes the agent's three sentences in one place on purpose; the person's
  needs one sentence in the same place, in the vocabulary, and whether it reads
  *a question you asked is going to …* or something else is a wording question,
  not this one.
- **The daemon's message shape.** Still ADR 0125's deferral to
  `docs/contracts/daemon-protocol.md`.
- **Whether a person's question becomes an agent turn.** Still no — ADR 0125
  marked that *the part most likely to be got wrong*, and this ADR is what makes
  the alternative buildable rather than only preferable.

## The test that decides whether this works

Three, and the second is the one worth writing first:

1. A person with a provider in their settings asks a question at a shell, and the
   egress is **on the indicator while it is in flight** with **no agent named**,
   and in the record afterwards as its own kind.
2. The same person on a machine whose organisation set *nothing leaves* is
   **refused**, in words, and nothing leaves — proving the person's door asks the
   policy that the errand door cannot.
3. A record read back a week later can tell a question the person asked from a
   question an agent asked, without reading anything but the entry.
