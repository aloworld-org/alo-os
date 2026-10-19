# ADR 0126 — A person's own question shows no indicator line, because they chose where it goes

**Status:** **accepted, 2026-10-19, by the owner**, who rejected the proposal
this file was opened to make. Proposed the same day by the Mac lane as *a third
authority on the indicator and in the record*; the owner's answer removes the
need for one. Nothing is built in the change that records it.
**Context:** law 1 (*every network egress **an agent causes***); law 2 (*the law
binds the agent, never the person*); law 5 (*never offering is not taking
away*); [ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md);
[ADR 0125](0125-a-command-does-not-report-an-egress-it-asks-a-service-that-can-show-one.md);
`docs/contracts/native-egress-indicator.md`; `crates/alo-egress`,
`crates/alo-record`, `crates/alo-ask`.

*The filename names the proposal this began as. It keeps that name because a
rename leaves every link to it reading exactly as it did before — see
`README.md` in this directory, and the five days ADR 0019 spent pointing at a
file that no longer existed.*

## What the owner decided, 2026-10-19

Shown three options — no line at all, a line naming nobody, or a line saying
*you* — they answered:

> *I think we should modify this adr, because users select the models themselves
> in the settings so no need to show all those indicators*

**Read as: a person's own question causes no indicator line.** The person chose
the provider in Settings; showing them an indicator for the thing they chose
tells them nothing they do not already know.

They had also corrected the lane's first draft earlier the same day, and that
correction is what dissolved the problem rather than solving it — see *What
`{agent}` actually holds*, below.

## Why this is law 1's own reading, not an exception to it

Law 1 in full:

> **Nothing leaves silently.** Every network egress **an agent causes** is
> visible at the moment it happens and afterwards in a record.

**An agent causes.** A question a person typed at their own shell is not
agent-caused, so the sentence that defines law 1 does not reach it. The headline
reads wider than the body — which is the same crack `CLAUDE.md` already records
in law 5, where *its headline is broader than its body and two careful readers
disagreed about which bound*. Law 2 settles the direction: *the law binds the
agent, never the person.*

And law 5 covers what was not built: **never offering is not taking away.**
Nobody was ever shown an indicator line for their own question, so not building
one withdraws nothing. This would be a different decision if such a line existed
and were being removed.

### The sentence that most looks like it forbids this, answered

Law 5 names the egress indicator in its list of protections that stay on at
every setting:

> Two kinds of protection stay on at every setting, because neither takes a
> choice away: those that restrict nothing (the record, undo, **the egress
> indicator**, plain warnings) …

**That sentence is about a protection a person could otherwise switch off, and
it is untouched here.** The indicator stays on, at every setting, showing
everything it shows today. Nothing on it is being removed and no setting is
being added to quiet it. What this ADR decides is that a surface which has only
ever shown what *software* on this machine is doing does not grow a row for the
person — and law 5's own clarification is the test: *never offering is not
taking away. Withdrawing is.* `docs/features.md` is the record of what was
offered — and **it made the promise in law 1's own scope**, which settles this
rather than merely supporting it:

> ★ **The egress indicator**: every network egress **an agent causes**, visible
> at the moment it happens …

So what a person was offered is the agent-scoped promise, and it is kept in
full.

ADR 0125 argued the other way, in one sentence — *a person who chose OpenAI has
chosen that their question leaves; they have not chosen to be unable to see it
leave*. **That is an argument and not the law**, and it is the sentence this ADR
supersedes.

## What was measured, 2026-10-19, before the proposal was rejected

Kept because the facts hold either way, and the next person to open the
indicator will want them.

| asked | answer |
|---|---|
| what `alo_egress::Leaving` holds | `agent: Grantee`, doc'd *whose authority this is under* |
| what the egress policy actually reads | **the destination only** — `permits` branches on `destination().stays_in_the_building()` / `.is_in(region)`, never on the agent |
| doors onto the indicator today | **two** — `Showing::Agent(Leaving)` and `Showing::Itself(OnItsOwn)` |
| what `Showing::agent()` answers for an errand | `None`, doc'd *the honest answer rather than a placeholder … naming an authority would be inventing one* |
| what the errand door does about policy | **nothing** — *this cannot refuse, and it can never fail to show* |
| `Happened` variants in `alo-record` | 21, of which 10 arms name `NotBounded` and 11 name `LeftOnItsOwn` |
| where a person's name would have landed | `Entry::left` does `Line::of(departing.agent().as_str())` |

## What `{agent}` actually holds — the owner's correction, and why it ended the question

The slot a person's question could not fill is not an abstraction. It holds
**alo chat's handles**, which is how a person addresses an app's agent in a
channel: `@mail are we in contact with ABC?`

Counted in `alo-workplace`, in its tracked `.ts`, `.tsx` and `.md` files:

| handle | mentions | | handle | mentions |
|---|---|---|---|---|
| `@alo` | 101 | | `@sites` | 17 |
| `@mail` | 34 | | `@docs` | 17 |
| `@tasks` | 30 | | `@meet` | 16 |
| `@inventory` | 29 | | `@insights` | 15 |
| `@agenda` | 23 | | `@drive` | 14 |
| `@sheets` | 18 | | `@chat` | 14 |

`@alo` is this machine's own agent — `alo-notifying` renders it *@alo, the agent
on this machine*. The other eleven are `alo-workplace`'s. **`@calendar` does not
exist**; the diary agent is `@agenda`.

**So the indicator is a list of what software on this machine is doing, named by
the handles a person would type at that software.** A person is not in that
list, cannot be `@`-mentioned, and was never granted anything — they are the one
who does the granting. The lane read that as *so a third kind is needed*. The
owner read it as *so a person does not belong on this surface at all*, and the
second reading is the one law 1's text supports.

*This repository uses `@mail` as its illustrative agent in a translator's note
and in fixtures. It is a workspace handle borrowed as an example, not an alo OS
agent — recorded because this ADR's first draft reasoned from it as though it
were one, and reached the wrong conclusion by doing so.*

## What this decides, in one line

**A question a person asks from their own shell causes no line on the egress
indicator, no third authority in `alo-egress`, and no new kind of entry in
`alo-record`.** The indicator stays exactly what it is today: what *software* on
this machine is doing.

## What is dropped

Everything the proposal would have built:

- a third `Showing` variant, its egress type, its departure and its refusal;
- three new `Happened` kinds and their arms in the ten-to-eleven `impl Happened`
  matches;
- the same treatment for `alo_asking::Asking::by`;
- a section in `docs/contracts/native-egress-indicator.md`, and one in
  `docs/contracts/record-file.md`;
- a sentence in the vocabulary and its 24 translations.

**The day's work the proposal costed is not spent.**

## What stays on, and this is the part not to lose

**The organisation's rule is still asked, before anything leaves.** That is
[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md) and law 5's
own carve-out — *Only the law, or an organisation's policy on machines it owns
(ADR 0016), may truly forbid* — and it is **not** law 1 and **not** an
indicator. A person
on a machine whose organisation set *nothing leaves the building* is refused, in
words, and nothing leaves.

This is a different mechanism from the one this ADR drops, and it is already
built and already ordered correctly: `alo-agentd`'s
`a_provider_ready_to_be_asked` asks the rule **first**, before the road and
before the keyring, so a question a rule refuses never reaches for the person's
credential. Landed 2026-10-19 (#826) with the ordering proved by mutation rather
than asserted.

**Also unchanged:** nothing silently substitutes. ADR 0008's *never a silent
fallback* and the setting the owner chose on 2026-10-18 — whether alo may use
another model when the chosen one is unavailable — both stand, and the second
landed in #822. A person who chose a provider and turned that off is refused and
told which of their own settings decided it.

## The record — recommended, not decided here

Law 1's record clause has the same subject as its indicator clause (*an agent
causes*), so it does not reach a person's own question either. The owner's words
were about what is **shown**, and a record is not shown — it is kept for
whenever somebody wants to look.

**Recommended: keep a record, written by the command itself, into the person's
own record file.** That is not a new pattern: ADR 0125 already decided exactly
this for a person's sealed-box run, on `docs/contracts/record-file.md`'s own rule
that *a record file has one writer* and its own answer that a new writer gets a
new file. It costs the person nothing, refuses them nothing, and keeps *I can
check what my machine did* true.

**It is also what makes the rest cheap.** A command that writes its own record
needs nothing only a long-lived service can do — which is the next section.

## What this means for ADR 0125 — measured after the first answer was wrong

**The first version of this section said `alo-ask` may now ask the provider
directly, and that the work left was an afternoon. It was written without
measuring and it is false.** It is kept as a heading rather than deleted
because the reasoning that produced it is the reasoning somebody will repeat:
with no line to show, a command appears to need nothing a service has.

**It needs two things a service has, and neither is about law 1.** Measured
2026-10-19:

| what a person's question needs before it leaves | can a command do it? |
|---|---|
| the organisation's rule, from `[questions]` in `/etc/alo/agentd.toml` | **yes, in principle** — the file is world-readable by design (`docs/contracts/machine-description.md`: *the file may be world-readable — `0644` in `/etc` is the ordinary case*) and holds no secrets … |
| … read with the same checks the daemon reads it with | **not without a crate that does not exist**: `describing.rs` is 1235 lines and `described.rs` 475, both inside `alo-agentd`, which **nothing depends on** (measured: 0 of 60-odd crates). A command would either pull the whole daemon in or re-implement the symlink and ownership checks on the file that names which login is the agent |
| the destination-level egress rule, which is **not** the same check | the rule is the same rule; the *subject* is not. `Answering::chosen` asks about the provider's **stated source**, and the egress policy asks about the **resolved address**. A provider claiming the EU and resolving to Virginia passes the first and fails the second |
| which road out the question takes, from `[proxy]` | yes — same world-readable file |
| **signing in to a proxy that asks who this machine is** | **no.** The password is a systemd credential at `/run/credentials/alo-agentd.service` (ADR 0059), which belongs to that unit. A command run by the person cannot read another unit's credentials — and a command that went straight out instead would be repeating a fault this repository has already had and fixed. `the_road_out`'s own header: *on a company network with no other route out, a machine read its organisation's proxy off the disk and then asked a provider directly* |

**So ADR 0125's route stands, and the reason is the one this ADR's own earlier
draft named as the thing that would overrule it:** one place that holds the
credentials. Two credentials, as it turns out — the provider's key and the
proxy's password — and the proxy's cannot be moved, because it is systemd's
answer to *which unit may read this*.

**What ADR 0126 still changes about ADR 0125's route** is the part that was
blocking it: the daemon answers the person **without a turn and without an
indicator**. That was the whole difficulty, and it is gone.

### So the work left, re-estimated

| | |
|---|---|
| 1 | a door in `alo-asking` that asks a provider **with no indicator**, which still asks the egress policy about the resolved address, because that check is not the one `Answering::chosen` made — **and this is the one piece that needs care** |
| 2 | `FromAPerson::TheirOwnQuestion` and `ToAPerson::Answered` in `alo-protocol` |
| 3 | a handler on the person's door, reusing `a_provider_ready_to_be_asked` (#826, built) |
| 4 | `alo-ask` opening `/run/alo/<uid>/agentd.sock` — the same one-connection-one-line-one-answer shape `alo-changing/src/door.rs` already uses (ADR 0017) |
| 5 | `docs/contracts/daemon-protocol.md` |

Still much smaller than the rejected proposal, and **not an afternoon.**

*Recorded this way on purpose. The lane had just written down that its mistake
was arguing* how *before asking* whether*, and then published an estimate in
the same file without measuring it. The second fault is the first one wearing a
different hat: an answer produced fluently feels measured.*

## What this costs, said plainly

**A quiet indicator stops meaning *nothing left this machine*, and starts
meaning *no software on this machine sent anything*.** That is a real narrowing
of what a person can read off that surface, and it is the price of this decision.

**The phrase is `docs/features.md`'s own**, which is why this cost is named
rather than argued away. Describing where the indicator sits, it says the
indicator is placed so that

> *"nothing has left this machine"* sits where a person already glances rather
> than somewhere they must learn to look

— and after this decision that glance answers a narrower question than those
words suggest. **Two sentences in one document, one scoped to an agent and one
not**, which is the same headline-versus-body shape as the laws above. Worth a
line in `docs/features.md` when somebody next edits it; not worth inventing a
surface to make the looser sentence true.

Three things that do *not* change, because each is sometimes offered as a reason
to reconsider:

- **The published measurement is unaffected.** Law 1's *a working day produces
  zero inference egress* is scoped *with a local model* and measured **at the
  network boundary**, not off the indicator. A person using a provider was never
  inside that claim.
- **No telemetry, either way.** Nothing about this gives alo OS a road to report
  anything about anybody.
- **An agent's egress is untouched.** Every line the indicator shows today it
  shows tomorrow.

## The test that decides whether this works

1. A person with a provider in their settings asks a question at a shell and
   **gets an answer**, with the terminal saying where it came from — *by OpenAI,
   in the United States* — and **no line on the indicator**.
2. The same person on a machine whose organisation set *nothing leaves the
   building* is **refused, in words, with nothing sent and the keyring never
   opened**. This is the one worth writing first: it is the half that proves the
   organisation's rule survived this decision.
3. An agent asking the same provider on the same machine **still puts a line on
   the indicator**, so what was dropped was dropped for the person only.
