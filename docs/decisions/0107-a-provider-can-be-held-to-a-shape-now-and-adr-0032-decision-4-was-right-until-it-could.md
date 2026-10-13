# ADR 0107 — A provider can be held to a shape now, and ADR 0032 decision 4 was right until it could

**Status:** **proposed, 2026-10-13**, by the Mac lane, for the owner. It asks to
change one clause of an accepted decision, so it does not change it.

## What ADR 0032 decided, and why it is being revisited

[ADR 0032](0032-a-local-model-is-held-to-the-envelope-not-the-call.md) decision 4:
only the pinned runtime may be asked to hold its answer to the protocol's
envelope. `alo-turn`'s `asking.rs` carries that out — the provider arm and the
paired-machine arm do not read `held` at all, and say so.

**That was right, and for a reason that has changed.** The only way to hold a
model to a shape was `llama.cpp`'s `grammar` field, and a hosted service answers
400 to it. A provider therefore could not be held to anything, so reading `held`
on that arm would have been a field with no effect.

Hosted services now accept a JSON schema with `strict`. Measured against OpenAI
on 2026-10-13, with the key the owner provided.

## What the measurements say

**The failure this is for.** Under the instructions a turn really shows, three
rounds of `alo-driving`'s fixed ten against `gpt-4o-mini` scored 30 of 30. Under
`Instructions::SeveralSteps` the same model scored 24 and then 26 of 30, and
**every failure was one kind** — the verb's name put where the protocol's door
belongs, `{"format":1,"asks":{"find_in_folder":{…}}}`. The agent's own
end-to-end run met the same error once in four attempts and the turn ended on
it.

**A local model cannot write that.** The runtime will not let it. So the shape
error is a hosted-only failure, and it is one of two things left standing
between the agent and work it can be trusted with.

**Held to a schema, the fixed ten scored 29 of 30 — `reliably` — with no shape
error at all.** The single failure was `MoreThanOneMessage`, a different kind.

## What a provider will and will not accept, measured

The envelope [`the_envelope`] hands the local runtime **cannot be sent to a
provider**, and the provider's own words say why:

- *in context=('properties', 'asks'), 'oneOf' is not permitted.* `anyOf` is
  accepted and means the same here, because the branches forbid each other's
  keys.
- *'additionalProperties' is required to be supplied and to be false.* So *a
  door whose contents are any object* — which is what the envelope says — is
  inexpressible: an object with no properties and `additionalProperties: false`
  admits only `{}`.

So **a provider cannot be held to the envelope alone.** The least it accepts is
the envelope *and* each door's own two fields, which is
`alo_models::in_the_envelope::the_envelope_a_provider_accepts`. Two renderings
of one piece of knowledge, which this repository normally refuses — the reason
there are two is above, and a test holds them to naming the same three doors and
the same protocol version.

**And it buys something smaller than it looks.** The verb's name and the
argument names are still the model's to get wrong. Asked under it to list a
folder, `gpt-4o-mini` answered
`{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"/home/anna/Invoices","is":"string"}]}}}`
— the path in `named`, and the schema's own word `string` in `is`. A correct
shape carrying nonsense, which is the placeholder-copying
`data/catalogue.toml` already records for local models.

That is still a better refusal to be given: `ArgumentRefused` names the
argument, and an agent can act on it, where an unreadable message ends a turn.

## The decision this proposes

1. **The provider arm reads `held`**, so an agent turn's question to a provider
   is held to `the_envelope_a_provider_accepts` and a person's question in prose
   is not. ADR 0032's decisions 1, 2, 3 and 5 are untouched; **decision 4 gains
   a second permitted place and keeps its reason**, which was never *only the
   pinned runtime is trustworthy* but *only the pinned runtime can be held*.
2. **Nothing falls back to asking unheld.** A service that refuses
   `response_format` answers 400 and the person is told *something answered, and
   not with an answer*. A constraint that quietly stops applying is one nobody
   can rely on, and it would fail exactly when a model is having trouble, which
   is when it is wanted.
3. **The paired-machine arm still does not read `held`**, unchanged: a paired
   machine is asked the question its own turn wrote.

## What it does not propose

**Enumerating the verbs and their arguments.** That would close the content
error too, and the knowledge already exists —
`alo_driving::grammar_for` builds exactly it in GBNF for the local road. It is
the next thing and not this one, because it is a larger schema, it has not been
measured, and a schema naming every verb is a schema that changes whenever the
verb list does.

## What it costs if it is wrong

A provider that accepts `response_format` and handles it badly would answer
worse than it does today, and the first sign would be a grade falling rather
than anything breaking — which the fixed ten measures on demand. Reverting is
one `bool` at one call site.
