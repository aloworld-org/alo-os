# Journal

One entry per loop iteration, newest last. What was built, what the gate said,
and anything the next iteration should know before it starts.

The stop markers `LOOP COMPLETE` and `LOOP HALT` are read from this file by the
supervisor, so they appear only when they are true.

---

## 2026-09-02 — before the first iteration

*(Superseded by the entry below. Kept because the warnings in it still hold.)*

The queue is written and nothing has been taken from it yet.

What exists already, outside the loop: `crates/alo-models` — the catalogue with
its licence gate, the `ModelRuntime` contract, and the Ollama adapter. 22 tests,
nine of them driving the adapter against a real socket. `cargo clippy` reports
zero warnings and zero errors against the workspace deny list.

What the next iteration should know:

- **Item 1 (grants) is the foundation**, and items 2, 3, 4 and 6 all depend on
  its vocabulary. Getting the shape wrong is expensive; read
  `docs/decisions/0001-the-capability-model.md` in full before starting, not the
  summary in the queue.
- **The gate is real and it bites.** An earlier commit in this repository
  claimed clippy was clean when it was not — ten missing-doc warnings and a
  genuine `indexing may panic` error. Run the gate, read what it says, and
  report what it said rather than what was expected.
- **`indexing_slicing` is denied workspace-wide**, tests included. In a test
  module the honest fix is an `#[expect]` with a reason, because a panic on a
  bad index there is the failure being reported. In library code, it is a bug.
- **Do not couple a test to a dependency's formatting.** One test asserted on
  the exact JSON its HTTP client emitted and failed over a space. A test that
  fails over whitespace is a test that eventually gets silenced.


---

## 2026-09-02 — queue refreshed, still before the first iteration

The loop has still not run. Everything in `crates/alo-models` was built directly
in a session, and four decisions arrived after the queue was first written, so
the queue has been rewritten against what the code actually is.

**What changed that a first iteration must know:**

- **ADR 0007 — the CPU is the default.** A GPU is acceleration. The catalogue
  states what a model costs in system memory and how it behaves without a card,
  and `default_for_cpu` picks for an ordinary laptop. Anything later that assumes
  a GPU is wrong.
- **ADR 0008 — a question is answered in one of three places**, and the person is
  always told which. `source.rs` carries that; item 4 (the record) must record
  the source, and item 5 (egress policy) starts from `SourcePolicy` rather than
  from nothing.
- **The region is the customer's to name**, and alo OS ships no default that
  chooses a provider. Built in Europe, not only for Europe. An item that
  hardcodes a region or a provider is wrong however convenient it looks.
- **`provider.rs` sets the pattern for anything holding a credential:** a
  reference into the keyring, never the secret, with a test that renders the
  struct through `Debug` and serde and asserts nothing secret-shaped survives.

**One rule of the house, before it is broken:** the loop and a person must not
work in the same checkout at once. `CLAUDE.md` forbids it, and this session came
close — the loop was reported as running in `C:/dev/alo-os` while ten commits
were being made there. It was not running; the check that said so had matched its
own command line. Whoever starts the loop owns that checkout until it reports
`LOOP COMPLETE` or halts, and nobody else edits it meanwhile.

Ten ready items. Item 1 (grants) is first because items 2, 3, 4 and 6 all speak
its vocabulary.


---

## 2026-09-02 — iteration 1: grants

**Built: item 1.** `crates/alo-capability`, a new crate holding ADR 0001 as
working code — grants now, and items 2 to 5 beside them later. The name is not
`alo-agentd` on purpose: this is the portable logic a daemon will one day serve,
and calling it the daemon would invite the Linux half to be written into it.

Four files, one job each:

| | |
|---|---|
| `reach.rs` | `Reach` — what a grant covers; `Ask` — what a verb wants to touch. Facing each other, with no method turning one into the other |
| `grant.rs` | One grant, and everything that has to be true of it, checked where it is made rather than where it is used |
| `grants.rs` | The list: what is granted to whom until when, revoke, and the refusal in words |
| `path.rs` | Whether one path is inside another, component by component, touching no disk |

**The gate:** `cargo fmt --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, 72 tests passing
(28 new, 44 in `alo-models`). Built and unit tested on Windows; **no hardware
verification, and none claimed** — there is nothing here that touches a machine
yet.

Every promise the queue asked for is a test rather than a sentence: an expired
grant permits nothing (including at the exact moment it expires), a revoked one
stops on the next question, a path outside a grant is refused, and asking about
a path a hundred times leaves the list byte-identical.

**What the next iteration must know:**

- **Nothing in the crate reads the clock.** Every time-dependent question takes
  `now`. Item 3 (approvals) has the same shape of problem and should do the same
  — an approval that expires needs the same testability, and a second way of
  telling the time is a second thing to disagree about.
- **Containment is lexical.** `path.rs` decides without a syscall, which means a
  symbolic link inside a granted folder can point outside it. The rule is
  written in that file's docs: whatever executes a verb resolves the real path
  and asks about *that*. **Item 6 is where this becomes real, and getting it
  wrong there defeats item 1 entirely.**
- **Identities are matched exactly** — agent names, application ids, paths. This
  differs from `alo-models`, where provider names are compared
  case-insensitively, and the difference is deliberate: a name a person types is
  matched kindly, a name that decides reach is not. If item 2 adds verb names,
  they are identities.
- **A grant cannot be made without an end.** There is no "for ever" variant and
  zero is refused. If something later genuinely needs an indefinite grant, that
  is an ADR, not a new enum variant.
- **The errors are English in the source**, as in `alo-models`. Item 9 (strings)
  is where that gets fixed, and `GrantError` plus `Grants::refusal` are on its
  list — they are user-facing text, and `provider.rs` is not the only file
  holding some.
- **Storage is serde only.** `Grants` round-trips and handles keep counting from
  where they were, so a revoke from a stale list cannot land on a grant made
  since. *Where* the list is written belongs to the daemon and does not exist.

Nine ready items left. Item 2 (the verb registry) is next, and it speaks this
crate's vocabulary — read `reach.rs` before starting it, because `Ask` is what a
verb's arguments have to reduce to when the registry asks whether a call is
within its grant.


---

## 2026-09-02 — iteration 2: the verb registry

**Built: item 2.** Five files in `crates/alo-capability`, beside the grants
rather than in a crate of their own, because a verb that cannot be compared
against a grant is not a capability model.

| | |
|---|---|
| `arg.rs` | What an argument can be, what may arrive (`Given`), and what survives arriving (`Value`) |
| `sentence.rs` | The sentence a person approves: words from the verb, holes filled from validated values |
| `verb.rs` | One verb, and everything that has to be true of one — checked where it is declared |
| `verbs.rs` | The closed list, and the only way into it |
| `call.rs` | A validated call: what it would touch, what it would say, and whether the grants permit it |

**The gate:** `cargo fmt --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, 109 tests passing
(37 new, 28 in item 1, 44 in `alo-models`). Built and unit tested on Windows;
**no hardware verification, and none claimed** — nothing here touches a machine,
and the file verbs that will are item 6.

The guarantee `CLAUDE.md` names — *a verb cannot reach outside its grant* — is
now a test rather than a sentence, in `call.rs`: a well-formed call with a
generated sentence over an ungranted folder is refused, and half a move is
refused too.

**What the next iteration must know:**

- **Law 2 is carried by two things, and only one of them is a guarantee.** The
  guarantee is that `Takes` is closed and has no free-text kind, so nothing a
  model composes can arrive as an argument at all. The other is a word list that
  refuses `run_command`, `filter_expression`, `sql_query` at declaration time —
  a tripwire, documented as one in `verb.rs`. **Neither stops a verb's
  implementation from passing an argument to a shell**, and item 6 is where that
  becomes a real temptation. It is on review, and the module docs say so.
- **Being permitted and being approved are separate questions**, and `call.rs`
  answers only the first on purpose. Item 3 must not add "and approved" to
  `Call::permitted_by`: merging them is how "one approval, one execution"
  quietly becomes "one approval, whatever the grant allows". A `Call` is
  `Serialize` and not `Deserialize`, so an approval cannot be built from a call
  read back off a disk.
- **Every argument is required, and the sentence names every argument.** Both
  are refusals at declaration, both are now in the contract, and both exist for
  the same reason: the person approves the sentence, so anything it does not
  describe is something they did not agree to. An item that wants an optional
  argument wants two verbs.
- **The registry starts empty and no verb is declared anywhere yet.** That is
  item 6 (file verbs) and the adapters. Nothing shipped a default capability.
- **`Value` serialises and does not deserialise**, like `Call`. Item 4 (the
  record) can therefore write what ran without inventing a second type — and
  must not add `Deserialize` to either to read its own records back, because a
  value nothing validated would then exist. Read records into a type of the
  record's own.
- **`GrantError`, `Grants::refusal`, `ArgError`, `VerbError`, `CallError` and
  every `Sentence` are English in the source.** Item 9's list is now
  considerably longer, and `sentence.rs` was built as parts rather than a format
  string precisely so that translating it moves only `Part::Words`.

Eight ready items left. Item 3 (approvals) is next, and it speaks this
iteration's vocabulary: read `call.rs` before starting it, because an approval
is over exactly one `Call` and exactly its arguments.


---

## 2026-09-02 — iteration 3: approvals

**Built: item 3.** Four files in `crates/alo-capability`, turning ADR 0001 §5
into a journey a change has to make rather than a rule a daemon has to
remember. Each type can only be reached from the one before it.

| | |
|---|---|
| `proposal.rs` | The question put to a person: the sentence, who it would run as, and when it lapses. A read is refused here, and so is a change the grants already do not permit |
| `approvals.rs` | The list waiting to be answered — propose, approve, decline, sweep — and numbers that are never reused |
| `approval.rs` | One answer, worth exactly one execution, spent by redeeming it |
| `authorised.rs` | The only type in the crate that means may-run, and the two doors into it: a read, or a redeemed approval |

One test-support file came with them: `test_calls.rs`, under `cfg(test)`, so
that four files asking about the same journey do not drift into four journeys.

**The gate:** `cargo fmt --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, 135 tests passing
(24 new plus 2 doctests, 65 in items 1 and 2, 44 in `alo-models`). `cargo doc`
clean. Built and unit tested on Windows; **no hardware verification, and none
claimed** — nothing here touches a machine.

The guarantee `CLAUDE.md` names — *one approval causes exactly one execution* —
is now carried twice. The list refuses a proposal answered a second time, and
`Approved::redeem` takes `self`, so a second execution is not a program that
compiles: that half is a `compile_fail` doctest, with a twin that passes, so
the pair cannot quietly become a test that a typo fails to compile.

**What the next iteration must know:**

- **Item 4 (the record) is now mostly a matter of writing down what these types
  already carry.** `Authorised` answers *what ran, under whose authority, from
  which approval* (`call`, `under`, `from_approval`, `at`), and `Refused`
  carries the call it refused so a refusal is recordable rather than only
  countable. The one thing missing is *against which grant*: `Grants::permits`
  answers yes or no and does not say which grant said yes, and item 4 will need
  a `GrantId` back from it. That is a small addition to `grants.rs` and it
  should be made there rather than by re-deriving the answer in the record.
- **The grants are asked three times on the way to an execution** — when the
  call is checked, when the change is proposed, and last inside `Authorised`.
  The last one is the one that matters, and it is why a revoked grant stops
  something already approved. Nothing caches an answer, and nothing should
  start.
- **`Refused` boxes its `Call`.** Clippy's `result_large_err` was right: every
  authorisation returns it in the `Err`, and the happy path should not carry a
  hundred bytes it never reads. Anything later returning a call-shaped error
  should do the same.
- **Two doctests exist now, where the crate had none.** They are the public
  worked example of the journey, and `cargo test` runs them. If the shape of
  the API moves, they move with it.
- **Nothing is deserialised on the approval path.** `Proposal` and `Approvals`
  serialise so a pending question can be shown or written down, and neither
  reads back — an unanswered question does not survive a restart, which is the
  intended behaviour and not a limitation. Item 4 must read its records into a
  type of the record's own, as iteration 2 already said of `Call` and `Value`.
- **The new user-facing English is on item 9's list**: `ProposalError`,
  `AnswerError`, `NotAuthorised` and the `Refused` display. Every refusal in
  this iteration is a sentence somebody reads.

Seven ready items left. Item 4 (the record) is next.


---

## 2026-09-02 — iteration 4: the record

**Built: item 4.** `crates/alo-record`, ADR 0001 §7 as working code: every
execution and every refusal, kept so that "explain what it did" is a question
put to the record rather than a search through text.

| | |
|---|---|
| `line.rs` | Text as the record keeps it: one line, printable, bounded. Everything that enters the record as words comes through here |
| `written.rs` | One validated argument, written down — a mirror of `Value`, with an exhaustive `From` so a new kind of argument cannot be quietly lost |
| `what.rs` | What a call was: the verb, its arguments, and the sentence a person read |
| `happened.rs` | The four things that can happen — it ran, it was stopped, it never became a call, or a question was answered somewhere |
| `entry.rs` | One moment and what happened at it, with one constructor per point in ADR 0001's journey |
| `record.rs` | The list. Append, read back, and no way to take anything out |
| `explain.rs` | What can be asked: by agent, by span, by grant, by approval, refusals only, egress only |

One test-support file came with them: `test_calls.rs`, under `cfg(test)`.

**The gate:** `cargo fmt --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, 179 tests passing
plus 2 doctests (42 new in `alo-record`, 3 new in `alo-capability`, 1 new in
`alo-models`). `cargo doc --workspace --no-deps` clean. Built and unit tested on
Windows; **no hardware verification, and none claimed** — nothing here touches a
machine, and nothing writes to a disk yet either.

The guarantee `CLAUDE.md` names — *every execution and every refusal leaves a
record* — is a test in `record.rs`, and the three kinds of refusal are each a
test of their own in `entry.rs`.

**The one decision that went against a plan already written down.** Iterations 1
to 3 said the record would live in `alo-capability`, and its module
documentation said so. It does not, and the reason is worth the deviation:

- **the deciding crate must not deserialise.** `Call`, `Value`, `Proposal` and
  `Approvals` serialise and deliberately do not read back, because one read off
  a disk would be a call nothing had validated. A record exists to be read back.
  Housing both in one crate would mean the rule "nothing here deserialises" had
  an exception in the same crate as the thing it protects;
- **the record needs `InferenceSource`**, which lives in `alo-models` — and
  `alo-models` carries `ureq` and a TLS stack for the Ollama adapter. Depending
  on it from `alo-capability` would have put a TLS stack behind the crate whose
  entire value is being small enough for somebody else to audit. `alo-capability`
  still depends on nothing but `serde` and `thiserror`;
- **a crate that decides should not be reachable from one that observes**, so a
  future `Grants` cannot quietly start writing entries about itself.

**What the next iteration must know:**

- **`Grants::permitting` is the one search now**, answering with the `GrantId`
  that permitted something or the refusal in words. `permits` and `refusal` are
  thin wrappers over it, so there is no second search that could disagree with
  the one that decided. `Call::permitting` joins it across every ask and stops
  at the first refusal; `Authorised::against` carries the result. Item 6 gets
  this for free and should not re-derive it.
- **The record's four answers exist at exactly one moment**, inside
  `Authorised`, and `Entry::ran` copies them rather than working them out. A
  daemon that recorded from anywhere else would be recording a second opinion.
- **`Record` has no `forget`**, and that is the item 4a now in the queue: how
  long evidence is kept is one decision made in the open, and it belongs to
  whatever writes the record to a disk. Nothing writes it to a disk yet.
- **Two things are never recorded**, and both are tests rather than sentences:
  the question a person asked (`Happened::Answered` has no field for it), and
  the arguments of a call that never validated. Item 6 will be tempted by the
  second one when file verbs start being refused in quantity.
- **Everything the record shows goes through `Line`** — control characters out,
  collapsed, bounded. The refusal path is why: text from a call that never
  validated was written by whatever the model was persuaded to send, and a
  record read in a terminal must not be able to show one thing and say another.
- **`InferenceSource` now serialises**, which it did not before. It is an
  additive change to `alo-models`' public surface, with a round-trip test beside
  it. Item 5 (egress policy) will want the same of anything it adds.
- **The new user-facing English is on item 9's list.** Less than expected: the
  record deliberately composes no prose, because the explanation a person reads
  is the sentence they already approved. What is there is `Stopped`'s stored
  words, which come from `alo-capability`'s errors and are already on that list.

Seven ready items left, one of them new. Item 5 (egress policy) is next, and it
starts from `source.rs` in `alo-models` and from `Happened::Answered` here —
`Asking::only(Only::Egress)` already answers *what left this machine*, so item 5
is the decision and the indicator rather than the record of it.


---

## 2026-09-02 — iteration 5: egress policy and the indicator

**Built: item 5**, minus the record, which is cut into a new item 5a and
explained below. `crates/alo-egress`, law 1 as working code: nothing an agent
causes leaves this machine without being decided about and shown while it
happens.

| | |
|---|---|
| `destination.rs` | Where something is going — a paired machine, a named provider, or an address a verb named — and what may be shown as one |
| `leaving.rs` | One egress about to happen: who, where to, and why. `Why` is a closed list, and the sentence a person reads lives here and only here |
| `policy.rs` | What an organisation permits, the refusal in words, and `NotPermitted` carrying what it refused |
| `departing.rs` | The only type meaning may-leave, and it has no constructor of its own |
| `indicator.rs` | What is leaving right now: the list a person reads, and the one thing that makes a `Departing` |

**The gate:** `cargo fmt --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, 206 tests passing
plus 5 doctests (27 new unit tests and 3 new doctests in `alo-egress`, one of
them `compile_fail`). `cargo doc --workspace --no-deps` clean. Built and unit
tested on Windows; **no hardware verification, and none claimed** — nothing here
opens a socket, and enforcement at the network boundary is a Linux item.

**The design decision the item turns on.** The policy and the indicator are one
call rather than two. `Indicator::beginning` asks `EgressPolicy` and, if it
permits, puts the egress on the list before handing back the `Departing` a
caller must hold to open the connection. *Permitted but unshown* is therefore
not a state a program can be in, which is how the guarantee `CLAUDE.md` makes in
public — no agent-caused egress escapes the indicator — stops depending on
whoever writes the next verb remembering to fire an event. It is the same shape
as `Authorised`, and the file says plainly what it does not stop: code that
opens a socket without asking for a `Departing` at all.

**What was cut, and why it is a decision rather than a leftover.** The queue
item said "and the record of it". `alo-record` already keeps
`Happened::Answered`, which records where an answer came from *and* answers
`Only::Egress`. An answer from a provider is both a departure and a provenance,
so adding a `Happened::Left` beside it would make one departure two entries in
the single query law 1 promises — and the alternatives (make `Answered`
provenance only; or have one constructor take a `&Departing` and choose the
variant from `Why`) both change semantics that shipped in iteration 4. Choosing
in a hurry, at the end of an iteration, would have been the wrong way to make
that decision. Item 5a states both options and what the new entry has to
guarantee. `Departing` already carries everything such an entry needs, decided
once at the moment it was allowed.

**What the next iteration must know:**

- **The rule is stated once.** `EgressPolicy` is `From<&SourcePolicy>`, and
  `the_wider_boundary_agrees_with_the_inference_one_about_every_source` walks
  every policy against every source to prove they cannot disagree. Anything
  later that adds a policy variant adds it in both places or fails that test,
  which is the intent.
- **A question answered on this machine is not a departure.** `Leaving::asking`
  refuses `InferenceSource::ThisMachine` rather than returning an empty
  destination, so law 1's zero-egress claim is the absence of a type rather
  than a counter that reads zero. `DestinationError::NothingLeaves` says what to
  do instead.
- **Only a paired machine is in the building.** A host that answers on the same
  wire is outside it, and `Destination::Address` satisfies no region at all —
  guessing a region from a hostname is how a customer ends up in breach while
  looking at a reassuring label. ADR 0003 is the reasoning.
- **An address is validated before it can be displayed**, because the host came
  from a verb's argument. Same refusals as `alo_capability::Arg` — nothing
  blank, no control characters — plus a length bound, because the indicator is
  one line a person trusts.
- **`Leaving` serialises and does not deserialise**, like `Call` and `Proposal`.
  `Destination` does both, because a record will need to read one back. Item 5a
  inherits that split.
- **The new user-facing English is on item 9's list**: `DestinationError`,
  `EgressPolicy::refusal`, and `Leaving::describe` — which is deliberately the
  only place the indicator's sentence is composed, so there is one thing to
  translate rather than one phrase in the shell and another in the settings
  panel.

Seven ready items left, one of them new. Item 5a (the record of what left) is
next, and it is a decision to make before it is code to write; item 6 (file
verbs, the portable half) is the next one that starts from a settled model.

---

## 2026-09-02 — iteration 6: the record of what left

**Built: item 5a**, whole, including the refusal path the item did not name.
`alo-record` now answers law 1's second half — *and afterwards in a record* —
and it answers it with one entry per departure.

| | |
|---|---|
| `departed.rs` | **New.** The only door from an egress into the record: `Entry::left` from a `Departing`, `Entry::held_back` from a `NotPermitted`, and nothing else |
| `happened.rs` | `Answered` became `AnsweredHere` and lost its source; `Left` and `HeldBack` are new; `caused_egress` is now a variant rather than a calculation, and `why_stopped` answers across all three refusals |
| `entry.rs` | `answered` became `answered_here`, which takes no source because there is only one it could name |
| `explain.rs` | `Only::Egress` is departures; `Only::Refusals` now includes what the policy held back |
| `leaving.rs` (egress) | `Why` deserialises, additively — a record of what left has to say why |

**The gate:** `cargo fmt --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, **218 tests
passing plus 8 doctests** (12 new unit tests and 3 new doctests, two of them
`compile_fail`). `cargo doc --workspace --no-deps` clean. Built and unit tested
on Windows; **no hardware verification, and none claimed** — nothing here opens
a socket.

**The decision the item asked for, and why it is neither of the two it
offered.** The queue put two options: make `Answered` provenance beside a new
`Left`, or have one constructor choose the variant from `Why`. The second is
not writable — `Answered` held an `InferenceSource` and a `Departing` holds a
`Destination`, and there is no honest way back from an address a verb named to
an inference source. The first works but leaves the record able to contradict
itself: a daemon could write *the answer came from a provider* while the egress
query answered *nothing left*, because nothing tied the two together.

So `Answered` became `AnsweredHere` and stopped carrying a source at all. A
question answered somewhere else **is** the departure it caused, kept once as
`Happened::Left` with `Why::Asking`; a question answered here has nowhere to
name and no field to name it in. **Nothing is lost by it** — `Destination` says
everything `InferenceSource` said, the same three kinds under the same names,
and `alo-egress` already maps one to the other in one place. Two things are
gained, and both are the kind that stay true when somebody who has not read the
file writes the next daemon: whether an entry is egress is a **variant** rather
than something re-derived per reader, and an answer from somewhere else cannot
be recorded without a departure the indicator showed.

**What the item did not name and the gate did.** An egress the policy refused
had no way into the record. `CLAUDE.md` requires every refusal to leave one, and
`policy.rs` had already made `NotPermitted` carry what it refused *because* a
refusal is recorded. So `Entry::held_back` is here, it is a refusal and not a
departure, and the query proves both: a held-back egress answers `Only::Refusals`
and answers `Only::Egress` with nothing. Recording only what succeeded would
have been cutting depth, which is the one thing scope may never be cut into.

**What the next iteration must know:**

- **Two doors, both `pub(crate)` on the far side.** `Happened::Left` is
  reachable only from a `Departing` and `Happened::HeldBack` only from a
  `NotPermitted`, and neither has a public constructor in `alo-egress`. Two
  `compile_fail` doctests assert it. They were checked by unmarking them and
  reading the error — both fail with E0624 on the privacy of `new`, not on an
  argument count, so they are not tests of a typo.
- **`departed.rs` is a file rather than two more constructors in `entry.rs`,**
  and that is law 4 applied honestly: `entry.rs` changes when the capability
  journey does, this changes when what leaves the machine does. It uses an
  inherent `impl Entry` in a sibling module, so the API is still `Entry::left`
  and there is one `pub(crate) fn Entry::new` behind both files.
- **`Why` now deserialises and `Leaving` still does not.** Same split as
  `Destination`: the parts a record keeps read back, the decision does not.
  Anything later that adds a field to what the record keeps about an egress
  needs the same, and needs it to be a part rather than a decision.
- **`alo-record` depends on `alo-egress`, and `alo-models` is now a
  dev-dependency only.** The direction is the one item 4 established — the
  observer reaches the decider, never the reverse — and `alo-egress` still does
  not know `alo-record` exists.
- **The contract moved**, so `docs/contracts/agent-verbs.md` moved with it: an
  entry is one of six things, a departure is the only kind that counts as
  egress, and an adapter cannot write an egress entry from a destination it
  named itself.
- **Item 4a is still the daemon's**, and it now has one more thing to get right:
  whatever writes the record to a disk writes the departure at the moment it
  begins, from the `Departing` it is holding, rather than reconstructing one
  when the connection closes.
- **No new user-facing English**, which is worth saying: the record composes no
  prose, and the words in a `HeldBack` entry are the policy's own — already on
  item 9's list from iteration 5.

Six ready items left. Item 6 (file verbs, the portable half) is next, and it is
the first item since 1 that starts with nothing cut from it: the vocabulary it
needs — `Grants::permitting`, `Call::permitting`, `Authorised::against`, and now
the record's two doors — is all settled and none of it should be re-derived.


---

## 2026-09-02 — iteration 7: the file verbs, and the real path

**Built: item 6.** `crates/alo-files`, a **new crate**: the six file verbs
`docs/features.md` promises at v0.01, and the last question the grants have to
be asked before anything opens a file.

| | |
|---|---|
| `verbs.rs` | The six, declared: `list_folder`, `read_file`, `find_in_folder`, `rename_file`, `move_file`, `archive_folder` |
| `real.rs` | `Real` — the path this machine would really open — and why it cannot be made outside the crate |
| `resolving.rs` | `Resolving`, and `OnThisMachine`: the only thing in the crate that touches a disk |
| `touching.rs` | `Touching` — the only type meaning *this may touch the disk*, and the three questions asked to get one |

**The gate:** `cargo fmt --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, **246 tests
passing plus 11 doctests** (24 new unit tests, 3 new integration tests against
a real filesystem, 1 new in `alo-capability`, 3 new doctests one of which is
`compile_fail`). `cargo doc --workspace --no-deps` clean. Built and unit tested
on Windows, with the integration tests run against a real filesystem there;
**no hardware verification, and none claimed** — that is a certified machine,
and this was a developer's.

**The item existed for one sentence iteration 1 left behind**: containment is
lexical, so whatever executes a verb resolves the real path and asks about
*that*, and getting it wrong here defeats item 1 entirely. `Touching::of` takes
an `Authorised` — the end of ADR 0001 §5's journey, already permitted and
approved — and asks three questions of **every path the call names**:

1. do the grants permit it as it was written? If not, that is the refusal and
   nothing is looked for on the disk;
2. where does it really lead;
3. do the grants permit *that*?

The third is where a link out of a granted folder dies, and it dies as a
refusal by the grants rather than as an error of this crate's own. **The order
is the security property, not tidiness.** If the disk were touched first, a
refusal would tell an agent whether a file it may not reach exists, and the
capability model would have a side channel in it that no grant list could see.

**Three decisions that were not obvious, and are load-bearing.**

- **Every path a call names is asked about, not only the ones the verb declared
  its grant is over.** The contract lets a verb require a grant over some of its
  paths; the test that proves this uses a verb whose author forgot one, and
  asserts both that it is refused and that the disk was never asked about the
  forgotten path. The six file verbs require a grant over all of theirs, and a
  test asserts that too — a verb that needed the enforcement to save it would be
  a verb declared wrongly.
- **`Real` has no public constructor, which seals `Resolving`.** One
  implementation ships, so nothing anywhere can hand the grant check a "real"
  path it made up — the same reasoning that keeps the clock out of
  `alo-capability`. The trait exists at all for one reason that earns it: the
  escape this crate exists to stop needs a symbolic link, and making one on
  Windows needs a privilege a developer's account may not have. So the
  *decision* is tested against a filesystem written down in the test, on every
  platform, and `OnThisMachine` is tested against a real disk — with the link
  followed for real under `#[cfg(unix)]`. A test that quietly skips itself is a
  test that stops being run, so neither of these does.
- **`Refused::not_granted` is new in `alo-capability`**, and it is the one
  addition to a shipped crate this iteration made. The deciding crate cannot
  ask the third question — it touches no disk — so the answer comes back as the
  same type, and `Entry::refused` writes it down like every other refusal. It is
  public and safe to be: it grants nothing, and a refusal made in error stops
  something, which is the safe way to be wrong.

**What was cut, and it is written down rather than left.** The `std::fs` calls
are item 6a, ready and not blocked — the queue had the file half sitting under
*blocked — linux* beside the application half, and that was wrong: opening a
folder needs no portal and no accessibility tree. The item names the two things
`docs/quirks.md` now records, so they are not rediscovered.

**One decision about words.** "Archive" here means *make an archive*, not *move
it to the archive folder*. The second is `move_file` under another name, and a
closed list with two names for one action is a list a model picks from at
random.

**What the next iteration must know:**

- **A grant is made over a resolved path.** A person picking a folder grants the
  real one. This is not a nicety: on Windows a resolved path carries a `\\?\`
  prefix the typed one does not, and the two compare as different paths — a
  grant over the unresolved spelling would match nothing. It is in
  `docs/quirks.md`, in the contract, and in the integration test.
- **Two things no path check can do**, both now in `docs/quirks.md`: a hard link
  is a second real name for a file that also lives elsewhere and resolves to the
  granted name; and a path checked and then opened by name can change in
  between. The answer to the second is the acting half holding on to what it
  opened — `openat` from a directory handle — rather than resolving twice, and
  item 6a says so.
- **`alo-files` is the only crate that touches a disk**, and only in
  `resolving.rs`. Anything later that wants to read a file should reach for a
  `Touching`, not for a `PathBuf` — the paths inside one have been checked, and
  the path a call arrived with has not been checked *as the thing to open*.
- **The new user-facing English is on item 9's list**: `RealError`'s two
  messages, `Touching`'s "really leads to" refusal, and the six verbs' purposes,
  argument purposes and sentences. The sentences are the largest addition to
  that list since item 2, and they are the words a person approves.
- **There is no delete verb and no search expression**, and both absences are
  now stated in the contract so that adding either is a deliberate act rather
  than an oversight being corrected.

Six ready items left, one of them new. Item 6a (the acting half) is next and is
the natural continuation; item 7 (keyboard shortcuts) is the next one that
starts somewhere else entirely.


---

## 2026-09-02 — iteration 8: the file verbs, doing it

**Built: item 6a.** The `std::fs` calls behind the six, taking a `Touching`
rather than a path. Ten new files in `crates/alo-files` — `doing.rs`,
`answer.rs`, `failed.rs`, `named.rs`, `looking.rs`, `changing.rs`,
`archiving.rs`, `walking.rs`, `zip.rs`, `crc.rs` — and one test module,
`testing.rs`, which `resolving.rs` now shares rather than keeping its own copy
of the temporary-folder fixture.

**Gate:** `cargo fmt` clean, `cargo clippy --workspace --all-targets -D
warnings` clean, `cargo doc --workspace` clean. 64 unit tests in `alo-files`
(was 24), 13 integration tests against a real filesystem (was 3), 3 doctests,
and the rest of the workspace green and untouched. Documentation in the same
change — the contract, `docs/quirks.md`, and a `CHANGELOG.md` line.

**The item asked for two things, and the third is the one worth reading.**
It opens what `Touching` resolved and resolves nothing a second time, and a
read asks the *open handle* how big a file is rather than asking the name
again. The third was not in the item: **a change creates a path that nothing
had asked the grants about.** `rename_file` invents a name, `move_file` and
`archive_folder` invent a full path inside a folder — and a grant can be over a
single file, which is what the document offered at invocation is (ADR 0001 §4).
Under one of those, renaming would put a file at a name nobody granted. So
`Did::of` asks the grants once more, at the authorisation's own moment, before
anything is touched, and a no comes back as a `Refused` in the grants' own
words. *A grant covers where a file goes, not only where it comes from* — the
contract now says so beside the three questions item 6 added.

**Four decisions that were not obvious, and are load-bearing.**

- **A refusal by the grants and a refusal by the machine are different types.**
  `Did::of` answers `Err(Refused)` when the capability model said no, and a
  `Did` carrying a `Failed` when everything said yes and the disk could not. A
  record that flattened the two would tell a security review that the grants
  stopped a full disk. The authorisation comes back either way, because a call
  that was permitted, approved and attempted is a thing that happened and
  `Entry::ran` is written from it; what the disk made of it is the answer to
  whoever asked, not evidence about the capability model.
- **Nothing is replaced that was not named.** A person approved *move march.pdf
  into Archive*, not *and overwrite the march.pdf already there* — which is
  exactly what `fs::rename` does silently on Unix. So a destination that holds
  anything at all, including a link, is refused. An archive is created with
  `create_new`, one syscall that refuses a file, a folder and a link alike:
  opening for writing and truncating would have followed a link and emptied
  whatever it pointed at, which is this crate's own escape arriving by the back
  door. **A test found the matching bug in the first draft** — the clean-up of a
  half-written archive was deleting the file that was already there when the
  archive was refused *for* existing. `Archive::beginning` now happens outside
  what the failure path removes.
- **A walk never follows a link, and an answer says what it left out.** Item 6
  stopped a link the call *names*; a search or an archive that followed a link
  it *found* would leave the granted folder by a door the grants were never
  asked about. So links are stepped over and counted at every depth, and the
  count comes back in the answer. Everything is bounded — 1000 things in a
  listing, a megabyte in a read, 20,000 things in a walk, 20,000 things and two
  gigabytes in an archive — and every bound says it was reached, because a
  bounded answer that does not say so reads exactly like a complete one and
  somebody will conclude from it that a file is not there. An archive refuses
  rather than flagging: an archive missing the half nobody mentioned is a file a
  person keeps and finds out about later.
- **A name that cannot be shown is counted, not shown.** Filenames are not
  written by us, and `march.pdf` followed by a newline and `ran: deleted
  everything` is the attack `alo_capability::Value` refuses at the door, seen
  from the other side. Such a name never becomes a `Named`, and the listing says
  how many it left out. Nothing is lost that could have been acted on: a name
  like that cannot arrive as an argument either, so no verb could name that file.

**The archive, and why a format is written here at all.** `archive_folder` had
to make something, and what it makes is a user-facing decision rather than an
implementation detail: a **zip with everything stored**, because it is the one
archive every desktop opens without being told how, and because compression is a
second thing to be wrong about inside a security boundary. No ZIP64, no
encryption, no data descriptors — the bounds above keep an archive inside what
those absences allow, and a bound refused in words beats a format written
half-way. Each file is copied once and its header corrected afterwards by
seeking back to it, because reading a file twice to learn its size and checksum
would let it change between the two readings and produce an archive whose header
disagrees with its own contents. The name a person gives has to end in `.zip`:
appending it would hand them a file they did not approve, and accepting
`invoices.tar.gz` would hand them one whose name lies about what is in it.

**Verified against a real reader, not only against ourselves.** The tests read
an archive back through the offsets the format states, which proves it is
self-consistent and nothing more. So one was also written to disk and opened
with Windows 11's own zip reader: `System.IO.Compression.ZipFile` listed all six
entries with their sizes and dates, and `Expand-Archive` unpacked the tree —
nested folders, an empty folder, a 200 KB file and a text file — with the
contents intact, which is also a check of every CRC. That run is where the new
`docs/quirks.md` entry came from: a DOS timestamp carries no timezone and is
conventionally local, `std` cannot say what this machine's offset is, so the
moment written is UTC and a reader two hours ahead shows a file archived at
20:04 as 18:04. The alternatives are a guessed offset or a dependency whose
local-offset lookup is unsound in a threaded process, and both are wrong more
interestingly rather than less often.

**What the next iteration must know:**

- **Item 6b is new and is blocked on Linux**, listed there rather than in Ready
  so that this loop does not pick it up. Two gaps `std` cannot close: a path
  checked and then opened by name can have a link swapped in between the two,
  and `fs::rename` has no portable no-clobber form, so a destination is checked
  for and then renamed onto. The Linux answers are `openat` with `O_NOFOLLOW`
  from a directory handle and `renameat2` with `RENAME_NOREPLACE`; the workspace
  forbids `unsafe`, so the item's first decision is a pinned dependency or an
  ADR. It replaces syscalls under settled decisions; it is not a rewrite.
- **`Did::of` takes the grants**, which is the shape anything else that acts on
  a `Touching` should copy. The grants are now asked at four points: when the
  call is made, when it is proposed, at the moment of execution about where the
  named paths really lead, and at the moment of execution about what would be
  created.
- **A `Failed` is not a `Refused`, and `alo-record` has no variant for one.**
  Whoever writes the daemon records `Entry::ran` from the authorisation
  `Did::into_parts` hands back, whether or not the disk cooperated, and answers
  the agent with the `Failed`. If that ever stops being right it is a change to
  `Happened` and to the contract's "an entry is one of six things", not a quiet
  reinterpretation of `Ran`.
- **The new user-facing English is on item 9's list**, which the queue now says
  outright: every `Failed` message, `RealError`'s pair, `Touching`'s refusal,
  and the six verbs' purposes, argument purposes and sentences.
- **`find_in_folder` matches a name, case-insensitively, and interprets
  nothing.** There is still no expression, no wildcard, and no delete verb.

Five ready items left. Item 7 (keyboard shortcuts) is next, and it is the first
one since item 1 that starts somewhere the file verbs do not reach.

---

## 2026-09-02 — iteration 9: keyboard shortcuts

**Built: item 7.** `crates/alo-shortcuts`, a new crate of eight files:
`modifier.rs`, `key.rs`, `chord.rs`, `action.rs`, `defaults.rs`, `changes.rs`,
`shortcuts.rs`, `clash.rs`. It depends on nothing else in the workspace — a
shortcut is a person pressing a key on their own machine, so there is no verb,
no grant and nothing to propose, and `lib.rs` says that outright so nobody
later wonders where the capability check went.

**Gate:** `cargo fmt --check` clean, `cargo clippy --workspace --all-targets -D
warnings` clean, `cargo doc --workspace` clean. 41 unit tests and 1 doctest in
the new crate; the workspace is 324 unit tests, 13 integration tests against a
real filesystem and 12 doctests, all green and the rest untouched. A
`CHANGELOG.md` line and the queue in the same change.

**The item's sentence was right and incomplete, which is the thing to read.**
"The model must express a conflict rather than letting the last binding win"
reads like one refusal, and refusing at bind time is the easy half. The half
that shaped the crate: **a release can add a default onto a chord a person
already moved something else onto.** No refusal at the moment of binding could
have caught it, because the binding was made before the default existed. So a
clash is a thing the model holds and reports, not only one it prevents, and the
resolution had to be decided rather than left to whichever list happened to be
searched first:

1. At the moment of binding, a clash is **refused**, and the refusal names what
   already has the chord. Nothing changes.
2. **A person's binding beats one we shipped.** Their chord still fires, our new
   default does not, and `clashes()` says so where Settings can show it.
   Dropping the binding they chose in favour of the one we chose would be the
   wrong way round on a machine sold on control.
3. **Two of their own on one chord fire nothing.** Only an unvalidated file can
   produce it, and picking one would be right half the time — with the wrong
   half closing a window somebody meant to maximise.

**Three decisions that were not in the item.**

- **Only the difference is written down.** The defaults live in the code and
  `Changes` — the person's moves and clearings — is the whole settings file.
  That is what makes a default improvable: a better one reaches every machine
  that never touched it and no machine that did. It is also the direct cause of
  rule 2 above, and the two belong together. `Changed` holds `Option<Chord>`
  because *I want no shortcut for this* is a decision, and a file that could not
  say it would hand the default back at the next sign-in.
- **A promise in `docs/features.md` is a refusal here.** `Ctrl+C`, `Ctrl+X` and
  `Ctrl+V` cannot be taken by a system shortcut, because copy and paste across
  applications is a v0.01 promise and a system shortcut is a key taken away from
  every application at once. Exactly those three chords, not those three keys —
  `Ctrl+Shift+V` is nobody's clipboard. The same reasoning keeps `Action` short:
  every action costs the whole machine a chord, so one arrives with the feature
  that needs it, and dividing the screen from the keyboard is v0.5 and is not
  there.
- **A key is the one printed on the person's own keyboard**, not a position on
  an American one, so `Super+Q` on a French keyboard is the key marked Q. What
  that leaves for the compositor is written in `key.rs` rather than discovered
  later: a layout that prints no Latin letters has no key marked Q at all, and
  matching the chord against the person's Latin layout is a lookup that belongs
  where the keyboard is read.

Two smaller ones worth keeping. **Shift is a modifier that does not modify** —
`Shift+2` is `@` — so every chord holds Super, Ctrl or Alt, and a bare key is
not something this model can express. And **the shipped defaults are built by
the compiler**, through a `pub(crate)` constructor that checks nothing, so a
test puts every one of them back through `Chord::checked`: the list we ship is
held to the rules a person is held to, or the rules are advice.

**What the next iteration must know:**

- **Item 8 (appearance) is next, and it should copy `changes.rs`.** Background,
  lock-screen image, light and dark with a schedule, accent colour, text
  scaling: same shape — what we ship lives in the code, what the person chose is
  the file. It wants a crate of its own rather than joining this one; shortcuts
  and wallpaper have no reason to change together.
- **Item 4a is in *Ready* but is not this loop's.** It is where the record is
  written and what prunes it, which is `alo-agentd`'s, and the daemon does not
  exist. Item 8, 9 and 10 are the ready work; whoever starts the daemon should
  move 4a out of Ready or start it there.
- **Item 9's list grew**, and the queue now says how: `Action::purpose`,
  `Key::label`, `Modifier::label`, the three `ChordError` sentences, and what
  `Taken` and `Clash` say. Two of them need a translator's judgement rather than
  their typing — a key is labelled with what is printed on it, and the Super key
  is not called Super on most of the keyboards this will run on.
- **Nothing here has been pressed.** The model is built and unit tested; no key
  has reached it, because the compositor does not exist. Law 3's "on real
  hardware" is owed, and `ROADMAP.md`'s "keyboard shortcuts a person can change"
  stays unticked until a key on the certified machine does something.

---

## 2026-09-02 — iteration 10: making it yours

**Built: item 8, minus one part that was cut on purpose.**
`crates/alo-appearance`, a new crate of thirteen files: `colour.rs`, `token.rs`,
`picture.rs`, `rotating.rs`, `background.rs`, `display.rs`, `time.rs`,
`scheme.rs`, `text.rs`, `lock.rs`, `shipped.rs`, `changes.rs`, `appearance.rs`.
Like `alo-shortcuts` it depends on nothing else in the workspace, and `lib.rs`
says why outright: a person choosing their own wallpaper in Settings is not an
agent doing something to their machine, so there is no verb, no grant and
nothing to propose. `docs/features.md` promises at v1 that an agent can be
*asked* for an appearance change; that arrives as a verb in `alo-capability`
proposing one of the values this crate defines, and nothing here has to move
for it.

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -D warnings` clean, `cargo doc --workspace` clean. 58 unit tests
and 1 doctest in the new crate; the workspace is 382 unit tests, 13 integration
tests against a real filesystem and 13 doctests, all green and the rest
untouched. A `CHANGELOG.md` line and the queue in the same change.

**What was cut, and why it is a cut rather than a gap.** The item asked for an
"accent colour drawn from the design tokens". Two documents disagree about what
that means and neither is wrong: `docs/features.md` wants an accent a person
picks that the whole shell follows, and `docs/design/figma-brief.md` says
terracotta is the agent's colour, spent nowhere else, about five percent of any
screen. An accent a person could set to terracotta would take away the one
signal that says the machine is acting on their behalf — and the other five
tokens are structure and grounds rather than accents, so navy is unreadable
against the charcoal rail and cream against the cream ground and there is
nothing left in the palette to offer. Resolving it means an accent set with a
light and a dark value per hue, which is a designer's decision. So it is **item
8a**, written into the queue with the whole argument, and `lib.rs` says in its
own words what this crate does not answer yet. LOOP.md says cut scope and never
depth; this is that, and the loop did not invent a palette to avoid writing the
sentence.

**The item's own words were wrong about one thing, and it matters.** "Background
per display" is the obvious model and it fails the first time somebody plugs in
a projector: the projector is a display nobody has set anything on, so a machine
whose owner chose a photograph would show a room full of strangers the wallpaper
*we* chose. So there is **one background — the person's — and a display they
singled out is an exception they made on purpose.** A display renamed by a
driver update loses its exception and falls back to their choice, which is the
right way round to fail.

**And the decision that was not in the item at all.** The desktop is seen by
whoever is signed in. The lock screen is seen by whoever walks past. A person
who pointed their background at a folder of their own photographs picked the
*folder*; they did not pick, one by one, the pictures a machine left alone in a
room shows to a corridor. So a lock screen that *follows* the desktop does not
follow a rotating one — it shows the shipped wallpaper while the folder rotates,
and `lock_is_holding_back` says so where Settings can put it in a line under the
switch, rather than leaving somebody to notice. Nothing is taken away:
`Lock::Its` takes any background including a rotating folder, so a person who
says they want their photographs there gets them. The rule only decides what
*following* means, which is the case where nobody said anything.

**Three smaller ones worth keeping.**

- **Nothing reads the clock or the disk**, which is item 1's rule reaching a
  fourth crate. A schedule is answered at a time of day that is passed in, and a
  rotating folder is asked *how many pictures it holds* and *how long it has
  been running* rather than going to look — so `showing()` is a position, and
  which file is at that position is decided where the folder is read. Both
  answers are testable without a disk and without waiting.
- **A wallpaper alo OS shipped is named; a person's picture is a path.** Not
  tidiness: a name that were allowed to be a path would be a path chosen by
  whoever wrote the settings file, pointed anywhere on the disk, wearing the
  image's clothes. `Picture::shipped` refuses anything with a separator or a
  component that is not an ordinary name, and the test walks `../../etc/shadow`
  and four others past it.
- **A promise in a standard is a test.** EN 301 549 — what an EU public-sector
  desktop is procured against — requires text to resize to 200%, so `text.rs`
  asserts its ceiling is at or above 200 rather than commenting that it ought to
  be. The ceiling shipped is 300%, because a person who needs 200% is not always
  a person who needs exactly 200%.

**What the next iteration must know:**

- **Item 9 (strings) is next**, and its list grew again — the queue now names
  the third one. `alo-appearance` adds `Token::name` and eight error types'
  sentences. `Token::name` needs a translator's judgement rather than their
  typing: several languages have no ordinary word for terracotta and the one
  reached for may not be the colour. Two things in that crate are deliberately
  *not* on the list: a time of day is written `18:00` in the file whatever the
  region does, and how a person is shown a time is the region's business.
- **The image now owes a wallpaper named `alo`** (`shipped::THE_WALLPAPER`),
  written into the image item under *blocked — linux*. An image without it boots
  to nothing behind the windows. That is deliberately not papered over with a
  colour nobody chose: a missing shipped wallpaper is the image's bug and should
  look like one.
- **Item 8a is a decision before it is code**, and it is not the loop's to make.
  Whoever answers it should read `docs/design/figma-brief.md`'s five principles
  first, because principle 1 is the constraint the answer has to survive.
- **Item 4a is still in *Ready* and is still not this loop's** — it is where the
  record is written and what prunes it, which is `alo-agentd`'s, and the daemon
  does not exist. Items 9 and 10 are the ready work.
- **Nothing here has been drawn.** The model is built and unit tested; no pixel
  has reached a screen, because the compositor does not exist. Law 3's "on real
  hardware" is owed, and `ROADMAP.md`'s "Making it yours" stays unticked until a
  person changes their background on the certified machine and sees it change.

## 2026-09-02 — iteration 11: the strings, and the end of silent English

**Built: item 9.** `crates/alo-strings`, a new crate of eleven files:
`key.rs`, `template.rs`, `filling.rs`, `phrase.rs`, `vocabulary.rs`,
`translation.rs`, `speaking.rs`, `language.rs`, `union.rs`, `said.rs`,
`strings.rs`. Like `alo-shortcuts` and `alo-appearance` it depends on nothing
else in the workspace, and for the same reason: a person reading their own
machine in their own language is not an agent doing something.

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -D warnings` clean, `cargo doc --workspace --no-deps` clean. 75
unit tests, 5 integration tests, 3 doctests and one `compile_fail` in the new
crate; the workspace is **475 tests and 17 doctests**, all green and the rest
untouched. A `CHANGELOG.md` line and the queue in the same change. The
`compile_fail` was checked to fail on the privacy — E0451, *fields `language`
and `texts` of struct `Speaking` are private* — and not on a typo, and it has a
passing twin.

**The decision the item asked for, and the answer that was bigger than the
question.** The item asked for "a test that a missing translation is visible in
development rather than silently English". The obvious build of that is a
development flag that wraps untranslated strings, and it is wrong: it answers
with a bare `String` the rest of the time, so *shown English because nobody
translated it* would be invisible in exactly the build a person in Latvia is
running, and the first person to find out would be them. So the marking is
there — `Showing::InDevelopment`, guillemets, and the test the item asked for by
name — but it is one of three ways of noticing rather than the only one. Every
answer is a `Said` carrying `CameFrom`: a translation and which language it
really is, the source, or a key nothing declares. `Strings::unanswered` lists
what a release note has to count. English is now impossible to show without the
system knowing it did.

**The decision that was not in the item at all.** A translation is written by
somebody whose language nobody on this team reads, so nobody here can see that
a sentence has gone wrong by reading it. A translator who moved `{bytes}` out of
*"{path} holds {bytes} bytes and a verb reads at most {most}"* would leave a
person told their file is too big and not told how big, in their own language,
with nothing anywhere saying so. So `Vocabulary::check` is the only door to a
showable translation — `Speaking` has no public constructor and no
`Deserialize`, which is `Departing` and `Touching` again — and it matches every
gap against the source's, both ways. What is refused: a dropped gap, an invented
one, a sentence that is not one, and a key nothing says any more. What is
deliberately **not** refused: a partial file. A language arrives a few hundred
strings at a time and a check that insisted on completeness would mean nobody
ever saw the first half of anybody's work. And everything wrong comes back at
once, because being told about the next mistake each time you try again is how a
translator gives up.

**Three smaller ones worth keeping.**

- **English is a source, not a default.** The sentence lives beside the key in
  the code, which is item 7's *only the difference is stored* reaching a fifth
  crate: the defaults are in the code and the file holds what somebody changed,
  and here the change is a translation. That is not the hardcoded English
  `CLAUDE.md` forbids — hardcoded English is English that reaches a screen
  without anything having asked whether a translation exists — and the
  difference is carried by `Said` rather than by a convention. It also means a
  release can improve an English sentence for every machine that has no
  translation of it, and that a phrase cannot exist without English at all.
- **A person names their own second language.** The chain is what they said,
  each with its broader forms (`pt-BR` brings `pt`), and nothing else. Somebody
  in Latvia who also reads Russian says so and meets Russian before English;
  nothing infers it for them, because *you are Latvian so you must read Russian*
  is not a thing software gets to decide.
- **A language is named in its own language, and gaps are named and never
  numbered.** `union.rs` holds the 24 with their endonyms, because a picker that
  said *Greek* is one the people it exists for cannot read; and `{}` is refused
  outright, because a translator reordering a sentence — German puts the size
  before the name — has to have something to reorder by.

**What the next iteration must know:**

- **The strings themselves have not moved.** `alo-files`, `alo-shortcuts` and
  `alo-appearance` still hold their English in their own error types and labels.
  That is items **9b, 9c and 9d**, one crate each, because a half-moved crate
  reads exactly like a finished one. The scaffolding was not built blind: the
  integration test carries the awkward real strings from all three — `TooBig`
  with its three gaps, `Token::name`, `Modifier::Super` — through the whole path
  verbatim, and the note `Token::name` needs is drafted there for 9d to take.
- **Plurals are item 9a and are a real gap, not a tidy-up.** `Failed::TooBig`
  says "{bytes} bytes" and is wrong in English for a one-byte file today; Polish
  has three forms, Irish five, Latvian one for zero. The CLDR rules are a table,
  and a table written from memory and shipped as a tested promise is what the
  gate exists to stop — so 9a starts with the rules in front of it. 9b is
  blocked behind it for that one message and says so.
- **Nothing here has been shown to anybody.** No screen has rendered one of
  these strings, because the compositor does not exist. `ROADMAP.md`'s
  "Language" stays unticked, and it should: it promises the shell in 24
  languages, and what exists is the machinery plus zero translations.
- **Item 10 is the last ready item**, and after it every remaining item in the
  queue is 9a–9d, 4a and 8a — of which 4a is `alo-agentd`'s and 8a is a
  designer's decision. The loop is close to having only its own cuts left.

---

## 2026-09-02 — iteration 12: halted, because the track it was given does not exist

The supervisor asked for one iteration of the loop for track **`business`**.
Nothing was built, and nothing in the queue was touched.

**Why.** This loop has no tracks. `LOOP.md` describes one loop over one queue,
and `QUEUE.md` is that queue: the capability model, the record, egress, the file
verbs, shortcuts, appearance and strings, every item naming an ADR, a contract
or a line of `docs/features.md` it implements. There is no business queue in
this repository, no business item in the one queue there is, and no section of
`docs/features.md` or `ROADMAP.md` that such an item could name. The word
appears in this repository only in `catalogue.rs` and ADR 0007, where it means
*commercial use of a model licence* — a property of a model, not a body of work.

**Why that is a halt rather than something to work around.** Building a business
track would have meant the loop inventing scope, and `CLAUDE.md` gates scope on
`docs/features.md` with a tier, inside the current release. It would also have
meant the loop deciding what this company's commercial work is — pricing,
positioning, who the first customer is — which is the clearest example there is
of *a decision that is not ours*. `LOOP.md` names both of those as halting
conditions, and halting with a clear reason is worth more than an iteration that
invents a way past a question nobody has looked at.

**What the supervisor should do with this.** One of three things, and the loop
cannot choose between them:

- **The track name was a mistake**, and the intended run is the ordinary loop.
  Re-run without a track. The first ready item is **10 — test a provider before
  saving it**; after it, only 9a–9d, 4a and 8a remain, and of those 4a is
  `alo-agentd`'s and 8a needs a designer.
- **A business track is genuinely wanted.** Then it needs its own queue file,
  written by a person, with items that name what they implement the way this
  queue's items do — and probably its own `LOOP.md` section saying what a gate
  means for work that has no `cargo clippy`. A build loop whose gate is tests
  cannot tick a pricing decision as done.
- **The business work belongs elsewhere.** Most of what that phrase usually
  covers is not this repository's at all — this repository is the shell, the
  daemon and the image. `LOOP.md` forbids the loop touching another one.

**The gate.** Not run, because nothing changed but this journal entry and no
Rust was touched. The workspace is as iteration 11 left it: 218-plus tests
passing, clippy clean against the deny list.

LOOP HALT
