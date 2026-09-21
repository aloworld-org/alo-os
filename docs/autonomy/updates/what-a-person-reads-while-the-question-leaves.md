# What a person reads while the question leaves

**Date:** 2026-09-21
**Workstream:** v0.5 — the models, measured, task 21
**Task:** 21, *A hosted provider answers, and the person is told who and where.*
**Done — and most of it was already built**, which is the finding.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB unified memory; built and tested in the Lima VM
on that Mac — Ubuntu 24.04.4 aarch64. Nothing is ticked *on the machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.
The test's HTTP service is a `TcpListener` on `127.0.0.1`; nothing left this
machine.

## The task's premise was stale by eighteen days

The task says the third place ADR 0008 names is *decided, scaffolded and
unreachable*, and that **nothing speaks to one**. That was not true when it was
written on 2026-09-21.

| | |
|---|---|
| `832ac90` | **2026-09-03** — `feat(asking): put a question to a provider, with the egress shown first` |
| what it added | `alo-asking/src/openai.rs`, `src/hosted.rs`, and `tests/from_a_question_to_what_left.rs` — an end-to-end test against a real HTTP service the test starts |
| the turn's wiring | `crates/alo-turn/src/asking.rs:236` calls `to_a_provider` |

So the speaking part has existed for eighteen days, is reached from a person's
turn, and is tested against a real socket. **Building it again would have been a
second answer to *how does a question reach a provider*** — the thing this
repository refuses, and the reason I checked before writing anything.

I record this plainly because the cost of not checking would have been a
duplicate provider client, and because the task was written the same morning I
was given it: the premise was not carelessness, it was a plan entry written from
a reading of the ADR rather than of the crates.

## What I measured before writing anything

Clause by clause, against the acceptance rather than against the summary:

| Acceptance clause | Where it already holds |
|---|---|
| a hosted endpoint can be given, and a question sent there is answered through it | `from_a_question_to_what_left.rs`, real socket |
| the answer arrives by the same road as a local one | same test — the answer is an ordinary `Answer`, and the record is written from the departure |
| the road out is `the_way` for `Road::AskingAProvider` | `Hosted::taking`, which also documents why a road decided by `HTTP_PROXY` in a process's environment is *a road nobody chose and nobody can be shown* |
| a provider that refuses is `RefusedThere` in the person's words | `refused_on_the_wire.rs` maps the wire refusals; anything else is another `WentWrong` refusal, also in the person's words |
| never a silent retry somewhere else | a failure carries **offers a person approves** (`failed.elsewhere().offers()`), not a retry — the rule held structurally rather than by a check |
| the credential never reaches an agent | `a_key_reaches_one_provider_only.rs`, `nothing_here_keeps_the_key.rs` |
| one end-to-end test against a real HTTP service the test starts | `from_a_question_to_what_left.rs` |

One correction to the acceptance's own shorthand: it says a provider that
refuses, times out or answers with nothing *is* `RefusedThere`. In fact
`RefusedThere` is the family for **the provider itself refusing**, and a timeout
or an unreachable endpoint is `TookTooLong` or `NothingAnswered`. That is finer
than the clause asks for, not coarser, and worth stating so nobody later reads
the clause as a specification and flattens them.

## The one thing genuinely missing

**Every test on that road asserts the indicator is quiet once the answer is
home.** That is the second half of law 1 — *afterwards in a record* — and it is
not the half ADR 0008 is about here:

> The indicator fires, and says **who** and **where**: a provider that will not
> say where it runs is reported as unknown rather than assumed to be nearby.

That sentence is on screen **while the question is still out**, and nothing read
it. The words existed and were unit-tested in `alo-egress`; what was untested was
that a real question to a real service puts the right one in front of a person at
the right moment.

`crates/alo-asking/tests/what_a_person_reads_while_the_question_leaves.rs` reads
it, against a real HTTP service on a real socket, in the two states the decision
distinguishes.

## The whole exchange

**A provider that says where it runs.**

```
--- what went out on the socket ---
POST /v1/chat/completions HTTP/1.1
content-length: 111
user-agent: ureq/3.4.0
accept: */*
host: 127.0.0.1:44879
content-type: application/json

{"model":"mistral-small-latest","messages":[{"role":"user","content":"may the tenant sublet?"}],"stream":false}
--- what the person read while it was out ---
@mail is asking a question of Mistral, in the EU
```

**A provider that will not say.**

```
--- what went out on the socket ---
POST /v1/chat/completions HTTP/1.1
content-length: 111
user-agent: ureq/3.4.0
accept: */*
host: 127.0.0.1:39555
content-type: application/json

{"model":"mistral-small-latest","messages":[{"role":"user","content":"may the tenant sublet?"}],"stream":false}
--- what the person read while it was out ---
@mail is asking a question of Aurora, which has not said where it runs
```

The service replies `HTTP/1.1 200` with
`{"choices":[{"message":{"role":"assistant","content":"No, not without written consent."}}]}`
in both, and both tests assert the answer arrives as
`No, not without written consent.` — so the line is about an egress that
happened, not one that was described.

## The assertion that earns it

The pleasant sentence is the first one. The promise is the second, and the test
asserts what it **does not** say as carefully as what it does:

```rust
for nearby in ["127.0.0.1", "localhost", "this machine", "on this network", "the EU"] {
    assert!(!line.contains(nearby), …);
}
```

`127.0.0.1` is where that service really was. A machine that read an address as
a region would put this provider next door, which is precisely the assumption
ADR 0008 forbids — and it would do it at the one moment a person could still
decide not to send the question.

A third test holds the two sentences apart, because a person reads this while
deciding whether to let a contract go: if the two lines looked alike at a
glance, the decision would rest on a difference nobody notices.

## What I did not do

I did not build a provider client, a second road, or a fallback. I did not touch
`alo-turn`, `alo-proxy`, `alo-egress` or `alo-models`. Nothing in this change
can cause a question to be sent anywhere, and a machine nobody pointed at a
provider still has zero inference egress over a working day — the tests here
start their own service on loopback and ask it deliberately.

## Gates

Nine in the Lima VM.

## Crates touched

`crates/alo-asking` — one new integration test — plus this plan and this report.
