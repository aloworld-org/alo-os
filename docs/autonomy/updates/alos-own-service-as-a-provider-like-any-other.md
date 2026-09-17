# alo's own service, as a provider like any other

**Date:** 2026-09-17
**Workstream:** v0.5 — models a person adapts, and the one they subscribe to
**Task:** *alo's own service, as a provider like any other*
(`docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`, task 5)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
tested and linted in the Lima VM (Ubuntu 24.04 aarch64).
**Egress:** none. **Nothing was sent to alo's service**, which is not built and
is not claimed to run: every test runs against the types every provider uses.
**Status:** done.

## What changed, for somebody outside this repository

alo will sell access to models it hosts. This is the code that makes sure
**buying it changes nothing about how this machine treats it.** The service is a
provider entry — an address, a region it declares, the chain it discloses — read
through the same types as a provider somebody typed in themselves.

**And a test now fails if any other crate learns that one provider is ours.**

## The conflict, and why these are tests

ADR 0014 says it plainly: *if alo sells inference, alo profits when questions
leave the machine.* It lists the pressures — make ours the default, make it look
safer, let the indicator treat it gently, let a failing local model become a paid
call, remind somebody about a subscription when the agent cannot answer.

Those pressures do not arrive as an argument. They arrive years later, with
somebody who means well, in a change that looks small. So each rule is a test,
and every one of them is written as `assert_eq!(ours, theirs)` rather than
`assert!(ours is fine)`:

| Clause of ADR 0014 | How it is held |
|---|---|
| no exemption from the indicator | ours reports the same `InferenceSource` variant as another provider's, and is not *this machine* |
| a policy refusing hosted inference refuses ours | five policies, ours and Mistral's compared: the same permission and the same refusal for every one |
| refused in the same words | the refusal variants match by discriminant, so a gentler sentence for ours cannot be written |
| no default, no pre-selection, no variant | no `DEFAULT`, `RECOMMENDED`, `PREFERRED` or `impl Default` anywhere in the crate |
| **no special case anywhere in the code** | every crate's shipped source is read for six ways of naming us, and none may |
| running out is reported once and falls back nowhere | the sentence for running out with us, with the names swapped, is character-for-character another provider's |
| cancelling leaves a machine that works | the entry holds no balance, credit, plan, subscription, expiry or tier — there is nothing to cancel into |

A future change that makes ours quieter now has to make every provider quieter,
which is a change somebody would notice in review.

## What a person reads when the money runs out

The test swaps the names and compares:

> *nothing was answered by **alo**, in the EU — the account there has run out, so
> nothing will be answered until it is paid for, and nothing else about this
> machine has changed*

That is the sentence `alo-answering` already writes for any hosted provider. It
is asserted here to contain *nothing else about this machine has changed*, and
asserted **not** to contain *subscribe* or *upgrade*: a refusal from our own
service may not sell anything. What may happen next is whatever ADR 0008 allows
for any failure — an offer a person answers, never a silent fallback.

## What the machine knows about the account: nothing

An address, a key in the keyring (ADR 0022), a region it declares, and whether
the last request was accepted. A test reads the entry for *balance*, *credit*,
*plan*, *subscription*, *expires* and *tier* and fails on any of them. The
account and the billing are outside this repository, and the machine is built so
that a person who stops paying has a machine that answers on its own hardware
exactly as it did before.

## The guard other lanes will meet

`tests/no_code_anywhere_knows_it_is_ours.rs` reads `crates/*/src` — every crate
but this one — and fails on `api.alo.computer`, `alo_hosted`, `alo-hosted`,
*our own service*, *our service* and *alo's service*, in code **and in comments**:
a comment saying *ours is faster* is a default waiting to be written.

**If it stops you:** the fix is never a better name. If alo's service needs that
code path, so does Mistral's; if it does not, neither do we. The behaviour is
the same for every provider, or it is a special case.

## What is not here

- **The service itself.** Not built in this repository, not claimed to run. Its
  address is data; nothing was sent to it, and no test needs it to exist.
- **A client.** A test reads this crate's own source and fails on `ureq`,
  `reqwest`, a socket or an `ask` function: alo's service is spoken to by the
  code that speaks to every provider.
- **The region, verified.** `Region::Declared` means *as the service states it*,
  for ours exactly as for anybody's. This machine cannot verify where a service
  runs, and a region we checked differently would be the special case.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched. This crate
does not build on macOS — `alo-saying` reaches a Linux-only crate — so its tests
and clippy were run in the VM.
