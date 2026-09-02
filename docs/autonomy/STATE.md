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
