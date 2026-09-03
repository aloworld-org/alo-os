# Queue — v0.01

Worked in order by `LOOP.md`. Every item names what it implements, so an
iteration can read the reasoning rather than guess at it.

## The honest constraint

**This loop cannot finish v0.01, and no amount of iterations will change that.**
Roughly half of what v0.01 needs ends on hardware and an operating system this
loop does not have:

- **A Wayland compositor, the portals and the bootable image are Linux.** They
  will not compile, let alone run, on the machine the loop runs on today.
- **The exit gate is the certified machine**, verified by somebody sitting in
  front of it, once. No test suite substitutes for that.
- **"Agents point at the local model" belongs to `alo-workplace`.** Different
  repository, and this loop never touches another one.

What the loop *can* finish is the part everything else stands on: the capability
model as working code. Grants, verbs, approvals and the record are pure logic —
portable, exhaustively testable, and the thing that makes `alo-agentd` a service
worth writing rather than a daemon with a permission dialogue. Doing that first
is also the right order: the Linux work becomes an implementation of a settled
model instead of a place where the model gets decided by accident.

Items are **ready** (buildable and testable here), **linux** (needs a Linux
host), or **hardware** (needs the certified machine). The loop takes ready items
and stops when there are none left.

---

## Already built, outside the loop

`crates/alo-models` — read it before starting item 1, because it sets the house
style the rest should match, and two of its decisions constrain later items.
`crates/alo-capability`, `crates/alo-record`, `crates/alo-egress`,
`crates/alo-files`, `crates/alo-applications`, `crates/alo-context`,
`crates/alo-keeping`, `crates/alo-shortcuts`, `crates/alo-appearance`,
`crates/alo-dock`, `crates/alo-answering`, `crates/alo-asking`,
`crates/alo-turn` and `crates/alo-strings` were built by the loop and are
described in the items below. **`alo-turn` is the only one that reaches
another crate in order to hold it to an order** rather than to ask it
something: since item 19 it is where an invocation, a call, an approval, an
execution and the record are joined, and it is the first crate whose value is
that none of five steps can be skipped rather than that one of them is decided
correctly. Nothing reaches it, and it says one sentence. **`alo-asking` is the only one of them that does anything to the world
outside this machine**, and it is the only one that reaches five others: it
holds no decision of its own, and every step it takes is one of theirs, in the
order they have to happen in. Since item 18b it has **three doors that divide on
law 1** — one that leaves and two that do not — so the sentence above is
narrower than it reads: it is the only crate that does anything outside this
machine, and two of its three doors deliberately do nothing outside it at all.
The address that decides which side a door is on is `alo-models`', asked rather
than repeated, which is what stops the crate that opens the socket from holding
an opinion about whether the socket leaves.
Nothing reaches it, and it deliberately does not
reach `alo-record` — it hands back the departure instead, which is what keeps
*the record observes, and is reachable from none of the crates it observes* true
of the crate that causes the largest egress this product has. **`alo-answering` is
the second crate to reach `alo-models`**, after `alo-egress`, and the two reach
it for halves of one thing: `alo-egress` decides about a question that is
already leaving, and `alo-answering` about one that was answered nowhere. It is
also the only crate in the workspace whose whole value is a thing it *cannot*
do — it decides where a failed question may go next, and has no client, no
socket and no serde with which to go there. `alo-context` sits where
`alo-files` and `alo-applications` do — it reaches `alo-capability` and
`alo-strings`, nothing reaches it — and it is the only one of the three that
**makes** a grant rather than only being checked against them. `alo-applications` sits
exactly where `alo-files` does — it reaches `alo-capability` and `alo-strings`,
nothing reaches it, and it is the other half of what an agent may do to this
machine. The first four depend on each other in one
direction only: `alo-capability` decides and reaches nothing, `alo-egress`
decides about what leaves, `alo-files` is the only one an agent can reach that
touches a disk, and `alo-record` observes them and is reachable from none of
them. `alo-keeping` reaches `alo-record` and puts it on a disk; nothing reaches
back, which is what keeps *nothing takes an entry out* true of `alo-record`
while something, somewhere, can shorten a record. `alo-shortcuts` and
`alo-strings` depend on nothing in this workspace at all, and `alo-appearance`
on nothing but `alo-strings`, because a person pressing a key on their own
machine, choosing their own wallpaper or reading their own machine in their own
language is not an agent doing something and needs no grant. **`alo-dock` is the
first crate in that group to reach another one of it** — it asks
`alo-appearance` how big the person has made their text, because how much room a
name needs is that answer and a second `TextScale` would be a second answer.
Nothing reaches it, and it is as far from `alo-capability` as the other three:
somebody moving their own dock is not an agent doing anything either.

Seven edges cross that: **`alo-files` depends on `alo-strings`** since item 9b,
**`alo-shortcuts` does** since 9c, **`alo-appearance` does** since 9d,
**`alo-capability` and `alo-record` do** since 9e — the deciding crate because
every refusal it makes is read by somebody, and the record because it writes
down the words that person was shown rather than a second rendering of its own —
**`alo-models` does** since 9f, because where an answer came from is a
sentence somebody reads before they decide what to send, and **`alo-egress`
does** since 9h — the last to cross, and the one whose sentence a person reads
while it is happening. `alo-strings` still depends on nothing, and the direction
is the one every other edge here takes — a crate that says something reaches the
crate that knows how things are said, never the reverse. Every crate in this
workspace has now crossed it, so *hardcoded English is a bug* is a rule with no
exceptions left in it rather than a rule with a list. **`alo-keeping` is the
first crate that never had to cross**: it was written after the 9-series, so it
has never held an English sentence and no type in it has ever had a `Display`.
That is what the rule looks like once it is finished being applied, and
`alo-applications` and `alo-context` are the second and third crates it is true
of.

Since **item 17** `alo-capability` holds one thing that is not about what an
agent may do: `Agent` is whether there is an agent on this machine at all
(ADR 0009), and `Grants` lives **inside** it. No crate and no edge moved — the
grants are where they always were, one type further in — and the reason it is
here rather than in a crate of its own is that a machine with no agent is the
limiting case of what an agent may reach rather than a subject beside it.

Since **item 9g** the edge is load-bearing rather than incidental: a verb is
*declared* from `alo_strings::Word`s, so `alo-capability` cannot express a
capability whose words are not translatable, and `alo-record` renders the
sentence a person approved rather than copying one. `alo-files` lost a file to
it — `saying.rs` — which is what it looks like when two things that said the
same thing become one.

| | |
|---|---|
| `catalogue.rs` | What alo OS offers, every licence stated, commercial use answered outright; the CPU costs and defaults from ADR 0007 |
| `runtime.rs` | `ModelRuntime` — what alo OS asks of a runtime, in our words |
| `ollama.rs` | The adapter, and the only file that knows Ollama exists (ADR 0006) |
| `source.rs` | Where a question is answered and what that costs in egress (ADR 0008); the region policy an organisation names |
| `provider.rs` | Providers somebody adds themselves; the key lives in the keyring, never in the settings |
| `secret.rs`, `tried.rs`, `trying.rs` | Added later by the loop — testing a provider before it is saved (item 10) |
| `words.rs`, `refusing.rs` | Added later by the loop — everything this crate says, and which rule refused a question (item 9f) |

**44 tests when the loop started, 90 now, clippy clean against the workspace
deny list.** Two patterns later items must follow:

- **A promise in `docs/` is a test, not a sentence.** "Every model states its
  licence" and "a paired machine is egress too" are tests. Anything an item
  claims should be one.
- **Errors say what to do, not what went wrong.** `provider.rs` is the reference:
  *"give the provider a name — it is what you will see when it answers"*.

---

## Ready

- [x] **1. Grants** — implements ADR 0001 §3 and `docs/contracts/agent-verbs.md`.
  `crates/alo-capability`: `reach.rs` (what a grant covers, what a verb asks
  for), `grant.rs` (one grant and what has to be true of it), `grants.rs` (the
  list, its queries and its refusals), `path.rs` (containment, decided
  lexically). 28 tests, clippy clean. **This is the vocabulary items 2–5 speak**
  — `Grantee`, `Reach`, `Ask`, `Grant`, `Grants`, `GrantId` — and they belong in
  this crate beside it rather than in one of their own.

  Three decisions the next items inherit. **Nothing reads the clock:** every
  question that depends on time takes `now`, so expiry is testable without
  sleeping and the daemon and the settings panel cannot disagree about the
  moment. **Identities are matched exactly** — agent names, application ids and
  paths — because matching loosely matches more than the person picked.
  **Containment is lexical and touches no disk**, so whatever executes a verb
  must resolve symbolic links *before* asking whether a path is granted; the
  reasoning is in `path.rs` and item 6 is where it gets honoured.

  Storage is serde, as with `Providers` — where the list is written and when is
  the daemon's, and it does not exist yet.

- [x] **2. The verb registry** — implements `docs/contracts/agent-verbs.md` and
  law 2. Five files in `crates/alo-capability`: `arg.rs` (what an argument can
  be, and what survives arriving), `sentence.rs` (the approval sentence and
  where its words come from), `verb.rs` (everything that has to be true of a
  verb), `verbs.rs` (the closed list), `call.rs` (a validated call, what it
  would touch, and whether the grants permit it). 37 tests, clippy clean.

  **The registry cannot hold a verb that breaks the contract, and it manages
  that by not being able to receive one:** `Verb::checked` is the only
  constructor and there is no `Deserialize` around it. Law 2 is carried by
  `Takes` being a closed list with no free-text kind in it — a model cannot
  compose something to run because there is no shape for it to arrive in — plus
  a declaration-time refusal of names that announce an interpreter, which is a
  tripwire and is documented as one. A verb's implementation passing an argument
  to a shell is still only stopped by review, and `verb.rs` says so.

  Three items inherit decisions from this. **Every argument is required** — an
  optional one makes the sentence conditional. **The sentence names every
  argument**, or the verb is refused. **Being permitted and being approved are
  separate**: `call.rs` answers the first and deliberately does not answer the
  second, which is item 3's.

- [x] **3. Approvals** — implements ADR 0001 §5. Four files in
  `crates/alo-capability`: `proposal.rs` (a change put to a person, and what is
  never put to one), `approvals.rs` (the list waiting to be answered, and the
  answering), `approval.rs` (one answer, worth one execution), `authorised.rs`
  (the only type meaning may-run, and both doors into it). 24 tests and two
  doctests, clippy clean.

  A change now travels `Call` → `Proposal` → `Approved` → `Authorised`, and
  each step can only be reached from the one before. **One approval, one
  execution** is carried twice over: the list refuses a proposal answered twice,
  and `Approved::redeem` takes `self`, so a second execution is not a program
  that compiles — which is what the `compile_fail` doctest on `redeem` asserts,
  with a twin that passes so the pair cannot rot into a test of a typo. The
  arguments travel inside the approval and there is no accessor lending them
  out, so an approval for one argument set cannot authorise another.

  Three decisions the next items inherit. **The grants are asked last at the
  moment of execution**, which is where a revoked grant becomes immediate; being
  permitted is checked when the call is made, again when it is proposed, and
  again there. **A read is never proposed and a change never takes the read
  door**, so ADR 0001 §5 is two refusals rather than a convention. **A refusal
  carries what it refused** (`Refused::call`), because item 4 records refusals
  and one that threw away the call could only say that something was stopped.

- [x] **4. The record** — implements ADR 0001 §7. Seven files in
  `crates/alo-record`, a **new crate** rather than more of `alo-capability`:
  `line.rs` (text as the record keeps it), `written.rs` (one argument),
  `what.rs` (what a call was), `happened.rs` (the four things that can happen),
  `entry.rs` (one moment and what happened at it), `record.rs` (the list),
  `explain.rs` (what can be asked). 42 tests, clippy clean.

  **Why a crate of its own**, against what `alo-capability`'s docs previously
  said. The deciding crate must not deserialise — `Call`, `Value` and `Proposal`
  serialise and deliberately do not read back — while a record exists to be read
  back, so it needs its own types either way. And the record needs
  `InferenceSource` from `alo-models`, which would have put an HTTP client and a
  TLS stack behind the crate whose whole value is being small enough to audit.
  `alo-capability` still depends on nothing but serde and thiserror.

  Three decisions the next items inherit. **`Grants::permitting` answers with
  the grant rather than with a yes**, and `permits`/`refusal` are now two halves
  of that one search — so nothing can be permitted by a grant the record cannot
  name, and `Authorised::against` carries it. **A record is evidence, not an
  instruction**: nothing in `alo-record` turns back into a call, and nothing
  takes an entry out. **Two things are never kept** — the question a person
  asked, and the arguments of a call that never validated.

- [x] **4a. Where the record is written, and what prunes it** — cut from item 4,
  and listed here as the daemon's until this iteration read it again. Two of the
  three things it names are portable and testable on any machine: **the file it
  is written to** and **the decision about how long evidence is kept**. Only
  *which* path and *when* a shortening runs are `alo-agentd`'s, and those are
  item 4b under *blocked — linux*. `crates/alo-keeping`, a **new crate**:
  `keeping.rs` (the rule), `head.rs` (the first line, and where the record
  starts), `writing.rs` (appending, one entry at a time), `reading.rs` (reading
  it back), `pruning.rs` (the only thing in alo OS that removes evidence),
  `damage.rs` (what could not be read), `failing.rs` (why there is no record to
  write to), `words.rs` (14 phrases and one countable string). 59 unit tests, 15
  integration tests against a real filesystem, 781 tests and 20 doctests across
  the workspace, clippy clean.

  **A new crate rather than more of `alo-record`, and the reason is the
  promise.** `alo-record` says nothing takes an entry out — no `remove`, no
  `edit`, no `forget` — and something has to be able to. In one crate that
  promise would be true of a type and false of the file list around it, which is
  how a security reviewer checks it. So the crate that *can* shorten a record is
  separate and small, and everything in it is about making that hard to do
  quietly: what goes is decided by a rule and a moment with no way to name an
  entry, shortening is a method on the writer so nothing that is not holding the
  record open can do it, and it refuses a record it cannot read all of rather
  than rewriting the evidence that something was wrong.

  **The decision the item did not contain: where the mark goes.** A record that
  has aged out its first six months and a machine that did nothing are the same
  short file, so the shortening has to leave one. An *entry* saying so was the
  obvious answer and is wrong — an entry has a moment, so the next shortening
  ages it out, and after two rounds the record looks untouched again. The mark
  is therefore the **first line**, which pruning never walks: a record says
  where it now starts, and no later shortening can take that back.

  Three decisions the next items inherit. **The record file is a public
  surface**, written down in `docs/contracts/record-file.md`: one line of JSON
  per entry, a format number in the first line, additive change only, and a
  record from a newer alo OS refused rather than appended to. **A missing record
  is not an empty one** — reading refuses it, because a deleted record answered
  as *nothing happened* is the failure this crate exists to prevent, and making
  one is a deliberate act by the daemon. **A window is measured from the epoch
  forwards, not from `now` backwards**: `SystemTime::checked_sub` walks into
  1969 on Windows and answers `None` elsewhere, so a wrong clock would remove
  different things on different machines. `docs/quirks.md` records that and the
  rename-over-an-open-handle it also found.

- [x] **5. Egress policy** — implements law 1 §8. Five files in
  `crates/alo-egress`, a **new crate**: `destination.rs` (where something is
  going, and what may be shown as an address), `leaving.rs` (one egress about to
  happen — who, where to, why), `policy.rs` (what an organisation permits, and
  the refusal in words), `departing.rs` (the only type meaning may-leave),
  `indicator.rs` (what is leaving right now). 27 tests and 3 doctests, clippy
  clean.

  **The decision and the indicator are one call**, and that is the whole design:
  `Indicator::beginning` asks the policy and shows the result, and it is the only
  constructor of `Departing` — the token a caller must hold before it opens a
  connection. *Permitted but not shown* is therefore not a state that exists,
  which is the guarantee `CLAUDE.md` names, carried by a type rather than by
  whoever writes the next verb remembering. Ending a departure takes `self`, so
  one connection can never take two lines off the indicator; the `compile_fail`
  doctest asserts it, with a passing twin.

  Three decisions the next items inherit. **The rule is stated once**:
  `EgressPolicy` is made `From<&SourcePolicy>` rather than written down again,
  and a test walks every policy against every source to prove the wider boundary
  and the inference one cannot disagree. **A question answered on this machine is
  not a departure** — `Leaving::asking` refuses it — so law 1's zero-egress claim
  is the absence of a type rather than a counter that happens to read zero.
  **Only a paired machine is in the building**: a host that answers on the same
  wire is outside it, because ADR 0003 refuses "it is only our internal network".

  Enforcement at the network boundary is Linux and is a later item; this crate
  guarantees the ordinary path, not code that never asked.

- [x] **5a. The record of what left** — implements law 1's second half in
  `alo-record`. One new file, `departed.rs` (the only door from an egress into
  the record), plus three new `Happened` variants and the query behind them. 12
  new tests and 3 new doctests, two of them `compile_fail`; 218 tests and 8
  doctests across the workspace, clippy clean.

  **The decision, which the item asked for before it asked for code.** Neither
  of the two options as written: `Answered` did not become provenance beside a
  departure, it became `AnsweredHere` and stopped having a source at all. A
  question answered somewhere else *is* the departure it caused, so it is
  `Happened::Left` and nothing beside it; a question answered here is
  `Happened::AnsweredHere`, which has nowhere to name and no field to name it
  in. Nothing is lost — `Destination` says everything `InferenceSource` said,
  and `alo-egress` already maps one to the other in one place — and two things
  are gained. `caused_egress` became a variant rather than a calculation, so no
  two readers can work it out differently. And the record can no longer
  contradict itself: saying *the answer came from a provider* now requires a
  `Departing`, which only the indicator makes.

  Three decisions the next items inherit. **An egress entry is unreachable
  except from a `Departing`, and a held-back one except from a `NotPermitted`**
  — both `pub(crate)`-constructed by the indicator, asserted by `compile_fail`
  doctests that were checked to fail on the privacy and not on a typo.
  **`Why` now deserialises**, additively, like `InferenceSource` gained
  `Serialize` in item 4; `Leaving` still does not. **`alo-record` depends on
  `alo-egress` and no longer on `alo-models`** except in tests — the observer
  reaches the decider, never the reverse.

- [x] **6. File verbs, the portable half** — implements `docs/features.md`'s
  v0.01 file verbs and the rule item 1 left for it. `crates/alo-files`, a **new
  crate**: `verbs.rs` (the six, declared), `real.rs` (the path this machine
  would really open), `resolving.rs` (the one thing here that touches a disk),
  `touching.rs` (the only type meaning *this may touch the disk*). 24 tests, 3
  integration tests against a real filesystem, 3 doctests, clippy clean.

  **The item exists for one sentence in item 1**: containment is lexical, so
  whatever executes a verb resolves the real path and asks about *that*.
  `Touching::of` takes an `Authorised` — the end of ADR 0001 §5's journey — and
  asks three questions of every path the call names, in this order: do the
  grants permit it as written; where does it really lead; do they permit that.
  A link out of a granted folder dies at the third, as a refusal by the grants,
  in their own words. The order is the security property: **a path nobody
  granted is refused before the disk is touched**, so a refusal cannot tell an
  agent whether a file it may not reach exists.

  Three decisions the next items inherit. **Every path a call names is asked
  about**, not only the ones the verb declared its grant is over — the test that
  proves it uses a verb whose author forgot one. **`Real` has no public
  constructor**, which seals `Resolving`: one implementation ships, so nothing
  can hand the grant check a real path it made up, and the crate's decision is
  still testable on a platform where making a symbolic link needs a privilege.
  **`Refused::not_granted` is new in `alo-capability`** — the one question that
  crate cannot ask itself comes back as the same type, so this refusal reaches
  the record by the road every other refusal takes.

- [x] **6a. File verbs, the acting half** — the `std::fs` calls behind the six,
  taking a `Touching` rather than a path. Nine files in `crates/alo-files`:
  `doing.rs` (the one door, and the last grant question), `answer.rs` (what each
  of the six answers with), `failed.rs` (why the machine could not), `named.rs`
  (one thing in a folder), `looking.rs` (the three reads), `changing.rs` (rename
  and move), `archiving.rs` (what goes into an archive), `walking.rs` (a folder,
  without walking out of it), `zip.rs` and `crc.rs` (the format). 64 unit tests,
  13 integration tests against a real filesystem, clippy clean.

  **The item asked for two things and found a third.** It opens what `Touching`
  resolved and resolves nothing twice, and a read asks the *open handle* how big
  a file is rather than asking the name again. The third is `Did::of`'s: a
  change **creates** a path that nothing had asked the grants about — a rename
  invents a name, a move and an archive invent one inside a folder — and under a
  grant over a single file (ADR 0001 §4) that name is one nobody granted. So the
  grants are asked once more, at the authorisation's own moment, and a no is a
  `Refused` like every other. *A grant covers where a file goes, not only where
  it comes from.*

  Three decisions the next items inherit. **A refusal by the grants and a
  refusal by the machine are different types** — `Refused` leaves by `Err`, and
  `Failed` travels inside a `Did` — because a record that called a full disk a
  refusal would tell a security review the grants stopped something they did
  not. **The authorisation comes back either way**, so `Entry::ran` is written
  from a call that was attempted, whatever the disk made of it. **Every bound
  says it was reached**: a listing, a read and a search answer with a flag, an
  archive refuses, and a bounded answer that did not say so would read exactly
  like a complete one.

  What it could not close is **item 6b, below under *blocked — linux***: the two
  gaps between checking a path and acting on it that only Linux calls close.

- [x] **7. Keyboard shortcuts** — implements `docs/features.md`'s v0.01 "keyboard
  shortcuts, and a person can change them". `crates/alo-shortcuts`, a **new
  crate** that depends on nothing in this workspace: `modifier.rs` (what is held
  down), `key.rs` (the closed list of keys), `chord.rs` (one combination, and the
  three it refuses to be), `action.rs` (what a shortcut does), `defaults.rs`
  (what alo OS ships with), `changes.rs` (what a person changed, which is all
  that is written down), `shortcuts.rs` (the two resolved, and every question
  asked of them), `clash.rs` (two actions wanting the same keys). 41 tests and a
  doctest, clippy clean.

  **The item's own sentence — express a conflict rather than letting the last
  binding win — turned out to be two problems, and only one of them is a
  refusal.** `bind` refusing a chord something else holds is the easy half. The
  half that shapes the crate is that **a release can add a default onto a chord a
  person already moved something onto**, which no refusal at bind time could have
  seen coming, because the binding was made before the default existed. So a
  clash is a thing the model *holds and reports* (`Clash`) as well as a thing it
  refuses (`Taken`), and the resolution is stated rather than emergent: a
  person's binding beats one we shipped, and two of their own on one chord fire
  nothing, because choosing between them would close a window somebody meant to
  maximise.

  Three decisions the next items inherit. **Only the difference is stored** —
  `Changes` is the settings file and the defaults live in the code, which is what
  lets a default be improved for everybody who never touched it; item 8's
  appearance model has the same shape and should copy it. **A promise elsewhere
  is a refusal here**: `Ctrl+C`, `Ctrl+X` and `Ctrl+V` cannot be taken by a system
  shortcut, because `docs/features.md` promises copy and paste across
  applications at v0.01 and a system shortcut is a key taken away from every
  application at once. **A key is the one printed on the person's own keyboard**,
  not a position on an American one — which leaves the compositor one job written
  down in `key.rs`: a layout with no Latin letters at all needs the shortcut
  matched against the person's Latin layout.

- [x] **8. Appearance** — implements `docs/features.md`'s "Making it yours",
  minus the accent, which is item 8a below. `crates/alo-appearance`, a **new
  crate** that depends on nothing in this workspace: `colour.rs` (one colour and
  how it is written), `token.rs` (the six in `docs/design/figma-brief.md`),
  `picture.rs` (one picture and how it meets the edges), `rotating.rs` (a folder
  of them, one at a time), `background.rs` (the three kinds), `display.rs`
  (which screen), `time.rs` (a time of day), `scheme.rs` (light, dark and the
  schedule), `text.rs` (how big the text is), `lock.rs` (what is on the screen
  when nobody is signed in), `shipped.rs` (what a machine looks like before
  anybody changes anything), `changes.rs` (what a person changed), and
  `appearance.rs` (the two resolved). 58 tests and a doctest, clippy clean.

  **The item said "background per display" and that turned out to be the wrong
  shape.** A background *per display and nothing else* reads the same until a
  projector is plugged in, at which point a machine whose owner chose a
  photograph shows a room full of strangers the wallpaper we chose, because the
  projector is a display nobody has set anything on. So there is one background
  — the person's — and a display they singled out is an exception they made on
  purpose. A display renamed by a driver update loses its exception and falls
  back to their choice, which is the right way round to fail.

  **The second decision was not in the item at all.** The desktop is seen by
  whoever is signed in; the lock screen is seen by whoever walks past. A person
  who pointed their background at a folder of their own photographs picked the
  *folder* — they did not pick, one by one, the pictures a machine left alone in
  a room shows to a corridor. So a lock screen that *follows* the desktop does
  not follow a rotating one: it shows the shipped wallpaper while the folder
  rotates, and says so (`lock_is_holding_back`) rather than leaving somebody to
  notice. Nothing is taken away — a person who wants their photographs on the
  lock screen sets them there and gets them, because saying so is a decision and
  following is not.

  Three decisions the next items inherit. **Only the difference is stored**, as
  in item 7: an untouched machine writes `{}`, and there is a test that says so,
  which is what lets a release ship a better wallpaper to every machine that
  never touched it. **Nothing reads the clock or the disk** — a schedule is
  answered at a time of day that is passed in, and a rotating folder is asked
  *how many pictures it holds* and *how long it has been running* rather than
  going to look — so item 1's rule now covers appearance too. **A promise in a
  standard is a test**: EN 301 549 requires text to reach 200%, so `text.rs`
  asserts the ceiling is at or above it rather than commenting that it should be.

  What this owes elsewhere: **the image must ship a wallpaper named `alo`**
  (`shipped::THE_WALLPAPER`), or a fresh machine has nothing behind its windows.
  That is written into the image item under *blocked — linux*.

- [x] **8a. The accent set** — implements **ADR 0010**. Two new files in
  `crates/alo-appearance`: `accent.rs` (the five, both values each, and the
  three refusals) and `contrast.rs` (how far apart two colours are to look at,
  to the standard). `Changes`, `Shipped` and `Appearance` gain the setting, and
  `Setting::Accent` puts it back. 17 new tests, 536 across the workspace,
  clippy clean.

  **Terracotta is unreachable rather than refused-at-the-door**, which is the
  shape the item asked for made stronger: the accent is a closed set of five, so
  a settings file naming terracotta fails to read at all, and the refusal in
  words is for the one road a colour can still arrive by —
  `Accent::of_colour`, which is where a panel that let somebody type a hex, and
  at v1 an agent asked to *make the accent this colour*, both land. Three
  refusals rather than one, because asking for the agent's colour, asking for a
  ground, and asking for a colour nobody designed are three different mistakes
  and send a person to three different places.

  Three decisions the next items inherit. **A colour set is a claim about
  legibility, so it is measured** — `contrast.rs` is EN 301 549 by way of WCAG
  2.1, and every accent is held to the text threshold against the grounds the
  design brief names, which is ADR 0010's "wants contrast verified" turned into
  a test. **The accent is stored by name and resolved against the scheme at the
  moment of asking**, so one choice covers both grounds and a release can
  correct a value for everybody who chose that colour. **The measurement found
  something the decision did not claim**: terracotta on cream is 2.87:1, under
  the threshold for a word *and* for a shape, so ADR 0010's mark and word are a
  measured requirement rather than a courtesy — noted in the ADR, and the
  shell's to honour.

- [x] **9. Strings** — implements `CLAUDE.md`'s *user-facing strings are
  externalized from day one* and `docs/features.md`'s "Language and access"
  (ADR-free). `crates/alo-strings`, a **new crate** that depends on nothing in
  this workspace: `key.rs` (what names one string), `template.rs` (a sentence
  with named gaps), `filling.rs` (what goes into them), `phrase.rs` (one string
  the code can say, and its note to a translator), `vocabulary.rs` (everything
  the code can say, and where a translation is checked against it),
  `translation.rs` (one language's strings as they arrive, and what can be wrong
  with them), `speaking.rs` (a checked translation — the only thing the lookup
  accepts), `language.rs` (which language, and which way it is read), `union.rs`
  (the 24, each named in itself), `said.rs` (one answer and where it came from),
  `strings.rs` (the lookup and the chain). 75 tests, 5 integration tests over
  the strings this repository already has, 3 doctests and a `compile_fail`,
  clippy clean.

  **The item said "so a missing translation is visible in development", and the
  design decision was refusing to make that a development-only fact.** A build
  flag that marked untranslated strings would answer with a `String` the rest of
  the time, and *shown English because nobody translated it* would be invisible
  in exactly the build a person in Latvia is running. So the answer is a `Said`
  that always carries where it came from — a translation, the source, or a key
  nothing declares — and marking in development is one of three ways of noticing
  rather than the only one; `Strings::unanswered` is the second, and
  `Said::is_translated` on every answer is the third.

  **The second decision was not in the item.** A translation is written by
  somebody whose language nobody here reads, so a sentence that dropped `{bytes}`
  would reach a person as *your file is too big* with no size in it, in their own
  language, with nothing anywhere saying so. `Vocabulary::check` is therefore the
  only door to a showable translation: it matches every gap against the source's,
  refuses a dropped or invented one in words addressed to a translator, and
  returns **everything** wrong at once, because being told about the next mistake
  each time you try again is how a translator gives up. A *partial* translation
  is deliberately not an error.

  Three decisions the next items inherit. **English is a source, not a default**
  — the sentence lives beside the key in the code, which is item 7's *only the
  difference is stored* reaching a fifth crate, and is what lets a release
  improve an English sentence for every machine that has no translation of it.
  **A person names their own second language**: the chain is what they said plus
  the broader form of each (`pt-BR` brings `pt`), and nothing infers a second
  language from a first. **A language is named in its own language** —
  `union.rs` holds the 24 endonyms, and a picker that said *Greek* would be one
  the people it exists for cannot read.

  What it could not close is items 9a–9e below; 9a is now built.

- [x] **9a. Plural forms** — cut from item 9, and built once the rules could be
  read rather than recalled: the loop fetched CLDR's
  `common/supplemental/plurals.xml` from `unicode-org/cldr` and worked from it.
  Four new files in `crates/alo-strings`: `form.rs` (the six shapes),
  `cldr.rs` (the table, each arm quoting the condition it came from, plus
  `Counting`), `plural.rs` (one countable string), and the plural half of
  `key.rs`, `vocabulary.rs`, `translation.rs` and `strings.rs`. 41 new tests and
  1 new doctest; 559 tests and 20 doctests across the workspace, clippy clean.

  **Whole numbers are the scope, and that is what made the table tractable.**
  alo OS counts things and a thing is a whole number, so `Counting` holds a
  `u64` and CLDR's `v`, `w`, `f`, `t` and `e` operands are all zero — which is
  why Czech's `many` and Lithuanian's `many` cannot happen here and French's
  `many` keeps only the half about whole millions. Each condition is quoted in
  full anyway, so what was dropped is visible. Counting something that is not
  whole is a decision to reopen, stated in `lib.rs`, not a form quietly picked.

  Three decisions the next items inherit. **A form's name says nothing about
  which numbers it covers** — Polish has no `other` for a whole number,
  Croatian's `one` covers 21, French's covers 0 — so three things are refusals:
  a form a language never uses, a number spelled out in a form that is not one
  number, and a countable string in a language whose rules are not in the table.
  `docs/quirks.md` records all three. **A countable string owns every form
  beneath its key**, in both directions, so `files.too-big` and
  `files.too-big.one` cannot both exist. **`unanswered` and `missing_from` now
  answer `Vec<Key>`** and expand a countable string into the forms *that*
  language needs — a Polish file with `one` and `other` is not two thirds done,
  and the old signature would have reported it complete.

- [x] **9b. `alo-files` onto `alo-strings`** — the largest list, and the one that
  includes words a person approves. Two new files in `crates/alo-files`:
  `words.rs` (every string this crate can say, the English beside each key, and
  the notes a translator needs) and `saying.rs` (what the six verbs are and what
  a person approves, in the language they read — **gone at 9g**, which put those
  three answers on the verb itself). `Failed`, `RealError`,
  `Touching`'s two refusals and `Did`'s all moved onto it; `verbs.rs` now
  declares the six **from** those constants. 41 phrases and one countable
  string, 14 new unit tests and 5 new integration tests; `Key::unchecked` is new
  in `alo-strings`. 578 tests and 20 doctests across the workspace, clippy
  clean.

  **The guarantee the item asked for is kept by there being one string, not two
  that agree.** `alo-capability` refuses a verb whose sentence does not name
  every argument, `alo-strings` refuses a translation that drops a gap the
  source has — and those are the same rule only while the string a translator is
  handed is the string the declaration was checked against. So the declaration
  reads the constant rather than repeating it, and a test walks all six.

  Three decisions the next items inherit. **`Failed` and `RealError` lost their
  `Display`**, which is the strong form of *hardcoded English is a bug*: a
  `Display` is one `to_string()` from a screen, in a shell whose author had no
  reason to think about it, so the only road to words is `said(&Strings)` and
  every answer says whether anybody translated it. What is given up is
  `std::error::Error` on two types that were never errors a programmer handles.
  **`Touching::of` and `Did::of` take the strings**, because the two refusals
  this crate words itself are carried into the record by
  `alo_capability::Refused` — so what a person was told is what is written down,
  one rendering rather than an English record beside a translated screen.
  **`Key::unchecked` takes a `&'static str`**, so a key can only come from a
  literal, and each crate that declares words puts every one of its own back
  through `Key::named` in a test — `alo-shortcuts`' shipped bindings and
  `alo-appearance`'s shipped wallpaper, one crate further on.

  What it does **not** do is put a translated sentence into a `Call` or an
  approval: `Call` renders and keeps its own, and moving that is 9e.

- [x] **9c. `alo-shortcuts` onto `alo-strings`** — the list a person reads every
  time they open the shortcuts panel. Two new files in `crates/alo-shortcuts`:
  `words.rs` (every string this crate can say — 38 phrases, the English beside
  each key, and the notes a translator cannot work without) and `refusing.rs`
  (why a combination cannot be a shortcut, and what it says). `Action`,
  `Modifier`, `Modifiers`, `Key`, `Chord`, `Taken` and `Clash` all lost their
  `Display` and gained `said` or `shown`. 63 unit tests (was 41), 6 new
  integration tests, 606 tests and 20 doctests across the workspace, clippy
  clean.

  **The item said "`Key::label`" and half of those labels turned out not to be
  strings at all.** Fifty-three of the sixty-nine keys print a mark that is the
  same on every keyboard in the union — `Q`, `7`, `,`, `F1` — and translating one
  would be naming a *position*, which is the model `key.rs` was built to reject:
  `Super+Q` is the key marked Q on the person's own keyboard. The other sixteen
  print a word, and it is a different word almost everywhere — *Entf*, *Pos1*,
  *Bild ↑* — so those are the strings, and they are the ones whose notes matter.
  Declaring all sixty-nine would have handed a translator forty-one rows reading
  `A`, `B`, `C` and made `unanswered` — *what a release note has to count* —
  report fifty-three strings nobody should ever translate. `docs/quirks.md`
  records it.

  Three decisions the next items inherit. **A sentence never joins a list**:
  `Taken` names the one action holding the chord in a gap, and `Clash` names the
  chord and hands `actions()` over to be drawn as rows, because the separator is
  not punctuation a program can pick — Greek writes `;` where English writes a
  question mark — and the conjunction would be a string placed by a machine that
  does not know the sentence. **A deserialiser has no `Strings` and never will**,
  so the one thing that still needs words where none can be asked for —
  `Chord`'s `serde(try_from)` — writes the *key* of the refusal rather than a
  sentence, and whoever shows it looks that key up. **`DefaultsError` keeps its
  English and its `Display`** and now names things by the stable names a
  settings file holds: it says a *release's* own defaults contradict themselves,
  which is read by whoever is fixing them, and `SnapLeft` is what they need
  rather than a row in whichever language was loaded. `Debug` on `Modifiers` and
  `Chord` is hand-written for the same reader.

  The stored format did not move: the settings file is written exactly as
  before, and the two serde tests say so.

- [x] **9d. `alo-appearance` onto `alo-strings`** — the eleven names a person
  picks a colour from, and every refusal about a value they chose. Three new
  files in `crates/alo-appearance`: `words.rs` (28 phrases, the English beside
  each key, and a note on every one of them), `unreadable.rs` (`NotRead` — what
  a settings file that did not read says, where nobody can be asked for words)
  and `testing.rs` (the fixture the other files' tests are written against).
  `Token`, `Accent` and all eight error types lost their `name` or their
  `Display` and gained `word` and `said`. `Word` moved into `alo-strings` as
  `word.rs` and all three declaring crates now use it. 98 unit tests (was 75), 8
  new integration tests; 635 tests and 20 doctests across the workspace, clippy
  clean.

  **A colour name is the one string that carries none of itself.** A sentence
  can be translated from its own words; *verdigris* cannot, and neither can
  *terracotta*, *charcoal* or *warm stone*. So every one of the 28 carries a
  note — the only one of the three lists where that is true — and the eleven
  colour ones describe the colour rather than assuming the word travels. The
  integration test is the argument made in German: *Grünspan* where English
  borrowed from French, *Anthrazit* where English named a grey after burnt wood,
  neither reachable from the other word by word.

  Three decisions the next items inherit. **`Word` lives in `alo-strings`
  now**, with `Word::phrase` replacing the loop each declaring crate had written
  out; `WordsError` in all three crates gained one `Word` variant in place of
  its `Sentence` and `Note` pair. **A deserialiser writes the key, and six of
  them share one type**: `serde(try_from)` requires a `Display` at the one point
  no `Strings` exists, so `NotRead` writes the key of the refusal and
  `docs/quirks.md` records it — `alo-shortcuts`' private `NotAChord` was the
  first of these and this is the sixth through tenth. **A number is not a
  string**: `TextScale` keeps `200%` and `TimeOfDay` keeps `18:00`, because how
  a number or a time is written belongs to the region rather than the language —
  but the percent sign is *in* the sentence, so a language that writes *200 %*
  can, and there is a test that says so.

  **`alo-strings`' integration test has gone**, which is what that file said
  would happen: it carried copies of four `alo-appearance` strings because it
  was built before any of its users existed, and all three users exist now. Its
  four tests live in `crates/alo-appearance/tests/what_this_crate_says.rs`,
  against the vocabulary the code actually uses rather than a copy of it.

- [x] **9e. `alo-capability` onto `alo-strings`** — every way the capability
  model says no, in the language the person reads. Three new files in
  `crates/alo-capability`: `words.rs` (33 phrases and one countable string, the
  English beside each key, and the notes a translator cannot work without),
  `refusing.rs` (`NotGranted` — why the grants refused, and what it says) and
  `testing.rs` (the fixture the other files' tests are written against).
  `GrantError`, `ArgError`, `CallError`, `ProposalError`, `AnswerError` and
  `NotAuthorised` lost their `Display`; `Reach::describe` and `Ask::describe`
  became `shown(&Strings)`. 107 unit tests (was 85), 6 new integration tests
  against the real vocabulary, 655 tests and 20 doctests across the workspace,
  clippy clean.

  **The decision this item turns on: a refusal is a value, and it is worded
  when somebody shows it.** Item 9b's rule — the words of a refusal are made
  where the refusal is made, so the record and the screen cannot disagree — is
  right and could not keep its shape here. Wording a refusal in this crate
  would mean handing `Grants` a `Strings`, and then *whether an agent may touch
  a folder* would depend on a vocabulary having been loaded. So
  `Grants::permitting` answers with a `NotGranted` carrying what was asked for
  and the grant that ran out, and `said(&Strings)` renders it. 9b's guarantee
  survives in the stronger form: the screen and the record render **the same
  value**, so neither can be a language the other is not.

  Three decisions the next items inherit. **`Refused` has two doors and they
  differ only in whose words they are**: `not_granted` for the refusal the
  grants made, `worded_elsewhere` for one only the crate holding a resolved
  path can say, which arrives already `Said`. **`Entry::refused` takes the
  strings rather than the words**, so a record cannot be handed a sentence
  about something else — `alo-record` depends on `alo-strings` for that and for
  nothing else. **Three errors keep their English**: `VerbError`, `VerbsError`
  and `SentenceError` refuse a *declaration* and are read by whoever is writing
  an adapter, which is `alo-shortcuts`' `DefaultsError` one crate on.

  What it could not close is 9f and 9g below.

- [x] **9f. `alo-models` onto `alo-strings`** — cut from 9e, which was two
  crates and one decision. Two new files in `crates/alo-models`: `words.rs` (29
  phrases, the English beside each key, and the notes a translator needs) and
  `refusing.rs` (`NotAllowed` — which rule refused a question and where it would
  have gone). `ProviderError`, `SecretError`, `RuntimeError` and `NotTried` lost
  their `Display`; `InferenceSource::describe` became `shown(&Strings)` and lost
  its `Display` too; `Tried::describe` became `said` and `caveats`. 20 new unit
  tests and 7 new integration tests against the real vocabulary; 682 tests and
  20 doctests across the workspace, clippy clean.

  **Two of these are not refusals, and that is what is different about this
  list.** *By someone, which has not said where it runs* is read **before** a
  person decides whether to paste a contract into a question (ADR 0008), so it
  is the one string here where a translation that softened it would take away
  the only thing on the screen saying the question is about to leave the
  building. Its note says so. The policy's three refusals put that clause inside
  themselves, which is why the source is a string rather than something a caller
  assembles: a refusal and the place named in it are one language.

  Three decisions the next items inherit. **A sentence that would have to count
  is a sentence with the number beside it** — item 10 settled that in this crate
  before `alo-strings` existed, and 9f kept it: `NotEnoughDisk` says *there is
  not enough room on this disk for that download* and carries the two numbers as
  fields, so nobody writes English's two plural shapes where a language has
  three. There is no `Plural` in this crate and a test says so. **A reason is a
  variant, never a `&'static str`**: `RuntimeError::Refused("…")` carried a
  sentence an adapter wrote in English, one `to_string()` from a screen, and is
  now `DownloadIncomplete` with a string of its own — an adapter that needs
  another reason adds one, the way a verb is added to a closed list. **A refusal
  the policy made is carried whole**: `NotTried::Forbidden` holds the
  `NotAllowed` rather than a rendering of it, so the words are the policy's in
  whichever language the person reads, and `Trying::under` still takes no
  `Strings` — what is permitted does not depend on a vocabulary having loaded.

  What it did not touch is `alo-egress`, which is 9h below and was not on any
  list before this iteration.

- [x] **9h. `alo-egress` onto `alo-strings`** — the last crate holding English,
  and the one with the indicator in it. Three new files in `crates/alo-egress`:
  `words.rs` (13 phrases, the English beside each key, and the notes a
  translator cannot work without), `refusing.rs` (`Refusal` — which rule refused
  — and `NotPermitted`, which moved here out of `policy.rs`) and `testing.rs`
  (the fixture the other files' tests are written against). `DestinationError`
  lost its `Display`, `Destination::describe` became `shown(&Strings)`,
  `Leaving::describe` became `said(&Strings)` and `Shown::describe` with it, and
  `EgressPolicy::refusal` answers with a value rather than a sentence. 45 unit
  tests (was 27), 7 new integration tests against the real vocabulary; 707 tests
  and 20 doctests across the workspace, clippy clean.

  **The indicator line is one sentence per reason, and that is the decision the
  item did not contain.** *`{agent} is asking a question of {destination}`* could
  have been a stem plus a place, which is three fewer strings and would have
  been wrong in English before it was wrong anywhere else: a question goes *of*
  somewhere, a fetch comes *from* it, and a language that inflects the place
  needs the whole sentence in front of it to choose. So there are three, each
  whole, and the preposition is inside the translated string where a translator
  can move it — `alo-shortcuts`' *a sentence never joins a list* met from the
  other side.

  Three decisions the next items inherit. **A destination that is data is not a
  string**: `Destination::word` answers `None` for a host a verb's argument
  named, because `alo.example` is somebody's address and a translation of it
  would be an invention — the rule a filename is held to in `alo-files`, now
  written into a type rather than a comment. **The rule and the refusal are two
  types**, because `alo-record` writes a held-back entry from a `NotPermitted`
  and from nothing else (item 5a): an enum carrying the egress in each variant
  would have made every variant a way to write down a refusal that never
  happened, so `NotPermitted` keeps private fields and a `pub(crate)`
  constructor the policy alone calls, and `Refusal` is the public value beside
  it. **`Entry::held_back` takes the strings**, as `Entry::refused` has since
  9e, so the record keeps the rendering the person read.

  What the twin guarantee turned out to be: `Destination::shown` and
  `InferenceSource::shown` **cannot** be one string, because one names where an
  answer came from (*by someone…*) and the other where a thing is going
  (*…of someone…*), which is a different grammatical position. What they must
  not do is differ about the provider, and that is a test walking every source
  that leaves: both name it, both name the region, and both say *has not said
  where it runs* when nobody has.

- [x] **9g. The sentence a person approves** — the question 9b left, 9c and 9d
  did not touch, and 9e answered without moving. A **public surface change**
  reaching `alo-capability`, `alo-files` and `alo-record` at once.
  `alo-capability`: `sentence.rs` rewritten around `alo_strings::Template`,
  `Arg` and `Verb` declared from `Word`s, `Call` carrying a `Key` and answering
  `sentence(&Strings)`, and `Proposal`, `Approved` and `Authorised` with it.
  `alo-files`: the six declared from the constants rather than out of them, and
  `saying.rs` gone. `alo-record`: `What::of` and four of `Entry`'s constructors
  take the strings. 108 unit tests in `alo-capability` (was 107), 7 integration
  tests (was 6); 707 tests and 20 doctests across the workspace, clippy clean.

  **The answer was 9e's, applied to the sentence, and the shape it took is a
  verb declared from the words rather than beside them.** `Verb::checked` and
  `Arg::taking` take an `alo_strings::Word` — a key, the English, the note —
  and there is no way to declare one from a bare string. That makes the
  guarantee structural rather than a convention `alo-files` happened to keep:
  the string a translator is handed is *necessarily* the string the declaration
  was checked against, so *a sentence names every argument* and *a translation
  may not drop a gap* are the same rule about the same string, in every crate
  that will ever declare a verb.

  **What the item did not say, and is the reason it was worth doing carefully:
  there were two parsers for one syntax.** `Sentence::parse` was this crate's
  own and `alo_strings::Template` is the vocabulary's, and they disagreed about
  `{{` — so a sentence with a literal brace in it was a declaration
  `alo-capability` refused and a phrase `alo-strings` accepted, which is the
  9-series' whole failure mode one level down. There is one parser now, and
  `sentence.rs` is what is left of the old one: the key, and the one rule that
  is about approval rather than about strings — a sentence made only of its
  arguments describes nothing.

  Three decisions the next items inherit. **`CallError::Unsayable` is gone**,
  because nothing renders a sentence at the moment a call is made any more and
  the variant could not happen — the unreachable branch deleted rather than
  translated, as in 9f and 9h. **`AnswerError::Lapsed` carries the call**, not a
  rendering of it, so the question quoted back to somebody is rendered with the
  vocabulary of the refusal around it; `docs/quirks.md`'s *two gaps in a
  translated sentence* is closed by that and by `CallError::Missing` carrying
  the purpose's key. **A word a verb is declared with can be one nobody
  declared**, which no check at declaration time can reach — it compiles and
  reaches a person as a key where the sentence they are approving belongs — so
  every crate that declares verbs owes the test `alo-files` now has
  (`everything_the_six_say_is_something_this_crate_declares`), and the contract
  says so.

- [x] **10. Test a provider before saving it** — implements `docs/features.md`'s
  v0.5 *test a provider before saving it* and closes the loose end in
  `provider.rs`. Three files in `crates/alo-models`: `secret.rs` (a key as it
  was just typed, for the length of one call), `tried.rs` (what a person is told
  — both shapes of one answer), `trying.rs` (the only file that knows what a
  provider's API looks like). One test-support file came with them,
  `testing.rs`, which `ollama.rs` now shares rather than keeping its own copy of
  the stub server. 26 new unit tests and 2 new doctests, one of them
  `compile_fail`; no new dependency.

  **The item is one sentence — a mistyped key is found now — and the design is
  the three things that sentence does not say.** *The policy is asked first, and
  there is no way to skip it*: `Trying::under` takes a `SourcePolicy`, so a
  machine set to keep questions in the building does not reach a provider
  outside it to see whether the key works, and a refused test opens no
  connection at all. *A redirect is refused rather than followed*, because the
  address the policy answered about is the address that gets reached and a key
  does not travel to a host nobody decided about. *Nothing of the person's
  leaves* — the test is a `GET` with no body, and the test that proves it reads
  what actually went out on the socket.

  Three decisions the next items inherit. **A key goes into this crate and does
  not come out**: `Secret` has no accessor, no `Display`, no `Serialize`, a
  hand-written `Debug`, and a `pub(crate)` reader asserted by a `compile_fail`
  doctest checked to fail on the privacy (E0624) — and it says outright what it
  does *not* claim, which is that the bytes are scrubbed on drop. **Names a
  provider wrote are held to `alo-files`' rule**: one that cannot be shown is
  counted and left out, a list longer than anybody reads is cut, and both are
  said in the answer. **A sentence here counts nothing out loud** — the plural
  rules are item 9a and are not written from memory — so `Tried::describe`
  carries no number and the numbers are accessors.

  Built and unit tested against a stub on a real socket. **Not run against a
  provider anybody pays for**, which is owed with the rest of the hardware
  verification; `docs/quirks.md` records the one convention this depends on and
  says the same thing about it.

- [x] **11. Application verbs, the portable half** — implements
  `docs/features.md`'s v0.01 *application verbs: open, focus, arrange, close*,
  and `docs/contracts/agent-verbs.md`, which gained a section for them.
  `crates/alo-applications`, a **new crate**: `verbs.rs` (three of the four,
  declared), `application.rs` (one installed application — the identifier it is
  granted by, and the name a person only ever sees), `installed.rs` (what this
  machine has, and how it is matched), `reaching.rs` (the only type meaning
  *this may reach an application*), `refusing.rs` (why this half says no),
  `words.rs` (13 phrases, the English beside each key, and a note on the ones
  that need one). 38 unit tests, 11 integration tests — 6 against the real
  vocabulary and 5 through the whole capability journey into `alo-record` — and
  1 doctest. **830 tests and 21 doctests across the workspace**, clippy clean.

  **The item was not on any list**, and finding it was the whole of the reading
  step. The queue had *application verbs, the acting half* under **blocked —
  linux** and nothing anywhere for the portable half, so a v0.01 promise had no
  item at all. That is item 6's shape a second time — the file verbs' acting
  half was listed as Linux's and was not — and it is the third blocker in three
  iterations that turned out to be part portable. **A blocker is a claim about
  code, and the claim about applications was only ever true of the half that
  drives a compositor.**

  **Three of the four, and the fourth was 11a below, because of what a choice
  does to an approval sentence.** `arrange` needs an argument saying where the
  window goes; that is a `Takes::Choice`, and a chosen option reached the
  sentence as the stable identifier a model picked it by — *put Blender on the
  `left_half`*. That is untranslated English inside the one string the whole
  capability model is built around, which is item 9g's guarantee failing for the
  one argument kind 9g did not reach. Wording around it here would have hidden
  it; declaring the verb anyway would have shipped it. So the scope was cut and
  the depth was not, and 11a built the fourth verb once the hole was closed.

  **The decision the item did not contain: closing asks, and the word is in the
  sentence.** `close_application` does what pressing the close button does — the
  application may put up its own *save your changes?* and the person answers
  that. Everything else these verbs do is reversible and unsaved work is not, and
  one approval covers one sentence: *ask Blender to close* is an approval to
  close an application, never to discard what is in it. Putting **ask** in the
  approval string rather than in a comment makes it a thing a translator is
  warned about and a reader cannot miss, and there is a test on the note.

  Three decisions the next items inherit. **The identifier is approved and the
  name is only shown**: the name is written by whoever packaged the application
  and two can claim one, so an approval sentence naming *Mail* could be chosen by
  the thing being approved — no two share an identifier, and a name that cannot
  be shown in one line is dropped while the application stays. **The list of what
  is installed is consulted second, never first**, and for a different reason
  than `alo-files`' order: answering *that is not installed* about an ungranted
  application would let an agent enumerate somebody's machine, so an ungranted
  application refuses identically either way and a test asserts the two refusals
  are the same string. **There is no read on this list at all** — what is running
  reaches an agent as context at invocation, and a `list_applications` would be
  the background reader `CLAUDE.md` forbids.

  Built and unit tested. **Nothing here has opened a window**: the acting half is
  Wayland and D-Bus and stays under *blocked — linux*.

- [x] **11a. A choice a person can read** — cut from item 11, and the reason
  `arrange` was not built. A **public surface change** reaching `alo-strings`,
  `alo-capability`, `alo-record` and `alo-applications` at once.
  `alo-capability`: `offered.rs` (a new file — one option a verb offers),
  `Takes::Choice` holding `Offered`s, `Value::Choice` carrying the name **and**
  the key, `Call::filling` taking the strings, and three new declaration
  refusals in `verb.rs`. `alo-strings`: `Filling::and_said`, the provenance of a
  filled gap carried through `Template::fill`, and `Said::is_translated`
  answering about the whole line. `alo-applications`: `arrange_application`,
  three arrangements, seven new words. 853 tests and 21 doctests across the
  workspace (was 830 and 21), clippy clean.

  **The answer is the one item 11 predicted and one half it did not.** An option
  is two things — a name a model sends and the record keeps, and a word a person
  reads — so `Offered` is both and neither stands in for the other; the reverse,
  identifying an option by its word, would let a translator change what a verb
  can be called and make the record say something different on a German machine
  than on a Greek one.

  **The half the item did not contain is what `Call::filling` does to a
  `Said`.** Rendering the option through the vocabulary puts a *string somebody
  translates* inside another one, and until now every gap held data — so a
  German sentence with an untranslated arrangement in it would have answered
  `Said::is_translated` with `true`, and been marked by nothing, counted by
  nothing, and read by somebody in Berlin. That is item 9's whole failure mode
  arriving through a gap rather than through a key. So `Filling::and_said` is
  the door for a gap that holds a word, `Filled::gaps_came_from` carries it, and
  **a sentence is only as translated as its least translated piece**.

  Three decisions the next items inherit. **An option's words complete the
  sentence rather than labelling a button** — the preposition lives in the
  option where a translator can move it, which is `alo-egress`' 9h decision met
  from the third side, and every option carries a note saying so. **A refusal
  names the options by name**, not by their words: a call that never validated is
  about what arrived, and `Arg::validate` still takes no `Strings`, so what an
  agent may do does not depend on a vocabulary having loaded. **Three things
  about an option are refused at declaration** — a name that is not an
  identifier, one offered twice, and one with nothing to say — which are the
  argument rules one level down.

  Built and unit tested. **Nothing here has moved a window**: the acting half is
  Wayland and stays under *blocked — linux*.

- [x] **12. Context on invocation, the portable half** — implements **ADR 0001
  §4** and `docs/features.md`'s v0.01 *★ context on invocation: focused window,
  selection, open document — offered, never watched*. It closes one of the six
  capability guarantees `CLAUDE.md` names in the gate, which until this
  iteration had no code to be a test of. `crates/alo-context`, a **new crate**:
  `context.rs` (what one invocation offered, and the moment), `focused.rs` (the
  window in front — told, never granted), `selection.rs` (the person's own text,
  bounded and saying so), `document.rs` (the only part that grants anything),
  `turn.rs` (the one turn it is good for, and the single grant it makes),
  `refusing.rs` (why a part could not be offered), `words.rs` (11 phrases and
  one countable string), `testing.rs`. 47 unit tests, 12 integration tests — 7
  against the real vocabulary in German, Polish and Greek, 5 through the whole
  capability journey into `alo-record` — and 3 doctests, two of them
  `compile_fail`. **912 tests and 24 doctests across the workspace**, clippy
  clean.

  **The item was not on any list**, which is item 11's finding a second time and
  worse: a ★ v0.01 promise, named in ADR 0001 as one of the eight numbered parts
  of the capability model, with no queue item anywhere and no crate. It was not
  even in *blocked — linux* under another name — the compositor line covers
  reading a screen, and nothing covered what a context **is**.

  **The decision the whole crate turns on: only the document grants anything.**
  ADR 0001 §3 names two deliberate acts that make a grant, and a context carries
  one of them. A window somebody was looking at is not a decision to hand
  anything over, and neither is text they had highlighted — so the focused
  window is *told, not granted*, and an agent that knows Blender is in front of
  the person still cannot touch Blender. Reading it the other way round is the
  quiet mistake: a capability model decided by where somebody's mouse was.

  **The decision the item did not contain: a grant a context makes is a grant
  like any other.** It goes into the machine's own `Grants` rather than into a
  list of this crate's, because ADR 0001 §3 says grants are enumerated, visible,
  revocable and expiring, and authority kept somewhere else would satisfy none
  of those four while still deciding what an agent may touch. It ends twice
  over: it **expires** at the turn's end, so a daemon that forgets a turn still
  has an agent that reaches nothing, and `Turn::ending` **revokes** it, so a turn
  that finishes early does not leave the document reachable for the rest of its
  allotted time.

  Three decisions the next items inherit. **This crate has no serde dependency
  at all**, and that is the guarantee rather than an omission — a context that
  could be read back off a disk would be one existing without an invocation, so
  *offered, never watched* is the absence of a dependency rather than the
  absence of a constructor somebody remembered not to write. **Nothing here
  reaches `alo-record`**: an entry per invocation saying what was on somebody's
  screen would build the watched-context log ADR 0001 §4 forbids, one entry at a
  time, so what the record keeps is what the agent then *did* and the grant it
  did it under. **What is offered is shown to the person** — one row per part,
  and a row saying nothing was offered — because a rule nobody can check is a
  promise, and somebody who cannot see what they are offering cannot tell this
  system from one that watches all day.

  Built and unit tested. **Nothing here has read a screen**: what is in front of
  somebody, what they have selected and what they have open are Wayland's and
  AT-SPI's to answer, and that half is under *blocked — linux* below.

- [x] **13. The dock, and the edge a person puts it on** — implements
  `docs/features.md`'s v0.01 *★ the dock, and the person decides where it goes*.
  `crates/alo-dock`, a **new crate**: `edge.rs` (the four, and the whole of what
  a person chooses), `along.rs` (which way it runs, and the one thing that never
  turns with it), `measures.rs` (the numbers, and what each answers to),
  `room.rs` (how much room something takes, and the arithmetic), `screen.rs`
  (the screen it is laid out on, and the side it takes from), `labels.rs` (what
  became of the names), `status.rs` (the status area: which end, which way),
  `layout.rs` (the whole answer, worked out), `shipped.rs`, `changes.rs`,
  `dock.rs` (the two resolved), `words.rs` (9 phrases), `testing.rs`. 56 unit
  tests, 8 integration tests — against the real vocabulary in German and Greek —
  and 1 doctest. **976 tests and 25 doctests across the workspace**, clippy
  clean.

  **The item said the threshold was the thing to decide, and it was.** *Labels
  give way to icons where the short edge demands it* is arithmetic now: a dock
  may take one part in six of the side of the screen it sits on, a name needs a
  line of text under an icon or five ems of width beside one, and the names stay
  while both fit. The two numbers are not taste — they are the loosest pair that
  keeps EN 301 549's *200% without loss of content* on the smallest screen alo
  OS lays out for, on all four edges, and `layout.rs` has the test that says a
  tighter share would fail it. **An em is the unit** because nothing in this
  crate can measure text and an em is the text's own size, so it scales without
  a font in the room.

  **The decision the item did not contain: which side a dock takes from.** The
  obvious reading of *the short edge* is the screen's short side, and it is
  wrong — one number would let a dock down the left of a wide screen grow while
  a dock along its bottom was squeezed. A dock takes from **the side it sits
  on**, so a wide screen gives a side dock more room than a bottom one, which is
  what makes the two orientations two layouts rather than one rotated. The other
  two halves of that: a name **beside** an icon needs a width where a name
  **under** one needs a line height, and text is never turned ninety degrees; and
  the status area is a **column** at the bottom of a vertical dock, while the far
  end of a horizontal one follows which way the person reads — the left for
  somebody reading Arabic. A column does not turn over when the reading does,
  because every script alo OS ships is read downwards.

  Three decisions the next items inherit. **A refusal at the door is what lets an
  answer be an answer**: `Screen::of` refuses a screen too small to hold a dock,
  so `Layout::of` returns a `Layout` rather than a `Result` — and the floor it
  refuses below is worked out from the ceiling rather than picked. **Giving way is
  not taking away**, and the reassurance is *inside* the string rather than beside
  it: `dock.labels.gave-way` says the name is still announced and still shown on
  hover, so a translator is handed it and a checked translation cannot lose it
  quietly — which is what keeps the 200% rule true when the names do eventually
  go. **A standard is asserted about what the code hands out, never about the
  constant behind it**: clippy folds an assertion whose expression is a constant,
  so `ICON >= 24` written directly is a test that cannot fail; it is written
  about `Room::an_icon()` instead, and the same is true of every crate that will
  later assert a floor.

  Built and unit tested. **Nothing here has drawn a dock**: the icons, what is in
  the status area (v0.5), and the hover and screen-reader name the *gave way*
  sentence promises are all the compositor's, and that half is under *blocked —
  linux* below. Per display and hiding when a window needs the room are v0.5 and
  are deliberately not built.

- [x] **14. Never a silent fallback** — implements **ADR 0008** and
  `docs/features.md`'s v0.01 *★ Never a silent fallback: a local model that
  fails does not quietly become an API call*. Found by iteration 27's reading
  step, built by iteration 28. `crates/alo-answering`, a **new crate**:
  `answering.rs` (the only type meaning *this question may be answered here*,
  and the two doors into it), `wrong.rs` (what can go wrong where a question was
  put, and what cannot go wrong there), `failed.rs` (it did not answer: what a
  person reads, and the one door out), `elsewhere.rs` (where else this machine
  may ask, and the doors a rule has closed), `offer.rs` (one place that could be
  asked instead, and the sentence a person approves), `refusing.rs` (an offer
  that was not this failure's, carrying the failure back), `words.rs` (12
  phrases, every one of them with a note), `testing.rs`. 40 unit tests, 10
  integration tests — 6 against the real vocabulary in Greek and Estonian, 4
  through `alo-egress` and into `alo-record` — and 3 doctests, two of them
  `compile_fail`. **1055 tests and 28 doctests across the workspace**, clippy
  clean.

  **The decision the item asked for, answered from ADR 0008: it is a change,
  not a setting.** A switch turned on in advance — *when the local model fails,
  use my provider* — **is** the fallback that ADR rejects, with a checkbox in
  front of it. The objection was never that alo OS decides badly; it is that
  the person is not there, and their records leave the building at the moment
  of a failure they never saw. A box ticked in March does not make somebody
  present in June. So the shape is ADR 0001 §5's — one sentence, one approval,
  one attempt, and an approval is never a session — carried to a place §5 does
  not itself reach, because §5 binds an agent changing this machine and a
  person's question is not that. `offer.rs` says why it is deliberately not an
  `alo_capability::Proposal`: building one would mean a verb whose argument is
  somebody's own question, which puts the thing ADR 0001 §4 keeps *out* of the
  capability model inside it.

  **Why a crate of its own rather than more of `alo-models`.** The promise here
  is about the **absence** of code — nothing ever asks a second place on its
  own — and a promise like that is worth what the code around it is small
  enough to prove. `alo-models` carries `ureq` and a TLS stack; a fallback
  hidden in it would be a line in a function. This crate has **no HTTP client,
  no socket and no serde**, so *the crate that decides where a failed question
  may go next cannot itself go there* is checkable from `Cargo.toml`. It is
  `alo-keeping`'s argument about `alo-record`, made about a second attempt
  rather than about deleting evidence.

  Three decisions the next items inherit. **A sentence is only as translated as
  the clause inside it**, so `InferenceSource::said` is new in `alo-models` and
  every sentence here fills `{source}` with `Filling::and_said` — item 11a's
  rule reaching the crates that were written before there was a door for it;
  `NotAllowed::said` was fixed in passing and the rest is item 15 below.
  **This crate holds no text anybody else wrote** — not the question, not a
  model's name, not what a provider said about itself — so `WentWrong` carries
  a status number and nothing else, and every sentence it produces is one alo OS
  wrote rather than one it passed on. **`Failed` is not `Clone`**, which is
  `alo_capability::Approved`'s rule: a clone would be a second way to take an
  offer from one failure, and *one failure, at most one attempt elsewhere*
  would hold only for callers who did not think of it.

  Built and unit tested. **Nothing here has asked a model anything**: there is
  no method that puts a question to one, because `ModelRuntime` has none — the
  asking arrives with the daemon, and this is the decision that had to be
  settled before then so that a fallback cannot be written into it by accident.

- [x] **15. A gap that holds a sentence** — cut from item 14, which needed one
  case of it and found the rest. A **public surface change reaching seven
  crates**, additive in six of them. `alo-strings`: `Filling::and_composed` (a
  gap holding a value assembled out of several things this crate said),
  `Filling::and_said` now carrying the gaps of the answer it is given, and
  `Filling::came_from` answering `&[CameFrom]` — the one break, and the only
  way either of the other two could be honest. `alo-capability`: `Reach::said`,
  `Ask::said` and `Ask::fills`. `alo-context`: `Focused::said` and
  `Focused::fills`. `alo-egress`: `Destination::said` and `Destination::fills`.
  `alo-shortcuts`: `Chord::fills` and `Key::fills`. Ten call sites moved, in
  `alo-capability`, `alo-context`, `alo-egress`, `alo-shortcuts`,
  `alo-appearance` and `alo-files`. **1042 tests and 28 doctests across the
  workspace** (was 1027 and 28), clippy clean.

  **The item said eight sites and there were ten**, because its grep looked for
  a gap filled from a `shown(strings)` and two of them were filled from
  `said(strings).into_text()` — a `Said` made correctly and then thrown away one
  line later. `alo-appearance`'s `accent.rs` is one and `alo-files`' `doing.rs`
  is the other, and the second is the worst of the ten: it is the one sentence
  in the workspace with **another crate's sentence** inside it.

  **The item's prescribed shape did not fit two of the four kinds, and that is
  the design this iteration had to make.** *A `said` beside the `shown`* assumes
  a clause is one string somebody translated. Two are not. A **chord** is
  composed — `Super+Bild ↑` is notation with a name in it for each key, and no
  translator is handed it whole — and a **destination**, a **key that prints a
  mark**, an **ask that is a path** and a **window with no title** are each
  sometimes a word and sometimes somebody's own data. So the provenance of a gap
  became a **list**: empty for data, one entry for a clause, several for a
  composed one. That is `Filling::came_from` answering a slice, and it is what
  makes *only as translated as its least translated piece* true of a chord and
  of a nested refusal at the same time, by the same rule.

  Three decisions the next items inherit. **A gap holding data can never make a
  line untranslated**, and that is asserted as carefully as its opposite: a
  German line naming `alo.example`, `/home/anna/Taxes/2024.pdf`, `#7F4A2D`,
  `org.gimp.GIMP` or `Super+Q` is a German line, and a rule that said otherwise
  would put a release note's count permanently out by the number of files
  anybody happened to ask about. **`and_said` carries the gaps of what it is
  given**, so the rule holds at any depth rather than one level down —
  `alo-files`' refusal with `alo-capability`'s inside it is the case, and it had
  no other honest answer. **A word whose translation is the same as its source
  is still a translation somebody has to make**: `alo-shortcuts`' German fixture
  had never translated *Super* or *Alt*, nothing had ever noticed because the
  text matched, and the first thing this item did on that crate was fail that
  test.

  What is **not** covered anywhere and is not this item: *★ No telemetry*
  (v0.01, the egress section). `alo-egress` decides about egress an **agent**
  causes; a promise that alo OS itself sends nothing is about egress with no
  agent behind it, and there is neither a crate nor a blocked entry for it.
  Whoever reads next should decide whether its portable half is a rule in
  `alo-egress` or whether all of it is the daemon's — it is the fifth promise
  the journal has watched go unlisted, and the third iteration to say so.
  **Answered by item 16 below**: most of it is portable, and it is a rule in
  `alo-egress`.

- [x] **16. Egress with no agent behind it** — implements `docs/features.md`'s
  v0.01 *★ No telemetry. Not "anonymised telemetry". None — and the policy lives
  in a Rust service, not a checkbox*, which three iterations of this journal
  watched go unlisted. Four new files in `crates/alo-egress`: `errand.rs` (the
  closed list of reasons alo OS itself reaches the network, and the promise
  beside it), `itself.rs` (`OnItsOwn` — one of them about to happen, and the
  line a person reads), `underway.rs` (the only type meaning *alo OS may do
  this*), `showing.rs` (what one line of the indicator is about, whichever kind
  it is). `indicator.rs` gains a second door onto the same list; `words.rs`
  gains four phrases. 70 unit tests in the crate (was 45), 9 integration tests
  (was 7) — in German and Greek — and 5 doctests, two of them `compile_fail`.
  **1066 tests and 30 doctests across the workspace** (was 1042 and 28), clippy
  clean.

  **The question the item had to answer first — a rule here, or all of it the
  daemon's — turned out to have an answer neither iteration 27, 28 nor 29 had
  in front of them: the promise is a *type*, and the daemon's half is only the
  wiring.** *No telemetry* is law 2's shape applied to what the system does
  rather than to what an agent does: `Errand` is a closed list of three, there
  is no `Other(String)` and no diagnostics member, so a machine cannot report on
  its owner for the same reason a verb cannot run a shell — there is no shape
  for it to arrive in. A checkbox is what the feature list contrasts the promise
  with, and the difference is exactly that a checkbox has the code behind it
  either way.

  **The decision the item did not contain, and the one that shaped every file:
  it goes on the same indicator.** A second list would be a second place to
  forget, and the failure law 1 exists to prevent is not *the policy was wrong*
  but *nobody could see it*. So `Indicator::is_quiet` is false while alo OS
  fetches a model, `Shown` holds either kind, and the promise being kept is
  stronger than *no telemetry*: it is *nothing at all that you cannot see*.

  Three decisions the next items inherit. **The organisation's egress policy is
  not asked about an errand**, and that is deliberate rather than an omission:
  `EgressPolicy` is `From<&SourcePolicy>` — a rule about where a *question* may
  be answered — and applying it to a model download would stop a machine set to
  `ThisMachineOnly` from ever fetching the model it is set to answer with. A
  policy that defeats the setting it came from is worse than no policy, and
  there is a test named for that machine. **There is no agent, and the type has
  no room for one**: giving alo OS a `Grantee` would say the system acts under a
  grant and would put it inside the capability model, so `Showing::agent`
  answers `None` for an errand rather than naming something invented. **A twin
  type rather than a widened one**: `Underway` is `Departing`'s twin because
  widening `Departing` would make `alo-record`'s *whose authority was this
  under* an `Option` in every entry it writes, in the crate whose whole job is
  saying who did what — the one break is `Shown::leaving`, which now answers
  `Option<&Leaving>`, and nothing outside this repository exists to break.

  Built and unit tested. **Nothing here has opened a connection**: what fetches
  a model, signs somebody in or checks for an update is the daemon's, and item
  16a below is what the record of it owes.

- [x] **16a. The record of what alo OS did on its own** — cut from item 16, and
  cut for a question rather than for size: law 1's second half is *and
  afterwards in a record*, and until this item there was no shape for a
  departure nobody caused to be written in. A **public surface change** reaching
  `alo-record` and the record file. `alo-record`: `Happened::LeftOnItsOwn` (a
  new variant, no agent field), `Happened::agent` answering `Option<&Line>`,
  `Happened::errand` and `Happened::on_its_own` new beside it,
  `Entry::left_on_its_own` in `departed.rs`, and `Only::OnItsOwn` in
  `explain.rs`. `alo-keeping` gains `alo-egress` as a dev-dependency and one
  integration test onto a real disk. 9 new tests and 2 new doctests, one of them
  `compile_fail`; **1076 tests and 32 doctests across the workspace** (was 1066
  and 30), clippy clean.

  **The decision the item asked for, and it is the first of the two it named.**
  A *stable identity for the system that is not a `Grantee`* was the tempting
  answer and is wrong in the one place it matters: whatever type it was, it
  reduces at the record file to a string in the position that answers *whose
  authority was this under*, and nobody granted alo OS anything. It would be
  read back by `Asking::by`, so a question about one agent's day could be
  answered with the machine's; it would sit in a SIEM's *who did what* column
  beside agents that really were granted something; and no spelling of it could
  be reserved, because the record is written by one alo OS and read by tools
  that are not it. So the variant has **no agent field**, `Happened::agent`
  answers `None`, and there is a test that walks five plausible spellings and
  finds nothing. It is `alo-egress`' answer one crate on — *the honest shape is
  a field that does not exist rather than one that is always empty* — and the
  record follows the indicator because a person watching something leave and a
  person reading about it afterwards are asking one question.

  **The half the item did not contain: whether a new `Happened` is additive.**
  It called one additive in passing, and `docs/contracts/record-file.md` did not
  say — its additive rule was written about *fields* inside an entry, and a new
  tag is the case it does not cover. An older reader cannot parse
  `left-on-its-own` at all. The answer written into the contract is that it **is
  additive and does not raise `format`**, for two reasons that are about what
  the alternative does: raising it makes the whole file unreadable to that
  reader rather than one line of it, and it would tie the record's version to
  the growth of the capability model, so a security team's tooling would stop
  reading a machine's record the first time alo OS learned to do something new.
  What makes it safe is already built — the file is appended to and never
  rewritten, so an older writer loses nothing, and an older *shortening* refuses
  a record with a line it could not read, so the version that does not
  understand an entry is the version that will not remove it.

  Three decisions the next items inherit. **`Only::Egress` counts everything
  that left**, errands included, because item 16's *one indicator, not two* is
  the same argument asked afterwards: a question that answered with the agents'
  share would be the second place to look. Which of the two it was is
  `Only::OnItsOwn`, which narrows rather than sits beside. **An entry is still
  made from the type the indicator hands out and from nothing else** —
  `Entry::left_on_its_own` takes an `Underway`, and the `compile_fail` doctest
  was checked to fail on the privacy (E0624) and not on a typo. **There is no
  `held_back` twin, and the absence is the design**: an errand is decided by
  being on the closed list and by nothing else, so there is no refusal for a
  record of one to be made from.

  Built and unit tested, and written to a real filesystem by `alo-keeping`'s
  integration test. **Nothing here has opened a connection**: what fetches a
  model, signs somebody in or checks for an update is still the daemon's.

- [ ] **16b. Finding machines on the local network, on the indicator or not** —
  the one thing item 16's list does not cover and says so. Discovery (ADR 0003,
  `docs/features.md` v0.5) announces and listens rather than reaching a named
  destination, so there is no `Destination` for it to be an errand about — and
  `alo-egress` has said since item 5 that a host answering on the same wire is
  *outside* the building, which makes multicast on that wire a thing a person
  might reasonably expect to see. This item decides which it is: an `Errand`
  with a destination of its own, or a documented exception with the reasoning
  written where somebody checking the no-telemetry claim will find it.

  **Blocked on nothing here, and not ready either**: there is no discovery code
  in this repository and none of it is portable, so deciding now would be
  deciding in the abstract about a shape nobody has built. It is listed so the
  hole in the list is a known hole rather than a discovered one, which is what
  the last four iterations kept finding was the difference.

- [x] **17. A machine with no agent at all** — implements **ADR 0009** and
  `docs/features.md`'s v0.01 *★ Or not at all. Setup's fourth choice, with the
  same weight as the other three: no model, no provider, no agent.* Found by
  iteration 30, built by iteration 32. One new file in `crates/alo-capability`,
  `agent.rs` (`Agent` — the choice, the two acts, and everything it may reach);
  `NotGranted::NoAgent` in `refusing.rs` and three new words. `alo-record`:
  `Only::ByAnAgent` in `explain.rs`. 20 new unit tests, 2 new doctests (one of
  them `compile_fail`), 2 new integration tests through `alo-capability`,
  `alo-egress` and `alo-record` at once, and 1 more in `alo-context`.
  **1094 tests and 34 doctests across the workspace** (was 1076 and 32), clippy
  clean.

  **Most of the ADR is the shell's and was not this item.** The hotkey doing
  nothing, the overlay not existing, and Grants, Models and providers being
  *absent* from Settings rather than greyed out are compositor and
  settings-panel work, and they are written into *blocked — linux* below.

  **The decision the item existed to make had a third answer, and it is both of
  the two it offered.** A flag beside `Grants` is the shape this repository
  refuses; the absence of any grant is stronger and cannot on its own survive a
  restart. So the list went **inside** the choice: `Agent::Present(Grants)` or
  `Agent::Declined`, and a declined machine holds no `Grants` at all rather than
  an empty one. There is nothing to remember to check because there is nothing
  to check — the only road to the machine's grants is through the choice, and on
  a declined machine it stops. That is the state (it serialises, so *turning it
  on later is a setting, not a reinstall* is true) and the absence (nothing can
  be granted, because `grants_mut` has nothing to lend), and the two cannot
  disagree because they are one value.

  Three decisions the next items inherit. **`Agent` has no `Default`**, asserted
  by a `compile_fail` doctest checked to fail on E0277 rather than on a typo: a
  default would be alo OS answering setup's fourth question on the person's
  behalf, in the type that exists because the question is theirs. **Turning it
  on again brings back an agent and not the folders** — ADR 0009 says grants
  *end*, and a suspension that restored itself would be a weaker promise wearing
  the same sentence; the grant an invocation's document made (item 12) ends with
  everything else, and `alo-context` has the test. **A refusal on a machine with
  no agent is a third `NotGranted` rather than a narrower `Never`**, because the
  grants panel is absent on that machine and *grants are made by picking a
  folder* would send somebody somewhere their machine does not have.

  **What the item did not contain: whether a refusal is still recorded.**
  *Nothing further is recorded as agent activity* reads like a licence to stop
  writing, and it is not — it is a statement about a machine with no agent doing
  nothing. If something does ask, that is exactly the entry somebody who
  declined would want, and `CLAUDE.md`'s gate says every refusal leaves one. So
  the promise is checked as the shape it really has: an ordinary day on a
  declined machine has no entry with an agent's name on it, and a call that
  arrives anyway is refused *and* written down. `Only::ByAnAgent` is what makes
  the first of those a question rather than a list of names somebody trusted.

  **No `Happened` variant was added, and the absence is the decision.** An entry
  saying *the person turned the agent off* was tempting and is the wrong crate:
  the record holds what an agent caused and what the machine did on its own, and
  a person's own act on their own machine is neither. A record that started
  keeping settings changes would become a log of the person rather than of the
  agent, which is ADR 0001 §4's watched context arriving through the back door.

  Built and unit tested. **Nothing here has hidden a panel**: the surfaces ADR
  0009 makes absent are the shell's, and are listed below.

## Ready — `alo-agentd`, the daemon nothing else can proceed without

**Five roadmap halves are all waiting on this one thing**, and it has never been
started: *or use an API instead*, *agents point at the local model*, *every
execution recorded*, `alo-agentd` itself, and the model stack's own last mile.
The roadmap said the same sentence three separate times — *there is still no
method anywhere in this repository that puts a question to a model* — and it is
now false in both of the two places this repository can reach. **Item 18
answered it for a hosted provider**: `alo-asking` sends one, shows it leaving,
and brings back the answer, the departure and the failure. **Item 18a answered
it for a model on this machine**: the same crate, a second door, and nothing on
the indicator because nothing goes anywhere. What is left of the sentence is the
third place, a machine on this network, which neither door reaches and both
refuse in words. **Item 19 joined the rest of them**: fifteen crates decide
correctly, two of them act, one holds the order the others happen in, and there
is still no daemon holding that.

**All of it is portable.** No compositor, no certified machine, no GPU. A turn
is a function call, and its result is a value to assert on. The acting halves
that genuinely need Wayland and D-Bus are marked below and stay out.

- [x] **18. Putting a question to a model — the hosted provider first.** The gap
  named three times in `ROADMAP.md`, and the first thing in this repository that
  ever sent anything on somebody's behalf. `crates/alo-asking`, a **new crate**:
  `question.rs` (what was asked, held the way a key is), `hosted.rs` (the only
  file that knows what a provider's chat API looks like), `asking.rs` (the one
  door, and the order the four steps happen in), `asked.rs` (an answer and the
  departure it came with), `unanswered.rs` (a question that left and did not come
  back, and the departure it left with), `answer.rs` (what came back, and always
  where it came from), `refusing.rs` (the four things that can come back
  instead), `words.rs` (2 phrases), `testing.rs`. 39 unit tests, 7 integration
  tests — 4 against the real vocabulary in Greek, 3 through `alo-egress` and into
  `alo-record` — and 4 doctests, two of them `compile_fail`. **1140 tests and 38
  doctests across the workspace** (was 1094 and 34), clippy clean.

  **A new crate rather than more of `alo-models`, and the dependency graph
  decides it.** The item's own sentence is *`alo-egress` consulted before a
  socket opens*, and `alo-egress` depends on `alo-models` — so the method cannot
  live in the crate that holds the provider without inverting an edge that
  exists for a reason. What is left in `alo-models` is what was always there: the
  provider, the key, and where an answer came from. This crate is the joining-up,
  and it reaches five crates and is reached by none.

  **The whole of it is the order, and every step produces what the next one
  needs.** The permission and the provider must be the same place; the place must
  be showable; the rule in force **now** must permit it, which is the same call
  that puts it on the indicator; and only then is anything opened. `hosted.rs` is
  `pub(crate)`, so there is no public function here that reaches a provider
  without law 1 having shown it first.

  **The decision the item did not contain: the departure comes back either
  way.** `alo_record::Entry::left` is made from a `Departing` and from nothing
  else, so a crate that took the line off the indicator itself would leave the
  record of what left *impossible to write* — in the one crate that causes the
  largest egress this product has. So both `Asked` and `DidNotAnswer` carry the
  departure and an `ended(&mut Indicator)` that spends it, and this crate still
  reaches `alo-record` from nowhere. It is item 6a's *the authorisation comes
  back either way*, about a departure. The other half of it is that **a question
  that failed still left the machine**: a machine that recorded only the
  questions that were answered would report a quieter day than it had.

  Three decisions the next items inherit. **The rule is asked twice and the
  second time is the one that counts** — item 3's *the grants are asked last, at
  the moment of execution* arriving at egress, so an organisation that tightened
  its rule between the choosing and the asking has a machine that sends nothing.
  **A question and an answer are held the way a key is**: no `Serialize`, a
  `Debug` written by hand, and the question's only reader `pub(crate)` — ADR 0001
  §7 keeps neither, and this is the only crate that has to hold them at all.
  **`alo_models::Secret` gained one method and no accessor**:
  `Secret::carried_by` takes a request and gives it back with the key on it, so
  a second crate can *send* a key without any crate being able to read one, and
  `bearer` stays private with its `compile_fail` doctest intact.

  **`alo-answering` gained a seventh `WentWrong`** — `SentSomewhereElse`, and it
  passes that crate's own bar in a way the other six do not: it is not a failure
  at the far end at all but a **refusal alo OS made**, and telling somebody
  *nothing usable came back* would hide the one thing that happened, which is
  that their machine stopped their question going to an address nobody agreed to.

  Built and unit tested against a stub on a real socket. **Not run against a
  provider anybody pays for**, which is owed with the rest of the hardware
  verification — `docs/quirks.md` records the two conventions it depends on and
  says the same thing about both.

- [x] **18a. The same path to a model on this machine.** The half item 18 cut,
  and the third thing this repository has ever done to a model. `alo-models`:
  `ModelRuntime::answers` — the method that trait never had — plus
  `RuntimeError::TookTooLong` and its word, and the Ollama adapter carrying a
  question over `/api/chat` (ADR 0006's *the only file that knows Ollama
  exists*). `alo-asking`: `locally.rs`, the second door, and `NotAnswered` in
  `refusing.rs`. 96 unit tests in `alo-models` (was 90), 48 in `alo-asking` (was
  39), 3 new integration tests through `alo-answering`, `alo-egress` and
  `alo-record` at once, and 1 new `compile_fail` doctest. **1157 tests and 39
  doctests across the workspace** (was 1140 and 38), clippy clean.

  **Two doors, and what divides them is law 1 rather than what speaks at the far
  end.** `to_a_provider` is four steps because something leaves;
  `to_this_machine` is two because nothing does. It takes no `Indicator`, makes
  no `Departing` and asks no policy — the last because there is no rule that can
  forbid a machine answering its own question, which is walked as a test over
  every `SourcePolicy` rather than written down as a sentence, so a fifth
  variant that did forbid it fails there instead of being permitted by a door
  that never asks. `docs/features.md`'s *a working day with a local model
  produces zero inference egress* is as far as code can carry it: the door has
  no parameter for an indicator, so there is no line to forget.

  **The decision the item asked for, and both halves of it.** An
  OpenAI-compatible service somebody pointed at loopback is **this machine** in
  what it means — `Provider::source` already says so, nothing leaves, an answer
  from it says *on this machine* — and it is **not the runtime** in what it is:
  alo OS cannot list, fetch, load or remove models on a service it did not
  install, so a `ModelRuntime` for one would be four methods that only refuse,
  which is a stub wearing an interface. So this door takes the runtime alo OS
  ships and item 18b below is the other local shape. `docs/quirks.md` records
  what came with that: **loopback is taken at face value**, so a proxy on
  `127.0.0.1` would be believed by every type here, and the place that is caught
  is egress enforcement at the network boundary.

  Three decisions the next items inherit. **A refusal names the door the
  permission's own place is behind, and never the other one** — `Miswired` gained
  `NotTheRuntime` and `NoPathToAPairedMachine`, and `NotAProvider` stopped saying
  *ask the runtime instead* about a paired machine, which was harmless advice
  while no local path existed and is a substitution ADR 0008 forbids now that one
  does. **The catalogue gates downloading and not asking**: offering is what we
  fetch, and a model already on somebody's disk is theirs — refusing to use it
  would be alo OS overruling the owner of the machine. **A slow model is not a
  missing one**: `RuntimeError::TookTooLong` exists because ADR 0007 makes the
  CPU the default, so thinking for five minutes is ordinary, and *nothing was
  running* would send somebody to look at a fault that is not there.

  Built and unit tested against a stub of the trait, and the adapter against a
  stub on a real socket. **Not run against a real Ollama on any machine**, which
  is owed with the rest of the hardware verification; `docs/quirks.md` records
  the runtime's two chat APIs and says the same thing about them.

- [x] **18b. An OpenAI-compatible service somebody runs on this machine.** Cut
  from 18a, which answered what it is and did not build it: vLLM, llama.cpp's
  server or LM Studio on loopback, which alo OS did not install and cannot
  manage. It is `InferenceSource::ThisMachine` and causes no egress, so it
  belongs behind the local door — and it is not a `ModelRuntime`, so
  `to_this_machine` as it stands could not carry it. **A third door**:
  `crates/alo-asking/served.rs` (`Served`, and
  `Asking::to_a_service_on_this_machine`) and `openai.rs`, which is the wire
  moved out of `hosted.rs` so that two things speaking one convention cannot
  become two renderings of it. `crates/alo-models/address.rs` is new and is the
  security half. 64 unit tests in `alo-asking` (was 48), 103 in `alo-models`
  (was 96), 2 new integration tests through `alo-record` and the indicator, and
  1 new `compile_fail` doctest. **1183 tests and 40 doctests across the
  workspace** (was 1157 and 39), clippy clean.

  **The dangerous part was the reason it was its own item, and it was worse than
  the item said.** Whatever carries this reaches the request shape without a
  `Departing`, which is a second road to a socket in the crate whose whole design
  is that there is one — so `Served::at` refuses any address
  `alo_models::Provider::source` does not call this machine, and there is no
  other constructor. **What may be reached without an indicator is decided by
  whether a value exists**, which is `Touching` and `Departing`'s shape brought
  to the one path in this crate that had no token of its own.

  Then the check it delegates to turned out not to hold. `Provider::source`
  asked whether the address *started with* `127.0.0.1`, `localhost` or `::1`
  after the scheme, so `http://localhost.attacker.example`,
  `http://127.0.0.1.attacker.example` and `http://127.0.0.1@attacker.example/`
  were all this machine: reachable over unencrypted http **with a key attached**,
  and — once this door existed — a question leaving with the indicator quiet,
  which is law 1 failing in the one way law 1 exists to prevent. `address.rs`
  parses the authority the way a URL is written and matches the host whole;
  `is_loopback` no longer exists. That is why the fix is in `alo-models` rather
  than in the new door: **two rules about loopback is one machine able to
  disagree with itself about whether a question left.**

  Three decisions the next items inherit. **A rule the new code depends on is
  read before it is depended on** — this item's own check would have passed
  against a broken one, and the queue would have recorded a guarantee that was
  not there. **An address that cannot be parsed is somewhere else**, so every
  unreadable case falls towards *refused over http, shown on the indicator, not
  carried by the local door*; `http://127.1` is the cost and `docs/quirks.md`
  says why that is the right way round. **`WentWrong::KeyNotAccepted` is no
  longer impossible on this machine** — a service somebody started with
  `--api-key` really can refuse one — so `alo-answering` narrowed that refusal to
  a paired machine, and *the runtime is never given a key* moved to where it is
  still total: `locally.rs` has no arm that can produce that reason, and a test
  walks every `RuntimeError` to say so.

  Built and unit tested against a stub on a real socket. **Not run against a real
  vLLM, llama.cpp server or LM Studio on any machine**, which is owed with the
  rest of the hardware verification.

- [x] **19. A turn, end to end, headless.** The item that makes the other
  thirteen crates one system, cut on the line law 1 draws: **this one is the
  turn that touches this machine**, and the question it puts to a model is 19a
  below. `crates/alo-turn`, a **new crate**: `machine.rs` (what every turn on
  this machine happens against, and what it can carry out), `turning.rs` (the
  turn and its five doors), `carrying.rs` (from *this may run* to *this is what
  happened*), `kept.rs` (where a turn writes what happened down), `refusing.rs`
  (the seven things that can come back instead), `words.rs` (one phrase),
  `testing.rs`. 31 unit tests, 9 integration tests — 5 against the real
  vocabulary in Finnish, 4 through a real filesystem and a real record file —
  and 1 `compile_fail` doctest. **1223 tests and 41 doctests across the
  workspace** (was 1183 and 40), clippy clean.

  **The item's own word *decision* turned out not to be this repository's**, and
  answering that is what shaped the crate. A model's answer becoming a verb and
  some arguments is the **agent's** work, and an agent is a client of
  `alo-agentd` rather than a part of it — item 21's protocol takes enumerated
  verbs with typed arguments, and this crate is what is behind that protocol. So
  a turn takes a name and a value per argument and makes the call **itself**,
  against the closed list; there is no door that takes a `Call` somebody else
  validated. Law 2 stops being a rule about what a model may send and becomes
  the absence of a second way in.

  **The guarantee the crate exists for is `CLAUDE.md`'s, made structural:
  nothing is handed back that has not been written down.** A `Turning` cannot be
  made without somewhere to keep its record, and every door writes its entry
  before it answers. What that cannot close is the window a change leaves open —
  a file has moved before there is anything to write about it — so a turn that
  could not write **stops**, every door afterwards refuses, and a daemon meeting
  it has a machine to halt rather than a call to retry. `docs/quirks.md` records
  why that closing is tested against a record that refuses everything rather
  than against a real disk.

  Three decisions the next items inherit. **A machine offers exactly the verbs
  it can carry out** — the registry is built by `Machine`, not handed to it, so
  a verb an agent can name is a verb something here can do and *the machine
  could not* is never about a capability that was advertised and does not exist.
  **A question put to a person is not a thing that happened**: what the record
  keeps is its answer, so a change nobody answered goes away with the turn and
  leaves no entry — an entry about somebody staying quiet would be a record of
  the person rather than of the agent, which is item 17's refusal met again.
  **This crate says one sentence.** Every refusal it hands back was worded by
  whoever made it, and the only string with nowhere else to come from is *this
  turn has stopped* — `alo-asking`'s two-string list, one crate further on.

  Built and unit tested, and walked end to end on a real filesystem with the
  record read back by `alo-keeping`. **Nothing here has been run on a certified
  machine**, and no disk has yet refused a write to it.

- [ ] **19a. The question a turn puts to a model.** Cut from item 19, on the
  line that divides `alo-asking`'s own doors: everything in 19 happens on this
  machine, and this is where a turn reaches off it. `alo-asking` has all three
  doors already, so this is the joining rather than the sending — but it is a
  real item and not wiring, because a turn that asks needs an
  `alo_egress::Indicator`, an `alo_models::SourcePolicy` and the list of places
  the person set up, and the departure comes back to be spent rather than being
  written down by the crate that caused it. Two questions the item has to
  answer. **Whose the indicator is**: one machine has one, and a `Machine` that
  held it would put an egress-showing surface in the type every turn borrows,
  which may be right and is a decision. **And what a turn does with an
  `alo_answering::Failed`** — an offer only a person can take, arriving in the
  middle of a turn that is holding a grant.

- [ ] **19b. What a turn does with an application verb.** The other half of
  `Machine::carrying_out_file_verbs`' name. `alo-applications` decides all four
  verbs and stops at `Reaching`, which is exactly what a compositor would be
  handed — so a turn could carry an application call that far and no further,
  and the question is whether a door that answers with *this may reach an
  application* is a capability or a stub wearing one. **Blocked on the acting
  half**, which is Wayland and D-Bus under *blocked — linux*: until something
  can move a window, a machine that offered these verbs would be a machine an
  agent can be refused by in a new way rather than one that can do more.

- [ ] **20. Where the record is written, and what prunes it.** Formerly 4b, and
  blocked all this time on the daemon not existing. The path a record is written
  to, the retention the organisation sets (ADR 0004), and the timer that
  shortens it. `alo-keeping` holds the shape; this gives it somewhere to live.

- [ ] **21. The daemon itself.** A long-lived process, a socket, and a typed
  request/response protocol — one file may name the transport, as `ollama.rs`
  names the runtime. **Law 2 is the whole design**: the protocol accepts
  enumerated verbs with typed arguments and there is no request that carries a
  command, a path to an executable, or anything a caller could shape into one.
  A malformed request is refused in the reader's own language, not dropped.

- [ ] **22. Running out is not a fault.** `alo-answering`'s `WentWrong` has no
  way to say *the money ran out*, so an exhausted balance arrives as
  `KeyNotAccepted` or `HavingTrouble(402)` — the first sends somebody to check a
  key that is perfectly correct, the second hands them a number. Neither says the
  one useful thing.

  A provider answering *payment required* or *quota exceeded* gets a variant of
  its own, said in the reader's own language: **this will not work until you pay,
  nothing else about your machine has changed, and here is what still does.**

  **Three rules it must not break.** It never becomes a reason to ask somewhere
  else on its own — ADR 0008's *never a silent fallback* in both directions, and
  spending somebody's money elsewhere because the first place was empty would be
  the worst possible version of it. It is said **once, where it happened**, and
  never again as a reminder: ADR 0009 already refused the greyed-out panel, and a
  buy-credit nag is that panel in a different coat. And it is **not an error** —
  running out is an ordinary state of an ordinary account, not a fault in the
  machine or in the person.

  ADR 0009's *since it was accepted* section is what this implements. Tests: the
  three provider replies that mean this in practice, each mapping to the new
  variant rather than to a key problem; and a test that nothing in the crate can
  turn it into an attempt somewhere else.

**Deliberately not here, and not this loop's:** the *acting* half of the
application verbs (Wayland and D-Bus — it is what actually moves a window), the
*reading* half of context (Wayland and AT-SPI), and everything that draws. Those
are listed under **Blocked — linux** and stay there until a compositor exists.

---

## Blocked — linux

Not this loop's, on this machine. Listed so the queue is a true picture of v0.01
rather than only of what is convenient.

- **Compositor** — Wayland via Smithay, one display, keyboard and pointer. It
  owes item 13 the half of the dock that is a picture rather than a
  measurement: the icons, and the hover and screen-reader name that
  `dock.labels.gave-way` promises is still there when the names give way. That
  sentence is a promise made to somebody who turned their text up because they
  could not read the screen, so a compositor that drew icons without it would
  make this repository say something untrue in twenty-four languages.
- **Sign-in and the local account**, the agent overlay, the launcher and window
  management, copy and paste, window switching — all draw on the compositor.
- **17a. The surfaces a machine with no agent does not have.** What item 17
  could not close, and the rest of ADR 0009: the hotkey doing nothing, the agent
  overlay not existing at all, and Grants, Models and providers being **absent**
  from Settings rather than present and disabled — *a greyed-out feature is an
  advertisement*. Plus setup's fourth choice as a screen, with the same weight
  as the other three and no persuasion attached. The model is built and is not
  a mode to check for: `alo_capability::Agent::has_an_agent` is the one question
  a shell asks, and `Agent::said` is the one line Settings still shows. All of
  it is compositor and settings-panel work and none of it compiles here.
- **Context on invocation, the reading half** — what is in front of the person,
  what they have selected and what they have open, answered by Wayland and
  AT-SPI at the moment the key is pressed, and the daemon that holds the turn
  the answers become. Item 12 above is the model those answers arrive into. The
  guarantee this half owes is the one `CLAUDE.md` names and no portable test can
  make: **with no invocation, `alo-agentd` makes no context calls at all** —
  which is a test against a running daemon and a compositor that counts what was
  asked of it, and there is neither here.
- **Application verbs, the acting half** — AT-SPI, D-Bus, the portal backend
  (ADR 0005): starting an application, bringing one to the front, and asking one
  to close, given a `Reaching`. The *file* half of this was listed here and was
  wrong: opening a folder needs no portal and no accessibility tree, so it was
  item 6a above and is built. **The portable half of the application verbs was
  wrong here too, and in a worse way** — it was not listed anywhere at all, so a
  v0.01 promise had no item until iteration 24 read this line properly. It is
  item 11 above. What is left here is genuinely Linux: nothing in this
  repository can start a program on a machine that has no compositor.
- **6b. Opening from a handle, and renaming without replacing.**
  `docs/quirks.md` records the two gaps the portable acting half cannot close: a
  path checked and then opened *by name* can have a link swapped in between the
  two, and `fs::rename` has no portable no-clobber form, so a destination is
  checked for and then renamed onto. Both have Linux answers with no portable
  spelling — `openat` with `O_NOFOLLOW` from a directory handle, and `renameat2`
  with `RENAME_NOREPLACE` — and both need a Linux host to compile as well as to
  test. Not a rewrite: the decisions, the refusals and the tests are settled,
  and this replaces the syscalls underneath them. The workspace forbids
  `unsafe`, so it needs either a pinned dependency wrapping the calls or an ADR,
  and choosing between those is the first thing the item does.
- **4b. Where the record file lives, and when it is shortened.** What item 4a
  could not close, and the whole of what is left of it: a path under `/var/lib`
  that the package decides, the setting the retention rule is read from and
  written to, and something with a timer that calls `Writing::prune`. All three
  are `alo-agentd`'s. The crate takes a path and a moment and holds no opinion
  about either, so this is wiring rather than design — but it is the daemon's
  wiring, and there is no daemon. It also owes one thing to the person: a
  managed machine's rule is the organisation's (ADR 0004), so where the setting
  is read from is a question about enrollment and not only about a file.

- **Egress enforcement** — item 5's policy, made true at the network boundary.
- **The image** — OCI-built, bootable, atomic. It owes item 8 one thing: a
  wallpaper named `alo` (`alo-appearance`'s `shipped::THE_WALLPAPER`), which is
  what a fresh machine shows behind its windows. An image without it boots to
  nothing behind the windows rather than to a colour nobody chose.
- **The workspace client running as an application on the shell.**

## Blocked — hardware

- **The model stack against a real Ollama.** Ticked in `ROADMAP.md` as built and
  tested, with this verification owed. A CPU-only run would close most of it and
  needs no GPU — but it needs Ollama installed, which is a person's decision to
  make rather than a loop's.
- **"The GPU works on first boot"**, which needs a machine that has one.
- **The v0.01 exit gate** — one person, one machine, one cold boot.

## Not ours

- **Agents point at the local model by default** — `alo-workplace`, and
  configuration rather than code (`AiConfig` has spoken to an OpenAI-compatible
  endpoint since 2025).
