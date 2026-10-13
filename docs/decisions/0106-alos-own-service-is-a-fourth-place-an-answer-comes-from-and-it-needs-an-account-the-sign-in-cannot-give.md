# ADR 0106 — alo's own service is a fourth place an answer comes from, and it needs an account the sign-in cannot give

**Status:** **proposed, 2026-10-13**, by the Mac lane, for the owner. One
question in it is not a lane's to answer, so this asks rather than changes.

## What the owner decided, and what it ran into

On 2026-10-13 the owner said what alo OS should offer a person who wants an
agent:

> it will be decided by the user to choose either local, apis or alo os emerged
> ones, so the users who will download the os will be able to use 3 and we shall
> the users for the alo os services so if the user choose alo os services he
> will not need to add local models or apis but we shall charge him

Three ways, then: a model on the person's own machine, a provider they hold a
key for, or **alo's own service**, which needs nothing set up and which they pay
for. Billing is later; the mechanism is now.

Asked how the service would know who the person is, the owner answered that
`desktop`'s lane is building the sign-in and people will sign in.

**That sign-in cannot carry it, and making it do so would break the first
promise this product makes.** `alo-greeting`'s first screen says, in the words
it ships with:

> Nobody has an account on this machine yet. Make one to sign in — **the account
> you make here is this machine's own, and it is not sent anywhere**

and the note beside that string says why the sentence is not simply *make an
account*: *the machine is sold on nothing leaving it, and this is the first
moment a person is asked to type*. [ADR
0024](0024-what-a-person-signs-in-at.md) makes
`alo-accounts` the authenticator **for this machine**. What the door answers is
one word and one number; what the screen becomes is a local session for a uid.
There is nowhere in it for a token, which is a property of the types rather than
a rule somebody keeps.

So the owner's two sentences are each sound and they do not meet. *This* sign-in
is a local one that is promised never to leave. A paid service needs an identity
it can recognise, which is **a second, online account** — a new concept, not an
extension of the existing one.

## The decision this proposes

1. **alo's own service is a `Hosted` provider named `alo`, and needs no new
   place for an answer to come from.** [ADR
   0008](0008-where-inference-happens.md)'s
   `InferenceSource::Hosted` already documents its provider field as *the
   provider, as a person would say it — "alo", not a hostname*. The egress
   indicator, the record, the region a person is told, and every policy an
   organisation can set therefore apply to it **unchanged**, because it is not a
   special case in the type.
2. **It is the loudest case and not the quietest.** A question to alo's own
   service lights the indicator and lands in the record identically to any other
   provider, and there is a **test that says so by name** — because this is the
   one provider where a person has most reason to wonder whether we gave
   ourselves an exception. Proposed by `desktop`'s lane in exactly those terms,
   and adopted here.
3. **It is a fifth choice at first start, and it creates nothing there.**
   `alo_setting_up::Offered` has four today, and the note on their order says
   the local one is first *because it is the one that needs nothing added — no
   key, no pairing, no account*. The alo-service choice needs an account, so it
   does not slot in silently: choosing it at first start **records the choice
   and creates no account**, and the account is made later, from Settings, by a
   person who has used the machine.

   [ADR 0004](0004-the-organisations-machine.md)'s *no
   silent enrollment* is the reason, and it cuts harder here than it looks: an
   account for a paid service made during first start is enrollment at the
   moment a person is least able to judge it. `desktop`'s lane pushed on this
   and is right.
4. **The local account and the service account are never the same account, and
   the words never suggest they are.** No sentence a person reads may imply that
   signing in to the machine signs them in to anything, and the first-start
   promise above stays exactly as it is.

## The question this does not answer

**What the online account is, and who builds it.** The destination is settled —
a token in the Secret Service keyring with a `SecretRef` in the person's
`settings.toml`, which `alo_agentd::questions::Questions::a_key_from` already
looks up and which `crates/alo-agentd/tests/the_agent_asks_a_real_provider.rs`
proves end to end. **The source does not exist.** Nothing in alo OS creates an
online identity, authenticates against one, or renews a token, and nothing in
the sign-in path writes to that reference.

That is a product decision with a service behind it rather than a lane's design
choice, so it is the owner's:

- is it an account made on alo's own service from Settings, with the token
  written to the keyring — or something else entirely;
- what happens to a machine whose token has expired or whose payment has
  stopped: proposed here as **the agent's surfaces say the service cannot be
  reached and nothing silently falls back to a local model or another provider**,
  because *never a silent fallback* is one of the two protections law 5 keeps on
  at every setting;
- and whether a person may hold both an alo-service account and their own
  provider key at once, which the settings already permit and nothing forbids.

Until that is answered, **nothing here is built**: a lookup wired to a reference
nobody writes is the shape of half a feature, and `docs/features.md` gains no
line for a choice that cannot yet be made.

## What this costs if it is wrong

Everything in decision 1 is already in the types, so adopting it costs nothing
and reverses cheaply. Decision 3 is the one with a real cost: a person who would
have been happy to make an account during setup is asked to do it later, and
some will not. That is the price of *no silent enrollment*, and the owner may
decide it is too high — in which case the thing to change is this ADR and not
the law.

## Not relitigated

ADR 0008's three places, ADR 0004's no silent enrollment, ADR 0024's scope for
`alo-accounts`, and law 1's indicator. This proposes a provider named `alo` and
a choice that creates nothing; it proposes no exception to any of them.
