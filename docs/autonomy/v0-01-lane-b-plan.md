# v0.01, lane B — accounts and session entry

The second loop's plan. `docs/autonomy/v0-01-delivery-plan.md` is the spine of
v0.01; this file carves out the one chain in it that is independent of the
overlay work, so two loops in two checkouts can build at once without ever
choosing the same task.

Point a loop at it with `ALO_LOOP_PLAN=docs/autonomy/v0-01-lane-b-plan.md`, from
a checkout of its own. **Never from a checkout another loop is in** — the lock
forbids it, and the lock is right.

## Why a second file rather than a shared plan

The supervisor has no task-claiming: two loops reading one plan both select the
same next task, launch two workers at it, and produce two competing
implementations racing to publish. Partitioning by file is the claiming
mechanism — a lane owns every task in its file, and the main plan marks these
two as lane B's so the first loop steps over them.

**The cost is a protocol, and it is written here so it is not folklore:** when a
task finishes in this file, the same handoff marks the matching task done in
`v0-01-delivery-plan.md`, in the same commit. The main plan's task 10 depends on
session work it can only see there; a lane that finished quietly would leave the
image task waiting on work that is already done.

## Rules

Everything `v0-01-delivery-plan.md` holds itself to, unchanged: nothing ticked
from a fixture, no release verdicts, tasks sized to a forty-five-minute worker,
and whoever finishes a task writes the next one in the same change.

## Tasks

### 1. The local account that needs no tenant

**Status:** ready. **Depends on:** nothing.

Phase 4's smaller half, and the one the exit gate actually requires: v0.01
opens with *sign in*, and nothing in this repository signs anybody in. A local
account, against the machine's own store — no identity provider, no tenant, no
network. Those are the other half of phase 4 and are not this task.

- **Acceptance:** a local account is created and authenticated against the
  machine's own store; a wrong password is refused in words and is not
  distinguishable by timing from an unknown user; and the session that results
  carries the uid `alo-agentd` is told about, so the machine description and
  the running session cannot disagree about who is signed in.
- **Constraint:** nothing in `crates/alo-shell` — the entry *surface* is the
  compositor lane's; this is the account and the authentication it will call.

**Done, 2026-09-10.** `crates/alo-accounts`: the store at
`/etc/alo/accounts.toml`, Argon2id behind one refusal that costs the same for
a wrong password and an unknown name, and `Session::opened`, which cannot
carry a uid the machine description does not name. Measured in
`tests/a_person_signs_in.rs`; the report is
`docs/autonomy/updates/the-local-account-that-needs-no-tenant.md`. Task 2 is
the next task and was already written.

### 2. The daemon's environment is the session's

**Status:** ready. **Depends on:** 1.

`alo-agentd` runs as the signed-in person and finds their bus at
`/run/user/<uid>`. That is measured (`a_session_that_really_ended.rs`) and not
wired: nothing starts the daemon into a real session with the right environment.

- **Acceptance:** the daemon starts under the session task 1 creates, reaches
  that session's bus, and stops when the session ends — the three states already
  measured, reached from a sign-in rather than from a test harness.
- **On finishing:** mark tasks 4 and 5 done in `v0-01-delivery-plan.md`, in the
  same handoff, so the image task there stops waiting on work that is done.

**Done, 2026-09-10.** `crates/alo-entering` derives what a session hands a
process — `/run/user/<uid>`, the bus address, and `user@<uid>.service` — from
the `Session` task 1 opens, so the sign-in and the machine cannot spell it two
ways. `image/usr/lib/systemd/system/alo-agentd.service` is wired to that
session rather than to `multi-user.target`: pulled in by the person's own
manager, `BindsTo=` it so signing out stops it, ordered after it, and given the
two variables. `crates/alo-image` checks every one of those lines against the
number the machine description names, and `crates/alo-agentd/src/session.rs`
refuses at start-up an environment naming another login's session — it checks
the variables and still never reads one to decide where to connect. The three
states from a real sign-in are
`crates/alo-entering/tests/a_daemon_in_the_persons_session.rs`, `#[ignore]`d for
`a_session_that_really_ended.rs`'s reasons. Report:
`docs/autonomy/updates/the-daemons-environment-is-the-sessions.md`. Task 3 below
is the next task and was written in the same change.

### 3. Where a machine keeps its grants between one sign-in and the next

**Status:** ready. **Depends on:** 2.

`docs/features.md` promises for v0.01: *Grants: pick a folder, see what is
granted, revoke it, and it expires*. A person can now make one —
`crates/alo-picking` — and `alo-agentd` honours the ones it is holding. **Nothing
keeps them.** `crates/alo-agentd/src/starting.rs` says so in as many words: the
service begins with no grants at all, because where a machine's grants live is a
question nobody has answered, and a list read from a file would be a list nothing
writes. So a grant made this morning is gone at the next sign-in, and *see what
is granted* has nothing to show.

This is lane B's because it is session-shaped rather than surface-shaped: a
grant belongs to the person who made it, it is written under their own
authority, and its life is measured from one sign-in to the next. The
**surface** that lists and revokes them is the compositor lane's, as picking's
was.

- **Acceptance:** a grant a person made survives a restart of the daemon and is
  honoured afterwards; a revoked grant does not come back; an expired one is
  gone when it is read rather than being read and then filtered; the file is the
  person's alone and a store somebody else could write is refused in words, as
  `crates/alo-accounts`' store already is; and nothing an agent can send over the
  socket writes a byte of it.
- **Constraint:** no new surface, and nothing in `crates/alo-shell`. This is
  where the list lives and what may write it.

**Done, 2026-09-10.** `crates/alo-remembering`: `/var/lib/alo/grants.toml`, in
the folder the image already makes `0700` for the person, believed under
`alo-accounts`' three rules — not a link, root's or the person's, nobody else
able to write it — and replaced whole or not at all. Every grant is built again
by `alo_capability::Grant::checked` on the way in, so a file hand-edited into
granting `/` is refused by the crate that owns that rule; an expired one is
dropped before the list exists rather than filtered afterwards; and
`alo_capability::Grants::remembered` refuses a file whose handles would collide,
so a revoke cannot land on the wrong grant after a restart.
`crates/alo-agentd/src/main.rs` reads it once, before the socket exists, and
hands `starting::until_stopped` a value with no path in it — which is what makes
*nothing an agent sends writes a byte of it* the shape of the crate rather than
a rule. A file that is there and is not believable stops the service
(`NotStarted::NoGrants`); a machine that has simply never been granted anything
starts and refuses everything, as before. Measured in
`crates/alo-remembering/tests/the_grants_a_machine_keeps.rs` and in
`crates/alo-agentd/src/starting.rs`. Report:
`docs/autonomy/updates/where-a-machine-keeps-its-grants.md`. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 4
below is the next task and was written in the same change.

### 4. A grant made now reaches the daemon now

**Status:** ready. **Depends on:** 3.

Task 3 gave the machine somewhere to keep its grants, and `alo-agentd` reads
them when it starts. What it cannot do is hear about one made **while it is
running**: a person picks a folder at eleven, the file on the disk says so, and
the service holding the turn is still serving under the list it read at sign-in.
The gap is narrow and honest — the daemon is bound to the person's session, so
the grant applies at the next sign-in — but `docs/features.md` promises *pick a
folder*, and *pick a folder and sign out again* is not that promise.

This is lane B's for task 3's reason: the message is the person's, made under
their own authority on their own door, and its lifetime is the session's. The
**surface** that lists and revokes is still the compositor lane's.

The shape that keeps law 2 and ADR 0001 §5 true is a **knock rather than a
payload**: the person's side says only *the grants have changed*, and the daemon
re-reads its own file under the same believing rules. Nothing on the wire
carries a grant, a path or a duration, so a request that arrived from anywhere
else could still not widen anything — and the kernel already says which door a
caller is on.

- **Acceptance:** a grant made while the daemon is running is honoured in the
  same session without a restart, and a revocation takes effect on the next
  question asked; the request carries no grant, no path and no duration, so
  re-reading the person's own file is the only thing it can cause; the same
  request on the agent's door is refused in words and the refusal is written
  down; and a grants file that has become unbelievable while the service is
  running leaves the grants it already had rather than emptying them, with the
  refusal in the record — a machine that forgot what was granted because
  somebody chmodded a file is a machine that went silent.
- **Constraint:** additive to `docs/contracts/daemon-protocol.md`, which is a
  public surface (*contracts outlive code*); no new surface, and nothing in
  `crates/alo-shell`.

**Done, 2026-09-10.** `granted` on the person's door — a knock with no field for
a grant, a path or a duration, so the only thing it can cause is
`crates/alo-agentd/src/rereading.rs` reading the person's own file again under
`alo-remembering`'s three rules. The list is replaced whole, because the file is
what is granted and anything less would make revoking slower than granting; the
one thing carried across is the grant a turn's own invocation made, under the
handle the turn will end it by. A file that has stopped being believable leaves
the grants where they were, says so in the person's language, and the refusal is
written down as `alo_record::Happened::GrantsNotReadAgain` — a new kind of entry
with **no agent field**, because making and revoking a grant is the person's
act. The same request on the agent's door is refused in this crate's own words
and is the one wrong-door message that leaves an entry. Measured end to end over
a real socket in `crates/alo-agentd/src/serving.rs`, and per-decision in
`rereading.rs`, `answering.rs` and `doing.rs`. Report:
`docs/autonomy/updates/a-grant-made-now-reaches-the-daemon-now.md`. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 5
below is the next task and was written in the same change.

### 5. A person can be told what their machine did

**Status:** ready. **Depends on:** nothing in this lane; 4 is done.

`docs/features.md` promises for v0.01: *A record of what the agent did, in
words*. `crates/alo-record` writes it, `crates/alo-keeping` shortens it and
`crates/alo-recounting` turns one entry into a clause a person reads — and
**nothing reads the file back for them.** `alo-recounting` is handed entries by
its caller, and on a running machine the only caller is a test: the record lives
at the path the machine description names, and no part of this repository opens
it on the person's behalf and answers *what happened, and when*.

This is lane B's for task 3's reason: the record is the person's, it is written
under their own authority, and the question *what did my machine do today* is one
they ask about their own session. The **surface** that shows it is the
compositor lane's, as picking's and the grants list's are.

- **Acceptance:** an account of what happened on this machine is read back off
  the real record file and answered as `alo_recounting::Told` values, oldest
  first and bounded, with `alo_keeping`'s *this record does not go all the way
  back* carried into the account rather than dropped; a record file that cannot
  be believed is refused in words rather than answered as an empty day — an
  empty account and a record nobody could read must not look the same; a
  question about a span answers only that span; and nothing an agent can send
  over the socket reaches any of it, as `alo-remembering`'s file is unreachable.
- **Constraint:** no new surface, and nothing in `crates/alo-shell`. Additive to
  `docs/contracts/record-file.md` only if something there moved, which it should
  not have to: this reads the shape that document already fixes.

**Done, 2026-09-10.** The gap was narrower than the heading and worse than it
looked: `alo-recounting` did read a file, but only one a caller already knew the
path of, and on a running machine nothing knew it. `crates/alo-recounting/src/where_it_is.rs`
reads `[record].path` out of the machine description — a third reader of that
file, answerable for one key, refusing a shape it does not know and ignoring
every section that is not its own — so `Recounting::on_this_machine` is a door
a surface can open. The record itself is read through
`alo_keeping::Reading::believed_at`, which asks `alo-accounts`' three questions
of the open file before a word of it is parsed, so a record behind a link, one
somebody else owns or one anybody could write is refused in `alo-keeping`'s own
words rather than answered as an empty day; `Reading::at` is untouched, because
the daemon holds its own record open and a service that stopped writing over a
mode bit would go quiet about the afternoon somebody wanted. `AtMost` is not
optional and there is no unbounded door beside it: an account keeps the **most
recent** of what answered, still oldest first, and says in words when it is not
all of it. Measured in `crates/alo-recounting/tests/what_this_machine_did.rs`
and per-decision in `where_it_is.rs`, `account.rs` and `alo-keeping`'s
`believing.rs`. Nothing moved in `docs/contracts/record-file.md`. Report:
`docs/autonomy/updates/a-person-can-be-told-what-their-machine-did.md`. No task
in `v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 6
below is the next task and was written in the same change.

### 6. The account a person asks for is the one their machine kept

**Status:** ready. **Depends on:** 5.

Task 5 gave the person's side a way to read the record their machine keeps, and
it is read under rules about who may have written the file. What nothing on this
machine can answer yet is the question underneath those rules: **is this the
record this machine wrote, or a believable copy of one?** Every rule task 5 added
is about the file's *place* — its owner, its mode, its not being a link — and all
three are satisfied by a record written whole by whoever already owns the file.
On a personal machine that is the person, and the person is not the threat; on a
managed machine (ADR 0004) an administrator holds a recovery key, and *no
administrator can act as a person* is a promise a record nobody can check makes
thinner than it reads.

The narrow, honest v0.01 shape is **not** a signature — signing needs a key this
repository has nowhere to keep yet, and inventing one here would be the kind of
decision ADR 0004 exists to have made deliberately. It is that the record says
how long it has been going: `alo-keeping::Head` already carries `since` and
`under`, and an entry carries the moment it happened. A file replaced whole by a
plausible copy loses the one thing it cannot forge cheaply — the agreement
between what the daemon has been appending and what the file's own beginning
says.

- **Acceptance:** a record whose entries and whose beginning disagree — an entry
  older than the moment the head says the record starts at, or moments that run
  backwards — is reported as *this record is not what it says it is*, in words,
  alongside everything that could be read, as an unreadable line already is and
  never as a refusal of the whole file; the disagreement is carried into the
  account rather than being a flag on a struct a surface may forget to draw; a
  record that was legitimately shortened is **not** reported, which is the case
  this is easiest to get wrong; and the daemon's own writing and shortening still
  produce a record this check is silent about, measured by writing one with
  `alo_keeping::Writing` and reading it back.
- **Constraint:** no new surface, nothing in `crates/alo-shell`, and nothing in
  `docs/contracts/record-file.md` that is not additive — this reads a shape that
  document already fixes, and what it adds is a reader's rule rather than a
  field.

**Done, 2026-09-11.** `crates/alo-keeping/src/disagreeing.rs`: two
disagreements a believable copy cannot avoid — an entry from before the moment
the head says the record starts at, and moments that run backwards — noticed
while `Reading` walks the file, so there is no reading without the check and no
second door that skips it. Both are sentences opening *this record is not what
it says it is*, drawn beside everything that could be read exactly as
`Damage`'s are, never a refusal; which lines disagree are numbers beside the
sentences. The boundary is legitimate on purpose: an entry **at** `since` is
kept by a shortening, two entries in one moment are a busy second, and a record
`Writing` wrote and pruned reads back silent — measured round-trip. `Account`
carries the disagreement into `said()`, so a surface drawing the sentences
shows it without knowing the check exists. `docs/contracts/record-file.md`
gained the reader's rule additively; not a byte of the shape moved. Measured in
`crates/alo-recounting/tests/a_record_that_is_not_what_it_says.rs` and
per-decision in `disagreeing.rs`. Report:
`docs/autonomy/updates/a-record-that-is-not-what-it-says-it-is.md`. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 7
below is the next task and was already written.

### 7. The pinned model runtime is on the image

**Status:** ready. **Depends on:** nothing in this lane.

ADR 0025 was accepted on 2026-09-11 and the definition now promises that **the
local model is what the machine arrives ready to run**. The ledger's entry for it
(`docs/autonomy/v0-01-evidence.md`) is reachable by work for the first time, and
this task is the first half of that work: the runtime, without the weights. The
weights are a separate task because they carry the sizing question (ADR 0007) and
the carry-or-fetch question ADR 0025 left open inside the work; a runtime with no
model is inert and safe to ship first.

ADR 0006 pinned the runtime and ADR 0019 settled that it is found rather than
configured. Engines are configured, never patched: the runtime goes onto the
image the way everything else got there — a pinned upstream artefact added by
`image/Containerfile`, with its version written where the other pins are, not a
source tree in this repository.

- **Acceptance:** the image carries the pinned model-runtime artefact, and
  `crates/alo-image` holds it the way it holds everything else — a check that
  names it and a twin that breaks one line; the check also holds the pin: an
  unpinned or floating version is refused in words that say why; discovery on a
  machine with the runtime present but no weights still answers with ADR 0019's
  found-nothing answer — a runtime alone must not read as a model; and no rented
  name reaches a person through any string this adds (`alo-saying`'s `rented`
  check stays green).
- **Constraint:** no weights, no catalogue change, no setup flow, and nothing in
  `crates/alo-shell`. If the runtime's licence or its packaging forces a decision
  this repository has not taken, that decision is an ADR handed over as this
  task, in the shape ADR 0024 and ADR 0025 used.

### 9. A model on the disk, sized for the machine it lands on

**Status:** blocked. **Depends on:** 8, and on the catalogue having an entry that
clears the verb-driving bar.

Task 8 took the measurement ADR 0025 owed and the first number did not exist:
every catalogued entry anybody has run `alo-driving` against grades `rarely`,
and the rest are `not-measured`, which is refused as a candidate on purpose. So
**the weights half of *the machine arrives ready to run* cannot be built yet**,
and this task is written down rather than started so that the thing it waits on
is named rather than forgotten.

What unblocks it is not infrastructure and not a decision. It is a catalogued
model that drives the verbs — measured by us, the way `docs/features.md` already
promises the catalogue is measured rather than claimed. Until one exists, a
worker taking this task would be choosing which model to ship by wishing.

- **Acceptance, when it is unblocked:** the image carries weights for the entry
  `Catalogue::agent_for_cpu` recommends on the certified machine's class, pinned
  and digest-checked the way the runtime is; `crates/alo-image` holds it with a
  check per promise and a twin that breaks one line; a machine whose class has
  no entry that clears the bar ships **no** weights and says so through the
  answer `alo-telling` already gives, rather than shipping a model that cannot
  drive anything; and the image's size is stated in `docs/quirks.md` beside the
  measurement task 8 wrote.
- **Constraint:** nothing here chooses for a person (ADR 0016, ADR 0025) — what
  the machine arrives *able* to do is not a value in anybody's settings file.
  Nothing in `crates/alo-shell`.

### 10. What a person is asked at setup, before there is anywhere to ask it

**Status:** ready. **Depends on:** nothing in this lane.

ADR 0009 gave setup a fourth choice — no model, no provider, no agent — with the
same weight as the other three and no persuasion attached, and ADR 0025 settled
that the local one is listed first because it is what the machine can already
do, with **nothing pre-selected**. Neither has anywhere to happen: there is no
setup flow in this repository at all, which the evidence ledger records against
three separate promises.

This is the flow as a value, in the shape `crates/alo-approving` and
`crates/alo-overlay` took their surfaces — decided without drawing one, so that
the drawing is the compositor lane's and the rules are testable now.

- **Acceptance:** the four choices are one enumerated value, ordered with the
  local one first and **nothing selected** until a person selects it; a setup
  that has not been answered is distinguishable from one answered *not at all*,
  and the second is a finished setup rather than a skipped one; choosing writes
  through `alo-choosing`'s own shapes into the person's own file and nowhere
  else; declining writes the same way and leaves a machine whose agent surfaces
  are absent rather than greyed out; and every string a person reads is in the
  vocabulary `alo-saying` collects, with a test that none of them asks anybody
  to buy anything or nudges toward a source.
- **Constraint:** no pixels and nothing in `crates/alo-shell`. It does not
  install a model, does not test a provider, and does not decide what the
  machine arrives carrying — that is task 9's, and this one only offers what is
  there. A choice pre-selected for the person, however reasonable, contradicts
  ADR 0016 and ADR 0025 and is the one thing this task may not do.
