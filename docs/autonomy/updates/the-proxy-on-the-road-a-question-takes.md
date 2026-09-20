# The proxy a machine was told about, carried to the question a turn puts

- Date: 2026-09-20
- Workstream: v0.5 software and the web (`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 11)
- Contributor: Claude Code
- Status: **ready for integration.** `cargo fmt --all --check`, `cargo clippy
  --all-targets` with warnings denied, `cargo doc --no-deps` with warnings
  denied, and `cargo test -p alo-asking -p alo-agentd` all ran on the change and
  passed; the exact commands, platform and results are under *Verification*. The
  workspace suite is the supervisor's and was deliberately not run here.

## What a person outside this repository needs to know

On a company network that has no other route out, alo OS now **actually uses**
the proxy its machine was told about when it asks a model a question.

Until this change it did not. An organisation could write the proxy into the
machine's description, the machine read it, a settings panel could show it, and
a person could be refused a change to it in words naming who set it — and then
the question an agent put to a provider went straight past it. On such a network
that means the question did not arrive at all, and the person had a setting in
front of them that was doing nothing. A setting that is read and not honoured is
worse than no setting: it is the one failure the proxy work was written to avoid.

Three things follow, and each of them is a promise rather than an
implementation detail.

**A machine told to use a proxy uses it, and is told where to go by nobody
else.** The road out is decided once, from the machine's own setting, for every
road — installing an application, fetching an update, listing a provider's
models, and now asking one a question.

**A machine that cannot work out its way refuses the question.** If the
description says *ask the network's own configuration script* and this machine
has nothing that can run one, the question is refused in words, and nothing is
sent. It does **not** quietly go straight out. Going straight out would be a
machine carrying a company's work around the company's own rule, with nobody
told.

**The indicator still names the provider.** Law 1 answers *where did my work
go*, and the answer does not change because a proxy was in the middle of the
road. A person on a company network sees the same line as a person at home: the
provider's name. *It went to the proxy* is not what anybody needs to know.

And one thing that is **not** here, stated plainly because a company proxy that
asks for a name is common: **a proxy password is still not read from anywhere.**
The machine's description names the keyring entry a password is kept under, and
nothing in this workspace reads it — not on this road and not on the three that
existed before it. A proxy that wants a name is therefore refused by the proxy,
which is at least a sentence somebody can act on. It is now task 12 of this
plan, with the decision it waits on written out there.

## Two findings, and the second changed what the tests assert

**The provider road was reading `HTTP_PROXY` out of whatever process the daemon
happened to be in.** `ureq::Config::default` — which `Config::builder()` starts
from — fills its proxy in from `ALL_PROXY`, `HTTPS_PROXY` and `HTTP_PROXY`. So
before this change, a question to a provider was not *unproxied*: it took
whichever proxy a service file, a shell or a systemd drop-in happened to have
left in the environment. That is a road nobody chose, nobody can be shown, and
nobody can change where they would look for it — the same objection `alo-secrets`
makes about `DBUS_SESSION_BUS_ADDRESS`, and the reason `alo_models::Trying::taking`
already said its road explicitly. Every request this crate makes now says which
it is, `None` included, on the corridor and the local-service roads as well as
this one.

**A test that watches addresses cannot see any of this.** A road out of alo OS is
*handed* the addresses it may use and resolves nothing (ADR 0020) — so when the
client is given a proxy, it asks that resolver for the proxy's name and gets back
the same addresses that were registered. Watching which address a socket opened
to therefore passes whether or not the proxy was configured at all. What
distinguishes them is the **first line on the wire**: a proxied road writes
`CONNECT api.example.com:443 HTTP/1.1`, and a straight one writes a TLS
handshake. The acceptance reads that line. Both halves of the change were
checked by mutation — removing `.proxy(through)` and removing the proxy branch in
`where_it_would_connect`, each on its own — and each made a test fail that had
passed on the address-only version.

## What changed

| | |
|---|---|
| `crates/alo-agentd/src/the_road_out.rs` (new) | the one place that asks `alo_proxy::the_way` for `Road::AskingAProvider`, and decides nothing itself |
| `crates/alo-agentd/src/the_proxy_on_the_road_a_question_takes.rs` (new) | the acceptance, from `/etc/alo/agentd.toml` text to bytes on a socket |
| `crates/alo-agentd/src/questions.rs` | carries the road out beside `[questions]`'s bound, as one field and one accessor |
| `crates/alo-agentd/src/starting.rs` | hands it over from `Described::proxy` |
| `crates/alo-agentd/src/doing.rs` | asks it after the organisation's rule and before the keyring; hands the answer to `Hosted::taking` |
| `crates/alo-agentd/src/words.rs` | one sentence, for a provider address no road can be decided about |
| `crates/alo-agentd/src/testing.rs` | the listener fixture now reports **what** it was told, not only that somebody came; `doing.rs`'s two private copies moved here |
| `crates/alo-agentd/src/lib.rs` | the two new modules and `TheRoadOut`'s re-export |
| `crates/alo-asking/src/hosted.rs` | `Hosted::taking`, and `where_it_would_connect` naming the proxy on a proxied road |
| `crates/alo-asking/src/openai.rs` | `put_through`, and `.proxy(...)` said on every request |
| `crates/alo-asking/Cargo.toml` | `alo-proxy`, named only in `hosted.rs` |
| `docs/autonomy/a-new-machine-becomes-a-lane.md` | records this plan taking `alo-asking`'s hosted door from a finished plan |
| `docs/autonomy/v0-5-software-and-the-web-plan.md` | task 11 done; task 12 written |

## Decisions taken, and why

**The crate ownership question the task asked about was answered by taking it,
not by writing an ADR.** The constraint said: if carrying a proxy to that road
needs `alo-asking` to change, and that crate turns out to be another plan's, stop
at a decision record. It does need `alo-asking` to change — `Hosted::ask` and
`openai::put` are both `pub(crate)`, so there is no way to configure a provider
request from outside. And the plan that owns `alo-asking`'s hosted and served
doors is `v0-5-the-models-measured-plan.md`, which records itself **Finished,
2026-09-15**, with its lane stopped. `a-new-machine-becomes-a-lane.md`'s *a
machine unblocks itself* (owner, 2026-09-18) says a blocker in a plan that has
finished is taken, and its row edited in the same commit. That is what happened;
the alternative would have been an ADR asking a stopped lane for permission it
has no way to give. `alo-asking/src/corridor.rs` and `src/held_to.rs` are the
local-network plan's and were read, not changed.

**The proxy reaches `alo-asking` as an `alo_proxy::Carried`, not as a
`ureq::Proxy`.** `alo_models::Trying::taking` takes the client's own type, and
copying that here was the obvious move. It is the wrong one for this road:
`where_it_would_connect` has to answer the proxy's host and port, and reading
those back out of a client's parsed value is a round trip through text that the
`ProxyAddress` already had as fields. Taking the `Carried` also keeps
`Carried::as_an_address` — the one function in this workspace that turns a proxy
password into text — as the one function, with `grep` still finding every caller.
The cost is a new dependency edge, `alo-asking` → `alo-proxy`, named in one file.

**`Hosted::where_it_would_connect` answers the proxy's address on a proxied
road.** This is the part that is easy to get wrong and expensive when it is. ADR
0020 says the caller resolves and registers where the request will connect
*before* the boundary is entered, and the boundary then permits those addresses
and nothing else. On a proxied road the socket opens to the proxy. Registering
the provider's addresses instead would bound a turn to an address it never uses
and leave the one it does use unbounded — both halves of ADR 0020 wrong at once,
and invisible on a developer machine where nothing is bounded. `named_source`,
which is what the indicator's line is built from, is untouched and still names
the provider; the two are asserted together in one test so that a later change
cannot make one follow the other.

**The road out is asked after the organisation's rule and before the keyring.**
Same argument twice: a question the bound refuses never reaches for a road, and a
question with no road to go on never reaches for the person's credential. A store
touched on the way to saying no is a store touched for nothing.

**`TheRoadOut` is its own file, and `Questions` holds one field of it.**
`questions.rs` answers *what does a question go to*; *which way does the road
there go* is a second reason to change, so it is a second file. What `Questions`
gains is a field, a builder method and an accessor, which is the same shape it
already has for `[questions]`'s bound — and it is there because both arrive from
one file at startup and are both wanted at the same moment in `doing.rs`.

**Absence is carried through the same door as a setting.** A description with no
`[proxy]` section is *nobody set one*, which `machine_wide_proxy.rs` keeps
distinct from *somebody wrote straight out*. For deciding a road they decide
identically, and `TheRoadOut` asks `alo_proxy::the_way` in both cases rather than
answering the absent case itself — so there is one door, not two.

**The evaluator has no production seam.** `TheRoadOut::of` uses the evaluator at
the path an alo OS image has one at, and there is no parameter or key naming
another; the `cfg(test)` constructor is the `alo_models`/ADR 0019 pattern, for
the reason `Questions::already_found` states. A machine with nothing at that path
refuses every road under an automatic configuration, which is `alo-proxy`'s
behaviour and not a second rule here.

**One new sentence, for one narrow refusal.** `alo_models::Provider` already
refuses an endpoint that is not `http`/`https` and one carrying a credential, so
the only way a checked provider's address fails `alo_proxy::Reaching` is a host
longer than a host can be. That is real, and it is the person's to fix, so it
gets a sentence naming what to do and quoting nothing they typed. The other
refusal on this road is `alo-proxy`'s and is carried in `alo-proxy`'s words.

**The environment half is tested in a second process.** Setting an environment
variable is `unsafe` under Rust 2024 and the workspace forbids `unsafe`, so the
test starts another copy of the test binary with `ALL_PROXY`, `HTTPS_PROXY` and
`HTTP_PROXY` pointing at a listener the parent owns, runs one `#[ignore]`d test
inside it, and then asks that listener whether anybody came. Nobody does. It is
more machinery than a test usually earns; it is the only honest way to assert
the thing that was actually broken.

## What this deliberately does not do

- **It adds no road to `alo_proxy::Road`.** A question down the corridor goes to
  a machine on this network, which is not one of the eight roads out that list
  names, and inventing a ninth is a decision rather than a side effect of this
  task. What that road now does is say *straight out* explicitly instead of
  inheriting a variable, which is strictly better than before and claims nothing
  more.
- **It does not sign in to a proxy.** Task 12, with the decision it waits on.
- **It has not been watched through a real proxy server.** The acceptance stands
  up two listeners of the test's own and reads the first line each is told; no
  Squid, and no company network. What that shows is that this machine addresses
  the proxy *as a proxy* and the provider *as a provider*, which is the part that
  was missing. Carrying a question through a real proxy to a real provider is a
  machine test, and it is named under *Remaining limitations*.

## Verification

Platform: Windows host, gates run on this machine's Linux side (WSL2 Ubuntu),
against the synchronized source copy at `/root/alo-trees/this-machine` with
`CARGO_TARGET_DIR=/root/alo-builds/this-machine`, which is the arrangement
`tools/kernel-loop/src/gates.rs` uses. `alo-agentd` compiles to nothing off
Linux, so none of its tests can run on the Windows side at all.

Executed, all passing:

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean |
| `cargo doc --workspace --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean |
| `cargo test -p alo-asking` | 114 lib tests and 47 across its integration targets and doctests, all passing; 3 ignored, all pre-existing |
| `cargo test -p alo-agentd` | 520 lib tests and 33 across its integration targets, all passing; 34 ignored, of which one is new — the child of the environment test, which its parent runs |
| `cargo test -p alo-citing` | clean — the citation check, which is what a documentation change can break |
| `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` in `tools/kernel-loop` | clean; 151 tests, including the check that reads every plan this repository drives |

Each acceptance test was also run on its own by name, `--exact`, as the handoff
records:

| Acceptance criterion | Test |
|---|---|
| the description's proxy reaches the road a turn's question takes | `the_proxy_on_the_road_a_question_takes::a_question_on_a_machine_told_about_a_proxy_is_put_to_the_proxy_as_a_proxy` |
| a machine told of no proxy goes straight out | `the_proxy_on_the_road_a_question_takes::a_question_on_a_machine_told_of_no_proxy_goes_straight_to_the_provider` |
| said explicitly, so no process environment can point the road anywhere | `the_proxy_on_the_road_a_question_takes::a_proxy_in_the_environment_points_this_machines_question_nowhere` |
| a configuration that cannot be worked out refuses the question | `the_proxy_on_the_road_a_question_takes::a_configuration_this_machine_cannot_work_out_refuses_the_question` |
| the indicator names the provider, never the proxy | `the_proxy_on_the_road_a_question_takes::the_line_the_indicator_shows_names_the_provider_whichever_way_the_road_went` |
| a question answered here is never sent through a proxy | `the_proxy_on_the_road_a_question_takes::a_question_answered_by_a_runtime_on_this_machine_is_never_sent_through_the_proxy` |
| — and from the other end, a provider at this machine's own address | `the_road_out::tests::a_provider_at_this_machines_own_address_is_never_proxied` |
| what is registered with the boundary is where the socket opens (ADR 0020) | `hosted::tests::a_road_through_a_proxy_connects_to_the_proxy_and_is_still_named_for_the_provider` (`alo-asking`) |

Mutation checks, run to show the tests are not passing on the state of the
repository:

| Mutation | What failed |
|---|---|
| `.proxy(through)` removed from `openai::sending` | `a_question_on_a_machine_told_about_a_proxy_is_put_to_the_proxy_as_a_proxy`, `a_proxy_in_the_environment_points_this_machines_question_nowhere` |
| the proxy branch removed from `Hosted::where_it_would_connect` | `a_question_on_a_machine_told_about_a_proxy_is_put_to_the_proxy_as_a_proxy` |

Not run here, deliberately: `cargo test --workspace`. It takes most of an hour
on this machine and is the supervisor's gate; two finished tasks have died
waiting on it.

Not run at all, and not claimed: anything on real hardware, and anything through
a real proxy server.

## Remaining limitations

- **No real proxy server has carried a question.** The acceptance reads the
  `CONNECT` line this machine writes; nothing answers it. A machine test — a
  pinned Squid on a certified machine, a question put through it to a provider,
  and the egress measured at the network boundary — is what would close that,
  and it belongs with the other on-machine acceptances this plan has recorded
  for tasks 1, 2 and 5.
- **A proxy that asks who you are is refused by the proxy.** Task 12.
- **A question down the corridor takes no proxy**, by the reasoning above. On a
  company network a paired machine is on the same network and would ordinarily
  be excepted anyway, but that is an argument rather than a decision, and the
  decision is not this task's.

## Proposed changelog entry

> **A machine's proxy now reaches the questions its agent asks.** On a network
> whose only way out is a company proxy, a question put to a provider is carried
> through the proxy the machine's description states — and a machine that cannot
> work out its way refuses the question rather than going around the rule. The
> egress indicator still names the provider, never the proxy, because *where did
> my work go* has the same answer either way. A road out is now told explicitly
> which way it goes, including when that way is straight, so a proxy left in a
> process's environment can no longer decide where somebody's question is sent.

## Proposed queue and roadmap updates

- `docs/autonomy/QUEUE.md`: software and the web task 11 done; task 12 (*A proxy
  that asks who you are, signed in to on every road*) added, ready, depending on
  4 and 11, and carrying an ADR as its first deliverable.
- `ROADMAP.md`: no line moves. *Corporate proxy support, machine-wide and
  honoured* is closer to true and is not finished while a proxy that asks for a
  name cannot be signed in to.
