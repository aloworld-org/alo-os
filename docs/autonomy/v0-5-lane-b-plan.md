# v0.5, lane B — providers and models, before there is anywhere to show them

**Workstream:** the second lane's v0.5 partition — `crates/alo-models`,
`crates/alo-choosing`, `crates/alo-asking`, `crates/alo-secrets`,
`crates/alo-telling`, and nothing in `crates/alo-shell` or `image/`.
**Why it exists now:** [ADR 0028](../decisions/0028-screenless-v0-5-work-begins-while-v0-01-waits-on-hardware.md).
v0.01's remaining items wait on a machine and on the desktop lane; these four
promises need neither. Every one is a rule a person is protected by, and every
one is decidable without a pixel.

**How the loop reads this file:** the tasks under `## Tasks`, numbered from one
in order, each with a `**Status:**` line. A finished task is marked
`**Done, <date>.**` at the start of a line of its own, in the change that
finishes it, and the next one is written there if none follows. The loop also
reads the mark directly after the `**Status:**` label, because that is where
this paragraph once led a worker to put it (`tools/kernel-loop/src/plan.rs`).

**What this plan may not do:** move any v0.01 box, line or wording (ADR 0028's
first term); tick anything *on the machine*; edit a crate another lane owns.
Before writing the next task, `git pull` and read the plan as published —
numbers are a shared space.

## Tasks

### 1. An address that is not https is refused, unless it is a service on this machine

**Status:** done. **Depends on:** nothing.

**Done, 2026-09-12.** Report:
[`updates/an-address-that-is-not-https-is-refused-at-the-write.md`](updates/an-address-that-is-not-https-is-refused-at-the-write.md).
The mark first sat after the `**Status:**` label, where the loop did not read
it and selected this task again after publishing it; the loop now reads it
there too, and the follow-up is
[`updates/a-done-mark-on-the-status-line-is-read.md`](updates/a-done-mark-on-the-status-line-is-read.md).

`docs/features.md`, v0.5: *an address that is not https is refused rather than
warned about, unless it is a service on this machine — "it is only our internal
network" is how a key ends up on the wire in clear.* Today a provider's address
is validated for shape and not for scheme, so a person can add `http://` and
their key travels in clear the first time they ask a question.

- **Acceptance:** adding or changing a provider whose address is not `https`
  is refused before anything is written — the settings file is byte for byte
  what it was — and the refusal says why in words `alo-saying` collects; a
  loopback address (`127.0.0.1`, `::1`, `localhost`) on any scheme is the one
  exception and is accepted, because a service on this machine is ADR 0021's
  own case; an address on the machine's own LAN is **not** an exception — *it
  is only our internal network* is the sentence the promise names as the
  failure; and the rule is applied in the one place a provider is written
  (`alo-choosing`'s own shapes), so no second path can bypass it.
- **Constraint:** no allow-list of hostnames and no environment variable that
  turns it off. It changes nothing about how a provider that is already saved is
  read — a file written before this rule is read as it always was, and the
  refusal happens at the next write rather than by editing the person's file
  behind them.

### 2. Test a provider before saving it

**Status:** ready. **Depends on:** 1.

*Test a provider before saving it, so a mistyped key is found now rather than
in the middle of a question.* The test is one request to the provider with the
key the person typed, made before the key goes to the keyring and before the
address goes to the file.

- **Acceptance:** one request is made, to the cheapest thing the provider's API
  answers (a model list, or whatever `alo-asking` already knows the provider
  offers), with the key — and the outcome is one of exactly three values: it
  answered, it refused the key, it could not be reached; each is a sentence
  `alo-saying` collects; the key is never logged, never in the sentence, never
  in an error, held by a test that reads the crate the way
  `nothing_here_keeps_the_password` reads `alo-greeting`; a refusal or an
  unreachable provider writes nothing, and a person may still save it anyway if
  they say so — a provider that is down today is not a wrong provider; and the
  request goes through `alo-egress`'s own accounting, because a test request is
  an egress somebody asked for and the indicator says so.
- **Constraint:** it never guesses an endpoint for a provider `alo-asking` does
  not know; for one it does not know, the honest answer is *this cannot be
  tested from here* and the save proceeds as it does today. No retry loop, no
  timeout longer than a person would wait at a dialog.

### 3. A provider that will not say where it runs is unknown, and unknown never satisfies a policy

**Status:** ready. **Depends on:** nothing.

*A provider that will not say where it runs is reported as **unknown**, never
assumed to be nearby — and unknown never satisfies a policy naming a region.*
`alo-models` already distinguishes where inference happens (ADR 0021,
`InferenceSource`), and an organisation's description may bound the region
(ADR 0016). What is missing is the value in between: a provider whose region
nobody stated.

- **Acceptance:** a provider saved with no region is `Unknown`, a value of its
  own and never a default region; the bound in the machine description that
  names a region refuses `Unknown` with a sentence that says the provider did
  not say where it runs — not that it runs elsewhere; a personal machine with no
  bound is not affected, because unknown is honest rather than forbidden; the
  egress indicator's wording carries *unknown* for such a source rather than a
  guessed place; and the refusal is tested beside the acceptance, in
  `alo-models` and in the one place `alo-agentd` reads the bound.
- **Constraint:** nothing infers a region from an address, a TLD or a company's
  name — that is the assumption the promise forbids. A provider that *does* say
  where it runs is taken at its word and labelled as its own claim, which is
  ADR 0021's existing rule.

### 4. Run a model we never catalogued — the catalogue recommends, it does not gate

**Status:** ready. **Depends on:** nothing.

Three sentences from `docs/features.md`, one task, because they are one rule
seen from three sides: *point alo OS at weights you already have and it runs
them; the catalogue recommends, it does not gate*; *a model too large for the
memory in this laptop is said so plainly, once — and then run anyway*; *what
you bring is yours, including its licence, and alo OS does not pretend to have
checked it.*

- **Acceptance:** a person can name a weights file on this machine as a model
  source, through `alo-choosing`'s own shapes; it is offered by `alo-models` as
  a model with no catalogue entry, whose licence is *yours* rather than a
  catalogue claim, and whose size is measured off the file rather than stated;
  a model whose measured size exceeds the machine's memory is told to the person
  **once** — through `alo-telling`, which already says a thing once — and is
  then run if they still choose it; nothing about the catalogue's own entries
  changes; and every sentence is in the vocabulary `alo-saying` collects, with a
  test that none of them nudges toward a catalogued model or away from the
  person's own.
- **Constraint:** the catalogue's grades and licences are for catalogued
  entries and are never guessed for a file — `drives_verbs` for a brought model
  is *not measured* and the sentence says so. Nothing here downloads anything.
  Nothing here changes what ships on the image (that is lane A's, and ADR 0025's
  reading of it stands).
