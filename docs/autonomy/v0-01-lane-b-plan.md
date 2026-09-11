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

**Done, 2026-09-11.** `image/Containerfile` carries the runtime the way it
carries everything else: `THE_RUNTIME=0.34.0` and the artefact's own sha256
written where the other pins are, fetched in a stage of its own and refused
unless the digest matches before a byte of it is unpacked, landing
`/usr/bin/ollama` and `/usr/lib/ollama/` — no weights, no unit and nothing that
starts it, because a runtime alone answers nothing and both belong to the
weights work. `crates/alo-image` grew `runtime.rs`, a reader of the recipe held
apart from ADR 0006's one-file rule on purpose (that rule is about how the
runtime is spoken to; where its files land is the image's own fact), and three
disagreements with twins: dropped from the image, version floating — refused in
words that say why `latest` cannot ship — and a digest the build stopped
checking. `alo_models::found_at` now answers ADR 0019's found-nothing for a
runtime holding no weights, so the artefact arriving on every machine cannot
read as a model on any of them; and `crates/alo-saying` pins that the runtime's
name stays on the rented list and still reaches nobody. Measured in
`crates/alo-image/src/checking.rs`, `crates/alo-models/src/ollama.rs` and
`crates/alo-saying/src/rented.rs`. Report:
`docs/autonomy/updates/the-pinned-model-runtime-is-on-the-image.md`. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 8
below is the next task and was written in the same change.

### 8. The carry-or-fetch measurement ADR 0025 owes

**Status:** ready. **Depends on:** 7.

ADR 0025 recommends carrying the weights on the certified image and fetching
only where an image cannot — and says in as many words that this is *a
recommendation with a measurement owed*, and that this measurement is **the
first thing the implementation owes**. Task 7 put the runtime aboard; nobody
may put weights aboard on a recommendation whose numbers nobody has.

The measurement is two numbers and a sentence. **The model:** the smallest
catalogued entry that clears the verb-driving bar on the certified machine's
shape (ADR 0007 makes the CPU the default; `crates/alo-driving` owns the bar and
the catalogue records the grades — an entry that is unmeasured is not a
candidate, it is a gap the ledger already carries). **The channel:** what the
image and its update stream can honestly carry, which is a question about the
update channel's own constraints (ADR 0011's bootc image, `docs/features.md`'s
promise that an upgrade cannot break a working stack), not about what a
developer's connection tolerates. **The sentence:** carried, or fetched at
setup where an image cannot — remembering ADR 0025's own caution that a machine
which fetches at setup is not local by default when it is offline at setup.

- **Acceptance:** the smallest catalogued model that clears the verb-driving
  bar is named, with its size in bytes and the grade that clears the bar, off
  the catalogue rather than from memory; what the update channel can carry is
  stated with its reasoning, honestly bounded where it cannot yet be measured
  on real infrastructure; the answer — carry, or fetch, or carry-here and
  fetch-there — is written in `docs/quirks.md` beside the numbers, which is
  where ADR 0025 says it goes; and the weights task that builds on the answer
  is written as the next task in this plan, in the same change.
- **Constraint:** a measurement, not an implementation — no weights on the
  image, no setup flow, no catalogue regrade and nothing in `crates/alo-shell`.
  If the honest answer is that the bar clears on no catalogued entry for the
  certified machine, that finding **is** the deliverable, written where the
  ledger can carry it, and the weights task waits on the catalogue rather than
  on wishes.

**Done, 2026-09-11.** The first number does not exist, and that finding is the
measurement: the five catalogued entries anybody has measured all grade
`rarely`, the seven others say `not-measured`, and an unmeasured entry does not
clear the bar on purpose — so the smallest catalogued model that clears the
verb-driving bar is no model at all, and **no weights go aboard**, carried or
fetched, on a recommendation with nothing to weigh. The channel half is
answered as far as it honestly can be: a bootc image's content-addressed layers
move weights once per weights *change* rather than once per update, a carried
layer rides inside the atomic deployment `bootc rollback` restores where a
setup-time fetch sits outside it, and the sizes in question (1.06–4.9 GB) are
the order of what the image already moves — with our registry, mirrors and the
certified machine's network unmeasured and said so. The numbers, the reasoning
and the sentence are in `docs/quirks.md` under *The carry-or-fetch measurement
ADR 0025 owes*, where the ADR says the answer goes, and
`crates/alo-models/tests/the_carry_or_fetch_measurement.rs` holds the entry to
`data/catalogue.toml`: every size and grade is compared against the catalogue,
and the day an entry clears the bar the test fails and sends whoever sees it
back to make the measurement again. Report:
`docs/autonomy/updates/the-carry-or-fetch-measurement.md`. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 9
below is the next task and was written in the same change.

### 9. The grade the weights wait on

**Status:** ready. **Depends on:** 8.

**The memory question is answered.** This task was written *ready, and it needs
a machine with room* — every unmeasured entry wants ten gigabytes or more, and
the box every existing grade was made on has six. The development machine this
lane runs on has **15.5 GB**, measured 2026-09-11, so the three entries below
are runnable here and the grade is no longer waiting on hardware. What a run
costs is time and a download, not a purchase.

Task 8 measured what there is and found the honest answer: no catalogued entry
clears the verb-driving bar, so the weights task waits on the catalogue rather
than on wishes. What unblocks it is a grade — a 7B-class entry measured
`reliably`, or the finding that this catalogue is not enough and needs entries
nobody has curated yet. Either answer is the deliverable; only running the
measurement can say which.

- **Acceptance:** the smallest unmeasured entries an ordinary business laptop
  could run — `mistral-7b-instruct`, `teuken-7b-instruct` and
  `qwen2.5-7b-instruct`, each `on_cpu = "workable"` with `min_ram_gb = 10` and
  an unconditional licence — are measured with `alo-driving` against the
  pinned runtime holding the weights each entry's `upstream` names at the
  quantisation it states; the grade each run earned is written into
  `data/catalogue.toml` and into `catalogue.rs`'s `MEASURED` list, exactly as
  the five before them were; what each model wrote goes into `docs/quirks.md`'s
  models section beside the earlier runs; and the next task is written from
  the outcome — the weights-aboard task if a grade clears the bar (task 8's
  test will insist the carry-or-fetch entry is revisited in the same change),
  or widening the catalogue if none does, with the finding stated rather than
  softened.
- **Constraint:** grades come only from runs actually made — no regrade from
  memory, no prompt or scoring loosened until a model passes; no weights on
  the image, no setup flow, and nothing in `crates/alo-shell`. A machine
  without the memory for a run does not guess; it leaves `not-measured`
  standing, which is the true sentence about it.

**Done, 2026-09-11.** The run was made and **no grade was earned**, which is
this task's finding rather than its failure: the memory question was answered
about the wrong machine. The 15.5 GB is the Windows host's; the pinned runtime
and every grade in the catalogue live inside a WSL guest that
`C:\Users\SBW\.wslconfig` caps at 6 GB and four cores, for a reason its own
comment gives — a 15.5 GB host that already pages cannot lend more.
`mistral:7b-instruct-v0.3-q4_K_M` was fetched at the quantisation the entry
states and put to `alo-driving` there twice. It loads, and the load alone takes
284–447 s against the five minutes `alo_models::WHILE_A_MODEL_THINKS` waits, so
the first run failed at the harness's own warm-up. Pre-loaded outside the
harness, it is 5.0 GB inside a 5.9 GB guest and runs against swap at **0.25
tokens per second**: the twelve-token warm-up took 101.7 s, and the `list`
exercise's 715-token prompt passed five minutes with three tokens written —
`TookTooLong`, the run stopped, and the harness graded nothing, which is what
it is built to do rather than blame a model for a machine. Between the two
attempts the guest itself went down. Nothing was loosened to get a number:
not the prompt, not the scoring, not the runtime's context window, not the
wait. So **`teuken-7b-instruct`, `mistral-7b-instruct` and
`qwen2.5-7b-instruct` stay `not-measured`**, task 10 stays blocked, and task 8's
carry-or-fetch entry stands unchanged because no grade moved.

Two things were found that outlive this box. **openGPT-X publishes Teuken
twice** — a research release (`license: other`) and a commercial one
(Apache-2.0) — and the entry named the research release while stating the
commercial licence, which is `data/catalogue.toml`'s first rule broken by the
catalogue itself; `upstream` now names the release the licence was always true
of, and a test refuses any entry that permits commercial use while naming a
research release. **And neither Teuken release ships a first-party GGUF**, so
its stated `Q4_K_M` names an artefact the publisher does not publish and a
grade for it waits on a decision about whose requantisation the entry means —
not only on a machine with room. The numbers, the three rejected ways round
the box, and both findings are in `docs/quirks.md`; the finding is held to the
catalogue by `crates/alo-models/tests/the_grade_the_weights_wait_on.rs`.
Report: `docs/autonomy/updates/the-grade-the-weights-wait-on.md`. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 12
below is written from this outcome and was written in the same change.

### 10. A model on the disk, sized for the machine it lands on

**Status:** blocked. **Depends on:** 9, and on the catalogue having an entry that
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

### 11. What a person is asked at setup, before there is anywhere to ask it

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

**Done, 2026-09-11.** `crates/alo-setting-up`: `THE_FOUR`, ordered with the
local one first, and `SettingUp` — which has no constructor taking a selection,
so *nothing is pre-selected* is a property of the type rather than an initial
value somebody may improve later, and pressing on without choosing is
`NotSetUp::NothingSelected` rather than a default quietly taken. The four are
`docs/features.md`'s own four; alo's own service has no variant, because ADR
0014 makes it one more provider. Answering with a machine on this network is
refused in words — this machine keeps no list of paired ones — and that refusal
is the honest form of a choice alo OS offers and cannot yet carry out.

**The state that made this possible is a bit in the person's own file.** A
machine nobody has configured and one whose owner said *not at all* both have
nothing answering questions, so without somewhere to record *and they were
asked*, setup would be shown again to everybody who declined — ADR 0009's *no
nagging* broken by the one mechanism guaranteed to meet all of them. So
`alo-choosing` grew `Setup`, a `[setup] answered` section, `THE_FORMAT = 3` with
`1` and `2` still read, and one door: `Choosing::setting_up`. It is **not** a
second copy of the choice — what was answered is `[answers]`, so there is
nothing for the two to disagree about — and `alo-setting-up` writes through that
door and no other, which `tests/nothing_here_writes_anywhere_else.rs` reads off
the manifest and the source rather than promising.

Every string is in `alo-saying`'s one vocabulary, and `crate::nudging` is the
*no persuasion attached* half that no structural rule could see: sixteen words
that would turn one of the four into the answer — a ranking, or money — walked
over this crate's sentences **and its translator notes**, because a translator
told which one is sensible writes that into twenty-four languages no test here
can read. Measured in `crates/alo-setting-up/tests/what_a_person_is_asked_at_setup.rs`,
`crates/alo-setting-up/tests/nothing_here_writes_anywhere_else.rs` and
per-decision in each of the crate's own files, and in `alo-choosing`'s
`choosing.rs`, `written.rs` and `writing.rs`.
`docs/contracts/person-settings.md` gained `[setup]` and format 3 additively.
Report: `docs/autonomy/updates/what-a-person-is-asked-at-setup.md`. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 12
below was already written and is the next task.

### 12. Candidates the measuring box can actually hold

**Status:** ready. **Depends on:** 9.

Task 9 found that the catalogue's unmeasured half is exactly the half this
lane's box cannot load, and that trying anyway takes the guest down. So the
route to a grade is not a bigger model run harder: it is **entries small enough
to be measured here that have a reason to clear the bar**. The five that were
measured are general chat models between 1.7B and 3.8B, and all five failed at
the *shape* rather than at the reasoning — the envelope right and the argument
list wrong, a fence round the answer, the prompt's own placeholders copied
back. Models trained for tool calls and constrained output are a different
population, and nobody here has put one to the fixed set.

This is lane B's because it is the catalogue's, and the catalogue is where
task 10's blocker lives. It is a curation task with a measurement in it, not a
decision: ADR 0007 already says the grade is ours to run and never the
publisher's to claim.

- **Acceptance:** at least two new catalogued entries whose weights fit the
  memory the measuring box has, chosen because they are trained for structured
  output or tool use rather than because they are small; each entry's licence
  read against the publisher's own metadata rather than from memory, stated
  with its conditions, and its size what the artefact actually is; each one
  measured with `alo-driving` against the pinned runtime in the same change and
  graded from the run it earned — `not-measured` with the reason where a run
  could not be made, never a grade from a parameter count; what each model
  wrote in `docs/quirks.md` beside the earlier runs; and if any grade clears
  the bar, task 8's carry-or-fetch entry revisited in the same change, which
  `crates/alo-models/tests/the_carry_or_fetch_measurement.rs` will insist on
  rather than suggest.
- **And the question Teuken raised, answered in words:** `data/catalogue.toml`
  states a `quantisation` for every entry, and for Teuken the publisher ships
  no such artefact at all — every Q4_K_M of it is a stranger's requantisation.
  Either an entry names the artefact it means, or it does not claim a
  quantisation nobody can point at. Whichever is chosen goes in the file's own
  rules, because the next curator reads those and not this plan.
- **Constraint:** grades come only from runs actually made, and nothing in the
  prompt, the scoring or the runtime's wait moves to help a model. No weights
  on the image, no setup flow, and nothing in `crates/alo-shell`. **A model the
  box cannot hold is not a candidate for this task**: the three 7B entries wait
  on a machine with room — 16 GB to the runtime and more than four cores — and
  getting one is an owner's decision about hardware or about
  `C:\Users\SBW\.wslconfig`, which `docs/autonomy/SHARED_MAIN.md` puts behind
  an idle handoff from both loops. Trying harder on this box is what task 9
  already did.

**Done, 2026-09-11.** Two entries added and both measured here:
`qwen3-1.7b` (`qwen3:1.7b`, Q4_K_M, 1,359,279,776 bytes, Apache-2.0) and
`granite-3.2-2b-instruct` (`granite3.2:2b`, Q4_K_M, 1,545,296,256 bytes,
Apache-2.0), chosen because Qwen and IBM train them for tool calls and
constrained output and only then filtered to what 5,926 MB can load. Both
licences were read off the publishers' own Hugging Face metadata and both sizes
are the artefact's own manifest rather than a round number. **Both graded
`rarely`** — `qwen3:1.7b` drove 3 of 20 and `granite3.2:2b` drove 1 of 20 —
which is this task's finding: *the bar is not being missed for want of tool-call
training*, so task 10's blocker is a size question rather than a curation one.
Nothing in the prompt, the scoring, the runtime's context or the five-minute
wait moved, and no grade cleared the bar, so task 8's carry-or-fetch sentence
stands and only its table grew two rows.

**The Teuken question is answered in the file's own rules.** `quantisation` is
now paired with a new `artefact` field naming what the pinned runtime fetches at
it; `Catalogue::parse` refuses either half alone, so *a quantisation nobody can
point at* cannot be written down again. Ten entries gained the runtime tag they
were always measured or fetched by; `teuken-7b-instruct` and
`eurollm-9b-instruct` state no quantisation, because neither publisher ships a
GGUF and this catalogue has chosen no stranger's requantisation. Their
`download_bytes` is left where it was and is now a number with no artefact
behind it — recorded in rule 4 and written up as task 13 rather than silently
corrected. A third finding is in `docs/quirks.md` under *Pinned engines*:
Ollama 0.33.3 cannot pull `granite3.3:2b` at all, which is why the entry names
the 3.2 release. Measured in
`crates/alo-models/tests/candidates_the_box_can_hold.rs` and in
`crates/alo-models/src/catalogue.rs`. Report:
`docs/autonomy/updates/candidates-the-measuring-box-can-hold.md`. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 13
below is the next task and was written in the same change.

### 13. The two sizes rule 4 left without an artefact

**Status:** ready. **Depends on:** 12.

Task 12 made `quantisation` a claim an entry has to be able to point at, and two
entries could point at nothing: `teuken-7b-instruct` and `eurollm-9b-instruct`
name publishers who ship no GGUF, so both now state no quantisation. Their
`download_bytes` was not touched, and that is the loose end: 4.6 GB and 5.6 GB
are four-bit figures for artefacts neither entry any longer claims. Rule 2 of
`data/catalogue.toml` says a size is *what the disk and the card actually lose,
for the quantisation named* — and there is no longer a quantisation named, so
those two numbers now answer a question the entry does not ask. `min_vram_gb`
and `min_ram_gb` were set from the same four-bit assumption and have the same
problem.

It is a small task and deliberately separate: correcting a size changes the
carry-or-fetch table task 8 holds to the catalogue, and doing it inside the
measurement run would have mixed a curation fix into a grade.

- **Acceptance:** each of the two entries states a size, a video-memory figure
  and a system-memory figure that a reader can check against something that
  exists — the publisher's own release, or a named third-party artefact the
  catalogue chooses on purpose and records why; `docs/quirks.md`'s
  carry-or-fetch table is brought back into agreement in the same change, which
  `crates/alo-models/tests/the_carry_or_fetch_measurement.rs` will insist on
  rather than suggest; and whichever road is taken is written into the
  catalogue's own rules beside rule 4, because the next curator reads those.
- **Constraint:** no grade moves — neither entry has been measured and neither
  may be, and a size is not a measurement of driving. No weights on the image,
  no setup flow, and nothing in `crates/alo-shell`. If the honest answer is that
  a catalogue entry must name a third party's requantisation to be complete,
  that is a decision about what this catalogue vouches for and belongs in an
  ADR, handed over as this task in the shape ADR 0024 and ADR 0025 used.
