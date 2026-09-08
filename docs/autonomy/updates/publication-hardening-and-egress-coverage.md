# Publication hardening, and the egress coverage audit

- Date: 2026-09-08
- Workstream: kernel enforcement (`tools/kernel-loop`, `alo-bounding`)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: Hardening publication, and the egress coverage audit
- Status: done — both halves, with regression tests and reproductions against
  the real loaded programme
- Follows: `end-to-end-network-enforcement.md`,
  [ADR 0020](../../decisions/0020-a-question-is-carried-out-inside-the-turns-boundary.md)

## Part one — publication fails closed

### What was actually wrong

Not the supervisor. `tools/kernel-loop` already stopped at the first failed
gate, staged nothing before gating, and left every failure's work in the tree.
What was wrong was the path **beside** it.

On 2026-09-07 this workstream gated a combined tree and pushed in one shell
line — the gate run and the `git push` joined by a newline rather than by a
check of the first one's result. The gates had failed: the machine had lost its
BPF filesystem between WSL sessions, so every test that loads the boundary
panicked about a missing directory. The push went out over a red run. The change
was sound and was re-verified afterwards, but *the sequence is what a supervisor
is for*, and a supervisor with a hand-rolled path beside it is a supervisor with
an exception.

So the fix is not a check added to the shell line. **There is no hand-rolled
publication path any more.**

### What changed

**`publish` and `verify` are subcommands.** A person finishing a task by hand,
or picking up after a loop that stopped, runs `alo-kernel-loop publish` — which
walks the same steps in the same order as the loop, because it is the same code:
readiness, every gate, the task's acceptance evidence, stage, commit, integrate
what arrived, gate the combination, push. `verify` runs the checks and publishes
nothing, and **refuses when no handoff is waiting** rather than reporting green
on the gates alone: the gates are the state of the repository, and an `ok` that
meant only that is the answer this program exists not to give.

**Readiness grew from one check to three**, each with a sentence somebody can
act on: the BPF filesystem at `/sys/fs/bpf` (the one this machine actually
loses), a toolchain the bridge can reach, and the pinned nightly the BPF target
needs. Each fails before the slow gates rather than in the middle of them. It
**reports rather than fixing** — a supervisor that mounted filesystems would be
changing shared state on a machine another worker is using.

**Acceptance evidence is read from cargo's result line, not its exit code.** A
test name that matches nothing prints

```text
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
```

and **exits zero**. An exit-code check waves that through, and so does the
whole-workspace run, which cannot tell a test that does not exist from one it
never had to look for. So the result line is parsed strictly: exactly one of
them, it must say `ok.`, and it must be one passed and none failed.

**A worker that exits unsuccessfully did not finish the task**, whatever it left
behind. The loop still never reads what a worker *said*; it reads the one thing
the operating system will state about a process and takes it the safe way round.
A handoff beside a failed exit is a contradiction that must not be resolved in
favour of publishing. The cost of being wrong in this direction is a task nobody
published, which somebody notices; the cost of the other is half a task on
`main`.

**A refused push with an unmoved remote stops at once.** A push that loses to
somebody else's is a race — integrate, re-gate, retry. A push refused while
`origin/main` has not moved is a real push error, and retrying it would be a
supervisor waiting for a network to heal while holding a lock.
`docs/autonomy/SHARED_MAIN.md` already said that; now the code does too.

### The regression tests, and what each one proves

`tools/kernel-loop`, **21 tests, all passing** (Windows; this crate is the
supervisor and runs there). Publication is a sequence, so the tests are about
**what did not happen**: the double records every step it was asked for, and the
assertions are on that list.

| Cannot publish | Test | What it asserts |
|---|---|---|
| A failed gate | `a_failed_gate_stages_nothing_and_pushes_nothing` | the recorded steps are exactly `["check this task's tree"]` — nothing staged, nothing committed, nothing pushed |
| A failed combined-tree check | `a_failed_combined_tree_check_leaves_the_commit_unpublished` | check, stage, commit, advanced, rebase, check — and stop. The commit stays local |
| A conflicted rebase | `a_conflicted_rebase_publishes_nothing` | no push, and the combination is not gated over a rebase that never finished |
| A push error that is not a race | `a_push_error_that_is_not_a_race_stops_at_once` | exactly one push attempt, and the sentence says the commit is intact |
| Zero matching acceptance tests | `a_name_that_matches_nothing_is_not_evidence` | cargo's zero-passed-and-exit-zero output is refused; the real one-passed output is accepted |
| A failed or never-run test | `a_test_that_failed_or_never_ran_is_not_evidence` | a `FAILED` result, a run that printed no result at all, two result lines, and a passing line beside a non-zero exit — all refused |
| Evidence that was already green | `evidence_has_to_live_in_a_file_the_task_is_publishing`, `evidence_outside_the_change_is_refused_before_anything_runs` | a test whose file is not in the change is refused before anything is run |
| A task showing nothing | `a_task_that_shows_nothing_is_not_published` | an empty evidence block is refused outright |
| A blocked or partial worker | `a_worker_that_did_not_finish_is_not_a_completed_task`, `a_partial_handoff_publishes_nothing` | a non-zero exit or a signal is not a finished task; a handoff missing any of task, report, subject, evidence or files is refused, one piece at a time |
| Vague evidence | `evidence_that_does_not_name_one_test_is_refused` | a line that is not three words is refused rather than guessed at |
| A gate result | `a_gate_that_failed_stops_and_says_which` | the refusal names the gate and carries the tail of what it printed |

And the two that keep the suite honest about the other direction:
`a_task_that_passes_everything_is_published_once` and
`a_lost_race_is_re_gated_before_the_second_attempt` — a real race integrates and
gates the combination *before* the second push rather than pushing again at
whatever is in the tree.

**Nothing is discarded on any of these roads.** There is no `reset`, no
`--force`, no `clean` and no `--abort` anywhere in the program;
`tools/kernel-loop/src/repository.rs` documents that absence as its subject. A
failed gate leaves the tree untouched, a conflicted rebase is left where it
stopped, and a refused push leaves the commit local. No other worker's BPF pins
are removed by anything here: the loop reports an unmounted filesystem, it does
not mount one, and every test in this workstream pins under a path named after
its own process id.

**This report's own publication went through `alo-kernel-loop publish`**, which
is the best evidence available that the manual path works.

## Part two — the egress coverage audit

Three gaps were named in reports and in `deciding.rs` and demonstrated nowhere.
They are now reproduced against **the real loaded programme on a running
kernel**, in `crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs`.

Each test drives a real turn: a control group, a bound naming one folder and
**no destination at all**, and a child process inside it. Each begins with a
**control** — a TCP connection to this machine's own `eth0` address, which the
boundary refuses with `EACCES` (13) — because a turn that reached something
proves nothing if the boundary was never applied; an unbounded process reaches
everything too. Every subject result is thrown away unless that refusal
happened. Each is asserted in the direction it behaves **today**, so the day one
is closed its own assertion fails and names the documents to change.

| Reproduced | Test | What happens |
|---|---|---|
| A socket already open or inherited | `a_connection_made_before_the_boundary_stays_usable_inside_it` | the child connects, *then* joins the control group; the bound is written after that. It is refused the control and writes freely on the connection it brought in, and the far server confirms the bytes arrived |
| A datagram sent without connecting | `an_unconnected_datagram_leaves_a_bound_turn_unchecked` | `sendto` on an unconnected socket reaches no `connect` hook; the datagram arrives at an address nobody showed the turn |
| A proxy on loopback | `a_proxy_on_loopback_carries_a_bound_turn_somewhere_nobody_showed_it` | an eleven-line proxy on `127.0.0.1` forwards to this machine's own `eth0` address. The turn is refused that address directly and reaches it through the proxy in the same breath |

**Nothing reaches a network.** Every address is a listener these tests own, on a
port the operating system chose, on loopback or on this machine's own interface;
the "elsewhere" the proxy forwards to is this same machine. Nothing resolves a
name and no host-wide networking is touched.

### Production-reachable v0.01 gaps, and later-release work

The distinction that matters, made by asking what a person or an agent can
actually cause **today**, not by what is theoretically possible.

**Production-reachable now — one, and it is v0.01-relevant:**

- **The loopback proxy.** A provider endpoint pointed at a local proxy is an
  ordinary thing to configure, and `alo_models::address` treats loopback at face
  value on purpose (ADR 0007 makes a model on this machine the default). No code
  has to be written by anybody: the person configures an endpoint, and every
  type in this repository believes the question stayed home. The indicator shows
  a quiet day. It bears on the v0.01 *Egress indicator* line because that line's
  promise is about what the person is shown.
  **What keeps it small:** law 2 — an agent cannot start the proxy. It takes the
  person's own action, or a second process they already trust.

**Not production-reachable, and v0.5 — the same class as the already-open
descriptor gap (plan task 6):**

- **A socket already open or inherited.** A turn's verbs are file verbs, and
  none of them hands a model a descriptor or a socket. What has the hole is the
  floor under a verb with a bug in it, not a path an agent can walk. The daemon's
  own record and sockets exist inside the process a turn runs a thread of, and
  nothing shipped exposes them.
- **A datagram sent without connecting.** Nothing alo OS ships sends one from
  inside a turn: a question is TCP, and resolution happens outside the boundary
  by ADR 0020, which is the whole point of resolving before entering. There is no
  code path an agent can reach that calls `sendto`.

**Already closed, and worth recording so it is not re-audited:** connection
pooling. The `Agent` is built per request and dropped with it (ADR 0020 §3), so
no connection survives to be reused past its permission. Redirects were already
prevented by `max_redirects(0)`.

### Two decisions this workstream is not taking on its own

Both would change accepted security semantics, and **the network-request
approval does not cover either.** They are put here as questions rather than
started as work.

**1. May anything watch a socket after it is opened?**

Closing the inherited-socket gap means a hook that fires on use rather than on
creation — `socket_sendmsg`, or a cgroup/skb programme. That is a different
claim about what a security module does: `file_open` and `socket_connect` both
decide once and say nothing afterwards, and *the LSM decides and forgets*
(ADR 0015) is the rule that keeps the programme's map list at exactly two. A
per-message hook does not by itself break that rule — deciding is not
remembering — but it changes the cost model of every send on the machine and it
is the first hook here that would see traffic rather than intent.

It would also close the unconnected-datagram gap, which is why the two are one
question rather than two.

**What would need deciding:** whether the boundary may sit on the send path at
all, and if so whether it decides per message or only for turns. Neither is a
thing to start without an answer.

**2. May egress enforcement stop being turn-scoped?**

The loopback proxy is not closable inside a turn's boundary, and the reason is
structural rather than an oversight: the proxy is not a turn, so its own outward
connection is not the boundary's business, and the turn's own connection really
does stay on the machine. Closing it needs enforcement over **everything this
machine sends** — a different blast radius, a different failure mode when it is
wrong, and a policy question about a person's own processes that this workstream
has no mandate to answer.

`docs/quirks.md` has carried this as an open hole since 2026-09-03, and its
forward reference to "egress enforcement at the network boundary" was already
corrected once. It is now reproduced there rather than only reasoned about.

**What would need deciding:** whether alo OS filters what the machine sends, or
whether it states plainly that a proxy the person starts is trusted because the
person started it. The second is a defensible answer and is cheaper; it is still
an answer somebody has to give.

## What is preserved, and checked

Everything ADR 0020 established is unchanged, and the suite says so rather than
this report:

- **Request-scoped destination registration** — `alo-turn` resolves outside the
  boundary, registers, enters, and the registration goes when the request does.
- **Original-hostname TLS verification** — only the resolver is replaced; URL,
  connector and certificate verification untouched.
- **Local-model operation** — loopback still unchecked, `Served` unchanged.
- **The indicator and the records** — untouched; a refused question still shows
  nothing and records nothing left.
- **Connection cleanup** — the client is still built per request and dropped
  with it.

`crates/alo-agentd/tests/a_question_is_bounded_by_the_kernel.rs` — all five
still pass, including the production request that answers and the one that is
refused an unregistered address.

## Verification

Ubuntu on WSL2, kernel 6.18.33.2, Rust 1.98.0, own `CARGO_TARGET_DIR`, readiness
checked first: `cargo fmt --all --check`; `cargo clippy --workspace
--all-targets -- -D warnings`; `cargo test --workspace`; `RUSTDOCFLAGS="-D
warnings" cargo doc --workspace --no-deps`; and the BPF target's `fmt --check`
and `clippy --release --target bpfel-unknown-none -Z build-std=core -- -D
warnings` on the pinned `nightly-2026-06-01`.

**Windows:** `tools/kernel-loop` is a Windows program — it runs where the git
credentials are — and its 21 tests, `cargo fmt --check`, `cargo clippy
--all-targets -D warnings` and `cargo doc` all pass there. The kernel crates do
not build on Windows and no Windows run is offered as enforcement evidence.

**WSL is development evidence and never certified-hardware acceptance.** No
*On the machine* box is affected by anything in this report.

**No claim of complete network enforcement.** Three gaps are open, reproduced
and placed above; two of them wait on decisions nobody has taken.

## Proposed shared-document updates

Not made here — the integration worker consolidates them.

**CHANGELOG.md** — nothing user-visible. Both halves are internal: a
supervisor's checks, and tests that document limits.

**ROADMAP.md** — the *Egress indicator* line's remaining clause stays. The
loopback proxy is production-reachable and open, and the line also names the
compositor surface, the daemon code that signs somebody in and fetches a model,
and physical acceptance.

**docs/autonomy/QUEUE.md** — no new item; this workstream's plan is its list.
The two decisions above belong to whoever schedules decisions, not to a queue.

**docs/autonomy/STATE.md** — two facts worth carrying. Publication in this
workstream now fails closed on every path including the manual one, with
regression tests naming each refusal. And the egress coverage is reproduced
rather than asserted: one production-reachable v0.01 gap (the loopback proxy)
and two v0.5 ones (inherited sockets, unconnected datagrams), with two
architectural decisions requested and neither assumed.
