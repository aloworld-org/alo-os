# The one thing that serves the model, and what it may reach

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 32
**Contributor:** Claude (separate checkout, `C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this was

The image carried a pinned model runtime (task 31's predecessor) and 2.23 GiB of
weights (task 31), and **nothing started either of them**. A machine built from
`image/Containerfile` booted with a model on its disk and no process serving it,
so `alo-models` knocked at the loopback address it knows and found nothing there
— which is byte for byte the answer a machine with no model at all gives, and
which is why *the local model is what the machine arrives ready to run* could not
be shown by a disk.

It was left out of task 31 deliberately, because a unit is not a `COPY` line's
worth of decision: which login it runs as, which group may reach it, what it may
reach off the machine and what it is pointed at are four decisions, and three of
them are about authority.

## What changed

### The unit — `image/usr/lib/systemd/system/alo-modeld.service`

Named for what it does and not for what it is. A person types `systemctl status
alo-modeld` and learns their machine serves a model; what we rented to do that is
ours to change and never theirs to learn (`docs/features.md`: *a person never
learns the name of anything we rented*).

- **A login of its own**, `alo-model` (60991), made by `sysusers.d` beside the
  agent's 60989 and the greeter's 60990, in the range systemd's own uid document
  leaves allocated by nothing — the measurement `docs/quirks.md` already carries
  about Fedora allocating system logins downward from 999. **Not the person**,
  whose session comes and goes and who would otherwise have a process in their
  name outliving a session they ended; **not the agent**, which ADR 0001 §2 and
  §5 spend their length keeping authority and identity away from, and which is
  the last login to lend anything to a process that reads every question put to
  this machine. A group of its own for the second half of the same reason: on a
  machine where what a file is reachable by is a group, the service holding the
  model may not stand inside the division ADR 0001 §5 draws around the agent.
- **It holds nothing, and both lines say so**, exactly as `alo-agentd.service`
  and `alo-sessiond.service` do. Serving a model needs no privilege at all: it
  reads weights the image installed, binds a port above 1024, and answers. That
  is ADR 0018's argument said in the third place it has to be said — the one
  privileged component is acceptable because of how little it is trusted with,
  and a capability added here to make something work is that argument quietly
  stopping being true.
- **Its store is the directory the weights landed in and nothing else.** The
  runtime's own default is a home directory belonging to a login this image does
  not make, so a unit that said nothing would serve nothing off a machine
  carrying the model — the emptiest version of this promise, and one that looks
  from the outside exactly like a machine nobody put a model on.
- **It answers at the one address this machine knocks at**, read off
  `alo_models::ollama::DEFAULT_ENDPOINT` rather than spelled a second time,
  because ADR 0019 keeps the address in one file and a checker that wrote its
  own copy would be the drift `alo-image` exists to catch.
- **It reaches nothing off this machine, enforced rather than asserted.**
  `IPAddressAllow=localhost` with `IPAddressDeny=any` under it is a kernel-side
  filter on this unit's own control group, so an update check, a telemetry call
  or a registry pull does not fail politely — it does not leave. Law 1, said
  about the one process on the machine holding the model, whose silence the whole
  measurement rests on. `RestrictAddressFamilies=` says the same thing from the
  other end. There is deliberately no comment doing this job: a comment is not a
  filter.
- Started at boot rather than at a sign-in, because *the machine arrives ready
  to run* is a promise about the machine rather than about whoever is signed in
  to it, and this service belongs to no person. Not restarted, for
  `alo-agentd.service`'s stated reason.

### The recipe — `image/Containerfile`, `image/usr/lib/sysusers.d/alo.conf`

The fourth unit is chmod'ed and enabled beside the other three, and the
Containerfile **asserts 60991** the way it already asserts the other four
numbers: `systemd-sysusers` does not fail on a number somebody else has, it says
so in one line of a build log and creates the login with a different one. The
comment beside the weights `COPY` that used to say *still no unit and nothing
that starts it* now says what starts it.

### The checks — `crates/alo-image`

`Image` reads a fourth service, `THE_SERVER`. `Service` gained `may_reach` and
`may_not_reach`, which are the two halves of systemd's IP access list and are
read as two because an allow list with no deny under it filters nothing at all.
`checking.rs` gained two functions and `wrong.rs` eight variants, each naming the
decision it breaks; `both_units_are_pulled_in` now covers the fourth, which is
this task's own failure mode — a unit in the image that `systemctl cat` shows and
nothing ever starts.

Eleven new tests. Ten break one line of a copy of the real image and assert the
finding; three read the shipped files back in the integration suite. The pair
worth naming: `User=alo` — the edit that looks like tidying up — and
`OLLAMA_HOST=http://0.0.0.0:11434`, one word, which offers this machine's model
to whatever network it is plugged into, to anybody, with no grant, no record and
a dark egress indicator, **because nothing left**.

## The decision this task had to make, and did not have the standing to make alone

The acceptance asked for the service to be *reachable by exactly one group, which
is neither the person's nor the agent's*. **It cannot be, and the reason is a
fact about the kernel rather than about this engine.**

Every door alo OS has decided who may knock at so far is a Unix socket:
`/run/alo/<uid>/…` is 0750 and the agent's group (ADR 0017), `/run/alo-sessiond`
is 0750 and the greeter's (ADR 0024). Both are decided by a `Group=` line and a
mode, because the filesystem carries an owner and a mode for a socket and the
kernel checks them on `connect(2)`. **A TCP socket carries neither.** No
directive in any systemd unit restricts which local uids may connect to a
listening port; `IPAddressAllow=`/`IPAddressDeny=` filter by address, and every
local process connects from the same one. The pinned runtime listens on TCP and
offers no Unix socket, and ADR 0011 forbids patching a rented engine to add one.

So `Group=alo-model` says who **answers**. It does not say, and cannot say, who
may **ask** — and a check written as though it did would be this repository's
worst habit: a green test standing where a boundary is not.

What was done instead:

- **The unit decides everything a unit can decide**, and every one of those is
  checked and has a twin that breaks it.
- **`docs/quirks.md`** carries the finding, in the section for engines behaving
  unlike their manuals, so the next person to read `Group=alo-model` does not
  read it as the sentence the other two units' `Group=` lines are.
- **`docs/decisions/0027-who-may-ask-the-model-anything.md`** is the decision,
  **proposed**. It sets out what is actually at stake — not a grant boundary (no
  verb touches the runtime), not an egress (nothing leaves), but the owner's
  compute today and, at v0.5, an ADR 0005 sandboxed application reaching the
  machine's model around the portal by opening a TCP connection to 127.0.0.1 —
  and prices four answers: leave it and say so; a door of ours in front of the
  runtime (rejected: a component of ours in the hottest path in the system,
  bought for an obstacle rather than a boundary); a shared network namespace
  (rejected, and recorded as rejected because it is the first thing a reader
  reaches for: whoever joins it has only it, and the process that would join is
  the one that talks to hosted providers); and a rule in the boundary this
  machine already loads, which is the real answer and is a second programme on
  the one privileged component, so its own ADR at its own time.
- The recommendation is **the machine as it is now, and the gap closed when the
  sandbox that makes it matter arrives** — asking the owner for one line in
  `docs/features.md`'s *v0.5* section beside the sandbox rather than a note in a
  report nobody inherits. **No promise was narrowed and `docs/features.md` was
  not touched**: only the owner moves the definition.

`crates/alo-image` says in as many words, beside the check, that it does not
answer who may connect, and points at ADR 0027. Nothing here may be read as
having answered it.

## Which of the two the egress claim is

**A setting read, not a machine watched.** The unit says it, in the one place
where saying it is enforcing it — systemd's IP access list is applied by the
kernel to the service's own control group — and `crates/alo-image` checks the
unit says it. **Nothing in this lane has booted this image or put a packet
counter beside it.** `docs/autonomy/v0-01-evidence.md` says so under the promise,
and *arrives ready to run* stays owed.

## A fixture that stopped testing what it said

`a_description_whose_agent_is_no_login_is_caught` used `agent = 60991` as *a
number no login has*. It is the model service's now, so the fixture was quietly
producing a different finding — a truer one, `TheServerIsSomebodyElse`, saying
that the process holding the model would be the agent. The fixture moved to
60992 and the test says why beside it. Caught by the gate rather than by a
reader, which is the whole argument for the twin tests.

## Decisions taken here that were open

- **The unit is `alo-modeld.service`**, not `ollama.service`. The rented name is
  one a person would otherwise read in `systemctl status`.
- **60991**, beside the agent and the greeter, and asserted in the build for the
  reason the other four are.
- **A system service started at boot**, not a user unit bound to a session: the
  promise is about the machine.
- **`StateDirectory=alo-model`, 0700,** and `HOME` pointed at it, because the
  runtime expects a writable home and `/usr/share/alo/models` is on the
  read-only half of a bootc machine.

## Verification

Windows 11, `C:\dev\alo-os-claude`, 2026-09-11.

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy -p alo-image --all-targets -- -D warnings` | clean |
| `cargo clippy -p alo-citing -p alo-reconciling --all-targets -- -D warnings` | clean |
| `cargo test -p alo-image` | 151 + 22 passed |
| `cargo test -p alo-citing` | 21 + 10 passed |
| `cargo test -p alo-reconciling` | 20 + 15 passed |

`alo-citing` and `alo-reconciling` are run because this change writes a decision
and edits the evidence ledger, and both read this repository's own documents
rather than a list kept beside them. The workspace suite was not run here; the
supervisor runs it.

**Not run and not claimed:** `docker build -f image/Containerfile`. Nothing in
this lane has built or booted this image, so the unit is a declaration held to
the decisions it is about rather than a process anybody has watched start.

## Remaining limitations

- *Arrives ready to run* stays owed. No machine has booted this image; no
  catalogued model clears the verb-driving bar, this one included.
- Who on this machine may ask the model anything is undecided and is ADR 0027's.
  It is not a v0.01 exposure and it is named as v0.5's.
- The egress claim is a setting, not a measurement.

## Proposed changelog entry

**The machine serves the model it arrived with.** alo OS images now start the
local model at boot, under a login of its own that holds no privilege, pointed at
the weights the machine shipped with, answering only to this machine and
reaching nothing off it — enforced by the kernel rather than promised in a
comment. Until now a machine arrived with a model on its disk and nothing serving
it.

## Proposed queue and roadmap updates

- `docs/autonomy/QUEUE.md`: nothing new. Task 32 is done in the delivery plan and
  task 33 is written there.
- `ROADMAP.md`: **no box moves.** An image that builds is not an image that
  boots, and *On the machine* is unchanged.

## Files touched

- `image/usr/lib/systemd/system/alo-modeld.service` (new)
- `image/usr/lib/sysusers.d/alo.conf`
- `image/Containerfile`
- `crates/alo-image/src/lib.rs`
- `crates/alo-image/src/image.rs`
- `crates/alo-image/src/service.rs`
- `crates/alo-image/src/checking.rs`
- `crates/alo-image/src/wrong.rs`
- `crates/alo-image/src/testing.rs`
- `crates/alo-image/tests/what_the_image_owes_the_daemons.rs`
- `docs/decisions/0027-who-may-ask-the-model-anything.md` (new)
- `docs/quirks.md`
- `docs/autonomy/v0-01-evidence.md`
- `docs/autonomy/v0-01-delivery-plan.md`
- `docs/autonomy/updates/the-one-thing-that-serves-the-model.md` (this report)
