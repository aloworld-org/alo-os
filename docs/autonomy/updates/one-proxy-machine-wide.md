# One proxy, machine-wide, honoured

**Date:** 2026-09-15
**Workstream:** v0.5 — software and the web (`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 4)
**Contributor:** Claude Code worker in `C:\dev\alo-os-2`
**Status:** ready for integration.

## What a person gets

A machine that works inside a company network. One place to set the proxy —
none, an address per scheme with the places excepted from it, or the address of
the rule the network itself publishes — and everything on the machine then uses
it: installing an application, updating the system, asking a provider, and the
applications the person installed. On a machine an organisation manages the
proxy is the organisation's, and a person who tries to change it is **told so**
rather than finding a field that quietly does nothing.

Two things a person can check. **The password never lands in a file** — the
setting holds the name it is kept under in the keyring, and there is nowhere in
it for a credential to be. And **the egress indicator still says where their work
actually went**: *alo OS is installing an application from dl.example.org*, not
*from the proxy*, because *it went to the proxy* is not what anybody needs to
know.

And one thing it deliberately will not do: where the network publishes a rule
this machine cannot read, the connection is **refused, in words**, rather than
quietly sent straight out around the company's own rule.

## What changed

### The new crate, `crates/alo-proxy`

It decides and reaches nothing — no socket, no thread, no file, no script
engine. Fifteen files, one subject each.

| | |
|---|---|
| `setting.rs` | `TheProxy` — the machine's one setting, in three shapes and no fourth. There is no member meaning *work it out from the environment*. |
| `kept.rs` | `Kept` and `SetBy` — whose it is (ADR 0016), and `NotChanged::AnOrganisationSetIt` with its sentence. |
| `reaching.rs` | `Reaching` — where a road is going, as a decision needs it: a scheme and a host, checked at the boundary, and **no path and no query**. |
| `address.rs` | `ProxyAddress` — one host, one port, how it is spoken to, and where its password is kept. A credential pasted into it is refused by name. |
| `password.rs` | `WhereThePasswordIs` (what the setting holds) and `Password` (held for one road, no accessor, `Debug` that says nothing). |
| `exceptions.rs` | `Exceptions` — the places that go straight out; a label-boundary match, no wildcards, no ranges. |
| `road.rs` | `Road` — the closed list of every road out — and `Way`, which names the proxy and **never a destination**. |
| `deciding.rs` | `the_way` — the one door every road goes through. |
| `carried.rs` | `Carried` — what a road is then given, and the one function in the crate that turns a password into text. |
| `automatic.rs` | `ConfigurationAddress`, `TheEvaluator`, `NotEvaluated`, and `understood` for what an answer may be. |
| `evaluator.rs` | `TheRentedEvaluator` — a separate program, environment cleared, no shell, three checked arguments. |
| `published.rs` | `Published` — what an application's own environment is given. |
| `portal.rs` | `looked_up` — the same answer, one address at a time, for the portal applications already ask. |
| `refusing.rs` | `NotOnTheRoad` — *nothing was sent*, in the person's language. |
| `words.rs` | Seventeen strings, no gaps in any of them, and a test that none names the machinery. |

### The three roads, each in its own crate

- **Installing, checking for application updates, updating** —
  `crates/alo-software/src/rented.rs`. `TheRentedTool` gained `taking`,
  `environment` and `at`; its environment is cleared and *then* given the
  machine's proxy, so what the tool honours is the setting and never what a
  caller happened to carry.
- **The system's own update** — `crates/alo-updating/src/the_base.rs`. `TheBase`
  gained `taking`, `environment` and `through`. This program's environment is
  the base's own and is **not** cleared, which is why a straight road says
  `http_proxy=` (empty) rather than saying nothing: an inherited line in a
  service file must not decide where a system is fetched from.
- **A provider** — `crates/alo-models/src/trying.rs`. `Trying` gained `taking`,
  and `under` now configures the request with it **explicitly, including
  `None`**. That is a correctness fix as much as a feature: `ureq`'s default
  configuration picks a proxy up from the process environment, so until this
  change alo OS's own road to a provider could be pointed somewhere by a
  variable nobody chose and nobody could be shown. `alo-secrets` refuses to read
  an environment for exactly that reason.

### Registration

- `Cargo.toml` — `crates/alo-proxy` is a workspace member.
- `crates/alo-saying/src/collecting.rs` and its manifest — the crate's words are
  collected into the machine's one vocabulary. `alo-collected` reads the
  workspace and would have found a crate with a `src/words.rs` that nothing
  collects; this is that registration, and nothing else in `alo-saying` moved.

## Decisions, and why

**The proxy is a leaf crate that depends on nothing of this workspace but
`alo-strings`.** It could have depended on `alo-egress` — the indicator is the
other closed list of what leaves this machine — and deliberately does not. Two
crates answering *where is this going* from one value is how the indicator's
answer and the proxy's answer end up being the same thing read two ways, and the
one property this task must not break is that they are different: the indicator
names the destination, the proxy names the way. `alo-egress` is a **dev**
dependency, for the one test that needs both.

**`Road` is a closed list, and the road is named even though the answer does not
depend on it.** *Machine-wide* is a claim about coverage, and a claim about
coverage is only as good as the list it is checked against. A road that reached
the network without naming itself would be the one nobody could hold to this.
That the answer is the same for all seven is the promise, not an oversight: a
setting that sent updates one way and questions another would be two settings
wearing one name. `tests/every_road_out_takes_it.rs` holds the six that are also
`alo_egress::Errand`s against that list, one at a time, so a seventh errand added
in `alo-egress` fails there with a name rather than with a number.

**An automatic configuration is fetched *and* evaluated by a separate program.**
The acceptance says the script must not run in any process that holds a grant,
and the honest way to make that true rather than intended is a process boundary:
`TheRentedEvaluator` starts a program directly, with `env_clear`, with three
checked arguments, and with an environment (`nothing_of_this_machines`) that
names no session, no bus, no home and no credential. A test reads that list for
each of those words. Which program is pinned at `THE_EVALUATOR` is the image's,
exactly as `alo_software::asked::THE_TOOL` is — and a machine with nothing there
**refuses**, which is the next decision.

**Fetching it needs no seventh `alo_egress::Errand`, and that is argued rather
than assumed.** A configuration is only ever asked about a road that is already
being taken, and that road is already on the indicator under its own errand —
the same way the rented tool's own name lookups and handshakes are inside *alo OS
is installing an application from …*. What would need an errand of its own is a
machine that fetched a configuration when nobody was going anywhere: on a timer,
at sign-in, to keep one warm. Nothing here does, and `TheEvaluator` has one
method and it takes a road, so nothing here could. `crates/alo-proxy/src/automatic.rs`
carries that argument where the next person to change it will read it.

**A refusal is never answered as *straight out*.** This is the decision with the
most at stake in the task. On a network where the proxy is a *rule* rather than
merely a route, a machine that fell back to a direct connection would send a
company's traffic around its own rule and nobody would be told. So: nothing
evaluates it → refuse; the evaluator failed → refuse; it answered something this
machine does not understand, SOCKS included → refuse; and the portal answers an
error rather than `direct://`. Four tests, one per way of failing.

**No fallback list.** A configuration answering `PROXY p:8080; DIRECT` is
honoured as *the proxy*. Taking the second half by ourselves would be this
machine deciding, on a network it cannot see, that a company's rule had stopped
applying.

**Loopback is never proxied, whatever is set.** A model answering on this
machine (ADR 0007) is reached on loopback, and a proxy setting that broke the
local runtime would stop the product working the day somebody typed a company
address into a settings panel. It is decided in `Reaching::is_this_machine`
rather than left to an exception somebody remembers to write, and it is on the
`no_proxy` list every program is given as well.

**No credential is published to applications.** What an application's
environment is given, and what the portal answers, carry the proxy's address and
never a name or a password. Handing a company's proxy credential to every
sandboxed application on the machine would be a grant nobody made, to software
nobody here wrote, in the one place a sandbox cannot take it back. **The cost is
named rather than hidden:** on a network whose proxy asks for a credential, an
application is asked for it once per application instead of never. That is the
same bargain the browser's own proxy prompt already is.

**An automatic configuration is not faked into a variable.** An environment says
*this proxy, for everything*; an automatic configuration says *it depends where
you are going*. `Published::OnlyWhenAsked` is the honest answer, and the road for
those applications is the portal, which asks per address — which is what the
configuration answers. Evaluating it once against an address nobody asked about
and publishing that would be a guess.

**`SpokenTo` has no SOCKS.** A SOCKS proxy carries every protocol rather than
this machine's two, and shipping a member this crate could not actually hand to
every road out would be a setting that silently does nothing on some of them.
Adding one later is additive. `understood` refuses every spelling of it by name
rather than reading it as *straight out*.

**A proxy address is held to being a host where it is typed.** `ProxyAddress`
checks labels, the four-part number form and a bracketed number, because the
value goes on to configure a client that parses it — and a refusal there would
land in the middle of somebody's work instead of in the settings panel they
typed it into. That is also what makes `Carried::for_a_request` a refusal nobody
has to invent an error for.

## Acceptance, clause by clause

| The plan says | Where |
|---|---|
| one proxy setting: none, manual per scheme with exceptions, or automatic | `setting.rs`, six tests |
| machine-wide as a bound an organisation or a person sets (ADR 0016) | `kept.rs`, five tests |
| every road out — installing, updates, providers — a test per road | `alo-software`, `alo-updating` and `alo-models` test files, and `alo-proxy/tests/every_road_out_takes_it.rs` |
| published to applications, through the environment and the proxy portal | `published.rs`, `portal.rs` |
| a proxy's password goes to the keyring, never to a file (ADR 0022) | `password.rs`, `address.rs`, `setting.rs` |
| the egress indicator names the real destination, not the proxy | `alo-proxy/tests/every_road_out_takes_it.rs` |
| an automatic configuration fetched and evaluated in no process holding a grant | `automatic.rs`, `evaluator.rs` |
| a policy that cannot be evaluated refuses | `deciding.rs`, `refusing.rs`, `portal.rs` |

## Verification

Run in WSL Ubuntu from `/mnt/c/dev/alo-os-2`, with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-2-72aa7fda7f7de151`. **The Windows
host cannot build this workspace at all** — `ring`'s build script needs a C
compiler the `x86_64-pc-windows-gnu` toolchain here has none of — so every gate
below was run on Linux.

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -p alo-proxy -p alo-software -p alo-updating -p alo-models -p alo-saying -p alo-collected -- -D warnings` — clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-proxy -p alo-software -p alo-updating -p alo-models -p alo-saying` — clean.
- `cargo test -p alo-proxy` — 85 unit tests, 4 integration tests, 1 doctest, all pass.
- `cargo test -p alo-software` — passes, including the five new road tests.
- `cargo test -p alo-updating` — passes, including the four new road tests.
- `cargo test -p alo-models` — 217 unit tests plus the four new road tests, all pass.
- `cargo test -p alo-saying` and `cargo test -p alo-collected` — pass, which is
  what says the new crate's words are collected.

The full workspace suite was **not** run here; the supervisor runs it.

### The second pass, after the gate refused once

Everything above was run again on the same tree, and each acceptance test was
run **on its own** rather than as part of a suite:

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets --workspace -- -D warnings` — clean. The whole
  workspace rather than this task's crates, because the refusal named a crate
  this task does not change and a cross-crate break had to be ruled out:
  `TheRentedTool` stopped being `Clone` and `PartialEq` in this change, and
  `cargo clippy --all-targets` is what says nothing else relied on either.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` for the five crates — clean.
- `cargo test -p alo-proxy` (85 + 4 + 1), `-p alo-saying` (63 + 4 + 1),
  `-p alo-collected` (8 + 11), `-p alo-models` (217 + 55), `-p alo-software`
  (75 + 29 + 1), `-p alo-updating` (9 + 25) — all pass.
- Each of the nineteen tests in this report's evidence, run by name with
  `--exact`, one `cargo test` invocation each — all pass.

## The gate that refused this task once, and why it was not this work

The supervisor refused a first hand-over of this task on the workspace suite, on
`alo-agentd`'s `a_machine_on_two_networks_is_found_on_each_of_them` — twice,
which is why it came back as work rather than as weather. It is not this work,
and the reason is worth writing down rather than asserting.

**What this change can reach in `alo-agentd` at all.** One thing: seventeen more
strings in the vocabulary `alo_saying::everything_this_machine_can_say`
assembles, which `alo_agentd::what_this_machine_says` loads. Nothing in this
change is on a network, a socket, a thread or a clock — `alo-proxy` opens
nothing by design, and the three road tests start a program (`/usr/bin/env`) and
read what it was given. None of the new tests binds a port or makes a namespace,
so none of them can be in the way of a fixture that does.

**What was run.** On this tree, with the other checkout's own
`cargo test --workspace` running beside it the whole time:

- `cargo test -p alo-agentd --test a_machine_on_two_networks` — **nine runs, nine
  passes**: six with the machine otherwise idle, three while the other
  checkout's suite was running.
- `cargo test -p alo-agentd --lib` — 394 pass, 8 ignored, under that same load.
  That is the path this change can actually reach: if the vocabulary this crate
  added a crate to had stopped assembling, `what_this_machine_says` is what would
  say so, and the studio inside that very fixture unwraps it.

**What it looks like instead.** That fixture asks a question on a real network
and waits `alo_agentd::WHILE_LOOKING` — two seconds — for a machine it started
moments earlier to answer, and a second at a time for datagrams after that. Both
checkouts were running full workspace suites on a four-processor machine when it
was refused, and both suites contain network-namespace fixtures. A two-second
window missed under that load is a failure of the deadline, not of the discovery
it is holding.

**What is not proposed here.** Nothing in `alo-agentd` is touched by this change,
and hardening that fixture is not this task's to do: `WHILE_LOOKING` is the
product's own constant, and giving one fixture a second reason to change belongs
to whoever owns it. It is left here as a finding for the integration owner, with
what would settle it — that fixture waiting on the answer it needs with a bound,
instead of on a fixed window sized for a quiet machine.

## What is not shown, and what would show it

- **A real proxy server.** No test in this change stands a proxy up and watches
  a question, an installation or an update go through it. What is shown is that
  the child process really received the setting (`/usr/bin/env` as the program,
  reading back what it got) and that the request really is configured with it.
  The machine acceptance is: set a proxy on a certified machine, watch an
  installation and a system update succeed through it with the direct route
  blocked, and watch both fail in words with the proxy stopped.
- **A real automatic configuration.** No machine in this change has a program at
  `THE_EVALUATOR`. What is shown is the refusal — a machine with nothing there
  refuses and never goes straight out — and the argument list and environment
  such a program would be given. The machine acceptance is: pin an evaluator in
  the image, publish a configuration on a test network, and watch a road that
  the configuration sends through a proxy and a road it sends direct.
- **Which program is pinned as the evaluator.** That is the installer plan's, as
  the rented tool's path is. This change names the door and the refusal.
- **Reading an organisation's proxy out of `/etc/alo/agentd.toml`.** That file
  is `alo-agentd`'s and this plan reads it and never edits it; `Kept` is the
  value it would hand over. The section would be shaped exactly as `[questions]`
  is in `docs/contracts/machine-description.md`, including the `format` rule that
  an older service must refuse a description whose proxy bound it cannot
  enforce. Task 8 does the same job for where applications come from and is the
  natural place to add it.
- **Wiring `looked_up` into the proxy portal.** `alo-portals` is the
  applications plan's and this plan reads it and never edits it — the same
  division task 3 named for `OpenURI`. `looked_up` is the answer; owning the
  name on the bus and turning `NotLookedUp` into a portal failure is that
  plan's.
- **Handing `Published` to an application as it starts.** The same division: the
  sandbox's own start is the applications plan's.

## Proposed changelog entry

> **A proxy the whole machine uses.** A machine on a company network now has one
> proxy setting — none, an address of your own, or the address of the rule the
> network publishes — and everything uses it: installing and updating
> applications, updating the system, asking a model that answers elsewhere, and
> the applications you installed. On a machine your organisation manages the
> proxy is theirs, and you are told so rather than finding a setting that does
> nothing. A proxy's password is kept in the keyring and never written into a
> file. Where the network publishes a rule this machine cannot read, the
> connection is refused in words rather than quietly sent around it. And the
> indicator still tells you where your work actually went, not that it went to
> the proxy.

## Proposed queue and roadmap updates

- `docs/autonomy/QUEUE.md`: v0.5 *corporate proxy support* — the decision, the
  three roads and both ways applications honour it are built and tested;
  remaining is the machine acceptance above, the `/etc/alo/agentd.toml` section
  (task 8's shape) and the portal wiring (the applications plan's).
- `ROADMAP.md`: the v0.5 *corporate proxy support* line has its code and its
  tests; it is not yet demonstrated on a machine.
- `docs/features.md`: unchanged. *Corporate proxy support, machine-wide and
  honoured by applications* is the promise this keeps; nothing was narrowed.
- One finding for whoever owns `alo-agentd`, not for this task:
  `crates/alo-agentd/tests/a_machine_on_two_networks.rs` holds its assertions on
  fixed windows — `WHILE_LOOKING`, and a second for datagrams — and misses them
  when both checkouts run a workspace suite at once on this machine. The section
  above says what was run to establish that. A fixture that waited on the answer
  with a bound rather than on a window would stop costing a task a gate.
