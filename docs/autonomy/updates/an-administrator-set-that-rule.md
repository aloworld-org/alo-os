# An administrator set that rule

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-agentd`, `alo-choosing`)
- Contributor: Claude Code
- Task: Explain organisation-policy refusals consistently
- Status: **the wording exists and the ordering is fixed.** Two dependencies
  remain and are named below; **the settings integration is not finished** and is
  not claimed to be.

## The ordering was wrong, and that is the substantive fix

`put_to_a_model` fetched the provider's key from the keyring and **then** asked
whether the organisation's rule permitted the place at all. A question the policy
refused had already taken a person's credential out of their store, to be thrown
away.

The rule is now asked first. Nothing reaches for a key on the way to saying no.

**Proved by mutation, because the first version of the test did not prove it.**
That version counted connections on the fixture's bus afterwards and **passed
with the wrong order** — the keyring handle is a temporary dropped inside the
call, so the count was back to baseline before anything looked.

Two tests now, and they are independent:

- `a_rule_that_refuses_never_reaches_for_the_key` uses a real, deliberately
  **empty** keyring. A daemon that looked would answer *there is no key saved*,
  a sentence this crate names exactly, so the refusal being the **rule's** is the
  evidence. That is an argument about *one scenario's sentence*, which is why it
  is not the only one.
- `a_rule_that_refuses_opens_no_connection_to_the_keyring` **watches the bus**. A
  lookup has to open it, so a socket nothing ever connected to is a lookup that
  never happened, whatever any sentence said. The bus is a Unix socket the test
  binds and never answers on; it passes `TheBus`'s checks, so the daemon would
  reach it if it reached for a key, and the accept count is zero.

Both fail when the lookup is put back in front, each for its own reason.

## One refusal value, worded once

`alo_models::NotAllowed` still decides and still words the refusal. `alo-agentd`
adds one sentence, `AN_ADMINISTRATOR_SET_THAT_RULE`, and `doing::a_rule_refused`
is the only thing that produces it — **both doors go through it**, the provider's
and the local one, so a person cannot meet two wordings of one rule.

The rule's own sentence is carried **inside** ours through `Filling::and_said`,
which is the typed mechanism `alo-models` already uses for the same purpose. Not
two sentences composed side by side: one value, one rendering, one language.

It names the rule and **that** an administrator set it. It does not name **who**:
`agentd.toml` is a file, not a person, and an invented name is one somebody could
go and ask for and not find.

### The origin is carried, not inferred — and not from an `Option` either

`Option<SourcePolicy>` was the first shape and it is not enough. It collapses two
different things into `Some`: a rule an administrator wrote, and a rule the
machine's owner chose for themselves. Attribution read off *a policy was
supplied* would tell a person an administrator restricted them when nobody did.

So the daemon carries `TheBound` — `Nobodys`, `ThePersons(policy)` or
`AnOrganisations(policy)` — and only the last earns the sentence. Nothing
supplies `ThePersons` today and this adds no key for it; it exists so the
distinction is **unrepresentable to get wrong**.

`a_persons_own_strict_rule_names_no_administrator` is the case an `Option` could
not tell apart: a policy *was* supplied, it is the strictest one there is, the
question is refused by it — and no administrator is named. It also asserts the
refusal is the rule's own words, so it is not passing because nothing happened.

## The one gap in this crate's vocabulary, and the rule it did not break

`words.rs` asserted that **no** sentence here has a gap, because a gap is how
text somebody else wrote reaches a person. The invariant is *no client text*; the
test approximated it as *no gaps*, which held while nothing had one.

The new sentence has one, and its only filling is `NotAllowed`'s own rendering
through `and_said` — this repository's words, never anything a client sent. So
the test exempts that word **by name**, and a second test asserts the exemption
list is exactly the set of words with a gap, so a stale exemption cannot quietly
turn the rule off.

**An exemption list is bookkeeping, not the security property**, so the property
itself is tested: `the_gap_takes_a_typed_refusal_and_never_a_clients_words` puts
braces and an instruction to ignore the rule into **the question** — the one
thing on this path a client controls — has it refused by policy, and asserts none
of it appears in what comes back, that no braces survive, and that the refusal is
byte-for-byte the one rendered from the typed refusal alone.

## The stale documentation is corrected

`alo-choosing`'s `bound.rs` said no rule an organisation can set refuses anything
a person can currently choose. That stopped being true when a person could choose
a provider: `Picked::FromAProvider` resolves through `Provider::source` to
`InferenceSource::Hosted`, and `ThisMachineOnly` refuses exactly that.
`a_rule_can_now_refuse_a_place_a_person_can_choose` holds it up through the
crate's own API, and pins the invariant that makes the sentence sayable — a
refusal from there always means an organisation set a rule, because `asking(None)`
is `Anywhere` and never refuses.

## Two dependencies, and neither is closed

**1. Production supplies no organisation bound.** `starting.rs` passes `None`,
because `docs/contracts/machine-description.md` has no key for a policy — the
queue's item 21o is where whether it gains one is decided. So **every machine
running this today is unmanaged**, the new sentence is unreachable in production,
and it is reachable only where a bound is supplied directly, which is what the
tests do. **The settings integration is not finished.**

**2. A question refused by policy is not recorded.** `alo_record::Entry` is built
around a `Call` — a verb — and `Turning::writing_down` is private, deliberately:
it is *the only road to the record in this crate*. A refusal that happens before
the turn therefore reaches no record at all, and that is true of the policy
refusal both before this change and after it.

So *agreement between the response and the record* is tested as far as it can
honestly be: `both_doors_word_a_rule_the_same_way` asserts there is **one** value
and that both doors produce it, so anything later handed the refusal is handed
the same sentence. **Recording a refused question needs a record entry that is
not about a verb, and that is a change to `alo-record` and `alo-turn`'s surface
rather than something to bolt on here.**

## What is preserved

All three model choices: local models, a provider the person added, and alo's own
service — which is a provider like any other (ADR 0014). The policy check is the
one that already existed; nothing re-decides it and nothing reworded it. No
fallback: a refused provider does not become a question for a model on this
machine, and that is asserted.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible, because no machine can state a bound
yet. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — 21l: the wording exists in `alo-agentd` and the
policy check precedes credential retrieval; blocked on 21o for a machine that can
state a bound, and on a record entry for a question refused before a turn.

**docs/autonomy/STATE.md** — an organisation's policy refusal is now worded once,
naming the rule and that an administrator set it, never inferred from the policy
value; the rule is asked before the keyring is opened; production still supplies
no bound.
