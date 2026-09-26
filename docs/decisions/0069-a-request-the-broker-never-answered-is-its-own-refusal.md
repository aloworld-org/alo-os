# ADR 0069 — A request the broker never answered is its own refusal, not a network that misbehaved

**Status:** proposed, 2026-09-26.

Written by the Mac lane from a test that fails about one time in two under a full
workspace run. The test is not the finding. Chasing it found that **two different
facts are told to a person in one sentence**, and one of them sends her to look at
a network that was never the problem. What a person is told is not a lane's call,
so this asks rather than changes.

## The decision in one line

`NotChanged::CouldNotFinish` claims *the change was accepted, and the machine
could not finish it*. A broker that never answered establishes **neither half** of
that, so it needs a refusal of its own — and that refusal should point a person at
the machine's own record, which knows, rather than at her network, which does not.

## The three states, and the two names they have

`alo_broker::ask` can end three ways, and `alo-changing-network` gives them two
names:

| What happened | What `ask` says | What the person is told |
|---|---|---|
| Nothing was reached | `NotAsked::NoDoor` | `NothingMakesChanges` — *nothing was there to make the change* |
| The broker answered, refusing | an `Answer` | `CouldNotFinish` and its kin, by the broker's own reason |
| **The request was written and no answer came** | `NotAsked::NoAnswer` | **`CouldNotFinish`** |

The first is right and the second is right. The third is the problem.

## What the third state actually is

`alo_broker::asking::NotAsked::NoAnswer`, in its own words:

> The request was sent, and what came back was not an answer — or nothing came
> back in time. **Whether the verb was carried out is not known**: the broker
> writes a request down before carrying it out, so its record says.

So the machine does not know whether the change was accepted, whether it was
carried out, or whether the request was ever read. And it says so — it even says
where the answer is.

## What the person reads instead

`changing-network.refused.could-not-finish`:

> This machine could not finish the change to its network, **for example because a
> password was not accepted**. Open the networks in Settings to see how they are
> now

with a note to whoever translates it:

> Said when an approved change to the network was started and the machine could
> not complete it: a network went out of range, **its password was declined or not
> accepted**, or Wi-Fi is switched off by a key on the machine.

Every cause that sentence offers is about the network. None of them is *the part
of this machine that makes changes did not answer*. A person whose broker was
gone, overloaded or whose door was abandoned is told her **password may have been
declined**, and goes to retype a password that was never wrong. On a managed
machine she may go to her administrator with it.

`CouldNotFinish`'s own rustdoc says *the change was accepted*. `NoAnswer` cannot
establish that the request was even read.

## Why the obvious fix is the wrong one

The test that found this asserts the **stronger** promise — that killing the
broker produces *nothing was reached* — and the tempting repair is to let it
accept either refusal. That would be wrong twice over.

It would write down that a machine may say *your password was not accepted* when
nothing was ever asked. And it would throw away the distinction the two refusals
exist to draw: `NothingMakesChanges` is a promise that **nothing happened**, and
`CouldNotFinish` is a warning that **something may have**. A person acts
differently on those. Collapsing them to whichever the machine happened to
produce is how a guarantee becomes a coin toss.

So the test was right to demand the strong promise, and it is the machine's two
names for three states that need the third one.

## The options

1. **A refusal of its own.** `NotChanged` gains a variant for *the request was
   sent and nothing answered*, with a sentence that says the machine does not
   know whether the change was made and names where it is written down. Costs a
   new key and a translation; `NotChanged` is not `#[non_exhaustive]`, so it
   costs exhaustive matchers a new arm.
2. **Consult the record before answering.** `NoAnswer` already says the broker
   writes a request down before carrying it out. The caller could read that
   record and answer *it was carried out* or *it was never read* instead of
   guessing. Strictly better for the person and strictly more work: it makes the
   record a thing the change path depends on being readable.
3. **Reword `CouldNotFinish` to cover both.** Cheapest, and it makes one sentence
   vaguer for every case in order to stop it lying in one. Under
   [ADR 0068](0068-a-published-sentence-changes-by-getting-a-new-key.md) a
   meaning change needs a new key anyway, so this is not as cheap as it looks.

This lane's reading favours **1 now and 2 later**: the split is what stops the
machine saying something untrue, and reading the record is a better answer that
can be built behind the same refusal without changing what a person is told
again.

## What is not being proposed

Nothing here weakens the test, and nothing here touches
`--enforce-container-sigpolicy` or any other refusal that is correct behaviour.
The flake is recorded in `docs/quirks.md` and stays open; it is a symptom, and
whichever option is chosen is what closes it.

## What this lane measured

- The two refusals and their sentences, read from
  `crates/alo-changing-network/src/{carrying_out,refusing,words}.rs` and
  `crates/alo-broker/src/asking.rs`.
- Two failures, both under `cargo test --workspace`, on trees that passed the
  same gate on either side of them.
- **Not reproducible** with that target alone: 30 runs at eight threads, all
  green; 30 more under twelve busy loops on six CPUs, all green. It needs the
  whole suite's file and socket pressure, which is why nobody has caught it at a
  desk.
- `alo_broker::Listening` has **no `Drop`**, so a door file outlives its broker.
  That is benign for the refusal — a connect to an abandoned socket file gives
  `ECONNREFUSED`, which is `NoDoor` exactly as a missing file is — and is noted
  because it was the first thing suspected and is worth ruling out in writing.

**What this lane could not establish** is the mechanism: after the broker thread
is joined its listening descriptor is closed, so a connect should fail cleanly,
and `NoAnswer` requires a connect that succeeded or a peer that answered.
Neither is accounted for. The finding above does not depend on the mechanism —
the sentence is wrong for `NoAnswer` however `NoAnswer` arises — but the flake
will not close until somebody who owns that crate finds it.
