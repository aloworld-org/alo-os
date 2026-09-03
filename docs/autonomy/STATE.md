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

`LOOP HALT` — raised at iteration 12, cleared below.


---

## 2026-09-02 — the halt is resolved: there was no business track

The halt was correct and the cause was outside this repository: the supervisor
was started without a track, so it defaulted to `business` and asked for a track
this repository does not have. Nothing was wrong with the queue, and nothing had
been invented.

`LOOP.md` now says plainly that there are no tracks here and that the word in
the prompt is noise. The marker above is cleared.

Where the work stands: **eleven items done, eight crates, 492 tests passing,
clippy clean.** The next ready item is **10 — test a provider before saving it**.
After that only 9a–9d, 4a and 8a remain, and of those 4a belongs to
`alo-agentd` and 8a needs a designer.


---

## 2026-09-02 — iteration 13: testing a provider before it is saved

**Built: item 10**, whole, including two refusal paths the item did not name.
Three files in `crates/alo-models` — `secret.rs`, `tried.rs`, `trying.rs` — and
one test-support file, `testing.rs`, which `ollama.rs` now shares rather than
keeping its own copy of the stub server. No new dependency: ureq was already
here, as the item said.

| | |
|---|---|
| `secret.rs` | A key as it was just typed, for the length of one call. No accessor, no `Display`, no `Serialize`, no `Clone`, a hand-written `Debug` |
| `tried.rs` | What a person is told: `Tried` when it worked, `NotTried` when it did not — both shapes of one answer, read in one dialogue |
| `trying.rs` | The only file that knows what a provider's API looks like: the policy question, the request, and what each answer means |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. `alo-models` is 70 unit tests (was 44) and 2
doctests (was 0); the workspace is **500 tests and 19 doctests**, all green and
the rest untouched. `CHANGELOG.md`, `docs/quirks.md` and the queue in the same
change. The `compile_fail` was checked by unmarking it and reading the error —
E0624, *method `bearer` is private* — so it is not a test of a typo, and it has
a passing twin.

**The item is one sentence, and the design is the three things that sentence
does not say.** *A mistyped key should be found when it is typed* describes a
request; what it leaves open is what else that request is allowed to be.

- **The policy is asked first, and there is no way to skip it.** `Trying` has
  one method and it takes a `SourcePolicy`, so testing is not a second road to
  a provider that the machine's policy forbids. A refused test opens no
  connection — the test that proves it points at an address nothing is
  listening on, so a leak would come back `Unreachable` and say so — and the
  refusal is `SourcePolicy::refusal`'s own words rather than a second
  explanation that could drift from the first.
- **A redirect is refused, never followed.** The address the policy answered
  about is the address that gets reached; following one would open a connection
  to a host nobody decided about, carrying a credential. ureq would not have
  forwarded the header (`redirect_auth_headers` is `Never` by default) — but
  *the key would have been safe* is not the objection. The connection is.
- **Nothing of the person's leaves.** The test is a `GET` with no body: no
  question, no document, no sample prompt to "check the model answers". What
  leaves is that somebody with this key asked this provider what it offers. The
  test asserts it on what actually went out on the socket, not on the intent.

**The two refusals the item did not name, and why they are not tidy-ups.**
*Given no key and refused* is a different sentence from *given a key and
refused*: telling somebody their key was rejected when they never typed one is
how they spend an hour checking a key that does not exist, so 401 with a key is
`KeyNotAccepted` and 401 without one is `NeedsAKey`. And *something answered,
but not like a provider* is neither — it is the address of the website instead
of the address of the API, and reporting it as a bad key sends somebody to
change the one thing that was right.

**Three smaller decisions worth keeping.**

- **A key goes into this crate and does not come out.** `Secret::bearer` is
  `pub(crate)` and is the only reader that exists. What the file says outright
  is what it does *not* claim: the bytes are not scrubbed on drop, because doing
  that honestly needs `unsafe` or a dependency and the workspace forbids the
  first — and a promise made and not kept is worse than one not made. What is
  kept is narrow and real: never written down, never rendered, never sent
  anywhere but to the provider it belongs to. A pasted key's surrounding
  whitespace is dropped (that *is* the mistyped key), and a control character
  in one is refused rather than quietly removed — a credential silently altered
  is an afternoon lost to a key that was right.
- **Names a provider wrote are held to `alo-files`' rule.** They arrive from
  somebody else's service and land in a settings panel beside things the system
  said itself, so a name that cannot be shown is counted and left out, a list
  longer than anybody reads is cut, and the answer says both happened. A
  bounded answer that does not say so reads exactly like a complete one.
- **No sentence in this item counts anything out loud.** *One model* and *two
  models* is one sentence in English and three in Polish, and item 9a is where
  the CLDR rules get read rather than remembered. So `Tried::describe` carries
  no number, and the numbers are accessors for whatever shows them later. This
  is the first place the loop has designed *around* a known gap rather than
  adding to it.

**What the next iteration must know:**

- **Item 9e is new in the queue.** No 9-series item named `alo-capability` or
  `alo-models`, and this item added `SecretError`, `NotTried` and
  `Tried::describe` to the second — so the gap is written down rather than
  discovered when somebody thinks the strings work is finished. It should follow
  9b–9d, not lead them.
- **This egress is deliberately not on the indicator, and the reasoning is in
  `trying.rs` where somebody will argue with it.** A person pressing *Test* on
  the screen they typed the address into is not an agent doing something; the
  indicator answers *what is my machine sending that I did not ask for*. Any
  egress an **agent** causes — a question put to this provider once it is saved
  — goes through `alo-egress` and is shown, as it does today. If that reasoning
  is ever rejected, the fix is not a comment: it is `alo-egress` gaining a way
  to say *the person did this*, and `alo-models` cannot depend on `alo-egress`
  because `alo-egress` depends on it.
- **Nothing here has met a real provider.** The tests drive a stub on a real
  socket, which is what `ollama.rs` does and is honest about what it proves.
  Testing against a service somebody pays for is owed with the rest of the
  hardware verification, and `docs/quirks.md`'s new entry says the same about
  the `/v1` convention it depends on: documented, not observed.
- **`ROADMAP.md` was not ticked and has no line to tick.** *Test a provider
  before saving it* is a v0.5 promise in `docs/features.md` that the roadmap
  covers only inside "Settings, as one place", and there is no Settings panel
  to press the button in. The model is built; the screen is the compositor's.

**Item 10 was the last numbered item, and what is left is the loop's own cuts —
with one of them unblocked while this iteration was running.** A person answered
item 8a in `ADR 0010` and pushed it: terracotta is reserved for the agent, five
designed hues are offered instead with a value for a light ground and one for a
dark, and the queue item is rewritten as work rather than as a question. That
commit is docs and a decision, no code, and this iteration was rebased onto it —
the only conflict was two `CHANGELOG.md` entries wanting the same line, and both
are kept.

So **8a is the next ready item, and it is now buildable here**: `Token` gains
the set and refuses terracotta as a personal accent, which is a refusal test
before it is a colour table. The part of ADR 0010 saying the agent is never
signalled by colour alone is the shell's and not `alo-appearance`'s — note it,
do not build it there. After 8a: 9a (the CLDR plural rules, which must be read
and not recalled — the loop should say so plainly if it cannot get them),
9b–9e (each crate's English onto `alo-strings`, which 9a partly blocks), and 4a,
which stays the daemon's because `alo-agentd` does not exist.

**And the rule that nearly broke.** `CLAUDE.md` says one agent per working tree
and that whoever starts the loop owns the checkout until it stops. Two sessions
wrote to this repository within seven minutes of each other today. Nothing was
lost — the work was in different crates and git did the rest — but the next
overlap will not be as lucky, and the reason it was survivable this time is that
both commits were small and pushed promptly, not that concurrency is safe here.


---

## 2026-09-02 — iteration 14: the accent set, measured

**Built: item 8a**, whole, including a refusal path and a measurement the item
did not name. Two new files in `crates/alo-appearance` — `accent.rs` and
`contrast.rs` — and the setting wired through `changes.rs`, `shipped.rs` and
`appearance.rs`.

| | |
|---|---|
| `accent.rs` | The five a person can choose from, both values each, and the three refusals |
| `contrast.rs` | How far apart two colours are to look at, to the standard EN 301 549 points at |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. `alo-appearance` is 75 unit tests (was 58); the
workspace is **517 tests and 19 doctests**, all green and the rest untouched.
`CHANGELOG.md`, `QUEUE.md` and ADR 0010 in the same change.

**The item said "`Token` gains the set and refuses terracotta", and the stronger
shape was to make terracotta unreachable rather than refused.** `Accent` is a
closed list of five, so a settings file naming terracotta does not read at all
and a picker has nothing to omit — the same reasoning as `alo-capability`'s
`Takes`, where law 2 is carried by there being no shape for the thing to arrive
in. The refusal in words is still there, for the one road a colour can arrive
by: `Accent::of_colour`, which is where a panel that let somebody type a hex
lands, and where `docs/features.md`'s old promise of an accent "drawn from the
design tokens" lands, and where at v1 an agent asked to *make the accent this
colour* will land — that one is a proposal a person approves, so it has to be
refusable in a sentence they can read. Three refusals rather than one, because
asking for the agent's colour, asking for a ground, and asking for a colour
nobody designed are three different mistakes that send a person to three
different places.

**The decision that was not in the item: a colour set is a claim about
legibility, so it is measured.** ADR 0010 says outright that the five hexes are
a designer's proposal rather than a measurement, and a proposal nobody measures
is how a hue that reads beautifully in a design tool reaches somebody who cannot
read it. So `contrast.rs` implements the sRGB relative luminance and the ratio
the standard defines, and `accent.rs` holds every accent to the 4.5:1 EN 301 549
requires of ordinary text — the light-ground value against cream *and* the
porcelain canvas, the dark-ground value against the charcoal rail. All five
clear it as designed; moss on porcelain is the closest at 4.75:1. A sixth hue
cannot be added now without being measured, which is the point.

**And the measurement found something the decision had not claimed.**
**Terracotta on cream is 2.87:1** — under the 4.5 a word needs and under the 3.0
WCAG 2.1 §1.4.11 asks of a shape carrying meaning. ADR 0010 argued the mark and
the word from colour blindness; the arithmetic says the agent's colour on the
reading ground does not reach the threshold for *anybody*. Nothing in the
decision changes and nothing was relitigated — the note is appended to the ADR
under *since it was accepted*, and it makes "never alone" a measured requirement
rather than a principled one. It is the shell's to honour, as the item said.

**Three smaller ones worth keeping.**

- **The accent is stored by name and resolved against the scheme at the moment
  of asking.** One choice, two values, and `Appearance::accent_at` is the fourth
  resolution in that file — so an accent picked in the morning is still readable
  at eight in the evening, and a release that corrects a hex corrects it for
  everybody who chose that colour rather than freezing the number they happened
  to be shown. It is *only the difference is stored* (item 7) reaching one more
  setting, and it reads the clock no more than the schedule does.
- **What ships is held to what a person is offered.** `Shipped` carries
  verdigris, and a test asserts it is one of the five — a default outside the
  offered set would be an accent somebody could lose by touching the setting and
  never get back.
- **`Shipped::of` gained a parameter**, which is a break in a public constructor
  and is deliberate: the alternative is a set of defaults where one of them is
  invisible, and every caller is in this workspace and in this commit. Anything
  a release ships is stated in one place or it is not stated.

**What the next iteration must know:**

- **`Colour::contrast_with` is public and is the crate's first piece of
  arithmetic about colour.** Anything later that puts text on a ground —
  the compositor drawing a label on an accent fill, a settings panel previewing
  a colour — should ask it rather than reason about hexes, and `ENOUGH_FOR_TEXT`
  and `ENOUGH_FOR_A_SHAPE` are the two numbers the standard has.
- **Contrast says nothing about colour blindness**, and `contrast.rs` says so.
  The hue-distance test in `accent.rs` is arithmetic and is documented as a
  floor rather than a perceptual claim — it catches a hue added later that is
  terracotta with two digits changed, and it cannot catch that deuteranopia
  makes terracotta and moss neighbours. That is what the mark and the word are
  for.
- **Item 9d's list grew**: `Accent::name` and the three `AccentError`
  sentences. The names are the same translator's judgement as `Token::name` —
  verdigris is the colour of weathered copper, two words in some languages and
  none in others — and these are names a person picks from a list rather than
  reads once.
- **Nothing here has been drawn.** No accent has reached a screen, because the
  compositor does not exist, and the mark-and-word half of ADR 0010 cannot be
  tested until it does. `ROADMAP.md`'s "Making it yours" stays unticked.

**The roadmap line moved, which is `LOOP.md` step 6 as it was rewritten while
this iteration was running.** *Making it yours* stays an empty box — half a
capability is not a capability — and its clause now says the accent set is
working code rather than a decision, its description drops "accent colour from
the design tokens" because ADR 0010 says it is not drawn from them, and its
*Owed* gains the mark and the word. This iteration was rebased onto that commit;
there was no conflict, and the file it added is the one this entry now honours.

**What is left is 9a–9e and 4a.** 8a was the last item that was neither a
strings item nor the daemon's. 9a needs the CLDR cardinal rules for the 24
languages put in front of it — read, not recalled, and an iteration that cannot
get them should say so plainly rather than write a plural table from memory. 9b
is partly blocked behind 9a; 9c, 9d and 9e are not blocked by anything. 4a is
`alo-agentd`'s and the daemon still does not exist.

---

## 2026-09-02 — iteration 15: the plural rules, read rather than recalled

**Built: item 9a**, whole, including three refusal paths the item did not name.
Four new files in `crates/alo-strings` — `form.rs`, `cldr.rs`, `plural.rs` — and
the plural half wired through `key.rs`, `vocabulary.rs`, `translation.rs` and
`strings.rs`.

| | |
|---|---|
| `form.rs` | The six shapes a counted sentence takes, named as a translator's own tools name them |
| `cldr.rs` | Which form a language uses for which number — the table — and `Counting`, the number a sentence counts |
| `plural.rs` | One countable string the code can say: two English sentences, and the gap that holds the number |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. `alo-strings` is 116 unit tests (was 75), 6
integration tests (was 5) and 5 doctests including the `compile_fail`; the
workspace is **559 tests and 20 doctests**, all green and the rest untouched.
`CHANGELOG.md`, `docs/quirks.md`, `QUEUE.md` and `ROADMAP.md` in the same change.

**The item's first instruction was to get the rules in front of it, and that is
what happened rather than being worked around.** `docs/autonomy/QUEUE.md` and
iteration 11 both said outright that an iteration which could not obtain the
CLDR cardinal rules should say so plainly instead of writing a plural table from
memory. It could obtain them: `common/supplemental/plurals.xml` was fetched from
`unicode-org/cldr`, and every arm in `cldr.rs` quotes the condition it came from
so the next person checks this file against the source rather than against its
own confidence. The samples Unicode publishes beside each rule are a test.

**The scope that made the table tractable, stated rather than discovered.** alo
OS counts things and a thing is a whole number, so `Counting` holds a `u64` and
there is no shape for a fraction to arrive in — the same move as
`alo-capability`'s closed `Takes`. That fixes five of CLDR's seven operands at
zero, which is why Czech's and Lithuanian's `many` cannot happen here and
French's keeps only the half about whole millions. What it costs is written into
`lib.rs`: counting *1.5 hours* is a decision to reopen with the operands in
front of it, not a form quietly picked as though the number had been whole.

**Three things the names lead you to assume, all wrong, all now refusals.**
This is the finding the item exists for and it is in `docs/quirks.md`:

- **Not every language has `other`.** Polish does not, for a whole number: its
  `one`, `few` and `many` cover every integer between them. A translator's file
  built on *every language has one and other* asks a Pole for a sentence nothing
  will ever show and leaves out the two forms most numbers take. So a form a
  language never uses is refused, and the refusal names the forms it does use.
- **`one` is not one number.** Croatian's covers 1, 21, 31 and 101; French's
  covers 0 as well as 1; Latvian's `zero` covers 0, 10, 11 and 20 alike. So a
  translation may spell the number out — *ein Ordner* — only where
  `cldr::names_one_number` says exactly one whole number takes that form, and a
  test walks every official language and every form against the rules to check
  that claim rather than trusting the table.
- **A form is picked by the number *and the language*.** `Strings::count` walks
  the chain asking each language for *its* form of this number, so Polish's
  `few` is never used to look up a Russian sentence, and a language whose rules
  are not in the table is stepped over rather than lent English's two.

**And the fourth, which is `Vocabulary::check`'s.** A countable string
translated into a language whose plural rules nobody has read is refused
outright, in words addressed to whoever is contributing that language, while
every string in the same file that does not count still loads. Falling back to
English's two forms would have been a sentence wrong for most numbers in a
language nobody here reads, with nothing anywhere saying so — the exact failure
`Said` was built to make impossible for plain strings.

**What the next iteration must know:**

- **`unanswered` and `missing_from` now answer `Vec<Key>`**, not `Vec<&Key>`,
  because a countable string's form keys are made rather than stored. They are
  plural-aware: `missing_from` hands a translator the forms *their* language
  needs, and `unanswered` counts a countable string as answered only where one
  chain language has every form it needs. The old signature would have reported
  a Polish file holding `one` and `other` as complete, which is the "bounded
  answer that does not say so" failure `alo-files` named in iteration 8.
- **A countable string owns every form beneath its key**, in both directions:
  `files.too-big` and a phrase called `files.too-big.one` cannot both exist,
  and `Vocabulary::join` checks it too. `VocabularyError::AlreadyCounted` is new.
- **Item 9b is unblocked**, and the shape it copies is already written down:
  `alo-strings`' integration test now carries `Failed::TooBig` as a `Plural`
  counting `bytes`, walked through Polish's three forms and Irish's five. The
  English `one` form is *"holds one byte"*, which is the bug the item named —
  `alo-files` says "1 bytes" today.
- **`Plural::source` is English's two forms**, and `strings.rs` asserts that
  the source language is one this table knows and counts in two rather than
  assuming it. A source language that ever changed would fail that test rather
  than silently answering with the general sentence.
- **Nothing here has been read by anybody.** No Polish sentence has reached a
  screen, because the compositor does not exist and there are still zero
  translations. `ROADMAP.md`'s "Language" stays unticked; its *Built* clause
  gains the plural rules and its *Owed* loses them.

**What is left is 9b–9e and 4a.** 9b is the largest and is no longer blocked by
anything; 9c, 9d and 9e were never blocked, and 9e should still follow the other
three. 4a is `alo-agentd`'s and the daemon still does not exist — after the
strings items, every remaining item in this queue belongs to a daemon, a Linux
host or a certified machine.

---

## 2026-09-02 — iteration 16: the file half, said in the reader's own language

**Built: item 9b**, whole, including two refusals and one guarantee the item did
not name. Two new files in `crates/alo-files`, and every type in the crate that
says something moved onto `alo-strings`.

| | |
|---|---|
| `words.rs` | **New.** Every string this crate can say: 41 phrases and one countable one, the English beside each key, and the notes a translator cannot work without |
| `saying.rs` | **New.** What the six verbs are, what each argument is for, and the sentence a person approves — in the language they read |
| `failed.rs` | The fourteen ways the machine can fail, with `said(&Strings)` and **no `Display`** |
| `real.rs` | `RealError`'s pair, the same way |
| `verbs.rs` | The six, now declared **from** `words.rs`' constants rather than from copies of them |
| `touching.rs`, `doing.rs` | The two refusals this crate words itself, rendered where they are made |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. `alo-files` is 78 unit tests (was 64) and 18
integration tests against a real filesystem and the real vocabulary (was 13);
the workspace is **578 tests and 20 doctests**, all green. `CHANGELOG.md`,
`docs/contracts/agent-verbs.md`, `QUEUE.md` and `ROADMAP.md` in the same change.

**The item named the awkward part correctly, and the answer was to have one
string rather than two that agree.** `alo_capability::Verb::checked` refuses a
verb whose sentence does not name every argument — a person approves the
sentence, so an argument it leaves out is one they did not agree to — and
`Vocabulary::check` refuses a translation that drops a gap the source has. Those
are the same rule in two languages, and they only hold together while the string
a translator is handed **is** the string the declaration was checked against. So
`verbs.rs` declares each of the six from the constant in `words.rs`, and a test
walks all six purposes, all twelve argument purposes and all six sentences back
against the declarations. A test that the two lists are equal would have been
the weaker version of this: it would fail after somebody had already written the
second copy.

**The decision that was not in the item: `Failed` and `RealError` lost their
`Display`.** A `Display` on a user-facing error is an English sentence one
`to_string()` away from a screen, in a shell whose author had no reason to think
about it — and `CLAUDE.md` calls hardcoded English a bug rather than a
preference. So the only road to words is `said(&Strings)`, which answers with a
`Said` that says whether anybody translated it. What is given up is
`std::error::Error` on two types that were never errors a programmer handles,
and the crate says so where somebody will argue with it.

**The second was `Touching::of` and `Did::of` growing a `&Strings`.** Two of the
three refusals in this crate are worded here rather than by the grants, and
`alo_capability::Refused` carries **words** into the record. Rendering them in
the person's language at the moment they are made means what somebody was told
is what is written down — one rendering rather than an English record beside a
translated screen, with nothing keeping the two accounts of one moment equal.
The cost is a parameter on two public constructors, every caller of which is in
this workspace; the alternative was `Refused` carrying a key and a filling,
which is `alo-capability`'s and is 9e's.

**And one addition to a shipped crate: `Key::unchecked`.** A crate that declares
what it can say writes its own keys as literals, and every one of them being a
`Result` would need a fallback that does not exist — a sentence that could not
be looked up is a sentence nobody can read. This is `alo-shortcuts`' shipped
bindings and `alo-appearance`'s shipped wallpaper, one crate further on: built
by the compiler, with a test putting every one back through `Key::named`. It
takes a `&'static str`, so the only thing that can reach it is a literal; a key
from a file still has to be checked and refused.

**What the next iteration must know:**

- **9c and 9d have a shape to copy rather than a decision to make.** A
  `words.rs` of `Word` constants under one area, `Key::unchecked` plus the test
  that walks them, `said(&Strings)` on each type, and no `Display` left behind.
  Neither of those crates' words is carried into somebody else's refusal, so
  neither needs a signature to grow a `&Strings` — a label is asked for where it
  is shown.
- **9e inherits one real question**, now written into the queue item. A `Call`
  renders its sentence in English when the call is made and keeps it, so the
  sentence the record keeps and the sentence a person is shown are two
  renderings of one string. 9b made the second translatable and could not touch
  the first. Whether a `Call` should carry a key and a filling instead is a
  decision about what a record is *for*, and `alo-record`'s `Line` is on the
  other side of it.
- **`alo-strings`' integration test gave three strings back.** That file said
  from the start that a crate's strings leave it when the crate moves, and
  `alo-files`' have — into `crates/alo-files/tests/what_this_crate_says.rs`,
  where Polish, Irish, Latvian and German are walked against the vocabulary the
  code actually uses. What is left there is `alo-shortcuts`' and
  `alo-appearance`', which are labels: no gaps, nothing counted.
- **A refusal never depends on a string table.** A `Strings` that was given no
  words refuses exactly what it refused before and says so with the key, marked
  — there is a test for it, and the contract now says it too. Anything later
  that renders a refusal should keep that property rather than assume the
  vocabulary was loaded.
- **Nothing here has been read by anybody.** There is still no screen and there
  are still zero translations in this repository; the German, Polish and Irish
  in the tests are the tests'. `ROADMAP.md`'s *Language* stays unticked — its
  *Built* clause gains `alo-files` and its *Owed* now says 9c–9e rather than
  9b–9e.

**What is left is 9c, 9d, 9e and 4a.** None of 9c–9e is blocked by anything, and
9e should still follow the other two. 4a is `alo-agentd`'s and the daemon still
does not exist — after the strings items, every remaining item in this queue
belongs to a daemon, a Linux host or a certified machine.

---

## 2026-09-02 — iteration 17: the keyboard, said the way the keyboard is printed

**Built: item 9c**, whole, including a division of the key list the item did not
name. Two new files in `crates/alo-shortcuts`, and every type in the crate that
says something moved onto `alo-strings`.

| | |
|---|---|
| `words.rs` | **New.** Every string this crate can say: 38 phrases, the English beside each key, and the notes a translator cannot work without |
| `refusing.rs` | **New.** Why a combination cannot be a shortcut — `ChordError`, the `Clipboard` trio, and what each says |
| `action.rs` | `Action::word` and `said(&Strings)`, and **no `Display`** |
| `modifier.rs` | The same for `Modifier`; `Modifiers::shown` replaces its `Display` |
| `key.rs` | One macro, two lists: the keys that print a mark and the keys that print a word |
| `chord.rs` | `Chord::shown`, a hand-written `Debug`, and a serde error that names a key rather than composing a sentence |
| `clash.rs` | `Taken::said` and `Clash::said` |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. `alo-shortcuts` is 63 unit tests (was 41) and 6
new integration tests against the real vocabulary; the workspace is **606 tests
and 20 doctests**, all green. `CHANGELOG.md`, `docs/quirks.md`, `QUEUE.md` and
`ROADMAP.md` in the same change.

**The item said `Key::label` and half of those labels are not strings.** This is
the finding, and it is the crate's own doctrine held to rather than a new idea.
`key.rs` has said from the day it was written that a key is *the one printed on
the person's own keyboard*, so `Super+Q` on a French keyboard is the key marked
Q. Fifty-three of the sixty-nine print a mark that is identical on every keyboard
in the union — `Q`, `7`, `,`, `F1` — and a translator rendering `Q` as `Й` would
be naming a **position**, which is exactly the model the file exists to reject.
The other sixteen print a word, and it is a different word almost everywhere:
*Entf*, *Einfg*, *Pos1*, *Strg*, *Bild ↑*. So the sixteen are the strings, and
they are the ones whose notes matter.

Declaring all sixty-nine was the alternative and is worse twice: it hands a
translator forty-one rows reading `A`, `B`, `C`, and it makes
`Strings::unanswered` — *what a release note has to count*, built in iteration 11
— report fifty-three strings nobody should ever translate. `docs/quirks.md`
records the split with what each keyboard actually prints, so the next person
inherits the reasoning rather than the argument.

**The second decision was not in the item either: a sentence never joins a
list.** `Clash` used to read *Super+Left is set to do more than one thing: Put
the window on the left half; Next window; change one of them*, which cannot be
translated — the separator is not punctuation a program can pick, since Greek
writes `;` where English writes a question mark and `·` where it writes a
semicolon, and the conjunction before the last item is a word a machine would
have to place inside a sentence it does not know. So `Clash::said` names the
chord and says *more than one thing*, and `Clash::actions` hands a panel the list
to draw as rows, each said in the reader's own language. Nothing is lost and the
rows are better UI than the sentence was. `Taken` is untouched by this: it names
exactly one action, which a gap holds. Also in `docs/quirks.md`, with the note
that CLDR's list patterns are data to be read if a sentence ever genuinely needs
one inside it.

**Three smaller ones worth keeping.**

- **A deserialiser has no `Strings` and never will.** `Chord` deserialises
  through `Chord::checked`, and `serde(try_from)` needs its error to have a
  `Display` — which is the one thing `ChordError` was supposed to lose. A
  message composed there would be English nothing could translate. So a private
  `NotAChord` writes the **key** of the refusal, and whoever reports a settings
  file that did not read looks that key up and shows the same words a settings
  panel shows. It is `Refused` carrying words (item 9b) with the other answer,
  because the two are in different positions: `alo-files` can ask, and a
  deserialiser cannot.
- **`DefaultsError` keeps its English, and that is the honest line.** It says a
  *release's* own list of defaults contradicts itself, so it is read by whoever
  is fixing it rather than by whoever is using the machine — it stays a
  `std::error::Error` and now names things by the stable names a settings file
  holds. `SnapLeft` is what a programmer needs; *Put the window on the left
  half*, in whichever language happened to be loaded, is not. `Debug` on
  `Modifiers` and `Chord` is hand-written to that same reader — `Super+Shift+Tab`
  — which is what made this possible without inventing a second label.
- **The stored format did not move.** Every `Action`, `Key` and `Modifier` name
  a settings file holds is unchanged, `Chord` serialises the same bytes, and the
  two serde tests say so. Somebody's shortcuts survive this change without
  anything having to migrate.

**What the next iteration must know:**

- **9d writes the third `Word` and should not.** `alo-files` and `alo-shortcuts`
  each declared their own — the same four fields, the same `Key::unchecked`, the
  same `declare_into` — and two copies were deliberately not treated as a
  pattern. A third is one. The queue item now says to lift `Word` into
  `alo-strings` beside `Phrase` and have all three declare from it; it is
  additive there and touches the other two crates only where the type is
  written down, not where their constants are.
- **`alo-strings`' integration test gave two more strings back**, as that file
  says happens when a crate moves. What is left in it is `alo-appearance`'s
  alone, and it now carries a real one with a gap in it — `AccentError::NotOffered`
  — so 9d still has the whole path exercised in front of it before it starts.
- **A refusal still never depends on a string table.** A `Strings` that was given
  no words refuses exactly what it refused before and answers with the key,
  marked; `refusing::tests::a_refusal_without_the_words_still_names_the_rule` is
  this crate's copy of the property `alo-files` established, and anything later
  that renders a refusal should keep it.
- **Nothing here has been pressed.** No key has reached this model, because the
  compositor does not exist, and there are still zero translations in this
  repository — the German, Maltese and the rest are the tests'. `ROADMAP.md`'s
  *Language* stays unticked, its *Built* clause gains `alo-shortcuts` and its
  *Owed* now says 9d–9e; *Keyboard shortcuts a person can change* stays unticked
  and its clause gains the panel being readable in the reader's own language.

**What is left is 9d, 9e and 4a.** Neither 9d nor 9e is blocked by anything, and
9e should still follow 9d. 4a is `alo-agentd`'s and the daemon still does not
exist — after the strings items, every remaining item in this queue belongs to a
daemon, a Linux host or a certified machine.

---

## 2026-09-02 — iteration 18: the colours, named the way the reader names colours

**Built: item 9d**, whole, including one thing the item did not name and one
file it did not expect to delete. Three new files in `crates/alo-appearance`,
one new file in `crates/alo-strings`, and every type in the appearance crate
that says something moved onto the strings crate.

| | |
|---|---|
| `alo-strings/word.rs` | **New.** `Word` and `Word::phrase` — lifted out of `alo-files` and `alo-shortcuts`, which had written the same four fields twice |
| `alo-appearance/words.rs` | **New.** Every string this crate can say: 28 phrases, the English beside each key, and a note on **every one of them** |
| `alo-appearance/unreadable.rs` | **New.** `NotRead` — what a settings file that did not read writes, which is the key of the refusal and never a sentence |
| `alo-appearance/testing.rs` | **New.** The two fixtures the crate's own tests are written against |
| `token.rs`, `accent.rs` | `name` became `word` and `said`; `AccentError` lost its `Display` |
| `colour.rs`, `display.rs`, `picture.rs`, `rotating.rs`, `scheme.rs`, `text.rs`, `time.rs` | The other seven error types, the same way |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. `alo-appearance` is 98 unit tests (was 75) and 8
new integration tests against the real vocabulary; `alo-strings` is 120 unit
tests (was 117). The workspace is **635 tests and 20 doctests**, all green.
`CHANGELOG.md`, `docs/quirks.md`, `QUEUE.md` and `ROADMAP.md` in the same
change.

**The item said eleven of these are colour names, and that turned out to be the
whole of what is different about this list.** A sentence carries enough of
itself for a translator to work from — *there is already something at {path}*
can be translated by somebody who has never seen alo OS. A single word naming a
colour cannot. *Verdigris* is a French loanword for the blue-green of weathered
copper; German has an ordinary word, *Grünspan*, and reaching it from the
English requires knowing what the colour **is**. *Charcoal* names a grey after
burnt wood and German names the same grey after a mineral, *Anthrazit*. Neither
list is reachable from the other word by word, and a colour name got wrong is
not a sentence that reads oddly — it is a row in a picker that does not match
the swatch beside it.

So **every one of the 28 carries a note**, which is not true of either of the
other two crates that declare words, and the eleven colour ones describe the
colour rather than assuming the word travels: *the colour of fired clay, an
orange-brown*; *the blue-green of weathered copper — a church roof, an old
statue*. `words.rs` says outright that describing it instead of borrowing it is
allowed. That is what `alo_strings::Phrase`'s note was built for — the queue
named terracotta as the example before this crate existed — and this is the
first list where it is load-bearing rather than occasional.

**The decision that was not in the item: six deserialisers needed a sentence and
had nobody to ask.** Six things in this crate are read back out of a settings
file through `serde(try_from = …)`, and `serde` requires that error to have a
`Display` — which is exactly what ten error types losing their `Display` was
for. `alo-shortcuts` met this once in item 9c and answered it with a private
`NotAChord`; six copies of that would have been the third pattern this change
exists to stop. So `NotRead` is one public type shared by all six: it writes the
**key** of the refusal, so whoever reports a settings file that did not read
looks it up and shows the same words a settings panel shows. `docs/quirks.md`
records it as the entry it is — a specification that disagrees with ours at the
one point where obeying ours is impossible.

**And the file this change deleted.** `alo-strings`' integration test carried
copies of four `alo-appearance` strings, and said from the day it was written
that a crate's strings leave it when the crate moves. All three of its users
exist now, so what was left in it was copies of strings that are declared for
real one crate away — which is the *half-moved crate reads exactly like a
finished one* failure wearing a test's clothes. Its four tests moved into
`crates/alo-appearance/tests/what_this_crate_says.rs`, where they run against
the vocabulary the code actually uses: a translation checked and shown, what is
not translated being visible and countable, a key nobody declared saying it is a
bug, and the notes on the two words the queue singled out.

**Three smaller ones worth keeping.**

- **`Word` moved, and `Word::phrase` moved with it.** The item asked only for
  the type; three copies of the seven-line loop that turns a `Word` into a
  `Phrase` is the same argument the item makes about the struct, so the loop is
  a method now and each crate's `declare_into` is two lines. The cost is one
  variant: `WordsError` in all three crates has a `Word` variant where it had a
  `Sentence` and a `Note`. Nobody's constants moved.
- **A number is not a string, and the percent sign is.** `TextScale` keeps
  `200%` and `TimeOfDay` keeps `18:00`, because how a number or a time is
  *written* belongs to the region rather than the language and a settings file
  holds one spelling whatever the region does — the queue settled that in item 8
  and it still holds. But the two refusals that carry a percentage put the
  numbers in bare and keep the sign in the sentence, so a language that writes
  *200 %* with a space, or puts the sign in front, can;
  `the_percent_sign_is_the_translators_to_place` is the test.
- **A refusal and the colour inside it are in one language.**
  `AccentError::NotAnAccent` names a `Token`, and what goes into the gap is that
  token *said* — so a German machine does not read a German sentence with an
  English colour in the middle of it. It is `alo-shortcuts`' *{chord} is already
  {action}* property reaching the palette.

**What the next iteration must know:**

- **9e is the last of the 9-series and it is the one with a decision in it.**
  `alo-capability` and `alo-models` still hold their English. The question 9b
  left and 9c and 9d did not touch is unchanged: a `Call` renders its sentence
  in English when the call is made and keeps it, so the sentence the record
  keeps and the sentence a person is shown are two renderings of one string.
  Whether a `Call` should carry a key and a filling instead is a decision about
  what a record is *for*, and `alo-record`'s `Line` is on the other side of it.
  What 9d adds to that question is only evidence: `NotRead` shows that writing a
  key where words cannot be asked for works and is readable.
- **The shape to copy is now three crates old and has not changed**: a
  `words.rs` of `alo_strings::Word` constants under one area, a test walking
  every key back through `Key::named`, `said(&Strings)` in place of `Display`,
  and the key written wherever a `Strings` cannot be reached. `alo-capability`'s
  `Sentence` is the one part with no precedent, because it is parts rather than
  a template — which is why `sentence.rs` was built that way in item 2.
- **`alo-strings` has no integration test of its own any more**, and that is
  deliberate rather than an oversight. Its 120 unit tests are the crate's, and
  the whole path a translation takes is now walked three times over, in
  `alo-files`, `alo-shortcuts` and `alo-appearance`, against real vocabularies.
  Anything later that wants to test the scaffolding should test it through a
  crate that uses it.
- **Nothing here has been seen.** No colour name has reached a screen, because
  the compositor does not exist, and there are still zero translations in this
  repository — the German in the tests is the tests'. `ROADMAP.md`'s *Language*
  stays unticked, its *Built* clause gains `alo-appearance` and its *Owed* now
  names 9e and the two crates it covers; *Making it yours* stays unticked and
  its *Built* clause gains being readable in the reader's own language.

**What is left is 9e and 4a.** 9e is not blocked by anything. 4a is
`alo-agentd`'s and the daemon still does not exist — after 9e, every remaining
item in this queue belongs to a daemon, a Linux host or a certified machine.

---

## 2026-09-02 — iteration 19: the capability model, saying no in the reader's own language

**Built: item 9e, cut to one of its two crates and whole in that one.**
`alo-capability` now says everything it says through `alo-strings`. Three new
files, and every type in the crate that a person reads moved onto them.

| | |
|---|---|
| `words.rs` | **New.** 33 phrases and one countable string, the English beside each key, and the notes a translator cannot work without |
| `refusing.rs` | **New.** `NotGranted` — why the grants refused, carried as what it was rather than as a sentence |
| `testing.rs` | **New.** The two fixtures the other files' tests are written against |
| `grant.rs`, `arg.rs`, `call.rs`, `proposal.rs`, `approvals.rs`, `authorised.rs` | Six error types lost their `Display` and gained `said(&Strings)` |
| `grants.rs`, `reach.rs` | `permitting` answers with a `NotGranted`; `describe` became `shown(&Strings)` |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. `alo-capability` is 107 unit tests (was 85) and 6
new integration tests against the real vocabulary; the workspace is **655 tests
and 20 doctests**, all green. `CHANGELOG.md`, `docs/contracts/agent-verbs.md`,
`docs/quirks.md`, `QUEUE.md` and `ROADMAP.md` in the same change.

**What was cut, and why it is a cut rather than a gap.** The item was two
crates and a decision. `alo-capability` alone is 34 strings across ten files, a
public surface that three other crates call, and the one crate where the shape
the other three settled on does not simply transfer. `alo-models` is item 9f,
unchanged in substance and now with four crates' worth of precedent in front of
it. The sentence question is item 9g, **answered here and not implemented**:
see below, because that distinction is the thing to read.

**The decision the item turns on: a refusal is a value, and it is worded where
somebody reads it.** Item 9b established the opposite shape and was right to:
`alo-files` renders its refusals at the moment they are made so that the record
and the screen cannot be two accounts of one moment. That shape does not
survive this crate. Wording a refusal here means handing `Grants` a `Strings`,
and then *whether an agent may touch a folder* depends on a vocabulary having
been loaded — which is precisely the dependency the deciding crate must not
have, for the same reason it does not read the clock.

So `Grants::permitting` answers with a `NotGranted` carrying the agent, what
was asked for and the grant that ran out, and `said(&Strings)` renders it when
somebody shows it or writes it down. **9b's guarantee survives in the stronger
form**: the screen and the record render the *same value* with the same
strings, so one of them cannot be English while the other is German — neither
is a language until it is asked for. `Entry::refused` takes the strings rather
than the words, so a record cannot be handed a sentence about something else.

**Three decisions that were not in the item.**

- **`Refused` has two doors, and they differ only in whose words they are.**
  `not_granted` carries the value the grants made; `worded_elsewhere` carries a
  `Said`, for the one refusal only the crate holding a resolved path can make —
  *this is granted where it was written and really leads somewhere nobody
  granted*. The second is 9b's rule intact: worded once, where the question was
  answered. `alo-files` keeps its `&Strings` for that reason and for one more —
  `WOULD_CREATE` puts the grants' own refusal inside its own sentence, so it
  renders the inner one there and the two are one language.
- **Three errors keep their English and their `Display`, and that is the same
  decision rather than an exception.** `VerbError`, `VerbsError` and
  `SentenceError` refuse a *declaration*: their reader is whoever is writing an
  adapter against the contract, at the moment their own declaration fails its
  tests. It is `alo-shortcuts`' `DefaultsError` one crate on, and the contract
  now says so where an adapter author will look. What a person hears instead is
  `CallError::Unsayable`, which says nothing ran and that the verb is the thing
  to fix.
- **A length counts.** `ArgError::TooLong` is the crate's one countable string,
  so *longer than one character* is a sentence rather than English's plural with
  a one in it, and a language with more forms than English has gets its own. It
  needed a second copy of `alo-files`' private `Counted` struct — deliberately,
  and `words.rs` says so: two copies are not a pattern, a third lifts it into
  `alo-strings` beside `Word`, which is the rule `Word` itself moved under.

**What 9e answered and did not build, which is 9g.** A `Call` renders its
sentence in English when the call is made and keeps the string. The answer this
iteration reached is that the approval, the record and the screen should be one
thing — the same argument made about refusals, applied to the sentence — which
means a `Call` carrying a key and a filling and `Verb::checked` taking the key
of its sentence. It is not built because it is a public surface change reaching
three crates at once, and doing it in the same iteration as thirty-four strings
is how a decision that size gets made where nobody reads it. It is written into
the queue with the argument, so 9g implements rather than re-decides.

**What the next iteration must know:**

- **Two gaps in a translated sentence still arrive in the source language**, and
  `docs/quirks.md` now records both: `{purpose}` in *{verb} needs {argument}*,
  and `{sentence}` in *"…" was proposed too long ago*. The notes on those two
  words say so to the translator rather than leaving them looking for a string
  that does not exist. 9g closes them.
- **`alo-capability` and `alo-record` depend on `alo-strings` now.** The
  deciding crate because every refusal it makes is read by somebody, and the
  record because it writes down the words the person was shown. `alo-strings`
  depends on nothing, so what the audit surface gained is a list of sentences.
- **The `compile_fail` doctest on `Approved::redeem` was re-checked**, because
  the errors around it stopped being `std::error::Error` and both doctests had
  to drop their `?`. It still fails on **E0382, use of moved value** — checked
  by compiling the body as a test rather than by assuming — so it is still a
  test of *one approval, one execution* and not of a conversion that no longer
  compiles.
- **`alo-files`' test fixture now holds both crates' vocabularies**, which is
  the arrangement a shell has: one vocabulary, one area per crate. A refusal met
  there can have been worded by either crate, and a fixture holding only its own
  words would have shown the key where the sentence should be.
- **Nothing here has been read by anybody.** There are still zero translations
  in this repository; the German in the tests is the tests'. `ROADMAP.md`'s
  *Language* stays unticked, its *Built* clause gains `alo-capability` and its
  *Owed* now names 9f and 9g; *`alo-agentd`* and *every execution recorded*
  gained clauses and stay unticked, because there is still no daemon.

**What is left is 9f, 9g and 4a.** Neither 9f nor 9g is blocked by anything, and
9g is the one with a public surface in it. 4a is `alo-agentd`'s and the daemon
still does not exist — after the strings items, every remaining item in this
queue belongs to a daemon, a Linux host or a certified machine.

---

## 2026-09-02 — iteration 20: the models crate, and where a question is about to go

**Built: item 9f**, whole. `alo-models` now says everything it says through
`alo-strings`, and it is the fifth crate to move. Two new files, and every type
in it that a person reads moved onto them.

| | |
|---|---|
| `words.rs` | **New.** 29 phrases, the English beside each key, and the notes a translator needs |
| `refusing.rs` | **New.** `NotAllowed` — which rule refused a question, and where it would have gone |
| `source.rs` | `InferenceSource::describe` became `shown(&Strings)` and lost its `Display`; `SourcePolicy::refusal` answers with a value |
| `tried.rs` | `Tried::describe` became `said` and `caveats`; `NotTried` lost its `Display` and carries the policy's refusal whole |
| `provider.rs`, `secret.rs`, `runtime.rs` | Three error types lost their `Display` and gained `said(&Strings)` |
| `testing.rs` | Gained the two string fixtures beside the socket one |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. `alo-models` is 90 unit tests (was 70) and 7 new
integration tests against the real vocabulary; the workspace is **682 tests and
20 doctests**, all green. `CHANGELOG.md`, `docs/quirks.md`, `QUEUE.md` and
`ROADMAP.md` in the same change.

**What is different about this list, and it is not a refusal.** Four of the 29
are where an answer came from — *on this machine*, *by Mistral, in the EU*, *by
someone, which has not said where it runs* — and ADR 0008 puts them where the
answer appears, which means they are read **before** somebody decides whether to
paste a contract into the next question. Every other list in the 9-series is
somebody being told that something did not happen. This one is somebody deciding
something, and the last of those four is the only thing on the screen saying the
question is about to leave the building. A translation that softened it would
take that away, so its note says so in the words a translator needs.

The policy's three refusals put that same clause inside themselves, which is why
the source is a string this crate declares rather than something a caller
assembles: **a refusal and the place named in it are one language**, which is
`alo-appearance`'s colour-inside-a-refusal property reaching the network.

**The decision that was not in the item: `SourcePolicy::refusal` had an
unreachable branch, and moving it deleted the branch rather than translating
it.** `Anywhere` permits everything, so the old `Option<String>` had a case that
could not happen, and a repository that forbids `unreachable!()` had filled it
with *"no policy forbids this"* — a sentence for a state nobody can reach, which
would have become a sentence 24 translators were asked to translate. As a value
it is simply absent: `NotAllowed` has three variants because there are three
rules that refuse, and a policy that refuses nothing produces no refusal.

**And the one that was not in the item at all: a reason is a variant, never a
`&'static str`.** `RuntimeError::Refused("the download did not complete")`
carried a sentence the Ollama adapter wrote, in English, one `to_string()` from
a screen — the exact failure item 9b named when it took `Display` off `Failed`,
wearing a payload instead of a trait. It is `DownloadIncomplete` now, with a
string of its own, and an adapter that needs another reason adds a variant and a
word beside it. That is a public surface change to `RuntimeError` and the
`CHANGELOG` says so.

**Three smaller ones worth keeping.**

- **A sentence that would have to count has the number beside it instead.**
  `NotEnoughDisk` said *not enough disk: 5000000000 bytes needed, 1000000000
  free*, which is two numbers and a plural noun in one sentence — and a `Plural`
  counts one number, not two. So the sentence is *there is not enough room on
  this disk for that download* and the two numbers stay fields, for whoever
  writes a size the way the region writes one. That is item 10's own decision in
  this crate — *a sentence here counts nothing out loud* — kept rather than
  quietly dropped now that item 9a exists, and there is a test asserting the
  sentence holds no digits and the crate declares no plural at all.
- **`Tried` answers with a line and up to two more, not one sentence with
  clauses glued on.** It built its answer by pushing `" — the list was cut"` onto
  the end, and that dash is punctuation a program picked — which is exactly what
  `alo-shortcuts` refused in item 9c. `said` is the answer and `caveats` is
  nought, one or two whole sentences to be drawn beneath it.
- **A key cannot reach a sentence in any language.** Neither of the two strings
  about a key has a gap, and `alo-strings` refuses a translation that invents
  one — so `models.key.blank` cannot become *paste the key {key}* in a language
  nobody here reads. It is asserted twice: once on the constants, once through
  the checker from outside the crate.

**What the next iteration must know:**

- **`alo-egress` is the last crate holding English, and nothing had it written
  down.** `DestinationError`, `Destination::describe`, `Leaving::describe` — the
  **indicator line**, which is the sentence law 1 exists to put in front of
  somebody — and `EgressPolicy::refusal`. Iteration 5 put all four on item 9's
  list in this journal and the queue never carried them across, so the whole
  9-series went past them. It is **item 9h** in the queue now, with the two
  decisions 9f already made for it: the policy refusal is a value for the same
  reason `SourcePolicy`'s is, and `Destination::describe` is
  `InferenceSource::shown`'s twin by construction and must not become a second
  sentence about the same provider.
- **A translated error cannot be a `std::error::Error`**, and 9f is where that
  reached a **public trait's** signature — `ModelRuntime` returns `RuntimeError`
  and third parties implement `ModelRuntime`. `docs/quirks.md` records it as the
  general form of the deserialiser entry from 9c/9d. The line held is *who is
  holding the machine when this appears*: `CatalogueError` refuses the catalogue
  this repository ships and keeps its English, like `VerbError` and
  `DefaultsError` before it.
- **Two doctests dropped their `?` and were re-checked.** `SecretError` is no
  longer a `std::error::Error`, so the `compile_fail` on `Secret::bearer` would
  have started failing on the conversion instead of on the privacy. It was
  compiled outside the doctest harness against the real rlib and still fails
  with **E0624, method `bearer` is private** — so it is still a test of *a key
  cannot be read back out*, and its passing twin still passes.
- **`CommercialUse` and `OnCpu` have no words at all**, and that is not a bug
  this item left. They are enums a licence panel would label and nothing in this
  repository labels them yet, so there is no English to move; whoever builds
  that panel declares those words rather than inventing them here. A model's
  name, its licence's name and its note are a publisher's data and are shown as
  written, like a filename in `alo-files`.
- **Nothing here has been read by anybody.** There are still zero translations
  in this repository; the German in the tests is the tests'. `ROADMAP.md`'s
  *Language* stays unticked, its *Built* clause gains `alo-models` and its *Owed*
  now names 9h and 9g; *Agents point at the local model* and *Add your own
  provider* gained clauses and stay unticked, because there is no Settings panel
  and no daemon.

**What is left is 9g, 9h and 4a.** Neither 9g nor 9h is blocked by anything; 9h
is the smaller and the one with the indicator in it, 9g is the one with a public
surface reaching three crates. 4a is `alo-agentd`'s and the daemon still does not
exist — after the strings items, every remaining item in this queue belongs to a
daemon, a Linux host or a certified machine.

## 2026-09-03 — iteration 21: the line that says something is leaving, in the reader's language

**Built: item 9h**, whole. `alo-egress` now says everything it says through
`alo-strings`, and it is the sixth crate to move and the last one that held
English. Three new files, and every type in it that a person reads moved onto
them.

| | |
|---|---|
| `words.rs` | **New.** 13 phrases, the English beside each key, and the notes a translator cannot work without |
| `refusing.rs` | **New.** `Refusal` — which rule refused — and `NotPermitted`, which moved here out of `policy.rs` |
| `testing.rs` | **New.** The two string fixtures the other files' tests are written against |
| `leaving.rs` | `describe` became `said(&Strings)`; lost its `Display`. **The indicator line** |
| `destination.rs` | `describe` became `shown(&Strings)`; `DestinationError` lost its `Display` and gained `said` |
| `indicator.rs` | `Shown::describe` became `said(&Strings)`; `beginning` returns the policy's own refusal |
| `policy.rs` | `refusal` answers with a value; the unreachable branch went with it |
| `alo-record` | `Entry::held_back` takes the strings and renders there, as `Entry::refused` has since 9e |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. `alo-egress` is 45 unit tests (was 27) and 7 new
integration tests against the real vocabulary; the workspace is **707 tests and
20 doctests**, all green. `CHANGELOG.md`, `docs/quirks.md`, `QUEUE.md` and
`ROADMAP.md` in the same change.

**The decision the item did not contain: the indicator line is three whole
sentences, not a stem and a place.** *`{agent} is asking a question of
{destination}`* could have been one string plus a destination appended, which is
two fewer strings to translate and would have been wrong in English before it was
wrong anywhere else — a question goes *of* somewhere, a fetch comes *from* it, a
send goes *to* it, and the preposition is not punctuation a program can pick.
Any language that inflects a place needs the whole sentence in front of it to
choose the form. So there are three, each whole, and the preposition sits inside
the translated string where a translator can move it. That is `alo-shortcuts`'
*a sentence never joins a list* from item 9c met from the other side: there the
machine must not assemble a list, here it must not assemble a sentence.

**The twin the queue asked for is not one string, and could not be.**
`Destination::of` maps an `InferenceSource` onto a destination in one place, and
the item said the two must not become two different sentences about the same
provider. They are already two sentences and have to be: `alo-models` says where
an answer **came from** — *by someone, which has not said where it runs* — and
this crate names a **place a thing is going to**, and *"…is asking a question of
by someone…"* is not English. What must not differ is what they say about the
provider, so that is a test rather than a shared constant: for every source that
leaves, both name the provider, both name the region, and both say *has not said
where it runs* when nobody has. A note on each of the three points at its twin.

**The third decision was about who may write down a refusal.** `alo-models`'
`NotAllowed` is a public enum and nothing depends on its being unforgeable. Here
item 5a's guarantee does: `alo-record` writes a held-back entry from a
`NotPermitted` and from nothing else, so an enum carrying the egress in each
variant would have made every variant a way to write down a refusal that nothing
stopped. So it is two types — `Refusal`, the public value saying which rule
refused, and `NotPermitted`, a struct with private fields and a `pub(crate)`
constructor that `EgressPolicy::refusal` alone calls. The `compile_fail` doctest
in `alo-record` that asserts this moved with the type and was **re-checked
outside the doctest harness**: it still fails with E0624, associated function
`new` is private, so it is still a test of the privacy and not of the new path.

**Three smaller ones worth keeping.**

- **A destination that is data is not a string.** `Destination::word` answers
  `None` for a host a verb's argument named, and `shown` returns it exactly as
  written. `alo.example` is somebody's address; a translation of it would be an
  invention, and declaring `"{host}"` as a phrase would have handed 24
  translators a row with nothing in it but a gap. It is the rule a filename is
  held to in `alo-files`, now carried by a type rather than by a comment.
- **The unreachable branch went, as it did in 9f.** `EgressPolicy::Anywhere`
  permits everything, so the old `Option<String>` had a case that could not
  happen and had been filled with *"no policy forbids this"* — a sentence for a
  state nobody can reach, which would have become a sentence 24 translators were
  asked to translate. There is no variant for it now.
- **A sentence that would have to count has the number beside it instead.**
  `DestinationError::TooLong` said *an address is at most 253 characters — this
  one is longer*, which is a count in a sentence. It now says *that address is
  longer than an address can be — check it is a hostname and nothing more*, and
  `longest` stays a field. There is a test that no sentence in this crate holds
  a digit, and that the crate declares no plural at all.

**What the next iteration must know:**

- **Every crate in this workspace has now crossed onto `alo-strings`**, so
  *hardcoded English is a bug* is a rule with no exceptions rather than a rule
  with a list. What is left of the 9-series is **9g**, which is not a
  translation item: it is the public surface change that makes the sentence a
  person approves one string instead of two renderings, and it reaches
  `alo-capability`, `alo-files` and `alo-record` at once.
- **`alo-egress` is not an `Error` crate any more.** `DestinationError` and
  `NotPermitted` have no `Display`, so `Indicator::beginning` and
  `Destination::at` cannot be `?`-ed into a `Box<dyn Error>`. Three doctests
  dropped their `?` — two here, one in `alo-record` — and `docs/quirks.md`
  records it beside 9f's entry rather than as a second quirk.
- **`alo-record`'s test fixture now declares two crates' words.** A record
  renders a capability refusal and an egress refusal, and a vocabulary holding
  only one of them would keep a key where the person read a sentence. The
  fixture says why.
- **Nothing here has been read by anybody.** There are still zero translations
  in this repository; the German in the tests is the tests'. `ROADMAP.md`'s
  *Language* line gains `alo-egress` and its *Owed* now names only 9g, a shell
  and the translations themselves; *Egress indicator* gains the line said in the
  reader's language and stays unticked, because the indicator itself is a
  compositor surface and there is no compositor.
- **What is left is 9g and 4a.** After 9g, every remaining item in this queue
  belongs to a daemon, a Linux host or a certified machine.

## 2026-09-03 — iteration 22: the sentence a person approves, said once

**Built: item 9g**, whole. The last of the 9-series, and the only one of them
that is not a translation item: it is the **public surface change** that makes
the sentence somebody approves one value instead of two renderings of one
string. It reaches `alo-capability`, `alo-files` and `alo-record` at once.

| | |
|---|---|
| `sentence.rs` (capability) | Rewritten. A `Sentence` is a `Key` and an `alo_strings::Template`, made from a `Word` and from nothing else. `Part`, `parts()` and `render()` are gone |
| `arg.rs` | `Arg` takes a `Word` for its purpose; its fields are private and `purpose(&Strings)` answers |
| `verb.rs` | `Verb::checked` takes a `Word` for the purpose and one for the sentence; `purpose(&Strings)` and `purpose_as_written()` |
| `call.rs` | `Call` carries a `Key` and answers `sentence(&Strings)`, `sentence_key()`, `filling()`. `CallError::Missing` carries the purpose's key; `CallError::Unsayable` is gone |
| `proposal.rs`, `approval.rs`, `authorised.rs` | `sentence(&Strings) -> Said` |
| `approvals.rs` | `AnswerError::Lapsed` carries the call, boxed, rather than a rendering of it |
| `words.rs` (capability) | 32 words, was 33; two notes rewritten because both gaps now arrive translated |
| `verbs.rs` (files) | The six declared *from* the constants — passed, not read out of |
| `saying.rs` (files) | **Gone.** Its three answers are the verb's now, so they work for anybody's verbs rather than for six |
| `words.rs` (files) | `Spoken` and `THE_SIX` gone with it; the table they were is a test |
| `what.rs`, `entry.rs` (record) | `What::of` and `Entry::ran`, `never_asked`, `declined` take the strings |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. **707 tests and 20 doctests across the
workspace**, all green — the same totals as iteration 21, because four tests in
`saying.rs` became three better-placed ones elsewhere. `CHANGELOG.md`,
`docs/contracts/agent-verbs.md`, `docs/quirks.md`, `QUEUE.md` and `ROADMAP.md`
in the same change.

**The decision the item named, and the shape it turned out to have.** The queue
asked for "a `Call` carrying a key and a filling rather than a rendered
sentence, `Verb::checked` taking the key of its sentence". A key would have been
enough to make it work and not enough to make it *stay* true: a declaration
could then be checked against one string and a translator handed another, which
is the exact drift the 9-series exists to remove, wearing a key. So a verb is
declared from `alo_strings::Word`s — `Verb::checked` and `Arg::taking` take one,
and there is no way to declare a verb from a bare string. The string a
translator is given is now *necessarily* the string `Verb::checked` checked, so
*a sentence names every argument* and *a translation may not drop a gap* are two
crates enforcing one rule about one string, structurally rather than because
`alo-files` remembered to declare its six from its constants.

**And the thing the item did not contain: there were two parsers for one
syntax.** `Sentence::parse` was `alo-capability`'s own and `Template::written`
is `alo-strings`', and they disagreed about `{{` — this crate refused a sentence
with a literal brace in it, and the vocabulary accepted the same string as a
phrase. Nothing had noticed because no verb had a brace in its sentence. That is
the 9-series' failure mode one level down: two readings of one string that
happen to agree. There is one parser now, and what is left in `sentence.rs` is
the key plus the one rule that is about approval rather than about strings — a
sentence made only of its arguments describes nothing, which `alo-strings` has
no reason to care about and this crate must.

**Three decisions the next iteration inherits.**

- **`CallError::Unsayable` is gone**, and so is the word behind it. Nothing
  renders a sentence when a call is made any more, so the variant described a
  state that cannot happen — the unreachable branch deleted rather than
  translated, which is 9f's and 9h's rule met a third time. `EVERY_WORD` is 32.
- **`AnswerError::Lapsed` carries the call rather than its words**, boxed as
  `Refused`'s is. That plus `CallError::Missing` carrying the purpose's key
  closes `docs/quirks.md`'s *two gaps in a translated sentence arrive in the
  language the code was written in*, which iteration 19 wrote down and named
  9g as the fix for. The entry is kept, marked closed, because the shape of the
  mistake is worth recognising again.
- **A verb can be declared with a word nobody declared**, and no check at
  declaration time can reach it: `Verb::checked` sees a `Word`, not a
  vocabulary. It compiles, it translates, and it reaches a person as a key in
  the place where the sentence they are approving belongs. So every crate that
  declares verbs owes the test `alo-files` now has —
  `everything_the_six_say_is_something_this_crate_declares` — and
  `docs/contracts/agent-verbs.md` says so to whoever writes an adapter.

**What was re-checked rather than assumed.** The `compile_fail` doctest on
`Approved::redeem` was unmarked and compiled: it still fails with **E0382, use
of moved value**, and with nothing else, so it is still a test of *one approval,
one execution* and not of a signature that stopped compiling. Its passing twin
had to gain a small vocabulary, because asserting on the sentence now means
asking for it — which is the change, shown in the one place a reader of the
public surface will meet it.

**What the next iteration must know:**

- **`Verb::purpose`, `Arg::purpose` and `Call::sentence` all take `&Strings`
  now**, and `Arg`'s fields are private. Anything that wants the source English
  — a test, a declaration check — asks `purpose_as_written()` or
  `Sentence::as_written()`, and both say in their own documentation that they
  are the translator's copy rather than the reader's.
- **`alo-record` renders what ran, not only what did not.** `Entry::refused` has
  taken the strings since 9e; `ran`, `never_asked` and `declined` do now. A
  daemon writing a record hands it the vocabulary the person in front of the
  machine reads, once, and what is written down is what they saw.
- **The fixtures declare words.** `alo-capability`'s `test_calls.rs` and
  `alo-record`'s each carry the two fixture verbs' seven words and a `Strings`
  that holds them, because a fixture verb declared without its strings writes a
  key into the record where the sentence belongs. `alo-capability`'s
  `testing.rs` grew `speaking(&[Word])` and `translating(&[Word], …)` for it.
- **Nothing here has been read by anybody.** There are still zero translations
  in this repository; the German in the tests is the tests'. `ROADMAP.md`'s
  *Language* line loses 9g from its *Owed* and gains the sentence to its
  *Built*; *`alo-agentd`* and *every execution recorded* gained clauses and stay
  unticked, because there is still no daemon and no shell.
- **What is left is 4a**, and it is `alo-agentd`'s: where the record is written
  and what prunes it. The daemon does not exist, so **every remaining item in
  this queue belongs to a daemon, a Linux host or a certified machine.** The
  next iteration should expect to find nothing ready and say so.

## 2026-09-03 — iteration 23: the record outlives the machine being turned off

**Built: item 4a**, whole — and the first thing this iteration did was decide
whether it was buildable at all. The queue listed it under *Ready* and described
it as "the daemon's", and iteration 22 predicted that nothing would be ready and
that this one should say so. Reading it again, that was two thirds right and one
third wrong, which is why the item was built rather than skipped.

**The item names three things, and only one of them is the daemon's.** *How long
evidence is kept* is a rule — pure logic, exhaustively testable here. *The file
it is written to* and *the appending* are `std::fs`, which `alo-files` has been
doing on this machine since item 6a. What is genuinely `alo-agentd`'s is **which
path** and **when a shortening runs**, and those are now item **4b** under
*blocked — linux*. This is item 6's shape met a second time: the queue listed
the file verbs' acting half under *blocked — linux* too, and a later iteration
found that opening a folder needs no portal. A blocker is a claim about code,
and a claim is worth reading twice before a loop stops on it.

`crates/alo-keeping`, a **new crate**.

| | |
|---|---|
| `keeping.rs` | The rule: kept for good, or for a number of whole days. The decision `alo-record` said belongs to whatever writes it to a disk |
| `head.rs` | The first line of the file: what shape it is in, and where the record now starts |
| `writing.rs` | Appending, one entry at a time, on the disk before the write answers |
| `reading.rs` | Reading it back: the beginning, what happened, and what could not be read |
| `pruning.rs` | The only thing in alo OS that removes evidence |
| `damage.rs` | What could not be read, which is never stepped over |
| `failing.rs` | Why there is no record to write to, or none to read |
| `words.rs` | 14 phrases and one countable string, the English beside each key, and a note on every one |
| `testing.rs` | The fixtures the other files' tests are written against |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. 59 unit tests, 6 integration tests through the
whole capability journey onto a real filesystem, 9 more against the real
vocabulary; the workspace is **781 tests and 20 doctests**, all green.
`CHANGELOG.md`, `docs/contracts/record-file.md` (new), `docs/quirks.md`,
`QUEUE.md` and `ROADMAP.md` in the same change.

**A new crate rather than more of `alo-record`, and the reason is a promise.**
`alo-record` says, in its own documentation and in a test, that nothing takes an
entry out: no `remove`, no `edit`, no `forget`. Something has to be able to, or
a record grows until the disk is full. Putting the two together would leave that
promise true of a type and false of the crate around it — and it is exactly the
kind of promise a security reviewer checks by reading the file list. So the
crate that can shorten a record is separate, it is small, and everything in it
is built to make shortening hard to do quietly: what goes is decided by a rule
and a moment with no way to name an entry, an agent or a day; shortening is a
method on the writer, so nothing that is not already holding the record open can
reach it; and it refuses a record it cannot read all of. That is item 4's own
argument for not being part of `alo-capability`, one crate further along.

**The decision the item did not contain: a shortening has to leave a mark, and
the mark cannot be an entry.** A record that aged out its first six months and a
machine that did nothing are the same short file, so *what did the agent do in
March* answers *nothing* and somebody believes it. An entry saying *this record
was shortened* is the obvious fix and is wrong in a way that only shows up on
the second round: an entry has a moment, so the next shortening ages it out, and
after two prunes the record looks untouched again. The mark is therefore the
**first line**, which pruning never walks — and there is a test that shortens
twice and asserts the record still says so. It is also why `Head` exists rather
than a seventh `alo_record::Happened` variant.

**The second decision was not in the item either: a missing record is not an
empty one.** `Reading::at` refuses a file that is not there rather than
answering with a record of nothing. The obvious response to a missing record is
to make a fresh one, and that is precisely how a deleted record becomes an
innocent one — so creating it is a deliberate act by the daemon
(`Writing::opening`) and never a side effect of somebody asking a question. The
sentence a person reads says why: *a machine with no record is not a machine
that has done nothing.*

**Three smaller ones worth keeping.**

- **A line that cannot be read is reported, and a record with one in it is not
  shortened at all.** A reader that skipped unparseable lines would make
  corrupting one line a way to lose an entry with no mark anywhere, and a
  shortening that rewrote the file would tidy away the only evidence that
  something was wrong. The **last** line, unparseable with no newline after it,
  is a different thing — a write the machine interrupted — and is tolerated and
  dropped. Two kinds of damage, one alarming, one ordinary, and two sentences
  rather than one with a clause glued on.
- **Zero days is unrepresentable, not refused at the door.** `Keeping::ForDays`
  holds a `NonZeroU32`, so a settings file saying zero fails to read, and the
  worded refusal exists for the one road a number can still arrive by — a panel
  where somebody typed it. That is `alo-appearance`'s shape from item 8a
  applied to a number instead of to a colour.
- **`alo-keeping` is the first crate that never had to cross onto
  `alo-strings`.** It was written after the 9-series, so it has never held an
  English sentence and no type in it has ever had a `Display`. Its `words.rs`
  also closes a small hole rather than copying one: `alo-files` says *{path}
  could not be {doing}* and fills `{doing}` with an English word, and this crate
  does four things to a file, so it has four whole sentences and no gap holding
  untranslated English. That is `alo-egress`' item 9h decision met again.

**What the gate found that the design did not.** Two entries in
`docs/quirks.md`, both from tests rather than from reasoning:

- **`SystemTime::checked_sub` walks back past 1970 on Windows.** The retention
  boundary was written the obvious way — `now` minus thirty days — and the test
  asserting that a machine whose clock says it is 1970 removes nothing *failed*,
  because Windows counts from 1601 and answered with a moment in 1969 where a
  Unix representation answers `None`. The window is now measured from the epoch
  forwards, so the boundary is the same on every platform and a wrong clock is
  never a way to empty a record.
- **A record is replaced while its own append handle is open**, and Windows
  allows it because `std` opens files with `FILE_SHARE_DELETE`. It works, the
  handle is reopened immediately after the rename, and the quirk says what to do
  if a filesystem ever refuses — close before renaming, never copy over the old
  file in place.

**What the next iteration must know:**

- **The record file is a public surface and now has a contract**,
  `docs/contracts/record-file.md`: one line of JSON per entry, a format number
  in the first line, additive change only, and a record from a newer alo OS
  refused rather than appended to. `docs/features.md` promises a SIEM export at
  v1, and that export reads this file — so the shape was fixed now rather than
  when somebody outside started depending on it.
- **Nothing here reads the clock or decides where the record lives.**
  `Writing::prune` takes the moment, `Writing::opening` takes the path, and the
  crate creates no folders. That is item 4b's, and it is `alo-agentd`'s.
- **Nothing here has been read by anybody.** There are still zero translations
  in this repository; the German in the tests is the tests'. `ROADMAP.md`'s
  *Every execution recorded* line gains `alo-keeping` to its *Built* and now
  owes 4b rather than 4a; it stays unticked, because there is no daemon writing
  to it and no screen reading it.
- **Built and unit tested, on whatever this loop runs on.** The integration
  tests write real files on a real filesystem, and that is Windows rather than
  the certified machine. What is owed with the rest of the hardware
  verification is a run on the machine alo OS ships on, where the two quirks
  above have their other halves.
- **What is left is 4b, and everything else was already blocked.** Every
  remaining item in this queue belongs to a daemon, a Linux host or a certified
  machine. The next iteration should expect to find nothing ready — and, having
  now seen one "blocked" item that was two thirds portable, should read each
  blocker once before writing `LOOP COMPLETE` rather than trusting the label.

## 2026-09-03 — iteration 24: what an agent may do to an application

**Built: item 11, which was not in the queue when this iteration started.** The
reading step is the whole first half of this entry, because iteration 23 left an
instruction — *read each blocker once before writing `LOOP COMPLETE` rather than
trusting the label* — and following it found not a mislabelled item but a
missing one.

Every *ready* item was ticked. The blocked lists name a compositor, sign-in and
window management, `6b`, `4b`, egress enforcement, the image, the workspace
client, and **"application verbs, the acting half"**. That last line is the one
that matters: `docs/features.md` promises *[v0.01] ★ Application verbs: open,
focus, arrange, close*, and the queue had an item for the acting half and
**nothing anywhere for the portable half**. A v0.01 promise had no item at all.
Item 6 is the precedent it should have had from the start — file verbs, the
portable half, then 6a for the acting one — and the same split was never written
for applications.

That is the third iteration running to find a blocker that was part portable
(6a, 4a, now 11). It is worth stating plainly rather than as a coincidence: **a
blocker is a claim about code, and a claim written once tends to be about the
hardest part of an item rather than about all of it.** The queue line now says
what is genuinely Linux — nothing here can start a program on a machine with no
compositor — instead of covering the whole subject.

`crates/alo-applications`, a **new crate**.

| | |
|---|---|
| `verbs.rs` | Three of the four, declared from the words a translator is handed |
| `application.rs` | One installed application: the identifier it is granted by, and the name a person only ever sees |
| `installed.rs` | What this machine has, matched exactly, first entry keeping the identifier |
| `reaching.rs` | The only type meaning *this may reach an application*, and the order the two questions are asked in |
| `refusing.rs` | Why this half says no: not installed, and an entry no verb could name |
| `words.rs` | 13 phrases, the English beside each key, and a note on every one that needs one |
| `testing.rs` | The fixtures the other files' tests are written against |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. 38 unit tests, 11 integration tests — 6 against
the real vocabulary in German and Maltese, 5 through the whole capability
journey and into `alo-record` — and 1 doctest. The workspace is **830 tests and
21 doctests**, all green. `CHANGELOG.md`, `docs/contracts/agent-verbs.md`,
`QUEUE.md` and `ROADMAP.md` in the same change.

**The scope was cut to three verbs, and the cut is the most useful thing this
iteration found.** `arrange` needs an argument saying where the window goes.
That is a `Takes::Choice`, and `Call::filling` fills a sentence from
`Value::describe` — so the chosen option arrives in the approval sentence as the
stable identifier the model picked it by: *put Blender on the `left_half`*.
Untranslated English, inside the one string the whole capability model is built
around, in every language on earth.

It is item 9g's guarantee failing for the one argument kind 9g did not reach. A
verb is declared from `Word`s so that the string a translator is handed is the
string the declaration was checked against — and a choice's options were never
`Word`s, so `alo-capability` can express a capability whose words are not
translatable after all. Nothing shipped is wrong, because no verb in this
workspace uses a choice. But nothing could declare one honestly either, and
wording around it here would have buried the finding while declaring `arrange`
anyway would have shipped it. So it is **item 11a**, it is *ready*, and it names
what it has to decide: the options declared as `Word`s while `Value::Choice`
keeps the stable identifier the record and the model need, which makes
`Call::filling` render through the vocabulary and reaches everything that fills
a sentence from a call.

**The decision the item did not contain: closing asks, and the word is in the
sentence a person approves.** `close_application` does what pressing the close
button does — the application is asked, it may put its own *save your changes?*
up, and the person answers that. The reasoning is what one approval covers.
Every other thing these verbs do is reversible: an application that was opened
can be closed, one brought forward can be sent back. Unsaved work is not, and
*ask Blender to close* is an approval to close an application and not an
approval to discard what is in it. Putting **ask** into the approval string
rather than into a comment makes it something a translator is warned about —
its note says a translation reading *close {application}* would promise
something alo OS does not do — and something a reader cannot miss.

**Three decisions the next iteration inherits.**

- **The identifier is approved; the name is only shown.** An application has two
  names and the second is written by whoever packaged it, so *approve: open
  Mail* reads identically whichever *Mail* is behind it — an approval sentence
  the approved thing can choose is not an approval. No two applications share an
  identifier. A name that cannot be shown in one line is dropped and the
  application stays, because nothing is ever acted on by name and refusing it
  would let a packager decide what this machine can reach.
- **What is installed is consulted second, and the reason is not `alo-files`'.**
  There, asking the disk first would tell an agent whether a file it may not
  touch exists. Here, answering *that is not installed* about an ungranted
  application would let an agent enumerate somebody's machine, and which
  applications a person uses says who they are and who they work for. The test
  is stronger than a spy on the list: the two refusals are asserted to be **the
  same string**, so the answer carries nothing either way.
- **There is no read on this list at all**, and the absence is the design. What
  is running and what is in front of a person reach an agent as *context*, for
  that turn. A `list_applications` would be the background reader `CLAUDE.md`
  calls a bug in this product, and `docs/contracts/agent-verbs.md` now says so
  to whoever writes an adapter.

**What the gate found that the design did not.** One test outside this crate
failed once, on the first whole-workspace run, and then passed in every run
afterwards and in isolation: `alo-keeping`'s
`a_shell_that_forgot_to_declare_these_words_shows_that_it_forgot`. It was not
this iteration's doing and it was not a fluke worth shrugging at — the fixture
`a_shortened_record` names its folder after the **process** alone, four tests in
that file call it, and the harness runs them in parallel threads of one process.
Four threads writing and reading one path. `alo-files`' `a_folder_of_our_own`
has counted as well as named since item 6a for exactly this reason, and the
counter is now in the `alo-keeping` fixture too, with the failure written into
the comment so the next person to meet it knows what it was. **A flaky test is a
gate that does not hold**, and fixing one is the opposite of weakening it.

**What the next iteration must know:**

- **The queue has a ready item again**, which it did not have when this
  iteration began: **11a**, and it is a change to `alo-capability`'s public
  surface rather than a new crate. It should be read beside item 9g in this
  journal, because it is the same argument about the same guarantee.
- **`alo-applications` is the second crate that never had to cross onto
  `alo-strings`**, after `alo-keeping`: written after the 9-series, it has never
  held an English sentence and no type in it has ever had a `Display`.
- **A dev-dependency on `alo-record` is new**, and deliberate. Two of the gate's
  guarantees — *one approval causes exactly one execution* and *every execution
  and every refusal leaves a record* — cannot be demonstrated inside one crate,
  so `tests/from_a_call_to_a_window.rs` walks the real journey and asserts the
  record kept **the sentence that was approved**, not a second rendering of it.
- **Nothing here has opened a window, and nothing has been read by anybody.**
  There is no compositor, and there are still zero translations in this
  repository; the German and Maltese in the tests are the tests'. `ROADMAP.md`'s
  `alo-agentd` line gains `alo-applications` to its *Built* and now owes the
  acting half, `arrange` and the context an agent is given; it stays unticked,
  because there is no daemon and no shell.

## 2026-09-03 — iteration 25: a choice a person can read

**Built: item 11a**, and with it the fourth application verb. It is the first
item since 9g that changes a public surface in four crates at once, and the
reading step was worth doing carefully for the same reason 9g's was: the item
described one change, and doing it honestly required a second one the item did
not contain.

| | |
|---|---|
| `alo-capability/offered.rs` | **New.** One option a verb offers: the name a model sends and the word a person reads, and why those cannot be one string |
| `alo-capability/arg.rs` | `Takes::Choice` holds `Offered`s; `Value::Choice` carries the name **and** what names its words; `Value::words` is the new question |
| `alo-capability/verb.rs` | Three refusals at declaration: an option that is not an identifier, one offered twice, one with nothing to say |
| `alo-capability/call.rs` | `Call::filling` takes the vocabulary, because one kind of value is a string somebody translates |
| `alo-strings/filling.rs` | `Filling::and_said` — a gap that holds a word rather than data |
| `alo-strings/template.rs` | `Filled::gaps_came_from` — the provenance of each word put in |
| `alo-strings/said.rs` | `Said::is_translated` answers about the whole line, not only the sentence |
| `alo-applications/verbs.rs` | `arrange_application`, and the three arrangements v0.01 promises |
| `alo-applications/words.rs` | Seven new strings: the verb, its sentence, its two arguments, and the three arrangements |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. The workspace is **853 tests and 21 doctests**
(was 830 and 21), all green. `CHANGELOG.md`, `docs/contracts/agent-verbs.md`,
`QUEUE.md` and `ROADMAP.md` in the same change.

**The shape the item asked for, and the direction it does not go.** An option is
two things at once: a **name** — `left_half` — which is what a model sends, what
the record keeps and what a script writes, matched exactly and never translated;
and a **word**, which is what a person reads inside the sentence they approve.
`Offered` is both, and `Value::Choice` carries both, so `Call::filling` can look
the word up without holding the verb.

Making the *word* the identity would have been the smaller change and is wrong
in three ways at once: a model would have to send `on the left half of the
screen`, a translator editing a sentence would be editing what a verb can be
called, and the record would hold a different value on a German machine than on
a Greek one. It is `alo-applications`' own identifier-versus-name decision from
iteration 24, one level down and for the same reason — **what is approved and
recorded must not be something anybody downstream can rewrite.**

**The half the item did not contain, and the reason this iteration is bigger
than it looks.** Rendering the option through the vocabulary puts one string
somebody translates *inside* another one, and until now every gap in every
sentence held data — a path, a size, an identifier. So a German approval
sentence with an untranslated arrangement in it would have come back from
`Strings::say` answering `Said::is_translated` with **true**: marked by nothing,
counted by nothing, and read by somebody in Berlin. That is item 9's entire
failure mode arriving through a gap instead of through a key, and shipping the
item without answering it would have been cutting depth rather than scope.

So `alo-strings` grew one door and one rule. `Filling::and_said` is how a gap is
filled with something the vocabulary said; `Filled::gaps_came_from` carries
those through the fill; and **a sentence is only as translated as its least
translated piece** — `is_translated` is now the sentence *and* every word put
into it, and a word whose key nothing declares makes the line `is_a_bug` wherever
it is sitting. The mark in a development build lands on the word rather than on
the sentence, which is where the work actually is.

**What `arrange_application` is, and what it deliberately is not.** Three
arrangements: `left_half`, `right_half`, `whole_screen`. Two windows on opposite
halves is what `docs/features.md` calls *tile* at v0.01 and the whole screen is
*maximise*; **quarters are v0.5** and are not offered, so an agent cannot ask
for one. Minimising is not here either, and that is a judgement rather than an
oversight: it appears on v0.01's *window management* list, which is what a
person does with their own keyboard, and this verb says where a window **goes** —
*out of the way* is not a place, and a verb for it would need a sentence of its
own that nothing in `docs/features.md` asks for.

**Three decisions the next iteration inherits.**

- **An option's words complete the sentence; they do not label a button.** The
  preposition lives in the option — *on the left half of the screen* — because a
  language that inflects the place needs the whole phrase in front of it, and
  the gap can then move to wherever that language puts it. That is
  `alo-egress`' 9h decision about its indicator line met from a third side, and
  every one of the three arrangements carries a note saying so, because a
  translator handed *on the left half of the screen* with no context would
  reasonably write a label.
- **A refusal names the options by name, not by their words.**
  `ArgError::NotOnTheList` lists what has to be *sent*, because a call that
  never validated is a refusal about what arrived. It also keeps `Arg::validate`
  free of a vocabulary, which is item 9e's rule — what is permitted does not
  depend on a string table having loaded — reaching the last place it could have
  been broken.
- **Three things about an option are refused where the verb is declared**: a
  name that is not a lower-case identifier, one name offered twice, and an
  option with nothing to say. They are `check_args`' three rules one level down,
  and none of them was a thing that *could* be checked while a choice was a list
  of plain strings.

**What the next iteration must know:**

- **`Call::filling` takes `&Strings` now**, and `Value::describe` answers with
  data rather than with words. Anything that fills a sentence from a call has to
  ask the vocabulary, and `Value::words` is the question that says whether it
  must.
- **Every crate that declares a verb with a choice owes two tests**, and
  `alo-applications` has both: that every option's word is one the crate
  declares (the 9g test, widened — an option nobody declared reaches a person as
  a key *inside* the sentence, which is harder to notice than one in place of
  it), and that the whole line is one language.
- **The queue has no ready items again.** Every remaining one belongs to a
  daemon, a Linux host or a certified machine. Iteration 23 left the standing
  instruction and it still holds: **read each blocker once before writing `LOOP
  COMPLETE` rather than trusting the label** — three iterations running found a
  blocker that was part portable, and iteration 24 found a v0.01 promise with no
  item at all. What is genuinely left here is the acting halves.
- **Nothing here has moved a window, and nothing has been read by anybody.**
  There is no compositor, and there are still zero translations in this
  repository; the German in the tests is the tests'. `ROADMAP.md`'s `alo-agentd`
  line gains the fourth verb and the readable choice to its *Built* and now owes
  only the daemon, the acting half and the context an agent is given; the
  *Language* line gains the rule about a sentence and the words put into it.
  Both stay unticked, because there is no daemon and no shell.

## 2026-09-03 — iteration 26: what an agent is given when it is invoked

**Built: item 12, which was not in the queue when this iteration started.** The
reading step is again the first half of this entry, because iteration 25 left
the standing instruction — *read each blocker once before writing `LOOP
COMPLETE` rather than trusting the label* — and following it found, for the
second iteration running, not a mislabelled item but a missing one.

Every *ready* item was ticked. `docs/features.md` promises at v0.01 **★ context
on invocation: focused window, selection, open document — offered, never
watched**; ADR 0001 numbers it §4; `CLAUDE.md` puts it in the gate as one of the
six capability guarantees that must be a test in CI. There was no queue item for
it, no crate, and — unlike item 11, which was at least half-covered by a line
about application verbs — **nothing in the blocked lists covered it under
another name either.** The compositor line is about reading a screen. Nothing
anywhere said what a context *is*.

That is now three iterations in four finding v0.01 work with no item (6a, 11,
12). Iteration 24's sentence holds and can be sharpened: **a blocker is a claim
about code, and an item that was never written is not even a wrong claim.** The
reading step that finds these is reading `docs/features.md` against the queue,
promise by promise, rather than reading the queue.

`crates/alo-context`, a **new crate**.

| | |
|---|---|
| `context.rs` | What one invocation offered, the moment it was offered at, and the rows a person reads |
| `focused.rs` | The window in front — told, never granted — and a title that is data rather than a string |
| `selection.rs` | The person's own text: what is taken out of it silently, what is not, and the bound that says it was reached |
| `document.rs` | The only part that grants anything, and the file it grants |
| `turn.rs` | The one turn the offer is good for, and the single grant it makes |
| `refusing.rs` | Why a part of a screen could not be offered |
| `words.rs` | 11 phrases and one countable string, the English beside each key, and the notes a translator needs |
| `testing.rs` | The invocation, the moment and the vocabularies the other files' tests are written against |

**Gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. 47 unit tests, 12 integration tests — 7 against
the real vocabulary in German, Polish and Greek, 5 through the whole capability
journey and into `alo-record` — and 3 doctests, two of them `compile_fail`. The
workspace is **912 tests and 24 doctests** (was 853 and 21), all green.
`CHANGELOG.md`, `docs/contracts/agent-verbs.md`, `QUEUE.md` and `ROADMAP.md` in
the same change.

**The decision the whole crate turns on, and the one a reader is most likely to
expect the other way round: only the document grants anything.** ADR 0001 §3
names two deliberate acts that make a grant — a folder chosen in a picker, and
the document offered at invocation — and a context carries exactly one of them.
So the focused window is *told, not granted*: an agent handed *Blender is in
front of you* still cannot open, focus, arrange or close Blender until somebody
grants it, and a selection is text whatever it says, including when it says
`/etc/shadow`.

The other reading is the obvious one and it is the quiet kind of mistake — a
capability model decided by where somebody's mouse happened to be when they
pressed a key. `turn.rs` has the test that offers all three parts at once and
asserts the grant list holds one thing, and the integration test walks it
through a proposal, an approval and into the record.

**The decision the item did not contain: a grant a context makes is a grant like
any other.** The first shape this design took kept the turn's grant in a
`Grants` of the crate's own, which reads as tidy and is wrong. ADR 0001 §3 says
grants are enumerated, visible where the person can find them, revocable in one
action and expiring; a list nobody but the turn can see satisfies none of those
four while still deciding what an agent may touch. So `Turn::beginning` takes
the machine's own list and puts the grant in it, and a person sees it beside the
folder they picked on Monday.

That brought a second thing with it. Revoking by a handle alone would let a turn
begun on one list take a grant off another — handles are unique to one list, not
across them — so `Turn::ending` checks that the list still holds *that grant* at
that handle before it removes anything, and a turn ended against somebody else's
list removes nothing. There is a test that gives two lists the same handle and
asserts it.

**Three smaller ones worth keeping.**

- **The crate has no serde dependency at all**, and that is the guarantee rather
  than an omission. A context that could be read back off a disk would be a
  context existing without an invocation, so *offered, never watched* is the
  absence of a dependency rather than the absence of a constructor somebody
  remembered not to write. `Context` is not `Clone` either, and
  `Turn::beginning` takes it by value: one invocation is one turn, asserted by a
  `compile_fail` doctest that was checked by unmarking it — both it and its twin
  on `Turn::ending` fail with **E0382, use of moved value**, so neither is a test
  of a typo.
- **Nothing here reaches `alo-record`**, and that is the point rather than an
  omission. An entry per invocation saying what was on somebody's screen would
  build the watched-context log §4 exists to forbid, one honest entry at a time.
  What the record keeps is what the agent then *did*, against the grant it did
  it under — which is the turn's grant like any other — and
  `tests/from_an_invocation_to_a_change.rs` asserts the selected text and the
  window title appear nowhere in it.
- **Two things are done to a selection and only one is announced.** Characters
  nobody can see come out silently, because a character that cannot be seen is
  not part of what somebody selected; text they *can* see is never removed
  quietly, so a selection over 200,000 characters says how many characters were
  left out — `alo-files`' *every bound says it was reached*, met where the thing
  cut short is a person's own document. The bidirectional marks are deliberately
  **kept**: they can reorder a line, and they are also how Arabic and Hebrew are
  written, so removing them would corrupt the text of exactly the readers
  *right-to-left ready* is a promise to, in order to defend a line this crate
  never draws.

**What the next iteration must know:**

- **The queue has a ready item again: 13, the dock.** Found by this iteration's
  reading step and deliberately not built — one item per iteration. It is the
  second v0.01 promise found with no item, and `ROADMAP.md` has been saying so
  in plain words the whole time: *· Built: nothing — the commit that added this
  line added no code*. Its portable half is a layout model in `alo-appearance`'s
  shape, and the thing it has to decide before writing anything is what *labels
  give way to icons where the short edge demands it* means as a threshold, since
  text scaling reaches 300% and the measure is EN 301 549 rather than an eye.
- **Read `docs/features.md` against the queue, not the queue against itself.**
  Three of the last four iterations found v0.01 work with no item. The blocked
  lists are a record of what somebody once thought was hard; the feature list is
  the promise.
- **`alo-context` is the third crate that never had to cross onto
  `alo-strings`**, after `alo-keeping` and `alo-applications`, and the first
  crate in the workspace that *makes* a grant rather than only being checked
  against them. Anything later that wants to create authority should read
  `turn.rs` first — the two ends and the handle check are the shape.
- **Nothing here has read a screen, and nothing has been read by anybody.**
  There is no compositor and no accessibility tree, and there are still zero
  translations in this repository; the German, Polish and Greek in the tests are
  the tests'. The guarantee `CLAUDE.md` names — *with no invocation,
  `alo-agentd` makes no context calls at all* — is a test against a running
  daemon and cannot be written here; it is now stated as what the reading half
  owes, under *blocked — linux*. `ROADMAP.md`'s `alo-agentd` line gains
  `alo-context` to its *Built* and names that half in its *Owed*; it stays
  unticked, because there is no daemon and no shell.

## 2026-09-03 — iteration 27: where the dock goes, and when a name gives way

**Built: item 13, the dock's portable half** — the first iteration in four whose
work was already sitting in the queue as a ready item, because iteration 26 put
it there and deliberately did not build it. The reading step was therefore
short: the item names `docs/features.md`'s v0.01 promise, the promise names four
edges and two orientations and a threshold, and `ROADMAP.md`'s dock line had been
saying *· Built: nothing — the commit that added this line added no code* since
it was written.

`crates/alo-dock`, a **new crate**.

| | |
|---|---|
| `edge.rs` | The four, and the whole of what a person chooses at v0.01 |
| `along.rs` | Which way it runs, and the one thing that never turns with it |
| `measures.rs` | The numbers this crate is built out of, and what each answers to |
| `room.rs` | How much room something takes, and the arithmetic that says so |
| `screen.rs` | The screen it is laid out on, and the side it takes its thickness from |
| `labels.rs` | What became of the names |
| `status.rs` | The status area: which end, and which way it runs |
| `layout.rs` | The whole answer, worked out |
| `shipped.rs` | Where the dock is before anybody moves it |
| `changes.rs` | What a person changed, which is all that is written down |
| `dock.rs` | The two resolved, and every question asked of them |
| `words.rs` | 9 phrases, the English beside each key, and a note on every one |
| `testing.rs` | The vocabularies the other files' tests are written against |

**The gate:** `cargo fmt` clean, `cargo clippy --workspace --all-targets -D
warnings` clean, 56 unit tests in this crate, 8 integration tests against the
real vocabulary in German and Greek, 1 doctest. **976 tests and 25 doctests
across the workspace**, up from 912 and 24.

**The threshold, which is the whole item.** *Labels give way to icons where the
short edge demands it* was the clause the item said had to be decided before
anything was written, and it is now arithmetic: a dock may take **one part in
six** of the side of the screen it sits on, a name needs **a line of text** under
an icon or **five ems** of width beside one, and the names stay while both fit.
Neither number is taste. They are the loosest pair that keeps EN 301 549's *200%
without loss of content* on the smallest screen alo OS lays out for — 1366 by
768, which is what `docs/hardware.md` says the Windows 10 fleet this product
exists to catch is full of — on all four edges. `layout.rs` has the test that
says so and a second that says a tighter share would fail it, so the numbers are
fixed *by* the requirement rather than checked against it afterwards.

**An em is the unit because nothing here can measure text.** An em is the text's
own size, so it scales with the setting without a font in the room, and five of
them is a floor on *room* rather than a promise about a particular name — a name
too long for its room is elided by whoever draws it. Anything else would have
been this crate guessing at metrics it cannot have.

**The decision the item did not contain: which side a dock takes from.** The
obvious reading of *the short edge* is the screen's short side, and it is wrong.
One number would let a dock down the left of a wide screen grow to a quarter of
it while a dock along the bottom of the same screen was squeezed — which is
precisely the *horizontal bar somebody turned sideways* the promise refuses,
arriving through the measurement instead of through the drawing. So a dock takes
from **the side it sits on**: the height for a bottom dock, the width for a side
one. A wide screen gives a side dock more room, and the two orientations become
two layouts rather than one rotated.

Two more halves of the same decision fell out of it. A name **beside** an icon
needs a width where a name **under** one needs a line height, and those are
different sizes at every text size — so on that screen a side dock loses its
names at 214% where a bottom dock keeps them to 274%, and neither number could
have been derived from the other. And **text is never turned ninety degrees**,
which is why a vertical dock's names sit beside rather than along: rotated text
is unreadable at a glance and no magnifier or screen reader expects it.
`Along::text_runs` is a method rather than a comment, so a later change that
wants to rotate a label has to delete a rule instead of adding one.

**The status area, and the one asymmetry worth defending.** The far end of a
*row* is the end the reader reaches last — the right in every official EU
language, the left in Arabic or Hebrew — so a horizontal dock's status area
follows `alo_strings::Direction`. The far end of a *column* does not: every
script alo OS ships or is likely to be given is read downwards, so a vertical
dock's status area is at the bottom in both directions. Mirroring the column
because the language mirrors the row would put the clock above the applications
for readers who did not ask for it. Nothing in the Union needs the row half
today, which is exactly why it is asserted now — `docs/features.md`'s
right-to-left promise is that adding a language later is translation rather than
rework, and this is the second crate to pay it in advance.

**Giving way is not taking away, and the reassurance is inside the string.** A
dock that dropped names as the text grew would be losing content in the one
setting a person turns up because they cannot read the screen, which is what
EN 301 549 forbids — unless the name is still there. So `dock.labels.gave-way`
says, in one sentence, that resting on an icon still gives its name and a screen
reader still reads it. Putting that in the string rather than beside it means a
translator is handed it, a checked translation cannot drop it silently, and the
note tells them which half matters. There is a test on the note, and the
compositor now owes that sentence a hover and an accessible name — written into
*blocked — linux*, because a compositor that drew icons without them would make
this repository say something untrue in twenty-four languages.

**Three smaller ones worth keeping.**

- **A refusal at the door is what lets an answer be an answer.** `Screen::of`
  refuses a screen too small to hold a dock, so `Layout::of` returns a `Layout`
  rather than a `Result` and nothing downstream has to handle a layout that
  could not be made. The floor it refuses below is *worked out* from the
  ceiling — six times a dock of icons, 384 — rather than picked, and a test
  asserts one pixel less would not hold a dock, so it is the tightest floor that
  works rather than a round number above it.
- **A standard is asserted about what the code hands out, never about the
  constant behind it.** `assert!(ICON >= SMALLEST_TARGET)` is a test that cannot
  fail: clippy's `assertions_on_constants` folds it, and without the lint it
  would still be a compile-time truth dressed as a runtime check. It is written
  about `Room::an_icon()` instead. This matters beyond this crate —
  `alo-appearance`'s 200% test survives only because its ceiling comes back from
  `TextScale::range()` — so **any later crate asserting a floor should assert it
  about the value the crate produces**, and `room.rs` says why in the test's own
  documentation.
- **`alo-dock` is the first of the person's-own-machine crates to reach
  another.** `alo-shortcuts` and `alo-strings` depend on nothing here and
  `alo-appearance` on `alo-strings` alone; this one asks `alo-appearance` how big
  the person has made their text, because how much room a name needs *is* that
  answer and a second `TextScale` would be a second answer. Nothing reaches it,
  and it is as far from `alo-capability` as the rest of that group: somebody
  moving their own dock is not an agent doing anything.

**What the next iteration must know:**

- **The queue has a ready item again: 14, *never a silent fallback*.** Found by
  this iteration's reading step and deliberately not built. It is the **third**
  v0.01 promise found with no item at all, after items 11 and 13, and it is a ★
  one: ADR 0008's *a local model that fails does not quietly become an API call*.
  Nothing in `crates/` mentions a fallback — searching the whole tree for the
  word returns nothing — so the thing between *where an answer would come from*
  and *a departure that is already happening* has never been written. The item
  names what it must decide first: whether asking somewhere else is a change in
  the sense of ADR 0001 §5 or a setting turned on in advance.
- **And one more that is not an item yet: ★ *No telemetry* (v0.01).**
  `alo-egress` decides about egress an **agent** causes. A promise that alo OS
  itself sends nothing is about egress with **no agent behind it**, and there is
  neither a crate nor a blocked entry for it under any name. Whoever reads next
  should decide whether its portable half is a rule in `alo-egress` or whether
  all of it is the daemon's — and should write the answer down either way, since
  that is the fourth promise this file has watched go unlisted.
- **Reading `docs/features.md` against the queue is still what finds these.**
  Four iterations in five now. The blocked lists record what somebody once
  thought was hard; the feature list is the promise.
- **Nothing here has drawn a dock**, and nothing has been seen by anybody. There
  is no compositor, no screen and still zero translations in this repository; the
  German and the Greek in the tests are the tests'. `ROADMAP.md`'s dock line
  gains a *Built* clause naming `alo-dock` and stays unticked, because a layout
  model is not a dock until something draws it.

## 2026-09-03 — iteration 28: what happens when the model does not answer

**Built: item 14, *never a silent fallback*** — the second iteration running
whose work was already a ready item in the queue, because iteration 27 put it
there and deliberately did not build it. The reading step was therefore short
and went to ADR 0008 rather than to the queue: the item named one decision to
make before writing anything, and the ADR answers it in a paragraph headed
*Alternatives rejected*.

`crates/alo-answering`, a **new crate**.

| | |
|---|---|
| `answering.rs` | The only type meaning *this question may be answered here*, and the two doors into it |
| `wrong.rs` | What can go wrong where a question was put, and what cannot go wrong there |
| `failed.rs` | It did not answer: what a person reads, and the one door out |
| `elsewhere.rs` | Where else this machine may ask, and the doors a rule has closed |
| `offer.rs` | One place that could be asked instead, and the sentence a person approves |
| `refusing.rs` | An offer that was not this failure's, carrying the failure back |
| `words.rs` | 12 phrases, the English beside each key, and a note on every one |
| `testing.rs` | The places, the failures and the vocabularies the other files' tests are written against |

**The gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. 40 unit tests, 10 integration tests — 6 against
the real vocabulary in Greek and Estonian, 4 through `alo-egress` and into
`alo-record` — and 3 doctests, two of them `compile_fail`. The workspace is
**1055 tests and 28 doctests** (was 976 and 25), all green. `CHANGELOG.md`,
`QUEUE.md` and `ROADMAP.md` in the same change.

**The decision the item asked for, and it is not a close call once ADR 0008 is
read properly.** *Asking somewhere else* could have been a setting a person
turns on in advance — *when the local model fails, use my provider* — or a
thing they are asked at the moment. It is the second, and the reason is that
the first **is** the thing ADR 0008 rejects, with a checkbox in front of it.

The objection in that ADR was never that alo OS would choose badly. It is that
*the person is not there*: their records leave the building at the moment of a
failure they never saw. A box ticked in March cannot make somebody present in
June, so a setting would satisfy the letter of *the person decided* while
losing every part of it that matters. What is built instead is ADR 0001 §5's
shape — one sentence, one approval, one attempt, and an approval is never a
session — carried to a place §5 does not itself reach, since §5 binds an agent
changing this machine and a person's question is not that.

**And the shape is borrowed without the machinery, which is written down rather
than left to look like an oversight.** An offer is not an
`alo_capability::Proposal`. Making one would mean declaring a verb whose
argument is somebody's own question — putting the thing ADR 0001 §4 keeps *out*
of the capability model inside it, in order to reuse an approval list. There is
no verb here, no grant and no path, and `offer.rs` says so at the top.

**Why a crate of its own rather than four more files in `alo-models`.** The
promise is about the **absence** of code: nothing ever asks a second place on
its own. A promise like that is worth exactly what the code around it is small
enough to prove, and `alo-models` carries `ureq` and a TLS stack — a fallback
in it would be one line in one function, and a reviewer would have to read the
whole crate to know there wasn't one. `alo-answering` has **no HTTP client, no
socket, and no serde**, so *the crate that decides where a failed question may
go next cannot itself go there* is checkable from `Cargo.toml`. That is
`alo-keeping`'s argument about `alo-record`, made about a second attempt rather
than about deleting evidence, and it is the third time this repository has
reached for it.

**What the item did not contain, and is the reason the words took as long as
the types.** Three offer sentences rather than one. *Nothing leaves*, *it
leaves this machine and stays on your network* and *it leaves the building* are
three different facts about somebody's records, and the offer is the one
sentence in the crate a person acts on — so where the question would go is
inside the sentence being approved rather than a label beside it. That is
`alo-egress`' item 9h decision (one whole sentence per reason, the preposition
inside it where a translator can move it) met from a fourth side, and `words.rs`
has the test that walks all three.

The other half of the words is a line that is always shown: **nothing was sent
anywhere, and nothing will be unless you say so**. It is shown whether or not
there is anywhere else to ask, because a person who has just watched a question
fail has no way of knowing their records did not go somewhere to be answered —
and a promise nobody is told about is not a feature. `Failed::nothing_was_sent`
is the whole of ADR 0008 in one sentence, and the note tells a translator that
shortening it to *nothing was sent* drops the half that is about the future.

**A hole found on the way, closed where it had to be and written down where it
did not.** Every sentence in this crate has a `{source}` in it, and
`InferenceSource::shown` answered a `String` — so filling that gap with
`Filling::of` would have made a German offer with an English clause in it
answer `Said::is_translated` with `true`. That is item 11a's whole failure mode,
arriving through a gap. So `InferenceSource::said` is new in `alo-models`
(additive; `shown` is now `said(…).into_text()`, one rendering rather than
two), `NotAllowed::said` was fixed with it and has a test of its own, and
everything here fills the gap with `Filling::and_said`. **The rest of the
workspace has the same hole in eight places** — `alo-capability`,
`alo-context`, `alo-egress`, `alo-shortcuts` — and that is **item 15**, written
into the queue with the list and with which two to do first. Fixing them here
would have been a second item in one iteration.

**Three smaller ones worth keeping.**

- **`Failed` is not `Clone`**, which `alo_capability::Approved` settled and this
  crate nearly got wrong: the first draft derived it, and a clone is a second
  way to take an offer from one failure — so *one failure, at most one attempt
  elsewhere* would have held only for callers who did not think of it. Both
  `compile_fail` doctests were checked by unmarking them and reading the error;
  both fail with **E0382, use of moved value**, so neither is a test of a typo.
- **This crate holds no text anybody outside this repository wrote.** Not the
  question, not a model's name, not what a provider said about itself. The
  tempting variant is *what the provider said*, which would make the most
  useful-looking failure line in the product and would be somebody else's
  service composing a sentence that arrives wearing alo OS's voice. `WentWrong`
  carries one `u16` and nothing else, so every sentence here is one alo OS
  wrote. It is also why `NoModelThere` does not name the model: a name that
  could not be shown would have had to be refused, and a genuine failure that
  became an error would leave the person with nothing at all.
- **A reason has to be possible where it is said to have happened.**
  `WentWrong::KeyNotAccepted` is refused on this machine and on a paired one,
  because neither is ever given a key — and *the key for this provider was not
  accepted* about somebody's own machine would send them looking for a key that
  does not exist, which is precisely the confusion `alo-models`' *needs a key*
  and *key not accepted* pair was written to prevent. It is refused where the
  failure is **reported** rather than corrected where it is shown, and
  `NotWhatFailed` keeps its English and its `Display` because its reader is
  whoever wrote the adapter.

**What the next iteration must know:**

- **The queue has a ready item again: 15, the sweep onto `Filling::and_said`.**
  Found by this iteration and deliberately not finished — one item per
  iteration — with the eight sites listed. It is a public surface change in
  every crate it touches and additive in all of them; `alo-models`' `source.rs`
  and `alo-answering`'s `offer.rs` are the worked example, and the two egress
  ones come first because that line is what law 1 shows a person while
  something is leaving.
- **★ *No telemetry* still has no item anywhere**, and this is the second
  iteration to say so. It is the fifth v0.01 promise this journal has watched go
  unlisted. Whoever reads next should decide whether its portable half is a rule
  in `alo-egress` or whether all of it is the daemon's, and **write the answer
  down either way** — a queue entry saying *it is the daemon's* is worth as much
  as the crate would be.
- **The roadmap had no line for this promise**, which is the queue's own gap
  arriving one file further out. Rather than adding an unticked line — the move
  iteration 26 warned about — the *Built/Owed* clause went onto **Agents point
  at the local model by default**, and that line now says outright that it
  carries a ★ promise with none of its own. A default is only a sovereignty
  guarantee if it cannot quietly un-point itself, so the two belong together;
  the reason it went there rather than onto a line of its own is written here so
  nobody has to guess.
- **Nothing in this repository can ask a model anything.**
  `alo_models::ModelRuntime` fetches, lists, loads, unloads and removes, and has
  no method that puts a question. That is not this crate's gap — it is why this
  crate exists now rather than later: the decision about what happens when the
  answer does not come had to be settled *before* something was written that
  asks, because a fallback is not designed, it is written into the asking by
  accident on a Thursday.
- **Nothing here has failed for real**, and none of it has been read by anybody.
  There is no daemon, no runtime answering, no screen and still zero
  translations in this repository; the Greek and the Estonian in the tests are
  the tests'. `ROADMAP.md`'s line stays unticked, because a machine that will
  not fall back is not a machine until something asks.

## 2026-09-03 — iteration 29: a gap that holds a sentence

**Built: item 15**, the third iteration running whose work was already a ready
item, because iteration 28 put it there and deliberately did not build it. The
reading step went to the two worked examples the item named —
`alo-models`' `source.rs` and `alo-answering`'s `offer.rs` — and then to a grep
of its own, which is where the item turned out to be bigger and differently
shaped than it was written.

A **public surface change reaching seven crates**, additive in six.

| | |
|---|---|
| `alo-strings/filling.rs` | `and_composed` — a gap holding a value assembled out of several things this crate said; `and_said` now carries the gaps of the answer it is given; `came_from` answers `&[CameFrom]` |
| `alo-strings/template.rs` | A gap's provenances are flattened into the filled sentence's, rather than one per gap |
| `alo-capability/reach.rs` | `Reach::said`, `Ask::said`, `Ask::fills` |
| `alo-context/focused.rs` | `Focused::said`, `Focused::fills` |
| `alo-egress/destination.rs` | `Destination::said`, `Destination::fills` |
| `alo-shortcuts/chord.rs`, `key.rs` | `Chord::fills` and its pieces, `Key::fills` |

Ten call sites moved: `alo-capability`'s `refusing.rs` (2), `alo-context`'s
`context.rs` (1), `alo-egress`'s `leaving.rs` and `refusing.rs` (2),
`alo-shortcuts`' `clash.rs` (3, one of them the item did not count) and
`refusing.rs` (1), `alo-appearance`'s `accent.rs` (1) and `alo-files`'
`doing.rs` (1).

**The gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. **1042 tests and 28 doctests across the
workspace** (was 1027 and 28), all green. `CHANGELOG.md`,
`docs/contracts/agent-verbs.md`, `QUEUE.md` and `ROADMAP.md` in the same change.

**A correction to the last entry's arithmetic before anything else.** Iteration
28 wrote *1055 tests and 28 doctests*; the workspace held 1027 tests and 28
doctests, and 1055 is 1027 plus the doctests counted a second time. The number
above is every `test result` line outside the `Doc-tests` sections added up,
which is the count that goes up when a test is written. Said here so the next
iteration does not read a rise as a fall.

**The item's own shape was wrong in a way worth reading, and finding out was
the design.** It prescribed *a `said(&Strings) -> Said` beside the
`shown(&Strings) -> String`* at every site, which assumes a clause is one string
somebody translated. Two of the four kinds are not:

- a **chord** is composed. `Super+Bild ↑` is the notation every desktop writes a
  shortcut in, holding a name for each key, and no translator is ever handed it
  whole — so there is no single place its words came from and no `Said` that
  could honestly be made for it;
- a **destination**, a **key that prints a mark**, an **ask that is a path** and
  a **window with no title** are each sometimes a word and sometimes somebody's
  own data, decided by which one it is rather than by who is asking.

So the change is one level lower than the item put it: **the provenance of a gap
is a list.** Empty for data, one entry for a clause, several for a composed one.
That is `Filling::came_from` answering `&[CameFrom]` — the single break in an
otherwise additive change — and it is what makes *only as translated as its
least translated piece* true of a chord and of a nested refusal by the same rule
rather than by two.

**Two sites the item's grep could not see, and the second is the worst of the
ten.** It looked for a gap filled from a `shown(strings)`; two were filled from
`said(strings).into_text()` — a `Said` made correctly and thrown away on the next
line. `alo-appearance`'s `accent.rs` is one. `alo-files`' `doing.rs` is the
other, and it is the one sentence in the workspace with **another crate's
sentence inside it**: *rename_file would put something at …, and @files has not
been granted it — a grant covers where a file goes, not only where it comes
from*. A German machine with `alo-files` translated and `alo-capability` not read
that line with its second clause in English, answered that it was German, and
was counted as done. Closing it needed `and_said` to carry the **gaps** of what
it is given as well as its own provenance, so the rule holds at any depth; there
is a test in `alo-strings` for the depth and one in `alo-files`' whole-journey
integration test for the line itself.

**Three decisions the next items inherit.**

- **A gap holding data can never make a line untranslated**, and every one of
  the new tests has a twin asserting it. A German line naming `alo.example`,
  `/home/anna/Taxes/2024.pdf`, `#7F4A2D`, `org.gimp.GIMP` or `Super+Q` **is** a
  German line. The opposite rule would have been easy to write and quietly
  ruinous: a release note's count of what is left to translate would be out by
  the number of files, hosts and hex values anybody happened to mention, and a
  count nobody can trust is a count nobody reads.
- **A word whose translation is the same as its source is still a translation
  somebody has to make.** `alo-shortcuts`' German fixture had never translated
  *Super* or *Alt* — German writes both the same way — and nothing had ever
  noticed, because the text matched either way. The first thing this item did to
  that crate was fail `a_refusal_and_everything_inside_it_are_in_one_language`.
  It is fixed in the fixture with the reason written beside it, and it is the
  shape of a gap this rule will keep finding.
- **`shown` is `said` with the provenance dropped, and where there is no `said`
  there is a total private fallback rather than an unreachable branch.**
  `Destination::as_named` and `Ask::as_written` are that: each answers something
  meaningful for every variant and is only *reached* for the kind with no words,
  so there is one rendering per type and no `String::new()` standing in for a
  case that cannot happen.

**What the next iteration must know:**

- **The queue has no ready item.** Item 15 was the last one, and every remaining
  entry is under *blocked — linux*, *blocked — hardware* or *not ours*. The next
  iteration's first job is therefore the reading step that has found work in
  five of the last six: **`docs/features.md` against the queue**, promise by
  promise. The blocked lists record what somebody once thought was hard; the
  feature list is what was promised.
- **★ *No telemetry* still has no item anywhere**, and this is the third
  iteration to say so. It is the fifth v0.01 promise this journal has watched go
  unlisted and the only one still unlisted. Whoever reads next should decide
  whether its portable half is a rule in `alo-egress` or whether all of it is
  the daemon's, and **write the answer down either way** — a queue entry saying
  *it is the daemon's* is worth as much as the crate would be, and would end
  three entries of this paragraph.
- **`Filling::came_from` changed shape**, from `Option<&CameFrom>` to
  `&[CameFrom]`. It is the only non-additive change in this iteration and the
  only one in the workspace since 9g. Anything written against the old signature
  is in this repository and has moved with it; nothing outside it exists yet.
- **Nothing here has been read by anybody.** There are still no translations in
  this repository — the German, the Greek and the Estonian in the tests are the
  tests' — so what this iteration changed is invisible today and is exactly what
  makes the first real translation honest about how far it got. `ROADMAP.md`'s
  Language line gains a clause and stays unticked.

## 2026-09-03 — iteration 30: what this machine does when nobody has asked it to

**Built: item 16**, which did not exist when this iteration started. The
reading step was the one iteration 29 prescribed — `docs/features.md` against
the queue, promise by promise — and it found exactly the thing three previous
entries had written down and left: **★ *No telemetry*** (v0.01, the
sovereignty section) had no queue item, no crate, and no blocked entry under
any name. It was the fifth v0.01 promise this journal has watched go unlisted
and the last one still unlisted.

Four new files in `crates/alo-egress`, one file's shape changed, and the words.

| | |
|---|---|
| `errand.rs` | `Errand` — the closed list of reasons alo OS itself reaches the network, `Errand::EVERY`, and `Errand::nothing_else`, which is the promise as a string a person reads |
| `itself.rs` | `OnItsOwn` — one errand about to happen, and the line said about it while it happens |
| `underway.rs` | `Underway` — the only type meaning *alo OS may open this connection*, made by the indicator alone |
| `showing.rs` | `Showing` — what one line of the indicator is about, whichever kind caused it |
| `indicator.rs` | `beginning_on_its_own` and `ended_on_its_own`, one private `line` both doors go through, and `Shown` holding either kind |
| `words.rs` | Four phrases: three errand lines and the promise |

**The gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. **1066 tests and 30 doctests across the
workspace** (was 1042 and 28), all green. `CHANGELOG.md`, `QUEUE.md` and
`ROADMAP.md` in the same change.

**The question three iterations left, and the answer.** They asked whether the
portable half of *no telemetry* is a rule in `alo-egress` or whether all of it
is the daemon's. It is a rule here, and the reason it was worth asking is that
the answer is not a rule at all — **it is a type**. *No telemetry* is law 2's
shape applied to what the system does rather than to what an agent does: a verb
cannot run a shell because `Takes` has no free-text kind for one to arrive in,
and this machine cannot report on its owner because `Errand` has no member for
it. There is no `Other(String)`, no diagnostics, no crash report, no *usage*
anything. What the daemon owes is the wiring — the code that actually signs
somebody in, fetches a model or checks for an update — and none of it can
introduce a fourth reason without editing a public enum and failing a test
written to be a tripwire.

`docs/features.md` says the policy lives *in a Rust service, not a checkbox*,
and the difference this file can now point at is that a checkbox has the code
behind it either way.

**The decision the item did not contain, and the one that shaped every file: it
goes on the same indicator.** The tempting shape is a list of the system's own
egress beside the list of the agents' — two lists, two screens, each correct.
It is wrong for the reason law 1 exists: the failure being prevented is not
*the policy was wrong*, it is *nobody could see it*, and a second place to look
is a second place to forget. So `Indicator::is_quiet` is now false while alo OS
fetches a model, `Shown` holds either kind, and the promise being kept is
strictly stronger than the one in `docs/features.md` — not *no telemetry* but
*nothing at all that you cannot see*.

That cost one public signature. `Shown::leaving` answered a `Leaving`, and it
cannot any more; it answers `Option<&Leaving>`. It is the only break in the
change and the only one in the workspace since item 15's `Filling::came_from`,
and it is the change rather than a casualty of it: a line that could always name
an agent would be a list that could only hold one kind.

**Three decisions the next items inherit.**

- **The organisation's egress policy is deliberately not asked about an
  errand.** `EgressPolicy` is `From<&SourcePolicy>` — a rule an organisation
  stated about *where a question may be answered*, widened to everything an
  agent can cause. `SourcePolicy::ThisMachineOnly` maps to
  `EgressPolicy::NothingLeaves`, so asking it about a model download would stop
  a machine set to answer on its own hardware from ever fetching the model it
  would answer with: a policy that defeats the setting it came from. An errand
  is decided by being on the list and by nothing else, and there is a test named
  for that machine. What an organisation controls about updates is *where they
  come from* (v1, a mirror they host), which is a destination and not a
  permission.
- **There is no agent here, and the type has no room for one to be missing
  from.** Giving alo OS a `Grantee` would have made `Leaving` do both jobs in
  one type and would have said the system acts under a grant — nobody granted
  their machine permission to sign them in. So `Showing::agent` answers `None`
  for an errand, and `OnItsOwn` has no `agent()` at all: the honest shape is a
  field that does not exist rather than one that is always empty.
- **A twin type rather than a widened one, and `alo-record` is why.**
  `Underway` duplicates `Departing` almost exactly, which looks like the thing
  law 4 dislikes until you follow the alternative: widening `Departing` makes
  `alo_record::Happened::Left`'s *whose authority was this under* an `Option` in
  every entry the crate writes, in the crate whose whole job is saying who did
  what. Two authorities, one indicator — the sharing happens where a person
  looks, not where a record is written.

**Two things worth keeping that are smaller.**

- **The promise is a string, not a paragraph in a README.**
  `Errand::nothing_else` says *alo OS reaches the network for these reasons and
  no others, and never to say anything about how you use this machine*, and it
  is shown beside the list. That is `alo-answering`'s *nothing was sent
  anywhere* made about the machine rather than about one failed question, and it
  is here for the same reason that one is: the person it is for does not read
  this repository and may not read English. Its note warns a translator that
  both halves are load-bearing and a shorter rendering must drop neither, and
  the integration test that reads it is in Greek.
- **The tripwire test is a tautology on purpose.** `Errand::EVERY.len() == 3`
  plus an exhaustive match over the three is a test that asserts nothing about
  behaviour and everything about who reads what. A fourth reason cannot be added
  without both failing, so whoever adds one arrives in `errand.rs` and finds out
  there that measurement is not a scope decision somebody may revisit.

**What the next iteration must know:**

- **The queue has two entries it did not have: 16a is ready, 16b is not.**
  **16a** is the record of what alo OS did on its own — law 1's second half for
  the half of egress this iteration built — and it was cut for a question rather
  than for size: `alo-record`'s `Happened::agent` answers a `Line` for every
  variant there is, an errand has nobody to name, and the record file is a
  public surface where the wrong answer is hard to take back. It is portable and
  it is the obvious next item. **16b** is discovery: ADR 0003's
  zero-configuration machine-finding announces and listens rather than reaching
  a named destination, so it is the one thing item 16's closed list does not
  cover — and since item 5 this crate has held that a host answering on the same
  wire is *outside* the building, which makes that traffic something a person
  might reasonably expect to see. It is deliberately **not** ready: there is no
  discovery code here and none of it is portable, so deciding now would be
  deciding in the abstract.
- **The reading step found a second unlisted v0.01 promise, and it is a ★ one:
  *Or not at all* (ADR 0009).** This entry very nearly claimed that every v0.01
  promise finally had an item; checking that claim rather than writing it is
  what turned it up. There is nothing anywhere in `QUEUE.md` for the fourth
  choice at setup — no model, no provider, no agent — and `ROADMAP.md`'s line
  for it says *Built: the decision (ADR 0009), not code · Owed: all of it*,
  which was accurate and which nothing had ever converted into work. It is
  **item 17**, written into the queue by this iteration and deliberately not
  built. Two thirds of it are the shell's, but the third that is not is the
  sharpest part of the ADR: *turning it off again removes the agent's reach at
  once — grants end, nothing further is recorded as agent activity*, which is
  `alo-capability` and `alo-record` and is portable.
- **So the count is now six unlisted v0.01 promises found by reading
  `docs/features.md`, in seven iterations.** The lesson has stopped being *do
  the reading step* and become something narrower: **the blocked lists and the
  roadmap are both written by people who already know what is hard, and neither
  of them is a list of what was promised.** Only `docs/features.md` is. The next
  iteration should read it again — v0.01 first, since it has now been wrong
  twice about being complete — and should treat a `ROADMAP.md` line whose
  *Built* clause cannot name a crate as the strongest available signal that the
  queue is missing an item.
- **Nothing here has opened a connection.** There is no code in this repository
  that signs anybody in, downloads a model or asks about an update; `alo-models`
  fetches over `ureq`, and it does so under `ModelRuntime` rather than under an
  errand. What was built is the shape those three will have to arrive in, and it
  was built before them for the reason item 14 was: a fourth reason to phone
  home is not designed, it is written into the first one by accident.
- **`ROADMAP.md`'s *Egress indicator, and no telemetry* line gains the second
  half of its subject and stays unticked**, because an indicator nothing draws
  is a measurement of what the code believes rather than of what the machine
  does. The line now names all three things owed: the compositor surface, queue
  16a, and enforcement at the network boundary.

---

## 2026-09-03 — iteration 31: the one entry with nobody's name on it

**Built: item 16a**, which was the first ready item on the queue and had been
written there by the iteration before this one, cut from item 16 for a question
rather than for size. Law 1's second half is *and afterwards in a record*, and
until this change there was no shape in `alo-record` for a departure nobody
caused: `Entry::left` takes a `Departing`, which carries a `Grantee`, and
`Happened::agent` answered a `Line` for every variant there was.

A **public surface change** in `alo-record`, and one in the record file with it.

| | |
|---|---|
| `happened.rs` | `Happened::LeftOnItsOwn` — a new variant with no agent field; `agent()` now answering `Option<&Line>`; `errand()` and `on_its_own()` beside it; `caused_egress()` true of it |
| `departed.rs` | `Entry::left_on_its_own`, made from an `Underway` and from nothing else |
| `explain.rs` | `Only::OnItsOwn` — the question somebody puts to the record having just read the *no telemetry* promise |
| `entry.rs`, `lib.rs` | `Entry::agent` answering an `Option`, and the seventh row of the table |
| `docs/contracts/record-file.md` | What an entry says happened, the entry with no `agent` on it, and what a new kind of `happened` means for a reader that predates it |

**The gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. **1076 tests and 32 doctests across the
workspace** (was 1066 and 30), all green. `CHANGELOG.md`, `QUEUE.md` and
`ROADMAP.md` in the same change.

**The decision the item existed to make, and it named both candidates.** *A
stable identity for the system that is not a `Grantee`* is the one that reads
well and is wrong. Whatever type it were, it reduces at the record file to a
string in the position that answers *whose authority was this under* — and
nobody granted alo OS anything. Three things follow that are not opinions:
`Asking::by` would answer a question about one agent's day with the machine's;
a SIEM export (`docs/features.md`, v1) would file it in the *who did what*
column beside agents that really were granted something; and no spelling could
be reserved against collision, because the record is written by one alo OS and
read by tools that are not it.

So the seventh variant has **no agent field**, `Happened::agent` answers `None`,
and `Entry::agent` answers an `Option` — the one break in the change, and the
change rather than a casualty of it, exactly as `Shown::leaving` was in item 16.
It is the same answer `alo-egress` gave one crate earlier: *the honest shape is
a field that does not exist rather than one that is always empty*. The record
follows the indicator because a person watching something leave and a person
reading about it a week later are asking one question, and two answers to one
question are two things to keep in agreement.

**The half the item did not contain, and the reason it was worth an iteration:
whether a new `Happened` is additive at all.** The queue item called one
additive in passing. `docs/contracts/record-file.md` did not say — its additive
rule was written about *fields inside an entry*, and a new tag on `happened` is
the third case, which it did not cover. An older reader cannot parse
`left-on-its-own`; it reports that line as one it could not read, with its line
number, alongside everything it could.

The contract now says outright that this **is** additive and does not raise
`format`, and the argument is about what the alternative costs. Raising it makes
the whole file unreadable to that reader rather than one line of it — the
contract already says a record from a newer alo OS is refused rather than
appended to — and it would tie the record's version to the growth of the
capability model, so a security team's tooling would stop reading a machine's
record the first time alo OS learned to do something new. What makes it safe was
already built and is now written down: the file is appended to and never
rewritten, so an older *writer* loses nothing, and `alo-keeping` refuses to
shorten a record with a line it could not read — so the version that does not
understand an entry is also the version that will not remove it.

**Three decisions the next items inherit.**

- **`Only::Egress` counts everything that left, errands included.** This is item
  16's *one indicator, not two* asked afterwards rather than watched at the
  time. A person asking what left their machine is not asking a question about
  authorship, and a query that answered with the agents' share would be the
  second place to look that item 16 refused to build. `Only::OnItsOwn` narrows
  it rather than sitting beside it, because every errand left.
- **An entry is still made from the type the indicator hands out and from
  nothing else.** `Entry::left_on_its_own` takes an `Underway`, whose only
  constructor is `Indicator::beginning_on_its_own`, so an errand the indicator
  never showed is not an entry that can be written. The `compile_fail` doctest
  asserting it was checked to fail on the privacy (E0624) and not on a typo, by
  compiling `Underway::at` beside `Underway::new` and watching only the second
  one fail.
- **There is no `held_back` twin, and the absence is the design.** An errand is
  decided by being on `Errand`'s closed list and by nothing else (item 16), so
  there is no policy that could have refused one and no refusal for a record of
  one to be made from. A twin constructor would have been a door for a refusal
  that cannot happen.

**Two smaller things worth keeping.**

- **`Why` and `Errand` stayed two questions.** `why_it_was_leaving()` answers
  `None` for an errand and `errand()` answers for it, rather than one field
  holding either. An agent's reasons are open to whatever verb needs one; the
  machine's own are a closed list of three, and a shared field would have been a
  place for a fourth to arrive without anybody editing that list.
- **`alo-keeping` gained `alo-egress` as a dev-dependency and one test.** The
  contract now makes a claim about the *file*, so the claim is checked on a real
  filesystem: an errand written down, the machine turned off, the record read
  back, and the line inspected as text to assert it contains
  `models.alo.example` and does not contain `agent`. Nothing in `alo-keeping`
  itself decides anything about egress, and the dependency says so.

**What the next iteration must know:**

- **The queue's next ready item is 17** — *a machine with no agent at all* (ADR
  0009), written in by iteration 30 and untouched since. It is `alo-capability`
  and `alo-record`, and item 16a is the one it was deliberately sequenced after:
  the ADR keeps the record and the egress indicator alive on a machine with no
  agent, on the grounds that somebody who declined an agent may want **more**
  than average to know what left their machine — which is precisely the entry
  this iteration built, and is now something that exists rather than something
  item 17 would have had to invent on the way past.
- **16b is still not ready and should stay that way.** Nothing about discovery
  has been built since iteration 30 wrote it down, so deciding whether multicast
  is an errand or a documented exception would still be deciding in the
  abstract.
- **The reading step found no unlisted v0.01 promise this time**, and this entry
  is deliberately not claiming that none is left — iteration 30's entry nearly
  made that claim, and checking it rather than writing it is what turned up item
  17. What was read here was `docs/features.md`'s v0.01 sovereignty and record
  sections against the queue, which is narrower than iteration 30's sweep
  because this item's subject was narrow. The standing advice from that entry
  holds and is not discharged: **only `docs/features.md` is a list of what was
  promised**, and a `ROADMAP.md` line whose *Built* clause cannot name a crate
  is the strongest available signal that the queue is missing an item.
- **Nothing here has opened a connection.** There is still no code in this
  repository that signs anybody in, downloads a model or asks about an update.
  What exists now is the whole of what those three will have to travel through:
  a closed list with no member for measuring anything, an indicator that cannot
  be bypassed, and a record entry that names nobody because there is nobody to
  name.

---

## 2026-09-03 — iteration 32: the machine somebody chose not to have an agent on

**Built: item 17**, which was the first ready item on the queue and had been
written there by iteration 30 while it was checking a claim rather than making
one. `ROADMAP.md`'s line for ADR 0009 had said *Built: the decision (ADR 0009),
not code · Owed: all of it* since it was written, and nothing had ever turned
that into work.

A **public surface change** in `alo-capability`, additive in `alo-record`.

| | |
|---|---|
| `agent.rs` | `Agent` — a new file: the fourth choice as a value, `present`/`declined`, `declining`/`accepting`, `grants`/`grants_mut`, `permitting`/`permits`, and the line Settings shows |
| `refusing.rs` | `NotGranted::NoAgent` — the third refusal, and the one that must not send somebody to a panel their machine does not have |
| `words.rs` | Three new phrases: the refusal, and the two lines that say what turning the agent off or on would **do** |
| `alo-record/explain.rs` | `Only::ByAnAgent` — *is there anything in here that an agent did at all?* |
| `docs/contracts/agent-verbs.md` | A machine may have no agent, and then there is no list |

**The gate:** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, `cargo doc
--workspace --no-deps` clean. **1094 tests and 34 doctests across the
workspace** (was 1076 and 32), all green. `CHANGELOG.md`, `QUEUE.md` and
`ROADMAP.md` in the same change.

**The decision the item existed to make, and it offered two answers that were
both incomplete.** *A state something asks about* is a flag every caller must
remember to check, which is the shape this repository refuses everywhere else —
and worse than that here, because a flag beside `Grants` can **disagree** with
it: *the agent is off* and *there are four grants* would both be true of one
machine and nothing could say which was the truth. *The absence of any grant and
any grantee* is stronger and cannot survive a restart on its own, which the item
said and was right about.

The answer is both, made one value: the list went **inside** the choice.
`Agent::Present(Grants)` or `Agent::Declined`, so a declined machine holds no
`Grants` at all rather than an empty one. There is nothing to remember to check
because there is nothing to check — the only road to the machine's grants runs
through the choice, and on a declined machine it stops. It is a state, because
it serialises and a machine reads it back; it is the absence of any grant,
because the state is what would have held them. They cannot disagree because
they are one thing.

That is `alo-egress`' item 16 answer met from the other direction. There, the
honest shape was *a field that does not exist rather than one that is always
empty*. Here it is *a list that does not exist rather than one that is always
empty*, and it buys the same thing: a guarantee carried by the shape of a value
instead of by whoever writes the next caller.

**Three decisions the next items inherit.**

- **`Agent` has no `Default`, and the absence is asserted.** A default would be
  alo OS answering setup's fourth question on the person's behalf, in the type
  that exists because that question is theirs — and whichever of the two it
  picked would be the answer nobody ever gave. The `compile_fail` doctest was
  checked by compiling `Agent::present()` beside `Default::default()` and
  watching only the second fail, with E0277 on the missing trait rather than on
  a typo.
- **Turning it on again brings back an agent and not the folders.** ADR 0009
  says grants *end*; a suspension that restored itself in June would be a weaker
  promise wearing the same sentence. `accepting` makes
  `Present(Grants::default())`, and a machine that already has an agent keeps
  what it has rather than being quietly cleared — a second door to `declining`
  with an innocent name is the mistake that test exists to catch. The grant an
  invocation's document makes (item 12) ends with everything else, and
  `alo-context` has the test, because that is the grant nobody remembers making.
- **A refusal on a machine with no agent is a third `NotGranted`, not a narrower
  `Never`.** On that machine the grants panel is **absent** rather than greyed
  out, so *grants are made by picking a folder, never by asking for one* — true,
  and this repository's best sentence — would send somebody to something their
  machine does not have. Reusing the variant would have been free and would have
  been the quiet kind of wrong: nothing would fail, and one person in
  twenty-four languages would go looking for a panel.

**The half the item did not contain, and the one that took the longest to get
right: whether a refusal is still recorded.** *Nothing further is recorded as
agent activity* reads like a licence to stop writing, and taking it that way
would have broken a guarantee `CLAUDE.md` names in the gate — every execution
**and every refusal** leaves a record. The two rules do not actually conflict:
the ADR is describing a machine with no agent doing nothing, not asking for
silence when something asks anyway. And if something does ask, that is precisely
the entry the person who declined would want, which is the same argument the ADR
itself makes for keeping the record and the indicator at all.

So the promise is checked as the shape it really has, in
`crates/alo-record/tests/a_machine_with_no_agent.rs`: an ordinary day on a
declined machine has nothing in the record with an agent's name on it, and a
call that arrives anyway is refused **and** written down. `Only::ByAnAgent` is
what makes the first of those a question rather than a list of names somebody
trusted — it is `Asking::by` seen from the other side, and naming every agent
that might have run is no way to establish that none did.

**No `Happened` variant was added, and the absence is the decision.** An entry
saying *the person turned the agent off at one o'clock* was tempting, reads
well, and is the wrong crate: `alo-record` holds what an agent caused and what
the machine did on its own, and a person's own act on their own machine is
neither. Nothing else a person does is in there — making a grant is not,
revoking one is not — and a record that started keeping settings changes would
become a log of the person rather than of the agent, which is ADR 0001 §4's
watched context arriving through the back door. If a security team wants *when
was the agent turned off*, that is a separate question with a separate answer,
and it is not this file.

**What the next iteration must know:**

- **The queue's remaining unticked items are 16b and 17a, and neither is
  ready.** **16b** is discovery, and nothing about it has been built since
  iteration 30 wrote it down, so deciding whether multicast is an errand or a
  documented exception would still be deciding in the abstract. **17a** is new,
  written by this iteration into *blocked — linux*: the rest of ADR 0009 is the
  hotkey doing nothing, the overlay not existing, Grants and Models and
  providers being absent from Settings, and setup's fourth choice as a screen.
  All of it is compositor and settings-panel work. **So the next iteration
  should expect to write `LOOP COMPLETE` unless its reading step finds an item,
  and the reading step is the whole of what it should do.**
- **The reading step found no unlisted v0.01 promise this time**, and as in
  iteration 31 this entry is not claiming none is left. What was read was
  `docs/features.md`'s AI-stack section against the queue, which is where item
  17's own line lives. The standing advice holds and is not discharged: **only
  `docs/features.md` is a list of what was promised**, and a `ROADMAP.md` line
  whose *Built* clause cannot name a crate is the strongest available signal
  that the queue is missing an item. Item 17's line was exactly that, and it had
  said so in plain words for two iterations before anybody acted on it.
- **`alo-capability` now holds one thing that is not about what an agent may
  do.** `Agent` is whether there *is* an agent, and `Grants` lives inside it. No
  crate and no edge moved; the grants are where they always were, one type
  further in. It is here rather than in a crate of its own because a machine
  with no agent is the limiting case of what an agent may reach rather than a
  subject beside it — which is the opposite of the call `alo-keeping` and
  `alo-answering` got, and for the opposite reason: those two exist because
  something had to be able to do a thing the crate beside it promises never to
  do, and there is no such promise here to protect.
- **Nothing here has hidden a panel.** The whole visible half of ADR 0009 is
  item 17a and needs a shell. What exists is the model those screens will read:
  one question for whether the surfaces are there at all, one line for the place
  Settings still offers, and one act that ends every grant on the machine.

---

## 2026-09-03 — iteration 33: the queue is a true picture of v0.01, and it is empty

**Built: nothing, and that is the finding rather than the failure.** Every item
in `QUEUE.md`'s *Ready* section is ticked. The two unticked entries are 16b,
which is deliberately not ready, and 17a, which is under *blocked — linux*.
`QUEUE.md` says in its own words what happens here: *the loop takes ready items
and stops when there are none left.*

**The gate was run anyway, on unchanged code.** `cargo fmt --all --check` clean,
`cargo clippy --workspace --all-targets -- -D warnings` zero warnings and zero
errors, **1094 tests and 34 doctests, all green** — the same numbers iteration
32 left. An entry that stops the loop is making a claim about the state of the
workspace, and taking the previous iteration's word for it is exactly the case
`LOOP.md` names as a halt: *a test that used to pass has started failing*. None
has. `ROADMAP.md` was not moved, because nothing was built for it to report —
which is the one honest reason for leaving that file alone, and this entry says
so rather than passing over it in silence.

**What the reading step covered, since it was the whole of the work.** All of
`docs/features.md`, v0.01 line by line, against `QUEUE.md` and `ROADMAP.md`.
Every v0.01 promise now has an item:

| Promise | Where it lives |
|---|---|
| File verbs; application verbs; context on invocation; grants; every execution recorded | items 6/6a, 11/11a, 12, 1–4, 4a — built |
| Keyboard shortcuts; the dock | items 7, 13 — built |
| Egress indicator; no telemetry | items 5, 5a, 16, 16a — built |
| The model stack, a provider of your own, an API instead, never a silent fallback, or no agent at all | `alo-models`, items 10, 14, 17 — built |
| Compositor, sign-in, overlay, launcher, window management, copy and paste, window switching, the image | *blocked — linux* |
| The GPU on first boot, a real Ollama, the exit gate | *blocked — hardware* |
| Agents point at the local model | *not ours* — `alo-workplace` |

**Six unlisted v0.01 promises were found by seven previous iterations reading
that file. This one found none**, and unlike iterations 31 and 32 it read the
whole of v0.01 rather than the section its item lived in. That is as strong a
claim as the method supports, and it is still not a proof: what it means is that
whoever finds the next hole will find it in another tier, not in the v0.01 list.

- `LOOP COMPLETE` — written here by that iteration, and **discharged**: items 18–21 were added afterwards. Kept as a record behind a bullet so it reads as history rather than acting as a signal, which is how this journal refers to its own markers everywhere else.

**Everything that remains, and what each is waiting on:**

- **Not ready — 16b, discovery on the local network.** Whether multicast is an
  `Errand` or a documented exception to the no-telemetry claim. Nothing about
  discovery has been built since iteration 30 wrote the item, so deciding now
  would still be deciding in the abstract about a shape nobody has.
- **Blocked — linux.** The compositor and everything drawn on it: sign-in and
  the local account, the agent overlay, the launcher, window management, copy
  and paste, window switching, the workspace client, and the image itself. Plus
  **17a** (the surfaces a machine with no agent does not have), the **reading
  half of context on invocation** — Wayland and AT-SPI, and with it the one
  capability guarantee no portable test can make, *with no invocation,
  `alo-agentd` makes no context calls at all* — the **acting half of the
  application verbs** (D-Bus and the portal backend), **6b** (`openat` with
  `O_NOFOLLOW` and `renameat2` with `RENAME_NOREPLACE`), **4b** (where the
  record file lives and the timer that shortens it), and **egress enforcement at
  the network boundary**, without which item 5 describes only the code that
  asked.
- **Blocked — hardware.** The model stack against a real Ollama, the GPU on
  first boot, and the v0.01 exit gate: one person, one machine, one cold boot.
- **Not ours.** Agents pointing at the local model is `alo-workplace`, and this
  loop never touches another repository.

**What is left behind is thirteen crates, 1094 tests and no daemon.** The
capability model is working code — grants, verbs, approvals, the record and what
keeps it, egress with and without an agent behind it, the portable halves of the
file and application verbs, what an invocation offers, the dock, appearance, the
shortcuts, every string in the reader's own language, and the machine somebody
chose not to have an agent on. What none of it has ever done is open a window,
read a screen, put a question to a model, or open a connection.

**The decision now, and why the loop must not make it.** There is portable,
promised, testable work left — but all of it is **v0.5**, and `QUEUE.md` is
titled *Queue — v0.01*. `CLAUDE.md` gates scope to *this file, the current
release, and Non-goals*, so a loop that wrote v0.5 items into a v0.01 queue
would be widening its own scope on its own authority, which is the one thing a
build loop should never be trusted to do. Opening a v0.5 queue is a person's
call. Three candidates turned up while reading, listed so that call is informed
rather than a fresh survey:

- **Regional formats** (v0.5, *Language and access*). `alo-appearance`'s
  `words.rs` already says outright that how a number is written belongs to the
  region rather than the language, and that nothing here does it: `TextScale`
  shows `200%` and `TimeOfDay` shows `18:00` because those are what the settings
  file holds, not what a person in Finland reads. `alo-models`' `NotEnoughDisk`
  hands two `u64`s to a caller with nowhere to write them, `alo-keeping` has a
  retention window, `alo-dock` has measures. **Strings are finished — every
  crate in the workspace has crossed onto `alo-strings` — and numbers are the
  half of *hardcoded English is a bug* that nobody has started.** It is a data
  item, so item 9a's rule governs it: read CLDR rather than recall it, which
  means it needs the network the way 9a did.
- **The capture indicator** (v0.5 ★): *a visible indicator whenever the screen,
  camera or microphone is in use — by any application, including ours*. It has
  no queue entry, blocked or otherwise, and no `ROADMAP.md` *Built* clause.
  `alo-egress` is the proven shape — the decision and the showing are one call,
  and the token it hands back is the only thing a caller can hold — and item
  16's argument applies exactly: an indicator written *after* the capture code
  is one somebody can bypass. The decision it exists to make is whether it is
  law 1's indicator or a second one.
- **One list of what has been granted to what** (v0.5 ★): agents and
  applications in the same place, revoked the same way. ADR 0005 already says a
  portal request is a grant in ADR 0001's sense, so the question is whether that
  is a `Grantee` and a `Reach` or a second list joined only at a panel. The most
  valuable of the three and the heaviest: it moves a public surface every other
  crate reads.

**One smaller thing, and it was answered while this iteration was reading.**
*★ Where the answer came from is said where the answer appears* had no
`ROADMAP.md` line of its own — it rode on the provider line and the local-model
line — and this entry was going to say so as advice. Commit `d0a1baf` landed on
the remote first and wrote it, together with lines for *★ It runs on the machine
you already own* and *★ Or use an API instead*: two promises this reading step
had counted as covered because their **code** exists, without asking the
narrower question of whether the roadmap could name them. It also makes that
file's rule run both ways — every promise at a release now has a line there, or
is named on the line that carries it — which is this journal's own conclusion
turned into a rule rather than advice repeated once an iteration.

None of the three changes the finding above. What each owes is the setup screen,
the daemon and the overlay, so none is a ready item; and the last of them is
flagged on its own line as the likeliest of the three to be lost, because it is
a sentence that must appear every single time and nothing in this repository yet
forces it to.

**No `CHANGELOG.md` line was written, and the absence is deliberate.** That file
is what somebody outside this repository can read, and nothing changed for them
this iteration. A changelog entry announcing that the loop stopped would be the
loop reporting on itself in the one file that is not about it.

---

## The queue was refilled, and that `LOOP COMPLETE` above is spent

The entry above is correct about the moment it was written: every ready item was
done, and the only thing left was blocked. It stopped for the right reason.

**Four items were added afterwards** — 18 to 21, `alo-agentd` — and they are
what five roadmap halves have been waiting on the whole time. The gap is one
sentence the roadmap already said three times and nobody had turned into work:
*there is no method anywhere in this repository that puts a question to a
model.* Thirteen crates decide correctly and nothing joins them up.

All four are portable. No compositor, no certified machine, no GPU: a turn is a
function call and its result is a value to assert on. What genuinely needs
Wayland and D-Bus — the acting half of the application verbs, the reading half
of context — stays under **Blocked — linux**, unchanged.

A decision also landed while the loop was stopped: **ADR 0011**, the base is
rented and the image is a bootable container. It changes nothing here; it is
recorded so the next iteration reads it before proposing an alternative.

The loop may proceed from item 18.

---

## The queue was refilled — the `LOOP COMPLETE` above is spent

That entry is right about the moment it was written: every ready item was done,
the only remainder was blocked, and it stopped for the correct reason. It is no
longer a description of this queue.

**Items 18 to 21 were added afterwards — `alo-agentd`.** They are what five
roadmap halves have been waiting on the whole time, and the gap is a sentence
`ROADMAP.md` already said three times without anybody turning it into work:
*there is no method anywhere in this repository that puts a question to a model.*
Thirteen crates decide correctly and nothing joins them up.

**All four are portable.** No compositor, no certified machine, no GPU. A turn is
a function call and its result is a value to assert on. What genuinely needs
Wayland and D-Bus — the acting half of the application verbs, the reading half of
context — stays under **Blocked — linux**, untouched.

**Start with 18, and build the hosted provider path before the local one.** That
inversion is deliberate and is written into the item: ADR 0008 makes a hosted API
a first-class choice rather than a fallback, and it is the path that can actually
be exercised, since `ThisMachine` needs a runtime installed and a model
downloaded before it can say anything. **No key is required to pass the item** —
the tests drive a stub on a real socket, as `ollama.rs` already does. A real
provider is the machine half.

### Two decisions landed while the loop was stopped

- **ADR 0011 — the base is rented, and the image is a bootable container.**
  Read it before proposing an alternative base. It changes nothing in the crates.
- **ADR 0008 gained its missing half.** *Never a silent fallback* was written in
  one direction only; a provider that fails must not quietly become a local model
  either. **Neither is the other's fallback**, because neither is a degraded
  version of the other — both are complete ways to run alo OS and the choice is
  the person's. Item 18 must not contain a substitution of any kind.

### And the workspace now builds on Linux

Everything here had only ever been compiled on Windows. It has now been cloned
into a real Linux environment and gated there: **1,132 tests across 52 suites,
zero failures, `fmt` clean and `clippy` silent** on `x86_64-unknown-linux-gnu`.
Four tests exist there that Windows never ran. That is not hardware verification
and nothing may be ticked for it — but a crate that has never been compiled for
its target platform is a different kind of unknown, and that one is now closed.

The loop may proceed from item 18.

---

## 2026-09-03 — iteration 34: the first thing here that ever sent anything

**Built: item 18, putting a question to a model — the hosted provider first.**
`crates/alo-asking`, a new crate, and the sentence three iterations of this
journal quoted from `ROADMAP.md` is no longer true of a provider: there is now a
method in this repository that puts a question to a model, shows it leaving, and
brings back the answer, the departure and the failure.

**The gate.** `cargo fmt --all --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` zero warnings and zero errors, **1140 tests and 38
doctests, all green** (was 1094 and 34). The new crate is 39 unit tests, 7
integration tests — 4 against the real vocabulary in Greek, 3 through
`alo-egress` and into `alo-record` — and 4 doctests, two of them `compile_fail`,
both checked by unmarking them: **E0624, method `text` is private** and
**E0382, use of moved value**.

### A new crate, and the dependency graph decided it rather than taste

The item's own sentence is *`alo-egress` consulted before a socket opens*.
`alo-egress` depends on `alo-models`, so a method living beside the provider
would have had to invert an edge that exists for a reason — the crate that
decides about a departure reaches the crate that knows where an answer comes
from, never the other way. There was no version of this that stayed in
`alo-models`.

What that leaves is a crate whose whole value is an **order**, and it reaches
five others while nothing reaches it. It holds no decision of its own: where a
question may go is `alo-answering`'s, what may leave is `alo-egress`', what a
provider is called is `alo-models`', and every sentence a person reads around
the moment belongs to one of the three. `words.rs` is two strings, the shortest
list in the workspace, and it says why in its own module documentation.

### The decision the item did not contain: the departure comes back either way

`alo_record::Entry::left` is made from an `alo_egress::Departing` and from
nothing else — that is item 4's guarantee, and it is what makes an egress the
indicator never showed an entry nobody can write. Which means a crate that took
the line off the indicator itself would leave **the record of what left
impossible to write**, in the one crate that causes the largest egress this
product has.

Three ways out were available and two are wrong. Taking `&mut Record` would make
this the first crate in the workspace to reach `alo-record`, breaking *the
record observes and is reachable from none of the crates it observes*. Ending
the line and returning nothing would keep the graph and lose law 1's second
half. So `Asked` and `DidNotAnswer` both carry the departure and both have
`ended(&mut Indicator)`, which spends it — the caller keeps the entry and then
takes the line off:

```
record.keep(Entry::left(asked.departing()));
let answer = asked.ended(&mut indicator);
```

It is item 6a's *the authorisation comes back either way*, one crate on and
about a departure. The half of it that is easy to lose is the other path: **a
question that failed still left the machine**, so the departure comes back
there too. A machine that recorded only the questions that were answered would
report a quieter day than it had.

What it costs is a moment where the indicator shows an egress that has just
finished. The alternative is a moment of not showing one that is happening, and
only one of those two is a lie law 1 cares about.

### The rule is asked twice, and the second time is the one that counts

`Answering::chosen` asked `SourcePolicy` when the place was chosen, which may
have been at the start of a turn or when somebody answered an offer about a
question that had already failed once. `to_a_provider` asks again at the moment
the socket would open, and asks the wider rule — `EgressPolicy` made from that
same `SourcePolicy` rather than stated a second time.

That is item 3's *the grants are asked last, at the moment of execution, which
is where a revoked grant becomes immediate*, arriving at egress. It is also what
makes the refusal path testable rather than dead: an organisation that tightened
its rule between the two has a machine that sends nothing, and there is a test
per `SourcePolicy` variant that says so — each against an address nothing is
listening on, so that a question which *had* been sent would come back as a
different answer.

### A question and an answer are held the way a key is

ADR 0001 §7 names two things alo OS never keeps, and the first is the question a
person asked. `alo-record` keeps that by having no field for one and
`alo-answering` by holding no question at all — neither of which is available to
the crate that has to put one on a wire. So it is `alo_models::Secret`'s shape:
no `Serialize`, a `Debug` written by hand, and `Question::text` `pub(crate)`
with its one caller in the file that builds the request body. `Answer` is the
same, and for the same reason: an answer is made out of somebody's question and
whatever they had open.

**And the key still cannot be read out of `alo-models`.** `Secret::carried_by`
takes a request and gives it back with the header on it, so a second crate can
*send* a key without any crate being able to read one; `bearer` stays
`pub(crate)` with its `compile_fail` doctest intact, and `trying.rs` now goes
through the same door.

### `alo-answering` gained a seventh `WentWrong`, and it earns its place

`SentSomewhereElse`. That crate's bar for a new reason is *a different thing to
be told*, not a different thing to have happened, and this one clears it from an
angle none of the other six do: it is not a failure at the far end at all. It is
a refusal **alo OS made** — the address answered by pointing somewhere nobody
agreed to, and the question was not carried there. Telling somebody *nothing
usable came back* would hide the only thing that happened, which is that their
machine stopped it. `alo_models::NotTried::Redirected` is the same call about
testing a provider, made where the stakes were smaller.

### The honest edge, written here because a reviewer will find it

`alo-asking` checks that the permission and the provider are the same place by
comparing the `Answering`'s source against **what the person wrote down** — the
provider's name and the region they stated — and not against
`alo_models::Provider::source`, which additionally answers *is this address on
this machine*. A daemon that derives the `Answering` from the provider, which is
the only sane way a person's choice becomes one, cannot pair them wrongly. A
caller that builds both by hand can, and this crate would not catch it.

The tests are how it was found: `Provider::source` answers `ThisMachine` for
**any** loopback address, so a hosted provider cannot be stubbed on `127.0.0.1`
at all, and checking against `Provider::source` would have made the successful
hosted path — the main path — untestable on any machine without a second host.
The refusal that is kept is the one that matters most: the indicator line is
composed out of the source and the socket is opened out of the endpoint, so what
is checked is that the line names the place the permission was for. It is
`alo-files`' honest edge, and it is written into `hosted.rs` and `asking.rs`
rather than only here.

### `ROADMAP.md` moved, and three lines were written into

- **★ Or use an API instead** — the code half now names `alo-asking` and what it
  does; the machine half is a provider somebody pays for, answering a real
  question with a real key, plus the local path, which is code and is item 18a.
- **★ Where the answer came from is said where the answer appears** — the file
  called this the promise on its list most easily lost, *a sentence that must
  appear every single time, and nothing yet forces it to*. `Answer` has no
  constructor that does not take the source, so a shell holding an answer is
  holding the sentence. The machine half is still the overlay.
- **Agents point at the local model / ★ never a silent fallback** — its machine
  half said *there is still no method anywhere in this repository that puts a
  question to a model*, which was half of what is now true. It says the half
  that still is.

No half was ticked that was not whole, and no machine half was touched.

**What the next iteration must know:**

- **The queue's next ready item is 18a**, the same path to a model on this
  machine, written by this iteration as the cut. It is not a branch inside
  `to_a_provider` and must not become one: `ThisMachine` causes no egress, so
  there is no `Leaving`, no `Departing` and nothing for law 1 to show, and
  `alo-asking`'s door refuses it in words that say so. What it needs first is a
  method on `ModelRuntime` that puts a question to a model, which that trait has
  never had — it loads, unloads, fetches and lists. **ADR 0008 runs both ways**
  and neither path may substitute for the other.
- **Then 19, 20 and 21**, unchanged: a turn end to end, where the record is
  written, and the daemon.
- **The reading step found no unlisted v0.01 promise.** What it read was
  `docs/features.md`'s AI-stack section and all three `ROADMAP.md` lines this
  item touches, against the queue. The standing advice holds: only
  `docs/features.md` is a list of what was promised, and a `ROADMAP.md` line
  whose *Built* clause cannot name a crate is the strongest available signal
  that the queue is missing an item.
- **Nothing here has been run against a provider anybody pays for.** Everything
  is a stub on a real socket, which is what `alo-models`' own tests do and is
  worth exactly what that is worth: the wire is real and the far end is ours.
  `docs/quirks.md` gained two entries — what a provider's status code means when
  a *question* fails, and ureq sending a pretty-printed body — and both of them
  say plainly that they have not been checked against a live service.

---

## 2026-09-03 — iteration 35: the question that goes nowhere

**Item 18a — the same path to a model on this machine.** `alo-models` gained
`ModelRuntime::answers`, the method that trait has never had, plus
`RuntimeError::TookTooLong` and its word; `ollama.rs` carries a question over the
runtime's own `/api/chat`. `alo-asking` gained `locally.rs` — a second door — and
`NotAnswered` beside `NotAsked`. 96 unit tests in `alo-models` (was 90), 48 in
`alo-asking` (was 39), 3 new integration tests, 1 new `compile_fail` doctest.
**1157 tests and 39 doctests across the workspace**, `cargo fmt` clean,
`cargo clippy --workspace --all-targets -D warnings` clean.

### The two doors divide on law 1, not on what speaks at the far end

The item's own warning was *do not grow a branch inside `to_a_provider`*, and the
useful form of that turned out to be a sentence about what the two paths actually
are. It is tempting to read them as *hosted* and *local*, which makes the local
one look like the hosted one with the network taken out — and then every
difference is an omission somebody could put back for convenience.

They are **the path where something leaves** and **the path where nothing does**,
and that is the only difference that matters:

| | `to_a_provider` | `to_this_machine` |
|---|---|---|
| Steps | Four | Two |
| The indicator | Shown before a socket opens | Not a parameter |
| The rule in force now | Asked again, and can refuse | Nothing to ask |
| What comes back | `Asked` — an answer **and** the departure | `Answer` — there is no departure |
| Refusals | `NotAsked`, four | `NotAnswered`, two |

`NotAnswered` is `NotAsked` with law 1's two variants removed, and **the two
missing variants are the zero-egress claim**. There is no `CannotBeShown` because
nothing is shown, and no `HeldBack` because nothing can be held back. A type with
either would be a type that believed something might leave.

### The refusal that had to change, and 18a is what made it wrong

`Miswired::NotAProvider` said *ask the runtime instead, and do not send it here*
for both `ThisMachine` **and** `PairedMachine`. That was written in iteration 34
and was harmless then, because there was no runtime path for it to be advice
towards. It is not harmless now.

For `ThisMachine` it is correct and is routing: the runtime is the place the
permission names. For `PairedMachine` it is **a substitution ADR 0008 forbids**,
worded as helpfulness — the person chose a machine in the next room, and the
sentence tells whoever is wiring it to answer them from this one instead.

So `Miswired` is five variants now, and the rule they are all held to is a test:
each names the door the permission's own place is behind, and the one place with
no door offers neither of the other two. `NoPathToAPairedMachine` is that place.
It is not a stub — there is nothing half-built behind it — it is the honest
answer that this repository cannot carry a question to a paired machine, said
where somebody would otherwise have to find out.

### The decision the item asked for, and it has two halves

*Whether an OpenAI-compatible provider a person pointed at loopback is the
runtime's path or `alo-asking`'s.*

**What it means: this machine.** `Provider::source` already answers
`ThisMachine` for any loopback address, nothing leaves, law 1 shows nothing, and
an answer from it says *on this machine*. Which door a question takes is decided
by whether it leaves — not by what shape the far end speaks.

**What it is: not the runtime.** alo OS cannot list, fetch, load or remove models
on a service it did not install. A `ModelRuntime` implementation whose four
management methods only refuse would be a stub wearing an interface, which law 3
forbids, and it would make the trait mean less everywhere else.

So the door takes the runtime alo OS ships, and the other local shape is **item
18b**, written into the queue with the reason it is its own item: whatever
carries it reaches `hosted.rs`'s request shape *without* a `Departing`, which is
a second road to a socket in the crate whose whole design is that there is one.
It must police loopback itself, and that check is the item.

### The thing that came with it, and is now written down

**Loopback is taken at face value**, everywhere in this repository. A process
listening on `127.0.0.1` that forwards a question off the machine would be
believed by `Provider::source`, by `Leaving::asking`, by the policy and by both
doors, and the person would read a quiet indicator.

Nothing is done about it in code, deliberately. Refusing loopback breaks the
ordinary case ADR 0007 makes the default; inspecting what is listening is a guess
about a process. The place it is caught is **egress enforcement at the network
boundary**, which is already a Linux item, and law 2 is what keeps the hole small
— an agent cannot start the proxy. `docs/quirks.md` has it, because a promise
with a hole nobody wrote down is a promise somebody will discover.

### Three smaller decisions, each of which could have gone the lazy way

- **The catalogue gates downloading and does not gate asking.** The licence
  promise in `docs/features.md` is about what alo OS *offers*, which is what it
  fetches. A model already on somebody's disk was either fetched through that
  gate or put there by the person whose machine it is, and refusing to ask it
  anything would be this system overruling the owner of the machine about their
  own hardware. `ollama.rs`'s module documentation used to say it does not offer
  a model the catalogue does not list; it now says which of the two calls that is
  true of, and why.
- **A slow model is not a missing one.** Every ureq error in `ollama.rs` used to
  become `RuntimeError::Unreachable`. On a question that would tell somebody
  *nothing was running* about a machine that was thinking — and ADR 0007 makes
  the CPU the default, so thinking for minutes is the ordinary case. Hence
  `TookTooLong`, one new variant and one new word. A timeout on a *listing* stays
  `Unreachable`, and the comment on `QUICK_TIMEOUT` says why: ten seconds for a
  local read means something is wrong.
- **This machine waits longer for itself than for anybody else.** Five minutes
  for the runtime against two for a provider, for the same ADR 0007 reason.

### No new string, and that is the finding rather than an omission

`alo-asking` still declares two words, the shortest list in the workspace. A
second door and nothing to say: *on this machine* is `alo-models`', *nothing
answered on this machine* is `alo-answering`'s, and `Miswired` keeps its English
because its reader is whoever wired the door. A list that grew with every path
would be a list that had started saying things twice, which is the failure the
9-series spent six items removing.

### `ROADMAP.md` moved, and three lines were written into

- **Model stack: catalogue, pull, serve, unload, remove** — the *serve* in that
  line was a heading over a trait that could not be asked anything. Its code half
  now names `ModelRuntime::answers` and the adapter that carries it.
- **★ Or use an API instead** — the code half now says all three of ADR 0008's
  places that this repository can reach are reachable, and the machine half says
  plainly that the third place, a machine on your network, has no path here at
  all and that both doors refuse it.
- **Agents point at the local model / ★ never a silent fallback** — its machine
  half said *the local model being asked at all* was still owed. It is not. The
  code half now records that the ADR's both-ways rule is carried by the code that
  would have had to contain the fallback, in both directions.

No half was ticked that was not whole, and no machine half was touched.

**What the next iteration must know:**

- **The queue's next ready item is 19**, a turn end to end and headless: the item
  that makes the other fourteen crates one system. Both ways of getting an answer
  now exist, so the model half of that turn is a call rather than a gap. 18b is
  ready and is nobody's blocker; 16b is still *not ready* rather than blocked,
  for the reason it states.
- **`Miswired` is a public surface that changed shape**, not only grew:
  `NotAProvider` narrowed to mean `ThisMachine` alone. Nothing outside this
  repository builds against it yet, and the `CHANGELOG.md` line is about what a
  person can do rather than about the enum.
- **Nothing here has been run against a real Ollama**, on any machine. Everything
  is a stub of the trait or a stub on a real socket, and `docs/quirks.md` gained
  two entries — the runtime's two chat APIs and what its 404 means, and the
  loopback one above — both of which say outright what has not been checked.
- **A `ModelRuntime` implementor now has seven methods.** A second implementation
  (vLLM, at v1) is a bigger job than it was this morning by exactly one method,
  which is the trait doing its job rather than a cost.

---

## 2026-09-03 — iteration 36: the door with no indicator on it, and the rule it stood on

**Item 18b — an OpenAI-compatible service somebody runs on this machine.**
`alo-asking` gained a third door: `served.rs` (`Served`, and
`Asking::to_a_service_on_this_machine`), plus `openai.rs`, which is the wire
lifted out of `hosted.rs` so that two things speaking one convention are not two
renderings of it. `alo-models` gained `address.rs`, which is the security half
and was not in the item. `alo-answering` narrowed one refusal. 64 unit tests in
`alo-asking` (was 48), 103 in `alo-models` (was 96), 2 new integration tests
through `alo-record` and the indicator, 1 new `compile_fail` doctest checked by
unmarking it (E0382, not a typo). **1183 tests and 40 doctests across the
workspace** (was 1157 and 39), `cargo fmt` clean,
`cargo clippy --workspace --all-targets` clean with zero warnings.

### The item named the danger, and the danger was one level below where it pointed

The item's warning was right: this door reaches a socket **without** an
`alo_egress::Departing`, which is a second road in the crate whose whole design
is that there is one. A `Served` pointed at `https://api.mistral.ai` would be a
way to send somebody's question to a provider with the indicator quiet. So the
address is policed at construction — `Served::at` refuses anything
`alo_models::Provider::source` does not call this machine, and there is no other
constructor. *What may be reached without an indicator is decided by whether a
value exists*, which is `alo_files::Touching` and `alo_egress::Departing`'s shape
arriving at the one path in this crate that had no token of its own.

Writing that check meant reading the rule it delegates to, and **the rule did not
hold.** `Provider::source` asked whether the address *started with* `127.0.0.1`,
`localhost` or `::1` once the scheme was stripped. So:

| Address | Was | Is |
|---|---|---|
| `http://localhost.attacker.example` | this machine | somewhere else |
| `http://127.0.0.1.attacker.example` | this machine | somewhere else |
| `http://127.0.0.1@attacker.example/` | this machine | somewhere else |
| `http://127.0.0.2:8000` | somewhere else | this machine |

The first three were already wrong before this item: each could be added as a
provider over unencrypted `http://` **with a key attached**, because
`Provider::checked` permits http only to this machine and believed them. Once
this door existed they would also have been a question leaving with the
indicator quiet, which is law 1 failing in the exact manner law 1 exists to
prevent. `alo_models::address` now takes the authority apart the way a URL is
written — scheme, then up to the first `/`, `?` or `#`, then whatever follows the
**last** `@`, then the host with its port off — and matches the host whole.
`is_loopback` is gone.

**The fix is in `alo-models`, not in the new door, and that is the decision.** A
loopback check written in `alo-asking` would have been a second rule about
loopback, and two rules about loopback is one machine able to disagree with
itself about whether a question left. There is one, and the three places that
rest on it — http being permitted, the indicator being silent, and this door
opening at all — ask it rather than repeat it.

### Three things this changed that the item did not mention

- **`WentWrong::KeyNotAccepted` is no longer impossible on this machine.**
  `alo-answering` refused that reason for `InferenceSource::ThisMachine`
  outright, on the reasoning that only a hosted provider is given a key. A
  service somebody started with `--api-key` breaks that, and it is *their own
  machine* — so the refusal narrowed to a paired machine, where nothing in this
  repository reaches at all. The runtime's half of the guarantee moved to
  `locally.rs`, where it is total rather than approximate: no arm of
  `what_went_wrong` can produce that reason, and a test walks every
  `RuntimeError` to say so. A guarantee carried by the absence of a branch beats
  one carried by a check that has to keep being true of a whole variant.
- **`Miswired::NotTheRuntime` is now `NotOnThisMachine`.** One variant for both
  local doors, because what it refuses is the same thing at each — answering here
  a question somebody chose a provider for — and the old name became false the
  moment the runtime stopped being the only thing here that can answer. A public
  surface renamed rather than duplicated; nothing outside this repository builds
  against it yet.
- **`127.0.0.2` moved sides**, which broke two test fixtures that used it to mean
  *a hosted provider nothing is listening on*. They are `0.0.0.0:1` now — still
  no name lookup, still a refused connection, and no longer inside `127.0.0.0/8`.

### What was deliberately not built

A `ModelRuntime` implementation for such a service, which is item 18a's answer
kept: alo OS cannot list, fetch, load or remove models on something it did not
install, so it would be four methods that only refuse. `Served` takes an address
and a key, and manages nothing.

### `ROADMAP.md` moved, and two lines were written into

- **★ Or use an API instead** — the code half now names the third door and says
  what makes it safe, and the test count on that line went from 57 to 73.
- **Add your own provider in Settings** — its code half claimed *https required
  off this machine*, which was true of the sentence and not of the check. It now
  records why that is true rather than approximately true.

No half was ticked that was not whole, and no machine half was touched.

**What the next iteration must know:**

- **The queue's next ready item is 19**, a turn end to end and headless. All
  three of ADR 0008's reachable places now have a door, so the model half of that
  turn is a call rather than a gap.
- **Read the rule before depending on it.** This item's own check would have
  passed against a broken `Provider::source`, and the queue would have recorded a
  guarantee that was not there. The tests that matter here assert against
  *addresses*, not against the function.
- **Nothing here has been run against a real vLLM, llama.cpp server or LM
  Studio**, on any machine. Everything is a stub on a real socket, and
  `docs/quirks.md` gained one entry — two addresses that really are this machine
  and are deliberately treated as somewhere else — which says the same thing
  about what has not been checked.

## 2026-09-03 — iteration 37: the order the other crates happen in

**Item 19 — a turn, end to end, headless.** `crates/alo-turn`, a new crate:
`machine.rs` (what every turn on this machine happens against, and what it can
carry out), `turning.rs` (the turn and its five doors), `carrying.rs` (from
*this may run* to *this is what happened*), `kept.rs` (where a turn writes what
happened down), `refusing.rs` (the seven things that can come back instead),
`words.rs` (one phrase), `testing.rs`. 31 unit tests, 9 integration tests — 5
against the real vocabulary in Finnish, 4 through a real filesystem with the
record written to a real file and read back by `alo-keeping` — and 1
`compile_fail` doctest checked by unmarking it (E0382, not a typo). **1223 tests
and 41 doctests across the workspace** (was 1183 and 40), `cargo fmt` clean,
`cargo clippy --workspace --all-targets` clean with zero warnings.

### The item said *a decision*, and the answer is that it is not ours

The chain the item named has one step this repository does not own. A model's
answer becoming a verb and some arguments is the **agent's** work, and an agent
is a client of `alo-agentd` rather than a part of it — item 21's protocol takes
enumerated verbs with typed arguments, so this crate is what sits behind that
protocol rather than what composes requests to it.

That turned out to make law 2 stronger rather than smaller. A turn takes a name
and a value per argument and makes the call **itself**, against the closed list
the machine offers; there is no door anywhere in the crate that accepts a
`Call`. So *what an agent can ask for is what the registry holds* stops being a
rule about what a model may send and becomes the absence of a second way in.

### The guarantee it exists for, and the window it cannot close

`CLAUDE.md`'s gate asks that *every execution and every refusal leaves a
record*, and until now that was a sentence somebody had to remember at each of
five call sites. It is the shape of the code now: a `Turning` cannot be made
without somewhere to keep its record — `Machine` takes a `Kept` and there is no
constructor that does not — and every door writes its entry before it answers.

What that cannot close is real and is written down rather than papered over: a
change has already happened on the disk before there is anything to write about
it, so a record that fails after that is a thing that happened with no evidence
of it. The answer is that **a turn that could not write something down does
nothing else** — every door afterwards answers `NotDone::TurnClosed`, and a
daemon meeting it has a machine to halt rather than a call to retry.

### Three things the item did not contain

- **A machine offers exactly the verbs it can carry out.** `Machine` builds the
  registry rather than receiving one, so the offered list and the executable
  list are one list. A registry handed in could hold an application verb — those
  are declared, portable, and unreachable until Wayland and D-Bus exist — and an
  agent asking for one would be told *the machine could not*, which is a
  sentence about a full disk rather than about a capability this machine does
  not have. It is also why the constructor is named
  `carrying_out_file_verbs`: the day there is a second executor the name is
  wrong until somebody fixes it.
- **A question put to a person is not a thing that happened.** Proposing writes
  nothing; what the record keeps is the answer. A change nobody answered goes
  away with the turn and leaves no entry, because *the person did not answer*
  would be the record starting to keep the person rather than the agent — item
  17's refusal about turning an agent off, met from the other side. A person who
  says **no** has acted, and that is `Entry::declined`.
- **`Kept` is a trait with two implementations and one of them is real.** A turn
  holding an `alo_keeping::Writing` directly would make the promise true and the
  crate untestable without a disk; one holding an `alo_record::Record` would make
  every test pass and every real machine lose its evidence at shutdown. It is
  `alo_files::Resolving`'s shape for the same reason.

### What the tests found that the design did not say

A path that is not there comes back as a **refusal**, not as *the machine could
not*: `alo-files` asks the grants about a path as written before resolving it
(item 6), so once they have said yes, *there is nothing there* is answered by
the resolver and arrives as `Refused::worded_elsewhere`. The first version of
one test assumed otherwise. There are two tests now, named for the difference,
because they read alike and are different facts about a machine.

`docs/quirks.md` gained one entry: on Windows a record file whose folder has
been removed **goes on accepting writes** through the open handle, so there is
no portable way to make a real disk refuse one. The closing is therefore tested
against a `Kept` that refuses everything, and the integration test asserts the
half a real disk can answer — that every entry is on the disk before the door
that made it answers.

### `ROADMAP.md` moved, and two lines were written into

- **`alo-agentd`: grants, file verbs, application verbs, context on
  invocation** — the code half now names `alo-turn` and what it makes true: the
  four crates joined into one order that cannot be taken out of sequence.
- **Every execution recorded with its origin, approval and grant** — its code
  half now records that *recorded* is structural rather than remembered.

No half was ticked that was not whole, and no machine half was touched.

**What the next iteration must know:**

- **The queue's next ready item is 19a**, the question a turn puts to a model —
  cut from this one on the line law 1 draws, since everything in 19 happens on
  this machine. It has two decisions in it and neither is wiring: whose the
  `Indicator` is, and what a turn does with an `alo_answering::Failed` — an
  offer only a person can take, arriving in the middle of a turn holding a
  grant. **19b** is the application half and is blocked on Linux.
- **`alo-turn` has no doctest that runs**, only a `compile_fail` and its named
  twin among the unit tests. A worked example needs a temporary folder, and a
  doctest that writes to one would be the first in this workspace to do so.
- **Nothing here has been run on a certified machine**, and **no disk has yet
  refused a write to a record** — the one refusal path in this crate that only
  real hardware can produce.

## 2026-09-03 — iteration 38: the question a turn puts to a model

**Item 19a — the question a turn puts to a model.** Four new files in
`crates/alo-turn`: `answers.rs` (the three things that can answer, and the door
each is behind), `places.rs` (the rule in force now, and everywhere else the
person set up), `asking.rs` (the door, and what is written down on each of its
roads), `unanswered.rs` (`NoAnswer` — the seven things that can come back
instead). `Machine` gained the indicator and a way to read it; `Turning` gained
`asking`, `showing`, and one road to the record that answers with
`alo_keeping::NotKept` rather than with a turn's refusal. 25 new unit tests, 2
new integration tests against a real socket with a real record file, 1 new
`compile_fail` doctest checked by unmarking it (E0382, not a typo). **1248 tests
and 42 doctests across the workspace** (was 1223 and 41), `cargo fmt` clean,
`cargo clippy --workspace --all-targets` clean with zero warnings.

### The item's first question, answered as it asked it

**The indicator is the machine's**, beside the record, and for the record's
reason: one machine has one of each and a second would be a second place to
look. Item 16 settled what that surface is for — what alo OS does on its own
goes on the same list as what an agent causes, because the failure law 1 exists
to prevent is not *the policy was wrong* but *nobody could see it* — and a turn
handed an indicator of its own would put two turns on one machine on two lists.
`Machine::showing` lends it back, so a shell can draw law 1's surface at the one
moment it matters, which is during a turn.

What is deliberately **not** on the machine is `alo_models::SourcePolicy`. An
organisation can tighten a rule while a turn is open, so it arrives at the door
at the moment a question is asked — item 3's rule about the grants, met at
egress.

### The second question had a shorter answer than it looked

**A turn does nothing at all with an `alo_answering::Failed`.** Holding it under
a number, the way a proposal is held, is the obvious shape and is wrong twice.
A person reading *nothing answered — shall I ask the provider instead?* may take
longer than the turn lasts, so a turn that held the offer would either expire it
or extend itself, and the second is a grant outliving the invocation that made
it. And the guarantee is already structural one crate down: `Failed::take` needs
an offer from that failure and spends the failure doing it.

So the failure comes back whole and belongs to nobody — it holds no grant, no
context and nothing of this machine — and the `Answering` a person's *yes*
produces walks in at **the same door**, shown and written down exactly as the
first attempt was. A second attempt needed no second method, and the test that
says so is the ADR 0008 one: the place that failed was asked once, the place
that was offered was asked nothing at all.

### The decision neither question contained

**The permission comes in rather than being made here**, and the reason is the
record. A turn that built its own `Answering` would ask `SourcePolicy` about the
place before the egress rule ever saw it, and the two refusals are not
interchangeable: only the second makes an `alo_egress::NotPermitted`, and
`Entry::held_back` is made from one of those and from nothing else (item 5a). A
machine whose rule was tightened between the person choosing and the agent
asking would then have refused the question **with no record of having refused
it**. So the rule is asked where its refusal can be written down.

That is not law 2's *no door takes a call* being weakened. That rule is about
what an **agent** may ask for; an agent does not choose where its question is
answered, ADR 0008 says the person does, and their decision arrives here the way
the grants arrive at every other door.

### Three things the item did not contain

- **The four absences are argued one at a time.** What is written down is what
  left, or what a rule stopped from leaving. Nothing is written for a question
  that never formed, a provider whose name cannot be drawn, a permission and a
  place that disagree, or a question this machine could not answer — and the
  last of those is `alo-answering`'s own decision followed rather than made
  again: an entry per failure would build a log of somebody's questions failing
  one honest entry at a time, and `Happened::AnsweredHere` would be a lie about
  a question nothing answered.
- **The line comes off the indicator whatever happens**, including when the
  record cannot be written. The indicator is a statement about *now*, so leaving
  a line up in order to signal that the record is broken would make law 1's
  surface wrong on purpose. What breaks is the turn.
- **A refusal that closes a turn says whether anything had already left.**
  `NoAnswer::NotRecorded` carries it, because *the record broke* and *somebody's
  question went to a provider and there is now no evidence of it* are two
  different mornings for whoever reads the machine, and only one of them is
  about evidence that is missing of something.

`ROADMAP.md` moved, and two code halves were written into — **★ Or use an API
instead**, which now records that a turn is what reaches those three doors, and
**Agents point at the local model by default / ★ never a silent fallback**,
which now records that the same is true one level up, where a fallback would
actually have been written. No half was ticked that was not whole, and no
machine half was touched.

### The checkout was reset under this iteration, twice

`git reflog` shows three `reset: moving to origin/main` during this iteration,
which discarded every edit to a tracked file while leaving new files alone; the
work was reapplied from scratch each time. Three documentation commits arrived
on `origin/main` in the same window (ADR 0014 and the ADR 0007 correction),
authored by the repository's owner, so something else was working in this tree
or pushing into it.

`CLAUDE.md` forbids exactly this — **one agent per working tree** — and it is
not something an iteration can work around: a reset landing between the gate and
the commit would have thrown away a finished item silently. This one did not,
because the commit went in immediately after the gate. **The supervisor has to
stop resetting a tree an iteration is working in**, or give each iteration a
tree of its own. It is written here rather than as a `LOOP HALT` because the
item was finished and committed; the next iteration should halt if it happens
again.

**What the next iteration must know:**

- **The queue's next ready item is 20**, where the record is written and what
  prunes it — the path, the retention the organisation sets, and the timer.
  **19b** is the application half and is blocked on Linux. Item 23 arrived on
  `origin/main` during this iteration and has not been read against the ready
  list.
- **`Machine::carrying_out_file_verbs` takes four things now**, the indicator
  among them, and every caller in and out of the crate was updated. The name is
  still right: asking a model something is not a verb and asks the grants
  nothing, so it is a door on `Turning` rather than a row in the registry.
- **Nothing here has been run against a provider anybody pays for, or against a
  real model runtime.** Every question in these tests went to a stub on a real
  socket or to a stub of the runtime trait.

## 2026-09-03 — iteration 39: what a client may ask the daemon

**Item 21a — the request half of item 21, cut from it and built.**
`crates/alo-protocol`, a **new crate**: `frame.rs` (one message — one line, a
length, a format number), `asked.rs` (the closed list of everything that can
arrive), `agent.rs` (`FromAnAgent`), `person.rs` (`FromAPerson`),
`argument.rs` (one argument exactly as it arrived), `refusing.rs`
(`NotUnderstood` — the seven ways a message is not a request), `words.rs` (7
phrases), `testing.rs`. 45 unit tests, 14 integration tests and 1 doctest.
**1307 tests and 43 doctests across the workspace** (was 1248 and 42),
`cargo fmt` clean, `cargo clippy --workspace --all-targets` clean with zero
warnings. `docs/contracts/daemon-protocol.md` is new.

### Why item 21 became three items

Items 16b, 19b and 20 were read first and skipped, each for a reason written in
the queue: 16b says outright that it is *blocked on nothing here, and not ready
either* — there is no discovery code to decide about; 19b waits on the acting
half of the application verbs, which is Wayland; and 20 needs a long-lived
process with a timer in it. So item 21 was the first ready one.

It is three items now, and the line the first cut falls on is **what a caller
can say**. Every sentence item 21 wrote for itself is about the request —
*enumerated verbs with typed arguments*, *no request that carries a command*, *a
malformed request refused in the reader's own language* — because that is where
law 2 meets code somebody else wrote. What comes back carries no such property
and needs a decision this item did not contain (**21b**, and the first thing it
decides is whether `alo_files::Answer` should serialise at all, since it holds
paths and a path is not always text). The process, the socket and peer
credentials need a Linux host (**21c**, under *blocked — linux*).

That is not a smaller item. A crate whose responsibility is *reading the
untrusted side of a socket* is one responsibility, which is law 4 rather than a
convenience, and it is the half that decides whether the other two are safe.

### The decision the item did not contain

**An agent must not be able to approve its own change**, and one `Request` enum
with five variants — which is the obvious shape, and the shape the item's own
wording suggests — is a hole. A socket that both a shell and an agent speak over
would let the side that proposed a change also answer it, and ADR 0001 §5 would
be true of `alo-capability` and false of the door in front of it.

So the closed list is one `pub(crate)` type and there are two public doors cut
out of it: `FromAnAgent` takes the three an agent asks during a turn,
`FromAPerson` the two a person answers with, and neither can produce the other's
requests. A sixth request has to be given to one door or the other before the
crate compiles, which is what makes it a division rather than two lists that
drift.

What it deliberately does **not** claim: which side a connection is really on is
peer credentials on a Unix socket, and that is 21c's. What is settled here is
that once the daemon knows, no message can cross.

### Four things are not on the wire, and each is an absence rather than a check

- **No moment.** Every door in `alo-turn` takes `now` from the machine. A
  request that named one could revive a grant that expired an hour ago.
- **No context.** ADR 0001 §4 — the compositor answers what the invocation
  offered, at the moment the key was pressed. A request carrying a document
  would be an agent handing itself the grant it wanted.
- **No place a question is answered.** ADR 0008 puts that with the person.
- **No turn.** The connection answers that. A number for it would be a number an
  agent could change.

Nothing begins a turn and nothing ends one either, for the same reason: both are
somebody else's act.

### Three smaller decisions the next items inherit

- **Arguments are a list and not an object.** A JSON object has no duplicates,
  so `{"file": …, "file": …}` would arrive as one `file` with the reader having
  silently chosen which — in the one place a person's approval sentence is built
  from. As a list both arrive, and `CallError::SameArgumentTwice` stays
  reachable. The wire shape exists to keep an existing refusal reachable.
- **The format is read before the message**, out of a shape that tolerates
  fields this version has never heard of, so a client from a newer alo OS is
  told to update the machine rather than told its message was gibberish.
  `docs/contracts/daemon-protocol.md` says a new request is additive and does
  not raise the number, which is `record-file.md`'s argument about a new kind of
  entry made again.
- **No refusal quotes the message back**, and the shape that keeps it true is
  that not one of the seven sentences has a gap in it — a gap is the only road
  text off a socket could take into a sentence a person reads. It is
  `alo-record`'s *the arguments of a call that never validated are never kept*,
  one step before there is a call. The two numbers a reader might want are
  fields on the refusal.

### What the tests found that the design did not say

The integration test carries a JSON line through a real `Turning` onto a real
filesystem, and it is what stops this crate being a description of a protocol
rather than the protocol: a verb name arrives as `/bin/sh` and is turned away by
the closed list rather than by anything here, an argument named twice reaches
`Verbs::call` and is refused there, and an approval arriving on the agent's door
leaves the file exactly where it was with nothing written down — because a
message that was never a request has not done anything to the machine.

`docs/quirks.md` gained nothing. The one thing worth knowing is already in it:
a resolved path on Windows carries a prefix, so the fixture grants over the
resolved folder and the test writes the path into JSON escaped.

### `ROADMAP.md` moved, and one code half was written into

**`alo-agentd`: grants, file verbs, application verbs, context on invocation** —
the code half now names `alo-protocol` and what it makes true, and its machine
half now names 21b and 21c rather than *the daemon itself*. The record line's
machine half said *queue 4b*, which was renumbered to 20 eight iterations ago;
it says 20 now. No half was ticked that was not whole, and no machine half was
touched.

**What the next iteration must know:**

- **The queue's next ready item is 22**, *running out is not a fault* — a
  provider answering *payment required* has no variant in `alo-answering`, so it
  arrives as a key problem or as a number. **21b** is the answering half and is
  ready; it is listed after 22 because it is a decision about serialising
  `alo_files::Answer` and 22 is not blocked behind it. **21c** and **20** are
  blocked on Linux and on a long-lived process.
- **`alo-protocol` opened no socket**, listened on nothing, and holds no
  transport. Everything in it is a `&str` in and a value out.
- **Nothing here has been run on a certified machine**, and no client that is
  not a test has ever spoken to this format.
## 2026-09-03 — iteration 40: what the daemon answers with

**Item 21b — the answering half of the protocol, and the decision the item
existed to make.** Eight new files in `crates/alo-protocol`: `told.rs` (the
closed list of everything that goes back), `to_an_agent.rs` and `to_a_person.rs`
(the two doors), `done.rs` (`alo_files::Answer`'s six as they cross), `thing.rs`
(one thing in a folder), `standing.rs` (a change waiting, with the sentence it
waits on), `wording.rs` (a sentence and whether anybody translated it),
`naming.rs` (the rule a path is held to). `frame.rs` gained the answer envelope
and `LONGEST_ANSWER`; `asked.rs` and `person.rs` gained the `waiting` request;
`refusing.rs` and `words.rs` gained two refusals. `alo-files` made `MOST_READ`
and `can_be_shown` public. 89 unit tests in `alo-protocol` (was 45), 22
integration tests (was 14), 1 doctest extended. **1358 tests and 43 doctests
across the workspace** (was 1307 and 43), `cargo fmt` clean, `cargo clippy
--workspace --all-targets -- -D warnings` clean with zero warnings.

Items 16b, 19b and 20 were read first and skipped for the reasons the queue
gives — 16b is *blocked on nothing here, and not ready either*, 19b waits on
Wayland, 20 on a long-lived process. The previous entry said the next ready item
was 22; it is 21b, which stands above 22 in the file and is not blocked. The
loop's rule is the first item that is not done and is not blocked, so it was
taken in that order.

### The decision the item asked for: no `Serialize` on `alo_files::Answer`

Three reasons, and the third decided it.

A derived `Serialize` on a `PathBuf` **fails** on a path that is not UTF-8. So
the road that works for everybody's files is a road that errors on somebody's,
and what that person's shell shows them is not *this file has an unusual name*
but whatever a daemon does with an answer it cannot write down: a read that
succeeded, arriving as a failure.

It would also put the wire's shape inside the crate that touches the disk. The
format is a public surface, and a crate whose job is `std::fs` should not be a
crate a protocol change has to be made in — item 4's argument for `alo-record`
being its own crate, met from the other end.

And `alo-files` had already decided what to do with text it did not write.
`Named` refuses a name that could rewrite what an answer appears to say, and the
listing **counts** what it left out. A path in an answer is the same text with
the same problem, so it is the same rule asked one crate further out —
`can_be_shown` is public now rather than copied — and the count travels beside
every list. A change that names one path is still reported as the change when
that path cannot be shown: the file really did move, and saying it failed would
be untrue about the disk.

**A file's contents are deliberately not held to that rule.** Contents are
contents rather than a name inside a sentence, and a read that refused a file
with a tab in it would be a verb that works on prose and nothing else. What
keeps it safe is the format rather than a check: JSON escapes a control
character, so a file with line breaks still crosses as one line. There is an
integration test that reads a real file with two of them.

### What the tests found that the design did not say: two bounds cannot be one

`alo-files` bounds a read at a megabyte. `frame.rs` bounded a message at a
megabyte. So the largest legitimate answer this machine can produce did not fit
in a message at all — and with JSON escaping, which writes a control character
as six bytes, a worst-case file is six megabytes on the wire.

One bound for both directions would therefore have meant a verb that succeeded
and a message no client is allowed to read. `LONGEST_ANSWER` is 8 MiB and is
**derived** from `alo_files::MOST_READ` rather than chosen beside it;
`MOST_READ` is public so the derivation is checkable from the crate that depends
on it, and `a_worst_case_read_fits_inside_the_bound` builds the worst case and
measures it rather than asserting the sentence.

### Three decisions the next items inherit

- **A sentence crosses with where it came from.** The daemon holds the
  vocabulary, so the daemon renders — and text alone would have put item 9's
  hole back at the last boundary before a person reads a sentence, for every
  string in the workspace at once. `Wording` carries `translation`,
  `the-source` or `no-sentence`, read off `Said` rather than judged again, so a
  shell can mark English shown in a Latvian session exactly as a development
  build does.
- **The answers divide by side as the requests do.** The requests divide because
  an agent must not approve its own change. The answers divide because
  `Turning::waiting_at` is a method a daemon holding an agent's connection can
  call: one public answer type is one where writing the person's own list onto
  that connection compiles. `ToAnAgent` has no shape for it, so it does not.
- **A refusal crosses as a sentence and carries no kind.** No code, no variant,
  no name of the crate that made it. A client that could branch on which refusal
  it was is a client that would, and an agent choosing what to try next from
  *the grants said no* rather than from the sentence is an agent working around
  the capability model.

### Two things the item named and one it did not

*What is waiting* is now a request on the person's door, which is what the item
said it owed. `person.rs` used to argue there was no `waiting` **yet**, on the
ground that what a shell draws is something the daemon answers rather than
something a person asks for. Half of that was right; the half that was wrong is
that a shell which never asked has nothing to draw if it started, restarted or
attached after the change was proposed. It carries no field at all, and what
comes back carries the sentence with every number.

The thing the item did not name is that `alo-protocol` now depends on
`alo-files` — the first time this crate reaches anything but `alo-capability`
and `alo-strings`. It deliberately does **not** reach `alo-asking`: a model's
answer arrives as text and a rendered sentence rather than as the type that
fetched it, because that crate carries an HTTP client and a TLS stack and item 4
refused exactly that dependency for `alo-record`.

`docs/quirks.md` gained nothing. Nothing in this iteration found reality
disagreeing with a specification; what it found was two of our own numbers
disagreeing, which is written into the contract and above.

### `ROADMAP.md` moved, and one code half was written into

**`alo-agentd`: grants, file verbs, application verbs, context on invocation** —
the code half now says what the daemon may say back and what divides it, and its
machine half no longer names 21b. No half was ticked that was not whole, and no
machine half was touched.

**What the next iteration must know:**

- **The queue's next ready item is 22**, *running out is not a fault* — a
  provider answering *payment required* has no variant in `alo-answering`, so it
  arrives as a key problem or as a number. **23** is ready after it. **21c**,
  **20** and **19b** are blocked on Linux and on a long-lived process, and
  **16b** is not ready by its own account.
- **`FromAPerson::number` answers `Option<u64>` now**, because the third request
  answers no change. A source-level change to an unreleased crate, and the wire
  format did not move: `waiting` is additive and does not raise `format`, which
  is what the contract already said about a new request.
- **Nothing has opened a socket.** Both halves of the protocol are a `&str` in
  and a value out, and no client that is not a test has ever read one of these
  answers.

## 2026-09-03 — iteration 41: running out is not a fault

**Item 22 — the failure `alo-answering` had no way to say, and the reason it was
not a rename.** `crates/alo-answering`: `WentWrong::RanOut` (the eighth),
`NotWhatFailed::NoAccountThere`, `needs_a_key_or_an_account` in `wrong.rs`, and
`RAN_OUT` in `words.rs`. `crates/alo-asking`: `ran_out.rs`, a **new file**, and
the two statuses in `openai.rs` that now consult it; `locally.rs` and `served.rs`
gained the reasoning about which of them can produce it. 16 new tests — 7 in
`ran_out.rs`, 3 in `openai.rs` against a real socket, 6 across `alo-answering`.
**1374 tests and 43 doctests across the workspace** (was 1358 and 43), `cargo
fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean with
zero warnings.

Items 16b, 19b, 20 and 21c were read first and skipped for the reasons the queue
gives — 16b is *blocked on nothing here, and not ready either*, 19b waits on
Wayland, 20 and 21c on a long-lived process and a Unix socket. 22 was the first
item that is neither done nor blocked, which is the loop's rule.

### The decision the item did not contain: there is no status that means this

The item asked for a variant for *a provider answering payment required or quota
exceeded*, and most providers answer neither. `402 Payment Required` has been in
HTTP since 1997; the services that send it are gateways and resellers. What the
large publishers document instead is an ordinary status carrying a
machine-readable name — `429` with `insufficient_quota`, `403` with a billing
name for an account they have stopped serving. So the two statuses a person most
needs told apart mean two things each, and neither of them carries this one.

That makes `openai.rs` read the body of a refusal, which nothing in this
workspace had done before: every other decision here is made from a status, a
type or a value one of our own crates produced. Three rules keep it from being
the road somebody else's words arrive by. **The identifier is compared and
dropped** — what leaves `ran_out.rs` is a `bool`, so `WentWrong` still holds no
text anybody outside alo OS wrote, which is the rule that file was written under
in the first place. **Spelling is not tracked**: the letters are matched, so
`insufficient_quota`, `INSUFFICIENT_QUOTA` and `InsufficientQuota` are one entry
rather than three, and which one a provider chose is not a thing this repository
has an opinion about. **Only two statuses are read at all**, and a `401` — which
has no second meaning — is still answered on the status, with a test that says
so.

### And the direction the doubt falls in is the design

`RESOURCE_EXHAUSTED` is Google's, `rate_limit_exceeded` is everybody's, and both
sound exactly like running out and mean *asking too fast*. Neither is on the
list, and there is a test that says neither is. The argument is about which
mistake is worse: an unmatched reply keeps the sentence it already had — a
number, or *your key was refused* — and a wrong match sends somebody to a
billing page to pay for something that was never the problem. The second is a
new harm the first does not do, so the list holds only names that mean an
account has nothing left **and mean nothing else**.

`docs/quirks.md` gained an entry for the whole of it, and the existing entry on
what a status means when a question fails gained a *what this entry no longer
covers* note pointing at it — the shape the loopback entry already uses.

### Three decisions the next items inherit

- **The two reasons that are about an arrangement rather than about the answer
  are refused together.** A key somebody pasted and an account somebody pays for;
  `needs_a_key_or_an_account` walks the closed list rather than wildcarding, so a
  reason added later has to answer the question instead of inheriting an answer.
  A machine on this network has neither — nothing here reaches one, and one in
  the next room bills nobody — and a gateway on *this* machine can have both,
  which is item 18b's precedent followed rather than argued again.
- **Running out opens no door that being switched off would not.** The test
  builds one failure of each over the same places and asserts the two
  `Elsewhere` are equal. ADR 0008's *never a silent fallback* runs hardest in the
  direction where somebody's money is at the far end, and a variant that quietly
  widened the offers would be the fallback with a receipt on it.
- **A nag needs a sentence, and there is exactly one.** The test walks every
  other string in `alo-answering` and refuses any mention of paying, credit,
  billing or buying — so ADR 0009's greyed-out panel cannot return as a
  buy-credit reminder for want of anything to say it with. It is the first test
  in this repository that guarantees something by the absence of a *string*
  rather than of a type or a dependency.

### What was considered and deliberately not done

`alo_models::NotTried` — testing a provider before it is saved — did not gain a
variant. That call lists what a provider offers, and a provider with no credit
still answers it, so the sentence would be one nothing can produce. The one
place an exhausted account is actually met is a question, and that is where it
now is.

### `ROADMAP.md` moved, and one code half was written into

**★ Or use an API instead (ADR 0008)** — the code half now says that a provider
whose account has run out says that and not something else, names the closed
list, and says it opens no door another failure would not. Its test count moved
from 73 to 88. No half was ticked that was not whole, and no machine half was
touched. The line for **AI can be declined entirely** was read and left alone:
ADR 0009's *since it was accepted* section is what this implements, but that
roadmap line is about setup's fourth choice as a screen, and writing an
inference failure into it would make the line report something it is not about.

**What the next iteration must know:**

- **The queue's next ready item is 23**, *a catalogue entry says whether the
  model can drive the verbs*, and it is the last ready item in the file. **21c**,
  **20** and **19b** are blocked on Linux and on a long-lived process, and
  **16b** is not ready by its own account. When 23 is done, the loop has no
  ready item left and the next iteration writes `LOOP COMPLETE` with that list.
- **`alo-asking` now reads a refusal body**, which is new for this workspace.
  Anything that adds a status to `openai.rs`'s match should decide whether it is
  ambiguous before adding it to the `matches!` above the match — a status added
  there costs a body read, and one added only to the match costs nothing.
- **Nothing has been run against an account that has really run out.** Nobody
  can produce one on demand without letting a real balance empty, so this is
  owed with the rest of the hardware verification and `docs/quirks.md` says so
  in the entry itself rather than in a note beside it.

## 2026-09-03 — iteration 42: whether a model can drive the verbs, measured

**Item 23 — the catalogue's last property, and the one that decides whether a
machine hands somebody's files to a model.** `crates/alo-driving`, a **new
crate**: `exercise.rs` (one request, the verb a correct answer calls, and the
prompt built from the verbs' own declared words), `exercises.rs` (the fixed ten
and the registry they are bound to), `attempt.rs` (what a model produced, and
what became of it), `measured.rs` (a whole run, and the grade). `alo-models`:
`driving.rs` (`Driving`), `choosing.rs` (`Catalogue::agent_for_cpu` and
`NoAgentHere`), the required `drives_verbs` field, 4 new words, and
`data/catalogue.toml` stating the grade on all twelve entries.
`alo-capability`'s `Arg::purpose_as_written` became public, additively, so a
prompt can be built from the declarations rather than from a second description.
43 new tests — 25 unit in `alo-driving`, 4 integration through `alo-models`, 14
in `alo-models`. **1417 tests and 43 doctests across the workspace** (was 1374
and 43), `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D
warnings` clean with zero warnings.

Items 16b, 19b, 20 and 21c were read first and skipped for the reasons the queue
gives. 23 was the last ready item in the file.

### The hole in the item's own scoring rule, and closing it is the decision

The item says to score *whether the call names a real verb and whether every
argument survives `alo-capability`'s validation*. Both halves are right and
together they are not enough: a model that answers `list_folder` with a valid
path to all ten requests names a real verb every time and validates every time,
and would grade as driving the verbs perfectly while being incapable of anything
but listing a folder. So an exercise names **the** verb a correct answer calls,
and `always_answering_with_one_valid_call_scores_nothing` is the test.

Two more followed from reading it that way. **The door is part of the bar**: a
change offered through the read door is a change that would run with nobody
approving it, and `Authorised::read` refuses it, so a model that cannot tell
ADR 0001 §5's two sides apart has not driven the verbs however well-formed its
call is. And **the six outcomes are kept apart rather than summed**, because *it
wrote prose*, *it invented a verb*, *it chose the wrong one*, *it used the wrong
door* and *it sent an argument the machine will not take* are five problems with
five different answers, and a percentage would tell whoever ran the measurement
only that something was wrong.

What the structural gate is *not* is weak. `Takes::Path` wants a full path with
no `..` and no control characters, `Takes::Name` wants one name and not a
journey, `Takes::Count` wants a number inside the verb's own range and
`Takes::Choice` wants one of the options the verb wrote down — so a model
answering `folder: "the invoices folder"` fails without anybody having written
down a right answer. Argument *values* are deliberately not scored: that would
measure how closely a model copied a sentence.

### Scored through the daemon's own door, and what that costs

An answer goes through `alo_protocol::FromAnAgent::read` and
`alo_capability::Verbs::call`, which is exactly what happens to a real client's
bytes inside a real turn, and this crate parses nothing. The alternative is what
makes the decision: a lighter shape invented here — a verb name and a map — is a
**second parser for one syntax**, which is the failure item 9g found and removed
one level down, and the one nobody uses in production is the one the score would
be about.

The cost is written into `docs/quirks.md` rather than hidden. The envelope is
part of what is measured, so a model wrapped by an agent that composes the
envelope for it may drive the verbs better than its grade says. And the prompt
is English, so a grade says how a model does *when it is asked in English* and
nothing about Latvian. Both errors fall toward not giving a model the agent,
which is the direction every other decision about this property takes.

### The half the item did not contain: the catalogue we ship now offers no agent

`Driving` had to have a fourth value, because this loop has no model to measure
against and writing `reliably` beside twelve entries would have been the claim
the whole item exists to prevent. `NotMeasured` is `Region::Unknown` one file
over — *not a synonym for "probably fine"* — and it is a **required** field with
no serde default, so an entry that says nothing fails to load and an entry that
says *not measured* has stated something.

What follows is real and is not a bug: **every entry says `not-measured`, so
`Catalogue::agent_for_cpu` refuses on every machine.** The old `default_for_cpu`
answered `Some(phi-3-mini)` on a 16 GB laptop and that answer was never earned.
The refusal is `NoAgentHere::NoneMeasured`, which says nobody has measured rather
than that the models failed — three variants, because *there was nothing to
choose from*, *nobody measured what there was* and *what was measured is not good
enough* send a person to three different places, and one sentence for all three
would claim a measurement in the case where none was run. **23a** is in the queue
under *blocked — hardware*, and a grade is a data change rather than a release.

### Three decisions the next items inherit

- **`default_for_cpu` is gone.** ADR 0007's own correction is that *default* was
  the wrong word and that the wrong word did damage; the method that carried it
  is `agent_for_cpu` and answers a `Result`, and *what this machine can run* is
  `to_choose_from_on_cpu` beside it. Running a model and giving it the agent are
  two questions, and that file conflating them is what the ADR names as the
  error.
- **A refusal that names an alternative hands back both lines or neither.**
  `NoAgentHere::lines` answers `[Said; 2]` and there is no method that gives you
  only the reason, so a screen cannot show somebody that their machine has no
  agent without the two places that would still answer. It is `alo-egress`'
  *permitted but not shown is not a state that exists*, applied to a refusal:
  the guarantee is the shape of the return value rather than whoever draws the
  panel remembering.
- **A crate whose only reader is us declares no words**, and `alo-driving` is
  the first. Nothing in it has a `Display` except two `thiserror` refusals of
  our own fixed set and our own run, whose reader is whoever ran the
  measurement — `CatalogueError`'s reader, and that crate's rule. The sentence a
  person reads about any of this is `alo-models`', made where the refusal is
  made.

### What was considered and deliberately not done

**The grade did not get words.** `CommercialUse` and `OnCpu` have none either,
for the reason iteration 20 wrote down: they are enums a catalogue panel would
label, nothing in this repository draws that panel, and inventing the English
here would be inventing it in the wrong place. What *did* get words is the
refusal, because a refusal is worded by whoever makes it (item 9e).

**`alo-driving` asks nothing.** No client, no socket, not even behind a feature:
it hands out a prompt and scores what comes back, and whoever runs it puts the
prompt to a model through `alo-asking`. That is `alo-answering`'s argument one
crate on — a promise about the absence of code is worth what the code around it
is small enough to prove.

### `ROADMAP.md` moved, and a missing line was written

The v0.01 ★ promise *the catalogue says whether a model can drive the verbs* had
**no line in `ROADMAP.md` at all** — another promise found with nowhere to be,
and the rule at the top of that file says the finding goes here before the work
does. It has a line now, with both boxes: the code half ticked and naming
`alo-driving` and `alo-models`, the machine half saying outright that the
measurement has never been run against a real model. *It runs on the machine you
already own* had its code half written into as well, because `default_for_cpu`
was that line's answer to *the system picks one* and it moved. No half was ticked
that was not whole, and no machine half was touched.

**What the next iteration must know:**

- **There are no ready items left.** 23 was the last one. **16b** is not ready
  by its own account, **19b** waits on Wayland, **20** and **21c** on a
  long-lived process and a Unix socket, and **23a** is new and blocked on a
  machine with a model on it. The next iteration writes `LOOP COMPLETE` with
  that list unless somebody has added work.
- **The shipped catalogue offers no machine a local agent**, and that is the
  honest state rather than a regression to fix in code. Anything that reads
  `agent_for_cpu` and finds it refusing has found the truth; the fix is 23a and
  it is not this loop's.
- **A changed `THE_SET` makes every grade in the catalogue stale.** The ten
  exercises are what make two grades comparable, so raising the measurement's
  coverage means re-measuring everything that was measured. `docs/quirks.md`
  says so, and the set is a `&'static` array in the source so it cannot drift
  quietly.

## 2026-09-03 — iteration 43: the queue has nothing left this loop can build

**No item was built, because there is no ready item to build.** The queue was
read top to bottom. Every item is either done, or unfinished for a reason this
machine cannot remove.

The gate was run anyway, because *nothing left to do* and *something has quietly
broken* look identical in a file and not at all alike in a terminal, and a
completion claimed over a red suite would be the worst line this journal could
hold. `cargo fmt --all -- --check` clean, `cargo clippy --workspace
--all-targets -- -D warnings` clean with zero warnings, **1460 tests and
doctests passing, zero failing** — exactly what iteration 42 left behind
(1417 tests and 43 doctests), so nothing has rotted between the two.

### What is left, and what each one waits on

- **16b — finding machines on the local network, on the indicator or not.**
  Not blocked by anything outside; not ready by its own account. There is no
  discovery code in this repository and none of it is portable, so the decision
  it asks for — an `Errand` with a destination of its own, or a documented
  exception — would be made about a shape nobody has built. It is a real hole in
  law 1's coverage and it is written down as one.
- **19b — what a turn does with an application verb.** Waits on Wayland and
  D-Bus. `alo-applications` decides all four verbs and stops at `Reaching`;
  until something can move a window, a turn that carried the call that far would
  be a new way to be refused rather than a thing the machine can do.
- **20 — where the record is written, and what prunes it.** Waits on 21c. It is
  a path, a setting and a timer, and a timer needs a long-lived process.
- **21c — the daemon itself.** Waits on Linux. A Unix socket and its peer
  credentials have no portable spelling, and peer credentials are what make
  21a's two doors a division rather than a convention.
- **23a — the measurement, run against real models.** Waits on a machine with a
  model on it. Until somebody runs it, every entry in `data/catalogue.toml` says
  `not-measured` and no machine is offered a local agent. It produces a data
  change rather than a release.

The two *blocked — hardware* entries beneath it are the same shape: the model
stack against a real Ollama, the GPU on first boot, and the v0.01 exit gate,
which is one person in front of one machine and is not substitutable by any
suite.

### Nothing was ticked, and that is the point

No queue item moved, `ROADMAP.md` was not touched, and `CHANGELOG.md` gained no
line — there is no change for a person outside this repository to read, and a
changelog entry saying the loop looked and found nothing would be noise in the
file people check to see what shipped. Ticking something to make this iteration
look productive is the exact failure `LOOP.md` names, and the honest record of
an iteration that built nothing is an iteration that says so.

**What the next iteration must know:**

- **The loop is complete, not finished.** Roughly half of v0.01 is still owed
  and none of it is owed by this loop: it is a compositor, a daemon, a bootable
  image, and one person sitting in front of a certified machine. What this loop
  was for — the capability model as working code, tested including every refusal
  path — is done, and it is what everything above it now gets to assume.
- **Restarting the loop needs new work in the queue, not another read of it.**
  Anything added should name its ADR, its contract and its line in
  `docs/features.md`, the way every built item above does, or the iteration that
  picks it up will spend itself deciding what it was asked for.
- **The one item that could become ready without a Linux host is 16b**, and what
  would make it ready is a decision rather than a machine — but the decision
  wants the discovery code in front of it, and that code is Linux.

- `LOOP COMPLETE` — written by that iteration and **discharged**: item 21c's Linux blocker was measured and answered (WSL2, kernel 6.6, `SO_PEERCRED` confirmed working), so the queue has buildable work again. Kept behind a bullet as a record rather than a signal.

## 2026-09-03 — iteration 44: the socket exists, and the kernel says who is on it

**Item 21c, re-scoped and built.** `crates/alo-agentd` is a new crate and the
first thing in this repository that is Linux rather than portable: a Unix
socket, the permissions around it, and the one question that decides which of
`alo-protocol`'s two doors a connection gets. 36 unit tests and 4 integration
tests; **1464 tests and doctests on Linux, 1460 on Windows**, `cargo fmt` clean
and `cargo clippy --workspace --all-targets -- -D warnings` clean with zero
warnings on both.

The Linux half was built and gated through WSL2 exactly as `LOOP.md` describes —
same working tree, separate `CARGO_TARGET_DIR`, about twelve seconds for a
workspace clippy. The Windows gate was run as well and is unchanged at 1460,
which is what says the new crate compiling to nothing off Linux really is
nothing rather than something that broke quietly.

### The item was three things, and it is now two

21c asked for a long-lived process, a socket, and peer credentials. What was
built is the socket and the credentials; the process is a new item, **21d**.

That is a scope cut and it was made for a reason rather than for time. A turn
belongs to an agent's connection and an approval arrives on the person's, so a
daemon has to hold two connections at once against one `alo_turn::Turning` that
borrows the machine mutably — serving one connection to completion would
deadlock on the first proposal, because the approval that releases it can only
come in on a connection nobody is reading. That is a design decision with
`alo-protocol`'s two doors and `alo-turn`'s borrow both in front of it, and
taking it in the same iteration as the socket would have meant taking it in a
hurry. It is written into 21d in those words.

### The decision the item named and did not settle

**rustix, and it is named in one file.** `UnixStream::peer_cred` is unstable in
std (rust-lang #42839) and `CLAUDE.md` forbids `unsafe` workspace-wide, so
`SO_PEERCRED` on a stable compiler is a rented crate. `unix.rs` is the only file
that spells it, as `ollama.rs` is the only file that knows Ollama exists.

**The measurement found something the queue item did not know.** rustix holds
the peer's process id in a *non-zero* integer, and `SO_PEERCRED` asked of a
socket with no peer answers `0` — so asking the wrong socket is undefined
behaviour inside the one crate that holds this workspace's `unsafe`. The answer
is a type rather than a comment: `who` takes a `&UnixStream`, and there is no
door in the crate that could hand it a listener. `docs/quirks.md` records both
halves.

### Three decisions this iteration made that were not in the item

- **The two sides are two Unix users, and a machine that names one login twice
  gets no socket at all.** `Sides::of` refuses it. On such a machine every
  connection would satisfy both tests, whichever was asked first would win, and
  *the side that proposes cannot approve* would be a sentence in a contract with
  nothing under it. Refusing to start is the only honest answer.
- **The directory is the first lock and the socket's mode is the second.** The
  crate makes its own directory `0750`, with that mode from the call that
  creates it, and hands it to the agent's group — which is what makes the moment
  between binding a socket and setting its mode harmless, because reaching the
  socket means traversing a directory that was shut before the socket existed. A
  directory that is a symbolic link, is not a directory, or belongs to somebody
  else is refused, and nothing in the crate will chmod, chown or empty one that
  is not the person's.
- **A stranger is closed on without a word.** It is the one place the protocol
  contract's *refused in words and never dropped* does not reach, and the
  contract now says so: that rule is about messages from the two clients this
  machine has doors for, and answering a stranger would tell whoever is knocking
  that there is an alo OS daemon here and what version it is.

### Two things this crate does not claim

**A connection from a second user has never been made.** Telling the two doors
apart takes two logins and a test process has one, so the mapping is tested
exhaustively as a value — including that the group opens no door and the process
id decides nothing — and the real connection that was tested end to end is the
person's. A real agent connecting as its own user is owed with the rest of the
verification that needs a machine set up as a machine.

**Nothing has run as a service.** The socket is bound and answered by tests,
never by systemd, and there is still no process that stays running.

### `ROADMAP.md` moved, and the contract gained a section

The `alo-agentd` line's **code** half now names the crate and what it decides;
its **On the machine** half was rewritten to say what is actually left — the
long-lived service (21d) rather than "the daemon itself", which is no longer the
whole truth. *Every execution recorded* had its machine half corrected the same
way: queue 20 waits on a timer, and a timer waits on 21d rather than on a crate
that now exists. **No half was ticked that was not whole, and no machine half
was touched.**

`docs/contracts/daemon-protocol.md` said the transport was "not here". It is
here now, and it is a public surface: the socket's path, the two modes, and the
rule that which door a connection is on is answered by the kernel and by nothing
on the wire.

**What the next iteration must know:**

- **21d is ready and is a design decision first.** It is not blocked on
  anything — the crate exists and the Linux host is reachable — but the shape of
  *one machine, two connections, one turn* wants writing down before it is
  written. It also inherits the one refusal 21c could not word: what a person is
  told when somebody who is neither of them knocks.
- **This workspace now rents `rustix`, and item 6b was about to rent something
  for the same kind of call.** `openat` and `renameat_with` are in the crate that
  is already here, so 6b's first decision is narrower than it was: a second
  wrapper would be two rented spellings of one kernel.
- **The Linux half of the gate is not optional for this crate.** On Windows
  `alo-agentd` compiles to nothing and its 40 tests do not exist, so an
  iteration that touches it and runs only the Windows gate has tested nothing it
  changed.


---

## An iteration was cut off mid-item, and its work is still in the tree

**Not a halt, and nothing is wrong with the code.** Anthropic's API returned
`529 Overloaded` to six consecutive headless iterations over twenty-two
minutes. Each one died before doing anything — but one earlier iteration was
already building **item 21d** when it was cut off, and its work was never
committed because it never reached the gate.

**Read this before starting 21d.** Seven files are sitting untracked in
`crates/alo-agentd/src/`, along with changes to `Cargo.toml` and `Cargo.lock`:

| File | Lines |
|---|---|
| `serving.rs` | 875 |
| `answering.rs` | 319 |
| `lines.rs` | 318 |
| `doing.rs` | 311 |
| `words.rs` | 181 |
| `stopping.rs` | 163 |
| `knocking.rs` | 94 |

**Measured, not assumed:** it compiles, `cargo check -p alo-agentd` is clean —
and `cargo test -p alo-agentd` reports **zero tests**. So it is real work that
stopped before the half `LOOP.md` cares most about. It is not half-finished
code; it is finished-looking code with nothing yet proving it.

**What the next iteration on 21d should do:**

- **Read those files before writing any.** They are a design somebody already
  chose. Starting over would throw away two thousand lines that compile, and
  worse, would produce a second design for the same item.
- **Assume nothing about them is verified.** No test has run against any of it,
  no clippy pass, no refusal path exercised. Treat every claim in them as
  unproven until a test says otherwise.
- **The gate is unchanged.** Whatever is kept from this work goes through
  `cargo fmt`, clippy at zero, tests including the refusal paths, the roadmap
  line, and a changelog entry — exactly as if it had been written fresh.

**Nothing was committed on their behalf.** Committing code that compiles but
has never been tested would be the dishonest tick this file has refused all the
way through, and it would be worse for arriving during an outage nobody caused.

**The loop was stopped deliberately**, rather than left retrying into an
overloaded API every four minutes. Restart it when the API is answering.

---

## 2026-09-03 — iteration 45: there is a service behind the socket

**Item 21d, built.** `crates/alo-agentd` is a service now rather than a door:
seven new files, five new dependencies, and the decision item 21c cut out of
itself taken with `alo-protocol`'s two doors and `alo-turn`'s borrow both in
front of it. 87 unit tests (was 36) and 4 integration tests; **1555 tests and
doctests on Linux** (was 1464), **1460 on Windows** (unchanged), `cargo fmt`
clean and `cargo clippy --workspace --all-targets -- -D warnings` clean with
zero warnings on both.

The Linux half was built and gated through WSL2 as `LOOP.md` describes. Windows
is unchanged at 1460, which is what says the crate compiling to nothing off
Linux is still nothing rather than something that broke quietly.

### The work was half in the tree, and reading it first was right

The previous entry recorded seven untracked files left by an iteration an API
outage cut off: about two thousand lines that compiled, with **no test having
run against any of them** and no clippy pass. It said to read them before
writing any, to assume nothing about them was verified, and to hold whatever
was kept to the whole gate. All three were followed.

What it did **not** say, and what this iteration found in the first ten
minutes: they had never been declared in `lib.rs`, and the tracked files they
were written against had never been changed. `cargo check` was clean because
nothing compiled them. So five things they needed did not exist — `unix::ready`,
`refusing::NotHeard`, `refusing::NotServed`,
`NotACaller::is_only_this_connection` and `Listening::waiting_on` — along with
eight fixtures and every dependency in `Cargo.toml`. **"It compiles" was true
of a crate that did not contain it.**

That is worth writing down as a rule rather than as an anecdote: *an untracked
file that compiles has proved nothing until something declares it.* A future
iteration finding work in this state should check `lib.rs` before believing a
`cargo check`.

### Two things in that work were wrong, and the tests are what found them

**The smaller.** A test answered a proposal by the literal number `1`;
`Approvals` starts counting at `0`, so it failed on the first run. Replaced by
reading the number out of what the person was actually shown, through
`ToAPerson::read`, which is the property worth asserting anyway — *the number
you answer with is the number you were shown* — and cannot rot when the
capability model changes where it starts counting.

**The larger, and it is a real defect rather than a test artefact.** An agent
hanging up and the next one knocking can be noticed in the **same** `poll`
wake-up. The round emptied the agent's slot and then answered the door, so the
newcomer landed in a slot that had just been emptied — *inside the turn the
previous agent's invocation made*, holding a grant that was never for it, and
never counted as a turn of its own. The test
`the_turn_ends_with_the_connection_and_the_next_one_gets_its_own` failed on the
count; the grant is the part that matters.

Closed with two changes, because they answer two different questions:

- **A round that ends a turn returns before the door.** Nothing is let in until
  the turn really is over. Nothing is lost by it: `poll` reports what is
  *there* rather than what has changed, so whoever is knocking is still
  knocking on the next round and gets a turn of their own.
- **The agent's door refuses while a turn is under way at all**, rather than
  while its slot is full. *An agent never acts under a grant another
  invocation made* is now carried by the condition rather than by the ordering
  happening to be right.

### Three decisions this iteration made that were not in the item

- **One clock, read once a round.** Item 1 said nothing reads the clock and
  every question that depends on time takes `now`; a rule like that needs
  somewhere to end, and a service is the honest place because it is the thing
  really running while time passes. Once a round rather than once a message, so
  two messages answered together cannot disagree about whether a grant expired
  between them. `alo-agentd` is now the only crate in this workspace that reads
  a clock.
- **An interrupted wait is resumed rather than reported.** `poll` ends with
  `EINTR` when a signal arrives, and on this machine the signal *is* the
  ordinary way a stop arrives — so treating it as a failure would turn the
  intended shutdown into a service that says it broke. The byte the handler
  wrote is still on the socket and the resumed wait finds it.
- **A wait with nothing to wait on is refused, not slept in.** `poll` with no
  descriptors and no timeout sleeps until the machine is turned off. It cannot
  happen while the two loops are the only callers, and it is an error rather
  than a comment, with a test.

### One thing renamed against the gate rather than silenced

`Line::next` tripped `clippy::should_implement_trait`. It is now `Line::heard`,
and the reason is better than the lint: a connection is not an iterator and must
not be read as one — an iterator that answered `None` and then something again
would be broken, and that is exactly what a socket does between two messages.
No `#[allow]` was added.

### `ROADMAP.md` moved, and the contract gained three sections

The `alo-agentd` line's **code** half now says the crate holds the turn and how;
its **On the machine** half was rewritten from "the long-lived service (21d)" to
the `main` that says what the machine is (21e). *Every execution recorded* and
*or use an API instead* had their machine halves corrected the same way: both
were waiting on a process that now exists, and what they are really waiting on
is a machine describing itself. **No half was ticked that was not whole, and no
machine half was touched.**

`docs/contracts/daemon-protocol.md` said the process was "not here yet". It has
three sections now — *a turn is a connection*, *what stops the service*, and
*what is still owed* — because a client author needs to know that a turn ends
with its connection, that a second agent is refused, and that a question to a
model is answered with a sentence today.

### What this service does not claim

**Nothing has run as a service.** There is no `main`, no signal handler
installed, and nothing has been started by systemd. Every test drives the loop
from a thread and stops it with the byte `Stop::stop` writes.

**A connection from a second user has never been made**, which is 21c's limit
unchanged: telling the two doors apart takes two logins and a test process has
one. `Pretending` is what stands in for it — a real socket, real connections,
real reading and closing, *told* which login each connection would have come
from — and `crate::knocking` has the test that the real socket answers the same
shape. Nothing about the mapping is faked.

**What the next iteration must know:**

- **21e is ready and is a file format before it is code.** What a machine says
  about itself is a public surface the moment anything reads or writes it, so it
  belongs in `docs/contracts/` beside the protocol. It unblocks item 20 and the
  last of *agents point at the local model*.
- **Item 20 is no longer blocked on a process.** It is blocked on 21e: the loop
  a timer would fire in exists, and what is missing is *which* path and *how
  long*.
- **The Linux half of the gate is not optional for this crate**, and it now
  matters more than it did: 87 of its tests do not exist on Windows, so an
  iteration that touches `alo-agentd` and runs only the Windows gate has tested
  nothing it changed.
- **ADR 0015 landed while this iteration was building, and 21d is what it was
  waiting on.** Queue item 26 — one BPF LSM hook, one grant, and a turn that
  reaches for `~/.ssh` getting `EACCES` from the kernel rather than a
  talking-to from us — says *blocked on nothing but the turn existing (21d)*.
  It exists now. Its first step is checking whether the WSL2 kernel has
  `CONFIG_BPF_LSM=y` and `bpf` in `CONFIG_LSM` at all, and the ADR is explicit
  that a no is a finding rather than something to work around.
